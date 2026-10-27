#!/usr/bin/env python3
"""Check a running LitricOS machine's memory against the profile budget.

    ci/check-memory-snapshot.py --snapshot /run/litric-budget.txt
    ci/check-memory-snapshot.py --self-test

WHY THIS EXISTS
---------------
`docs/PERF-budget.md` carries per-line targets and a 320 MB TOTAL, and
`ci/check-memory-budget.py` checks only that every *claim* in the ledger matches
a shipped `MemoryMax=`. Both are static: a cap is where the cgroup OOM killer
starts, not what a service costs, and nothing on a booted machine ever added the
lines up. This is the runtime half.

The snapshot is written **inside the image** by
`image/mkosi.extra/usr/lib/litric/budget-snapshot` and contains the kernel's own
`/proc/meminfo`, each resident unit's cgroup `memory.current`/`memory.peak`, and
its PSS from `smaps_rollup`. This script does the arithmetic:

* whole-system used = `MemTotal - MemAvailable`, which is what the budget means;
* the sum of the resident units' PSS, and what the units do not account for;
* a FAIL when the used total is over the limit, or when a resident unit is
  missing from the snapshot. A missing unit is a failure rather than an
  omission, because the total would otherwise silently shrink.

WHAT THIS DOES NOT CLAIM
------------------------
The kernel, slab and page tables have no cgroup of their own; they appear only
in the "unaccounted" column. Without `--snapshot`, the script runs nothing and
says so: a self-test PASS proves the checker works, never that a machine is
inside its budget.
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass

# Units that are enabled at boot and are therefore part of an idle desktop.
# A unit here that is absent from a snapshot fails the check: the sum would be
# a sum of the units that happened to answer.
RESIDENT_UNITS = ("litric-shell", "litric-sysd", "litric-envd", "litric-notifyd")

PROTOCOL = "litric-budget/1"


@dataclass
class Finding:
    level: str  # "ERROR" or "INFO"
    subject: str
    message: str


def parse(text: str) -> dict[str, str]:
    """Parse the snapshot's `key=value` lines.

    The first `=` splits; a value may contain `=` (no current key does, but a
    version string could). Blank lines and `#` comments are ignored.
    """
    values: dict[str, str] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, value = line.partition("=")
        values[key.strip()] = value.strip()
    return values


def number(values: dict[str, str], key: str) -> int | None:
    raw = values.get(key)
    if raw is None:
        return None
    try:
        return int(raw)
    except ValueError:
        return None


def check(values: dict[str, str], limit_mb: int) -> tuple[list[Finding], dict[str, int]]:
    """Apply the runtime budget to a parsed snapshot.

    Returns the findings and the numbers the report prints. `used_mb` is
    `MemTotal - MemAvailable`; no page-cache subtraction happens here, because
    subtract the cache and the number stops describing memory the kernel can
    reclaim.
    """
    findings: list[Finding] = []
    numbers: dict[str, int] = {}

    if values.get("snapshot") != PROTOCOL:
        findings.append(
            Finding(
                "ERROR",
                "snapshot",
                f"protocol is {values.get('snapshot')!r}, expected {PROTOCOL!r}",
            )
        )
        return findings, numbers

    total = number(values, "mem.total_kb")
    available = number(values, "mem.available_kb")
    if total is None or available is None:
        findings.append(
            Finding("ERROR", "meminfo", "MemTotal/MemAvailable are missing from the snapshot")
        )
        return findings, numbers
    if available > total:
        findings.append(
            Finding("ERROR", "meminfo", f"MemAvailable {available} kB exceeds MemTotal {total} kB")
        )
        return findings, numbers

    used_kb = total - available
    numbers["total_mb"] = total // 1024
    numbers["used_mb"] = used_kb // 1024

    unit_pss = 0
    for unit in RESIDENT_UNITS:
        present = values.get(f"unit.{unit}.present")
        if present is None:
            findings.append(
                Finding(
                    "ERROR",
                    unit,
                    "missing from the snapshot: a resident unit that does not answer "
                    "would silently shrink the total",
                )
            )
            continue
        if present != "1":
            findings.append(
                Finding("ERROR", unit, "not running in an idle snapshot; it is enabled at boot")
            )
            continue
        pss = number(values, f"unit.{unit}.pss_kb")
        current = number(values, f"unit.{unit}.cgroup_current_kb")
        peak = number(values, f"unit.{unit}.cgroup_peak_kb")
        if pss is None:
            findings.append(Finding("ERROR", unit, "present but has no PSS reading"))
            continue
        if current is None:
            findings.append(Finding("ERROR", unit, "present but has no cgroup memory.current"))
            continue
        if peak is not None and peak < current:
            findings.append(
                Finding(
                    "ERROR",
                    unit,
                    f"memory.peak {peak} kB is below memory.current {current} kB",
                )
            )
        if pss > current:
            findings.append(
                Finding(
                    "ERROR",
                    unit,
                    f"PSS {pss} kB exceeds cgroup memory.current {current} kB",
                )
            )
        unit_pss += pss
        numbers[f"{unit}_mb"] = pss // 1024

    numbers["units_mb"] = unit_pss // 1024
    numbers["unaccounted_mb"] = max(used_kb - unit_pss, 0) // 1024
    numbers["limit_mb"] = limit_mb

    if numbers["used_mb"] > limit_mb:
        findings.append(
            Finding(
                "ERROR",
                "budget",
                f"used {numbers['used_mb']} MB is over the {limit_mb} MB budget "
                f"({numbers['unaccounted_mb']} MB of it outside the resident units' PSS)",
            )
        )
    tasks = number(values, "system.tasks")
    if tasks is not None:
        numbers["tasks"] = tasks
    return findings, numbers


def good_snapshot(used_mb: int = 200) -> str:
    total = 3_900 * 1024
    available = total - used_mb * 1024
    lines = [
        f"snapshot={PROTOCOL}",
        "snapshot.at=2026-10-27T00:00:00Z",
        f"mem.total_kb={total}",
        f"mem.available_kb={available}",
        "mem.memfree_kb=2500000",
        "mem.cached_kb=300000",
        "system.tasks=120",
    ]
    for unit, pss in (
        ("litric-shell", 70 * 1024),
        ("litric-sysd", 6 * 1024),
        ("litric-envd", 4 * 1024),
        ("litric-notifyd", 2 * 1024),
    ):
        lines.extend(
            [
                f"unit.{unit}.present=1",
                f"unit.{unit}.pid=100",
                f"unit.{unit}.cgroup_current_kb={pss + 512}",
                f"unit.{unit}.cgroup_peak_kb={pss + 2048}",
                f"unit.{unit}.pss_kb={pss}",
                f"unit.{unit}.rss_kb={pss + 1024}",
            ]
        )
    return "\n".join(lines) + "\n"


def self_test() -> int:
    failures = 0

    def expect(name: str, ok: bool) -> None:
        nonlocal failures
        print(f"  {'PASS' if ok else 'FAIL'}  {name}")
        if not ok:
            failures += 1

    findings, numbers = check(parse(good_snapshot()), 320)
    expect("a complete snapshot inside the budget passes", not findings)
    expect("used is MemTotal minus MemAvailable", numbers.get("used_mb") == 200)
    expect("unit PSS is summed", numbers.get("units_mb") == 82)

    findings, _ = check(parse(good_snapshot(340)), 320)
    expect(
        "a snapshot over the budget fails and names the total",
        any(f.level == "ERROR" and "over the 320 MB budget" in f.message for f in findings),
    )

    missing = good_snapshot().replace("unit.litric-envd.present=1\n", "")
    findings, _ = check(parse(missing), 320)
    expect(
        "a resident unit missing from the snapshot fails",
        any(f.subject == "litric-envd" and "missing" in f.message for f in findings),
    )

    no_pss = "\n".join(
        line
        for line in good_snapshot().splitlines()
        if not line.startswith("unit.litric-sysd.pss_kb=")
    )
    findings, _ = check(parse(no_pss), 320)
    expect(
        "a unit without a PSS reading fails rather than counting as zero",
        any(f.subject == "litric-sysd" and "no PSS" in f.message for f in findings),
    )

    wrong_protocol = good_snapshot().replace(PROTOCOL, "litric-budget/9")
    findings, _ = check(parse(wrong_protocol), 320)
    expect("an unknown protocol fails", any(f.subject == "snapshot" for f in findings))

    stopped = good_snapshot().replace(
        "unit.litric-notifyd.present=1", "unit.litric-notifyd.present=0"
    )
    findings, _ = check(parse(stopped), 320)
    expect(
        "a stopped resident unit fails: a quiet unit is not an absent one",
        any(f.subject == "litric-notifyd" and "not running" in f.message for f in findings),
    )

    print(f"self-test: {'PASS' if failures == 0 else 'FAIL'} ({failures} failed)")
    return 0 if failures == 0 else 1


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--snapshot", help="file written by litric-budget-snapshot")
    parser.add_argument("--limit", type=int, default=320, help="whole-system budget in MB")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()
    if not args.snapshot:
        print(
            "nothing to check: pass --snapshot from a booted image, or --self-test.\n"
            "A checker that did not read a machine cannot report one inside its budget.",
            file=sys.stderr,
        )
        return 2

    try:
        with open(args.snapshot, encoding="utf-8") as fh:
            values = parse(fh.read())
    except OSError as exc:
        print(f"cannot read snapshot: {exc}", file=sys.stderr)
        return 2

    findings, numbers = check(values, args.limit)
    print(f"snapshot {args.snapshot}")
    print(
        "  total {total} MB · used {used} MB · resident units {units} MB · "
        "unaccounted {unaccounted} MB · budget {limit} MB".format(**numbers)
    )
    if "tasks" in numbers:
        print(f"  tasks {numbers['tasks']}")

    errors = [f for f in findings if f.level == "ERROR"]
    for finding in findings:
        print(f"  {finding.level:<5} {finding.subject}: {finding.message}")
    if errors:
        print(f"RESULT: FAIL ({len(errors)} errors)")
        return 1
    print("RESULT: PASS -- the running machine is inside its whole-system budget")
    print(
        "NOTE: unaccounted memory includes the kernel, slab and page tables, "
        "which have no cgroup of their own."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
