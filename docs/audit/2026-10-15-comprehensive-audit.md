# LitricOS Comprehensive Static Audit — 2026-10-15

**Target:** `main` at `ec19fb40865bdb693c5b6c4365a2edcedb760cf7`  
**Audit branch:** `audit/comprehensive-2026-10-15`  
**Mode:** repository inventory + architecture/document review + targeted source review; proposed fixes are isolated on the audit branch.

## Executive summary

This repository is a substantial OS prototype, not just a desktop mock-up: it contains a Rust workspace, image/ISO build tooling, systemd units, a software-framebuffer shell, a filesystem service, a browser engine, a capability policy core, a defender policy core, and a large evidence/documentation system. The repository also has several strong engineering controls already: Rust workspace lints forbid `unsafe`, panic/unwrap/indexing in system code are denied by Clippy, archive code has explicit bounds, and `ci/verify.sh` includes negative controls for important static checkers.

This pass made three focused corrections on the audit branch:

1. Corrected the Trash 1.0 `Path=` field format in **both** writers, normalized new absolute paths into the internal logical-path namespace, retained support for historical `file://` entries, and added regression coverage to both service and shell tests.
2. Added a GitHub Actions workflow to run the existing verification gate for pushes, pull requests, and manual dispatch. External actions are pinned to full commit SHAs; workflow permissions are read-only.
3. Changed the local development-key check so a clean checkout skips intentionally absent, git-ignored keys, but still fails if any key material is present and the key-pair validation is incomplete or incorrect.

The edits are on a separate branch and have **not** been merged to `main`. CI execution is required before calling the Rust changes verified.

## Scope and evidence discipline

The repository tree contained 840 entries at the audited base revision. The audit inventoried the workspace, build/image pipeline, tools, tests, documentation, prior bug reports, security model and release gates. Deep source reading focused on the file-operation and trash paths, the filesystem protocol, capability policy, defender policy/cache, workspace manifest and verification gate.

This is a source-backed engineering review, not a claim that every line of every file has been manually proven correct. I did not treat the existence of tests, build artifacts or design documents as runtime proof. No local runtime or QEMU environment was available through the GitHub connector; current automated-run results must be recorded separately from historical evidence.

## Findings

### AUD-001 — Trash metadata was not interoperable with standard Linux trash readers

**Severity:** High (interoperability; no direct data-loss claim)  
**Original files:** `src/litric-files/src/trash.rs`, `src/litric-files/src/server.rs`, `src/litric-shell/src/fsbridge.rs`

Both trash writers encoded `file:///Documents/…` into `Path=`. Trash 1.0 defines this field as an absolute pathname or an allowed relative pathname, not a file URI. Litric's own readers stripped the URI scheme, so its round trip worked while external file managers could interpret the location incorrectly.

**Change on this branch:** new entries store a percent-encoded absolute host path; the daemon and shell map it back into their root-relative logical namespace. Paths outside the configured root are rejected instead of being treated as restorable locations. Legacy `file://` entries remain readable. Regression tests assert the new disk format and legacy restore behavior in both code paths.

**Verification:** tests added; pending execution by the new CI workflow. External interoperability with `gio trash --list` still requires a booted image and a live check.

### AUD-002 — No GitHub Actions workflow was present to enforce repository gates

**Severity:** High (regression risk)  
**Original location:** `.github/workflows/` had no workflow files; `ci/verify.sh` already existed.

A developer could change code without GitHub automatically running the repository's Python/shell/static checks and Rust tests. This is particularly risky for the filesystem and security code, where historical reports document bugs that static syntax checks alone could not find.

**Change on this branch:** added `.github/workflows/verify.yml`. It invokes `bash ci/verify.sh` on push, pull request and manual dispatch, requests only `contents: read`, disables persisted checkout credentials, has a timeout and uses full-SHA-pinned actions. The existing gate remains the single source of repository-check behavior rather than duplicating a second script in YAML.

**Follow-up:** confirm the new workflow's run is green; preserve the distinction between checks executed in CI and environment-dependent image/boot steps that the gate explicitly skips.

### AUD-003 — A clean checkout could fail the verification gate due to deliberately ignored local keys

**Severity:** Medium (developer experience / repeatability)  
**File:** `ci/verify.sh`

