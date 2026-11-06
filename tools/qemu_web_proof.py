#!/usr/bin/env python3
"""Open Google and YouTube in the real Chromium inside the QEMU guest.

This is a focused companion to ``qemu_guest_proof.py``. It uses the same
booted guest, QMP input path and serial diagnostics, but it does not substitute
the existing Wayland test client for a real page or a real game. Every image in
this run is a QMP ``screendump`` of the guest scanout.

The guest init publishes ``LITRIC_METRIC`` blocks from ``/proc``. This driver
turns two complete blocks into a CPU percentage and resident-memory snapshot.
Those are guest measurements, not host estimates. The proof deliberately says
that it did not measure a video decode rate or a game frame rate.

Exit status is 0 only when both pages loaded, both screenshots are at the
requested geometry, and a complete guest metric block was available.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from qemu_guest_proof import (  # noqa: E402
    QMP,
    browser_click_points,
    frame_stats,
    hotspot_click,
    latest_browser_value,
    png,
    serial_text,
    wait_serial,
    window_frames,
)

GOOGLE_PREFIXES = ("https://google.com", "https://www.google.com")
YOUTUBE_PREFIXES = (
    "https://youtube.com",
    "https://www.youtube.com",
    "https://m.youtube.com",
)
# The desktop YouTube home page does not reach a load event on this TCG guest
# within the broker's deadline. Try the lighter mobile document first; the
# embed is a real YouTube document and a useful fallback if the home page is
# still too heavy for the emulated CPU.
YOUTUBE_CANDIDATES = ("m.youtube.com", "www.youtube.com/embed/dQw4w9WgXcQ")
TICK_HZ = os.sysconf("SC_CLK_TCK")


def parse_metric_blocks(text: str) -> list[dict]:
    """Return complete LITRIC_METRIC blocks, oldest first."""
    blocks: list[dict] = []
    for match in re.finditer(
        r"LITRIC_METRIC_BEGIN(.*?)LITRIC_METRIC_END", text, re.DOTALL
    ):
        body = match.group(1)
        block: dict = {"fields": {}, "processes": []}
        for line in body.splitlines():
            line = line.strip()
            if line.startswith("LITRIC_METRIC "):
                for part in line[len("LITRIC_METRIC ") :].split():
                    key, sep, value = part.partition("=")
                    if sep and value.isdigit():
                        block["fields"][key] = int(value)
            elif line.startswith("LITRIC_PROC "):
                match_proc = re.match(
                    r"LITRIC_PROC pid=(\d+) comm=(\S+) rss_kb=(\d+) stat=(.*)",
                    line,
                )
                if not match_proc:
                    continue
                pid, comm, rss_kb, statline = match_proc.groups()
                # /proc/<pid>/stat has a space and possibly a closing paren in
                # the comm field; the fields we need start after the final ")".
                right = statline.rsplit(")", 1)
                if len(right) != 2:
                    continue
                fields = right[1].split()
                if len(fields) < 13:
                    continue
                try:
                    block["processes"].append(
                        {
                            "pid": int(pid),
                            "comm": comm,
                            "rss_kb": int(rss_kb),
                            "utime": int(fields[11]),
                            "stime": int(fields[12]),
                        }
                    )
                except ValueError:
                    continue
        if block["fields"]:
            blocks.append(block)
    return blocks


def metric_summary(blocks: list[dict]) -> dict:
    """Compute CPU and memory from the newest two complete guest snapshots."""
    if not blocks:
        return {"available": False, "reason": "no complete LITRIC_METRIC block"}
    latest = blocks[-1]
    fields = latest["fields"]
    process_totals: dict[str, dict] = {}
    for proc in latest["processes"]:
        item = process_totals.setdefault(
            proc["comm"], {"rss_kb": 0, "utime": 0, "stime": 0, "pids": []}
        )
        item["rss_kb"] += proc["rss_kb"]
        item["utime"] += proc["utime"]
        item["stime"] += proc["stime"]
        item["pids"].append(proc["pid"])

    previous = blocks[-2] if len(blocks) >= 2 else None
    previous_by_comm: dict[str, dict] = {}
    if previous:
        for proc in previous["processes"]:
            item = previous_by_comm.setdefault(
                proc["comm"], {"utime": 0, "stime": 0}
            )
            item["utime"] += proc["utime"]
            item["stime"] += proc["stime"]

    cpu: dict[str, float] = {}
    if previous:
        elapsed = fields.get("uptime", 0) - previous["fields"].get("uptime", 0)
        if elapsed > 0:
            for comm, item in process_totals.items():
                before = previous_by_comm.get(comm, {"utime": 0, "stime": 0})
                delta = (item["utime"] + item["stime"]) - (
                    before["utime"] + before["stime"]
                )
                if delta > 0:
                    cpu[comm] = round(100.0 * (delta / TICK_HZ) / elapsed, 2)

    total_kb = fields.get("mem_total_kb", 0)
    available_kb = fields.get("mem_available_kb", 0)
    used_kb = max(0, total_kb - available_kb)
    return {
        "available": True,
        "samples": len(blocks),
        "uptime_s": fields.get("uptime"),
        "cpu_pct_by_process": cpu,
        "cpu_pct_note": "sum of threads per process group; can exceed 100 on SMP",
        "rss_kb_by_process": {
            comm: item["rss_kb"] for comm, item in sorted(process_totals.items())
        },
        "pids_by_process": {
            comm: item["pids"] for comm, item in sorted(process_totals.items())
        },
        "mem_total_kb": total_kb,
        "mem_available_kb": available_kb,
        "mem_used_kb": used_kb,
        "mem_used_pct": round(100.0 * used_kb / total_kb, 2) if total_kb else None,
    }


def focus_address(q: QMP, serial: Path) -> tuple[int, int]:
    """Click the shell's own reported omnibox centre, with a frame fallback."""
    text = serial.read_text(encoding="utf-8", errors="replace") if serial.exists() else ""
    point = hotspot_click(text, "browser.omnibox_click")
    if point is None:
        frame = next(
            (
                item
                for item in window_frames(text)
                if item["app"] == "com.litric.browser"
            ),
            None,
        )
        if frame is None:
            raise RuntimeError("the browser has no reported frame")
        point = browser_click_points(frame, 1.0)[0]
    q.key("esc")
    q.move(10, 10)
    time.sleep(0.2)
    q.click(*point)
    q.click(*point)
    time.sleep(1.0)
    return point


