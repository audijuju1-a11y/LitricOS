# Desktop 2.0 — the shell as a working desktop

Written 2026-10-22. This is the record of one pass over the whole desktop
surface: what was added, what it is measured against, and what is still
missing. It exists so the next pass starts from the state of the system rather
than from the state of a conversation.

The reference points are the two desktops people already know. From macOS:
Mission Control, one search field that finds everything, per-application window
cycling, a menu that always answers to the keyboard. From Windows: quarters as
well as halves, a minimize key that restores before it hides, and a window
overview that shows every workspace. Neither is copied wholesale; each feature
had to fit the rules the shell already enforces (no panics, no invented data,
hotspots registered where things are drawn, a frame budget that is measured).

---

## 1. Mission Control

`Meta+Tab` opens an overview of every window on every workspace.

| Behaviour | Detail |
|---|---|
| Cards | One per window, ordered by workspace and then by window id, so the grid never reorders itself as focus moves. |
| What a card shows | Application icon, window title, workspace number, placement (`Normal`, `Snapped`, `Maximized`, `Minimized`, `Full screen`), and a window-shaped map of the content. |
| Keyboard | Arrows move the selection (left/right wrap, up/down clamp), `Tab` moves right, `Enter` opens the selected window, `Esc` closes. |
| Pointer | Click a card to open it; click the dimmed area to close. |
| Opening a minimized window | The card's window is revealed: its workspace is switched to, the minimize is undone, and it takes focus. One operation, in `litric-wm::WindowManager::reveal`. |
| What it is not | Not a live thumbnail. The canvas has no render-to-texture path, so drawing five full windows to shrink them would cost more than a frame for a screen that is waiting for a choice. The card is an honest map, not a fake screenshot. |

Mission Control replaces the desktop while it is open: windows and the dock are
not drawn underneath it. The backdrop still registers a full-screen hotspot, so
nothing behind the modal can be clicked through it.

Files: `src/litric-shell/src/overview.rs` (grid, selection),
`src/litric-shell/src/views.rs` (`draw_overview`),
`src/litric-shell/src/shell.rs` (`Overlay::Overview`, `overview_ids`,
`overview_open`), `src/litric-wm/src/manager.rs` (`reveal`).

## 2. One search field for the machine

The launcher's field (`Meta` or the dock's search bubble) used to search
applications. It now searches four kinds of thing:

| Kind | Example query | What Enter does |
|---|---|---|
| Application | `files` | Opens or focuses the application. |
| Action | `show desktop` | Runs the action, exactly as its shortcut would. |
| Settings page | `bluetooth` | Opens Settings on that page. |
| Window | `quarterly` | Switches to the window's workspace, unminimizes it, focuses it. |

Ranking is text first and kind second: an application keeps a large bonus, a
settings page that merely *contains* the word in its detail line is discounted
to a quarter, and results are deduplicated by title and kind. The same query
returns the same list on every machine: no recency ordering, no cache.

The launcher still owns the keyboard while it is open, including over a focused
application. That is a bug fix as much as a feature: typing into the search
field over a Files window used to put the characters into the Files filter
behind it.

Files: `src/litric-shell/src/seek.rs` (index and ranking),
`src/litric-shell/src/shell.rs` (`seek_open`, launcher key handling),
`src/litric-shell/src/views.rs` (`launcher_rows`).

## 3. Window placement

| Shortcut | Action |
|---|---|
| `Meta+Left` / `Meta+Right` | Left or right half. |
| `Meta+Shift+Left` / `Meta+Shift+Right` | Top-left or top-right quarter. |
| `Meta+Shift+Down` / `Meta+Shift+Up` | Bottom-left or bottom-right quarter. |
| `Meta+Up` | Maximize, or restore. |
| `Meta+Down` | Restore a snapped/maximized window; a second press minimizes it. |
| `Meta+R` | Restore to the size before snapping. |
| `Meta+C` | Centre. |
| `` Meta+` `` | Cycle this application's windows, including minimized ones and windows on other workspaces. |

`Meta+Ctrl+arrows` remain the workspace cycle; the quarters deliberately use
`Shift` so no combination has two meanings. The shortcut table test
(`no_two_shortcuts_share_a_binding`) is what caught the first attempt at
`Meta+Ctrl` and is the reason the rule is written down here.

Files: `src/litric-shell/src/shortcuts.rs`,
`src/litric-shell/src/shell.rs` (`perform`), `src/litric-wm/src/manager.rs`
(`cycle_app`).

## 4. Evidence

| Check | Result |
|---|---|
| `cargo test -p litric-shell` | 401 library tests, 21 binary tests, 26 integration tests (design lint, font standards, overflow, text layout). |
| `cargo test -p litric-wm` | 48 tests. |
| `cargo clippy -p litric-shell -p litric-wm --all-targets` | No warnings in either crate (one pre-existing warning in `litric-sysd`). |
| `cargo fmt --all` | Clean. |
| Frame dump | `21-mission-control` and `15-search` are written by `examples/frame_dump.rs`; the text-layout audit reports no overlap on the Mission Control frame (`overlaps=none (15 runs checked)`). |

Frames were produced with substituted Thai faces (the development host does not
carry Noto Sans Thai). The Latin faces were the image's Noto Sans and JetBrains
Mono, so layout is representative; typography of Thai text is not.

## 5. What is still missing

This pass made the *shell* behave like a desktop. It did not redesign every
application, and it did not add the remaining system surfaces:

1. **Snap Layouts** — the Windows 11 gesture of hovering a window's maximize
   button to choose halves, quarters or thirds. The manager already supports
   the geometry; the picker does not exist.
2. **Control Centre** — no volume or brightness slider, no Wi-Fi/Bluetooth
   tiles reading the providers, no theme switch. The panel is still toggles and
   session actions.
3. **Tabs and multiple windows per application** — opening an application twice
   still focuses its first window. The application models are singletons, so
   this is a data-model change in `apps.rs`, not a window-manager change.
4. **Application interiors** — Files has places, commands and dialogs but no
   tabs, breadcrumb editing or drag-and-drop. Settings has real facts on most
   pages but no search over them. The browser's session is a single engine.
5. **Accessibility tree** — the shell registers controls and a keyboard focus
   order, but there is no semantic tree or screen-reader bridge yet
   (`docs/master-plan/08-accessibility-i18n-input.md` is the specification).
6. **Golden coverage** — `ci/check-golden-frames.py` still names the old ten
   screens; Mission Control and the system-wide search should join that set.

The order above is the order they should be attacked: (1) and (2) are shell
work with a clear gate, (3) and (4) are application-model work, (5) needs a
decision about the accessibility transport, and (6) is bookkeeping that should
follow each of them rather than lead.
