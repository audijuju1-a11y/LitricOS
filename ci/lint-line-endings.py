#!/usr/bin/env python3
"""Detect CRLF line endings in files destined for a Linux system.

Every file in this repository that a Linux kernel, shell or systemd will read
must use LF. A single carriage return at the end of a line causes failures that
look like anything except what they are:

  * `set -euo pipefail\\r` -> bash reports "pipefail: invalid option name"
  * `#!/usr/bin/env bash\\r` -> "bad interpreter: no such file or directory",
    naming an interpreter that visibly exists
  * a systemd unit value silently carries a trailing CR
  * a kernel config symbol becomes `CONFIG_FOO=y\\r` and is quietly ignored

This bug reached the repository through Python: `pathlib.Path.write_text()` on a
Windows host translates every `\\n` into `\\r\\n`. Several tooling scripts were
patched that way and became unrunnable on Linux while still looking correct.

It survived initial inspection because **Git Bash hides it**. Git Bash reads
files in text mode, so `grep $'\\r'`, `od -c` and `cat -A` all show clean LF on a
file that actually contains CRLF. Only `file`, a hex dump of the raw bytes, or
reading it from WSL reveals the truth. This checker therefore reads bytes and
never text.

Usage:
    python3 ci/lint-line-endings.py          check, non-zero exit if any found
    python3 ci/lint-line-endings.py --fix    rewrite them as LF
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Extensions whose contents are consumed by Linux.
PATTERNS = [
    "*.sh",
    "*.py",
    "*.conf",
    "*.service",
    "*.slice",
    "*.socket",
    "*.target",
    "*.mount",
    "*.network",
    "*.rules",
    "*.config",
    "*.nft",
    "*.toml",
    "*.rs",
]

# Extensionless files that are still scripts or configuration.
EXTRA = [
    "image/mkosi.extra/usr/lib/litric/mm-tune",
    "keys/dev/README",
]

# Directories that are not ours, are generated, or are kept deliberately frozen.
SKIP_DIRS = {".backup-pre-v2", "build", "target", ".git", "__pycache__"}

# PowerShell is executed by Windows and CRLF is correct there.
NEVER = {".ps1"}


def candidates() -> list[Path]:
    out: set[Path] = set()
    for pat in PATTERNS:
        for p in ROOT.rglob(pat):
            if p.suffix in NEVER:
                continue
            if any(part in SKIP_DIRS for part in p.relative_to(ROOT).parts):
                continue
            if p.is_file():
                out.add(p)
    for e in EXTRA:
        p = ROOT / e
        if p.is_file():
            out.add(p)
    return sorted(out)


def main(argv: list[str]) -> int:
    fix = "--fix" in argv
    offenders: list[tuple[Path, int]] = []

    for p in candidates():
        try:
            data = p.read_bytes()
        except OSError as exc:
            print(f"WARN: cannot read {p}: {exc}")
            continue
        count = data.count(b"\r\n")
        if count == 0:
            continue
        offenders.append((p, count))
        if fix:
            # write_bytes, never write_text: on a Windows host write_text would
            # translate the LF straight back into CRLF and the "fix" would be a
            # no-op that reports success.
            p.write_bytes(data.replace(b"\r\n", b"\n"))

    rel = lambda p: str(p.relative_to(ROOT)).replace("\\", "/")

    if not offenders:
        print(f"OK: {len(candidates())} file(s) checked, all LF")
        return 0

    if fix:
        print(f"converted {len(offenders)} file(s) from CRLF to LF:")
        for p, n in offenders:
            print(f"  {rel(p)}  ({n} lines)")
        return 0

    print(f"{len(offenders)} file(s) have CRLF line endings and will fail on Linux:")
    for p, n in offenders:
        print(f"  {rel(p)}  ({n} lines)")
    print("\nFix with: python3 ci/lint-line-endings.py --fix")
    print("Note: Git Bash hides this. Verify with `file <path>` or read it from WSL.")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
