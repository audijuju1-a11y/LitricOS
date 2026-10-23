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
Status        : **FIXED 2026-10-15** — the frame that opens a locked session fell
                from 293.7 ms to 35.8 ms (8.2x) and paints 1.91M pixels instead
                of 10.43M, on the host and build the numbers below were taken
                on; see "Fix" and "Verification". The customize sheet is 2.6x
                and still over budget, which "Remaining work" keeps open
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

## What the split showed, and the four changes

The hypothesis above named three directions; the per-site counters
(`sites_per_frame` on every `lock_frame` line, added by this fix) named a fourth,
and it was the largest one.

| Step | `lock_frame` median | Pixels per frame | Where the work was |
|---|---:|---:|---|
| As found | 293.7 ms | 10 426 249 | `text=5 618 264`, `glass=431 148`, `draw=4 351 301` |
| Text layout collapsed | 148.2 ms | 5 123 110 | `text=336 624` |
| Backdrop cached | 34.4 ms | 2 933 744 | `glass` gone |
| Wallpaper restore skipped | **35.8 ms** | **1 909 744** | `draw=1 571 118`, `text=336 624` |

The step from 148 ms to 34 ms is what the split was for: **5.62M of the 10.43M
pixels were text** — more than all the drawing put together — and almost none of
it was glyphs. The clock and the entry sheet were re-running the whole line
breaker, shaper and rasteriser per frame per element; laying each string out once
took the same pixels from 5.62M to 0.34M with no change to any glyph. This is the
finding that replaces the blur as the prime suspect: a full-screen two-pass blur
is ~92 ms, but the surface was not blurring the screen five times, it was shaping
the same sentences sixty times.

Then, in the order they mattered:

1. **The text layout is done once per string.** Words are shaped, measured and
   cached by (text, face, size) instead of re-laid-out on every visit.
2. **The glass glow is one bloom, not a stack.** The layered radial glows that
   made up the halo were collapsed into a single pass (294 ms → 148 ms before
   any caching).
3. **The blurred backdrop is cached** — build once per (geometry, radius, phase
   key), then blit. Two things had to be right for this to hold: touching regions
   are merged before blurring, because two overlapping blurs re-blurred in place
   do not equal one blur of their union, and the key is quantised coarsely
   (~1 px of field drift), which is invisible under the 8-bit wash the fields sit
   on. Without the merge a warm frame differed from a cold one by a few levels,
   which is exactly the kind of drift that only shows up as a flicker.
4. **A locked frame restores no wallpaper.** The lock lays down an opaque
   background over every pixel it visits, so the copy of the wallpaper the
   compositor makes under it could not be seen. `Compositor::repaint` skips it
   while locked, `render_bench`'s `frame()` mirrors the skip so the benchmark
   still models the loop, and the frames are byte-identical without it —
   `frame_dump`'s fingerprints for 11-lock, 12-lock-password and
   13-lock-customize are unchanged (0x367c7dc0b89dca35, 0x3ba8b424d52ac4f9,
   0xb57ffe3c0e3e5b5d).## Verification, before and after

`cargo +stable-x86_64-pc-windows-gnu run --release --example render_bench`,
15 runs per line, 1280x800, medians. The interval is `lock.interval_ms=33`.

| Scene | Before | After | Speedup | Pixels before | Pixels after |
|---|---:|---:|---:|---:|---:|
| `lock_frame` | 293.727 ms | **35.819 ms** | 8.2x | 10 426 249 | **1 909 744** |
| `lock_frame_entry` | 311.362 ms | **37.983 ms** | 8.2x | 10 692 758 | 2 009 989 |
| `lock_frame_still` | 306.114 ms | **36.843 ms** | 8.3x | 10 607 884 | 2 001 669 |
| `lock_frame_customize` | 477.656 ms | **185.718 ms** | 2.6x | 16 011 499 | 7 270 677 |

"Before" is the same host and build, after only the per-site counters were added
(the counters add no drawing, so they cannot change the frame). The 569.664 ms
figure in *Actual* above is the WSL2 host, and is left as the number the bug was
opened with.

| Check | Result |
|---|---|
| `cargo test --workspace` (Windows GNU, this host) | **PASS** — 1095 passed, 0 failed; the `cfg(unix)` suites are skipped here as always |
| Lock frames, byte for byte | **IDENTICAL** — `frame_dump` fingerprints for the three lock scenes are unchanged by the wallpaper-restore skip (values listed in the fix above) |
| The interval the surface asks for (33 ms) | **NOT MET, but 8.2x closer** — 35.8-38.0 ms is 1.08-1.15x over on this host, against 8.9x before. The customize sheet at 185.7 ms is 5.6x over |

## Remaining work

* **The customize sheet.** 185.7 ms, and 4.06M of its 7.27M pixels are `shadow`:
  one rectangular soft shadow on a 7 680-pixel damage region, i.e. 22x overdraw
  inside its own damage rectangle. It is the last large flat cost on the surface
  and the only lock scene still far outside its interval.
* **The last 3-6 ms.** A locked frame still visits 1.91M pixels for a 1 024 000
  pixel screen: the lock's own background blit is 1.02M of that, and the panels,
  digit glass and clock are the rest. Either the background blit becomes a
  damage-region blit, or the interval moves; the drift is slow enough that 40 ms
  would be honest, but it is not what `LifeState::Locked` claims today.
* **No display has shown this surface yet.** Every number here is a software
  raster on a development host. The lock screen as a person sees it — at 30 FPS,
  drifting — has still never been observed on a screen.

## Environment, and what this does not establish

* Host: WSL2 (Arch) on the development machine, 8 vCPU, 12 GiB, release build
  (`opt-level=2`, `lto=fat`, `overflow-checks=true`), no GPU in the path — this
  renderer is software by design (ADR-0017).
* The fix's before/after table is a second host: Windows 11, `stable-x86_64-pc-
  windows-gnu`, the same release profile. The two hosts disagree in absolute
  terms by roughly 2x (569.664 ms there and 293.727 ms here for the same frame),
  which is why the ratio and the pixel counts are what this record leans on.
* The absolute milliseconds are this host's. The **ratio** to the 33 ms interval
  and the pixel counts are properties of the code and do not depend on the host.
* Not established: what a locked frame costs on the target hardware, and what it
  costs in the booted image, where the framebuffer write path is real. No boot
  measurement of this surface exists yet.
* Not established: whether the surface is usable *despite* this number. The e2e
  journeys drive it through synthetic input and pass; a person looking at a
  1.75 FPS lock screen has not been observed.
