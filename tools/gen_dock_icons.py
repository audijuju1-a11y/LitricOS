#!/usr/bin/env python3
"""Paint the LitricOS dock icons: original raster artwork, 256x256 RGBA PNGs.

Each icon is a squircle tile with a soft contact shadow, a body in one hue,
light falling from the top (a raster shading pass, not CSS), a lit rim, and a
white glyph with its own shadow. Everything is drawn here, from shapes, so the
set is original and reproducible:

    python3 tools/gen_dock_icons.py assets/icons/dock

The shell embeds the PNGs and never draws a gradient: the depth is in the
pixels.
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

OUT = 256          # delivered size
S = 4              # supersampling for the drawing
N = OUT * S        # working size
PAD = 0.075        # the tile's margin, as a share of the canvas
PAPER = (244, 242, 237)
INK = (20, 24, 33)


def squircle_mask(size, inset, n=5.0):
    """A superellipse mask (anti-aliased by supersampling) inside `size`."""
    y, x = np.mgrid[0:size, 0:size].astype(np.float32)
    c = (size - 1) / 2.0
    r = size / 2.0 - inset
    v = (np.abs((x - c) / r) ** n + np.abs((y - c) / r) ** n)
    return (v <= 1.0).astype(np.float32)


def to_img(a):
    return Image.fromarray((np.clip(a, 0, 1) * 255).astype(np.uint8), "L")


def shade(base, size, mask):
    """Body colour with light from above: lighter top, deeper bottom."""
    y = np.linspace(0.0, 1.0, size, dtype=np.float32)[:, None]
    lift = 0.20 * (1.0 - y) - 0.16 * y            # +20 % at the top, -16 % at the foot
    body = np.zeros((size, size, 3), np.float32)
    for k in range(3):
        c = base[k] / 255.0
        ch = np.where(lift >= 0, c + (1 - c) * lift, c * (1 + lift))
        body[:, :, k] = np.broadcast_to(ch, (size, size))
    return body


def tile(base, glyph_fn, glyph_color=(255, 255, 255), glyph_shadow=0.30):
    inset = N * PAD
    mask = squircle_mask(N, inset)
    m_img = to_img(mask)

    # soft contact shadow, a little lower than the tile
    sh = to_img(mask).filter(ImageFilter.GaussianBlur(N * 0.018))
    sh = ImageChops.offset(sh, 0, int(N * 0.018))
    shadow_a = np.asarray(sh, np.float32) / 255.0 * 0.45

    body = shade(base, N, mask)
    rgb = np.clip(body, 0, 1)

    # a broad soft sheen over the upper half, clipped to the tile
    y, x = np.mgrid[0:N, 0:N].astype(np.float32)
    sheen = np.clip(1.0 - ((y - N * 0.18) / (N * 0.42)) ** 2, 0, 1) * 0.10
    rgb = rgb + (1 - rgb) * sheen[:, :, None]

    # lit rim: the tile's own edge, 1.2 % wide, brightest at the top
    eroded = squircle_mask(N, inset + N * 0.010)
    rim = np.clip(mask - eroded, 0, 1)
    top = np.clip(1.0 - y / (N * 0.75), 0, 1)
    rim_light = rim * (0.10 + 0.42 * top)
    rgb = rgb + (1 - rgb) * rim_light[:, :, None]
    rim_dark = rim * np.clip((y - N * 0.55) / (N * 0.4), 0, 1) * 0.30
    rgb = rgb * (1 - rim_dark[:, :, None])

    # glyph with its own shadow
    glyph = Image.new("L", (N, N), 0)
    glyph_fn(ImageDraw.Draw(glyph), N)
    g = np.asarray(glyph, np.float32) / 255.0
    gs = np.asarray(
        ImageChops.offset(glyph.filter(ImageFilter.GaussianBlur(N * 0.012)), 0, int(N * 0.016)),
        np.float32,
    ) / 255.0
    rgb = rgb * (1 - gs[:, :, None] * glyph_shadow)
    gc = np.array(glyph_color, np.float32) / 255.0
    rgb = rgb * (1 - g[:, :, None]) + gc[None, None, :] * g[:, :, None]

    # compose: shadow underneath, tile on top
    alpha = np.clip(mask + shadow_a * (1 - mask), 0, 1)
    out_rgb = rgb * mask[:, :, None]
    out_rgb = out_rgb + 0.0 * (1 - mask[:, :, None])  # the shadow is black
    img = np.dstack([out_rgb, alpha])
    im = Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8), "RGBA")
    return im.resize((OUT, OUT), Image.LANCZOS)


# ---- glyphs, in 0..1 of the canvas ---------------------------------------

def P(n, *pts):
    return [(x * n, y * n) for x, y in pts]


def line(d, n, pts, w, fill=255):
    pts = P(n, *pts)
    d.line(pts, fill=fill, width=int(w * n), joint="curve")
    r = w * n / 2
    for x, y in (pts[0], pts[-1]):
        d.ellipse([x - r, y - r, x + r, y + r], fill=fill)


def rrect(d, n, box, r, fill=255, outline=None, width=0):
    x0, y0, x1, y1 = [v * n for v in box]
    d.rounded_rectangle([x0, y0, x1, y1], radius=r * n, fill=fill,
                        outline=outline, width=int(width * n))


def g_launcher(d, n):
    s, gap, x0 = 0.115, 0.05, 0.5 - (0.115 * 3 + 0.05 * 2) / 2
    for i in range(3):
        for j in range(3):
            x, y = x0 + i * (s + gap), x0 + j * (s + gap)
            rrect(d, n, (x, y, x + s, y + s), 0.032)


def g_files(d, n):
    # the folder: a tab, then the body, then a lighter front leaf
    rrect(d, n, (0.215, 0.285, 0.52, 0.42), 0.035)
    rrect(d, n, (0.215, 0.335, 0.785, 0.715), 0.05)
    # the front leaf is knocked back so the folder has two planes
    rrect(d, n, (0.215, 0.42, 0.785, 0.715), 0.05, fill=215)
    rrect(d, n, (0.215, 0.335, 0.785, 0.43), 0.05, fill=255)


def g_browser(d, n):
    c, r = 0.5, 0.265
    d.ellipse([(c - r) * n, (c - r) * n, (c + r) * n, (c + r) * n],
              outline=255, width=int(0.045 * n))
    d.ellipse([(c - 0.115) * n, (c - r) * n, (c + 0.115) * n, (c + r) * n],
              outline=255, width=int(0.032 * n))
    line(d, n, [(c - r, c), (c + r, c)], 0.032)
    line(d, n, [(c - r + 0.04, c - 0.12), (c + r - 0.04, c - 0.12)], 0.026)
    line(d, n, [(c - r + 0.04, c + 0.12), (c + r - 0.04, c + 0.12)], 0.026)


def g_terminal(d, n):
    line(d, n, [(0.27, 0.345), (0.445, 0.5), (0.27, 0.655)], 0.065)
    line(d, n, [(0.50, 0.675), (0.73, 0.675)], 0.065)


def g_settings(d, n):
    c, ro, ri = 0.5, 0.275, 0.20
    teeth = 8
    pts = []
    for i in range(teeth * 4):
        a = i / (teeth * 4) * 2 * math.pi
        out = (i % 4) in (0, 1)
        r = ro if out else ri
        pts.append((c + math.cos(a) * r, c + math.sin(a) * r))
    d.polygon(P(n, *pts), fill=255)
    d.ellipse([(c - ri + 0.02) * n, (c - ri + 0.02) * n, (c + ri - 0.02) * n, (c + ri - 0.02) * n], fill=255)
    h = 0.085
    d.ellipse([(c - h) * n, (c - h) * n, (c + h) * n, (c + h) * n], fill=0)


def g_activity(d, n):
    line(d, n, [(0.20, 0.54), (0.33, 0.54), (0.41, 0.32), (0.52, 0.70), (0.60, 0.44),
                (0.66, 0.54), (0.80, 0.54)], 0.062)


def g_defender(d, n):
    d.polygon(P(n, (0.5, 0.2), (0.755, 0.285), (0.74, 0.54), (0.5, 0.80), (0.26, 0.54),
                (0.245, 0.285)), fill=255)
    line(d, n, [(0.385, 0.49), (0.465, 0.575), (0.625, 0.395)], 0.06, fill=0)


def g_environments(d, n):
    rrect(d, n, (0.20, 0.20, 0.60, 0.60), 0.07, fill=0, outline=255, width=0.05)
    rrect(d, n, (0.40, 0.40, 0.80, 0.80), 0.07, fill=255)


def g_trash(d, n):
    rrect(d, n, (0.30, 0.30, 0.70, 0.31), 0.0)
    line(d, n, [(0.26, 0.31), (0.74, 0.31)], 0.05)
    line(d, n, [(0.42, 0.25), (0.58, 0.25)], 0.05)
    d.polygon(P(n, (0.31, 0.37), (0.69, 0.37), (0.65, 0.76), (0.35, 0.76)), fill=255)
    for x in (0.43, 0.5, 0.57):
        line(d, n, [(x, 0.44), (x, 0.69)], 0.028, fill=0)


ICONS = {
    # name:        (body,            glyph colour, glyph)
    "launcher":    ((236, 234, 229), INK,              g_launcher),
    "files":       ((222, 140, 36),  (255, 255, 255),  g_files),
    "browser":     ((48, 124, 226),  (255, 255, 255),  g_browser),
    "terminal":    ((34, 38, 52),    (125, 226, 160),  g_terminal),
    "settings":    ((118, 126, 144), (255, 255, 255),  g_settings),
    "activity":    ((218, 76, 68),   (255, 255, 255),  g_activity),
    "defender":    ((34, 154, 104),  (255, 255, 255),  g_defender),
    "environments": ((122, 88, 226), (255, 255, 255),  g_environments),
    "trash":       ((86, 94, 112),   (255, 255, 255),  g_trash),
}


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "assets/icons/dock"
    os.makedirs(out, exist_ok=True)
    for name, (body, glyph_color, fn) in ICONS.items():
        im = tile(body, fn, glyph_color)
        path = os.path.join(out, f"{name}.png")
        im.save(path, optimize=True)
        print(path)


if __name__ == "__main__":
    main()
