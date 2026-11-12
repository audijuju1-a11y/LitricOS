#!/usr/bin/env python3
"""Log into a running LitricOS guest over its ttyS1 socket and run commands.

The guest's second serial port has a getty on it. When the launcher is started
with LITRIC_DEBUG_CONSOLE=1 that port is a unix socket instead of a file, so the
host can type at the login prompt and read what the guest says back. This is how
"does the guest really have a lease, a driver, or a unit running" is answered
with the guest's own output rather than an inference from the outside.

    tools/guest-console.py --user root --password litric -- "ip -br addr" "ip route"
    tools/guest-console.py --user litric --password litric -- "iwctl device list"

It does not touch the guest's state beyond the commands given to it.
"""

from __future__ import annotations

import argparse
import re
import socket
import sys
import time
from pathlib import Path

# CSI sequences, OSC sequences (systemd's shell integration prints OSC 3008
# with the session's machine id and cwd on every prompt), and lone escape bytes.
ANSI = re.compile(
    rb"\x1b\[[0-9;?]*[A-Za-z]"
    rb"|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)"
    rb"|\x1b[()][A-Za-z0-9]"
    rb"|\x1b[=>]"
)


class Console:
    def __init__(self, path: Path, timeout: float = 30.0) -> None:
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(timeout)
        self.sock.connect(str(path))
        self.buf = b""

    def read_until_count(self, pattern: str, count: int, timeout: float = 30.0) -> str:
        """Read until `pattern` has been seen `count` times.

        A terminal echoes what is typed, so the command line itself contains any
        marker the command prints -- the first occurrence is the echo and the
        second is the real output. Waiting for one would return before the
        command ran.
        """
        want = pattern.encode()
        deadline = time.time() + timeout
        while time.time() < deadline:
            if self.buf.count(want) >= count:
                idx = -1
                for _ in range(count):
                    idx = self.buf.find(want, idx + 1)
                text, self.buf = self.buf[: idx + len(want)], self.buf[idx + len(want) :]
                return clean(text)
            try:
                chunk = self.sock.recv(1 << 16)
            except TimeoutError:
                break
            if not chunk:
                break
            self.buf += chunk
        text, self.buf = self.buf, b""
        return clean(text)

    def read_until(self, pattern: str | bytes, timeout: float = 30.0) -> str:
        want = pattern.encode() if isinstance(pattern, str) else pattern
        deadline = time.time() + timeout
        while time.time() < deadline:
            if want in self.buf:
                text, self.buf = self.buf.split(want, 1)
                return clean(text + want)
            try:
                chunk = self.sock.recv(1 << 16)
            except TimeoutError:
                break
            if not chunk:
                break
            self.buf += chunk
        text, self.buf = self.buf, b""
        return clean(text)

    def write(self, data: str) -> None:
        self.sock.sendall(data.encode().replace(b"\n", b"\r\n"))

    def drain(self, timeout: float = 0.6) -> None:
        """Throw away whatever the terminal has already sent."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            try:
                chunk = self.sock.recv(1 << 16)
            except TimeoutError:
                break
            if not chunk:
                break
            self.buf += chunk
        self.buf = b""

    def command(self, cmd: str, marker: str, timeout: float = 30.0) -> str:
        """Run one command and return its output, bracketed by the marker.

        Echo is off by the time this runs (`main` turns it off), so the marker
        appears once, in the output.
        """
        self.write(f"echo {marker}B; {cmd} 2>&1; echo {marker}E$?\n")
        return self.read_until(f"{marker}E", timeout)


def clean(raw: bytes) -> str:
    text = ANSI.sub(b"", raw).decode("utf-8", "replace").replace("\r", "")
    return text


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sock", default="/tmp/litricos-ttyS1.sock")
    parser.add_argument("--user", default="root")
    parser.add_argument("--password", default="litric")
    parser.add_argument("--prompt-timeout", type=float, default=20.0)
    parser.add_argument("commands", nargs="*", help="commands to run; separated by --")
    args = parser.parse_args(argv)

    console = Console(Path(args.sock))
    # The socket is a real terminal: a session that is already logged in stays
    # logged in, and the connection may well attach to that prompt rather than
    # to a fresh login. Nothing here logs out; the session belongs to the guest.
    console.write("\n")
    state = console.read_until("# ", 2.0) or console.read_until("$ ", 1.0)
    if "# " in state or "$ " in state:
        print("[attached to an existing shell]")
    else:
        console.write("\n")
        pushed = console.read_until("login:", args.prompt_timeout)
        if "login:" not in pushed:
            print("guest-console: no login prompt seen. Last output:", file=sys.stderr)
            print(pushed[-800:], file=sys.stderr)
            return 1
        console.write(f"{args.user}\n")
        seen = console.read_until("assword:", args.prompt_timeout)
        if "assword:" not in seen:
            print("guest-console: no password prompt. Last output:", file=sys.stderr)
            print(seen[-800:], file=sys.stderr)
            return 1
        console.write(f"{args.password}\n")
        shell = console.read_until("# ", args.prompt_timeout) or console.read_until("$ ", args.prompt_timeout)
        if "# " not in shell and "$ " not in shell:
            print("guest-console: login did not reach a shell prompt:", file=sys.stderr)
            print(shell[-800:], file=sys.stderr)
            return 1
        print(f"[logged in as {args.user}]")

    # The getty is an 80x24 terminal and echoes what is typed, so a command
    # longer than the line wraps through the middle of its own output. Turning
    # echo off and asking for a wide terminal keeps the transcript readable.
    console.write("\x03")
    time.sleep(0.2)
    console.write("stty -echo -onlcr cols 400 2>/dev/null; echo READY\n")
    console.read_until("READY", 5.0)
    console.drain()

    rc = 0
    for i, cmd in enumerate(args.commands):
        marker = f"MC{i}J"
        raw = console.command(cmd, marker)
        # Drop the echoed command line and the two markers, keep the output.
        body = raw
        for token in (f"echo {marker}B; {cmd} 2>&1; echo {marker}E$?", f"{marker}B", f"{marker}E"):
            body = body.replace(token, "")
        text = "\n".join(line for line in body.splitlines() if line.strip()).strip()
        print(f"$ {cmd}")
        print(text)
        print()
        # The bytes after the second marker are the exit status, and only it.
        tail = console.read_until("\n", 2.0).strip()
        if tail and not tail.isdigit():
            console.read_until("\n", 0.5)
        elif tail.isdigit() and tail != "0":
            rc = 1
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
