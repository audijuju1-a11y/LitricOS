#!/usr/bin/env python3
"""LitricOS — full-feature photo tour over QMP.

Boots the disposable image headless and drives the REAL guest input path:
QMP input-send-event (absolute pointer to the usb-tablet, qcode keyboard to
the i8042/evdev stack), then photographs the framebuffer with a screendump
issued through the same QMP socket. Screenshots land in section folders so
the gallery reads like a narrative of the OS.

    python3 tools/photo-tour.py [--quick] [--out DIR]

Every click is an absolute-position USB-tablet event, i.e. exactly what the
guest's evdev stack, device discovery, hit-testing and window manager must
survive. Nothing is stubbed on our side of the wire.
"""

from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
BUILD = os.path.join(ROOT, "build")

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
    "dot": "dot", "minus": "minus", "ret": "ret", "shift": "shift",
    "ctrl": "ctrl", "alt": "alt", "meta": "meta",
}


class Qmp:
    """A minimal QMP client: negotiate, then one command at a time."""

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


def move(q: Qmp, x: int, y: int) -> None:
    """Absolute pointer motion to pixel (x, y) via the usb-tablet."""
    q.cmd(
        "input-send-event",
        events=[
            {"type": "abs", "data": {"axis": "x", "value": x}},
            {"type": "abs", "data": {"axis": "y", "value": y}},
        ],
    )


def button(q: Qmp, which: str, down: bool) -> None:
    q.cmd(
        "input-send-event",
        events=[
            {"type": "btn", "data": {"down": down, "button": which}},
        ],
    )


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
    q.cmd(
        "input-send-event",
        events=[
            {
                "type": "key",
                "data": {"down": down, "key": {"type": "qcode", "data": qcode}},
            }
        ],
    )


def key(q: Qmp, name: str) -> None:
    """Press one key by shorthand; combos like 'ctrl-a' or 'alt-f4' work."""
    parts = name.split("-")
    codes = [QCODES.get(p, p) for p in parts]
    for c in codes:
        key_event(q, c, True)
    for c in reversed(codes):
        key_event(q, c, False)
    time.sleep(0.25)


def type_text(q: Qmp, text: str) -> None:
    for ch in text:
        if ch == " ":
            key(q, "spc")
        else:
            key(q, ch.lower())


