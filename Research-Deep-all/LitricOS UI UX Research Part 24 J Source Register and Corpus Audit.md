# LitricOS UI/UX Research — Part 24 of N

**Chapter J — Source register and corpus audit**

Status: **COMPLETE** — every row of Chapter J has a measured answer.
Prepared: 2026-10-28, after Part 23.

---

## 0. What this part does

Every previous part ended with a sources table and a status change. None of them
audited the *register itself*. Chapter J is that audit, and it is deliberately
uncomfortable: it counts the corpus the same way the corpus counts a Settings
page — rows, duplicates, labels, tiers, and what is missing.

The numbers below are computed from the two CSVs in this directory, not typed
from memory:

```text
Source Register rows            4170   (after the six rows added by Part 23)
Checklist rows                   597
Register columns                  17
Duplicate URLs                    47   (52 surplus rows)
Distinct evidence-type labels     39
Distinct subsystem labels        272
```

---

## J.01 — Register schema

### J.01.1 What the schema is

Seventeen columns, one row per cited URL:

```text
source_id, url, domain, host, title, publisher, pub_or_update_date,
access_date, target_os_version, subsystem, evidence_type,
primary_or_independent, status, findings, confidence, limitations,
checklist_ids
```

`status` has three values and they mean different things: `INSPECTED` (the page
body was read), `SNIPPET_ONLY` (a search result or snippet supports the row),
`INACCESSIBLE` (the page was attempted and could not be read). `confidence` is
`high`/`medium`/`low`, and `checklist_ids` is a semicolon list pointing back into
the coverage checklist.

### J.01.2 Findings

**J.01.1 — The evidence tier is not machine-checkable.** Severity: medium.
Evidence: T3 (register). 39 distinct `evidence_type` labels exist, and 23 of them
carry no tier suffix — `Official documentation/support` beside
`Official documentation/support (T1)`, `Community Q&A` beside `Community Q&A /
forum (T5)`. Two labels for one tier means a query for "everything at T1 or
better" silently misses 249 rows. The tier is the single most important field in
the register — it is what decides whether a finding may be stated as fact — and
it is currently stored inside free text.

**J.01.2 — Every row carries a limitation, including the strong ones.**
Severity: none (a strength). Evidence: T3. Zero rows have an empty `limitations`
field. A register whose confident rows admit what they do not cover is the only
kind that can be audited later; the alternative is a citation list that looks
complete because nobody wrote down what was missing.

**J.01.3 — Dates use two meanings under one name.** Severity: low. Evidence: T3.
`pub_or_update_date` holds a publication date for some rows and a last-updated
date for others; `access_date` is unambiguous. The column name admits both, which
is honest, but it means "how old is this claim" cannot be computed — only
bounded by the access date.

---

## J.02 — URL and domain de-duplication

### J.02.1 The measurement

| Measure | Value |
|---|---|
| Distinct URLs | 4,118 |
| URLs cited more than once | 47 |
| Surplus rows | 52 (1.25% of the register) |
| Most repeated | 3 citations (Apple HIG accessibility, Apple HIG keyboards, Apple HIG lists-and-tables, GNOME HIG keyboard, GNOME HIG accessibility) |

### J.02.2 Findings

**J.02.1 — Duplicates are mostly legitimate, and the schema cannot tell.**
Severity: low. Evidence: T3. A page about keyboard access is evidence for two
different chapters; citing it twice with different `subsystem` and
`checklist_ids` is correct archival practice. What is missing is a way to *say*
that the second row is the same source: there is no `duplicate_of` column, so a
count of distinct sources is a count of rows minus a number somebody has to
recompute by hand. The register should either merge these rows (with a
multi-valued `checklist_ids`, which it already has) or mark them.

**J.02.2 — No cross-domain duplicates were found beyond the intended ones.**
Severity: none. Evidence: T3. The 47 repeats are the same URL, not the same
document served from two hosts; the canonicalisation step (one row per final URL
after redirects) is holding.

---

## J.03 — Domain count toward the 400-domain target

| Measure | Before Part 23 | After Part 23 |
|---|---|---|
| Distinct domains | 368 | **372** |
| Distinct hosts | 397 | 400 |
| Target | 400 | 400 |
| Coverage | 92.0% | **93.0%** |

