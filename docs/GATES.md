# LITRIC — Gate Results Log

Append-only. Every entry carries date, the exact conditions, and raw numbers.
A gate that "almost passes" does not pass (MASTER §6).

Legend: `PASS` · `PASS_WITH_LIMITATIONS` · `BLOCKED` · `REJECTED` · `NOT RUN`

---

## GATE 00 — Architecture & Threat Model
- **Date:** 2026-08-26
- **Result:** PASS
- **Re-affirmed:** 2026-08-26 (v2 audit) — still PASS; superseded in part by the
  v2 ADRs 0009–0013, which are additive, not contradictory.

### Conditions
- [x] All 7 documents exist and are internally consistent
- [x] Threat model covers every trust boundary (A–F + AI-specific)
- [x] Every capability in SEC-01 maps to at least one API in API-00
- [x] Every baseline decision in MASTER §3 has an ADR with "Revisit-if"
- [x] Risk register contains ≥6 honest limitations (14 recorded)

### Notes
Documents are design artifacts; Phase 00 produces no code, so "no execution
evidence" is expected here rather than a defect.

---

## GATE 01 — Bootable Prototype

### Entry 1 — 2026-08-26 — **REVOKED**
- **Originally recorded as:** `PASS_WITH_LIMITATIONS`
- **Corrected result:** **BLOCKED**
- **Reason for revocation:** Zero of the seven GATE 01 conditions were met. The
  sub-condition checklist in the original entry was honest (every line read
  `BLOCKED` or `NOT RUN`); the **verdict line was not**. Under PROTOCOL §45,
  missing critical evidence is `BLOCKED`, and §45 further states: *"Never convert
  BLOCKED into PASS by changing wording."* Recording a gate as passed on the basis
  of *files existing* rather than *behaviour observed* is the exact failure LAW 1
  and MASTER §2 G3 exist to prevent.
- **Additional defects found in audit:**
  - `image/mkosi.conf` used a configuration schema that does not exist in mkosi
    and would fail at parse time — [BUG-0001](bugs/BUG-0001-fabricated-mkosi-config.md), S2.
  - `tests/smoke_boot.py`, listed as a completed deliverable with "4 test cases",
    was truncated mid-statement and could not be imported — plus three defects
    that would have made it pass while testing nothing —
    [BUG-0002](bugs/BUG-0002-smoke-test-never-executed.md), S2.
  - Both share one root cause: **an artifact was recorded as complete because it
    existed, not because it ran.** `ci/verify.sh` is the countermeasure.

### Entry 2 — 2026-08-26 (v2) — **BLOCKED (build environment), tooling now complete**
- **Result:** BLOCKED
- **Blocking reason:** No Linux build host. Verified on the development machine:
  `wsl.exe --status` → *"The Windows Subsystem for Linux is not installed."*;
  `qemu-system-x86_64`, `docker`, `podman`, `make`, `gcc` all absent from PATH.
  A Linux-based OS image cannot be produced from this host as configured.

- **Unblock tooling: written and syntax-verified, NOT YET EXECUTED.**

  | Step | Artifact | State |
  |---|---|---|
  | 1 | `tools/bootstrap-windows.ps1` | preflight **executed** on this machine and passing; the install path is **NOT RUN** (needs Administrator + reboot) |
  | 2 | `tools/provision-builder.sh` | syntax verified; **NOT RUN** (needs a Linux host) |
  | 3 | `tools/build.sh` | syntax verified; guard rails **executed** and correct; the build itself is **NOT RUN** |
  | 4 | `tests/smoke_boot.py` | syntax verified; **NOT RUN** |

- **Host facts established by running the preflight** (not assumed):
  - Windows build 26200 — WSL2 capable ✓
  - Hardware virtualization present ✓
  - Free space 16.5 GB — **enough for the `nano` and `core` profiles (~10 GB),
    not enough for `desktop` or `gaming` (~25 GB)**
- **Consequence:** the achievable next target is `--profile nano`, which is also
  exactly what GATE 01 asks for. `desktop` needs disk space freed first, and
  `tools/build.sh` refuses to build it for a second reason as well (see below).

- **Second, independent blocker for `desktop`:** the systemd units in
  `image/mkosi.extra` reference `/usr/bin/litric-shell`, `/usr/bin/litric-capd`
  and `/usr/bin/litric-defender`. **Those binaries do not exist.** `src/` contains
  libraries only. `tools/build.sh` refuses `--profile desktop` rather than
  producing an image that boots to a failed unit and a black screen, which would
  look like progress and be worse than nothing.

- **Commands, in order, once a build host exists:**
  ```
  powershell -ExecutionPolicy Bypass -File toolsootstrap-windows.ps1   # Administrator
  wsl -d archlinux
  cd ~/litricos
  sudo bash tools/provision-builder.sh
  ./tools/build.sh --profile nano
  ./tools/run-qemu.sh
  python3 tests/smoke_boot.py
  ```

### Entry 3 — 2026-08-27 — **LitricOS BOOTS**

The build host was created, the image was built, and it booted. This is the
first entry in this file backed by an executed boot rather than by files
existing.

```
Arch Linux 7.1.9-arch1-2 (ttyS0)
litricos login:
```

Raw serial log: `docs/evidence/gate01-boot.log` (1151 lines).
Artifact hashes: `docs/evidence/gate01-artifacts.sha256`.

**Result:** `PASS_WITH_LIMITATIONS` — see the conditions table and the open
defects below. It boots; it is not yet within budget and eight units fail.

#### What was measured

| Metric | Value | Target | Verdict |
|---|---:|---:|---|
| Boots to a login prompt, unattended | **yes** | yes | **PASS** |
| Boot time (last kernel timestamp) | **9.17 s** | ≤ 1.2 s (`nano`) | **FAIL** |
| Systemd timeouts during boot | **0** | 0 | **PASS** |
| Failed units | **8** | 0 | **FAIL** |
| Compressed image | 489 MB | ≤ 90 MB (`nano` ISO) | **FAIL** |
| UKI size | 218 MB | ≤ 18 MB initrd | **FAIL** |

Environment: QEMU 11.1.0, KVM, 2 vCPU, 1024 MB, OVMF 4M, WSL2 Arch on
Windows 11 build 26200. **VM numbers, not metal.** n=1, so these are
observations, not measurements under the Appendix §3 protocol (which requires
n≥5 with mean and stddev). They are recorded as first-boot observations and must
not be quoted as LitricOS performance figures.

#### Boot-time finding

The first successful boot took **98 s**. Nearly all of it was a single
90-second systemd timeout waiting for `/dev/zram0`, caused by
`zram-generator.conf` being applied to the `nano` profile — which is defined as
the memory floor and should never have had zram. Correcting that scope error
took the boot to **9.17 s**, a 10× improvement from one configuration fix and
zero optimisation work.

