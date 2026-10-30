# LITRIC — Threat Model
**Document:** SEC-00-threatmodel.md | **Phase:** 00 | **Status:** DESIGN-ONLY
**Method:** STRIDE per trust boundary

---

## 1. Adversary Profiles

| ID | Adversary | Capability | Goal |
|----|-----------|------------|------|
| ADV-1 | Malicious app (signed or unsigned) | Runs in user space | Escape sandbox, access other apps' data, persist |
| ADV-2 | Malicious USB device | Physical access | Execute code, exfiltrate data, persist |
| ADV-3 | Network attacker | Remote, MITM | Intercept traffic, inject code, DoS |
| ADV-4 | Malicious update server | Controls update infra | Push backdoored image, rollback attack |
| ADV-5 | Physical attacker | Disk removal | Decrypt data, modify boot chain |
| ADV-6 | Compromised driver | Kernel module | Rootkit, data exfil, persistence |
| ADV-7 | Malicious AI prompt/model | User-triggered | Data exfil via AI, prompt injection |

---

## 2. STRIDE Analysis by Boundary

### BOUNDARY A: Kernel ↔ Userspace

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Kernel exploit / privilege escalation | Elevation of Privilege | Kernel LTS updates, Landlock, seccomp, sysctl hardening | P00/P07 | Kernel vulns exist between patch cycles |
| Syscall abuse by sandboxed app | Elevation of Privilege | seccomp-bpf deny-by-default, Landlock LSM | P07 | Zero-day kernel bypass |
| Malicious kernel module | Tampering | Module signing, lockdown mode | P07 | Hardware root-of-trust needed for full chain |

### BOUNDARY B: Core Service ↔ App

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| IPC spoofing / confused deputy | Elevation of Privilege | Capability engine + message-level auth on every IPC | P00/P07 | Complex service interactions may have gaps |
| Rogue D-Bus service | Spoofing | Service activation policy, bus name ownership | P02 | Trusted services only on system bus |
| Capability escalation via IPC | Elevation of Privilege | Capabilities are non-transferable, broker validates | P07 | IPC implementation bugs |

### BOUNDARY B1: Files daemon socket

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Another local uid connects to the Files daemon | Spoofing / Elevation of Privilege | `/run/litric/files.sock` is owned by `litric`, the user `litric-files@.service` runs as, and is mode `0600`; `connect` from every other uid fails with `EACCES` before a request byte. `ci/lint-unit-sockets.py` rejects a root-owned unit and has a planted negative control. | P00 | No `SO_PEERCRED` check; the kernel's owner/mode check is the boundary. Cross-uid refusal is not executed on this one-uid host. |
| Compromised root shell uses the Files daemon | Elevation of Privilege | Accepted by design: the shell is uid 0 and is already trusted to drive the display. The daemon confines logical paths to `/var/lib/litric-files/root` and holds no network or device access. | P00 | This is not a sandbox against root; a root shell can use the daemon and can reach root-owned paths. |

### BOUNDARY C: App ↔ App

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Memory disclosure via shared memory | Information Disclosure | No shared memory by default; explicit handle negotiation | P00/P07 | Side-channel attacks (Spectre-class) |
| Process impersonation | Spoofing | PID + identity verification via broker | P02/P07 | Timing-based attacks |
| Resource exhaustion by app | Denial of Service | cgroups v2 limits per-slice | P02 | CPU/IO starvation within quota |

### BOUNDARY D: Native ↔ Windows Compatibility

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Windows app escape to native | Elevation of Privilege | Separate process tree, namespace, no ambient access | P00/P07 | Wine/Proton complex attack surface |
| Windows app accessing native secrets | Information Disclosure | Separate filesystem namespace, no shared credential store | P07 | Misconfigured compat profile |
| Registry/path virtualization bypass | Tampering | Virtual filesystem with capability checks | P07 | TOCTOU in path resolution |

### BOUNDARY E: System ↔ External

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Network MITM | Tampering/Info Disclosure | DNS-over-TLS, nftables default-deny inbound | P04 | Compromised CA |
| Malicious USB device | Elevation of Privilege | USBGuard / udev policy, device capability model | P11 | Zero-day USB exploit |
| Physical disk access | Information Disclosure | LUKS2 + TPM2-sealed, PCR policy | P03 | Cold boot attack, evil maid |
| Malicious update artifact | Tampering | cosign + TUF + monotonic version counter | P08 | Compromised signing key |

### BOUNDARY F: Image Integrity

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Tampered rootfs | Tampering | dm-verity + root hash in signed UKI cmdline | P03 | Key compromise |
| Rollback to vulnerable version | Elevation of Privilege | Monotonic version counter, anti-downgrade | P08 | User-initiated recovery override |
| ESP modification | Tampering | Signed UKI, ESP integrity check at boot | P01/P03 | Physical access + no Secure Boot |

### ADV-7: Malicious AI Prompt/Model

| Threat | STRIDE | Mitigation | Phase | Residual |
|--------|--------|------------|-------|----------|
| Prompt injection via user files | Tampering | AI reads files only with explicit `fs.*` capability, scoped | P09 | Indirect injection via user documents |
| AI exfiltrating data to cloud | Information Disclosure | `ai.cloud` capability required, off by default, packet-verified | P09 | Compromised local model |
| AI capturing screen silently | Information Disclosure | No silent screen capture; mic/camera indicators mandatory | P09 | Firmware-level capture |

---

## 3. Threat Traceability

Every trust boundary in ARCH-00-boundaries.md has at least one STRIDE entry above.

| Boundary | Covered? |
|----------|----------|
| A: Kernel ↔ Userspace | ✓ |
| B: Core Service ↔ App | ✓ |
| C: App ↔ App | ✓ |
| D: Native ↔ Windows Compat | ✓ |
| E: System ↔ External | ✓ |
| F: Image Integrity | ✓ |
| AI-specific | ✓ |

## 4. Open Risks (recorded, not resolved)

- Kernel anti-cheat games cannot be supported (accepted, documented)
- NPU driver/runtime maturity (experimental only)
- Commercial AV detection rate not matched (advisory defender)
- Side-channel attacks (Spectre-class) across app boundary (unmitigated without microcode)