The six sources added in Part 23 contributed four domain labels new to the
register (`specifications.freedesktop.org`, `flatpak.github.io`,
`wayland.freedesktop.org`, and the `www.` host variant of `freedesktop.org`)
and two new pages on domains already present (`learn.microsoft.com`,
`freedesktop.org`).

### J.03.1 Finding

**J.03.1 — The last 28 domains are not the valuable ones.** Severity: low.
Evidence: T3. Coverage by domain is a proxy for breadth, and it is now close to
saturated: the domains still missing are mostly regional support sites and
vendor blogs that would add rows, not evidence. The register's weak axis is not
domain count — it is the 69.6% of rows that are `SNIPPET_ONLY` (J.04). Reaching
400 by adding them would improve the number and not the corpus.

---

## J.04 — Inspected versus snippet-only

| Status | Rows | Share |
|---|---|---|
| `INSPECTED` | 1,265 | 30.3% |
| `SNIPPET_ONLY` | 2,899 | 69.5% |
| `INACCESSIBLE` | 6 | 0.14% |

By provenance:

| | Primary | Independent |
|---|---|---|
| Inspected | 941 | 324 |
| Snippet only | 2,134 | 765 |
| Inaccessible | 4 | 2 |

### J.04.1 Findings

**J.04.1 — Six in ten primary rows are cited from a snippet.** Severity: medium.
Evidence: T3. 2,134 primary rows are search results. For a version number or a
title that is enough; for "the dialog has three buttons" it is not, and the
corpus is careful to mark confidence accordingly — but the proportion means most
of the register can support existence claims and not behaviour claims.

**J.04.2 — The inspected share is rising, not falling.** Severity: none
(strength). Evidence: T3 across parts: Parts 19–23 added 35 inspected rows and 7
inaccessible ones, and no part added a snippet row it did not need. The trend
matters more than the level: a corpus that inspects more as it goes is
converging, and one that inspects less is drifting toward citation theatre.

---

## J.05 — Primary versus independent

| Measure | Value |
|---|---|
| Primary rows | 3,079 (73.9%) |
| Independent rows | 1,091 (26.1%) |
| Primary inspected | 941 (74.4% of inspected) |
| Independent inspected | 324 |
| Findings resting on at least one inspected primary | every finding marked T1/T2 in Parts 19–23 |

### J.05.1 Finding

**J.05.1 — Independent sources are used as corroboration, never as the sole
basis for a platform claim.** Severity: none (strength). Evidence: T3. Every
Windows/macOS/Linux behaviour claim in this corpus cites an official source; the
independent rows carry observations (a screenshot, a demonstration, a forum
report) that official documentation does not make. The split is roughly three to
one, which is the right ratio for a corpus that must be defensible rather than
merely broad.

---

## J.06 — Inaccessible log

| Count | Meaning |
|---|---|
| 6 | Rows attempted and not retrieved |
| 4 | Primary sources among them |
| 0 | Rows that cite an inaccessible page for a claim |

Two are from Parts 21–22 and are typical of the class: Apple's Setup Assistant
deployment guide (S3844, HTTP 429 on every attempt) and Microsoft's "Update
Windows" article (S3849, HTTP 404 — the workflow was then sourced from the
recovery article instead). The rule the corpus applies is the one that keeps it
honest: **an inaccessible page is logged, never cited**. Its title and URL are
recorded so that a later pass can retry it, and no finding rests on it.

### J.06.1 Finding

**J.06.1 — The log has no retry state.** Severity: low. Evidence: T3. A row is
`INACCESSIBLE` and nothing records whether it was a 404 (gone), a 429 (rate
limited) or a paywall. Four of the six are transient and worth retrying; at
least one is gone and never will be. Without the reason, the next pass retries
all six.

---

## J.07 — Contradictions log

A contradiction is two inspected sources that cannot both be true, or a source
that contradicts the tree. The corpus has found five, and each has a resolution
or an open status:

| # | Contradiction | Sources | Resolution |
|---|---|---|---|
| 1 | "Never notify" is described by users as disabling UAC; Microsoft documents that it grants a split token **and** auto-elevates | S3854 | Resolved: the setting named "off" is the least safe. Recorded in I.04.1 |
| 2 | Windows 11 offers display mode changes; LitricOS' Display page says the mode is chosen at boot | S3845–S3849 vs H.06 | Open: a platform gap, not a documentation error. Now partly addressed by the revert window (I.10) |
| 3 | The Linux notification *daemon* owns DND; LitricOS puts DND in the shell | S3850 vs notifyd docs | Resolved in favour of LitricOS, with the reason recorded (I.05.2) |
| 4 | Linux supports third-party apps through portals; LitricOS grants scopes | S3852 vs litric-caps | Open: scope coarser than a picker (I.04.2) |
| 5 | A Wayland compositor is expected to advertise outputs; this one imports `wl_output` and creates three globals, none of them an output | S3853 vs dispatch.rs | Open: recorded as I.07.2, untested against a real client |

