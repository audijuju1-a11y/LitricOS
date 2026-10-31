# LitricOS UI review — 2026-10-31 (S82, updated in S89)

This directory holds **45 screens**, each **1280×800 px**. Forty-four are the
fixture gallery regenerated through the shipping `litric-shell/examples/frame_dump.rs`
and `litric_shell::views::draw`. The forty-fifth is not a fixture: it is the frame composed during the S82
real-socket e2e after `weston-simple-shm` was installed through `APP INSTALL`,
launched by the shell through `litric-sysd`, and mapped as a Wayland client.

## What changed in S90

- Settings has an **App Sandbox** page backed by `litric-sysd`: the system Flatpak
  updates on offer, one application's effective sandbox permissions as discrete
  attributed facts, two system overrides (host filesystem and network) and a reset.
  Every write is confirmed; a missing Flatpak is an explicit unavailable state.
- `45-settings-app-sandbox` is new and was reviewed as an image before recording. It
  shows two updates, three installed apps, four effective facts, both override states,
  the removal attributed to the global override, and no clipped or elided values.
- `44-settings-app-store` moved by 3.01% of cells because the sidebar gained the
  App Sandbox row; the change was inspected and the frame re-recorded. The other
  Settings frames moved by 0.04% or less. The golden set records 44 fixture frames.

## What changed in S89

- Settings has an **App Store** page backed by `litric-sysd`: Flatpak search,
  installed system applications with version/origin/size, and confirmed install
  or remove actions. A missing Flatpak is an explicit unavailable state.
- `44-settings-app-store` is new and was reviewed as an image before recording.
  It shows the search field, four catalogue rows, two installed rows and both
  actions without clipping or elided values.
- The other 42 fixture frames remain within the golden threshold; `39-linux-app`
  is unchanged. The golden set now records 43 fixture frames.

## What changed in S88

- Settings has a **Services** page: AppArmor profile modes, Polkit action
  policies, firewalld zones, Flatpak system apps/remotes, bubblewrap availability
  and NetworkManager state, read from the daemon's `svc.*` facts.
- The page draws a reload control only when the tool and runtime state make it
  actionable; the frame shows the three controls a configured machine would
  offer, and the installed-but-stopped NetworkManager has none.
- `43-settings-services` is new. The other Settings frames moved by 0.04% of
  cells because the sidebar gained one row; that is within the golden threshold
  and the set was re-recorded.

## What changed in S87

- A discovered application can be pinned to the dock from its launcher-tile
  context menu. The pin is the dock's existing tile language, inserted between
  the built-in applications and the Trash separator.
- `42-pinned-dock` is new: it shows the pinned app in the open launcher and on
  the same dock strip. The other 40 fixture frames are byte-identical to S86;
  `39-linux-app` is unchanged. The golden set now records the new frame.
- Uninstall removes the pin together with the launcher tile. The pin list is
  session state, not a preference record, so there is no persisted id to clean
  up on a later reload.

## What changed in S86

- An installed application's launcher tile now has a real context menu entry,
  **Uninstall application…**. It opens the same desktop dialog, scrim, spring and
  popover material as install, with a destructive **Uninstall** action.
- `41-uninstall-dialog` is new. It shows the confirmation state, not a mock:
  the name comes from the same discovered desktop entry the launcher drew.
- The other 39 fixture frames are byte-identical to S85; `39-linux-app` is
  unchanged. The golden set now records the new `41-uninstall-dialog` frame.
- The dialog's busy state replaces Cancel with a blocking panel, so a running
  worker cannot be dismissed or retried through the UI.

## What changed in S85

- The launcher now has an **Install app** entry beside Lock screen. The screen
  `14-launcher` was regenerated because that button is part of the rendered
  surface.
- `40-install-dialog` is new: the existing desktop dialog with the bundle-folder
  field, optional reverse-DNS id, Install/Cancel actions and the same scrim,
  spring and popover material as every other desktop dialog.
- The other 37 fixture frames are byte-identical to S82; `39-linux-app` is
  unchanged. The golden set now records the changed `14-launcher` and the new
  `40-install-dialog`.

## What changed in S82

- `LAUNCHENV` carries `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` from the shell to
  the daemon. Without it, a graphical application started by the daemon had no
  display and exited before mapping. See `docs/adr/0027-launch-environment.md`.
- `launch_desktop` now sends that environment; the argv-only `LAUNCH` request is
  unchanged.
- The real-socket e2e installs Ubuntu Weston 13 `weston-simple-shm`, starts it
  from launcher discovery, asserts a `litric.client` window titled `simple-shm`,
  checks the client's ring pixels through `views::draw`, closes the window and
  observes `Gone`, then uninstalls it.
- The thirty-eighth fixture set includes every Settings page; no Settings screen
  changed in S82, which is why the regenerated set matched byte-for-byte.

`39-linux-app` shows the real client's window over the LitricOS wallpaper and
dock. The launch toast has expired so the content is unobscured. The image is
deterministic: two independent e2e runs produced identical SHA-256
`0e9ca27edb96460140ac7a387debffbbb04e42cf51fa1a1bf252f7312e6b8b4d`.

## How to regenerate

```sh
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
cd src
cargo run -q -p litric-shell --example frame_dump -- \
  --fonts ../build/fonts ../build/frames-s87
for p in ../build/frames-s87/*.ppm; do
  python3 ../tools/ppm2png.py "$p" "${p%.ppm}.png"
done
LITRIC_E2E_PNG=build/s82-linux-app.png \
  cargo test -p litric-shell --lib \
  journey_real_linux_app_is_installed_launched_mapped_rendered_and_removed
```

The e2e writes `build/s82-linux-app.png` relative to the repository root; copy
it to `png/39-linux-app.png`; keep `png/40-install-dialog.png` from the fixture
dump and rebuild `all-screens.png` with a 4-column labelled contact sheet. `build/` is gitignored.

## What this is not

The 41 fixture frames prove what `views::draw` composes from fixed facts. The
real-app frame proves one genuine `wl_shm` client was installed, spawned,
connected to the LitricOS Wayland socket, mapped by the compositor, and painted
by `views::draw`. None of these frames prove a booted VM, a framebuffer/scanout
or GPU path, a real physical display, XWayland, clipboard/IME, AppImage or
package-manager installation, or the full breadth of Linux desktop software.
C2/KMS and restore remain unexercised here.
