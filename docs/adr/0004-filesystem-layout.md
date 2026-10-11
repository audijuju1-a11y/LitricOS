# ADR-0004: Filesystem Layout
**Status:** Accepted | **Date:** 2026-08-26

## Context
Litric needs a tamper-evident system partition and a snapshot-capable user volume.

## Decision
- System: erofs + dm-verity (read-only, signed)
- User: Btrfs + zstd:1 (snapshots = rollback)
- Encrypted: LUKS2 + Argon2id + TPM2-sealed

## Consequences
+ erofs is read-only (smaller, faster, tamper-evident)
+ dm-verity catches rootfs modification
+ Btrfs snapshots enable instant rollback
+ LUKS2 + Argon2id resists GPU brute force
+ TPM2-sealed auto-unlock
- erofs is read-only (no live modification of system)
- Btrfs CoW write amplification on game/DB dirs (mitigated with nodatacow)
- TPM PCR churn on firmware update requires re-enrollment

## Revisit-if
- erofs performance is insufficient for system partition
- Btrfs instability under heavy write load
- bcachefs becomes production-ready
