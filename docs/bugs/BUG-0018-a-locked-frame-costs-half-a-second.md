# BUG-0018 — A locked frame costs 0.57 s, against the 33 ms interval it asks for

```
Bug ID        : BUG-0018
Severity      : S2 (HIGH) — the lock screen is the shell's only steady state at
                30 FPS, and it is 17x-24x over its own interval
Class         : PERFORMANCE / RENDERER
Subsystem     : src/litric-shell/src/lock.rs (the surface),
                src/litric-shell/src/views.rs::draw_lock,
                src/litric-shell/src/shell.rs::tick (a locked frame is always damage),
                measured by src/litric-shell/examples/render_bench.rs and
                src/litric-shell/examples/frame_dump.rs
First Observed: 2026-10-12, the first time a locked frame was timed. The lock
                surface itself landed that day, so this is not a regression: it
                is the first measurement of new code
Related        : BUG-0015 (a frame with a window costs 206 ms) is the same
                renderer with the same root cause class — per-frame repainting of
                regions whose content did not change
Status        : OPEN — measured, not fixed
```

## Reproducer

```bash
cd src
cargo run --release -p litric-shell --example render_bench            # 15-run statistics
cargo run --release -p litric-shell --example frame_dump -- ../build/frames
```

## Actual

`render_bench`, 15 runs per line, release, 1280x800, WSL2 development host
(see *Environment* below). The interval a locked shell asks the loop for is
`LifeState::Locked` = **33 ms** (30 FPS), printed by the run as
`lock.interval_ms=33`.

| Measurement | median | mean ± stddev | min | Pixels painted |
|---|---:|---:|---:|---:|
| `lock_frame` (as it opens, shipped look) | **569.664 ms** | 608.179 ± 90.267 ms | 556.913 ms | 17 531 985 |
| `lock_frame_entry` (password field out) | **612.181 ms** | 618.296 ± 27.960 ms | 572.133 ms | 17 792 277 |
| `lock_frame_customize` (customize sheet open) | **804.543 ms** | 820.711 ± 49.188 ms | 762.377 ms | 23 145 487 |
| `lock_frame_still` (motion "Still", no breathing) | **574.577 ms** | 576.755 ± 18.668 ms | 551.922 ms | 17 732 173 |

`frame_dump`, one frame per scene, release, same host:

| Scene | `draw_ms` | `visited_px` | coverage |
|---|---:|---:|---:|
| 11-lock | **560.66** | 16 474 045 | 16.088x |
| 12-lock-password | **594.45** | 16 684 075 | 16.293x |
| 13-lock-customize | **756.81** | 22 006 492 | 21.491x |

The screen is 1 024 000 pixels. A locked frame paints **17x** that, and
**22.6x** with the customize sheet open.

Also in the same run, for context: full desktop frame 185.339 ms median
(`full_frame`), a frame after opening a window 423.675 ms (`open_window_frame`),
a full-screen two-pass blur at r3 / r10 / r30 **92.851 / 91.202 / 91.667 ms**
median — the blur is measured on its own, and it is the largest single cost the
renderer has.

## Expected

* `docs/PERF-budget.md` §4 frame budget: **16.67 ms p99** at 60 Hz, with
  input-to-present p99 ≤ 2 frames and zero missed flips over a 60 s animation.
* `docs/UI-PLAN.md` renderer reality: software canvas @1280x800, frame budget
  **50 ms**, phase P7 accept "frame budget ≤50 ms verified".
* The lock's own contract, in `src/litric-shell/src/shell.rs::tick`: a locked
  frame is always damage and the interval is 33 ms. The surface promises a drift
  at 30 FPS.

Measured: **17.3x** the interval the lock itself asks for (**34.2x** with the
customize sheet open), and 11.6x the relaxed 50 ms software figure. At 570 ms a
locked screen draws **1.75 frames per second** while claiming 30, so the drift
and the breathing clock are not visible as motion at all.

## Why the frame is expensive, from the numbers

Two numbers point at the same place:

1. **A full-screen two-pass blur costs ~92 ms** on this host. The lock surface
   has to blur the backdrop behind everything glass on it: three drifting colour
   fields, the digit glass, the password sheet, the widget row, and (when open)
   the customize sheet. Five to six full-screen-equivalent blurs at ~92 ms each
   is 460-550 ms, which is most of the 570 ms frame.
2. **Coverage is 17x.** The fields are large, they overlap each other, and each
   one repaints its own backdrop plus a soft edge; the customize sheet adds
   102 336 px of pure `shadow` sites on a 7 680 px damage region
   (`partial_sites=13-lock-customize shadow=102336`), which is a 25.3x overdraw
   inside its own damage rectangle.

What is *not* the cause, and this is why the fix is tractable: nothing about the
backdrop changes between locked frames. The wallpaper is static, the shell does
not move, and the only things that move are the drifting fields and the
breathing clock. The expensive work is re-deriving a blur of a picture that is
the same picture it was 33 ms ago.

Direction a fix would take, stated as a hypothesis and not as a result:

* **Cache the blurred backdrop.** Blur the wallpaper once per (radius, size) and
  sample it for every glass element; re-derive only when the wallpaper, the
  radius or the resolution changes. The fields drift *over* the backdrop, so what
  moves is a shape, not the picture behind it.
* **Blur the union of the glass regions once** per frame rather than each
  element's region, when they overlap.
* **Split the costs** the way `frame_dump` already splits them by site
  (`shadow` / `brightest` / `draw` / `text`), so the next measurement says which
  of these three the work went to instead of only the total.

## Environment, and what this does not establish

* Host: WSL2 (Arch) on the development machine, 8 vCPU, 12 GiB, release build
  (`opt-level=2`, `lto=fat`, `overflow-checks=true`), no GPU in the path — this
  renderer is software by design (ADR-0017).
* The absolute milliseconds are this host's. The **ratio** to the 33 ms interval
  and the pixel counts are properties of the code and do not depend on the host.
* Not established: what a locked frame costs on the target hardware, and what it
  costs in the booted image, where the framebuffer write path is real. No boot
  measurement of this surface exists yet.
* Not established: whether the surface is usable *despite* this number. The
  e2e journeys drive it through synthetic input and pass; a person looking at a
  1.75 FPS lock screen has not been observed.
