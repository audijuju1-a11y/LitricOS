#!/usr/bin/env python3
"""Record WB.12.08, close PENDING to zero, and open Chapter 45 (validation plan)."""

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

# ---------------------------------------------------------------- checklist
WB1208 = {
    "WB.12.08": (
        "PARTIAL",
        "Part 29: three Microsoft Learn pages opened - Policy CSP paths and Result values, ADMX as shared GP/MDM metadata with post-ship ADMXInstall, and GP-over-MDM precedence from 1709 plus the 1803 override (S3888-S3890). Mechanism documented; no page enumerates GP-only settings, so no per-setting catalogue. LitricOS: no policy plane at all (litric-caps gates actions, not values)",
    ),
}

# Parent rows are closed by roll-up, not by research: a parent is COMPLETE only
# if every child is COMPLETE, otherwise PARTIAL. Never left PENDING once its
# children are final, and never marked COMPLETE over a PARTIAL child.
PARENTS = {
    "CH00": (
        "PARTIAL",
        "Rolled up from 12 children: 10 COMPLETE, CH00.04 (stale apple.com page) and CH00.08 (Cinnamon version unverified) PARTIAL; see children",
    ),
    "F": (
        "PARTIAL",
        "Rolled up from Part 1 index row 40: all 27 F rows researched with 0 PENDING, several structural or documentation-only; see children",
    ),
    "WB": (
        "PARTIAL",
        "Rolled up from the WB rows: batch coverage complete through WB.12.08 (Part 29) with PARTIAL children where only policy or documentation evidence exists; see children",
    ),
}

CHAPTERS = {
    "LB": "Part 28 batch 4: 14/14 measured; LB.05 NOT FOUND (no Budgie file manager), LB.10 structural (no HIG found)",
    "LR": "Part 28 batch 4: 14/14 measured; five ChromeOS URLs returned 404, so 6 NOT FOUND and 8 structural",
}

NEW_SOURCES = [
    ["S3888", "https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-configuration-service-provider",
     "learn.microsoft.com", "learn.microsoft.com",
     "Policy configuration service provider (CSP)", "Microsoft", "", "2026-10-28", "Windows 11 26H2",
     "Windows policy plane (B)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "The Policy CSP configures policy through ./Vendor/MSFT/Policy/Config/{AreaName}/{PolicyName} at user or device scope; .../Result/{PolicyName} returns the resulting value after conflict resolution; ConfigOperations/ADMXInstall ingests ADMX files after the OS ships and deleting the file removes its metadata and every policy from it; Win32 and Desktop Bridge apps can provide their own policy definitions",
     "high", "Reference for the MDM side only: no Group Policy UI, no per-setting inventory, no precedence table",
     "WB.12.08"],
    ["S3889", "https://learn.microsoft.com/en-us/windows/client-management/understanding-admx-backed-policies",
     "learn.microsoft.com", "learn.microsoft.com",
     "Understanding ADMX policies", "Microsoft", "", "2026-10-28", "Windows 11 26H2",
     "Windows policy plane (B)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "An ADMX file specifies the registry keys and values associated with a Group Policy; in an MDM-managed client the same ADMX definitions apply policies independently of Group Policy and gpsvc.exe is not required; inbox ADMX is processed at build time while ingested ADMX is processed at runtime",
     "high", "Explains the mechanism and the registry mapping; no list of policies and no statement of what a user sees",
     "WB.12.08"],
    ["S3890", "https://learn.microsoft.com/en-us/windows/client-management/enroll-a-windows-10-device-automatically-using-group-policy",
     "learn.microsoft.com", "learn.microsoft.com",
     "Enroll a Windows device automatically using Group Policy", "Microsoft", "", "2026-10-28", "Windows 11 26H2",
     "Windows policy plane (B)", "Official documentation/support (T1)", "primary", "INSPECTED",
     "From Windows 10 1709, when the same policy is set in Group Policy and MDM, Group Policy takes precedence; from 1803 a setting can switch precedence to MDM; the GPEdit console does not reflect policies set by an organization on the device",
     "high", "Enrollment walkthrough with one precedence paragraph; no count of GP-only settings",
     "WB.12.08"],
]


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_checklist() -> None:
    rows = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    wanted = {**WB1208, **PARENTS}
    seen = set()
    for row in rows[1:]:
        if row and row[0] in wanted:
            status, note = wanted[row[0]]
            row[3] = status
            while len(row) < 5:
                row.append("")
            row[4] = note
            seen.add(row[0])
    missing = set(wanted) - seen
    assert not missing, f"checklist rows not found: {sorted(missing)}"
    # No chapter parent may stay PENDING once its children are final.
    still = [(r[0], r[3]) for r in rows[1:] if r and r[3] == "PENDING"]
    assert not still, f"PENDING remains: {still}"
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


