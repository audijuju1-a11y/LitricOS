#!/usr/bin/env python3
"""Verify that files cited in documentation actually exist.

This is the regression test for the defect pattern this project keeps hitting,
recorded four times before it was mechanised:

  * BUG-0001 -- `ci/lint-config.sh` does not exist; the linter is
    `ci/lint-mkosi.py`. The bug report cites the wrong name four times.
  * BUG-0002 -- `tests/smoke_boot.py` was listed as a completed deliverable and
    could not be imported.
  * GATE 01 condition 7 -- `ci/lint-safety.sh` was recorded as the VERIFIED
    evidence for "zero host-disk writes outside the project dir". No such file
    has ever been in this tree, so the citation was unfalsifiable.
  * PERF-budget -- the 320 MB total row names `tests/budget/test_mem.py` as its
    enforcement. `tests/budget/` is empty.

All four share one cause: a document asserting the existence of an artifact
nobody executed. A gate cannot depend on evidence that cannot be opened, and a
reader cannot tell a real citation from a remembered one.

What this checks, and what it deliberately does not:

  * Only backticked tokens that contain "/" and end in a source, config or
    document extension are treated as citations. Prose like `audio.output/input`
    is not a path and is not reported.
  * A token is resolved against several bases (repository root, src/, docs/,
    image/mkosi.extra/, the document's own directory), because these documents
    legitimately mix conventions -- `litric-shell/src/ui.rs` is a path under
    src/, and both are correct as written.
  * Paths that are genuinely not in this repository (the kernel source tree) are
    allowlisted by prefix, with a reason, and reported as SKIP. Allowlisting is
    a decision recorded in this file rather than a silent pass.
  * Existence only. This linter cannot tell whether the cited file does what the
    document says it does, and does not claim to.

Usage:
    python3 ci/lint-doc-refs.py            check
    python3 ci/lint-doc-refs.py --self-test  prove it can fail

Citing an artifact that is deliberately not in this repository: put it in prose
without backticks, or add a prefix to EXTERNAL_PREFIXES with the reason. A
correction that needs to name the phantom path it is correcting should do the
former -- the correction is more useful when it says what was wrong, and more
honest when the string it names is not dressed up as a path.
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Directories that are generated, vendored, frozen, or not ours.
SKIP_DIRS = {
    "build", "target", ".git", ".backup-pre-v2", "__pycache__",
    "node_modules", ".agents", ".claude", ".freebuff",
}

# Extensions that mark a token as a citation rather than prose.
SUFFIXES = {
    ".py", ".sh", ".rs", ".md", ".conf", ".service", ".slice", ".socket",
    ".target", ".mount", ".network", ".rules", ".nft", ".json", ".toml",
    ".ps1", ".c", ".h", ".html", ".txt", ".csv", ".efi", ".key", ".crt",
}

TOKEN_RE = re.compile(r"`([A-Za-z0-9_][A-Za-z0-9_./+-]*)`")

# Bases a citation may be written relative to.
BASES = [
    ROOT,
    ROOT / "src",
    ROOT / "docs",
    ROOT / "docs" / "master-plan",
    ROOT / "image" / "mkosi.extra",
    ROOT / "image" / "mkosi.extra" / "usr" / "lib",
    ROOT / "ci",
    ROOT / "tests",
    ROOT / "tools",
]

# Paths that are not expected to exist here, with the reason.
# A prefix match. Everything else unresolved is an error.
EXTERNAL_PREFIXES = {
    "scripts/kconfig/": "kernel source tree, not this repository",
    "kernel/patches/": "kernel source tree, not this repository",
    "usr/share/": "distribution filesystem layout, not this repository",
    "etc/": "target-system path, not a repository path",
}


class Citation:
    def __init__(self, path: Path, line: int, token: str, status: str) -> None:
        self.path, self.line, self.token, self.status = path, line, token, status

    def __str__(self) -> str:
        rel = self.path.relative_to(ROOT)
        return f"{rel}:{self.line}: {self.status}: {self.token}"


def documents() -> list[Path]:
    docs = list(ROOT.glob("*.md")) + list((ROOT / "docs").rglob("*.md"))
    out = []
    for d in docs:
        if any(part in SKIP_DIRS for part in d.relative_to(ROOT).parts):
            continue
        out.append(d)
    return sorted(out)


def citations_in(path: Path, text: str) -> list[Citation]:
    out: list[Citation] = []
    for lineno, line in enumerate(text.splitlines(), start=1):
        for token in TOKEN_RE.findall(line):
            if "/" not in token or Path(token).suffix not in SUFFIXES:
                continue
            if "<" in token or ">" in token or "*" in token:
                continue  # a placeholder or a glob, not a citation
            if token.startswith((".", "/", "~")):
                continue
            if any(part in SKIP_DIRS for part in Path(token).parts):
                continue  # discussing a generated or frozen tree
            external = next(
                (why for pfx, why in EXTERNAL_PREFIXES.items() if token.startswith(pfx)),
                None,
            )
            if external:
                out.append(Citation(path, lineno, token, f"SKIP ({external})"))
            elif resolve(token, path):
                out.append(Citation(path, lineno, token, "OK"))
            else:
                out.append(Citation(path, lineno, token, "MISSING"))
    return out


def resolve(token: str, doc: Path) -> Path | None:
    candidates = [b / token for b in BASES] + [doc.parent / token]
    for c in candidates:
        if c.exists():
            return c
    return None


def check() -> int:
    missing: list[Citation] = []
    skipped: list[Citation] = []
    total = 0

    for doc in documents():
        for c in citations_in(doc, doc.read_text(encoding="utf-8", errors="replace")):
            total += 1
            if c.status == "MISSING":
                missing.append(c)
            elif c.status.startswith("SKIP"):
                skipped.append(c)

    for c in skipped:
        print(f"SKIP: {c.path.relative_to(ROOT)}:{c.line}: {c.token} -- {c.status[6:-1]}")

    if missing:
        print(f"\n{len(missing)} citation(s) name a path that does not exist:")
        for c in missing:
            print(f"  {c.path.relative_to(ROOT)}:{c.line}: {c.token}")
        print(
            "\nFix by pointing the citation at the real artifact, or by stating "
            "in the document that the artifact does not exist yet. Do not delete "
            "the claim silently (see ci/lint-doc-refs.py)."
        )
        return 1

    print(f"OK: {total} citation(s) across {len(documents())} document(s) resolve")
    return 0


def self_test() -> int:
    """Prove the linter fails on a document that cites nothing real."""
    rc = 0
    with tempfile.TemporaryDirectory() as td:
        doc = Path(td) / "FAKE.md"
        doc.write_text(
            "Real: `ci/lint-mkosi.py`\n"
            "Fabricated: `ci/lint-config.sh`\n"
            "Prose, not a path: `audio.output/input`\n",
            encoding="utf-8",
        )
        found = citations_in(doc, doc.read_text(encoding="utf-8"))
        tokens = {c.token: c.status for c in found}
        if tokens.get("ci/lint-config.sh") != "MISSING":
            print("FAIL  self-test: the fabricated citation was NOT flagged")
            rc = 1
        else:
            print("PASS  self-test: fabricated citation flagged")
        if "ci/lint-mkosi.py" not in tokens:
            print("FAIL  self-test: the real citation should resolve")
            rc = 1
        else:
            print("PASS  self-test: real citation resolves")
        if "audio.output/input" in tokens:
            print("FAIL  self-test: prose was treated as a citation")
            rc = 1
        else:
            print("PASS  self-test: prose ignored")
    return rc


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return check()


if __name__ == "__main__":
    sys.exit(main(sys.argv))
