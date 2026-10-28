#!/usr/bin/env python3
"""Make `source_id` a real key: first row keeps the id, later rows get a suffix.

The register was built batch by batch and early batches reused one id for every
URL in a topic group: `S0001` covered the Windows release-health page and five
Apple HIG pages, `S0112` covered a Windows display article, a WinUI component
library and a macOS `defaults` issue. The CSV stayed well-formed, so six audits
that checked its shape passed it; a lookup by id still landed on an arbitrary
row.

The rule applied here, in order:

1. rows are visited in file order;
2. the **first** row that carries an id keeps the bare `S####`;
3. every later row with the same id becomes `S####a`, `S####b`, ... in file
   order, which keeps lexical sorting equal to file order (`S0001 < S0001a <
   S0001b < S0002`);
4. every new id is asserted not to collide with an existing one;
5. the rewrite is all-or-nothing: the file is only written after every id in it
   is unique.

Suffixing is preferred to renumbering because a renumber shifts every id above
the duplicate, and 9 existing citations would then point at rows they never
meant. A suffix leaves every unaffected id stable and makes the first
occurrence - the one most citations use - keep its name.

The mapping is written to `validation/source_id_remap.csv` (row, old id, new id,
url). A second run is a no-op: it finds no duplicates, writes nothing, and
leaves the mapping untouched.

Usage:
    python3 tools/fix_source_id_keys.py            apply
    python3 tools/fix_source_id_keys.py --check    report only, exit 1 if dupes
"""

from __future__ import annotations

import csv
import io
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTER = ROOT / "Research-Deep-all" / "LitricOS Source Register.csv"
MAPPING = ROOT / "validation" / "source_id_remap.csv"


def load() -> tuple[list[list[str]], str]:
    raw = REGISTER.read_bytes()
    newline = "\r\n" if b"\r\n" in raw[:4096] else "\n"
    text = raw.decode("utf-8")
    return list(csv.reader(io.StringIO(text, newline=""))), newline


def main(argv: list[str]) -> int:
    check_only = "--check" in argv
    rows, newline = load()
    header, data = rows[0], [row for row in rows[1:] if row]
    ids = [row[0] for row in data]
    counts = Counter(ids)
    duplicates = {sid: n for sid, n in counts.items() if n > 1}

    if not duplicates:
        print(
            f"no-op: {len(data)} rows, {len(counts)} distinct ids, 0 duplicates; "
            "nothing written"
        )
        return 0

    if check_only:
        print(
            f"FAIL: {len(data)} rows, {len(counts)} distinct ids, "
            f"{len(duplicates)} duplicated ids ({sum(n - 1 for n in duplicates.values())} extra rows)"
        )
        return 1

    used = set(ids)
    seen: Counter[str] = Counter()
    changes: list[dict[str, object]] = []
    for index, row in enumerate(data, start=2):
        sid = row[0]
        if counts[sid] == 1:
            continue
        seen[sid] += 1
        if seen[sid] == 1:
            continue
        suffix = chr(ord("a") + seen[sid] - 2)
        new_id = f"{sid}{suffix}"
        assert new_id not in used, f"{new_id} already exists"
        assert len(row) == 17, (index, len(row))
        used.add(new_id)
        changes.append(
            {
                "row": index,
                "old_id": sid,
                "new_id": new_id,
                "url": row[1],
                "title": row[4],
            }
        )
        row[0] = new_id

    after = [row[0] for row in data]
    assert len(after) == len(set(after)), "ids still collide after rewrite"
    assert len(after) == len(data), "row count changed"

    MAPPING.parent.mkdir(exist_ok=True)
    exists = MAPPING.exists()
    with open(MAPPING, "a" if exists else "w", encoding="utf-8", newline="") as handle:
        if not exists:
            handle.write("row,old_id,new_id,url,title\n")
        for change in changes:
            url = str(change["url"]).replace('"', '""')
            title = str(change["title"]).replace('"', '""')
            handle.write(
                f'{change["row"]},{change["old_id"]},{change["new_id"]},'
                f'"{url}","{title}"\n'
            )

    buffer = io.StringIO(newline="")
    writer = csv.writer(buffer, lineterminator=newline)
    writer.writerows([header, *data])
    REGISTER.write_bytes(buffer.getvalue().encode("utf-8"))

    print(
        json.dumps(
            {
                "rows": len(data),
                "before_distinct": len(counts),
                "duplicated_ids": len(duplicates),
                "renamed_rows": len(changes),
                "after_distinct": len(set(after)),
                "mapping": str(MAPPING.relative_to(ROOT)),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
