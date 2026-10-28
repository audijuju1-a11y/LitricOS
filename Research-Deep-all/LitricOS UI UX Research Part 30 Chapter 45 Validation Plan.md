# LitricOS UI/UX Research — Part 30
## Chapter 45: Validation plan (full)

**Part:** 30
**Chapter:** 45 — deliverable 10 ("Validation plan (full)")
**Predecessor:** Chapter 6, `Part1.md` — "Validation Plan (skeleton; full version in Chapter 45)"
**Date:** 2026-10-28
**New URLs opened for this part:** none. Chapter 45 is a plan over artifacts that already exist; every figure it cites is re-derived from the repository rather than from a web page.

---

## PHASE A — state check before writing

Part 28's claims were re-derived with `tools/record_chapter_l_batch4.py`'s own `figures()` and independent shell counts. Every published number matched: checklist **597 rows / 0 malformed**, `LB` **14/14**, `LR` **14/14**, `LO`/`LP` **14/14**, **PENDING 4**, register **4,202 rows** ending at **S3887**, inspected **1,297**, snippets **2,899**, inaccessible **6**, domains **382**, duplicate URLs **48 / surplus 53**, Part 28 at **14 + 14** data rows with S3878–S3887 each present exactly once, and **0 CRLF** in the Part 28 file.

One discrepancy was found, and it was in Part 1 rather than in Part 28: Part 1's Appendix A tables still printed `PENDING` for every Chapter L row (`LB.*`, `LC.*`, `LO.*`, `LP.*`, `LR.*`, `LX.*`, `LG.*`, `LK.*`) even though the index line above them said the chapter was complete, and index row 44 still carried Part 24's figures (4,170 rows / 372 domains). That is bookkeeping drift of **516 Appendix A rows plus four index rows** (chapters 6, 11, 44 and 45), not a research error. This turn's script syncs the Appendix A statuses from the checklist and rewrites rows 6, 11, 44 and 45; the numbers are quoted in §7.

---

## 1. Scope — and why this scope had to be defined

The master index allocates Chapter 45 as a title only: "Validation plan (full) | deliverable 10". The repository defines no requirements for it. What it does supply is:

* **Chapter 6** — a seven-row skeleton (`VP-01`–`VP-07`) with columns *what is validated / method / environment / pass criterion*, and the note that the "full version" is Chapter 45;
* **Part 4** — `VP-08`–`VP-12` (Fluent shadows, icon `cmap`, Mica sampling, high contrast, `perf2026` loading) with the corpus's first evidence grades (`ESTIMATE (fitted)`, `MEASURED (live)`, `MEASURED (source)`);
* **Part 5** — `VP-13`–`VP-17` (macOS sidebar order, Siri language matrix, settings-URL register, HDR × scaled resolutions, defaults audit);
* **Part 6** — rows deferred to "`VP-18`–`VP-22`" (macOS geometry, shadow, timing, Dock geometry) **without ever defining them**;
* **Chapter 5** — `GAP-001`…`GAP-008`, including `GAP-005` (rollout ambiguity requires a per-feature flag) and `GAP-008` (no live OS in this environment).

So the scope below is **defined here, from those conventions**, because the plan specified a title and nothing else. The definition is deliberately narrow: it turns every validation point the corpus has already referenced into a row with a criterion and an evidence format, it states which rows can be executed from this repository today, and it marks every row that needs hardware as NOT RUN rather than pretending it passed. No new research is introduced; Chapter 45 validates what Parts 2–29 already recorded.

**Not in scope:** new platform research, re-measurement of any Chapter L row, changes to the Rust shell, and any claim about how a real device behaves.

---

## 2. Principles

| # | Principle | Consequence in this plan |
|---|---|---|
| P1 | A validation point either has a mechanical criterion or it is not a validation point | Every row states a pass condition a second person could apply without asking the author |
| P2 | "Could not run" is a result | `NOT RUN` is a first-class status with a reason field; it is never collapsed into `PASS` |
| P3 | Docs are not observations | Any row whose evidence is a documentation page says so; measured rows carry the environment and build |
| P4 | Evidence is a file, not a sentence | Each run emits one JSON object per check (§5), with the artifact's SHA-256 |
| P5 | The plan must be cheaper to run than to argue about | Tier `T0` rows are one command in this repository; they gate the rest |

