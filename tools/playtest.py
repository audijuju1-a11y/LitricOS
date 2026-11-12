#!/usr/bin/env python3
"""Playtest LitricOS the way its first real user would — on an image COPY.

Never touches a VM the user has open: boots build/playtest.raw (a copy),
drives the real input path (QMP abs/btn/key -> evdev), and confirms every
step from inside the guest (serial diagnostics + ttyS1 console), never from
assumption. Clicks are computed from the shipped geometry (dock formula,
window frames from `window=` diagnostics, chrome buttons from
Chrome::default), so a hit is proven, not hoped.

The shell re-prints its diagnostic dump every ~5s, so every assertion POLLS
until a condition shows up in a fresh dump (or a timeout names the stage) —
a fixed sleep reads the previous dump and lies about the result, which is
exactly what the first playtest run did.
"""

import hashlib
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from photo_tour_lib import Qmp, boot_qmp, click, exec_guest, key, move, type_text  # noqa: E402

BUILD = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "build"))
SRC_ZST = os.path.join(BUILD, "out", "litricos.raw.zst")
PLAY_IMG = os.path.join(BUILD, "playtest.raw")

SCREEN_W, SCREEN_H = 1280, 800
TOP_BAR = 34
TITLE_H = 34
# Dock, from draw_dock(): item=52 pad=10 h=56, centered, 8 apps.
DOCK_ITEM, DOCK_PAD, DOCK_H = 52, 10, 56
DOCK_X = (SCREEN_W - (DOCK_ITEM * 8 + DOCK_PAD * 2)) / 2 + DOCK_PAD  # first icon x
DOCK_Y = SCREEN_H - DOCK_H / 2  # safe center row
APP_INDEX = {"activity": 0, "env": 1, "settings": 2, "files": 3,
             "browser": 4, "terminal": 5, "trash": 6, "defender": 7}


def dock_xy(app: str) -> tuple[int, int]:
    return (int(DOCK_X + APP_INDEX[app] * DOCK_ITEM + DOCK_ITEM / 2), int(DOCK_Y))


def drag(q: Qmp, x0: int, y0: int, x1: int, y1: int, steps: int = 8) -> None:
    move(q, x0, y0)
    time.sleep(0.15)
    q.cmd("input-send-event", events=[{"type": "btn", "data": {"down": True, "button": "left"}}])
    for i in range(1, steps + 1):
        move(q, int(x0 + (x1 - x0) * i / steps), int(y0 + (y1 - y0) * i / steps))
        time.sleep(0.05)
    time.sleep(0.15)
    q.cmd("input-send-event", events=[{"type": "btn", "data": {"down": False, "button": "left"}}])
    time.sleep(0.4)


def parse_windows(lines: list[str]) -> dict[str, tuple[int, int, int, int, str]]:
    """`...litric.shell.window=#id app "title" WxH+X+Y state wsN` -> {app: geo}.

    Titles contain spaces, so parsing goes from the right: state and
    workspace sit at the tail, geometry right before them. Newest wins.
    """
    out: dict[str, tuple[int, int, int, int, str]] = {}
    for ln in lines:
        i = ln.find(".window=#")
        if i == -1:
            continue
        try:
            parts = ln[i + 1:].split()
            app = parts[1]
            geo = parts[3]
            wh, xy = geo.split("+")[0], geo.split("+")[1:3]
            w, h = map(int, wh.lower().split("x"))
            x, y = int(xy[0]), int(xy[1])
            state = parts[4] if len(parts) > 4 else "?"
            out[app] = (x, y, w, h, state)
        except (ValueError, IndexError):
            continue
    return out


