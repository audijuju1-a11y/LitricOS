#!/usr/bin/env python3
"""Fail when a render-benchmark run regresses against a known-good run.

WHAT THIS GATE IS
-----------------
`litric-shell`'s `render_bench` example already prints a median and a sample
count for every scene. That is a measurement, not yet a gate: nothing compared
two runs, so the W9 acceptance sentence ("the harness fails on a planted 15 %
regression") could not be executed.

This script is the comparator. It reads the raw `key=value` output of two
`render_bench` runs, takes every `*.median_ms` line, and fails when the
candidate is at or above the threshold relative to the baseline. It also fails
when a baseline scene is absent, a metric is duplicated or non-finite, or a
scene does not carry the required number of runs. A partial run is not a result,
and a missing metric must not silently disappear from the gate.

It is deliberately not a benchmark of its own: the real measurement stays in
`src/litric-shell/examples/render_bench.rs`, so the harness cannot drift into
timing a different scene. The two runs must use the same host, release build,
canvas and workload; this script cannot check those facts, and says so rather
than implying them.

USAGE
-----
    cargo run --release -p litric-shell --example render_bench -- /tmp/perf/base > baseline.txt
    cargo run --release -p litric-shell --example render_bench -- /tmp/perf/candidate > candidate.txt
    python3 ci/check-perf-regression.py --baseline baseline.txt --actual candidate.txt

`--threshold` defaults to 10 % because `docs/PERF-budget.md` rule 4 sets a 10 %
CI regression limit. W9's acceptance plants a 15 % regression; the self-test
uses a 15 % threshold so that the exact acceptance boundary is proved to fail.

A self-test PASS proves the comparator works. It does not prove that any
machine, image or candidate is fast enough; that requires two real runs on one
controlled environment, with the environment and sample count recorded.

Exit code: 0 when every baseline scene is within the limit, 1 otherwise.
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

DEFAULT_THRESHOLD = 10.0
MIN_RUNS = 15
MEDIAN_SUFFIX = ".median_ms"
RUNS_SUFFIX = ".runs"


class InputError(ValueError):
    """The run text is not a usable render-benchmark record."""


def parse_metrics(text: str) -> dict[str, float]:
    """Parse numeric `key=value` metrics, ignoring labels and prose.

    `render_bench` also prints `canvas=1280x800`, scene names and per-site
    labels. Those are records, not numeric measurements, and are skipped. A
    numeric key that appears twice is an error: taking the last one would make
    two different runs look like one.
    """
    values: dict[str, float] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, raw_value = line.partition("=")
        key = key.strip()
        raw_value = raw_value.strip()
        if not key:
            raise InputError("a metric has an empty name")
        try:
            value = float(raw_value)
        except ValueError:
            # Labels such as `canvas=1280x800` and `full_frame.sites_per_frame=...`
            # are not measurements this comparator consumes.
            continue
        if not math.isfinite(value) or value < 0.0:
            raise InputError(f"{key} is not a finite non-negative number: {raw_value}")
        if key in values:
            raise InputError(f"{key} appears more than once")
        values[key] = value
    if not values:
        raise InputError("no numeric metrics were found")
    return values


def scene_medians(values: dict[str, float], label: str) -> dict[str, float]:
    """Return each scene's median and enforce the sample-count contract."""
    medians = {
        key[: -len(MEDIAN_SUFFIX)]: value
        for key, value in values.items()
        if key.endswith(MEDIAN_SUFFIX) and key[: -len(MEDIAN_SUFFIX)]
    }
    if not medians:
        raise InputError(f"{label} contains no median_ms metrics")
    for scene, median in medians.items():
        if median <= 0.0:
            raise InputError(f"{label} {scene} has a non-positive median")
        runs_key = f"{scene}{RUNS_SUFFIX}"
        runs = values.get(runs_key)
        if runs is None:
            raise InputError(f"{label} {scene} has a median but no {runs_key}")
        if not runs.is_integer() or int(runs) < MIN_RUNS:
            raise InputError(
                f"{label} {scene} has {runs:g} runs; at least {MIN_RUNS} are required"
            )
    return medians


