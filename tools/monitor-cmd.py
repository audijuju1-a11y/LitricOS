#!/usr/bin/env python3
"""Send one QEMU monitor command through the show-desktop monitor socket.

Usage: monitor-cmd.py <command> [args...]

Designed for `wsl.exe -d archlinux -e python3 /mnt/c/.../monitor-cmd.py screendump <path>`.
"""
import socket
import sys
import time

SOCK = "/root/litricos/build/show-monitor.sock"


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: monitor-cmd.py <command> [args...]", file=sys.stderr)
        return 2
    cmd = " ".join(sys.argv[1:])
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK)
    time.sleep(0.5)
    try:
        s.recv(4096)  # banner
    except OSError:
        pass
    s.sendall((cmd + "\n").encode())
    time.sleep(2.0)
    s.setblocking(False)
    out = b""
    try:
        while True:
            chunk = s.recv(4096)
            if not chunk:
                break
            out += chunk
    except BlockingIOError:
        pass
    print(out.decode(errors="replace"))
    s.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
