#!/usr/bin/env python3
"""LitricOS — run a command on the guest's interactive console and print it.

WHY THIS EXISTS
---------------
The image has two serial ports and they are not the same console:

* **ttyS0** is the kernel and systemd log. `console=tty0 console=ttyS0,115200`
  is in `image/mkosi.conf`, and it is where every boot message goes.
* **ttyS1** is the *interactive* console. `serial-getty@ttyS1` is enabled in
  `image/mkosi.postinst.chroot` and gets an autologin root shell from
  `image/mkosi.extra/usr/lib/systemd/system/serial-getty@ttyS1.service.d/`.

Both `tests/smoke_boot.py` and `tools/boot-test.sh` attach QEMU's first serial
port (`-serial mon:stdio`) — that is ttyS0 — and then wait for a shell prompt
that no getty ever writes there. Measured 2026-10-11: a 300 s run reached
userspace and 78 targets with no prompt, and the prompt was waiting on ttyS1 the
whole time. This probe attaches to **ttyS1**, which is where a prompt is.

It is also the only way to ask the running image a question. The root filesystem
is erofs plus dm-verity and root's password is locked, so nothing can be run
"in" the image from the host except through this console.

USAGE
-----
    python3 tools/console-probe.py --image build/boot/litricos.raw \\
        --command "litric-sysd --report"

    # Just the default fact query, which is what the rollout checks read:
    python3 tools/console-probe.py --image build/boot/litricos.raw

Boots QEMU itself unless `--no-boot` is given (then it attaches to an existing
socket). The transcript is written to stdout and to `--out` when given.
"""

from __future__ import annotations

import argparse
import os
import shlex
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

# The prompt the autologin shell actually prints. Measured, not guessed:
# Arch's /etc/bash.bashrc sets `PS1='[\u@\h \W]\$ '`, so root on this image
# prints `[root@litricos ~]# ` -- with a leading bracket, which the anchored
# patterns in tests/smoke_boot.py do not match.
PROMPT = "[root@litricos ~]#"

# The command to run when none is given: the subsystem facts this wave added,
# read from the running machine rather than from a fixture.
DEFAULT_COMMAND = (
    "litric-sysd --report 2>&1 | "
    "grep -E '^(net\\.|bt\\.|audio\\.|power\\.|thermal\\.|session\\.|auth\\.|litric\\.root\\.)' | head -60"
)

FIRMWARE_CODE = [
    Path("/usr/share/edk2/x64/OVMF_CODE.4m.fd"),
    Path("/usr/share/OVMF/OVMF_CODE.fd"),
]


def discover_firmware() -> Path:
    for candidate in FIRMWARE_CODE:
        if candidate.exists():
            return candidate
    raise SystemExit("no OVMF firmware found; pass --firmware-code")


def start_qemu(args: argparse.Namespace) -> subprocess.Popen:
    """Boot the image with ttyS0 to a log file and ttyS1 to a unix socket."""
    socket_path = Path(args.socket)
    if socket_path.exists():
        socket_path.unlink()
    socket_path.parent.mkdir(parents=True, exist_ok=True)
    vars_path = Path(args.vars)
    if not vars_path.exists():
        raise SystemExit(f"firmware variable store not found: {vars_path}")

    accel = "kvm" if args.accel == "kvm" else "tcg"
    # kernel-irqchip=off is what tests/smoke_boot.py passes with TCG; KVM
    # rejects it, so it is only added for TCG.
    machine = "q35,accel=kvm" if accel == "kvm" else "q35,accel=tcg,kernel-irqchip=off"
    cpu = "host" if accel == "kvm" else "max"
    cmd = [
        args.qemu,
        "-machine", machine,
        "-cpu", cpu,
        "-smp", str(args.smp),
        "-m", args.mem,
        "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={discover_firmware()}",
        "-drive", f"if=pflash,format=raw,unit=1,file={vars_path}",
        "-drive", f"if=virtio,format=raw,file={args.image}",
        "-device", "virtio-net-pci,netdev=n0",
        "-netdev", "user,id=n0",
        "-display", "none",
        # ttyS0: the log, to a file. ttyS1: the console, to a socket we type on.
        "-serial", f"file:{args.log}",
        "-chardev", f"socket,id=cons1,path={socket_path},server=on,wait=off",
        "-serial", "chardev:cons1",
        "-no-reboot",
    ]
    print(f"[probe] {' '.join(shlex.quote(c) for c in cmd)}", file=sys.stderr)
    return subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)


def connect(path: str, timeout: float) -> socket.socket:
    """Connect to QEMU's console socket, retrying until it is listening."""
    deadline = time.time() + timeout
    last = None
    while time.time() < deadline:
        try:
            s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            s.settimeout(2.0)
            s.connect(path)
            return s
        except OSError as exc:  # not listening yet, or gone
            last = exc
            time.sleep(0.5)
    raise SystemExit(f"console socket never became reachable: {last}")


def read_until(sock: socket.socket, needle: str, deadline: float) -> str:
    """Read until `needle` appears or the deadline passes. Returns what came."""
    buf = ""
    while time.time() < deadline:
        try:
            chunk = sock.recv(4096)
        except socket.timeout:
            continue
        except OSError:
            break
        if not chunk:
            break
        buf += chunk.decode("utf-8", "replace")
        if needle in buf:
            break
    return buf


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--image", default="build/boot/litricos.raw")
    p.add_argument("--socket", default="build/boot/console.sock")
    p.add_argument("--log", default="build/boot/console-s0.log")
    p.add_argument("--vars", default="build/OVMF_VARS.smoke.fd")
    p.add_argument("--qemu", default=os.environ.get("LITRIC_QEMU", "qemu-system-x86_64"))
    p.add_argument("--accel", choices=["kvm", "tcg"], default="kvm")
    p.add_argument("--mem", default="3G")
    p.add_argument("--smp", type=int, default=4)
    p.add_argument("--timeout", type=float, default=180.0, help="seconds to wait for a prompt")
    p.add_argument("--command", default=DEFAULT_COMMAND)
    p.add_argument("--out", default="")
    p.add_argument("--no-boot", action="store_true", help="attach to an existing socket")
    args = p.parse_args()

    proc = None
    if not args.no_boot:
        proc = start_qemu(args)
        # A boot that dies takes the socket with it; stop waiting on a corpse.
        signal.signal(signal.SIGINT, lambda *_: proc.kill() or sys.exit(130))

    try:
        sock = connect(args.socket, 30.0)
        deadline = time.time() + args.timeout
        boot = read_until(sock, PROMPT, deadline)
        if PROMPT not in boot:
            print(boot[-2000:], file=sys.stderr)
            print(
                f"[probe] no prompt within {args.timeout:.0f}s. The console is ttyS1; "
                "if this is a timeout the guest may still be booting (TCG is slow).",
                file=sys.stderr,
            )
            return 1

        sock.sendall(f"{args.command}\n".encode())
        # Echo of the command, then its output, then the prompt again.
        out = read_until(sock, PROMPT, time.time() + 60.0)
        text = out
        print(text)
        if args.out:
            Path(args.out).write_text(text)
        return 0
    finally:
        if proc is not None:
            proc.terminate()
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    sys.exit(main())
