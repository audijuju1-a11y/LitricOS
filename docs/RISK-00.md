# LITRIC — Risk Register
**Document:** RISK-00.md | **Phase:** 00 | **Status:** DESIGN-ONLY

---

## 1. Critical Risks (must be accepted/document BEFORE Phase 01)

### RISK-001: Kernel Anti-Cheat Games
- **Severity:** S2 (HIGH)
- **Impact:** Some major games will never run on Litric
- **Details:** EAC and BattlEye require per-developer opt-in for Proton/Wine. Fortnite, GTA Online, Apex Legends have declined or withdrawn support. Litric cannot change this.
- **Mitigation:** Document honestly in public docs. Game manager shows compatibility status per title. Never imply universal game support.
- **Status:** ACCEPTED — documented

### RISK-002: Windows-Host QEMU Cannot Do Accelerated Graphics
- **Severity:** S2 (HIGH)
- **Impact:** Graphics work (compositor, gaming) cannot be tested on primary dev host
- **Details:** virgl/venus GL acceleration is generally disabled in Windows QEMU builds. virtiofs is Linux-host-only.
- **Mitigation:** Use Linux host or WSL2 for accelerated testing. Windows host for software-rendered functional tests only. Record which host each measurement came from.
- **Status:** ACCEPTED — documented

### RISK-003: WHPX SMP Boot Bug
- **Severity:** S3 (MEDIUM)
- **Impact:** SMP boot may hang on Windows host
- **Details:** QEMU #3178 — WHPX SMP boot bug. Also reported CPUID/AVX issues.
- **Mitigation:** Keep TCG fallback launcher. Provide `--tcg` flag. Document the issue.
- **Status:** ACCEPTED — documented

### RISK-004: AMD XDNA NPU Stack Immature
- **Severity:** S3 (MEDIUM)
- **Impact:** NPU acceleration on AMD is experimental only
- **Details:** AMD XDNA driver (`amdxdna`) is upstream but runtime stack immature. Only Intel NPU (`intel_vpu` + OpenVINO) is broadly usable.
- **Mitigation:** Mark NPU as experimental. Intel-only for production NPU claims. Fall back silently to CPU/GPU if NPU fails.
- **Status:** ACCEPTED — documented

### RISK-005: Solo Developer Effort
- **Severity:** S2 (HIGH)
- **Impact:** Timeline may extend to 36-72 months for solo developer
- **Details:** ~18-36 person-months for a small team (1-3 people) to reach daily-driver prototype. Solo developer timeline can double.
- **Mitigation:** Integration-first approach (use upstream kernel, Mesa, PipeWire, systemd). Don't rewrite what works. Ship incremental milestones.
- **Status:** ACCEPTED — documented

### RISK-006: Commercial AV Detection Rate Not Matched
- **Severity:** S3 (MEDIUM)
- **Impact:** Defender is advisory, not a replacement for commercial antivirus
- **Details:** fanotify + YARA + hash denylist will not match ClamAV resident or commercial AV detection rates. False positives are possible.
- **Mitigation:** Sandbox-first approach. Defender is defense-in-depth, not the primary security mechanism. Honest documentation. False-positive reporting path.
- **Status:** ACCEPTED — documented

---

## 2. High Risks

### RISK-007: Musl Rejection
- **Severity:** S2 (HIGH)
- **Impact:** glibc increases image size vs musl
- **Details:** musl was rejected on evidence: multi-thread malloc lock contention (~4x slower in Chainguard testing), small default thread stacks, DNS edge cases. These break gaming/Proton.
- **Mitigation:** Use glibc. Accept the size cost. Document the decision (ADR-001).
- **Status:** ACCEPTED — documented

### RISK-008: bcachefs / ZFS Limitations
- **Severity:** S3 (MEDIUM)
- **Impact:** Btrfs is the only viable option for user filesystem
- **Details:** bcachefs is not production-ready. ZFS is excellent but CDDL keeps it out-of-tree.
- **Mitigation:** Use Btrfs with nodatacow for game/DB dirs. Monitor bcachefs maturity.
- **Status:** ACCEPTED — documented

### RISK-009: Side-Channel Attacks
- **Severity:** S3 (MEDIUM)
- **Impact:** Spectre-class attacks across app boundary
- **Details:** Kernel mitigations (IBRS, retpoline) have performance cost. Full mitigation across all side channels is an ongoing research problem.
- **Mitigation:** Keep mitigations enabled by default. `mitigations=off` is opt-in with explicit warning.
- **Status:** ACCEPTED — documented

---

## 3. Medium Risks

### RISK-010: Smithay API Instability
- **Severity:** S3 (MEDIUM) → **NOT APPLICABLE as of 2026-10-11**
- **Impact:** Compositor may need rework as Smithay evolves
- **Details:** Smithay is actively developed; APIs may change between versions.
- **Mitigation:** Pin Smithay version. Test against upstream. Budget for API migration.
- **Status:** **NOT APPLICABLE** — [ADR-0017](../adr/0017-framebuffer-shell.md)
  replaced the compositor plan with a software framebuffer shell, so the project
  takes no compositor dependency and there is no Smithay version to pin or API to
  migrate. `ci/lint-deps.py` fails the gate if a member gains a non-path
  dependency, which is what would make this risk live again. The real constraint
  on graphics is RISK-002 (no accelerated graphics under a Windows host's QEMU),
  not an upstream API.
- **Corrected:** 2026-10-11, from `docs/audit/2026-10-11-beta-1.0-stage-a-audit.md` §3.1

### RISK-011: Flatpak Runtime Bloat
- **Severity:** S3 (MEDIUM)
- **Impact:** Disk usage may exceed expectations
- **Details:** Flatpak runtimes duplicate libraries. Dedup helps but doesn't eliminate overhead.
- **Mitigation:** Measure and publish real disk cost. Dedup is not an excuse to stop measuring.
- **Status:** MONITORING

### RISK-012: systemd Overhead
- **Severity:** S4 (LOW)
- **Impact:** ~30-50 MB RAM overhead
- **Details:** systemd is not the lightest init. Trade-off for features (sysupdate, oomd, sandboxing).
- **Mitigation:** Trim units aggressively. Monitor RAM budget. Revisit if overhead exceeds budget.
- **Status:** ACCEPTED — documented

---

## 4. Low Risks

### RISK-013: Secure Boot Enrollment Complexity
- **Severity:** S4 (LOW)
- **Impact:** Users may struggle with key enrollment
- **Details:** MokManager / KEK+db enrollment varies by firmware.
- **Mitigation:** Document clearly. Provide recovery key. Ship signed fallback shim.
- **Status:** DOCUMENTED

### RISK-014: GPU Driver Variability
- **Severity:** S4 (LOW)
- **Impact:** Some GPUs may not work well
- **Details:** Mesa covers most hardware, but edge cases exist.
- **Mitigation:** HCL (Hardware Compatibility List) with TESTED-WORKING/PARTIAL/BROKEN/UNTESTED.
- **Status:** MONITORING
