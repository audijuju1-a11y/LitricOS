# The update service: boot entrypoint, status, and the activation seam

Status: implemented in §75; §76 adds crash-consistent state writes and a
cross-process root lock; §77 makes commit require health evidence written by
the running system, not by the caller. The engine from §74 is driven by the real
`litric-sysd` boot unit and exposed over the existing Unix socket. It is still
not a bootloader, a partition writer or a signature verifier; those limits are
listed at the end.

## Why a service, not another library method

The §74 engine owned the A/B state but no running process owned the root. A
shell that drew "update staged" from a direct library read would bypass the
daemon's authorization and would have no answer for a damaged record. The
service boundary fixes both:

```text
litric-update-boot? no — litric-sysd --boot (ExecStartPre)
        |
        v
/var/lib/litric/update/system/state
        ^
        |
litric-sysd --serve  <— Unix socket, mode 0600 —> UPDATE STATUS/STAGE/COMMIT/ROLLBACK
```

`UpdateService` holds exactly one configured root. The path is configuration,
never a request field. `--boot` opens the same root in a separate, short-lived
process before the socket exists; `--serve` opens it lazily for queries and
writes.

## Boot entrypoint

`litric-sysd.service` runs:

```text
ExecStartPre=/usr/bin/litric-sysd --boot --update-root /var/lib/litric/update
ExecStart=/usr/bin/litric-sysd --serve --socket /run/litric/sysd.sock \
                               --update-root /var/lib/litric/update
```

The entrypoint:

1. opens `<root>/system/state`; a missing state prints
   `litric.update.state=uninitialized` and writes nothing;
2. checks whether a `failed` health record names the exact pending
   generation/attempt. If it does, the candidate is rejected under the root
   lock (`outcome=rejected-health`, `last_result=reject:health-check`) and no
   new attempt is counted;
3. otherwise calls `Engine::note_boot`, which increments a staged candidate's
   attempt count;
4. when the result is `booting`, writes a `booting` health record for that
   slot, version and attempt through the same atomic path as the state record;
5. prints `booting` with the attempt number, or `active` when no candidate is
   staged;
6. after the third recorded attempt, the fourth call makes the engine reject
   the candidate and clear it. The active pointer never moved, so the machine
   remains on the previously committed generation. The console says
   `outcome=rejected-budget` and `rollback=active-generation`;
7. a damaged record prints `state=corrupt` and a reason, then the daemon still
   starts. A broken update record must not make a machine unbootable.

Point 6 is deliberately not a call to `Engine::rollback`. A staged candidate
is never the active pointer, so there is no other committed slot to switch to.
The observable guarantee — the failed candidate is not advertised and the
committed generation stays active — is the rollback that is possible at this
layer. `UPDATE ROLLBACK` is the separate operation that moves the pointer and
records a `rollback` event with `reason=manual`.

The attempt is counted once per `litric-sysd` start, not once per kernel boot.
There is no bootloader reading the state file yet; a systemd restart of this
unit is therefore counted as another start. This is a real limitation, not a
claim about kernel boot counting.

## Durability ordering (§76)

Every state transition is one atomic replacement, in this order:

1. write the complete encoded record to a sibling temporary,
   `<system>/.state.tmp-<pid>`, opened `create_new` and mode `0600`;
2. `sync_all` the temporary, so its bytes are on stable storage before the name
   changes;
3. `rename` it over `<system>/state`, the single name a reader opens;
4. `fsync` the containing `system` directory, so the rename itself is durable.

A reader therefore sees either the previous complete record or the next
complete record, never a prefix of one. A crash before the rename leaves the old
record in place; a crash after it leaves the new one. The temporary is never
opened by `Store::load`: only `<system>/state` is. `Store::save` removes its own
same-pid temporary before creating it, and removes it again if the write fails;
temporaries left by an older process are ignored, not parsed.

Health evidence (§77) uses that same primitive: its sibling temporary is
`<system>/.health.tmp-<pid>`, it is fsynced before the rename, and the
`system` directory is fsynced after it. It is a separate record from
`<system>/state`; a commit reads both under the root lock. A leftover health
temporary is never evidence.