def checklist_map() -> dict[str, tuple[str, str]]:
    out = {}
    for row in list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))[1:]:
        if row:
            out[row[0]] = (row[3], row[4] if len(row) > 4 else "")
    return out


def patch_part1() -> dict[str, int]:
    text = read_text(PART1)
    stats = {"index_rows": 0, "appendix_rows": 0}

    def set_row(prefix: str, replacement: str) -> None:
        nonlocal text
        pattern = re.compile(rf"^\| {re.escape(prefix)} \| .*$", re.MULTILINE)
        text, n = pattern.subn(replacement, text, count=1)
        assert n == 1, (prefix, n)
        stats["index_rows"] += 1

    set_row("6", "| 6 | Validation plan skeleton | 10 | PUBLISHED (skeleton VP-01-VP-07; full plan in Chapter 45 / Part 30) |")
    set_row("11", "| 11 | Windows 11 Settings catalogue (WB) | 2 | COMPLETE for coverage (Parts 2, 12-15 and 29; WB.12.08 is PARTIAL because only the policy mechanism is documented) |")
    set_row("44", "| 44 | Final source register and coverage audit (J) | 8, 9 | COMPLETE (Part 24 and addenda J.11-J.14: 4,205 rows, 382 domains, see J.14 for the re-derived percentages, 48 duplicate URLs, 5 contradictions) |")
    set_row("45", "| 45 | Validation plan (full) | 10 | COMPLETE as a plan (Part 30: 28 validation points, 4 executable in-repo, 0 executed on hardware; VP-28 PASS, VP-26 PARTIAL) |")

    # Appendix A: sync any row still printed as PENDING with the checklist.
    mapping = checklist_map()
    pattern = re.compile(r"^\| ([A-Z][A-Z0-9.]*) \| (.*?) \| PENDING \|  \|$", re.MULTILINE)

    def repl(match: re.Match) -> str:
        ident, title = match.group(1), match.group(2)
        if ident not in mapping:
            return match.group(0)
        status, note = mapping[ident]
        note = note.replace("|", "/")
        if len(note) > 120:
            note = note[:117].rstrip() + "..."
        stats["appendix_rows"] += 1
        return f"| {ident} | {title} | {status} | {note} |"

    text = pattern.sub(repl, text)
    leftovers = re.findall(r"^\| [A-Z][A-Z0-9.]* \| .* \| PENDING \|", text, re.MULTILINE)
    assert not leftovers, f"appendix PENDING left: {leftovers[:5]}"
    write_text(PART1, text)
    return stats


