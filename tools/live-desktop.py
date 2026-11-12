#!/usr/bin/env python3
"""Use the LitricOS desktop from a browser: boot it headless, stream its screen.

WHY THIS EXISTS
---------------
`tools/show-desktop.sh` and `tools/photo-tour.sh` photograph a boot: they prove
what the system looks like, and they cannot be interrupted. That is the right
shape for evidence and the wrong shape for using the thing.

QEMU on this host has no working local display (no GTK, no SDL, no WSLg surface
we can read back), and the shell itself renders straight into the framebuffer, so
there is no window to attach to anyway. QEMU's QMP socket can do both halves of a
window over a pipe: `screendump` hands back the pixels, `input-send-event`
injects pointer and key events as if they came from the emulated tablet and
keyboard. This script puts a small HTTP server in front of those two commands and
serves one page, so the desktop can be driven in a browser with the real mouse
and the real keyboard.

    tools/live-desktop.py --disk build/out/litricos.raw --port 8777
    # then open http://127.0.0.1:8777/

WHAT IS REAL AND WHAT IS NOT
----------------------------
The guest is a real boot of the real disk image: systemd, the litric services,
the shell, its own event handling. Input is injected at the emulated device, so
it travels the same path as a physical mouse: USB HID -> evdev -> the shell. A
click that opens a window in the page opened a window in the guest.

The picture is a screenshot loop, not a video stream -- 8 frames a second at
1920x1080 over loopback. Fast animation will tear or be skipped. It is a way to
use and inspect the desktop, not a way to benchmark its frame pacing: use
`tools/render_bench` for that.

SAFETY
------
Binds 127.0.0.1 only. Reads the disk image, never writes it: QEMU is started
without a block device that can be written except the image itself, and the
image is a scratch copy made by tools/build.sh. No network device is added
unless asked for, so the guest cannot reach the host or the internet by default.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import signal
import socket
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

# ---------------------------------------------------------------------------
# QMP: one connection, serialised, because QMP is a single-client protocol and
# a screendump racing an input event on two connections would interleave the
# replies and hang whichever asked second.
# ---------------------------------------------------------------------------


class Qmp:
    def __init__(self, sock_path: Path) -> None:
        self.sock_path = sock_path
        self.proc: subprocess.Popen[bytes] | None = None
        self.lock = threading.Lock()
        self.buf = b""
        self.connected = False
        self.greeting: dict = {}

    def connect(self, timeout: float = 30.0) -> None:
        deadline = time.time() + timeout
        while time.time() < deadline:
            if not self.sock_path.exists():
                time.sleep(0.1)
                continue
            try:
                while True:
                    try:
                        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                        self.sock.connect(str(self.sock_path))
                        break
                    except (ConnectionRefusedError, FileNotFoundError):
                        time.sleep(0.1)
                        if time.time() > deadline:
                            raise
            except OSError:
                time.sleep(0.1)
                continue
            self.buf = b""
            self.greeting = self._read_one()
            self._cmd("qmp_capabilities")
            self.connected = True
            return
        raise SystemExit(f"live-desktop: no QMP socket at {self.sock_path}")

    def _read_one(self) -> dict:
        while True:
            nl = self.buf.find(b"\n")
            if nl != -1:
                line, self.buf = self.buf[:nl], self.buf[nl + 1 :]
                try:
                    return json.loads(line)
                except json.JSONDecodeError:
                    continue
            chunk = self.sock.recv(1 << 16)
            if not chunk:
                raise ConnectionError("QMP closed the connection")
            self.buf += chunk

    def _cmd(self, name: str, **arguments) -> dict:
        payload = {"execute": name}
        if arguments:
            payload["arguments"] = arguments
        self.sock.sendall(json.dumps(payload).encode() + b"\n")
        # Events may arrive between the command and its reply; the reply is the
        # one carrying "return" or "error".
        while True:
            reply = self._read_one()
            if "return" in reply or "error" in reply:
                if "error" in reply:
                    raise RuntimeError(f"{name}: {reply['error']}")
                return reply["return"]

    def cmd(self, name: str, **arguments) -> dict:
        with self.lock:
            if not self.connected:
                self.connect()
            return self._cmd(name, **arguments)

    # -- the two commands the page needs -----------------------------------
    def screendump(self, target: Path) -> bytes:
        for fmt in ("png", "ppm"):
            tmp = target.with_suffix("." + fmt)
            try:
                self.cmd("screendump", filename=str(tmp), format=fmt)
            except RuntimeError:
                continue
            if tmp.exists() and tmp.stat().st_size:
                if fmt == "png":
                    return tmp.read_bytes()
                return ppm_to_png(tmp.read_bytes())
        raise RuntimeError("screendump produced no file")

    def send_events(self, events: list[dict]) -> None:
        self.cmd("input-send-event", events=events)

    def status(self) -> dict:
        return self.cmd("query-status")


# ---------------------------------------------------------------------------
# PPM -> PNG, so a QEMU built without PNG support still gives the page an image
# the browser can draw. Minimal, no dependencies: zlib + three filter-0 rows.
# ---------------------------------------------------------------------------


def ppm_to_png(data: bytes) -> bytes:
    import struct
    import zlib

    tokens: list[bytes] = []
    pos = 0
    while len(tokens) < 4:
        while pos < len(data) and data[pos : pos + 1].isspace():
            pos += 1
        if data[pos : pos + 1] == b"#":
            while pos < len(data) and data[pos : pos + 1] != b"\n":
                pos += 1
            continue
        start = pos
        while pos < len(data) and not data[pos : pos + 1].isspace():
            pos += 1
        tokens.append(data[start:pos])
    pos += 1  # exactly one whitespace byte after maxval
    width, height = int(tokens[1]), int(tokens[2])
    raw = data[pos:]
    stride = width * 3
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        rows += raw[y * stride : (y + 1) * stride]

    def chunk(tag: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + tag
            + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(bytes(rows), 6))
        + chunk(b"IEND", b"")
    )


# ---------------------------------------------------------------------------
# Key mapping: a browser gives us characters; QEMU wants QKeyCode names.
# ---------------------------------------------------------------------------

KEY_NAMES = {
    " ": "spc", "\n": "ret", "\r": "ret", "\t": "tab", "\b": "backspace",
    "-": "minus", "=": "equal", "[": "bracket_left", "]": "bracket_right",
    "\\": "backslash", ";": "semicolon", "'": "apostrophe", ",": "comma",
    ".": "dot", "/": "slash", "`": "grave_accent",
}
SHIFTED = {
    "!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6", "&": "7",
    "*": "8", "(": "9", ")": "0", "_": "minus", "+": "equal",
    "{": "bracket_left", "}": "bracket_right", "|": "backslash",
    ":": "semicolon", '"': "apostrophe", "<": "comma", ">": "dot",
    "?": "slash", "~": "grave_accent",
}
NAMED = {
    "Enter": "ret", "Backspace": "backspace", "Tab": "tab", "Escape": "esc",
    " ": "spc", "ArrowUp": "up", "ArrowDown": "down", "ArrowLeft": "left",
    "ArrowRight": "right", "Delete": "delete", "Home": "home", "End": "end",
    "PageUp": "pgup", "PageDown": "pgdn", "Shift": "shift", "Control": "ctrl",
    "Alt": "alt", "Meta": "meta_l",
}


def key_events(text: str) -> list[dict]:
    """Key-down/up pairs for a string, with shift where the character needs it."""
    events: list[dict] = []

    def press(code: str, shift: bool = False) -> None:
        if shift:
            events.append(_key("shift", True))
        events.append(_key(code, True))
        events.append(_key(code, False))
        if shift:
            events.append(_key("shift", False))

    for ch in text:
        if ch in SHIFTED:
            press(SHIFTED[ch], shift=True)
        elif ch.isupper():
            press(ch.lower(), shift=True)
        elif ch in KEY_NAMES:
            press(KEY_NAMES[ch])
        else:
            press(ch)
    return events


def _key(code: str, down: bool) -> dict:
    return {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": code}}}


# ---------------------------------------------------------------------------
# The page.
# ---------------------------------------------------------------------------

PAGE = """<!doctype html>
<meta charset="utf-8">
<title>LitricOS — live desktop</title>
<style>
  :root { color-scheme: dark; }
  * { box-sizing: border-box; }
  body { margin: 0; background: #0b0c10; color: #c9d1d9;
         font: 13px/1.4 ui-monospace, "JetBrains Mono", Consolas, monospace; }
  header { display: flex; gap: 12px; align-items: center; padding: 8px 12px;
           border-bottom: 1px solid #21262d; background: #11131a; }
  header b { color: #e6edf3; font-weight: 600; letter-spacing: .02em; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: #3fb950;
         display: inline-block; margin-right: 6px; }
  .dot.bad { background: #f85149; }
  #screen { display: block; width: 100%; height: auto; background: #000;
            cursor: crosshair; outline: none; image-rendering: auto; }
  #wrap { position: relative; }
  #hint { position: absolute; inset: 0; display: grid; place-items: center;
          color: #7d8590; pointer-events: none; }
  footer { display: flex; gap: 8px; align-items: center; padding: 8px 12px;
           border-top: 1px solid #21262d; background: #11131a; flex-wrap: wrap; }
  button { background: #21262d; color: #e6edf3; border: 1px solid #30363d;
           border-radius: 6px; padding: 4px 10px; font: inherit; cursor: pointer; }
  button:hover { background: #30363d; }
  input[type=text] { background: #0d1117; color: #e6edf3; border: 1px solid #30363d;
                     border-radius: 6px; padding: 4px 8px; font: inherit; width: 16ch; }
  #stat { margin-left: auto; color: #7d8590; }
  #log { padding: 6px 12px; color: #7d8590; max-height: 7em; overflow: auto;
         border-top: 1px solid #161b22; white-space: pre-wrap; font-size: 11px; }
</style>
<header><b>LitricOS</b><span><span class="dot" id="dot"></span><span id="state">connecting</span></span></header>
<div id="wrap">      <img id="screen" alt="LitricOS desktop" tabindex="0" width="1280" height="800">
      <div id="hint">waiting for the first frame…</div>
</div>
<footer>
  <span>type:</span><input id="text" type="text" placeholder="click here, then type" autocomplete="off">
  <button data-key="ret">Enter</button>
  <button data-key="esc">Esc</button>
  <button data-key="tab">Tab</button>
  <button data-key="up">Up</button>
  <button data-key="down">Down</button>
  <button data-key="left">Left</button>
  <button data-key="right">Right</button>
  <span id="stat"></span>
</footer>
<div id="log"></div>
<script>
const img = document.getElementById('screen');
const hint = document.getElementById('hint');
const stat = document.getElementById('stat');
const dot = document.getElementById('dot');
const state = document.getElementById('state');
const logEl = document.getElementById('log');
let frames = 0, t0 = performance.now(), natW = 1920, natH = 1080;

function log(msg) {
  const line = document.createElement('div');
  line.textContent = msg;
  logEl.appendChild(line);
  while (logEl.childElementCount > 40) logEl.removeChild(logEl.firstChild);
  logEl.scrollTop = logEl.scrollHeight;
}

async function post(body) {
  try {
    const r = await fetch('/input', {method: 'POST', body: JSON.stringify(body)});
    if (!r.ok) log('input rejected: ' + (await r.text()).slice(0, 160));
  } catch (e) { log('input failed: ' + e); }
}

const pending = [];
let timer = null;
function queue(ev) {
  pending.push(ev);
  if (timer) return;
  timer = setTimeout(() => {
    timer = null;
    const batch = pending.splice(0, pending.length);
    post(batch.length === 1 ? batch[0] : {type: 'batch', events: batch});
  }, 12);
}

function coords(ev) {
  const r = img.getBoundingClientRect();
  const x = Math.round((ev.clientX - r.left) * (natW / r.width));
  const y = Math.round((ev.clientY - r.top) * (natH / r.height));
  return {x: Math.max(0, Math.min(natW - 1, x)), y: Math.max(0, Math.min(natH - 1, y))};
}

img.addEventListener('mousemove', ev => { const {x, y} = coords(ev); queue({type: 'move', x, y}); });
img.addEventListener('mousedown', ev => {
  ev.preventDefault();
  img.focus();
  const {x, y} = coords(ev);
  queue({type: 'move', x, y});
  queue({type: 'button', button: ['left','middle','right'][ev.button] || 'left', down: true});
});
img.addEventListener('mouseup', ev => {
  const {x, y} = coords(ev);
  queue({type: 'move', x, y});
  queue({type: 'button', button: ['left','middle','right'][ev.button] || 'left', down: false});
});
img.addEventListener('contextmenu', ev => ev.preventDefault());
img.addEventListener('wheel', ev => {
  ev.preventDefault();
  queue({type: 'button', button: ev.deltaY < 0 ? 'wheel-up' : 'wheel-down', down: true});
  queue({type: 'button', button: ev.deltaY < 0 ? 'wheel-up' : 'wheel-down', down: false});
}, {passive: false});

document.addEventListener('keydown', ev => {
  if (ev.target && ev.target.tagName === 'INPUT' && ev.target.id === 'text') return;
  if (ev.metaKey || ev.ctrlKey) return;
  if (['Tab','Backspace','Space','ArrowUp','ArrowDown','ArrowLeft','ArrowRight','Enter'].includes(ev.code) ||
      ev.key.length === 1) {
    if (ev.key === 'Tab' || ev.key.startsWith('Arrow')) ev.preventDefault();
    queue({type: 'key', key: ev.key, code: ev.code, down: true});
    queue({type: 'key', key: ev.key, code: ev.code, down: false});
  }
});

document.querySelectorAll('button[data-key]').forEach(b => {
  b.addEventListener('click', () => {
    queue({type: 'qcode', code: b.dataset.key, down: true});
    queue({type: 'qcode', code: b.dataset.key, down: false});
  });
});

const text = document.getElementById('text');
text.addEventListener('keydown', ev => {
  if (ev.key === 'Enter') { queue({type: 'text', text: text.value + '\\n'}); text.value = ''; ev.preventDefault(); }
});
document.getElementById('text').addEventListener('change', () => {});

let running = true;
function next() {
  if (!running) return;
  const url = '/frame.png?t=' + Date.now();
  img.src = url;
}
img.addEventListener('load', () => {
  frames++;
  hint.style.display = 'none';
  // The frame's own size is the guest's mode. Taking it from the image rather
  // than from a configured constant is what keeps the pointer mapping honest on
  // a host whose QEMU has no virtio-gpu to honour the kernel's video= argument.
  if (img.naturalWidth && (img.naturalWidth !== natW || img.naturalHeight !== natH)) {
    natW = img.naturalWidth; natH = img.naturalHeight;
    log('guest framebuffer: ' + natW + '×' + natH);
  }
  const dt = (performance.now() - t0) / 1000;
  stat.textContent = natW + '×' + natH + '  ' + (frames / Math.max(dt, 0.001)).toFixed(1) + ' fps';
  setTimeout(next, 90);
});
img.addEventListener('error', () => setTimeout(next, 800));
setTimeout(next, 300);
fetch('/info').then(r => r.json()).then(i => {
  natW = i.width; natH = i.height;
  img.width = natW; img.height = natH;
  state.textContent = i.state || 'running';
  dot.classList.toggle('bad', !(i.state === 'running'));
  log('guest: ' + i.detail);
}).catch(() => {});
</script>
"""


# ---------------------------------------------------------------------------
# HTTP front end.
# ---------------------------------------------------------------------------


class Handler(BaseHTTPRequestHandler):
    qmp: Qmp
    size: tuple[int, int] = (1920, 1080)
    frame_cache: dict = {"png": b"", "at": 0.0}
    frame_lock = threading.Lock()

    def log_message(self, fmt: str, *args) -> None:  # quiet: this is a UI
        pass

    def _send(self, code: int, body: bytes, ctype: str) -> None:
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802 -- http.server's spelling
        if self.path.startswith("/frame.png"):
            with self.frame_lock:
                age = time.time() - self.frame_cache["at"]
                if age > 0.06 or not self.frame_cache["png"]:
                    try:
                        png = self.qmp.screendump(Path("/tmp/litricos-frame"))
                    except Exception as exc:  # a dead guest should not kill the page
                        self._send(503, str(exc).encode(), "text/plain; charset=utf-8")
                        return
                    self.frame_cache["png"] = png
                    self.frame_cache["at"] = time.time()
                png = self.frame_cache["png"]
            self._send(200, png, "image/png")
            return
        if self.path.startswith("/info"):
            try:
                status = self.qmp.status()
            except Exception as exc:
                status = {"status": "error", "detail": str(exc)}
            body = json.dumps({
                "width": self.size[0],
                "height": self.size[1],
                "state": status.get("status", "unknown"),
                "detail": "running" if status.get("running") else str(status.get("status")),
            }).encode()
            self._send(200, body, "application/json")
            return
        if self.path.startswith("/serial"):
            path = Path("/tmp/litricos-serial.log")
            body = path.read_bytes()[-20000:] if path.exists() else b"(no serial log)"
            self._send(200, body, "text/plain; charset=utf-8")
            return
        self._send(200, PAGE.encode(), "text/html; charset=utf-8")

    def do_POST(self) -> None:  # noqa: N802
        length = int(self.headers.get("Content-Length") or 0)
        try:
            payload = json.loads(self.rfile.read(length) or b"{}")
        except json.JSONDecodeError:
            self._send(400, b"bad json", "text/plain; charset=utf-8")
            return
        try:
            events = payload if payload.get("type") == "batch" else None
            batch = events["events"] if events else [payload]
            send_input(self.qmp, batch, self.size)
        except Exception as exc:
            self._send(500, str(exc).encode(), "text/plain; charset=utf-8")
            return
        self._send(204, b"", "text/plain; charset=utf-8")


def send_input(qmp: Qmp, batch: list[dict], size: tuple[int, int]) -> None:
    """Translate the page's vocabulary into QMP input events, in one call."""
    out: list[dict] = []
    for item in batch:
        kind = item.get("type")
        if kind == "move":
            out.append({"type": "abs", "data": {"axis": "x", "value": _abs(item["x"], size[0])}})
            out.append({"type": "abs", "data": {"axis": "y", "value": _abs(item["y"], size[1])}})
        elif kind == "button":
            out.append({"type": "btn", "data": {"down": bool(item["down"]), "button": item["button"]}})
        elif kind == "qcode":
            out.append(_key(item["code"], bool(item["down"])))
        elif kind == "key":
            out.extend(events_for_key(item["key"]))
        elif kind == "text":
            out.extend(key_events(item["text"]))
    if out:
        qmp.send_events(out)


def _abs(pos: int, extent: int) -> int:
    # The tablet's absolute axes are 0..32767; QEMU scales them to whatever the
    # guest device reports, so the page never needs to know the tablet's range.
    return max(0, min(32767, int(pos * 32767 / max(extent - 1, 1))))


def events_for_key(key: str) -> list[dict]:
    if key in NAMED:
        return [_key(NAMED[key], True), _key(NAMED[key], False)]
    if len(key) == 1:
        return key_events(key)
    return []


# ---------------------------------------------------------------------------
# Boot.
# ---------------------------------------------------------------------------


def pick_display_device(explicit: str | None) -> str:
    """The best display device this QEMU actually has.

    The image's kernel command line forces `video=Virtual-1:1920x1080`, which is
    the virtio-gpu connector name, and this host's QEMU is built without
    virtio-gpu -- so a hard-coded `-device virtio-vga` fails with "not a valid
    device model name" before the guest ever runs. Rather than hard-code a
    second guess, ask QEMU what it has and take the first that gives the guest a
    real framebuffer: bochs-display (a DRM device with an EDID, so the kernel
    modesets to a sensible preferred mode) before plain VGA.
    """
    if explicit:
        return explicit
    probe = subprocess.run(
        ["qemu-system-x86_64", "-device", "help"], capture_output=True, text=True, check=False
    )
    available = probe.stdout
    for candidate in ("virtio-gpu-pci", "virtio-vga", "bochs-display", "VGA"):
        if f'name "{candidate}"' in available:
            return candidate
    raise SystemExit(
        "live-desktop: this qemu has no display device this script knows "
        "(looked for virtio-gpu-pci, virtio-vga, bochs-display, VGA)"
    )


def start_qemu(args: argparse.Namespace) -> subprocess.Popen[bytes]:
    sock = Path(args.qmp)
    serial = Path(args.serial)
    for path in (sock, serial):
        path.unlink(missing_ok=True)

    cmd = [
        "qemu-system-x86_64",
        "-name", "LitricOS-live",
        "-machine", "q35,accel=kvm:tcg,vmport=off",
        "-cpu", "max",
        "-smp", str(args.smp),
        "-m", f"{args.memory}M",
        "-boot", "order=c",
        "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={args.firmware}",
        "-drive", f"if=pflash,format=raw,unit=1,file={args.vars}",
        "-drive", f"if=virtio,format=raw,file={args.disk}",
        # Chosen from what this QEMU has: virtio-gpu when present (it gives the
        # guest the "Virtual-1" connector the kernel command line names), bochs
        # or VGA otherwise. The page reads the frame's real size, so whichever
        # one lands, the picture is the guest's own mode rather than a guess.
        "-device", args.display_device,
        "-device", "usb-ehci,id=ehci",
        "-device", "usb-tablet,bus=ehci.0",
        "-device", "usb-kbd,bus=ehci.0",
        "-display", "none",
        "-qmp", f"unix:{sock},server,nowait",
        "-serial", f"file:{serial}",
        "-no-reboot",
    ]
    if args.net:
        cmd += ["-device", "virtio-net-pci,netdev=n0", "-netdev", "user,id=n0"]
    if args.extra:
        cmd += args.extra

    log = Path("/tmp/litricos-qemu.log").open("wb")
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
    time.sleep(1.0)
    if proc.poll() is not None:
        raise SystemExit(
            "live-desktop: qemu exited immediately:\n"
            + Path("/tmp/litricos-qemu.log").read_text(errors="replace")[-2000:]
        )
    return proc


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--disk", default=str(root / "build/out/litricos.raw"))
    parser.add_argument("--firmware", default="/usr/share/edk2/x64/OVMF_CODE.4m.fd")
    parser.add_argument("--vars", default="/tmp/litricos-OVMF_VARS.fd")
    parser.add_argument("--qmp", default="/tmp/litricos-qmp.sock")
    parser.add_argument("--serial", default="/tmp/litricos-serial.log")
    parser.add_argument("--port", type=int, default=8777)
    parser.add_argument("--memory", type=int, default=2048)
    parser.add_argument("--smp", type=int, default=4)
    parser.add_argument("--net", action="store_true", help="give the guest user-mode networking")
    parser.add_argument("--width", type=int, default=1920)
    parser.add_argument("--height", type=int, default=1080)
    parser.add_argument("--no-qemu", action="store_true", help="attach to an existing QMP socket")
    parser.add_argument(
        "--display-device",
        default=None,
        help="override the display device (default: the best one this qemu offers)",
    )
    parser.add_argument("extra", nargs="*", help="extra QEMU arguments")
    args = parser.parse_args()
    args.display_device = pick_display_device(args.display_device)

    disk = Path(args.disk)
    if not disk.exists():
        raise SystemExit(f"live-desktop: no disk image at {disk} (run tools/build.sh first)")

    # A fresh NVRAM copy per run: reusing one keeps the previous boot order and
    # any half-finished efibootmgr entry, which reads as "the image stopped
    # booting" on the second run only.
    vars_path = Path(args.vars)
    stock = Path("/usr/share/edk2/x64/OVMF_VARS.4m.fd")
    if not args.no_qemu and stock.exists():
        shutil.copyfile(stock, vars_path)

    qmp = Qmp(Path(args.qmp))
    proc = None
    if not args.no_qemu:
        proc = start_qemu(args)

    Handler.qmp = qmp
    Handler.size = (args.width, args.height)

    def shutdown(*_args) -> None:
        if proc is not None and proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
        sys.exit(0)

    signal.signal(signal.SIGTERM, shutdown)
    signal.signal(signal.SIGINT, shutdown)

    qmp.connect()
    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    print(f"live-desktop: http://127.0.0.1:{args.port}/", flush=True)
    print(f"live-desktop: qmp {args.qmp}, serial {args.serial}", flush=True)
    try:
        server.serve_forever()
    finally:
        shutdown()


if __name__ == "__main__":
    sys.exit(main())
