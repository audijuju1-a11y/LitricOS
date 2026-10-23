"""Render the LitricOS obsidian wave wallpaper and its thumbnail.

Procedural approximation of the reference image: a solid Obsidian field with a
single metallic ribbon. No CSS-style gradients: the ribbon is a lit surface
computed from distance to its own curve, not a background gradient.

Usage: python3 tools/wallpaper/gen_wallpaper.py
"""

from pathlib import Path

import numpy as np
from PIL import Image

OUT = Path(__file__).resolve().parents[2] / "assets" / "wallpapers"
OBSIDIAN = np.array([0x11, 0x13, 0x18], dtype=np.float32)
TITANIUM = np.array([0xB8, 0xBD, 0xC7], dtype=np.float32)

# Control points of the ribbon's top edge: (x fraction, y fraction).
CONTROL = [(0.0, 0.29), (0.35, 0.52), (0.62, 0.65), (0.80, 0.50), (0.90, 0.25), (1.0, 0.04)]


def edge(xs: np.ndarray) -> np.ndarray:
    """One smooth curve through the control points, in fractions of height.

    A single polynomial has no joints, so the ribbon has no kinks where two
    segments meet.
    """
    cx = np.array([p[0] for p in CONTROL])
    cy = np.array([p[1] for p in CONTROL])
    return np.polyval(np.polyfit(cx, cy, 5), xs)


def render(width: int, height: int) -> Image.Image:
    xs = np.linspace(0.0, 1.0, width, dtype=np.float32)
    top = edge(xs) * height  # per column
    ys = np.arange(height, dtype=np.float32)[:, None]
    d = ys - top[None, :]  # signed distance below the edge, in pixels
    sigma = height * 0.045

    body = np.exp(-np.clip(d, 0, None) / (height * 0.22))  # soft falloff under the ribbon
    sheen = np.exp(-(d / sigma) ** 2)  # bright band on the edge
    rim = np.exp(-(d / max(1.2, height / 900)) ** 2)  # hairline specular

    lum = 0.08 * body + 0.30 * sheen + 0.85 * rim
    lum = np.clip(lum, 0, 1)[..., None]
    img = OBSIDIAN * (1 - lum) + TITANIUM * lum
    img = np.where(d[..., None] < -height * 0.0, OBSIDIAN, img)
    return Image.fromarray(img.astype(np.uint8), "RGB")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    full = render(1920, 1080)
    full.save(OUT / "litric-obsidian-wave-1920x1080.png", optimize=True)
    thumb = full.resize((480, 270), Image.LANCZOS)
    thumb.save(OUT / "litric-obsidian-wave-thumb-480x270.png", optimize=True)
    print("wrote", OUT)


if __name__ == "__main__":
    main()