#### Defects found and fixed to get here

Every one of these was found by running the thing, and none would have been
found by reading it.

| # | Defect | Consequence |
|---|---|---|
| 1 | All shell/config files had CRLF line endings | `set -euo pipefail
` → "invalid option name". Git Bash **hides this**; only `file` or WSL shows it. Affected the kernel config, where CRLF would silently void every symbol. |
| 2 | `pacman` keyring uninitialised on fresh Arch WSL | No package could install |
| 3 | `mkosi` installed from PyPI, which does not carry it | Provisioning failed; the distro package is newer *and* signed |
| 4 | `CacheDirectory` in `[Output]` | mkosi rejects it — and **my linter had it in the wrong section too**, so the linter passed a config mkosi refused |
| 5 | No PCR signing key | `SignExpectedPcr` needs a *third* key, separate from Secure Boot and verity |
| 6 | `UnifiedKernelImages=yes` | Not a boolean. Coerced to `signed`, meaning "use a prebuilt UKI", which Arch does not ship. Correct value: `unsigned` |
| 7 | ESP copied only `/boot` | mkosi splits `/efi` (bootloader) from `/boot` (kernels). The ESP got a UKI and no bootloader; firmware fell through to PXE |
| 8 | **`tools/build.sh` reported success when mkosi did nothing** | mkosi skipped on existing output; the script printed stale artifacts as if freshly built. Fixed with `--force` plus explicit skip detection |
| 9 | `console=ttyS0 console=tty0` | Last `console=` becomes `/dev/console`. Serial went silent the moment userspace started — indistinguishable from a hang under `-nographic` |
| 10 | `KernelModulesInitrdInclude` matched zero modules | `dm_verity` vs the real `dm-verity`. Initrd had no modules; verity could not assemble; 90 s timeout → emergency shell |
| 11 | `rw` on a dm-verity root | Verity roots are read-only by construction |
| 12 | zram in the `nano` profile | 90 s of a 98 s boot |

#### Open defects — 8 units still fail

Root cause is a single cascade: **`boot.mount` (ESP automount) fails**, which
takes `dbus-broker` with it, which takes `systemd-logind` and `systemd-homed`
with it. The login prompt is reached regardless because `agetty` does not
require logind.

```
mount EFI System Partition Automount     <- root cause
start D-Bus System Message Bus           <- cascade
start User Login Management              <- cascade
start Home Area Manager                  <- cascade
start Network Time Synchronization
start Enable Persistent Storage in systemd-networkd
start Automatic Boot Loader Update
start Update Boot Loader Random Seed
```

This is Phase 02 work and is tracked as the next task. It does not block the
GATE 01 condition, which is "boots to a shell, unattended".

### Entry 4 — 2026-08-28 — **zero failed units**

Eight failed units traced to root causes and fixed. Evidence:
`docs/evidence/gate01-boot-clean.log`.

```
litric.units.failed=0
litric.verity.active=yes          litric.verity.targets=1
litric.root.fstype=erofs          litric.root.readonly=yes
litric.root.source=/dev/mapper/root
litric.lsm.active=capability,landlock,lockdown,yama,safesetid,bpf
litric.kernel.mglru=0x0007        litric.kernel.sched_ext=present
litric.mem.total_mb=946           litric.mem.used_mb=634
litric.proc.count=140
```

#### What was actually wrong

| # | Root cause | Symptom it produced |
|---|---|---|
| 1 | No writable `/var` — the image is a single read-only erofs root | `logind` and `homed`: "Failed to set up special execution directory in /var/lib: Read-only file system". Looked like a login-manager bug. |
| 2 | `fsck.vfat` absent (no `dosfstools`) | `efi.mount` had an unsatisfiable fsck dependency → Local File Systems failed → **emergency shell** |
| 3 | ESP formatted with 4096-byte sectors; QEMU virtio-blk presents 512 | Mount failed before the FAT driver ran, so there was no `FAT-fs` error to find |
| 4 | ESP as `x-systemd.automount` | An autofs mountpoint breaks mount-namespaced services: `dbus-broker: Failed to set up mount namespacing: /efi: No such device` |
| 5 | **`loadpin` pinned to the initrd tmpfs, not the verity root** | Denied `vfat`, `tcp_bbr`, `tun` and every other post-switch-root module. Presented as `unknown filesystem type 'vfat'`. See [BUG-0004](bugs/BUG-0004-loadpin-denied-every-module.md), **S1**. |

Cause 5 was introduced by the fix for BUG-0003, on a premise that was stated in a
code comment and never checked against LoadPin's actual behaviour.

#### The tool that found them

None of these were findable by reading. All five were found within one session of
`litric-selftest` existing — an in-guest reporter that prints failed units, their
reasons, and each failed unit's full journal to the serial console. The serial
log is the only channel out of a headless VM, so the report is greppable rather
than pretty.

Before it existed, every diagnosis was a guess from the outside.

#### Measurements — VM, n=1, first-boot observations

Not measurements under the Appendix §3 protocol (n≥5, mean and stddev). They must
not be quoted as LitricOS performance figures.

| Metric | Value | Note |
|---|---:|---|
| Failed units | **0** | was 8 |
| dm-verity active | **yes** | root is `/dev/mapper/root`, read-only |
| LSMs live | **5 + capability** | landlock, lockdown, yama, safesetid, bpf |
| MGLRU | **0x0007** | all generations enabled |
| Processes at idle | 140 | budget target ≤ 32 — **over**, and unmeasured under load |
| Memory used | 634 MB / 946 MB | VM has only 946 MB and `/var` is tmpfs; not comparable to the ≤55 MB `nano` budget until measured on a properly sized VM |

`integrity` is listed in `CONFIG_LSM` but is **not** in `/sys/kernel/security/lsm`.
It is inert rather than harmful, and is recorded as residual risk in BUG-0004.

### Conditions (current status)

| # | Condition | Status | Evidence |
|---|---|---|---|
| 1 | Boots to a login prompt in QEMU, unattended | **PASS** | `docs/evidence/gate01-boot.log` |
| 2 | Image build reproducible (2 builds → identical hash) | NOT RUN | needs a second clean build with the same `SOURCE_DATE_EPOCH` |
| 3 | Smoke test passes, and fails on a corrupted image | NOT RUN | `tests/smoke_boot.py` still never executed |
| 4 | Boot time within budget | **FAIL** | 9.17 s vs 1.2 s target |
| 5 | Headless idle RAM within budget | NOT RUN | requires logging in and measuring |
| 6 | `-s -S` + gdb attach demonstrated | NOT RUN | — |
| 7 | Zero host-disk writes outside the project dir | **NOT VERIFIED** | was recorded as VERIFIED citing ci/lint-safety.sh, which does not exist in this tree (corrected 2026-10-09) |
| 8 | Secure Boot enabled boot | NOT RUN | `tools/boot-test.sh --secureboot` exists but has not been run |

