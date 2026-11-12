#!/usr/bin/env python3
"""Convert a QEMU screendump (PPM, whatever its extension) to a real PNG.

QEMU's `screendump` writes P6 PPM; naming the output .png does not change
the format, which is why the file would not open on the host. This converts
it losslessly with nothing but zlib and struct, and reports the scanout
resolution the guest is actually driving.
"""
import struct
import sys
import zlib


def read_ppm(path: str) -> tuple[int, int, bytes]:
    with open(path, "rb") as f:
        data = f.read()
    if not data.startswith(b"P6"):
        raise SystemExit(f"not a P6 PPM: {path}")
    # Header: P6, whitespace, width, height, maxval, single whitespace, pixels.
    parts: list[bytes] = []
    i = 2
    while len(parts) < 3:
        while i < len(data) and data[i : i + 1].isspace():
            i += 1
        if data[i : i + 1] == b"#":
            while data[i : i + 1] not in (b"\n", b""):
                i += 1
            continue
        j = i
        while j < len(data) and not data[j : j + 1].isspace():
            j += 1
        parts.append(data[i:j])
        i = j
    w, h, _maxval = (int(p) for p in parts)
    i += 1  # exactly one whitespace byte before the raster
    raw = data[i : i + w * h * 3]
    if len(raw) != w * h * 3:
        raise SystemExit(f"truncated raster: {len(raw)} of {w*h*3}")
    return w, h, raw


def write_png(path: str, w: int, h: int, raw: bytes) -> None:
    def chunk(tag: bytes, payload: bytes) -> bytes:
        c = tag + payload
        return struct.pack(">I", len(payload)) + c + struct.pack(">I", zlib.crc32(c))

    stride = w * 3
    scanlines = b"".join(b"\x00" + raw[y * stride : (y + 1) * stride] for y in range(h))
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(scanlines, 6))
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as f:
        f.write(png)


def main() -> int:
    src, dst = sys.argv[1], sys.argv[2]
    w, h, raw = read_ppm(src)
    write_png(dst, w, h, raw)
    print(f"{src}: scanout {w}x{h} -> {dst}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