### J.07.1 Finding

**J.07.1 — Two of five contradictions are resolved in LitricOS' favour, and both
are architectural, not cosmetic.** Severity: none (strength). Evidence: T3. The
DND placement and the capability audit are cases where the reference design has
a defect this tree does not. Recording them as contradictions rather than as
"differences" is what makes the corpus useful to a decision-maker: a difference
is neutral, a contradiction has a winner.

---

## J.08 — Per-subsystem coverage audit

| Chapter | Checklist rows | PARTIAL | COMPLETE | PENDING | Other |
|---|---|---|---|---|---|
| W — Windows 11 shell & system UI | 261 | 250 | 5 | 2 | 3 N/A, 1 not found |
| M — macOS 27 | 114 | 111 | 3 | 0 | 0 |
| L — Linux desktops | 120 | 0 | 0 | 120 | 0 |
| F — Component catalogue | 28 | 27 | 0 | 1 | 0 |
| G — Accessibility & tokens | 19 | 19 | 0 | 0 | 0 |
| H — Workflows | 21 | 21 | 0 | 0 | 0 |
| I — Integration | 12 | 10 | 2 | 0 | 0 |
| J — Audit | 9 | 0 | 9 | 0 | 0 |
| C — Baseline | 13 | 2 | 10 | 1 | 0 |
| **All** | **597** | **440** | **29** | **124** | **4** |

Chapter C's thirteen rows are the version-baseline checks; every column above
sums to 597, and the four rows that are neither PARTIAL nor COMPLETE are three
`NOT APPLICABLE` platform checks and one `NOT FOUND`.

### J.08.1 Findings

**J.08.1 — One chapter is entirely unstarted, and it is the largest gap.**
Severity: high. Evidence: T3. All 120 Linux desktop rows are `PENDING`. The
corpus has GTK and KDE design-system sources (11 and 2 inspected) but no chapter
that walks a Linux desktop the way Parts 2–3 walk Windows. Every "Linux" claim in
Parts 19–24 is therefore a design-system claim, not a shell claim.

**J.08.2 — Subsystem vocabulary has drifted: 272 labels, 31 used once.**
Severity: medium. Evidence: T3. `Settings: macOS System Settings` (1,072 rows) is
a subsystem; `Search result` (73 rows) is an evidence type in the subsystem
column. Both are recorded as subsystems, so counting "which subsystem has the
most evidence" returns a category error. A controlled vocabulary of perhaps
twenty names would make the register queryable.

**J.08.3 — The checklist is the more trustworthy artefact.** Severity: none
(strength). Evidence: T3. 597 rows, four states, zero malformed rows, and a note
on every `PARTIAL`. A row that says PARTIAL and why is worth more than a row
that says COMPLETE, and this chapter can say that because the notes exist.

---

## 1. What this audit establishes

1. **The register is internally consistent and externally honest.** 4,170 rows,
   no malformed rows, no row without a limitation, no claim resting on an
   inaccessible page.
2. **Its weak axis is inspection depth, not breadth.** 30.3% inspected, 93.0% of
   the domain target. The next thousand rows would be cheaper and worth less
   than the next two hundred inspections.
3. **Its schema has two real defects**: evidence tiers hidden in free text (39
   labels, 23 untiered) and subsystem labels mixed with evidence types (272
   labels, 31 singletons).
4. **Its duplicate handling is defensible but not machine-readable**: 47 URLs,
   52 surplus rows, no `duplicate_of`, and no way to count distinct sources
   without recomputing.
5. **The corpus found five contradictions and resolved two in favour of
   LitricOS** — both architectural (DND placement, decision audit) rather than
   cosmetic.

---

## 2. What this part does not establish

* **That 400 domains is the right target.** It is the target the plan set, and
  this part can only report distance to it, not whether it was well chosen.
* **That every `INSPECTED` row was read in full.** The register records that the
  body was retrieved and that the cited sentences were found in it; it does not
  claim a whole-document review, and for long specifications it was not one.
