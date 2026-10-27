# BUG-0024 — the details pane's empty state ended the window

```
Bug ID        : BUG-0024
Severity      : S2 (HIGH) — a modal dialog with no visible button: the user is
                asked for a name and cannot confirm it with the pointer
Class         : UI / RENDERER (early return that skips the overlays)
Subsystem     : src/litric-shell/src/views.rs (`draw_files`)
First Observed: 2026-10-27, while writing the regression test for BUG-0023
Status        : FIXED (covered by the New-folder journeys)
```

## Reproducer

```bash
cd src
cargo test -p litric-shell --lib -- journey_new_folder_from_the_files_menu_lands_in_the_open_folder
```

By hand, on a desktop: open Files (the details pane is on by default), click the
empty part of the table so nothing is selected, then **New folder**. A scrim and
the name field appear. There is no **Create** button, and nothing under the
pointer closes the dialog. Pressing Return still creates the folder, which is why
the dialog looked alive in the keyboard journeys and dead in the pointer ones.

With the dispatch fix from BUG-0023 in place and this one still in, the journey
fails here (`docs/evidence/2026-10-27/repro-views-only-reverted.log`):

```
thread 'e2e::journey_new_folder_from_the_files_menu_lands_in_the_open_folder' panicked at
litric-shell/src/e2e.rs:316:10:
the dialog drew its Create button
```

## Root cause

`draw_files` draws the window in parts — places, command bar, table, details pane
— and then, at the end, the two overlays that belong to the window:

```rust
// --- modal dialog over the window ---
draw_files_dialog(canvas, shell, body);

// --- text viewer overlay ---
draw_viewer(canvas, shell, body);
```

The details pane is the only part drawn *inside* the body with a `return` in it:

```rust
let Some(sel) = app.selection() else {
    font::draw(canvas, "Nothing selected", ...);
    return;                     // <-- leaves draw_files, not the details pane
};
```

That `return` was meant to end the details pane. It ends the window. With nothing
selected — which is the state a fresh Files window opens in, and the state right
after a folder is created — both overlays were skipped, so the dialog that
`shell.rs` had opened existed as state only: no panel, no button, no hotspot.

The two symptoms are the same bug: the dialog drew nothing, and therefore its
`Hot::DialogConfirm` hotspot was never registered, so a click could not reach it.
The text viewer had the same hole for the same reason.

## Fix

The empty details state now draws the overlays it used to skip, then returns:

```rust
if app.selection().is_none() {
    font::draw(canvas, "Nothing selected", ...);
    draw_files_dialog(canvas, shell, body);
    draw_viewer(canvas, shell, body);
    return;
}
```

Keeping the early return (rather than restructuring `draw_files` into a details
function) is deliberate: the details pane's own drawing is a good deal longer than
the branch, and the overlays are already idempotent at the end of the function. The
comment in the source says why the two calls are there, because a `return` that
must remember to draw something is the kind of line that gets tidied away.

## Regression tests

`e2e::journey_new_folder_from_the_files_menu_lands_in_the_open_folder` covers it
structurally: the fixture opens Files with the default details pane, sets the
listing to empty so no row is selected, asks for a new folder, and then *touches
the drawn button* rather than calling a handler:

```rust
let ok = s.hotspots().rect_of(Hot::DialogConfirm)
    .expect("the dialog drew its Create button");
click(&mut s, ok.center().0, ok.center().1);
```

A dialog that exists only in state has no `Hot::DialogConfirm` rect, so the test
fails at the `expect` — which is exactly how this defect was found.

## Verification

* `cargo test -p litric-shell --locked` — PASS (see
  `docs/evidence/2026-10-27/files-newfolder.log`).
* With only this half reverted, the journey fails at the `expect` above; with both
  halves present it passes and the folder exists in `/Pictures`.

## Residual risk

`draw_files` still returns early from two other places (the details pane is drawn
in one branch with a selection and one without). They are now symmetrical, but the
structure invites the same mistake: the next overlay added to the window has to be
added to every early exit. Folding the details pane into its own function, the way
`draw_files_dialog` already is, would remove the class rather than the instance;
that is a refactor this repair deliberately did not take on.
