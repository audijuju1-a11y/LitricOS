# BUG-0015 — A frame with a window costs 206 ms, against a 16.67 ms budget

```
Bug ID        : BUG-0015
Severity      : S2 (HIGH) — every motion phase and every interaction is gated on
                this number, and the number is 4x-12x over the stated budget
Class         : PERFORMANCE / RENDERER
Subsystem     : src/litric-shell/src/main.rs (render loop), src/litric-shell/examples/render_bench.rs,
                src/litric-shell/examples/frame_dump.rs
First Observed: 2026-10-09, first frame timings ever measured on the project's own
                development host (release build, offline, 1280x800)
Status        : PRICED (host) — 6.83 ms on this commit; the image number is unmeasured
```

## Reproducer

```bash
cd src
cargo run --release -p litric-shell --example render_bench      # 15-run medians
cargo run --release -p litric-shell --example frame_dump -- ../build/frames   # one-shot per scene
```

## Actual

`render_bench`, medians over 15 runs, release, 1280x800:

| Measurement | p50 | min | Pixels painted |
|---|---:|---:|---:|
| Full frame, two windows open (`full_frame`) | **206.359 ms** | 196.926 ms | 6 185 255 |
| Frame after one pointer move | **210.418 ms** | 202.597 ms | 5 263 561 |
| Frame after opening a window | **498.565 ms** | 463.844 ms | 13 032 905 |
| Blur, full screen, 2 passes, r3 / r10 / r30 | 99.5 / 100.7 / 95.3 ms | — | — |
| Present full frame (`write_all`, 4 096 000 bytes) | 6.961 ms | 5.446 ms | — |
| Present one damage band (163 840 bytes) | 0.175 ms | 0.164 ms | — |

`frame_dump`, one frame per scene, release:

| Scene | draw_ms |
|---|---:|
| 01-desktop-idle (no windows) | **8.79** |
| 02-desktop-start (Activity + Environments) | 212.63 |
| 03-windows-overlap (three windows) | 466.05 |
| 04-files | 264.22 |
| 05-settings | 263.25 |
| 06-browser | 370.83 |
| 07-terminal | 195.32 |
| 08-trash | 150.21 |
| 09-defender | 168.42 |
| 10-activity | 116.60 |

The screen is 1 024 000 pixels; a full frame paints 6.0x that, because glass
panels solve their own backdrop and shadow by repainting the region behind them.

## Expected

* `PERF-budget` frame budget table: 60 Hz = **16.67 ms p99**, plus
  **input-to-present p99 ≤ 2 frames** and **zero missed flips over a 60 s
  animation**.
* `UI-PLAN` renderer reality: "software canvas @1280×800, frame budget **50ms**",
  and phase P7 accept: "frame budget ≤50ms verified".
* `DESIGN-SYSTEM-v3/05-ROADMAP` release condition includes "frame budget ผ่าน".

Measured: 206 ms is **12.4x** the 16.67 ms gate and **4.1x** the relaxed software
figure; a frame that opens a window is **10.0x** the relaxed figure. At 206 ms the
shell draws **4.8 fps**; the motion spec (menu 120 ms, window 220 ms, banner
160 ms) would show one or two frames of any animation.

## Why this is not a warm-up artefact

Two independent harnesses agree: `render_bench` warms up and reports the median of
15 runs (206.359 ms), and `frame_dump` draws each scene once in a fresh process
(212.63 ms for the same scene). The idle desktop in the same one-shot run costs
8.79 ms, so the cost arrives with the windows, not with process start.

## The damage path, measured (question answered 2026-10-09)

`main.rs`'s `render()` delegates to
`session.compositor.repaint(&mut session.shell, full, damage)`, and `repaint`
**does** clip: it plans the damage regions, sets the clip to each in turn, and
draws inside it. `render_bench` measured the same loop by calling `views::draw`
**unclipped**, so its 210 ms pointer-move figure was never the loop's cost.

`frame_dump` now measures one damage-clipped frame through `repaint` itself, for
a 160x48 region (7 680 px) at 1280x800:

| Scene (one clipped frame) | partial_ms | pixels the loops were given | overdraw |
|---|---:|---:|---:|
| 01-desktop-idle | 1.52 | 44 368 | 5.8x |
| 10-activity | 4.98 | 207 568 | 27.0x |
| 07-terminal | 5.42 | 300 368 | 39.1x |
| 04-files | 26.40 | 503 252 | 65.5x |
| 05-settings | 26.45 | 538 980 | 70.2x |
| 06-browser | 25.16 | 712 288 | 92.7x |
| 03-windows-overlap | 27.03 | 755 391 | 98.4x |

