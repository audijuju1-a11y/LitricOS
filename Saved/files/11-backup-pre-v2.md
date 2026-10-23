# Batch B11: Historical `.backup-pre-v2/` snapshot

This report catalogs 40 retained files. They are a historical pre-v2 planning snapshot, not current source-of-truth contracts. Current source and current ADR/docs take precedence where the backup diverges.

## `.backup-pre-v2/` (18 entries)

### `.backup-pre-v2/00-MASTER-WORKFLOW.md`

- Inventory `F0002`; 7926 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 180 lines; SHA-256 `9fd14220b4b0a18b1a27b106cd1cd9160bc5089036225e84663ba6ce24abe59f`.
- Summary: --- Every work session = paste **two** files into a fresh AI session: Topics: ILTRIC OS — MASTER WORKFLOW PROMPT (PHASE CONTROLLER), 0. HOW TO USE THIS PACK, 1. ROLE, 2. GLOBAL INVARIANTS (NEVER VIOLATE), 3. LOCKED TECHNICAL BASELINE (decided — do not re-litigate without evidence), 4. STA

### `.backup-pre-v2/01-PHASE-00-ARCHITECTURE.md`

- Inventory `F0003`; 4670 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 100 lines; SHA-256 `9ee7ab06e4b12b6097c406a1584b3ef67666db54ce85f88008fb40de12aabea0`.
- Summary: **Prerequisite:** none. This is the first session. **Deliverable type:** documents + interface definitions. **No code yet.** --- Topics: ILTRIC — PHASE 00 PROMPT: ARCHITECTURE & THREAT MODEL, OBJECTIVE, WHY, TASKS, ASSUMPTIONS TO STATE, EXIT GATE — GATE 00, FAILURE MODES, NEXT STEP

### `.backup-pre-v2/02-PHASE-01-BOOT.md`

- Inventory `F0004`; 4483 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 99 lines; SHA-256 `cf789efc6608fb3b578fdc595a1fe18625f966d5f95e01e9a7b85a2f92c0976e`.
- Summary: **Prerequisite:** GATE 00 passed. **Deliverable:** an ILTRIC disk image that boots to a serial shell inside QEMU on a Windows host, reproducibly, with CI smoke test. --- Topics: ILTRIC — PHASE 01 PROMPT: BOOTABLE PROTOTYPE (QEMU-FIRST), OBJECTIVE, WHY, TASKS, EXIT GATE — GATE 01, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/03-PHASE-02-CORE.md`

- Inventory `F0005`; 3894 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 94 lines; SHA-256 `17dc0d135f226d22ab87868a7d5403712ac4e5cfccf3d6605f0ce6fa9b3c504e`.
- Summary: **Prerequisite:** GATE 01 passed. **Deliverable:** a lean, supervised, resource-governed userland with a memory policy that is measured, not guessed. --- Topics: ILTRIC — PHASE 02 PROMPT: CORE SYSTEM (INIT, MEMORY, PROCESSES, IPC), OBJECTIVE, TASKS, EXIT GATE — GATE 02, FAILURE MODES, NEXT STEP

### `.backup-pre-v2/04-PHASE-03-STORAGE.md`

- Inventory `F0006`; 3531 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 71 lines; SHA-256 `0af49136cfe2fb1b3a07977511faa07524c701a3bbebf8486741c5031c51f59d`.
- Summary: **Prerequisite:** GATE 02 passed. **Deliverable:** a signed read-only system image, a snapshot-capable user volume, encryption, and a proven recovery path. --- Topics: ILTRIC — PHASE 03 PROMPT: STORAGE, FILESYSTEM, ENCRYPTION, RECOVERY, OBJECTIVE, TASKS, EXIT GATE — GATE 03, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/05-PHASE-04-NETWORK.md`

- Inventory `F0007`; 3147 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 60 lines; SHA-256 `e06227fb820cef7b2363e06562f69567cf357ef560f5992c50751578bfe5d98c`.
- Summary: **Prerequisite:** GATE 03 passed. **Deliverable:** lean, fast networking where network access is a *capability*, not a default. --- Topics: ILTRIC — PHASE 04 PROMPT: NETWORKING (PERMISSION-AWARE), OBJECTIVE, TASKS, EXIT GATE — GATE 04, FAILURE MODES, NEXT STEP