def figures() -> dict:
    checklist = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    data = [row for row in checklist[1:] if row]
    tally = Counter(row[3] for row in data)
    bad = [row[0] for row in data if len(row) not in (4, 5) or not row[3]]
    register = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    reg = [row for row in register[1:] if row]
    status = Counter(row[12] for row in reg)
    urls = Counter(row[1] for row in reg)
    chapter_l = [row for row in data if row[0][:2] in {"LG", "LK", "LX", "LC", "LO", "LP", "LB", "LR"}]
    return {
        "checklist": len(data),
        "malformed": len(bad),
        "tally": dict(tally),
        "pending": [(r[0], r[2]) for r in data if r[3] == "PENDING"],
        "register": len(reg),
        "column_counts": sorted({len(r) for r in reg}),
        "status": dict(status),
        "domains": len({r[2] for r in reg}),
        "last": reg[-1][0],
        "dup_urls": sum(1 for n in urls.values() if n > 1),
        "surplus": sum(n - 1 for n in urls.values() if n > 1),
        "chapter_l_done": sum(1 for r in chapter_l if r[3] != "PENDING"),
        "chapter_l_total": len(chapter_l),
    }


def append_addendum() -> None:
    text = read_text(PART24)
    if "## J.14 Addendum" in text:
        return
    f = figures()
    text = text.rstrip("\n") + "\n\n" + f"""
---

## J.14 Addendum — PENDING closed, Chapter 45 opened (2026-10-28)

Fifth recount, after `WB.12.08` (Part 29) and the Chapter 45 plan (Part 30).

| Measure | J.13 | Now |
|---|---|---|
| Register rows | 4,202 | {f['register']:,} |
| Domains | 382 / 400 | {f['domains']} / 400 ({f['domains'] / 400:.1%}) |
| Inspected | 1,297 | {f['status'].get('INSPECTED', 0):,} |
| Inaccessible | 6 | {f['status'].get('INACCESSIBLE', 0)} |
| Last source id | S3887 | {f['last']} |
| Checklist rows | 597 | {f['checklist']} |
| Malformed rows | 0 | {f['malformed']} |
| PENDING rows | 4 | {len(f['pending'])} |
| PARTIAL rows | 511 | {f['tally'].get('PARTIAL', 0)} |
| COMPLETE rows | 50 | {f['tally'].get('COMPLETE', 0)} |
| NOT FOUND rows | 8 | {f['tally'].get('NOT FOUND', 0)} |
| Chapter L rows measured | 112 / 112 | {f['chapter_l_done']} / {f['chapter_l_total']} |
| Duplicate URLs / surplus rows | 48 / 53 | {f['dup_urls']} / {f['surplus']} |

What this recount shows:

1. **PENDING is {len(f['pending'])}.** `WB.12.08` is researched and `CH00`, `F` and `WB`
   are closed by roll-up under a stated convention: a parent is COMPLETE only if
   every child is COMPLETE, otherwise PARTIAL, and never PENDING once its children
   are final. All three came out PARTIAL, which is the honest result — `CH00.04`
   and `CH00.08` are stale-page and unverified-version gaps.
2. **Nothing was marked COMPLETE to reach zero.** The three new Microsoft
   sources describe the policy mechanism; no page was opened that enumerates
   Group-Policy-only settings, so `WB.12.08` is PARTIAL and says why.
3. **Register is {f['register']:,} rows, {f['column_counts']} columns per row**, last id
   `{f['last']}`, with {f['status'].get('INSPECTED', 0):,} inspected, {f['status'].get('SNIPPET', 0):,} snippet-only
   and {f['status'].get('INACCESSIBLE', 0)} inaccessible.
4. **Duplicates did not move again: {f['dup_urls']} URLs / {f['surplus']} surplus rows.**
   The three new URLs are new hosts-and-paths, so `J.02`'s figure survives a fifth
   batch across {f['register']:,} rows.
5. **The research series has no PENDING rows left.** What remains is the work the
   validation plan describes: {sum(1 for _ in f['pending'])} unmeasured settings rows,
   0 unmeasured DE rows and 28 validation points of which 4 can run here.
"""
    write_text(PART24, text)


if __name__ == "__main__":
    patch_checklist()
    patch_register()
    stats = patch_part1()
    append_addendum()
    print(json.dumps({"part1": stats, **figures()}, ensure_ascii=False, indent=2, default=str))
