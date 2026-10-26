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

## State of the committed set (2026-10-26)

**Re-recorded, and it matches this tree.** The set that had gone stale was
replaced while merging the V6 design branch: the dump now carries 24 scenes
rather than 20 (`22-power-menu`, `23-litro-menu` and `24-boot-intro` were drawn
by that branch and had never been recorded), and every frame in it was composed
from the merged code. `ci/check-golden-frames.py --frames build/frames` reports
24 frames unchanged against the recorded fingerprints.

The review that justifies it is in [`../2026-10-26/`](../2026-10-26/): the six
scenes that carry the redesign — desktop, Control Centre, lock, launcher, boot,
windows — in both themes, as PNGs straight from the dump.

Recording again, when the interface next changes, is a review and not a rebuild:
dump the frames, look at them, then

```
cd src && cargo run --offline -p litric-shell --example frame_dump -- --fonts ../build/fonts ../build/frames
python3 ci/check-golden-frames.py --frames build/frames --update
```

and copy `build/frames/*.ppm` over `artifacts/frames/` (regenerating the PNGs with
`tools/ppm2png.py`) so the pictures a person can look at are the same frames the
gate recorded. Say why in the commit, the way every other evidence change does.
