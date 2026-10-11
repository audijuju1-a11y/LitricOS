#!/usr/bin/env python3
"""LitricOS — deep app tour over QMP (companion to photo-tour.py).

Drives the launcher and every app window: for each app, launch it, photograph
the window, open its app menu, open a context menu inside it, then close.
Section folders: 20-launcher, 21-<app>, ...

    python3 tools/photo-tour-apps.py [--out DIR]
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from photo_tour_lib import click, rclick, key, type_text  # noqa: E402
from photo_tour_lib import boot_qmp, Tour  # noqa: E402

APPS = [
    ("files", "files", "Files"),
    ("terminal", "terminal", "Terminal"),
    ("defender", "defender", "Defender"),
    ("browser", "browser", "Browser"),
    ("environments", "environments", "Environments"),
    ("activity", "activity", "Activity"),
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    out = args.out or os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "build", "tour", time.strftime("%Y%m%d-%H%M%S") + "-apps")

    proc, q, serial, diag = boot_qmp(out)
    tour = Tour(q, out)

    def S(s: float) -> None:
        time.sleep(s)

    try:
        # wait for desktop
        waited = 0
        for t in (12, 13, 15):
            S(t)
            waited += t
        tour.section = "20-launcher"
        tour.shoot("desktop-idle")

        # Launcher: try Ctrl+Space first
        key(q, "ctrl-spc")
        S(1.0)
        tour.shoot("launcher-open")
        key(q, "esc")

        for slug, typed, label in APPS:
            tour.section = f"21-{slug}"
            # open launcher
            key(q, "ctrl-spc")
            S(0.8)
            tour.shoot("launcher")
            type_text(q, typed)
            S(0.4)
            tour.shoot(f"launcher-typed-{slug}")
            key(q, "ret")
            S(2.0)
            tour.shoot(f"{label.lower()}-window")
            # app menu in menubar (app name is leftmost)
            click(q, 200, 12)
            S(0.5)
            tour.shoot(f"{slug}-appmenu")
            key(q, "down")
            tour.shoot(f"{slug}-appmenu-nav")
            key(q, "esc")
            # context menu inside the window body
            rclick(q, 640, 400)
            S(0.6)
            tour.shoot(f"{slug}-ctx")
            key(q, "esc")
            # keyboard: select-all, f-keys that apps own
            key(q, "ctrl-a")
            S(0.3)
            tour.shoot(f"{slug}-ctrl-a")
            key(q, "esc")
            # close
            key(q, "alt-f4")
            S(0.8)
            tour.shoot(f"{slug}-closed")

        # final desktop
        tour.section = "22-final"
        tour.shoot("after-apps")

        q.close()
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()

    png = sum(1 for root, _, files in os.walk(out) for f in files if f.endswith(".png"))
    print(f"\n[tour-apps] complete: {out}")
    print(f"[tour-apps] screenshots: {png}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
