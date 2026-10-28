# LitricOS UI/UX Research — Part 31
## Chapter 45, wave 1: the validation points that can be measured here

**Part:** 31
**Chapter:** 45 — deliverable 10, execution of the Part 30 plan
**Predecessor:** Part 30 (`Chapter 45 Validation Plan`), which defined 28 validation points and executed one
**Date:** 2026-10-28
**New URLs opened for this part:** 2 (`S3891`, `S3892`) — both Microsoft, both for the `WB.12.08` method gap
**Machine-written evidence:** `validation/receipts.jsonl`, 8 rows, one JSON object per check, each artifact hashed

---

## 1. What changed since Part 30

Part 30 ended with a plan and an honest admission: 28 validation points, **4 executable in this
repository, 1 executed**. Its §9 named the next step — "`VP-09` (fetch the font, enumerate `cmap`),
`VP-24` (compute contrast from the token tables), `VP-12` (read the WinUI load path)" — none of which
had been done.

This part does the part of that work that needs no desktop: it measures the **repository half** of
every validation point whose criterion has one, and it does so with a script that can be re-run on
any commit. The result is not "the plan is now validated". It is:

| | Part 30 | Part 31 |
|---|---|---|
| Checks with a receipt | 1 (`VP-28`) | **8** |
| `PASS` | 1 | **1** (`VP-28`, figures re-derived) |
| `PARTIAL` | 1 | **6** (`VP-01`, `VP-02`, `VP-09`, `VP-11`, `VP-24`, `VP-26`) |
| `FAIL` | 0 | **1** then **0** (`VP-28b`, a register defect — §5; repaired in Part 32) |
| Hardware-dependent rows | 26 | **21**, still `NOT RUN` |
| Receipts on disk | 0 | **8**, in `validation/receipts.jsonl` |

The new `FAIL` is the most useful result in this part. It is a defect that every earlier audit
passed, because every earlier audit checked the CSV's shape and never its key.

---

## 2. How each number was produced

The harness is [`tools/validate_chapter45.py`](../tools/validate_chapter45.py). It never imports the
Rust code — it reads the source and recomputes, which is deliberate: a check that calls the same
function it is checking proves only that the function is deterministic.

| Check | What is read | What is computed |
|---|---|---|
| `VP-28` | `LitricOS Coverage Checklist.csv`, `LitricOS Source Register.csv`, Part 28, Part 1, Part 24 | row counts, malformed rows, status distribution, PENDING, columns per row, duplicate URLs, Part 28's `LB`/`LR` rows and `S3878`–`S3887` presence |
| `VP-28b` | the register's `source_id` column | distinct ids, shared ids, max rows per id, and how many prose citations hit a shared id |
| `VP-24` | `src/litric-design/src/lib.rs` (`Theme::of`, `with_accent`, `Glass`) | 60 WCAG contrast ratios: 36 palette pairs, 4 glass-over-wallpaper pairs, 20 accent pairs |
| `VP-11` | the same palette | 11 semantic tokens × 4 themes, 12 high-contrast text pairs at AAA, and the `with_accent` early return |
| `VP-09` | `src/litric-shell/src/icons.rs` | `Icon` variants vs `name()` arms vs `draw()` arms, including `|` arms |
| `VP-01` | `settings.rs` (`Page`), `registry.rs` (`ENTRIES`), `prefstore.rs` (`encode`/`parse`) | pages, pages with stored rows, reading-only pages, store/registry parity in both directions |
| `VP-02` | `settings.rs`, `registry.rs` | page count, unique slugs, linkable rows, `link_for` round-trip |
| `VP-26` | `prefstore.rs` | encode/parse key parity, `ui_scale` presence, temp+`sync_all`+`fsync` rename, the two tests |

The WCAG arithmetic is implemented a second time in Python (gamma decode, 0.2126/0.7152/0.0722,
`(L1+0.05)/(L2+0.05)`) rather than trusted from the crate. Both implementations agree to two
decimals on every pair, which is the only reason the ratios below can be called measurements rather
than restatements.

---

## 3. Results

### 3.1 `VP-24` — token contrast: 60 measurements, 0 failures

