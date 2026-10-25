# BUG-0025 — Closing a panel painted thirteen frames of nothing

```
Bug ID        : BUG-0025
Severity      : S2 (HIGH) — pure CPU waste on every menu close, and 450 ms of it
                on every Mission Control open
Class         : MOTION / RENDERER
Subsystem     : src/litric-shell/src/anim.rs (ShellAnims::sync, BannerAnims::sync)
First Observed: 2026-10-25, by asking which draw call samples each spring
Status        : FIXED — `cargo test -p litric-shell --lib a_closed_panel`
```

## Symptom

`Shell::tick` treats "a spring moved" as "repaint". Ten springs were stepped, but
only five values were read by any draw call. `views::draw_overlay` matches on
`shell.overlay` and paints the current surface only, and nothing paints a surface
that is no longer current — so a closing panel travelled from 1 to 0 behind
pixels that had already been erased, and each of those frames was a whole-screen
redraw of the frame before it.

## Measured

Test harness (`a_closed_panel_does_not_ask_for_frames_no_surface_paints`): open
Quick Settings, let the reveal run, close it, then tick at 16 ms and count the
frames that report damage while the exit spring is in flight.

| | frames repainted |
|---|---:|
| before | **13** |
| after | **0** |

At the frame costs this tree measured, 13 frames of a 1280×800 desktop is roughly
0.5 s of CPU per menu close on the host, and ~1.5 s at the pre-BUG-0024 frame
cost. Mission Control was worse in kind: `overview` is a `SLOW` (450 ms) spring
with no sampler at all, so *opening* it also spent 450 ms of full-frame redraws
for a transition the screen never showed.

## Which channels were real

Found by searching for a draw-side reader of each field:

| Channel | Read by a draw call? | Where |
|---|---|---|
| `launcher_morph` | yes | `views::draw_dock` (the dock's morph) |
| `quick_settings` | yes, while open | `draw_quick_settings`, `draw_power_menu`, `draw_litro_menu`, `draw_notification_list` |
| `notifications` | yes, while open | `draw_notification_list` |
| `context_menu` | yes, while open | `draw_context_menu` |
| `focus` | yes, always | `draw_windows` (per-window elevation) |
| `launcher` | **no** | — |
| `overview` | **no** | — |
| `switcher` | **no** | — |
| `dialog` | **no** | — |
| `banners` (`banner_reveal`) | **no** | — |

`Shell::show_banner` and `Shell::retire_banner` are also dead (nothing calls
them), and the whole `retiring_overlay` / `retiring_context_menu` /
`retiring_desktop_dialog` snapshot mechanism is written, documented and never
populated or read — see "Open", below.

## Fix

Two policies, both in `ShellAnims::sync`, chosen by whether a draw call samples
the value in that state:

* **Sampled while open** (`quick_settings`, `notifications`, `context_menu`):
  travel on the way in, snap to zero on the way out (`open_or_snap`). The
  opening reveal is a real transition and is kept exactly as it was; the closing
  travel was never drawn, so snapping it changes no pixel.
* **Never sampled** (`launcher`, `overview`, `switcher`, `dialog`): held at their
  end state in both directions (`hold`). This is the same pixels as before with
  no frames spent on them.
* **Banners**: `BannerAnims::sync` keeps a spring only for live ids and puts it at
  its end state; an entry is dropped when the notification is neither active nor
  retiring, instead of lingering until it settles.

`launcher_morph` is deliberately *not* held: the dock is on screen at rest, so
its morph must travel down as well as up.

## Test

`shell::tests::a_closed_panel_does_not_ask_for_frames_no_surface_paints` — asserts
the closed spring has settled on the same tick and that zero further frames
report damage.

`shell::tests::a_spring_settles_within_its_token_budget_at_every_frame_pace` was
repointed from `anims().launcher` (a channel nothing samples, now held) to
`anims().quick_settings`, which carries the observable transition. The assertion
it makes — one token's settle budget means one number of milliseconds at every
frame pace — is unchanged.

## Open — exit transitions are not implemented, and this does not pretend otherwise

An exit animation needs the outgoing surface to still be painted. The fields for
that exist (`Shell::retiring_overlay`, `retiring_context_menu`,
`retiring_desktop_dialog`, `retiring_banners`) and their accessors are public, but
`views` reads none of them and `Shell::set_overlay` / `set_context_menu` /
`set_desktop_dialog` / `retire_banner` / `dismiss_notification` /
`clear_notifications` are dead — every one of the 30 overlay assignments writes
`self.overlay` directly. Wiring them is a change to the draw side first; when it
lands, `open_or_snap` becomes `set_target` in both directions again and the
snapshot fields start being populated.

## Rollback

Revert `open_or_snap` / `hold` in `ShellAnims::sync` and the end-state snapping in
`BannerAnims::sync` to plain `set_target`, and repoint the settle-budget test back
to `anims().launcher`.
