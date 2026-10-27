# BUG-0029 — The tab strip drew tabs outside the window, with no way to reach them

```
Bug ID        : BUG-0029
Severity      : S2 (HIGH) — a tab opened past the window edge could not be
                selected, closed or even read: the strip had no overflow model
Class         : UI / LAYOUT (a fixed-width row with no scroll, no clip, no cap)
Subsystem     : src/litric-shell/src/views.rs (`draw_browser`),
                src/litric-shell/src/apps.rs (`BrowserApp`),
                src/litric-shell/src/shell.rs (`Hot::BrowserStripLeft/Right`)
First Observed: 2026-10-27, F.08 finding 38 in `Research-Deep-all` Part 13
Status        : FIXED (four regression tests)
```

## Why this exists

`draw_browser` laid the strip out as a plain accumulation:

```rust
let mut tx = body.x + t.px(4.0);
for tab in &app.tabs {
    let w = t.px(150.0);
    let r = Rect::new(tx, body.y + t.px(2.0), w, strip_h - t.px(4.0));
    ...
    tx += w + t.px(2.0);
}
```

152 px per tab, no check against `body.w`, no clip of its own, and the `+`
button drawn after the last tab. Past roughly `body.w / 152` tabs the strip ran
off the window: the tabs to the right, and the `+` button, were drawn outside the
window body — at 1280 px, twelve tabs put the last one at 1,836 px. Nothing
scrolled, so the only pointer path to those tabs was the part of their rectangle
that happened to overlap another control. The keyboard model added in BUG-0028
could *name* a tab past the edge, but that is a mitigation: a browser whose tabs
cannot be clicked is still broken.

Both platform families name a mechanism for exactly this case:

* Windows (`TabView`): **scroll bumpers** move the strip when tabs no longer fit.
* GNOME/libadwaita (`AdwTabBar`): the tab bar **scrolls**, with pinned tabs
  outside the scroll area.

The shell had neither.

## Fix

**The strip scrolls.** `BrowserApp` gained `strip_scroll` (device px from the
first tab's left edge) and `reveal_active` (set only by selection, never by a
bumper click). The view owns the geometry and clamps the offset to
`content_w - area.w` before laying anything out, so an over-scroll is corrected
in the same frame it is asked for — `strip_scroll` can never describe a strip
position that does not exist.

**Two bumpers, only while the strip overflows.** When the content fits, the
strip is laid out exactly as before: no bumper, no reserved width, first tab at
the body's 4 px inset. When it overflows, 20 px bumpers take the left and right
edges, and the tab area between them is clipped with `Canvas::push_clip`, so a
tab half-scrolled out is drawn clipped rather than over the omnibox.

**What is drawn is what is clickable.** A tab whose rectangle is wholly outside
the strip is not registered as a hotspot at all; a close ✕ is only registered
when it is wholly on screen; the `+` button likewise. A bumper is pushed after
the tabs, so at the edges the bumper wins the hit test instead of the tab
underneath it.

**A selected tab is revealed.** Selecting a tab — by pointer on a visible tab,
`Ctrl+1…9`, `Ctrl+Tab`, or a new tab — sets `reveal_active`; the next frame
scrolls the minimum amount that puts the tab fully inside the strip. Re-selecting
the focused tab does not move the strip, and a bumper click is never undone by a
reveal, because the pointer can only click a tab that is already visible.

**The offset is dropped when it can no longer mean anything.** Closing back to
one tab zeroes `strip_scroll`: one tab always fits, and an offset left over from
the wider strip would have hidden that tab behind the left bumper.

The bumpers are chevrons drawn with two `stroke_line` calls, not arrow glyphs:
the shell's bitmap face has no arrow characters and an unknown character draws a
box, which is not an affordance.

## Regression tests

`src/litric-shell/src/shell.rs`, module `browser_tab_strip`:

* `a_strip_that_fits_has_no_bumpers` — one tab: no bumper hotspots, offset 0,
  the tab at the body's inset, and the `+` button drawn.
* `the_right_bumper_reaches_a_tab_past_the_window_edge` — twelve tabs: the last
  tab is not a hotspot at offset 0; pressing the right bumper until it disappears
  reaches the end, the last tab is inside the strip (right of it is the bumper,
  not the window edge), the `+` button returns, the left bumper appears, and a
  5,000 px over-scroll is clamped to the same end offset.
* `selecting_a_tab_past_the_edge_reveals_it` — `Ctrl+9` from the first tab
  scrolls the strip and the revealed tab answers a click at its own corner.
* `closing_back_to_one_tab_resets_the_strip_offset` — scroll to the end, close
  eleven tabs, and the offset is zero with no bumpers and the tab drawn.

## Verification

* `cargo test --offline -p litric-shell browser_tab_strip` — 4 passed.
* `cargo test --offline -p litric-shell browser_tab` — 10 passed (the six
  BUG-0028 tests still pass). Both runs are in
  `docs/evidence/2026-10-27/tab-strip-overflow.log`.
* Golden desktop frames re-dumped (`frame_dump --fonts build/fonts/noto`) and
  compared: every frame identical or within threshold.
* `cargo fmt --all -- --check` — clean.
* `cargo test --offline --workspace` — 1603 passed, 0 failed.
* `cargo clippy --offline --workspace --all-targets` — no warnings.
* `ci/verify.sh` — report in `docs/evidence/2026-10-27/verify-after-interaction-repair.log`.

## Residual risk

* **No drag reordering, pinning, muting or tear-out.** Windows and GNOME both
  document tab states and reordering beyond selection; the strip still has only
  `active`, a URL, a title and an engine id.
* **Bumper steps are three tabs**, not a full page measured from the strip's
  width. A step larger than the strip would skip tabs entirely, and a step of
  one tab makes twenty tabs twenty clicks; three is a compromise, not a
  platform-specified number.
* **No `Ctrl+Shift+T`** (reopen a closed tab): the shell keeps no closed-tab
  stack.
* **No wheel-to-scroll on the strip.** Bumpers and the keyboard are the two
  paths; the wheel over the strip still goes to the page, because the strip is
  not a hotspot bound to the wheel.
* **The tab context menu still has two items** (Reload, Close tab); GNOME's
  Duplicate/Pin/Mute/Move to New Window are not implemented.
* Everything above is verified on the host by unit tests and frame dumps. No
  LitricOS image was built or booted for this record.