The writes are clipped -- which is why the desktop does not smear -- but
`pixels_visited` counts "the area of every pixel range a drawing, blending or
sampling loop **was given**, whether or not it then changed the pixel". A
7 680-pixel region hands the loops up to 755 391 pixels to walk: the clip stops
the write, not the loop. That overdraw, not the full frame, is the cost of an
interaction, and with a window open it lands at **25-27 ms against a 16.67 ms
gate** for one small region.

So the numbers separate cleanly:

* **Full redraw** (window open, theme change, wallpaper damage): 206 ms measured,
  and it must paint everything either way. This is the cost that must come down.
* **One interaction** (a pointer move, a hover): 1.5 ms idle, 25-27 ms with a
  window open. Under the 50 ms software figure, over the 60 Hz gate.

## Run-to-run variance on a shared host

One-shot full-frame numbers moved between runs of the same binary --
02-desktop-start 212 ms, then 335 ms, then 692 ms -- on a development machine
that was also running the test suites. Read every full-frame figure here as a
range and compare only medians taken in one session; the partial-frame figures
were stable (1.5 / 5.0 / 5.4 / 25-27 ms across runs).

## Where the time goes (measured, not guessed)

* Blur alone, full screen, two passes: ~100 ms, and **independent of radius**
  (r3 99.5 ms, r30 95.3 ms). Cost scales with bytes touched, not with the blur
  kernel, which points at scratch-buffer traffic rather than arithmetic.
* So at least ~48% of a 206 ms full frame is the blur/glass path; the remainder is
  anti-aliased geometry and text.
* An idle desktop with no windows draws in 8.79 ms, so the base cost of wallpaper,
  menubar and dock is inside budget. The defect is the per-window glass path.

## Fix direction (not implemented here)

1. **Bound the pixel loops by the clip, not just the writes.** Every frame already
  reports `pixels_visited`; the gate is that a 7 680-pixel damage region must not
  hand the loops 755 391 pixels. That means intersecting spans with the clip in
  the fill/blit/blur paths, which is a change to `canvas.rs` rather than to the
  composition.
2. **Stop re-solving glass the same way every frame.** Cache the blurred backdrop
  per panel and invalidate on change of the region behind it, instead of blurring
  a wallpaper that did not move.
3. **Consider dropping live blur for unfocused surfaces**, where the token alpha
  path is visually equivalent at this size and costs a fraction.
4. **Add a frame-budget test to the suite**, in the same shape as the memory
  checker: a claim with a measurement behind it, not a document with a table.

## The 98x, taken apart (measured 2026-10-10)

Item 1 of the fix direction is done, and the measurement it was based on was
wrong. `pixels_visited` was billing **the rectangle a loop was given**, and
`brightest_sampled` reads one pixel in sixteen: the search that decides a
panel's glass alpha walks a 4x4 grid and the counter charged it for every pixel
of the panel. That single line accounts for 617 319 of the 755 391.

The counter now bills the samples a sampled loop really takes, and each range is
tagged with the kind of loop that asked for it, so a partial frame reports where
its pixels went (`partial_sites=` in `frame_dump`). Same scenes, same session,
release, 1280x800, 160x48 (7 680 px) damage region:

| Scene | before px | after px | drop | before ms | after ms | after split |
|---|---:|---:|---:|---:|---:|---|
| 01-desktop-idle | 44 368 | 10 768 | 4.1x | 0.59 | 0.62 | draw 8 528 · brightest 2 240 |
| 02-desktop-start | 314 252 | 27 663 | 11.4x | 4.43 | 4.45 | brightest 19 135 · draw 8 528 |
| 03-windows-overlap | **755 391** | **138 072** | 5.5x | 12.53 | 13.15 | shadow 69 270 · brightest 41 525 · draw 25 416 · text 1 861 |
| 04-files | 503 252 | 113 402 | 4.4x | 8.64 | 8.83 | shadow 68 880 · brightest 25 990 · draw 18 532 |
| 05-settings | 543 570 | 158 970 | 3.4x | 11.14 | 10.94 | shadow 100 368 · draw 28 372 · brightest 25 640 · text 4 590 |
| 06-browser | 712 288 | 172 438 | 4.1x | 11.92 | 15.15 | shadow 102 336 · brightest 35 990 · draw 34 112 |
| 07-terminal | 300 368 | 26 768 | 11.2x | 4.83 | 4.41 | brightest 18 240 · draw 8 528 |
| 08-trash | 257 168 | 24 068 | 10.7x | 4.20 | 4.60 | brightest 15 540 · draw 8 528 |
| 09-defender | 279 568 | 25 468 | 11.0x | 3.92 | 3.65 | brightest 16 940 · draw 8 528 |
| 10-activity | 207 568 | 20 968 | 9.9x | 3.14 | 2.83 | brightest 12 440 · draw 8 528 |
| **total** | **3 917 793** | **718 585** | **5.45x** | | | |