The state record is not the whole update. Slot trees and any future EROFS image
are outside this ordering, and there is no power-cut matrix in CI.

## Cross-process lock (§76)

Atomic replacement makes one write indivisible; it does not make a
read-modify-write sequence exclusive. Two `--boot` processes could read attempt
1 and both write attempt 2. The root therefore has one lock file,
`<root>/update.lock`, created with `create_new` (the std equivalent of `O_EXCL`)
and mode `0600`:

```text
litric-update-lock\t1
pid\t1234
at\t1800000000
reason\tboot
```

`--boot`, `UPDATE STAGE`, `UPDATE COMMIT` and `UPDATE ROLLBACK` take it before
opening the engine, and hold it through the atomic save. A writer waits at most
2 seconds for a live owner and then fails with the typed `locked` error
(`ERR locked` over IPC; non-zero `--boot` exit). A lock-IO failure is
`ERR lock-io`. Neither panics and neither silently proceeds.

`UPDATE STATUS` uses the same lock non-blockingly (`create_new` once) and holds
it only while reading and verifying the record, so its fields are one snapshot.
The lock is coordination metadata, not update state: a status query creates and
removes that file but never writes `<system>/state`. If a writer owns it, the
answer is immediate:

```text
state=busy
locked_pid=1234
locked_reason=stage
locked_at=1800000000
retry=lock
```

Reclaim is deliberately narrow:

| Observation | Result |
|---|---|
| complete record, PID has `/proc/<pid>` | live; writer waits, status reports busy |
| complete record, PID has no `/proc/<pid>` | stale; removed and retried |
| malformed record younger than 2 s | initialising; treated as busy, not evicted |
| malformed record older than 2 s | stale; removed and retried |
| no `/proc` (non-Linux) | a complete record is fail-closed as live; only malformed old records are reclaimed |

A live PID is never evicted because of an old `at` timestamp: staging a large
tree may legitimately take minutes. `Drop` removes the lock only after
re-reading it and matching the exact content this process wrote, so a reclaimed
or replaced lock is never deleted by its old owner.

## Boot-attested health evidence (§77)

The state record can name a staged candidate and still say nothing about
whether the running machine reached a usable serving state. §77 adds one
record, `<system>/health`, written only by `litric-sysd`. It is bound to the
candidate generation `(slot, version)` and to the exact attempt:

```text
litric-update-health\t1
generation\tB
version\t1.1.0
attempt\t1
at\t1800000000
verdict\thealthy
checks\troot,state,marker,serving
```

The record is at most 1 KiB, mode `0600`, and rejects a symlink, a non-regular
file, a zero-length or overlong file, non-UTF-8 bytes, an unknown header, a
missing/duplicate field, an attempt of zero, an unknown verdict or trailing
records. `checks` is a bounded comma-separated diagnostic; it does not open the
gate. `verdict` is `booting`, `healthy` or `failed`.

The lifecycle is two-phase:

1. `--boot` opens the engine, verifies the candidate marker and counts the
   attempt. It then writes `verdict=booting` with
   `checks=root,state,marker`. A write failure makes the boot entrypoint exit
   non-zero (`health-io`); it does not pretend the attempt was attested.
2. After the Unix socket is bound, the serve loop makes one non-blocking
   attempt on the root lock. If the lock is free, the engine opens, the pending
   generation still verifies, and the existing evidence names the same
   slot/version/attempt with verdict `booting`, it replaces the record with
   `verdict=healthy` and `checks=root,state,marker,serving`. If the lock is
   busy, the state is absent/damaged, or the evidence does not match, it leaves
the record alone and the commit gate stays closed. It does not wait for the
lock and does not invent evidence for a manually started daemon.

**What `healthy` means, concretely:** the boot entrypoint opened the generation
record and verified the candidate slot/marker and counted this attempt; then
the daemon bound its real Unix socket, entered the serve loop, reopened the same
engine, and the candidate slot/marker and slot/version/attempt still matched.
It does **not** mean the kernel booted from this slot, the network is up, the
display works, a session exists, storage is mounted or a critical service is
healthy. Those probes do not exist here.

