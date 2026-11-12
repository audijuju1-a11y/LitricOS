#!/usr/bin/env python3
"""Granular write tests inside litric-sysd's namespace.

Separates the layers the password write touches: plain overlay write, file
create, rename, the PAM path (chpasswd), the non-PAM path (usermod -p), the
overlay's upper directory on /var, and the namespace's effective caps.
"""

import os
import sys
import time

sys.path.insert(0, "/root/litricos-precision-v2/tools")
from photo_tour_lib import boot_qmp, exec_guest  # noqa: E402

OUT = "/root/litricos-precision-v2/build/e2e/qprobe-layers"
IMAGE = os.environ.get(
    "QDRIVE_IMAGE", "/root/litricos-precision-v2/build/out/litricos.raw")
RESULT = "/tmp/qprobe4.out"

NS = ('nsenter -t $(systemctl show --value -p MainPID litric-sysd) -m -- '
      '/usr/bin/sh -c ')

INNER = (
    'mount -o remount,rw,bind /etc; '
    'cat /proc/self/status | grep ^Cap; '
    'echo probe > /etc/qw.txt; echo plain_write_rc=$?; '
    'cp /etc/shadow /etc/shadow.bak; echo create_rc=$?; '
    'mv /etc/shadow.bak /etc/shadow.bak2; echo rename_rc=$?; '
    'touch /var/lib/litric/etc-overlay/upper/qw2; echo upper_write_rc=$?; '
    'usermod -p lockedhash root; echo usermod_rc=$?; '
    'echo root:litric123 | /usr/sbin/chpasswd; echo chpasswd_rc=$?; '
    'ls -la /etc/shadow+ /etc/.pwd.lock 2>&1'
)

CMDS = [
    # Outside the namespace as the control.
    'echo probe > /etc/qw.txt; echo outside_plain_rc=$?; '
    'usermod -p lockedhash root; echo outside_usermod_rc=$?',
    NS + '"' + INNER + '"',
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
