# Batch 01 — Root Configuration, Hidden Metadata, Master Workflow & Phase Specifications

Total files documented in this file: **24**

---

## F0001 — `.agents/skills/litric-v-2/SKILL.md`

### A. Identity
- **Path**: `.agents/skills/litric-v-2/SKILL.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 44,591 bytes
- **Classification**: `configuration` | Role: `repo-metadata`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project repo-metadata file (Markdown).
- **Subsystem**: `repo-metadata`

### C. Internal Structure
- **Line Count**: 2107 lines
- **Major Sections / Headings**:
  - L6: `# LITRIC — EXECUTABLE ENGINEERING PROTOCOL`
  - L7: `## Principal AI Engineering Skill / Autonomous Development Constitution`
  - L8: `### Version 2.0 — Deep Verification, Decision Gates, Risk Control, Red-Team & Release Authorization`
  - L18: `# 0. EXECUTIVE DIRECTIVE`
  - L70: `# 1. SUPREME LAWS`
  - L74: `## LAW 1 — NO EVIDENCE, NO CLAIM`
  - L98: `## LAW 2 — UNDERSTAND BEFORE MODIFYING`
  - L102: `## LAW 3 — ROOT CAUSE BEFORE PATCH`
  - L110: `## LAW 4 — ARCHITECTURE BEFORE IMPLEMENTATION`
  - L114: `## LAW 5 — TEST AFTER EVERY SIGNIFICANT CHANGE`
  - L118: `## LAW 6 — VERIFY THE VERIFICATION`
  - L122: `## LAW 7 — CRITICAL UNKNOWN RISK BLOCKS PROGRESSION`
  - L126: `## LAW 8 — NEVER HIDE FAILURE`
  - L130: `## LAW 9 — LITRIC CORE MUST REMAIN SELF-OS`
  - L135: `## LAW 10 — FIX THE SYSTEM, NOT ONLY THE SYMPTOM`
  - L141: `# 2. LITRIC ARCHITECTURAL NORTH STAR`
  - L199: `# 3. EXECUTION MODES`
  - L203: `## MODE A — DISCOVERY`
  - L225: `## MODE B — ARCHITECTURE`
  - L241: `## MODE C — IMPLEMENTATION`
  - L253: `## MODE D — VERIFICATION`
  - L269: `## MODE E — RED TEAM`
  - L285: `## MODE F — RELEASE REVIEW`
  - L298: `# 4. UNIVERSAL ENGINEERING LOOP`
  - L333: `# 5. DECISION GATES`
  - L337: `## GATE G0 — INTAKE`
  - L353: `## GATE G1 — SYSTEM UNDERSTANDING`
  - L367: `## GATE G2 — DESIGN`
  - L383: `## GATE G3 — IMPLEMENTATION`
  - L396: `## GATE G4 — FUNCTIONAL VERIFICATION`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `PROFILE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0042 — `.claude/launch.json`

### A. Identity
- **Path**: `.claude/launch.json`
- **Type / Language**: `.json` (JSON)
- **Size**: 221 bytes
- **Classification**: `configuration` | Role: `repo-metadata`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project repo-metadata file (JSON).
- **Subsystem**: `repo-metadata`

### C. Internal Structure
- **Line Count**: 11 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0043 — `.git`

### A. Identity
- **Path**: `.git`
- **Type / Language**: `no_ext` (Plaintext/Config)
- **Size**: 63 bytes
- **Classification**: `configuration` | Role: `repo-metadata`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project repo-metadata file (Plaintext/Config).
- **Subsystem**: `repo-metadata`

### C. Internal Structure
- **Line Count**: 1 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0044 — `.gitattributes`

### A. Identity
- **Path**: `.gitattributes`
- **Type / Language**: `no_ext` (Plaintext/Config)
- **Size**: 811 bytes
- **Classification**: `configuration` | Role: `repo-metadata`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Line endings are a contract in this tree, not a preference. `ci/lint-line-endings.py` fails the gate on CRLF in any file a Linux shell, kernel or systemd will read (a shell script with CRLF fails at the shebang with an error that names the wrong file). The repository is edited on Windows, where the host's Git config had `core.autocrlf=true`: without this file, a checkout rewrites LF to CRLF and the defect appears without anyone editing the file.
- **Subsystem**: `repo-metadata`

### C. Internal Structure
- **Line Count**: 27 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `ci/lint-line-endings.py`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0045 — `.gitignore`

### A. Identity
- **Path**: `.gitignore`
- **Type / Language**: `no_ext` (Plaintext/Config)
- **Size**: 617 bytes
- **Classification**: `configuration` | Role: `repo-metadata`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Build output. Regenerated by `cargo` and by the gate; never reviewed as source. Generated tool and gate output: logs, dumped frames, page dumps, screenshots. Agent-tool host state. `project-id` names this checkout on this machine only. VM and image artifacts. Large, regenerable, and never the thing under review. Development signing keys are excluded by keys/.gitignore, which is the file that states why: a committed signing key — even a development one — is a key that
- **Subsystem**: `repo-metadata`

### C. Internal Structure
- **Line Count**: 21 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `keys/.gitignore`, `src/target/`, `tools/logs/`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0046 — `00-MASTER-WORKFLOW.md`

### A. Identity
- **Path**: `00-MASTER-WORKFLOW.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 7,926 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 180 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC OS — MASTER WORKFLOW PROMPT (PHASE CONTROLLER)`
  - L7: `## 0. HOW TO USE THIS PACK`
  - L23: `## 1. ROLE`
  - L33: `## 2. GLOBAL INVARIANTS (NEVER VIOLATE)`
  - L50: `## 3. LOCKED TECHNICAL BASELINE (decided — do not re-litigate without evidence)`
  - L77: `## 4. STATE BLOCK (the user maintains and pastes this every session)`
  - L96: `## 5. MANDATORY RESPONSE FORMAT (every major task)`
  - L116: `## 6. GATE SYSTEM`
  - L133: `## 7. PHASE MAP`
  - L155: `## 8. HARD "DO NOT" LIST`
  - L167: `## 9. SESSION CLOSING RITUAL`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/GATES.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0047 — `01-PHASE-00-ARCHITECTURE.md`

