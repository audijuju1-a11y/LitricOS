# 26 — Execution plan

**Created:** 2026-10-26, on top of commit `fd81ce6`.
**Derived from:** the phase controller pack (`00-MASTER-WORKFLOW.md` and
`01-PHASE-00-ARCHITECTURE.md` … `14-PHASE-13-PERF-MEMORY.md`, `15-APPENDIX-*`),
[the Beta 1.0 gate matrix](25-beta-1.0-gate-matrix.md), `docs/SYSTEM-SCOPE.md`,
`docs/GATES.md`, `docs/PERF-budget.md`, `docs/DESKTOP-2.0.md` and
`Saved/UNRESOLVED_ITEMS.md`.

This document is a **plan**, not evidence. Nothing in it may be quoted as a
result. Its only job is to say, in order, what has to be built and what would
prove it.

## 1. The rules this plan obeys

These are the pack's own rules, restated so a later session does not have to
re-derive them:

1. A phase is not done until its **EXIT GATE** passes with recorded numbers
   (`00-MASTER-WORKFLOW.md` §6). "Almost passes" is a fail.
2. Evidence has a level and the level is named: `STATIC`, `UNIT`, `CONTRACT`,
   `INTEGRATION`, `VM E2E`, `HARDWARE E2E` (gate matrix §2). A `UNIT` result is
   never reported as `VM E2E`.
3. `P0` before `P1` before `P2`. A `P1` may never be used to postpone a
   security or data-loss defect.
4. A capability that cannot be built has its **claim reduced and its limitation
   written down** — never replaced by a control that reports success.
5. Regressions block. A gate that passed before and fails now stops the next
   phase until it is fixed.
6. Client-facing surfaces (shell, files, browser) ship only what the backend
   can actually do, and say so in the UI.

## 2. Where the system actually is, `fd81ce6`

| Phase | Gate | State today | What the state is based on |
|---|---|---|---|
| P00 Architecture | GATE 00 | **PASS** | `docs/GATES.md` GATE 00 |
| P01 Boot | GATE 01 | booted 2026-08-28; **5 conditions `NOT RUN`, 1 `FAIL`** | GATE 01 entry 4; no image or QEMU on the current host |
| P02 Core | GATE 02 | partial: units, sandbox, cgroup slices; **no oomd policy, no zram tuning, no recovery target** | `image/mkosi.extra/usr/lib/systemd/system/`; the pack's P02 task list |
| P03 Storage | GATE 03 | erofs + dm-verity in the image; **no Btrfs policy, no LUKS2/TPM2, no recovery environment** | `tools/build.sh`, `image/mkosi.conf` (`cryptsetup` installed, unused) |
| P04 Network | GATE 04 | no provider code; no D-Bus client in any crate | gate matrix row; `src/` has no network crate |
| P05 Graphics | GATE 05 | software raster on a KMS scanout path with fbdev as fallback; **nothing measured on a real card** | `src/litric-shell/src/{drm,present}.rs`, `drmpresent.c`; ADR-0026 |
| P06 Desktop | GATE 06 | shell is far along (lock, browser, files UI, overview, settings); **no notifications daemon, no crash-restart proof, no soak test** | `docs/DESKTOP-2.0.md` §5; `src/litric-shell` |
| P07 Security + Defender | GATE 07 | `litric-defender` is an on-demand scanner binary; **no on-access daemon or hook**; capability grants are in memory | `src/litric-defender`, `src/litric-sysd/src/state.rs` |
| P08 Update | GATE 08 | **nothing** | no `update/` subtree, no sysupdate transfer |
| P09 AI | GATE 09 | nothing | — |
| P10 Apps | GATE 10 | `litric-manifest` parser; **no installer, no store, no SDK** | gate matrix row |
| P11 Hardware | GATE 11 | never attempted | requires physical hardware |
| P12 Gaming | GATE 12 | nothing | — |
| P13 Perf | GATE 13 | recorded **FAIL** (boot 9.17 s vs ≤ 1.2 s, image 489 MB vs ≤ 90 MB, 140 procs vs ≤ 32) | `docs/PERF-budget.md`, GATE 01 entry |

Two rows of the gate matrix are stale against the code and should be corrected
in the same commit as the next work on them: **login/lock/auth** (implemented:
`Verifier::UnixChkpwd`, `Shell::can_lock`, `AUTH VERIFY`) and **the browser**
(implemented: broker, LCP/1 over a socket, Chromium over CDP). Stale rows are the
drift the matrix itself warns about.

## 3. The critical path is a machine, not a feature