#### Correction — 2026-10-09: condition 7's evidence did not exist

Condition 7 was recorded as `VERIFIED` against ci/lint-safety.sh. No file of
that name has ever been in this tree; the row cited an artifact that cannot be
opened, which is the same defect class as Entry 1 and no less serious for being
a citation rather than a verdict. It is now `NOT VERIFIED`.

The property itself is plausible and uninteresting — the build ran inside WSL,
and the change that produced this correction was made without running a build at
all. What matters is that a gate condition cannot be satisfied by a filename: a
reader had no way to tell the difference between this citation and a real one.

Two countermeasures, applied in the same session:

  * `ci/lint-doc-refs.py` — every backticked path cited in `docs/**` and the
    top-level `*.md` files is resolved against several bases and must exist. It
    found seven such citations in this tree (this row, four in
    [BUG-0001](bugs/BUG-0001-fabricated-mkosi-config.md), one in
    [PERF-budget.md](PERF-budget.md), one in `docs/DESIGN-SYSTEM-v3/00-AUDIT.md`).
    The others are corrected; each says what it was and where the real artifact
    is. A path that is intentionally not in the repository is listed in the
    linter with a reason and reported as SKIP.
  * `ci/lint-doc-refs.py --self-test` proves the linter fails on a fabricated
    citation, so a green run means something.

Condition 7 remains `NOT VERIFIED` rather than `FAIL`: nothing here shows a host
write happened, and nothing conducted a build during this session that could
have shown one.

**GATE 01 verdict: PASS_WITH_LIMITATIONS.** The gate's headline condition is met
and demonstrated. Conditions 2, 3, 5, 6 and 8 are `NOT RUN`, condition 4 fails,
and condition 7 was downgraded from `VERIFIED` to `NOT VERIFIED` on 2026-10-09
because its cited evidence did not exist, so this is explicitly not a clean pass,
and the numbers above may not be published as LitricOS performance claims.

---

---

## GATE 00-K — Unified compatibility, deep recovery, Defender (DESIGN)
- **Date:** 2026-09-14
- **Result:** PASS (design gate — architecture and contracts only, no
  runtime claims are made or implied)

### Conditions
- [x] Every requirement in the unified-compatibility set maps to an ADR
  (0020/0021/0022) that states the decision, the alternatives' fate, and
  what is honestly not built yet
- [x] The deviations (Linux runtime = platform itself; 5-value health
  vocabulary) are documented with rationale, not silently reinterpreted
- [x] Implementable contracts exist (master-plan/22, master-plan/23) with
  invariants, functional work, and failure tests
- [x] SYSTEM-SCOPE Part K added (rows 1118–1165); tally verified by
  `ci/count-scope.py`; no row marked D without evidence
- [x] Defender Phase 0 exists as tested code consistent with ADR-0022
  (see `src/litric-defender`; test counts recorded in EVIDENCE.md, not
  claimed here)

### Notes
A design gate passes on the coherence of decisions, not on behaviour.
Rows added in Part K carry `S`/`P` statuses; none is `D`.

## GATE 02 … GATE 13 — NOT REACHED

Blocked behind GATE 01. No entries.

---

## Phase 6 desktop feature completion — 2026-09-19 — **NOT A GATE**

Recorded because LAW 1 cuts both ways: work that ran and passed must be
recorded, and so must the line it does not cross. This session completed
and verified the shell feature set on the **host**: context menus, modal
dialogs, Files ops wired to `litric-files` through `fsbridge`, a Trash
application, the Defender application over the real Phase-0 engine with
EICAR detection proven, Google search plus engine selection in the
browser, the `litric-browserd` broker (ADR-0018), desktop icons with a
shortcut to any folder, an overflow audit proving drawn text never
escapes its box, and 14 synthetic-input E2E journeys.

Verification run for this record (all executed, output summarised in
EVIDENCE.md): `cargo build --workspace` zero warnings; `cargo clippy
--workspace --all-targets` zero warnings under the workspace lint set;
`cargo fmt --check` clean; `cargo test --workspace` **885 passed,
0 failed**; rendered frames at 640/1280/1920 px in `evidence/`.

**This is not a gate verdict.** It is host-level behaviour of library and
binary code under test doubles for the filesystem where required. It
establishes nothing about a booted image: the QEMU journey of the new
features (right-click → scan → banner on real hardware) is NOT RUN, and
GATE 01's conditions and verdict are unchanged by this entry.

---

## Verification-harness repair — 2026-10-09 — **NOT A GATE**

Recorded for the same reason as the entry above: work that ran must be
recorded, and so must the line it does not cross.

This session ran under a hard code-only constraint: **no build, no compiler, no
VM, no GUI, no dependency installation.** `cargo build`/`cargo test` were
prohibited, so the Rust workspace was read but not compiled, and no LitricOS
image was produced or booted. Everything below was executed; nothing was
inferred.

### What was found

`ci/verify.sh` was already red before any audit work: section `[1b]` reported
five CRLF tool scripts. That is [BUG-0007](bugs/BUG-0007-crlf-in-five-tool-scripts.md),
fixed here by running the linter's own converter.

The two artifacts that decide whether LitricOS boots could not report what they
claimed to report. Four defects in [BUG-0005](bugs/BUG-0005-smoke-test-could-not-fail.md)
(the boot smoke test) and two in [BUG-0006](bugs/BUG-0006-gate01-verifier-aborted-and-could-not-pass.md)
(the in-VM GATE 01 verifier). They are the BUG-0002 class one level deeper:
not a test that was never run, but a test that could not fail, could not time
out, could not pass, or was pointed at artifacts the pipeline had stopped
producing.

Measured against the unmodified files, with QEMU and the system commands
replaced by stubs:

| Defect | Measurement |
|---|---|
| Smoke test deadline was advisory | 4 s deadline returned at **19.5 s** and at **25.0 s** — the child's lifetime, not the deadline |
| Success matched `#`, `>` and `shell` | an initrd line returned `status='SHELL'` in **0.0 s** for a boot that never reached userspace |
| Smoke test artifacts were stale | all four cases `skip` on a tree holding build/out/litricos.raw |
| Smoke test wrote host firmware NVRAM | `unit=1` pflash handed the OVMF_VARS template directly |
| GATE 01 verifier aborted | stopped after one check, exit **1**, no summary |
| Three GATE 01 probes could not pass | MGLRU compared against `"y"` while evidence records `0x0007`; Landlock read from the wrong file; `CONFIG_PREEMPT` grepped out of the kernel banner |

