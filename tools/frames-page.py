#!/usr/bin/env python3
"""Build a review page from the frames ``frame_dump`` produced.

Why this exists: the governance plan asks for golden screens and requires them
to come from the machine. Until an image is booted, a reviewer has no way to see
what a change did, and "the diff looks right" is not a review. This turns the
output of ``cargo run --example frame_dump`` into one page a person can actually
look at, with the numbers attached to each picture:

  * the fingerprint from ``frame_dump`` -- the identity of the frame, so a
    changed picture is visible as a changed number rather than as a file that
    might still be the old one;
  * draw time and painted coverage -- the frame budget evidence for that scene.

Images are embedded as data URIs. A page that fetches its pictures from the
directory it happens to sit in breaks the moment it is published, moved, or
opened from somewhere else, and evidence that only loads on the machine that
made it is not evidence.

Usage::

    cd src && cargo run --release -p litric-shell --example frame_dump -- ../build/frames \\
        > ../build/frames/metrics.txt
    python tools/frames-page.py build/frames build/frames.html

The metrics file is optional: without it the page still shows the frames and
says the numbers are missing, rather than inventing them.
"""

from __future__ import annotations

import base64
import html
import sys
from pathlib import Path

FIELDS = ("scene", "ppm", "bytes", "px", "visited_px", "coverage", "draw_ms", "fingerprint")


def parse_metrics(path: Path) -> dict[str, dict[str, str]]:
    """Map scene name -> its metric fields, from ``key=value`` lines."""
    if not path.exists():
        return {}
    out: dict[str, dict[str, str]] = {}
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        fields: dict[str, str] = {}
        for token in line.split():
            key, sep, value = token.partition("=")
            if sep and key in FIELDS:
                fields[key] = value
        scene = fields.get("scene")
        if scene:
            out[scene] = fields
    return out


def frames(directory: Path) -> list[tuple[str, Path, Path]]:
    """(scene, png, ppm) for every frame, in name order."""
    found = []
    for ppm in sorted(directory.glob("*.ppm")):
        png = ppm.with_suffix(".png")
        if png.exists():
            found.append((ppm.stem, png, ppm))
    return found


def data_uri(png: Path) -> str:
    return "data:image/png;base64," + base64.b64encode(png.read_bytes()).decode("ascii")


def card(scene: str, png: Path, metrics: dict[str, dict[str, str]]) -> str:
    m = metrics.get(scene, {})
    fp = html.escape(m.get("fingerprint", "missing -- metrics not provided"))
    draw = html.escape(m.get("draw_ms", "?"))
    cov = html.escape(m.get("coverage", "?"))
    visited = html.escape(m.get("visited_px", "?"))
    return f"""<figure class="card">
  <figcaption>
    <strong>{html.escape(scene)}</strong>
    <span class="fp" title="FNV-1a over the frame's pixels">{fp}</span>
    <span class="nums">draw {draw} ms &middot; coverage {cov} &middot; {visited} px painted</span>
  </figcaption>
  <img alt="{html.escape(scene)}" src="{data_uri(png)}">
</figure>"""


def page(directory: Path, metrics_path: Path) -> str:
    metrics = parse_metrics(metrics_path)
    cards = "\n".join(card(s, p, metrics) for s, p, _ in frames(directory))
    have = "given" if metrics else "missing -- the page reports that rather than inventing numbers"
    return f"""<!doctype html>
<title>LitricOS dumped frames</title>
<meta charset="utf-8">
<style>
  :root {{ color-scheme: dark; }}
  body {{ margin: 0 auto; max-width: 1100px; padding: 32px 20px 80px;
         background: #0b0d12; color: #e7e9ef;
         font: 15px/1.5 -apple-system, "Segoe UI", system-ui, sans-serif; }}
  h1 {{ font-size: 22px; margin: 0 0 4px; }}
  p.lede {{ color: #a3a8b8; margin: 0 0 28px; }}
  figure.card {{ margin: 0 0 28px; }}
  figcaption {{ display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: baseline;
                padding: 8px 10px; background: #141824; border: 1px solid #232838;
                border-bottom: 0; border-radius: 10px 10px 0 0; }}
  .fp {{ font: 13px ui-monospace, "Cascadia Mono", monospace; color: #8fd7c0; }}
  .nums {{ color: #a3a8b8; font-size: 13px; }}
  img {{ display: block; width: 100%; height: auto; border: 1px solid #232838;
         border-radius: 0 0 10px 10px; }}
</style>
<h1>LitricOS: frames dumped from the shipping renderer</h1>
<p class="lede">Rendered on this host by <code>litric-shell --example frame_dump</code>
into an in-memory canvas &mdash; no display, no virtual machine. Same drawing code
the booted machine runs; the system facts are fixed fixtures, so two runs of the
same code produce the same fingerprint. Metrics: {have}.</p>
{cards}
"""


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print(__doc__)
        return 2
    directory = Path(sys.argv[1])
    out = Path(sys.argv[2]) if len(sys.argv) == 3 else directory / "frames.html"
    found = frames(directory)
    if not found:
        print(f"frames-page: no .ppm/.png frame pairs in {directory}")
        return 1
    out.write_text(page(directory, directory / "metrics.txt"), encoding="utf-8")
    print(f"frames-page: {len(found)} frame(s) -> {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
