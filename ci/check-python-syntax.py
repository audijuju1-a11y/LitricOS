#!/usr/bin/env python3
"""Parse every Python file this repository ships, without writing bytecode.

Two reasons this exists instead of `python -m py_compile`:

  1. `py_compile` writes `__pycache__/` into the tree it is checking. A
     verification step should not add files to the repository it verifies --
     the `tests/__pycache__/smoke_boot.cpython-314.pyc` in this tree is exactly
     that residue, and it predates this file.
  2. The check it replaces covered `ci/` and `tests/` only, so `tools/` -- 18
     scripts, including the QEMU harnesses that drive real runs -- was never
     parsed. BUG-0002 (a Python file truncated mid-statement in `tests/`) is the
     defect class this is for, and it could have happened in `tools/` just as
     easily.

Parsing only. This is a syntax check: it establishes that a file can be read as
Python, nothing about whether it works.

Usage:
    python3 ci/check-python-syntax.py
    python3 ci/check-python-syntax.py --self-test
"""

from __future__ import annotations

import ast
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TREES = ["ci", "tests", "tools"]
SKIP_DIRS = {"__pycache__", ".backup-pre-v2", "build", "target"}


def files() -> list[Path]:
    out: list[Path] = []
    for tree in TREES:
        base = ROOT / tree
        if not base.is_dir():
            continue
        for p in sorted(base.rglob("*.py")):
            if any(part in SKIP_DIRS for part in p.relative_to(ROOT).parts):
                continue
            out.append(p)
    return out


def label(path: Path) -> str:
    """Repository-relative where possible.

    `--self-test` parses files in a temporary directory, and `relative_to`
    raises for a path outside the repository -- which turned a reported
    self-test failure into an unhandled ValueError until the self-test caught
    it. Keep the label total: every path gets one.
    """
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def parse(path: Path) -> str | None:
    """Return an error message, or None if the file parses."""
    try:
        ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    except SyntaxError as exc:
        return f"{label(path)}:{exc.lineno}: {exc.msg}"
    except OSError as exc:
        return f"{label(path)}: cannot read: {exc}"
    return None


def self_test() -> int:
    """A truncated file must be reported, or this check proves nothing."""
    with tempfile.TemporaryDirectory() as td:
        good = Path(td) / "good.py"
        good.write_text("def f():\n    return 1\n", encoding="utf-8")
        bad = Path(td) / "bad.py"
        # The exact shape BUG-0002 shipped: a file that ends mid-statement.
        bad.write_text("def f():\n    try:\n        pass\n    finally:\n", encoding="utf-8")

        if parse(good) is not None:
            print("FAIL  self-test: a valid file was reported as broken")
            return 1
        print("PASS  self-test: valid file parses")
        reported = parse(bad)
        if reported is None:
            print("FAIL  self-test: a truncated file was NOT reported")
            return 1
        print(f"PASS  self-test: truncated file reported ({reported})")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()

    targets = files()
    if not targets:
        print("ERROR: no Python files found -- check TREES in this script")
        return 1

    errors = [e for e in (parse(p) for p in targets) if e]
    for e in errors:
        print(f"  {e}")
    if errors:
        print(f"\n{len(errors)} file(s) do not parse")
        return 1
    print(f"OK: {len(targets)} python file(s) parse (no bytecode written)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