### What was executed

| Command | Result |
|---|---|
| `python -m pytest tests/test_harness_integrity.py -q` | **23 passed** — stub guests and stub system commands; no VM |
| fixed `tests/gate01-verify.sh`, healthy stubs | **17 passed, 0 failed**, exit 0 |
| fixed `tests/gate01-verify.sh`, `degraded` stub | FAIL reported, exit 1 (negative control) |
| `python tests/smoke_boot.py` | exit **77** (skipped), naming build/out/litricos.raw |
| `python -m pytest tests/smoke_boot.py -q` | **4 skipped**, no errors, reasons name every candidate path |
| `python ci/lint-doc-refs.py` | **OK, 82 citations across 86 documents resolve** |
| `python ci/lint-doc-refs.py --self-test` | **PASS** — proves it rejects a fabricated citation |
| `python ci/lint-mkosi.py`, `ci/lint-mkosi.py --self-test` | **PASS** |
| `python ci/lint-kconfig.py`, `ci/lint-kconfig.py --self-test` | **PASS** |
| `python ci/count-scope.py` | **PASS** |
| `python ci/lint-line-endings.py` | **FAIL before, PASS after** — 5 files in tools/ were CRLF; see [BUG-0007](bugs/BUG-0007-crlf-in-five-tool-scripts.md) |
| `python ci/check-python-syntax.py` | **PASS** — 22 files parse, no bytecode written; `tools/` is now covered for the first time |
| `python ci/check-python-syntax.py --self-test` | **PASS** — rejects a truncated file (negative control: a truncated copy of `tests/smoke_boot.py` fails the check, exit 1) |
| `bash -n` over tools/, iso/, ci/, image/ | **PASS** |
| `cargo build` / `cargo test` / `cargo clippy` | **SKIPPED — prohibited by this session's constraints** |
| image build, QEMU boot, GUI | **NOT RUN** |

### What this does not establish

Nothing here says LitricOS boots. The smoke test's detection logic is now proven
against guests whose behaviour was chosen by a test; it has not met a real guest,
so **GATE 01 condition 3 remains NOT RUN** and becomes *satisfiable* rather than
satisfied. GATE 01 condition 7 is downgraded to `NOT VERIFIED` — see the
correction in Entry 4. No performance number in `docs/PERF-budget.md` may be
quoted as a measurement, and that document's 320 MB total now states plainly
that no automated check enforces it.

The Rust workspace was not compiled in this session, so the 885-test result
recorded on 2026-09-19 is unchanged and unverified here.

---

## Beta 1.0 Stage A/B discovery pass — 2026-10-11 — **NOT A GATE**

A repository-truth and research pass requested before any implementation. No
production code, unit, image or boot behaviour was touched, so this entry makes
no claim about the system and explicitly does not move any gate.

### Documents produced

- [audit/2026-10-11-beta-1.0-stage-a-audit.md](audit/2026-10-11-beta-1.0-stage-a-audit.md)
  — Stage A evidence ledger: repository state, workspace constraints, crate and
  test inventory, backend reality, dead artifacts, risk map.
- [research/platform-parity.md](research/platform-parity.md)
  — Stage B, sourced from official NetworkManager, BlueZ, systemd, Linux-PAM,
  UPower, PipeWire/WirePlumber, Microsoft, Apple and W3C pages, with the rows
  that could not be verified marked as such.
- [master-plan/25-beta-1.0-gate-matrix.md](master-plan/25-beta-1.0-gate-matrix.md)
  — the Beta gate view, derived from `docs/SYSTEM-SCOPE.md` rather than
  duplicating it.
- `docs/ai-prompts/deepseek/` — three bounded implementation tickets. No model
  was invoked directly: this environment has no route to one, so the tickets are
  written for a human to carry, and are stated as such.

### Commands actually executed

| Command | Result |
|---|---|
| `python3 ci/lint-doc-refs.py` | **PASS** — 256 citations across 107 documents resolve (the linter rejected 10 citations in these new documents first; all 10 were corrected, not suppressed) |
| `python3 ci/lint-doc-refs.py --self-test` | **PASS** — still rejects a fabricated citation |
| `python3 ci/lint-line-endings.py` | **PASS** — 230 files, all LF |
| `python3 ci/count-scope.py` | **PASS** — 1,165 rows, tally unchanged (D 9 / P 113 / S 294 / T 749) |
| `cargo build` / `cargo test` / `cargo clippy` | **NOT RUN** |
| image build, QEMU boot, GUI, hardware | **NOT RUN** |

### Findings that a later gate will have to answer for

1. **There is no `.git` in this working copy.** No branch, no diff, no revert.
   The pass could not record modified/staged/untracked state because there is
   none to read, and per-patch rollback is not available until that is restored.
2. **Network, Bluetooth, audio, power and authentication have no backend in this
   tree** — no crate takes a D-Bus dependency, and there is no authentication
   subsystem of any kind. The prompt set's "connect Settings to the existing
   service" waves are therefore greenfield privileged work and are marked
   blocked on design, not on effort.
3. **Three systemd units name binaries that no crate produces**
   (`litric-capd`, `litric-defender`, `litric-filesd`). ADR-0016 already asks for
   their removal; they are still present.
4. **Three documents contradict the tree** — `docs/ARCH-00-boundaries.md` and
   `docs/REPO-00-layout.md` still describe a Smithay compositor and directories
   that do not exist, and `docs/RISK-00.md` RISK-010 tracks that non-dependency.
5. The workspace's headline rule — zero third-party dependencies in the core
   crates — **is not enforced by any check** in `ci/verify.sh`.

### What this does not establish

Nothing about whether the system builds, boots or performs. The last recorded
test result (941 passed, host-only, 2026-09-20) is untouched and unverified
here, `docs/EVIDENCE.md` remains stale by its own note, and every figure in
`docs/PERF-budget.md` remains a target rather than a measurement.

---

## Wave 0 corrections and the first clean full gate run — 2026-10-11 — **NOT A GATE**

The Stage A pass above ended with five findings a later gate would have to answer.
This is the pass that answered them, and the first full `ci/verify.sh` run since
2026-09-20. It is not a gate: nothing here boots the system.

### Commands actually executed