### Commit gate

`UPDATE COMMIT` reopens the engine under the root lock, re-checks the pending
candidate, and requires a valid record whose slot, version and attempt all
match. The caller's `PASSED` word is only permission to ask.

| Evidence for this candidate | Result | State |
|---|---|---|
| no record, or only a `.health.tmp-*` | `ERR health-missing` | unchanged |
| zero-length, truncated, malformed, symlinked or unreadable | `ERR health-invalid` | unchanged |
| candidate has `attempts=0` | `ERR health-not-booted` | unchanged |
| valid record for another slot, version or attempt | `ERR health-stale` | unchanged |
| matching `booting` (serve did not attest) | `ERR health-unattested` | unchanged |
| matching `failed` | `ERR health-failed` | unchanged |
| matching `healthy` | engine commit, which re-verifies the candidate | active pointer moves |

There is no override flag. A malformed or stale record is refused before the
engine's commit, so no partial activation is possible. A prior attempt's
`healthy` record is not replayable: the attempt counter is part of the match.
The record is read after the state, under the same lock, so a boot increment
cannot land between the match and the commit.

### Failed evidence and rollback coupling

The next `--boot` reads `failed` evidence before counting a new attempt. If it
names the exact pending generation/attempt, that boot calls
`Engine::reject_pending(HealthCheck)` while holding the root lock: the candidate
is cleared, the active pointer stays where it was, and the event says
`reject:health-check`. This is the same observable return to the committed
generation as the boot-budget path; it is not `Engine::rollback`, because the
candidate was never active. A stale `failed` record for another generation or
attempt is ignored.

## Read degradation precedence (§76)

Status classifies what it observed in this order. The first matching row wins;
the table is the contract, and every row is exercised without a panic.

| # | Observation | Result |
|---:|---|---|
| 1 | no configured root | `state=disabled` |
| 2 | root path does not exist | `state=uninitialized` |
| 3 | a live writer holds the lock (including a fresh initialising record) | `state=busy` + `locked_pid`/`locked_reason`/`locked_at`/`retry=lock` |
| 4 | the lock file cannot be created or inspected | `state=unavailable` + reason |
| 5 | `<system>/state` missing | `state=uninitialized` |
| 6 | state path is a symlink, not a regular file, or over 16 KiB (metadata or the post-read length) | `state=corrupt` + reason |
| 7 | bytes are not UTF-8 | `state=corrupt` + reason |
| 8 | zero-length, unknown header, duplicate/missing fields, truncated line, bad number, trailing record | `state=corrupt` + reason |
| 9 | record parses, but a named slot directory/marker is missing, mismatched, a symlink or malformed | `state=incomplete` + reason |
| 10 | record parses and every referenced generation verifies | `state=ready` |

A leftover `.state.tmp-*` is not in this table because it is never a candidate
for reading. Corruption in the primary record is never repaired by guessing
which temporary was intended.

Health evidence is a second dimension, considered after the state
classification. A bad health record can close the commit gate but cannot turn
`state=ready` into `state=corrupt`, and a missing health record is not a missing
state record. The health reader's own precedence is:

| # | Observation | Result |
|---:|---|---|
| H1 | `<system>/health` is absent (including a temp-only directory) | `health_evidence=absent`, `health_matches=0` |
| H2 | path is a symlink/non-regular, unreadable, zero-length, over 1 KiB, non-UTF-8, unknown header, missing/duplicate field, bad number, unknown verdict or trailing record | `health_evidence=invalid` + `health_reason`, `health_matches=0` |
| H3 | complete record | verdict/slot/version/attempt/at/checks; `health_matches=1` only for the exact pending slot/version/attempt |

Status reads this under the same non-blocking snapshot lock as the state. A
busy root reports `state=busy` and does not guess at health. Nothing in this
reader writes, repairs or deletes the record; a damaged file stays damaged for
an operator to inspect.

The record is root-owned and mode `0600`, but it has no MAC or signature. A root
process that can write the update root can forge it; this gate removes the
ordinary IPC caller's ability to assert health, not a compromised root's.

