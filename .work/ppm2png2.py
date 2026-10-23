import struct
import sys
import zlib


def write_png(path, w, h, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(t, b):
        return (
            struct.pack(">I", len(b))
            + t
            + b
            + struct.pack(">I", zlib.crc32(t + b) & 0xFFFFFFFF)
        )

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9))
    png += chunk(b"IEND", b"")
    open(path, "wb").write(png)


def read_ppm(path):
    data = open(path, "rb").read()
    assert data[:2] == b"P6"
    fields = []
    pos = 2
    while len(fields) < 3:
        while data[pos : pos + 1].isspace():
            pos += 1
        if data[pos : pos + 1] == b"#":
            while data[pos : pos + 1] not in (b"\n", b""):
                pos += 1
            continue
        start = pos
        while not data[pos : pos + 1].isspace():
            pos += 1
        fields.append(int(data[start:pos]))
    pos += 1
    w, h, _maxv = fields
    return w, h, data[pos : pos + w * h * 3]


def main():
    src, dst = sys.argv[1:3]
    crop = [int(v) for v in sys.argv[3:7]] if len(sys.argv) > 6 else None
    scale = int(sys.argv[7]) if len(sys.argv) > 7 else 1
    w, h, px = read_ppm(src)
    x0, y0, x1, y1 = crop if crop else (0, 0, w, h)
    x1, y1 = min(x1, w), min(y1, h)
    rows = []
    for y in range(y0, y1, scale):
        row = bytearray()
        for x in range(x0, x1, scale):
            i = (y * w + x) * 3
            row += px[i : i + 3]
        rows.append(row)
    write_png(dst, (x1 - x0 + scale - 1) // scale, len(rows), rows)
    print("wrote", dst)


main()