| Group | Pairs | Min ratio | Bar | Verdict |
|---|---|---|---|---|
| Palette (4 themes × 9 pairs) | 36 | **4.97** | AA 4.5 (standard), AAA 7.0 (high contrast) | pass |
| Glass over white and black wallpaper | 4 | **4.81** | AA 4.5 | pass |
| Accent choices (2 standard themes × 5 accents × 2 pairs) | 20 | **5.35** | AA 4.5 | pass |
| **Total** | **60** | **4.81** | | **0 failures** |

The five closest pairs, all in the light theme, are the ones a designer would be tempted to
"brighten":

| Theme | Pair | Ratio |
|---|---|---|
| Light | `success` on `bg_base` | 4.97 |
| Light | `danger` on `bg_base` | 5.26 |
| Light | `accent` on `bg_base` | 5.35 |
| Light | `warning` on `bg_base` | 5.53 |
| Light | `on_accent` on `accent` | 5.74 |

Two things follow. First, the light theme's status colours are the real constraint in the palette,
not its body text — `text_secondary` on white is 7.0, but a green that reads as green at 4.97 has
almost no room left. Second, the glass floor is doing real work: with the dark tint at 82% over pure
white the secondary text lands at 4.81, and at the old 72% it would be 3.34 (the crate's own comment
records that measurement). The harness reproduces both directions, so lowering `tint_alpha_pct`
would now fail a check rather than merely look worse.

**Status stays `PARTIAL`.** The plan's criterion is "computed ratio equals device ratio". There is
no device here; the computed half is now evidence, the equality is not.

### 3.2 `VP-11` — high-contrast resolution: 12/12 at AAA, minimum 10.0

| Measure | Value |
|---|---|
| Semantic tokens | 11 |
| High-contrast themes | 2 |
| Text pairs at AAA | 12 (100%) |
| Minimum ratio | **10.0** (`HighContrastLight` `accent` on white) |
| Accent ignored in high contrast | yes — `with_accent` returns the theme unchanged |
| Border equals background | no — outlines survive in both high-contrast themes |

