#!/usr/bin/env python3
"""Decide the exact write scope chpasswd needs inside litric-sysd's namespace.

Variant A mimics ReadWritePaths=/etc/shadow /etc/.pwd.lock (file binds).
Variant B mimics ReadWritePaths=/etc (directory bind-remount).
"""

import os
import sys
import time

sys.path.insert(0, "/root/litricos-precision-v2/tools")
from photo_tour_lib import boot_qmp, exec_guest  # noqa: E402

OUT = "/root/litricos-precision-v2/build/e2e/qprobe-scope"
IMAGE = os.environ.get(
    "QDRIVE_IMAGE", "/root/litricos-precision-v2/build/out/litricos.raw")
RESULT = "/tmp/qprobe3.out"

NS = ('nsenter -t $(systemctl show --value -p MainPID litric-sysd) -m -- '
      '/usr/bin/sh -c ')

CMDS = [
    # Where does /etc's overlay write to -- does anything on /etc survive a
    # reboot at all?
    'mount | grep -E " /etc| / " ; echo ---',
    # Outside the namespace: a plain chpasswd, and does /etc/shadow change?
    'stat -c "before %y %s" /etc/shadow; touch /etc/.pwd.lock; chmod 600 '
    '/etc/.pwd.lock; echo root:litric123 | /usr/sbin/chpasswd; '
    'echo rc=$?; stat -c "after %y %s" /etc/shadow',
    # Variant A: only the lock file and the shadow file writable.
    NS + '"mount --bind /etc/.pwd.lock /etc/.pwd.lock; '
         'mount -o remount,rw,bind /etc/.pwd.lock; '
         'mount --bind /etc/shadow /etc/shadow; '
         'mount -o remount,rw,bind /etc/shadow; '
         'echo root:litric123 | /usr/sbin/chpasswd; echo A_rc=$?; '
         'ls -la /etc/shadow+ 2>&1"',
    # Variant B: the whole /etc directory writable (old bind semantics).
    NS + '"mount -o remount,rw,bind /etc; '
         'echo root:litric123 | /usr/sbin/chpasswd; echo B_rc=$?; '
         'ls -la /etc/.pwd.lock /etc/shadow+ 2>&1"',
]

log = open(RESULT, "w", buffering=1)


def say(msg: str) -> None:
    print(msg, flush=True)
    log.write(msg + "\n")


proc, q, serial, diag = boot_qmp(OUT, image=IMAGE)
try:
    deadline = time.monotonic() + float(os.environ.get("QDRIVE_BOOT_WAIT", "360"))
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
            say(exec_guest(c, diag, timeout=45))
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
