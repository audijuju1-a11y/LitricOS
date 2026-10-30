# Update generations: A/B state, boot counting and rollback

Status: the core library landed in §74 and is now driven by `litric-sysd` in
§75. It is not wired to a bootloader or an image. This note records what the
engine guarantees, what it deliberately does not, and how a temp-root test
proves each guarantee. The service boundary is in
[2026-10-30-update-service.md](2026-10-30-update-service.md).

## Why this exists

ADR-0021 separates four things that are easy to conflate: live data, snapshots,
backups and **generations**. A generation is an immutable, bootable system tree.
Until this batch the repository had the boot chain that makes a generation
possible, but no code that owned the A/B pointer, staged a candidate, counted
failed boots or rolled back. That left `SYSTEM-SCOPE` rows 760, 762 and 1145 at
`S`, and the parity ledger had to defer every update and recovery surface
because there was no backend behind it.

The implementation is `src/litric-update`, a zero-third-party-dependency crate
beside `litric-core`. The README's "smallest coherent change" rule is why it is
not inside `litric-sysd`: updates have a different privilege boundary (a
system-wide root, not the desktop socket) and a different caller (an installer,
a boot service or a recovery environment), and neither should be able to reach
the other's state by accident.

## The two-slot model

```text
<root>/system/
  state                    one atomic record
  slots/A/generation       marker: protocol + version
  slots/B/generation
```

`state` is a bounded, tab-separated record:

```text
litric-generations	1
active	B
pending	A
slot	A	1.2.0	staged	2
slot	B	1.1.0	committed	0
event	1800000000	commit	B	A	health-check	untouched
```

The parser is strict. A damaged record is an error, not a guess. This is the
opposite of the lockout meter's fail-open rule, on purpose: refusing every
login because a lockout file is damaged would lock the only administrator out,
while advertising a slot whose state is unknown could boot a tree that was
never verified. A recovery environment can still choose a known slot; this
engine refuses to invent one.

Bounds:

| Bound | Value | Why |
|---|---:|---|
| State file | 16 KiB | two slots and eight events need far less |
| State line | 512 bytes | one version and six short fields |
| Switch history | 8 events | enough to answer "what changed recently", bounded on every write |
| Boot attempts | 3 | three failed boots discard the candidate on the fourth request |
| Staging entries | 100,000 | finite inode budget; overrideable for a known target |
| Staging bytes | 8 GiB | finite disk budget; overrideable for a known target |
| Relative path | 4,096 bytes | refuses a tree that cannot be addressed reliably |

## The state machine

| Operation | Preconditions | Effect on disk | Active pointer |
|---|---|---|---|
| `init(root, version)` | no `state`, no `A`/`B` directory | writes A marker and a committed-A record | A |
| `stage(source, version)` | no pending candidate; `version > active` | empties the inactive slot, copies a closed tree to a hidden staging directory, writes and syncs its marker, renames it into place, records it staged with attempt 0 | unchanged |
| `note_boot(now)` | a candidate is staged | increments its attempt count; on the fourth request, or a missing/mismatched marker, marks it bad and clears pending | unchanged |
| `commit(now)` | a staged candidate whose marker matches the state | marks it committed, makes it active, records a `commit` event, leaves the old active committed | candidate |
| `discard_pending(now)` | a staged candidate | marks it bad, clears pending, records a `reject` event | unchanged |
| `rollback(reason, now)` | no pending; the other slot is committed and its marker matches | marks the old active bad, makes the other committed slot active, records a `rollback` event | other slot |

`commit` does not run health checks itself. It is the seam after the caller has
checked the graphical session, storage mount and critical services. This is
deliberate: those checks are `litric-sysd`/boot-service work, and a library that
pretended to run them would be the "backend faked to justify a control" defect
the plan forbids.

A rollback marks the generation it leaves **bad**. The reason is not cosmetic:
if it remained committed, the next rollback could select it again. A manual
rollback is an operator's assertion that the generation should not be used; a
later forward update stages a freshly verified candidate.

## Crash ordering

The order in `stage` is the crash story:

1. Persist the inactive slot as empty. An old generation is never advertised
   while its directory is being replaced.
2. Remove the old directory and copy into a hidden staging directory on the
   same filesystem.
3. Write and sync the marker, then rename the staging directory into the slot.
4. Persist the candidate and its boot budget.

A crash between any two steps leaves either the active generation untouched, or
an unadvertised staging directory that the next stage removes. The active
pointer never moves before `commit` has re-read and matched the marker.