### A. Identity
- **Path**: `01-PHASE-00-ARCHITECTURE.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 4,670 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 100 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 00 PROMPT: ARCHITECTURE & THREAT MODEL`
  - L8: `## OBJECTIVE`
  - L11: `## WHY`
  - L16: `## TASKS`
  - L75: `## ASSUMPTIONS TO STATE`
  - L82: `## EXIT GATE — GATE 00`
  - L95: `## FAILURE MODES`
  - L99: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/API-00-contracts.md`, `docs/ARCH-00-boundaries.md`, `docs/REPO-00-layout.md`, `docs/RISK-00.md`, `docs/SEC-00-threatmodel.md`, `docs/SEC-01-capabilities.md`, `docs/adr/0001-kernel-foundation.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0048 — `02-PHASE-01-BOOT.md`

### A. Identity
- **Path**: `02-PHASE-01-BOOT.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 4,483 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 99 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 01 PROMPT: BOOTABLE PROTOTYPE (QEMU-FIRST)`
  - L8: `## OBJECTIVE`
  - L11: `## WHY`
  - L16: `## TASKS`
  - L84: `## EXIT GATE — GATE 01`
  - L93: `## FAILURE MODES & ROLLBACK`
  - L98: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/DEV-01-debug.md`, `image/mkosi.conf`, `kernel/litric-base.config`, `tests/smoke_boot.py`, `tools/logs/boot-`, `tools/run-qemu.ps1`, `tools/run-qemu.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0049 — `03-PHASE-02-CORE.md`

