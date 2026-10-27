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
| Protocol | `src/litric-sysd/src/proto.rs` | `AUDIO LEVEL`, `AUDIO VOLUME\|MUTE`, `NET LINK\|RECONFIGURE\|WIFI JOIN\|WIFI FORGET`, `BT STATUS`, `BT POWER\|SCAN\|PAIR\|CONNECT\|FORGET`, `AUTH VERIFY ... CONFIRM` |
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

---

## 2026-10-23 — the wizard could not set a password, and the fix was a mount namespace

Host: Windows 11 + WSL2 (Arch), KVM inside WSL, `qemu-system-x86_64` with
`bochs-display` 1920x1080. Image: `build/out/litricos.raw.zst` (desktop
profile), decompressed and booted through OVMF via `tools/photo_tour_lib.py`
(headless, QMP screendumps) and once through `tools/desktop-app.sh` (GTK
window). Every claim below is a screenshot or a probe transcript under
`build/e2e/`.

### The defect this session found

The first-run wizard answered a new password with a raw
`ERR action-failed /usr/sbin/chpasswd exited 1` and would go no further, so the
desktop was unreachable by the path a person uses
(`build/e2e/qdrive-wizard4/11-desktop.png` is the error on screen). The
in-guest probe isolated it in four steps, each one boot:

| Probe | Result |
|---|---|
| `echo root:litric123 \| chpasswd` on a plain root shell | **succeeds**, silent |
| the same inside `litric-sysd`'s own mount namespace (`nsenter -t $(systemctl show --value -p MainPID litric-sysd) -m`) | `Authentication token lock busy` — PAM could not create `/etc/.pwd.lock` |
| inside the namespace with file-level rw binds on `/etc/.pwd.lock` and `/etc/shadow` | `Authentication token manipulation error` — the lock now worked, the update did not |
| inside the namespace with `/etc` bind-remounted read-write | **password changes**; `/etc/shadow` 593 B -> 665 B, no `/etc/shadow+` left behind |

The reason is shadow's update protocol: PAM creates `/etc/.pwd.lock` and the
update writes `/etc/shadow+` then `rename(2)`s it over `/etc/shadow`. A
file-level bind is not merely insufficient, it makes the rename fail with
EBUSY; the grant has to be the *directory*. `/etc` is an overlay whose upperdir
lives on the read-only `/var`, but an overlay writes through its own upper
mount, which the unit's namespace does not touch: a direct write to
`/var/lib/litric/etc-overlay/upper` inside the same namespace still fails with
EROFS (measured), so `/var` stays read-only and only the `/etc` view opens.

**Fix:** `ReadWritePaths=/var/lib/iwd /etc` in
`image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service`, with the
reason, the measurements and the EBUSY trap recorded in the unit the way
PHASE-02 requires. No capability changed; the bounding set is still exactly
`CAP_NET_ADMIN`.

### The fix, verified by clicking

`./tools/build.sh --profile desktop` (exit 0), decompress, boot, and drive the
real UI with QMP input events (usb-tablet absolute pointer + usb-kbd, screen
pixels mapped through the measured 0..0x7FFF client range):

| Step | Evidence |
|---|---|
| Welcome -> Get started -> language (ไทย) -> Next | `build/e2e/qdrive-wizard/02-after-get-started.png` |
| password, confirm, Set password | completes with no error; `build/e2e/qverify/10-after-wizard.png` is the desktop after it |
| click the dock's Files icon | **Files window opens** — sidebar, toolbar, `build/e2e/qverify/20-files.png` |
| click Terminal and Settings | both open; `build/e2e/qverify/40-settings.png` shows Settings with 203 settings and the panes |
| `litric-shell` frames actually reach the scanout | `rows_presented=170640` in `/tmp/litricos-app-serial.log` (the `litric-fbpresent` helper from the interrupted session; before it, the QMP screendump was the frozen firmware screen — `build/e2e/first-run-audit/first-boot-after-verity.ppm`) |

`tools/desktop-app.sh` was then started for the user (GTK window, 1.5 GiB,
2 vCPU) and its serial shows `litric.shell.mode=ready` with 158 frames drawn.

### What this does not establish

| Claim | Why |
|---|---|
| that the wizard's result survives a reboot | it does not: the wizard returns on every boot of the same disk file, [BUG-0023](bugs/BUG-0023-the-first-run-wizard-came-back-after-every-reboot.md). `/var` and the `/etc` overlay persist (measured), so the completion marker is the suspect |
| that the `litric` account password persists | the set succeeds now, but every boot re-runs the wizard, so no hash has been observed across a reboot |
| a keyboard-driven app launch | `ctrl-spc` produced no launcher in the harness; the dock icons respond to clicks. Not investigated |
| an empty folder is error-free | Files shows a red `no such item` for an empty Documents (`build/e2e/qverify/20-files.png`). Cosmetic, untriaged |
| a frame budget | the GUI session logged `frames_over_budget=154` of 159 on 2 vCPU under WSLg; recorded, not analysed |
| anything about Wi-Fi, Bluetooth or audio | the VM still has no radio and no sound device |

### Rollback

The unit change is one line plus its comment; reverting
`image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service` and rebuilding
restores the previous (broken wizard) behaviour. The previous image is kept by
`tools/build.sh` as `litric.last-good`.

---

## 2026-10-25 — client surfaces: a foreign Wayland application gets a window

**Stage result:** the capability is real and tested. **Workspace gate: 23 PASS,
3 FAIL, 4 SKIP.** All three failures are in code this stage did not write, and
all three were already red at `c28ec35`; they are recorded here because this is
the first run of `ci/verify.sh` in this tree that was not reading cargo's cache
instead of the code (see "Why the previous green was not a reading").

### What was built

| Piece | What it is |
|---|---|
| `src/litric-compositor/` | A real Wayland server (ADR-0024): `wl_compositor`, `wl_shm`, `wl_seat`, `wl_output`, `xdg_wm_base`, on its own thread, `#![forbid(unsafe_code)]` |
| `src/litric-compositor/src/keymap.xkb` | A resolved `us` keymap, shipped with the crate and handed to every keyboard |
| `src/litric-shell/src/clients.rs` | Surfaces become `litric-wm` windows: LitricOS title bar, client pixels in the body, input forwarded in z-order |
| `docs/adr/0024-client-surfaces.md` | The decision, the advertised globals, and the list of what is deliberately absent |

### Evidence — a client from outside this repository

The client is `weston-simple-shm` from the Weston tree: C, `libwayland-client`,
`wl_shm` + `xdg-shell`. Nothing in this path is two halves of the same Rust code
agreeing with each other.

| Test | Assertion | Result |
|---|---|---|
| `litric-compositor --test client` (`LITRIC_WAYLAND_CLIENT` set) | maps a window, commits frames with real content, honours a configure to 320×200, survives pointer and key events, exits when the toplevel is closed | PASS (4 tests) |
| same binary, no `LITRIC_WAYLAND_CLIENT` | the foreign-client test prints its `SKIP` line and returns; the three server-only tests (relative runtime dir refused, second bind refused, killed client) run | PASS (3 real, 1 skip) |
| `litric-compositor` unit tests | stride walk, pool bounds, `XRGB` opacity, buffer scale, XKB modifier bits | PASS (6 tests) |
| `litric-shell --test foreign_clients` | the desktop hosts that client as a window with a LitricOS title bar, paints its pixels, forwards click and keystroke, closing the window ends the process; a typed command launches a GUI application | PASS (3 tests) |
| `litric-shell --lib` | every shell test, including the new broker/protocol ones | PASS (438 tests) |
| `cargo test --offline` (workspace) | all core crates | PASS |

The rendered frame is `artifacts/foreign-client-window.png`: the client's
concentric-ring drawing inside a LitricOS window titled `simple-shm`, on the
LitricOS desktop, with the dock and top bar drawn by the shell around it.

The keymap was checked with libxkbcommon itself — the same call a toolkit makes
(`xkb_keymap_new_from_string`), not a string comparison:

| Check | Result |
|---|---|
| `xkb_keymap_new_from_string(keymap.xkb)` | compiles |
| layouts | 1 — `English (US)` |
| evdev 16/30/2/57/28 (+8, as a client counts) | `q`, `a`, `1`, space, Return |
| the same with Shift held | `!`, `A` |

### Why the previous green was not a reading

`cargo build`, `cargo clippy` and `cargo fmt --check` print diagnostics only for
targets they actually recompile. The [5] entry of the 2026-10-14 run and the
"gate green" claim inside `c28ec35` were made against a warm `target/`: nothing
was recompiled, so nothing was reported. `touch src/litric-shell/src/shell.rs`
and rebuild, and seven private helpers in `shell.rs` are reported as never used
(`set_overlay`, `same_overlay_kind`, `set_context_menu`, `set_desktop_dialog`,
`retire_banner`, `dismiss_notification`, `clear_notifications`). Those are the
overlay, context-menu, dialog and notification *exit* animations: the 2026-10-23
redesign replaced the call sites with direct field writes, so the springs are
computed nowhere. That is a real gap in the shell, not a lint annoyance, and it
is why this entry says the gate is red rather than quoting the old green.

### The three failures

| Item | Site | Why it is red |
|---|---|---|
| [3d] documentation citations | `docs/DESKTOP-2.0.md:96` | the frame-dump example is cited with its path missing the `src/litric-shell/` prefix, so the cited file does not exist |
| [5] `cargo build` emits zero warnings | `src/litric-shell/src/shell.rs` | the seven unused exit-animation helpers above |
| [5] `cargo clippy --all-targets -- -D warnings` | `src/litric-design/src/lib.rs:645`, `src/litric-sysd/src/platform/input.rs:138, 237, 241, 269, 289, 304` | `pair[0]`/`pair[1]` in a design test, `bit / 64` in the sysd capability reader, and fixture `devices[0]`/`devices[1]` indexing in six sysd tests |

Nothing was suppressed to arrive at this number. This stage's own code is clean
under the same gate: `litric-compositor` (lib and tests) and the new
`litric-shell` modules and tests produce no diagnostics, the formatting check
passes, and the 12 deletion sites the new code adds are recorded with a reason in
`ci/lint-file-safety.py`, which now passes (44 sites reviewed).

### What this does not establish

