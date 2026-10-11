# LITRIC — PHASE 01 PROMPT: BOOTABLE PROTOTYPE (QEMU-FIRST)

**Prerequisite:** GATE 00 passed.
**Deliverable:** an LITRIC disk image that boots to a serial shell inside QEMU on a Windows host, reproducibly, with CI smoke test.

---

## OBJECTIVE
`BOOT LITRIC SAFELY IN A VIRTUAL MACHINE.` — the first milestone from the master prompt.

## WHY
Nothing else can be measured, debugged, or trusted until there is a reproducible image + boot + log pipeline.

---

## TASKS

1. **Kernel config fragment** — `/kernel/litric-base.config`
   Must enable, at minimum:
   ```
   CONFIG_LRU_GEN=y  CONFIG_LRU_GEN_ENABLED=y        # MGLRU
   CONFIG_SCHED_CLASS_EXT=y                          # sched_ext (scx_lavd later)
   CONFIG_NTSYNC=y                                   # Wine/Proton NT primitives
   CONFIG_FANOTIFY=y  CONFIG_FANOTIFY_ACCESS_PERMISSIONS=y   # defender
   CONFIG_SECURITY_LANDLOCK=y  CONFIG_BPF_LSM=y
   CONFIG_DM_VERITY=y  CONFIG_EROFS_FS=y  CONFIG_BTRFS_FS=y
   CONFIG_ZRAM=y  CONFIG_ZRAM_MULTI_COMP=y  CONFIG_CRYPTO_ZSTD=y  CONFIG_CRYPTO_LZ4=y
   CONFIG_PREEMPT=y  CONFIG_HZ_1000=y  CONFIG_NO_HZ_IDLE=y
   CONFIG_EFI_STUB=y  CONFIG_GDB_SCRIPTS=y
   ```
   Then **trim**: disable every driver class not needed for QEMU virtio + a modern PC. Record image size before/after in MB.

2. **Image pipeline** — `/image/mkosi.conf` + profiles
   Produce a GPT disk image with: ESP (FAT32) + system partition. Output `litric.qcow2` and `litric.raw`.
   Must be reproducible: same inputs → same hash. Record the hash.

3. **UKI** — build a Unified Kernel Image (kernel + initrd + cmdline, single signed PE) placed on the ESP. Use dev keys for now.

4. **QEMU launcher (Windows host)** — `/tools/run-qemu.ps1` and `/tools/run-qemu.sh`
   Baseline command:
   ```
   qemu-system-x86_64 \
     -machine q35,accel=whpx,kernel-irqchip=off -cpu max -smp 4 -m 4G \
     -drive if=pflash,format=raw,unit=0,readonly=on,file=OVMF_CODE_4M.fd \
     -drive if=pflash,format=raw,unit=1,file=OVMF_VARS_4M.fd \
     -drive if=virtio,format=qcow2,file=litric.qcow2 \
     -device virtio-net-pci,netdev=n0 -netdev user,id=n0,hostfwd=tcp::2222-:22 \
     -serial mon:stdio
   ```
   Requirements:
   - Use the **4 MB split OVMF** (CODE readonly + VARS writable). 2 MB variants are insufficient.
   - Provide `--debug` flag adding `-s -S` (gdbstub :1234, frozen at start).
   - Provide `--tcg` fallback flag (WHPX SMP boot bug #3178 workaround: also allow `-smp 1`).
   - Never touch a host physical disk. Assert the image path is inside the project dir.

5. **Serial + logging** — capture all boot output to `/tools/logs/boot-<timestamp>.log`. Enable `earlyprintk` and `console=ttyS0`.

6. **Kernel debugging doc** — `/docs/DEV-01-debug.md`
   ```
   qemu ... -s -S
   gdb vmlinux
     add-auto-load-safe-path <src>/scripts/gdb/vmlinux-gdb.py
     target remote :1234
     hbreak start_kernel        # hardware bp — software bp on early code is missed
     c
   ```
   Boot with `nokaslr` when debugging.

7. **CI smoke test** — `/tests/smoke_boot.py`
   Headless QEMU + expect/pytest on the serial console. Assert:
   - UEFI hands off to the UKI
   - kernel reaches userspace
   - a login/shell prompt string appears
   - exit code 0 within a timeout
   Fail loudly on kernel panic / oops strings.

8. **First measurements** — record into STATE BLOCK:
   - `systemd-analyze` (firmware / loader / kernel / userspace / total)
   - idle RAM headless (`free -m` after 60 s settle)
   - image size, kernel image size, initrd size

---

## EXIT GATE — GATE 01
- [ ] `litric.qcow2` boots to a shell in QEMU on the Windows host, unattended.
- [ ] Image build is reproducible (two builds → identical hash).
- [ ] CI smoke test passes and fails correctly when a deliberate panic is injected.
- [ ] **Boot time ≤ 8 s** total on SSD (record the number; if > 8 s, record and list the top 3 offenders from `systemd-analyze blame`).
- [ ] **Headless idle RAM ≤ 150 MB.**
- [ ] `-s -S` + gdb attach demonstrated once with a screenshot/log.
- [ ] Zero host-disk writes outside the project directory (verify).

## FAILURE MODES & ROLLBACK
- WHPX hang on SMP → fall back to `-smp 1` or `--tcg`; document, do not silently work around.
- OVMF variable corruption → delete `OVMF_VARS_4M.fd`, re-copy from template (keep a pristine copy).
- Unbootable image → previous `litric.qcow2` is kept as `litric.last-good.qcow2` by the build script. Always.

## NEXT STEP
`03-PHASE-02-CORE.md`
