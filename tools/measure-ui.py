#!/usr/bin/env python3
"""Measure the rendered desktop against the muted-material targets.

Reads the PPM the QEMU screendump actually wrote and reports the statistics
the design pass is accountable for: mean chroma (saturation proxy) and mean
luminance over the whole frame, plus the chroma at the three wallpaper wash
anchors. Old build vs new build through the same ruler.
"""
import struct
import sys


def read_ppm(path):
    with open(path, "rb") as f:
        data = f.read()
    if not data.startswith(b"P6"):
        raise SystemExit(f"not a P6 PPM: {path}")
    parts = []
    i = 2
    while len(parts) < 3:
        while data[i : i + 1].isspace():
            i += 1
        if data[i : i + 1] == b"#":
            while not data[i : i + 1] in (b"\n", b""):
                i += 1
            continue
        j = i
        while not data[j : j + 1].isspace():
            j += 1
        parts.append(data[i:j])
        i = j
    w, h = int(parts[0]), int(parts[1])
    i += 1
    return w, h, data[i : i + w * h * 3]


def main():
    w, h, raw = read_ppm(sys.argv[1])
    n = w * h
    chroma_sum = 0
    lum_sum = 0.0
    for i in range(0, n * 3, 3):
        r, g, b = raw[i], raw[i + 1], raw[i + 2]
        chroma_sum += max(r, g, b) - min(r, g, b)
        lum_sum += 0.2126 * r + 0.7152 * g + 0.0722 * b
    anchors = []
    for fx, fy in ((0.14, 0.10), (0.88, 0.22), (0.62, 0.98)):
        px = int(fy * (h - 1)) * w + int(fx * (w - 1))
        i = px * 3
        r, g, b = raw[i], raw[i + 1], raw[i + 2]
        anchors.append((fx, fy, max(r, g, b) - min(r, g, b)))
    print(f"frame        : {w}x{h}")
    print(f"mean chroma  : {chroma_sum / n:.1f} / 255   (old build ~60+, target < 40)")
    print(f"mean luminance: {lum_sum / n:.1f} / 255")
    for fx, fy, c in anchors:
        print(f"wash anchor  : ({fx:.2f},{fy:.2f}) chroma {c}")


if __name__ == "__main__":
    main()