---

## 3. Tiers

| Tier | Meaning | Runs where | Count |
|---|---|---|---|
| **T0** | Repo-executable: `cargo test`, `cargo fmt --check`, `ci/*.py`, CSV integrity, script re-derivation | This workspace | 4 |
| **T1** | Replayable without a desktop: fetch a URL, launch a settings URI, read a font table, enumerate a register | Any Linux host with network | 9 |
| **T2** | Needs a live OS session or specific hardware | VM or device per §6 | 15 |

Tier is a property of the *method*, not of the row's importance: `VP-09` (icon `cmap`) is T1 because the font file can be fetched, while `VP-13` (macOS sidebar) is T2 because `Sidebar.plist` exists only on a running Mac.

---

## 4. The validation matrix

Status column: **this turn's** state, not a promise. `PASS` = executed with evidence. `PARTIAL` = part of the criterion executed. `NOT RUN` = no execution attempted, with the reason in the next column.

### 4.1 Settings inventories and surfaces

| VP | Validates | Method | Environment | Pass criterion | Tier | Status | Why |
|---|---|---|---|---|---|---|---|
| VP-01 | Settings inventories (WB, MD, the eight Chapter L DEs) | Walk every sidebar/page, screenshot, diff labels against the catalogue CSV | Clean VM/device at the exact build (26300.9711, 26A434, GNOME 51.1, Plasma 6.8.1, Xfce 4.20, Cinnamon 6.6, COSMIC 1.11, Pantheon 8.1.1, Budgie 10.10.3, ChromeOS 16805.33.0) | 100% label match, or a documented delta per label | T2 | NOT RUN | No Windows, macOS or Linux desktop session in this environment |
| VP-02 | Settings URI resolution (`ms-settings:`, `x-apple.systempreferences:`, `gnome-control-center`, `systemsettings`, `cinnamon-settings`, `xfce4-settings-manager`, `cosmic-settings`, `io.elementary.switchboard`) | Script-launch each URI; record the page opened or the failure | Same as VP-01, one host per OS family | Each URI resolves to the catalogue page or is recorded absent | T1 | NOT RUN | Requires a session bus and the DE installed |
| VP-03 | Keyboard shortcuts | Scripted key injection plus observation | Same as VP-01, reduced-motion on and off | Behaviour matches the shortcut table per row | T2 | NOT RUN | No input device or compositor here |
| VP-07 | Rollout / SKU flags (`GAP-005`) | Compare Home/Pro/Enterprise, managed/unmanaged, region; record enablement state per feature | Multiple VMs at the same build | Every catalogue row carries a confirmed flag | T2 | NOT RUN | Requires licensed SKUs and an MDM tenant |

### 4.2 Visual and token measurements

| VP | Validates | Method | Environment | Pass criterion | Tier | Status | Why |
|---|---|---|---|---|---|---|---|
| VP-04 | Motion timing | 120/240 fps capture; frame-count transitions | Hardware, reduced motion on/off | `ESTIMATE ±1 frame`, labelled | T2 | NOT RUN | Needs capture hardware |
| VP-08 | Fluent shadow profile per elevation (16/32/128) | Capture over uniform background; fit blur, offset, opacity | Windows 26H2 at 100% and 150% scale | Fitted constants within ±1 px / ±0.02 alpha of the documented recipe | T2 | NOT RUN | Closed WinAppSDK/DWM code; needs pixels |
| VP-09 | Segoe Fluent Icons `cmap` | Read the font's `cmap` on 26H2; compare with 1,574 / 1,533 | Windows 26H2 | Enumerated count reconciles the two published figures | T1 | NOT RUN | Font file not present in this repository |
| VP-10 | Mica / Mica Alt sampling | Sample window background against known wallpapers | Windows 26H2, light/dark, 3 wallpapers | Sampled tint matches the documented material within ΔE 2 | T2 | NOT RUN | Needs a compositor |
| VP-11 | High-contrast swap | Switch each of the 4 contrast themes; scan a WinUI 3 sample app | Windows 26H2 | No hard-coded `#FF0000` leak; every token resolves | T2 | NOT RUN | Needs Windows |
| VP-12 | `perf2026` resource loading | Inspect the WinUI 3 runtime load path or a debug build | Windows 26H2 + debugger | Load path matches the source-described behaviour | T1 | NOT RUN | No Windows image |
| VP-23 | Chapter L DE rows re-measured live (112 rows) | For each row, perform the described action and screenshot the result | One live session per DE | Row text matches observed behaviour, or is downgraded | T2 | NOT RUN | The entire Chapter L column is documentation-derived; see §6 |
| VP-24 | Token contrast reproduction (Chapter G) | Compute contrast from the shipped token tables, then verify on device | Linux/Windows/macOS | Computed ratio equals device ratio; AA/AAA verdict unchanged | T1 | NOT RUN | Token tables are in the corpus; device half missing |

