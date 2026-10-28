# LitricOS UI/UX Research — Part 32
## Register key repair: `source_id` becomes a key

**Part:** 32
**Chapter:** J — source register and corpus audit (repair of a defect found in Chapter 45 wave 1)
**Predecessor:** Part 31 (§5), which measured the defect and left the repair open
**Date:** 2026-10-28
**New URLs opened for this part:** none. This is a data-correctness repair over rows that already exist.
**Artifacts:** `tools/fix_source_id_keys.py`, `validation/source_id_remap.csv`, refreshed `validation/receipts.jsonl`

---

## 1. The rule

Applied in file order to `Research-Deep-all/LitricOS Source Register.csv`:

1. Rows are visited top to bottom.
2. The **first** row carrying an id keeps the bare `S####`.
3. Every **later** row with the same id becomes `S####a`, `S####b`, `S####c`, … in file order.
4. A new id is rejected if it collides with any existing id.
5. The file is written only after every id in the result is unique; otherwise the script aborts
   without touching it.

The suffix preserves lexical order: `S0001 < S0001a < S0001b < S0002`. A reader who sorts the CSV
by `source_id` gets the same order as the file, and the first occurrence — the one most citations
already used — keeps its name.

### Why suffixing, not renumbering

Renumbering every row above a duplicate would shift thousands of ids and break the meaning of every
citation that used one. Suffixing changes only the 470 rows that were unreachable, leaves the other
3,737 ids byte-identical, and makes the defect's history readable: a suffixed id is visibly "the
second URL that used to share this number".

The alternative considered and rejected was a new-id block appended at the end (`S3893`…). It would
also produce unique ids, but it would move 470 rows away from the topic they belong to, break the
id-ordering relationship the file has had since Part 2, and inflate the duplicate-URL count as a
side effect.

---

## 2. Before and after

| Measure | Before | After |
|---|---|---|
| Register rows | 4,207 | **4,207** |
| Distinct `source_id` values | 3,737 | **4,207** |
| Ids shared by 2+ rows | **279** | **0** |
| Rows with a suffixed id | 0 | **470** |
| Rows unreachable by id | 470 | **0** |
| Columns per row | 17 | **17** |
| Last id in file order | S3892 | **S3892** |

Suffix distribution, re-derived from the mapping file:

| Suffix | Rows | Meaning |
|---|---|---|
| bare | 3,737 | first occurrence of its id |
| `a` | 279 | second occurrence |
| `b` | 155 | third |
| `c` | 25 | fourth |
| `d` | 8 | fifth |
| `e` | 3 | sixth (`S0001e`, `S0007e`, `S0023e`) |
| **total** | **4,207** | |

The worst groups were exactly the ones Part 31 named: `S0001` (Windows release health + five Apple HIG
pages), `S0007` and `S0023`, each split into six rows.

---

## 3. Citation resolution

Part 31 measured 9 citations across Parts 21 and 23 that referenced a shared id. Each was read in
context and resolved against the row it actually means. Seven already pointed at the first row, which
is the right page; one needed the suffix; one cluster was citing the wrong pages entirely.

| Part:line | Citation | Resolved to | Why |
|---|---|---|---|
| Part 21:324 | `[S0115]` | `S0115` (unchanged) | first row is *Change your screen resolution and layout in Windows* — the display-change/revert claim |
| Part 21:395 | `S0278` | `S0278` (unchanged) | first row is *Connect to a Wi-Fi network in Windows* |
| Part 21:426 | `S0278` | `S0278` (unchanged) | same row, status line for H.08 |
| Part 21:577 | `S0115`, `S0278` | unchanged | the reuse list names exactly those two topics, and the base rows are those pages |
| Part 23:574 | `S0112` (`defaults`) | **`S0112b`** | row 3,814 is the macOS `defaults` write issue; `S0112` is a Windows windowed-games article and `S0112a` a WinUI component library |
| Part 23:575 | `S0130`, `S0131` (notification APIs) | unchanged | both base rows are the Windows notifications / Do Not Disturb pages, which is the notification surface the sentence compares |
| Part 23:576 | `S0174`/`S1103`/`S1507`/`S1521`/`S1669` (portals) | **`S3852`** (XDG Desktop Portal) | all five ids are Windows help-desk PDFs about activation, printing, touch keyboards or quick references; none is a portal. Part 23's own new-source table lists `S3852` as the portal source, so the parenthetical now names it |