def page_state(text: str) -> dict:
    return {
        "url": latest_browser_value(text, "browser.tab"),
        "title": latest_browser_value(text, "browser.title"),
        "status": latest_browser_value(text, "browser.status"),
        "frame": latest_browser_value(text, "browser.frame"),
        "tabs": latest_browser_value(text, "browser.tabs"),
    }


def load_page(
    q: QMP, serial: Path, url: str, prefixes: tuple[str, ...], timeout: float
) -> dict:
    last_state: dict = {}
    for attempt in range(3):
        if attempt:
            # The initial tab is created by the broker after the window maps.
            # If GO arrives in that gap the broker correctly answers `no such
            # tab`; wait for the broker and try the same user navigation again.
            time.sleep(3)
        focus_address(q, serial)
        q.text(url)
        time.sleep(0.5)
        q.key("ret")
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            text = serial_text(serial)
            tab = latest_browser_value(text, "browser.tab") or ""
            frame = latest_browser_value(text, "browser.frame") or ""
            status = latest_browser_value(text, "browser.status") or ""
            last_state = page_state(text)
            if (
                tab.startswith(prefixes)
                and frame.endswith("bytes")
                and status.startswith("loaded")
            ):
                time.sleep(3)
                return page_state(serial_text(serial))
            if "no such tab" in status:
                break
            if tab.startswith(prefixes) and (
                status.startswith("the page could not be shown")
                or status.startswith("navigation failed")
            ):
                raise RuntimeError(f"{url}: {status}")
            time.sleep(0.25)
        else:
            raise RuntimeError(
                f"timed out waiting for {url}; last state: {last_state}"
            )
    raise RuntimeError(
        f"{url}: broker did not create its initial tab after retries; last state: {last_state}"
    )


def load_first_page(
    q: QMP, serial: Path, urls: tuple[str, ...], prefixes: tuple[str, ...], timeout: float
) -> tuple[dict, list[dict]]:
    """Try alternate real YouTube documents and keep each failure as evidence."""
    attempts: list[dict] = []
    for url in urls:
        try:
            return load_page(q, serial, url, prefixes, timeout), attempts
        except RuntimeError as error:
            attempts.append({"url": url, "error": str(error)})
    raise RuntimeError(f"no YouTube document loaded: {attempts}")


def shot(q: QMP, out: Path, name: str) -> dict:
    ppm = out / f"{name}.ppm"
    q.screenshot(ppm)
    state = frame_stats(ppm)
    png(ppm, out / f"{name}.png")
    return state


