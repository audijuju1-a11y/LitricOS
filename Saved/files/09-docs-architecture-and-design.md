# Batch B09: Architecture, ADR, audit, design and governance documents

This report catalogs 109 assigned documents and HTML artifacts. It distinguishes accepted architecture decisions from requirements/roadmaps, post-implementation audit evidence, bug records, design specs and prompts; status claims should be read with the recorded verification state rather than treating a planned feature as shipped.

## `API-00-contracts.md/` (1 entries)

### `docs/API-00-contracts.md`

- Inventory `F0118`; 7670 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 237 lines; SHA-256 `a42fa90707fe746e62e94d3470d8986e18ef4107a69fb78cb7b9482327656b70`.
- Scope summary: **Document:** API-00-contracts.md | **Phase:** 00 | **Status:** DESIGN-ONLY **Rule:** An app must not reach a kernel API except through one of these interfaces. --- Topics: LITRIC — Interface Contracts (Signatures Only), 1. litric.caps — Capability Management, 2. litric.fs — Permissioned Filesystem Access, 3. litric.net — Permissioned Network Access, 4. litric.gfx — Surface/Window/

## `ARCH-00-boundaries.md/` (1 entries)

### `docs/ARCH-00-boundaries.md`

- Inventory `F0119`; 8473 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 166 lines; SHA-256 `be3716043954f487bfb181abc6ff3b6c23decbd8410aa345b77ad2177092df9d`.
- Scope summary: **Document:** ARCH-00-boundaries.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: LITRIC — System Boundary Diagram, 1. Full Stack, 2. Trust Levels, 3. Trust Boundaries (Critical Interfaces), 4. Dependency Direction (MUST NOT invert), 5. Component Ownership, Correction — 2026-10-11: the L4/L5 

## `DESIGN-SYSTEM-v3/` (7 entries)

### `docs/DESIGN-SYSTEM-v3/00-AUDIT.md`