| Command | Result |
|---|---|
| `bash ci/verify.sh --evidence` | **PASS — 25 pass, 0 fail, 5 skipped**, exit 0. The five skips are the image build and the four boot measurements, all of which need a Linux host |
| `cargo clippy --offline --all-targets -- -D warnings` | **FAIL on the first run** — 14 diagnostics in `litric-shell`: indexing in `src/litric-shell/src/taskmgr.rs`, arithmetic side effects in `src/litric-shell/tests/text_layout_audit.rs` and `src/litric-shell/examples/frame_dump.rs`, `panic!` and indexing in `src/litric-shell/tests/font_standards.rs`, and `assertions_on_constants`. All 14 fixed at the sites; no lint lowered, allowed or skipped. **PASS** after the fix |
| `cargo test --offline` | **PASS — 1046 passed, 0 failed, 44 suites** |
| `python3 ci/lint-deps.py` and its `--self-test` | **PASS** — 16 of 18 members covered by the policy, 0 violations; 9 self-test cases including "an unknown manifest group fails closed" |
| `python3 ci/lint-unit-binaries.py` and its `--self-test` | **PASS** — every installed unit starts a binary the workspace builds; the dead-unit fixture fails |
| `git init -b main` and the first commits | **DONE** — `c70f3ba` records 637 files; `docs/RISK-00.md` RISK-015 records the gap and how it closed |
| image build, QEMU boot, GUI, hardware | **NOT RUN** — no Linux host |

### The five Stage A findings, closed

| Finding | State |
|---|---|
| no version control | closed: `git init`, `.gitignore`, and `.gitattributes` pinning LF — the host's Git had `core.autocrlf=true`, and a checkout would otherwise have rewritten the line endings that `ci/lint-line-endings.py` fails the gate for |
| no backend for network, Bluetooth, audio, power or authentication | **unchanged, and unchanged on purpose**: these stay design-first, so no code was written for them |
| units naming binaries that do not exist | the five units and `litric-defender.slice` are parked in `image/units-pending/`, with the reason each is waiting and the condition for its return; gate section [3g] now fails when an installed unit names a binary no crate builds |
| documents contradicting the tree | corrected: `docs/ARCH-00-boundaries.md`, `docs/REPO-00-layout.md`, `docs/master-plan/24-feature-parity.md` and `docs/RISK-00.md` (RISK-010 marked not applicable) |
| the zero-dependency rule unenforced | gate section [3h]: `ci/lint-deps.py` derives coverage from `src/Cargo.toml`'s own group comments, fails closed on a group with no decision, refuses an in-workspace build-dependency, and requires a path dependency to point at a declared member |

Three further defects were found by running things rather than reading them, and
are recorded because this is the pattern the repository keeps hitting: `ci/verify.sh
--evidence` overwrote `docs/EVIDENCE.md` whole, so running it would have deleted
the VM, benchmark and design-system records it sits beside; three documents said
the workspace has 19 crates where `src/Cargo.toml` lists 18; and
`image/mkosi.profiles/desktop.conf` claimed an enforcement (a budget linter,
lint-budget.sh in ci/) that has never existed, alongside two binaries no crate
builds.

### The image, built from this tree on the WSL builder

The audit's §8 rows were all `HISTORICAL` because every recorded boot predated the
tree. This entry adds the first build and boot of the current tree; see the section
below for exactly what was and was not observed.

### What this does not establish

Nothing about whether LitricOS boots. No image was built and nothing was booted,
so GATE 01 stays BLOCKED, the numbers in `docs/PERF-budget.md` remain targets
rather than measurements, and 1046 passing tests are behaviour under test on a
Windows host — not a runtime claim about a booted system.

> **Correction, later the same day:** this section is superseded by the entry
> below. An image *was* built from this tree and booted, and GATE 01 condition 3
> now runs. The rows above were true when written and are kept because the log is
> append-only; what they said about the build host is no longer the case.

---

## Wave 1 system providers, the first image built from this tree, and the first boot — 2026-10-11

**Verdict for GATE 01: still BLOCKED, but three of its conditions moved. The
image builds, it boots to a login shell, and the smoke test runs and passes on
the real artifact — including its own negative control.** What keeps the gate
blocked is listed under *not established* below, and none of it is wording.

### What was implemented in this pass

| Area | Where | State |
|---|---|---|
| Network facts | `src/litric-sysd/src/platform/net.rs` | interfaces, addresses (v4 + v6), routes, DNS, fixture-tested |
| Audio facts | `src/litric-sysd/src/platform/audio.rs` | cards, PCM devices, sound-server presence; `audio.control=none` says out loud that no level can be set |
| Power facts | `src/litric-sysd/src/platform/power.rs` | supplies, thermal zones, cooling devices, backlights |
| Bluetooth facts | `src/litric-sysd/src/platform/bluetooth.rs` | adapters, `rfkill` hard/soft blocks, `bluetoothd` presence, a four-word verdict |
| Session facts | `src/litric-sysd/src/platform/session.rs` | `systemd-logind` session records, and `auth.available=none` with `auth.lock=refused` |
| Capabilities | `src/litric-caps/src/capability.rs` | `net.configure`, `audio.set`, `power.control`, `auth.session`, `device.bluetooth.admin`, plus `Scope::parse_resource` for `wifi://`, `iface://`, `bt://`, `capture://`, `backlight://`, `lock://`, `power://system/*` and the rest |

The write half is **not** implemented and each design note says so in its own §8:
the `networkd` drop-in writer, the `iwd` credential file, the ALSA mixer helper,
the power menu and the PAM helper are all still decisions rather than code.

### Commands actually executed

| Command | Result |
|---|---|
| `cargo test -p litric-caps -p litric-sysd --locked` (on the WSL builder) | **PASS — 49 + 94 + 5 tests, 0 failed.** The three pre-existing tests that assert an unreadable root reports nothing rather than zeroes still pass, which is what forced `Platform::unreadable` to exist rather than emitting `net.iface.count=0` for a machine nobody read |
| `./tools/build.sh --profile desktop` (builder, profile `core desktop`) | **PASS, exit 0.** `litricos.raw.zst` 1,639,964,553 B; `litricos.system.raw.zst` 1,151,049,092 B; `litricos.verity.raw.zst` 153,713,037 B; `litricos.esp.raw.zst` 334,386,506 B; `initrd.cpio.zst` 64,511,633 B; `SHA256SUMS` written |
| QEMU boot, both serial ports captured (KVM, q35) | **PASS.** Kernel 7.2.9-arch1-1, dm-verity on (`litric.root.verity=on`), 78 `Reached target` lines, `bochs-drm` framebuffer, `systemd-networkd` up, and `litric-shell` reporting `frames=34` / `rows_presented=26400` |
| Login shell on the image's own console (ttyS1) | **PASS.** `litricos login: root (automatic login)` then `[root@litricos ~]#` |
| `litric-sysd --report` **inside the booted guest** | **PASS.** Real values, not fixtures — see the excerpt below |
| `pytest tests/smoke_boot.py` with `LITRIC_ACCEL=kvm LITRIC_BOOT_TIMEOUT=150` | **PASS — 4 passed, 180.66 s**, including `test_smoke_test_can_actually_fail` against a corrupted image |

