# BUG-0002 — The boot smoke test was truncated and could not run

```
Bug ID        : BUG-0002
Severity      : S2 (HIGH) — the only check gating GATE 01 was itself broken
Class         : TEST (PROTOCOL §8: false-negative test)
Subsystem     : tests/
First Observed: 2026-08-26, ci/verify.sh first run
```

## Reproducer

```bash
python -m py_compile tests/smoke_boot.py
```

## Expected

The file compiles; `pytest tests/smoke_boot.py` collects four test cases, as
`docs/GATES.md` claimed ("CI smoke test (4 test cases)").

## Actual

```
IndentationError: expected an indented block after 'finally' statement on line 250
```

The file ended mid-statement at line 251. It had never been executed by anything.

## Three further defects found while repairing it

Each of these would have made the test *pass while testing nothing*, which is
worse than the syntax error — a syntax error announces itself.

1. **`-append` is ignored for this image.** `_build_cmd()` passed
   `-append "console=ttyS0 …"`. QEMU only honours `-append` together with
   `-kernel` (direct kernel boot). LitricOS boots a Unified Kernel Image from the
   ESP through UEFI, and that command line is baked into the signed PE. The
   `-append` had no effect, so any behaviour it appeared to configure was not
   actually configured.
2. **A panic-injection written against that same mechanism.** The repair
   initially added a `test_panic_is_detected` that injected
   `init=/nonexistent` via `-append`. Same defect: it would have reported a
   confident PASS having changed nothing. Replaced with corruption of a *copy* of
   the image, which exercises the real path and additionally covers the Phase 03
   "tampered rootfs fails closed" property.
3. **`runner.image` was set but never read.** The repair set an attribute to
   redirect the runner at the corrupted copy, while `_build_cmd()` still
   interpolated the module-level `IMAGE` constant. Code that looks like it
   works and does not.

A fourth, lower severity: `preexec_fn=os.setsid` is POSIX-only and raises on a
Windows host. Now applied conditionally.

## Impact

`docs/GATES.md` listed this file as a completed Phase 01 deliverable. It was the
only artifact standing between "we built an image" and "the image boots". It
could not have run, and three of its mechanisms would not have worked if it had.

This is the same root cause as [BUG-0001](BUG-0001-fabricated-mkosi-config.md):
**an artifact was recorded as complete on the basis of existing rather than
executing.**

## Root Cause

No check existed that ran the repository's own files. Nothing compiled the
Python, nothing parsed the shell scripts, nothing built the Rust. A file could be
written, listed as a deliverable, and never touched again.

## Fix

1. `tests/smoke_boot.py` completed; the four defects above corrected.
2. `ci/verify.sh` added — runs every check that can execute on the current host
   and reports `SKIP` with a reason for every check that cannot. It is the
   mechanism that makes this defect class impossible to repeat: a file that does
   not compile now fails the verification run.
3. `docs/EVIDENCE.md` is generated from that run, so the list of verified claims
   is produced by execution rather than by writing.

## Regression Test

`ci/verify.sh` check `[2] all python files compile`. Verified to fail against the
truncated file and pass against the repaired one.

## Verification

- `ci/verify.sh`: **10 pass, 0 fail, 5 skipped** (2026-08-26).
- The smoke test itself: **NOT RUN.** It requires a built image and QEMU, neither
  of which exists yet. Its syntax is verified; its behaviour is not.

## Residual Risk

High, and stated plainly: a syntactically valid test that has never been executed
against a real image is unproven. The `-append` defect is exactly the kind that
only surfaces on first real execution. Nothing in `tests/smoke_boot.py` may be
cited as evidence until it has run against a booting image and been shown to fail
against a corrupted one.
