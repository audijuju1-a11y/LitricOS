# ADR-0001: Kernel Foundation
**Status:** Accepted | **Date:** 2026-08-26

## Context
Litric needs a kernel. Writing from scratch (Redox, SerenityOS model) costs years before hardware support.

## Decision
Use mainline Linux LTS with a custom trimmed `.config`.

## Consequences
+ Hardware support out-of-the-box (virtio, DRM/KMS, USB, NVMe)
+ Community patches + security updates
+ Upstream drivers, firmware, ecosystem
- Linux kernel is C (not memory-safe)
- Config complexity (trimming unused drivers)

## Revisit-if
- A memory-safe kernel (e.g., Redox kernel) reaches production-quality hardware support
- Linux licensing conflicts with Litric distribution model
- Litric needs custom syscall ABI that conflicts with Linux ABI stability
