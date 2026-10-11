# ILTRIC — PHASE 03 PROMPT: STORAGE, FILESYSTEM, ENCRYPTION, RECOVERY

**Prerequisite:** GATE 02 passed.
**Deliverable:** a signed read-only system image, a snapshot-capable user volume, encryption, and a proven recovery path.

---

## OBJECTIVE
Make ILTRIC's on-disk state tamper-evident, snapshot-able, encrypted, and recoverable from corruption.

---

## TASKS

1. **Partition layout** — `/docs/STOR-03-layout.md`
   ```
   p1 ESP        FAT32   1 GB    UKI_A, UKI_B, bootloader
   p2 system_A   erofs   4 GB    read-only + dm-verity
   p3 system_B   erofs   4 GB    read-only + dm-verity   (A/B slot)
   p4 var        Btrfs   grow    /var, machine state
   p5 home       Btrfs   grow    user data, zstd:1
   ```
   State the disk overhead of A/B explicitly (2× system size) and why it is worth it.

2. **erofs + dm-verity** — build the system image read-only and compressed; generate the verity hash tree; embed the root hash **in the signed UKI cmdline** so the kernel refuses a tampered rootfs.
   Test: flip a byte in system_A with a hex editor → boot must fail closed, not silently continue.

3. **Btrfs policy** — `/core/btrfs/`
   - Subvolumes: `@var`, `@home`, `@snapshots`, `@games`
   - `compress=zstd:1` on `@home`/`@var`
   - `nodatacow` on `@games` and any DB directory (CoW write-amplification is real for large random-write files)
   - automatic pre-update snapshot hook (ties into Phase 08)

4. **Encryption** — `/core/crypt/`
   - LUKS2 with **Argon2id** KDF (memory-hard; resists GPU brute force)
   - `systemd-cryptenroll` TPM2 sealing bound to PCRs covering firmware + bootloader + UKI
   - fallback recovery key generated at install, shown once, never stored
   - `fscrypt` for per-user home encryption
   **Measure:** dm-crypt overhead with `fio` — sequential and 4K random, encrypted vs plain. Record the % delta. Expect small on AES-NI, larger on 4K random.

5. **Storage API** — implement `iltric.fs` from Phase 00.
   Apps get paths through capability-scoped handles, never raw `/`. No app enumerates the real filesystem tree without `fs.read.all`.

6. **Recovery environment** — `/recovery/`
   Bootable from the ESP. Must be able to:
   - verify + repair Btrfs (`btrfs check`, scrub)
   - re-derive verity and report which slot is corrupt
   - roll back to the other slot
   - dump logs to removable media
   - reset the user password with the recovery key
   Never able to bypass encryption.

7. **Integrity monitoring** — enable `fs-verity` on executable app content (used by defender in Phase 07).

---

## EXIT GATE — GATE 03
- [ ] Tampered system partition → boot fails closed with a clear message and offers recovery. Demonstrated.
- [ ] Btrfs snapshot create + rollback of `@home` demonstrated, timed (record seconds).
- [ ] LUKS2+TPM2 auto-unlock works; changing the UKI breaks the seal (PCR change) and falls back to passphrase. Demonstrated.
- [ ] `fio` numbers recorded for encrypted vs plain (seq read/write, 4K random read/write).
- [ ] Recovery environment boots from a deliberately broken system slot.
- [ ] Idle RAM not regressed (≤ 120 MB headless).

## FAILURE MODES & ROLLBACK
- Verity root hash mismatch after a legit build → build pipeline must embed the hash automatically; never hand-copy.
- TPM PCR churn on firmware update → document the re-enrollment flow; recovery key is the escape hatch.
- Btrfs ENOSPC → reserve a global block reserve and add a monitor that warns at 85% and blocks updates at 95%.

## NEXT STEP
`05-PHASE-04-NETWORK.md`