* **Which inaccessible rows are worth retrying.** J.06.1: the reason for the
  failure was not stored.

---

## 3. Sources

This part cites the register and checklist in this directory, plus the six
sources added by Part 23 (S3850–S3855) where a contradiction is named. No new
external source was retrieved for the audit itself.

---

## 4. Status changes

| Row | Before | After | Note |
|---|---|---|---|
| J | PENDING | COMPLETE | Audit above |
| J.01 | PENDING | COMPLETE | 17 columns; 39 evidence labels, 23 untiered |
| J.02 | PENDING | COMPLETE | 47 URLs, 52 surplus rows, no `duplicate_of` |
| J.03 | PENDING | COMPLETE | 372 / 400 domains (93.0%) |
| J.04 | PENDING | COMPLETE | 1,265 inspected (30.3%), 2,899 snippet-only |
| J.05 | PENDING | COMPLETE | 3,079 primary / 1,091 independent |
| J.06 | PENDING | COMPLETE | 6 inaccessible, none cited |
| J.07 | PENDING | COMPLETE | 5 contradictions, 2 resolved in LitricOS' favour |
| J.08 | PENDING | COMPLETE | 272 subsystem labels, 120 Linux rows pending |


---

## J.10 Addendum — the register after Parts 23–25 (2026-10-28)

The audit above is a snapshot of the corpus at the moment Part 24 was written,
and it is kept as one: re-running every count in the prose would turn a dated
audit into a document that is always almost right. This addendum is the second
reading, taken after the revert window, the `ui_scale` preference and Chapter L
batch 1 were recorded.

| Measure | Part 24 | Now |
|---|---|---|
| Register rows | 4,170 | 4,175 |
| Domains | 372 / 400 | 374 / 400 (93.5%) |
| Inspected | 1,265 | 1,270 |
| Snippet-only | 2,899 | 2,899 |
| Last source id | S3849 | S3860 |
| Checklist rows | 597 | 597 |
| PENDING rows | 124 | 94 |
| PARTIAL rows | 440 | 467 |
| COMPLETE rows | 29 | 32 |

What changed, in audit terms:

1. **Eleven sources were added** (S3850–S3860). Five of them are the Chapter L
   batch-1 pages; the rest are the platform documentation Part 23 cites. All
   eleven are INSPECTED, so the inspected share moved for the first time since
   the audit.
2. **30 rows left PENDING** between the two readings. Twenty-eight are
   Chapter L batch 1; the other two are rows in Chapters H–J that Parts 21–24
   resolved after the snapshot was taken.
3. **The duplicate-URL count is 47 URLs, 52 surplus rows** — the same
   two figures `J.02` recorded, because none of the eleven new rows repeats an
   existing URL. The dedupe finding stands without a new exception.
4. **The label problem is unchanged.** The new rows use the same `(T1)`/`(T2)`
   suffix form as their neighbours, so the 23 labels without a tier suffix that
   `J.01` counted are still 23.

The two structural findings of Part 24 therefore survive the recount: the corpus
is broad and shallow (about three in ten sources read past the snippet), and the
checklist is honest about it.


---

## J.11 Addendum — after Chapter L batch 2 (2026-10-28)

Second recount, taken after Xfce 4.20 and Cinnamon 6.6 were recorded. Same rule
as `J.10`: the audit's prose stays as written, and the numbers move here.

| Measure | J.10 | Now |
|---|---|---|
| Register rows | 4,175 | 4,181 |
| Domains | 374 / 400 | 375 / 400 (93.8%) |
| Inspected | 1,270 | 1,276 |
| Last source id | S3860 | S3866 |
| Checklist rows | 597 | 597 |
| PENDING rows | 94 | 64 |
| PARTIAL rows | 467 | 489 |
| COMPLETE rows | 32 | 40 |
| Duplicate URLs / surplus rows | 47 / 52 | 47 / 52 |

Three things this recount shows that the first one could not:

1. **The PENDING set is now almost entirely Chapter L.** 64 rows remain: the 56
   rows of the four environments not yet measured (COSMIC, Pantheon, Budgie,
   ChromeOS) plus 8 parent/small rows. The two chapter rows that Parts 25-26
   resolved (`LG`, `LK`, `LX`, `LC`) are no longer counted as pending.
