#!/usr/bin/env python3
"""Fail when an installed systemd unit starts a binary that does not exist.

The defect this was written for, found in the 2026-10-11 Stage A audit: three
units in the image started binaries no crate in `src/` produces.

  * `litric-capd.service` / `.socket` -> /usr/bin/litric-capd
  * `litric-defender.service`         -> /usr/bin/litric-defender
  * `litric-files@.service`           -> /usr/bin/litric-filesd

The last one was the dangerous one, because it was **enabled** by
`image/mkosi.postinst.chroot`. A socket unit needs no binary to start, so the
boot reported zero failed units and looked healthy; the failure appeared only
when the shell first connected, which is indistinguishable from a bug in the
shell. `image/mkosi.postinst.chroot` asks for the same property in prose ("no
damage ... an image that boots to a failed unit and a black screen would look
like progress and be worse than nothing"), and prose is what this replaces.

What it checks:

  * Every `ExecStart*=` line in every unit file under `image/mkosi.extra/`.
  * A program path under `/usr/bin/` or `/usr/sbin/` must be a binary the
    workspace can actually build: an explicit `[[bin]] name` in a member's
    manifest, or a `src/main.rs` (Cargo's implicit bin, named after the
    package). Listing the binaries is derived from the manifests, never
    hand-copied, so adding a daemon is what makes a unit legal.
  * A program path under `/usr/lib/litric/` must be a file the image ships, so
    `/usr/lib/litric/selftest` is checked by existence rather than by an
    allowlist entry someone has to remember to write.
  * Anything exported by the base distribution (`/bin/sh`, `/usr/bin/cat`, ...)
    is not ours to produce and is reported as SKIP with the reason, so a skip is
    visible rather than silent.

Usage:
    python3 ci/lint-unit-binaries.py             check
    python3 ci/lint-unit-binaries.py --self-test prove it can fail
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The tree mkosi applies to the image (`mkosi.extra/` is a magic name; a sibling
# directory such as `image/units-pending/` is not installed, which is exactly why
# the not-yet-buildable units live there).
IMAGE_TREE = ROOT / "image" / "mkosi.extra"
SRC = ROOT / "src"

# Where our own binaries are installed, and what a path in there must resolve to.
BIN_DIRS = ("/usr/bin/", "/usr/sbin/")
LIB_DIRS = ("/usr/lib/litric/",)

# ExecStart may be prefixed: `-` (failure tolerated), `+` (full privileges),
# `:` (no environment expansion), `@`/`!` (argv[0] overrides). Unit.
EXEC_RE = re.compile(r"^\s*Exec(Start|StartPre|StartPost|Reload)\s*=\s*(.*)$")
NAME_RE = re.compile(r'^\s*name\s*=\s*"([^"]+)"\s*$')
BIN_SECTION_RE = re.compile(r"^\s*\[\[bin\]\]\s*$")


def workspace_members(root: Path) -> list[Path]:
    """Member directories, from the workspace manifest's `members` list."""
    manifest = (root / "src" / "Cargo.toml").read_text(encoding="utf-8", errors="replace")
    m = re.search(r"members\s*=\s*\[(.*?)\]", manifest, re.S)
    if not m:
        raise SystemExit("lint-unit-binaries: src/Cargo.toml has no members list")
    return [root / "src" / name for name in re.findall(r'"([^"]+)"', m.group(1))]


def producible_binaries(root: Path) -> dict[str, str]:
    """Binary name -> a sentence saying where the workspace builds it."""
    out: dict[str, str] = {}
    for member in workspace_members(root):
        manifest = member / "Cargo.toml"
        if not manifest.is_file():
            continue
        text = manifest.read_text(encoding="utf-8", errors="replace")
        package = None
        declared: list[str] = []
        for line in text.splitlines():
            if line.strip() == "[package]":
                package = None
                continue
            if BIN_SECTION_RE.match(line):
                declared.append("__pending__")
                continue
            m = re.match(r'^\s*name\s*=\s*"([^"]+)"\s*$', line)
            if m:
                if declared and declared[-1] == "__pending__":
                    declared[-1] = m.group(1)
                elif package is None:
                    package = m.group(1)
        if (member / "src" / "main.rs").is_file():
            name = package or member.name
            out[name] = f"src/{member.name}/src/main.rs (implicit bin)"
        for name in declared:
            if name and name != "__pending__":
                out[name] = f"src/{member.name}/Cargo.toml [[bin]]"
    return out


def units(tree: Path) -> list[Path]:
    if not tree.is_dir():
        return []
    return sorted(p for p in tree.rglob("*") if p.suffix in (".service", ".socket", ".mount", ".timer", ".target", ".path"))


def program_of(value: str) -> str | None:
    """The program a unit directive runs, with systemd's prefixes stripped."""
    value = value.strip()
    while value[:1] in ("-", "+", ":", "@", "!"):
        value = value[1:].lstrip()
    if not value:
        return None
    return value.split()[0]


