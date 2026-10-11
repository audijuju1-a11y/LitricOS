# LitricOS — Continuation Audit, 2026-10-09

**Scope:** memory-budget enforcement · file-operation integrity · defender/benchmark
counter forensics · repository evidence · bounded hardening
**Mode:** code-only. No build, no compile, no cargo, no mkosi, no QEMU, no GUI, no
install, no host mutation, no service started. Every verdict below is labelled with
what was actually executed.
**Result vocabulary:** PASS (executed and succeeded) · FAIL (executed and found a
defect) · SKIPPED (deliberately not run) · NOT VERIFIED (available checks cannot
establish the claim) · HISTORICAL (from a prior report, not reproduced here).

---

## สรุปผู้บริหาร (Thai executive summary)

งานรอบนี้ยึดหลัก "แก้เฉพาะที่พิสูจน์ได้" — ไม่ build ไม่รัน VM/GUI โดยทำ 4 สายงานหลัก
แล้วปิดท้ายด้วยการตรวจแบบจำกัดขอบเขต ได้ผลลัพธ์สำคัญคือ **บั๊กข้อมูลเสียหายจริง 2 ตัว**
(การแทนที่ไฟล์แบบ `Replace` ลบของเดิมก่อนสำเนาใหม่เสร็จ · ถังขยะย้ายไฟล์ก่อนเขียน
`.trashinfo` และไม่จองชื่อแบบ exclusive) ซึ่งแก้ไปแล้วพร้อมเทสต์ Rust ที่ยัง **ไม่ได้รัน**
เพราะห้ามคอมไพล์, **บั๊กที่ยังไม่แก้ 1 ตัว** (ค่า `Path=` ใน `.trashinfo` ไม่ใช่ path ตามสเปก
— ต้องแก้พร้อมกัน 4 จุดใน 2 crate) และ **บั๊กตัวเลขสถิติของ benchmark** ที่ตัด u64 เหลือ u32
จนได้ค่าที่ดูสมเหตุสมผลแต่ไม่มีตัวอย่างไหนผลิตค่านั้นได้ (แก้แล้ว + มีเทสต์ที่ล้มกับโค้ดเก่า)
นอกจากนี้ยังทำให้ memory budget มีตัวตรวจ static ที่รันได้จริงใน CI (14 pass / 0 fail / 6 skip)
แต่ย้ำชัด: **ยังไม่มีอะไรพิสูจน์ว่าระบบใช้ RAM ตามเป้าได้จริง** เพราะต้องบูตระบบจริง

---

## 1. Scope and constraints

| Constraint | State |
|---|---|
| Build / compile / cargo / rustc / mkosi / make / ninja | **never invoked** |
| VM, emulator, guest OS, desktop shell | **never launched** |
| GUI or browser-driven test | **never run** |
| Package install / host configuration / `sudo` | **never run** |
| Tests weakened, deleted or relabelled | **none** |
| Gate marked VERIFIED | **none** |

Work was done in four ordered phases (memory budget → file operations → counters →
evidence), then a bounded scan. Two permitted tool classes were used beyond reading:
the existing `ci/` Python checkers, and `rustfmt --check` as a **parser** (it does not
compile or type-check; it writes nothing with `--check`).

## 2. Repository state before the changes

* **No Git.** `git rev-parse --is-inside-work-tree` → `fatal: not a git repository`.
  There is **no version-control safety net**: no baseline to diff against, no way to
  revert, and no record of who changed what. Stated here because every claim about
  "unchanged" is weaker without it than it would be with it.
* Pre-existing modifications from the prior audit turn were left alone: `tests/smoke_boot.py`,
  `tests/gate01-verify.sh`, `tests/test_harness_integrity.py`, `ci/check-python-syntax.py`,
  `ci/lint-doc-refs.py`, `docs/GATES.md`, `docs/EVIDENCE.md`,
  `docs/DESIGN-SYSTEM-v3/00-AUDIT.md`, `docs/bugs/BUG-0001…0007`.
* Generated artefacts present: `ci/__pycache__/`, `tests/__pycache__/` (Python bytecode
  from earlier test runs). Not deleted — unknown ownership. `ci/verify.sh`'s Python
  stage runs the syntax checker in a mode that writes no bytecode.
* `tests/budget/` still exists and is still **empty**; this session did not put a runtime
  budget test there (it would need a booted image). The static checker lives in `ci/`.
