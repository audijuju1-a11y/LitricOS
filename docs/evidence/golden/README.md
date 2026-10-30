# Golden desktop frames

`fingerprints.json` is the recorded identity of every frame `frame_dump` composes
from the shipping interface. `ci/check-golden-frames.py` compares a fresh dump
against it, coarsely (64x40 luminance cells, 2% of cells changed and 0.5 levels of
mean delta allowed), and `ci/verify.sh` runs both halves as its step 5b.

Produce the frames with the faces the image installs. `verify.sh` reads them from
the system font directories, or from `build/fonts` when that directory exists —
`build/` is ignored, so a host without the packages can unpack
`noto-fonts` and `ttf-jetbrains-mono-nerd` there (`noto/NotoSans-Regular.ttf`,
`noto/NotoSans-Bold.ttf`, `noto/NotoSansThai-{Regular,Bold}.ttf`,
`TTF/JetBrainsMonoNerdFontMono-{Regular,Bold}.ttf`) and get frames that mean the
same thing. Without either, step 5b is a SKIP and the set is not checked at all.

## State of the committed set (2026-10-30)

**Re-recorded, and it matches this tree.** S81 re-recorded the set after moving
the language control from Appearance to Region. The dump now carries 38 scenes:
the previous 25 plus `25-settings-updates` from §80 and the 13 Settings pages
added by S81. `ci/check-golden-frames.py --frames build/frames` reports all 38
frames unchanged against the recorded fingerprints; the 13 new Settings pages
are in its optional comparison list.

The review that justifies it is in [`../2026-10-26/`](../2026-10-26/): the six
scenes that carry the redesign — desktop, Control Centre, lock, launcher, boot,
windows — in both themes, as PNGs straight from the dump.

The current 38-screen review is
[`docs/ui-review/2026-10-30/`](../../ui-review/2026-10-30/index.html). It adds
every Settings page to the review set, and those 13 page frames are now in the
gate's optional comparison list so a future Settings change cannot silently
leave the review stale.

Recording again, when the interface next changes, is a review and not a rebuild:
dump the frames, look at them, then

```
cd src && cargo run --offline -p litric-shell --example frame_dump -- --fonts ../build/fonts ../build/frames
python3 ci/check-golden-frames.py --frames build/frames --update
```

and copy `build/frames/*.ppm` over `artifacts/frames/` (regenerating the PNGs with
`tools/ppm2png.py`) so the pictures a person can look at are the same frames the
gate recorded. Say why in the commit, the way every other evidence change does.