## IPC schema

`UPDATE STATUS` is read-only and needs no capability. It always answers, even
when the answer is bad:

```text
schema=3
root=/var/lib/litric/update
state=ready | uninitialized | corrupt | incomplete | disabled | busy | unavailable
active_slot=A active_version=1.0.0 active_state=committed
committed_slot=A
candidate_slot=B candidate_version=1.1.0 candidate_attempts=1 candidate_state=staged
rollback_slot=none rollback_version=none
can_rollback=0
last_result=commit:health-check
last_result_at=1800000000
last_rollback_reason=manual
health_evidence=healthy
health_slot=B health_version=1.1.0 health_attempt=1
health_at=1800000000 health_checks=root,state,marker,serving
health_matches=1
```

`state=incomplete` means the state file parsed but a slot directory or marker
it names is missing or mismatched. This is checked on every status query with
`Engine::verify_generation`, so a deleted slot cannot be displayed as ready.
`state=corrupt` means the record itself did not parse. `state=busy` is not a
state error: a writer owns the root and the caller may retry.

`UPDATE STATUS` is accepted bare (the current spelling), or with an explicit
schema: `UPDATE STATUS 1`, `2` and `3` are served. The reply always states
`schema=3` and echoes `requested_schema=<n>` when one was given. Schema 1 and 2
answers omit the health fields; schema 3 (and bare) include them. Schema 0 and
schemas above 3 are refused with `ERR schema-unsupported` rather than returning
fields the caller did not ask for. Other fields are additive, so an older
client that ignores unknown lines still works. The three mutating requests
require the literal `CONFIRM` word:

```text
UPDATE STAGE <version> <relative-path> CONFIRM
UPDATE COMMIT PASSED|FAILED CONFIRM
UPDATE ROLLBACK CONFIRM
```

`STAGE` accepts only a relative path below `<root>/incoming`. Absolute paths,
`.` and `..` are refused before the engine sees a source tree. `COMMIT` still
carries the caller's typed word, but that word is not evidence: `FAILED` is
refused before the engine is called, and `PASSED` only permits the service to
read the system-written record. Missing, stale, malformed, unattested or failed
evidence is a typed `ERR health-*` and leaves the state byte-for-byte unchanged.

## Authorization

The daemon uses the existing capability broker, not a second permission model.
A new `update.control` capability has three scopes:

| Scope | Operation |
|---|---|
| `update://stage` | copy a candidate into the inactive slot |
| `update://commit` | activate the staged candidate |
| `update://rollback` | return to the other committed generation |

The local session client receives all three as policy grants, as it does for
the other first-party actions. Each action is checked with live consent, so a
grant without `CONFIRM` is `needs-live-consent`. A record loaded without those
grants is `no-grant` (or `unknown-application` when the record is empty) and the
engine is never called. The socket is still `0600` and root-owned; there is no
per-peer identity yet, so this is authorization inside the one local client,
not a multi-user boundary.

## Failure behavior

| Condition | `UPDATE STATUS` | Mutating action |
|---|---|---|
| no root configured | `state=disabled` | `ERR missing-state` |
| root has no state file | `state=uninitialized` | `ERR missing-state` |
| state exists but does not parse | `state=corrupt` + reason | `ERR invalid-state` |
| a live writer owns the root | `state=busy` + lock fields | `ERR locked` (or non-zero `--boot`) |
| lock file cannot be created/read | `state=unavailable` + reason | `ERR lock-io` |
| state parses, slot/marker missing | `state=incomplete` + reason | operation-specific engine error |
| candidate staged, marker absent | `state=incomplete` | `commit` refuses `incomplete` |
| no pending candidate | `state=ready`, candidate `none` | `commit` refuses `no-pending` |
| candidate never booted | `health_evidence=absent` | `commit` refuses `health-not-booted` |
| no health record after a boot | `health_evidence=absent` | `commit` refuses `health-missing` |
| health record malformed | `health_evidence=invalid` + reason | `commit` refuses `health-invalid` |
| health record for another generation/attempt | health fields + `health_matches=0` | `commit` refuses `health-stale` |
| boot record not upgraded by the serve loop | `health_evidence=booting` | `commit` refuses `health-unattested` |
| running system recorded failure | `health_evidence=failed` | `commit` refuses `health-failed`; next boot rejects |
| no other committed generation | `can_rollback=0` | `rollback` refuses `no-rollback-target` |

