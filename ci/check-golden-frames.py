#!/usr/bin/env python3
"""Compare dumped desktop frames with the recorded golden set.

WHAT THIS GATE IS
-----------------
`litric-shell`'s `frame_dump` example composes the *shipping* desktop -- the same
`views::draw` and `desktop::wallpaper` that `main.rs` calls on a booted machine
-- into an in-memory canvas and writes each scene as a P6 PPM. This script turns
those frames into a gate: the ten screens the governance document requires as
golden (04-QA-LINT-GOVERNANCE §2) must be present, and every screen in the
golden set must still match the recorded one within a threshold.

WHAT IT DOES NOT PROVE
----------------------
It proves what the *composition* does with fixed inputs. It cannot prove that
the machine boots, that the framebuffer is the right depth, or that the typeface
rasterises on the target; those need the image, and `tools/photo-tour*.py` is
where they are checked. A frame here differs from a frame on the machine only by
the facts that were fed in -- which is exactly what makes it a usable golden.

HOW A FRAME IS COMPARED
-----------------------
A raw hash answers "did anything change", which is not the question. The
question is "did the picture change in a way a person would see", and a
single antialiased pixel is not that. So each frame is reduced to a coarse
luminance grid (64x40 cells, one byte each) and compared cell by cell:

  * `changed` -- the fraction of cells whose luminance differs by more than
    `cell_tolerance` levels. Over `max_changed_fraction`, the gate FAILS and
    prints where the change is.
  * `mean_delta` -- the average absolute difference across all cells, in
    levels. Over `max_mean_delta`, the gate FAILS too: a wide, small shift
    (a font weight, a token colour) moves this number without moving the
    changed-cell count much.

The exact-hash line is printed for the record. A hash that differs while both
thresholds pass is *not* a failure: it is the definition of a sub-threshold
change, and it is printed so a reviewer can see that the frame did move.

Run:
    ci/check-golden-frames.py --frames build/frames
    ci/check-golden-frames.py --frames build/frames --update   # after review

Exit code: 0 within threshold and complete, 1 otherwise.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

# The ten required golden screens, in the words of the governance document, and
# the frame each one is dumped as. Scene names are the frame file's stem: the
# dump writes `01-desktop-idle.ppm` and the gate reads the same name, so a
# renamed scene fails here rather than silently dropping out of the set.
REQUIRED: list[tuple[str, str]] = [
    ("Desktop idle", "01-desktop-idle"),
    ("Desktop + 3 windows overlap", "03-windows-overlap"),
    ("Files (grid, with files)", "04-files"),
    ("Files (empty + edge state)", "16-files-empty"),
    ("Settings / Appearance", "05-settings"),
    ("Control Center open", "17-control-centre"),
    ("Seek open with results", "15-search"),
    ("Notification Center", "18-notifications"),
    ("Dialog (danger delete file)", "19-dialog-danger"),
    ("Lock screen", "11-lock"),
]

# Screens the dump writes that are not in the required ten. They are recorded and
# compared too -- a change to the launcher is a change to the launcher whether or
# not governance listed it -- but the gate does not fail when one is missing.
OPTIONAL: list[str] = [
    "02-desktop-start",
    "06-browser",
    "07-terminal",
    "08-trash",
    "09-defender",
    "10-activity",
    "12-lock-password",
    "13-lock-customize",
    "14-launcher",
    "40-install-dialog",
    "41-uninstall-dialog",
    "42-pinned-dock",
    "20-environments",
    "25-settings-updates",
    "26-settings-display",
    "27-settings-input",
    "28-settings-network",
    "29-settings-bluetooth",
    "30-settings-power",
    "31-settings-region",
    "32-settings-environments",
    "33-settings-accessibility",
    "34-settings-notifications",
    "35-settings-sound",
    "36-settings-privacy",
    "37-settings-accounts",
    "38-settings-diagnostics",
    "43-settings-services",
    "44-settings-app-store",
]

GRID_W = 64
GRID_H = 40
CELLS = GRID_W * GRID_H

# Thresholds. 2% of cells is the number the governance document names ("diff
# เกิน 2% ต้องอัปเดต golden"); the mean is half a level, which is the point
# where a change is a decision about the design rather than about rounding.
MAX_CHANGED_FRACTION = 0.02
MAX_MEAN_DELTA = 0.5
CELL_TOLERANCE = 8

DEFAULT_FRAMES = Path("build/frames")
DEFAULT_GOLDEN = Path("docs/evidence/golden/fingerprints.json")


class FrameError(Exception):
    """A frame that could not be read or understood."""


def read_ppm(path: Path) -> tuple[int, int, bytes]:
    """Read a binary P6 PPM, returning `(width, height, rgb bytes)`."""
    data = path.read_bytes()
    if not data.startswith(b"P6"):
        raise FrameError(f"{path}: not a binary PPM (magic {data[:2]!r})")
    pos = 2
    fields: list[int] = []
    # The three numbers after the magic: width, height, maximum value. Comments
    # are legal anywhere a token could start, and the frame dumper writes none,
    # so they are skipped rather than relied on being absent.
    while len(fields) < 3:
        while pos < len(data) and data[pos : pos + 1].isspace():
            pos += 1
        if data[pos : pos + 1] == b"#":
            while pos < len(data) and data[pos : pos + 1] != b"\n":
                pos += 1
            continue
        start = pos
        while pos < len(data) and not data[pos : pos + 1].isspace():
            pos += 1
        if pos == start:
            raise FrameError(f"{path}: truncated PPM header")
        fields.append(int(data[start:pos]))
    width, height, maxval = fields
    if maxval != 255:
        raise FrameError(f"{path}: {maxval}-level PPM is not a frame this gate reads")
    pos += 1  # exactly one whitespace byte separates the header from the raster
    pixels = data[pos:]
    if len(pixels) < width * height * 3:
        raise FrameError(
            f"{path}: {len(pixels)} bytes for {width}x{height} pixels, which is short"
        )
    return width, height, pixels


def grid_of(width: int, height: int, pixels: bytes) -> list[int]:
    """The frame as a GRID_W x GRID_H grid of average luminance, 0-255.

    Averaged rather than sampled: a sample lands on whichever pixel the geometry
    happens to pick, so moving a panel by one pixel can change a sample by a
    hundred levels while the picture a person sees is identical.
    """
    out = [0] * CELLS
    for cell_y in range(GRID_H):
        y0 = cell_y * height // GRID_H
        y1 = max(y0 + 1, (cell_y + 1) * height // GRID_H)
        for cell_x in range(GRID_W):
            x0 = cell_x * width // GRID_W
            x1 = max(x0 + 1, (cell_x + 1) * width // GRID_W)
            total = 0
            count = 0
            for y in range(y0, y1):
                row = y * width
                for x in range(x0, x1):
                    off = (row + x) * 3
                    r, g, b = pixels[off], pixels[off + 1], pixels[off + 2]
                    total += (r * 299 + g * 587 + b * 114) // 1000
                    count += 1
            out[cell_y * GRID_W + cell_x] = total // max(count, 1)
    return out


def fingerprint_of(width: int, height: int, pixels: bytes) -> dict[str, object]:
    """Everything the gate records about one frame."""
    grid = grid_of(width, height, pixels)
    return {
        "size": f"{width}x{height}",
        # FNV-1a over the pixels: the same function `frame_dump` prints, so the
        # number in a log and the number here are the same number.
        "hash": fnv1a(pixels),
        "sha256": hashlib.sha256(pixels).hexdigest()[:16],
        # The grid is stored as hex, two characters a cell, so the file is
        # readable in a diff without being a picture nobody can parse.
        "grid": bytes(grid).hex(),
        "grid_size": f"{GRID_W}x{GRID_H}",
    }


def fnv1a(data: bytes) -> str:
    h = 0xCBF29CE484222325
    for byte in data:
        h ^= byte
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return f"0x{h:016x}"


def compare(golden: dict[str, object], fresh: dict[str, object]) -> tuple[float, float]:
    """`(changed fraction, mean delta)` between two fingerprints' grids."""
    before = bytes.fromhex(str(golden["grid"]))
    after = bytes.fromhex(str(fresh["grid"]))
    if len(before) != len(after):
        raise FrameError(
            f"golden grid is {len(before)} cells and this frame's is {len(after)}"
        )
    changed = 0
    total = 0
    for a, b in zip(before, after):
        delta = abs(a - b)
        total += delta
        if delta > CELL_TOLERANCE:
            changed += 1
    n = max(len(before), 1)
    return changed / n, total / n