### A. Identity
- **Path**: `03-PHASE-02-CORE.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,894 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 94 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 02 PROMPT: CORE SYSTEM (INIT, MEMORY, PROCESSES, IPC)`
  - L8: `## OBJECTIVE`
  - L13: `## TASKS`
  - L81: `## EXIT GATE — GATE 02`
  - L89: `## FAILURE MODES`
  - L93: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/sys/kernel/debug/lru_gen`, `/sys/kernel/mm/lru_gen/enabled`, `docs/CORE-02-cgroups.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0050 — `04-PHASE-03-STORAGE.md`

### A. Identity
- **Path**: `04-PHASE-03-STORAGE.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,531 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 71 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 03 PROMPT: STORAGE, FILESYSTEM, ENCRYPTION, RECOVERY`
  - L8: `## OBJECTIVE`
  - L13: `## TASKS`
  - L57: `## EXIT GATE — GATE 03`
  - L65: `## FAILURE MODES & ROLLBACK`
  - L70: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/STOR-03-layout.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0051 — `05-PHASE-04-NETWORK.md`

### A. Identity
- **Path**: `05-PHASE-04-NETWORK.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,147 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 60 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 04 PROMPT: NETWORKING (PERMISSION-AWARE)`
  - L8: `## OBJECTIVE`
  - L13: `## TASKS`
  - L47: `## EXIT GATE — GATE 04`
  - L55: `## FAILURE MODES`
  - L59: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/NET-04-tuning.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0052 — `06-PHASE-05-GRAPHICS.md`

### A. Identity
- **Path**: `06-PHASE-05-GRAPHICS.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,787 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 70 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 05 PROMPT: GRAPHICS & COMPOSITOR`
  - L10: `## OBJECTIVE`
  - L15: `## TASKS`
  - L55: `## EXIT GATE — GATE 05`
  - L64: `## FAILURE MODES & ROLLBACK`
  - L69: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `tools/bench/latency/`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0053 — `07-PHASE-06-DESKTOP.md`

### A. Identity
- **Path**: `07-PHASE-06-DESKTOP.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,626 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 69 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 06 PROMPT: LITRIC DESKTOP SHELL`
  - L8: `## OBJECTIVE`
  - L11: `## WHY`
  - L16: `## TASKS`
  - L55: `## EXIT GATE — GATE 06`
  - L64: `## FAILURE MODES`
  - L68: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/UX-06-model.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0054 — `08-PHASE-07-SECURITY-DEFENDER.md`

### A. Identity
- **Path**: `08-PHASE-07-SECURITY-DEFENDER.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 5,458 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 85 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 07 PROMPT: SECURITY, SANDBOX & LITRIC DEFENDER`
  - L10: `## OBJECTIVE`
  - L15: `## PART A — SANDBOX & CAPABILITIES`
  - L42: `## PART B — LITRIC DEFENDER`
  - L68: `## EXIT GATE — GATE 07`
  - L79: `## FAILURE MODES & ROLLBACK`
  - L84: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0055 — `09-PHASE-08-UPDATE.md`

### A. Identity
- **Path**: `09-PHASE-08-UPDATE.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,771 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 64 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 08 PROMPT: ATOMIC A/B UPDATE & AUTO-ROLLBACK`
  - L8: `## OBJECTIVE`
  - L13: `## TASKS`
  - L49: `## EXIT GATE — GATE 08`
  - L58: `## FAILURE MODES & ROLLBACK`
  - L63: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0056 — `10-PHASE-09-AI.md`

### A. Identity
- **Path**: `10-PHASE-09-AI.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 4,468 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 72 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 09 PROMPT: AI SUBSYSTEM (SYSTEM SERVICE, NOT A CHATBOT)`
  - L8: `## OBJECTIVE`
  - L13: `## TASKS`
  - L57: `## EXIT GATE — GATE 09`
  - L66: `## FAILURE MODES & ROLLBACK`
  - L71: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0057 — `11-PHASE-10-APPS.md`

