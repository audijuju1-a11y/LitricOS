# ADR-0021 — Platform authority, deep recovery and data protection

- **Status:** Accepted (design). Parts implemented: host-authority invariants
  and environment health/containment are tested in `litric-env`; the boot chain
  (UKI + dm-verity, fail-closed) is demonstrated in GATE 01.
- **Date:** 2026-09-14
- **Extends:** `master-plan/04-boot-integrity-recovery.md`,
  `master-plan/06-storage-data-lifecycle.md`, ADR-0015 §"What LitricOS keeps
  owning while a guest is in front"
- **Companion:** ADR-0020 (unified compatibility — the runtimes this recovery
  story must outlive)

---

## Requirement

LitricOS is the primary operating system platform: the trust anchor and
management layer of the machine even while the user is working inside a
Windows or Linux environment. Recovery must be deep — down to "all three
environments are broken" — and must prioritize the user's data above the
operating system. Windows being compromised, Linux being compromised, even
LitricOS itself being corrupted must each be recoverable **independently**,
without destroying the others or the user's files.

## The decision

### 1. Exactly one owner

Boot configuration, storage policy, resource budgets, security policy and
recovery belong to LitricOS at all times. Guests and compatibility runtimes
request changes through controlled interfaces; none has direct write access to
boot metadata (row 886) or platform keys. `litric-env` already refuses, in
tested code, the one transition that would surrender this: the host cannot be
suspended, hibernated or stopped — because a control plane that can be put to
sleep by a guest is not a control plane.

Two identities the architecture must keep distinct:

```text
Environment failure   ≠   Platform failure
One environment       ⇏   compromise of another
```

### 2. Four things that must never be confused

| Concept | What it is | Lifecycle owner |
|---|---|---|
| **Live data** | the files and state in active use | user and apps |
| **Snapshot** | point-in-time view of data for quick restore | recovery-managed, bounded, **writable-storage-dependent** |
| **Backup** | independent, immutable, verified copy — local or external | backup policy; must survive total storage loss |
| **Generation** | immutable, bootable system state (OS, not data) | update system (A/B, boot counting) |

A snapshot is not a backup (ransomware can reach writable snapshots); a
generation is not a snapshot (it is OS-scoped); user data has its own lifecycle
separate from all of them. **System rollback never rolls back user data** —
files created after a generation was built must survive that generation's
rollback. This invariant is stated in master-plan/06 and is restated here as
the load-bearing one.

### 3. Generations and activation

System updates produce immutable generations (systemd-sysupdate A/B, locked
baseline). A generation activates only after health checks; boot counting rolls
back automatically after repeated failed boots (rows 5, 47). Every generation
switch records slot, reason and user-data-integrity result so a later "why did
I lose my morning" question has an answer.

### 4. User data is a first-class domain

User data is an independent data domain with its own permissions, encryption,
integrity, versioning and recovery metadata — not a folder inside an OS
(master-plan/06 namespaces: `user://`, `app://`, `config://`, `secure://` …).
Cross-environment access follows ADR-0020's matrix. Encryption is
hardware-rooted where available (LUKS2 + Argon2id, TPM2-sealed, locked
baseline) with **per-environment key scoping**: a Windows or Linux environment
never holds Litric platform keys, and the compromise of one environment cannot
decrypt another's secrets.

### 5. Failure containment ladder

| Failure | Contained by | Recovery action |
|---|---|---|
| App crash | app process boundary; sandbox | app dies, nothing else |
| Runtime failure | runtime restart, capability re-check | runtime restarts, apps resume confined |
| Environment failure | guest lifecycle policy (`litric-env`: failed is **reset, never resumed**); budgets | environment reset from trusted state |
| Platform (Litric core) failure | verity fail-closed, generation rollback | reboot to recovery, restore trusted generation |
| Hardware failure | nothing architectural can help | external/offline backup restore |

Each rung must have a mechanism and a test before the rung above it can be
claimed. Mechanisms that exist and are tested today: shell-module crash
isolation by design (ADR-0009 mitigations), environment fallback to host
(`litric-env` tests), fail-closed verity (GATE 01 evidence).

### 6. Health states

The environment health vocabulary is deliberately the five-value set in
`litric-env` — Healthy, Degraded, Unresponsive, Compromised, Corrupted —
because it answers two orthogonal questions (is it alive? is it trustworthy?)
without derived states. RECOVERING / RECOVERED are **transitions**, not health
values; UNAVAILABLE is a **lifecycle** state (NotInstalled/Stopped), not a
health value. Adding derived states to the enum would duplicate information
that can drift; this mapping documents how the 8-state vocabulary from the
requirement maps onto what is implemented.

### 7. Recovery environment and priority

Recovery boots as a separate signed entry, independent of the OS being
repaired, with only what it needs: storage and filesystem drivers, integrity
verification, generation management, backup/restore, restricted network
support (explicit user confirmation), authentication, minimal UI, terminal.
It never offers "disable verification and continue" as an ordinary path
(master-plan/04).

Priority order, fixed:

1. Preserve user data
2. Preserve encryption identity and keys
3. Repair existing system state
4. Roll back to the previous trusted generation
5. Restore a trusted snapshot
6. Reinstall from verified image
7. Destructive recovery — last resort, explicit, multi-confirmed

Repair / rollback / reset / reinstall are distinct operations with distinct
data consequences, and the UI states which data each touches before running.
The recovery decision engine diagnoses before it chooses: corrupted executable
→ repair; unstable latest generation → rollback; environment malware → isolate
+ reset; all generations invalid → verified reinstall; physical storage loss →
external backup restore.

### 8. Watchdogs and assistance

Every environment and critical service has a heartbeat and an owner of its
recovery: per-environment watchdog (future `litric-envd`), systemd supervision
for services, shell self-restart (ADR-0009 §5), and a recovery watchdog inside
the recovery environment itself. AI may *explain* recovery state and recommend
actions; it is permission-checked and can never execute or bypass a security
control (row 1067).

## What is honestly not true yet

Generations tooling, snapshot management, the recovery environment, watchdogs
and the recovery decision engine are **specified, not built** (Part K rows
record them as `S`). What exists today is the boot chain that makes generation
rollback possible, the environment lifecycle policy that contains guest
failure, and the storage invariants that protect user data. This ADR is the
design contract those implementations are built against.

## Revisit if

- Btrfs snapshot + sysupdate generation tooling proves unable to express the
  four-concept separation cleanly — revisit the storage layout (ADR-0004
  territory), not the separation.
- Boot counting shows false rollbacks in the field (healthy system rolled
  back) — revisit the health-check gate, never the rollback safety itself.
- A credible adversarial scenario reaches platform keys through the recovery
  environment — the recovery environment's own attack surface gets a dedicated
  threat-model update before anything else changes.
