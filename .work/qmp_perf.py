"""Drive the guest's pointer over QMP and read the shell's counters off the
serial console. Used to prove the frame path: a burst of absolute moves should
cost cursor-only frames, and opening the launcher should not overrun the budget.
"""

import json
import socket
import time

QMP = "/root/litric-latest/build/gui-qmp.sock"
SERIAL = "/root/litric-latest/build/gui-serial.sock"
W, H = 1920, 1080


class Qmp:
    def __init__(self):
        self.s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.s.connect(QMP)
        self.s.settimeout(10)
        self.f = self.s.makefile("rwb")
        self.f.readline()
        self.cmd("qmp_capabilities")

    def cmd(self, name, args=None):
        payload = {"execute": name}
        if args is not None:
            payload["arguments"] = args
        self.f.write((json.dumps(payload) + "\n").encode())
        self.f.flush()
        while True:
            line = self.f.readline()
            if b'"return"' in line or b'"error"' in line:
                return line

    def abs_move(self, x, y):
        self.cmd(
            "input-send-event",
            {
                "events": [
                    {"type": "abs", "data": {"axis": "x", "value": int(x * 65535 / W)}},
                    {"type": "abs", "data": {"axis": "y", "value": int(y * 65535 / H)}},
                ]
            },
        )

    def click(self, x, y):
        self.abs_move(x, y)
        time.sleep(0.05)
        for down in (True, False):
            self.cmd(
                "input-send-event",
                {"events": [{"type": "btn", "data": {"down": down, "button": "left"}}]},
            )
            time.sleep(0.05)


class Shell:
    def __init__(self):
        self.s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.s.connect(SERIAL)
        self.s.settimeout(1.0)

    def read(self, seconds=1.5):
        buf = b""
        end = time.time() + seconds
        while time.time() < end:
            try:
                data = self.s.recv(65536)
                if not data:
                    break
                buf += data
            except socket.timeout:
                continue
        return buf.decode("utf-8", "replace")

    def run(self, command, wait=2.0):
        self.s.sendall(command.encode() + b"\n")
        return self.read(wait)

    def counters(self):
        out = self.run("grep -E '^(frames|frames_drawn|frames_over_budget|life)=' /run/litric/shell.status", 2.0)
        seen = {}
        for line in out.splitlines():
            line = line.strip()
            for key in ("frames", "frames_drawn", "frames_over_budget", "life"):
                if line.startswith(key + "="):
                    seen[key] = line.split("=", 1)[1]
        return seen


def main():
    q = Qmp()
    sh = Shell()
    sh.read(0.5)
    before = sh.counters()
    print("before", before)

    # 2 seconds of pointer movement at ~125 a second.
    for i in range(250):
        q.abs_move(300 + (i * 7) % 900, 200 + (i * 5) % 500)
        time.sleep(0.008)
    time.sleep(1.5)
    after = sh.counters()
    print("after moves", after)

    # Open the launcher the way a person does, and let it settle.
    q.click(425, 1021)
    time.sleep(2.5)
    opened = sh.counters()
    print("after launcher", opened)

    # Close it again, so the shell is left at the desktop.
    q.click(425, 1021)
    time.sleep(2.0)
    closed = sh.counters()
    print("after close", closed)


main()