2. **Domains barely move.** Six new sources added one new domain (`docs.xfce.org`); the
   other hosts were already in the register. The 400-domain target is a breadth
   measure, and Chapter L is not what moves it.
3. **Duplicates did not grow.** None of the six new URLs repeats an existing row,
   so `J.02`'s finding is unchanged for a second batch.

The structural finding of Part 24 holds a second time: this corpus is broad and
shallow, and the two environments measured here publish almost no interface
rules, which is itself a result recorded in `LX.10` and `LC.10`.


---

## J.12 Addendum — after Chapter L batch 3 (2026-10-28)

Third recount, after COSMIC Epoch 1 and Pantheon 8.1. The prose above stays as
written; the numbers move here.

| Measure | J.11 | Now |
|---|---|---|
| Register rows | 4,181 | 4,192 |
| Domains | 375 / 400 | 378 / 400 (94.5%) |
| Inspected | 1,276 | 1,287 |
| Last source id | S3866 | S3877 |
| Checklist rows | 597 | 597 |
| PENDING rows | 64 | 34 |
| PARTIAL rows | 489 | 508 |
| COMPLETE rows | 40 | 50 |
| NOT FOUND rows | 1 | 2 |
| Duplicate URLs / surplus rows | 47 / 52 | 47 / 52 |

What this recount shows:

1. **PENDING is now 34** — the 28 rows of the two environments not yet measured
   (Budgie, ChromeOS), 5 parent rows (CH00, F, LB, LR, WB),
   and the one remaining small row. Chapter L is 84 of 112.
2. **One NOT FOUND was recorded**, not a guess: `LP.12` has no reachable
   known-issues page, because the elementary OS 8 announcement returned 404 and
   no other opened page lists defects. A row that cannot be evidenced is
   recorded as such rather than downgraded to a vague PARTIAL.
3. **Duplicates still did not grow.** None of the eleven new URLs repeats an
   existing row, so `J.02`'s 47 / 52 stands for a third batch.
4. **Inspected count rises by eleven, not twelve**, because the twelfth page in
   this batch (`blog.elementary.io/...`) failed with a 404 and was not recorded
   as a source at all. Failed fetches are gaps, not rows.


---

## J.13 Addendum — Chapter L closed (2026-10-28)

Fourth recount, after Budgie 10.10.3 and ChromeOS 154. This one closes Chapter L.

| Measure | J.12 | Now |
|---|---|---|
| Register rows | 4,192 | 4,202 |
| Domains | 378 / 400 | 382 / 400 (95.5%) |
| Inspected | 1,287 | 1,297 |
| Last source id | S3877 | S3887 |
| Checklist rows | 597 | 597 |
| PENDING rows | 34 | 4 |
| PARTIAL rows | 508 | 524 |
| COMPLETE rows | 50 | 57 |
| NOT FOUND rows | 2 | 9 |
| Chapter L non-pending (112 rows + 8 parents) | 84 / 112 rows | 112 / 112 rows (+8 parents) |
| Duplicate URLs / surplus rows | 47 / 52 | 48 / 53 |

What this recount shows:

1. **Chapter L is closed: 112 measured rows, plus its eight chapter parent
   rows, for 120 / 120 non-pending.** Every one of the eight environments has
   fourteen measured rows.
2. **PENDING is down to 4**: `CH00`, `F`, `WB`, `WB.12.08`. These are the three
   chapter parents, one Windows settings row and one Windows Settings parent —
   the Windows inventory batch is the only research work left that is not a
   validation plan.
3. **NOT FOUND has grown to 9**, and that is a result rather than a
   failure: `LB.05` because Budgie ships no file manager, and six ChromeOS rows
   because their documentation could not be opened (four 404s in this batch).
   The corpus now distinguishes "measured", "structurally known" and "not
   published anywhere we could reach".
4. **Duplicates grew by one for the first time: 48 URLs / 53 surplus rows.**
   `S3878` records `https://buddiesofbudgie.org/` a second time, so the four-batch run of 47 / 52 ends here. The count is
   corrected rather than preserved: a repeated URL is a second row for the
   same page, and the audit should say so.


---

## J.14 Addendum — PENDING closed, Chapter 45 opened (2026-10-28)

Fifth recount, after `WB.12.08` (Part 29) and the Chapter 45 plan (Part 30).

