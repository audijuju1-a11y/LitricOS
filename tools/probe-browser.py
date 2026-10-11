#!/usr/bin/env python3
"""Probe: does a dock click open the Browser, and does the engine answer?

Staged so a failure names its stage instead of failing at the end:
  1. boot, settle
  2. rest pointer on the dock edge, wait for the slide, click Browser
  3. screendump -> window present?
  4. click omnibox, type, Enter
  5. screendump -> page changed? serial says engine status.
"""

import hashlib
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from photo_tour_lib import Qmp, boot_qmp, click, exec_guest, key, move, type_text  # noqa: E402


def md5(path: str) -> str:
    with open(path, "rb") as f:
        return hashlib.md5(f.read()).hexdigest()


def main() -> None:
    out = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "build", "tour",
        time.strftime("%Y%m%d-%H%M%S") + "-probe",
    )
    os.makedirs(out, exist_ok=True)
    proc, q, serial_path, diag = boot_qmp(out)

    def S(s: float) -> None:
        time.sleep(s)

    def shot(name: str) -> str:
        ppm = os.path.join(out, f"{name}.ppm")
        q.cmd("screendump", filename=ppm)
        S(0.5)
        return md5(ppm)

    try:
        S(14); S(15); S(16)
        d0 = shot("boot")
        print("boot hash", d0)

        # Dock reveal: rest on the edge LONGER than the spring, then click.
        move(q, 666, 782)
        S(1.5)
        click(q, 666, 776)
        S(2.5)
        d1 = shot("after-dock-click")
        print("after-dock hash", d1, "CHANGED" if d1 != d0 else "UNCHANGED (dock click missed?)")

        # Omnibox: window 3 -> cascade(0) = (120, 132); field center y = 211.
        click(q, 120 + 450, 211)
        S(0.8)
        type_text(q, "rust programming")
        S(0.4)
        key(q, "ret")
        S(6.0)
        d2 = shot("after-enter")
        print("after-enter hash", d2, "CHANGED" if d2 != d1 else "UNCHANGED")
        S(6.0)
        d3 = shot("after-enter-late")
        print("late hash", d3, "CHANGED" if d3 != d2 else "UNCHANGED")
    finally:
        # Ground truth from inside the guest: the diagnostic console on
        # ttyS1 answers why the broker unit died, in systemd's own words.
        if diag and os.path.exists(diag):
            for name, cmd in (
                ("unit-status", "systemctl status litric-browser@0.service -n 12 --no-pager 2>&1 | tail -14"),
                ("journal", "journalctl -u litric-browser@0.service -n 14 --no-pager 2>&1 | tail -14"),
                ("net", "ip -4 addr show; ip route; timeout 3 curl -sI https://www.google.com 2>&1 | head -2"),
            ):
                out_txt = exec_guest(cmd, diag, timeout=40.0)
                print(f"[guest:{name}]")
                for ln in out_txt.splitlines()[-16:]:
                    print("   ", ln[:170])
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()
        q.close()

    serial = os.path.join(out, "serial.log")
    if os.path.exists(serial):
        for line in open(serial, encoding="utf-8", errors="replace"):
            if any(k in line for k in ("browser.status=", "browser.tab=", "browser.frame=", "net.probe=")):
                print("[serial]", line.strip()[:160])


if __name__ == "__main__":
    main()
