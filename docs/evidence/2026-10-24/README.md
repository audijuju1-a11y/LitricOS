# 2026-10-24 — evidence for Shell v4 (top bar, dock, lock)

Every image is a frame from `frame_dump` (the shipping `views::draw`, software
canvas, fonts from `/usr/share/fonts/truetype/noto`), not a mock-up. They are
**host renders**, not a screen of a booted image: no QEMU/GPU boot was run in
this session.

Reproduce:

    cd src
    cargo run --release -p litric-shell --example frame_dump -- \
        --fonts /usr/share/fonts/truetype/noto --size 1920x1080 --theme dark OUTDIR

| file | what |
|---|---|
| `before-*` | commit `c28ec35`, 1280x800 dark |
| `after-*-1280` | this branch, 1280x800 dark |
| `after-*-1920`, `after-lock-960x600`, `*-light` | the other sizes/themes the plan lists |

Not covered here: Settings, Files and the apps (out of scope), boot intro, OOBE.