### The running guest's own facts

Read out of the booted image with `tools/console-probe.py` (which exists because
of [BUG-0017](bugs/BUG-0017-smoke-test-listened-to-the-wrong-console.md)):

```
litric.root.verity=on
net.iface.count=2
net.iface enp0s2 state=up carrier=1 mac=52:54:00:12:34:56 mtu=1500 rx=626 tx=1952
net.addr enp0s2 family=4 addr=10.0.2.15 prefix=24 scope=global
net.addr enp0s2 family=6 addr=fe80::5054:ff:fe12:3456 prefix=64 scope=link
net.route destination=0.0.0.0/0 via=10.0.2.2 dev=enp0s2 metric=0
net.route destination=10.0.2.0/24 via=- dev=enp0s2 metric=0
net.dns nameserver=10.0.2.3
net.dns.source=resolved
net.addr.v4=iproute2
audio.card.count=0
audio.state server=none socket=-
audio.control=none
power.battery.count=0
thermal.cooling name=cooling_device0 type=Processor state=0
bt.adapter.count=0
bt.state=no-adapter
bt.daemon=absent
session.count=2
session.logind=present
auth.available=none
auth.lock=refused
session.id 1 user=root uid=0 seat=- type=tty state=active tty=ttyS1 display=- leader=482
```

Every one of those lines is consistent with what the machine is: QEMU's slirp
address on the virtio NIC, the sentinel gateway written as `-` for the directly
connected route, no sound hardware and no Bluetooth radio in this VM, and an
autologin session on ttyS1 that says `auth.available=none` because it is true.

### The build host, and why every earlier build died

Recorded because it cost the most time and is not obvious from any log:

- The image builder is a WSL2 Arch distribution. Its disk is a dynamically
expanding VHD on `C:`, and `C:` had **0.26 GB free**. Builds died one to three
minutes in, mid-`pacman`, every time, because the VHD could not grow — the guest
reported 935 GB free, the host had none, and the failure looked like a killed VM.
- Fixing it: 4.45 GB of caches cleared, the guest's own stale build artifacts
removed (6 GB inside the filesystem), and the VHD compacted from **37.52 GB to
15.55 GB** with `diskpart compact vdisk`, which needs elevation once. `C:` then
had 30 GB free and the build completed on the first attempt.
- Also learned, and applied: the WSL VM shuts down after the last `wsl.exe` client
exits, which kills a build started in the background. `vmIdleTimeout` is now set
in `%USERPROFILE%\.wslconfig`; builds are launched detached and survive.

### What this does not establish

| Claim | Why it is not established |
|---|---|
| GATE 01 condition 3 as a *default* run | the pass above used `LITRIC_ACCEL=kvm` and a 150 s deadline. Under the harness's default TCG acceleration the same boot did not reach a prompt inside 300 s. TCG is the default for a reason (it needs no nested virtualisation) and this host cannot yet run the gate the way CI would |
| the ≤3.5 s boot budget | measured ~75 s to a shell under KVM in QEMU. A VM boot is not the hardware the budget describes, but the budget is **not met** and no number from this pass may be quoted as meeting it |
| idle RAM and idle CPU (`docs/PERF-budget.md` rows) | not measured. `litric-sysd` reports its own RSS; nothing in this pass sampled the machine at idle from inside |
| the providers actually managing anything | every subsystem above is read-only in this pass. No interface was configured, no volume set, no device paired, no session locked |
| hardware beyond QEMU | the only machine this has run on is QEMU with virtio devices, which is why `audio.card.count=0` and `bt.state=no-adapter` are the *expected* readings there and not evidence about a laptop |

---

## The wave-1 write half, and the first boot that asked the system to *do* something — 2026-10-12

**Verdict for GATE 01: still BLOCKED. Two conditions moved, and the boot found
two defects that no test on a host could have found.** What keeps the gate
blocked is listed under *not established*, and none of it is wording.

### What was implemented in this pass

| Area | Where | State |
|---|---|---|
| Audio control | `src/litric-sysd/src/providers.rs` (`AudioAction`) | `/usr/bin/wpctl set-volume --limit 1.0 @DEFAULT_AUDIO_SINK@ <0.00-1.00>` and `set-mute ... 0\|1`; the percentage is clamped in code |
| Network actions | same (`NetAction`) | `ip link set <iface> up\|down`, `networkctl reconfigure <iface>`, `iwd` credential file + `iwctl station <iface> connect <ssid>`, and forgetting one |
| Bluetooth | same (`BtAction`), `image/mkosi.profiles/desktop.conf` | `bluez` + `bluez-utils` in the profile; `bluetoothctl` for radio power, discovery, pair, connect, forget; `bluetooth.service` enabled in the chroot |
| Authentication | same (`AuthAction`), `src/litric-sysd/src/platform/session.rs` | `/usr/bin/unix_chkpwd <user> nonull` with the password on **stdin**; `auth.available` is now read from the filesystem (`unix-chkpwd` or `none`) |
| Protocol | `src/litric-sysd/src/proto.rs` | `AUDIO VOLUME\|MUTE`, `NET LINK\|RECONFIGURE\|WIFI JOIN\|WIFI FORGET`, `BT POWER\|SCAN\|PAIR\|CONNECT\|FORGET`, `AUTH VERIFY ... CONFIRM` |
| Capabilities | `src/litric-caps/src/capability.rs`, `src/litric-sysd/src/state.rs` | `audio://` added to the resource schemes; the shell is granted `audio://`, `iface://`, `wifi://`, `bt://` and `lock://`, and each request is still checked and audited |
| Bounded waits | `src/litric-sysd/src/action.rs` | `Runner::run(program, args, stdin, timeout_secs)`; a tool that outlives its deadline is killed and reported as `reason=timeout` |
| In-guest probes | `image/mkosi.extra/usr/lib/litric/selftest` | four provider probes and a table of the tools that are actually in the image |

### Commands actually executed

