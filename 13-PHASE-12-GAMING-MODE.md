# LITRIC — PHASE 12 PROMPT: GAMING MODE & DEEP SETTINGS

**Prerequisite:** GATE 05 (graphics). Best run after GATE 06. Cross-cutting phase.
**Deliverable:** a gaming stack that matches or beats Windows on the same hardware — measured, not asserted.

> **Reality anchor:** SteamOS already beats Windows 11 on identical handheld hardware — in Ars Technica's June 2025 Legion Go S testing at 1920×1200 High, Returnal held a steady 33 FPS on SteamOS while Windows "barely managed 18 FPS with Lenovo's official drivers"; SteamOS won 4 of 5 titles (≈24% average across the wins), with Windows's single win by ~2%. The win comes from a lean stack + gaming-tuned integration, **not** from magic. LITRIC's job is to integrate that stack well and expose the knobs.

---

## OBJECTIVE
A Gaming Mode that changes real system state, plus a Deep Settings surface that exposes every knob with its measured cost.

---

## TASKS

1. **gamescope integration** — `/gaming/gamescope/`
   Micro-compositor: games run in an isolated Xwayland sandbox; frames flip directly via DRM/KMS (fewer copies, lower latency); compositing via async Vulkan compute so the overlay renders even while the game saturates the GPU.
   Expose: resolution spoofing, FSR 1.0 (`-F fsr`), NIS (`-F nis`), integer scaling, FPS cap, VRR (`--adaptive-sync`), HDR (`--hdr-enabled`).
   ```
   gamescope -W 2560 -H 1440 -r 144 --hdr-enabled --adaptive-sync -- %command%
   ```
   **Document the trade-off:** gamescope's own `--fps-limit` adds latency versus an in-engine limiter — surface that warning in the UI for competitive titles.

2. **GameMode integration** — `/gaming/gamemode/`
   D-Bus-activated daemon. Confirmed effects to wire and verify:
   - CPU governor → `performance` (`desiredgov`)
   - I/O priority raised, process `renice`
   - amdgpu `amd_performance_level=high` → `power_dpm_force_performance_level=high`
   - NVIDIA "Prefer Maximum Performance"
   - screensaver/idle inhibit
   - optional `SCHED_ISO` (not in mainline; auto-disabled ≤3 cores, falls back above ~70% CPU)
   - `igpu_desiredgov`: note the documented iGPU case where a *powersave* governor under iGPU load improved Shadow of the Tomb Raider by ~25–30% on an i7-1065G7 laptop — a single-title laptop-iGPU result. **Re-measure on LITRIC hardware; do not repeat it as a general claim.**
   User must be in the `gamemode` group; the installer handles it.

3. **Proton stack** — `/gaming/proton/`
   Proton = Wine + DXVK (D3D9/10/11→Vulkan) + VKD3D-Proton (D3D12→Vulkan) + Fossilize shader pre-caching (this is what removes first-run stutter — treat the shader cache directory as a first-class system path with its own storage budget).
   Wine 11 completed WoW64, so 32-bit titles no longer need a full 32-bit library set.

4. **ntsync** — `/gaming/ntsync/`
   Kernel NT synchronization primitives (mainlined in Linux 6.14), removing wineserver RPC round-trips. Load `ntsync` by default (Valve enabled it by default in SteamOS 3.7.20 Beta).
   **Honesty requirement:** the headline 124–678% figures are versus *vanilla upstream Wine*, "which almost nobody uses for gaming anymore." Users already on Proton with fsync typically see **10–50%**, concentrated in heavily multi-threaded titles and frame-time consistency; GPU-bound titles see ~nothing. Never quote the big number without this context.

5. **sched_ext scheduler** — `/gaming/sched/`
   Offer `scx_lavd` (Latency-criticality Aware Virtual Deadline — BPF+Rust, built for gaming, includes Core Compaction for power) as the Gaming Mode scheduler; `scx_bpfland` / `scx_flash` / `scx_rusty` as alternatives; EEVDF as the default.
   Switching schedulers must be live, reversible, and never require a reboot. Measure FPS **and 1% lows** for each — average FPS alone is not evidence.

6. **DEEP SETTINGS surface** — `/shell/settings/gaming/`
   Every knob shows: current, default, what it does, measured impact, and risk. Grouped:
   ```
   CPU     governor / EPP, core isolation + pinning, split-lock detection
   Memory  hugepages, zram algorithm (lz4 for gaming vs zstd default)
   GPU     power_dpm_force_performance_level, pp_od_clk_voltage (AMD OC), power cap
   IO      IRQ affinity, IO scheduler, IOWeight for game.slice
   Net     net.core.busy_poll, NIC coalescing, BBR, fq_codel
   Display VRR on/off, tearing-control, FPS cap, FSR/NIS, HDR
   Audio   PipeWire quantum (see Phase: audio profile)
   Sched   EEVDF | scx_lavd | scx_bpfland | scx_flash
   Wine    ntsync on/off, DXVK/VKD3D options, shader cache location + size
   ```
   **`mitigations=off` is a special case:** off by default, behind a confirmation dialog that states plainly that it re-exposes the machine to Spectre/Meltdown-class attacks. Gains are workload-dependent (single digits to ~10%). Never enable it silently, never enable it to win a benchmark.

7. **Game profiles** — per-title profile storage: which knobs, which Proton version, which scheduler, which upscaler. Import/export. `game.slice` gets its own cgroup weights and the notification service suppresses non-urgent popups.

8. **Anti-cheat honesty** — `/docs/GAMING-anticheat.md`
   EAC and BattlEye *can* support Proton but require per-developer opt-in. Some major titles do not enable it (Fortnite is unplayable on SteamOS; GTA Online and Apex removed Deck support). **LITRIC cannot fix this.** The game manager must show a compatibility status per title and never imply otherwise.

---

## EXIT GATE — GATE 12
- [ ] A reference set of **≥ 3 titles** benchmarked on LITRIC vs Windows 11 **on identical hardware**, reporting avg FPS, **1% lows**, and frame-time graphs. LITRIC matches or beats Windows in ≥ 2 of 3, or the gap is explained with profiling data.
- [ ] Gaming Mode toggle verifiably changes: governor, GPU perf level, ioprio, scheduler, notification suppression — each confirmed by reading the corresponding sysfs/sysctl.
- [ ] Toggling Gaming Mode off restores every value to its exact prior state (diff before/after).
- [ ] ntsync on/off measured on ≥ 2 titles with the honest context recorded.
- [ ] scx_lavd vs EEVDF measured with 1% lows on ≥ 2 titles.
- [ ] Shader pre-caching demonstrably reduces first-run stutter (frame-time capture, first run vs second).
- [ ] `mitigations=off` requires explicit confirmation and is off after a fresh install.
- [ ] Anti-cheat compatibility page exists and is accurate.

## FAILURE MODES & ROLLBACK
- Gaming Mode leaves the GPU pinned at max after a crash → a watchdog restores defaults on daemon death; test by SIGKILLing it.
- Live scheduler swap hangs → sched_ext falls back to EEVDF automatically; verify the fallback fires.
- OC values (pp_od_clk_voltage) cause instability → per-boot reset unless explicitly persisted, plus a "safe boot" entry that ignores all gaming overrides.

## NEXT STEP
`14-PHASE-13-PERF-MEMORY.md`
