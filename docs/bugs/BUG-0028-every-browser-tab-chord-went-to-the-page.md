# BUG-0028 — Every browser tab chord went to the page, and Ctrl+T cycled the theme

```
Bug ID        : BUG-0028
Severity      : S2 (HIGH) — the browser's tab strip could not be driven by the
                keyboard at all, and the one chord every browser user tries first
                changed the desktop theme instead
Class         : UI / INPUT (keys routed past the shell; a chord bound twice)
Subsystem     : src/litric-shell/src/shell.rs (`Shell::engine_takes_key`,
                `Shell::app_key`), src/litric-shell/src/views.rs (`draw_browser`)
First Observed: 2026-10-27, implementing F.08's finding 39 from
                `Research-Deep-all` Part 13
Status        : FIXED (five regression tests)
```

## Why this exists

`engine_takes_key` decides whether the page owns the keyboard:

```rust
if self.browser_page_focused() {
    return true;
}
```

That is the right rule for typing and for a page's own shortcuts. It was applied
to *everything* except Meta and Alt+Tab, so with a browser page focused — which
is the normal state — `Ctrl+T`, `Ctrl+W`, `Ctrl+Tab`, `Ctrl+PageDown` and
`Ctrl+1…9` were all delivered to Chromium. Chromium has no tab of its own to act
on: the strip belongs to the shell, and the broker owns the engine tabs. The
keys therefore did nothing at all.

The one place the shell did see a chord was the window next door: Settings bound
`Ctrl+T` to **cycle theme** (`Shell::app_key`, `com.litric.settings`). The
research corpus recorded that as finding 39 — "the one shortcut a browser user
will try first does something unrelated in the window next door".

The strip's only keyboard path was the generic focus order: two Tab stops per tab
(the tab, then its ✕), no arrow movement and no way to name a tab directly.

## Fix

**The browser's own chords stay with the shell.** `engine_takes_key` now refuses
to hand them to the page:

```rust
if mods.ctrl
    && matches!(
        key,
        Key::Tab | Key::PageUp | Key::PageDown | Key::Char('t' | 'w' | '1'..='9')
    )
{
    return false;
}
```

**One code path per command.** `app_key`'s browser arm calls the same
`activate(Hot::…)` the pointer uses, so the model and the broker cannot be told
different things by a key and a click:

| Key | Action |
|---|---|
| `Ctrl+T` | new tab (chord guarded against Alt and Shift) |
| `Ctrl+W` | close the focused tab; the last tab leaves one fresh tab |
| `Ctrl+Tab`, `Ctrl+Shift+Tab` | next / previous tab, wrapping |
| `Ctrl+PageDown`, `Ctrl+PageUp` | the same step, GNOME's binding |
| `Ctrl+1` … `Ctrl+8` | the tab at that position, if it exists |
| `Ctrl+9` | the last tab |

**The collision is gone.** Settings' theme cycle moved to `Ctrl+Alt+T`, and the
Settings window's own hint line says so. `Ctrl+T` in the browser no longer
touches the theme, and `Ctrl+T` in Settings no longer does anything.

**The active tab has a second channel.** A material fill alone is one channel;
the active tab now also carries a two-pixel accent edge along its bottom, which
survives a theme where both materials read alike.

**Two smaller corrections in the same strip.** The close ✕ is 20 px rather than
15 (a 15 px target is a third of the general 40 px guidance, on the control that
ends a page), and the title is ellipsized by *measured width* instead of a
17-character cap, which mis-measured every CJK title.

## Regression tests

`src/litric-shell/src/shell.rs`, module `browser_tab_keys`:

* `ctrl_t_opens_a_tab_and_ctrl_w_closes_it` — the chord opens a tab on the start
  page, closes it, and closing the last tab leaves one tab rather than zero.
* `ctrl_tab_cycles_and_wraps_in_both_directions` — forward wrap past the last
  tab, backward with Shift, and `Ctrl+PageDown` as the same step.
* `ctrl_digits_choose_a_tab_and_nine_is_the_last` — `Ctrl+3`, `Ctrl+9`, and a
  digit past the strip changing nothing.
* `ctrl_t_is_new_tab_in_the_browser_and_not_a_theme_cycle_in_settings` — the
  collision, asserted from both windows, plus `Ctrl+Alt+T` cycling the theme.
* `a_selected_tab_draws_a_second_state_channel` — pixels, not intent: the active
  tab's rect carries accent pixels and an inactive tab's carries none.

## Verification

* `cargo test --offline --workspace` — 1599 passed, 0 failed
  (`docs/evidence/2026-10-27/tab-keyboard.log`).
* `cargo test --offline -p litric-shell browser_tab_keys` — 5 passed (same log).
* `cargo clippy --offline --workspace --all-targets` — clean.
* `cargo fmt --all -- --check` — clean.
* `ci/verify.sh` — 29 pass, 0 fail, 3 skip (image build, boot, idle metrics need
  mkosi and qemu, which this host does not have).
* Golden desktop frames still pass after the strip change.

## Residual risk

* **The strip's overflow is repaired separately** (BUG-0029): it now scrolls
  with bumpers and reveals a selected tab, so the keyboard model here is no
  longer the only route to a tab whose pixels ran past the window edge.
* No `Ctrl+Shift+T` (reopen a closed tab): the shell keeps no closed-tab stack.
* Arrow-key movement inside a focused strip is not implemented; the arrows still
  belong to the page while it has focus.
* `Ctrl+9` is "last tab", not "ninth tab": with nine or more tabs both readings
  are expected on some platforms, and this follows Chrome.
* Everything above is verified on the host by unit tests. No LitricOS image was
  built or booted for this record.