| Claim | Why |
|---|---|
| that a client window appears on a booted image | no image was built or booted: mkosi, qemu and the Latin face the golden-frame dump needs are absent on this host |
| that X11 applications run | no XWayland, and none is advertised |
| clipboard, drag-and-drop, IME, Thai typing into a client | `wl_data_device_manager` and `zwp_text_input_v3` are not implemented; the keymap is a fixed `us` layout, so switching the shell's layout does not switch a client's |
| that a client repaints at display rate | frames are copies through the CPU (`pread` after each commit); no dmabuf import |
| that a menu near the screen edge flips | popups are anchored at the positioner's rect origin; constraint adjustment is recorded but unused |

### Rollback

The stage is additive: delete `src/litric-compositor/`,
`src/litric-shell/src/clients.rs`, `src/litric-shell/tests/foreign_clients.rs`
and the `clients`/`compositor` wiring in `src/litric-shell/src/shell.rs`, and the
desktop is the 2026-10-23 software desktop again, with no window for a foreign
application.
## 2026-10-25 — the three red rows closed: exits animate, citations and lints fixed

- **Result:** PASS_WITH_LIMITATIONS
- **Command:** `ci/verify.sh` (Linux x86_64, cargo +stable; no mkosi, qemu or pytest on this host)
- **Raw:** **26 pass, 0 fail, 4 skipped** — the entry above recorded 23/3/4 on the
  same tree. The three failing rows are the three fixed below; nothing else moved.
- **Scope:** the exit animations the previous entry called "a real gap in the
  shell", plus the two lint rows. No new feature, no suppressed diagnostic.

### The exit animations were missing, not just unused

The seven helpers the compiler reported were the *shell half* of the exit
animations: `set_overlay`, `same_overlay_kind`, `set_context_menu`,
`set_desktop_dialog`, `retire_banner`, `dismiss_notification`,
`clear_notifications`. The drawing half did not exist either — `views.rs` never
read `retiring_overlay`, `retiring_context_menu`, `retiring_desktop_dialog` or
`retiring_banners`, and the `switcher`, `overview` and `dialog` springs were
stepped every frame and read by nothing. The 2026-10-23 redesign had written the
fields directly, so a closed surface disappeared in one frame. Restoring the
call sites alone would have kept the dead routines alive and changed no pixel.

