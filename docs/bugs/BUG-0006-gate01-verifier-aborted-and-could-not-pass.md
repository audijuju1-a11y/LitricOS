# BUG-0006 — The GATE 01 verifier aborted after one check, and three of its probes could never pass

```
Bug ID        : BUG-0006
Severity      : S1 (CRITICAL) — the in-VM verifier named by GATE 01 could not
                complete a run, and three of its checks were guaranteed to FAIL
                on a machine that was working correctly
Class         : TEST (PROTOCOL §8: false-negative test; §32: a check that cannot
                fail proves nothing — this is its mirror image, a check that
                cannot pass)
Subsystem     : tests/
First Observed: 2026-10-09, code-only audit (script executed on the development
                host against stub commands; no VM, no build)
```

## Reproducer

The script is meant to run inside the guest. It is execrable without one by
putting stubs for `systemctl`, `dmsetup`, `findmnt`, `systemd-analyze` and `cat`
on `PATH`, which is how it was executed here — no service was started, no guest
was booted, nothing outside a temporary directory was touched:

```bash
PATH="<stub-dir>:$PATH" bash tests/gate01-verify.sh; echo "exit=$?"
```

## Expected

Seventeen checks run, the summary line prints, and the exit status reflects the
FAIL count.

## Actual

Two independent defects.

### 1. `((PASS++))` under `set -e` killed the script at the first successful check

A post-increment evaluates to the value *before* the increment, so the first
`((PASS++))` (0 → 1) returned status 1, and `set -e` terminated the script —
inside a function, at top level, not in a condition context. Measured against
the unmodified file:

```
============================================
  LitricOS GATE 01 Verification
============================================

1. Boot Completion
  ✅ PASS: systemd running
REAL-SCRIPT-EXIT=1
```

Sections 2–7 never ran, the summary never printed, and the exit status was 1
for a machine whose first check had just passed. `((FAIL++))` and `((WARN++))`
had the same defect, so the script could not have survived a failure either.

The same class was avoided elsewhere in the repository — `ci/verify.sh` uses
`PASS=$((PASS + 1))` — which is why this is recorded as a defect in one file
rather than as a house style.

### 2. Three probes could not pass, whatever the machine did

| Check | Pre-fix probe | Actual source | Consequence |
|---|---|---|---|
| MGLRU enabled | `cat /sys/kernel/mm/lru_gen/enabled`, expected `"y"` | a bitmask; boot evidence records `litric.kernel.mglru=0x0007` | guaranteed FAIL on a machine where MGLRU works |
| Landlock LSM active | `cat /proc/self/attr/current`, expected `"landlock"` | that file is the *label of the calling process*, not the LSM stack; the stack is `/sys/kernel/security/lsm` | guaranteed FAIL |
| PREEMPT enabled | `grep CONFIG_PREEMPT /proc/version` | `/proc/version` is the kernel banner and contains no `CONFIG_*` symbol at all | guaranteed FAIL |

A verifier that reports three permanent FAILs against a healthy system trains
its reader to ignore FAILs, which is worse than not checking. The evidence for
what the real sources return is already in this repository: `litric-selftest`
(`image/mkosi.extra/usr/lib/litric/selftest`) reads
`/sys/kernel/security/lsm`, `/sys/kernel/mm/lru_gen/enabled` and
`dmsetup table`, and the boot log records its output. The replacements use those
same sources, so the gate and the in-guest report cannot disagree about what the
machine is doing.

A fourth probe was weaker than it looked: `mount | grep verity` searches mount
output for the string "verity", but the root mapper device is `/dev/mapper/root`
— the word need not appear. Replaced with `dmsetup table | grep -c verity`,
which is what `litric-selftest` already used.

## Impact

`tests/gate01-verify.sh` is the script GATE 01 names for verifying a booted
image ("Run inside the VM"). Anyone who followed that instruction got a run that
stopped after one line, exited non-zero, and gave no way to tell a broken image
from a broken script. The two failure modes are opposite in sign — one aborts
early, the others never pass — and both make the run's output meaningless while
looking like an ordinary result.

## Root Cause

The script was written to be run once, by hand, inside a guest, and it was never
executed by the checks it belongs to: `ci/verify.sh` syntax-checks shell files
with `bash -n`, which cannot see a `set -e` interaction or a probe that never
matches. `bash -n` passing is the reason defect 1 survived: `((PASS++))` is
perfectly valid syntax.

## Fix

1. Counters are assigned (`PASS=$((PASS + 1))`), which always returns 0.
2. `FAILED=$(... | wc -l)` gained `|| FAILED=0`: under `pipefail` a failing
   `systemctl` would otherwise abort at the assignment.
3. Every probe now reads through `cat` and matches with bash's own regex, so the
   script needs no `grep -oP`/`-E` flags and can be exercised against a stub
   `cat` rather than the real `/proc` and `/sys` of whatever host runs it.
4. New section 5b records the root filesystem's type and asserts read-only —
   the structural claim the whole verity design rests on, previously unchecked.
5. The MGLRU, Landlock, PREEMPT, seccomp and verity probes use the sources
   `litric-selftest` uses.

## Regression Test

`tests/test_harness_integrity.py` runs the real script against stub commands:

| Test | Property |
|---|---|
| `test_gate_verifier_runs_to_completion` | reaches the summary and section 7, prints `GATE 01: PASS`, exits 0 |
| `test_gate_verifier_reports_a_failure_and_exits_nonzero` | a `degraded` machine prints `GATE 01: FAIL` and exits 1 |
| `test_gate_probes_can_pass_on_a_healthy_machine` | MGLRU, verity, Landlock, PREEMPT and seccomp all PASS on stubs that report a healthy machine |

The middle case is the negative control: it is what makes the first one mean
something.

## Verification

Executed 2026-10-09, Windows host, bash 5.x (Git Bash), stub commands on PATH.
No VM, no guest, no service started.

| Command | Result |
|---|---|
| `bash -n tests/gate01-verify.sh` | **PASS** (also passed before the fix — that is the point) |
| fixed script, all-stub healthy environment | **17 passed, 0 failed, 0 warnings**, exit **0** |
| fixed script, `SYSTEM_STATE=degraded` | FAIL reported, exit **1** |
| unmodified script, same stubs | aborted after 1 check, exit **1** |
| `python -m pytest tests/test_harness_integrity.py -q` | **23 passed** |

## Residual Risk

**NOT VERIFIED against a real guest.** The probes were validated against stubs
whose output was chosen to represent a healthy machine, read from this
repository's own boot evidence where possible. What remains unproven is whether
the real Console, unit names and paths behave as assumed on a booted LitricOS:
`litric-sysd`, `litric-shell` and the four systemd units named in section 6 are
asserted to exist by name, and the script has never run on the image.

`grep -oP` still appears in section 7 (boot timing); PCRE support is a GNU grep
feature, so a busybox-only guest would report `unknown` there rather than fail.
Left as is because section 7 only reports, it does not decide.

The script remains untested by `ci/verify.sh`'s own Python and shell checks
beyond `bash -n`; the stub-execution test above lives in pytest, which is where
a stub can be assembled.