Again `PARTIAL`: the token swap resolves, the **on-device** scan for a hard-coded `#FF0000` leak
(`VP-11`'s actual failure mode) cannot run without Windows.

### 3.3 `VP-09` — icons: 32 variants, 32 names, 32 draw arms

| Measure | Value |
|---|---|
| `Icon` variants | **32** |
| With a `name()` arm | 32 |
| With a `draw()` arm | 32 (the `Audio | AudioMuted` arm counts once per icon) |
| Missing | none |

This is the **repository** side of `VP-09`. The plan's row is about Segoe Fluent Icons' `cmap` —
whether the documented 1,574 or 1,533 codepoints is the true count — and that font is not in this
workspace. What was measured instead is the property the font check exists to protect: no icon can
ship without a glyph. The file is 648 lines and every variant has both a stable name and a
normalised-box drawing, so the icon set is complete in the sense the shell controls.

### 3.4 `VP-01` / `VP-02` — settings inventory and deep links

| Measure | Value |
|---|---|
| Settings pages (`Page::ALL`) | **13** |
| Pages with stored rows | **5** — Appearance, Display, Region, Accessibility, Privacy |
| Pages that are read-only | **8** — Input, Network, Bluetooth, Power, Environments, Sound, Accounts, Diagnostics |
| Registry rows (`ENTRIES`) | **8** |
| Keys the store writes | **10** (8 searchable + `version`, `onboarded`) |
| Store ↔ registry parity | **both directions**, no orphan |
| Unique page slugs | **13 / 13** |
| Linkable rows | **8** |
| `link_for` round-trip | present |

The 8 reading-only pages are the module's documented decision, not a coverage hole: a row for a
kernel measurement would be a control that cannot do anything. `VP-01`'s criterion — 100% label
match against a screenshot diff — still needs a desktop, so `PARTIAL`.

### 3.5 `VP-26` — UI scale persistence, repository half

| Measure | Value |
|---|---|
| Encoded keys / parsed keys | 10 / 10, identical sets |
| `ui_scale` in both | yes |
| Write path | temp file → `sync_all` → `fs::rename` → directory `sync_all` |
| Tests present | `the_ui_scale_round_trips_and_a_size_this_build_does_not_draw_is_ignored`, `unknown_keys_and_values_keep_the_defaults` |

`PARTIAL` for one reason only: the restart is not run, because no image was booted. The file:line
evidence is unchanged from Part 30 §7 and is re-quoted in §7 below.

---

## 4. `VP-28` — every published figure re-derived

| Measure | Part 30 | Now | Drift |
|---|---|---|---|
| Checklist rows | 597 | **597** | 0 |
| Malformed rows | 0 | **0** | 0 |
| PENDING | 0 | **0** | 0 |
| Status distribution | 57 / 528 / 9 / 3 | **57 / 528 / 9 / 3** | 0 |
| Register rows | 4,205 | **4,207** | +2 (this part's sources) |
| Columns per row | 17 | **17** | 0 |
| Last source id | S3890 | **S3892** | +2 |
| Inspected | 1,300 | **1,302** | +2 |
| Snippet-only | 2,899 | **2,899** | 0 |
| Inaccessible | 6 | **6** | 0 |
| Domains | 382 | **382** | 0 (both URLs are new paths on existing hosts) |
| Duplicate URLs / surplus | 48 / 53 | **48 / 53** | 0 |
| Chapter L rows | 120 (112 leaf + 8 parent) | **120** | 0 |
| Part 28 `LB` / `LR` | 14 / 14 | **14 / 14** | 0 |
| `S3878`–`S3887` present once | yes | **yes** | 0 |
| Part 28 CRLF | 0 | **0** | 0 |

`VP-28` passes: the arithmetic in Parts 28–30 is reproducible with zero drift.

---

## 5. The finding: `source_id` is not a key

> **Repaired in [Part 32](LitricOS%20UI%20UX%20Research%20Part%2032%20Register%20Key%20Repair.md).**
> The first row of each group keeps the bare id and later rows take `S####a`, `S####b`, …;
> 470 rows were renamed and the register now has **4,207 distinct ids for 4,207 rows**.
> `VP-28b` passes. Everything in this section is the *pre-fix* measurement, kept as history.

`VP-28b` is a sub-check added by this execution, not a row in Part 30's matrix. It asks one
question the earlier audits never asked: can `source_id` identify a row?

| Measure | Value |
|---|---|
| Register rows | **4,207** |
| Distinct `source_id` values | **3,737** |
| Ids shared by 2+ rows | **279** |
| Worst case | **6 rows** for one id (`S0001`, `S0007`, `S0023`) |
| Rows lost to sharing | **470** |
| Citations in prose that hit a shared id | **9**, in 2 documents (Parts 21 and 23) |
| Result | **FAIL** → **PASS** after Part 32 |

`S0001` is the clearest example. It is recorded six times, for six unrelated pages:

| Row | URL |
|---|---|
| 1 | `learn.microsoft.com/.../windows11-release-information` |
| 2 | `developer.apple.com/design/human-interface-guidelines/branding` |
| 3 | `developer.apple.com/.../inclusion` |
| 4 | `developer.apple.com/.../layout-and-organization` |
| 5 | `developer.apple.com/.../spatial-layout` |
| 6 | `developer.apple.com/.../writing` |

Why this survived six audits: `J.01` checked the schema (17 columns), `J.02` checked URL and domain
duplication, `J.03`–`J.06` checked counts and statuses, and every batch script asserted
`len(source) == 17`. None of them tested uniqueness. A CSV with a repeated primary key is still a
well-formed CSV, which is exactly why the defect is invisible to a shape check.

**What it does not mean.** No published *count* is wrong: 4,207 rows exist, the last id is `S3892`,
1,302 are inspected. What is wrong is the implication that an id names one source. A reader who
follows `S0001` from a table lands on an arbitrary one of six rows.

**Why it was not repaired in this part.** The obvious fix — renumber the duplicates — changes what
existing citations point at. Nine citations across Parts 21 and 23 use shared ids (`S0115`,
`S0278`, `S0280`, `S0930`, `S0941`, `S1079`, `S1103`, `S1112`, `S0130`), and deciding which of the
two-to-six URLs each meant requires re-reading the surrounding argument, not a script. Three
options, with their costs:

| Option | Change | Risk |
|---|---|---|
| A. Suffix scheme (`S0001a`…`S0001f`) | rewrite `source_id` for 470 rows, update 9 citations after reading their context | ids stop matching `S\d{4}`; every audit script and the "last id" claim need updating |
| B. New ids for duplicates, appended | 470 new ids from `S3893`, rows move to the end | register order no longer matches id order; duplicate-URL count changes |
| C. Declare the column a batch id | documentation only, no renumbering | honest but leaves the lookup broken; the register cannot be joined on its key |

This part takes **C as an interim statement** and leaves the choice to the owner, because A and B
rewrite the corpus's identifiers — a decision with more blast radius than a validator run. The
check now fails on every run until one is chosen, which is the point.

---

## 6. `WB.12.08` — the gap is now a method, not a mystery

Part 29 left `WB.12.08` `PARTIAL` with a precise reason: the three Microsoft pages it opened explain
the policy mechanism but **no page enumerates Group-Policy-only settings**. Two more pages close the
method half of that gap:

| Source | What it establishes |
|---|---|
| `S3891` — Intune Group Policy analytics | imports a GPO as XML and reports, **per setting**, `MDM Support` = Yes/No; **No** means "there isn't a matching setting available to MDM providers, including Intune"; the tool parses Policy CSP, PassportForWork, BitLocker, Firewall, AppLocker CSP and Group Policy Preferences; anything outside those lands in `Unknown Settings`; readiness is `Ready for migration` / `Not supported` / `Deprecated` |
| `S3892` — Group Policy Settings Reference Spreadsheet (25H2) | the authoritative row-by-row list: every computer and user policy setting in the 25H2 administrative templates, `Windows11PolicySettings25H2.xlsx`, 786.5 KB, published 2025-10-02, filterable |

So "which settings exist only in Group Policy" now has a mechanical answer: take the spreadsheet's
rows, join them against the six parsed CSP families, and every row with no `MDM Support` mapping is
GP-only. That is a procedure, not a catalogue, and the spreadsheet was **not downloaded or parsed**
here — which is why the row stays `PARTIAL`. What changed is that the remaining work is now a data
extraction with a named input, not an open question.

---

## 7. `STEP 0b` re-confirmation: UI scale reaches the disk

No Rust file was modified in this part. Re-quoted, unchanged:

| Path | Evidence |
|---|---|
| Write | [`prefstore.rs:170`](../src/litric-shell/src/prefstore.rs#L170) `out.push_str("ui_scale=")` + [`prefstore.rs:171`](../src/litric-shell/src/prefstore.rs#L171) `preferences.ui_scale.token()` inside `encode()`, written by `Store::save` as temp → `sync_all` → `fs::rename` → directory `sync_all` |
| Read | [`prefstore.rs:112`](../src/litric-shell/src/prefstore.rs#L112) arm `"ui_scale"` → [`prefstore.rs:114`](../src/litric-shell/src/prefstore.rs#L114) `preferences.ui_scale = scale` |
| Apply at boot | [`shell.rs:3017`](../src/litric-shell/src/shell.rs#L3017) `let scale = self.base_scale * preferences.ui_scale.factor();` → `set_ui_scale` |
| Write on Keep | [`shell.rs:3252`](../src/litric-shell/src/shell.rs#L3252) `unwrap_or(self.settings.preferences.ui_scale)` → `save_preferences` → `Effect::PersistPreferences` |
| Test | [`prefstore.rs:458`](../src/litric-shell/src/prefstore.rs#L458) `the_ui_scale_round_trips_and_a_size_this_build_does_not_draw_is_ignored` |

**Verdict: PASS**, with the same caveat as before — the round-trip is a unit test, not a reboot.

`git status --porcelain -- 'src/**/*.rs'` lists 11 pre-existing modified files from the UI-scale
turns (mtimes 03:14–03:17); `find src -name '*.rs' -newer` the Part 29 file returns **0**. No Rust
file changed here.

The Rust suite is **quoted, not re-run**: `cargo` is not installed in this workspace
(`cargo: command not found`), so the last measured figure remains **597 passed, `cargo fmt --check`
clean** from the turn that last changed Rust.

---

## 8. Gate after wave 1

| Chapter | Validation points | Before | Now |
|---|---|---|---|
| 4 (macOS) | `VP-13`–`VP-22` | OPEN, 0/10 | OPEN, 0/10 |
| 5 (Chapter L) | `VP-23` | OPEN, 0/112 rows observed | OPEN, **0/112** — the plan measured documents, not desktops |
| 7 (tokens) | `VP-24` | OPEN | **PARTIAL** — 60 ratios computed, device half absent |
| 8 (a11y) | `VP-05`, `VP-27` | OPEN | OPEN — no a11y tree exists to traverse |
| 10 (integration) | `VP-25` | OPEN | OPEN — no image |
| 11 (persistence) | `VP-26` | PARTIAL | **PARTIAL** — same, now with 10/10 key parity evidence |
| 12 (corpus) | `VP-28` | CLOSED | **CLOSED** for figures; **FAIL** on `VP-28b` key uniqueness |
| 2/3 (Windows shell, Settings) | `VP-01`–`VP-12` | OPEN, 1/12 partial | **PARTIAL** — `VP-01`, `VP-02`, `VP-09`, `VP-11`, `VP-24` repo halves done |

The corpus gate is now two-valued on purpose: the numbers pass, the key does not. Collapsing them
would hide the defect behind a green row.

---

## 9. What still cannot run, and why

| Missing capability | Blocks |
|---|---|
| Windows 26H2 (26300.9711) | `VP-03`, `VP-04`, `VP-05`, `VP-07`, `VP-08`, `VP-10`, `VP-11` device half, `VP-12`, `VP-24` device half |
| macOS 27.0.1 (26A434) | `VP-13`–`VP-22` |
| Any Linux desktop session | `VP-02` handler half, `VP-06`, `VP-23` |
| LitricOS image (mkosi) | `VP-25`, `VP-26` restart half, `VP-27` |
| `cargo` toolchain | re-running the 597 tests (quoted here instead) |
| A managed ChromeOS device | the `LR` rows stay documentation-only |

That is 21 of 28 points. A validation plan that reported 28 green rows from this workspace would be
lying about hardware it does not have.

---

## 10. Limitations

* **Not run on real hardware.** Every result above is a property of source files and CSV rows. No
  pixel, frame time, a11y role or settings label was observed on a running system.
* **Docs vs. observed.** Chapter L's 112 rows remain documentation-derived. `VP-23` exists to
  convert them and is still `NOT RUN`; this part does not narrow that gap.
* **`PARTIAL` is not a softer `PASS`.** Six rows are partial because their criterion spans a device.
  The repo half is now measured; the other half is not "assumed".
* **The register key defect was repaired in Part 32**, not left open: 470 rows renamed by
  `tools/fix_source_id_keys.py`, `VP-28b` now passes, and the pre-fix figures in §5 are history.
* **`WB.12.08` has a method, not a catalogue.** The 25H2 spreadsheet was named, not downloaded.
* **No new platform research.** The two new sources are about how Microsoft exposes policy
  enumeration; no UI claim was added to any chapter.
* **Rust figure is quoted.** `cargo` is absent from this workspace, so 597 tests were not re-run.
* **Nothing was committed or pushed.** The worktree carries the research files and the two new
  scripts; HEAD is still `66417cd`.

---

## 11. Reproduction

```text
$ python3 tools/validate_chapter45.py
VP-28: PASS      {"checklist_rows": 597, "malformed": 0, "pending": 0, ... "last_source_id": "S3892"}
VP-28b: FAIL     {"rows": 4207, "distinct_ids": 3737, "shared_ids": 279, "max_rows_per_id": 6, ...}
VP-24: PARTIAL   {"measurements": 60, "min_palette_ratio": 4.97, "min_glass_ratio": 4.81, ...}
VP-11: PARTIAL   {"semantic_tokens": 11, "min_ratio": 10.0, ...}
VP-09: PARTIAL   {"icon_variants": 32, "named": 32, "drawn": 32}
VP-01: PARTIAL   {"settings_pages": 13, "pages_with_stored_rows": 5, "pages_reading_only": 8}
VP-02: PARTIAL   {"pages": 13, "unique_slugs": 13, "rows_linkable": 8}
VP-26: PARTIAL   {"encoded_keys": 10, "parsed_keys": 10, "atomic_write": true, ...}

8 checks: 1 PASS, 6 PARTIAL, 1 FAIL
receipts: validation/receipts.jsonl (+8 rows)
```

The console prints each check's full `observed` object on one line; the block above keeps the
leading fields and elides the rest with `...`. The complete objects are the receipts themselves,
one JSON document per line.

The harness exits non-zero because of `VP-28b`; that is intended. `--check` runs the same checks
without appending receipts.