### `.backup-pre-v2/06-PHASE-05-GRAPHICS.md`

- Inventory `F0008`; 3787 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 70 lines; SHA-256 `a1a540bd2def9dcc607a0931bb79b19dc80e2e2aa661807e075cecf8e9e6e458`.
- Summary: **Prerequisite:** GATE 04 passed. **Deliverable:** an ILTRIC compositor on DRM/KMS with input, modern Wayland protocols, and *measured* latency. --- Topics: ILTRIC — PHASE 05 PROMPT: GRAPHICS & COMPOSITOR, OBJECTIVE, TASKS, EXIT GATE — GATE 05, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/07-PHASE-06-DESKTOP.md`

- Inventory `F0009`; 3626 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 69 lines; SHA-256 `fcc03d0d6a9e5c6d6f151ce409e7e2b20875e00fae8e7813da106d10bf9546c5`.
- Summary: **Prerequisite:** GATE 05 passed. **Deliverable:** the shell — this is where ILTRIC stops being "a distro" and becomes a platform with an identity. --- Topics: ILTRIC — PHASE 06 PROMPT: ILTRIC DESKTOP SHELL, OBJECTIVE, WHY, TASKS, EXIT GATE — GATE 06, FAILURE MODES, NEXT STEP

### `.backup-pre-v2/08-PHASE-07-SECURITY-DEFENDER.md`

- Inventory `F0010`; 5458 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 85 lines; SHA-256 `25b4d8ffd28d7b9f12e63bc2539c3851937bde5b7eb61ea16636ee998a560713`.
- Summary: **Prerequisite:** GATE 06 passed. **Deliverable:** capability sandbox enforced for real + a lightweight, always-on defender that does NOT eat the RAM budget. --- Topics: ILTRIC — PHASE 07 PROMPT: SECURITY, SANDBOX & ILTRIC DEFENDER, OBJECTIVE, PART A — SANDBOX & CAPABILITIES, PART B — ILTRIC DEFENDER, EXIT GATE — GATE 07, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/09-PHASE-08-UPDATE.md`

- Inventory `F0011`; 3771 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 64 lines; SHA-256 `28ee967ecfa510c4d9ffca3e8e3f52d3c62a2ab44f2d6f5a1489a803e30a2c77`.
- Summary: **Prerequisite:** GATE 07 passed. **Deliverable:** an update that can be power-cut at any instant and still leave a bootable machine. --- Topics: ILTRIC — PHASE 08 PROMPT: ATOMIC A/B UPDATE & AUTO-ROLLBACK, OBJECTIVE, TASKS, EXIT GATE — GATE 08, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/10-PHASE-09-AI.md`

- Inventory `F0012`; 4468 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 72 lines; SHA-256 `c5b1471a0dd76e9dbd599b0774e7ca0cf9685296439afd5ac4e3a059a290445a`.
- Summary: **Prerequisite:** GATE 08 passed. **Deliverable:** `iltric.ai` as a permissioned OS service with a capability/model router, local inference, and speech — on-device by default. --- Topics: ILTRIC — PHASE 09 PROMPT: AI SUBSYSTEM (SYSTEM SERVICE, NOT A CHATBOT), OBJECTIVE, TASKS, EXIT GATE — GATE 09, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/11-PHASE-10-APPS.md`

- Inventory `F0013`; 3559 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 64 lines; SHA-256 `3db7b61465fe7396bfa5cb55fc663c1521987626db955a09bfa205b7e524c1ee`.
- Summary: **Prerequisite:** GATE 09 passed. **Deliverable:** apps can be installed, sandboxed, permissioned, updated, rolled back, and built by third parties. --- Topics: ILTRIC — PHASE 10 PROMPT: APPLICATION PLATFORM, PACKAGES & SDK, OBJECTIVE, TASKS, EXIT GATE — GATE 10, FAILURE MODES, NEXT STEP

### `.backup-pre-v2/12-PHASE-11-HARDWARE.md`

