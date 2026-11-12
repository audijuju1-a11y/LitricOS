#!/usr/bin/env python3
"""Drive a booted LitricOS guest through the real Games page and Stop button.

The driver talks only to QEMU's QMP socket and reads the guest's serial log.
It does not import the shell, call a shell function, or inspect a screenshot
made by the shell. The keyboard, absolute pointer, framebuffer and process
table all belong to the booted guest.

Inputs
------
--qmp PATH       QMP Unix socket created by tools/qemu-guest-proof.sh
--serial PATH    guest ttyS0 log (the script redirects the shell there)
--out DIR        screenshots and result.json
--timeout SEC    upper bound for each wait, including the first frame

The proof has three positive assertions and one negative one:

1. the guest installs ``com.litric.test-game`` and lists the installed files;
2. a real Wayland client maps a fullscreen ``litric.client`` window at exactly
   the requested ``--width`` x ``--height``;
3. the framebuffer contains the client's own colourful pixels;
4. Stop removes the client window and its pixels, leaving the desktop.

Exit status is non-zero if any assertion times out or the pixel evidence is
not present. A screenshot is a hypervisor read of the guest scanout, not a
renderer-side test fixture.
"""

from __future__ import annotations

import argparse
import json
import re
import socket
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE_W, BASE_H = 1280, 800
W, H = BASE_W, BASE_H
AXIS_MAX = 0x7FFF
# The Settings window is 980x620 at +64,+76 at the base display and the installed
# game is the only row on its Games page. The shell derives its layout scale
# from the framebuffer width, so the same controls move with the window.
PLAY_BASE = (981, 266)
STOP_BASE = (713, 26)
PLAY = PLAY_BASE
STOP = STOP_BASE
GAME_COLOUR_MIN = 50_000
STOPPED_COLOUR_MAX = 30_000
# A pinned public release asset: a small, immutable file GitHub serves with
# Content-Disposition: attachment, so the engine downloads it instead of
# rendering it. The size and digest were computed from the bytes on the host, so
# the guest's own numbers are compared with an independent copy of the file.
DOWNLOADS = [
    {
        "name": "gh-release",
        "url": "github.com/cli/cli/releases/download/v2.62.0/gh_2.62.0_linux_amd64.tar.gz",
        "file": "gh_2.62.0_linux_amd64.tar.gz",
        "size": 13_065_800,
        "sha256": "41c8b0698ad3003cb5c44bde672a1ffd5f818595abd80162fbf8cc999418446a",
        "large": True,
    },
    {
        "name": "sha256sum",
        "url": "github.com/jqlang/jq/releases/download/jq-1.7.1/sha256sum.txt",
        "file": "sha256sum.txt",
        "size": 2_129,
        "sha256": "f1f692fe9f923a4f5c6741729fb5931212ebccfff976225feb7a2ccec8635da4",
        "large": False,
    },
]

def stop_point(width: int) -> tuple[int, int]:
    """Return the Game Mode Stop centre for a framebuffer width."""
    scale = max(width / BASE_W, 0.75)
    return (round(STOP_BASE[0] * scale), round(STOP_BASE[1] * scale))


def play_point(settings: dict, width: int) -> tuple[int, int]:
    """Return Play's centre from the actual Settings frame and UI scale.

    The Settings window is 980x620 at +64,+76 on the base display. Its origin
    is chosen by the window manager (and can move vertically to fit the
    screen), while its content scales with the framebuffer width. Using the
    captured frame instead of scaling the absolute base point is what keeps
    the click inside the button at 1920x1080.
    """
    scale = max(width / BASE_W, 0.75)
    return (
        settings["x"] + round((PLAY_BASE[0] - 64) * scale),
        settings["y"] + round((PLAY_BASE[1] - 76) * scale),
    )


class QMP:
    def __init__(self, path: str, width: int = W, height: int = H):
        self.width = width
        self.height = height
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(10)
        self.sock.connect(path)
        self.file = self.sock.makefile("rb")
        self._read()
        self.command("qmp_capabilities")

    def _read(self) -> dict:
        while True:
            line = self.file.readline()
            if not line:
                raise RuntimeError("QMP connection closed")
            message = json.loads(line)
            if "event" in message:
                continue
            return message

    def command(self, name: str, **arguments) -> dict:
        payload = {"execute": name}
        if arguments:
            payload["arguments"] = arguments
        self.sock.sendall(json.dumps(payload).encode() + b"\n")
        reply = self._read()
        if "error" in reply:
            raise RuntimeError(f"QMP {name}: {reply['error']}")
        return reply.get("return", {})

    def move(self, x: int, y: int) -> None:
        self.command(
            "input-send-event",
            events=[
                {
                    "type": "abs",
                    "data": {"axis": "x", "value": round(x * AXIS_MAX / (self.width - 1))},
                },
                {
                    "type": "abs",
                    "data": {"axis": "y", "value": round(y * AXIS_MAX / (self.height - 1))},
                },
            ],
        )

    def button(self, down: bool) -> None:
        self.command(
            "input-send-event",
            events=[{"type": "btn", "data": {"down": down, "button": "left"}}],
        )

    def click(self, x: int, y: int) -> None:
        self.move(x, y)
        time.sleep(0.15)
        self.button(True)
        time.sleep(0.08)
        self.button(False)
        time.sleep(0.4)

    def key_event(self, qcode: str, down: bool) -> None:
        self.command(
            "input-send-event",
            events=[
                {
                    "type": "key",
                    "data": {
                        "down": down,
                        "key": {"type": "qcode", "data": qcode},
                    },
                }
            ],
        )
        time.sleep(0.05)

    def key(self, qcode: str) -> None:
        self.key_event(qcode, True)
        self.key_event(qcode, False)

    def chord(self, *qcodes: str) -> None:
        """Press keys together, then release them in reverse order.

        QMP has no modifier field on a key event: the guest's own input layer
        tracks Ctrl from its key-down, which is exactly what a person's keyboard
        sends. Ctrl+T therefore arrives as Ctrl down, T down, T up, Ctrl up.
        """
        for qcode in qcodes:
            self.key_event(qcode, True)
        for qcode in reversed(qcodes):
            self.key_event(qcode, False)
        time.sleep(0.1)

    def text(self, value: str) -> None:
        qcodes = {" ": "spc", "-": "minus", ".": "dot", "/": "slash"}
        for character in value:
            # QEMU has no "_" qcode: it is Shift+minus on the guest's keyboard.
            # The GitHub asset names carry underscores, so a driver that cannot
            # type one cannot reach the URL it claims to test.
            if character == "_":
                self.chord("shift", "minus")
            else:
                self.key(qcodes.get(character, character))
            time.sleep(0.08)

    def screenshot(self, path: Path) -> None:
        self.command("screendump", filename=str(path))
        # QEMU writes the file synchronously, but TCG can still have a queued
        # frame callback after the command returns.
        time.sleep(0.3)


