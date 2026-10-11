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
