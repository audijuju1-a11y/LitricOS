#!/usr/bin/env python3
"""Install and play a real Freedoom/Chocolate Doom game in the QEMU guest.

The driver uses the same QMP input and guest serial status as the other
proofs. It opens the Games page, clicks Play, captures the guest scanout while
the demo is running, reads the game's own timedemo FPS line, and reports guest
`/proc` CPU/RAM measurements.

Exit status is 0 only when the game was installed by `APP INSTALL`, a non-
Settings game window appeared, the framebuffer contains game pixels, and the
game emitted a timedemo FPS result. Nothing here substitutes a host render or
a fabricated performance number.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from qemu_guest_proof import (  # noqa: E402
    QMP,
    frame_stats,
    png,
    play_point,
    serial_text,
    wait_serial,
    window_frames,
)
from qemu_web_proof import metric_summary, parse_metric_blocks  # noqa: E402

BASE_W, BASE_H = 1280, 800
GAME_ID = "com.litric.doom"
FPS_RE = re.compile(
    r"Timed\s+(\d+)\s+gametics\s+in\s+(\d+)\s+realtics\s*=\s*([0-9.]+)\s*fps",
    re.IGNORECASE,
)


def latest_game_window(text: str) -> dict | None:
    """The newest shell window that is not the Settings app."""
    for frame in reversed(window_frames(text)):
        if frame["app"] != "com.litric.settings":
            return frame
    return None


def latest_doom_log(text: str) -> str:
    start = text.rfind("LITRIC_DOOM_LOG_BEGIN")
    end = text.rfind("LITRIC_DOOM_LOG_END")
    if start < 0 or end < start:
        return ""
    return text[start:end]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--qmp", required=True)
    parser.add_argument("--serial", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--width", type=int, default=BASE_W)
    parser.add_argument("--height", type=int, default=BASE_H)
    parser.add_argument("--net-port", type=int, default=18080)
    parser.add_argument("--timeout", type=float, default=300)
    args = parser.parse_args()

    width, height = args.width, args.height
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "result.json").unlink(missing_ok=True)
    serial = Path(args.serial)
    q = QMP(args.qmp, width, height)

    assertions: dict[str, bool] = {}
    result: dict = {
        "status": "FAIL",
        "game": {"id": GAME_ID},
        "assertions": assertions,
        "screenshots": {},
    }

    try:
        wait_serial(
            serial,
            lambda text: "litric.shell.mode=ready" in text
            and "input.pointer=true" in text
            and (
                f"litric.shell.fb={width}x{height} " in text
                or f"litric.shell.drm.mode={width}x{height}" in text
            ),
            "desktop ready with an absolute pointer",
            args.timeout,
        )
        network = wait_serial(
            serial,
            lambda text: "LITRIC_NETWORK_OK" in text
            or "LITRIC_NETWORK_FAILED" in text,
            "guest network probe",
            args.timeout,
        )
        if "LITRIC_NETWORK_OK" not in network:
            raise RuntimeError("guest network probe failed")
        if "LITRIC_INSTALL_OK" not in serial_text(serial):
            raise RuntimeError("APP INSTALL did not report state=installed")
        assertions["app-installed"] = True

        q.move(width // 2, height // 2)
        time.sleep(0.5)
        q.key("meta_l")
        time.sleep(0.8)
        q.text("games")
        time.sleep(0.8)
        launch_shot = out / "launcher.ppm"
        q.screenshot(launch_shot)
        png(launch_shot, out / "launcher.png")
        q.key("ret")
        wait_serial(
            serial,
            lambda text: "com.litric.settings" in text,
            "Settings window from the Games search",
            args.timeout,
        )
        games_shot = out / "games.ppm"
        q.screenshot(games_shot)
        png(games_shot, out / "games.png")
        frames = window_frames(serial_text(serial))
        settings = next(
            (frame for frame in frames if frame["app"] == "com.litric.settings"),
            None,
        )
        if settings is None:
            raise RuntimeError(f"no Settings window geometry: {frames}")
        play = play_point(settings, width)
        print(f"clicking Play at {play}", flush=True)
        q.click(*play)

        wait_serial(
            serial,
            lambda text: latest_game_window(text) is not None,
            "native game window",
            args.timeout,
        )
        game = latest_game_window(serial_text(serial))
        if game is None:
            raise RuntimeError("game window disappeared before it could be read")
        result["game"].update(game)
        assertions["game-window"] = True

        # Capture the guest's real scanout until the game has drawn its
        # colourful status bar/world; do not accept a blank or Settings frame.
        game_shot = out / "game.ppm"
        stats: dict = {}
        deadline = time.monotonic() + min(45.0, args.timeout)
        while time.monotonic() < deadline:
            q.screenshot(game_shot)
            stats = frame_stats(game_shot)
            if stats.get("colourful", 0) >= 10_000:
                break
            time.sleep(0.5)
        png(game_shot, out / "game.png")
        result["screenshots"]["game"] = "game.png"
        result["frame"] = stats
        if stats.get("colourful", 0) < 10_000:
            raise RuntimeError(f"the game frame had too few game pixels: {stats}")
        assertions["game-pixels"] = True

        fps_text = wait_serial(
            serial,
            lambda text: FPS_RE.search(latest_doom_log(text)) is not None,
            "Chocolate Doom timedemo FPS result",
            args.timeout,
        )
        log = latest_doom_log(fps_text)
        match = FPS_RE.search(log)
        if match is None:
            raise RuntimeError("no timedemo result in the game log")
        gametics, realtics, fps = match.groups()
        result["fps"] = {
            "gametics": int(gametics),
            "realtics": int(realtics),
            "fps": float(fps),
            "raw": match.group(0),
        }
        assertions["fps-measured"] = True

        # The graphics frame above is captured before the demo exits. Use a
        # second capture for the final screenshot if the process is still up.
        final_shot = out / "game-final.ppm"
        q.screenshot(final_shot)
        png(final_shot, out / "game-final.png")
        result["screenshots"]["game_final"] = "game-final.png"

        metrics = parse_metric_blocks(serial_text(serial))
        summary = metric_summary(metrics)
        result["metrics"] = summary
        assertions["guest-metrics"] = bool(summary.get("available"))
        result["status"] = "PASS" if all(assertions.values()) else "FAIL"
        return 0 if result["status"] == "PASS" else 1
    except Exception as error:  # noqa: BLE001 - write the failure evidence we have
        result["error"] = str(error)
        return 1
    finally:
        (out / "result.json").write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(json.dumps(result, indent=2, sort_keys=True), flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