The gate validated development signing key pairs whenever OpenSSL was installed. The keys are intentionally git-ignored, so a clean clone normally has none. That made a machine-local build prerequisite look like a repository defect.

**Change on this branch:** when all development key files are absent, the gate records a skip with a reason. If any key file exists, it still validates that both certificate and private key exist, parse, and match. This avoids committing development secrets and does not soften validation of a present or partially generated key pair.

### AUD-004 — Filesystem confinement still has a check-then-use race

**Severity:** High-priority security design risk; exploitability depends on process permissions and attacker control of directories  
**Files:** `src/litric-files/src/ops.rs`, `src/litric-shell/src/fsbridge.rs`, ADR-0019.

The daemon checks components with `symlink_metadata` while walking a path, but subsequent filesystem operations reopen paths by name. A directory component can theoretically change between validation and use. The source documentation already identifies this TOCTOU class as an accepted residual risk. The shell bridge also contains direct host-path operations and its trash setup uses `create_dir_all`, unlike the daemon's explicit symlink-aware directory-chain creation. Static inspection does not establish that a practical attack is possible in the shipped privilege model, so this is recorded as a design risk, not as a proven exploit.

**Recommendation:** create an ADR and threat-model decision before changing the implementation. Prefer descriptor-relative operations that bind lookup and use together (for example, a carefully audited `openat2`/directory-FD layer with beneath/no-symlink constraints). Keep any unavoidable platform-specific unsafe code isolated in a tiny reviewed boundary; do not sprinkle ad-hoc unsafe calls through the current safe-Rust core. Add race-oriented adversarial tests and tests for symlinked `.local/share/Trash` components.

### AUD-005 — Runtime readiness and performance targets need fresh end-to-end evidence

**Severity:** Release blocker until gates are current and pass  
**Evidence:** `docs/EVIDENCE.md`, `docs/GATES.md`, `docs/audit/2026-10-11-beta-1.0-stage-a-audit.md`.

The recorded gate history separates successful Rust tests from image construction and boot evidence, correctly marking environment-dependent checks as not run when they were not executed. Historical boot records also include failed systemd units, image/UKI sizes above their budgets, and boot time above the target; those values are historical observations, not valid current performance claims. A green Rust gate alone cannot prove the OS image boots or that the resource targets are met.

**Recommendation:** treat a reproducible image build, unattended boot with zero unexpected failed units, and runtime memory/CPU/boot measurements as explicit release gates. Capture host/toolchain/configuration, sample size, raw logs and artifact hashes. Do not mark target features or performance values done from configuration intent alone.

### AUD-006 — Large modules concentrate unrelated behavior and raise change risk

**Severity:** Medium (maintainability; not by itself a functional bug)  
**Areas:** `src/litric-shell/src/shell.rs` (~193 KB), `views.rs` (~132 KB), `taskmgr.rs` (~121 KB), plus large `canvas.rs`, `sysd/providers.rs` and `lock.rs`.

The crate boundaries are useful, but several individual modules are large enough that unrelated UI state, rendering, service orchestration and filesystem behaviors can become tightly coupled. Large files also make review and regression localization harder.

**Recommendation:** refactor incrementally without changing behavior first. Extract by cohesive ownership (for example, shell window/application lifecycle, desktop menus, task-manager model vs. view, and rendering primitives). Require baseline unit/E2E evidence before each extraction and keep public APIs stable. Avoid a wholesale rewrite or splitting files solely by line count.

### AUD-007 — Trash behavior is implemented independently in two layers

**Severity:** Medium (consistency risk)  
**Files:** daemon trash implementation and the shell's `fsbridge.rs`.

The duplicate disk-format code caused a bug to require coordinated edits to both writers/readers. This branch updates both and adds coverage at both layers, but duplicate implementations remain a future drift hazard (including filename collision conventions already called out in the continuation audit).

**Recommendation:** after current behavior is verified, consider moving the pure `.trashinfo` encoder/decoder and compatibility rules into one shared, dependency-light crate or contract module. Keep host-root confinement in the owning layer. Share conformance fixtures rather than forcing platform I/O into the pure parser.

### AUD-008 — Partial multi-item operations need visible UI reconciliation

**Severity:** Medium (user-visible state can lag disk state)  
**Evidence:** `docs/audit/2026-10-09-continuation-recovery-audit.md` §7 and §12.