### A. Identity
- **Path**: `11-PHASE-10-APPS.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,559 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 64 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 10 PROMPT: APPLICATION PLATFORM, PACKAGES & SDK`
  - L8: `## OBJECTIVE`
  - L13: `## TASKS`
  - L50: `## EXIT GATE — GATE 10`
  - L59: `## FAILURE MODES`
  - L63: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0058 — `12-PHASE-11-HARDWARE.md`

### A. Identity
- **Path**: `12-PHASE-11-HARDWARE.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,696 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 61 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 11 PROMPT: REAL HARDWARE BRING-UP & POWER`
  - L8: `## OBJECTIVE`
  - L15: `## TASKS`
  - L46: `## EXIT GATE — GATE 11`
  - L55: `## FAILURE MODES & ROLLBACK`
  - L60: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/HCL.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0059 — `13-PHASE-12-GAMING-MODE.md`

### A. Identity
- **Path**: `13-PHASE-12-GAMING-MODE.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 6,885 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 86 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 12 PROMPT: GAMING MODE & DEEP SETTINGS`
  - L10: `## OBJECTIVE`
  - L15: `## TASKS`
  - L70: `## EXIT GATE — GATE 12`
  - L80: `## FAILURE MODES & ROLLBACK`
  - L85: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/GAMING-anticheat.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0060 — `14-PHASE-13-PERF-MEMORY.md`

### A. Identity
- **Path**: `14-PHASE-13-PERF-MEMORY.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 4,709 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 77 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — PHASE 13 PROMPT: PERFORMANCE, MEMORY BUDGET & SMOOTHNESS`
  - L10: `## OBJECTIVE`
  - L15: `## TASKS`
  - L61: `## EXIT GATE — GATE 13 (re-runnable)`
  - L71: `## FAILURE MODES`
  - L76: `## NEXT STEP`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/PERF-budget.md`, `docs/PERF-claims.md`, `tools/bench/latency/`, `tools/bench/mem/`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0061 — `15-APPENDIX-TARGETS-MEASUREMENT.md`

### A. Identity
- **Path**: `15-APPENDIX-TARGETS-MEASUREMENT.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 6,381 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 133 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC — APPENDIX: TARGETS, BASELINES & MEASUREMENT PROTOCOL`
  - L7: `## 1. TARGETS LITRIC IS HELD TO`
  - L26: `## 2. BASELINES TO BEAT (measured elsewhere — re-verify locally before quoting)`
  - L45: `## 3. MEASUREMENT PROTOCOL (mandatory for every number)`
  - L67: `## 4. STANDARD TOOL SET`
  - L85: `## 5. HONEST LIMITATIONS (must appear in LITRIC's public docs)`
  - L99: `## 6. LICENSING NOTES`
  - L109: `## 7. PRIOR ART — WHAT TO COPY, WHAT TO AVOID`
  - L120: `## 8. EFFORT ESTIMATE (small team, integration-first)`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0062 — `LITRIC_NATIVE_ECOSYSTEM_MASTER_ARCHITECTURE_v1.md`

