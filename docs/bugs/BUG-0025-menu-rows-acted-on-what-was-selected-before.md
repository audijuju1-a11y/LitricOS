# BUG-0025 — menu rows acted on what was selected before

```
Bug ID        : BUG-0025
Severity      : S2 (HIGH) — the three instances move or delete the wrong item:
                a file restored from the wrong Trash entry, the wrong tab
                closed, and focus handed to a tab nobody chose
Class         : UI / DISPATCH (target not carried from the click to the row)
Subsystem     : src/litric-shell/src/shell.rs (`menu_surface_at`, the
                right-click branch, `perform_menu`, `menu_tab`),
                src/litric-shell/src/apps.rs (`ContextMenu`, `BrowserApp::close`,
                `BrowserApp::navigate_tab`)
First Observed: 2026-10-27, auditing the siblings of BUG-0023
Status        : FIXED (four regression tests)
```

## Why this exists

BUG-0023 was fixed by reading the surface of the menu that had just been closed.
The obvious next question is which *other* rows dispatch against state the click
did not set. This record is the answer: three, all of the same shape — the row
runs with a target that was chosen at some earlier moment, not the thing under
the pointer.

The audit covered every arm of `perform_menu`:

| Rows | Target | Verdict |
|---|---|---|
| `NewFolder` | the menu's surface, or focus for the toolbar | fixed as BUG-0023 |
| `NewFile`, `Paste`, `Rename`, `Copy`, `Cut`, `Trash`, `Compress`, `Extract` | the Files selection | sound: a `FilesRow` right-click selects the row first |
| `SortName`, `SortSize`, `SortModified`, `ToggleDetails` | Files' own view state | sound: `Files` menu only |
| `EmptyTrash` | `trash.confirm`, not a row | sound |
| `Restore`, `PurgeTrashItem` | the Trash selection | **wrong** — right-clicking a Trash row selected nothing |
| `BrowserCloseTab`, `BrowserReload` | the active tab | **wrong** — the menu is opened for the tab under the pointer, and Chrome does not switch tabs on a right-click |
| `ScanNow`, `OpenTrash`, `NextWallpaper`, `DisplaySettings`, `BrowserStart`, `EngineGoogle/DuckDuckGo/Bing` | no target | sound |

`MA::OpenSelection`'s first branch reads `desktop_icons.selected`, which nothing
in the shell ever sets any more: the V6 desktop draws no icon hotspots (the
journey `journey_the_desktop_is_a_bar_and_a_wallpaper` asserts exactly that). It
is dead code, not a defect: `files.selected_path()` is what answers.

## The three instances

### 1. Trash rows restore the previously selected entry

Right-clicking a Trash row opened the Trash menu without touching
`trash.selected`. `Restore` and `Delete permanently` then ran against whatever
row was selected before — or, if none was, returned silently, which is the
"row that does nothing" from finding 65.

The shell already had the rule and the comment for it — "Explorer's rule:
right-clicking a row also selects it" — but the code applied it only to
`MenuSurface::FilesRow`.

### 2. The tab menu closes the wrong tab

`MenuAction::BrowserCloseTab` is documented as "Close the browser tab under the
pointer". The implementation read `self.browser.active`. Right-clicking a
background tab and choosing **Close tab** closed the page the user was reading,
not the tab they pointed at. **Reload** was the same: it reloaded the focused
tab and cleared its error flag.

The tab cannot be recovered after the click the way a surface is: right-clicking
a tab does not switch to it (neither Chrome nor Firefox does), so `active` never
names the tab the menu is about. The menu has to record it while the pointer
still can.

### 3. Closing a tab in front of the focused one moved focus

`BrowserApp::close(i)` removes tab `i` and then only clamped `active` if it ran
past the end. With four tabs and the third focused, closing the first left
`active = 2`, which now names the fourth tab: the page changes under the user for
no reason they asked for. This was reachable before this audit through each tab's
own close button; closing a background tab from the menu made it easy to hit.

## Fix

**The menu records its target.** `ContextMenu` gained

```rust
/// The browser tab the pointer was over, when the surface is a tab.
pub tab: Option<usize>,
```

set by `ContextMenu::for_tab(i)` while the pointer is still on the tab. The
right-click branch now handles all three surfaces it can name:

```rust
match (surface, self.hot.hit(at.x, at.y)) {
    (MenuSurface::FilesRow, Some(Hot::Files(FilesHot::Row(i)))) => self.files.select_row(i),
    (MenuSurface::Trash, Some(Hot::TrashRow(i))) => {
        self.trash.selected = Some(i);
        self.trash.confirm = None;
    }
    (MenuSurface::BrowserTab, Some(Hot::BrowserTab(i))) => menu = menu.for_tab(i),
    _ => {}
}
```

**The row reads it back through the closed menu.** `menu_tab()` is the tab
equivalent of `new_folder_from_menu`: it reads the live menu or the retiring
snapshot, and falls back to the active tab for the page-area menu, which really is
about the focused tab.

**Close keeps the focused tab focused.** `close` now notes whether the removed
tab was the focused one before the `remove`:

```rust
let was_active = i == self.active;
self.tabs.remove(i);
...
if was_active {
    self.active = i.min(self.tabs.len().saturating_sub(1));
} else if i < self.active {
    self.active = self.active.saturating_sub(1);
}
```

**Reload has a per-tab entry point.** `BrowserApp::navigate_tab(i)` clears tab
`i`'s error and returns its URL; `navigate_active` delegates to it. The shell only
touches the visible page's status line, frame buffers and `loading` flag when the
reloaded tab is the focused one — reloading a background tab must not claim the
page on screen is loading.

## Regression tests

* `e2e::journey_right_click_trash_row_targets_that_row` — two entries in a real
  trash, row 0 selected, right-click row 1, **Restore**: `b.txt` returns, `a.txt`
  stays in the trash. Asserts the selection the menu carries too.
* `e2e::journey_right_click_background_tab_closes_that_tab` — three tabs, the
  first focused, right-click the third, **Close tab**: the third goes and the
  first stays focused. On the old targeting the remaining tabs were
  `["two", "three"]` — the wrong page had been closed.
* `e2e::journey_right_click_background_tab_reloads_that_tab` — reload clears the
  error on the tab the menu was opened for and leaves `browser.status` empty.
* `apps::tests::browser_close_keeps_the_focused_tab_focused` — four tabs, close
  the first while the third is focused: focus must still be on the same page;
  then close the focused tab and the next one takes its place.

## Verification

* `cargo test -p litric-shell --locked` — PASS (see
  `docs/evidence/2026-10-27/files-newfolder.log`).
* All four tests fail with the fixes reverted, each at its own assertion
  (`docs/evidence/2026-10-27/repro-sibling-rows.log`).
* With only the tab targeting reverted, the close journey reports
  `left: ["https://two.example", "https://three.example"]` against
  `right: ["https://one.example", "https://two.example"]`
  (`docs/evidence/2026-10-27/repro-sibling-tab-target.log`) — the focused page was
  the one closed.

## Residual risk

* The rows still have no **disabled** state: on an empty Trash, Restore and Delete
  permanently are drawn and do nothing. That is finding 60/65, a decision about a
  state model rather than a target, and it is unchanged.
* A background tab's reload is sent to the engine but its progress is not shown
  anywhere: the status line belongs to the visible page by design, and a per-tab
  spinner is part of the same missing state model.
* `ContextMenu` now carries a target in one field for one surface. A second
  surface that needs one (a Trash row identity, a workspace row) should
  generalise the field rather than add another; doing that now, for a single
  caller, would be a type for a hypothetical.
