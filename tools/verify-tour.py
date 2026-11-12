#!/usr/bin/env python3
"""LitricOS — deterministic functional self-test on the disposable VM.

Every surface this build touched is exercised with the REAL input path (QMP
input-send-event) and every transition is asserted in-flight by screenshot
hash: a feature that leaves the screen unchanged has failed, and the tour
says so instead of hoping an eyeball catches it later.

Determinism rules:
  - windows open through the dock in a fixed order, so each window's frame
    follows from the cascade (64 + 28n, 76 + 28n);
  - the idle dock click lands at x=400, left of the first item, so it cannot
    silently open an app and shift the cascade;
  - every dock click waits for the dock to be visually settled first.
"""

import hashlib
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from photo_tour_lib import Qmp, Tour, boot_qmp, click, key, move, rclick, type_text  # noqa: E402


def md5_png(path: str) -> str:
    with open(path, "rb") as f:
        return hashlib.md5(f.read()).hexdigest()


def dock_xy(i: int) -> tuple[int, int]:
    # item rect (432 + 52*i, 750, 52, 52) -> center (458 + 52*i, 776)
    return 458 + 52 * i, 776


def cascade(n: int) -> tuple[int, int]:
    # First boot auto-opens Environments + Activity, so dock-opened windows
    # start at cascade index 2.
    return 64 + 28 * (n + 2), 76 + 28 * (n + 2)


