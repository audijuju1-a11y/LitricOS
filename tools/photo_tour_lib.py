#!/usr/bin/env python3
"""Shared QMP boot and input helpers for the LitricOS photo tours."""

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
BUILD = os.path.join(ROOT, "build")

# The display the tours assume (must match the shell's framebuffer) and the
# usb-tablet's CLIENT range (must match what QEMU expects from us).
# 0x7FFF is measured, not remembered: QEMU's usb-tablet treats client abs
# values as 0..0x7FFF and rescales them onto the device's 0..0xFFFF descriptor
# range (pixel math across two live runs: client 666 -> guest 1332 -> lands at
# 52px of 1280; client p*65535/1279 lands at 2p -> clamped corner). The guest
# honestly learns 65535 via EVIOCGABS and divides by it, so the two constants
# are different on purpose and this one must stay 0x7FFF.
SCREEN_W, SCREEN_H = 1280, 800
AX_MAX = 0x7FFF

QCODES = {
    "esc": "esc", "ret": "ret", "spc": "spc", "tab": "tab", "bs": "backspace",
    "up": "up", "down": "down", "left": "left", "right": "right",
    "f1": "f1", "f2": "f2", "f4": "f4", "f5": "f5",
    "a": "a", "b": "b", "c": "c", "d": "d", "e": "e", "f": "f", "g": "g",
    "h": "h", "i": "i", "j": "j", "k": "k", "l": "l", "m": "m", "n": "n",
    "o": "o", "p": "p", "q": "q", "r": "r", "s": "s", "t": "t", "u": "u",
    "v": "v", "w": "w", "x": "x", "y": "y", "z": "z",
    "0": "0", "1": "1", "2": "2", "3": "3", "4": "4", "5": "5",
    "6": "6", "7": "7", "8": "8", "9": "9",
    "dot": "dot", "minus": "minus", "shift": "shift", "ctrl": "ctrl",
    "alt": "alt", "meta": "meta_l", "meta_l": "meta_l", "meta_r": "meta_r",
}


class Qmp:
    """A minimal QMP client."""

    def __init__(self, path: str) -> None:
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(path)
        self._f = self.sock.makefile("rb")
        self._greeting()

    def _read(self) -> dict:
        while True:
            line = self._f.readline()
            if not line:
                raise ConnectionError("QMP closed")
            msg = json.loads(line)
            if "event" in msg:
                continue
            return msg

    def _greeting(self) -> None:
        self._read()
        self.cmd("qmp_capabilities")

    def cmd(self, name: str, **args) -> dict:
        payload = {"execute": name}
        if args:
            payload["arguments"] = args
        self.sock.sendall(json.dumps(payload).encode() + b"\n")
        resp = self._read()
        if "error" in resp:
            raise RuntimeError(f"{name}: {resp['error']}")
        return resp.get("return", {})

    def close(self) -> None:
        try:
            self.sock.close()
        except OSError:
            pass


def boot_qmp(out_dir: str, image: str | None = None):
    """Boot the disposable image headless with a QMP socket.

    Returns (proc, Qmp, serial_path, diag_sock_path). `image` defaults to
    build/out/litricos.raw; pass a copy (e.g. playtest.raw) so a playtest
    can never disturb a VM the user has open.
    """
    if image is None:
        image = os.path.join(BUILD, "out", "litricos.raw")
    if not os.path.exists(image):
        zst = image + ".zst"
        if os.path.exists(zst):
            subprocess.run(["zstd", "-d", "-f", "-o", image, zst], check=True)
        else:
            raise SystemExit(f"no image: {image}")

    code = None
    for c in ("/usr/share/edk2/x64/OVMF_CODE.4m.fd", "/usr/share/OVMF/OVMF_CODE.fd",
              "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"):
        if os.path.exists(c):
            code = c
            break
    vars_ = None
    for v in ("/usr/share/edk2/x64/OVMF_VARS.4m.fd", "/usr/share/OVMF/OVMF_VARS.fd",
              "/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"):
        if os.path.exists(v):
            vars_ = v
            break
    if not code or not vars_:
        raise SystemExit("no OVMF firmware")

    accel = "kvm" if os.access("/dev/kvm", os.W_OK) else "tcg"
    cpu = "host" if accel == "kvm" else "max"
    sock = os.path.join(BUILD, "tour-qmp.sock")
    if os.path.exists(sock):
        os.remove(sock)
    serial = os.path.join(out_dir, "serial.log")
    serial_sock = os.path.join(BUILD, "tour-serial.sock")
    if os.path.exists(serial_sock):
        os.remove(serial_sock)
    os.makedirs(out_dir, exist_ok=True)
    varsfd = os.path.join(BUILD, "OVMF_VARS.tour.fd")
    with open(vars_, "rb") as s, open(varsfd, "wb") as d:
        d.write(s.read())

    proc = subprocess.Popen([
        "qemu-system-x86_64", "-machine", f"q35,accel={accel}", "-cpu", cpu,
        "-smp", "4", "-m", "2048",
        "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={code}",
        "-drive", f"if=pflash,format=raw,unit=1,file={varsfd}",
        "-drive", f"if=virtio,format=raw,file={image}",
        "-device", "virtio-net-pci,netdev=n0", "-netdev", "user,id=n0",
        "-device", "VGA,vgamem_mb=32",
        "-device", "usb-ehci,id=ehci", "-device", "usb-tablet,bus=ehci.0",
        "-display", "none",
        # ttyS0 is the log (file); ttyS1 is the lab diagnostic console —
        # a Unix socket the harness can type commands into and read the
        # answers from, which turns every "engine session ended" class
        # failure into a one-line answer from inside the guest.
        "-serial", f"file:{serial}",
        "-chardev", f"socket,id=diag,path={serial_sock},server=on,wait=off,logfile={serial}",
        "-serial", "chardev:diag",
        "-qmp", f"unix:{sock},server,nowait",
        "-no-reboot",
    ])
    for _ in range(100):
        if os.path.exists(sock):
            break
        time.sleep(0.1)
    q = Qmp(sock)
    return proc, q, serial, serial_sock


