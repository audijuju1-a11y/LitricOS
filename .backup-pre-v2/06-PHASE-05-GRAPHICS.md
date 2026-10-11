# ILTRIC — PHASE 05 PROMPT: GRAPHICS & COMPOSITOR

**Prerequisite:** GATE 04 passed.
**Deliverable:** an ILTRIC compositor on DRM/KMS with input, modern Wayland protocols, and *measured* latency.

> **HOST WARNING:** QEMU builds for Windows generally ship with OpenGL disabled — virgl/venus acceleration is not available from a Windows host, and virtiofs is Linux-host-only. Do this phase on a Linux host or WSL2 for accelerated testing; keep the Windows host for software-rendered functional tests only. Record which host each measurement came from.

---

## OBJECTIVE
A compositor that is ours, memory-safe, and modern — not a re-skin.

---

## TASKS

1. **Foundation: Smithay (Rust).** Start from its example compositor. Rationale: memory-safe, supports DRM syncobj / dmabuf / DRM leasing, and is proven at desktop scale (it powers COSMIC). Alternative considered and rejected for this project: wlroots (C, most mature but unsafe language); fully custom (months of effort for basics).

2. **Core plumbing** — `/compositor/`
   - session: `libseat`/`seatd`
   - input: `libinput` (keyboard, mouse, touchpad gestures, touch)
   - display: DRM/KMS + GBM
   - render: Vulkan-first, GL fallback

3. **Protocols to implement (2026 baseline)**
   ```
   xdg-shell, xdg-decoration
   linux-dmabuf-v1
   linux-drm-syncobj-v1        # explicit sync — REQUIRED for sane NVIDIA
   fractional-scale-v1
   presentation-time           # this is your latency measurement instrument
   tearing-control-v1          # gaming
   wp-color-management-v1      # HDR (upstream successor to frog-protocols)
   adaptive sync / VRR
   ```

4. **Output capability detection** — never assume. Query and expose: refresh rates, VRR range, HDR metadata, color primaries, bit depth, DPI. Feature flags derive from the query, not from a hardcoded table.

5. **Mesa build policy** — build only needed drivers (radeonsi/RADV, ANV/Iris, NVK, Zink, virtio). Record the disk + RAM saving vs a full Mesa build.

6. **Latency instrumentation** — `/tools/bench/latency/`
   - in-compositor: `presentation-time` deltas, frame-time histogram, missed-flip counter
   - external: document the click-to-photon method (high-speed camera or LDAT)
   - Establish an ILTRIC baseline **and** measure a reference compositor (KWin or Mutter) on the same hardware, so any claim is a comparison you actually ran.

7. **Frame budget enforcement** — the compositor logs a warning whenever a frame exceeds budget:
   ```
   60 Hz = 16.67 ms | 120 Hz = 8.33 ms | 144 Hz = 6.94 ms | 240 Hz = 4.17 ms
   ```

8. **Multi-monitor + hotplug** — different scales, different refresh rates, hotplug without a crash. Test in QEMU with two virtual outputs.

---

## EXIT GATE — GATE 05
- [ ] Compositor starts on DRM/KMS, shows a surface, accepts keyboard+mouse+touchpad input.
- [ ] Explicit sync, fractional scaling, and presentation-time verified with a test client.
- [ ] VRR and tearing-control demonstrated (or documented as unavailable on the test hardware — with the query output proving it).
- [ ] **Desktop idle RAM ≤ 600 MB** with compositor running (record the exact number).
- [ ] Frame-time histogram captured for a 60 s animation; 99th percentile within budget.
- [ ] Multi-monitor hotplug with mixed scale does not crash and does not leak (RSS stable over 30 min).
- [ ] A recorded latency comparison vs one reference compositor exists. **No latency claim without it.**

## FAILURE MODES & ROLLBACK
- GPU driver hang → compositor must be restartable without killing the session; user data survives.
- Unsupported HDR metadata → fall back to SDR, log the reason, never black-screen.
- No KMS available (software-only host) → headless/nested mode for functional tests.

## NEXT STEP
`07-PHASE-06-DESKTOP.md`
