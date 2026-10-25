#!/usr/bin/env python3
"""Paint the LitricOS status icons: white on transparent, 128x128 alpha PNGs.

One grid (24 units), one stroke (2.0), round caps and joins, so the set reads
as one hand. The shell tints them with the theme's text colour or a tone:

    python3 tools/gen_status_icons.py assets/icons/status
"""
import math
import os
import sys

from PIL import Image, ImageDraw

OUT, SS, U = 128, 4, 24.0
N = OUT * SS
K = N / U
W = 2.0  # stroke, in grid units


def new():
    im = Image.new("L", (N, N), 0)
    return im, ImageDraw.Draw(im)


def pts(*p):
    return [(x * K, y * K) for x, y in p]


def stroke(d, points, w=W, fill=255, closed=False):
    pp = pts(*points)
    if closed:
        pp = pp + [pp[0], pp[1]]
    d.line(pp, fill=fill, width=int(w * K), joint="curve")
    r = w * K / 2
    for x, y in ([pp[0], pp[-1]] if not closed else pp[:-2]):
        d.ellipse([x - r, y - r, x + r, y + r], fill=fill)


def arc(cx, cy, r, a0, a1, n=48):
    return [(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)),
             cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n))) for i in range(n + 1)]


def dot(d, x, y, r, fill=255):
    d.ellipse([(x - r) * K, (y - r) * K, (x + r) * K, (y + r) * K], fill=fill)


def rrect(d, box, r, fill=None, outline=None, w=W):
    x0, y0, x1, y1 = [v * K for v in box]
    if outline is not None:
        d.rounded_rectangle([x0, y0, x1, y1], radius=r * K, fill=outline)
        d.rounded_rectangle([x0 + w * K, y0 + w * K, x1 - w * K, y1 - w * K],
                            radius=max(r - w, 0.2) * K, fill=0)
    if fill is not None:
        d.rounded_rectangle([x0, y0, x1, y1], radius=r * K, fill=fill)


def wifi(d):
    for r in (13.0, 8.6, 4.2):
        stroke(d, arc(12, 19.2, r, -135, -45))
    dot(d, 12, 19.0, 1.7)


def wifi_off(d, im):
    wifi(d)
    # a clear channel either side of the slash, then the slash itself
    stroke(d, [(4.2, 3.6), (20.0, 20.4)], w=W + 3.2, fill=0)
    stroke(d, [(4.2, 3.6), (20.0, 20.4)])


def volume(d, muted=False):
    d.polygon(pts((3.2, 9.4), (7.6, 9.4), (12.6, 5.2), (12.6, 18.8), (7.6, 14.6), (3.2, 14.6)),
              fill=255)
    stroke(d, [(3.2, 9.4), (7.6, 9.4), (12.6, 5.2), (12.6, 18.8), (7.6, 14.6), (3.2, 14.6)],
           w=1.2, closed=True)
    if muted:
        stroke(d, [(16.2, 9.4), (21.2, 14.6)])
        stroke(d, [(21.2, 9.4), (16.2, 14.6)])
    else:
        stroke(d, arc(12.6, 12, 4.4, -45, 45, 24))
        stroke(d, arc(12.6, 12, 8.2, -45, 45, 24))


def battery(d):
    rrect(d, (1.8, 6.6, 19.6, 17.4), 3.0, outline=255, w=1.8)
    rrect(d, (20.8, 10.2, 22.4, 13.8), 0.8, fill=255)


def shield(d):
    stroke(d, [(12, 2.8), (19.6, 5.6), (19.6, 12.2), (12, 21.2), (4.4, 12.2), (4.4, 5.6)],
           closed=True)
    stroke(d, [(8.4, 11.8), (11, 14.4), (15.8, 9.2)])


def controls(d):
    rrect(d, (2.4, 4.4, 21.6, 10.4), 3.0, outline=255, w=1.8)
    dot(d, 17.4, 7.4, 2.0)
    rrect(d, (2.4, 13.6, 21.6, 19.6), 3.0, outline=255, w=1.8)
    dot(d, 6.6, 16.6, 2.0)


def bell(d):
    body = arc(12, 10.2, 5.6, 180, 360, 28)
    body += [(17.6, 15.0), (19.4, 17.0), (4.6, 17.0), (6.4, 15.0)]
    stroke(d, body, closed=True)
    stroke(d, arc(12, 18.6, 2.3, 10, 170, 12))
    dot(d, 12, 3.2, 1.0)


def search(d):
    stroke(d, arc(10.4, 10.4, 6.6, 0, 360, 72))
    stroke(d, [(15.4, 15.4), (20.6, 20.6)])


ICONS = {
    "wifi": lambda d, im: wifi(d),
    "wifi_off": wifi_off,
    "volume": lambda d, im: volume(d),
    "volume_muted": lambda d, im: volume(d, True),
    "battery": lambda d, im: battery(d),
    "shield": lambda d, im: shield(d),
    "controls": lambda d, im: controls(d),
    "bell": lambda d, im: bell(d),
    "search": lambda d, im: search(d),
}


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/icons/status"
    os.makedirs(out, exist_ok=True)
    sheet = Image.new("RGB", (len(ICONS) * 150, 220), (22, 25, 33))
    for i, (name, fn) in enumerate(ICONS.items()):
        im, d = new()
        fn(d, im)
        a = im.resize((OUT, OUT), Image.LANCZOS)
        rgba = Image.new("RGBA", (OUT, OUT), (255, 255, 255, 0))
        rgba.putalpha(a)
        rgba.save(os.path.join(out, f"{name}.png"), optimize=True)
        for j, s in enumerate((96, 18)):
            sm = rgba.resize((s, s), Image.LANCZOS)
            tile = Image.new("RGB", (s, s), (244, 242, 237))
            sheet.paste(tile, (i * 150 + 10, 8 + j * 110), sm.split()[3])
        print(name)
    if len(sys.argv) > 2:
        sheet.save(sys.argv[2])


if __name__ == "__main__":
    main()