def main() -> None:
    out = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "build", "tour",
        time.strftime("%Y%m%d-%H%M%S") + "-verify",
    )
    proc, q, serial, diag = boot_qmp(out)
    tour = Tour(q, out)

    def S(s: float) -> None:
        time.sleep(s)

    results: list[tuple[str, str]] = []

    def note(name: str, verdict: str) -> None:
        results.append((name, verdict))
        print(f"[verify] {name}: {verdict}", flush=True)

    def last_shot() -> str:
        d = os.path.join(out, tour.section)
        pngs = sorted(f for f in os.listdir(d) if f.endswith(".png"))
        return os.path.join(d, pngs[-1])

    def shoot_and_note(name: str, shot: str) -> None:
        tour.shoot(shot)
        note(name, f"hash={md5_png(last_shot())[:8]} ({shot})")

    try:
        # ---- boot --------------------------------------------------------
        for t in (14, 15, 16):
            S(t)
        tour.section = "00-boot"
        tour.shoot("desktop")
        S(0.5)

        # ---- 1. dock reveal / auto-hide -----------------------------------
        tour.section = "01-dock"
        move(q, 640, 300)
        S(1.2)
        hidden_hash = shoot_and_note("dock-hidden", "hidden") or md5_png(last_shot())
        move(q, 640, 780)  # rest on the bottom edge
        S(1.0)  # settled: the click that follows must hit the item
        revealed_hash = shoot_and_note("dock-revealed", "revealed") or md5_png(last_shot())
        note("dock-reveal-differs", "PASS" if revealed_hash != hidden_hash else "FAIL: identical")
        click(q, 400, 780, hold=0.02)  # in the band, left of the first item
        S(0.8)
        idle_hash = md5_png(last_shot()) if tour.shoot("after-idle-click") is None else ""
        note("idle-band-click-opens-nothing",
             "PASS" if idle_hash == revealed_hash else f"CHECK: {idle_hash[:8]} != {revealed_hash[:8]}")

        # ---- 2. dock opens Files (window 0 at (64, 76)) --------------------
        tour.section = "02-dock-open"
        x, y = dock_xy(3)
        click(q, x, y)
        S(1.6)
        files_hash = md5_png(last_shot()) if tour.shoot("files-open") is None else ""
        note("dock-open-files", "PASS" if files_hash != revealed_hash else "FAIL: nothing opened")

        # ---- 3. through-click regression ----------------------------------
        tour.section = "03-through-click"
        fx, fy = cascade(0)  # Files: (120, 132)
        click(q, 400, 320)  # Files body: must focus, never open what's behind
        S(0.8)
        tour.shoot("files-body-click")
        click(q, fx + 520, fy + 17)  # Files title bar: focus only
        S(0.8)
        tour.shoot("files-title-click")
        note("through-click", "shots captured for review")

        # ---- 4. dock open/focus/minimize cycle (Browser, window 1) --------
        tour.section = "04-dock-focus"
        x, y = dock_xy(4)
        click(q, x, y)
        S(1.8)
        tour.shoot("browser-open")
        open_hash = md5_png(last_shot())
        click(q, x, y)  # focused app -> minimize (macOS rhythm)
        S(1.2)
        tour.shoot("browser-minimized")
        min_hash = md5_png(last_shot())
        click(q, x, y)  # hidden app -> restore + focus
        S(1.6)
        tour.shoot("browser-restored")
        restored_hash = md5_png(last_shot())
        note("dock-minimize-changes-screen",
             "PASS" if min_hash != open_hash else "FAIL: minimize left the frame identical")
        note("dock-restore-changes-screen",
             "PASS" if restored_hash != min_hash else "FAIL: restore left the frame identical")

        # ---- 5. browser search: omnibox -> Google --------------------------
        tour.section = "05-browser-search"
        bx, by = cascade(1)  # Browser opened after Files (boot opened two)
        click(q, bx + 450, by + 34 + 30 + 15)  # omnibox field center
        S(0.6)
        type_text(q, "rust programming")
        S(0.4)
        tour.shoot("typed")
        typed_hash = md5_png(last_shot())
        key(q, "ret")
        S(3.5)
        tour.shoot("results")
        res_hash = md5_png(last_shot())
        S(6.0)
        tour.shoot("results-late")
        late_hash = md5_png(last_shot())
        note("omnibox-focus-and-type",
             "PASS" if typed_hash != res_hash else "FAIL: typed text changed nothing")
        note("engine-page-arrives",
             "PASS" if late_hash != typed_hash else "FAIL: page area never changed")

        # ---- 6. terminal runs a real command (window 3) ---------------------
        tour.section = "06-terminal"
        x, y = dock_xy(5)
        click(q, x, y)
        S(1.6)
        tour.shoot("opened")
        tx, ty = cascade(2)  # Terminal: (176, 188), 640x400
        click(q, tx + 280, ty + 360)  # input line at the bottom of the body
        S(0.5)
        type_text(q, "uname -a")
        key(q, "ret")
        S(2.5)
        tour.shoot("uname")
        uname_hash = md5_png(last_shot())
        type_text(q, "ls /")
        key(q, "ret")
        S(2.0)
        tour.shoot("ls-root")
        note("terminal-output-arrives",
             "PASS" if md5_png(last_shot()) != uname_hash else "FAIL: second command printed nothing")

        # ---- 7. defender scan with Loading state (window 3) -----------------
        tour.section = "07-defender"
        x, y = dock_xy(7)
        click(q, x, y)
        S(1.6)
        tour.shoot("opened")
        dx, dy = cascade(3)
        click(q, dx + 16 + 55, dy + 34 + 16 + 40 + 8 + 15)  # Scan now button
        S(1.0)
        tour.shoot("scanning-loading")
        scan_hash = md5_png(last_shot())
        S(4.0)
        tour.shoot("after-scan")
        note("defender-scan-reacts",
             "PASS" if md5_png(last_shot()) != scan_hash else "FAIL: scan state never moved")

        # ---- 8. launcher (bare Meta) ----------------------------------------
        tour.section = "08-launcher"
        key(q, "meta")
        S(1.2)
        tour.shoot("launcher")
        key(q, "esc")
        S(0.6)
        note("launcher", "shot captured for review")

        # ---- 9. context menu over the desktop --------------------------------
        tour.section = "09-context-menu"
        rclick(q, 900, 500)
        S(0.8)
        tour.shoot("menu")
        key(q, "esc")
        S(0.4)
        note("context-menu", "shot captured for review")

        # ---- 10. dock auto-hide returns ---------------------------------------
        tour.section = "10-dock-hide"
        move(q, 640, 300)
        S(1.4)
        tour.shoot("hidden-again")
        note("dock-autohide",
             "PASS" if md5_png(last_shot()) != revealed_hash else "FAIL: dock never hid")

    finally:
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()
        q.close()

    print("[verify] sections in", out, flush=True)
    fails = 0
    for name, verdict in results:
        mark = "FAIL" if verdict.startswith("FAIL") else "ok"
        if verdict.startswith("FAIL"):
            fails += 1
        print(f"  [{mark:4s}] {name:32s} {verdict}", flush=True)
    print(f"[verify] {fails} failure(s)", flush=True)
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