def serial_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        return ""


def latest_block(text: str, begin: str, end: str) -> str:
    start = text.rfind(begin)
    if start < 0:
        return ""
    finish = text.find(end, start + len(begin))
    return text[start : finish + len(end)] if finish >= 0 else text[start:]


def latest_status(text: str) -> str:
    return latest_block(text, "LITRIC_STATUS_BEGIN", "LITRIC_STATUS_END")



def download_events(text: str, marker: str) -> list[dict]:
    """Parse the guest's staged/saved download lines into their fields."""
    events = []
    for line in text.splitlines():
        if marker not in line:
            continue
        fields = {}
        for part in line.split():
            if "=" in part:
                key, value = part.split("=", 1)
                if key in ("name", "size", "sha256"):
                    fields[key] = value
        if fields:
            events.append(fields)
    return events


def hotspot_click(text: str, key: str) -> tuple[int, int] | None:
    """The shell's own centre for a drawn control, in display coordinates."""
    value = latest_value(text, key)
    if not value or "," not in value:
        return None
    x, y = value.split(",", 1)
    try:
        return int(float(x)), int(float(y))
    except ValueError:
        return None


def latest_window_text(text: str) -> str:
    """The diagnostics from the newest `windows=` line onward.

    The shell writes status to its own file, and also to the console. The
    guest init wraps only some snapshots in LITRIC_STATUS markers, so window
    lines can sit outside the last marked block; taking the newest state also
    avoids treating a pre-game Settings-only block as the current one.
    """
    at = text.rfind("litric.shell.windows=")
    return text[at:] if at >= 0 else ""


def has_client_window(text: str, width: int = W, height: int = H) -> bool:
    """Whether the latest window state has the client fullscreen at *exactly* w×h."""
    return bool(
        re.search(
            rf"litric\.shell\.window=#\d+ litric\.client\b.*?"
            rf"\b{width}x{height}\+0\+0 fullscreen\b",
            latest_window_text(text),
        )
    )


def client_window_line(text: str, width: int = W, height: int = H) -> str | None:
    match = re.search(
        rf"litric\.shell\.window=#\d+ litric\.client\b.*?"
        rf"\b{width}x{height}\+0\+0 fullscreen\b[^\r\n]*",
        latest_window_text(text),
    )
    return match.group(0).strip() if match else None


def framebuffer_matches(text: str, width: int, height: int) -> bool:
    """Whether the shell opened a surface at the requested geometry.

    The framebuffer path names its geometry in `fb=`, while the KMS path names
    it in `drm.mode=`. Requiring only `fb=` made every DRM boot wait forever
    even though the shell had reached the desktop; the proof must accept the
    path it asked the shell to prefer.
    """
    return (
        f"litric.shell.fb={width}x{height} " in text
        or f"litric.shell.drm.mode={width}x{height}" in text
    )


def last_window_count(text: str) -> int | None:
    matches = re.findall(r"litric\.shell\.windows=(\d+)", text)
    return int(matches[-1]) if matches else None


WINDOW_LINE = re.compile(
    r'litric\.shell\.window=#(\d+) (\S+) "([^"]*)" (\d+)x(\d+)\+(-?\d+)\+(-?\d+)'
)


def window_frames(text: str) -> list[dict]:
    frames = []
    for match in WINDOW_LINE.finditer(latest_window_text(text)):
        frames.append(
            {
                "id": int(match.group(1)),
                "app": match.group(2),
                "title": match.group(3),
                "width": int(match.group(4)),
                "height": int(match.group(5)),
                "x": int(match.group(6)),
                "y": int(match.group(7)),
            }
        )
    return frames


def frame_inside(frame: dict, width: int, height: int) -> bool:
    return (
        frame["x"] >= 0
        and frame["y"] >= 0
        and frame["x"] + frame["width"] <= width
        and frame["y"] + frame["height"] <= height
    )


def client_gone_after_stop(text: str, mark: int) -> bool:
    """Require the newest console window state after the Stop click.

    The serial log can contain a stale `windows=2` and client line immediately
    after the click, before the shell's next frame. Looking for the absence of
    a client line anywhere in the tail treats that stale state as failure; the
    newest `windows=` line is the one that describes the current frame.
    """
    tail = text[mark:]
    states = list(re.finditer(r"litric\.shell\.windows=(\d+)", tail))
    if not states or states[-1].group(1) != "1":
        return False
    after = tail[states[-1].end() :]
    return not re.search(r"litric\.shell\.window=#\d+ litric\.client\b", after)


def latest_prefixed(text: str, prefix: str) -> str | None:
    """The newest line with a prefix, returned without that prefix."""
    value = None
    for line in text.splitlines():
        line = line.strip()
        if line.startswith(prefix):
            value = line[len(prefix) :]
    return value


def latest_value(text: str, key: str) -> str | None:
    """The newest `key=value` status line, or None when it was never written."""
    value = None
    for line in text.splitlines():
        line = line.strip()
        if line.startswith(key + "="):
            value = line[len(key) + 1 :]
    return value


