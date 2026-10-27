# AI HANDOFF — LitricOS: build the image and the installer from this tree

**ข้อความถึง AI ตัวถัดไป:** ในกล่องนี้คือทุกโฟลเดอร์ทุกไฟล์ของ LitricOS
รวม `image/`, `iso/`, `kernel/`, `keys/`, `src/`, `docs/`, `tools/`, `tests/`,
`ci/`, `evidence/`, `artifacts/`, `Research-Deep-all/`, `Saved/`, `package-f10/`
และไฟล์ซ่อนทั้งหมดของคุณ งานของคุณคือ **สร้าง image + bootable installer
จากซอร์สนี้ แล้ว boot ทดสอบใน QEMU และรายงานตามจริง** — ห้ามรายงานผลจาก
การอ่านโค้ดเพียงอย่างเดียว

- Date: 27/10/69 (2026-10-27)
- Repository: `https://github.com/audijuju1-a11y/LitricOS` — branch `main`, commit `cf6b523`
- This package: `LitricOS-27-10-69.tar.gz` — a snapshot of every folder and file
  of the workspace at this date, including the earlier root-level backup
  archives (`my-backup.tar.gz`, `my-LitricOS.tar.gz`, `all-files.tar.gz`,
  `package-f10.tar.gz`), which are older backups of this same tree. The only
  things outside the archive are the archive itself, its generated file list
  (`LitricOS-27-10-69.filelist.txt`) and its checksum file.

## What you must do

Build the LitricOS disk image and a bootable installer (disk image + live ISO)
from this source tree, boot it in QEMU, and report exactly what works and what
does not. Do not stop at analysis. Do not claim a result you did not measure.

## Rules that are not optional (from `00-MASTER-WORKFLOW.md`)

- **G1** — the host OS is untouched. All work happens in QEMU/VM. Never run a
  partitioning tool against a physical disk.
- **G2** — measurement > assumption. No performance claim without a number and
  a repeatable command.
- **G3** — no fabricated results. Not run = "NOT RUN". Failed = show the output.
- Read the actual file before trusting any summary, including this one.

## Reading order

1. `00-MASTER-WORKFLOW.md` — role, invariants, gate system, response format.
2. `docs/master-plan/README.md` and `docs/master-plan/25-beta-1.0-gate-matrix.md`
   — the plan and the definition of done.
3. `docs/GATES.md` — every gate result recorded so far.
4. `docs/SYSTEM-SCOPE.md` — what is implemented vs documented target.
5. `Saved/README.md` — a source-grounded audit of this tree (it states the
   overall release state as **NOT_VERIFIED**).
6. `docs/PERF-budget.md` — the RAM/CPU budget ledger.

## Where things are

| Path | What it holds |
|---|---|
| `src/` | Cargo workspace: shell, services (`sysd`, filesd, notifyd), browser, Defender, compositor protocol… (`src/target` is build cache) |
| `image/` | mkosi image definition, profile stack (`nano`/`core`/`desktop`/`gaming`), postinst, systemd units, repart geometry |
| `iso/` | live ISO assembly + systemd-boot entries (live, safe, noscan, recovery) |
| `kernel/config/litric-base.config` | the kernel config |
| `keys/dev/` | **development** Secure Boot / verity / PCR keys, needed by the build — never ship them in a production image |
| `tools/` | `build.sh` (mkosi image), `build-image.sh`, `run-qemu.sh`, `desktop-app.sh`, probes |
| `tests/`, `ci/` | boot smoke test, `ci/verify.sh` lint/verify suite |
| `docs/` | ADRs, master plan 00–27, gates, parity ledger, bugs, design system, evidence |
| `evidence/`, `artifacts/` | captured frames, logs, screenshots |
| `Research-Deep-all/` | UI/UX research corpus |
| `Saved/` | third-party repository audit |
| `00-…` … `15-APPENDIX-…` (root) | the phase spec set |

## How to build (the commands this tree already ships)

Prerequisites (Arch names; Debian/Ubuntu equivalents are printed by the scripts):

- `mkosi` >= 25.3 (systemd >= 256), run with root privileges
- `qemu-system-x86_64` + OVMF (`OVMF_CODE_4M.fd`, `OVMF_VARS_4M.fd` in `build/`)
- `xorriso` >= 1.5.4, `mtools`, `sbsigntools`, `zstd`, `e2fsprogs`
- Rust toolchain for the Cargo workspace under `src/`

Step 1 — dry run (runs every preflight check, builds nothing):

    ./tools/build.sh --profile desktop --dry-run

Step 2 — build the disk image:

    ./tools/build.sh --profile desktop        # nano | core | desktop | gaming

Output lands under `build/out/` (image plus split UKI / verity artifacts).

Step 3 — master the bootable live ISO:

    ./iso/assemble-iso.sh                     # needs build/out/ from step 2

Output: `build/out/litricos-<profile>-<ver>-x86_64.iso` (+ `.sha256`).
`tools/build-image.sh` is an older full-image script and is marked DESIGN-ONLY.

Step 4 — boot it in QEMU:

    ./tools/run-qemu.sh                       # KVM; add --tcg when no KVM

It needs `build/litric.qcow2` plus the OVMF files. Serial log:
`tools/logs/boot-<timestamp>.log`.

## Current state (verify it yourself before relying on it)

- 2026-10-27, on a cloud build host, at commit `cf6b523`:
  `cargo test --workspace`, `cargo clippy --workspace --all-targets` and
  `cargo fmt --check` pass; `ci/verify.sh` reports 30 pass / 0 fail / 3 skipped.
  The skips are image build, boot and idle-RAM checks — that host had no mkosi
  and no QEMU.
- **NOT verified:** the image build, the ISO, and any boot. No install, no
  hardware test. The gate matrix still marks the release `NOT_VERIFIED`.
- This tree's own rule: a file that exists is not evidence. Only a build and a
  boot you observed count.

## When you report back

Use the mandatory format (`00-MASTER-WORKFLOW.md` §5): OBJECTIVE, WHY,
ARCHITECTURE, ASSUMPTIONS, IMPLEMENTATION, FILES/MODULES, SECURITY,
PERFORMANCE, TEST PLAN, FAILURE MODES, ROLLBACK, MEASURE, NEXT STEP.
Record every gate number into `docs/GATES.md` with date, commit and raw output.