class Tour:
    def __init__(self, q: Qmp, base: str) -> None:
        self.q = q
        self.base = base
        self.section = "00-boot"
        self.count = 0
        os.makedirs(base, exist_ok=True)

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
                self.count += 1

    def goto(self, section: str) -> None:
        self.section = section
        print(f"[tour] {section}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    W = 0.5 if args.quick else 1.0
    out = args.out or os.path.join(BUILD, "tour", time.strftime("%Y%m%d-%H%M%S"))

    image = os.path.join(BUILD, "out", "litricos.raw")
    if not os.path.exists(image):
        zst = image + ".zst"
        if os.path.exists(zst):
            subprocess.run(["zstd", "-d", "-f", "-o", image, zst], check=True)
        else:
            print("no image", file=sys.stderr)
            return 1

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
        print("no OVMF firmware", file=sys.stderr)
        return 1

    accel = "kvm" if os.access("/dev/kvm", os.W_OK) else "tcg"
    cpu = "host" if accel == "kvm" else "max"
    sock = os.path.join(BUILD, "tour-qmp.sock")
    serial = os.path.join(out, "serial.log")
    os.makedirs(out, exist_ok=True)
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
        "-serial", f"file:{serial}",
        "-qmp", f"unix:{sock},server,nowait",
        "-no-reboot",
    ])
    try:
        for _ in range(100):
            if os.path.exists(sock):
                break
            time.sleep(0.1)
        q = Qmp(sock)
        tour = Tour(q, out)
        S = lambda s: time.sleep(s * W)

        # ---- 00 boot ----
        for t in (12, 20, 30, 42):
            S(t if t == 12 else t - prev)  # noqa: F821
            prev = t  # noqa: F841
            tour.shoot(f"boot-t{t}" if t != 42 else "desktop-idle")

        # ---- 01 desktop ----
        tour.goto("01-desktop")
        tour.shoot("start")
        click(q, 64, 140); S(0.5); tour.shoot("icon-select")
        dblclick(q, 64, 140); S(1.2); tour.shoot("icon-open-files")
        key(q, "esc")

        # ---- 02 menubar menus ----
        tour.goto("02-menubar-menus")
        for i, name in enumerate(["menu-app", "menu-file", "menu-edit", "menu-view",
                                  "menu-window", "menu-help"]):
            click(q, 200 + i * 62, 12)
            S(0.5)
            tour.shoot(name)
            key(q, "down")
            tour.shoot(f"{name}-navigated")
            key(q, "esc")
        click(q, 1200, 12); S(0.4); tour.shoot("status-area")
        click(q, 1240, 12); S(0.4); tour.shoot("clock-area")

        # ---- 03 desktop context menu ----
        tour.goto("03-context-desktop")
        rclick(q, 900, 600); S(0.6); tour.shoot("ctx-desktop")
        tour.shoot("ctx-desktop-hold")
        key(q, "esc")
        rclick(q, 700, 300); S(0.6); tour.shoot("ctx-desktop-2")
        click(q, 730, 340); S(0.8); tour.shoot("ctx-item-clicked")

        # ---- 04 files app ----
        tour.goto("04-files-app")
        click(q, 200, 12); S(0.4); key(q, "down"); key(q, "ret"); S(1.5)
        tour.shoot("files-window")
        rclick(q, 500, 400); S(0.6); tour.shoot("files-ctx-blank")
        key(q, "esc")
        for i, (x, y, name) in enumerate([
                (90, 180, "sidebar-home"), (90, 215, "sidebar-docs"),
                (90, 250, "sidebar-downloads"), (90, 285, "sidebar-desktop")]):
            click(q, x, y); S(0.6); tour.shoot(f"files-{name}")
        click(q, 700, 30); S(0.5); tour.shoot("files-toolbar-view")
        dblclick(q, 420, 200); S(1.0); tour.shoot("files-navigated")

        # ---- 05 new folder dialog ----
        tour.goto("05-new-folder-dialog")
        rclick(q, 500, 450); S(0.5)
        tour.shoot("ctx-before-new-folder")
        click(q, 530, 470); S(0.9); tour.shoot("new-folder-dialog")
        type_text(q, "Tour Folder")
        tour.shoot("typed-name")
        key(q, "ret"); S(1.0); tour.shoot("folder-created")
        click(q, 420, 220); S(0.3)
        key(q, "f2"); S(0.6); tour.shoot("rename-active")
        key(q, "ret")

        # ---- 06 keyboard ----
        tour.goto("06-keyboard")
        key(q, "f5"); S(0.5); tour.shoot("kbd-refresh")
        key(q, "ctrl-a"); S(0.4); tour.shoot("kbd-select-all")
        key(q, "esc")
        key(q, "alt-f4"); S(0.8); tour.shoot("kbd-close-window")

        # ---- 07 defender ----
        tour.goto("07-defender")
        click(q, 200, 12); S(0.4)
        key(q, "down"); key(q, "down"); key(q, "down"); key(q, "ret"); S(1.8)
        tour.shoot("defender-window")
        click(q, 700, 420); S(0.5); tour.shoot("defender-scan-click1")
        click(q, 700, 460); S(0.5); tour.shoot("defender-scan-click2")
        S(2.0); tour.shoot("defender-scanning")
        S(3.0); tour.shoot("defender-results")
        rclick(q, 600, 350); S(0.6); tour.shoot("defender-ctx")
        key(q, "esc")

        # ---- 08 browser ----
        tour.goto("08-browser")
        click(q, 200, 12); S(0.4)
        for _ in range(4):
            key(q, "down")
        key(q, "ret"); S(2.5)
        tour.shoot("browser-open")
        click(q, 640, 55); S(0.5); tour.shoot("browser-omnibox-focus")
        type_text(q, "google bench")
        tour.shoot("browser-typed-google")
        key(q, "ret"); S(3.0); tour.shoot("browser-google-search")
        rclick(q, 640, 55); S(0.5); tour.shoot("browser-omnibox-ctx")
        key(q, "esc")
        click(q, 640, 55); S(0.4)
        type_text(q, "litricos")
        tour.shoot("browser-typed-2")
        key(q, "ret"); S(3.0); tour.shoot("browser-search-2")
        click(q, 90, 55); S(0.8); tour.shoot("browser-back")
        click(q, 130, 55); S(0.8); tour.shoot("browser-forward")
        key(q, "alt-f4"); S(0.8)

        # ---- 09 trash ----
        tour.goto("09-trash")
        dblclick(q, 1208, 760); S(1.2); tour.shoot("trash-open")
        tour.shoot("trash-contents")
        key(q, "alt-f4"); S(0.5)

        # ---- 10 dock ----
        tour.goto("10-dock")
        for i, x in enumerate(range(400, 1100, 120)):
            move(q, x, 764); S(0.25); tour.shoot(f"dock-hover-{i}")
            click(q, x, 764); S(0.6); tour.shoot(f"dock-launch-{i}")

        # ---- 11 settings ----
        tour.goto("11-settings")
        click(q, 200, 12); S(0.4)
        for _ in range(5):
            key(q, "down")
        key(q, "ret"); S(1.5)
        tour.shoot("settings-open")
        click(q, 400, 300); S(0.4); tour.shoot("settings-body")
        key(q, "alt-f4"); S(0.5)

        # ---- 12 final ----
        tour.goto("12-final")
        click(q, 800, 500); S(0.4)
        tour.shoot("desktop-final")
        rclick(q, 640, 400); S(0.5); tour.shoot("final-ctx")
        key(q, "esc")
        tour.shoot("the-end")

        q.close()
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        if os.path.exists(sock):
            os.remove(sock)

    print(f"\n[tour] complete: {out}")
    total = sum(len(files) for _, _, files in os.walk(out) if any(f.endswith('.png') for f in files))
    png = sum(1 for root, _, files in os.walk(out) for f in files if f.endswith(".png"))
    print(f"[tour] screenshots: {png}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
