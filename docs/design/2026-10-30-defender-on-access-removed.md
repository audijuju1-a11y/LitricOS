# B3 — the on-access Defender unit is removed

Date: 2026-10-30
Batch: B3 (master plan Stage B)
Status: landed as a removal, not as an on-access implementation.

## The claim that could not be kept

The former on-access service file described a resident Defender that
would gate file opens through fanotify. Its `ExecStart` asked for `--rules`,
`--cache`, `Type=notify` and `CAP_SYS_ADMIN`. None of that transport exists.
`src/litric-defender/src/main.rs` is an **on-demand** scanner: `--scan PATH` and
`--stdio`, no fanotify, no notification daemon and no file-open gate. The unit
was already parked, but the parked file and the slice beside it still read as a
planned capability, and several current documents still called Defender a
resident service.

B3 offers two outcomes: ship the daemon the unit names, or remove the unit and
every on-access claim until the daemon exists. A fanotify responder needs a
kernel-facing transport, `CAP_SYS_ADMIN` and a booted system; this host has none
of those, so the honest outcome is removal.

## The change

* Deleted the former on-access service file.
* Deleted its slice; a slice whose service
  does not exist governs nothing.
* Added `ci/lint-defender-claims.py` and wired it into `ci/verify.sh` as `[3l]`.
  It fails if either file returns anywhere under `image/`.
* Corrected the current claims in the master plan, beta gate matrix, memory
  ledger, API contract, architecture ownership table, repository layout,
  executive summary, shipped copies and the scanner source header.
* Kept the on-demand scanner and its tests. The Defender application still
  scans when asked; it does not claim to sit on the open path.

## Precedence and what did not change

| Layer | Rule |
|---|---|
| ADR-0022 | Phase 0 policy core remains; Phase 2 fanotify is still future work |
| `src/litric-defender` | unchanged scanner behavior and tests; only the stale header changed |
| Files application | unchanged; no file-open interception is added or removed at runtime |
| `docs/PERF-budget.md` | row 10 is explicitly a future resident target, not a configured cap; `ci/check-memory-budget.py` passes |
| `SYSTEM-SCOPE` | no row moves: rows 529 and 1156 remain `S`, and the Phase-0 engine rows remain `P` |

## Failure and degradation rules

* If a unit or slice is restored without a fanotify daemon, `[3l]` fails; the
  image cannot ship it silently.
* If no Defender daemon runs, file opens are **not** gated. The on-demand
  scanner can still be invoked by the Files/Defender surfaces and reports what
  it actually scanned.
* A missing scanner is reported as an unavailable scan, never as "clean".
* The removal changes no LFP/1 request, no capability decision and no user file.

## Negative control

`python3 ci/lint-defender-claims.py --self-test` plants
`litric-defender.service` and `litric-defender.slice` in a temporary tree and
fails if either is accepted. The real tree now has neither. This is the exact
regression the removal is meant to prevent: a future edit can add a convincing
unit file again, and the gate will reject it before it looks installed.

## Limits

This does not implement on-access scanning, fanotify, a quarantine enforcer or
the `--rules`/`--cache` CLI. It does not boot systemd, load a kernel hook or
prove that any file open was blocked. Those are ADR-0022 Phase 2 and later, and
they remain open.
