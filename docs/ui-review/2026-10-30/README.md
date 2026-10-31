# LitricOS UI review — 2026-10-30 (S81)

This directory holds **38 screens**, each **1280×800 px**, rendered by the shipping
`litric-shell/examples/frame_dump.rs`, which calls `litric_shell::views::draw` —
the same composition path `main.rs` uses. The gallery covers every Settings page
(`settings::Page::ALL`, 15 pages) and the 23 other reachable surfaces the example
already renders: desktop, applications, overlays, lock states and boot intro.

## What changed in S81

- **Region** now owns the language control, matching `registry.rs`; the control
  was removed from Appearance.
- **Privacy** now draws the persisted search-engine row that `registry.rs`
  already assigned to it.
- **Network** exposes the existing `NET LINK` and `NET RECONFIGURE` actions.
- **Bluetooth** exposes `BT POWER` only when the radio facts and a read-back
  measurement make a switch meaningful.
- **Sound** exposes the existing `AUDIO VOLUME`/`MUTE` actions and the measured
  level.
- **Sound** and **Bluetooth** ask `litric-sysd` for measurements when the page
  opens, through the same `Effect::ReadPanelState` path as quick settings.

`05-settings` and `25-settings-updates` are marked changed because the shared
Settings view and page navigation changed. The 13 new frames (`26`–`38`) are
the Settings pages the previous 25-frame review did not include.

## How to regenerate

```sh
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
cargo run --manifest-path src/Cargo.toml -p litric-shell --example frame_dump -- \
  --fonts build/fonts build/frames
for p in build/frames/*.ppm; do
  python3 tools/ppm2png.py "$p" "build/frames/$(basename "${p%.ppm}").png"
done
```

`build/` is gitignored; this directory is the committed copy. `index.html` links
each PNG and `all-screens.png` is the contact sheet. No PPM files are committed.

## What this is not

These are deterministic fixtures, not photographs of a booted VM. They prove
what `views::draw` composes from fixed facts. They do **not** prove the image
boots, the framebuffer scanout depth, that fonts rasterise on the target, that
the daemon socket is reachable on a real machine, or that C2/KMS, restore or the
remaining `DEFER` settings exist. Interactive browser and sign-in checks were
not possible in this environment.
