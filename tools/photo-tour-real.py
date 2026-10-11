#!/usr/bin/env python3
"""LitricOS — precise interaction tour with code-derived coordinates.

Icon grid (views.rs draw_desktop_icons): top_bar=28, space Lg=16 → top=44;
cell 92x88, x0=16, COLUMN-major: index i → col=i/4, row=i%4.
Icons: 0 Files, 1 Browser, 2 Terminal, 3 Defender, 4 Settings, 5 Trash,
6 Documents, 7 Downloads (apps.rs DesktopIcons::standard).

Launcher opens with a bare Meta press+release (shortcuts.rs).
"""

from __future__ import annotations

import argparse
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from photo_tour_lib import click, rclick, dblclick, key, type_text, move, boot_qmp, Tour  # noqa: E402


def icon_center(i: int) -> tuple[int, int]:
    col, row = divmod(i, 4)
    x = 16 + col * 92 + 42
    y = 44 + row * 88 + 44
    return x, y


def meta_launcher(q) -> None:
    key(q, "meta")
    time.sleep(0.7)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    out = args.out or os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "build", "tour",
        time.strftime("%Y%m%d-%H%M%S") + "-real",
    )

    proc, q, serial, diag = boot_qmp(out)
    tour = Tour(q, out)

    def S(s: float) -> None:
        time.sleep(s)

    try:
        for t in (14, 15, 16):
            S(t)
        tour.section = "30-real-desktop"
        tour.shoot("idle")

        # --- desktop icons: select + open every app ----------------------
        apps = ["files", "browser", "terminal", "defender", "settings", "trash"]
        for idx, name in enumerate(apps):
            tour.section = f"31-icon-{name}"
            x, y = icon_center(idx)
            click(q, x, y)
            S(0.4)
            tour.shoot("selected")
            dblclick(q, x, y)
            S(1.6)
            tour.shoot("window")
            rclick(q, 640, 420)
            S(0.5)
            tour.shoot("ctx")
            key(q, "esc")
            rclick(q, 200, 12)
            S(0.5)
            tour.shoot("appmenu-ctx")
            key(q, "esc")
            key(q, "alt-f4")
            S(0.8)
            tour.shoot("closed")

        # --- launcher via Meta --------------------------------------------
        tour.section = "32-launcher"
        meta_launcher(q)
        tour.shoot("open")
        type_text(q, "browser")
        S(0.4)
        tour.shoot("typed-browser")
        key(q, "ret")
        S(2.5)
        tour.shoot("browser-launched")

        # --- browser real use: omnibox + engines --------------------------
        tour.section = "33-browser-use"
        # omnibox: click into the browser window top area
        click(q, 480, 130)
        S(0.4)
        tour.shoot("omnibox-click")
        type_text(q, "litric")
        S(0.3)
        tour.shoot("omnibox-typed")
        key(q, "ret")
        S(3.0)
        tour.shoot("search-run")
        # right-click omnibox → engine menu if any
        rclick(q, 480, 130)
        S(0.5)
        tour.shoot("omnibox-ctx")
        key(q, "esc")
        key(q, "alt-f4")
        S(0.8)

        # --- files: new folder via desktop context menu -------------------
        tour.section = "34-files-ops"
        x, y = icon_center(0)
        dblclick(q, x, y)
        S(1.6)
        tour.shoot("files-open")
        rclick(q, 400, 350)
        S(0.5)
        tour.shoot("ctx-blank")
        # click the first menu row (New Folder) — menu opens +1px from cursor
        click(q, 410, 365)
        S(0.8)
        tour.shoot("new-folder-dialog")
        type_text(q, "TourDir")
        S(0.3)
        tour.shoot("typed")
        key(q, "ret")
        S(1.0)
        tour.shoot("created")
        rclick(q, 400, 350)
        S(0.5)
        tour.shoot("ctx-2")
        key(q, "esc")
        key(q, "alt-f4")
        S(0.8)

        # --- defender: launch and scan ------------------------------------
        tour.section = "35-defender"
        x, y = icon_center(3)
        dblclick(q, x, y)
        S(1.6)
        tour.shoot("window")
        rclick(q, 640, 400)
        S(0.5)
        tour.shoot("ctx")
        key(q, "esc")
        # right-click body may offer ScanNow; also try appmenu
        rclick(q, 200, 12)
        S(0.5)
        tour.shoot("appmenu")
        key(q, "esc")
        key(q, "alt-f4")
        S(0.8)

        # --- wallpaper cycle + display settings via desktop ctx -----------
        tour.section = "36-desktop-ops"
        rclick(q, 1000, 650)
        S(0.5)
        tour.shoot("ctx-desktop")
        # walk rows and shoot each hover
        for i in range(4):
            move(q, 1020, 665 + i * 26)
            S(0.3)
            tour.shoot(f"ctx-row-{i}")
        click(q, 1020, 665 + 3 * 26)  # a row click (wallpaper/display)
        S(1.0)
        tour.shoot("row-clicked")
        key(q, "esc")

        # --- final ---------------------------------------------------------
        tour.section = "37-final"
        tour.shoot("done")

        q.close()
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()

    png = sum(1 for root, _, files in os.walk(out) for f in files if f.endswith(".png"))
    print(f"\n[tour-real] complete: {out}")
    print(f"[tour-real] screenshots: {png}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
