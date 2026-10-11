#!/usr/bin/env python3
"""Fail when a workspace member that must stay dependency-free gains a dependency.

`src/Cargo.toml` opens with a HARD RULE in prose: the crates in its `core-crates`
group have ZERO third-party dependencies, std only. Two reasons are given --
every dependency of a system component is code that runs with that component's
authority (the capability broker does not get to pull 200 transitive crates from
the internet), and those crates must compile and test on a Windows host with no
network and no Linux. `docs/master-plan/24-feature-parity.md` repeated the claim:
"No third-party crates in the system workspace."

Nothing checked it. The gate ran shell syntax, line endings, Python syntax, mkosi
schema, kconfig, the scope tally, documentation citations, the memory ledger,
deletion sites, signing keys and the Cargo build/test/fmt/clippy set -- and no
check looked at a dependency list at all. A rule that is only asserted holds
until the first person in a hurry breaks it.

What is enforced, and why it is derived rather than typed out here:

  * **Coverage comes from the manifest.** `src/Cargo.toml` groups its members
    under comment headers (`--- core: ...`, `--- runtime: ...`, `--- defender:
    ...`, `--- applications ...`, `--- native ecosystem: ...`, `--- benchmark:
    ...`). Each group name has an explicit decision in CLASS_POLICY below. A
    member whose group has no decision is a **failure**, not a skip: a new class
    of crate must not silently arrive outside the policy.
  * A member whose own manifest declares itself dependency-free is covered too,
    whatever group it sits in.
  * For a covered crate every entry in `dependencies`, `dev-dependencies`,
    `build-dependencies` and their `target.'...'` forms must be a **path
    dependency on another member of this workspace**. `serde = "1"`, a git
    dependency, a path pointing outside the workspace, a path to a directory that
    is not a member: all violations.
  * A `build-dependencies` entry is refused **even when the path is inside the
    workspace**, because a build script executes on the build host at build time,
    which is the same supply-chain objection in a different place.

Recorded exceptions live in `ci/dep-policy-allow.txt`: one `member: dependency`
line each, with the reason on it. An exception is printed as ALLOWED on every run
rather than silently accepted, so the decision stays reviewable.

Usage:
    python3 ci/lint-deps.py             check
    python3 ci/lint-deps.py --self-test prove it can fail
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOW_FILE = ROOT / "ci" / "dep-policy-allow.txt"

# Member groups in src/Cargo.toml (`# --- <group>: <description>`), and whether
# the members under them are covered by the no-dependency policy. This table is
# the decision; the manifest supplies the membership. Unknown group -> failure.
CLASS_POLICY: dict[str, tuple[bool, str]] = {
    "core": (True, "the manifest's HARD RULE names this group"),
    "runtime": (True, "runs inside the OS; the manifest says no third-party deps"),
    "defender": (True, "the manifest says zero dependencies of any kind"),
    "benchmark": (True, "the manifest says zero deps"),
    "applications": (False, "application crates; the manifest does not require zero deps"),
    "native ecosystem": (False, "platform crates (litric-shell); allowed to take deps later"),
}

DEP_TABLES = ("dependencies", "dev-dependencies", "build-dependencies")

TABLE_RE = re.compile(r"^\s*\[(.+?)\]\s*$")
KEY_RE = re.compile(r"^\s*([A-Za-z0-9_.-]+)\s*=\s*(.*)$")
PATH_RE = re.compile(r"\bpath\s*=\s*\"([^\"]+)\"")
SELF_DECLARED_RE = re.compile(
    r"deliberately empty|zero third-party|zero dependencies|zero deps"
    r"|no third-party dependenc|no dependencies",
    re.I,
)


class Member:
    def __init__(self, name: str, group: str, group_text: str, covered: bool, why: str) -> None:
        self.name, self.group, self.group_text = name, group, group_text
        self.covered, self.why = covered, why
        self.deps: list[tuple[str, str, str]] = []   # (table, dependency, raw line)
        self.paths: list[str] = []                   # accepted path dependencies
        self.violations: list[str] = []
        self.self_declared = False


def members_of(manifest_text: str) -> tuple[list[tuple[str, str, str]], str | None]:
    """[(name, group, group_text)] from the members list, grouped by comment."""
    block = re.search(r"members\s*=\s*\[(.*?)\]", manifest_text, re.S)
    if not block:
        return [], "src/Cargo.toml has no members list"
    out: list[tuple[str, str, str]] = []
    group, group_text, pending = "", "", []
    for raw in block.group(1).splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            # A group header is a comment block; the name is on its first line
            # (`--- core: ...`) and the rest of the block is its description. Two
            # mistakes are recorded here rather than repeated: reading only the
            # last comment line classified `defender` under the group "arrives in
            # Phase 2 and is what makes this a platform crate", and clearing the
            # group at the first member left every later member of a group
            # unclassified.
            if not pending:
                group, group_text = "", ""
            pending.append(line.lstrip("#").strip().strip("-").strip())
            group_text = " ".join(pending)
            head = pending[0]
            group, _ = (p.strip() for p in head.split(":", 1)) if ":" in head else (head, "")
            continue
        m = re.match(r'^"([^"]+)"\s*,?$', line)
        if m:
            out.append((m.group(1), group, group_text))
            pending = []
    return out, None


def is_dependency_table(table: str) -> bool:
    """`dependencies`, `dev-dependencies`, `build-dependencies`, and
    target-specific forms such as `target."cfg(unix)".dependencies`."""
    return table.split(".")[-1] in DEP_TABLES


def parse(manifest: Path, member: Member, member_dirs: set[str]) -> None:
    text = manifest.read_text(encoding="utf-8", errors="replace")
    member.self_declared = any(
        SELF_DECLARED_RE.search(line) for line in text.splitlines() if line.strip().startswith("#")
    )
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
        kind = table.split(".")[-1]
        member.deps.append((kind, name, raw.strip()))
        path = PATH_RE.search(value)
        if kind == "build-dependencies":
            member.violations.append(
                f"{name} in [{table}]: path={path.group(1)!r}" if path else f"{name} in [{table}]"
            )
            member.violations[-1] += (
                " -- a build script runs on the build host at build time,"
                " so it is refused even inside the workspace"
            )
            continue
        if not path:
            member.violations.append(f"{name} in [{table}] -- not a path dependency ({value.strip()})")
            continue
        target = (manifest.parent / path.group(1)).resolve()
        if str(target) not in member_dirs:
            member.violations.append(
                f"{name} in [{table}] -- path {path.group(1)!r} is not a member of this workspace"
            )
            continue
        member.paths.append(name)


def load_allow() -> set[str]:
    if not ALLOW_FILE.is_file():
        return set()
    out = set()
    for line in ALLOW_FILE.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            out.add(line.replace(" ", ""))
    return out


def classify(manifest: Path, name: str, group: str, group_text: str) -> Member:
    key = group.strip().lower()
    decision = CLASS_POLICY.get(key)
    if not group:
        covered, why = False, "UNCLASSIFIED: no group comment above it in src/Cargo.toml"
    elif decision is None:
        covered, why = False, f"UNKNOWN GROUP {group!r}: no decision in this linter's CLASS_POLICY"
    else:
        covered, why = decision[0], decision[1]
    member = Member(name, group, group_text, covered, why)
    if member.self_declared:
        member.covered = True
        member.why = "its own manifest declares it dependency-free"
    return member


def collect(root: Path) -> tuple[list[Member], list[str]]:
    """Covered and uncovered members, plus fail-closed classification problems."""
    manifest = (root / "src" / "Cargo.toml").read_text(encoding="utf-8", errors="replace")
    listed, err = members_of(manifest)
    problems = [err] if err else []
    member_dirs = {str((root / "src" / n).resolve()) for n, _, _ in listed}
    out: list[Member] = []
    for name, group, group_text in listed:
        crate = root / "src" / name / "Cargo.toml"
        member = classify(crate, name, group, group_text)
        if not crate.is_file():
            problems.append(f"src/{name}: a workspace member with no Cargo.toml")
            out.append(member)
            continue
        parse(crate, member, member_dirs)
        if not member.group:
            problems.append(f"src/{name}: no group comment in src/Cargo.toml, so no policy decision")
        elif member.group.strip().lower() not in CLASS_POLICY:
            problems.append(
                f"src/{name}: group {member.group!r} has no decision in ci/lint-deps.py CLASS_POLICY"
            )
        out.append(member)
    return out, problems


def check() -> int:
    allow = load_allow()
    members, problems = collect(ROOT)
    covered = [m for m in members if m.covered]
    skipped = [m for m in members if not m.covered]

    print("dependency policy -- coverage derived from src/Cargo.toml")
    for m in members:
        state = "covered " if m.covered else "outside "
        detail = f"{len(m.deps)} dependency entr(y/ies)"
        print(f"  {state} {m.name:<16} group {m.group or '<none>':<18} {detail:<26} {m.why}")

    allowed: list[str] = []
    for m in covered:
        remaining: list[str] = []
        for v in m.violations:
            dep = v.split(" in [", 1)[0]
            if f"{m.name}:{dep}" in allow:
                allowed.append(f"{m.name}: {v} -- {ALLOW_FILE.name}")
            else:
                remaining.append(v)
        m.violations = remaining

    for line in allowed:
        print(f"ALLOWED: {line}")
    print(
        f"\n{len(members)} member(s): {len(covered)} covered by the no-dependency policy,"
        f" {len(skipped)} outside it ({', '.join(sorted({m.group or 'unclassified' for m in skipped})) or 'none'})"
    )

    for line in problems:
        print(f"\nERROR  {line}")
    bad = [(m.name, v) for m in covered for v in m.violations]
    if bad:
        print("\ncovered crates depending on code from outside this workspace:")
        for name, v in bad:
            print(f"  src/{name}/Cargo.toml: {v}")
    print(f"\n{len(bad)} violation(s) in covered member(s)")

    if bad or problems:
        print(
            "\nFix by removing the dependency, or by recording the decision in"
            " ci/dep-policy-allow.txt with the reason on the line."
        )
        return 1
    return 0


def fixture(root: Path) -> Path:
    """A miniature workspace that mirrors the manifest's structure."""
    src = root / "src"
    (src / "litric-core").mkdir(parents=True, exist_ok=True)
    for name, group in (
        ("litric-core", "core: zero-dependency, platform-independent"),
        ("litric-pure", "runtime: runs inside the OS, still no third-party deps"),
        ("litric-caps", "core: zero-dependency"),
        ("litric-shell", "native ecosystem: the desktop itself"),
        ("litric-mystery", "experimental: something new"),
    ):
        d = src / name
        d.mkdir(parents=True, exist_ok=True)
        (d / "Cargo.toml").write_text(f'[package]\nname = "{name}"\n', encoding="utf-8")
    (src / "Cargo.toml").write_text(
        "[workspace]\nmembers = [\n"
        "    # --- core: zero-dependency, platform-independent --------------------\n"
        '    "litric-core",\n'
        '    "litric-caps",\n'
        "    # --- runtime: runs inside the OS, still no third-party deps ---------\n"
        '    "litric-pure",\n'
        "    # --- native ecosystem: the desktop itself ---------------------------\n"
        '    "litric-shell",\n'
        "    # --- experimental: something new -----------------------------------\n"
        '    "litric-mystery",\n'
        "]\n",
        encoding="utf-8",
    )
    (src / "outside").mkdir(parents=True, exist_ok=True)
    (src / "outside" / "Cargo.toml").write_text("[package]\nname = \"outside\"\n", encoding="utf-8")
    return src


