# Games: installed applications, a fullscreen session and a real test package

**Date:** 2026-10-31 (S91)
**Status:** implemented and real-client-tested; no image build, boot or physical display on this host.
**References:** the Linux desktop-entry specification (`Categories`, `Game`) and the
existing `LAUNCHENV` application path. No game runtime, store or anti-cheat service was
added.

## The gap this closes

The shell could install and launch a Linux application from a local bundle, but it had
no games surface and no notion of a game session. A game advertised with the standard
`Categories=Game` was just another launcher tile; a graphical program's first window
opened as an ordinary window. There was also no end-to-end test package proving that a
game-shaped bundle survives install, discovery, launch, mapping, fullscreen composition
and teardown.

## Design

### A game is still an application

Discovery parses `Categories=` from the installed `.desktop` entry into
`DesktopApp.categories`. `DesktopApp::is_game()` accepts the exact category token
`Game`; it does not match a substring or infer a game from its name. The installer
already generated the entry, and now carries the bundle's `Categories=` into the
generated file rather than dropping it.

That means there is one catalog, not a parallel game store. The same install, uninstall,
launch, failure and uninstall-convergence paths apply. A machine with no game entries
shows an explicit empty Games page; a machine with one shows the rows discovery
returned.

### The Games page

Settings gains **Games** (`Page::Games`), a view of the applications whose entries
declare `Game`. Each row shows the discovered name, its icon when available, and either
**Play** or, while that game owns the session, **Stop** and `Playing fullscreen`.

Play calls the existing desktop launch path. `launch_desktop` marks the game as pending
and the request carries the same `LAUNCHENV` environment (`WAYLAND_DISPLAY`,
`XDG_RUNTIME_DIR`) that S82 added. There is no game-specific spawn or daemon.

The sidebar is now long enough that its last rows are below the fold. Navigation selects
through `Model::select`, which asks the view to reveal the selected row once; a pointer
scroll does not snap back, because only navigation sets the one-shot flag.

### A session owns the first mapped window

After a game launch, the first `litric.client` window that maps becomes the game window.
The window manager's new `toggle_fullscreen` stores the previous frame, fills the whole
screen, and restores that frame when toggled again. While the game is fullscreen the
shell skips its top bar and dock and draws the client body over the whole display.

A fullscreen window with no shell chrome would have no way out. The Game Mode overlay
therefore stays above the game: a compact pill with the game status and a **Stop**
control. Stop closes the session, clears the pending state, and the normal desktop
returns. Closing the client window or losing the stale window id also ends the session.

Only the first mapped window is promoted. A game that opens a second window, a launcher
or a dialog does not become a multi-window game manager in this increment.

### The test package

`src/litric-shell/tests/fixtures/litric-test-game.desktop` declares:

```ini
[Desktop Entry]
Type=Application
Name=Litric Test Game
Exec=app
Categories=Game;
X-Litric-Test-App=true
```

The executable is the repository's real Ubuntu Weston 13 `weston-simple-shm` fixture,
not a mock process. The test copies it and the entry into a bundle, installs the bundle
as `com.litric.test-game`, discovers it, opens Settings › Games, clicks Play, waits for
the real Wayland window, checks the fullscreen frame at 1280×800, samples the client's
bright ring pixels through `views::draw`, clicks Stop, and uninstalls the package.

The same journey is covered by `journey_test_game_plays_fullscreen_and_stops`. The
ordinary browser-download journey
(`journey_a_download_opens_in_files_and_is_scanned`) is unchanged and was rerun; it
proves the browser → `filesd` import → `/Downloads` → scan path still works.

## Verification

| Check | Result |
|---|---|
| `cargo test --workspace` | **2,049 passed, 0 failed, 0 ignored** |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| golden frames | every frame within threshold; new `46-settings-games`; 10 required screens pass |
| `bash ci/verify.sh` | PASS -- 39 pass, 0 fail, 3 skipped; skips are image build, boot smoke and idle measurement |
| image build / boot | **NOT RUN**; no `mkosi` or `qemu-system-x86_64` |

The new coverage is the `Categories` parse and exact-game test, the installer's category
round-trip, the Games page and `GamePlay`/`GameStop` activation, fullscreen state, the
sidebar reveal, and the real-socket game journey. The journey asserts the real client's
window title, fullscreen frame, composed pixels and cleanup, not just that a command was
sent.

## Limits

* No game runtime, compatibility layer, Steam/Proton integration, anti-cheat support or
  gamescope is added. A game that needs those still needs them installed and configured.
* No `.deb`, `.rpm` or `.AppImage` package installer is added. The supported installation
  source remains a local directory bundle with an executable and an `app.desktop` entry.
  Browser downloads still import into `/Downloads`; they do not install themselves.
* The real test process is a Wayland demo client, not a playable game. It proves the
  session and composition path; it cannot prove a particular commercial game runs.
* Only the first mapped window is fullscreen; multi-window games and dialogs are not
  managed as a group.
* No image was built or booted and no physical display, GPU, audio or input device was
  exercised. The frame is composed by the shipping `views::draw` from fixed content.
* No RAM or performance claim is made for this increment. The added state is a few
  shell fields and a page; no new process or per-frame buffering was introduced.

## Rollback

Revert the S91 commit. The Games page, `Game` category parsing, installer category
round-trip, fullscreen toggle, Game Mode overlay, test fixture and journey, the golden
`46-settings-games` entry and the S91 documents disappear; S90's App Sandbox and all
earlier work remain.
