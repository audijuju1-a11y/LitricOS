# LitricOS FPS & Frame-Time Performance Targets

**Status: TARGETS, NOT MEASUREMENTS.** These numbers are engineering acceptance targets. They are not claims about current LitricOS performance or any user's QEMU run. A target becomes verified only when reproducible benchmark evidence is recorded in `docs/EVIDENCE.md` with the exact build/commit, host and guest configuration, commands, workload, sample count, and raw results.

## 1. Acceptance targets

| Scenario | Target | Frame-time target |
|---|---:|---:|
| Normal desktop use | Sustained 60 FPS | ≤ 16.67 ms per frame |
| Multiple open windows | Sustained 60 FPS | ≤ 16.67 ms per frame |
| Standard animations and VFX | Sustained 60 FPS | ≤ 16.67 ms per frame |
| Heavy VFX / multiple-window workload in the documented QEMU reference setup | At least 30 FPS | ≤ 33.33 ms per frame |
| Frame-time p95 during normal workloads | — | ≤ 20 ms |
| Frame-time p99 during normal workloads | — | ≤ 33.33 ms |

At 60 FPS, the frame budget is approximately 16.67 ms. At 30 FPS, it is approximately 33.33 ms. These are budgets, not guarantees: measure the actual renderer and end-to-end presentation path before reporting a pass.

## 2. Required workloads

Benchmark at minimum:

1. Idle desktop after the system has settled.
2. Desktop with multiple windows open.
3. Window dragging and resizing.
4. Dock and launcher open/close and interaction animations.
5. Workspace transitions and window minimize/restore transitions.
6. Lock-screen animation and unlock flow, where supported by the test setup.
7. Standard VFX enabled versus reduced-effects mode.
8. A documented heavy workload with multiple windows and the most expensive supported effects.

Record the number of windows, resolution, display refresh rate, QEMU version and display/renderer configuration, host hardware, guest build/commit, and whether measurements are from the guest renderer or end-to-end display presentation.

## 3. Required measurements

Report, where instrumentation permits:

- Actual FPS over a defined measurement interval; include sustained results, not only a short peak.
- Frame-time p50, p95, p99, maximum, and distribution/histogram.
- Missed frame deadlines, dropped frames, long frames, and stutter frequency.
- Input-to-visible-response latency for representative interactions.
- CPU usage, RSS/RAM, and GPU usage only when a real GPU rendering path is in use and measurable.
- Renderer and compositor timing separately when possible, so bottlenecks can be attributed.
- Measurement overhead and any unavailable metrics.

Do not infer FPS solely from a single average frame time if frame pacing is uneven. Do not describe QEMU virtual GPU support as proof that LitricOS's own renderer is hardware-accelerated.

## 4. Engineering requirements

1. Instrument first and reproduce the bottleneck before optimizing.
2. Check animation scheduling and frame wakeups, especially the case where an animation remains unsettled after damage is cleared.
3. Profile full redraws, blur, shadows, compositing, pixel loops, text rendering, and window-count scaling.
4. Preserve visual quality where feasible; if an effect is reduced or disabled to meet the budget, document the trade-off and ensure reduced-effects mode remains usable.
5. Consider a GPU-backed renderer only after measurements show that it is necessary and the supported hardware/software path can be tested. Keep a functional software-renderer fallback.
6. Add regression tests for frame scheduling and animation progression where feasible. Automated timing tests should avoid flaky hard real-time assertions on shared CI runners; use deterministic logic tests in CI and repeatable performance benchmarks on a documented reference machine.
7. Mark every target as `PASS`, `FAIL`, or `NOT TESTED`. Never claim 60 FPS unless it was measured under the documented conditions.

## 5. Evidence and reporting

Store reproducible benchmark procedure and results in `docs/EVIDENCE.md` or a linked performance report. Include:

- Git commit SHA and build/profile.
- Host OS, CPU, RAM, GPU (if applicable), QEMU version, guest resolution, and display backend.
- Exact commands and workload steps.
- Measurement duration and sample count.
- Raw output or an attached artifact, plus p50/p95/p99/max frame times and sustained FPS.
- Known limitations and whether the result measures renderer time or end-to-end presentation.

A historical benchmark, a target in a planning document, or a result from a different machine must not be presented as a measurement of the current QEMU run.

## 6. Priority

Prioritize stable frame pacing and responsive input over effects that cause persistent frame drops. The primary goal is 60 FPS for normal desktop interaction and standard animations, with a minimum 30 FPS target for the explicitly documented heavy QEMU workload. If targets are missed, report the measured bottleneck and a remediation plan rather than hiding the regression.