Two lines changed. No citation is left pointing at an id whose row is a different URL from the one
the sentence describes.

**Repaired after the first pass:** the same "Reused" paragraph cited `S0930` as "registry", while
`S0930` is a Windows-forum search article. No registry source exists in the register for that claim,
so the false attribution was removed from Part 23 rather than repointed to a page that would have
been equally wrong. The rest of the "Reused" list still needs a dedicated citation audit.

---

## 4. Enforcement

`VP-28b` in `tools/validate_chapter45.py` now checks the invariant that matters:

* distinct `source_id` count **==** row count;
* every id appears exactly once;
* `python3 tools/fix_source_id_keys.py --check` exits 0 (i.e. a re-run would be a no-op).

The check is not satisfied by "no duplicate URLs" or "17 columns" — both of which the broken register
already had.

Harness after the repair:

```text
VP-28: PASS    {... "register_rows": 4207, "distinct_source_ids": 4207, "duplicate_source_ids": 0 ...}
VP-28b: PASS   {"rows": 4207, "distinct_ids": 4207, "shared_ids": 0, "remap_guard": "no-op: ..."}
...
8 checks: 2 PASS, 6 PARTIAL, 0 FAIL
```

`VP-24`, `VP-11`, `VP-09`, `VP-01`, `VP-02` and `VP-26` remain `PARTIAL` for the reason Part 31 gave:
their criteria have a device half this workspace cannot run.

---

## 5. Idempotency evidence

Run twice, with the register and mapping hashed after each run:

```text
$ python3 tools/fix_source_id_keys.py
{"rows": 4207, "before_distinct": 3737, "duplicated_ids": 279,
 "renamed_rows": 470, "after_distinct": 4207}

64f77495050debc3e7e2df5ebdc2391295d35a9a917191a84a3dc25e7260d535  LitricOS Source Register.csv
e55b913c936bd5cd8c75bbbb48bc0a02997c9ece1f4d24358bb9e561b6269b82  validation/source_id_remap.csv

$ python3 tools/fix_source_id_keys.py
no-op: 4207 rows, 4207 distinct ids, 0 duplicates; nothing written

64f77495050debc3e7e2df5ebdc2391295d35a9a917191a84a3dc25e7260d535  LitricOS Source Register.csv
e55b913c936bd5cd8c75bbbb48bc0a02997c9ece1f4d24358bb9e561b6269b82  validation/source_id_remap.csv
```

Both hashes are identical across the two runs. The mapping file has 470 data rows plus a header and
is never rewritten by a no-op run.

---

## 6. What this repair changes in the corpus

| Artifact | Change |
|---|---|
| `LitricOS Source Register.csv` | 470 `source_id` values suffixed; 17 columns and row order untouched |
| `validation/source_id_remap.csv` | new: row, old id, new id, URL, title for every rename |
| `tools/fix_source_id_keys.py` | new: the rule, the assertions, `--check` mode |
| `tools/validate_chapter45.py` | `VP-28b` now enforces uniqueness and the no-op guard |
| Part 23 | two citation lines corrected (`S0112b`, `S3852`) |
| Part 24 J.15 | post-fix distinct/shared counts, pre-fix numbers marked as history |
| Part 31 §5 | marked as the pre-fix measurement, pointer to this part |
| Part 1 row 45 | gate now records `VP-28b` PASS |

No research claim was re-derived, no URL was added and no Rust file was touched.

---

## 7. Limitations

* **The repair is mechanical, the citation audit is not.** Only the 9 citations that referenced a
  shared id were read in context. The rest of the "Reused" lists were not re-audited; `S0930` is a
  known example that still needs one.
* **Suffixing encodes history, not meaning.** `S0001a` says "second URL that used to be S0001"; it
  does not say which chapter uses it. That is the register's existing design, not a new property.
* **No new sources.** This part opens no URL, so the domain/inspected counters do not move; the only
  numbers that change are the id counts.
* **Not run on real hardware** — irrelevant here, but still true: nothing in this part observes a
  desktop. The Chapter 45 gate stays where Part 31 left it, minus one `FAIL`.
* **Nothing was committed by this part alone.** The delivery commit and push are recorded in the
  final report of the turn.
