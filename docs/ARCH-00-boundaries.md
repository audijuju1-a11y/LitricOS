# LITRIC — System Boundary Diagram
**Document:** ARCH-00-boundaries.md | **Phase:** 00 | **Status:** DESIGN-ONLY

---

## 1. Full Stack

```
┌─────────────────────────────────────────────────────────┐
│ L7  Native Litric Applications                          │
│     Owner: 3rd-party / Litric team                      │
│     Language: Rust, C, Python (via SDK)                 │
│     Trust: 4 (untrusted)                                │
├─────────────────────────────────────────────────────────┤
│ L6  Litric SDK / Runtime                                │
│     Owner: Litric team                                  │
│     Language: Rust/C                                     │
│     Trust: 2 (trusted)                                  │
├─────────────────────────────────────────────────────────┤
│ L5  Litric Desktop Shell                                │
│     Owner: Litric team                                  │
│     Language: Rust (software framebuffer, ADR-0017)     │
│     Trust: 2 (trusted)                                  │
├─────────────────────────────────────────────────────────┤
│ L4  Litric Core Services                                │
│     Owner: Litric team                                  │
│     Language: Rust/C                                     │
│     Trust: 1 (highly trusted)                           │
│     Services: init, securityd, storaged, netd,          │
│     audiod, gfxd, packaged, updated, defender, AI       │
├─────────────────────────────────────────────────────────┤
│ L3  System Libraries / ABI Surface                      │
│     Owner: Litric team / upstream                       │
│     Language: C/Rust                                    │
│     Trust: 1 (highly trusted)                           │
│     glibc, libdrm, PipeWire client, D-Bus               │
├─────────────────────────────────────────────────────────┤
│ L2  HAL / Drivers                                       │
│     Owner: upstream Linux / Litric config               │
│     Language: C                                         │
│     Trust: 0 (kernel-privileged)                        │
│     DRM/KMS, virtio, NVMe, WiFi, audio codecs           │
├─────────────────────────────────────────────────────────┤
│ L1  Litric Kernel (Linux LTS, custom .config)           │
│     Owner: upstream Linux + Litric patches              │
│     Language: C                                         │
│     Trust: 0 (kernel-privileged)                        │
│     Features: MGLRU, sched_ext, ntsync, fanotify,       │
│     Landlock, BPF-LSM, dm-verity, PREEMPT              │
├─────────────────────────────────────────────────────────┤
│ L0  Firmware / Boot (UEFI + OVMF / real firmware)       │
│     Owner: hardware vendor / Litric UKI                 │
│     Trust: 0 (hardware-privileged)                      │
└─────────────────────────────────────────────────────────┘
```

## 2. Trust Levels

| Level | Label | Examples |
|-------|-------|----------|
| 0 | Kernel / Firmware | Linux kernel, UEFI, drivers |
| 1 | Core Services | init, securityd, defender, update |
| 2 | Trusted Platform | compositor, shell, SDK, IPC broker |
| 3 | Signed Native App | Litric-registered apps |
| 4 | Untrusted Content | downloaded files, Windows binaries, plugins |

## 3. Trust Boundaries (Critical Interfaces)

```
BOUNDARY A: Kernel ↔ Userspace
  Type: syscall boundary
  Enforcement: SELinux/Landlock/seccomp, capability checks
  Risk: kernel exploit, privilege escalation

BOUNDARY B: Core Service ↔ App
  Type: IPC (D-Bus / native socket)
  Enforcement: capability engine, message-level auth
  Risk: confused deputy, IPC spoofing

BOUNDARY C: App ↔ App
  Type: process isolation (ASLR, namespaces)
  Enforcement: cgroups, Landlock, seccomp
  Risk: shared memory abuse, side-channel

BOUNDARY D: Native ↔ Windows Compatibility
  Type: compatibility runtime boundary
  Enforcement: separate process tree, namespace
  Risk: Windows app escape, ambient authority

BOUNDARY E: System ↔ External (Network/USB/Physical)
  Type: input/output boundary
  Enforcement: nftables, fanotify, USB policy
  Risk: network attack, malicious device, physical access

BOUNDARY F: Signed Image ↔ Tampered Image
  Type: integrity boundary
  Enforcement: dm-verity, UKI signing, fs-verity
  Risk: boot chain compromise, rootfs tampering
```

## 4. Dependency Direction (MUST NOT invert)

```
Hardware
  → Firmware
    → Kernel
      → Core Services
        → System API
          → Runtime / SDK
            → Native Apps

Windows Compat branch (subordinate):
  Windows App
    → PE Loader
      → Compat Runtime
        → Litric System API
          → Litric Kernel
```

**INVARIANT:** Litric Core must NEVER depend on Windows compatibility.

## 5. Component Ownership

| Component | Owner | Language | Upstream/Own | Trust |
|-----------|-------|----------|--------------|-------|
| Kernel | upstream Linux | C | upstream + config | 0 |
| Init/service supervisor | systemd | C | upstream | 1 |
| Desktop shell (no separate compositor) | Litric | Rust | own, software framebuffer (ADR-0017) | 2 |
| Defender on-demand scanner (no resident on-access daemon) | Litric | Rust | own | 1 |
| AI service | Litric + llama.cpp | Rust/C | integrated | 1 |
| Package manager | Flatpak + Litric | C/Rust | upstream + own | 1 |
| Update system | systemd-sysupdate | C | upstream | 1 |
| Network stack | iwd + networkd | C | upstream | 1 |
| Audio stack | PipeWire | C | upstream | 1 |
| Graphics stack | Mesa | C | upstream | 1-2 |
| Security broker | Litric | Rust | own | 1 |
| Capability engine | Litric | Rust | own | 1 |

---

## Correction — 2026-10-11: the L4/L5 layers this document describes are targets, not an inventory

This document is a Phase 00 boundary design and it was read as a description of
the tree, which it is not. Three specific claims were false when checked against
the checkout (`docs/audit/2026-10-11-beta-1.0-stage-a-audit.md` §3.1):

1. **L5 is not a Smithay compositor stack.**
   [ADR-0017](adr/0017-framebuffer-shell.md) decided the desktop is drawn in
   software onto `/dev/fb0` by `litric-shell`, with no Wayland compositor and no
   compositor dependency; `litric-shell/Cargo.toml` depends only on sibling
   workspace crates. §5's "Compositor" row is corrected above. There is no
   `compositor/` directory in this repository and none is planned.
2. **L4's service list is a target list.** The tree has no `securityd`,
   `storaged`, `netd`, `audiod`, `gfxd`, `packaged` or `updated`. PID 1 is
   `systemd` (ADR-0002), and the only daemons this project builds and installs
   are `litric-sysd` (facts over a Unix socket, [ADR-0016](adr/0016-system-daemon.md))
   and `litric-envd`; everything else in the L4 line is a `PLANNED` row in
   [`docs/SYSTEM-SCOPE.md`](SYSTEM-SCOPE.md).
3. **The trust table in §2 names a `compositor` at level 2.** With no
   compositor process, level 2 is the shell and the daemons in §5, exactly as
   ADR-0017 and ADR-0016 define them.

The layering, the trust levels and the boundary list are unchanged design work
and remain the reference for those. What changed is that L4/L5 are now stated as
intent, with the parities and gaps tracked in `docs/SYSTEM-SCOPE.md` rather than
implied by this diagram.
