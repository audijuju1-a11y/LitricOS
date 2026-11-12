#!/usr/bin/env python3
"""Step-driven QMP driver: boots the image and performs scripted UI actions.

Coordinates given on the command line are true screen pixels of the guest
display (1920x1080); they are converted to the usb-tablet client range the
same way photo_tour_lib.move does, but against the real framebuffer size.

Steps (each is followed by an automatic screendump):
  click:X,Y   left click at pixel X,Y
  rclick:X,Y  right click
  key:NAME    press a key name from photo_tour_lib.QCODES
  type:TEXT   type text
  wait:SECS   sleep
  dump:NAME   extra screendump with a chosen name

    python3 /tmp/qdrive.py click:1205,736 dump:wiz-after-get-started ...
"""

import os
import sys
import time

sys.path.insert(0, "/root/litricos-precision-v2/tools")
from photo_tour_lib import boot_qmp, Qmp  # noqa: E402
from photo_tour_lib import click, rclick, key, type_text  # noqa: E402

SCREEN_W, SCREEN_H = 1920, 1080
LIB_W, LIB_H = 1280, 800  # photo_tour_lib.move's assumed space

OUT = sys.argv[1]
STEPS = sys.argv[2:]
BOOT_WAIT = float(os.environ.get("QDRIVE_BOOT_WAIT", "32"))
IMAGE = os.environ.get(
    "QDRIVE_IMAGE", "/root/litricos-precision-v2/build/out/litricos.raw")


def to_lib(px: int, py: int) -> tuple[int, int]:
    return (round(px * (LIB_W - 1) / (SCREEN_W - 1)),
            round(py * (LIB_H - 1) / (SCREEN_H - 1)))


def dump(q: Qmp, name: str) -> None:
    ppm = os.path.join(OUT, f"{name}.ppm")
    q.cmd("screendump", filename=ppm)
    png = ppm[:-4] + ".png"
    os.system(f"python3 /root/litricos-precision-v2/tools/ppm2png.py "
              f"{ppm} {png} >/dev/null 2>&1")
    print(f"QDRIVE_DUMP={png}", flush=True)


def main() -> int:
    proc, q, serial, diag = boot_qmp(OUT, image=IMAGE)
    print(f"QDRIVE_SERIAL={serial}", flush=True)
    try:
        time.sleep(BOOT_WAIT)
        dump(q, "00-start")
        for i, step in enumerate(STEPS, 1):
            kind, _, arg = step.partition(":")
            try:
                if kind == "click":
                    x, y = map(int, arg.split(","))
                    click(q, *to_lib(x, y))
                elif kind == "rclick":
                    x, y = map(int, arg.split(","))
                    rclick(q, *to_lib(x, y))
                elif kind == "key":
                    key(q, arg)
                elif kind == "type":
                    type_text(q, arg)
                elif kind == "wait":
                    time.sleep(float(arg))
                elif kind == "dump":
                    dump(q, f"{i:02d}-{arg}")
                    continue
                else:
                    print(f"QDRIVE_BAD_STEP={step}", flush=True)
                    continue
            except Exception as exc:  # keep going; dumps show the fallout
                print(f"QDRIVE_STEP_ERROR {step}: {exc}", flush=True)
            time.sleep(1.5)
            dump(q, f"{i:02d}-{kind}")
        return 0
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


if __name__ == "__main__":
    raise SystemExit(main())
