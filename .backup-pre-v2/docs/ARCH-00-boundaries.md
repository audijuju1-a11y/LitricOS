# ILTRIC — System Boundary Diagram
**Document:** ARCH-00-boundaries.md | **Phase:** 00 | **Status:** DESIGN-ONLY

---

## 1. Full Stack

```
┌─────────────────────────────────────────────────────────┐
│ L7  Native Iltric Applications                          │
│     Owner: 3rd-party / Iltric team                      │
│     Language: Rust, C, Python (via SDK)                 │
│     Trust: 4 (untrusted)                                │
├─────────────────────────────────────────────────────────┤
│ L6  Iltric SDK / Runtime                                │
│     Owner: Iltric team                                  │
│     Language: Rust/C                                     │
│     Trust: 2 (trusted)                                  │
├─────────────────────────────────────────────────────────┤
│ L5  Iltric Desktop Shell                                │
│     Owner: Iltric team                                  │
│     Language: Rust (Smithay)                            │
│     Trust: 2 (trusted)                                  │
├─────────────────────────────────────────────────────────┤
│ L4  Iltric Core Services                                │
│     Owner: Iltric team                                  │
│     Language: Rust/C                                     │
│     Trust: 1 (highly trusted)                           │
│     Services: init, securityd, storaged, netd,          │
│     audiod, gfxd, packaged, updated, defender, AI       │
├─────────────────────────────────────────────────────────┤
│ L3  System Libraries / ABI Surface                      │
│     Owner: Iltric team / upstream                       │
│     Language: C/Rust                                    │
│     Trust: 1 (highly trusted)                           │
│     glibc, libdrm, PipeWire client, D-Bus               │
├─────────────────────────────────────────────────────────┤
│ L2  HAL / Drivers                                       │
│     Owner: upstream Linux / Iltric config               │
│     Language: C                                         │
│     Trust: 0 (kernel-privileged)                        │
│     DRM/KMS, virtio, NVMe, WiFi, audio codecs           │
├─────────────────────────────────────────────────────────┤
│ L1  Iltric Kernel (Linux LTS, custom .config)           │
│     Owner: upstream Linux + Iltric patches              │
│     Language: C                                         │
│     Trust: 0 (kernel-privileged)                        │
│     Features: MGLRU, sched_ext, ntsync, fanotify,       │
│     Landlock, BPF-LSM, dm-verity, PREEMPT              │
├─────────────────────────────────────────────────────────┤
│ L0  Firmware / Boot (UEFI + OVMF / real firmware)       │
│     Owner: hardware vendor / Iltric UKI                 │
│     Trust: 0 (hardware-privileged)                      │
└─────────────────────────────────────────────────────────┘
```

## 2. Trust Levels

| Level | Label | Examples |
|-------|-------|----------|
| 0 | Kernel / Firmware | Linux kernel, UEFI, drivers |
| 1 | Core Services | init, securityd, defender, update |
| 2 | Trusted Platform | compositor, shell, SDK, IPC broker |
| 3 | Signed Native App | Iltric-registered apps |
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
        → Iltric System API
          → Iltric Kernel
```

**INVARIANT:** Iltric Core must NEVER depend on Windows compatibility.

## 5. Component Ownership

| Component | Owner | Language | Upstream/Own | Trust |
|-----------|-------|----------|--------------|-------|
| Kernel | upstream Linux | C | upstream + config | 0 |
| Init/service supervisor | systemd | C | upstream | 1 |
| Compositor | Iltric | Rust | own (Smithay-based) | 2 |
| Shell/Desktop | Iltric | Rust | own | 2 |
| Defender daemon | Iltric | Rust/C | own | 1 |
| AI service | Iltric + llama.cpp | Rust/C | integrated | 1 |
| Package manager | Flatpak + Iltric | C/Rust | upstream + own | 1 |
| Update system | systemd-sysupdate | C | upstream | 1 |
| Network stack | iwd + networkd | C | upstream | 1 |
| Audio stack | PipeWire | C | upstream | 1 |
| Graphics stack | Mesa | C | upstream | 1-2 |
| Security broker | Iltric | Rust | own | 1 |
| Capability engine | Iltric | Rust | own | 1 |