def load_frames(directory: Path) -> dict[str, dict[str, object]]:
    frames: dict[str, dict[str, object]] = {}
    for path in sorted(directory.glob("*.ppm")):
        width, height, pixels = read_ppm(path)
        frames[path.stem] = fingerprint_of(width, height, pixels)
    return frames


def describe_change(golden: dict[str, object], fresh: dict[str, object], limit: int = 3) -> str:
    """Where the two grids differ most, in grid coordinates."""
    before = bytes.fromhex(str(golden["grid"]))
    after = bytes.fromhex(str(fresh["grid"]))
    worst = sorted(
        (
            (abs(a - b), i)
            for i, (a, b) in enumerate(zip(before, after))
        ),
        reverse=True,
    )[:limit]
    parts = []
    for delta, index in worst:
        cell_x, cell_y = index % GRID_W, index // GRID_W
        parts.append(f"cell {cell_x},{cell_y} (-{delta} levels)")
    return ", ".join(parts)


def write_ppm(path: Path, width: int, height: int, pixels: bytes) -> None:
    """A P6 PPM, which is what `frame_dump` writes and this gate reads."""
    path.write_bytes(f"P6\n{width} {height}\n255\n".encode("ascii") + pixels)


def self_test() -> int:
    """Prove the gate fails when it should, with frames it makes itself.

    The gate is a threshold, and a threshold that never fires and one that fires
    on anything are both useless. This plants the three cases a reviewer needs
    it to get right -- an unchanged frame, a frame changed by a block a person
    would see, and a frame changed by one pixel -- and reports what it found
    rather than asserting it through the real golden set, which a self-test has
    no business rewriting.
    """
    import tempfile

    width, height = 96, 60
    flat = bytes([120, 120, 120]) * (width * height)
    block = bytearray(flat)
    for y in range(15, 45):
        for x in range(20, 76):
            off = (y * width + x) * 3
            block[off] = block[off + 1] = block[off + 2] = 0
    pixel = bytearray(flat)
    pixel[(30 * width + 48) * 3] = 0

    failures: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        write_ppm(root / "same.ppm", width, height, flat)
        write_ppm(root / "block.ppm", width, height, bytes(block))
        write_ppm(root / "pixel.ppm", width, height, bytes(pixel))
        dumped = load_frames(root)
        if len(dumped) != 3:
            return self_test_report([f"read {len(dumped)} of 3 planted frames"])
        golden = {"same": dumped["same"], "block": dumped["same"], "pixel": dumped["same"]}

        # An unchanged frame is not reported.
        changed, mean = compare(golden["same"], dumped["same"])
        if changed != 0.0 or mean != 0.0:
            failures.append(f"an identical frame moved by {changed:.4f}/{mean:.4f}")
        # A block a person would see is over both thresholds.
        changed, mean = compare(golden["block"], dumped["block"])
        if not (changed > MAX_CHANGED_FRACTION or mean > MAX_MEAN_DELTA):
            failures.append(
                f"a planted 56x30 block passed the gate ({changed:.4f}/{mean:.4f})"
            )
        # One changed pixel is not: that is the whole reason the gate compares
        # blocks of the picture rather than a hash of the bytes.
        changed, mean = compare(golden["pixel"], dumped["pixel"])
        if changed > MAX_CHANGED_FRACTION or mean > MAX_MEAN_DELTA:
            failures.append(f"a one-pixel change failed the gate ({changed:.4f}/{mean:.4f})")
        # And the failure names where the change is, which is what makes a red
        # build actionable.
        where = describe_change(golden["block"], dumped["block"])
        if "cell" not in where:
            failures.append(f"a failure did not name a cell: {where!r}")

    return self_test_report(failures)


