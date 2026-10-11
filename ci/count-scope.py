#!/usr/bin/env python3
"""Verify that the SYSTEM-SCOPE tally matches the rows it summarises.

A hand-maintained progress number drifts, and a drifting progress number is how
a project starts believing it is further along than it is. This parses the rows
and fails if the summary table disagrees.

It caught its own motivating bug immediately: the first version of the tally was
written before the rows were finished and claimed 707 rows across 4 status
buckets. The parser found 854.

Also enforced:
  - numbering is contiguous from 1 with no gaps and no duplicates, so a row
    cannot be silently dropped or double-counted;
  - every row carries a status from the legend;
  - no row is marked `D` (done) unless docs/EVIDENCE.md exists, because `D`
    means "a test proves it" and EVIDENCE.md is where tests are recorded.

Usage:
    python3 ci/count-scope.py           check
    python3 ci/count-scope.py --fix     rewrite the tally to the measured values
"""

from __future__ import annotations

import collections
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCOPE = ROOT / "docs" / "SYSTEM-SCOPE.md"
EVIDENCE = ROOT / "docs" / "EVIDENCE.md"

ROW_RE = re.compile(r"^\|\s*(\d+)\s*\|(.+)$", re.M)
VALID_STATUS = {"D", "P", "S", "T"}

STATUS_LABEL = {
    "D": "`D` done and tested",
    "P": "`P` partial, real code with passing tests",
    "S": "`S` specified, no code",
    "T": "`T` not started",
}


def parse_rows(text: str) -> list[tuple[int, str]]:
    """Return (number, status) for every numbered feature row."""
    out: list[tuple[int, str]] = []
    for num, rest in ROW_RE.findall(text):
        cells = [c.strip() for c in rest.split("|")]
        status = ""
        for cell in reversed(cells):
            if cell:
                status = cell.replace("*", "").strip()
                break
        out.append((int(num), status))
    return out


def main(argv: list[str]) -> int:
    if not SCOPE.exists():
        print(f"ERROR: {SCOPE} not found")
        return 1

    text = SCOPE.read_text(encoding="utf-8")
    rows = parse_rows(text)
    if not rows:
        print("ERROR: no numbered feature rows found")
        return 1

    problems: list[str] = []
    numbers = [n for n, _ in rows]
    counts = collections.Counter(s for _, s in rows)
    total = len(rows)

    # --- numbering integrity ------------------------------------------------
    dupes = sorted(n for n, c in collections.Counter(numbers).items() if c > 1)
    if dupes:
        problems.append(f"duplicate row numbers: {dupes}")

    expected = set(range(1, max(numbers) + 1))
    gaps = sorted(expected - set(numbers))
    if gaps:
        problems.append(f"gaps in numbering: {gaps[:20]}{' ...' if len(gaps) > 20 else ''}")

    if min(numbers) != 1:
        problems.append(f"numbering starts at {min(numbers)}, expected 1")

    # --- status validity ----------------------------------------------------
    bad = sorted({s for s in counts if s not in VALID_STATUS})
    if bad:
        problems.append(f"rows with a status outside the legend {sorted(VALID_STATUS)}: {bad}")

    # --- `D` requires evidence ----------------------------------------------
    if counts.get("D", 0) > 0 and not EVIDENCE.exists():
        problems.append(
            f"{counts['D']} row(s) marked D (done) but docs/EVIDENCE.md does not exist. "
            "D means a test proves it; EVIDENCE.md is where tests are recorded."
        )

    # --- tally agreement ----------------------------------------------------
    # Read the declared numbers out of the TALLY table.
    declared: dict[str, int] = {}
    declared_total = None
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        for key, label in STATUS_LABEL.items():
            if label in line:
                m = re.search(r"\*\*(\d+)\*\*", line)
                if m:
                    declared[key] = int(m.group(1))
        if "**Total**" in line:
            m = re.search(r"\*\*(\d+)\*\*\s*\|", line.split("**Total**", 1)[1])
            if m:
                declared_total = int(m.group(1))

    for key in VALID_STATUS:
        want = counts.get(key, 0)
        got = declared.get(key)
        if got is None:
            problems.append(f"tally does not declare a count for status {key}")
        elif got != want:
            problems.append(f"tally says {key}={got}, rows say {key}={want}")

    if declared_total is None:
        problems.append("tally does not declare a total")
    elif declared_total != total:
        problems.append(f"tally says total={declared_total}, rows say total={total}")

    # --- report -------------------------------------------------------------
    print(f"rows        : {total}  (numbered {min(numbers)}..{max(numbers)})")
    for key in ("D", "P", "S", "T"):
        n = counts.get(key, 0)
        pct = (n * 100.0 / total) if total else 0.0
        print(f"  {key} : {n:>4}  {pct:5.1f}%")

    if "--fix" in argv and problems:
        # Only the tally numbers are rewritten; structural problems are never
        # "fixed" automatically, because renumbering rows silently would hide
        # the fact that a row went missing.
        structural = [p for p in problems if "tally says" not in p and "does not declare" not in p]
        if structural:
            print("\nrefusing --fix: structural problems must be fixed by hand")
            for p in structural:
                print(f"  - {p}")
            return 1
        new = text
        for key, label in STATUS_LABEL.items():
            n = counts.get(key, 0)
            pct = (n * 100.0 / total) if total else 0.0
            new = re.sub(
                rf"(\| {re.escape(label)} \| )\*\*\d+\*\*( \| )[\d.]+%",
                rf"\g<1>**{n}**\g<2>{pct:.1f}%",
                new,
            )
        new = re.sub(r"(\| \*\*Total\*\* \| )\*\*\d+\*\*", rf"\g<1>**{total}**", new)
        SCOPE.write_text(new, encoding="utf-8")
        print("\ntally rewritten to the measured values")
        return 0

    if problems:
        print(f"\n{len(problems)} problem(s):")
        for p in problems:
            print(f"  - {p}")
        return 1

    print("\nOK: the tally matches the rows")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
