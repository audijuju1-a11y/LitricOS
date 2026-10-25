#!/usr/bin/env python3
"""End-to-end check of the Litric browser engine: broker ⇄ Chromium ⇄ frames.

This drives `litric-browserd` over its real LBP/1 Unix socket, exactly as the
shell does, and checks what actually comes back: a tab state, a frame of the
right size, and a picture that changes when the page is scrolled. It is not a
mock: Chromium renders, the broker decodes the screenshot, and the pixels are
in this process.

Stages, each printed with PASS / FAIL / SKIP so a failure names its step:

    handshake   HELLO answers with the engine it owns
    keepalive   PING ⇄ PONG
    policy      file:, http: and unknown commands are refused by name
    start page  about:blank answers without waking the engine
    navigation  a real HTTPS page reports loaded, a title and a frame
    rendering   the frame is the announced size and is not a blank fill
    scroll      a wheel event changes the picture
    input       click, key and text are accepted and the page still answers
    tabs        a second tab opens, loads, closes
    shutdown    QUIT ends the process

Usage:

    # against a broker already running on its socket
    tools/browser-e2e.py --socket /run/litric/browser.sock

    # start one from this tree (Chromium must exist, or is taken from
    # BROWSERD_CHROMIUM / --chromium)
    tools/browser-e2e.py --broker src/target/debug/litric-browserd \\
        --chromium /usr/bin/chromium

Options:
    --url URL          the HTTPS page to load (default https://example.com/)
    --out DIR          where PNG snapshots go (default build/e2e-browser/<ts>)
    --require-engine   treat "no Chromium" as a failure instead of a skip
    --timeout SECONDS  per stage (default 45)

Exit status: 0 when every stage passed or was skipped, 1 when a stage failed.
"""

import argparse
import os
import socket
import struct
import subprocess
import sys
import time
import zlib


# --------------------------------------------------------------------------
# LBP/1 client
# --------------------------------------------------------------------------

class Broker:
    """One LBP/1 connection: line commands, line replies, FRAME payloads."""

    def __init__(self, sock, timeout):
        self.sock = sock
        self.buffer = b""
        self.timeout = timeout
        self.frames = []

    def send(self, command):
        self.sock.sendall(command.encode() + b"\n")

    def _fill(self, deadline):
        remaining = deadline - time.time()
        if remaining <= 0:
            return False
        self.sock.settimeout(min(remaining, 0.5))
        try:
            chunk = self.sock.recv(1 << 20)
        except socket.timeout:
            return True
        if not chunk:
            return False
        self.buffer += chunk
        return True

    def read_until(self, predicate, seconds=None):
        """Read replies until `predicate(lines)` is true or time runs out."""
        limit = self.timeout if seconds is None else seconds
        deadline = time.time() + limit
        lines = []
        start_size = len(self.frames)
        alive = True
        while time.time() < deadline:
            # Consume everything already buffered first.
            while b"\n" in self.buffer:
                line, self.buffer = self.buffer.split(b"\n", 1)
                text = line.decode("utf-8", "replace")
                if text.startswith("FRAME "):
                    parts = text.split(" ")
                    if len(parts) < 5:
                        raise RuntimeError(f"malformed frame header: {text!r}")
                    width, height, length = int(parts[2]), int(parts[3]), int(parts[4])
                    while len(self.buffer) < length:
                        if not self._fill(deadline):
                            raise RuntimeError("connection ended inside a frame payload")
                    pixels = self.buffer[:length]
                    self.buffer = self.buffer[length:]
                    self.frames.append((width, height, pixels))
                else:
                    lines.append(text)
            if predicate(lines):
                break
            if not self._fill(deadline):
                alive = False
                break
        return lines, alive, len(self.frames) > start_size

    def command(self, line, predicate=None, seconds=None):
        self.send(line)
        if predicate is None:
            predicate = lambda ls: bool(ls)  # noqa: E731
        return self.read_until(predicate, seconds)

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass


def tab_line(lines):
    for line in reversed(lines):
        if line.startswith("TAB "):
            return line
    return None


