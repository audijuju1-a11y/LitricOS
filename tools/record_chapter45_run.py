#!/usr/bin/env python3
"""Record the Chapter 45 wave-1 run: two sources, checklist note, index, J.15.

The register key defect found by `tools/validate_chapter45.py` (279 source ids
shared by up to six unrelated URLs) is *reported*, not repaired: renumbering
470 rows would make every existing citation ambiguous in a new way. What this
script does is record the two new sources that close WB.12.08's enumeration-method
gap and re-derive every published figure from the files.
"""

from __future__ import annotations

import csv
import io
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "Research-Deep-all"
CHECKLIST = ROOT / "LitricOS Coverage Checklist.csv"
REGISTER = ROOT / "LitricOS Source Register.csv"
PART1 = ROOT / "LitricOS UI UX Research Part1.md"
PART24 = ROOT / "LitricOS UI UX Research Part 24 J Source Register and Corpus Audit.md"

NEW_SOURCES = [
    [
        "S3891",
        "https://learn.microsoft.com/en-us/intune/device-configuration/import-group-policy-analytics",
        "learn.microsoft.com", "learn.microsoft.com",
        "Use Microsoft Intune to import and analyze group policies",
        "Microsoft", "", "2026-10-28", "Windows 11 26H2 + Intune",
        "Windows policy plane (B)",
        "Official documentation/support (T1)", "primary", "INSPECTED",
        "Group Policy analytics imports an on-premises GPO as XML and reports, per setting, MDM Support Yes or No; No means no matching setting is available to MDM providers including Intune; the tool parses Policy CSP, PassportForWork, BitLocker, Firewall, AppLocker CSP and Group Policy Preferences; settings outside those land in Unknown Settings; migration readiness labels each setting Ready for migration, Not supported or Deprecated",
        "high",
        "Needs an Intune tenant and an exported GPO; the GP-only list is produced per import, not published as a static catalogue",
        "WB.12.08",
    ],
    [
        "S3892",
        "https://www.microsoft.com/en-us/download/details.aspx?id=108395",
        "microsoft.com", "www.microsoft.com",
        "Group Policy Settings Reference Spreadsheet for Windows 11 2025 Update (25H2)",
        "Microsoft", "", "2026-10-28", "Windows 11 25H2",
        "Windows policy plane (B)",
        "Official documentation/support (T1)", "primary", "INSPECTED",
        "The spreadsheet lists the computer and user policy settings included in the administrative templates delivered with Windows 11 25H2, filterable by column, published 2025-10-02 as Windows11PolicySettings25H2.xlsx (786.5 KB), and describes them as the settings configurable when editing Group Policy Objects",
        "high",
        "The xlsx was not downloaded or parsed here, so the page names the authoritative list and its scope but not its rows",
        "WB.12.08",
    ],
]

WB1208_NOTE = (
    "Part 29: three Microsoft Learn pages opened - Policy CSP paths and Result values, "
    "ADMX as shared GP/MDM metadata with post-ship ADMXInstall, and GP-over-MDM precedence "
    "from 1709 (S3888-S3890). Part 31 closes the enumeration method: Intune Group Policy "
    "analytics reports MDM Support No per setting across six parsed CSP families (S3891), "
    "and the authoritative row-by-row list is the 25H2 spreadsheet (S3892); the list itself "
    "was not extracted, so the row stays PARTIAL rather than claiming a catalogue it does not have"
)


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_checklist() -> None:
    rows = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    seen = False
    for row in rows[1:]:
        if row and row[0] == "WB.12.08":
            row[3] = "PARTIAL"
            while len(row) < 5:
                row.append("")
            row[4] = WB1208_NOTE
            seen = True
    assert seen, "WB.12.08 not found"
    assert not [r[0] for r in rows[1:] if r and r[3] == "PENDING"], "PENDING rows remain"
    buffer = io.StringIO(newline="")
    csv.writer(buffer, lineterminator="\r\n").writerows(rows)
    write_text(CHECKLIST, buffer.getvalue())


def patch_register() -> None:
    rows = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    existing = {row[0] for row in rows[1:]}
    for source in NEW_SOURCES:
        assert source[0] not in existing, f"{source[0]} already recorded"
        assert len(source) == 17, (source[0], len(source))
        rows.append(source)
    buffer = io.StringIO(newline="")
    csv.writer(buffer, lineterminator="\r\n").writerows(rows)
    write_text(REGISTER, buffer.getvalue())