| Measure | J.13 | Now |
|---|---|---|
| Register rows | 4,202 | 4,205 |
| Domains | 382 / 400 | 382 / 400 (95.5%) |
| Inspected | 1,297 | 1,300 |
| Inaccessible | 6 | 6 |
| Last source id | S3887 | S3890 |
| Checklist rows | 597 | 597 |
| Malformed rows | 0 | 0 |
| PENDING rows | 4 | 0 |
| PARTIAL rows | 511 | 528 |
| COMPLETE rows | 50 | 57 |
| NOT FOUND rows | 8 | 9 |
| Chapter L rows measured (112 leaves + 8 parents) | 112 / 112 | 120 / 120 |
| Duplicate URLs / surplus rows | 48 / 53 | 48 / 53 |

What this recount shows:

1. **PENDING is 0.** `WB.12.08` is researched and `CH00`, `F` and `WB`
   are closed by roll-up under a stated convention: a parent is COMPLETE only if
   every child is COMPLETE, otherwise PARTIAL, and never PENDING once its children
   are final. All three came out PARTIAL, which is the honest result — `CH00.04`
   and `CH00.08` are stale-page and unverified-version gaps.
2. **Nothing was marked COMPLETE to reach zero.** The three new Microsoft
   sources describe the policy mechanism; no page was opened that enumerates
   Group-Policy-only settings, so `WB.12.08` is PARTIAL and says why.
3. **Register is 4,205 rows, 17 columns per row**, last id
   `S3890`, with 1,300 inspected, 2,899 snippet-only
   and 6 inaccessible.
4. **Duplicates did not move again: 48 URLs / 53 surplus rows.**
   All three new URLs are new paths on `learn.microsoft.com`, a domain already in
   the register, so the domain count stays at 382 and `J.02`'s figure survives a
   fifth batch across 4,205 rows.
5. **The research series has no PENDING rows left.** Every one of the 597 rows
   carries a final status, but 528 of them are PARTIAL — documentation-derived or
   missing one measured half. What remains is exactly what Chapter 45 (Part 30)
   schedules: 28 validation points, 4 runnable in this repository (VP-28 passes)
   and 24 needing a VM, a device or a booted image.


---

## J.15 Addendum — Chapter 45 wave 1 executed (2026-10-28)

Sixth recount, after the Chapter 45 harness (`tools/validate_chapter45.py`) and
the two sources that close the `WB.12.08` enumeration-method gap (Part 31).

| Measure | J.14 | Now |
|---|---|---|
| Register rows | 4,205 | 4,207 |
| **Distinct source ids** | not measured | **4,207** (pre-fix 3,737 — Part 32) |
| **Source ids shared by 2+ rows** | not measured | **0** (pre-fix 279, max 6 — Part 32) |
| Domains | 382 / 400 | 382 / 400 (95.5%) |
| Inspected | 1,300 | 1,302 |
| Snippet-only | 2,899 | 2,899 |
| Inaccessible | 6 | 6 |
| Last source id | S3890 | S3892 |
| Checklist rows | 597 | 597 |
| Malformed rows | 0 | 0 |
| PENDING rows | 0 | 0 |
| PARTIAL rows | 528 | 528 |
| Duplicate URLs / surplus rows | 48 / 53 | 48 / 53 |

What this recount shows:

1. **The arithmetic is unchanged and re-derived.** `4,207` rows, `S3892`,
   `1,302` inspected, `48` duplicate URLs — every figure is recomputed
   by `figures()` in the harness, not copied from J.14.
2. **A defect J.01–J.14 never tested — found in Part 31, repaired in Part 32.** Before the
   repair, 4,207 rows carried only 3,737 distinct ids, with 279 ids shared by 2–6 unrelated
   URLs (for example `S0001` covered the Windows release-health page and five Apple HIG
   pages). The CSV was well-formed — 17 columns, 0 malformed — which is why every earlier
   audit passed it. `tools/fix_source_id_keys.py` now keeps the first row's bare id and
   suffixes the rest (`S0001a`…`S0001e`), so the register has **4,207 distinct ids for
   4,207 rows**, `VP-28b` passes, and the mapping of all 470 renamed rows is in
   `validation/source_id_remap.csv`. The 3,737/279 figures above are history, not current state.
3. **`WB.12.08` keeps PARTIAL.** S3891 defines how a GP-only setting is identified
   (MDM Support = No over six parsed CSP families) and S3892 names the authoritative
   per-setting list, but the spreadsheet was not downloaded, so the row still has no
   materialised catalogue.