def compare(
    baseline_text: str, actual_text: str, threshold: float
) -> tuple[list[str], list[str], list[str]]:
    """Compare two runs and return (passes, failures, new scenes)."""
    if not math.isfinite(threshold) or threshold <= 0.0:
        raise InputError("the threshold must be a finite positive percentage")
    baseline = scene_medians(parse_metrics(baseline_text), "baseline")
    actual = scene_medians(parse_metrics(actual_text), "candidate")

    passes: list[str] = []
    failures: list[str] = []
    for scene in sorted(baseline):
        if scene not in actual:
            failures.append(f"{scene}: present in the baseline but absent from the candidate")
            continue
        before = baseline[scene]
        after = actual[scene]
        change = (after - before) * 100.0 / before
        if change >= threshold:
            failures.append(
                f"{scene}: {before:.3f} ms -> {after:.3f} ms ({change:+.2f}% >= {threshold:.2f}%)"
            )
        else:
            passes.append(f"{scene}: {before:.3f} ms -> {after:.3f} ms ({change:+.2f}%)")

    new = sorted(set(actual) - set(baseline))
    return passes, failures, new


def report(
    baseline_path: Path, actual_path: Path, threshold: float
) -> int:
    try:
        baseline_text = baseline_path.read_text(encoding="utf-8")
        actual_text = actual_path.read_text(encoding="utf-8")
        passes, failures, new = compare(baseline_text, actual_text, threshold)
    except (OSError, InputError) as error:
        print(f"perf-regression: FAIL -- {error}", file=sys.stderr)
        return 1

    print(
        f"perf-regression: {len(passes) + len(failures)} scene(s), "
        f"threshold {threshold:.2f}%"
    )
    for line in passes:
        print(f"  PASS  {line}")
    for scene in new:
        print(f"  NEW   {scene}: not in the baseline; no comparison was made")
    for line in failures:
        print(f"  FAIL  {line}")

    if failures:
        print(
            f"perf-regression: FAIL -- {len(failures)} scene(s) regressed or disappeared"
        )
        return 1
    print("perf-regression: PASS -- every baseline scene is within the threshold")
    return 0


def self_test() -> int:
    """Prove the gate fails on the exact 15% boundary and on bad input."""
    baseline = (
        "full_frame.median_ms=100.000\n"
        "full_frame.runs=15\n"
        "open_window_frame.median_ms=200.000\n"
        "open_window_frame.runs=15\n"
    )
    cases: list[tuple[str, str, float, bool, str]] = [
        ("unchanged", baseline, 15.0, True, ""),
        ("planted +15.0%", baseline.replace("100.000", "115.000"), 15.0, False, "15.00%"),
        ("+14.9%", baseline.replace("100.000", "114.900"), 15.0, True, ""),
        ("missing scene", "full_frame.median_ms=100.000\nfull_frame.runs=15\n", 15.0, False, "absent"),
        ("short run", baseline.replace("full_frame.runs=15", "full_frame.runs=14"), 15.0, False, "14 runs"),
        ("duplicate metric", baseline + "full_frame.median_ms=101.000\n", 15.0, False, "more than once"),
        ("negative median", baseline.replace("100.000", "-1.000"), 15.0, False, "non-negative"),
    ]

    failures: list[str] = []
    for name, actual, threshold, should_pass, needle in cases:
        try:
            _passes, errors, _new = compare(baseline, actual, threshold)
        except InputError as error:
            passed = False
            detail = str(error)
        else:
            passed = not errors
            detail = "; ".join(errors)
        if passed != should_pass:
            failures.append(f"{name}: expected pass={should_pass}, got pass={passed} ({detail})")
        elif needle and needle not in detail:
            failures.append(f"{name}: failure did not name {needle!r}: {detail}")

    if failures:
        print("perf-regression --self-test: FAIL")
        for line in failures:
            print(f"  {line}")
        return 1
    print(
        "perf-regression --self-test: PASS -- an unchanged run passes, a planted "
        "15.0% regression fails, +14.9% passes, and missing/short/duplicate/bad "
        "metrics fail"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, help="known-good render_bench output")
    parser.add_argument("--actual", type=Path, help="candidate render_bench output")
    parser.add_argument(
        "--threshold",
        type=float,
        default=DEFAULT_THRESHOLD,
        help=f"regression percentage that fails (default {DEFAULT_THRESHOLD:g})",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="prove the comparator fails on a planted 15% regression",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()
    if args.baseline is None or args.actual is None:
        parser.error("--baseline and --actual are required unless --self-test is given")
    return report(args.baseline, args.actual, args.threshold)


if __name__ == "__main__":
    sys.exit(main())