Raw runs: `build/partial-before.txt`, `build/partial-after.txt`. Every scene's
fingerprint is **byte-identical** before and after (`frame_dump`'s `scene=` lines),
so this changed what a frame reports, not what it draws.

### What the split says, including where the original claim was wrong

* **The `93%` of the 98x that was not real work.** The worst-pixel search is the
  largest single number in a partial frame and it is one read in sixteen; a
  640x480 panel in the damage path reported 307 200 visited pixels for 19 200
  reads. The gate the fix direction set -- "a 7 680-pixel damage region must not
  hand the loops 755 391 pixels" -- is met at 138 072, but 617 319 of the old
  figure were never walked and the time never moved (12.5 ms -> 13.2 ms is
  run-to-run variance, not a regression).
* **`blur=0` and `glass=0` in every compositor frame.** `Compositor::new` attaches
  a pre-blurred backdrop, so `glass_backdrop` takes its sampling branch and no
  live blur runs during a repaint at all. The blur was never in this number; the
  "blur loops" in the table above are the wall-clock blur figures from
  `render_bench`, a different path.
* **The glass path is now clip-aware anyway** (`glass_backdrop` takes its working
  region from the clip grown by the kernel's reach, `2 * blur + 2`), so the live
  path -- a first frame, or a canvas with no backdrop -- pays the damage region
  instead of the panel, and pays nothing at all when the clip misses the panel.
  `a_partial_glass_blur_paints_what_the_full_one_painted` holds that to one bit:
  the clipped panel and the unclipped panel leave identical pixels.
* **The real remaining overdraw is the shadow.** `soft_shadow` draws twelve
  concentric rounded rectangles, so a damage region covered by a panel is
  composited twelve times: 69 270 of scene 03's 138 072 pixels and 100 368 of
  scene 05's are shadow rings. That is genuine layered compositing, not a loop
  walking somewhere it cannot write, and it is now the largest line in the bill.

### Next, with a measurement to aim at

The shadow is the only large real cost left. Its twelve rings differ only in how
much they are grown, so the eleven rings that fully cover a pixel contribute a
fixed alpha sequence -- collapsible into one pass with a 256-entry per-channel
lookup that is bit-identical to the twelve blends. That is a `canvas.rs` change
under the test above, and it is the next thing worth measuring.

## What is not claimed here

* These are **host** numbers, release build, on the development machine's CPU. The
  target image runs the same rasteriser on QEMU vCPUs; the image number still has
  to be measured, and `PERF-budget` already says the shell logs a warning on every
  over-budget frame -- that counter on a booted machine is the release evidence.
* The full-frame cost (206 ms) is **not** fixed by this change. Nothing above
  reduces the pixels a full redraw paints; the pixel-loop item was about what a
  clipped frame is handed, and the full frame remains the number that must come
  down.
* The shadow optimisation above is **not implemented**, only identified and
  attributed by measurement.

## Update 2026-10-24 (branch `redesign/shell-v4`)

Done, with a measurement and a test behind each:

* `Canvas::soft_shadow` composes its twelve rings in one pass and, when a glass
  backdrop is attached, skips the pixels under its own panel. Three-window frame:
  shadow 8.77 M -> 0.80 M pixel visits, total 11.36 M -> 3.39 M. Tests:
  `the_one_pass_shadow_equals_the_twelve_ring_shadow`,
  `a_shadow_under_an_opaque_panel_with_a_backdrop_leaves_the_same_frame`.
* `blend` / `with_alpha` round with an integer step instead of `f32::round`
  (a library call on this target); `fill_round_rect` skips the distance in the
  fully-covered core. Tests: `integer_rounding_blends_exactly_as_float_rounding_did`,
  `the_filled_core_skips_the_distance_and_changes_no_pixel`.
* All 24 `frame_dump` scenes are pixel-identical before and after.

`render_bench`, same host, old and new builds interleaved 3x, minimum of the
medians: full frame 133.7 -> 92.4 ms, pointer move 132.9 -> 95.8 ms, open window
288.1 -> 206.0 ms.

**Not fixed.** A full frame is still ~92 ms against the 16.67 ms gate; this is a
host figure, not a QEMU one. Remaining from the callgrind profile: the glass
backdrop's `box_blur_pass`, per-pixel float `blend`, and
`stroke_round_rect` walking the whole bounding box for an outline.

## Update 2026-10-29 (commit `ce08ee8`): re-measured, and the number is not 206 ms

`render_bench` rebuilt from this commit on this host, `RUNS = 15` per scene,
release, 1280x800. Two consecutive runs; the table is run 1. Run 2 moved the
same rows by 3-8 %: `full_frame` 6.614 ms, `pointer_move_frame` 6.457 ms,
`open_window_frame` 11.669 ms, `lock_frame` 4.326 ms. That spread is stated here
because it is the same order as the deltas discussed below, and only a
consistent direction across both runs is treated as a result.

| Scene | p50 | min | mean | sd |
|---|---:|---:|---:|---:|
| Full frame, two windows (`full_frame`) | **6.832 ms** | 6.696 | 6.853 | 0.119 |
| Frame after one pointer move | **7.034 ms** | — | — | — |
| Frame after opening a window | **11.980 ms** | — | — | — |
| Lock frame | 4.423 ms | — | — | — |
| Lock, password field out | 4.352 ms | — | — | — |
| Lock, customize sheet | 7.623 ms | — | — | — |
| Full frame, live-blur fallback | 29.936 ms | — | — | — |
| Present full frame (`write_all`) | 3.943 ms | — | — | — |
| Present one damage band | 0.151 ms | — | — | — |
| Blur, full screen, r3 / r10 / r30, 2 passes | 35.157 / 37.192 / 37.721 ms | — | — | — |

The same binary, with the call timers on for one frame, splits the 15.182 ms it
could attribute: `stroke_line` 4.234 ms (27.9 %, 2,325 calls), `soft_shadow`
3.993 ms (26.3 %, 3 calls), `stroke_line_attrib` 1.797 ms, `stroke_round_rect`
1.686 ms (7 calls), `fill_round_rect` 1.227 ms (358,198 px at 3.42 ns/px),
`glass_backdrop` 0.534 ms. The profile's own diagnostic pass is inside the
`stroke_line` figure and is measured separately as `stroke_line_attrib`.

**What this means, stated narrowly.** On this host and this commit the two-window
full frame is 6.83 ms, which is **2.44x inside** the 16.67 ms gate and 7.3x
inside the 50 ms software figure; the open-window frame at 11.98 ms is also
inside 16.67 ms. The 206.359 ms and 185.3 ms figures recorded above were taken on
older builds (the 2026-10-24 update already reported 92.4 ms after the one-pass
shadow and integer-blend work), and neither reproduces here: a full frame is now
6.83 ms.

**What this does not mean.** It is not the release number. The target image runs
this rasteriser on QEMU vCPUs, no image was built in this environment (mkosi and
QEMU are absent), and the earlier rows of this record show the same binary moving
between 212 ms and 692 ms on a loaded host. The gate is a QEMU-frame claim on a
booted desktop (`litric-shell.status` over-budget counter), and that measurement
still does not exist. The bug therefore moves from **OPEN, measured, not fixed**
to **PRICED on the host**: the host number is known, the image number is not, and
this record does not claim the gate is passed.

### The change that was tried, measured, and reverted

The remaining candidate named above was the per-row walk: a capsule paints only
where its distance is under `half + 0.5`, yet `stroke_line` walks its whole
inflated box. A row-clamped version (the segment's x-window for that row, grown
by the pen, with the inflated box kept as the fallback for non-finite input) was
implemented and proved **pixel-identical** to the old loop over the existing
144-case matrix (`the_stroke_line_equals_the_per_pixel_distance`, including NaN,
fractional geometry, thick pens and each of four clips) plus the other 56 canvas
tests. It was then measured with the identical command and n=15:

| Scene | before | after | delta |
|---|---:|---:|---:|
| `full_frame` | 6.832 ms | 6.817 ms | -0.2 % |
| `pointer_move_frame` | 7.034 ms | 7.156 ms | **+1.7 %** |
| `open_window_frame` | 11.980 ms | 12.081 ms | **+0.8 %** |
| `lock_frame` | 4.423 ms | 4.557 ms | **+3.0 %** |
| `full_frame_live_blur` | 29.936 ms | 28.504 ms | -4.8 % |

No repeatable gain, two rows worse: the scene's strokes are short and mostly
axis-aligned, so the clamp saves a few skipped pixels per call and pays per-row
arithmetic for them. It was reverted rather than shipped as an unmeasured
"improvement". The conclusion worth keeping is the attribution, not the code:
`stroke_line`'s cost is per-call work and blending, not the corner pixels of its
bounding box.

A wall-clock budget test was also considered and **not** added: on a shared host
the same binary moves by 3x between runs (this record's own variance section),
so a millisecond assertion in the suite would be a flake generator, not a gate.
The golden-frame check already holds the pixels; the frame-time claim stays where
the evidence can be reproducible — a booted image.