### A. Identity
- **Path**: `LITRIC_NATIVE_ECOSYSTEM_MASTER_ARCHITECTURE_v1.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 18,867 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 739 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC NATIVE ECOSYSTEM — MASTER ARCHITECTURE SPECIFICATION v1.0`
  - L9: `## 0. EXECUTIVE PRINCIPLE`
  - L49: `# 1. DESIGN GOALS`
  - L51: `## 1.1 Primary goals`
  - L66: `## 1.2 Non-goals`
  - L76: `# 2. ARCHITECTURAL LAYERS`
  - L78: `## 2.1 Layer 0 — Firmware and Boot`
  - L90: `## 2.2 Layer 1 — Litric Kernel`
  - L111: `## 2.3 Layer 2 — Hardware Abstraction and Driver Framework`
  - L129: `## 2.4 Layer 3 — Litric Core Services`
  - L156: `# 3. NATIVE SYSTEM CONTRACT`
  - L167: `## 3.1 Contract hierarchy`
  - L185: `# 4. NATIVE APPLICATION MODEL`
  - L187: `## 4.1 Application identity`
  - L209: `## 4.2 Application boundaries`
  - L222: `## 4.3 Lifecycle`
  - L236: `# 5. NATIVE EXECUTABLE AND PACKAGE MODEL`
  - L248: `## 5.1 Executable format requirements`
  - L266: `## 5.2 Package requirements`
  - L282: `# 6. CAPABILITY-BASED SECURITY`
  - L313: `## 6.1 Object-level authorization`
  - L329: `# 7. MEMORY AND PROCESS ISOLATION`
  - L345: `# 8. IPC ARCHITECTURE`
  - L370: `# 9. GRAPHICS`
  - L401: `# 10. AUDIO`
  - L431: `# 11. STORAGE`
  - L451: `# 12. NETWORKING`
  - L465: `# 13. DEVELOPER ECOSYSTEM`
  - L485: `# 14. ABI GOVERNANCE`
  - L512: `# 15. PACKAGE AND UPDATE SECURITY`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0063 — `LITRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md`

### A. Identity
- **Path**: `LITRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 25,184 bytes
- **Classification**: `documentation` | Role: `architecture-or-phase-doc`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project architecture-or-phase-doc file (Markdown).
- **Subsystem**: `architecture-or-phase-doc`

### C. Internal Structure
- **Line Count**: 1193 lines
- **Major Sections / Headings**:
  - L1: `# LITRIC NATIVE RUNTIME, ABI, PROCESS, IPC & SECURITY — DEEP SPECIFICATION v1.0`
  - L11: `# 0. CORE THESIS`
  - L52: `# 1. NATIVE EXECUTION CONTRACT`
  - L54: `## 1.1 Process creation`
  - L76: `## 1.2 Entry-point semantics`
  - L92: `# 2. PROCESS MODEL`
  - L121: `# 3. THREAD MODEL`
  - L138: `# 4. MEMORY MODEL`
  - L140: `## 4.1 Virtual memory`
  - L151: `## 4.2 Shared memory`
  - L162: `## 4.3 Memory allocation`
  - L183: `# 5. ABI DESIGN`
  - L202: `## 5.1 ABI stability layers`
  - L212: `## 5.2 ABI handles`
  - L232: `# 6. ERROR MODEL`
  - L252: `# 7. HANDLE MODEL`
  - L276: `# 8. IPC ABI`
  - L278: `## 8.1 Goals`
  - L288: `## 8.2 Message structure`
  - L307: `## 8.3 RPC version negotiation`
  - L319: `# 9. CAPABILITY ENGINE`
  - L344: `# 10. SECURITY BROKER`
  - L368: `# 11. DYNAMIC LINKING`
  - L389: `# 12. RUNTIME SERVICES`
  - L409: `# 13. ASYNCHRONOUS I/O`
  - L425: `# 14. UI AND WINDOWING ABI`
  - L441: `## 14.1 Composition model`
  - L457: `# 15. GRAPHICS RESOURCE MODEL`
  - L475: `# 16. AUDIO RUNTIME`
  - L501: `# 17. STORAGE ABI`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/nvme0n1`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0064 — `LitricOS.cmd`

### A. Identity
- **Path**: `LitricOS.cmd`
- **Type / Language**: `.cmd` (Windows Batch)
- **Size**: 716 bytes
- **Classification**: `project source` | Role: `launcher-script`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS as an app: double-click to open the desktop in its own window. The window is a WSLg surface, so the console this starts has to stay open -- closing it is how the machine is powered off. Ctrl+C in this window sends SIGINT to QEMU, which is an abrupt power cut; use the desktop's own shutdown from inside the guest when it matters.
- **Subsystem**: `launcher-script`

### C. Internal Structure
- **Line Count**: 23 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `tools/desktop-app.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---