| Command | Result |
|---|---|
| `cargo test -p litric-sysd -p litric-caps` (builder) | **PASS — 154 + 53 + 5 tests, 0 failed.** Includes the stdin plumbing against a real program (`/bin/sh -c 'read line && [ "$line" = secret ]'`), a real timeout (`/bin/sleep 30` killed at 1 s, asserted to return `TimedOut`), and the whole provider decision path with a recording runner |
| `ci/lint-mkosi.py`, `ci/lint-line-endings.py`, `ci/lint-doc-refs.py`, `ci/lint-unit-binaries.py` (builder) | **PASS** — 5 mkosi files, 242 files all LF, 398 citations across 114 documents, 11 program references (6 resolvable, 5 skipped, 0 broken) |
| `./tools/build.sh --profile desktop` (builder, profile `core desktop`) | artifacts produced: `litricos.raw.zst` 1,644,797,212 B, `litricos.system.raw.zst` 1,155,348,213 B, `litricos.esp.raw.zst` 334,303,142 B, `litricos.efi` 343,182,264 B, `initrd.cpio.zst` 64,452,621 B. `zstd -t` on the disk image, the ESP and the initrd: **all three intact** (3,392,213,504 / 536,870,912 / 190,169,600 bytes). **The build script did not finish**: the detached process was killed while mkosi was compressing the disk image, so `SHA256SUMS` is empty and the build's own closing report was never written. The artifacts above were integrity-tested instead, and the boot below is the evidence that they work |
| `tools/console-probe.py --accel kvm` booting `build/boot/litricos.raw` and running `/usr/lib/litric/selftest` in the guest | **PASS as a boot; one probe failed and one never returned** — see the defect table |

### The running guest's own report

```
litric.kernel=7.2.9-arch1-1
litric.boot.summary=Startup finished in 2.535s (kernel) + 3.113s (initrd) + 6.386s (userspace) = 12.035s
litric.units.failed=0
litric.mem.total_mb=2892
litric.mem.used_mb=845
litric.mem.avail_mb=2047
litric.proc.count=180
litric.thread.count=184
litric.root.fstype=erofs
litric.root.readonly=yes
litric.verity.active=yes
litric.sysd.binary=present
litric.sysd.rss_kb=3052
litric.sysd.ping=pong
litric.sysd.denies_ungranted_read=yes
litric.sysd.refuses_unconfirmed_poweroff=yes
litric.sysd.net_link_lo=NO -- ERR bad-argument on/off cannot be up
litric.sysd.refuses_unconfirmed_verify=yes
litric.sysd.refuses_wrong_password=yes
litric.sysd.audio_mute=unavailable -- no sound server in this VM
```

`litric.mem.used_mb=845` is the first idle-ish memory number this project has ever
had from inside its own image; it is **not** yet a budget result. It is a 3 GiB
QEMU guest with a framebuffer, a shell, `bluez`, `iwd` and the daemon, and the
`docs/PERF-budget.md` targets are for a smaller footprint than this VM; the
comparison needs a run with the same geometry as the target. The `bluez` row
added to that ledger the same day says the same thing.

### The three defects this boot found

| # | What the guest said | What it was | Fix, and the test that would catch it again |
|---|---|---|---|
| 1 | `sysd.net_link_lo=NO -- ERR bad-argument on/off cannot be up` | the protocol's `NET LINK` arm accepted `on`/`off` while the design note, the action and the self test all say `up`/`down`. A host-side test that only checked "a link request parses" would have passed, because it would have been written against the parser | `up_down` in `proto.rs`; `a_link_takes_up_and_down_and_not_on_and_off` asserts the three cases (up, DOWN, and a refusal for `on`) |
| 2 | the self test *stopped* after `sysd.audio_mute=…`: no Bluetooth line, no tool table, no report | `bluetoothctl` on a machine with no radio waits for a D-Bus name that never appears. The daemon had no deadline on a child process, so the action never returned and the report never continued | `Runner::run` now takes `timeout_secs`; every plan sets one (10 s for a socket round trip, 15 s for a local change, 30 s where a radio or a device may answer); a killed tool is `reason=timeout` and not an exit status. `a_tool_that_never_exits_is_killed_and_reported_as_a_timeout` runs `/bin/sleep 30` with a 1 s deadline and asserts the elapsed time |

| 3 | `litric.sysd.net_link_lo=NO -- ERR action-failed /usr/bin/ip exited 1` | `image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service` said, in as many words, "needs no capability at all -- so it is given none of them", and gave it an empty `CapabilityBoundingSet` and `RestrictAddressFamilies=AF_UNIX`. That was true of the daemon that only read `/proc` and `/sys`; the write half arrived later and kept the sentence. `ip link set <iface> up` goes through an `AF_NETLINK` socket and needs `CAP_NET_ADMIN`, so the action was refused by the kernel before `ip` could do anything. The unit's own comment was a claim about a daemon that no longer existed, and only a boot inside the unit's namespace could falsify it | `CapabilityBoundingSet=CAP_NET_ADMIN` and `AmbientCapabilities=CAP_NET_ADMIN` (ambient as well, so the grant does not depend on the call path), `RestrictAddressFamilies=AF_UNIX AF_NETLINK`, and `ReadWritePaths=/var/lib/iwd` for the `NET WIFI JOIN` credential write that `ProtectSystem=strict` would otherwise fail with `EROFS`. Each is justified in the unit with the reason, as `03-PHASE-02-CORE.md` requires of a non-empty bounding set, and `image/mkosi.extra/usr/lib/tmpfiles.d/litric-iwd.conf` guarantees the one writable path exists before the namespace is built — systemd fails a unit whose `ReadWritePaths` is absent. What catches it again is the in-guest probe: `litric.sysd.net_link_lo` can only read `ok` when the sandbox actually permits the action |

None of the three was reachable from the host, and none would have appeared in a
`cargo test` run: every one needed the action to be asked for **inside the
system**, which is the argument for the in-guest probes existing at all.

### The corrected build, booted

The three fixes above were made, the tree re-synced (`tools/sync-builder.sh`,
which verified 662 files identical), the image rebuilt and booted again.

| Step | Result |
|---|---|
| `./tools/build.sh --profile desktop` | **PASS, 5 m 37 s, exit 0** — and this time the build finished its own closing report, so `build/out/SHA256SUMS` holds eleven real hashes. The earlier attempt's was empty because the process was killed mid-compression |
| `zstd -d` of the disk image | `litricos.raw` 3,392,221,696 B, decompressed from `litricos.raw.zst` 1,644,904,629 B |
| `tools/console-probe.py --accel kvm`, then `/usr/lib/litric/selftest` in the guest | **PASS** — the lines below are that transcript |
| `pytest tests/smoke_boot.py -v` with `LITRIC_ACCEL=kvm LITRIC_BOOT_TIMEOUT=150 LITRIC_SMOKE_IMAGE=build/out/litricos.raw` | **PASS — 4 passed in 175.78 s (0:02:55)**, including `test_smoke_test_can_actually_fail` against its corrupted-image stub. This retires the row below that said the smoke test had not been run against this image |
| an earlier attempt at the same two commands in parallel | **FAILED, and worth recording**: `pytest` reported `Boot did not reach a shell. Status: TIMEOUT` with `qemu-system-x86_64: -drive ...: Failed to get "write" lock`. QEMU takes an exclusive write lock on the image, and `console-probe.py` was still holding it. Two boots of one image must be serialised; a `TIMEOUT` whose detail names a lock is that, and not a boot failure |

