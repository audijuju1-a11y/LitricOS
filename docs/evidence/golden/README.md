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

**The set is stale, and no longer matches this tree.** With the image's faces
installed, the gate reports 17 of 20 frames changed by more than its threshold:
the recorded frames predate the wallpaper, the dock, the move of the session rows
out of the control centre and the Thai-locale scenes, and 17-control-centre also
carries the network and Bluetooth rows added in WP-05. It is not a regression
introduced by one change; it is a gate that could not run on any host without the
fonts, so nothing noticed while the interface changed.

Re-recording is a review, not a rebuild: dump the frames, look at them, then

```
cd src && cargo run --offline -p litric-shell --example frame_dump -- --fonts ../build/fonts ../build/frames
python3 ci/check-golden-frames.py --frames build/frames --update
```

and copy `build/frames/*.ppm` over `artifacts/frames/` (regenerating the PNGs with
`tools/ppm2png.py`) so the pictures a person can look at are the same frames the
gate recorded. Say why in the commit, the way every other evidence change does.
