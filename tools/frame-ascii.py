#!/usr/bin/env python3
"""Render a PPM screenshot as coarse luminance text, for terminals and CI logs.

WHY THIS EXISTS
---------------
A frame is a picture, and this environment has no image viewer: an agent or a
CI job that has just captured the framebuffer needs to answer "is the desktop
up, or is this still a black console?" without a human. Downsampling to a
character grid answers exactly that question -- the system bar reads as a band,
a window as a bright rectangle, a console prompt as a few scattered glyphs --
and it prints into a build log where the frame itself cannot.

It is deliberately not an image viewer: no colour, no text recognition, no
claim that a glyph is legible. `tools/ppm2png.py` makes the file a human can
look at.

    python3 tools/frame-ascii.py frame.ppm --width 100
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

RAMP = " .:-=+*#%@"


def read_ppm(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if not data.startswith(b"P6"):
        raise SystemExit(f"{path}: not a binary PPM (P6)")
    pos = 2
    tokens: list[int] = []
    while len(tokens) < 3:
        while pos < len(data) and data[pos : pos + 1].isspace():
            pos += 1
        if data[pos : pos + 1] == b"#":
            while pos < len(data) and data[pos : pos + 1] != b"\n":
                pos += 1
            continue
        start = pos
        while pos < len(data) and not data[pos : pos + 1].isspace():
            pos += 1
        tokens.append(int(data[start:pos]))
    pos += 1
    width, height, _ = tokens
    return width, height, data[pos:]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("frame")
    parser.add_argument("--width", type=int, default=100)
    parser.add_argument("--stats", action="store_true", help="print colour statistics")
    args = parser.parse_args(argv)

    width, height, pixels = read_ppm(Path(args.frame))
    cols = max(8, min(args.width, width))
    rows = max(4, round(cols * height / width / 2.1))  # 2.1: a cell is taller than wide
    step_x, step_y = width / cols, height / rows

    out: list[str] = []
    for r in range(rows):
        line: list[str] = []
        for c in range(cols):
            x0, y0 = int(c * step_x), int(r * step_y)
            x1 = max(x0 + 1, int((c + 1) * step_x))
            y1 = max(y0 + 1, int((r + 1) * step_y))
            total = 0
            count = 0
            for y in range(y0, min(y1, height), max(1, (y1 - y0) // 3)):
                base = (y * width) * 3
                for x in range(x0, min(x1, width), max(1, (x1 - x0) // 3)):
                    i = base + x * 3
                    total += pixels[i] * 3 + pixels[i + 1] * 6 + pixels[i + 2]
                    count += 1
            lum = total / max(count, 1) / 10  # 0..255
            line.append(RAMP[min(len(RAMP) - 1, int(lum / 255 * len(RAMP)))])
        out.append("".join(line))

    print(f"# {width}x{height} -> {cols}x{rows}")
    print("\n".join(out))

    if args.stats:
        stride = max(1, width * height // 20000)
        seen: dict[tuple[int, int, int], int] = {}
        for p in range(0, width * height, stride):
            i = p * 3
            key = (pixels[i], pixels[i + 1], pixels[i + 2])
            seen[key] = seen.get(key, 0) + 1
        top = sorted(seen.items(), key=lambda kv: -kv[1])[:8]
        print("# top colours:")
        for (r, g, b), n in top:
            print(f"#   #{r:02x}{g:02x}{b:02x}  {n * 100 / (width * height / stride):.1f}%")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