### 4.3 macOS geometry (the `VP-18`–`VP-22` gap)

Part 6 deferred measurements to five ids and never defined them. They are defined here so the forward references resolve.

| VP | Validates | Method | Environment | Pass criterion | Tier | Status | Why |
|---|---|---|---|---|---|---|---|
| VP-18 | Window geometry: corner radius, titlebar height, traffic-light spacing and order | Screenshot a stock window at 1× and 2×; measure in pixels | macOS 27.0.1 (26A434), 1440×900 and 3024×1964 | Radius/height/spacing within ±1 px of the fitted value; order matches | T2 | NOT RUN | No Mac |
| VP-19 | Window and menu shadow profile | Capture over a uniform background; fit blur/offset/alpha | Same | Fitted profile stable across wallpapers | T2 | NOT RUN | No Mac |
| VP-20 | Dock geometry and magnification curve | Measure tile size, padding, corner radius, magnified tile at 5 cursor positions | Same, Dock bottom/left/right, autohide on/off | Curve monotonic; tile sizes within ±1 px of the fit | T2 | NOT RUN | No Mac |
| VP-21 | Animation timing (minimise, Mission Control, app launch) | 240 fps capture; frame count | Same, reduced motion on/off | `ESTIMATE ±1 frame`; reduced-motion path is instant | T2 | NOT RUN | No capture hardware |
| VP-22 | Menu bar and menu geometry | Measure bar height, item padding, menu width for a fixed item set | Same, with and without notch | Values within ±1 px; notch behaviour matches | T2 | NOT RUN | No Mac |

### 4.4 Accessibility, workflows, integration

| VP | Validates | Method | Environment | Pass criterion | Tier | Status | Why |
|---|---|---|---|---|---|---|---|
| VP-05 | Accessibility semantics | Accessibility Insights (Windows), Accessibility Inspector (macOS), Accerciser (AT-SPI) | Per-OS VM | Role/name/state as specified for every interactive element | T2 | NOT RUN | Needs an a11y tree; the LitricOS shell exposes none (`G` rows) |
| VP-27 | LitricOS a11y tree per surface | Traverse the shell with Accerciser over AT-SPI; record role/name/state per node | LitricOS image on a Linux host | Every control reachable and named; no unnamed container with actions | T2 | NOT RUN | The image has not been built here |
| VP-06 | Workflows (Chapter H, 20 workflows) | Scripted success / failure / cancel / retry / rollback runs | Per-OS VM | Observed states match the state diagrams | T2 | NOT RUN | No desktop |
| VP-25 | Integration surface (portals, D-Bus, revert timer) | Start the image; call each portal and D-Bus name; kill the settings daemon mid-change and confirm the revert timer fires | LitricOS image + one host DE | Every documented name answers; revert restores the previous value within the window | T2 | NOT RUN | No image boot here |

### 4.5 Persistence and corpus integrity

| VP | Validates | Method | Environment | Pass criterion | Tier | Status | Why |
|---|---|---|---|---|---|---|---|
| VP-26 | UI scale persists across restart | Write a scale, restart the shell, read it back; then corrupt the file and confirm the fallback | LitricOS image | Value survives restart; corrupt file yields the default, not a crash | T2 (repo half T0) | PARTIAL | `prefstore.rs` encode/parse/round-trip covered by unit tests; no restart test on a booted image |
| VP-28 | Corpus reproducibility | Re-run the batch scripts; compare checklist counts, register ids, duplicate URLs with the published audit | This workspace | Every published figure re-derived with zero drift | T0 | PASS | Executed this turn (§7) |

### 4.6 Tally

