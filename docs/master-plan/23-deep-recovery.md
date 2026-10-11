# 23 — Deep recovery and data protection

## Requirement

Recovery that survives the loss of every operating environment, prioritises
user data above the operating system, and keeps LitricOS the platform
authority even while users work inside Windows or Linux environments.
[ADR-0021](../adr/0021-deep-recovery-data-protection.md) is the decision
record; this document is the implementable contract.

## The four concepts (never conflated)

| Concept | Owner | Survives |
|---|---|---|
| Live data | user and apps | nothing by itself |
| Snapshot | recovery policy, bounded | storage failure: **no** |
| Backup | backup policy, immutable, verified | storage failure: **yes** |
| Generation (OS) | update system, A/B + boot counting | n/a — rolled back, not preserved |

**System rollback never rolls back user data.** Every recovery operation
states, before running, which of the four it touches.

## Storage model (logical)

```text
ESP + signed UKI          restricted
system slot A/B           EROFS + dm-verity, immutable
Litric platform state     owner: Litric only
Windows environment       guest disk, host-owned
Linux environment         guest disk, host-owned
recovery environment      separate signed entry
user data                 LUKS2-protected Btrfs, independent domain
snapshots                 bounded, recovery-managed
backup metadata           integrity-verified, restore-tested
secure metadata           platform keys, never guest-readable
```

Physical layout may vary; the logical separation and the boundaries may not.

## Recovery environment

A separate signed boot entry, independent of the OS being repaired: storage
and filesystem drivers, integrity verification, generation management,
backup/restore, restricted network support (explicit confirmation),
authentication, minimal UI, terminal. It never offers "disable verification
and continue" as an ordinary path (master-plan/04).

## Recovery priority (fixed order)

1. Preserve user data
2. Preserve encryption identity and keys
3. Repair existing system state
4. Roll back to the previous trusted generation
5. Restore a trusted snapshot
6. Reinstall from verified image
7. Destructive recovery — explicit, multi-confirmed, last resort

## Recovery decision engine

Diagnose before choosing:

| Observation | Action |
|---|---|
| Corrupted system executable | repair |
| Latest generation unstable | rollback |
| Environment malware (Defender signal) | isolate + reset environment |
| System state severely corrupted | restore snapshot |
| All generations invalid | verified reinstall |
| Disk metadata corrupted | storage recovery |
| Physical disk failure | external/offline backup restore |

Every operation is transactional, records slot/generation/reason and a
user-data-integrity result, and is auditable afterwards.

## Health, watchdogs, containment

Health vocabulary is `litric-env`'s five values (Healthy, Degraded,
Unresponsive, Compromised, Corrupted); RECOVERING/RECOVERED are transitions,
UNAVAILABLE is lifecycle state. Watchdog ownership: per-environment heartbeat
(future `litric-envd`), systemd supervision for services, shell self-restart
(ADR-0009), recovery watchdog inside the recovery environment. Failure
domains nest: app → runtime → environment → platform → hardware; a failure is
contained at its domain and never propagates upward uncontrolled.

AI assists by *explaining* state and recommending actions under the same
permission system as everything else; it never executes recovery or bypasses
a control (row 1067).

## Functional work

1. systemd-sysupdate A/B generations + boot counting + auto-rollback
   (rows 5, 47) with health confirmation only after graphical session,
   storage mount and critical services succeed (master-plan/04).
2. Btrfs snapshot policy: pre-update automatic, bounded, retention-tested.
3. Backup: immutable local + external/offline targets, integrity
   verification, **restore testing** (a backup that was never restored in a
   test is a hypothesis, not a backup).
4. Recovery environment image and entry.
5. Watchdog heartbeats and containment actions per domain.
6. Recovery UI: diagnose → present → confirm → execute → verify, with the
   priority order above structurally enforced (later options require the
   earlier ones to be exhausted or explicitly declined).

## Failure tests

Power loss before/after each update commit; full ESP; corrupt verity block;
wrong/revoked signing key; failed data unlock; malformed boot count; failed
health check; boot loop; rollback after kernel regression; interrupted
snapshot; restore collision (never silently overwrite newer files);
ransomware against snapshots (must not defeat backups); disk full during
every transaction; recovery environment itself failing to boot (second
fallback: removable recovery media).