def self_test_report(failures: list[str]) -> int:
    if failures:
        print("golden-frames --self-test: FAIL")
        for line in failures:
            print(f"  {line}")
        return 1
    print(
        "golden-frames --self-test: PASS -- an unchanged frame passes, a planted "
        "block fails, a one-pixel change passes, and the failure names a cell"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frames", type=Path, default=DEFAULT_FRAMES)
    parser.add_argument("--golden", type=Path, default=DEFAULT_GOLDEN)
    parser.add_argument(
        "--update",
        action="store_true",
        help="record the frames just dumped as the new golden set (after review)",
    )
    parser.add_argument(
        "--allow-missing",
        action="store_true",
        help="treat a missing required frame as a skip instead of a failure",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="prove the comparison fails on a planted change and passes on noise",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if not args.frames.is_dir():
        print(f"golden-frames: no frame directory at {args.frames}", file=sys.stderr)
        print(
            "  run: cargo run -p litric-shell --example frame_dump -- "
            f"{args.frames}",
            file=sys.stderr,
        )
        return 1

    fresh = load_frames(args.frames)
    if not fresh:
        print(f"golden-frames: {args.frames} holds no .ppm frames", file=sys.stderr)
        return 1

    missing = [scene for _, scene in REQUIRED if scene not in fresh]
    if missing and not args.allow_missing:
        print("golden-frames: FAIL -- the required golden screens were not dumped:")
        for scene in missing:
            print(f"  missing: {scene}")
        return 1

    if args.update:
        args.golden.parent.mkdir(parents=True, exist_ok=True)
        record = {
            "tool": "ci/check-golden-frames.py",
            "frames_command": (
                "cargo run -p litric-shell --example frame_dump -- build/frames"
            ),
            "grid": f"{GRID_W}x{GRID_H}",
            "thresholds": {
                "max_changed_fraction": MAX_CHANGED_FRACTION,
                "max_mean_delta": MAX_MEAN_DELTA,
                "cell_tolerance": CELL_TOLERANCE,
            },
            "required": [{"screen": screen, "scene": scene} for screen, scene in REQUIRED],
            "frames": {scene: fresh[scene] for scene in sorted(fresh)},
        }
        args.golden.write_text(
            json.dumps(record, indent=1, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(f"golden-frames: recorded {len(fresh)} frame(s) in {args.golden}")
        for screen, scene in REQUIRED:
            print(f"  {scene:<20} {screen}")
        return 0

    if not args.golden.is_file():
        print(f"golden-frames: no golden set at {args.golden}", file=sys.stderr)
        print("  record one with --update, review the fingerprint diff, and commit it", file=sys.stderr)
        return 1

    record = json.loads(args.golden.read_text(encoding="utf-8"))
    golden = record.get("frames", {})
    recorded_grid = record.get("grid", f"{GRID_W}x{GRID_H}")
    if recorded_grid != f"{GRID_W}x{GRID_H}":
        print(
            f"golden-frames: the golden set was recorded on a {recorded_grid} grid, "
            f"this gate computes {GRID_W}x{GRID_H}",
            file=sys.stderr,
        )
        return 1

    failures = 0
    print(f"golden-frames: {len(REQUIRED)} required screen(s), {len(fresh)} frame(s) dumped")
    for screen, scene in REQUIRED:
        if scene not in fresh:
            print(f"  SKIP  {scene:<20} {screen} (not dumped)")
            continue
        failures += report(screen, scene, golden, fresh)
    for scene in OPTIONAL:
        if scene not in fresh:
            print(f"  SKIP  {scene:<20} (not dumped)")
            continue
        failures += report(scene, scene, golden, fresh)

    if failures:
        print(
            f"golden-frames: FAIL -- {failures} frame(s) changed by more than the "
            "threshold. If the change is the point, review the frames, then record "
            "them with --update and say why in the commit."
        )
        return 1
    print("golden-frames: PASS -- every frame is within the threshold of its golden")
    return 0


def report(
    screen: str, scene: str, golden: dict[str, dict[str, object]], fresh: dict[str, dict[str, object]]
) -> int:
    """Print one frame's comparison. Returns 1 when it fails the gate."""
    before = golden.get(scene)
    after = fresh[scene]
    if before is None:
        # A frame that has never been recorded cannot be compared. It is not a
        # failure of the frame, it is a gap in the golden set, and it is printed
        # as one rather than passed silently.
        print(f"  NEW   {scene:<20} {screen}: not in the golden set (record with --update)")
        return 0
    if str(before.get("size")) != str(after.get("size")):
        print(
            f"  FAIL  {scene:<20} {screen}: {before.get('size')} -> {after.get('size')}, "
            "the frame is a different size"
        )
        return 1
    changed, mean = compare(before, after)
    same_hash = before.get("hash") == after.get("hash")
    if changed > MAX_CHANGED_FRACTION or mean > MAX_MEAN_DELTA:
        print(
            f"  FAIL  {scene:<20} {screen}: {changed * 100:.2f}% of cells changed "
            f"(limit {MAX_CHANGED_FRACTION * 100:.2f}%), mean delta {mean:.2f} levels "
            f"(limit {MAX_MEAN_DELTA:.2f})"
        )
        print(f"        worst: {describe_change(before, after)}")
        return 1
    if same_hash:
        print(f"  PASS  {scene:<20} {screen}: identical")
    else:
        print(
            f"  PASS  {scene:<20} {screen}: {changed * 100:.2f}% of cells changed, "
            f"mean delta {mean:.2f} levels -- within threshold"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