| Tier | Rows | Executable now |
|---|---|---|
| T0 | 4 (`VP-28` plus the three repo validators counted inside it) | 4 |
| T1 | 9 | 0 without a Windows host or network fetch harness |
| T2 | 15 | 0 |
| **Total** | **28** | **4** |

---

## 5. Evidence format

One JSON object per check, appended to `validation/receipts.jsonl` (not created by this plan; the path is the contract):

```json
{"vp":"VP-28","run":"2026-10-28T09:00:00+07:00","tier":"T0","env":"workspace:linux-x86_64","build":"repo@HEAD",
 "result":"PASS","criterion":"all published figures re-derived","observed":{"checklist_rows":597,"malformed":0},
 "artifacts":[{"path":"Research-Deep-all/LitricOS Coverage Checklist.csv","sha256":"…"}]}
```

Rules:

1. `result` ∈ `PASS | FAIL | PARTIAL | NOT RUN | BLOCKED`; `NOT RUN` requires `reason`.
2. `artifacts` carries a SHA-256 for every file the check read or wrote, so a claim can be re-checked without re-running.
3. The file is append-only; a re-run adds a row and never edits one.
4. A chapter's gate (§8) is computed from the receipts, not from prose.

---

## 6. Environment matrix and what this workspace cannot do

| Platform | Required build | Available here | Consequence |
|---|---|---|---|
| Windows 11 26H2 | 26300.9711 | No | `VP-01`, `03`–`05`, `07`, `08`, `10`, `11` unexecuted |
| Windows (font/image only) | 26H2 `Segoe Fluent Icons.ttf`, WinUI debug symbols | No | `VP-09`, `VP-12` unexecuted |
| macOS 27.0.1 | 26A434, Apple silicon | No | `VP-13`–`VP-22` unexecuted |
| GNOME 51.1 | Fedora 44 or equivalent | No | Chapter L rows and `VP-02`/`VP-06` unexecuted |
| Plasma 6.8.1, Xfce 4.20, Cinnamon 6.6, COSMIC 1.11, Pantheon 8.1.1, Budgie 10.10.3 | Per Part 25–28 | No | `VP-23` unexecuted |
| ChromeOS 16805.33.0 | Managed device | No | `LR` rows stay documentation-only |
| LitricOS image | mkosi build | Not built in this workspace | `VP-25`, `VP-26` (restart half), `VP-27` unexecuted |
| Linux + this repo | Any | **Yes** | `VP-28` and the three `ci/` validators |

This is `GAP-008` restated as a table: the corpus measured **0** of its 112 Chapter L rows on hardware, and the validation plan is the mechanism that records that honestly instead of implying otherwise.

---

## 7. What was executed this turn

### 7.1 Repo validators (T0)

| Command | Result |
|---|---|
| `python3 ci/check-python-syntax.py` | see §7.4 |
| `python3 ci/lint-line-endings.py` | see §7.4 |
| `python3 ci/lint-doc-refs.py` | see §7.4 |

### 7.2 Corpus figures after this turn (VP-28 evidence)

| Measure | Value |
|---|---|
| Checklist rows / malformed | 597 / 0 (5 columns) |
| Status distribution | PARTIAL 528, COMPLETE 57, NOT FOUND 9, NOT APPLICABLE 3 |
| PENDING | **0** |
| Register rows / last id | 4,205 / S3890 (17 columns) |
| Inspected / snippet / inaccessible | 1,300 / 2,899 / 6 |
| Domains | 382 (95.5% of the 400 target) |
| Duplicate URLs / surplus | 48 / 53 |
| Chapter L rows measured | 112 / 112 |

### 7.3 Rust suite (quoted, not re-run)

`597 passed` and `cargo fmt --check` clean, from the last run in which a Rust file changed. The worktree is **not** clean: `git status --porcelain -- 'src/**/*.rs'` lists 11 files from the earlier UI-scale turns (`prefstore.rs` 03:14, `shell.rs` 03:15). No Rust file was modified in this turn; `find src -name '*.rs' -newer "Part 29 … .md"` returns **0** (quoted in §7.4). The figure is therefore quoted, not re-measured.

### 7.4 Exact output

