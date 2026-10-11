# BUG-0016 — The typeface printed boxes, letters touched, and three layouts collided

```
Bug ID        : BUG-0016
Severity      : S3 (MEDIUM) — every window is legible, but the marks a person
                reads most (a sort arrow, a folder's size, a close button) were
                wrong, and two labels sat on top of each other
Class         : TYPOGRAPHY / LAYOUT
Subsystem     : src/litric-shell/src/font.rs, src/litric-shell/src/ui.rs,
                src/litric-shell/src/views.rs
First Observed: 2026-10-10, by measurement rather than by eye
Status        : FIXED — every item below has a test that fails without the fix
```

## Reproducer

```bash
cd src
cargo test --offline -p litric-shell --test font_standards
cargo test --offline -p litric-shell --test text_layout_audit
python ../tools/scan-nonascii.py          # what the shell's strings contain
cargo run --release -p litric-shell --example frame_dump -- ../build/frames
```

## Actual, and what each item was

Four separate defects, all of them found by asserting a property rather than by
looking at a screen.

### 1. The face was missing the glyphs the interface prints

`font::glyph` returns `None` for anything outside the drawn set and `font::draw`
draws a box, on purpose ("a typeface that is short a character"). The face was
missing, and the shell uses:

| Character | Where it was drawn | What a person saw |
|---|---|---|
| `↑` U+2191 | the sorted column's heading in Files | a box after "Name" |
| `—` U+2014 | a folder's size cell in Files | a box in the size column |
| `✕` U+2715 | the Details pane's close control | a box as a button label |
| `…` U+2026 | "Scanning…" in the Defender | a box mid-word |

`tools/scan-nonascii.py` walks the shell's string literals and lists the
non-ASCII characters in them; `src/litric-shell/tests/font_standards.rs` now holds the face to
that list, so the gap cannot reopen quietly. Added: `↑ ↓ ← → — … × ✕`.

### 2. Letters touched each other, because a stroke has a round cap

Every stem is drawn as a path with a round cap, so its ink reaches half a stroke
beyond the path's end on both axes. A glyph whose path fills its advance — `T`'s
crossbar, `_`, most capitals — therefore reached into the letter after it at
every size. Measured at the interface's own 13 px: **5 370 of 8 836 printable
pairs overlapped**, worst `!0` at -0.72 px; 6 301 left less than a quarter pixel
of paper, worst `/j` at -2.28 px.

The fix is the one real typefaces use: a **side bearing**. `font::bearings`
computes, per glyph and per size, the paper each side must carry — half a stroke
plus `BEARING_GAP_PX` (0.25 px) — and widens the advance by exactly the deficit
instead of redrawing the letter narrower. The floor scales with the weight,
because the overhang does: a bold glyph needs the same stem drawn with more
room around it.

This changed the width of text everywhere, which is the point: the previous
widths were wrong. Every string in the interface is now slightly wider, and
three layouts that had been hiding behind the squeeze collided.

### 3. Three layouts collided once text was its real width

| Collision | Measured | Cause | Fix |
|---|---|---|---|
| `Environment` over `00:00 UTC` on the top bar at 480 px | 17 x 18 px | the menu cluster was laid out with no idea where the status cluster began | the status cluster's width is computed *before* the menus, and a menu that does not fit is replaced by a `›` that opens the launcher (where "Apps" went anyway) |
| `Trash` over `0 item(s)` in the Trash header | 6 x 18 px | the count was placed at a fixed 80 px offset | placed at `font::draw`'s returned pen position plus a 10 px gap |
| `Environments` (Settings sidebar) over `Keyboard` (page) | 12 x 8 px | `ui::list_item` drew its title without eliding to its own row | the component elides its title, subtitle and trailing text against its own box |
| `Size` over `Modified` in a 480 px Files table | 6 x 17 px | column headings were drawn at fixed fractions | `files_columns` drops a column whose *heading* does not fit its own width and gives the width to the name column, as Explorer does |

### 4. The audit only checked one size per app

The five defects the first audit found were all geometry that fails below some
width. `src/litric-shell/tests/text_layout_audit.rs` covered 1280x800, 760x620 and 560x420 for
one app each — which is the same mistake in a smaller shape. It now walks every
app at 480x360, 560x420, 900x640 and 1264x784, both themes, plus a dense frame
(dialog over a window, populated trash, terminal output, three browser tabs, a
Defender scan in progress, a notification) and a 560x420 frame with a Files
confirmation dialog on it.

## Expected

* Every character the shell's own strings contain has a glyph.
* Two letters set next to each other leave at least a quarter pixel of paper
  between their inks, at every size on the type scale and at both weights.
* No two strings drawn inside one surface overlap, at every size an app can be
  given. (Text from two stacked windows overlapping is expected: the upper
  window's own background covers the lower one's letters, which is why the audit
  separates the two cases by surface.)

## Verified by

* `src/litric-shell/tests/font_standards.rs` — 10 tests: glyph coverage, collision and bearing
  floors across the size scale, tabular and lining figures, x-height/line/weight
  bands, space width, punctuation advances, and `measure` agreeing with where
  `draw` leaves the pen.
* `src/litric-shell/tests/text_layout_audit.rs` — 8 tests, all sizes and both themes; zero
  within-surface collisions in every one.
* `frame_dump`: `text=` lines report `overlaps=0 within a surface` for all ten
  scenes; the remaining pairs are cross-surface, i.e. stacked windows.

## What is not claimed

* **Thai text still draws as boxes.** The shell's own strings contain Thai
  (`apps.rs`, `shell.rs`, `textedit.rs` and others); a monoline Latin face cannot
  draw Thai, and a Thai face is a second typeface with its own mark stacking and
  its own metrics. `scan-nonascii.py` reports it; this bug does not fix it.
* Unicode beyond the printed set is still a box, by design.
* No claim about beauty. What is claimed is that the face has standard
  proportions, no colliding pairs, tabular lining figures and full coverage of
  what the interface prints — all four measured, none of them a matter of taste.