No path panics. A status query is a question and must return a line even when
the answer is that the state cannot be trusted.

## Tests

`src/litric-sysd/src/update.rs` covers uninitialized and corrupt status, a
failed health signal, service commit/rollback with a `<root>/user` canary,
four boot requests exhausting the budget, a missing active slot, and source
path traversal. `src/litric-sysd/src/update_lock.rs` covers live contention,
dead-pid reclaim, the initialising grace and the bounded wait.
`src/litric-update/src/tests.rs` covers a leftover temporary, truncated, empty
and partial records; all are refused without a panic.

`src/litric-sysd/tests/update_service.rs` starts the real binary and talks over
the real socket:

* boot attempt 1 → status shows candidate B at 1 → `COMMIT PASSED` → active B →
  `ROLLBACK` → active A, user file unchanged;
* four `--boot` processes → candidate rejected, active A, `last_result=reject:boot-count`,
  user file unchanged;
* a header-only grant record → `ROLLBACK` is `ERR refused`, nothing moves;
* a damaged state file → `UPDATE STATUS` answers `state=corrupt`, and the
  daemon is still alive to report it;
* two concurrent `--boot` processes → both succeed and the candidate reaches
  attempt 2, proving the lock serialises the read-modify-write;
* a live PID's lock → `UPDATE STATUS 2` answers `state=busy` without waiting,
  and a second writer exits non-zero with the documented locked error;
* a dead-PID lock → the next boot reclaims it and counts attempt 1;
* a leftover temporary plus truncated and zero-length primary files → the
  primary record degrades to `corrupt` and boot still exits 0.

§77 adds real-binary/socket cases for:

* boot → serve `healthy` → commit B, with the `<root>/user` canary unchanged;
* no boot (`health-not-booted`) and a removed record with only a temp left
  (`health-missing`), both with state bytes unchanged;
* a wrong generation and a wrong attempt, each `health-stale` with state bytes
  unchanged;
* a system-written `failed` record → commit `health-failed`, then the next boot
  rejects the candidate with `last_result=reject:health-check`;
* zero-length, truncated and temp-only evidence → `invalid`/`absent`, typed
  commit refusal, no panic, state unchanged;
* replaying attempt 1's healthy record after boot attempt 2 → `health-stale`;
* a boot and a commit started together → exactly one serialised outcome, and
  status still answers with `state=ready`.

## What is still missing

* **No bootloader integration.** The state file is not read by systemd-boot,
  systemd-sysupdate, a UKI or the ESP. The service counts starts; it does not
  choose a kernel or partition.
* **No partition writer.** The engine copies directories; no EROFS image,
  dm-verity hash or signed boot entry is written.
* **No signature, hash or monotonic counter.** `STAGE` trusts the tree already
  placed under `incoming`. Verification belongs before that placement.
* **No kernel/network/display/session/storage/critical-service probes.**
  `healthy` means the boot entrypoint verified the candidate record and the
  serve loop reached its accept loop on the real socket; it does not mean the
  kernel came from this slot or that any service is healthy. Row 1147 (session,
  storage, critical services) stays `S`.
* **No power-cut test.** State writes are atomic and fsynced, but no power-cut
  matrix runs in CI.
* **No power-cut test.** The state record uses the ordering above and simulated
  interrupted writes are tested, but no machine is actually power-cycled and no
  EROFS/image stage is covered by that ordering.
* **No updater UI.** Status is reachable over IPC; no shell panel consumes it.
* **No download/update history UI.** `last_result` is the engine's bounded
  event tail, not a seven-day undo history.

## References

* [Update generations](2026-10-30-update-generations.md)
* [ADR-0021](../adr/0021-deep-recovery-data-protection.md)
* [whole-system plan §75, §76 and §77](../master-plan/28-whole-system-improvement-plan.md)