* `src/` is the Rust workspace root (`src/Cargo.toml`); `version.workspace` and
  `[lints] workspace = true` in each crate. Workspace lints: `unsafe_code` forbid;
  `unwrap_used`/`expect_used`/`panic`/`indexing_slicing` deny, with `src/clippy.toml`
  allowing the first three **in tests only**.

## 3. Memory budget (Phase 1)

**What the number means.** `docs/PERF-budget.md` §2 defines four profiles; `desktop`
is `≤ 320 MB` **idle RAM** for the whole desktop, as a sum of per-unit cgroup limits,
not a boot peak and not a per-process figure. The historical `634/946 MB` figures are
whole-system boot measurements at a different scope and are **HISTORICAL** — they are
not comparable with the ledger and are not used arithmetically here.

**Unit inventory (14 units + 3 slices in `image/mkosi.extra`).** The ledger's claims
were compared against the shipped units and drop-ins. Confirmed in current source:
nine rows claimed `MemoryMax=` values that no unit or drop-in sets; `litric-shell.service`
does not set `Slice=`, so `litric-shell.slice` (the file carrying the reclaim-before-kill
design) governs nothing; `litric-ai.slice` has limits and no member; two boot-resident
services (`litric-sysd`, `litric-envd`) had no ledger row at all. Recorded as **BUG-0008**.