- Inventory `F0014`; 3696 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 61 lines; SHA-256 `337bf03ddeb8b128b0f0b1d13680826b38c48c875a09f3997b225e5b964f7744`.
- Summary: **Prerequisite:** GATE 10 passed. **Do not start this phase early.** **Deliverable:** ILTRIC running on real machines, with power management, and an honest hardware compatibility list. --- Topics: ILTRIC — PHASE 11 PROMPT: REAL HARDWARE BRING-UP & POWER, OBJECTIVE, TASKS, EXIT GATE — GATE 11, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/13-PHASE-12-GAMING-MODE.md`

- Inventory `F0015`; 6885 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 86 lines; SHA-256 `f427a6ba8ff773a8d34d1e4613eeac28fb9af9425b12563936cd7cffd43ed2f7`.
- Summary: **Prerequisite:** GATE 05 (graphics). Best run after GATE 06. Cross-cutting phase. **Deliverable:** a gaming stack that matches or beats Windows on the same hardware — measured, not asserted. --- Topics: ILTRIC — PHASE 12 PROMPT: GAMING MODE & DEEP SETTINGS, OBJECTIVE, TASKS, EXIT GATE — GATE 12, FAILURE MODES & ROLLBACK, NEXT STEP

### `.backup-pre-v2/14-PHASE-13-PERF-MEMORY.md`

- Inventory `F0016`; 4709 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 77 lines; SHA-256 `07f3a45cf9fe6b158fdec40dbc5b8cfc2056aabf81ea64a031873b49ffd1741c`.
- Summary: **Prerequisite:** GATE 02+. Cross-cutting — re-run after every major phase. **Deliverable:** the low-RAM, high-smoothness claim, backed by a repeatable harness and published numbers. --- Topics: ILTRIC — PHASE 13 PROMPT: PERFORMANCE, MEMORY BUDGET & SMOOTHNESS, OBJECTIVE, TASKS, EXIT GATE — GATE 13 (re-runnable), FAILURE MODES, NEXT STEP

### `.backup-pre-v2/15-APPENDIX-TARGETS-MEASUREMENT.md`

- Inventory `F0017`; 6381 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 133 lines; SHA-256 `bf6cd9a05fc7e0ad6ee9b47e514549b169267d9df85dc5752641b6e2e3103907`.
- Summary: Paste this alongside any phase when numbers are in play. --- Topics: ILTRIC — APPENDIX: TARGETS, BASELINES & MEASUREMENT PROTOCOL, 1. TARGETS ILTRIC IS HELD TO, 2. BASELINES TO BEAT (measured elsewhere — re-verify locally before quoting), 3. MEASUREMENT PROTOCOL (mandatory for ev

### `.backup-pre-v2/ILTRIC_NATIVE_ECOSYSTEM_MASTER_ARCHITECTURE_v1.md`

- Inventory `F0018`; 18867 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 739 lines; SHA-256 `260059de60b7a5805ffe7bc1e3d9ea6f52485065c21697686fc8341e91e7678f`.
- Summary: Status: Foundational / Normative Project: Iltric OS Primary Goal: Define a genuinely native application ecosystem in which Iltric applications execute directly against Iltric's own system contracts, w Topics: ILTRIC NATIVE ECOSYSTEM — MASTER ARCHITECTURE SPECIFICATION v1.0, 0. EXECUTIVE PRINCIPLE, 1. DESIGN GOALS, 1.1 Primary goals, 1.2 Non-goals, 2. ARCHITECTURAL LAYERS, 2.1 Layer 0 — Firmware and Boot, 2.2 Layer 1 

### `.backup-pre-v2/ILTRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md`

- Inventory `F0019`; 25184 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 1193 lines; SHA-256 `aa2b405914246ab5f3c0a264d7e8ec050f7051dc30c3bbd692d1b64c335778cb`.
- Summary: Status: Foundational / Technical Design Project: Iltric OS Scope: Native execution semantics beneath the SDK and above kernel primitives. This document defines how a native Iltric application should a Topics: ILTRIC NATIVE RUNTIME, ABI, PROCESS, IPC & SECURITY — DEEP SPECIFICATION v1.0, 0. CORE THESIS, 1. NATIVE EXECUTION CONTRACT, 1.1 Process creation, 1.2 Entry-point semantics, 2. PROCESS MODEL, 3. THREAD MODEL, 4.

## `.backup-pre-v2/docs/` (8 entries)

### `.backup-pre-v2/docs/API-00-contracts.md`

- Inventory `F0020`; 7670 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 237 lines; SHA-256 `c011dabfb7b1299f0f1fa4fe345ab2dd1dd078b6b460bd6b0c0159ea09811de5`.
- Summary: **Document:** API-00-contracts.md | **Phase:** 00 | **Status:** DESIGN-ONLY **Rule:** An app must not reach a kernel API except through one of these interfaces. --- Topics: ILTRIC — Interface Contracts (Signatures Only), 1. iltric.caps — Capability Management, 2. iltric.fs — Permissioned Filesystem Access, 3. iltric.net — Permissioned Network Access, 4. iltric.gfx — Surface/Window/

### `.backup-pre-v2/docs/ARCH-00-boundaries.md`

- Inventory `F0021`; 6819 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 138 lines; SHA-256 `a2244c9081c9ee3c924678b2ffa85dd484e9a79d3671be96e4a0e2841e6c1308`.
- Summary: **Document:** ARCH-00-boundaries.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: ILTRIC — System Boundary Diagram, 1. Full Stack, 2. Trust Levels, 3. Trust Boundaries (Critical Interfaces), 4. Dependency Direction (MUST NOT invert), 5. Component Ownership

### `.backup-pre-v2/docs/DEV-01-debug.md`

- Inventory `F0022`; 3649 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 155 lines; SHA-256 `45748475d4436a97831c148e528154a263064964ac85fbbdfd2f5c7232981a00`.
- Summary: **Document:** DEV-01-debug.md | **Phase:** 01 | **Status:** DESIGN-ONLY --- Topics: ILTRIC — Kernel Debugging Guide, 1. Prerequisites, 2. Quick Start, Terminal 1: Start QEMU (frozen at start), or on Windows:, Terminal 2: Connect GDB, Load Iltric's GDB scripts (for pretty-printing), Connect to Q

### `.backup-pre-v2/docs/GATES.md`

- Inventory `F0023`; 2791 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 61 lines; SHA-256 `c45bffa9b18bbff3bc252850c292c0ebf5646ad72ee63a77694dc1d430fd788f`.
- Summary: --- --- Topics: ILTRIC — Gate Results Log, GATE 00 — Architecture & Threat Model, Gate conditions:, Notes:, Next: 02-PHASE-01-BOOT.md, GATE 01 — Bootable Prototype (DESIGN-ONLY), Gate conditions:, Limitations:, Next: 03-PHASE-0

### `.backup-pre-v2/docs/REPO-00-layout.md`

- Inventory `F0024`; 7265 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 146 lines; SHA-256 `06c2e551b408696072b6606bce7317ba38eaabea04cedc86b49bcb2a27df40c9`.
- Summary: **Document:** REPO-00-layout.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: ILTRIC — Repository Layout, 1. Directory Tree, 2. Ownership by Directory, 3. Naming Conventions

### `.backup-pre-v2/docs/RISK-00.md`

- Inventory `F0025`; 5688 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 116 lines; SHA-256 `eb8dd3f6aeea663cf0b8a2f74e11ca225af0fcfb86c3fbf90bad3ea303d2863b`.
- Summary: **Document:** RISK-00.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: ILTRIC — Risk Register, 1. Critical Risks (must be accepted/document BEFORE Phase 01), RISK-001: Kernel Anti-Cheat Games, RISK-002: Windows-Host QEMU Cannot Do Accelerated Graphics, RISK-003: WHPX SMP Boot Bug, 

### `.backup-pre-v2/docs/SEC-00-threatmodel.md`

- Inventory `F0026`; 5678 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 101 lines; SHA-256 `e807f60c548127c3aa282ec4447919ca924f68f661499fbdae834691934dbf85`.
- Summary: **Document:** SEC-00-threatmodel.md | **Phase:** 00 | **Status:** DESIGN-ONLY **Method:** STRIDE per trust boundary --- Topics: ILTRIC — Threat Model, 1. Adversary Profiles, 2. STRIDE Analysis by Boundary, BOUNDARY A: Kernel ↔ Userspace, BOUNDARY B: Core Service ↔ App, BOUNDARY C: App ↔ App, BOUNDARY D: Native ↔ Windows Compatibility, BO

### `.backup-pre-v2/docs/SEC-01-capabilities.md`

- Inventory `F0027`; 2959 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 58 lines; SHA-256 `f5eab3ce6e0680a0ed238af6e3e89354d3b06dedd9c19570a14449c7170d3a3c`.
- Summary: **Document:** SEC-01-capabilities.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: ILTRIC — Capability Vocabulary, 1. Capability Specification, 2. Capability Classes, 3. Rules

## `.backup-pre-v2/docs/adr/` (8 entries)

### `.backup-pre-v2/docs/adr/0001-kernel-foundation.md`

- Inventory `F0028`; 741 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 20 lines; SHA-256 `08bb7083da669f330f9365927d41e436f4dce8e5db6f4ab239740e6fe014d8e3`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs a kernel. Writing from scratch (Redox, SerenityOS model) costs years before hardware support. Topics: ADR-0001: Kernel Foundation, Context, Decision, Consequences, Revisit-if

### `.backup-pre-v2/docs/adr/0002-init-system.md`

- Inventory `F0029`; 580 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 21 lines; SHA-256 `75f63d8eb809f6cc0c8cb6209499f9e913622381d1f72272c231c2beb88e97e4`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs service supervision, cgroups integration, A/B update support, and sandboxing. Topics: ADR-0002: Init System, Context, Decision, Consequences, Revisit-if

### `.backup-pre-v2/docs/adr/0003-image-builder.md`

- Inventory `F0030`; 616 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 20 lines; SHA-256 `90004d28a1ca2d8e7be4997b1f8ee281c00c445e931dc7b32f395a371d8c5681`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs reproducible UEFI images with ESP, dm-verity, and UKI in one pipeline. Topics: ADR-0003: Image Builder, Context, Decision, Consequences, Revisit-if

### `.backup-pre-v2/docs/adr/0004-filesystem-layout.md`

- Inventory `F0031`; 881 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 25 lines; SHA-256 `3f99db8602ea761f74397f73232f720235cd6337f58c62e0901d2e346f7912cf`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs a tamper-evident system partition and a snapshot-capable user volume. Topics: ADR-0004: Filesystem Layout, Context, Decision, Consequences, Revisit-if

### `.backup-pre-v2/docs/adr/0005-compositor.md`

- Inventory `F0032`; 831 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 26 lines; SHA-256 `25e4f63768a2aad19188997975273489dffd85dc78d1e5f4ed4d19c16d299a76`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs a compositor that is memory-safe, modern, and supports DRM syncobj, dmabuf, and HDR. Topics: ADR-0005: Compositor, Context, Decision, Consequences, Alternatives considered, Revisit-if

### `.backup-pre-v2/docs/adr/0006-audio-stack.md`

- Inventory `F0033`; 570 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 21 lines; SHA-256 `6f0ef7fe9c758bc1a5717b5e1011e8103543f9969978a36cd0a03aee05a46825`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs low-latency audio with per-app volume and spatial audio. Topics: ADR-0006: Audio Stack, Context, Decision, Consequences, Revisit-if

### `.backup-pre-v2/docs/adr/0007-package-format.md`

- Inventory `F0034`; 912 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 27 lines; SHA-256 `caa1514f3678fb468a465cc94a0bb5369cf6a7f2d90ac8f168f4a15ab080a7a3`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs a sandboxed app platform with atomic updates and a capability model. Topics: ADR-0007: Package Format, Context, Decision, Consequences, Alternatives considered, Revisit-if

### `.backup-pre-v2/docs/adr/0008-security-model.md`

- Inventory `F0035`; 861 bytes; `Markdown`; role `historical-backup`.
- Historical content fingerprint: 23 lines; SHA-256 `8b6c9b55c236ce03f054d0255c714e2103bb3c8a57c4dcb9bc1e4e8ef971061f`.
- Summary: **Status:** Accepted | **Date:** 2026-08-26 Iltric needs a layered security model that is deny-by-default without being unusable. Topics: ADR-0008: Security Model, Context, Decision, Consequences, Revisit-if

## `.backup-pre-v2/image/` (1 entries)

### `.backup-pre-v2/image/mkosi.conf`

- Inventory `F0036`; 2963 bytes; `INI/Config`; role `historical-backup`.
- Historical content fingerprint: 135 lines; SHA-256 `e71e8a427bef23af995b70cb7f392c542640eaede6097df631b7889d61f3dc14`.
- Summary: [Distribution] Distribution=arch [Build] Topics: =============================================================================, ILTRIC OS — mkosi Image Pipeline Configuration, Produces: GPT disk image with ESP + system partition, Output: iltric.raw / iltric.qc

## `.backup-pre-v2/kernel/config/` (1 entries)

### `.backup-pre-v2/kernel/config/iltric-base.config`

- Inventory `F0037`; 8126 bytes; `Kconfig`; role `historical-backup`.
- Historical content fingerprint: 312 lines; SHA-256 `d0df491a8a65aa8935beed309c5b3a0954f2abda6d117bfa8d568bfaec2b7b28`.
- Summary: CONFIG_LRU_GEN=y CONFIG_LRU_GEN_ENABLED=y CONFIG_SCHED_CLASS_EXT=y Topics: ILTRIC OS — Kernel Configuration Fragment, Based on: x86_64 defconfig + Iltric-specific overlay, Target: QEMU virt + modern PC (x86-64 first-class), , Usage: apply this fragment on top of defconfig, make defconf

## `.backup-pre-v2/tests/` (1 entries)

### `.backup-pre-v2/tests/smoke_boot.py`

- Inventory `F0038`; 8130 bytes; `Python`; role `historical-backup`.
- Historical content fingerprint: 251 lines; SHA-256 `107554b3fa3333a42ee19524f8b42fcde498255eef840e4df3afe711d8e79db7`.
- Summary: """ ILTRIC OS — CI Smoke Boot Test Headless QEMU + serial console assertion. Asserts: UEFI → kernel → userspace → login/shell prompt. Usage: pytest tests/smoke_boot.py -v python tests/smoke_boot.py    Topics: =============================================================================, CONFIGURATION, =============================================================================, Boot timeout (seconds), Expected strin

## `.backup-pre-v2/tools/` (3 entries)

### `.backup-pre-v2/tools/build-image.sh`

- Inventory `F0039`; 6939 bytes; `Bash/POSIX Shell`; role `historical-backup`.
- Historical content fingerprint: 221 lines; SHA-256 `c696ea9e5ce0db2239141309baef5f4a32865615cdef8257a04c6703fc88160c`.
- Summary: set -euo pipefail PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)" BUILD_DIR="${PROJECT_ROOT}/build" IMAGE_DIR="${PROJECT_ROOT}/image" OUTPUT_RAW="${BUILD_DIR}/iltric.raw" OUTPUT_QCOW2="${BUILD_DIR}/i Topics: =============================================================================, ILTRIC OS — Image Build Script, Builds the Iltric disk image from mkosi configuration, Prerequisites:, - Linux host (or WSL2), - mko

### `.backup-pre-v2/tools/run-qemu.ps1`

- Inventory `F0040`; 4923 bytes; `PowerShell`; role `historical-backup`.
- Historical content fingerprint: 142 lines; SHA-256 `73ceccf188bf35c8a1f482c4cc3c01aea4f55a0f61930177da36b3f81c535226`.
- Summary: param( [switch]$Debug, [switch]$Tcg, [int]$Smp = 4, [string]$Memory = "4G", [string]$Image = "", [int]$SshPort = 2222 ) $ErrorActionPreference = "Stop" Topics: =============================================================================, ILTRIC OS — QEMU Launcher (Windows Host), Launches Iltric in QEMU with WHPX acceleration, Prerequisites:, - QEMU for Windows (https:

### `.backup-pre-v2/tools/run-qemu.sh`

- Inventory `F0041`; 4924 bytes; `Bash/POSIX Shell`; role `historical-backup`.
- Historical content fingerprint: 164 lines; SHA-256 `ba0a35a3f63f5f9a0f5ddbd432011d39a6d43c1b9a9bd01d5291b58ee737bb69`.
- Summary: set -euo pipefail SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)" PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)" BUILD_DIR="${PROJECT_ROOT}/build" LOGS_DIR="${SCRIPT_DIR}/logs" TIMESTAMP="$(date +%Y%m%d-%H% Topics: =============================================================================, ILTRIC OS — QEMU Launcher (Linux/Mac Host), Launches Iltric in QEMU with KVM acceleration, Prerequisites:, - QEMU (qemu-system-x86_6

