#!/usr/bin/env python3
"""Send one command to a QEMU human monitor over a Unix socket.

Used by tools/screen-test.sh to ask QEMU for a screenshot of the guest's
display. socat and netcat would both do this and both are optional packages; a
screenshot tool that silently does nothing because netcat is not installed is
precisely the failure the screenshot exists to rule out, so this uses the
Python that is already required to convert the image.

Usage::

    python3 tools/qemu-monitor.py /path/to/monitor.sock "screendump /tmp/s.ppm"
"""

from __future__ import annotations

import socket
import sys
import time


def main() -> int:
    if len(sys.argv) != 3:
        sys.stderr.write(f"usage: {sys.argv[0]} <monitor.sock> <command>\n")
        return 2
    path, command = sys.argv[1], sys.argv[2]

    try:
        sock = socket.socket(socket.AF_UNIX)
        sock.settimeout(15)
        sock.connect(path)
    except OSError as exc:
        sys.stderr.write(f"cannot reach the QEMU monitor at {path}: {exc}\n")
        return 1

    with sock:
        time.sleep(0.2)
        try:
            sock.recv(65536)  # the monitor banner and prompt
        except OSError:
            pass

        sock.sendall((command + chr(10)).encode())

        # screendump writes the whole framebuffer to disk before it answers, so
        # the reply is the signal that the file is complete. Waiting a fixed
        # time instead would race a large display.
        deadline = time.time() + 15
        reply = b""
        while time.time() < deadline:
            try:
                chunk = sock.recv(65536)
            except socket.timeout:
                break
            except OSError:
                break
            if not chunk:
                break
            reply += chunk
            if b"(qemu)" in reply:
                break
        text = reply.decode(errors="replace").strip()
        if text:
            print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