**What was built.** `ci/check-memory-budget.py` — a static checker that resolves each
ledger claim against the unit files, computes the effective limit along a unit's slice
chain (systemd's rule: the most stringent value in the chain wins), rejects a bare
numeric value (`MemoryMax=12` is twelve bytes), rejects a directive inside a comment,
detects a dead slice (limits and no members), and fails when a cap exists that the
ledger does not list. It reports `ERROR` / `ACKNOWLEDGED` separately; `--strict` fails on
acknowledged gaps too.

**Result.** `python3 ci/check-memory-budget.py` → **PASS, 0 ERROR / 11 ACKNOWLEDGED / 1 INFO**;
`--self-test` → **PASS 10/10** (including "a directive inside a comment does not count"
and "a value configured on a slice the unit does not join must fail"). `--strict` exits 1
by design — the acknowledged gaps are real and open.

**What is NOT claimed.** The checker proves **configuration policy**, never runtime
consumption. No cgroup was read, no RAM was measured. `tests/budget/` remains empty and the
320 MB total has no runtime enforcement: that needs a booted image reading each slice's
`memory.current`. The ledger was corrected to say `TARGET (not enforced: …)` on every
line that has nothing behind it, rather than being edited to look enforced.

## 4. File operations and data integrity (Phase 2)

Functions traced end-to-end (public API → helper → syscall → error): `Root::open`,
`resolve`, `resolve_parent`, `ensure_dir_chain`, `stat`, `list_dir`, `measure`, `mkdir`,
`new_file`, `rename`, `copy_item`, `move_item`, `pick_name`, `copy_into`, `remove_host`,
`delete`, `stat_with_items`, `search`, `recent`; `DiskTrash::put/restore/purge/empty/list`;
`ArchiveKind::from_name`, `load_archive_bytes`, `compress_zip`, `extract_archive`,
`extract_zip`, `extract_tar`, `extract_single_gzip`, `extract_path`, `write_entry`;
`zip::read_zip_slice/write_zip`; `tar::read_archive/write_header`.

**Verified sound (checked, no defect found — do not "fix" these later):**

| Area | Evidence |
|---|---|
| Confinement | every component is walked with `symlink_metadata`; a symlink anywhere is refused. `create_dir`/`create_new` (`O_EXCL`) are used for creation, so the no-follow guarantee comes from the syscall, not from a check |
| Traversal in archives | absolute names, `..`, `.`, drive letters, backslashes and control bytes are refused on entry *and* again at `extract_path`; links, hard links and devices are refused by the readers |
| ZIP bomb rules | ratio rule (`ratio_exceeded`) applied before decompression, overlapping-data-range rule, declared-vs-actual size equality, CRC check, ZIP64 refused |
| tar limits | checksum recomputed, size parsed from octal **before** allocation, `total_output` bounded before `vec![0u8; size]`, link/device typeflags refused |
| Deletion choke point | `fs::remove_dir_all`/`remove_file` outside test code appear only in `ops::remove_host`, `trash::purge`, and socket-socket cleanup |
| `into_source_dir` and "into itself" | `server.rs` sets it only when `dest == parent(src)`, so the recursion case (`dest_dir == src`) can never slip past the `is_within` guard — **checked specifically, not a bug** |

**Defects found and fixed:**

| Bug | Defect | Fix |
|---|---|---|
| [BUG-0010](bugs/BUG-0010-replace-deleted-the-item-it-was-replacing.md) | `Policy::Replace` deleted the destination **before** copying, contradicting its own documented promise ("only once the new one is complete") — a failed copy lost both items | stage the copy as `.litric-staged-N` in the destination directory, then `rename` it into place; for a directory, move the old one aside, swap, and remove it only afterwards; move across filesystems copies first and deletes the source last |
| [BUG-0011](bugs/BUG-0011-trash-moved-the-item-before-its-record-existed.md) | `DiskTrash::put` moved the item first and wrote `.trashinfo` second; the name was reserved with `exists()` and `fs::write` (truncating), so concurrent or repeated trashing of one name could overwrite an earlier copy | create the record **first** with `create_new` (`O_EXCL`) as the name reservation; treat a name as taken when either `info/<n>.trashinfo` or `files/<n>` exists; remove the reservation if the move fails; `restore` now applies the same one-segment name rule as `get`/`purge` |
| [BUG-0012](bugs/BUG-0012-extraction-aborted-on-entries-it-should-accept.md) | the gzip suffix match was case-sensitive while classification is case-insensitive (`NOTES.TXT.GZ` extracted onto itself → `exists`); a repeated directory entry aborted the extraction part-way | `strip_suffix_ci` (byte-safe, `get()`-guarded); `write_entry` accepts `AlreadyExists` for a directory **only** when a real directory stands there |
| [BUG-0013](bugs/BUG-0013-resolve-checked-nothing-about-its-intermediates.md) | ADR-0019 §3 says every intermediate component must be a real directory; the code's check was `let _ = meta.is_dir();` — a no-op, so the caller got a wrong `not-found` | `peekable()` walk: a component with another behind it must be a directory (`not-a-directory`); the last may be anything |

**Regression coverage added (Rust, NOT EXECUTED here):** four tests in `src/litric-files/src/ops.rs`
(`replace_keeps_the_destination_when_the_copy_fails` — uses a UNIX socket as the uncopyable
source, `replace_installs_the_new_item_and_removes_the_old`,
`move_onto_an_occupied_directory_swaps_instead_of_deleting_first`,
`resolve_reports_a_file_standing_in_the_middle`).

**Design credited, unchanged:** extraction validates and decompresses the **whole** archive
before writing the first byte, so `ENOSPC`-class failures are the only partial-write case
left, and the answer is an error rather than a silent success. Making extraction atomic
via a temporary directory is a real improvement but a separate change, with its own tests.

**Deliberately deferred:** [BUG-0009](bugs/BUG-0009-trash-path-field-is-not-a-location.md) —
`Path=` holds `file:///Documents/x`, which is neither an absolute path nor a usable relative
one, so another tool reading the trash restores to the wrong place. The specification
requires the original **location**; the fix spans two crates and four sites (both writers,
both readers) as one atomic change, because half of it would make the shell restore
daemon-trashed files into `/`. Specified in the bug record, not landed blind.

## 5. Defender and benchmark counters (Phase 3)

**Defender (`src/litric-defender`): no counters.** It is a per-scan function returning a
`ScanReport`; there is nothing to accumulate. What was inspected instead is the decision
semantics:

| Property | Finding |
|---|---|
| Asymmetry | a **detection denies in every mode**; only unknowns follow `Mode`. This is checked in-code and tested (`detection_denies_even_when_permissive`) — the classic fail-open trap is closed deliberately |
| Scenario hazard | `Decision::Deny { tier: Tier::Pattern, id: "unscannable:<reason>" }` gives a *tooling failure* a tier label that means "byte-pattern match". No aggregator in this repository counts by tier, so the impact is audit-log labelling only. Recorded, not changed (it would alter a public enum that `litric-caps` also matches on) |
| Cache | generation guard is bumped on every feed reload, so stale verdicts cannot survive a definitions change. Caching `AllowUnknown` as `Clean` is documented as the Phase 0 trade-off. Two gaps, both recorded below: `set_mode` does not bump the generation, and stale entries are pruned by eviction pressure only (the comment says "on insert/lookup"; `lookup` returns `StaleGeneration` without removing) — memory stays bounded by `capacity`, so neither is a leak |

**Benchmark (`src/litric-bench`): the counters are real and were narrowed.**

| Counter | Semantics (from source) | Verdict |
|---|---|---|
| `Scoreboard::{true_positives, false_negatives, false_positives, true_negatives, unsupported, operational}` | one increment per recorded sample, exhaustive by `match` arm; `operational` deliberately lumps every non-classifying outcome (crash, timeout, recovery states, environment limitation) | semantics sound; `record` never double-counts |
| `LatencyProfile::samples_us` | one wall-clock delta per scan, monotonic clock, clamped to `max(1)` µs | sound |
| `LatencyProfile::{mean, percentile, samples_per_sec}` | mean over `u128` (correct); percentile = linear interpolation over a sorted copy | **two defects, see below** |
| `Scoreboard::{detection_rate, false_positive_rate}` | `TP/(TP+FN)`, over *classified* samples | **narrowing defect, see below** |

* **[BUG-0014](bugs/BUG-0014-benchmark-statistics-narrowed-counters-to-u32.md)** — the rates
  converted their `u64` operands to `u32` first (wrapping above 4 294 967 295), and
  `percentile` routed every sample through `u32` and answered an impossible index with the
  constant `u32::MAX` — a plausible latency no sample produced. Fixed: direct `u64 → f64`,
  a rejected `p` outside 0..=100 (including NaN) reports 0, and the fallback is the slowest
  real sample. `REPORT.md` now prints the population each rate is computed over
  ("of N classified hostile samples") instead of leaving the denominator implied.
* **Invariant added:** `Scoreboard::recorded()` and `classified_hostile()`, plus a test over
  all 2 × 10 (truth, outcome) pairs asserting exactly one counter moves —
  `attempted = TP + FN + FP + TN + unsupported + operational`. This is the terminal-state
  invariant the workstream asks for, and it is now expressible and tested.
* **NOT examined here** (honest scope limit): warm-up, repeated trials, worker counts,
  outlier handling, and the fact that cache-hit scans and cold scans share one latency
  profile. The per-sample `from_cache` flag makes the split possible; the printed
  mean/P95/P99 mixes the two populations today.

## 6. Repository evidence and CI (Phase 4)

* `bash ci/verify.sh` (with cargo hidden from `PATH` so the Rust stage reports SKIP rather
  than compiling): **14 pass, 0 fail, 6 skipped**, exit status **0**.
* **No gate verdict was changed.** Nothing in this session produced boot, QEMU, image or
  measurement evidence, so GATE 01 conditions 3/5/6/8 stay `NOT RUN` and condition 7 stays
  `NOT VERIFIED`. `docs/EVIDENCE.md` was deliberately **not** hand-edited: it declares itself
  generated by `ci/verify.sh --evidence`, so a hand-written claim in it would be a new
  instance of the very problem the previous audit fixed.
* Citation integrity: every path cited in the new documents resolves in this checkout; the
  CI stage `[3d]` ("every file cited in docs exists" + "citation linter detects a fabricated
  citation") was re-run after the new files were written.
* `ci/verify.sh` itself: searched for commands that could implicitly build or launch
  (cargo, mkosi, qemu, systemctl). All such stages are guarded and report SKIP with a named
  reason; no stage prints PASS without a command that ran.

## 7. Other high-severity findings from the bounded scan

Evidence-backed only; each is a finding, not a fix, unless stated.

1. **Partial multi-item requests reported as a plain failure — fixed.** `transfer` returns
   `Outcome::Fail(e)` from inside the per-source loop, so if the third of five copies failed,
   two files existed and the client saw one `ERR` with no count. LFP/1 can express `OK n` or
   one `ERR`, never both, so the reply cannot enumerate what landed; the message now can.
   `server.rs::partial` prefixes the failing item and "(n of m items completed)" for
   `COPY`/`MOVE`, `TRASH`, `RESTORE`, `PURGE` and `DELETE`, and ADR-0019 gained a
   **Partial failure** clause stating the contract (items already done stay done; a client
   that sees `ERR` after `PROGRESS` must re-read the destination).
   *Remaining:* the shell does not yet re-read automatically — §12.
2. **The second trash implementation had the same defects — fixed.**
   `src/litric-shell/src/fsbridge.rs` (`trash`, `trash_list`, `trash_restore`) writes the
   same on-disk layout from the shell side with the same move-then-record order and the same
   `exists()`-based name check, and it listed a record whose item was missing. It now uses
   `reserve_trash_name` (record first, `create_new`/`O_EXCL`, both halves of the entry counted,
   rollback on a failed move) and skips phantom records, which is what the daemon already did.
   *Remaining inconsistency:* the two writers pick different collision names
   (`name.2.ext` in the daemon, `name - Copy.ext` in the shell). Both are unique, so nothing
   breaks; unifying them is a naming decision, not a safety one.
3. **`Scoreboard.operational` is one bucket for six different things** (crash, timeout,
   recovered, failed recovery, environment limitation, unsupported-by-other-truth). The
   report prints `environment_limitations` separately by rescanning the results, so the two
   views can disagree if the result vector and the scoreboard are ever built from different
   inputs. Low impact today (one caller), worth a single derived field later.
4. **`ci/__pycache__/` and `tests/__pycache__/` exist.** Generated artefacts in the tree.
   Not deleted (unknown ownership); the Python checks avoid writing new ones.
5. **Two defects in the new checker, found by running it.** `ci/lint-file-safety.py` first
   counted a call mentioned inside a **comment** and inside a **string literal** as a real
   deletion site (the same false-positive class as a directive in a comment, BUG-0008), and
   then crashed with `UnicodeEncodeError` on a cp874 console because one allowlist reason
   contained `§`. Both are fixed: the scanner blanks comments and string/char/raw-string
   literals while preserving line structure, and `stdout` is reconfigured with
   `errors="replace"` so a verdict can never turn into a traceback. Recorded because a
   checker that has never been made to fail is not evidence.

## 8. Exact file changes

**Modified**

| File | Why |
|---|---|
| `src/litric-files/src/ops.rs` | BUG-0010 (staged replace, `stage_copy`/`install_staged`/`move_staged`/`staging_name`), BUG-0013 (`peekable` intermediate-directory rule), four new tests |
| `src/litric-files/src/trash.rs` | BUG-0011 (`put` record-first, `reserve_info` with `O_EXCL`, `numbered_names`, orphan-aware occupancy, rollback), `restore` name guard, BUG-0009 note on `original_path` |
| `src/litric-files/src/server.rs` | `partial()` and its five call sites: a multi-item request that stops part-way now names the failing item and how many completed |
| `src/litric-shell/src/fsbridge.rs` | the same BUG-0011 shape in the shell's own trash writer: `reserve_trash_name` (`O_EXCL`, both halves, rollback), `trash_list` skips a record with no item behind it |
| `docs/adr/0019-litric-files.md` | **Partial failure** clause: what an `ERR` after `PROGRESS` means and what a client must do about it |
| `src/litric-files/src/archive.rs` | BUG-0012 (`strip_suffix_ci`, case-insensitive gzip stem, `AlreadyExists` tolerance for a real directory) |
| `src/litric-bench/src/severity.rs` | BUG-0014 (rates without `u32` narrowing, `recorded()`, `classified_hostile()`, two new tests) |
| `src/litric-bench/src/scan.rs` | BUG-0014 (`percentile` guard + no clamp, two new tests) |
| `src/litric-bench/src/report.rs` | BUG-0014 (rates labelled with their population) |
| `ci/check-memory-budget.py` | new static checker for the ledger (Phase 1) |
| `ci/verify.sh` | stage `[3e]` runs the memory checker and its self-test |
| `docs/PERF-budget.md` | ledger made truthful: unenforced rows marked `TARGET (not enforced: … - BUG-0008)`, effective shell values recorded, sysd/envd rows added, TOTAL row names a mechanism that exists |

**Added**

| File | Why |
|---|---|
| `docs/audit/2026-10-09-continuation-recovery-audit.md` | this report (new directory: the existing `docs/evidence/` holds raw logs/screenshots, not reports) |
| `ci/lint-file-safety.py` | source-level guard: every production `fs::remove_file`/`fs::remove_dir_all` in `src/` must sit in a function named in its allowlist with a reason; 7 self-test cases including a comment, a string, a raw string and a test-module site |
| `docs/bugs/BUG-0009…BUG-0014.md` | one record per confirmed defect, using the existing template |

**Not touched:** `docs/GATES.md`, `docs/EVIDENCE.md`, `docs/SYSTEM-SCOPE.md`, ADRs,
`tests/`, `image/`, `tools/`, and every file not listed above.

## 9. Verification log

| # | Command | Why | Result |
|---|---|---|---|
| 1 | `git rev-parse --is-inside-work-tree` | establish whether a safety net exists | **no git** — recorded, nothing initialised |
| 2 | `wc -l src/litric-files/src/*.rs src/litric-bench/src/*.rs …` | inventory the real units of work | 13 files, 12 946 lines |
| 3 | `grep -rn "remove_file\|remove_dir_all" src/` | find every deletion site | only `ops::remove_host`, `trash::purge`, socket cleanup, tests |
| 4 | `grep -rn "into_source_dir" src/` | test the "copy into itself" suspicion | refuted: `dest == parent(src)` by construction |
| 5 | `python3 ci/check-memory-budget.py` | enforce the ledger | **PASS 0 ERROR / 11 ACK / 1 INFO** |
| 6 | `python3 ci/check-memory-budget.py --self-test` | negative control | **PASS 10/10** |
| 7 | `python3 ci/check-memory-budget.py --strict` | acknowledged gaps visible | exit **1**, as designed |
| 8 | `bash ci/verify.sh` (cargo removed from `PATH`) | full permitted CI subset | **14 pass, 0 fail, 6 skipped**, exit 0 — that run predates stage `[3f]` and the second-crate changes; row 14 is the run that covers the final state |
| 9 | `rustfmt --check --edition 2021 <8 files>` | syntax/format validation **without compiling** | after fixing formatting of my own new code: **exit 0 on all eight** (`ops`, `trash`, `archive`, `severity`, `scan`, `report`, `server`, `fsbridge`) |
| 10 | `for f in …; do python -c 'brace/paren balance'; done` | catch a truncated edit | balanced on the three files of the first pass |
| 11 | `python3 ci/lint-file-safety.py` | the new deletion choke-point guard | **OK: 25 deletion sites, all inside reviewed functions**, exit 0 |
| 12 | `python3 ci/lint-file-safety.py --self-test` | negative controls for the new guard | **7/7 behaved as designed**; runs 5 and 6 of it caught the comment/string false positive and the Unicode crash |
| 13 | `bash -n ci/verify.sh` | the edited script still parses | **OK** |
| 14 | `bash ci/verify.sh` (final, cargo hidden) | full permitted subset after every change | **16 pass, 0 fail, 6 skipped**, exit 0 |

Side effects: none outside the workspace. `rustfmt --check` and the Python checkers write
nothing; the CI run created no new artefacts; no unit, service, container or VM was started.
Not a single build, compile or type-check was performed.

## 10. Skipped / blocked / NOT VERIFIED

| Item | Status | Reason |
|---|---|---|
| Every Rust test (existing and new) | **NOT RUN** | compilation is prohibited in this mode |
| `cargo check` / `clippy` / `fmt --check` on the workspace | **SKIPPED** | the CI stage reports SKIP: `cargo not on PATH`; `rustfmt --check` was used as a parser only |
| Type-level correctness of the six edited Rust files | **NOT VERIFIED** | `rustfmt` parses, it does not type-check |
| Image build, UKI, dm-verity | **SKIPPED** | not permitted; `mkosi` also absent |
| Boot smoke test, GATE 01 conditions 3/5/6/8 | **NOT RUN** | needs QEMU + a real image |
| Idle RAM / CPU / wakeups / boot time | **NOT RUN** | needs a booted, logged-in image |
| Runtime memory compliance against 320 MB | **NOT VERIFIED** | no cgroup was read; the static checker proves configuration only |
| `tests/budget/` runtime enforcement | **SKIPPED / absent** | empty directory; runtime work |
| BUG-0009 behavioural fix | **NOT APPLIED** | cross-crate format change, specified with an ordered plan |
| `fsbridge` trash fixes | **APPLIED, NOT VERIFIED** | the shell's second trash writer now reserves its record first; no compiler has seen it, and its tests have not run |
| `server.rs` partial-failure messages + the ADR clause | **APPLIED, NOT VERIFIED** | same: parse-clean and format-clean, never executed |
| Defender behaviour under a real feed or `fanotify` | **NOT VERIFIED** | transport is a later phase; the engine is pure and untested at runtime here |
| Benchmark warm-up / repeated trials / worker counts | **NOT EXAMINED** | out of this session's bounded scope; recorded as follow-up |

## 11. Residual risk and the runtime gate

1. **Nothing here is proven at runtime.** Eight source files changed — `ops.rs`, `trash.rs`,
   `archive.rs`, `server.rs` (litric-files), `fsbridge.rs`, `severity.rs`, `scan.rs` and
   `report.rs` — and **not one line of them has been compiled**. The tests are written, reviewed and unexecuted; the strongest
   statement available is "parses, format-clean, and the defect is provable from the call
   sequence". Two defects in this session's *own* new checker were found only by running it,
   which is the argument for making every new guard fail on purpose before trusting it.
2. **The TOCTOU class is unchanged and accepted** (ADR-0019): a same-user process can race
   any check-then-act. Closing it needs `openat`/`O_NOFOLLOW`, i.e. `unsafe` or libc, both
   forbidden by workspace policy. The staged-replace change narrows the window's *effects*,
   it does not remove the window.
3. **Partial extraction and partial multi-item transfer** remain the two places where a
   failure leaves visible state behind; both report an error rather than success, and both
   have a named improvement.
4. **The 320 MB budget remains a target.** Configuration is now checked; consumption is not.
5. **No Git** means no diff review of these changes. Anyone taking this work forward should
   put the tree under version control before the next change, whatever else they do.

## 12. Next prioritised work

| # | Task | Depends on | Acceptance |
|---|---|---|---|
| 1 | Run the new Rust tests in an authorised build (`ops.rs`, `severity.rs`, `scan.rs`, `fsbridge.rs`) | cargo | all four ops tests + three bench tests pass; `replace_keeps_the_destination_when_the_copy_fails` must fail if the staging change is reverted |
| 2 | Fix `Path=` across both crates (BUG-0009) | — | one patch, four sites; a legacy `file://` fixture still restores; `gio trash --list` shows the original path on a booted image |
| 3 | Refresh the shell's view after a partial failure | — | a `COPY` that reports `ERR (2 of 5 items completed)` leaves the window showing what is actually on disk |
| 4 | Runtime budget gate in `tests/budget/` reading each slice's `memory.current` | booted image | fails on a total over 320 MB, with the per-line breakdown in the failure message |
| 5 | Split the latency profile by `from_cache`; document warm-up and excluded runs | — | `REPORT.md` states what population each percentile describes |
| 6 | Close BUG-0008's four policy decisions (shell `Slice=`, the 110/140 vs 96 number, sysd/envd budgets, the dead AI slice) | product decision | `ci/check-memory-budget.py --strict` exits 0 |
| 7 | Unify the two trash collision-name rules (`name.2.ext` vs `name - Copy.ext`) | — | one rule in both writers, or a documented reason for two |

## 13. Research references

| Source | What it decided |
|---|---|
| [Freedesktop Trash 1.0](https://specifications.freedesktop.org/trash/latest) | `.trashinfo` MUST be created **first**, atomically via `O_EXCL`; a later trashing must never overwrite an earlier copy; `Path=` is the original *location* (absolute pathname, or a relative one measured from the trash directory's parent, never containing `..`). Grounds BUG-0011's fix and BUG-0009's deferral |
| [systemd.resource-control](https://www.freedesktop.org/software/systemd/man/latest/systemd.resource-control.html) and [kernel cgroup-v2](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html) | the effective memory limit is the most stringent value along a unit's slice chain — the rule `ci/check-memory-budget.py` implements, and the reason the shell's `MemoryHigh` above its own `MemoryMax` can never fire (BUG-0008) |
| [Rust `std::fs`](https://doc.rust-lang.org/std/fs/) and [`File`](https://doc.rust-lang.org/std/fs/struct.File.html) | metadata checks race, and `create_new` (`O_EXCL`) is the atomic alternative; the basis for staging the replacement and for `create_new` in `write_entry`/`reserve_info` |
| PKWARE APPNOTE (via the module header) and POSIX ustar | the bomb, overlap, size and typeflag rules verified in `zip.rs`/`tar.rs` (no change made) |
| [OWASP File Upload Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html) | the traversal / overwrite / archive-bomb checklist used to decide the archive matrix |
| ADR-0019 (this repository) | the confinement contract, the accepted TOCTOU risk, and the extraction limits the code was checked against — including the intermediate-directory rule that BUG-0013 found unimplemented |
