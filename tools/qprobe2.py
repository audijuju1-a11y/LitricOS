#!/usr/bin/env python3
"""Reproduce the wizard's chpasswd failure inside litric-sysd's mount namespace."""

import os
import sys
import time

sys.path.insert(0, "/root/litricos-precision-v2/tools")
from photo_tour_lib import boot_qmp, exec_guest  # noqa: E402

OUT = "/root/litricos-precision-v2/build/e2e/qprobe-nsenter"
IMAGE = os.environ.get(
    "QDRIVE_IMAGE", "/root/litricos-precision-v2/build/out/litricos.raw")
RESULT = "/tmp/qprobe2.out"

CMDS = [
    # Inside litric-sysd's namespace, /etc is read-only and chpasswd fails with
    # "Authentication token lock busy": it cannot create /etc/.pwd.lock. Open
    # exactly /etc read-write inside the namespace and retry, then look at what
    # the update path touched: a surviving /etc/shadow+ means shadow's
    # temp-file+rename protocol and the fix needs directory write; only the lock
    # file and an in-place /etc/shadow edit means two file binds are enough.
    'nsenter -t $(systemctl show --value -p MainPID litric-sysd) -m -- '
    '/usr/bin/sh -c "mount -o remount,rw /etc; '
    'echo root:litric123 | /usr/sbin/chpasswd; echo rc=$?; '
    'ls -la /etc/.pwd.lock /etc/shadow+ 2>&1"',
    # Outside the namespace: what did the successful update leave behind?
    'ls -la /etc/.pwd.lock /etc/shadow+ 2>&1; '
    'stat -c "%y %s" /etc/shadow',
]

log = open(RESULT, "w", buffering=1)


def say(msg: str) -> None:
    print(msg, flush=True)
    log.write(msg + "\n")


say(f"accel={'kvm' if os.access('/dev/kvm', os.W_OK) else 'tcg'}")
proc, q, serial, diag = boot_qmp(OUT, image=IMAGE)
try:
    # Poll for a usable diagnostic shell instead of a fixed sleep: a TCG
    # fallback can take minutes to boot and a blind timer types into nothing.
    deadline = time.monotonic() + float(os.environ.get("QDRIVE_BOOT_WAIT", "420"))
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
    for c in CMDS:
        say(f"=== CMD: {c}")
        try:
            say(exec_guest(c, diag, timeout=30))
        except Exception as exc:
            say(f"=== ERR {type(exc).__name__}: {exc}")
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
