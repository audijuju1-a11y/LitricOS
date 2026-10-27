# BUG-0026 — The scan read whole files on the frame thread and called what it never read clean

```
Bug ID        : BUG-0026
Severity      : S2 (HIGH) — two defects in one scan: the desktop stops drawing
                for as long as a file takes to read, and a file too large to read
                is reported as "Clean" with "No threats found in N files"
Class         : UI / SCANNER (file I/O on the draw thread; a missing unknown
                state presented as a pass)
Subsystem     : src/litric-shell/src/shell.rs (`DefenderState`, `ScanShared`,
                `start_scan`, `scan_step`, `scan_worker`, `scan_one`,
                `collect_files`), src/litric-shell/src/apps.rs
                (`DefenderApp::record`, `DefenderApp::finish`)
First Observed: 2026-10-27, reading the scan path against ADR-0022 and the
                Defender report's findings 3 and 7
Status        : FIXED (six regression tests)
```

## Why this exists

`scan_step` ran on the tick — that is, on the thread that composes frames — and
it started with `Engine::new()` every call:

```rust
let mut engine = litric_defender::engine::Engine::new();
engine.load_denylist(eicar_denylist());
...
let report = match std::fs::read(&host) { ... };
```

Three consequences, all of them the same defect wearing different clothes:

1. **The frame thread did file I/O.** `std::fs::read` reads the whole file. One
   1 GiB file in `Downloads` is one 1 GiB read inside a frame, and a scan of a
   few thousand documents is a few thousand of them. The desktop does not
   partially stall; it stops.
2. **The verdict cache could never hit.** ADR-0022 makes the cache the reason a
   second scan of the same folder is cheap. A fresh engine per call — per
   *file*, not even per scan — throws it away before it can be consulted.
3. **"Could not read" was reported as "clean".** The read's `Err` arm produced
   `AllowUnknown { reason }`, and `DefenderApp::record` turned that into a row
   reading `Clean`. `finish` then said `No threats found in {scanned} files`,
   counting a file nobody opened. PROTOCOL LAW 8 forbids exactly that: a check
   that could not run is a SKIP, never a silent pass.

The folder walk had a fourth, quieter version of the same problem:
`collect_files` stopped at its count and depth caps and returned, with nothing
recorded about what it refused to enter.

## Fix

**The scan is a worker.** `start_scan` resolves every logical path to a host path
on the thread that owns the file bridge (`FsBridge::host_path`), then moves the
pairs to a thread named `litric-defender-scan`. The worker needs nothing but
`std::fs`, so it cannot reach back through the file layer, and the draw thread
never opens a file.

**The frame only moves verdicts.** `scan_step(budget)` pops at most `budget`
finished verdicts from `ScanShared::results`, so a scan cannot lengthen a frame
by more than a bounded model update. The scan ends when the queue is empty
**and** the worker has published `done`; an empty queue with a live worker is
"no results yet", not "finished".

**One engine for the run.** `scan_worker` builds one `Engine`, loads the denylist
once, and keeps it for every file — which is what makes the cache reachable.

**A bounded read.** `scan_one` stats the file and refuses anything over
`MAX_SCAN_FILE_BYTES` (64 MiB) with

```
skipped: … bytes is over the … byte scan limit
```

and reads through `Read::take(MAX + 1)` so a file that grows between the stat
and the read is caught as well. Both cases are `AllowUnknown`, which the window
already drew as `Unknown (reason)` — the model had the state; the scanner never
used it.

**The result admits what it did not cover.** `ScanShared` counts `unread` files,
`DefenderState::skipped` counts entries the walk refused, and `scan_step`
appends both to the same sentence as the count of files scanned:

```
1 threat found in 2 files · 1 file(s) not read (over the 67108864 byte limit or
an I/O error) · 7 entries not walked (count or depth cap)
```

## Regression tests

In `src/litric-shell/src/shell.rs`, module `scan_worker_tests`:

* `one_engine_makes_a_second_scan_of_the_same_file_a_cache_hit` — two `scan_one`
  calls on one engine: the second is `from_cache`, which a fresh engine per call
  could never be.
* `a_file_past_the_read_ceiling_is_reported_as_unread_not_clean` — a sparse file
  one byte over the ceiling: `scan_was_skipped` is true and the decision is
  `AllowUnknown` naming the limit, never `Allow`.
* `the_pump_moves_a_bounded_number_of_verdicts_per_frame` — three verdicts, a
  budget of two: the window advances by two, the scan stays running; an empty
  queue with a live worker does not end it; the worker's `done` does.
* `a_finished_scan_says_what_it_did_not_cover` — a detection plus an unread file
  plus seven unwalked entries: the message names all three, the state is
  `Done { threats: 1 }`, and the detection raises a `Critical` notification.

## Verification

* `cargo test --offline --workspace` — 1594 passed, 0 failed
  (`docs/evidence/2026-10-27/workspace-tests-after-interaction-repair.log`).
* `cargo test --offline -p litric-shell scan_worker_tests` — 6 passed
  (`docs/evidence/2026-10-27/scan-banner-terminal-tests.log`).
* `ci/verify.sh` — 29 pass, 0 fail, 3 skip (image build, boot and idle metrics
  need mkosi and qemu, which this host does not have)
  (`docs/evidence/2026-10-27/verify-after-interaction-repair.log`).

## Residual risk

* The scan is still on demand: nothing watches the filesystem, so a file that
  arrives after the scan is not scanned until the next one. That is the missing
  on-access protection the report names separately.
* The worker's result queue is unbounded between two pumps. It is bounded in
  practice by the walk's 2 000-file cap, which is what keeps that honest.
* No image was built or booted for this record: the worker is verified by unit
  tests on the host, not on a booted LitricOS.