Every gate from 01 onward needs an image built with `mkosi` and booted in QEMU.
The current cloud host has neither, so **no gate can move** here, whatever else
gets written. That makes the first work stream a verification host, not a
feature — and it is the only stream that unblocks all the others.

## 4. Work streams

Each stream names the gate it unblocks, what proves it, and where it can run.
"Here" means this repository on a host without `mkosi`/QEMU/Chromium; "builder"
means a Linux host with `tools/provision-builder.sh` applied.

### W0 — Verification host (blocks everything)

- **Outcome:** one command builds the `desktop` image, boots it, runs the in-guest
  probes and archives the log; a second build produces the same UKI hash.
- **Gate:** GATE 01 conditions 2, 3, 5, 6, 8; then every later gate.
- **Acceptance:** `tools/build.sh --profile desktop` twice → identical
  `SOURCE_DATE_EPOCH` artifacts; `tools/boot-test.sh` runs unattended; the boot
  log lands under `evidence/` with a date; a deliberately injected panic makes
  the smoke test fail.
- **Runs:** builder only.

### W1 — Files privilege split (`litric-filesd`) — **DONE 2026-10-26, unit level**

- **Outcome:** the Files application stops writing files in-process. The shell
  talks to a daemon over a socket; the daemon runs as its own user with exactly
  the file root writable.
- **What landed:** `litric-filesd` (`--stdio` for `Accept=yes`, `--socket` for a
  standalone run, `--info`); `WRITETEXT` added to LFP/1; path fields carry an
  encoded form so a folder named *My Documents* can be addressed at all; the
  shell's `fsclient.rs` routes every `FsBridge` operation through it and refuses
  to fall back when the daemon is absent; the socket and service units moved out
  of `image/units-pending/` and the socket is enabled by `mkosi.postinst.chroot`;
  the shell unit no longer declares `litric-files` as a writable StateDirectory.
  Three protocol defects were found by driving it over a socket (double-encoded
  entry names, host paths in replies, `SEARCH` lines missing their `e ` prefix)
  and fixed with tests.
- **Still open:** no booted image and no QEMU on this host, so the socket
  activation, the sandbox and the uid split are reviewed but not exercised.
- **Why now:** `src/litric-files/src/server.rs` and `ops.rs` already exist and
  the unit is parked in `image/units-pending/`; the shell's direct `fsbridge`
  path is `Saved/UNRESOLVED_ITEMS.md` items 2 and 8, and the matrix's Files row
  says "no Files application window over the ops" while the window now exists.
- **Gate:** P03 storage API (`litric.fs`), P07 sandbox (one user, one path).
- **Acceptance:** create/rename/delete/trash over the socket; path traversal and
  symlink escape rejected at the daemon, not the client; `ReadWritePaths` names
  the file root only; the client cannot read outside it even if it asks; unit
  passes `ci/lint-unit-binaries.py` with its binary built by `tools/build.sh`.
- **Runs:** here (unit), builder (image + boot).

### W2 — Capability grant durability

- **Outcome:** grants, denials and audit survive a daemon restart; a corrupt
  store degrades to "no grants" rather than "all grants".
- **Gate:** P07 capability enforcement; `Saved/UNRESOLVED_ITEMS.md` item 9.
- **Acceptance:** schema + version, atomic write (the `prefstore` pattern already
  proven in `src/litric-shell/src/prefstore.rs`), retention and purge policy,
  corruption tolerated, revocation audited; restart test in the daemon's own
  suite.
- **Runs:** here.

### W3 — Notifications daemon

- **Outcome:** notifications have a source outside the shell process, survive a
  shell restart and can be snoozed; DND persists.
- **Gate:** P06 desktop gate item "full session … notification"; matrix B5.
- **Acceptance:** a notification raised while the shell is down is delivered when
  it starts; DND state survives restart; every action reachable by keyboard; the
  shell's in-process path is removed, not left as a second truth.
- **Runs:** here (unit), builder (journey).

### W4 — Desktop gate items (P06)

- **Outcome:** the six GATE 06 conditions become runnable and recorded: a full
  session journey, idle RAM breakdown per process, keyboard-only reachability,
  ten expert settings with revert, shell restart inside 2 s, 4-hour soak.
- **Gate:** GATE 06.
- **First slices here:** keyboard-only audit over the existing controls;
  expert-settings list with byte-identical revert; kill-and-restart harness.
- **Runs:** here for the audits, builder for the numbers.

### W5 — Update engine (P08)

- **Outcome:** signed A/B image updates with boot counting, health check,
  auto-rollback and an undo window; power-cut safe.
- **Gate:** GATE 08 — the power-cut test is 10 kills at 10 points, every one
  booting a working system.