def errors(lines):
    return [line for line in lines if line.startswith("ERR ")]


def tab_fields(line):
    fields = {}
    parts = line.split(" ")
    if len(parts) < 2:
        return fields
    fields["tab"] = parts[1]
    for part in parts[2:]:
        if "=" in part:
            key, value = part.split("=", 1)
            fields[key] = value
    return fields


def percent_decode(value):
    out = bytearray()
    i = 0
    raw = value.encode()
    while i < len(raw):
        if raw[i] == 0x25 and i + 2 < len(raw):
            try:
                out.append(int(raw[i + 1:i + 3], 16))
                i += 3
                continue
            except ValueError:
                pass
        out.append(raw[i])
        i += 1
    return out.decode("utf-8", "replace")


# --------------------------------------------------------------------------
# Frame inspection
# --------------------------------------------------------------------------

def sampled_colours(pixels, step=37):
    """Distinct RGB values in a BGRA buffer, sampled to stay fast."""
    seen = set()
    for i in range(0, len(pixels) - 4, 4 * step):
        seen.add(bytes(pixels[i:i + 3]))
    return seen


def find_link_targets(width, height, pixels, limit=4):
    """Candidate link positions, most link-like first.

    Link text is the strongly blue thing on pages like example.com. Rows are
    grouped into bands so a click lands in the middle of a line of text rather
    than on a single-pixel edge of a glyph, and several bands are returned
    because a page can have blue that is not a link: the caller clicks through
    the candidates until one navigates. An empty list means this frame has no
    link-coloured text at all.
    """
    bands = []
    current = None
    for y in range(height):
        row = pixels[y * width * 4:(y + 1) * width * 4]
        hits = []
        for x in range(0, width, 2):
            b, g, r = row[x * 4], row[x * 4 + 1], row[x * 4 + 2]
            if b > 140 and r < 110 and g < 130 and b - r > 60:
                hits.append(x)
        if hits:
            if current is None:
                current = {"top": y, "bottom": y, "left": hits[0],
                           "right": hits[-1], "count": len(hits)}
            else:
                current["bottom"] = y
                current["left"] = min(current["left"], hits[0])
                current["right"] = max(current["right"], hits[-1])
                current["count"] += len(hits)
        elif current is not None:
            bands.append(current)
            current = None
    if current is not None:
        bands.append(current)
    bands.sort(key=lambda band: band["count"], reverse=True)
    return [((band["left"] + band["right"]) // 2,
             (band["top"] + band["bottom"]) // 2) for band in bands[:limit]]


def find_link_pixel(width, height, pixels):
    """The best link candidate, or None."""
    targets = find_link_targets(width, height, pixels, limit=1)
    return targets[0] if targets else None


def write_png(path, width, height, pixels):
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        row = pixels[y * width * 4:(y + 1) * width * 4]
        for x in range(width):
            b, g, r = row[x * 4], row[x * 4 + 1], row[x * 4 + 2]
            raw += bytes((r, g, b))

    def chunk(tag, data):
        return (struct.pack(">I", len(data)) + tag + data
                + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 6))
    png += chunk(b"IEND", b"")
    with open(path, "wb") as handle:
        handle.write(png)


# --------------------------------------------------------------------------
# The run
# --------------------------------------------------------------------------

class Report:
    def __init__(self):
        self.failures = 0
        self.skips = 0

    def _line(self, status, stage, detail):
        print(f"{status:<4} {stage:<11} {detail}", flush=True)

    def passed(self, stage, detail=""):
        self._line("PASS", stage, detail)

    def failed(self, stage, detail):
        self.failures += 1
        self._line("FAIL", stage, detail)

    def skipped(self, stage, detail):
        self.skips += 1
        self._line("SKIP", stage, detail)


def wait_for_socket(path, seconds=20):
    deadline = time.time() + seconds
    while time.time() < deadline:
        if os.path.exists(path):
            return True
        time.sleep(0.05)
    return False


def run(args):
    report = Report()
    out_dir = args.out or os.path.join(
        "build", "e2e-browser", time.strftime("%Y%m%d-%H%M%S"))
    os.makedirs(out_dir, exist_ok=True)

    broker = None
    socket_path = args.socket
    if args.broker:
        socket_path = args.socket or "/tmp/litric-browser-e2e.sock"
        try:
            os.unlink(socket_path)
        except FileNotFoundError:
            pass
        env = dict(os.environ)
        # The broker binds the socket named on its environment, so the harness
        # and the broker must agree on it: without this the broker binds the
        # system path and the harness waits for a socket nobody created.
        env["BROWSERD_SOCK"] = socket_path
        if args.chromium:
            env["BROWSERD_CHROMIUM"] = args.chromium
        log_path = os.path.join(out_dir, "browserd.log")
        log = open(log_path, "wb")
        broker = subprocess.Popen(
            [args.broker, "--profile", os.path.join(out_dir, "profile"),
             "--downloads", os.path.join(out_dir, "downloads")],
            env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        print(f"started {args.broker} (pid {broker.pid}), log: {log_path}", flush=True)
        if not wait_for_socket(socket_path):
            report.failed("start", f"the broker never created {socket_path}")
            broker.kill()
            return report
        report.passed("start", f"broker listening on {socket_path}")

    if not socket_path:
        report.failed("start", "no socket: pass --socket or --broker")
        return report

    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.settimeout(10)
    sock.connect(socket_path)
    client = Broker(sock, args.timeout)

    try:
        # --- handshake ----------------------------------------------------
        lines, _, _ = client.command("HELLO 1", lambda ls: bool(ls), 10)
        hello = lines[-1] if lines else ""
        engine = None
        if hello.startswith("HELLO 1 engine="):
            engine = hello.split("engine=", 1)[1].split(" ")[0]
            report.passed("handshake", hello)
        elif hello.startswith("ERR engine-unavailable"):
            reason = percent_decode(hello.split(" ", 2)[-1])
            if args.require_engine:
                report.failed("handshake", f"engine unavailable: {reason}")
            else:
                report.skipped("handshake", f"engine unavailable: {reason}")
        else:
            report.failed("handshake", f"unexpected reply: {hello!r}")

        # --- keepalive ----------------------------------------------------
        lines, _, _ = client.command("PING", lambda ls: bool(ls), 10)
        if lines and lines[-1] == "PONG":
            report.passed("keepalive", "PONG")
        else:
            report.failed("keepalive", f"expected PONG, got {lines!r}")

        # --- policy -------------------------------------------------------
        policy_ok = True
        for command, expect in [
            ("GO 1 file%3A%2F%2F%2Fetc%2Fshadow", "file"),
            ("GO 1 http%3A%2F%2Fexample.com%2F", "http"),
            ("JUNK", "bad-request"),
        ]:
            lines, _, _ = client.command(command, lambda ls: bool(ls), 20)
            text = lines[-1] if lines else ""
            if not text.startswith("ERR "):
                policy_ok = False
                report.failed("policy", f"{command} was accepted ({text!r})")
            elif expect not in text:
                policy_ok = False
                report.failed("policy", f"{command} refused for the wrong reason: {text}")
        if policy_ok:
            report.passed("policy", "file:, http: and unknown commands refused by name")

        # --- start page ---------------------------------------------------
        client.frames.clear()
        lines, _, got_frame = client.command(
            "GO 1 about%3Ablank",
            lambda ls: any(l.startswith("TAB ") for l in ls) or errors(ls),
            20,
        )
        tab = tab_line(lines)
        if tab and "state=empty" in tab and not got_frame:
            report.passed("start page", tab)
        else:
            report.failed("start page", f"{lines!r}")

        if engine is None:
            for stage in ("navigation", "rendering", "scroll", "input", "tabs"):
                report.skipped(stage, "needs a Chromium engine")
            return report

        # --- navigation ---------------------------------------------------
        client.frames.clear()
        lines, _, got_frame = client.command(
            f"GO 1 {percent_encode(args.url)}",
            lambda ls: any(l.startswith("TAB ") for l in ls) or errors(ls),
            args.timeout,
        )
        errs = errors(lines)
        tab = tab_line(lines)
        if not errs and tab is not None:
            # The TAB state line is written before the frame that shows the
            # page, so a navigation is only finished once its picture is here.
            more, _, _ = client.read_until(lambda ls: bool(client.frames), args.timeout)
            errs += errors(more)
        if errs:
            report.failed("navigation", errs[-1])
        elif tab is None:
            report.failed("navigation", f"no TAB reply: {lines!r}")
        else:
            fields = tab_fields(tab)
            url = percent_decode(fields.get("url", ""))
            if fields.get("state") != "loaded":
                report.failed("navigation", f"state is {fields.get('state')!r}: {tab}")
            elif not url.startswith("https://"):
                report.failed("navigation", f"loaded a non-HTTPS address: {url}")
            else:
                report.passed(
                    "navigation",
                    f"{url} title={percent_decode(fields.get('title', ''))!r}",
                )

        # --- rendering ----------------------------------------------------
        if not client.frames:
            report.failed("rendering", "no frame arrived after a navigation")
        else:
            width, height, pixels = client.frames[-1]
            colours = sampled_colours(pixels)
            snapshot = os.path.join(out_dir, "page.png")
            write_png(snapshot, width, height, pixels)
            if len(pixels) != width * height * 4:
                report.failed(
                    "rendering",
                    f"frame is {len(pixels)} bytes, {width}x{height} needs "
                    f"{width * height * 4}",
                )
            elif len(colours) < 8:
                report.failed(
                    "rendering",
                    f"the page is a {len(colours)}-colour fill: not a rendered site",
                )
            else:
                report.passed(
                    "rendering",
                    f"{width}x{height} BGRA, {len(colours)} sampled colours, {snapshot}",
                )

        # --- following a link ---------------------------------------------
        # The strongest evidence that a click reaches the page: click text that
        # is styled as a link and watch the tab's own address change. Several
        # candidates are tried because blue text is not always a link, and the
        # navigation is polled because it is asynchronous: a capture can arrive
        # before the new document does.
        if not client.frames:
            report.skipped("link", "no frame to click")
        else:
            width, height, pixels = client.frames[-1]
            targets = find_link_targets(width, height, pixels)
            if not targets:
                report.skipped("link", "this page has no link-coloured text")
            else:
                url = args.url
                moved = False
                failures = []
                for (x, y) in targets:
                    client.send(f"MOUSE 1 move {x} {y} none 0 0")
                    client.send(f"MOUSE 1 down {x} {y} left 1 0")
                    client.send(f"MOUSE 1 up {x} {y} left 1 0")
                    for _ in range(6):
                        client.frames.clear()
                        client.send("ACTIVATE 1")
                        lines, _, _ = client.read_until(
                            lambda ls: bool(client.frames) or errors(ls), args.timeout)
                        failures += errors(lines)
                        tab = tab_line(lines)
                        if tab is not None:
                            url = percent_decode(tab_fields(tab).get("url", ""))
                        if url.split("#")[0] != args.url.split("#")[0]:
                            moved = True
                            break
                        time.sleep(0.4)
                    if moved or failures:
                        break
                if failures:
                    report.failed("link", failures[-1])
                elif moved:
                    report.passed("link", f"clicked through to {url}")
                else:
                    tried = ", ".join(f"{x},{y}" for x, y in targets)
                    report.failed(
                        "link",
                        f"no candidate link navigated (tried {tried}; still {url})",
                    )

        # --- scroll -------------------------------------------------------
        if not client.frames:
            report.skipped("scroll", "no frame to scroll")
        else:
            width, height, before = client.frames[-1]
            client.frames.clear()
            client.send(f"WHEEL 1 {width // 2} {height // 2} 0 600")
            client.send("ACTIVATE 1")
            lines, _, got_frame = client.read_until(
                lambda ls: bool(client.frames) or errors(ls), args.timeout)
            if errors(lines):
                report.failed("scroll", errors(lines)[-1])
            elif not got_frame:
                report.failed("scroll", f"no frame after a wheel event: {lines!r}")
            elif client.frames[-1][2] == before:
                report.failed("scroll", "the picture did not change after scrolling")
            else:
                report.passed("scroll", "the page moved in response to the wheel")
                write_png(os.path.join(out_dir, "scrolled.png"),
                          client.frames[-1][0], client.frames[-1][1],
                          client.frames[-1][2])

        # --- input --------------------------------------------------------
        if not client.frames:
            report.skipped("input", "no frame to click")
        else:
            width, height, pixels = client.frames[-1]
            target = find_link_pixel(width, height, pixels) or (width // 2, height // 2)
            x, y = target
            client.frames.clear()
            for command in (
                f"MOUSE 1 move {x} {y} none 0 0",
                f"MOUSE 1 down {x} {y} left 1 0",
                f"MOUSE 1 up {x} {y} left 1 0",
                "KEY 1 down Tab Tab 9 0 %09",
                "KEY 1 up Tab Tab 9 0 -",
                "TEXT 1 litric",
            ):
                client.send(command)
            client.send("ACTIVATE 1")
            lines, _, got_frame = client.read_until(
                lambda ls: bool(client.frames) or errors(ls), args.timeout)
            if errors(lines):
                report.failed("input", errors(lines)[-1])
            elif not got_frame:
                report.failed("input", "the engine stopped answering after input")
            else:
                report.passed("input", f"click at {x},{y} and typed text were accepted")
                write_png(os.path.join(out_dir, "after-input.png"),
                          client.frames[-1][0], client.frames[-1][1],
                          client.frames[-1][2])

        # --- tabs ---------------------------------------------------------
        lines, _, _ = client.command("NEW 2", lambda ls: bool(ls), 20)
        new_tab = tab_line(lines) or (lines[-1] if lines else "")
        ok = "TAB 2" in new_tab and "state=empty" in new_tab
        client.frames.clear()
        lines, _, _ = client.command(
            f"GO 2 {percent_encode(args.url)}",
            lambda ls: any(l.startswith("TAB 2") for l in ls) or errors(ls),
            args.timeout,
        )
        # As above: the tab's frame follows its state line.
        if not errors(lines):
            client.read_until(lambda ls: bool(client.frames), args.timeout)
        ok = ok and bool(client.frames) and not errors(lines)
        lines, _, _ = client.command("CLOSE 2", lambda ls: bool(ls), 20)
        closed = any(l == "CLOSED 2" for l in lines)
        if ok and closed:
            report.passed("tabs", "opened, loaded and closed a second tab")
        else:
            report.failed("tabs", f"opened={ok} closed={closed} last={lines!r}")

        # --- shutdown -----------------------------------------------------
        client.send("QUIT")
        if broker is not None:
            try:
                broker.wait(timeout=10)
                report.passed("shutdown", "the broker exited on QUIT")
            except subprocess.TimeoutExpired:
                report.failed("shutdown", "the broker kept running after QUIT")
        else:
            report.skipped("shutdown", "connected to an already running broker")
    finally:
        client.close()
        if broker is not None and broker.poll() is None:
            broker.terminate()
            try:
                broker.wait(timeout=5)
            except subprocess.TimeoutExpired:
                broker.kill()
    return report


def percent_encode(text):
    out = []
    for byte in text.encode():
        if chr(byte).isalnum() or byte in b"-._~/":
            out.append(chr(byte))
        else:
            out.append(f"%{byte:02X}")
    return "".join(out)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--socket", help="connect to a broker already listening")
    parser.add_argument("--broker", help="path to litric-browserd to start")
    parser.add_argument("--chromium", help="Chromium binary for the started broker")
    parser.add_argument("--url", default="https://example.com/")
    parser.add_argument("--out", help="directory for logs and PNG snapshots")
    parser.add_argument("--timeout", type=float, default=45.0)
    parser.add_argument("--require-engine", action="store_true")
    args = parser.parse_args()
    if not args.socket and not args.broker:
        parser.error("pass --socket or --broker")
    report = run(args)
    if report.failures:
        print(f"\n{report.failures} stage(s) FAILED, {report.skips} skipped")
        return 1
    print(f"\nall stages passed ({report.skips} skipped)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