- Inventory `F0120`; 8054 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 62 lines; SHA-256 `7835a94f0566a68d16bb545f5eab55557b9db657bf06326490c1fae0c50c4cb1`.
- Scope summary: --- ยืนยันจาก grep: ไม่มี dock (ตัดสินใจถูก — แต่ต้องมี **app grid/launchpad** มาตรฐานแทน), ไม่มี Control Center, ไม่มี Notification Center, ไม่มี Universal Search/Command Palette, ไม่มี window snapping (เฉพาะ fullscreen) Topics: 00 — AUDIT: สถานะ UI จริงของ LitricOS (หลักฐาน file:line), 1. สิ่งที่มีอยู่แล้วและใช้งานจริง (อย่าทำทับ — ต่อยอด), 2. จุดพังที่ทำให้ "UI กาก/ไม่มาตรฐาน" — ระบุด้วยหลักฐาน, A. Motion เขียนไว้แต่ไม่ใช้ (สาเหตุหลัก

### `docs/DESIGN-SYSTEM-v3/01-FOUNDATION-TOKENS.md`

- Inventory `F0121`; 7629 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 96 lines; SHA-256 `86830b62eceba02ad8ec30c0d66f7a5e3c0497a122e09fbca4b4f92f6552cead`.
- Scope summary: L1 Primitive   → สิ่งที่ litric-design มี: TypeStep(8), Space, Radius, Motion(4), Rgb L2 Semantic    → Theme struct: text/text_dim/danger/success/accent + surface(t) 7 ระดับ + glass adaptive L3 Component   → ยังไม่มีระบบ — สร้าง `component_style()` ใน ui.rs (ด Topics: 01 — FOUNDATION & TOKENS (ผูกกับ MASTER §4-§14), 1. Token Architecture 3 ชั้น (MASTER §4), 2. สิ่งที่ต้องเพิ่มใน L3 (Component Tokens) — ระบุเป็นโค้ดจริง, 3. Typography Roles (แก้จุดอ่อน B จาก audit — bold 7 ครั

### `docs/DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md`

- Inventory `F0122`; 6660 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 77 lines; SHA-256 `b7172a3f74d402d97fc72543a08b2e37ff49b9641449fd10ed9b595ec52ea590`.
- Scope summary: **มีอยู่แล้ว (ใช้ต่อ ห้ามเขียนใหม่):** ปุ่ม, list row, tab, dialog พื้นฐาน, context menu (มี 7 จุดใน fsbridge/desktop), text input (omnibox + rename), launcher grid, toggle ใน Settings, window chrome พร้อม traffic lights **ขาด:** state ครบชุด, disabled/loading Topics: 02 — COMPONENTS & STATES (MASTER §15-§19, §37), 1. สถานะปัจจุบันจาก audit, 2. State Matrix บังคับทุก control (MASTER §17), 3. Interaction Model (MASTER §18-§19) — ผูก Motion token, 4. Component Worklist (ลำดับตา

### `docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md`

- Inventory `F0123`; 8301 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 84 lines; SHA-256 `98cc8f0114a678ed30c3b6b2ccbb3c8f59448221924e6671822719ebddb3ec56`.
- Scope summary: State ที่ต้องมีครบ: `Normal / Focused / Unfocused / Fullscreen / Maximized / Minimized / Snapped / Dragging / Resizing / Busy` **มีแล้ว:** focus model จริง (shell.rs:1659 Alt+Tab ถูกต้อง), fullscreen, drag, resize **เพิ่ม:** Topics: 03 — SHELL SYSTEM (MASTER §23-§36), 1. Window System (MASTER §23), 2. Desktop Layer Contract (MASTER §24-§25) — z-order เหล็ก, 3. System Bar (MASTER §27) — จาก "แถบเดียว" สู่ "แถบที่สมบูรณ์", 4. Control Center (

### `docs/DESIGN-SYSTEM-v3/04-QA-LINT-GOVERNANCE.md`

- Inventory `F0124`; 8914 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 82 lines; SHA-256 `3310f6f4b9fd97af13a864dc58d5976734dcb9c0a0b03562f8dffe038c052bc5`.
- Scope summary: Lint ทำงานบน **โค้ดจริง** (ไม่ใช่ภาพ) — static check ใน test ใหม่ `design_lint.rs`: กระบวนการ: **REPORT → FIX → RECHECK** — lint fail = CI/build fail (ไม่มีเส้นโค้ง) Topics: 04 — QA, DESIGN LINT, GOVERNANCE (MASTER §51-§59, §63-§64), 1. Design Lint — `cargo test -p litric-shell design_lint`, 2. Golden Screens (MASTER §52) — 10 ใบบังคับ, 2.1 Gate ที่รันได้แล้ว (2026-10-16), 3. Visual

### `docs/DESIGN-SYSTEM-v3/05-ROADMAP.md`

- Inventory `F0125`; 6641 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 71 lines; SHA-256 `c9f949e43b179490bc3e32d103eeaa6e0ec20fec216866d5258a5152f671a92d`.
- Scope summary: **ออกจาก Wave 0 เมื่อ:** เปิดเมนู/โฟกัสหน้าต่าง/กดปุ่ม มี motion จริง + ภาพใหม่ดู "มีชั้น" ขึ้นจากภาพเดิม (แนบ before/after) 1. **ก่อนเขียนโค้ด:** Design Review Output ส่วน A-K (MASTER §63) ลง PR/notes 2. **ระหว่าง:** lint ต้องเขียวตลอด ห้ามสะสม 3. **ก่อนจบ Wa Topics: 05 — ROADMAP: จากแผนสู่โค้ด (MASTER §60, §65 — mapping 34 phases), Wave 0 — รื้นฐานที่พัง (เล็ก, สูงผลตอบแทน) ≈ เฟส 03-10 ของ MASTER, Wave 1 — Component state ครบ (MASTER §15-§19) ≈ เฟส 11-13, Wave 2 — เชลล์ใหม่

### `docs/DESIGN-SYSTEM-v3/06-APP-REDESIGN.md`

- Inventory `F0126`; 11825 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 181 lines; SHA-256 `f3da01b5b4a08e4aa666464105ecd0dd70f96819b0bc622ca4e4009148e7cb17`.
- Scope summary: `UI-PLAN` defines the grammar every surface shares: menubar, window chrome, dock, Spotlight, context menus, motion, notifications. It does not define what is *inside* each window. Today the applications inside that chrome are the weakest part of the system --  Topics: 06 — APP REDESIGN (in-app surfaces, to the standard of Windows and macOS), 0. The gate that comes first, 1. Method: every app closes with an artefact, not a screenshot, 2. Activity -> System Monitor (the user na

## `DESKTOP-APP.md/` (1 entries)

### `docs/DESKTOP-APP.md`

- Inventory `F0127`; 5765 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 128 lines; SHA-256 `3bf619b121917160bb0ff5c7b7a8ab7536683db170366535d832361bddcfd023`.
- Scope summary: Double-click `LitricOS.cmd` at the root of the checkout. It opens the operating system in its own window: UEFI firmware, the signed disk image, systemd, the LitricOS daemons, and the shell drawing at 1920x1080. The same thing from a terminal, inside WSL: Topics: Running LitricOS as an application, What the guest really has, Wi-Fi and Bluetooth, Measured on this host, 2026-10-17, Memory, Powering off, First boot

## `DEV-01-debug.md/` (1 entries)

### `docs/DEV-01-debug.md`

- Inventory `F0128`; 3649 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 155 lines; SHA-256 `3785381055e38c10882fd3ee0704393099cfe493e0814e346b332636b8fc7e34`.
- Scope summary: **Document:** DEV-01-debug.md | **Phase:** 01 | **Status:** DESIGN-ONLY --- Topics: LITRIC — Kernel Debugging Guide, 1. Prerequisites, 2. Quick Start, Terminal 1: Start QEMU (frozen at start), or on Windows:, Terminal 2: Connect GDB, Load Litric's GDB scripts (for pretty-printing), Connect to Q

## `EVIDENCE.md/` (1 entries)

### `docs/EVIDENCE.md`

- Inventory `F0129`; 37598 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 583 lines; SHA-256 `0b9c3e9396358834e566fa91a971493cfb183ba0c75b6c4a8c8c3c24b84880b1`.
- Scope summary: Generated by `ci/verify.sh --evidence`. Every PASS below was executed. Every SKIP was not run, with the reason given. Nothing here is inferred. date : 2026-10-17T08:29:20Z host : Linux x86_64 rust : rustc 1.99.0 (b940084d7 2026-09-28) (Arch Linux rust 1:1.99.0 Topics: LITRIC — EXECUTED EVIDENCE, Verified, NOT RUN, What this does and does not establish, Earlier records, preserved and NOT re-executed in this run, Earlier records, preserved and NOT re-executed in this run, Earli

## `EXECUTIVE-SUMMARY.md/` (1 entries)

### `docs/EXECUTIVE-SUMMARY.md`

- Inventory `F0130`; 22827 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 277 lines; SHA-256 `68a55d535fc3764fe86e6f53594ece924b414be6f46b568402c27ba9b15068e3`.
- Scope summary: --- --- Topics: LitricOS — Executive Summary (สรุปผู้บริหาร), ตารางเปรียบเทียบ OS หลัก (Summary), วิเคราะห์เชิงลึก, 1. ฟีเจอร์เด่นและ Competitive Moat, 2. Moat Priority, 3. ช่องว่างทางตลาด (Market Gaps) — โอกาสของ LitricOS, Gap

## `FONT-LICENSES.md/` (1 entries)

### `docs/FONT-LICENSES.md`

- Inventory `F0131`; 9890 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 203 lines; SHA-256 `999e50c0f2b99124f735aa45cd5f8cb59423017aee184de70e3e39fb5d7ce240`.
- Scope summary: LitricOS uses the distro-installed, unmodified Noto Sans Regular/Bold for Latin, Noto Sans Thai Regular/Bold for Thai fallback, and JetBrains Mono Nerd Font Mono Regular/Bold for terminal text. They are loaded from `/usr/share/fonts` and are never taken from a Topics: Font and text stack notices, Noto Sans Thai license, JetBrains Mono license

## `GATES.md/` (1 entries)

### `docs/GATES.md`

- Inventory `F0132`; 52252 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 804 lines; SHA-256 `6e56f71b7e8d1d90c88aaf656e75e4615b4f482c6577bb16f4e9ea7130da51c5`.
- Scope summary: Append-only. Every entry carries date, the exact conditions, and raw numbers. A gate that "almost passes" does not pass (MASTER §6). Legend: `PASS` · `PASS_WITH_LIMITATIONS` · `BLOCKED` · `REJECTED` · `NOT RUN` Topics: LITRIC — Gate Results Log, GATE 00 — Architecture & Threat Model, Conditions, Notes, GATE 01 — Bootable Prototype, Entry 1 — 2026-08-26 — **REVOKED**, Entry 2 — 2026-08-26 (v2) — **BLOCKED (build environment), t

## `OPTIMIZATION.md/` (1 entries)

### `docs/OPTIMIZATION.md`

- Inventory `F0133`; 11735 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 353 lines; SHA-256 `ad6ec87963167acecebd573ad64b6a4f885248e59f1425792af1e6ccb16880c3`.
- Scope summary: Reduce idle desktop RAM from **786MB → ≤320MB** on a 4GB QEMU VM. **Achieved: ~295MB idle RAM** (after drop-caches) ✅ Topics: LitricOS — RAM Optimization Log, Target, Result, Root Cause Analysis, What was consuming RAM (786MB breakdown), Changes Applied, 1. Sysctl: Memory Management (`10-litric-mm.conf`), 2. Service: Drop Page Cache (`

## `PERF-budget.md/` (1 entries)

### `docs/PERF-budget.md`

- Inventory `F0134`; 18584 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 320 lines; SHA-256 `acf208dff01a7e473af5dfd04bf78e2644071f5ef1e780aaaa37a398e2945d90`.
- Scope summary: The budget is the mechanism that keeps LitricOS light. Good intentions are not a mechanism. `tests/budget/` enforces every line below in CI; exceeding a line **blocks that phase's gate**, with no "close enough". --- Topics: LITRIC — RESOURCE BUDGET (v2, AGGRESSIVE), 1. WHY THE v1 BUDGET WAS NOT AGGRESSIVE ENOUGH, 2. PROFILE LADDER, Comparison context (external measurements — re-verify before quoting), 3. `desktop` PROFILE LEDGER (t

## `REPO-00-layout.md/` (1 entries)

### `docs/REPO-00-layout.md`

- Inventory `F0135`; 9059 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 174 lines; SHA-256 `16ece7e2ada783386354abea419fcbae2847656ac6ad7c3412084987bba13787`.
- Scope summary: **Document:** REPO-00-layout.md | **Phase:** 00 | **Status:** DESIGN-ONLY — §0 says what the tree actually is --- Topics: LITRIC — Repository Layout, 0. What this repository actually is — 2026-10-11, 1. Directory Tree (the Phase 00 plan), 2. Ownership by Directory, 3. Naming Conventions (in force)

## `RISK-00.md/` (1 entries)

### `docs/RISK-00.md`

- Inventory `F0136`; 7761 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 149 lines; SHA-256 `2111bbb17c3d23ac23d178ecde96ff4a4a0fbd2304573bce441b7f199086a426`.
- Scope summary: **Document:** RISK-00.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: LITRIC — Risk Register, 1. Critical Risks (must be accepted/document BEFORE Phase 01), RISK-001: Kernel Anti-Cheat Games, RISK-002: Windows-Host QEMU Cannot Do Accelerated Graphics, RISK-003: WHPX SMP Boot Bug, 

## `SEC-00-threatmodel.md/` (1 entries)

### `docs/SEC-00-threatmodel.md`

- Inventory `F0137`; 5678 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 101 lines; SHA-256 `50052402bd1572b12f305a8cab7e2fb9896e2a1094441ab3f0650b5d8e1bc8ed`.
- Scope summary: **Document:** SEC-00-threatmodel.md | **Phase:** 00 | **Status:** DESIGN-ONLY **Method:** STRIDE per trust boundary --- Topics: LITRIC — Threat Model, 1. Adversary Profiles, 2. STRIDE Analysis by Boundary, BOUNDARY A: Kernel ↔ Userspace, BOUNDARY B: Core Service ↔ App, BOUNDARY C: App ↔ App, BOUNDARY D: Native ↔ Windows Compatibility, BO

## `SEC-01-capabilities.md/` (1 entries)

### `docs/SEC-01-capabilities.md`

- Inventory `F0138`; 4076 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 74 lines; SHA-256 `7d677d653b7b0fef7d8e0b5e8cb68736f57fdbf9559d0c771a56e1074a3914b1`.
- Scope summary: **Document:** SEC-01-capabilities.md | **Phase:** 00 | **Status:** DESIGN-ONLY --- Topics: LITRIC — Capability Vocabulary, 1. Capability Specification, 2. Capability Classes, 3. Rules

## `SYSTEM-SCOPE.md/` (1 entries)

### `docs/SYSTEM-SCOPE.md`

- Inventory `F0139`; 81672 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 1735 lines; SHA-256 `a1c61ac3f913f96588807ed129bfed979d4bebea29ac0f6c475a3c296b9d6af4`.
- Scope summary: **Owner:** `O` = we write it · `U` = upstream, we integrate · `O+U` = upstream component, our UI and policy on top **Status:** `D` = done and tested · `P` = partial, real code with passing tests · `S` = specified, no code · `T` = not started Topics: LITRIC — COMPLETE SYSTEM SCOPE, TALLY, 0. WHAT WE WRITE VERSUS WHAT WE INTEGRATE, PART A — FOUNDATION, A1. Boot and firmware (18), A2. Kernel and core system (26), A3. Storage and filesystems (34), PART B — SHEL

## `UI-INVENTORY.md/` (1 entries)

### `docs/UI-INVENTORY.md`

- Inventory `F0140`; 24185 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 355 lines; SHA-256 `71f6f0fe40e756fca9071276c30bf09514a6ebc61be5168c64f3cfef1415eb4a`.
- Scope summary: Written 2026-10-13, on a machine that had the desktop running in QEMU at the time (see *Seeing it* at the end). It is a **map**, not a specification: it says which file holds which decision and which files talk to each other, so a change can be aimed rather th Topics: UI inventory — every file the desktop is made of, how it is wired, what it configures, 1. What is on the screen, 2. The shell crate — every source file, What it depends on ([Cargo.toml](../src/litric-shell/Cargo

## `UI-PLAN.md/` (1 entries)

### `docs/UI-PLAN.md`

- Inventory `F0141`; 7441 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 135 lines; SHA-256 `332a869d6711d92774ec69d10ad700f4754ade123f335c45745ca29d1e5c55bf`.
- Scope summary: Status: PLAN · 2026-09-19 · feeds from docs/UI-RESEARCH-MACOS.md Renderer reality: software canvas @1280×800, frame budget 50ms, no GPU blur — all "glass" is alpha+tint approximation (honest per research §3). One sentence: **every surface reads as a system mat Topics: UI Plan — Standard (Windows-behavior) × Premium (macOS-look) for LitricOS, North star, Phases (each ships green + E2E-tested; no phase breaks previous), P0 — Design tokens (litric-design), P1 — Menu bar 24px gla

## `UI-RESEARCH-MACOS.md/` (1 entries)

### `docs/UI-RESEARCH-MACOS.md`

- Inventory `F0142`; 9899 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 176 lines; SHA-256 `d3a1d953dc9918d9b21113b107847835fb843c3462a03dd4c4f3fbf96878a19f`.
- Scope summary: Status: RESEARCH COMPLETE · 2026-09-19 · 76 distinct sources (8 read in full depth) Scope: make LitricOS desktop feel **standard like Windows, premium like macOS** — per user's attached macOS 27 / Tahoe reference images (glass dock, centered Spotlight, translu Topics: UI Research — Standard-Premium macOS-style for LitricOS, 1. Source register (≥40 required — 76 collected), Primary — Apple official (13), Engineering deep-dives (read in full) (8), Dock magnification implementat

## `adr/` (18 entries)

### `docs/adr/0001-kernel-foundation.md`

- Inventory `F0143`; 741 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 20 lines; SHA-256 `5ca2d0e38be8db78d916e6b589ccbf475faebe707ac2bf5d652c27e6df0ac36c`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs a kernel. Writing from scratch (Redox, SerenityOS model) costs years before hardware support. Topics: ADR-0001: Kernel Foundation, Context, Decision, Consequences, Revisit-if

### `docs/adr/0002-init-system.md`

- Inventory `F0144`; 580 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 21 lines; SHA-256 `21e2ea39e198175ce5ab1f7801ea856f8e00b44ca589d6f532e0a1eb85b6cb64`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs service supervision, cgroups integration, A/B update support, and sandboxing. Topics: ADR-0002: Init System, Context, Decision, Consequences, Revisit-if

### `docs/adr/0003-image-builder.md`

- Inventory `F0145`; 616 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 20 lines; SHA-256 `8cae5d51ee8fc6edb9fa51c1d79c3e179db506dd1f39f25ddc5fe52b15711cc4`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs reproducible UEFI images with ESP, dm-verity, and UKI in one pipeline. Topics: ADR-0003: Image Builder, Context, Decision, Consequences, Revisit-if

### `docs/adr/0004-filesystem-layout.md`

- Inventory `F0146`; 881 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 25 lines; SHA-256 `574a4ba3c7ae0502b590f00d6fbf6a7068d3a7563820fcd6d1e4974190515801`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs a tamper-evident system partition and a snapshot-capable user volume. Topics: ADR-0004: Filesystem Layout, Context, Decision, Consequences, Revisit-if

### `docs/adr/0005-compositor.md`

- Inventory `F0147`; 831 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 26 lines; SHA-256 `dcb434da9975fd9bd68a7879e1dd905505f4fd7c4ccac30d47b4b5a2918260f3`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs a compositor that is memory-safe, modern, and supports DRM syncobj, dmabuf, and HDR. Topics: ADR-0005: Compositor, Context, Decision, Consequences, Alternatives considered, Revisit-if

### `docs/adr/0006-audio-stack.md`

- Inventory `F0148`; 570 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 21 lines; SHA-256 `94425f383975f818f864f9860d6190264c424e80190860361610246b8a51b485`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs low-latency audio with per-app volume and spatial audio. Topics: ADR-0006: Audio Stack, Context, Decision, Consequences, Revisit-if

### `docs/adr/0007-package-format.md`

- Inventory `F0149`; 912 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 27 lines; SHA-256 `e3171472f8516c3871ce50d134ca02cfa01b8c0ec1dad54bc2d7920ae1061c34`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs a sandboxed app platform with atomic updates and a capability model. Topics: ADR-0007: Package Format, Context, Decision, Consequences, Alternatives considered, Revisit-if

### `docs/adr/0008-security-model.md`

- Inventory `F0150`; 861 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 23 lines; SHA-256 `c4590f6a713f709d3412679c1192cbd99a819e5eb650725a7438faa8393fa31e`.
- Scope summary: **Status:** Accepted | **Date:** 2026-08-26 Litric needs a layered security model that is deny-by-default without being unusable. Topics: ADR-0008: Security Model, Context, Decision, Consequences, Revisit-if

### `docs/adr/0009-unified-shell-process.md`

- Inventory `F0151`; 8687 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 169 lines; SHA-256 `d02c08d6c1750d859a19e0e764f40450f78285afddb34cd16b991f472ea10158`.
- Scope summary: the process topology above it changes. --- Topics: ADR-0009 — The compositor and the shell are one process, Context, Requirements, Options considered, A. Conventional multi-process, GTK4 shell, B. Multi-process shell on a minimal toolkit (e.g. custom Rust + wgpu

### `docs/adr/0015-multi-environment-switching.md`

- Inventory `F0152`; 9401 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 229 lines; SHA-256 `9106fa4abd45048502e233d6f4862eafacf36c22ef29a9d594fea3331b27a7bb`.
- Scope summary: model in `LITRIC_NATIVE_ECOSYSTEM_MASTER_ARCHITECTURE_v1.md` §17. --- Topics: ADR-0015 — LitricOS as host: Windows and Linux as switchable environments, Requirement, The decision, Why this shape and not the alternatives, Two tiers, and the order matters, Constraints that are real, and sta

### `docs/adr/0016-system-daemon.md`

- Inventory `F0153`; 5761 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 132 lines; SHA-256 `d825094b247ccb42509efb57b749fb398f42c4d8f49c946f39b6bfbf41ab286f`.
- Scope summary: --- By 2026-08-28 LitricOS had a capability engine, a storage broker, a manifest parser, a Windows-compatibility planner, an environment manager, a trash policy and a design system. All of them had tests. None of them had ever executed inside the operating sys Topics: ADR-0016 — `litric-sysd`: one process that runs the Layer 2 and Layer 3 code, The problem this solves, The decision, Why one process and not one per subsystem, Why a text protocol, Why the caller's identity neve

### `docs/adr/0017-framebuffer-shell.md`

- Inventory `F0154`; 6106 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 133 lines; SHA-256 `5c90f7e60e2a080cf9a4fa901faa2b9755c2feb393662b505c3e05c300e46c4f`.
- Scope summary: describe, not a replacement for it. --- Topics: ADR-0017 — The desktop is drawn in software, on `/dev/fb0`, with no `unsafe`, The requirement, The constraint that shaped every decision, The decisions, 1. Geometry from sysfs, pixels through `write()`, 2. Signe

### `docs/adr/0018-browser-engine-chromium.md`

- Inventory `F0155`; 8059 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 126 lines; SHA-256 `9929ce65a25d25280cc88b3494c2093be92e7f6405fd73c28a725fa8569b5774`.
- Scope summary: desktop profile contains no browser engine: from this ADR on, the `desktop` profile ships the Arch `chromium` package, and the `nano`/`core` profiles still do not. --- Topics: ADR-0018 — Litric Browser renders with Chromium, driven over CDP by a socket-activated broker, Context, Decision, Why not the alternatives, LBP/1 — the shell ↔ broker wire protocol, Commands (client → server), E

### `docs/adr/0019-litric-files.md`

- Inventory `F0156`; 6641 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 107 lines; SHA-256 `d194d9141881b41e44c597280ccfda11e415a8b19b638683a615d22acfc6892b`.
- Scope summary: names and `litric-trash` for what deleting means. --- Topics: ADR-0019 — Litric Files: a confined, socket-activated file daemon and an Explorer-style window, Context, Decision, Residual risk accepted, LFP/1 — the shell ↔ daemon wire protocol, Partial failure, Extraction li

### `docs/adr/0020-unified-application-compatibility.md`

- Inventory `F0157`; 6484 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 129 lines; SHA-256 `e58537e8a564dc5b164875d111c13560913f65cd35561fcb6a7de2f307c2b207`.
- Scope summary: exists and is tested (`litric-winbridge`), environment policy exists and is tested (`litric-env`). Windows and Linux as KVM guests) SYSTEM-SCOPE Part F3 (Windows compatibility) and Part J1 (multi-environment). --- Topics: ADR-0020 — One desktop, three runtimes: unified application compatibility, Requirement, The decision, 1. One desktop, by construction, 2. The Application Manager is a decision function, 3. The Linux "compatibili

### `docs/adr/0021-deep-recovery-data-protection.md`

- Inventory `F0158`; 7879 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 158 lines; SHA-256 `0293e17891e5f71e1309a10d187fc9b5ec3fcf007f6b6986c5409331b184732b`.
- Scope summary: and environment health/containment are tested in `litric-env`; the boot chain (UKI + dm-verity, fail-closed) is demonstrated in GATE 01. `master-plan/06-storage-data-lifecycle.md`, ADR-0015 §"What LitricOS keeps owning while a guest is in front" story must out Topics: ADR-0021 — Platform authority, deep recovery and data protection, Requirement, The decision, 1. Exactly one owner, 2. Four things that must never be confused, 3. Generations and activation, 4. User data is a fir

### `docs/adr/0022-defender-architecture.md`

- Inventory `F0159`; 8449 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 150 lines; SHA-256 `45bc559abac81b369ea57435c3fdea0235c382239ece500fa45e96bf741aa347`.
- Scope summary: third-party dependencies, std-only); Phases 1–5 specified with entry conditions. sandboxing.md`, SYSTEM-SCOPE E3 (Defender rows 529–546) YARA + BPF-LSM; **ClamAV is on-demand only** (a resident clamd costs ~500 MB–1 GB RSS and would break the ≤600 MB idle budg Topics: ADR-0022 — Litric Defender: architecture, roadmap and the honesty contract, Requirement, The decision, 1. Layered engine, decision separated from enforcement, 2. The policy core is a pure function, 3. The verdic

### `docs/adr/0023-wave1-system-providers.md`

- Inventory `F0160`; 12845 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 194 lines; SHA-256 `fabddeb5c083821e3deaa1cd0af67f2a458d20b0a930d6768ea3fa38de3b5add`.
- Scope summary: **Date:** 2026-10-11 **Status:** ACCEPTED for Phase 1 (facts, configuration, provider management) — Phase 2 (Bluetooth and authentication) is decided in principle and blocked on the helper decisions named below. **Implemented 2026-10-11:** the facts half of de Topics: ADR-0023 — Wave 1 reaches system providers through files, sysfs and the project's own socket, and takes no D-Bus dependency, The problem this solves, The decisions, 1. No D-Bus dependency in this wave. The three

## `ai-prompts/` (4 entries)

### `docs/ai-prompts/deepseek/README.md`

- Inventory `F0161`; 3020 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 53 lines; SHA-256 `c3e99dda0f1c0a7293d60f76a0475d17cca7a0459341cfdeb89b2202bb283f5d`.
- Scope summary: These files are **prompts for a human to carry to DeepSeek**. Nothing in this repository, and nothing in the agent environment this pass ran in, can invoke a model directly: there is no DeepSeek API key configured here, no model-call tool available, and no rou Topics: DeepSeek task prompts — how these are used in this project, What is true about invocation, Rules for every ticket in this directory, Tickets

### `docs/ai-prompts/deepseek/TASK-0001-dead-systemd-units.md`

- Inventory `F0162`; 9499 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 169 lines; SHA-256 `4aed20364a7742434c7f600b6432a103c8015b4d4a2def4427cfb3b7d7937232`.
- Scope summary: The five units were **moved out of the installed unit directory** to [`image/units-pending/`](../../../image/units-pending/README.md) rather than deleted, because the checkout had no version control at the time of the change (`docs/audit/2026-10-11-beta-1.0-st Topics: TASK LITRICOS-W0-0001 — delete the systemd units whose binaries do not exist, 0. Outcome — 2026-10-11 (written after execution), 1. Context from verified repository, 2. One objective, 3. Non-goals, 4. Required i

### `docs/ai-prompts/deepseek/TASK-0002-doc-contradictions.md`

- Inventory `F0163`; 9669 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 154 lines; SHA-256 `4c86c74e5a44f84605cf18af5721d1a7d30ee8548e9641e12348ba1c38f3f754`.
- Scope summary: All three contradictions are corrected, and a fourth was found while correcting them. Rather than retarget RISK-010 to the risk the ticket proposed (§5.3), the version-control gap is recorded as its own entry, **RISK-015**, with the mitigation that actually ha Topics: TASK LITRICOS-W0-0002 — correct the documentation contradictions against the tree, 0. Outcome — 2026-10-11 (written after execution), 1. Context from verified repository, 2. One objective, 3. Non-goals, 4. Requi

### `docs/ai-prompts/deepseek/TASK-0003-dependency-policy-check.md`

- Inventory `F0164`; 11446 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 189 lines; SHA-256 `a3844dcfdd3d9d33ad95c43d7a8e6caf95e93575135e97c623b288c4d0d682ec`.
- Scope summary: `ci/lint-deps.py` exists, is wired into `ci/verify.sh` as section **[3h]**, and passes on the real tree: `18 member(s): 16 covered by the no-dependency policy, 2 outside it (applications, native ecosystem)`, 0 violations, exit 0. **The rule implemented.** Topics: TASK LITRICOS-W0-0003 — enforce the dependency policy instead of asserting it, 0. Outcome — 2026-10-11 (written after execution), 1. Context from verified repository, 2. One objective, 3. Non-goals, 4. Required 

## `audit/` (4 entries)

### `docs/audit/2026-10-09-continuation-recovery-audit.md`

- Inventory `F0165`; 30722 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 348 lines; SHA-256 `456e473ccb60de90fe21e74662c30af6f4e4f49ccd17f4246e1b012c5fc84e69`.
- Scope summary: **Scope:** memory-budget enforcement · file-operation integrity · defender/benchmark counter forensics · repository evidence · bounded hardening **Mode:** code-only. No build, no compile, no cargo, no mkosi, no QEMU, no GUI, no install, no host mutation, no se Topics: LitricOS — Continuation Audit, 2026-10-09, สรุปผู้บริหาร (Thai executive summary), 1. Scope and constraints, 2. Repository state before the changes, 3. Memory budget (Phase 1), 4. File operations and data integr

### `docs/audit/2026-10-11-beta-1.0-stage-a-audit.md`

- Inventory `F0166`; 20900 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 308 lines; SHA-256 `59cf89cae3e52a7bab973150697b8863fcf02d5c9dbc2641c20b1a38cd64dc97`.
- Scope summary: **Date:** 2026-10-11 **Scope:** Stage A of the LitricOS Beta 1.0 E2E prompt set (repository truth before code). No implementation was performed in this pass; nothing below changes the tree. **Host:** Windows 11 (`MINGW64_NT-10.0`, the same host recorded in [EV Topics: Stage A — Repository discovery and evidence ledger (Beta 1.0 E2E pass), 1. Repository root and version-control state, 2. Workspace shape and the constraints every patch must satisfy, 3. What this operating syste

### `docs/audit/2026-10-15-bug-0009-fix-and-ui-round-1.md`

- Inventory `F0167`; 18541 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 265 lines; SHA-256 `b77cd87eef74e73f8f93abeb848762429cf8fd9edcaa7c4824218629989ea85c`.
- Scope summary: **Scope:** the confirmed open interoperability bug (BUG-0009, the trash `Path=` field) fixed across the daemon and the shell in one branch, with regression tests and legacy-entry recovery; the UI round-1 patch applied and verified; the tree put under version c Topics: 2026-10-15 — BUG-0009 + UI round 1: what landed, and the evidence, สรุปผู้บริหาร (Thai executive summary), 1. Was it usable? — yes, and here is the number, 2. Environment findings that shaped the work, 3. BUG-00

### `docs/audit/2026-10-16-text-stack-and-review-loop.md`

- Inventory `F0168`; 12328 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 193 lines; SHA-256 `09837a85d038101c4e1d7f96af37d77a9b65cad09ed8c731e26e015b3dd02a11`.
- Scope summary: **Scope:** two defects that made UI work unjudgeable — one `Style` drawing at two different sizes depending on which text path is live, and a language being printed by a typeface that has no glyphs for it — plus the tooling that makes a frame possible on a hos Topics: 2026-10-16 — the text stack told the truth, and the review loop reopened, สรุปผู้บริหาร (Thai), 1. Defect A — one `Style`, two sizes, 2. Defect B — printing a language the typeface cannot draw, 3. Defect C — the

## `bugs/` (22 entries)

### `docs/bugs/BUG-0001-fabricated-mkosi-config.md`

- Inventory `F0169`; 3796 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 98 lines; SHA-256 `3c0c6fb73de2bd348a8744cf46d7b34f9d4ebf7c12f15a89a42b183ea5e33ec5`.
- Scope summary: Bug ID        : BUG-0001 Severity      : S2 (HIGH) — build pipeline cannot execute; false GATE 01 evidence Class         : TOOLCHAIN + DOC (see PROTOCOL §8 bug-class matrix) Subsystem     : image/ (build pipeline) First Observed: 2026-08-26, audit of the pre-v Topics: BUG-0001 — Fabricated mkosi configuration schema, Reproducer, Expected, Actual, Impact, Root Cause, Why Existing Tests Missed It, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0002-smoke-test-never-executed.md`

- Inventory `F0170`; 3999 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 98 lines; SHA-256 `52f850bc8384ec947d4d2e84b65536ee17c2935a225603995be4819794acb50d`.
- Scope summary: Bug ID        : BUG-0002 Severity      : S2 (HIGH) — the only check gating GATE 01 was itself broken Class         : TEST (PROTOCOL §8: false-negative test) Subsystem     : tests/ First Observed: 2026-08-26, ci/verify.sh first run python -m py_compile tests/sm Topics: BUG-0002 — The boot smoke test was truncated and could not run, Reproducer, Expected, Actual, Three further defects found while repairing it, Impact, Root Cause, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0003-lsm-stack-was-two-not-seven.md`

- Inventory `F0171`; 6438 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 148 lines; SHA-256 `36bf9b0ac4817feaa923dec05a7ef54f727149ae373514ac5c0be88e1858e8f0`.
- Scope summary: Bug ID        : BUG-0003 Severity      : S1 (CRITICAL) — the Phase 07 sandbox was substantially absent while the configuration read as though it were present Class         : SEC (PROTOCOL §8) Subsystem     : kernel/config/ First Observed: 2026-08-26, first run Topics: BUG-0003 — The LSM stack listed seven modules and enabled two, Reproducer, Expected, Actual, Impact, Root Cause, Why Existing Tests Missed It, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0004-loadpin-denied-every-module.md`

- Inventory `F0172`; 5163 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 117 lines; SHA-256 `4caccd9a74a4f0b5588e3c71de06d3766ef2511cb811792921e7a379b23c3525`.
- Scope summary: Bug ID        : BUG-0004 Severity      : S1 (CRITICAL) — a security control silently disabled most of the running system, including the ESP, BBR and tun Class         : SEC (PROTOCOL §8) Subsystem     : kernel/config, image/ First Observed: 2026-08-28, first b Topics: BUG-0004 — LoadPin denied every kernel module after switch-root, Reproducer, Expected, Actual, Impact, Root Cause, Why Existing Tests Missed It, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0005-smoke-test-could-not-fail.md`

- Inventory `F0173`; 9608 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 189 lines; SHA-256 `1e03417ffb9a96fee971f6a8aa9a16530fce88efc116dbdbe15f79f58a385dad`.
- Scope summary: Bug ID        : BUG-0005 Severity      : S1 (CRITICAL) — the only test standing between "we built an image" and "the image boots" reported a wedged guest as a hang to be investigated and a failed boot as PASS Class         : TEST (PROTOCOL §8: false-negative a Topics: BUG-0005 — The boot smoke test could not time out, could pass on a broken boot, and looked for artifacts that no longer exist, Reproducer, Expected, Actual — four defects, all in one 329-line file, 1. The timeou

### `docs/bugs/BUG-0006-gate01-verifier-aborted-and-could-not-pass.md`

- Inventory `F0174`; 7283 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 154 lines; SHA-256 `82f37e0d7011b5b29588c56e03866999b902ff78cd0778fe8eadcc0ffdfd12ab`.
- Scope summary: Bug ID        : BUG-0006 Severity      : S1 (CRITICAL) — the in-VM verifier named by GATE 01 could not complete a run, and three of its checks were guaranteed to FAIL on a machine that was working correctly Class         : TEST (PROTOCOL §8: false-negative tes Topics: BUG-0006 — The GATE 01 verifier aborted after one check, and three of its probes could never pass, Reproducer, Expected, Actual, 1. `((PASS++))` under `set -e` killed the script at the first successful check, 2.

### `docs/bugs/BUG-0007-crlf-in-five-tool-scripts.md`

- Inventory `F0175`; 3700 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 96 lines; SHA-256 `728a1f8aea44df074ad45a1b1630be8bb9c13662676b713aec6727b379c5f8c2`.
- Scope summary: Bug ID        : BUG-0007 Severity      : S3 (MEDIUM) — no data loss and no boot impact, but the repository's central verification run reported FAIL, and five executable Python tools could not be run via their shebang Class         : BUILD-HYGIENE (PROTOCOL LAW Topics: BUG-0007 — Five tool scripts had CRLF line endings, and `ci/verify.sh` was red because of it, Reproducer, Expected, Actual, Impact, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0008-memory-budget-claims-unenforced-limits.md`

- Inventory `F0176`; 10709 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 204 lines; SHA-256 `0ccf91ec7f97e0b189ed9df65525cb93de057017a84b6df96219dfd819924f42`.
- Scope summary: Bug ID        : BUG-0008 Severity      : S1 (CRITICAL) — the document that decides whether the desktop is admissible described enforcement that does not exist, and the largest component's documented behaviour is not the behaviour the units produce Class        Topics: BUG-0008 — The memory budget claimed limits that were not configured, and a slice that governed nothing, Reproducer, Expected, Actual — fourteen divergences, in four groups, 1. Nine rows claimed limits the image

### `docs/bugs/BUG-0009-trash-path-field-is-not-a-location.md`

- Inventory `F0177`; 9365 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 162 lines; SHA-256 `4e4f4797f4d660d32b1deb1404f1a1086565975775b4d94d62fa7384c266b673`.
- Scope summary: Bug ID        : BUG-0009 Severity      : S3 (MEDIUM) — interoperability, not loss: another tool reading Litric's trash restores the file to the wrong place Class         : SPEC-CONFORMANCE (Freedesktop Trash 1.0, `Path=`) Subsystem     : src/litric-files/src/{ Topics: BUG-0009 — The trash `Path=` field carried a URI, not the original location, Reproducer, Expected, Actual, Impact, Root Cause, Fix, Regression tests added, Verification, Remaining work

### `docs/bugs/BUG-0010-replace-deleted-the-item-it-was-replacing.md`

- Inventory `F0178`; 5098 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 112 lines; SHA-256 `754c2acf2c28b96e75281987e0583cf4e613dae249ced13ca3c52a66c113c336`.
- Scope summary: Bug ID        : BUG-0010 Severity      : S2 (HIGH) — user data loss on a failed copy/move; the documented contract promised the opposite Class         : DATA-INTEGRITY (failure semantics of a public operation) Subsystem     : src/litric-files/src/ops.rs First  Topics: BUG-0010 — `Policy::Replace` deleted the item it was replacing before the new copy existed, Reproducer, Expected, Actual, Impact, Root Cause, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0011-trash-moved-the-item-before-its-record-existed.md`

- Inventory `F0179`; 6657 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 132 lines; SHA-256 `7003541c21fda703ef3a525482b8d2a1f7f0dc501278e0f8bd9b835077ae66e1`.
- Scope summary: Bug ID        : BUG-0011 Severity      : S2 (HIGH) — a trashing that reports failure can still remove the item from its place, and a later same-name trash could overwrite an earlier trashed copy Class         : DATA-INTEGRITY / SPEC-CONFORMANCE (Freedesktop Tr Topics: BUG-0011 — The trash moved the item before its record existed, and never reserved the name, Reproducer, Expected, Actual, Root Cause, Fix, Regression Test, Verification, The second implementation (same session),

### `docs/bugs/BUG-0012-extraction-aborted-on-entries-it-should-accept.md`

- Inventory `F0180`; 4382 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 90 lines; SHA-256 `b3b11d5962980ca1b4c9d02a5c192e4fd75c8ac4a07fe38b3e09c411615589ea`.
- Scope summary: Bug ID        : BUG-0012 Severity      : S3 (MEDIUM) — a valid archive fails, or fails part-way with the entries already written left behind Class         : FUNCTIONAL / PARTIAL-FAILURE SEMANTICS Subsystem     : src/litric-files/src/archive.rs First Observed:  Topics: BUG-0012 — Extraction aborted on an archive it should have accepted, Reproducer, Defect 1 — the suffix match was case-sensitive while the classification is not, Defect 2 — a directory entry that is already there

### `docs/bugs/BUG-0013-resolve-checked-nothing-about-its-intermediates.md`

- Inventory `F0181`; 3324 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 82 lines; SHA-256 `003ec93a17018020f7f4549a551abad4b543a64124fdb2e4625537d5e22fce65`.
- Scope summary: Bug ID        : BUG-0013 Severity      : S3 (MEDIUM) — a documented confinement rule was not implemented; the failure surfaced as the wrong error for the wrong reason Class         : CONFINEMENT / DOCUMENTATION-INTEGRITY (ADR-0019 §3) Subsystem     : src/litri Topics: BUG-0013 — `Root::resolve` checked nothing about the components it walked through, Reproducer, Expected, Actual, Impact, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0014-benchmark-statistics-narrowed-counters-to-u32.md`

- Inventory `F0182`; 5164 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 106 lines; SHA-256 `787eb4d9195d331db82b6dc6d935a8aad9f18579b8e6bf20acc11b2d869f6fbb`.
- Scope summary: Bug ID        : BUG-0014 Severity      : S3 (MEDIUM) — a plausible-looking number that no sample produced is exactly what this harness exists to prevent Class         : COUNTER / MEASUREMENT INTEGRITY Subsystem     : src/litric-bench/src/severity.rs, src/litri Topics: BUG-0014 — The benchmark narrowed 64-bit counters and latencies to 32 bits, Reproducer, Actual, Fix, Regression Test, Verification, Residual Risk

### `docs/bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md`

- Inventory `F0183`; 11768 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 220 lines; SHA-256 `7b619f88e0b445ea32165f91cc37dc4e5386fbb7372fa6c75d9ddc57a83ea00f`.
- Scope summary: Bug ID        : BUG-0015 Severity      : S2 (HIGH) — every motion phase and every interaction is gated on this number, and the number is 4x-12x over the stated budget Class         : PERFORMANCE / RENDERER Subsystem     : src/litric-shell/src/main.rs (render l Topics: BUG-0015 — A frame with a window costs 206 ms, against a 16.67 ms budget, Reproducer, Actual, Expected, Why this is not a warm-up artefact, The damage path, measured (question answered 2026-10-09), Run-to-run va

### `docs/bugs/BUG-0016-the-typeface-printed-boxes-and-the-grid-collided.md`

- Inventory `F0184`; 6446 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 116 lines; SHA-256 `83b2303e6d4ffaafb9f2f218e4571b301ec22d113e79a727bec21753e81962b4`.
- Scope summary: Bug ID        : BUG-0016 Severity      : S3 (MEDIUM) — every window is legible, but the marks a person reads most (a sort arrow, a folder's size, a close button) were wrong, and two labels sat on top of each other Class         : TYPOGRAPHY / LAYOUT Subsystem  Topics: BUG-0016 — The typeface printed boxes, letters touched, and three layouts collided, Reproducer, Actual, and what each item was, 1. The face was missing the glyphs the interface prints, 2. Letters touched each ot

### `docs/bugs/BUG-0017-smoke-test-listened-to-the-wrong-console.md`

- Inventory `F0185`; 5439 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 108 lines; SHA-256 `750ec0d4bf7fbfc5e72545f9a99085b8f25ef6a3eeb4e947dacfbe0bc7c224f9`.
- Scope summary: Bug ID        : BUG-0017 Run           : the first image built from this tree, 2026-10-11 No collision  : BUG-0016 is *The typeface printed boxes, letters touched, and three layouts collided*, which is a different bug in a different subsystem. This record was  Topics: BUG-0017 — the boot smoke test listened to a console that never gets a prompt, What was wrong, The measurement, The fix, Proof the fix works, and that it did not become a rubber stamp, What is still not establis

### `docs/bugs/BUG-0018-a-locked-frame-costs-half-a-second.md`

- Inventory `F0186`; 11285 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 198 lines; SHA-256 `6aacc12a509170f9e380f6a16f334038fae9d4c1f60b3fff41d191effb56427d`.
- Scope summary: Bug ID        : BUG-0018 Severity      : S2 (HIGH) — the lock screen is the shell's only steady state at 30 FPS, and it is 17x-24x over its own interval Class         : PERFORMANCE / RENDERER Subsystem     : src/litric-shell/src/lock.rs (the surface), src/litr Topics: BUG-0018 — A locked frame costs 0.57 s, against the 33 ms interval it asks for, Reproducer, Actual, Expected, Why the frame is expensive, from the numbers, What the split showed, and the four changes, Remaining 

### `docs/bugs/BUG-0019-a-snap-that-changed-the-value-reported-no-frame.md`

- Inventory `F0187`; 6445 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 150 lines; SHA-256 `1c09222b9e7c427110d225068ad0965f68d1b176acc12b0f4ed773a05fb93549`.
- Scope summary: Bug ID        : BUG-0019 Severity      : S3 (MEDIUM) — a contained rendering-correctness defect: the pixels a spring drives stay wrong until some unrelated repaint Class         : UI / RENDERER (damage tracking) Subsystem     : src/litric-shell/src/anim.rs (`A Topics: BUG-0019 — A spring that snapped to its target reported "nothing moved", Reproducer, Actual, Root cause, Why the existing tests missed it, Fix, Regression tests, Verification, Residual risk

### `docs/bugs/BUG-0020-the-initrd-shipped-with-no-kernel-modules.md`

- Inventory `F0188`; 5734 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 135 lines; SHA-256 `bbf3bcc703e51244542655d4e5a8192381d35102fd28df16ae7a0ee47ec88fe2`.
- Scope summary: Bug ID        : BUG-0020 Severity      : S1 (CRITICAL) — the image does not boot: it reaches an emergency shell instead of a desktop, and nothing above the initrd runs Class         : BUILD / IMAGE (kernel packaging) Subsystem     : image/mkosi.conf (`KernelIn Topics: BUG-0020 — The initrd shipped with no kernel modules, so the root never mounted, Reproducer, on the Linux builder, Actual, Root Cause, Fix, What this teaches (LAW 10), Verification, the initrd in the rebuilt UKI

### `docs/bugs/BUG-0021-the-image-booted-by-firmware-landed-in-emergency-mode.md`

- Inventory `F0189`; 4236 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 106 lines; SHA-256 `b50807ddb689fb9012cd3b07148172ce9e0cfe0572748e489a6ab3616a5ff9c9`.
- Scope summary: Bug ID        : BUG-0021 Severity      : S1 (CRITICAL) — the shipped disk image does not boot on real hardware or in any VM that boots through its own bootloader Class         : BOOT / IMAGE CONFIGURATION Subsystem     : image/mkosi.conf (KernelCommandLine=),  Topics: BUG-0021 — The image booted by firmware landed in emergency mode, Symptom, Root cause, Why every earlier boot missed it, Fix, Verification

### `docs/bugs/BUG-0022-the-first-boot-prompt-flashed-and-disappeared.md`

- Inventory `F0190`; 3707 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 89 lines; SHA-256 `184bb2e1256c3f355217e8e6caea3c1286a779e97a3663305f5735094f58c6c1`.
- Scope summary: Bug ID        : BUG-0022 Severity      : S1 (CRITICAL) — no image built without `ncurses` ever reaches the desktop on its first boot Class         : BOOT / FIRST RUN Subsystem     : image/mkosi.extra/usr/lib/litric/first-run.sh, image/mkosi.extra/usr/lib/syste Topics: BUG-0022 — The first-boot prompt flashed and disappeared, Symptom, Root cause, Fix, Verification

## `design/` (8 entries)

### `docs/design/2026-10-16-why-it-reads-as-a-mockup.md`

- Inventory `F0191`; 11544 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 171 lines; SHA-256 `1e5a46896ea9ea09a5555e5a6b8f8352795b82443b077a3bc4b2fded4418e75b`.
- Scope summary: **สถานะ:** รีวิวจากโค้ดจริง + ภาพจริง (VM 1920×1080, frame dump 1280×800) + เอกสาร design ทั้งชุด **ขอบเขต:** เชลล์ทั้งใบ (top bar, desktop, dock/launcher, lock, 8 แอป) — ไม่ใช่ไฟล์เดียว **ผลลัพธ์ที่ต้องการ:** เดสก์ท็อปที่ "เป็นระบบจริง" แบบ macOS: ทุกอย่างมีช Topics: 2026-10-16 — ทำไม UI ยังอ่านเป็น "mockup" และแผนแก้ทั้งเครือข่าย, สรุปผู้บริหาร (Thai), 1. What I actually looked at, 2. The five causes, with evidence, A. Two text paths disagree about what a pixel size means —

### `docs/design/precision-atelier-prototype.html`

- Inventory `F0192`; 31913 bytes; HTML; role `architecture-or-phase-doc`.
- Content fingerprint: 349 lines; SHA-256 `387d2620c3fe32d6da636caec8c2dc9f02d7e041a7c81fe9ffc5e5563b947985`.
- Scope summary: <!doctype html> <html lang="en"> <head> <meta charset="utf-8"> <meta name="viewport" content="width=device-width, initial-scale=1"> <meta name="color-scheme" content="dark light"> <title>LitricOS — Precision Atelier prototype</title> <style> :root { color-sche

### `docs/design/precision-atelier.md`

- Inventory `F0193`; 6036 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 94 lines; SHA-256 `7739924697bedaf1353a6c3163ef93ba6865bbbf2e7ba95609a4c3fb42c666ea`.
- Scope summary: This is the product design contract for the complete LitricOS shell refresh. It covers the lock and first-run surfaces, desktop chrome, system panels, dialogs, and all eight built-in applications. The attached QEMU screenshot is a visual reference for the curr Topics: LitricOS Precision Atelier, Purpose, Visual direction, Foundation rules, Product surface inventory, Quality bar, Delivery order

### `docs/design/wave1/00-network.md`

- Inventory `F0194`; 11917 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 169 lines; SHA-256 `8088bc94070697693ab6e312bd62ac5f205bb322ec33ad40148f639633cf4a6a`.
- Scope summary: **Status:** Phase 1 design accepted ([ADR-0023](../adr/0023-wave1-system-providers.md)). **Facts implemented** in `src/litric-sysd/src/platform/net.rs`, fixture-tested on any host, and since 2026-10-12 **four actions are implemented too** in `src/litric-sysd/s Topics: Wave 1 — network, 1. What exists in the tree today, 2. The provider, and how it is reached, 3. Data surface (facts), 4. Action surface (capabilities), 5. Threat model, 6. What is verifiable today, 7. NOT verifie

### `docs/design/wave1/01-bluetooth.md`

- Inventory `F0195`; 9446 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 123 lines; SHA-256 `b4625836ae80a2050e6d85650583d353b431492648e873a31a0eb3b110e02ca7`.
- Scope summary: **Status:** **the two decisions below are answered, and the actions are implemented as of 2026-10-12.** The *facts* are in `src/litric-sysd/src/platform/bluetooth.rs` (adapter presence, the adapter's address, `rfkill` soft and hard blocks, whether `bluetoothd` Topics: Wave 1 — Bluetooth, 1. What exists in the tree today, 2. The two decisions this note exists to force, 3. If the answer is yes: the shape, 4. Threat model (for the shape above), 5. What is verifiable today, 6. NO

### `docs/design/wave1/02-audio.md`

- Inventory `F0196`; 10065 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 151 lines; SHA-256 `d4bd84cad112d769f89b6aaf522534fe0159a5fcb17a7f4c2e16ec0bac19e171`.
- Scope summary: **Status:** Phase 1 design accepted ([ADR-0023](../adr/0023-wave1-system-providers.md)). **State and control are both implemented.** `src/litric-sysd/src/platform/audio.rs` reports cards, PCM devices, sound-server presence and which tool can change a level, fi Topics: Wave 1 — audio, 1. What exists in the tree today, 2. The provider, and how it is reached, 3. Data surface (facts), 4. Action surface (capabilities), 5. Threat model, 6. What is verifiable today, 7. NOT verified,

### `docs/design/wave1/03-power.md`

- Inventory `F0197`; 12082 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 170 lines; SHA-256 `c73348ec8eb730b2649156e81c9eca195ab15df418fc0e997eee14613e071621`.
- Scope summary: **Status:** Phase 1 design accepted ([ADR-0023](../adr/0023-wave1-system-providers.md)). **Facts implemented** in `src/litric-sysd/src/platform/power.rs` (supplies, thermal zones, cooling devices, backlights), fixture-tested on any host. **Session actions are  Topics: Wave 1 — power, 1. What exists in the tree today, 2. The provider, and how it is reached, 3. Data surface (facts), 4. Action surface (capabilities), 5. Threat model, 6. What is verifiable today, 7. NOT verified,

### `docs/design/wave1/04-authentication.md`

- Inventory `F0198`; 15551 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 180 lines; SHA-256 `c52fd137ef6b385c29f41894e61a6b6b680a1f32cfc714a27539939f2c58b694`.
- Scope summary: **Status:** **a password can be verified, and since 2026-10-12 there is a lock surface that uses it.** `litric-sysd` answers `AUTH VERIFY` through `/usr/bin/unix_chkpwd` — the helper `pam_unix` itself runs, setuid root, password on standard input — and `Verifi Topics: Wave 1 — authentication, sessions and the lock screen, 1. What exists in the tree today, 2. The three decisions this note forces, 3. Data surface (facts, Phase 1), 4. Action surface (capabilities), 5. Threat mod

## `master-plan/` (27 entries)

### `docs/master-plan/00-vision-and-non-goals.md`

- Inventory `F0316`; 2557 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 55 lines; SHA-256 `c8354e877682f71b07b053daad471b8277493d10ade48548736029e02d55b199`.
- Scope summary: LitricOS is a native-first personal computing platform for x86-64 laptops and desktops with 8 GB RAM or more. It is designed to make everyday work feel immediate, private, recoverable, and resource-efficient. Its differentiator is not a claim to replace every  Topics: 00 — Vision and non-goals, Product statement, Success criteria, Target users, Non-goals for v1, Product principles, Decision gates

### `docs/master-plan/01-product-requirements.md`

- Inventory `F0317`; 2331 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 55 lines; SHA-256 `b7223af306d87c8a597dcded2f92d48d3e0990b7e962e75c64647f83c16f0251`.
- Scope summary: Wayland client, and recovers from a failed staged update. network, audio, settings, diagnostics, and an initial HCL work end-to-end. Topics: 01 — Product requirements, Supported baseline, Release outcomes, FND — system proof, DP — platform proof, BETA — daily-use validation, V1 — supported promise, User journeys that cannot regress, Product metrics

### `docs/master-plan/02-system-architecture.md`

- Inventory `F0318`; 2109 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 50 lines; SHA-256 `0da5b552297e1b1f4dfeba2d91a5da44ae8998d3c69f18cb007d6d10e78745e1`.
- Scope summary: Firmware → signed UKI → Linux LTS / drivers → Litric core services → stable public contracts → SDK / native or sandboxed apps Windows or Linux app → compatibility environment → Litric contracts → kernel Topics: 02 — System architecture, Dependency direction, Trust and ownership, Mandatory boundaries, Architectural invariants, Required ADR triggers

### `docs/master-plan/03-native-contract-governance.md`

- Inventory `F0319`; 2290 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 49 lines; SHA-256 `2dc4de6a02a2517a4bb29c16a0ca029f065a030a6ad43aba6d08ea4de659d3b4`.
- Scope summary: Existing conceptual contracts in `docs/API-00-contracts.md` are the starting point; they are not implementation proof. Each accepted API must acquire a machine-readable schema, error taxonomy, ownership rules, cancellation semantics, threading model, permissio Topics: 03 — Native contract governance, Contract layers, v1 public service families, Compatibility policy, Rules for handles and IPC, Acceptance tests

### `docs/master-plan/04-boot-integrity-recovery.md`

- Inventory `F0320`; 1970 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 43 lines; SHA-256 `e44c7829fe14588d03bc90e49974f094bbd3fee7920cf778ac8511f251877ca3`.
- Scope summary: firmware → Secure Boot verification → signed UKI → initramfs → verify root image → unlock mutable data → health checks → selected slot → mark healthy / decrement boot attempt / rollback No stage may silently continue after an integrity failure. Recovery is a s Topics: 04 — Boot integrity and recovery, Boot state machine, Required implementation milestones, Recovery functions, Fault tests

### `docs/master-plan/05-kernel-driver-strategy.md`

- Inventory `F0321`; 2210 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 47 lines; SHA-256 `e8fee0b4cadda4521f8005512eb1b5bb753bcf0839c3eb92a28c141ef9c3c205`.
- Scope summary: Litric ships a supported Linux LTS kernel with a reviewable Litric config and minimal downstream patches. The goal is not the smallest theoretical config; it is a bootable, secure, updateable kernel whose enabled features are evidenced on the actual image and  Topics: 05 — Kernel and driver strategy, Decision, Configuration policy, Driver and firmware policy, Hardware enablement order, Test gates

### `docs/master-plan/06-storage-data-lifecycle.md`

- Inventory `F0322`; 1903 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 42 lines; SHA-256 `3270f7e505890c80e10fc09e06d6e435fb5ecba49df2ec04f0ac06b8504382ab`.
- Scope summary: metadata without separate confirmed action. distinguish code, cache, settings, and user-created data. `cache://`, `config://`, `secure://`, `temp://`); physical paths are private. records the source snapshot, destination, and final decision. 1. Implement Btrfs Topics: 06 — Storage and data lifecycle, Layout, Data invariants, Functional work, Failure tests

### `docs/master-plan/07-compositor-shell-protocol.md`

- Inventory `F0323`; 2322 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 48 lines; SHA-256 `4a6c8e4521ba5a0df2a595023dddf6f4a3d021d60372e4fc6ae7c17987f94dcd`.
- Scope summary: The current framebuffer shell demonstrates Litric-owned presentation and input; it is not yet proof of a client desktop. Production work must introduce a Smithay/Wayland compositor with native clients while keeping shell policy and untrusted decoding outside t Topics: 07 — Compositor and shell protocol, Production boundary, Responsibilities, Protocol milestones, Reliability and performance rules, Test matrix

### `docs/master-plan/08-accessibility-i18n-input.md`

- Inventory `F0324`; 1721 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 38 lines; SHA-256 `6d33930b34445dd6af0c763c60bb119e290fcc3bd192956878cd3eeef5dbe15a`.
- Scope summary: Every Litric-owned interface must be operable with keyboard alone, expose a semantic accessibility tree, preserve focus across asynchronous updates, show focus visibly, honour high contrast/text scale/reduced motion, and never make a security confirmation mous Topics: 08 — Accessibility, localisation, and input, v1 accessibility contract, Required features, Design rules, Verification

### `docs/master-plan/09-app-model-portals-packaging.md`

- Inventory `F0325`; 2014 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 46 lines; SHA-256 `eab502b61f6e05d9090c49af02f0d61f441b62a604da1b66ef53b465302b9f8f`.
- Scope summary: Applications are distributed as signed Flatpak packages with Litric desktop integration. Litric adds a stable app identity, capability policy, resource class, diagnostics and UX; it does not create a second general-purpose package format in v1. by default. Rev Topics: 09 — Application model, portals, and packaging, v1 model, Application manifest additions, Capability model, Lifecycle, Acceptance tests

### `docs/master-plan/10-native-sdk-and-devtools.md`

- Inventory `F0326`; 1980 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 44 lines; SHA-256 `c9d16c4732831f95259b1f60f2325d233825b84849cfd536a3e8bc5bde1e0b9c`.
- Scope summary: The SDK makes the safe path ordinary: versioned public interfaces, capability aware APIs, cancellable operations, explicit ownership, useful errors and stable debugging symbols. Applications target Litric contracts, not shell internals, private D-Bus names or  Topics: 10 — Native SDK and developer tools, SDK promise, v1 SDK deliverables, API requirements, Tooling quality gates, Deferred decision

### `docs/master-plan/11-system-services-network-audio.md`

- Inventory `F0327`; 2105 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 47 lines; SHA-256 `70f499e8bb3a2ce4691e283eabd68b2dbd1645cca6146ad7f9dcc7feaa4a980d`.
- Scope summary: System services run under systemd with an explicit unit owner, privilege set, capability boundary, restart policy, cgroup slice, readiness condition, log budget, health probe and shutdown ordering. Socket/dbus activation is preferred for non-essential services Topics: 11 — System services, network, and audio, Service model, Core service catalogue, Networking, Audio, Tests

### `docs/master-plan/12-threat-model-and-sandboxing.md`

- Inventory `F0328`; 2012 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 46 lines; SHA-256 `dd008a3b179b1c4496d8cb2a21e4d3d9002a36cef571d7975b266004a07d0a41`.
- Scope summary: Protect user files, secrets, microphone/camera, clipboard, network identity, system image, signing keys, recovery keys and integrity metadata from malicious apps, hostile files, USB devices, network attackers, compromised repositories, physical attackers and m Topics: 12 — Threat model and sandboxing, Assets and adversaries, Defence composition, Required policies, Red-team cases

### `docs/master-plan/13-supply-chain-updates.md`

- Inventory `F0329`; 1852 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 40 lines; SHA-256 `b8ade0346381c3606f20bbe8786fa7dd91f9ce8232710a802554da8d241c7eda`.
- Scope summary: `source revision → hermetic/reproducible build → SBOM/provenance → signed image and metadata → verified download → staged A/B target → health-confirmed boot`. Each link records the build inputs, toolchain, dependency lock state, artifact hash, signer identity, Topics: 13 — Supply chain and updates, Trust chain, Required controls, Update lifecycle, Test and response

### `docs/master-plan/14-performance-methodology.md`

- Inventory `F0330`; 1966 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 43 lines; SHA-256 `113d43c482ce62dd256da7e4c633ec35af94de105a8e23b8f6b1b3f3c4f84efc`.
- Scope summary: No performance claim is valid unless its artifact records image hash, Git/source revision when available, kernel/Mesa/firmware versions, device model, AC/battery state, display configuration, ambient thermal condition, test command, raw samples, sample count,  Topics: 14 — Performance methodology, Measurement contract, Workloads, Baseline policy, Budgets and regression, Experiment discipline

### `docs/master-plan/15-power-thermal-resource-engine.md`

- Inventory `F0331`; 1910 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 44 lines; SHA-256 `95fe4984908a480ae520d459a17b6e4b43b3bfc0b6f2255af04966c1013a52a1`.
- Scope summary: Treat CPU, memory, storage, GPU, network, display and thermal headroom as one policy system. Preserve foreground responsiveness and hardware safety before throughput, background completion, benchmark scores or compatibility guests. Observe cgroup CPU/memory/I/ Topics: 15 — Power, thermal, and resource engine, Objective, Resource classes, Control loop, Power rules, Tests

### `docs/master-plan/16-compatibility-environments.md`

- Inventory `F0332`; 1792 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 42 lines; SHA-256 `ccfc2b9225c686bafc4fd4fb7e38509d59b76c3cbb625c16f2db2565e35f8fff`.
- Scope summary: Compatibility removes adoption friction; it is not Litric's kernel, app model, security model, or v1 success criterion. Native SDK applications and sandboxed Flatpaks always remain supported when compatibility components are absent. An environment has immutabl Topics: 16 — Compatibility environments, Scope, Tiers, Environment model, UX rules, Test gates

### `docs/master-plan/17-test-lab-and-fuzzing.md`

- Inventory `F0333`; 1874 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 41 lines; SHA-256 `61f2a0680294b76e015b7df431d8f73ecaa136f049063a777b9d28df4d7dc8e9`.
- Scope summary: Fuzz package manifests, signatures/metadata parsing, PE inspection, protocol decoders, Wayland clients, image/media/archive helpers, URI/settings import, IPC messages, storage metadata and update metadata. Harnesses define seed corpus, coverage, time/memory ca Topics: 17 — Test lab and fuzzing, Test pyramid, Fuzz inventory, Fault injection inventory, Hardware laboratory, Definition of test completion

### `docs/master-plan/18-observability-crash-support.md`

- Inventory `F0334`; 1791 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 39 lines; SHA-256 `016c187e32f4d00811e0988a2c4904c5617c5722fb478c64055e6bfa0e47dc12`.
- Scope summary: Observability must make defects diagnosable without turning the OS into a data collector. Local diagnostics are default; export is user initiated, minimised, redacted, encrypted in transit and visible before sending. dependency failure. and capability decision Topics: 18 — Observability, crash, and support, Principles, Required records, Crash workflow, Support bundles, Quality metrics

### `docs/master-plan/19-first-run-settings-builtins.md`

- Inventory `F0335`; 1641 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 35 lines; SHA-256 `a4b6f570bd659d75aac6adcb433327e0595522f23d27e34b7d584cd4bb9977ef`.
- Scope summary: `installer → disk/encryption choice → account → locale/input → network → update policy → accessibility → privacy/AI choices → desktop`. Each step is resumable, keyboard-accessible, understandable offline, and never claims a security feature is active until it  Topics: 19 — First run, settings, and built-ins, First-run path, Settings architecture, v1 built-ins, UX acceptance

### `docs/master-plan/20-release-security-response.md`

- Inventory `F0336`; 1811 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 43 lines; SHA-256 `f19b328a61f1b549c52ee534c895d96c1a949d072432bdedfedebf4a516c1c99`.
- Scope summary: Every candidate carries signed image hashes, SBOM, provenance, release notes, known issues, HCL, update metadata, test summary, raw performance evidence, rollback instructions, recovery image and a list of intentionally unverified claims. `PASS_WITH_LIMITATION Topics: 20 — Release and security response, Release artefacts, Gate policy, Vulnerability response, Channel policy, Governance

### `docs/master-plan/21-feature-ledger-and-release-mapping.md`

- Inventory `F0337`; 2471 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 48 lines; SHA-256 `4af646a843b9033c08668d9ee293370901a5be81846fdbd397576cdacee2c0c1`.
- Scope summary: `docs/SYSTEM-SCOPE.md` contains the complete numbered feature inventory. This plan does not duplicate its 1,117 rows because duplicated feature status will drift. Instead, every row must receive operational metadata in a machine-checked ledger or adjacent tabl Topics: 21 — Feature ledger and release mapping, Source of truth, Required metadata per feature, Release mapping by section, Status rules

### `docs/master-plan/22-unified-compatibility.md`

- Inventory `F0338`; 3620 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 71 lines; SHA-256 `cfcdad50e2d21a7d87c135c8fd20423d267ea64776db0a51d263cfe5a5d47acc`.
- Scope summary: One desktop, three application ecosystems. Litric, Windows and Linux applications run directly in the Litric desktop; the user does not choose a runtime, switch desktops, or boot a second OS to use an installed application. Full Windows and Linux environments  Topics: 22 — Unified application compatibility, Requirement, Runtime selection, Invariants, Functional work, Failure tests

### `docs/master-plan/23-deep-recovery.md`

- Inventory `F0339`; 4829 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 112 lines; SHA-256 `3b37b97dfb31042ad5b82922f173eee24ef3e1536ca4adfd365b1942520046c6`.
- Scope summary: Recovery that survives the loss of every operating environment, prioritises user data above the operating system, and keeps LitricOS the platform authority even while users work inside Windows or Linux environments. [ADR-0021](../adr/0021-deep-recovery-data-pr Topics: 23 — Deep recovery and data protection, Requirement, The four concepts (never conflated), Storage model (logical), Recovery environment, Recovery priority (fixed order), Recovery decision engine, Health, watchdo

### `docs/master-plan/24-feature-parity.md`

- Inventory `F0340`; 21569 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 306 lines; SHA-256 `e0acc1308296cee59879d2e8b65d638a8108550394157a33a0c57a15abd30b8f`.
- Scope summary: The claim "a real desktop OS" is only meaningful against the feature sets users already have. This catalogue compares LitricOS with Windows 11, macOS, and the major Linux desktops (GNOME/KDE) across every domain a daily-driver OS must cover, and maps each feat Topics: 24 — Feature parity: Windows, macOS, Linux, and LitricOS, Purpose, Scoreboard, 1. Desktop & window management, 2. Files & archives, 3. Search & indexing, 4. Settings & personalisation, 5. Security & privacy, 6. 

### `docs/master-plan/25-beta-1.0-gate-matrix.md`

- Inventory `F0341`; 13453 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 162 lines; SHA-256 `9db7067b018a24d9fa404a05dcecc8a929c3d30ea65565b2cb792f4a97785176`.
- Scope summary: **Created:** 2026-10-11, from the Stage A audit ([docs/audit/2026-10-11-beta-1.0-stage-a-audit.md](../audit/2026-10-11-beta-1.0-stage-a-audit.md)) and the Stage B research ([docs/research/platform-parity.md](../research/platform-parity.md)). **[`docs/SYSTEM-SC Topics: 25 — Beta 1.0 gate matrix (whole-system view), The single-source-of-truth rule, Priority definitions used below, 1. Coverage by area, 2. Evidence level required before a row may be called verified, 3. Wave plan,

### `docs/master-plan/README.md`

- Inventory `F0342`; 3114 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 55 lines; SHA-256 `79825fe7ed6cec8d6899a97d1d757aa787f2eed06c9bab0366cfc26d95b4ff7d`.
- Scope summary: This directory turns the high-level roadmap into an implementable, evidence-led programme. It complements rather than replaces `docs/SYSTEM-SCOPE.md`: that file remains the complete feature ledger and this directory supplies ordering, interfaces, invariants, o Topics: LitricOS Master Plan, Reading order, Rules, Release labels

## `research/` (1 entries)

### `docs/research/platform-parity.md`

- Inventory `F0343`; 29297 bytes; Markdown; role `architecture-or-phase-doc`.
- Content fingerprint: 178 lines; SHA-256 `6e74f9c6eb788df6270cb3300836cb3414583f6504b7934e71cabffc6bb14892`.
- Scope summary: **Created:** 2026-10-11 (Stage B of the Beta 1.0 E2E pass) **Method:** primary/official sources only. Every row below says whether it was **verified in a fetched page** or **not verified**. Where a source could not be read, the row says so instead of asserting Topics: Platform parity — Windows / macOS / Linux backends, and what LitricOS should do, 1. Verification boundary (read this before quoting anything below), 2. Capability status vocabulary (one model, used everywhere), 

