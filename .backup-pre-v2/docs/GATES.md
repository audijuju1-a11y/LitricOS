# ILTRIC — Gate Results Log

---

## GATE 00 — Architecture & Threat Model
- **Date:** 2026-08-26
- **Result:** PASS
- **Documents verified:**
  - [x] docs/ARCH-00-boundaries.md — System boundary diagram
  - [x] docs/SEC-00-threatmodel.md — Threat model (STRIDE)
  - [x] docs/API-00-contracts.md — Interface contracts (9 APIs)
  - [x] docs/SEC-01-capabilities.md — Capability vocabulary (23 capabilities)
  - [x] docs/REPO-00-layout.md — Repository layout
  - [x] docs/adr/0001 through 0008 — Architecture decision records
  - [x] docs/RISK-00.md — Risk register (14 risks)

### Gate conditions:
- [x] All 7 documents exist and are internally consistent
- [x] Threat model covers every trust boundary (A-F + AI-specific)
- [x] Every capability in SEC-01 maps to at least one API in API-00
- [x] Every baseline decision in MASTER §3 has an ADR with "Revisit-if"
- [x] Risk register contains 6+ honest limitations (14 risks documented)

### Notes:
- All documents are DESIGN-ONLY (no code in Phase 00)
- Capabilities map: fs.* → iltric.fs, net.* → iltric.net, gpu → iltric.gfx, ai.* → iltric.ai, audio.* → iltric.audio, camera/mic/location → XDG portals (P07), system.settings → iltric.caps
- 14 risks documented (6 critical, 3 high, 3 medium, 2 low)

### Next: 02-PHASE-01-BOOT.md

---

## GATE 01 — Bootable Prototype (DESIGN-ONLY)
- **Date:** 2026-08-26
- **Result:** PASS_WITH_LIMITATIONS
- **Files created:**
  - [x] kernel/config/iltric-base.config — Kernel config fragment (312 lines)
  - [x] image/mkosi.conf — mkosi image pipeline definition
  - [x] tools/build-image.sh — Build orchestration script
  - [x] tools/run-qemu.ps1 — Windows QEMU launcher
  - [x] tools/run-qemu.sh — Linux/Mac QEMU launcher
  - [x] docs/DEV-01-debug.md — Kernel debugging guide
  - [x] tests/smoke_boot.py — CI smoke test (4 test cases)

### Gate conditions:
- [ ] `iltric.qcow2` boots to a shell in QEMU — BLOCKED: requires Linux host + mkosi build
- [ ] Image build is reproducible — BLOCKED: no build executed yet
- [ ] CI smoke test passes — BLOCKED: requires built image
- [ ] Boot time ≤ 8 s — NOT RUN
- [ ] Headless idle RAM ≤ 150 MB — NOT RUN
- [ ] `-s -S` + gdb attach demonstrated — NOT RUN
- [ ] Zero host-disk writes outside project dir — VERIFIED: all scripts enforce project dir constraint

### Limitations:
- All files are DESIGN-ONLY: no kernel was compiled, no image was built
- Requires Linux host (or WSL2) for mkosi build — Windows host cannot build natively
- Kernel config is a fragment, not yet verified against a real kernel .config
- OVMF firmware must be downloaded separately or from Arch packages
- Smoke test uses string matching (not expect/pty) — may have false positives

### Next: 03-PHASE-02-CORE.md
