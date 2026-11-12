#!/usr/bin/env python3
"""List the non-ASCII characters the shell puts inside string literals.

The typeface is drawn rather than loaded, so a character it does not have draws
as a box. `font::draw` does that on purpose -- silence would look like a spacing
bug -- but it means the face has to cover everything the interface actually
prints. This walks the shell's sources and reports which characters those are,
most used first, so the typeface's coverage can be checked against reality
rather than against a guess about which symbols a UI needs.

Usage::

    python tools/scan-nonascii.py [DIR]
"""

from __future__ import annotations

import collections
import sys
from pathlib import Path

BACKSLASH = chr(92)
QUOTE = chr(34)


def literals(source: str):
    """Yield the contents of the string literals on each line."""
    for line in source.splitlines():
        buf = []
        inside = False
        i = 0
        while i < len(line):
            ch = line[i]
            if inside and ch == BACKSLASH:
                buf.append(line[i + 1 : i + 2])
                i += 2
                continue
            if ch == QUOTE:
                if inside:
                    yield "".join(buf)
                    buf.clear()
                inside = not inside
                i += 1
                continue
            if inside:
                buf.append(ch)
            i += 1


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "src/litric-shell/src")
    seen: collections.Counter[str] = collections.Counter()
    where: dict[str, set[str]] = {}
    for path in sorted(root.glob("*.rs")):
        for text in literals(path.read_text(encoding="utf-8", errors="replace")):
            for ch in text:
                if ord(ch) > 127:
                    seen[ch] += 1
                    where.setdefault(ch, set()).add(path.name)
    if not seen:
        print("scan-nonascii: no non-ASCII characters in string literals")
        return 0
    for ch, n in sorted(seen.items(), key=lambda kv: -kv[1]):
        files = ",".join(sorted(where[ch]))
        print(f"U+{ord(ch):04X} {ch!r} {n} in {files}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
