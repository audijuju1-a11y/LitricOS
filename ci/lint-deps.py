#!/usr/bin/env python3
"""Fail when a workspace member takes a dependency from outside the workspace.

`src/Cargo.toml` opens with a HARD RULE in prose:

    the crates in `core-crates` below have ZERO third-party dependencies.
    std only.

for two stated reasons -- every dependency of a system component is code that
runs with that component's authority, and the workspace has to compile and test
offline on a host with no network and no Linux. `docs/master-plan/24-feature-parity.md`
repeats the claim: "No third-party crates in the system workspace."

Nothing checked it. `ci/verify.sh` ran shell syntax, line endings, Python syntax,
mkosi schema, kconfig, the scope tally, documentation citations, the memory
ledger, deletion sites, signing keys and the Cargo build/test/fmt/clippy set --
and no check looked at a dependency list at all. A rule that is only asserted is
a rule that holds until the first person in a hurry breaks it.

The rule enforced here is the tree's **current** state, not a new aspiration:
every dependency in every member is a `path = "../…"` dependency today, so the
ratchet is "a member may not gain a non-path dependency".

    python3 ci/lint-deps.py             check
    python3 ci/lint-deps.py --self-test prove it can fail

Exceptions are not silent. If `ci/dep-policy-allow.txt` exists, each line names
one `crate: dependency` pair that has been decided on; an exception is reported
as ALLOWED with the file as its evidence, and the gate still passes. A decision
recorded in a file can be reviewed; a decision baked into a checker cannot.
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOW_FILE = ROOT / "ci" / "dep-policy-allow.txt"

TABLE_RE = re.compile(r"^\s*\[(.+?)\]\s*$")
KEY_RE = re.compile(r"^\s*([A-Za-z0-9_.-]+)\s*=\s*(.*)$")


def members(root: Path) -> list[Path]:
    manifest = (root / "src" / "Cargo.toml").read_text(encoding="utf-8", errors="replace")
    m = re.search(r"members\s*=\s*\[(.*?)\]", manifest, re.S)
    if not m:
        raise SystemExit("lint-deps: src/Cargo.toml has no members list")
    return [root / "src" / name for name in re.findall(r'"([^"]+)"', m.group(1))]


def is_dependency_table(table: str) -> bool:
    """True for `dependencies`, `dev-dependencies`, `build-dependencies` and
    their target-specific forms such as `target."cfg(unix)".dependencies`."""
    return table.split(".")[-1] in ("dependencies", "dev-dependencies", "build-dependencies")


def load_allow() -> set[str]:
    if not ALLOW_FILE.is_file():
        return set()
    out = set()
    for line in ALLOW_FILE.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            out.add(line.replace(" ", ""))
    return out


def inspect(manifest: Path) -> tuple[list[tuple[str, str, str]], int]:
    """(violations, build_scripts) for one member manifest.

    A violation is (table, dependency, whole-line). `build_scripts` counts
    build-dependency entries -- reported, not failed, because the policy question
    of whether an in-workspace build script is acceptable has not been decided.
    """
    text = manifest.read_text(encoding="utf-8", errors="replace")
    violations: list[tuple[str, str, str]] = []
    build = 0
    table = ""
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        t = TABLE_RE.match(line)
        if t:
            table = t.group(1)
            continue
        if not is_dependency_table(table):
            continue
        k = KEY_RE.match(line)
        if not k:
            continue
        name, value = k.group(1), k.group(2)
        path_based = re.search(r"\bpath\s*=", value) is not None
        if table.split(".")[-1] == "build-dependencies":
            build += 1
        if not path_based:
            violations.append((table, name, raw.strip()))
    return violations, build


def check() -> int:
    allow = load_allow()
    failures: list[str] = []
    allowed: list[str] = []
    build_scripts: list[str] = []
    covered = 0

    for member in members(ROOT):
        manifest = member / "Cargo.toml"
        if not manifest.is_file():
            print(f"MISSING: src/{member.name}/Cargo.toml (a workspace member with no manifest)")
            failures.append(f"src/{member.name}: no Cargo.toml")
            continue
        covered += 1
        violations, build = inspect(manifest)
        for table, name, line in violations:
            key = f"{member.name}:{name}"
            if key in allow:
                allowed.append(f"src/{member.name}: {name} (in {table}) -- {ALLOW_FILE.name}")
            else:
                failures.append(f"src/{member.name}/Cargo.toml: {name} in [{table}] -- {line}")
        if build:
            build_scripts.append(f"src/{member.name}: {build} build-dependenc(y/ies)")

    for line in allowed:
        print(f"ALLOWED: {line}")
    for line in build_scripts:
        print(f"NOTE: {line} -- a build script runs on the build host; recorded, not failed")
    print(f"\n{covered} workspace member(s) checked; {len(failures)} third-party dependency(ies)")

    if failures:
        print("\nmembers depending on code outside this workspace:")
        for line in failures:
            print(f"  {line}")
        print("\nFix by removing the dependency, or by recording the decision in"
              " ci/dep-policy-allow.txt with a reason.")
        return 1
    return 0


def self_test() -> int:
    """Prove the checker fails on a planted dependency and passes on a path one."""
    rc = 0
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "src").mkdir()
        (root / "src" / "Cargo.toml").write_text(
            '[workspace]\nmembers = [\n    "litric-a",\n    "litric-b",\n    "litric-c",\n]\n',
            encoding="utf-8",
        )
        for name in ("litric-a", "litric-b", "litric-c"):
            (root / "src" / name).mkdir()
        (root / "src" / "litric-a" / "Cargo.toml").write_text(
            '[package]\nname = "litric-a"\n\n[dependencies]\nserde = "1"\n', encoding="utf-8")
        (root / "src" / "litric-b" / "Cargo.toml").write_text(
            '[package]\nname = "litric-b"\n\n[dependencies]\nlitric-a = { path = "../litric-a" }\n',
            encoding="utf-8")
        (root / "src" / "litric-c" / "Cargo.toml").write_text(
            '[package]\nname = "litric-c"\n\n[target.\'cfg(unix)\'.dependencies]\nlibc = "0.2"\n',
            encoding="utf-8")

        global ROOT, ALLOW_FILE
        real_root, real_allow = ROOT, ALLOW_FILE
        ROOT, ALLOW_FILE = root, root / "ci" / "dep-policy-allow.txt"
        try:
            violations, _ = inspect(root / "src" / "litric-a" / "Cargo.toml")
            if not violations or violations[0][1] != "serde":
                print("FAIL  self-test: a registry dependency was NOT flagged")
                rc = 1
            else:
                print("PASS  self-test: a registry dependency is flagged")

            violations, _ = inspect(root / "src" / "litric-b" / "Cargo.toml")
            if violations:
                print(f"FAIL  self-test: a path dependency was flagged ({violations})")
                rc = 1
            else:
                print("PASS  self-test: a path dependency is accepted")

            violations, _ = inspect(root / "src" / "litric-c" / "Cargo.toml")
            if not violations or violations[0][1] != "libc":
                print("FAIL  self-test: a target-specific dependency was NOT flagged")
                rc = 1
            else:
                print("PASS  self-test: a target-specific dependency is flagged")

            ALLOW_FILE.parent.mkdir(parents=True, exist_ok=True)
            ALLOW_FILE.write_text(
                "# crate: dependency\nlitric-a: serde\nlitric-c: libc\n", encoding="utf-8"
            )
            if check() != 0:
                print("FAIL  self-test: recorded exceptions did not clear the violations")
                rc = 1
            else:
                print("PASS  self-test: a recorded exception clears the violation and is reported")
        finally:
            ROOT, ALLOW_FILE = real_root, real_allow
    return rc


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return check()


if __name__ == "__main__":
    sys.exit(main(sys.argv))
