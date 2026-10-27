# BUG-0030 — The viewer read whole files, and the Files view re-sorted its listing per row

```
Bug ID        : BUG-0030
Severity      : S2 (HIGH) — a capped reply hid an uncapped read, and a per-row
                query repeated a full sort of the listing up to 24 times a frame
Class         : PERFORMANCE (allocation before the cap; repeated work per paint)
Subsystem     : src/litric-files/src/server.rs (`read_text`),
                src/litric-shell/src/apps.rs (`FilesApp`, `FileViewer`),
                src/litric-shell/src/views.rs (`draw_files`, `draw_viewer`)
First Observed: 2026-10-27, static performance audit of `main` (`b270aa3`)
Status        : FIXED (eight regression tests)
```

## 1. The 64 KiB cap limited the reply, not the read

`READTEXT` promised "at most 64 KiB" and did this:

```rust
let bytes = std::fs::read(&host)?;          // the whole file
let truncated = bytes.len() > TEXT_CAP;
let slice: Vec<u8> = bytes.iter().take(TEXT_CAP).copied().collect();
```

Opening a 2 GiB log asked the daemon for 2 GiB in one allocation, then copied
the first 64 KiB out of it. The daemon's `MemoryMax=256M` was the only thing
between a large file and an OOM kill, and the reply looked capped the whole
time — which is why this survived review.

**Fix.** `read_capped(source, cap)` reads through `Read::take(cap + 1)`, asks
for one byte past the cap so "exactly cap" and "more than cap" stay
distinguishable, and truncates the buffer it already holds. `read_text` opens
the file and hands the handle to it; peak allocation is now bounded by the cap,
not by the file.

**Tests** (`server::tests`):

* `read_capped_never_reads_past_the_cap` — an 8 MiB counting reader is asked for
  `TEXT_CAP + 1` bytes in total, no more.
* `read_capped_keeps_short_files_whole` — a short file is returned whole and
  untruncated.
* `readtext_caps_a_file_larger_than_the_viewer_limit` — a file 4 KiB over the
  cap comes back `truncated=1` with exactly the cap.

## 2. Every drawn row re-filtered and re-sorted the whole listing

`draw_files` built `visible()` once for the table, but called
`app.row_is_selected(i)` for each of up to 24 rows, and that method called
`visible()` again. `visible()` filters, allocates a vector and sorts it, with
`to_lowercase()` on both names in the comparator. A 10,000-entry directory —
the listing cap — therefore cost up to 24 full sorts per frame, plus a second
one for the command row's "N selected" readout through `selected_paths()`.

**Fix.** The ordered list is built once at the top of `draw_files` and passed
to two new methods, `row_is_selected_in(&rows, i)` and
`selected_paths_in(&rows)`. The old names remain as thin wrappers so every
existing caller keeps its meaning. The readout uses the same list.

**Test** (`apps::tests::selection_asked_once_per_built_list_matches_the_one_shot_query`):
the per-row answer and the selection list agree with the one-shot query for
every row of a filtered, sorted listing.

## 3. The text viewer cloned the file and re-split it on every frame

`draw_viewer` cloned the whole `FileViewer` (text included), and `window()`
collected `text.lines().map(String::from)` — every line of the file, owned —
before returning the slice that fits. A 64 KiB file was split into owned lines
about sixty times a second to show twenty of them.

**Fix.** `FileViewer::new` indexes each line's byte offset once, when the file
is opened. `window(rows)` returns `&str` slices of the text for the visible
range only, strips the line ending and the `\r` of a CRLF file, and clamps a
scroll past the end. `draw_viewer` borrows the viewer just long enough to copy
the path, the scroll and the visible lines, then releases the borrow before it
registers hotspots — so the file itself is never copied into a frame.

**Tests** (`apps::tests`):

* `a_viewer_shows_only_the_lines_that_fit` — page, end clamp and an
  out-of-range scroll.
* `a_viewer_counts_lines_the_way_an_editor_does` — a trailing newline does not
  open an empty line, CRLF keeps no carriage return, an empty file has no
  lines, and `"\n"` is one empty line.
* `a_viewer_slices_multibyte_lines_on_their_boundaries` — Thai text, where a
  byte offset one byte off would split a character.

## Verification

| Command | Result |
|---|---|
| `cargo test --offline -p litric-files readtext` | 2 passed |
| `cargo test --offline -p litric-files read_capped` | 2 passed |
| `cargo test --offline -p litric-shell apps::tests::` | 31 passed |
| `cargo test --offline -p litric-shell viewer` | viewer journeys pass |
| `cargo test --offline --workspace` | see the run recorded in `docs/evidence/2026-10-27/performance-audit.log` |
| `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets` | clean |
| `python3 ci/check-golden-frames.py --frames build/frames` | PASS |

The frame gate earned its keep here: the first version of the viewer rewrite drew
the scrim and panel *before* checking whether a viewer existed, so a closed
viewer left a dark rectangle over the Files window. No unit test noticed, the
Files golden frame did (`04-files`: 9.14% of cells changed). The surfaces are
now drawn only after the viewer is known to be present, and every frame is
identical to its golden again.

## Residual risk

* **`draw_files` still clones the `FilesApp`** (rows, selection sets, clipboard)
  once per paint. The clone is now the largest per-frame allocation for Files,
  and a borrowed view model is the next step; it was not taken here because the
  draw path registers hotspots on the same shell the model lives in.
* **`Activity` and `Browser` still clone their models per paint.** The browser
  clone grows with tab history; nothing measured its cost yet.
* **Damage-region repaint still runs the full view logic per region.**
  `Compositor::repaint` calls `views::draw` inside the damage loop, so layout,
  model cloning and string formatting repeat per rectangle even though pixel
  writes are clipped.
* **No benchmark was run.** The audit's 3.5-million-comparison figure was a
  call-graph estimate; the tests here prove the repeated work is gone, not how
  many microseconds it cost.
* Everything above is verified on the host. No LitricOS image was built or
  booted, so the end-to-end gain on the target machine is **not measured**.
