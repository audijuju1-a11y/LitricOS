#!/usr/bin/env python3
"""One boot, everything: persistence check, wizard click-through, app clicks."""

import os
import sys
import time

sys.path.insert(0, "/root/litricos-precision-v2/tools")
from photo_tour_lib import boot_qmp, exec_guest  # noqa: E402
from photo_tour_lib import click, key, type_text  # noqa: E402

OUT = "/root/litricos-precision-v2/build/e2e/qverify"
IMAGE = os.environ.get(
    "QDRIVE_IMAGE", "/root/litricos-precision-v2/build/out/litricos.raw")
RESULT = "/tmp/qverify.out"
SCREEN_W, SCREEN_H = 1920, 1080
LIB_W, LIB_H = 1280, 800

log = open(RESULT, "w", buffering=1)


def say(msg: str) -> None:
    print(msg, flush=True)
    log.write(msg + "\n")


def to_lib(px, py):
    return (round(px * (LIB_W - 1) / (SCREEN_W - 1)),
            round(py * (LIB_H - 1) / (SCREEN_H - 1)))


def dump(q, name):
    ppm = os.path.join(OUT, f"{name}.ppm")
    q.cmd("screendump", filename=ppm)
    png = ppm[:-4] + ".png"
    os.system(f"python3 /root/litricos-precision-v2/tools/ppm2png.py "
              f"{ppm} {png} >/dev/null 2>&1")
    say(f"QVERIFY_DUMP={png}")


proc, q, serial, diag = boot_qmp(OUT, image=IMAGE)
try:
    deadline = time.monotonic() + 360
    ready = False
    while time.monotonic() < deadline:
        try:
            if "READY" in exec_guest("echo READY", diag, timeout=8):
                ready = True
                break
        except Exception:
            pass
        time.sleep(3)
    say(f"diag_ready={ready}")
    if not ready:
        raise SystemExit("diagnostic shell never answered")

    # Did anything from the previous boots survive on /etc and /var?
    say("=== PERSISTENCE CHECK")
    say(exec_guest(
        "findmnt -no SOURCE,FSTYPE,OPTIONS /var; "
        "ls -la /var/lib/litric/etc-overlay/ 2>&1 | head -8; "
        "ls -la /etc/qw.txt 2>&1; "
        "grep ^root /etc/shadow | cut -c1-30",
        diag, timeout=30))

    time.sleep(4)  # let the wizard paint
    dump(q, "00-start")
    for step in [
        "click:1205,736",   # Get started
        "click:1205,736",   # Next (language)
        "click:960,480",    # focus password
        "type:litric123",
        "click:1205,736",   # Next -> confirm
        "click:960,480",    # focus confirm
        "type:litric123",
        "click:1205,736",   # Set password
    ]:
        kind, _, arg = step.partition(":")
        if kind == "click":
            click(q, *to_lib(*map(int, arg.split(","))))
        elif kind == "type":
            type_text(q, arg)
        time.sleep(1.2)
    time.sleep(3)
    dump(q, "10-after-wizard")

    # Click the dock's app icons: Files, Terminal, Settings.
    for name, x in [("20-files", 725), ("30-terminal", 896), ("40-settings", 987)]:
        click(q, *to_lib(x, 987))
        time.sleep(3.0)
        dump(q, name)

    say("=== ACCOUNT STATE AFTER WIZARD")
    say(exec_guest("grep ^root /etc/shadow | cut -c1-30", diag, timeout=20))
finally:
    try:
        q.cmd("quit")
    except Exception:
        pass
    q.close()
    try:
        proc.wait(timeout=10)
    except Exception:
        proc.kill()