def latest_browser(text: str) -> dict[str, str]:
    """The browser fields from the newest contiguous group in the serial log.

    Reading each key with `latest_value` mixes a frame from an older group with
    a tab count from a newer one. Parsing only the marked status block is also
    too slow: the shell writes an unmarked browser group more often, and the
    driver waited on the older marked snapshot. Walk back to the newest field,
    then consume only that field's contiguous group, so every value describes
    the same instant and cannot be older than the last one written.
    """
    lines = text.splitlines()
    end = None
    prefix = ""
    prefixes = ("litric.shell.browser.", "browser.")
    for i in range(len(lines) - 1, -1, -1):
        line = lines[i].strip()
        for candidate in prefixes:
            if line.startswith(candidate):
                end = i
                prefix = candidate
                break
        if end is not None:
            break
    if end is None:
        return {}
    start = end
    while start > 0 and lines[start - 1].strip().startswith(prefix):
        start -= 1
    values: dict[str, str] = {}
    for line in lines[start : end + 1]:
        line = line.strip()
        if not line.startswith(prefix):
            continue
        field = line[len(prefix) :]
        key, sep, value = field.partition("=")
        if sep:
            values[key] = value
    return values


def latest_browser_value(text: str, key: str) -> str | None:
    """One field from the newest browser group, with either prefix accepted."""
    for prefix in ("litric.shell.browser.", "browser."):
        if key.startswith(prefix):
            key = key[len(prefix) :]
            break
    return latest_browser(text).get(key)


def browser_click_points(frame: dict, scale: float) -> list[tuple[int, int]]:
    """Candidate centres of the browser's address field, from its window frame.

    The omnibox row sits under the tab strip and the field is inset 4 px inside
    that row. The frame the window manager reports may also include the title
    bar, so the driver tries a ladder of offsets and lets the shell's own
    `browser.omnibox=` diagnostic say which click landed.
    """
    x = int(frame["x"] + frame["width"] / 2)
    # Stay above the Downloads toolbar button at y+~213: the old 105px
    # fallback clicked it while looking for the address bar, which opened the
    # Downloads panel and made the next focus attempt fail for the wrong reason.
    return [(x, int(frame["y"] + offset * scale)) for offset in (75, 45, 60, 90, 30)]


def wait_serial(path: Path, predicate, what: str, timeout: float) -> str:
    deadline = time.monotonic() + timeout
    last = ""
    while time.monotonic() < deadline:
        last = serial_text(path)
        if predicate(last):
            return last
        time.sleep(0.25)
    tail = last[-2500:]
    raise RuntimeError(f"timed out waiting for {what}\n--- serial tail ---\n{tail}")