```
$ python3 ci/check-python-syntax.py
OK: 57 python file(s) parse (no bytecode written)          exit=0

$ python3 ci/lint-line-endings.py
OK: 348 file(s) checked, all LF                            exit=0

$ python3 ci/lint-doc-refs.py
SKIP: docs/bugs/BUG-0003-lsm-stack-was-two-not-seven.md:33: scripts/kconfig/merge_config.sh -- kernel source tree, not this repository
OK: 746 citation(s) across 150 document(s) resolve        exit=0

$ python3 tools/record_chapter_l_batch5.py
checklist 597 / malformed 0 / PENDING 0 / register 4,205 / last S3890
inspected 1,300 / snippet 2,899 / inaccessible 6 / domains 382 / dup 48 / surplus 53
part1: 4 index rows rewritten, 516 Appendix A rows synced

$ git status --porcelain -- 'src/**/*.rs' | wc -l
11            # pre-existing UI-scale work, all mtimes 03:14-03:17
$ find src -name '*.rs' -newer "Research-Deep-all/…Part 29….md" | wc -l
0             # nothing touched after this turn began
```

---

## 8. Gate and exit criteria

A chapter is **validated** when every `VP` that names it is `PASS`, or is `NOT RUN` with a reason accepted in Chapter 5. Current state, computed from §4:

| Chapter | Gate | State |
|---|---|---|
| 0 (baseline) | `VP-01`, `VP-07` | OPEN — both T2 |
| 2/3 (Windows shell, Settings, Fluent) | `VP-01`–`VP-12` | OPEN — 1 of 12 partially executed |
| 4 (macOS) | `VP-13`–`VP-22` | OPEN — 0 of 10 |
| 5 (Chapter L) | `VP-23` | OPEN — 0 of 112 rows re-measured |
| 7 (design tokens) | `VP-24` | OPEN |
| 8 (a11y) | `VP-05`, `VP-27` | OPEN |
| 9 (workflows) | `VP-06` | OPEN |
| 10 (integration) | `VP-25` | OPEN |
| 11 (persistence) | `VP-26` | PARTIAL — repo half green |
| 12 (corpus) | `VP-28` | **CLOSED** |

Entry criterion for any T2 run: a receipt for `VP-28` from the same commit, so the catalogue being validated is the catalogue that was published.
Exit criterion for the plan itself: 28 receipts exist, each `PASS` or an accepted `NOT RUN`.

---

## 9. Execution order

1. **Now (T0):** `VP-28` plus the three validators — done; re-run on every commit that touches `Research-Deep-all/`.
2. **Next, no hardware:** `VP-09` (fetch the font, enumerate `cmap`), `VP-24` (compute contrast from the token tables), `VP-12` (read the WinUI load path from a debug package) — all reduce rows that are currently `PARTIAL` for documentation-only reasons.
3. **First VM wave:** Windows 26H2 and macOS 26A434, since `VP-01`/`VP-02`/`VP-17` unblock the largest `PARTIAL` block.
4. **Chapter L wave:** one session per DE, strictly in checklist order, `VP-23` only.
5. **Image wave:** build the mkosi image, then `VP-25`, `VP-26`, `VP-27`.

---

## 10. Limitations

* **Not run on real hardware.** Every T2 row is `NOT RUN`; the plan contains no observed pixel, timing or accessibility result. The only executed rows are `VP-28` and the repo validators.
* **Docs vs. observed.** Chapter L's 112 rows, Chapter B's Settings rows and Chapters 4–6's macOS rows are documentation-derived. `VP-23` and `VP-01` exist precisely to convert them; until then the corpus measures documents, not desktops.
* **The scope is this plan's invention.** Chapter 45 had a title and a deliverable number only. `VP-18`–`VP-22` were referenced but undefined; they are defined here. If the project owner intended a different scope, this file is the thing to correct, and §1 lists exactly which earlier documents constrain it.
* **No receipts exist yet.** `validation/receipts.jsonl` is specified, not created; §7's evidence is the command output quoted in this file, which is weaker than a receipt with a hash.
* **The Rust figure is quoted, not re-measured.** No Rust file changed in Parts 29–30, so re-running a 597-test suite would prove nothing new.
* **No new sources.** Part 30 opens no URL. Its inputs are `Part1.md` (Chapters 5–6 and the master index), Parts 4, 5 and 6 (the `VP-08`–`VP-22` references) and the repository's own scripts.
