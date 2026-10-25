# BUG-0024 — A soft shadow was painted twelve times over

```
Bug ID        : BUG-0024
Severity      : S2 (HIGH) — 73–78 % of every window frame's pixel work
Class         : PERFORMANCE / RENDERER
Subsystem     : src/litric-shell/src/canvas.rs (Canvas::soft_shadow)
First Observed: 2026-10-25, by billing frame_dump's per-site pixel split
Status        : FIXED — `cargo test -p litric-shell --lib shadow`
```

## Symptom

[BUG-0015](BUG-0015-a-frame-with-a-window-costs-206ms.md) left one cost
identified and unimplemented: "the real remaining overdraw is the shadow …
collapsible into one pass". This is that pass.

`frame_dump`'s per-site split named the size of it before the change, at
1280×800, one frame per scene:

| Scene | total px visited | of which `shadow=` | share |
|---|---:|---:|---:|
| 02-desktop-start | 5 392 660 | 4 187 722 | **77.7 %** |
| 04-files | 7 212 676 | 5 440 612 | **75.4 %** |
| 05-settings | 6 979 549 | 5 368 612 | **76.9 %** |
| 19-dialog-danger | 8 632 624 | 6 283 876 | **72.8 %** |

`soft_shadow` drew twelve concentric rounded rectangles with
`fill_round_rect`, so a pixel inside a window was blended twelve times and then
covered by the window body. Twelve rings is right — the banding between fewer is
visible — but twelve *passes over the same pixels* is not.

## Fix

One pass. The rings are nested (ring `i` is ring 0 grown by `spread * i / 12` in
both its box and its radius, every ring carrying the same `dy`), so coverage
rises monotonically inward and is exactly 1 for the rings that contain a pixel
whole. Per pixel:

* outside the outermost ring — one distance, no write;
* inside the innermost ring (the window's own area) — one distance and one
  8-bit table lookup that folds the twelve full-coverage blends;
* in the band between the two — a binary search for the innermost fully
  covering ring, a lookup, then a blend per partially covering ring (one or two
  in practice).

The fold table is built from the same per-channel arithmetic `blend` performs for
a black source (`round(d * (1 - a))`), so this is the same bytes, not an
approximation. Ring geometry (`cx`, `cy`, half-extent minus radius, radius) is
hoisted out of the pixel loop into `ShadowRing`, which removes five operations
per ring visit.

The binary search is only sound while the rings nest. They do for every box the
shell draws, but a degenerate one could break it, so the answer is verified
against the ring below it and the twelve-blend loop runs instead when it does not
hold. That path is unreachable in the measured scenes; it exists so a wrong
boundary cannot become a wrong shadow.

## Evidence that the pixels did not change

* `canvas::tests::a_shadow_matches_the_twelve_ring_oracle` — the old loop is kept
  in the test module as the oracle and compared byte-for-byte over six cases,
  including a 3×3 box (corner radius saturates), a 180×6 strip (it saturates in
  one axis only), zero spread (all rings coincident), negative `dy`, and a spread
  below the anti-aliasing ramp (two rings partially covering).
* `canvas::tests::a_clipped_shadow_matches_the_oracle_and_writes_nothing_outside_the_clip`
  — the same comparison with a clip set, which is the partial-frame path.
* All 24 `frame_dump` scene fingerprints are byte-identical before and after
  (`docs/evidence/golden/fingerprints.json` still describes this tree), and the
  PPMs of `02-desktop-start` and `05-settings` are byte-identical files.
* `main::tests::a_partial_redraw_is_pixel_identical_to_a_full_redraw` still
  passes.

## Measured

Same machine, same command, release, 1280×800, software rasteriser, n = 15,
median:

```bash
cargo run --release -p litric-shell --example render_bench
```

| Frame | before | after | change |
|---|---:|---:|---:|
| `full_frame` (two windows) | 115.233 ms | **38.095 ms** | 3.02× faster |
| `pointer_move_frame` | 118.973 ms | **38.416 ms** | 3.10× |
| `open_window_frame` | 248.147 ms | **77.502 ms** | 3.20× |
| `lock_frame_customize` | 102.910 ms | **26.737 ms** | 3.85× |
| `lock_frame` (no shadow) | 8.183 ms | 8.254 ms | unchanged |
| `blur_full_screen_r3_2passes` | 36.435 ms | 36.435 ms | unchanged |
| `present_full` | 4.440 ms | 4.390 ms | unchanged |

Pixel visits, `frame_dump`, whole frame:

| Scene | before | after | |
|---|---:|---:|---|
| 02-desktop-start | 5 392 660 | 1 598 236 | 3.37× fewer |
| 03-windows-overlap | 11 364 375 | 3 388 845 | 3.35× |
| 19-dialog-danger | 8 632 624 | 2 927 302 | 2.95× |
| 13-lock-customize | 6 886 151 | 3 197 713 | 2.15× |

The shadow's own line fell 10.6× (4 187 722 → 393 298 on `02-desktop-start`).

## What is still over budget, and what is not

* A **full frame** is 38 ms against the 16.67 ms gate: still 2.3× over. The
  remaining work is `draw=` (736 020 px, general fills/strokes/text) and
  `glass=` (450 394 px, sampling the cached backdrop per panel pixel).
* A **clipped interaction frame** — what a pointer move or a hover actually
  costs the loop — is now 0.8–2.1 ms for a 160×48 damage region
  (`partial_ms=` in `frame_dump`), inside budget with room.
* None of this was measured on the target image under QEMU, on a GPU backend, or
  on a 120 Hz display. See `docs/PERF-budget.md`.

## Rollback

Revert `Canvas::soft_shadow` and the `ShadowRing` type in `canvas.rs` to the
twelve-ring loop the test module still contains, and delete the two oracle tests.
`twelve_ring_shadow` in the test module is the exact pre-fix code.
