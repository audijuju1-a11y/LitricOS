# LitricOS UI review — 2026-10-31 (S82, updated in S85)

This directory holds **40 screens**, each **1280×800 px**. Thirty-nine are the
fixture gallery regenerated through the shipping `litric-shell/examples/frame_dump.rs`
and `litric_shell::views::draw`. The fortieth is not a fixture: it is the frame composed during the S82
real-socket e2e after `weston-simple-shm` was installed through `APP INSTALL`,
launched by the shell through `litric-sysd`, and mapped as a Wayland client.

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
  --fonts ../build/fonts ../build/frames-s85
for p in ../build/frames-s85/*.ppm; do
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

The 39 fixture frames prove what `views::draw` composes from fixed facts. The
real-app frame proves one genuine `wl_shm` client was installed, spawned,
connected to the LitricOS Wayland socket, mapped by the compositor, and painted
by `views::draw`. None of these frames prove a booted VM, a framebuffer/scanout
or GPU path, a real physical display, XWayland, clipboard/IME, AppImage or
package-manager installation, or the full breadth of Linux desktop software.
C2/KMS and restore remain unexercised here.