def exec_guest(cmd: str, sock_path: str, timeout: float = 45.0,
               wait_boot: float = 0.0) -> str:
    """Run one command inside the guest through the serial diagnostic shell.

    Connects to the ttyS1 chardev socket, types the command, and reads until
    the sentinel prompt or timeout. Returns everything between the markers,
    so failures come back as the unit's own words instead of our guesses.
    """
    import socket as _s
    time.sleep(wait_boot)
    conn = _s.socket(_s.AF_UNIX, _s.SOCK_STREAM)
    conn.settimeout(timeout)
    conn.connect(sock_path)
    # Drain whatever the console already printed.
    conn.setblocking(False)
    try:
        conn.recv(65536)
    except BlockingIOError:
        pass
    conn.setblocking(True)
    sent = f"echo S{SENTINEL}B; {cmd}; echo S{SENTINEL}E\n"
    conn.sendall(sent.encode())
    buf = b""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            chunk = conn.recv(65536)
        except _s.timeout:
            break
        if not chunk:
            break
        buf += chunk
        if f"S{SENTINEL}E".encode() in buf:
            break
    conn.close()
    text = buf.decode("utf-8", "replace")
    start = text.find(f"S{SENTINEL}B")
    end = text.rfind(f"S{SENTINEL}E")
    if start == -1 or end == -1:
        return text.strip()
    return text[start + len(f"S{SENTINEL}B"):end].strip()


SENTINEL = "1C"


def move(q: Qmp, x: int, y: int) -> None:
    # Screen pixels in, device units out. QEMU's usb-tablet rescales client
    # values 0..0x7FFF onto the device's 0..0xFFFF range (see AX_MAX above),
    # and the guest divides by the 65535 it learned from EVIOCGABS — the
    # composition is the identity, so aiming at pixel p needs exactly this
    # scale. Sending 0xFFFF-scale instead lands every aim at 2p (clamped
    # corner): the failure mode the playtest caught on 2026-09-24.
    dx = round(x * AX_MAX / (SCREEN_W - 1))
    # regression test pins. Skipping this scale aimed every click at the
    # top-left sixth of the screen; the dock was never hit at all.
    dx = round(x * AX_MAX / (SCREEN_W - 1))
    dy = round(y * AX_MAX / (SCREEN_H - 1))
    q.cmd("input-send-event", events=[
        {"type": "abs", "data": {"axis": "x", "value": dx}},
        {"type": "abs", "data": {"axis": "y", "value": dy}},
    ])


def button(q: Qmp, which: str, down: bool) -> None:
    q.cmd("input-send-event", events=[
        {"type": "btn", "data": {"down": down, "button": which}},
    ])


def click(q: Qmp, x: int, y: int, hold: float = 0.08) -> None:
    move(q, x, y)
    time.sleep(0.12)
    button(q, "left", True)
    time.sleep(hold)
    button(q, "left", False)
    time.sleep(0.35)


def rclick(q: Qmp, x: int, y: int) -> None:
    move(q, x, y)
    time.sleep(0.12)
    button(q, "side", True)
    time.sleep(0.08)
    button(q, "side", False)
    time.sleep(0.4)


def dblclick(q: Qmp, x: int, y: int) -> None:
    click(q, x, y, hold=0.05)
    click(q, x, y, hold=0.05)


def key_event(q: Qmp, qcode: str, down: bool) -> None:
    q.cmd("input-send-event", events=[
        {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": qcode}}},
    ])


def key(q: Qmp, name: str) -> None:
    parts = name.split("-")
    codes = [QCODES.get(p, p) for p in parts]
    for c in codes:
        key_event(q, c, True)
    for c in reversed(codes):
        key_event(q, c, False)
    time.sleep(0.25)


PUNCT = {"-": "minus", "/": "slash", ".": "dot", ",": "comma", "=": "equal",
         ";": "semicolon", "'": "apostrophe", "[": "bracket_left", "]": "bracket_right"}


def type_text(q: Qmp, text: str) -> None:
    for ch in text:
        if ch == " ":
            key(q, "spc")
        elif ch in PUNCT:
            key(q, PUNCT[ch])
        else:
            key(q, ch.lower())


class Tour:
    """Sectioned screenshot collector."""

    def __init__(self, q: Qmp, base: str) -> None:
        self.q = q
        self.base = base
        self.section = "00-boot"

    def shoot(self, name: str) -> None:
        d = os.path.join(self.base, self.section)
        os.makedirs(d, exist_ok=True)
        n = len([f for f in os.listdir(d) if f.endswith(".png")])
        ppm = os.path.join(d, f"{n:02d}-{name}.ppm")
        png = ppm[:-4] + ".png"
        try:
            self.q.cmd("screendump", filename=ppm)
        except RuntimeError as e:
            print(f"[tour] shot failed {name}: {e}", file=sys.stderr)
            return
        time.sleep(0.25)
        if os.path.exists(ppm) and os.path.getsize(ppm) > 0:
            subprocess.run(
                ["python3", os.path.join(HERE, "ppm2png.py"), ppm, png],
                check=False, capture_output=True,
            )
            if os.path.exists(png):
                os.remove(ppm)