- **Depends on:** W0 (to boot), W6 (Btrfs snapshot policy).
- **Acceptance:** tampered artifact refused before any write; downgrade refused
  by a monotonic counter; deliberately broken update rolls back three boots;
  apply+commit ≤ 60 s recorded.
- **Runs:** builder.

### W6 — Storage, encryption and recovery (P03)

- **Outcome:** Btrfs `@var`/`@home` policy, LUKS2 + TPM2 unseal, a recovery
  target reachable from the bootloader, verity root hash embedded by the build.
- **Gate:** GATE 03.
- **Acceptance:** tampered rootfs fails closed with a clear message; TPM seal
  breaks when the UKI changes; snapshot rollback timed; recovery boots from a
  broken slot.
- **Runs:** builder.

### W7 — Providers: network, Bluetooth, audio (P04)

- **Outcome:** a design note first (secret agent for Wi-Fi, portal grants, no
  D-Bus client inside the shell), then contract tests against a fake provider,
  then integration.
- **Gate:** GATE 04; unblocks desktop tiles and Control Centre.
- **Blocked on:** the design note. Writing provider code before it is how an
  unredacted passphrase ends up in a log.
- **Runs:** contract here, integration on builder.

### W8 — Graphics decision (P05)

- **Outcome:** a recorded architecture decision: stay on the framebuffer output
  for Beta, or move to DRM/KMS with `unsafe` ioctls confined to one audited
  module. The gate cannot pass on framebuffer-only, and the current
  `unsafe_code = "forbid"` in the shell rules the move out without a decision.
- **Decided 2026-10-27:** move to DRM/KMS. [ADR-0026](../adr/0026-kms-output-path.md)
  confines every ioctl to `drmpresent.c`, keeps the shell in safe Rust, presents
  damage bands with a hardware page flip, and states the output path in the
  status file. The **raster is unchanged**: this moves the present cost, not
  the draw cost, and BUG-0015 is still open.
- **Gate:** GATE 05.
- **Acceptance:** decision recorded in an ADR with its cost; until then the
  product statement says single-surface, no VRR, no multi-monitor mode setting.
  The decision is recorded; the gate is **not** passed, because the KMS path has
  never run on a card. The next step is a desktop image on hardware, reading
  `present=drm` and `drm.mode` from `litric-shell.status`.

### W9 — Performance budget (P02/P13)

- **Outcome:** zram, MGLRU, oomd policy and a measurement harness that fails on a
  planted 15 % regression.
- **Gate:** GATE 02 and GATE 13; the recorded `FAIL` is the current truth.
- **Acceptance:** `docs/PERF-budget.md` numbers move from `TARGET` to `MEASURED`
  with tool, command, environment and n; boot and idle numbers per profile.
- **Runs:** builder.

### W10 — Apps, installer, AI, gaming (P10/P09/P12)

- **Outcome:** after the P0 gates. Deliberately last: an installer that wipes a
  disk before updates are safe is the most expensive mistake available.
- **Gate:** GATE 10, 09, 12.

## 5. Order of work

| Sprint | Contents | Exit |
|---|---|---|
| 1 | W0 (builder + boot log), W1 (files daemon), matrix truth for lock/browser rows | a booted desktop image with a files daemon, and `ci/verify.sh` green |
| 2 | W2 (grants), W3 (notifications) | grants and notifications survive a restart, in the daemon suites |
| 3 | W4 (desktop gate items), W9 (measurement harness) | GATE 06 conditions runnable; the perf harness fails on a planted regression |
| 4 | W6 (storage), then W5 (update) | GATE 03, then GATE 08 power-cut |
| 5 | W7 (providers), W8 (graphics decision) | GATE 04 contract, ADR recorded |
| 6 | W10 | GATE 09/10/12, only after 01–08 are green |

## 6. Scope guards

- No new shell surface while W1–W3 are open: the shell already shows more than
  the backends can back.
- No claim of a booted, measured or working system without a log under
  `evidence/` and an entry in `docs/GATES.md`.
- No `--no-sandbox`, no TLS verification off, no widening `ReadWritePaths` to
  make a write succeed, no `STATUS: DESIGN-ONLY` removed before the code exists.
- No second source of truth: a matrix row is corrected in the commit that
  changes the fact, not in a later documentation pass.

## 7. What "done" means for this plan

The Beta readiness label in the gate matrix may change from `NOT_VERIFIED` only
when GATE 01 through GATE 13 each have an entry with the evidence level the row
requires. Until then the honest label stays `NOT_VERIFIED`, and this plan is a
list of the work between here and there.
