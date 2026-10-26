#!/usr/bin/env python3
"""Paint the LitricOS mark: a pen-drawn ring with a written L inside it.

White on transparent, 256x256, so the shell can tint it for any theme:

    python3 tools/gen_mark.py assets/brand/litric-mark.png

It is drawn with the same pen model as the boot wordmark: the line is thick
where the pen moves down and thin where it moves across, and it tapers where the
pen lands and leaves. One hand wrote both.
"""
import math
import sys

from PIL import Image, ImageDraw

OUT, SS = 256, 4
N = OUT * SS
NIB = math.radians(8)


def catmull(p0, p1, p2, p3, t):
    t2, t3 = t * t, t * t * t
    return tuple(0.5 * ((2 * b) + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t2
                        + (-a + 3 * b - 3 * c + d) * t3)
                 for a, b, c, d in zip(p0, p1, p2, p3))


def smooth(pts, per=24):
    out = []
    n = len(pts)
    at = lambda k: pts[max(0, min(n - 1, k))]
    for i in range(n - 1):
        for s in range(per):
            out.append(catmull(at(i - 1), at(i), at(i + 1), at(i + 2), s / per))
    out.append(pts[-1])
    return out


def pen(draw, pts, width, land=0.05, lift=0.12, hair=0.30):
    pts = smooth(pts)
    seg = [math.dist(pts[i], pts[i + 1]) for i in range(len(pts) - 1)]
    total = sum(seg)
    run = 0.0
    for i in range(len(pts) - 1):
        a, b = pts[i], pts[i + 1]
        th = math.atan2(b[1] - a[1], b[0] - a[0])
        w = hair + (1 - hair) * abs(math.sin(th - NIB))
        w *= min(1.0, 0.25 + 0.75 * run / (land * total + 1e-6))
        w *= min(1.0, 0.25 + 0.75 * (total - run) / (lift * total + 1e-6))
        run += seg[i]
        r = max(0.6, width * N * w / 2)
        x0, y0, x1, y1 = (a[0] * N, a[1] * N, b[0] * N, b[1] * N)
        draw.line([x0, y0, x1, y1], fill=255, width=int(r * 2))
        draw.ellipse([x1 - r, y1 - r, x1 + r, y1 + r], fill=255)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/brand/litric-mark.png"
    im = Image.new("L", (N, N), 0)
    d = ImageDraw.Draw(im)
    # the ring: one turn of the pen, from the upper right, anticlockwise
    c, rx, ry = 0.5, 0.41, 0.41
    ring = []
    for k in range(0, 349, 8):
        a = math.radians(-70 - k)
        wob = 1.0 + 0.012 * math.sin(a * 2)
        ring.append((c + rx * wob * math.cos(a), c + ry * wob * math.sin(a)))
    pen(d, ring, 0.115, land=0.03, lift=0.07)
    # the L: stem down, foot sweeping out
    pen(d, [(0.415, 0.26), (0.405, 0.45), (0.395, 0.62), (0.425, 0.71),
            (0.51, 0.725), (0.63, 0.71), (0.715, 0.67)], 0.14, land=0.04, lift=0.18)
    a = im.resize((OUT, OUT), Image.LANCZOS)
    rgba = Image.new("RGBA", (OUT, OUT), (255, 255, 255, 0))
    rgba.putalpha(a)
    rgba.save(out, optimize=True)
    # a contact sheet for a human to judge at real sizes
    sheet = Image.new("RGB", (4 * 140, 140), (22, 25, 33))
    for i, s in enumerate((128, 64, 32, 18)):
        sm = rgba.resize((s, s), Image.LANCZOS)
        sheet.paste((244, 242, 237), (i * 140 + 6, 6), sm.split()[3].point(lambda v: v) if False else None) if False else None
        tile = Image.new("RGB", (s, s), (244, 242, 237))
        sheet.paste(tile, (i * 140 + 6, 6), sm.split()[3])
    sheet.save(out.replace(".png", "-sheet.png"))
    print(out)


if __name__ == "__main__":
    main()