def patch_part1() -> None:
    text = read_text(PART1)
    old = re.compile(r"^\| 45 \| Validation plan \(full\) \| 10 \| .*$", re.MULTILINE)
    new = (
        "| 45 | Validation plan (full) | 10 | COMPLETE and executed (Part 30 plan; "
        "Part 31 wave 1: 8 repo-side checks, 1 PASS, 6 PARTIAL, 1 FAIL on a register "
        "key defect; 21 points still NOT RUN without hardware) |"
    )
    text, n = old.subn(new, text, count=1)
    assert n == 1, n
    # Appendix A carries the same note as the checklist; keep them identical.
    row = re.compile(r"^\| WB\.12\.08 \| .*$", re.MULTILINE)
    note = WB1208_NOTE.replace("|", "/")
    text, m = row.subn(f"| WB.12.08 | &nbsp;&nbsp;&nbsp;&nbsp;Group Policy / MDM-only settings | PARTIAL | {note} |", text, count=1)
    assert m == 1, m
    write_text(PART1, text)


def figures() -> dict:
    checklist = [row for row in csv.reader(io.StringIO(read_text(CHECKLIST), newline=""))][1:]
    data = [row for row in checklist if row]
    tally = Counter(row[3] for row in data)
    register = [row for row in csv.reader(io.StringIO(read_text(REGISTER), newline=""))][1:]
    reg = [row for row in register if row]
    ids = [row[0] for row in reg]
    counts = Counter(ids)
    urls = Counter(row[1] for row in reg)
    return {
        "checklist": len(data),
        "malformed": sum(1 for row in data if len(row) not in (4, 5) or not row[3]),
        "pending": sum(1 for row in data if row[3] == "PENDING"),
        "tally": dict(tally),
        "register": len(reg),
        "distinct_ids": len(counts),
        "shared_ids": sum(1 for n in counts.values() if n > 1),
        "max_per_id": max(counts.values()),
        "columns": sorted({len(row) for row in reg}),
        "last": ids[-1],
        "domains": len({row[2] for row in reg}),
        "inspected": sum(1 for row in reg if row[12] == "INSPECTED"),
        "snippet": sum(1 for row in reg if row[12] == "SNIPPET_ONLY"),
        "inaccessible": sum(1 for row in reg if row[12] == "INACCESSIBLE"),
        "dup_urls": sum(1 for n in urls.values() if n > 1),
        "surplus": sum(n - 1 for n in urls.values() if n > 1),
    }


def append_addendum() -> None:
    text = read_text(PART24)
    if "## J.15 Addendum" in text:
        return
    f = figures()
    text = text.rstrip("\n") + "\n\n" + f"""
---

## J.15 Addendum — Chapter 45 wave 1 executed (2026-10-28)

Sixth recount, after the Chapter 45 harness (`tools/validate_chapter45.py`) and
the two sources that close the `WB.12.08` enumeration-method gap (Part 31).

| Measure | J.14 | Now |
|---|---|---|
| Register rows | 4,205 | {f['register']:,} |
| **Distinct source ids** | not measured | {f['distinct_ids']:,} |
| **Source ids shared by 2+ rows** | not measured | {f['shared_ids']} (max {f['max_per_id']} rows per id) |
| Domains | 382 / 400 | {f['domains']} / 400 ({f['domains'] / 400:.1%}) |
| Inspected | 1,300 | {f['inspected']:,} |
| Snippet-only | 2,899 | {f['snippet']:,} |
| Inaccessible | 6 | {f['inaccessible']} |
| Last source id | S3890 | {f['last']} |
| Checklist rows | 597 | {f['checklist']} |
| Malformed rows | 0 | {f['malformed']} |
| PENDING rows | 0 | {f['pending']} |
| PARTIAL rows | 528 | {f['tally'].get('PARTIAL', 0)} |
| Duplicate URLs / surplus rows | 48 / 53 | {f['dup_urls']} / {f['surplus']} |

What this recount shows:

1. **The arithmetic is unchanged and re-derived.** `{f['register']:,}` rows, `{f['last']}`,
   `{f['inspected']:,}` inspected, `{f['dup_urls']}` duplicate URLs — every figure is recomputed
   by `figures()` in the harness, not copied from J.14.
2. **A defect J.01–J.14 never tested: `source_id` is not a key.** `{f['register']:,}` rows
   carry only `{f['distinct_ids']:,}` distinct ids; `{f['shared_ids']}` ids are shared by 2–{f['max_per_id']}
   unrelated URLs (for example `S0001` covers the Windows release-health page and five
   Apple HIG pages). The CSV is well-formed — 17 columns, 0 malformed — which is why
   every earlier audit passed it. Part 31 §5 states the numbers and the repair options;
   no id was renumbered here, because that would silently change what 9 existing
   citations in Parts 21 and 23 point at.
3. **`WB.12.08` keeps PARTIAL.** S3891 defines how a GP-only setting is identified
   (MDM Support = No over six parsed CSP families) and S3892 names the authoritative
   per-setting list, but the spreadsheet was not downloaded, so the row still has no
   materialised catalogue.
"""
    write_text(PART24, text)


if __name__ == "__main__":
    patch_checklist()
    patch_register()
    patch_part1()
    append_addendum()
    print(json.dumps(figures(), ensure_ascii=False, indent=2, default=str))