| Probe | Result |
|---|---|
| `litric.sysd.net_link_lo` | **`ok`** — `ip link set lo up` really ran, once the unit's sandbox permitted it (defect 3) |
| `litric.sysd.bt_power` | `unavailable -- no Bluetooth radio in this VM`. Before the deadline fix this probe never returned at all |
| `litric.sysd.audio_mute` | `unavailable -- no sound server in this VM` |
| `litric.sysd.refuses_wrong_password` / `refuses_unconfirmed_verify` / `refuses_unconfirmed_poweroff` / `denies_ungranted_read` | `yes`, `yes`, `yes`, `yes` |
| `litric.units.failed` | `0` |
| `litric.boot.summary` | `1.868s (kernel) + 2.803s (initrd) + 5.810s (userspace) = 10.482s`; a second boot of the same image measured `2.043 + 2.926 + 5.925` |
| `litric.mem.total_mb` / `used_mb` / `avail_mb` | `2892` / `849` / `2043` in a 3 GiB guest — the first idle-ish memory reading from inside this image, and still not a budget result |
| `litric.shell.state` / `.result` / `.version` / `.canvas` | `active` / `success` / `0.2.0` / `1280x800` |
| `litric.tool.usr.bin.*` | `wpctl`, `iwctl`, `bluetoothctl`, `networkctl`, `ip`, `systemctl` all `present mode=755`; `unix_chkpwd` `present mode=6755` |

The raw lines are reproduced in [docs/EVIDENCE.md](EVIDENCE.md).

### What this does not establish

| Claim | Why it is not established |
|---|---|
| that Wi-Fi, Bluetooth or audio *work* | the VM has no radio and no sound device. Every one of those actions was asked for and answered honestly, which is evidence about the answer path and **not** evidence about a laptop |
| that a correct password verifies | no account in this image has a known password, deliberately (`RootPassword=!locked`). What is established is the refusal direction: a wrong password is refused, an unconfirmed request never reaches the helper, and the helper is present and setuid |
| that a person can reach any of this | **no Settings pane exists for Network, Bluetooth or Audio** (`src/litric-shell/src/views.rs` says so in the interface). Everything in this entry is reachable from the console and the daemon's socket, and from nowhere else. This is the largest honest gap in the pass |
| that the lock screen is meaningful | there is still no lock screen, and `auth.lock=refused` is still in the reply even though a password can now be verified |
| the smoke test under the harness's *default* `accel=tcg` | the 4-case pass above used `LITRIC_ACCEL=kvm` and a 150 s deadline. TCG needs no nested virtualisation, which is why it is the default, and this host has not completed a boot under it inside 300 s — so the gate cannot yet be run the way CI would run it |
| the ≤3.5 s boot budget | 12.035 s to `systemd-analyze`'s userspace anchor inside a 3 GiB QEMU guest. Better than the ~75 s to a *login prompt* recorded on 2026-10-11 (different anchors, so not comparable), and still **not the budget**: that budget describes hardware this has never run on |
| idle CPU | not measured. Nothing in the guest sampled CPU at idle |

---

## 2026-10-14 — the Rust gate went from red to green, and the partial-redraw oracle found its first real defect

Host: Linux container, 4 vCPU, no GPU, default toolchain `1.85.0`. No QEMU and no
`mkosi`, so nothing in section [6] could run and every line of it is SKIP, not
PASS.

### What was red before anything was changed

| Check | Before | After |
|---|---|---|
| `cargo build --offline` | **FAIL** — `error[E0658]`: `litric-files` names `io::ErrorKind::InvalidFilename`, which is unstable on the workspace's declared `rust-version = 1.85` | PASS |
| `cargo clippy --all-targets -- -D warnings` | **FAIL** — 8 findings in committed code (`litric-files` ops/zip, `litric-bench`, `litric-shell`) | PASS |
| `cargo fmt --all -- --check` | **FAIL** — unformatted code in the uncommitted frame-pacing work | PASS |
| `cargo test --offline -p litric-shell --bin litric-shell a_partial_redraw_is_pixel_identical` | **FAIL** — `18768` pixels differed from a full redraw | PASS |

The MSRV failure is the one worth naming: the workspace *declares* 1.85, the CI
toolchain on a developer machine is current stable, and so the declared floor was
never actually exercised. `--offline` on a 1.85 toolchain is the cheapest test of
that claim and it did not pass.

### The gate, after

`bash ci/verify.sh`:

| Section | Result |
|---|---|
| `[1]`–`[3]` repository, documentation references, linters | PASS |
| `[4] Development signing keys` | PASS after `tools/gen-dev-keys.sh` — the dev pair is generated locally and is excluded by `keys/.gitignore`, so this is setup, not a committed artifact |
| `[5] Native workspace (Rust)` | **5/5 PASS** — build, test, zero warnings, `fmt --check`, `clippy --all-targets -- -D warnings` |
| `[6] Image build and boot` | SKIP x5 — `mkosi` and `qemu-system-x86_64` are not installed |

`summary: 23 pass, 2 fail, 5 skipped` on the first run; `2 fail` was the signing
key pair, which `tools/gen-dev-keys.sh` supplies. It is not a code defect.

### The defect the oracle found

[BUG-0019](bugs/BUG-0019-a-snap-that-changed-the-value-reported-no-frame.md):
`Anim::step` reported "nothing moved" on a settle-budget snap that moved the
value, so `Shell::tick` did not request the frame that would have painted it. It
is S3, it is fixed, and it has regression tests. The oracle itself had been
passing **vacuously** — its fixture's 1 016 ms first delta only became reachable
when the frame pacing started stepping springs by real elapsed time, which is
also why the defect is older than the change that exposed it.

### The renderer baseline from this host

`cargo run --release -p litric-shell --example render_bench`, 15 runs per line,
in [docs/EVIDENCE.md](EVIDENCE.md): a windowed full frame **106.8 ms**, an opened
window **245.8 ms**, a locked frame **334.8 ms** against its own 33 ms interval,
a full-screen two-pass blur **43–45 ms** and flat in radius, present **3.9 ms**.
Both renderer defects (BUG-0015, BUG-0018) stand. Nothing in this entry claims
they are fixed.

### What this entry does not establish

* Nothing about the image. `mkosi` and QEMU are absent here; no boot, no idle
  RAM, no idle CPU, no boot time was measured.
* Nothing about the frame-budget warning's runtime behaviour. `run_display` needs
  a framebuffer; the counter and its rate-limit policy are unit-tested, and the
  end-to-end path is **NOT RUN**.
* Nothing about the target hardware. Every number above is a container's.
