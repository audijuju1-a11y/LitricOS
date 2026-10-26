# The V6 desktop, merged into the line that carries the daemons

The interface that ships in this tree is the one drawn by the `litricos-shell-v6`
bundle (`redesign/shell-v4`, head `7030ce1`): a glass top bar with no background
of its own, a floating dock of 46-px tiles with original icon artwork, the
pen-drawn LitricOS mark, a Control Centre of measured cards, and the lock screen
redrawn around the same tokens. That work lived on a branch this line never
merged, while this line grew the daemons (notifyd, filesd, browserd, sysd) and
the research registers. Both halves are in the tree now, and this folder is what
the merged result actually renders.

## What the merge touched

`git merge tmp-v6` conflicted in five files, in descending order of consequence:

| File | Ours | Theirs | Resolution |
| --- | --- | --- | --- |
| `src/litric-shell/src/views.rs` | quick settings with measured POWER/SYSTEM/NETWORK/BLUETOOTH/VOLUME/ENVIRONMENT cards | the V6 card sheet, dock lift, top-bar chrome | V6's sheet and dock, with the network, Bluetooth and volume cards carried over from this line |
| `src/litric-shell/src/shell.rs` | `BOOT_INTRO_MS = 2400`, a boot stage note | `BOOT_INTRO_MS = 2300` with a cross-fade cover, `TOP_BAR_LOGICAL = chrome::TOP_BAR_H` | V6's timings and cover, both sides' tests kept |
| `src/litric-shell/src/lib.rs` | daemon clients, `prefstore`, `seek` | `appicons`, `chrome` modules | both |
| `src/litric-shell/src/e2e.rs` | a 30-px strip under 32 px of tiles | a 62-px strip with 46-px tiles, 12 px of float | V6's contract, asserted against the eight measured hotspots |
| `src/litric-shell/examples/frame_dump.rs` | 1280x800 dark only | `--size WxH` and `--theme dark\|light` | V6's flags |

`src/litric-shell/src/lock.rs` merged cleanly from V6 and cost one dead
parameter: the password field's radius is now `LOCK_FIELD_H / 2` inside the pill,
so `entry()` no longer takes a radius. `chrome.rs` gained a second doc line for
each of the two constants that shared one, which is what `missing_docs` asked
for.

## What the gates said, and what had to be fixed

The V6 branch was written before this line tightened `[workspace.lints]`, so its
code did not survive `clippy -D warnings` unchanged. None of it was a behaviour
change:

* 31 `clippy::indexing_slicing` denials. The quick-settings sheet indexed the
  card list (`rects[0]`, `cards[0].label`). It now walks the cards and their
  rectangles with one iterator, so a card's label is the label it was measured
  from and no list is indexed. The icon scaler reads its four channels by
  destructuring the slice it just asked for. The blend test samples its `extra`
  table with `get`.
* 26 `clippy::arithmetic_side_effects` warnings. Integer pixel counts became
  `saturating_add`; the one per-channel lerp in `mix_from` keeps its plain
  arithmetic under an `#[allow]` that states the bound that makes it safe
  (255 * 256 + 255 * 256 + 128, which is not near `u32::MAX`).
* `clone_on_copy`, `manual_range_contains` and `type_complexity`, fixed at the
  site.

## Commands, run on this host

```
cd src && cargo check --workspace --all-targets --locked      # 0 errors
cd src && cargo clippy --workspace --all-targets --locked     # 0 warnings
cd src && cargo fmt --all -- --check                          # clean
cd src && cargo test --workspace --locked                     # 1560 passed, 0 failed
cd src && cargo run --offline -p litric-shell --example frame_dump -- \
    --fonts ../build/fonts ../build/frames                     # 24 scenes
cd src && cargo run --offline -p litric-shell --example frame_dump -- \
    --fonts ../build/fonts --theme light ../build/frames-light # 24 scenes, light
```

The frames in this folder are those dumps, converted with `tools/ppm2png.py` and
not retouched. `dark-` and `light-` prefix the same six scenes in both themes.

## What the pictures show

* `*-01-desktop-idle` — the top bar with the mark at 12 px, the dock floating
  12 px above the edge with eight tiles and the search pill, and the wave
  wallpaper. The dock is the V6 measurement: 62-px strip, 46-px tiles.
* `*-17-control-centre` — the card sheet at its measured height. POWER is the
  four-segment profile track, SYSTEM the four switches, and the NETWORK,
  BLUETOOTH and VOLUME cards are this line's (they read `litric-sysd`), which is
  why the merge had to keep both halves. The card titles come from
  `quick_cards`, the same list the panel is measured from.
* `*-11-lock` — the clock, the three glass status pills, the avatar, the
  password pill and Customize.
* `*-14-launcher` — the launcher over a dimmed desktop.
* `*-24-boot-intro` — the pen-drawn mark mid-write.
* `*-03-windows-overlap` — three windows over the desktop, the frame a
  compositor has to compose rather than a single panel.

## What is not verified here

Rendered on a host, not in the image: these frames are `frame_dump`'s
composition of the shipping code with a fixed fixture of facts, not a booted
LitricOS. Nothing in this folder shows Chromium, a real input device or a real
session. The image build and the QEMU boot are separate evidence, and neither
was run for this merge.
