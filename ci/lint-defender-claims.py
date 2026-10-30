#!/usr/bin/env python3
"""Fail when an on-access Defender unit exists without an on-access daemon.

B3 removed the parked `litric-defender.service` and `litric-defender.slice`.
The executable in `src/litric-defender` is an **on-demand** scanner: it accepts
`--scan` and `--stdio`, not the unit's `--rules` / `--cache` / `Type=notify`
invocation, and it has no fanotify transport. A unit that names an on-access
daemon is therefore a promise the image cannot keep.

This checker is intentionally small and fail-closed: no file called
`litric-defender.service` or `litric-defender.slice` may exist under `image/`,
installed or parked. Reintroducing either requires the daemon, the transport and
the boot evidence first -- and then this check must be updated deliberately.

Usage:
    python3 ci/lint-defender-claims.py             check
    python3 ci/lint-defender-claims.py --self-test prove it can fail
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IMAGE = ROOT / "image"
FORBIDDEN = ("litric-defender.service", "litric-defender.slice")


def check_root(root: Path) -> list[str]:
    """Return one finding for each forbidden on-access unit under `root`."""
    errors: list[str] = []
    for path in sorted(root.rglob("*")):
        if path.name in FORBIDDEN:
            errors.append(
                f"{path.relative_to(root).as_posix()}: no on-access daemon exists; "
                "keep only the on-demand scanner until fanotify is implemented"
            )
    return errors


def self_test() -> int:
    """Plant the removed unit and slice; the checker must reject both."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "ok.service").write_text("[Service]\n", encoding="utf-8")
        if check_root(root):
            print("FAIL: the checker rejected a tree with no on-access unit")
            return 1

        (root / "litric-defender.service").write_text("[Service]\n", encoding="utf-8")
        if not any("litric-defender.service" in line for line in check_root(root)):
            print("FAIL: the checker accepted a planted on-access service")
            return 1

        (root / "litric-defender.slice").write_text("[Slice]\n", encoding="utf-8")
        if not any("litric-defender.slice" in line for line in check_root(root)):
            print("FAIL: the checker accepted a planted on-access slice")
            return 1

    print(
        "OK: checker rejects a planted litric-defender.service and "
        "litric-defender.slice"
    )
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    errors = check_root(IMAGE)
    if errors:
        print(f"FAIL: {len(errors)} on-access Defender artefact(s) remain")
        for line in errors:
            print(f"  {line}")
        return 1
    print(
        "OK: no litric-defender.service or litric-defender.slice under image/; "
        "the on-demand scanner is the only Defender executable"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