def main() -> int:
    stamp = time.strftime("%Y%m%d-%H%M%S") + "-play"
    out = os.path.join(BUILD, "tour", stamp)
    os.makedirs(out, exist_ok=True)
    fails: list[str] = []

    # The contract: play on a copy, never the user's image. Always fresh so a
    # rebuilt image is what gets exercised.
    if os.path.exists(PLAY_IMG):
        os.remove(PLAY_IMG)
    subprocess.run(["zstd", "-d", "-f", SRC_ZST, "-o", PLAY_IMG], check=True)

    def check(name: str, ok: bool, detail: str = "") -> None:
        print(f"[{'PASS' if ok else 'FAIL'}] {name}" + (f" — {detail}" if detail else ""))
        if not ok:
            fails.append(name)

    def guest(sock: str, log: str, n: int, cmd: str, timeout: float = 50.0) -> list[str]:
        """exec in guest; the serial LOG is the ground truth, not the socket echo."""
        exec_guest(f"echo MK{n}S; {cmd}; echo MK{n}E", sock, timeout=timeout)
        time.sleep(2.0)
        text = open(log, encoding="utf-8", errors="replace").read()
        start = text.rfind(f"MK{n}S")
        end = text.rfind(f"MK{n}E")
        if start == -1 or end == -1 or end <= start:
            return []
        return [ln.strip() for ln in text[start + len(f"MK{n}S"):end].splitlines() if ln.strip()]

    proc, q, log, diag_sock = boot_qmp(out, PLAY_IMG)
    try:
        def S(sec: float) -> None:
            time.sleep(sec)

        def diag() -> dict:
            """Newest-wins view of the shell's own journal over serial."""
            raw = open(log, encoding="utf-8", errors="replace").read()
            lines = [ln.strip() for ln in raw.splitlines() if "litric.shell." in ln]
            last: dict[str, str] = {}
            for ln in lines:
                body = ln.split("litric.shell.", 1)[1].strip()
                if "=" in body:
                    k, v = body.split("=", 1)
                    last[k] = v
            return {"last": last, "windows": parse_windows(lines)}

        def wait_for(name: str, pred, timeout: float = 16.0) -> dict:
            """Poll fresh dumps until `pred(diag)` holds; dump cadence is ~5s."""
            deadline = time.time() + timeout
            d = diag()
            while time.time() < deadline:
                if pred(d):
                    return d
                S(0.8)
                d = diag()
            return d  # caller's check() names the failure

        print("[boot] settling 45s...")
        S(14); S(15); S(16)

        # --- stage 0: guest is alive; input pipeline truth ---
        g0 = guest(diag_sock, log, 1, "echo alive; grep -c . /proc/uptime")
        check("guest console answers", any("alive" in ln for ln in g0), "; ".join(g0[:2]))
        d = diag()
        absmax = d["last"].get("input.absmax", "?")
        check("device abs range learned", absmax in ("65535x65535", "16383x16383"),
              f"input.absmax={absmax}")

        # --- stage 1: pointer truth across the screen ---
        for px, py in ((100, 300), (640, 360), (1000, 600)):
            move(q, px, py); S(0.8)
        d = wait_for("pointer", lambda d: d["last"].get("pointer", "?") != "?")
        got = d["last"].get("pointer", "?")
        gx, gy = (int(v) for v in got.split(",")) if "," in got else (-1, -1)
        check("pointer lands where aimed", abs(gx - 1000) < 24 and abs(gy - 600) < 24, f"pointer={got}")

        # --- stage 2: dock reveal + open Browser ---
        move(q, 640, SCREEN_H - 8); S(1.6)
        bx, by = dock_xy("browser")
        click(q, bx, by)
        d = wait_for("browser window", lambda d: "com.litric.browser" in d["windows"])
        bwin = d["windows"].get("com.litric.browser")
        check("dock click opens Browser", bwin is not None, str(list(d["windows"])))
        if not bwin:
            raise SystemExit("cannot continue without a browser window")
        wx, wy, ww, wh, _st = bwin
        # Omnibox field center: title(34) + tabstrip(30) + bar(30) -> field 4..26
        omni = (wx + 180, wy + TITLE_H + 30 + 15)

        # --- stage 3: type a real search and commit ---
        click(q, *omni); S(0.8)
        type_text(q, "rust programming"); S(0.4)
        key(q, "ret")
        d = wait_for("google url", lambda d: "google.com/search" in d["last"].get("browser.tab", ""))
        tab = d["last"].get("browser.tab", "none")
        check("omnibox commits to Google URL", "google.com/search" in tab, f"tab={tab}")
        d = wait_for("engine frame", lambda d: d["last"].get("browser.frame", "")
                     not in ("", "none", "0 bytes"), timeout=30.0)
        status = d["last"].get("browser.status", "?")
        frame = d["last"].get("browser.frame", "none")
        check("engine session alive", ("error" not in status.lower() and "end" not in status.lower()),
              f"status={status}")
        check("engine delivered a frame", frame not in ("none", "", "0 bytes"), f"frame={frame}")
        q.cmd("screendump", filename=os.path.join(out, "search-results.ppm")); S(0.5)

        # --- stage 4: Files + Defender from the dock ---
        fx, fy = dock_xy("files")
        move(q, 640, SCREEN_H - 8); S(1.2); click(q, fx, fy)
        d = wait_for("files window", lambda d: "com.litric.files" in d["windows"])
        check("Files opens", "com.litric.files" in d["windows"], str(list(d["windows"])))
        dx_, dy_ = dock_xy("defender")
        move(q, 640, SCREEN_H - 8); S(1.2); click(q, dx_, dy_)
        d = wait_for("defender window", lambda d: "com.litric.defender" in d["windows"])
        check("Defender opens", "com.litric.defender" in d["windows"], str(list(d["windows"])))

        # --- stage 5: drag the Activity window by its title bar, then close it ---
        d = diag()
        awin = d["windows"].get("com.litric.activity")
        check("boot windows still present (Activity)", awin is not None, str(list(d["windows"])))
        if awin:
            ax, ay, aw, ah, _s0 = awin
            tx, ty = ax + aw // 2, ay + TITLE_H // 2
            drag(q, tx, ty, tx + 120, ty + 60)
            d = wait_for("activity moved", lambda d: _moved(d, ax, ay))
            n2 = d["windows"].get("com.litric.activity")
            check("title-bar drag moves window", bool(_moved(d, ax, ay)),
                  f"{(ax, ay)} -> {n2 and (n2[0], n2[1])}")
            # close it: button 0 center = (x+22, y+17) from Chrome::default
            click(q, ax + 22, ay + 17)
            d = wait_for("activity closed", lambda d: "com.litric.activity" not in d["windows"])
            check("close button closes window", "com.litric.activity" not in d["windows"],
                  str(list(d["windows"])))

        # --- stage 6: careless input must not wedge the shell ---
        click(q, 640, 400); S(0.4)                      # empty desktop
        type_text(q, "zzz")                             # typing with no text field
        move(q, 640, SCREEN_H - 8); S(1.0)
        click(q, bx, by); S(0.3); click(q, bx, by); S(0.3); click(q, bx, by); S(2.0)  # rapid triple
        f0 = int(diag()["last"].get("frames", "0"))
        d = wait_for("frames advance", lambda d: int(d["last"].get("frames", "0")) > f0, timeout=12.0)
        f1 = int(d["last"].get("frames", "0"))
        check("shell keeps rendering after abuse", f1 > f0, f"frames {f0} -> {f1}")
        check("browser survived abuse", "com.litric.browser" in d["windows"], str(list(d["windows"])))

        # --- stage 7: guest-side sanity after everything ---
        g = guest(diag_sock, log, 2,
                  "systemctl is-failed --quiet litric-shell.service && echo SHELL-FAILED || echo SHELL-OK;"
                  "systemctl is-active litric-browser.socket; ip -4 -br addr | grep -c UP")
        print("[guest:final]", " | ".join(g))
        check("no failed shell unit", "SHELL-FAILED" not in " ".join(g), " | ".join(g))
    finally:
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()
        q.close()

    print(f"\n[artifacts] {out}")
    print(f"[result] {'ALL PASS' if not fails else 'FAILURES: ' + ', '.join(fails)}")
    return 0 if not fails else 1


def _moved(d: dict, ax: int, ay: int) -> bool:
    n = d["windows"].get("com.litric.activity")
    return bool(n and (abs(n[0] - ax) > 60 or abs(n[1] - ay) > 40))


if __name__ == "__main__":
    raise SystemExit(main())