The state write is the same atomic sequence used by the grant and lockout
stores: a temporary file in the same directory, `fsync`, `rename`, directory
`fsync`. The file is created `0600`; its parent is `0700`. A failed write leaves
the previous record in force, and every mutating operation restores its
in-memory snapshot before returning the error.

## Anti-downgrade

`stage` refuses a version that is not strictly newer than the active version.
This uses `litric_core::Version`, whose ordering puts a pre-release below its
release and rejects ambiguous forms such as `01.2.3`. The refusal happens before
the state file or a slot is touched, so a rejected downgrade leaves no trace.

This is only the local half of anti-downgrade. A monotonic counter signed by the
update metadata is still required to stop an attacker replaying an older but
validly signed image; that belongs to the download/verification service, not to
this mechanism.

## User data

The engine writes only `<root>/system`. It never writes, snapshots or restores
`<root>/user`, a separate data domain in ADR-0021. The lifecycle test creates a
user file, commits a second generation, rolls back and asserts the file's bytes
are unchanged. Every switch event records `untouched`, which is the truthful
result for this engine rather than a verification it did not perform.

## What the tests establish

The crate has 19 tests. The lifecycle tests use real directories under the
system temp directory, a real state file and real markers; no mocked
filesystem.

| Test | What it pins |
|---|---|
| `a_candidate_is_staged_copied_and_committed_without_touching_the_active_slot` | copy, marker, pending state, health commit, reopen, previous generation retained |
| `three_failed_boots_discard_the_candidate_and_keep_the_active_generation` | exact three-attempt budget, active stays A, candidate marked bad |
| `rollback_switches_to_the_previous_committed_slot_and_leaves_user_data_alone` | rollback record, slot states and the user-data canary |
| `a_downgrade_is_refused_before_the_state_or_a_slot_is_written` | strict `>` and byte-identical state after refusal |
| `a_second_stage_requires_the_first_candidate_to_finish` | one pending candidate; explicit discard |
| `the_boot_budget_is_a_boundary_not_an_accumulator` | fourth boot request rejects and the next request reports active |
| `a_symlink_in_the_source_is_refused_and_the_active_slot_stays` | closed-tree rule and no published candidate |
| `a_missing_marker_is_never_advertised` | commit refuses a slot without its proof |
| `a_corrupt_state_is_refused_and_left_byte_for_byte` | fail-closed read with no repair-in-place |
| `limits_stop_a_candidate_before_it_is_published` | entry and byte caps, no slot published |
| `init_refuses_to_replace_an_existing_state` | no accidental reinitialisation |
| `the_state_file_is_owner_only` | `0600` on Unix |
| `the_switch_history_stays_bounded_across_many_updates` | eight-event cap across ten updates |
| `rollback_without_a_second_generation_is_refused` | no invented target |
| state parser tests (5) | round trip, malformed records, attempt bound, event bound, oversized line |

## What is not true yet

* **No bootloader integration.** This crate writes a state file; `systemd-boot`,
  `systemd-sysupdate`, the UKI and the ESP still have to read it or be configured
  from it. No image was built or booted in this batch.
* **No health checks.** The caller must perform them; the engine only gates the
  commit on a marker match.
* **No signature or hash verification.** The caller must verify the source tree
  before staging. The marker proves what the engine copied, not who built it.
* **No partition writer.** It copies a directory tree into a slot directory; it
  does not write an EROFS image, a verity hash or a boot entry.
* **No power-cut test.** The ordering and fsyncs are designed for it, but CI
  cannot cut power mid-rename. The state parser and atomic-write tests are the
  executable part; a QEMU power-cut matrix remains `NOT RUN`.
* **No updater UI.** `WA.10.05`, `WB.11.*` and `WB.01.12.*` remain deferred; the
  service now exposes the state, but no shell panel consumes it.
* **No cross-process lock.** The normal path is serialised by systemd ordering
  (`--boot` before `--serve`); two update services pointed at one root are not
  prevented by a file lock.

## Next step

The boot entrypoint and `UPDATE STATUS` now exist in `litric-sysd`; see the
service note. The next consumer is the shell, which can say "update staged /
next boot / rollback available" from `UPDATE STATUS` without inventing state.
The settings page is only worth drawing after that consumer exists.

## References

* [ADR-0021](../adr/0021-deep-recovery-data-protection.md)
* [master-plan/23 — deep recovery](../master-plan/23-deep-recovery.md)
* [master-plan/13 — supply chain and updates](../master-plan/13-supply-chain-updates.md)
* [whole-system plan §6, batches 74–75](../master-plan/28-whole-system-improvement-plan.md)
* [The update service](2026-10-30-update-service.md)