| Layer | Change |
|---|---|
| `src/litric-shell/src/shell.rs` | Every overlay, menu, dialog and notification close goes through the exit-aware setter; `set_overlay` keeps a snapshot only when the revealed spring can actually fall (the launcher's dock morph and the three top-bar panels that share one spring replace each other with nothing to animate). `tick` drops each snapshot when its spring reaches zero, not on a timer. `retire_banner` now retires the id that was dismissed; it used to read the last snapshot's id, so dismissing a second banner left the first one's exit running. |
| `src/litric-shell/src/views.rs` | Retiring overlays, menus and dialogs draw beneath the surface that replaced them, through the same reveal spring, inside `Shell::draw_retired`: hotspots go to a scratch table and hover answers false, so a panel on its way out is pixels, never a target. Switcher and dialog unfold from the middle; Mission Control's dim follows its slow spring; banners slide from the screen edge. |
| `src/litric-shell/src/anim.rs` | Unchanged. The springs were already stepped, bounded and reduced-motion aware; what was missing was a caller and a reader. |

The canvas has no compositing layer, so these exits are clip and travel, not
alpha fades. A fade needs a layer that does not exist, and adding one for four
surfaces was a larger change than the gate row asked for.

### The two lint rows

| Row | Fix |
|---|---|
| `[3d]` citation | `docs/DESKTOP-2.0.md:96` now names `src/litric-shell/examples/frame_dump.rs`, the file that exists. |
| `[5]` clippy | `src/litric-design/src/lib.rs` takes `windows(2)` elements with `first`/`get` instead of `[0]`/`[1]`; `src/litric-sysd/src/platform/input.rs` reads word `bit >> 6` and bit `bit & 63` instead of dividing by 64, and its fixtures use `first`/`get` instead of indexing; the dock assertion in `src/litric-shell/src/e2e.rs` uses a range `contains`; the setup drawing in `src/litric-shell/src/views.rs` moved above the test module it had been appended after. |

While in `draw_notification_list`, an early return for an empty queue that never
restored its clip was fixed: it leaked a small clip into every surface drawn
after it on that frame. It was not what the gate reported; it was on the path
being edited.

### Evidence

`cargo test --offline -p litric-shell --tests`, all from a recompiled tree:

| Target | Result |
|---|---|
| lib | 444 passed (438 before this change; 6 new) |
| `src/litric-shell/src/main.rs` | 31 passed |
| `src/litric-shell/tests/design_lint.rs` | 4 passed |
| `src/litric-shell/tests/font_standards.rs` | 10 passed |
| `src/litric-shell/tests/foreign_clients.rs` | 3 passed |
| `src/litric-shell/tests/overflow_audit.rs` | 3 passed |
| `src/litric-shell/tests/text_layout_audit.rs` | 9 passed |

The six new tests, in `src/litric-shell/src/shell.rs`:

| Test | What it would catch |
|---|---|
| `a_closed_panel_is_kept_exactly_while_its_exit_spring_has_pixels` | a close that is still a cut (snapshot dropped in one frame) or a snapshot that outlives its settle budget |
| `reopening_a_panel_reverses_its_spring_instead_of_drawing_two_copies` | a reopen that leaves the old panel fading under the new one |
| `swapping_between_panels_that_share_a_spring_does_not_keep_a_ghost` | a power menu that keeps a quick-settings ghost because both read one spring |
| `a_retired_menu_is_still_painted_but_registers_no_click_target` | an exit that draws nothing, or one whose rows can be clicked after the menu closed |
| `a_dismissed_dialog_keeps_its_panel_but_not_its_buttons` | a fading dialog that repeats a confirmation |
| `a_dismissed_banner_keeps_its_pixels_until_its_spring_lands` | a banner dropped before its spring reaches zero |

`ci/verify.sh` then reported, for `[5]`: `cargo build`, `cargo test`, **zero
warnings**, `cargo fmt --check` and `clippy --all-targets -- -D warnings` all
PASS. The four SKIPs are unchanged and named: image build, boot smoke test,
idle measurement (no mkosi/qemu/pytest) and golden frames (no Latin face on this
host).

### What this does not establish

| Claim | Why |
|---|---|
| that the exits look right on a real framebuffer | no image was built or booted; the evidence is composed frames (`views::draw`) and state assertions, not a photograph of a screen. `artifacts/foreign-client-window.png` is still the only rendered frame from this tree |
| that an exit is perceptually smooth at 60 Hz | the springs are bounded by their tokens (tested in `src/litric-shell/src/anim.rs`) and the shell damages what moved, but no frame-timing measurement was taken on hardware |
| that every animated surface is covered | the launcher's dock morph is unchanged and the three top-bar panels still replace each other without an exit, deliberately and stated above |

### Rollback

Revert this commit. The surfaces close in one frame again, the three gate rows
go back red, and the six tests above fail — which is the point of the tests.

## 2026-10-25 — accent colour is a setting, and the contrast gate survives it

- **Result:** PASS_WITH_LIMITATIONS
- **Command:** `ci/verify.sh` (Linux x86_64, cargo 1.99.1; no mkosi, qemu or pytest on this host)
- **Raw:** **26 pass, 0 fail, 4 skipped** — same counts as the entry above. The four
  SKIPs are unchanged and named: image build, boot smoke test, idle measurement and
  golden frames.
- **Scope:** Wave 2 (appearance), first slice, taken from this matrix's own blocker:
  "accent-colour and wallpaper settings do not exist". This entry covers the accent
  half only; the wallpaper half is not claimed and not implemented.

### Why an accent is a slot, not a hex value

An accent picker that stores one hex value and applies it to every theme ships an
unreadable selection colour: the blue that reads on a near-black background is not
the blue that reads on white. So the preference stores an [`Accent`] *slot*
(`default`, `sky`, `violet`, `teal`, `amber`, `rose`) and every theme carries its own
value for that slot, resolved by `Theme::with_accent`.

Two rules make it safe rather than merely pretty:

| Rule | Why |
|---|---|
| `with_accent` moves only `accent` and `on_accent` | the surfaces, the text and the status colours are the contrast contract; a decorative choice that could move them would be a second theme system |
| high-contrast themes return themselves unchanged | their accent is part of the AAA guarantee, so the picker must not be able to weaken it. The Settings row says "fixed in high contrast themes" instead of offering a control that silently does nothing |

### What changed

| File | Change |
|---|---|
| `src/litric-design/src/lib.rs` | `Accent` (token/parse/all) and `Theme::with_accent`; seven new tests including the two contrast gates over every accent in both standard themes |
| `src/litric-shell/src/settings.rs` | `Preferences.accent`, defaulting to the theme's own accent |
| `src/litric-shell/src/shell.rs` | `resolved_theme` is the single place a preference becomes a theme; `cycle_accent`, `Hot::SettingsAccent`, Ctrl+A on the Settings surface |
| `src/litric-shell/src/views.rs` | the Appearance row, with the high-contrast wording and the updated shortcut hint |
| `src/litric-shell/src/main.rs` | `accent=` written and parsed; an unknown token falls back to the default instead of failing the record |
| `docs/SYSTEM-SCOPE.md` | row 246 (accent colour selection) `T` → `P`, tally rewritten by `ci/count-scope.py --fix` |

### Evidence

| Target | Result |
|---|---|
| `cargo test -p litric-design` | 40 passed (33 before; 7 new) |
| `cargo test -p litric-shell --lib` | 445 passed (444 before; 1 new) |
| `src/litric-shell/src/main.rs` | 32 passed (31 before; 1 new) |
| `src/litric-shell/src/e2e.rs` | the Settings journey now clicks the accent row and asserts the persistence effect |

The new tests and what they would catch:

| Test | What it would catch |
|---|---|
| `every_accent_keeps_text_legible_on_its_own_fill` | a swatch that is fine and a button label that is not, because `on_accent` was left at the default theme's value |
| `every_accent_is_readable_as_text_on_the_theme_backgrounds` | a pastel meant as a fill, reused as the accent text tone |
| `high_contrast_themes_ignore_the_accent_choice` | a picker that can weaken an AAA theme |
| `choosing_an_accent_moves_only_the_accent_pair` | a decorative option becoming a second theme |
| `the_default_accent_is_the_theme_itself` | a default that quietly differs from the shipped palette |
| `accent_tokens_are_unique_and_round_trip` | two accents sharing a token, so a stored choice loads as a different colour |
| `the_accent_choice_survives_a_restart_and_unknown_tokens_fall_back` | a write that is never read back, or a newer file that cannot load |
| `the_accent_applies_on_standard_themes_and_not_on_high_contrast_ones` | the shell drawing an accent its own settings page says is ignored |

### What this does not establish

| Claim | Why |
|---|---|
| that the accent looks right on a real framebuffer | no image was built or booted and the golden-frame row is SKIP (no Latin face on this host). The evidence is tokens, tests and composed state, not a photograph |
| that the accent survives a reboot on the image | the preference round-trip is tested in `main.rs` against a temporary root; the image's writable location was not exercised |
| that an accent changes the wallpaper or the glass | it does not, deliberately: the wallpaper sampler and the glass solver are unchanged. Row 247 (accent extracted from the wallpaper) is still `T` |
| that every surface uses the accent consistently | the token is global, but no frame-diff test asserts each surface; the design lint rules were not extended for it |

### Rollback

Revert this commit. `Accent` disappears, `Preferences` loses the field, the
Appearance page is theme → language again, and the seven design tests and the two
shell tests above fail with it — which is the point of adding them.

## 2026-10-25 — a wallpaper choice, and the menu row that finally does what it says

- **Result:** PASS_WITH_LIMITATIONS
- **Command:** `ci/verify.sh` (Linux x86_64, cargo 1.99.1; no mkosi, qemu or pytest on this host)
- **Raw:** **26 pass, 0 fail, 4 skipped** — unchanged from the entry above; the four
  SKIPs are still image build, boot smoke test, idle measurement and golden frames.
- **Scope:** Wave 2 (appearance), second slice: the wallpaper half the previous entry
  left open. Same gate, one commit later.

### The row that named one thing and changed another

The desktop context menu has always carried **"Next wallpaper"**. It called
`cycle_theme`, so it changed the colour theme and left the wallpaper alone. That is
the kind of defect a status table cannot show: the control existed, the action
existed, and the wiring between them was wrong in a way only a click reveals. The
row now cycles the wallpaper, and a test performs that action and asserts the theme
did not move.

### What a wallpaper choice is allowed to be

Three built-in wallpapers, all drawn by the shell from tokens and its one embedded
image. No directory of images, because ADR-0009 keeps decoders out of the privileged
drawing process, and no user-supplied file format support: a choice that opened
`/usr/share/backgrounds` to the shell's decoder would be a new attack surface sold as
a feature.

| Choice | What it paints | Why it exists |
|---|---|---|
| `system` | the embedded obsidian wave (dark) / solid pearl (light) — byte-identical to what shipped before the setting | the default must change no pixel |
| `flat` | one field of the theme's own `bg_base` | the calm option; nothing to sample, nothing to decode |
| `glow` | a vertical wash `bg_elevated` → `bg_base` with a 10% accent glow top-right | a second look that still comes from the palette, so the contrast gate has seen every colour in it |

### What changed

| File | Change |
|---|---|
| `src/litric-shell/src/desktop.rs` | `Wallpaper` (all/token/parse), `wallpaper_for`, and `Compositor::new`/`retheme` take the choice. `wallpaper()` remains the system default for the frame dump and the older render path |
| `src/litric-shell/src/settings.rs` | `Preferences.wallpaper`, defaulting to `system` |
| `src/litric-shell/src/shell.rs` | `cycle_wallpaper`, `Hot::SettingsWallpaper`, Ctrl+W on the Settings surface, and `NextWallpaper` routed to it instead of `cycle_theme` |
| `src/litric-shell/src/views.rs` | the Appearance row and the updated shortcut hint |
| `src/litric-shell/src/main.rs` | `wallpaper=` written and parsed; the compositor is built and rebuilt with the stored choice |
| `docs/SYSTEM-SCOPE.md` | rows 219 (wallpaper: static image) and 425 (wallpaper picker) `T` → `P`; tally rewritten by `ci/count-scope.py --fix` |

### Evidence

| Target | Result |
|---|---|
| `cargo test -p litric-shell --lib` | 450 passed (445 before; 5 new) |
| `cargo test -p litric-shell` (all targets) | 450 + 32 + 4 + 10 + 3 + 3 + 9 passed, 0 failed |
| `src/litric-shell/src/e2e.rs` | the Settings journey clicks the wallpaper row and asserts the persistence effect |

The new tests and what they would catch:

| Test | What it would catch |
|---|---|
| `every_wallpaper_choice_paints_something_different` | a picker whose entries all paint the same frame, compared as whole frames rather than one pixel |
| `the_flat_wallpaper_is_one_colour_and_the_glow_is_not` | `glow` collapsing to a flat fill, which would keep the setting "working" and change nothing on screen |
| `wallpaper_tokens_are_unique_and_round_trip` | two choices sharing a token, so a stored choice loads as a different wallpaper |
| `every_wallpaper_still_leaves_the_glass_a_readable_alpha` | a new wallpaper becoming a way to make text unreadable without touching contrast: for every theme and wallpaper the glass solver must still find an alpha that keeps body text at 4.5:1 over the brightest pixel |
| `the_next_wallpaper_row_changes_the_wallpaper_and_not_the_theme` | the original defect: a menu row that names the wallpaper and cycles the theme |

`the_wallpaper_stays_a_backdrop_not_a_gradient_demo` was extended from the shipped
wallpaper to all three, because `glow` is exactly the kind of accent wash that test
exists to keep muted (chroma ≤ 90 at three anchors).

### What this does not establish

| Claim | Why |
|---|---|
| that the wallpaper looks right on a real framebuffer | no image was built or booted; golden frames are SKIP (no Latin face on this host). The evidence is painted canvases and state, not a photograph |
| that it is a wallpaper *picker* in the product sense | it is a cycling row over three built-ins: no thumbnails, no user images, no slideshow, no per-monitor wallpaper (row 221 still `T`), no time-of-day wallpaper (row 222 still `T`), no accent extracted from the wallpaper (row 247 still `T`) |
| that frame cost is unchanged | the flat and glow fields are cheaper than the wave, but no frame-time measurement was taken on a booted image; the project's perf budgets are still `FAIL` as recorded |
| that the choice survives a reboot on the image | the record round-trip is tested against a temporary root, not the image's writable location |

### Rollback

Revert this commit. `Wallpaper` disappears, the compositor takes a theme only, the
context-menu row cycles themes again, and the five tests above fail — including the
one written for the original defect.

## 2026-10-25 — one preference store: versioned, atomic, and it survives garbage

- **Result:** PASS_WITH_LIMITATIONS
- **Command:** `ci/verify.sh` (Linux x86_64, cargo 1.99.1; no mkosi, qemu or pytest on this host)
- **Raw:** **26 pass, 0 fail, 4 skipped** — the four SKIPs are image build, boot smoke
  test, idle measurement and golden frames.
- **Scope:** the gate matrix blocker "Settings application and registry — no settings
  store, no schema, no persistence path", narrowed to the settings the shell itself
  owns. It does **not** claim a settings daemon.

### What the old path did wrong

The record was parsed and written inline in `main.rs`. Three specific failures, in
increasing order of consequence:

| Failure | What a user sees |
|---|---|
| two parsers waiting to drift | nothing today; a second reader of the file would have had to copy the match arms, and the copies would diverge on the first new key |
| `fs::read_to_string` treated **any** read error as "no file" | an invalid-UTF-8 or truncated record — a write interrupted by a power cut — made the account look brand new, so first-run appeared and the desktop settings looked lost |
| no `version` key | there was nowhere for a migration to start, so the only way to change the format was to guess from the keys that happened to be present |

### What changed

| File | Change |
|---|---|
| `src/litric-shell/src/prefstore.rs` | new module: `VERSION = 1`, `parse`, `encode`, `Store::{new, path, load, save}`. Line-oriented `key=value`, unknown keys ignored, unparsable values keep the caller's default, invalid UTF-8 decoded lossily instead of refused. Writes go to `settings.conf.tmp.<pid>`, are `fsync`ed, renamed, and the directory is `fsync`ed; the directory is `0700` and the file `0600` |
| `src/litric-shell/src/main.rs` | `load_preferences` / `store_preferences` are now two short calls into the store. **Behaviour change:** a record that exists but cannot be read is reported on stderr and defaults are used with setup still finished, instead of being silently treated as a fresh account |
| `src/litric-shell/src/lib.rs` | module registration |
| `ci/lint-file-safety.py` | one reviewed entry for `prefstore::save` — the only deletions are the store's own temporary file, before writing and on the failure path (45 sites reviewed, up from 44) |
| `docs/master-plan/25-beta-1.0-gate-matrix.md` | the settings row now names what exists and what still does not |

### Evidence

`cargo test -p litric-shell --lib` — 460 passed (450 before; 10 new), `main.rs` 32
passed, every other target unchanged. `cargo clippy --all-targets -- -D warnings`
passes; `ci/lint-file-safety.py` reports 45 reviewed deletion sites.

The new tests and what they would catch:

| Test | What it would catch |
|---|---|
| `a_record_round_trips_through_the_file` | a save that is never read back, or a key written under one name and parsed under another |
| `an_absent_file_is_not_an_error_and_a_present_one_counts_as_set_up` | "missing" and "unreadable" collapsing into the same state again |
| `a_file_from_before_the_flag_existed_still_counts_as_finished_setup` | an upgraded account being shown first-run because the new flag is absent from an old file |
| `unknown_keys_and_values_keep_the_defaults` | a newer build's file resetting this build's settings to garbage, or a `version=9` file being rejected outright |
| `malformed_lines_are_skipped_rather_than_failing_the_whole_record` | one bad line costing the user every other setting |
| `bool_tokens_accept_the_forms_a_person_writes` | a hand-edited `yes` being read as `false` |
| `the_encoded_record_names_its_version_first` | a writer that loses the version, making migration impossible again |
| `the_file_is_private_to_the_account_and_leaves_no_temporary_behind` | a world-readable preference file, or a temp file left behind on every save |
| `a_garbage_file_parses_to_defaults_instead_of_stopping_the_desktop` | binary noise or a truncated write taking the desktop down at startup |
| `saving_over_a_stale_temporary_still_succeeds` | a killed process's temp file making every later save fail forever |

### What this does not establish

| Claim | Why |
|---|---|
| that this is a settings store for the system | it is the shell's own preferences. `litric-settingsd` still does not exist and settings owned by other components have no store; the gate matrix row stays `NOT_IMPLEMENTED` |
| that the write survives a power cut | atomicity is by construction (temp file, `fsync`, rename, directory `fsync`), and the tests prove the failure paths that can be exercised in-process. No power-loss injection was performed |
| that the file lands where the image allows | the path is `/var/lib/litric-shell/settings.conf` under the caller's root; the image's writable layout was not exercised because no image was built |
| that `Saved/UNRESOLVED_ITEMS.md` item 11 is closed | that file is a dated audit snapshot of an earlier tree and was deliberately not edited; this entry is the newer evidence, and the daemon/registry half of the item remains open |

### Rollback

Revert this commit. The inline parser and writer come back, the record loses its
`version` line (it still parses), the behaviour change above reverts to showing
first-run for an unreadable file, and the ten tests above disappear.

## 2026-10-26 — the browser's build path and its settings, audited against a repair brief

### Verdict

**PASS_WITH_LIMITATIONS** at the `UNIT` / `STATIC` level. Two real defects were
found and fixed. The brief's central claim — that the browser cannot load a page
in a booted image — could **not** be reproduced or refuted here, because this
machine has no Chromium, no `mkosi` and no QEMU. Nothing in this entry is `VM
E2E`.

### What the brief asked for, and what was already true

The brief was written against an older tree. Of its fourteen audit questions,
eleven are already implemented and unit-tested at `abf70e2`: the broker is built
with `--features broker` by `tools/build.sh`, the binary is staged and the
desktop profile refuses to build without it, the socket path matches on both
sides (`/run/litric/browser.sock`), LBP/1 agrees command for command, and the
shell checks frame length and preserves aspect before drawing. The full table is
in `docs/audit/2026-10-26-browser-repair-audit.md`.

The two that were not true are below.

### Defect 1 — a feature-gated binary satisfied the unit gate by existing

`ci/lint-unit-binaries.py` accepted any `[[bin]]` name in any manifest as proof
that a unit's `ExecStart` resolves. `litric-browserd` carries
`required-features = ["broker"]`, so a plain workspace build never produces it
while the checker still reported the unit healthy — the same "exists, therefore
present" reasoning that once shipped an image with an enabled socket and no
broker.

| File | Change |
|---|---|
| `ci/lint-unit-binaries.py` | reads `required-features` from every `[[bin]]`; parses the `cargo build` lines in `tools/build.sh` for package and features (including continuations and `--all-features`); a unit whose binary needs a feature no documented build enables is `MISSING` |
| `ci/lint-unit-binaries.py --self-test` | three new cases: no build line, the right feature, a different feature |

Negative control on the real tree: deleting the broker build line makes the
checker fail on `litric-browser@.service:8` and name `broker`; restoring it makes
the check pass. This is the regression test the brief asked for, and it fails for
the right reason rather than by a hard-coded path.

### Defect 2 — the search-engine choice was remembered for one window

Both routes to the setting — the start page's chips and the browser context menu
— assigned `BrowserApp::engine` and returned no effect. Nothing wrote it and
nothing read it at startup, so DuckDuckGo was Google again on the next launch.

| File | Change |
|---|---|
| `src/litric-shell/src/apps.rs` | `SearchEngine::token()` / `SearchEngine::parse()` — stable storage tokens, not translated titles |
| `src/litric-shell/src/settings.rs` | `Preferences::search_engine` |
| `src/litric-shell/src/prefstore.rs` | `search_engine=` parsed and encoded; an unknown token keeps the default instead of resetting the record |
| `src/litric-shell/src/shell.rs` | `set_search_engine` returns `Effect::PersistPreferences`; `load_preferences` / `save_preferences` keep the live model and the stored record in step, so a failed write rolls the engine back too |
| `docs/audit/2026-10-26-browser-repair-audit.md` | the audit itself: every question, its answer, its evidence, and what was not verified |

### Defect 3 — the frame overwrote the page's own status

`accept_frame` ended with `self.browser.status = "page loaded"`. The `TAB` line
that always precedes a frame had already put the page's real state and title
there (`loaded — Example`), so the frame replaced the title with a generic
sentence, and the status line claimed "page loaded" whatever the line before it
had said. The assignment is gone; the line that knows the state keeps it. The
existing fake-broker test
`the_shell_shows_the_frame_a_broker_sends_and_forwards_a_click` now also asserts
that the title survives the frame.

### Evidence

| Command | Result |
|---|---|
| `cargo build --release --locked -p litric-browser --features broker` | ok; `litric-browserd` produced |
| `cargo test -p litric-browser --features broker` | 85 + 7 passed |
| `cargo test -p litric-shell` | lib 463 passed (460 before; 3 new), bin 32, other targets unchanged |
| `cargo clippy -p litric-shell --all-targets` | clean |
| `cargo fmt --all -- --check` | clean |
| `python3 ci/lint-unit-binaries.py --self-test` | 7 passed |
| `python3 ci/lint-unit-binaries.py` | 16 program references, 0 broken |
| `bash ci/verify.sh` | 26 pass, 0 fail, 4 skipped (golden frames need a Latin face; image, boot and idle need mkosi/QEMU) |

The three new shell tests and what each would catch:

| Test | What it would catch |
|---|---|
| `search_engines_round_trip_through_their_stored_tokens` | a token that is translated, so the saved file depends on the UI language |
| `the_search_engine_round_trips_and_an_unknown_one_keeps_the_default` | a newer build's engine resetting the whole record, or silently becoming a different engine |
| `choosing_a_search_engine_persists_it_and_a_restart_applies_it` | either route to the setting being forgotten at the next launch |

### What this does not establish

| Claim | Why |
|---|---|
| that Chromium starts and a page is captured | no Chromium on this host. The spawn path (`--headless=new`, `--remote-debugging-pipe`, fd 3/4, sandbox left on) is read, not run |
| that the socket activates inside the image | no `mkosi`, no QEMU. `tools/build.sh` and the units are checked statically; the built image was not inspected |
| that the browser works end to end | the honest status is: implemented and unit-tested, not booted. `docs/audit/2026-10-26-browser-repair-audit.md` lists the exact commands that would close it |
| that anything in the brief about PAM, lock screens or the Files app was regressed | those paths were read and found already implemented at HEAD (`StateDirectory=litric-shell litric-browser litric-files`, `Verifier::UnixChkpwd`, `files_root` under `/var/lib/litric-files/root`) and were not modified |

### Rollback

Revert this commit. The unit checker goes back to accepting a `required-features`
binary on the strength of its name, the engine choice goes back to lasting one
window, and the three shell tests and three self-test cases disappear. The audit
document is additive and can stay.

## 2026-10-26 — a settings registry, a deep link, and a ledger that fails CI

The research corpus in `Research-Deep-all/` catalogues ~446 settings panes, 573
shortcut actions and 5,358 design tokens. The shell had 13 pages, no way to find
a setting by its own name, and no record of what it had decided about any of the
rest. This session built the two things that make that inventory usable instead
of overwhelming: a registry the shell resolves through, and a ledger the build
fails on.

### What changed

| Area | Before | After |
|---|---|---|
| finding a setting | the launcher search could reach a **page** ("Accessibility") and never a **row** ("reduce motion") | every stored preference is a registered row, and the search reaches it by title, synonym or page |
| addressing a setting | none | `litric://settings/<page>/<key>` resolves through the registry, in the search field and in code |
| appearance | light / dark / high-contrast, chosen by hand | plus an automatic schedule: dark 19:00–07:00 local, light otherwise, re-resolved on every tick and never persisted as a theme |
| the store and the UI | could drift: a key written by `prefstore::encode` and offered by no control | a two-way test fails if either side gains a key the other does not have |
| decisions about the research | prose in a plan | `docs/parity/ledger.csv` — 41 rows, machine-checked |

### The rules the ledger is held to

`ci/lint-parity.py` fails the build when a row cites a `PENDING` research item,
when an `ADOPT`/`ADAPT` row names no backend (`none`), when an `ADAPT`/`REJECT`/
`DEFER` row's justification is shorter than 40 characters, when the evidence
cannot be opened (a `STATIC` path that does not exist, a `UNIT` test name that
is not a `fn` in the tree), when a `store_key` is not a key the preference store
writes, or when an item appears twice. Its self-test proves it fails on all 16 of
those forms and accepts 5 valid ones.

### Evidence

| Check | Result |
|---|---|
| `cargo test -p litric-shell` | lib **486 passed** (484 before; 2 new), bin 32, other targets unchanged — **547 total, 0 failed** |
| `cargo clippy -p litric-shell --all-targets` | clean |
| `cargo fmt -p litric-shell` | applied, then clean |
| `python3 ci/lint-parity.py` | 41 rows: 20 `ADAPT`, 17 `DEFER`, 4 `ADOPT`, 0 problems |
| `python3 ci/lint-parity.py --self-test` | 16 defects caught, 5 valid forms accepted |
| `python3 ci/lint-doc-refs.py` | 629 citations across 133 documents resolve |
| `bash ci/verify.sh` | **28 pass, 0 fail, 4 skipped** (the four skips are golden frames, image build, boot and idle: no Latin face, no `mkosi`, no QEMU) |

The two new shell tests:

| Test | What it would catch |
|---|---|
| `the_appearance_schedule_follows_the_clock_and_nothing_else` | a schedule that ignores the clock, one that moves a manual theme, or an off-by-one at 07:00/19:00 |
| `opening_a_setting_row_from_search_lands_on_its_page_with_the_row_focused` | a search hit that opens the wrong page, or a highlight that outlives its link |

### What this does not establish

| Claim | Why |
|---|---|
| that the schedule looks right at dusk on a real screen | the test sets the clock; no booted image has been left running across 19:00 |
| that the deep link works from another program | the resolver and the search field are tested; nothing in the image registers a handler for the scheme yet |
| that the 41 ledger rows are *complete* | they are the surfaces this shell owns today plus the ones the plan reaches first. 193 research items are still `PENDING` and may not be cited at all |
| that any `ADAPT` row behaves as its justification says | the linter checks that a backend is named and that the evidence opens. It cannot read the backend |

### Rollback

Revert this commit. The Appearance page loses its schedule row, the search goes
back to finding pages rather than settings, `litric://settings/...` stops
resolving, `theme_schedule` disappears from the record (older builds ignore an
unknown key, so a downgrade is safe), and `ci/lint-parity.py` and
`docs/parity/` are removed. The three corrected gate-matrix rows revert with it.

## 2026-10-26 — a notification queue that outlives the desktop

The notification centre was honest about its limit: the queue lived in the
compositor's memory, so everything raised while the shell was restarting — or
before it started — was lost, and nothing could tell the user it had existed.
This session gave it a daemon with a file. The plan is
[ADR-0025](adr/0025-notification-queue.md); the decisions it records are in
`docs/parity/ledger.csv` (`WA.07.01`–`WA.07.04`, `WB.01.03`, `WB.01.03.01`).

### What changed

| Area | Before | After |
|---|---|---|
| a notification raised while the shell is down | lost | queued in `/var/lib/litric-notifyd/queue.conf`, written atomically before the sender is answered |
| where the queue lives | the compositor's heap | a daemon with its own unit, `StateDirectory`, empty capability set and no network |
| the shell's side | nothing | `TAKE` at startup and on the 2 s facts cadence; adopted notes keep the queue's timestamps and do not raise banners |
| expiry | a ttl in memory | converted to an absolute expiry on acceptance, so a five-second toast cannot reappear after a reboot |
| "should this interrupt?" | one answer (the shell's DND) | still one answer: the queue has no DND flag on purpose |

### Evidence

| Check | Result |
|---|---|
| `cargo test --workspace --locked` | **1492 passed, 0 failed** across 53 test binaries; `litric-notifyd` alone is 46 lib + 7 bin |
| `cargo clippy --workspace --all-targets --locked` | clean (three warnings in the new code found and fixed: a derivable `Default`, a `drain(..).collect()`, a doubled `must_use`) |
| `cargo fmt --all -- --check` | clean |
| `bash ci/verify.sh` | **28 pass, 0 fail, 4 skipped** (golden frames, image build, boot, idle: no Latin face, no `mkosi`, no QEMU) |
| `python3 ci/lint-parity.py` | 45 rows: 23 `ADAPT`, 18 `DEFER`, 4 `ADOPT`, 0 problems |
| `python3 ci/lint-unit-binaries.py` | 17 program references, 7 resolvable, 0 broken — the new unit resolves to a real `[[bin]]` |
| `python3 ci/check-memory-budget.py` | PASS — the unit's `MemoryMax=32M` is now in the ledger as row 17 |
| `python3 ci/lint-file-safety.py` | PASS — the two new deletion sites (a temp file, a stale socket) are recorded with their reasons |

### The binary, run on this host

Not inside a booted image — there is no `mkosi` or QEMU here — but the real
binary, its real socket and its real file:

```text
litric-notifyd --serve --socket $D/notify.sock --state $D/queue.conf &
litric-notifyd --post --app com.litric.browser --title "Download finished" --ttl-ms 60000   -> id=1
litric-notifyd --count                                                                    -> 1
kill %1                                  # the desktop is not running any more
cat $D/queue.conf                        # version=1, next_id=1, one note, absolute expires_ms
litric-notifyd --serve ...               # litric.notifyd.queued=1
litric-notifyd --take                    # NOTE 1 … END, count=1   (what the shell does at startup)
litric-notifyd --take                    # count=0                 (a delivered note is not redelivered)
```

### What this does not establish

| Claim | Why |
|---|---|
| that the unit starts and the queue survives a reboot inside the image | no `mkosi`, no QEMU. The unit is checked statically (`lint-unit-binaries`, `check-memory-budget`) and enabled in `image/mkosi.postinst.chroot`; the boot was not run |
| that the shell adopts from a real booted daemon | the shell's side is unit-tested with real `Note` values, and the client is tested against a real socket in-process. The two have not met inside a guest |
| that a notification arrives within the documented 2 s | the drain runs on the facts cadence and the client has a 250 ms budget; the interval was not measured on a running desktop |
| that the memory cap is right | 32 MB is a ceiling, not a measurement. The queue is 64 notes of a few hundred bytes; nothing measured the resident set |
| that a sender cannot flood other notifications out | the cap evicts oldest-non-critical, so a flood costs other notes rather than memory. Per-sender rate limiting belongs with the capability broker, which has no persisted grants yet (ADR-0025, "what this does not do") |

### Rollback

Revert this commit. The shell goes back to an in-process queue (nothing else
depends on the daemon), `src/litric-notifyd/` and its unit disappear, the
`litric.notifyd` lines leave the boot log, and the ledger rows go back to
`WA.07.01`–`WA.07.04` being absent and `WB.01.03` naming only the shell. A
`queue.conf` left behind is inert: nothing reads it once the unit is gone.

## 2026-10-26 — the V6 desktop, merged into the line that carries the daemons

The interface in this tree is now the one drawn by the `litricos-shell-v6`
bundle (`redesign/shell-v4`, head `7030ce1`) — glass top bar, floating dock with
the original icon artwork, pen-drawn mark, Control Centre of measured cards,
lock screen on the same tokens — merged into the line that grew the daemons and
the research registers. The two halves had not met before: the design branch
stopped at `c28ec35`, and every daemon, ADR and ledger row since was written on
the other side.

### What changed

* **The merge**, five conflicted files. `views.rs`: V6's card sheet, dock lift
  and top-bar chrome, with this line's NETWORK, BLUETOOTH and VOLUME cards
  carried onto the sheet (they read `litric-sysd`, which the design branch never
  had). `shell.rs`: V6's boot timings and cross-fade cover, `TOP_BAR_LOGICAL`
  from `chrome::TOP_BAR_H`. `lib.rs`: both halves' modules. `e2e.rs`: the dock
  contract is V6's — 46-px tiles in a 62-px strip, eight measured hotspots.
  `frame_dump.rs`: `--size` and `--theme`.
* **The lint debt the design branch owed this one.** V6 was written before
  `[workspace.lints]` denied `indexing_slicing`, so 31 sites had to go: the
  quick-settings sheet now walks the cards and their rectangles with one
  iterator (a card's label is the label it was measured from), the icon scaler
  destructures the four channels it asked for, the blend test samples its table
  with `get`. 26 `arithmetic_side_effects` warnings became `saturating_add`,
  except the per-channel lerp in `mix_from`, which keeps plain arithmetic under
  an `#[allow]` that states the bound (255·256 + 255·256 + 128, far inside
  `u32`). Plus `clone_on_copy`, `manual_range_contains`, `type_complexity`, two
  missing doc lines and a dead `radius` parameter that the new pill made dead.
* **The ledger batch.** 85 new rows, all `WA` (Windows 11 shell and system UI),
  which is the chapter this tree has the most surfaces for -- every one of its
  98 citable items is now decided. The ledger is 130 rows over the 400 citable
  research items: 67 `ADAPT`, 43 `DEFER`, 15 `REJECT`, 5 `ADOPT`. The rows that name a backend name one that exists; the
  ones that do not are `DEFER` or `REJECT` with the reason.

### Evidence

| Check | Result |
|---|---|
| `cargo check --workspace --all-targets --locked` | 0 errors, 0 warnings |
| `cargo clippy --workspace --all-targets --locked` | clean |
| `cargo fmt --all -- --check` | clean |
| `cargo test --workspace --locked` | **1560 passed, 0 failed** |
| `bash ci/verify.sh` | **29 pass, 0 fail, 3 skipped** (image build, boot, idle: no `mkosi`, no QEMU, no `pytest`) |
| `python3 ci/check-golden-frames.py --frames build/frames` | PASS — 24 frames, every one within threshold of the re-recorded set |
| `python3 ci/lint-parity.py` | PASS — 130 rows, 0 problems |
| `cargo run --example frame_dump -- --fonts ../build/fonts` | 24 scenes, dark; 24 more with `--theme light` |

The frames are in [`docs/evidence/2026-10-26/`](evidence/2026-10-26/README.md) —
six scenes in both themes, PNGs from the dump, not retouched — and the golden set
in `docs/evidence/golden/` was re-recorded against them, which is why the stale-set
note that was in that README is gone.

### What this does not establish

| Claim | Why |
|---|---|
| that the V6 interface renders on a real display | the frames are `frame_dump`'s composition with a fixed fixture of facts. No framebuffer, no scanout, no boot |
| that the merged daemons and the new chrome coexist in a session | the shell is tested against fixtures and in-process clients; no guest ran the two together |
| that the image builds with this tree | no `mkosi` on this host. `verify.sh` reported the image step SKIP, not pass |
| that the remaining 270 citable research items are decided | the batch covered `WA` only. `WB` (128 of 143), `MC` (0 of 64), `MD` (24 of 39), `G` (0 of 19), `CH00` (0 of 12), `WC` (12 of 14) and `MG` (0 of 11) are still open |

### Rollback

Revert the merge commit. The branch returns to its pre-merge shell (no glass top
bar, the older dock, the boot without a cross-fade), the ledger returns to 45
rows, and the golden fingerprints go back to the stale set that could not be
checked on a host without the image's fonts. Delete
`docs/evidence/2026-10-26/` with it: it documents a tree that would no longer
exist.

## 2026-10-26 — the ledger closes over every citable surface

The parity ledger was 45 rows this morning and 130 after the `WA` chapter. It is
388 now: every item in the research checklist that is finished enough to cite and
that names a surface has a decision.

### What changed

* **Five chapters closed in one pass** — `WB` (Windows Settings, 143), `MC`
  (the macOS desktop model, 64), `MD` (macOS System Settings, 39), `G` (tokens and
  accessibility, 19), `MG` (the HIG, 11), plus the rest of `WC` and `WA`. 288 new
  rows in this commit.
* **The split is the decision, not a placeholder.** 168 `ADAPT`, 126 `DEFER`,
  88 `REJECT`, 6 `ADOPT`. An `ADAPT` names the module that carries it
  (`litric-shell::views::quick_cards`, `litric-sysd::platform::net`,
  `litric-filesd::ops`, `litric-shell::lock`, …). A `DEFER` says which backend is
  missing (`no print subsystem`, `no accelerator address`, `one framebuffer`). A
  `REJECT` says what this system does not do and why it will not claim to — the
  vendor-account, telemetry, assistant and licensing families are the bulk of it.
* **`CH00` is not a surface.** It is the version baseline the research was taken
  against; there is nothing to adopt or reject. The twelve citable items in it are
  deliberately absent, and `docs/parity/README.md` now says so, so the next reader
  does not mistake a decision for a gap.

### Evidence

| Check | Result |
|---|---|
| `python3 ci/lint-parity.py` | PASS — 388 rows, 0 problems (every row sourced, backed and evidenced) |
| `python3 ci/lint-doc-refs.py` | OK — 661 citations across 138 documents resolve |
| `python3 ci/lint-line-endings.py` | OK — 338 files, all LF |
| `python3 ci/lint-parity.py --self-test` | PASS — the checker still fails an inert row |

No code changed in this commit, so `cargo test`, `clippy` and `ci/verify.sh` were
not re-run for it; the run recorded in the section above is the one that covers
the tree it describes.

### What this does not establish

| Claim | Why |
|---|---|
| that an `ADAPT` row's backend behaves as the justification says | the linter checks a row is well-formed, sourced and openable, not that the code does what the sentence claims. The row's evidence column is how a reader checks one |
| that a `DEFER` will be built | it is a promise with a milestone, not a plan item with an owner. `M6` is where the ones nobody built become `REJECT` |
| that the `PENDING` chapters are covered | they are not citable. `F` (28), `H` (21), `I` (12), `J` (9) and the eight Linux desktops (120) are Track R's research work |

### Rollback

Revert this commit. The ledger returns to 130 rows — the `WA` chapter and the
earlier scattered decisions — and the coverage tables in the plan and the parity
README go back with it. Nothing else reads the ledger yet, so no code changes.

## 2026-10-26 — reviewing chapter F, and the substring count that lied

The plan's Track R queue said 56 of the 193 `PENDING` research items were "written
up already", chapter `F` among the best covered at 12 of 28. This is the review
that checked, and it found the number was an artifact of the way it was counted.

### What changed

* **`F` has one researched item, not twelve.** The corpus's `F` section
  (`Part1` §"F — Research F — Shared component catalog") is the checklist table
  itself: 27 rows of `PENDING` and no prose. What the count matched were headings
  in other chapters that happened to contain the word — "Buttons" inside a toast
  lifecycle heading, "Menus" inside a File Explorer one. A substring is not a
  section.
* **`F.16 Sheets` is `PARTIAL`, and only that.** `Part6` §MC.04.06 carries
  Apple's HIG guidance on sheets — eight elements, four behavioural guidelines, a
  named source — so the item has one platform of the three the catalogue needs.
  Its note says exactly that, and it has a ledger row (a sheet's role is the
  shell's dialog component, `ADAPT`, `litric-shell::apps::DialogKind`).
* **The other 27 carry the audit instead of a status.** Each note now reads
  "Reviewed 2026-10-26: the corpus has no section for this item; stays PENDING
  until one exists." A review that writes down what is missing is worth more than
  one that guesses, and the next reviewer starts from the note rather than the
  heading.
* **The plan's queue is corrected to what the stricter count finds**: 24 items
  with a section that names them (by id or exact title), not 56. `F` drops from
  12 to 1; the Linux desktops from 4 to 2 each.

### Evidence

| Check | Result |
|---|---|
| checklist statuses | `PARTIAL` 382 → 383, `PENDING` 193 → 192, total still 597 |
| `python3 ci/lint-parity.py` | PASS — 389 rows, 0 problems |
| `python3 ci/lint-doc-refs.py` | OK — 663 citations across 138 documents |
| the `F` section as it stands | `Part1` line 793: a table of 27 `PENDING` rows and no prose |

No code changed, so `cargo test`, `clippy` and `ci/verify.sh` were not re-run.

### What this does not establish

| Claim | Why |
|---|---|
| that `F` will be researched | 27 items are still nobody's work. This review only proved they are not done and wrote that down |
| that `F.16` is fully documented | one platform of three. A components catalogue that knows only macOS sheets cannot decide a cross-platform sheet |
| that the corrected 24-item queue is complete | it counts headings that name an item. A section that answers an item under another name is still invisible to it |

### Rollback

Revert this commit. `F.16` returns to `PENDING` (and its ledger row with it), the
27 notes return to empty, and the plan's Track R table returns to the 56/12
numbers -- which were wrong, and which the next reviewer would have had to
discover again.
## 2026-10-26 -- the four components the shell already has

The review above ended with "27 items are still nobody's work". Four of them are
now somebody's: `Part 10` compares Windows 26H2, macOS 27 and GNOME 51 guidance
with the shell's own component layer, item by item, and every claim about the
shell in it can be checked by opening `src/litric-shell/src/ui.rs`.

### What changed

* **A new research part, `Part 10`, with four items.** `F.01` buttons,
  `F.03` switches, `F.06` sliders and `F.19` lists each get platform guidance
  with tiered sources, the shell's implementation read out of the source with
  `MEASURED` values, a delta table, and a status. Four of the 27 are researched;
  the other 23 are untouched.
* **Two of the four components are dead code, and nothing said so.** `ui::slider`
  and `ui::icon_button` have no call site anywhere in the shell. The volume
  control -- the canonical slider on two of the three platforms -- is two step
  buttons and a mute button; every icon-only control calls `icons::draw` at its
  own site and inherits none of the component's states. "The shell has a slider"
  was true of the component layer and false of the product.
* **Two state defects.** A disabled **off** switch renders pixel-identical to an
  enabled off switch (the disabled branch only lowers the *on* fill), which
  contradicts the module's own contract in its header comment; and `ui::slider`
  ignores `Disabled`, `Error`, `Loading` and `Selected` entirely, so a slider
  nobody can move draws exactly like one they can.
* **Keyboard focus is drawn as hover.** `Shell::is_highlighted`
  (`src/litric-shell/src/shell.rs`) merges the pointer and the keyboard focus, and
  no surface asks for `State::Focused` -- the focus ring in `ui::focus_ring` is
  live code with no caller in the product. Windows documents a distinct
  high-visibility focus visual and GNOME documents Tab/Shift+Tab with Return to
  activate, so this is a delta against both, found by searching for the variant
  rather than by looking at a screen.
* **Numbers where the platforms give none.** Microsoft publishes one button
  minimum (120 px) and one target (7.5 mm / 40 px); Apple publishes 44 x 44 pt;
  GNOME publishes no numbers. The shell's targets are 34 x 22 px step buttons,
  a 110 px scan button, and 30 px Settings rows -- all measured from the call
  sites and the token file, none estimated.
* **Sources registered.** 31 new rows in `Research-Deep-all/LitricOS Source Register.csv`,
  including the Apple pages as `SNIPPET_ONLY`: the reader returned titles and
  summaries for those, so the macOS column is the weakest of the three and says so.

### Evidence

| Check | Result |
|---|---|
| checklist statuses | `PARTIAL` 383 -> 387, `PENDING` 192 -> 188, total still 597 |
| `python3 ci/lint-parity.py` | PASS -- 389 rows, 0 problems |
| `python3 ci/lint-doc-refs.py` | OK -- 666 citations across 138 documents |
| `python3 ci/lint-line-endings.py` | OK -- 338 files, all LF |
| `python3 ci/check-python-syntax.py` | OK -- 48 files parse |

No code changed, so `cargo test`, `clippy` and `ci/verify.sh` were not re-run.
The two state defects above were read out of the source and are recorded, not
fixed: fixing them is a code change with its own tests, and neither has been
decided yet.

### What this does not establish

| Claim | Why |
|---|---|
| that any of the four components is correct | the part compared guidance with an implementation and decided nothing. Eight findings, zero decisions |
| that the macOS column is as good as the other two | four Apple pages returned summaries; one item's detail came through a third-party mirror |
| that the shell's state matrix matches a platform | none of the three publishes a full rest/hover/pressed/disabled/focus specification for these controls in the pages read |
| that anything was seen on screen | no image was booted and no pixel measured. Every rendering claim is a claim about the code that draws it |
| that `F` is nearly done | 23 of 27 items remain `PENDING` |

### Rollback

Revert this commit. The four statuses return to `PENDING` with their audit notes,
`Part 10` disappears, the 31 source rows go with it, and the plan's counts return
to 192/383 -- along with the four comparisons, including the one that found the
dead slider, which the next reviewer would have had to rediscover by counting call
sites that nobody had counted.

## 2026-10-26 — text, progress, search and errors, read against the layer that draws them

Part 11 of the research corpus continues chapter `F` on the four items that are
either the last two primitives in the component layer or the places where a
primitive is missing. It compares Windows 26H2, macOS 27 and GNOME 51 guidance
with `ui::text_field`, `ui::progress`, the shell's input path and its five search
surfaces, and sets `F.02`, `F.18`, `F.23` and `F.26` to `PARTIAL`.

### What it found

| Finding | Where |
|---|---|
| The shell has a full single-line editor (`TextEdit`: caret, selection, Home/End, word delete, 8192-char limit) and reaches it with **one Backspace in one dialog**. Every other field is a `String` appended at the end, so clicking mid-name and typing puts the character at the end, and Home/End/Ctrl+A/Delete do nothing | `apps.rs` line 1832, `shell.rs` line 4774, six `push_str` sites |
| The field renderer takes `(text, placeholder, focused)` and cannot be told where the caret is or what is selected. The one dialog with a caret model still draws its caret at the end | `ui.rs` line 599 |
| A magnifier is drawn in New Folder and Rename; the generic field is the only search affordance the two filter fields have | `views.rs` lines 339, 1450 |
| No filter field has a clear button, and the launcher's ✕ (`Hot::DockCollapse`) closes the launcher instead of clearing the query | `shell.rs` line 5521 |
| `Ctrl+F` in Files focuses the table, not the filter; no search surface has a keyboard route | `shell.rs` line 4896 |
| Progress is one determinate bar with a count label, no indeterminate form and no spinner; the `total == 0` guard draws an empty track during a live scan, and `ui::badge` has no call site | `views.rs` line 4072, `ui.rs` line 509 |
| Validation is a round-trip (and, on a booted image, the daemon's): a failure replaces the form, loses the typed name, elides the reason to one line and acknowledges with a danger-toned OK | `shell.rs` line 2360, `views.rs` lines 1422, 1442 |
| No limit ties the name field to the filesystem's 255-byte segment; the live path is unbounded | `fsbridge.rs` line 94, `textedit.rs` line 53 |

### What changed

| File | Change |
|---|---|
| `Research-Deep-all/LitricOS UI UX Research Part 11 F Text Progress Search and Errors.md` | new: 457 lines, 16 findings, 0 decisions |
| `Research-Deep-all/LitricOS Coverage Checklist.csv` | `F.02`, `F.18`, `F.23`, `F.26` set to `PARTIAL` with notes that name the part and the finding |
| `Research-Deep-all/LitricOS Source Register.csv` | 24 new rows (Windows, Apple and GNOME/GTK sources; rows that Part 10 had already registered were merged rather than re-added); 13 existing rows had `F.02`/`F.18`/`F.23`/`F.26` added to their checklist ids |
| `Research-Deep-all/LitricOS UI UX Research Part1.md` | chapter 40's index row names both parts and the eight researched items |
| `docs/master-plan/27-research-driven-plan.md` | file table, status counts, citable-by-chapter row, Track R queue and a Part 11 subsection |
| `docs/parity/README.md` | `PENDING` count 188 -> 184 |
| `docs/GATES.md` | this entry |

### Evidence

| Check | Result |
|---|---|
| checklist statuses | `PARTIAL` 387 -> 391, `PENDING` 188 -> 184, total still 597 |
| `python3 ci/lint-parity.py` | PASS -- 389 rows, 0 problems |
| `python3 ci/lint-doc-refs.py` | OK -- 677 citations across 138 documents |
| `python3 ci/lint-line-endings.py` | OK -- 338 files, all LF |
| `python3 ci/check-python-syntax.py` | OK -- 48 files parse |

No code changed, so `cargo test`, `clippy` and `ci/verify.sh` were not re-run.
Every claim in Part 11 is a reading of a named line; the two that would need a
keyboard (mid-text editing and Ctrl+F) were read from the handlers, not typed.

### What this does not establish

| Claim | Why |
|---|---|
| that any of the four items is correct | the part compared guidance with an implementation and decided nothing. Sixteen findings, zero decisions |
| that the macOS column is as good as the other two | every Apple page returned a title and a summary; no Apple number or state rule is reported as verified |
| that Thai can be typed | the model documents per-`char` editing and there is no preedit surface. No IME test ran |
| that anything was seen on screen | no image was booted and no pixel measured. Every rendering claim is a claim about the code that draws it |

### Rollback

Revert this commit. The four statuses return to `PENDING` with their audit notes,
`Part 11` disappears, the 24 source rows go with it, and the plan's counts return
to 188/387 -- along with the comparison that found the editor model unused, which
the next reviewer would have had to rediscover by reading six call sites one at a
time.

## 2026-10-26 — the checkbox that is a circle, and the radio that does not exist

Part 12 continues chapter `F` on the two selection controls. It compares Windows
26H2 and GNOME 51 at page depth (Apple's checkbox and radio pages returned titles
and summaries only, so the macOS column is an API capability and nothing more)
with the single hand-drawn checkbox in the product and the chip-based single-select
surfaces that stand in for a radio group. `F.04` checkboxes and `F.05` radio
controls become `PARTIAL`.

### What it found

| Finding | Where |
|---|---|
| The only multi-select checkbox is a **circle**: a 12 x 12 box at `Radius::Sm` = 6, which is half the side | `views.rs` line 1243 |
| Its hit target is 18 x 18 px, inside a 24 px row | `views.rs` line 1281, 1147 |
| No indeterminate/mixed state and no header/select-all box; `Ctrl+A` selects all but has no visible control | `views.rs` lines 1243-1280; `apps.rs` line 653 |
| Space does not toggle it. Enter activates the focused hotspot; the shell's only Space handling is the lock screen's wake | `shell.rs` line 3933 |
| Two Tab stops per row (the row and its box): a 24-row folder is 48 stops, and there is no arrow-key list model | `shell.rs` line 445; `views.rs` lines 1242, 1277 |
| No shift-range or ctrl-click selection; the click path takes no modifiers | `apps.rs` line 621 |
| **No radio control exists at all**, and no dot-in-ring is drawn; the strings `radio`/`Radio` in the shell mean the Bluetooth radio | grep: no radio control, no `stroke_circle` |
| The three-way search-engine choice has two chip designs (96 x 26 on the start page, 18 px in Settings) and only one has a group label | `views.rs` lines 1986-2010, 2064-2100 |
| Both chip designs rest on colour alone, and the start-page comment claims a check glyph that the loop never draws | `views.rs` line 1985 vs lines 1986-2010 |
| Three chips are three Tab stops, with no arrow navigation and no selection-follows-focus | both chip loops; `shell.rs` line 445 |
| "Reduce motion" is a switch in Quick settings and a state-in-label accent button in Settings and first-run | `views.rs` lines 2930, 4907, 5654 |

The shape mismatch is the item's headline: a control whose form says "choose one"
and whose behaviour says "choose several", sitting in the one table that also has a
single-selection state. It is recorded, not changed.

### What changed

| File | Change |
|---|---|
| `Research-Deep-all/LitricOS UI UX Research Part 12 F Checkboxes and Radio Controls.md` | new: 331 lines, 13 findings (25-37), 0 decisions |
| `Research-Deep-all/LitricOS Coverage Checklist.csv` | `F.04` and `F.05` set to `PARTIAL` with notes that name the part and the findings |
| `Research-Deep-all/LitricOS Source Register.csv` | 17 new rows (Windows, GNOME, GTK, Apple, two standards); 18 existing rows gained `F.04`/`F.05` in their checklist ids |
| `Research-Deep-all/LitricOS UI UX Research Part1.md` | chapter 40's index row names all three parts and the ten researched items |
| `docs/master-plan/27-research-driven-plan.md` | file table, status counts, citable-by-chapter row, Track R queue and a Part 12 subsection |
| `docs/parity/README.md` | `PENDING` count 184 -> 182 |
| `docs/GATES.md` | this entry |

### Evidence

| Check | Result |
|---|---|
| checklist statuses | `PARTIAL` 391 -> 393, `PENDING` 184 -> 182, total still 597 |
| `python3 ci/lint-parity.py` | PASS -- 389 rows, 0 problems |
| `python3 ci/lint-doc-refs.py` | OK -- 677 citations across 138 documents |
| `python3 ci/lint-line-endings.py` | OK -- 338 files, all LF |
| `python3 ci/check-python-syntax.py` | OK -- 48 files parse |

No code changed, so `cargo test`, `clippy` and `ci/verify.sh` were not re-run. The
one behavioural test that touches the checkbox (`e2e.rs` line 317) was not run for
this documentation-only commit; nothing here alters the code it exercises.

### What this does not establish

| Claim | Why |
|---|---|
| that a circular checkbox is wrong | the part records the mismatch with every platform's square box and with the control's own multi-select behaviour. A deliberate disc is possible, just undocumented |
| that chips should be radios | chips are a legitimate single-select pattern. The finding is that they are the only one the shell has |
| that the macOS position is known | Apple's checkbox and radio pages did not load; only the toggles summary and the NSButton mixed-state API were readable |
| that a screen reader would mis-announce the box | there is no accessibility tree. The claim is that it cannot announce it, not that a failure was observed |
| that `F` is nearly done | 17 of 27 items remain `PENDING`, ten are `PARTIAL`, none is `COMPLETE` |

### Rollback

Revert this commit. The two statuses return to `PENDING` with their audit notes,
`Part 12` disappears, the 17 source rows go with it, and the plan's counts return
to 184/391 -- along with the comparison that found the circle, which the next
reviewer would have had to notice by reading a radius argument against a width.

## 2026-10-26 — five implementations of three containers, and no shared component

`Part 13` of the research corpus compares Windows 26H2, macOS 27 and GNOME 51
guidance for tabs, sidebars and toolbars with the five places the shell draws them,
and sets `F.08`, `F.10` and `F.11` to `PARTIAL`. This entry records what it found
and what was run; no source file changed.

### What was found

| Finding | Where |
|---|---|
| The tab strip is fixed 150 px with **no overflow policy**: no scroll, no cap, no clip, so the `+` button leaves the window past roughly `body.w / 152` tabs | `views.rs` lines 1575–1621 |
| Tab active state is a material fill only; same 13 px text, same weight, no accent edge or underline | `views.rs` lines 1584–1592 |
| No tab keyboard model: `app_key` handles Escape and Enter; no Ctrl+T/W, Ctrl+Tab, Ctrl+1–9 or arrow movement, and **Ctrl+T cycles the theme in Settings** | `shell.rs` line 4699; `views.rs` lines 1608, 1618 |
| The tab context menu has exactly two items (Reload, Close tab); no duplicate, pin, mute, move-to-window, reorder or tear-out | `apps.rs` lines 1738–1741 |
| Title truncation is by **character count** (20 -> 17 + `...`), not measured width, so 17 wide glyphs overflow a 150 px tab | `views.rs` lines 1601–1607 |
| Two sidebars with no shared component: Files 132 px / 20 px rows / fill-only vs Settings 36 % / 30 px rows / icon + marker | `views.rs` lines 957, 2337 |
| The Settings sidebar is 36 % of the body with no collapse, resize or scroll, over 414 px of rows | `views.rs` lines 2337–2360 |
| Neither sidebar has an arrow-key model; one Tab stop per row | `shell.rs` line 445; `views.rs` lines 980, 2341 |
| Three toolbars, three overflow policies: wrap + scale (browser), silent drop (Files), fixed widths (Trash) | `views.rs` lines 1702–1723, 1083, 4172 |
| Browser overflow splits by half the item count (`div_ceil(len/2)`), not by importance, and scaling shrinks the target with the label | `views.rs` lines 1702, 1723 |
| Files removes controls that do not fit and draws inapplicable actions (Paste, Compress, Extract, Empty) as enabled | `views.rs` lines 1083, 1102–1136 |
| Toolbars are label-only at 22–24 px with no tooltips, access keys, accelerators or separators | `views.rs` lines 1676–1755, 1032–1136 |
| "Empty permanently?" and "Delete forever?" are longer than their 112 px fixed-width buttons and are centred with overhang | `views.rs` lines 4189–4200; `ui::button` |

The structural finding: `ui.rs` exists so two windows cannot invent two versions of
one control, and both the sidebars and the toolbars did exactly that.

### What changed

| File | Change |
|---|---|
| `Research-Deep-all/LitricOS UI UX Research Part 13 F Tabs Sidebars and Toolbars.md` | new: 329 lines, 12 findings (38–49), 0 decisions |
| `Research-Deep-all/LitricOS Coverage Checklist.csv` | `F.08`, `F.10`, `F.11` set to `PARTIAL` with notes naming the part and the findings |
| `Research-Deep-all/LitricOS Source Register.csv` | 27 new rows (Windows, GNOME, GTK, Apple); 12 existing rows gained the three ids |
| `Research-Deep-all/LitricOS UI UX Research Part1.md` | chapter 40's index row names Parts 10–13 and the thirteen researched items |
| `docs/master-plan/27-research-driven-plan.md` | file table, status counts, citable-by-chapter row, Track R queue, a Part 13 subsection, and a missing space repaired |
| `docs/parity/README.md` | `PENDING` count 182 -> 179 |
| `docs/GATES.md` | this entry |

### Evidence

| Check | Result |
|---|---|
| checklist statuses | `PARTIAL` 393 -> 396, `PENDING` 182 -> 179, total still 597 |
| `python3 ci/lint-parity.py` | exit 0 -- 389 rows (169 `ADAPT`, 126 `DEFER`, 88 `REJECT`, 6 `ADOPT`), no problems reported |
| `python3 ci/lint-doc-refs.py` | OK -- 681 citations across 138 documents |
| `python3 ci/lint-line-endings.py` | OK -- 338 files, all LF |
| `python3 ci/check-python-syntax.py` | OK -- 48 files parse |

No code changed, so `cargo test`, `clippy` and `ci/verify.sh` were not run. Nothing
here exercises a compiled path.

### What this does not establish

| Claim | Why |
|---|---|
| that tabs visibly run off the screen | the loop was read, not a resized window. The claim is that nothing bounds the strip; the pixels follow from the window clip |
| that a sidebar should be 320 px | Windows' 320 DIP is a default, not a rule; libadwaita's 180–280sp is not HIG. The finding is that the shell has no width policy in either direction |
| that the toolbars should be icon-first | GNOME prefers backgroundless icons; Windows keeps icon + label and hides the label when compact. The finding is that there is no overflow at all, not that labels are wrong |
| that macOS agrees with any of this | every Apple page returned a title and a summary, and the toolbar-items page returned 404. No macOS number is reported as verified |
| that any of it was keyboard- or screen-reader-tested | no keyboard was used; the claims are readings of `app_key` and the hotspot table |
| that `F` is nearly done | 14 of 27 items remain `PENDING`, thirteen are `PARTIAL`, none is `COMPLETE` |

### Rollback

Revert this commit. The three statuses return to `PENDING`, `Part 13` disappears
with its 27 source rows, and the plan returns to 182/393 -- along with the record
that the shell draws five containers with no shared component and has no tab
overflow policy, which the next reviewer would have to rediscover by reading
`draw_browser`'s strip loop against the body width.

## 2026-10-27 — a menu bar with no menus, and six context menus the keyboard cannot open

`Part 14` of the research corpus compares Windows 26H2, macOS 27 and GNOME 51
guidance for menus and context menus with the three menu surfaces the shell draws,
and sets `F.12` menus and `F.13` context menus to `PARTIAL`. It is the first part
measured against the shell's own specification (`docs/DESIGN-SYSTEM-v3/`) as well
as the platforms'. This entry records what it found and what was run; no source
file changed.

### What was found

| Finding | Where |
|---|---|
| Three labels in menu position open **no menu**: Apps opens the launcher overlay, Workspace opens the Activity window, Environment opens the quick-settings panel | `views.rs` lines 527–568; `shell.rs` lines 5357–5358, 5423–5424 |
| "Workspace" opens a window from the bar, which the shell's own spec forbids: "click opens a popover — never a new window from the bar" | `shell.rs` line 5423; `DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` line 30 |
| Labels that do not fit are replaced by one `›` whose hotspot is `Hot::MenuApps`, so a dropped label is unreachable from the bar | `views.rs` lines 566–579 |
| The two real menus (LitricOS 2 rows, Power 4 rows, 34 px) have **no keyboard binding**, no open state on their trigger, no grouping, no disabled or checked row, one Power glyph for sleep/shut down/restart, and no ellipsis on the three rows that confirm | `views.rs` lines 5267–5405; `shortcuts.rs` lines 563, 681 |
| **No keyboard path into or out of a context menu**: `Key::Menu` and `F10` are decoded and unbound, Shift+F10 resolves to nothing, and Escape resolves to `CloseOverlay`, which does not touch `context_menu` | `keys.rs` line 76; `shortcuts.rs` line 690; `shell.rs` line 5900 |
| Only a wallpaper click or a chosen row dismisses a context menu; clicking a window leaves it floating, and a second right-click re-derives the surface under the pointer | `shell.rs` lines 5782–5796, 5841–5879 |
| Files "New folder" — toolbar and both menus — always writes `/Documents`, because `perform_menu` reads the `context_menu` that `Hot::MenuRow` cleared one line earlier; `Ctrl+Shift+N` is the only correct path | `shell.rs` lines 5793–5796, 2371–2384, 2652–2663; `views.rs` line 1096 |
| No disabled, checked or radio state in any of the 28 rows; no icon, shortcut or access key; four rows that confirm carry no ellipsis; destructive rows sit mid-menu and untinted | `apps.rs` lines 1595–1601, 1702–1747; `views.rs` lines 216–238 |
| Two menu metrics and two hover alphas: 26 px / 13 px / fixed 190 px / 9 % on the desktop against 34 px / 15 px / measured / 6 % at the bar | `views.rs` lines 184–232, 5291, 5379 |
| Context-menu labels bypass i18n entirely; `"New folder"` is in the Thai dictionary and drawn in English at five call sites, and the bar menus miss five of their six labels | `apps.rs` lines 1702–1747; `i18n.rs` lines 109–130, 298 |
| The row test asserts no panic rather than an effect, so a silent no-op row (Restore with nothing selected) passes | `e2e.rs` lines 1371–1393 |
| A right-click on a live page goes to Chromium; the same gesture opens the shell's Browser menu when no frame exists | `shell.rs` lines 5199–5202, 1947–1965 |

The structural finding: menus are the surface the shell uses most and names least.
There is no menu component in `ui.rs`; three bespoke loops produce three row
metrics, and the one dismissal mechanism every platform shares — Escape — exists
for popovers and not for context menus.

### What changed

| File | Change |
|---|---|
| `Research-Deep-all/LitricOS UI UX Research Part 14 F Menus and Context Menus.md` | new: 377 lines, 17 findings (50–66), 0 decisions |
| `Research-Deep-all/LitricOS Coverage Checklist.csv` | `F.12`, `F.13` set to `PARTIAL` with notes naming the part and the findings |
| `Research-Deep-all/LitricOS Source Register.csv` | 27 new rows (Microsoft Learn, Apple HIG and Apple Support, GNOME HIG) |
| `Research-Deep-all/LitricOS UI UX Research Part1.md` | chapter 40's index row names Parts 10–14 and Part 6's `F.16`, with 11 items `PENDING` |
| `docs/master-plan/27-research-driven-plan.md` | file table, status counts and shares, citable-by-chapter row, Track R queue and its stale total, a Part 14 subsection, and the source-register count corrected 3,979 -> 4,105 |
| `docs/parity/README.md` | `PENDING` count 179 -> 177 |
| `docs/GATES.md` | this entry |

### Evidence

| Check | Result |
|---|---|
| checklist statuses | `PARTIAL` 396 -> 398, `PENDING` 179 -> 177, total still 597 |
| `python3 ci/lint-parity.py` | exit 0 -- 389 rows (169 `ADAPT`, 126 `DEFER`, 88 `REJECT`, 6 `ADOPT`), no problems reported |
| `python3 ci/lint-doc-refs.py` | OK -- 687 citations across 138 documents |
| `python3 ci/lint-line-endings.py` | OK -- 338 files, all LF |
| `python3 ci/check-python-syntax.py` | OK -- 48 files parse |

No code changed, so `cargo test`, `clippy` and `ci/verify.sh` were not run. Nothing
here exercises a compiled path.

### What this does not establish

| Claim | Why |
|---|---|
| that the bar labels read as menus to a user | no user was asked. The finding is what they do, not how they look |
| that a bar label is dropped at a plausible window width | the layout loop was read, not a resized window; the claim is that nothing in the loop preserves the label |
| that the Files "New folder" bug was captured on a booted image | the code path is unambiguous but untested; it is recorded as a reading of `perform_menu` and `submit_desktop_dialog` |
| that Escape leaves a menu open in every focus state | the call sites that clear `context_menu` are exhaustive for the code as written, not observed on a frame |
| that macOS agrees with any of this | every Apple HIG page returned a title and a summary; the macOS column is roles plus two indexed rules, both marked `SNIPPET_ONLY` |
| that `F` is nearly done | 11 of 27 items remain `PENDING`, sixteen are `PARTIAL`, none is `COMPLETE` |

### Rollback

Revert this commit. The two statuses return to `PENDING`, `Part 14` disappears with
its 27 source rows, and the plan returns to 179/396 -- along with the record that
the bar's three labels open no menu, that neither popover menu has a key, and that
Files "New folder" writes `/Documents` regardless of the folder on screen, all of
which the next reviewer would have to rediscover by reading `menu_surface_at`
against `perform_menu`.
