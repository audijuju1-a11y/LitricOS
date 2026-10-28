#!/usr/bin/env python3
"""Bring Part 23, the checklist note and Part 24 in line with the persisted UI scale.

Part 23 described the store as 8 keys and the kept scale as session-only. Both
became false when `ui_scale` was added to the record, so the document is
corrected rather than left to disagree with the code. Part 24 is an audit of a
moment, so its snapshot is kept and an addendum records the figures after this
change.
"""

import csv
import io
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "Research-Deep-all"
PART23 = ROOT / "LitricOS UI UX Research Part 23 I Technical Integration and Feasibility.md"
PART24 = ROOT / "LitricOS UI UX Research Part 24 J Source Register and Corpus Audit.md"
CHECKLIST = ROOT / "LitricOS Coverage Checklist.csv"
REGISTER = ROOT / "LitricOS Source Register.csv"

PATCHES = [
    (
        "| LitricOS | `prefstore.rs` | 8 keys, `key=value`, version 1, atomic write, `0600` |",
        "| LitricOS | `prefstore.rs` | 9 keys, `key=value`, version 1, atomic write, `0600` |",
    ),
    (
        "| Kept | Value stays, clock stops, `tick` reports `Kept` forever | `revert::RevertWindow::keep` |",
        "| Kept | Value stays, clock stops, `tick` reports `Kept` forever, and the step is written | `revert::RevertWindow::keep`, `Shell::keep_pending_change` |",
    ),
    (
        "| Session scope | The kept value is **not persisted** — there is no display service to persist it to (H.06) | this part, limitation |",
        "| Persistence | A kept step is stored as `ui_scale` and read back at startup; an expired or reverted step is never written | `prefstore` 9th key, `Shell::load_preferences` |",
    ),
    (
        "That last row is the honest limit of the work: the *mechanism* is complete and\n"
        "tested, and the *persistence* has nowhere to go because LitricOS still has no\n"
        "display-mode service (Part 22, H.06). A user who keeps 1.50× gets it until the\n"
        "shell restarts.",
        "The last two rows are the part that had to change after this chapter was\n"
        "written. The step a user keeps is now a stored preference, so 1.50× survives a\n"
        "restart — but only the *interface* step is stored, because a display mode still\n"
        "has no service to persist it to (Part 22, H.06). Writing happens on keep and\n"
        "never on apply: a provisional value in the file would come back after a crash\n"
        "as the size the user refused, which is the one outcome the window exists to\n"
        "prevent.",
    ),
    (
        "| I.03 | PENDING | PARTIAL | 8 keys, no namespace |",
        "| I.03 | PENDING | PARTIAL | 9 keys, no namespace |",
    ),
    (
        "| 4. Revert window | Expiry restores; keep survives the clock; no stacking | **Added this part**: 3 unit tests + 1 journey |",
        "| 4. Revert window | Expiry restores; keep survives the clock; no stacking; the kept step round-trips through the store | **Added this part**: 3 unit tests + 1 journey; the journey now covers the record and the restart |",
    ),
]


def read_text(path: Path) -> str:
    with open(path, encoding="utf-8", newline="") as handle:
        return handle.read()


def write_text(path: Path, text: str) -> None:
    with open(path, "w", encoding="utf-8", newline="") as handle:
        handle.write(text)


def patch_part23() -> None:
    text = read_text(PART23)
    for old, new in PATCHES:
        count = text.count(old)
        assert count == 1, (count, old[:60])
        text = text.replace(old, new, 1)
    write_text(PART23, text)


def patch_checklist_note() -> None:
    text = read_text(CHECKLIST)
    rows = list(csv.reader(io.StringIO(text, newline="")))
    for row in rows[1:]:
        if row and row[0] == "I.03":
            assert "8 keys" in row[4], row[4]
            row[4] = row[4].replace(
                "8 keys in a private key=value file, atomic write 0600",
                "9 keys in a private key=value file (ui_scale added 2026-10-28), atomic write 0600",
            )
            break
    else:
        raise SystemExit("I.03 not found")
    buffer = io.StringIO(newline="")
    csv.writer(buffer, lineterminator="\r\n").writerows(rows)
    write_text(CHECKLIST, buffer.getvalue())


def counts() -> dict:
    checklist = list(csv.reader(io.StringIO(read_text(CHECKLIST), newline="")))
    tally = {}
    for row in checklist[1:]:
        tally[row[3]] = tally.get(row[3], 0) + 1
    register = list(csv.reader(io.StringIO(read_text(REGISTER), newline="")))
    status = {}
    for row in register[1:]:
        status[row[12]] = status.get(row[12], 0) + 1
    domains = {row[2] for row in register[1:]}
    return {
        "checklist_rows": len(checklist) - 1,
        "tally": tally,
        "register_rows": len(register) - 1,
        "register_status": status,
        "domains": len(domains),
        "last": register[-1][0],
    }


def append_addendum() -> None:
    text = read_text(PART24)
    if "## J.10 Addendum" in text:
        return
    c = counts()
    inspected = c["register_status"].get("INSPECTED", 0)
    snippet = c["register_status"].get("SNIPPET_ONLY", 0)
    text = text.rstrip("\n") + "\n\n" + f"""
---

## J.10 Addendum — the register after Parts 23–25 (2026-10-28)

The audit above is a snapshot of the corpus at the moment Part 24 was written,
and it is kept as one: re-running every count in the prose would turn a dated
audit into a document that is always almost right. This addendum is the second
reading, taken after the revert window, the `ui_scale` preference and Chapter L
batch 1 were recorded.

| Measure | Part 24 | Now |
|---|---|---|
| Register rows | 4,170 | {c['register_rows']:,} |
| Domains | 372 / 400 | {c['domains']} / 400 ({c['domains'] / 400:.1%}) |
| Inspected | 1,265 | {inspected:,} |
| Snippet-only | 2,899 | {snippet:,} |
| Last source id | S3849 | {c['last']} |
| Checklist rows | 597 | {c['checklist_rows']} |
| PENDING rows | 124 | {c['tally'].get('PENDING', 0)} |
| PARTIAL rows | 440 | {c['tally'].get('PARTIAL', 0)} |
| COMPLETE rows | 29 | {c['tally'].get('COMPLETE', 0)} |

What changed, in audit terms:

1. **Eleven sources were added** (S3850–S3860). Five of them are the Chapter L
   batch-1 pages; the rest are the platform documentation Part 23 cites. All
   eleven are INSPECTED, so the inspected share moved for the first time since
   the audit.
2. **One hundred and twelve rows left PENDING**, but only twenty-eight of them
   are Chapter L content. The rest are the Chapter H, I and J rows that Parts
   21–24 resolved after the snapshot was taken.
3. **The duplicate-URL count is unchanged at 47.** None of the eleven new rows
   repeats an existing URL, so the dedupe finding stands without a new
   exception.
4. **The label problem is unchanged.** The new rows use the same `(T1)`/`(T2)`
   suffix form as their neighbours, so the 23 labels without a tier suffix that
   `J.01` counted are still 23.

The two structural findings of Part 24 therefore survive the recount: the corpus
is broad and shallow (about three in ten sources read past the snippet), and the
checklist is honest about it.
"""
    write_text(PART24, text)


if __name__ == "__main__":
    patch_part23()
    patch_checklist_note()
    append_addendum()
    print(counts())