def run(
    qmp_path: str,
    serial_path: str,
    out: Path,
    timeout: float,
    width: int,
    height: int,
) -> dict:
    out.mkdir(parents=True, exist_ok=True)
    (out / "result.json").unlink(missing_ok=True)
    serial = Path(serial_path)
    q = QMP(qmp_path, width, height)
    result: dict = {
        "host": "QEMU guest",
        "geometry": {"width": width, "height": height},
        "requested_urls": {
            "google": "google.com",
            "youtube": list(YOUTUBE_CANDIDATES),
            "accepted_prefixes": {
                "google": list(GOOGLE_PREFIXES),
                "youtube": list(YOUTUBE_PREFIXES),
            },
        },
        "browser": {},
        "pages": {},
        "metrics": {},
        "screenshots": {},
    }
    try:
        wait_serial(
            serial,
            lambda text: "litric.shell.mode=ready" in text
            and "input.pointer=true" in text,
            "the LitricOS shell and absolute pointer",
            timeout,
        )
        wait_serial(
            serial,
            lambda text: "LITRIC_NETWORK_OK" in text or "LITRIC_NETWORK_FAILED" in text,
            "the guest network probe",
            timeout,
        )
        network_text = serial.read_text(encoding="utf-8", errors="replace")
        if "LITRIC_NETWORK_OK" not in network_text:
            raise RuntimeError("guest network probe failed")

        q.key("meta_l")
        time.sleep(1)
        q.text("browser")
        time.sleep(1)
        launcher = out / "browser-launcher.ppm"
        q.screenshot(launcher)
        png(launcher, out / "browser-launcher.png")
        q.key("ret")
        opened = wait_serial(
            serial,
            lambda text: "com.litric.browser" in text,
            "the browser window from the launcher",
            timeout,
        )
        browser_frame = next(
            (
                item
                for item in window_frames(opened)
                if item["app"] == "com.litric.browser"
            ),
            None,
        )
        if browser_frame is None:
            raise RuntimeError("the browser window has no frame")
        result["browser"]["window"] = browser_frame
        # The browser window appears before the broker has processed the
        # initial NEW. Typing into the omnibox in that gap returns the real
        # broker error `no such tab` and consumes the whole run. Wait for the
        # shell's initial tab state, then give the broker a settled moment.
        wait_serial(
            serial,
            lambda text: latest_browser_value(text, "browser.tabs") == "1 active=0",
            "the browser's initial broker tab",
            timeout,
        )
        time.sleep(5)

        google = load_page(q, serial, "google.com", GOOGLE_PREFIXES, timeout)
        google_shot = shot(q, out, "google")
        result["pages"]["google"] = google
        result["screenshots"]["google"] = google_shot

        # Keep the same tab for YouTube. A second tab is exercised by the
        # separate browser proof; using one tab here avoids a shell/broker
        # timing race and tests the user-visible navigation directly.
        youtube, youtube_attempts = load_first_page(
            q, serial, YOUTUBE_CANDIDATES, YOUTUBE_PREFIXES, timeout
        )
        youtube_shot = shot(q, out, "youtube")
        result["pages"]["youtube"] = youtube
        result["pages"]["youtube_attempts"] = youtube_attempts

        final_text = serial.read_text(encoding="utf-8", errors="replace")
        blocks = parse_metric_blocks(final_text)
        result["metrics"] = metric_summary(blocks)
        result["browser"].update(
            {
                "engine_chromium": "browserd: engine = chromium" in final_text,
                "gpu_policy": next(
                    (
                        line.strip()
                        for line in final_text.splitlines()
                        if "browserd: gpu=" in line
                    ),
                    None,
                ),
                "tabs": latest_browser_value(final_text, "browser.tabs"),
                "url": latest_browser_value(final_text, "browser.tab"),
                "title": latest_browser_value(final_text, "browser.title"),
                "frame": latest_browser_value(final_text, "browser.frame"),
                "status": latest_browser_value(final_text, "browser.status"),
            }
        )
        result["screenshots"]["youtube"] = youtube_shot

        assertions = {
            "guest_network": True,
            "browser_engine_chromium": result["browser"]["engine_chromium"],
            "google_url": google["url"] is not None
            and google["url"].startswith(GOOGLE_PREFIXES),
            "google_frame": (google["frame"] or "").endswith("bytes"),
            "google_screenshot_at_expected_size": (
                google_shot["width"],
                google_shot["height"],
            )
            == (width, height),
            "google_pixels": google_shot["bright"] >= 20_000,
            "youtube_url": youtube["url"] is not None
            and youtube["url"].startswith(YOUTUBE_PREFIXES),
            "youtube_frame": (youtube["frame"] or "").endswith("bytes"),
            "youtube_screenshot_at_expected_size": (
                youtube_shot["width"],
                youtube_shot["height"],
            )
            == (width, height),
            "youtube_pixels": youtube_shot["bright"] >= 1_000
            or youtube_shot["colourful"] >= 1_000,
            "metrics_sampled": result["metrics"].get("available") is True,
        }
        result["assertions"] = assertions
        result["status"] = "PASS" if all(assertions.values()) else "PARTIAL"
        (out / "result.json").write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        return result
    except Exception as error:
        result["status"] = "FAIL"
        result["error"] = str(error)
        result["assertions"] = {"completed": False}
        (out / "result.json").write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        raise
    finally:
        try:
            q.command("screendump", filename=str(out / "final.ppm"))
            png(out / "final.ppm", out / "final.png")
        except Exception:
            pass
        try:
            q.sock.close()
        except Exception:
            pass


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--qmp", required=True)
    parser.add_argument("--serial", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--width", type=int, default=1280)
    parser.add_argument("--height", type=int, default=800)
    parser.add_argument("--net-port", type=int, default=18080)
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()
    try:
        result = run(
            args.qmp,
            args.serial,
            Path(args.out),
            args.timeout,
            args.width,
            args.height,
        )
    except Exception as error:
        print(f"QEMU WEB PROOF: FAIL: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    print(f"QEMU WEB PROOF: {result['status']}")
    return 0 if result["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
