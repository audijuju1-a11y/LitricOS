# BUG-0023 — "New folder" read a menu the click had already closed

```
Bug ID        : BUG-0023
Severity      : S2 (HIGH) — silent data placement: the folder was created, in
                the wrong place, with no message to say so
Class         : UI / DISPATCH (stale state read after the click)
Subsystem     : src/litric-shell/src/shell.rs (`perform_menu`,
                `set_context_menu`, `new_folder_from_menu`)
First Observed: 2026-10-27, recording chapter F of the research corpus: the
                dispatch was read against the menu's lifetime to write F.13 down
Status        : FIXED (three regression tests)
```

## Reproducer

```bash
cd src
cargo test -p litric-shell --lib -- journey_files_context_menu_creates_its_folder_in_the_open_folder
```

## Actual

Open Files, go to `/Pictures`, right-click the table, choose **New folder**, type
`Poster`, press **Create**. The folder appears in `~/Documents`, not in
`~/Pictures`. The toolbar's **New folder** button and the same row in the
places-pane menu do the same thing. `Ctrl+Shift+N` is the only path that creates
the folder in the folder on screen — and it was the reason nobody noticed: it is
the shortcut most people reach for, and it takes a different code path.

Before the fix, the journey that clicks the row and checks the dialog state
fails like this (captured in `docs/evidence/2026-10-27/repro-both-reverted.log`):

```
thread 'e2e::journey_new_folder_from_the_files_menu_lands_in_the_open_folder' panicked at
litric-shell/src/e2e.rs:297:5:
the Files dialog must own the row: None
```

`None` is the Files dialog. The desktop's dialog opened instead — and the desktop's
dialog writes `~/Documents`.

## Root cause

`perform_menu` decided where the folder belongs by asking which surface the open
context menu came from:

```rust
MA::NewFolder => {
    let in_files = self.context_menu.as_ref().is_some_and(|m| matches!(
        m.surface,
        MenuSurface::Files | MenuSurface::FilesRow
    ));
    if in_files { self.files.ask_new_folder(); }
    else { self.set_desktop_dialog(Some(DesktopDialog::NewFolder)); }
}
```

But the click that reaches this code has already closed the menu. `Hot::MenuRow`
calls `set_context_menu(None)` before it dispatches the row's action, and
`set_context_menu` keeps the closed menu as `retiring_context_menu` — the snapshot
that draws the menu's exit — so `self.context_menu` is `None` by the time
`perform_menu` runs. `is_some_and` on `None` is `false`, every time, for every
surface. The branch that opened the Files dialog was unreachable.

The two facts the row needed — *which surface was clicked* and *which menu is
live* — are different facts. Only the second one was read.

## Fix

The closed menu is still there, one field over, with its `surface` intact. The row
now reads the live menu or, when there is none, the retiring snapshot:

```rust
let from_files = match self.context_menu.as_ref().or(self.retiring_context_menu.as_ref()) {
    Some(menu) => matches!(menu.surface, MenuSurface::Files | MenuSurface::FilesRow),
    None => self.files_has_focus(),
};
```

The `None` arm is the Files toolbar's **New folder**, which is not a menu at all:
it is the same `Hot::MenuRow` hotspot, drawn in the command bar, with no menu
behind it. For that one the answer is the window in front.

Reading the surface rather than the focus is what keeps the desktop honest: with
Files in front, right-clicking the wallpaper and choosing **New folder** must
still create the folder on the desktop, and a focus-only fix would have sent it
into Files' current folder. `journey_desktop_menu_keeps_its_folder_on_the_desktop_with_files_in_front`
is the test that says so.

## Regression tests

* `e2e::journey_files_context_menu_creates_its_folder_in_the_open_folder` — the
  defect: right-click in Files, choose the row, create `Poster`, and assert it is
  in `/Pictures` and not in `/Documents`. Fails on the old code at the dialog
  assertion.
* `e2e::journey_new_folder_from_the_files_menu_lands_in_the_open_folder` — the
  toolbar path, which has no menu behind it, plus the confirmation flow through
  the drawn button.
* `e2e::journey_desktop_menu_keeps_its_folder_on_the_desktop_with_files_in_front`
  — the over-correction guard: the desktop's row stays the desktop's while Files
  is focused.

## Verification

* `cargo test -p litric-shell --locked` — PASS (see
  `docs/evidence/2026-10-27/files-newfolder.log`).
* The fix's two halves were reverted separately and the journey failed in each
  case, for the reason the record names:
  * both reverted → `the Files dialog must own the row: None`
    (`repro-both-reverted.log`)
  * dispatch fixed, renderer still old → `the dialog drew its Create button`
    (`repro-views-only-reverted.log`, which is BUG-0024)

## Residual risk

`retiring_context_menu` is cleared when the next menu opens and when the exit
spring lands, so the surface is only readable while the row's action runs — which
is exactly when it is read. A future caller that defers the action to the next
frame would find the snapshot gone and fall back to focus; that is why the
fallback exists rather than an `expect`.
