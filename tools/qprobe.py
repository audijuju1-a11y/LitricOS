#!/usr/bin/env python3
"""Boot the image and run guest commands on the diagnostic console."""

import os
import sys
import time

sys.path.insert(0, "/root/litricos-precision-v2/tools")
from photo_tour_lib import boot_qmp, exec_guest  # noqa: E402

OUT = sys.argv[1]
CMDS = sys.argv[2:]
IMAGE = os.environ.get(
    "QDRIVE_IMAGE", "/root/litricos-precision-v2/build/out/litricos.raw")

proc, q, serial, diag = boot_qmp(OUT, image=IMAGE)
try:
    time.sleep(float(os.environ.get("QDRIVE_BOOT_WAIT", "32")))
    for c in CMDS:
        print(f"=== CMD: {c}", flush=True)
        try:
            print(exec_guest(c, diag, timeout=30), flush=True)
        except Exception as exc:
            print(f"=== ERR {type(exc).__name__}: {exc}", flush=True)
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