The protocol was improved to report partial completion, but the recorded remaining task is for the shell to refresh its view after an error that says some items already completed. Until the UI reconciles the filesystem after partial `COPY`, `MOVE`, trash/restore or delete operations, the user can see stale contents after an error.

**Recommendation:** implement a shared “operation result + affected directories” model, refresh those directories on partial completion, and test failures in the middle of multi-item operations. Do not assume a failed operation was atomic unless the API explicitly guarantees it.

### AUD-009 — Security and dependency checkers are guardrails, not proofs

**Severity:** Medium (process/design)  
**Files:** `ci/lint-file-safety.py`, `ci/lint-deps.py`, `ci/check-memory-budget.py`.

The repository has unusually good negative controls for several static checkers, but the file-deletion checker itself says it cannot prove an allowed deletion site is correct. Similarly, a static memory-policy ledger cannot show actual runtime RSS or prove a unit is assigned to the intended cgroup at run time.

**Recommendation:** keep negative-control tests, add behavior tests around every reviewed destructive function, and connect policy ledgers to runtime evidence. For the memory budget, close all acknowledged policy decisions and add a boot-time cgroup measurement gate instead of relying on configuration parsing alone.

## Architecture assessment

**Good foundations to preserve**
- Separation between policy/domain crates and system-facing crates.
- `unsafe_code = "forbid"` at workspace level and strict Clippy lints for panic-prone constructs.
- Bounded archive parsing/extraction and symlink-aware checks in the daemon filesystem layer.
- Capability scopes, explicit contracts/ADRs, risk registers, evidence ledgers, and checkers designed to fail on planted defects.
- A single existing verification gate with a record of skipped environment-dependent checks.

**Corrections to keep explicit in product and design documentation**
- The desktop currently uses a software framebuffer shell on `/dev/fb0`; it is not a Smithay/Wayland compositor architecture.
- The correction in `docs/ARCH-00-boundaries.md` distinguishes target architecture from implemented inventory. Preserve that discipline in all comparison tables, feature lists and release messaging.
- The two currently built daemons identified in the inventory are `litric-sysd` and `litric-envd`; planned services should remain marked planned until binaries, service wiring, tests and runtime evidence exist.

**Architecture direction:** stabilize the kernel/image boot path and trust boundaries before broad feature expansion. Prefer upstream components where intended by the project, cohesive crate contracts, measurable gates and small reversible changes over a large cross-subsystem rewrite.

## Priority plan

1. **Now:** run and pass the Rust gate on this branch; inspect formatting and both trash regression tests.
2. **Before merge:** verify CI's clean-checkout behavior; confirm absent development keys are a reported skip, not a failure; review the workflow run.
3. **Next security milestone:** ADR and threat-model review for descriptor-relative filesystem confinement, with adversarial tests.
4. **Next release milestone:** rebuild the image and produce current boot, failed-unit, runtime budget, and performance evidence.
5. **Then:** resolve partial-operation UI refresh and acknowledged memory-budget gaps.
6. **After baselines are green:** split the oversized shell modules and consolidate duplicate trash format logic under behavior-preserving tests.

## Branch changes made in this audit

| File | Change |
|---|---|
| `.github/workflows/verify.yml` | New read-only, SHA-pinned GitHub Actions workflow calling `ci/verify.sh` |
| `ci/verify.sh` | Clean checkout skips absent ignored development keys; any present key material remains strictly checked |
| `src/litric-files/src/trash.rs` | Write compliant absolute-path `Path=`; normalize to validated logical path; legacy URI reads retained |
| `src/litric-files/src/server.rs` | Restore from normalized logical path; regression test for new format and legacy URI |
| `src/litric-shell/src/fsbridge.rs` | Update trash writer/reader; root-constrained path normalization; regression test for new format and legacy URI |
| `docs/bugs/BUG-0009-trash-path-field-is-not-a-location.md` | Update defect status and verification requirements |
| `docs/audit/2026-10-15-comprehensive-audit.md` | This report |

## Verification status

- **Static review:** completed for the scoped architecture and critical filesystem/trash paths described above.
- **Rust regression tests:** added; must be confirmed by CI before merge.
- **Image build, QEMU boot, third-party trash interoperability, physical hardware and runtime performance:** not established by this source-level pass and must not be reported as passing without execution evidence.