def verdict(src: Path, name: str) -> tuple[list[str], list[str]]:
    """(violations, problems) for one fixture member.

    Violations are reported only for a covered member, which is what `check()`
    does: a platform crate with a registry dependency is not a violation, it is
    outside the policy.
    """
    members, problems = collect(src.parent)
    for m in members:
        if m.name == name:
            return (m.violations if m.covered else []), problems
    raise AssertionError(f"fixture member {name} was not collected")


def self_test() -> int:
    """Prove the checker fails on every planted violation and passes on the
    legal forms. Each case names what it establishes."""
    rc = 0
    with tempfile.TemporaryDirectory() as td:
        src = fixture(Path(td))

        def write(name: str, body: str) -> None:
            (src / name / "Cargo.toml").write_text(
                f'[package]\nname = "{name}"\n\n{body}', encoding="utf-8"
            )

        def expect(what: str, name: str, want: bool, needle: str = "") -> None:
            nonlocal rc
            violations, _ = verdict(src, name)
            got = bool(violations)
            if got != want or (want and needle and not any(needle in v for v in violations)):
                print(f"FAIL  self-test: {what} ({violations})")
                rc = 1
            else:
                print(f"PASS  self-test: {what}")

        write("litric-core", '[dependencies]\nserde = "1"\n')
        expect("a registry dependency in a core crate is flagged", "litric-core", True, "serde")

        write("litric-core", '[dependencies]\nlitric-caps = { path = "../litric-caps" }\n')
        expect("a path dependency on a member is accepted", "litric-core", False)

        write("litric-core", '[dependencies]\nlitric-outside = { path = "../outside" }\n')
        expect("a path dependency outside the workspace is flagged", "litric-core", True)

        write("litric-core", '[target.\'cfg(unix)\'.dependencies]\nlibc = "0.2"\n')
        expect("a target-specific dependency is flagged", "litric-core", True, "libc")

        write("litric-core", '[build-dependencies]\nlitric-caps = { path = "../litric-caps" }\n')
        expect("an in-workspace build-dependency is refused", "litric-core", True, "build script")

        write("litric-shell", '[dependencies]\nserde = "1"\nlibc = "0.2"\n')
        expect("a platform crate is not covered by the policy", "litric-shell", False)

        write("litric-mystery", '[dependencies]\nserde = "1"\n')
        violations, problems = verdict(src, "litric-mystery")
        if not any("CLASS_POLICY" in p for p in problems):
            print(f"FAIL  self-test: an unknown manifest group was not a fail-closed problem ({problems})")
            rc = 1
        else:
            print("PASS  self-test: an unknown manifest group fails closed")

        # The allowlist: a recorded exception clears a violation, and the same
        # tree without the record fails. Both halves are asserted, so the file
        # -- not the checker -- is doing the work.
        global ROOT, ALLOW_FILE
        real_root, real_allow = ROOT, ALLOW_FILE
        ROOT, ALLOW_FILE = Path(td), Path(td) / "ci" / "dep-policy-allow.txt"
        try:
            (src / "Cargo.toml").write_text(
                "[workspace]\nmembers = [\n"
                "    # --- core: zero-dependency -------------------------------------\n"
                '    "litric-core",\n'
                "    # --- applications --------------------------------------------\n"
                '    "litric-shell",\n'
                "]\n",
                encoding="utf-8",
            )
            write("litric-core", '[dependencies]\nserde = "1"\n')
            ALLOW_FILE.parent.mkdir(parents=True, exist_ok=True)
            ALLOW_FILE.write_text("# member: dependency\nlitric-core: serde\n", encoding="utf-8")
            if check() != 0:
                print("FAIL  self-test: a recorded exception did not clear the violation")
                rc = 1
            else:
                print("PASS  self-test: a recorded exception clears the violation and is reported")
            ALLOW_FILE.unlink()
            if check() == 0:
                print("FAIL  self-test: the tree passed with no exception recorded")
                rc = 1
            else:
                print("PASS  self-test: without the record the same tree fails")
        finally:
            ROOT, ALLOW_FILE = real_root, real_allow
    return rc


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return check()


if __name__ == "__main__":
    sys.exit(main(sys.argv))