def judge(program: str, produced: dict[str, str], tree: Path) -> tuple[str, str]:
    """Return (status, detail) where status is OK, SKIP or MISSING."""
    if not program.startswith("/"):
        return "SKIP", f"not an absolute path ({program})"
    for d in BIN_DIRS:
        if program.startswith(d):
            name = program[len(d):]
            if name in produced:
                return "OK", produced[name]
            # Only our own names are failures. This checker does not model the
            # base distribution's /usr/bin (it cannot: the image is built on a
            # Linux host and this gate also runs on Windows), so a name that
            # does not start with `litric-` is reported as a skip rather than
            # guessed at. The first version of this file called every such path
            # MISSING, which would have failed the gate on a unit running
            # /usr/bin/systemctl -- a false alarm, and a checker that cries wolf
            # gets an allowlist.
            if name.startswith("litric-"):
                return "MISSING", f"no crate builds a binary named {name!r}"
            return "SKIP", f"base distribution binary ({program})"
    for d in LIB_DIRS:
        if program.startswith(d):
            rel = program[len("/"):]
            if (tree / rel).exists():
                return "OK", f"shipped as {rel}"
            return "MISSING", f"{rel} is not in the image tree"
    if program.startswith(("/bin/", "/usr/sbin/")):
        return "SKIP", f"base distribution binary ({program})"
    return "SKIP", f"not one of our install locations ({program})"


def scan(root: Path) -> tuple[list[str], list[str], list[str]]:
    """(violations, skips, oks) for the real tree."""
    tree = root / "image" / "mkosi.extra"
    produced = producible_binaries(root)
    violations: list[str] = []
    skips: list[str] = []
    oks: list[str] = []
    for unit in units(tree):
        for lineno, line in enumerate(unit.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            m = EXEC_RE.match(line)
            if not m:
                continue
            program = program_of(m.group(2))
            if program is None:
                continue
            status, detail = judge(program, produced, tree)
            where = f"{unit.relative_to(root)}:{lineno}"
            if status == "MISSING":
                violations.append(f"{where}: {program} -- {detail}")
            elif status == "SKIP":
                skips.append(f"{where}: {program} -- {detail}")
            else:
                oks.append(f"{where}: {program} -- {detail}")
    return violations, skips, oks


def check() -> int:
    violations, skips, oks = scan(ROOT)
    for line in skips:
        print(f"SKIP: {line}")
    print(f"\nchecked {len(oks) + len(skips) + len(violations)} program reference(s); "
          f"{len(oks)} resolvable, {len(skips)} skipped, {len(violations)} broken")
    if violations:
        print("\nunits that start a binary which does not exist:")
        for line in violations:
            print(f"  {line}")
        print("\nFix by building the binary, or by moving the unit out of"
              " image/mkosi.extra/ (see image/units-pending/README.md).")
        return 1
    return 0


def self_test() -> int:
    """Prove the checker fails on a dead unit and passes on a live one."""
    rc = 0
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        tree = root / "image" / "mkosi.extra" / "usr" / "lib" / "systemd" / "system"
        tree.mkdir(parents=True)
        crate = root / "src" / "litric-real"
        (crate / "src").mkdir(parents=True)
        (crate / "src" / "main.rs").write_text("fn main() {}\n", encoding="utf-8")
        (crate / "Cargo.toml").write_text(
            '[package]\nname = "litric-real"\nversion = "0.0.0"\n', encoding="utf-8")
        (root / "src" / "Cargo.toml").write_text(
            '[workspace]\nmembers = [\n    "litric-real",\n]\n', encoding="utf-8")
        helper = root / "image" / "mkosi.extra" / "usr" / "lib" / "litric"
        helper.mkdir(parents=True)
        (helper / "selftest").write_text("#!/bin/sh\n", encoding="utf-8")

        good = tree / "litric-real.service"
        good.write_text("[Service]\nExecStart=/usr/bin/litric-real --serve\n", encoding="utf-8")
        violations, _, oks = scan(root)
        if violations or not oks:
            print(f"FAIL  self-test: the live unit was not accepted ({violations})")
            rc = 1
        else:
            print("PASS  self-test: a unit backed by a real binary is accepted")

        bad = tree / "litric-dead.service"
        bad.write_text("[Service]\nExecStart=/usr/bin/litric-dead --serve\n", encoding="utf-8")
        violations, _, _ = scan(root)
        if not any("litric-dead" in v for v in violations):
            print("FAIL  self-test: a dead unit was NOT flagged")
            rc = 1
        else:
            print("PASS  self-test: a dead unit is flagged")

        bad.write_text("[Service]\nExecStart=/usr/lib/litric/missing-helper\n", encoding="utf-8")
        violations, _, _ = scan(root)
        if not any("missing-helper" in v for v in violations):
            print("FAIL  self-test: a missing image-shipped helper was NOT flagged")
            rc = 1
        else:
            print("PASS  self-test: a missing image-shipped helper is flagged")

        base = tree / "litric-base.service"
        base.write_text("[Service]\nExecStart=/usr/bin/systemctl --version\n", encoding="utf-8")
        violations, skips, _ = scan(root)
        if any("systemctl" in v for v in violations):
            print("FAIL  self-test: a base-distribution binary was reported as missing")
            rc = 1
        elif not any("systemctl" in s for s in skips):
            print("FAIL  self-test: a base-distribution binary was not reported as a skip")
            rc = 1
        else:
            print("PASS  self-test: a base-distribution binary is skipped, not failed")
    return rc


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return check()


if __name__ == "__main__":
    sys.exit(main(sys.argv))
