# BUG-0026 — A panel reveal repainted the whole screen

```
Bug ID        : BUG-0026
Severity      : S2 (HIGH) — every menu open is 2x more work than the menu
Class         : MOTION / RENDERER
Subsystem     : src/litric-shell/src/shell.rs (tick, overlay_damage),
                src/litric-shell/src/views.rs (note_panel),
                src/litric-shell/src/anim.rs (Moved), src/litric-shell/src/desktop.rs
First Observed: 2026-10-25, by asking where a moving spring's pixels actually are
Status        : FIXED — `cargo test -p litric-shell --lib every_panel_reveal`
```

## Symptom

`Shell::tick` answered "a panel's spring moved" with `request_full_redraw`. A
120 ms reveal is eight frames at 60 Hz, so opening Quick Settings, the
notification list, the power menu, the Litro menu or a context menu repainted
1 024 000 pixels per frame to move a panel that occupies about 190 000 of them.

The code said so out loud: *"Overlays are large and can move; repainting the whole
screen when one opens or closes is cheaper than tracking their exact extent and
getting it subtly wrong."* The first half of that turned out to be false, and the
second half is a test, not a hope.

## What a reveal actually does

Every panel in `views` reveals the same way: it lays itself out at its settled
rectangle and clips the draw to `panel.h * reveal`, a growing height from its own
top edge. So a reveal can only ever touch the rectangle the panel settles into —
nothing outside it, at any value of the spring. That is what makes the settled
rectangle a safe damage bound, and it is a property of the draw code, not an
assumption in the damage code.

## Fix

The draw side publishes what it painted; `tick` damages that.

| Piece | Where |
|---|---|
| `Shell::begin_panel_frame()` — called once at the top of `views::draw`, so a frame with no panel leaves nothing to damage | `views.rs` |
| `Shell::note_panel(rect)` — unioned by each panel as it draws (top-bar panels, context menu, dialog scrim, overview, switcher, banner stack) | `views.rs` |
| `Moved::panels` — the springs whose pixels are panels, replacing the catch-all `Moved::other` | `anim.rs` |
| panel damage = published rectangle + `panel_reach()` (40 px spread + 14 px offset of the largest dialog shadow, +2 px of anti-aliasing) | `shell.rs` |
| first open of a session still costs one full frame: no rectangle exists yet to trust | `shell.rs` |

`Moved` also grew `windows`, because a focus change is two window frames and
their shadows — `wm.visible()` — and never the desktop.

## Verified

`every_panel_reveal_is_a_partial_frame_that_matches_a_full_redraw`
(`src/litric-shell/src/main.rs`) opens and closes Quick Settings, the notification
list and the launcher **through the hotspot registry, with synthetic pointer
events — the path a person takes** — and, for every frame that reports damage:

1. asserts at most one of them repainted the whole screen, and at least one did
   not;
2. draws that frame from its own damage, then draws the same shell state again
   with a forced full redraw and compares the two canvases **byte for byte**.

It then focuses one of two open windows and holds the focus transition to the same
two rules. A damage rectangle that is too small fails assertion 2 on the first
frame it is wrong.

`render_bench`'s new `panel_reveal` section measures the same reveal through
`Compositor::repaint` (see `docs/PERF-budget.md`): 100 panel frames at
**13.056 ms** median against 10 whole-screen reveal frames at **26.171 ms**, on
the 1280x800 desktop, 198 900 damaged pixels per panel frame. All 24
`frame_dump` scene fingerprints are byte-identical to HEAD — the picture did not
change, only the work.

## Found on the way: the cursor was painted twice

The pixel comparison failed at first with 65 wrong pixels along the top bar, on
a frame whose damage did not include the pointer. `Compositor::paint_cursor`
snapshots what the arrow covers and `move_cursor` and `repaint` both forgot to
give it back before composing — `repaint` dropped the snapshot with
`self.cursor = None`. A damage region that excludes the pointer therefore left
the old arrow in the scene and blended a second copy over its own edge. Nothing
noticed while every frame was a full redraw.

`Compositor::restore_cursor` now runs at the top of `repaint`, before the first
region. It is the same bug class as BUG-0019: state the compositor owns, applied
by a path that assumes it owns the whole scene.

## Found on the way: springs for windows that had closed

`FocusAnims` kept a spring for every window id it had ever seen. A window closed
while its elevation was still gliding left the spring behind, and `step` kept
reporting "windows moved" for the rest of that budget — which damages every
window still on screen. `FocusAnims::forget_closed` (called from `Shell::tick`
with `wm.all()`, so minimized and other-workspace windows keep their elevation)
drops them. Test: `a_spring_for_a_window_that_closed_is_forgotten_not_travelled`.

## Open

An exit transition still does not exist. `Shell::retiring_overlay`,
`retiring_context_menu`, `retiring_desktop_dialog` and `retiring_banners` are
declared, documented and never written or read, and `Shell::set_overlay` /
`set_context_menu` / `set_desktop_dialog` / `retire_banner` /
`dismiss_notification` / `clear_notifications` are dead code (`cargo clippy`
says so). Panels therefore appear by travelling in and disappear by being erased,
which is why `open_or_snap` snaps on the way out (BUG-0025). When the draw side
learns to paint a leaving surface, those channels travel in both directions again
and the exit becomes a partial frame like the entrance.