def read_ppm(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if not data.startswith(b"P6"):
        raise RuntimeError(f"{path} is not a P6 PPM")
    fields: list[bytes] = []
    at = 2
    while len(fields) < 3:
        while at < len(data) and data[at : at + 1].isspace():
            at += 1
        if data[at : at + 1] == b"#":
            at = data.find(b"\n", at) + 1
            continue
        end = at
        while end < len(data) and not data[end : end + 1].isspace():
            end += 1
        fields.append(data[at:end])
        at = end
    width, height, _maximum = (int(field) for field in fields)
    at += 1
    pixels = data[at : at + width * height * 3]
    if len(pixels) != width * height * 3:
        raise RuntimeError(f"truncated raster in {path}")
    return width, height, pixels


def frame_stats(path: Path) -> dict:
    width, height, pixels = read_ppm(path)
    colourful = bright = dark = 0
    for at in range(0, len(pixels), 3):
        r, g, b = pixels[at], pixels[at + 1], pixels[at + 2]
        if max(r, g, b) - min(r, g, b) > 80:
            colourful += 1
        if r > 200 and g > 200 and b > 200:
            bright += 1
        if r < 40 and g < 40 and b < 40:
            dark += 1
    return {
        "width": width,
        "height": height,
        "colourful": colourful,
        "bright": bright,
        "dark": dark,
    }


def png(path: Path, out: Path) -> None:
    """Turn the QEMU PPM into a viewable PNG with the repository converter."""
    converter = ROOT / "tools" / "ppm2png.py"
    if not converter.exists():
        return
    try:
        subprocess.run(
            [sys.executable, str(converter), str(path), str(out)],
            check=True,
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError as error:
        print(f"warning: could not convert {path}: {error.stderr}", file=sys.stderr)


def framebuffer_line(text: str) -> str | None:
    for line in reversed(text.splitlines()):
        if line.startswith("litric.shell.drm="):
            return line.strip()
        if line.startswith("litric.shell.fb="):
            return line.strip()
    return None


def run(
    qmp_path: str,
    serial_path: str,
    out_dir: Path,
    timeout: float,
    width: int = W,
    height: int = H,
    net_port: int = 18080,
) -> dict:
    if width < 320 or height < 200:
        raise RuntimeError(f"invalid guest geometry {width}x{height}")
    out_dir.mkdir(parents=True, exist_ok=True)
    # A failed run must not leave an earlier run's PASS-looking result behind.
    (out_dir / "result.json").unlink(missing_ok=True)
    serial = Path(serial_path)
    ui_scale = max(width / BASE_W, 0.75)
    stop = stop_point(width)
    q = QMP(qmp_path, width, height)

    print(
        f"waiting for the LitricOS desktop at {width}x{height} and its absolute pointer",
        flush=True,
    )
    wait_serial(
        serial,
        lambda text: "litric.shell.mode=ready" in text
        and "input.pointer=true" in text
        and framebuffer_matches(text, width, height),
        f"shell ready at exactly {width}x{height} with an absolute pointer",
        timeout,
    )

    network_text = wait_serial(
        serial,
        lambda text: "LITRIC_NETWORK_OK" in text or "LITRIC_NETWORK_FAILED" in text,
        "guest network probe result",
        timeout,
    )
    if "LITRIC_NETWORK_OK" not in network_text:
        raise RuntimeError("the guest network probe did not reach the host HTTP server")

    # The image used by this proof pre-seeds onboarded=1. If a different rootfs
    # still shows first-run setup, the driver says so instead of typing into a
    # dialog whose controls are unrelated to the Games page.
    if "litric.shell.window=" not in serial_text(serial):
        q.move(width // 2, height // 2)
        time.sleep(1)
        q.key("meta_l")
        time.sleep(1)
        q.text("games")
        time.sleep(1)
        launcher = out_dir / "launcher.ppm"
        q.screenshot(launcher)
        png(launcher, out_dir / "launcher.png")
        q.key("ret")
    else:
        raise RuntimeError(
            "the guest already has a window before the proof started; "
            "use a fresh guest boot"
        )

    wait_serial(
        serial,
        lambda text: "com.litric.settings" in text,
        "Settings window from the Games search result",
        timeout,
    )
    games = out_dir / "games.ppm"
    q.screenshot(games)
    png(games, out_dir / "games.png")
    settings_frames = window_frames(serial_text(serial))
    if not settings_frames:
        raise RuntimeError("the status has no Settings window geometry")
    if not all(frame_inside(frame, width, height) for frame in settings_frames):
        raise RuntimeError(
            f"the shell window is outside the {width}x{height} framebuffer: {settings_frames}"
        )
    settings_frame = next(
        (frame for frame in settings_frames if frame["app"] == "com.litric.settings"),
        None,
    )
    if settings_frame is None:
        raise RuntimeError(f"the status has no com.litric.settings frame: {settings_frames}")
    play = play_point(settings_frame, width)

    print(f"clicking Play at {play} on the installed game", flush=True)
    q.click(*play)
    game_text = wait_serial(
        serial,
        lambda text: has_client_window(text, width, height),
        f"fullscreen litric.client window at exactly {width}x{height}",
        timeout,
    )
    before_windows = last_window_count(game_text)
    game_frames = window_frames(game_text)
    if before_windows != 2:
        raise RuntimeError(f"expected 2 shell windows with the game open, got {before_windows}")
    if not all(frame_inside(frame, width, height) for frame in game_frames):
        raise RuntimeError(
            f"a shell window is outside the {width}x{height} framebuffer: {game_frames}"
        )
    client_line = client_window_line(game_text, width, height)
    if client_line is None:
        raise RuntimeError(f"no client window geometry at {width}x{height} in the status")

    print("waiting for the real client's pixels", flush=True)
    game_shot = out_dir / "game.ppm"
    game_stats = {}
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        q.screenshot(game_shot)
        game_stats = frame_stats(game_shot)
        if game_stats["colourful"] >= GAME_COLOUR_MIN:
            break
        time.sleep(0.5)
    png(game_shot, out_dir / "game.png")
    if game_stats.get("colourful", 0) < GAME_COLOUR_MIN:
        raise RuntimeError(
            f"the client window mapped, but its pixels did not reach the scanout: {game_stats}"
        )
    if (game_stats["width"], game_stats["height"]) != (width, height):
        raise RuntimeError(
            f"screenshot is {game_stats['width']}x{game_stats['height']}, expected {width}x{height}"
        )

    # Status blocks are written every five seconds. A read that catches a
    # half-written or simply stale block must not count as "the client is gone".
    stop_mark = len(serial_text(serial))
    print(f"clicking Stop at {stop}", flush=True)
    q.click(*stop)
    wait_serial(
        serial,
        lambda text: client_gone_after_stop(text, stop_mark),
        "fresh windows=1 console state after Stop",
        timeout,
    )
    time.sleep(1)
    stopped_shot = out_dir / "stopped.ppm"
    q.screenshot(stopped_shot)
    stopped_stats = frame_stats(stopped_shot)
    png(stopped_shot, out_dir / "stopped.png")
    if stopped_stats["colourful"] > STOPPED_COLOUR_MAX:
        raise RuntimeError(
            f"the game's pixels are still on the scanout after Stop: {stopped_stats}"
        )
    if (stopped_stats["width"], stopped_stats["height"]) != (width, height):
        raise RuntimeError(
            f"stopped screenshot is {stopped_stats['width']}x{stopped_stats['height']}, "
            f"expected {width}x{height}"
        )

    post_stop_serial = serial_text(serial)
    final_serial = post_stop_serial
    after_windows = last_window_count(final_serial)
    stopped_frames = window_frames(final_serial)
    if after_windows != 1:
        raise RuntimeError(f"expected 1 shell window after Stop, got {after_windows}")
    if not all(frame_inside(frame, width, height) for frame in stopped_frames):
        raise RuntimeError(
            f"a shell window is outside the {width}x{height} framebuffer after Stop: {stopped_frames}"
        )

    # --- the browser: a real engine rendering a real page, in this guest ----
    print("opening the browser from the launcher", flush=True)
    q.key("meta_l")
    time.sleep(1)
    q.text("browser")
    time.sleep(1)
    launcher_shot = out_dir / "browser-launcher.ppm"
    q.screenshot(launcher_shot)
    png(launcher_shot, out_dir / "browser-launcher.png")
    q.key("ret")
    browser_text = wait_serial(
        serial,
        lambda text: "com.litric.browser" in text,
        "the browser window from the launcher",
        timeout,
    )
    browser_frame = next(
        (
            frame
            for frame in window_frames(browser_text)
            if frame["app"] == "com.litric.browser"
        ),
        None,
    )
    if browser_frame is None:
        raise RuntimeError(
            f"the shell has no com.litric.browser frame: {window_frames(browser_text)}"
        )
    if not frame_inside(browser_frame, width, height):
        raise RuntimeError(
            f"the browser window is outside the {width}x{height} framebuffer: {browser_frame}"
        )

    print("loading https://example.com/ from the address bar", flush=True)

    # The field the shell says is focused is the only evidence that a click
    # landed on it, and opening the app leaves it focused already. Escape first,
    # so a point can only be accepted when the click itself flips the field from
    # false to true. The old scan accepted the first point while the field was
    # already focused and then found itself clicking the Downloads button after
    # the next tab switch.
    address_click: list[tuple[int, int]] = []

    def focus_address() -> None:
        # The omnibox is a shell-owned field, not a TextEdit: a click sets
        # `omnibox_fresh`, which is the shell's own promise that the next typed
        # character replaces the address. Focus alone (Ctrl+T, tab switches)
        # does not set it, and keyboard selection shortcuts never reach this
        # field. Clicking until the diagnostic says fresh is therefore the
        # only measured focus this driver can trust.
        # The shell's own hotspot is the authoritative centre of the field.
        # Its status block can lag by tens of seconds under TCG, so waiting for
        # `browser.omnibox_fresh=true` after an otherwise-correct click made the
        # driver click a fallback over the Downloads button and destroy its own
        # focus. Click the hotspot, let the input land, and let the navigation
        # assertion prove the focus; do not infer it from a delayed snapshot.
        q.key("esc")
        time.sleep(0.2)
        exact = hotspot_click(serial_text(serial), "browser.omnibox_click")
        point = exact
        if point is None:
            point = browser_click_points(browser_frame, ui_scale)[0]
        q.move(10, 10)
        time.sleep(0.2)
        q.click(*point)
        q.click(*point)
        time.sleep(1.0)
        if not address_click:
            address_click.append(point)

    focus_address()
    q.text("example.com")
    time.sleep(0.5)
    q.key("ret")
    wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.title") == "Example Domain"
        and (latest_browser_value(text, "browser.frame") or "").endswith("bytes"),
        "the engine rendering https://example.com/ (title and frame)",
        timeout,
    )
    first_serial = serial_text(serial)
    first_tab_url = latest_browser_value(first_serial, "browser.tab")
    first_title = latest_browser_value(first_serial, "browser.title")

    browser_shot = out_dir / "browser.ppm"
    browser_stats: dict = {}
    bright_min = max(40_000, (width * height) // 20)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        q.screenshot(browser_shot)
        browser_stats = frame_stats(browser_shot)
        if browser_stats["bright"] >= bright_min:
            break
        time.sleep(0.5)
    png(browser_shot, out_dir / "browser.png")
    if browser_stats.get("bright", 0) < bright_min:
        raise RuntimeError(
            f"the page has a frame, but its pixels did not reach the scanout: {browser_stats}"
        )
    if (browser_stats["width"], browser_stats["height"]) != (width, height):
        raise RuntimeError(
            f"browser screenshot is {browser_stats['width']}x{browser_stats['height']}, "
            f"expected {width}x{height}"
        )

    # --- two tabs, both live ------------------------------------------------
    print("opening a second tab with Ctrl+T", flush=True)
    q.chord("ctrl", "t")
    tabs_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "2 active=1",
        "two browser tabs with the new one focused",
        timeout,
    )
    focus_address()
    q.text("example.org")
    time.sleep(0.5)
    q.key("ret")
    second_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tab") == "https://example.org/"
        and latest_browser_value(text, "browser.title") == "Example Domain",
        "the second tab rendering https://example.org/",
        timeout,
    )
    time.sleep(2)
    tabs_shot = out_dir / "browser-tabs.ppm"
    q.screenshot(tabs_shot)
    tabs_stats = frame_stats(tabs_shot)
    png(tabs_shot, out_dir / "browser-tabs.png")
    if (tabs_stats["width"], tabs_stats["height"]) != (width, height):
        raise RuntimeError(
            f"two-tab screenshot is {tabs_stats['width']}x{tabs_stats['height']}, "
            f"expected {width}x{height}"
        )

    print("switching back to the first tab with Ctrl+1", flush=True)
    q.chord("ctrl", "1")
    back_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tab") == "https://example.com/"
        and latest_browser_value(text, "browser.title") == "Example Domain",
        "the first tab still rendering https://example.com/",
        timeout,
    )

    # --- four tabs: two pages, two download targets ------------------------
    # A response that becomes a download has no document, so a tab that hosted
    # one cannot stand in for an ordinary page afterwards. The two page tabs
    # stay untouched; the downloads get their own tabs, and those are the tabs
    # Ctrl+W removes. This keeps the tab assertions about pages the engine can
    # actually render, not about Chromium's download error document.
    print("opening two download tabs with Ctrl+T", flush=True)
    for expected in (3, 4):
        q.chord("ctrl", "t")
        wait_serial(
            serial,
            lambda text, expected=expected: latest_browser_value(text, "browser.tabs")
            == f"{expected} active={expected - 1}",
            f"{expected} browser tabs with the new one focused",
            timeout,
        )

    large, small = DOWNLOADS
    print(f"starting the {large['name']} download in the third tab", flush=True)
    q.chord("ctrl", "3")
    wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "4 active=2",
        "the third tab selected for the large download",
        timeout,
    )
    focus_address()
    q.text(large["url"])
    time.sleep(0.5)
    q.key("ret")

    # A partial `.crdownload` of at least a megabyte is the guest's own proof
    # that the first transfer is still running. The second download is started
    # only after that line exists, so the overlap is measured from the guest's
    # disk rather than assumed from the order of two keystrokes.
    print("waiting for the large transfer to be visibly in flight", flush=True)
    active_text = wait_serial(
        serial,
        lambda text: any(
            event.get("size", "0").isdigit()
            and int(event.get("size", "0")) >= 1_000_000
            for event in download_events(text, "LITRIC_DOWNLOAD_ACTIVE")
        ),
        "a partial large download in the guest's staging directory",
        timeout,
    )
    large_active = any(
        event.get("size", "0").isdigit() and int(event.get("size", "0")) >= 1_000_000
        for event in download_events(active_text, "LITRIC_DOWNLOAD_ACTIVE")
    )

    print(f"starting the {small['name']} download in the fourth tab", flush=True)
    q.chord("ctrl", "4")
    wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "4 active=3",
        "the fourth tab selected for the small download",
        timeout,
    )
    focus_address()
    q.text(small["url"])
    time.sleep(0.5)
    q.key("ret")

    print("waiting for both downloads to finish staging", flush=True)
    staged_text = wait_serial(
        serial,
        lambda text: all(
            any(
                event.get("name") == item["file"]
                and event.get("size") == str(item["size"])
                and event.get("sha256") == item["sha256"]
                for event in download_events(text, "LITRIC_DOWNLOAD_STAGED")
            )
            for item in DOWNLOADS
        ),
        "both downloads staged with their declared size and digest",
        timeout,
    )

    # The Downloads panel is what asks the broker for finished files and hands
    # them to the file service. Its toolbar button sits under the address bar;
    # the shell reports its exact centre, and the ladder remains as a fallback.
    print("opening the Downloads panel", flush=True)
    panel_click = None
    exact_panel = hotspot_click(serial_text(serial), "browser.downloads_click")
    if exact_panel is not None:
        q.click(*exact_panel)
        q.click(*exact_panel)
        panel_deadline = time.monotonic() + 8
        while time.monotonic() < panel_deadline:
            if latest_browser_value(serial_text(serial), "browser.panel") == "downloads":
                panel_click = exact_panel
                break
            time.sleep(0.4)
    for dx, dy in ((523, 108), (505, 108), (541, 108), (523, 86), (480, 108), (566, 108)):
        if panel_click is not None:
            break
        point = (
            browser_frame["x"] + round(dx * ui_scale),
            browser_frame["y"] + round(dy * ui_scale),
        )
        q.click(*point)
        q.click(*point)
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if latest_browser_value(serial_text(serial), "browser.panel") == "downloads":
                panel_click = point
                break
            time.sleep(0.5)
        if panel_click is not None:
            break
    if panel_click is None:
        raise RuntimeError("the Downloads toolbar button never opened its panel")

    download_text = wait_serial(
        serial,
        lambda text: all(
            any(
                event.get("name") == item["file"]
                and event.get("size") == str(item["size"])
                and event.get("sha256") == item["sha256"]
                for event in download_events(text, "LITRIC_DOWNLOAD_SAVED")
            )
            for item in DOWNLOADS
        )
        and latest_browser_value(text, "browser.panel") == "downloads"
        and latest_browser_value(text, "browser.downloads") == "2"
        and latest_browser_value(text, "browser.tabs") == "4 active=3",
        "both downloads imported into /Downloads and listed in the panel",
        timeout,
    )
    time.sleep(2)
    downloads_shot = out_dir / "browser-downloads.ppm"
    q.screenshot(downloads_shot)
    downloads_stats = frame_stats(downloads_shot)
    png(downloads_shot, out_dir / "browser-downloads.png")
    if (downloads_stats["width"], downloads_stats["height"]) != (width, height):
        raise RuntimeError(
            f"downloads screenshot is {downloads_stats['width']}x"
            f"{downloads_stats['height']}, expected {width}x{height}"
        )

    # Close only the two download tabs. Both page tabs keep the documents they
    # loaded before the downloads started; no page is restored from a download
    # error document.
    print("closing the small-download tab with Ctrl+W", flush=True)
    q.chord("ctrl", "4")
    wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "4 active=3"
        and latest_browser_value(text, "browser.panel") == "page",
        "the small-download tab selected with the page panel",
        timeout,
    )
    q.key("esc")
    q.chord("ctrl", "w")
    first_close_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "3 active=2",
        "the small-download tab closed, leaving the large-download tab",
        timeout,
    )
    print("closing the large-download tab with Ctrl+W", flush=True)
    q.key("esc")
    q.chord("ctrl", "w")
    closed_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "2 active=1"
        and latest_browser_value(text, "browser.tab") == "https://example.org/"
        and latest_browser_value(text, "browser.title") == "Example Domain"
        and (latest_browser_value(text, "browser.frame") or "").endswith("bytes")
        and (latest_browser_value(text, "browser.status") or "").startswith("loaded"),
        "the two page tabs left on https://example.org/ after two Ctrl+W closes",
        timeout,
    )
    time.sleep(2)
    closed_shot = out_dir / "browser-tabs-closed.ppm"
    q.screenshot(closed_shot)
    closed_stats = frame_stats(closed_shot)
    png(closed_shot, out_dir / "browser-tabs-closed.png")
    if (closed_stats["width"], closed_stats["height"]) != (width, height):
        raise RuntimeError(
            f"closed-tab screenshot is {closed_stats['width']}x"
            f"{closed_stats['height']}, expected {width}x{height}"
        )

    # With two page tabs left, exercise every keyboard path the requirement
    # names: Ctrl+1, Ctrl+2 and Ctrl+Tab. Each wait requires the shell's active
    # URL, title, loaded status and a broker frame, so a stale title cannot pass.
    print("switching to the first page tab with Ctrl+1", flush=True)
    q.chord("ctrl", "1")
    after_one_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "2 active=0"
        and latest_browser_value(text, "browser.tab") == "https://example.com/"
        and latest_browser_value(text, "browser.title") == "Example Domain"
        and (latest_browser_value(text, "browser.frame") or "").endswith("bytes")
        and (latest_browser_value(text, "browser.status") or "").startswith("loaded"),
        "Ctrl+1 selecting the surviving first tab",
        timeout,
    )
    print("switching to the second page tab with Ctrl+2", flush=True)
    q.chord("ctrl", "2")
    after_two_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "2 active=1"
        and latest_browser_value(text, "browser.tab") == "https://example.org/"
        and latest_browser_value(text, "browser.title") == "Example Domain"
        and (latest_browser_value(text, "browser.frame") or "").endswith("bytes")
        and (latest_browser_value(text, "browser.status") or "").startswith("loaded"),
        "Ctrl+2 selecting the second page tab",
        timeout,
    )
    print("wrapping to the first tab with Ctrl+Tab", flush=True)
    q.chord("ctrl", "tab")
    after_cycle_text = wait_serial(
        serial,
        lambda text: latest_browser_value(text, "browser.tabs") == "2 active=0"
        and latest_browser_value(text, "browser.tab") == "https://example.com/"
        and latest_browser_value(text, "browser.title") == "Example Domain"
        and (latest_browser_value(text, "browser.frame") or "").endswith("bytes")
        and (latest_browser_value(text, "browser.status") or "").startswith("loaded"),
        "Ctrl+Tab wrapping to the first tab",
        timeout,
    )
    time.sleep(2)
    switched_shot = out_dir / "browser-tabs-switched.ppm"
    q.screenshot(switched_shot)
    switched_stats = frame_stats(switched_shot)
    png(switched_shot, out_dir / "browser-tabs-switched.png")
    if (switched_stats["width"], switched_stats["height"]) != (width, height):
        raise RuntimeError(
            f"switched-tab screenshot is {switched_stats['width']}x"
            f"{switched_stats['height']}, expected {width}x{height}"
        )

    staged_events = download_events(staged_text, "LITRIC_DOWNLOAD_STAGED")
    saved_events = download_events(download_text, "LITRIC_DOWNLOAD_SAVED")
    downloads = []
    for item in DOWNLOADS:
        staged = next((event for event in staged_events if event.get("name") == item["file"]), {})
        saved = next((event for event in saved_events if event.get("name") == item["file"]), {})
        downloads.append(
            {
                "candidate": item["name"],
                "url": "https://" + item["url"],
                "name": item["file"],
                "size": saved.get("size", ""),
                "sha256": saved.get("sha256", ""),
                "expected_size": str(item["size"]),
                "expected_sha256": item["sha256"],
                "staged_size": staged.get("size", ""),
                "staged_sha256": staged.get("sha256", ""),
                "staged": staged.get("size") == str(item["size"])
                and staged.get("sha256") == item["sha256"],
                "saved": saved.get("size") == str(item["size"])
                and saved.get("sha256") == item["sha256"],
                "large": item["large"],
            }
        )
    final_serial = serial_text(serial)
    gpu = {
        "present": latest_prefixed(final_serial, "litric.shell.present="),
        "card": latest_prefixed(final_serial, "litric.shell.gpu.card="),
        "driver": latest_prefixed(final_serial, "litric.shell.gpu.driver="),
        "vendor": latest_prefixed(final_serial, "litric.shell.gpu.vendor="),
        "device": latest_prefixed(final_serial, "litric.shell.gpu.device="),
        "render": latest_prefixed(final_serial, "litric.shell.gpu.render="),
        "kms_mode": latest_prefixed(final_serial, "litric.shell.drm.mode="),
        "connector": latest_prefixed(final_serial, "litric.shell.drm.connector="),
        "flip": latest_prefixed(final_serial, "LITRIC_DRM_FLIP="),
    }
    browser_viewport = latest_browser_value(final_serial, "browser.viewport")
    close_state = {
        "tabs": latest_browser_value(closed_text, "browser.tabs"),
        "url": latest_browser_value(closed_text, "browser.tab"),
        "title": latest_browser_value(closed_text, "browser.title"),
        "status": latest_browser_value(closed_text, "browser.status"),
        "frame": latest_browser_value(closed_text, "browser.frame"),
    }
    after_one_state = {
        "tabs": latest_browser_value(after_one_text, "browser.tabs"),
        "url": latest_browser_value(after_one_text, "browser.tab"),
        "title": latest_browser_value(after_one_text, "browser.title"),
        "status": latest_browser_value(after_one_text, "browser.status"),
        "frame": latest_browser_value(after_one_text, "browser.frame"),
    }
    after_two_state = {
        "tabs": latest_browser_value(after_two_text, "browser.tabs"),
        "url": latest_browser_value(after_two_text, "browser.tab"),
        "title": latest_browser_value(after_two_text, "browser.title"),
        "status": latest_browser_value(after_two_text, "browser.status"),
        "frame": latest_browser_value(after_two_text, "browser.frame"),
    }
    after_cycle_state = {
        "tabs": latest_browser_value(after_cycle_text, "browser.tabs"),
        "url": latest_browser_value(after_cycle_text, "browser.tab"),
        "title": latest_browser_value(after_cycle_text, "browser.title"),
        "status": latest_browser_value(after_cycle_text, "browser.status"),
        "frame": latest_browser_value(after_cycle_text, "browser.frame"),
    }
    browser_summary = {
        "window": browser_frame,
        "tab": first_tab_url,
        "title": first_title,
        "tabs": latest_browser_value(tabs_text, "browser.tabs"),
        "second_tab": latest_browser_value(second_text, "browser.tab"),
        "switched_back": latest_browser_value(back_text, "browser.tab"),
        "downloads": downloads,
        "download": downloads[0],
        "download_count": latest_browser_value(download_text, "browser.downloads"),
        "download_panel": latest_browser_value(download_text, "browser.panel"),
        "download_click": list(panel_click),
        "downloads_concurrent": large_active,
        "downloads_screenshot": downloads_stats,
        "tabs_before_close": latest_browser_value(download_text, "browser.tabs"),
        "tabs_after_first_close": latest_browser_value(first_close_text, "browser.tabs"),
        "tabs_after_close": close_state,
        "tabs_after_switch_one": after_one_state,
        "tabs_after_switch_two": after_two_state,
        "tabs_after_cycle": after_cycle_state,
        "tabs_closed_screenshot": closed_stats,
        "tabs_switched_screenshot": switched_stats,
        "final_status": latest_browser_value(final_serial, "browser.status"),
        "frame": latest_browser_value(final_serial, "browser.frame"),
        "viewport": browser_viewport,
        "engine_chromium": "engine = chromium" in final_serial,
        "gpu_policy": next(
            (
                line.strip()
                for line in final_serial.splitlines()
                if "browserd: gpu=" in line
            ),
            None,
        ),
        "url": "https://example.com/",
        **browser_stats,
    }
    browser_assertions = {
        "browser_window": browser_frame is not None,
        "browser_window_inside_framebuffer": bool(browser_frame)
        and frame_inside(browser_frame, width, height),
        "browser_engine_chromium": browser_summary["engine_chromium"],
        "browser_gpu_software_fallback": bool(browser_summary["gpu_policy"])
        and "gpu=software" in browser_summary["gpu_policy"],
        "browser_https_url": browser_summary["tab"] == "https://example.com/",
        "browser_title": browser_summary["title"] == "Example Domain",
        "browser_frame_received": (browser_summary["frame"] or "none").endswith("bytes"),
        "browser_viewport": browser_viewport not in (None, "none"),
        "browser_pixels": browser_stats.get("bright", 0) >= bright_min,
        "browser_screenshot_at_expected_size": (
            browser_stats.get("width"),
            browser_stats.get("height"),
        )
        == (width, height),
        "browser_two_tabs": latest_browser_value(tabs_text, "browser.tabs") == "2 active=1",
        "browser_second_tab_loaded": latest_browser_value(second_text, "browser.tab")
        == "https://example.org/",
        "browser_tab_switch_back": latest_browser_value(back_text, "browser.tab")
        == "https://example.com/",
        "browser_download_active": large_active,
        "browser_download_staged": all(download["staged"] for download in downloads),
        "browser_download_saved": all(download["saved"] for download in downloads),
        "browser_downloads_concurrent": large_active
        and all(download["staged"] and download["saved"] for download in downloads),
        "browser_download_large": downloads[0]["saved"]
        and int(downloads[0]["expected_size"]) >= 5_000_000,
        "browser_download_listed": latest_browser_value(download_text, "browser.panel")
        == "downloads"
        and latest_browser_value(download_text, "browser.downloads") == "2"
        and latest_browser_value(download_text, "browser.tabs") == "4 active=3",
        "browser_download_screenshot_at_expected_size": (
            downloads_stats.get("width"),
            downloads_stats.get("height"),
        )
        == (width, height),
        "browser_download_tabs_closed": latest_browser_value(download_text, "browser.tabs")
        == "4 active=3"
        and latest_browser_value(first_close_text, "browser.tabs") == "3 active=2"
        and close_state["tabs"] == "2 active=1",
        "browser_tab_closed": close_state["tabs"] == "2 active=1"
        and close_state["url"] == "https://example.org/"
        and close_state["title"] == "Example Domain"
        and (close_state["status"] or "").startswith("loaded")
        and (close_state["frame"] or "").endswith("bytes"),
        "browser_tab_switch_after_close": after_one_state["tabs"] == "2 active=0"
        and after_one_state["url"] == "https://example.com/"
        and (after_one_state["status"] or "").startswith("loaded")
        and (after_one_state["frame"] or "").endswith("bytes")
        and after_two_state["tabs"] == "2 active=1"
        and after_two_state["url"] == "https://example.org/"
        and (after_two_state["status"] or "").startswith("loaded")
        and (after_two_state["frame"] or "").endswith("bytes"),
        "browser_tab_cycle_after_close": after_cycle_state["tabs"] == "2 active=0"
        and after_cycle_state["url"] == "https://example.com/"
        and (after_cycle_state["status"] or "").startswith("loaded")
        and (after_cycle_state["frame"] or "").endswith("bytes"),
        "browser_final_status_loaded": (
            latest_browser_value(final_serial, "browser.status") or ""
        ).startswith("loaded"),
        "gpu_present_drm": gpu["present"] == "drm",
        "gpu_card_node": bool(gpu["card"])
        and (gpu["card"] or "").startswith("/dev/dri/card"),
        "gpu_driver_named": bool(gpu["driver"])
        and gpu["driver"] != "unknown",
        "gpu_kms_mode": gpu["kms_mode"] == f"{width}x{height}",
        "gpu_kms_connector": bool(gpu["connector"])
        and gpu["connector"] != "unknown",
        "gpu_pageflip": gpu["flip"] == "ok",
        "browser_tabs_closed_screenshot_at_expected_size": (
            closed_stats.get("width"),
            closed_stats.get("height"),
        )
        == (width, height),
        "browser_tabs_switched_screenshot_at_expected_size": (
            switched_stats.get("width"),
            switched_stats.get("height"),
        )
        == (width, height),
    }

    assertions = {
        "guest_network": "LITRIC_NETWORK_OK" in final_serial,
        "installed": "state=installed" in final_serial,
        "client_window_gone_after_stop": client_gone_after_stop(post_stop_serial, stop_mark),
        "colourful_above_floor": game_stats["colourful"] >= GAME_COLOUR_MIN,
        "colourful_below_ceiling_after_stop": stopped_stats["colourful"]
        <= STOPPED_COLOUR_MAX,
    }
    geometry_checks = {
        "framebuffer": framebuffer_line(game_text) is not None,
        "client_fullscreen_at_expected_size": client_line is not None,
        "game_screenshot_at_expected_size": (game_stats["width"], game_stats["height"])
        == (width, height),
        "stopped_screenshot_at_expected_size": (
            stopped_stats["width"],
            stopped_stats["height"],
        )
        == (width, height),
        "settings_window_inside_framebuffer": all(
            frame_inside(frame, width, height) for frame in settings_frames
        ),
        "settings_window_scaled": (
            settings_frame["width"],
            settings_frame["height"],
        )
        == (round(980 * ui_scale), round(620 * ui_scale)),
        "windows_inside_framebuffer": all(
            frame_inside(frame, width, height) for frame in game_frames + stopped_frames
        ),
        "window_counts": before_windows == 2 and after_windows == 1,
    }
    assertions.update(browser_assertions)
    if not all(assertions.values()) or not all(geometry_checks.values()):
        raise RuntimeError(
            f"proof assertions failed: assertions={assertions} geometry={geometry_checks}"
        )
    result = {
        "qmp": qmp_path,
        "serial": serial_path,
        "framebuffer": {
            "width": width,
            "height": height,
            "status_line": framebuffer_line(game_text),
        },
        "network": {
            "probe": f"http://10.0.2.2:{net_port}/proof.txt",
            "ok": "LITRIC_NETWORK_OK" in final_serial,
        },
        "game": {
            "play": list(play),
            "window": "litric.client fullscreen",
            "window_line": client_line,
            "shell_windows": before_windows,
            "frames": game_frames,
            **game_stats,
        },
        "stopped": {
            "stop": list(stop),
            "shell_windows": after_windows,
            "frames": stopped_frames,
            **stopped_stats,
        },
        "browser": browser_summary,
        "gpu": gpu,
        "assertions": assertions,
        "geometry_checks": geometry_checks,
    }
    (out_dir / "result.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--qmp", required=True)
    parser.add_argument("--serial", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--width", type=int, default=W)
    parser.add_argument("--height", type=int, default=H)
    parser.add_argument("--net-port", type=int, default=18080)
    parser.add_argument("--timeout", type=float, default=90.0)
    args = parser.parse_args()
    try:
        result = run(
            args.qmp,
            args.serial,
            Path(args.out),
            args.timeout,
            args.width,
            args.height,
            args.net_port,
        )
    except Exception as error:
        print(f"QEMU GUEST PROOF: FAIL: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    print("QEMU GUEST PROOF: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
