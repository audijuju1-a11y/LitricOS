# LitricOS UI review — 2026-11-01 (S92, Litric Ledger)

This directory holds **two complete 46-screen sets** at **1280×800**: `before/` is the S91
baseline and `after/` is the S92 redesign. Forty-five screens in each set come from the
shipping `litric-shell/examples/frame_dump.rs` composition through
`litric_shell::views::draw`. The forty-sixth, `39-linux-app`, is a real frame composed during
the S82 real-socket e2e after `weston-simple-shm` was installed, launched and mapped as a
Wayland client. It is not a mock-up or a fixture.

## Design direction

The redesign is called **Litric Ledger**. It replaces the wave wallpaper, mixed blue accents
and macOS-style traffic lights with a mineral/paper surface, a single fired-clay registration
accent, monochrome glyph window controls, a ledger wallpaper, a larger working type scale and
a consistent 5/8/12/16/20 px radius family. The full rationale and component rules are in
[the S92 design record](../../design/2026-11-01-litric-ledger-redesign.md).

## Evidence

- [index.html](index.html) — after gallery, with a before link on every screen.
- [all-screens.png](all-screens.png) — 46 after screens.
- [all-screens-before.png](all-screens-before.png) — 46 before screens.
- [before-after.png](before-after.png) — eight paired surfaces.
- [light-theme.png](light-theme.png) — five real light-theme frames.
- `png/` — the 46 after PNGs used by the gallery; `before/` and `after/` keep the source sets.

## What changed in S92

- **All 46 screens** inherit the new palette, type scale, surface material, window rail, icon
  grammar and dock state.
- **Desktop**: procedural ledger wallpaper with margin rules, registration spine, crop marks,
  edge ticks and a warm paper light-theme field.
- **Settings**: full 19-page sidebar at a 26 px row, `LITRIC / SETTINGS` header, page counter,
  larger titles and neutral action rows. Accent is reserved for selection and the primary
  action.
- **Files**: empty details guidance and consistent metadata line spacing.
- **Activity**: single-column at 920 px so the graphs no longer overlap the meters.
- **App Store / App Sandbox / Games**: spacing, clipped copy, dialog material and primary
  action alignment corrected.
- **Window chrome**: glyph-only controls, left title and a wider reserved title run. The real
  250 px `simple-shm` client now shows its full name rather than `Si...`.
- **Terminal**: a line containing Thai switches to the Thai-capable sans face; ASCII output
  stays monospaced, so the welcome text is no longer a row of missing-glyph boxes.
- **Lock**: the full “Click or press Enter to unlock” instruction remains visible at caption
  size instead of being elided by the larger footnote type.
- **Theme evidence**: five light-theme screens were rendered after correcting `frame_dump` to
  attach the same prepared glass backdrop as the real compositor. Before that correction a
  light window frame blended its own black shadow and read as grey; the shipped compositor
  path already had the backdrop.

## How it was produced

```sh
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
cd src
cargo run -q -p litric-shell --example frame_dump -- --fonts ../build/fonts ../build/frames
for p in ../build/frames/*.ppm; do
  python3 ../tools/ppm2png.py "$p" "../docs/ui-review/2026-11-01/after/$(basename "${p%.ppm}").png"
done
LITRIC_E2E_PNG="$PWD/../docs/ui-review/2026-11-01/after/39-linux-app.png" \
  cargo test -p litric-shell --lib \
  journey_real_linux_app_is_installed_launched_mapped_rendered_and_removed
python3 ci/check-golden-frames.py --frames build/frames --update
```

The 45 fixture frames are deterministic. The real-application frame is composed by the same
`views::draw` path in the e2e; its client buffer is deterministic, so it can be re-generated
with the command above.

## Verification and limits

The five light-theme frames are real `frame_dump --theme light` output, not recoloured dark
screens. The S92 change was checked with `cargo test --workspace` (2,049 passed, 0 failed, 0 ignored),
`cargo fmt`, clippy, golden-frame comparison, the text-layout audit (**no overlapping text
runs**) and the before/after review. The full `ci/verify.sh` result is recorded in
`docs/GATES.md`.

What the frames do **not** prove: a booted image, a framebuffer/scanout or GPU path, a real
physical display, XWayland, clipboard/IME, audio, Steam/Proton, anti-cheat, `.deb`/`.rpm`/
`.AppImage` installation, multi-monitor or fractional-scale behaviour. This host has no
`mkosi` or `qemu-system-x86_64`, so the image was neither built nor booted. No RAM or
performance measurement was made for this redesign.
