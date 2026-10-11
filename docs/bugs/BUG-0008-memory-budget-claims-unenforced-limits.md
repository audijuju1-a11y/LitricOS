# BUG-0008 — The memory budget claimed limits that were not configured, and a slice that governed nothing

```
Bug ID        : BUG-0008
Severity      : S1 (CRITICAL) — the document that decides whether the desktop is
                admissible described enforcement that does not exist, and the
                largest component's documented behaviour is not the behaviour
                the units produce
Class         : POLICY / DOCUMENTATION-INTEGRITY (PROTOCOL §45: missing critical
                evidence is BLOCKED, never PASS; the memory ledger had no
                evidence at all)
Subsystem     : image/, docs/
First Observed: 2026-10-09, static check of `docs/PERF-budget.md` against
                `image/mkosi.extra/usr/lib/systemd/system/*`
```

## Reproducer

```bash
python3 ci/check-memory-budget.py          # the checker written for this defect
python3 ci/check-memory-budget.py --self-test
grep -c 'MemoryMax' image/mkosi.extra/usr/lib/systemd/system/*.service
grep -n 'Slice=' image/mkosi.extra/usr/lib/systemd/system/litric-shell.service
```

No build, no boot, no unit started. Every claim below is a comparison between two
files in this repository.

## Expected

`docs/PERF-budget.md` §3 says: "Each line is either a configured
`MemoryMax=`/`MemoryHigh=` … or an explicit `TARGET (not enforced: ...)`". For
every claim in the table, the limit is configured on the unit the line names (or
on a slice that unit joins), and the value that applies to that unit is the value
the table states.

## Actual — fourteen divergences, in four groups

### 1. Nine rows claimed limits the image never configures (6 claims + 1 slice claim)

| Row | Claimed | Reality |
|---|---|---|
| 3 journald | `MemoryMax=12M` | the journald drop-in (`image/mkosi.extra/usr/lib/systemd/journald.conf.d/00-litric.conf`) sets `Storage=`, `RuntimeMaxUse=` and friends — that is journal *configuration*, not a unit limit. No `systemd-journald.service.d/` drop-in exists. |
| 4 udevd | `MemoryMax=10M` | no drop-in |
| 5 dbus-broker | `MemoryMax=8M` | no drop-in |
| 6 iwd + networkd + resolved | `MemoryMax=20M` | no drop-in |
| 7 seatd | `MemoryMax=4M` | no drop-in |
| 9 pipewire + wireplumber | `MemoryMax=24M` | no drop-in |
| 2 systemd PID 1 | `system.slice` | two errors in one cell: PID 1 is not in `system.slice` (it is in `init.scope`), and `system.slice` carries no memory limit in this image either. |

These are distribution units, so the correct mechanism is a drop-in under
`image/mkosi.extra/…/<unit>.d/`. None exists. The numeric coincidence is worth
noting for anyone tempted to trust a grep: `MemoryMax=24M` *does* appear in the
tree — on `litric-capd.service`, which has nothing to do with pipewire.

### 2. The shell's documented behaviour is not the behaviour its units produce

`litric-shell.slice` sets `MemoryHigh=110M`, `MemoryMax=140M`, `MemorySwapMax=64M`,
`CPUWeight=1000`, `IOWeight=500`, `TasksMax=512`, and its comment explains the
design: "the gap between them is deliberate: a shell that briefly spikes should
be reclaimed, not killed".

`litric-shell.service` **does not set `Slice=`**, so it defaults to `system.slice`
and joins none of that. The slice governs nothing.

What actually applies to the shell today:

| Property | Documented | Effective |
|---|---|---|
| `MemoryMax` | 140M (slice) | **96M** (the service's own directive) |
| `MemoryHigh` | 110M (slice) | **unbounded** |
| `CPUWeight` | 1000 | **default** (100) |
| `IOWeight` | 500 | **default** (100) |
| `MemorySwapMax` | 64M | **default** |

So the reclaim-before-kill design is not in effect, the hard kill threshold is
lower than every documented number, and the shell does not have the highest CPU
weight on the system — which the slice's own comment says is deliberate so "the
thing drawing the frame the user is looking at" wins contention.

This is not a one-line fix. Adding `Slice=litric-shell.slice` restores
`CPUWeight`, `IOWeight`, `MemorySwapMax` and `TasksMax`, but it does **not**
restore the documented reclaim behaviour: `MemoryMax=96M` on the service is
tighter than the slice's 140M, and systemd's effective value is the most
stringent along the chain, so a `MemoryHigh` of 110M sitting above an effective
hard maximum of 96M can never fire. **Which number wins is a product decision,
not a mechanical one**, and it is the reason this bug is recorded rather than
patched:

* Option A — 96M is the real cap: keep the service directive, and either set the
  slice's `MemoryHigh` below it (so reclaim happens before the kill) or remove the
  slice's memory lines and correct the ledger to 96M.
* Option B — 110/140 is the real budget: remove or raise the service's
  `MemoryMax` and put the service in the slice, accepting a 140M kill threshold.
* Option C — split the difference with a new pair of numbers, which requires its
  own evidence, because no measurement supports it today.

### 3. Two slices carry limits that nothing joins

`litric-shell.slice` (§2) and `litric-ai.slice` (`MemoryHigh=10M`,
`MemoryMax=12M`, `CPUWeight=10`). There is no `litric-aid.service` in the image,
so the AI slice governs nothing either. A slice with limits and no members is not
a conservative setting; it is a comment with a syntax.

### 4. Five caps were not in the budget at all

| Unit | Cap | Resident at idle? |
|---|---:|---|
| `litric-sysd.service` | 64M | **yes** — `WantedBy=multi-user.target` |
| `litric-envd.service` | 128M | **yes** — `WantedBy=multi-user.target` |
| `litric-browser@.service` | 1536M | no — socket-activated per session |
| `litric-files@.service` | 256M | no — socket-activated per operation |
| `litric-capd.service` | 24M | no — socket-activated via `litric-capd.socket` |

The first two matter: they are enabled at boot, so they are part of an idle
desktop, and the 320 MB total has no line for either. Their caps alone are
192 MB. The idle total is therefore the total of the lines that happen to be
written down, which is not the same claim as "the idle desktop fits in 320 MB".

## Impact

The budget is the mechanism this project uses to decide whether a profile is
admissible, and it was reading as enforced while nine of its fifteen lines had
nothing behind them. The consequences are not memory-related: a future
contributor adds a service, adds a `MemoryMax` from the table's numbers, and the
table's numbers were never the numbers.

`litric-shell` is the specific risk. Its documented design (reclaim, then a
higher hard ceiling) is not what runs; the running configuration kills it at the
lowest number in the document, with no reclaim pressure and no CPU priority.

## Root Cause

Two causes, both familiar in this repository:

1. The table was written as intent and read as configuration. `MemoryMax=12M` in
   a table cell is a plan; the same string in a unit file is a limit. Nothing
   compared the two, and `tests/budget/` — the file the table named as its
   enforcement — did not exist (BUG-0007's sibling: a citation that cannot be
   opened).
2. A slice was created with the design rationale in a comment and no member ever
   joined it. The slice looks correct in isolation, and `systemd-analyze verify`
   would not complain: an empty slice is legal.

## Fix (this session)

1. **`ci/check-memory-budget.py`** — a static checker that resolves every ledger
   claim against the units, computing the effective limit along each unit's slice
   chain (systemd's `EffectiveMemoryMax=` is the most stringent of the unit and
   its parent slices). It fails on an ungrounded claim, a cap the ledger does not
   list, a directive in the wrong section, a bare-number value (`MemoryMax=12`
   means twelve bytes), a dead slice, or a table that does not add up. Ten
   self-test cases, including that a directive inside a comment does not count.
   `--strict` also fails on acknowledged gaps, for when they are closed.
2. **The ledger now says what is true.** The nine unenforced rows carry
   `TARGET (not enforced: ... - BUG-0008)`; the shell row carries an `[ACK: ...]`
   naming the effective values and this bug id; rows 14 and 15 were added for
   `litric-sysd` and `litric-envd` with `unset` budgets, so the two resident
   services are at least visible in the table; §3b lists the three non-idle caps.
3. **The TOTAL row names the mechanism that checks it**, which is the one it
   previously claimed (a file that did not exist).
4. **`ci/verify.sh` section `[3e]`** runs the checker and its self-test.

No unit file, no limit value, and no policy was changed. The decision in §2 and
the six drop-ins in §1 are the owner's, and both are specified above rather than
guessed at.

## Regression Test

`ci/check-memory-budget.py --self-test` — 10 cases. The two that pin this bug:
`ungrounded claim` (a cell claiming a value no unit sets must fail) and
`claim not effective for the unit` (a value configured on a slice the unit does
not join must fail rather than pass). Both were verified to fail correctly
against deliberately broken fixtures.

## Verification

Executed 2026-10-09 on the Windows development host. Static file comparison
only: no unit was started, no systemd was queried, no image was built.

| Command | Result |
|---|---|
| `python3 ci/check-memory-budget.py` before the ledger correction | **FAIL — 16 errors** (14 divergences + arithmetic/accounting) |
| `python3 ci/check-memory-budget.py` after | **PASS — 0 ERROR, 11 ACKNOWLEDGED, 1 INFO** |
| `python3 ci/check-memory-budget.py --strict` | exit **1** (acknowledged gaps remain open, as intended) |
| `python3 ci/check-memory-budget.py --self-test` | **PASS — 10/10** |

## Residual Risk

* **Nothing measured.** This bug is about configuration and documentation. Every
  number in `docs/PERF-budget.md` is still a target: no runtime compliance check
  exists, and none can exist without a booted image reading each cgroup's
  `memory.current`.
* **The checker models systemd, it does not use it.** Slice defaulting, drop-in
  merge order and value parsing are implemented from the documented rules; the
  only authority is `systemctl show` on a booted system, which is NOT RUN. A
  divergence between this model and systemd's would show up as a wrong verdict in
  either direction, so the model is deliberately conservative: it reports
  unknowns, it does not assume them away.
* **`MemoryHigh` above the effective `MemoryMax`.** The claim that such a
  `MemoryHigh` can never fire follows from the hierarchy rules the sources
  describe; it has not been observed on the project's pinned systemd version.
  Anyone acting on Option A should confirm it on a booted system first.
* **Rows 14/15 have `unset` budgets.** The checker prints them as INFO rather
  than inventing numbers, so the arithmetic stays honest — but the 320 MB total
  remains incomplete until a human sets those two lines.
