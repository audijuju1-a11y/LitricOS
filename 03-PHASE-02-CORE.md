# LITRIC — PHASE 02 PROMPT: CORE SYSTEM (INIT, MEMORY, PROCESSES, IPC)

**Prerequisite:** GATE 01 passed.
**Deliverable:** a lean, supervised, resource-governed userland with a memory policy that is measured, not guessed.

---

## OBJECTIVE
Turn "boots to a shell" into "runs a governed system" — service supervision, cgroups v2, memory pressure handling, IPC, and a hard RAM budget.

---

## TASKS

1. **Unit inventory & trimming** — `/core/units/`
   - List every enabled unit. For each: justify or delete. Target: minimum viable set.
   - Record `systemd-analyze blame` before/after and the RAM delta in MB.

2. **journald memory policy** — `/core/journald.conf.d/litric.conf`
   ```
   Storage=volatile
   RuntimeMaxUse=32M
   RuntimeMaxFileSize=8M
   ForwardToSyslog=no
   ```
   Persistent logs only in diagnostic boot mode.

3. **zram** — `/core/zram-generator.conf`
   ```
   [zram0]
   zram-size = ram          # kernel assumes ~2:1, effective headroom is larger
   compression-algorithm = zstd
   ```
   Plus sysctl `/core/sysctl.d/10-litric-mm.conf`:
   ```
   vm.swappiness = 180            # correct for zram-only systems
   vm.watermark_scale_factor = 125
   vm.dirty_ratio = 10
   vm.dirty_background_ratio = 5
   vm.page-cluster = 0
   ```
   **Measure:** compression ratio (`zramctl`), CPU cost under load, and stall time. zstd ≈ 5:1 real world; lz4 decompresses ~5× faster — reserve lz4 for the Gaming Mode profile (Phase 12).

4. **MGLRU enable + verify**
   ```
   echo y > /sys/kernel/mm/lru_gen/enabled
   cat /sys/kernel/debug/lru_gen
   ```
   Ship it enabled by default. Record before/after: kswapd CPU %, low-memory kills, app launch time.

5. **systemd-oomd (PSI-based)** — `/core/oomd.conf.d/litric.conf`
   Configure on memory + swap pressure, with per-slice policy:
   - `user.slice` — protect the shell, kill the greediest app
   - `app-*.slice` — eligible
   - `system.slice` — protected
   Test with a deliberate memory bomb; assert the desktop survives and the bomb dies.

6. **cgroups v2 resource model** — `/docs/CORE-02-cgroups.md`
   Slice layout: `system.slice`, `user.slice`, `app.slice`, `game.slice`, `ai.slice`, `defender.slice`.
   Each gets `MemoryHigh=`, `MemoryMax=`, `CPUWeight=`, `IOWeight=`. **`defender.slice` gets a hard `MemoryMax=100M`** (see Phase 07 gate).

7. **Service sandboxing defaults** — every LITRIC service unit must carry:
   ```
   NoNewPrivileges=yes
   ProtectSystem=strict
   ProtectHome=yes
   PrivateTmp=yes
   PrivateDevices=yes
   RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
   SystemCallFilter=@system-service
   CapabilityBoundingSet=      # empty unless justified in the unit's comment
   ```
   Any deviation requires an inline comment naming the threat it accepts.

8. **IPC** — choose and document one primary bus (D-Bus broker recommended for ecosystem compatibility). Define message-level permission checks that call `litric.caps` from Phase 00.

9. **Recovery target** — a `litric-recovery.target` reachable from the bootloader that starts: shell, storage tools, network, log extraction. No compositor, no AI, no defender.

---

## EXIT GATE — GATE 02
- [ ] **Headless idle RAM ≤ 120 MB** (improved vs Gate 01) — record `free -m` and per-slice `systemd-cgtop`.
- [ ] Boot time not regressed vs Gate 01 (≤ 8 s).
- [ ] Memory-bomb test: system stays interactive, oomd kills the bomb, logged with reason.
- [ ] Every shipped unit has the sandboxing block or a written justification.
- [ ] Recovery target boots and can mount + read the system partition.
- [ ] MGLRU + zram verified active with recorded ratios.

## FAILURE MODES
- oomd kills the wrong thing → wrong slice weights; fix policy, add regression test.
- zstd zram raises CPU under heavy swap → switch that profile to lz4 and re-measure. Do not guess; measure both.

## NEXT STEP
`04-PHASE-03-STORAGE.md`
