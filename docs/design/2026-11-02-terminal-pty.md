# Terminal: a PTY, a styled alternate screen, mouse and paste

**Date:** 2026-11-02
**Status:** implemented and host-tested; image build and real display remain unverified
**Scope row:** `docs/SYSTEM-SCOPE.md` #549, now `P`

## The problem

The first Terminal ran one command as `/bin/sh -c` under `timeout 30`, with
`stdin` connected to null. It was real execution, but it was not a terminal:

- `cd`, exported variables and shell state died after every Enter;
- any command that needed stdin (a password prompt, a REPL, an editor) could not
  work;
- a legitimate command was killed at 30 seconds;
- there was no Ctrl+C path, and a new command was refused while one was running.

A later slice made the shell persistent under `script(1)`, then a C helper gave
it real `TIOCSWINSZ` resize. That slice still modelled the alternate screen as
one-character, one-colour cells, so vim could enter the screen but its syntax
colours, attributes and wide glyphs were lost. This document records the PTY
path, the styled cell grid, mouse reporting and bracketed paste that now sit on
top of it.

## Process model

```text
 litric-shell (UI thread)
   │  bytes                         ┌──────────────────────────────┐
   ├───────────────────────────────►│ litric-pty (C, -lutil)       │
   │                                │  openpty + login_tty          │
   │  Event::Output / Ready         │  /bin/sh -i on the PTY        │
   ◄────────────────────────────────│  TIOCSWINSZ + SIGWINCH        │
        reader thread                └──────────────────────────────┘
        writer thread                 ▲
                                      │ "COLS ROWS\n"
                                      │ 0600 size file
```

`src/litric-shell/src/litric-pty.c` is the only code that needs
`openpty`/`login_tty`/`ioctl`; the Rust crate keeps `#![forbid(unsafe_code)]`.
The helper starts `/bin/sh -i`, copies bytes between its stdin/stdout and the
master side, and polls a size file every ~100 ms. When the file changes it
applies `TIOCSWINSZ`, finds the foreground process group with `tcgetpgrp` and
sends `SIGWINCH` to that group.

The file is created by `terminal.rs` with mode 0600 and a name built from the
process id, a nonce and a counter. It is never a user path. It is removed on a
spawn failure or when `Session` is dropped through the single
`remove_size_file` helper whose reason is recorded in `ci/lint-file-safety.py`.

The Rust side starts two threads: the writer consumes a `Vec<u8>` channel, and
the reader strips the readiness marker and emits `Output`, `Ready` or
`Closed`. Dropping the session kills the helper; the child shell does not
survive the window.

The initial size is 100×30. The shell's `terminal_resize` and the layout pass
keep the PTY size and the model's grid size in step. `stty size` observes the
helper's ioctl, not a number captured at spawn.

## Prompt marker and fallback

A PTY has no "command finished" event. `PS1` is set to `__LITRIC_READY__`; the
reader turns that marker into `Ready` and never displays it. The first command
is queued until that marker arrives, so it cannot race the shell's startup. The
marker is display state, not a security boundary: a program that deliberately
prints the same text could look like a prompt.

If the helper is absent or cannot start, the old one-shot path still runs with
its own 30-second bound, and the interface says which path ran. It is not
presented as a persistent PTY session.

## The alternate-screen grid

The primary screen remains the bounded scrollback. It handles UTF-8 lossy
decoding, CRLF, progress-line carriage returns, backspace and a final line
without a newline. ANSI CSI and OSC sequences are consumed rather than printed.

A full-screen program switches to `terminal_screen::Screen` on `CSI ?1049h`,
`?1047h` or `?47h`. The grid supports cursor addressing (`H`/`f`, `A`–`D`,
`G`, `d`), erase (`J`, `K`), cursor visibility (`?25h/l`), save/restore
(`s`/`u`, `ESC 7/8`), wrapping, scrolling and the matching leave sequences.
The primary scrollback is untouched while the program owns the alternate
screen.

### Styled cells

Each cell owns:

- a display cluster (`String`), not just a `char`, so a combining mark stays
  attached to its base;
- `width`: `1`, `2` for a wide base, or `0` for the continuation column owned
  by a wide base;
- a `CellStyle` with foreground, background and attributes.

The SGR parser handles the forms vim and ordinary terminfo emit:

| Request | Representation |
|---|---|
| `30`–`37`, `90`–`97` | ANSI foreground `Indexed(0..15)` |
| `40`–`47`, `100`–`107` | ANSI background `Indexed(0..15)` |
| `38;5;n`, `48;5;n` | xterm 256-colour `Indexed(n)` (including the 6×6×6 cube and grey ramp) |
| `38;2;r;g;b`, `48;2;r;g;b` | `Rgb(r,g,b)` |
| `1`, `2`, `3`, `4`, `7` | bold, dim, italic, underline, reverse |
| `0` / `22` / `23` / `24` / `27` | reset all / bold+dim / italic / underline / reverse |
| `39`, `49` | default foreground, default background |

A style is copied into every cell when that cell is written. Changing SGR
later cannot recolour text already on screen, and the next cell starts from the
new state. The parser's current SGR state is reset on **entry to and exit from**
the alternate screen, and by `SGR 0`. As in a real terminal, an SGR state
without a reset persists across a newline; the per-cell copy is what prevents
one cell's stored style from bleeding into another.

The background is filled only when the cell requests one (or reverse video);
a default-background cell leaves the terminal surface visible. Bold brightens
an indexed foreground 0–7 and uses the bold face weight. Dim reduces foreground
alpha. Italic uses a synthetic oblique because the bundled faces have no italic
cut. Underline is a line under the cell. Reverse swaps the resolved foreground
and background, including both defaults.

### Scroll regions

`CSI r` (DECSTBM) sets an inclusive top/bottom region and homes the cursor.
A line feed at the region bottom scrolls only those rows; `ESC M` reverse-indexes
at the top; `CSI S`/`T` scroll the region up/down; `CSI L`/`M` insert/delete
lines at the cursor inside it. Rows outside the region are not touched. Invalid
or one-line regions are ignored, and resize clamps a region to the new height
or restores the full screen if it no longer has two rows.

### Wide characters and combining marks

`terminal_screen::display_width` returns:

- `2` for CJK/full-width ranges and emoji;
- `0` for combining marks, Thai vowel/tone marks, variation selectors, zero
  width joiners and related controls;
- `1` for ordinary characters, including Thai base letters.

A wide base occupies its column and the next continuation column. Writing a
wide glyph at the last column wraps before drawing rather than splitting it.
Writing over either half clears the whole glyph. `backspace` lands on the wide
base, never its continuation. Erase operations clear any wide glyph that
crosses the erased range. Resizing repairs a base or continuation cut by the
new edge.

Combining marks append to the preceding base cluster and do not advance the
cursor. Simple emoji ZWJ sequences and variation selectors are kept in the same
cluster. This is a small dependency-free approximation of Unicode width, not a
full UAX #29 grapheme-cluster implementation; see the limits below.

## Input

While the alternate screen is active, the shell stops treating input as a line:

- printable text arrives as `InputEvent::TextInput` and is sent as UTF-8;
- arrows become `ESC [ A` … `ESC [ D`;
- Home/End/Delete/Insert/PageUp/PageDown and F1–F12 become the usual xterm
  sequences;
- Shift+Tab becomes `ESC [ Z`;
- Escape, Enter, Tab and Backspace are sent as bytes;
- Ctrl+letter becomes its control byte, so Ctrl+C and Ctrl+D reach the program
  without the shell's normal prompt handling.

The app's `TerminalApp` gets a viewport from the layout pass; the same width and
height are written to the PTY. `views::draw_terminal` draws each cell at its
own column, using its style, and draws the cursor when the program asks for it.
A click anywhere on the full-screen surface focuses the Terminal, so the raw
input path is reachable.

### Mouse reporting

`TerminalApp` parses the private modes a program uses to ask for pointer input:

| Sequence | Effect |
|---|---|
| `CSI ? 1000 h` / `l` | normal tracking: press and release, no motion |
| `CSI ? 1002 h` / `l` | button-event tracking: press, release and motion while a button is held |
| `CSI ? 1003 h` / `l` | any-event tracking: every move, held or not |
| `CSI ? 1006 h` / `l` | SGR encoding, including lowercase `m` releases and coordinates past 223 |
| `CSI ? 2004 h` / `l` | bracketed paste |

The three tracking flags are independent because a program may enable more
than one; the effective mode is `Any > Button > Normal > Off`. `mouse_held` is
a bitmask so motion in `1002` names the held button (button code plus 32).
Wheel notches are press-only reports with buttons 64/65. Without `?1006`, the
legacy X10 six-byte form is used; a coordinate beyond its 223 limit is dropped
rather than encoded at the wrong cell. Shift, Alt/Meta and Ctrl add the xterm
modifier bits, and a release keeps those bits in the X10 form.

`views::draw_terminal` publishes the rectangle where it actually drew the grid.
`Shell::terminal_mouse` maps the pointer through that rectangle only while the
Terminal window is focused and a tracking mode is on, then writes the report
through `terminal_send_raw` to the same PTY writer as typed keys. A button
press/release focuses the Terminal; a wheel report is consumed before shell
scrolling; a move is consumed for the report and still updates the desktop
pointer. Turning any tracking mode off clears the held-button mask, so a button
that was down across the transition cannot become a phantom drag.

Once a press has been delivered, the held button captures motion and release
until it comes up. If the pointer leaves the grid, the report is clamped to the
nearest edge cell rather than dropped on the window frame; the program sees the
end of the drag. A release whose press was never delivered is suppressed, and
losing focus or opening shell chrome clears the held mask so a later hover
cannot become a phantom drag.

Shell chrome that is open takes precedence: an overlay, context menu or desktop
dialog owns the pointer, paste and raw keys even though the Terminal window is
still focused underneath. The grid is still drawn under those surfaces, but it
is not an input target until they close.

Mode ownership ends at a screen boundary. Entering the alternate screen for
the first time, leaving it, `clear_screen`, and `ESC c` (full reset) clear all
mouse flags, SGR encoding, held buttons and bracketed paste. A program cannot
leave a pointer mode behind for the next application.

### Bracketed paste

The host delivers a paste as `InputEvent::Paste { text }`, a separate event from
key presses. While `?2004` is on, `TerminalApp::paste_bytes` wraps the complete
payload once as `ESC[200~ … ESC[201~`; while it is off, it sends the text
directly. Newlines and carriage returns stay bytes in one write instead of
being replayed as many Enter key events. A 20 KB payload is accepted as one
payload, not truncated. An embedded `ESC[201~` is removed from the text before
wrapping, so clipboard contents cannot terminate their own paste early. The
shell routes the event only when the Terminal owns focus and the PTY session is
alive; the write goes through `terminal_send_raw`, the same writer as raw keys.

The OS clipboard is an input-host responsibility. This crate defines and
routes `InputEvent::Paste`; it does not capture the Wayland/X11 clipboard into
that event on its own.

## Evidence

All tests below ran in this workspace on 2026-11-05:

| Check | Result |
|---|---|
| `cargo test --workspace` | **2,091 passed, 0 failed, 0 ignored** |
| `cargo test -p litric-shell --lib` | **820 passed, 0 failed** |
| `resizing_the_window_changes_what_stty_size_reports` | a real PTY reports `30 100`, then **`40 120`** after `Session::resize(120, 40)` |
| `a_real_vim_session_styles_cells_and_restores_the_primary_screen` | **VIM 9.1** ran in a real PTY with `-c 'set t_Co=256' -c 'syntax on'` on `terminal_screen.rs`; captured SGR output produced a non-default foreground/background cell, then `:q!` left `?1049l` and `echo restored-after-vim` ran in the same session |
| `a_real_pty_fixture_delivers_sgr_and_wide_cells_to_the_grid` | a PTY fixture emitted `31;1;4`, `38;5;196`, `38;2;1;2;3`, CJK `界`, Thai `ก` + combining `ั`; the grid asserted the exact colours, `界` width 2 with a continuation, `กั` width 1, and cursor column 12, then left the alternate screen |
| `alternate_sgr_stores_16_256_truecolor_and_attributes_per_cell` | 16-colour, 256-colour, truecolor, bold, underline and per-cell reset parsed from real bytes |
| `alternate_sgr_resets_all_attributes_and_does_not_leak_across_transitions` | bold/dim/italic/underline/reverse set, all cleared, `CSI m` reset, and alt entry/re-entry resets |
| `alternate_grid_places_wide_and_combining_text_through_the_parser` | parser-level wide/combining placement |
| grid tests | 15 tests for per-cell style, palette resolution, wide wrapping/overwrite, backspace/erase, resize repair, combining clusters and scroll regions |
| scroll-region tests | region-only line feed, reverse index, insert/delete lines, bounded explicit scroll and resize clamp |
| parser scroll test | `CSI 2;4r` plus `CSI S` kept rows outside the region still |
| view test | bold brightening, synthetic italic weight and reverse background resolution in `terminal_cell_style` |
| `a_raw_fixture_receives_arrow_bytes_and_restores_the_screen` | a shell fixture entered `?1049h`, set `stty raw`, a real PTY delivered `ESC [ A` (`1b 5b 41` from `od`), left `?1049l`, and `echo after-fixture` still worked |
| `ui_mouse_events_reach_a_real_program_in_each_tracking_mode` | three real PTY phases enabled `?1000`, then `?1002`, then `?1003` with `?1006`; `Shell::handle` pointer events produced byte-exact `ESC[<…M/m` files for press, release, drag, free motion and wheel, while the disallowed 1000/1002 motions were absent; `?1049l` left every mode off |
| `ui_paste_reaches_a_real_program_wrapped_only_when_enabled` | a real PTY fixture read a 20,024-byte paste with newlines and `\r\n`; with `?2004h` the file was exactly `ESC[200~` + payload + `ESC[201~`, and with `?2004l` it was the raw payload |
| mouse/paste model tests | tracking flags and DECRST, SGR press/release/drag/any motion, modifier bits, wheel, X10 offsets and release button, mode-reset held state, paste wrapping/off, newline preservation, 50,000-byte payload and embedded-terminator removal |
| overlay boundary test | with `?1000h`, `?1006h` and `?2004h` active, an open Quick Settings overlay left both a paste and a click out of the PTY while the terminal grid was still drawn |
| pointer capture test | a `ScriptedSource` packet stream (evdev records to `InputState`) pressed at cell (1,1), dragged above the grid and released there; the real PTY read the press, the drag and the release at the clamped edge, with no held button left |
| `evdev_mouse_reports_reach_a_real_child_and_stop_after_it_disables` | evdev records through `InputState` and `Shell` reached a real PTY child that enabled `?1000`, `?1002`, `?1003` and SGR `?1006` itself: byte-exact press, release, drag, free-motion and wheel reports carried the Shift modifier, including a drag released outside the grid clamped to cell (0,0); after the child disabled all four modes a timed read received no bytes, and a wheel after its PTY session reported closed left the report files unchanged |
| `pushing_into_an_empty_source_keeps_packet_order` | the evdev replay append path preserves record/`SYN_REPORT` order when the source starts empty; this was the cause of the first timeout, fixed in the source rather than by weakening the byte assertions |
| existing session tests | `cd`/environment, Ctrl+C stopping `sleep 10`, streaming before completion, and raw key bytes still pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `bash ci/verify.sh` | **39 pass, 0 fail, 3 skipped** — image build, boot smoke and idle measurement need mkosi/QEMU |

The vim and fixture tests use the **real PTY and real byte stream**, not a
parser-only string table. The fixture is a shell program, not a canned terminal
emulator, and the vim test is the installed VIM 9.1. `less`, `top`, `nano` and
`script` are installed on this host but were not used as evidence in this
batch.

## Security and privilege

- The shell command is a constant; user text is only written as data after the
  shell is running.
- The shell runs with the desktop user's authority. It is not a capability
  broker and is not a sandbox.
- The PTY helper is linked with `-lutil` and installed by `tools/build.sh` beside
  the Rust binary. `script(1)` is no longer required for the persistent path.
- The session keeps only the last 400 scrollback lines and 100 history entries;
  the alternate grid is bounded to 500×300 cells.
- The size file is 0600 and is not user-controlled; the C helper refuses
  out-of-range sizes instead of clamping the kernel's `winsize`.
- The file-safety linter has one reviewed deletion entry for
  `remove_size_file`; no deletion call is unreviewed and no lint or test is
  skipped.

## Not verified / limits

- The image was not built or booted here (no `mkosi`/QEMU and no physical
  display), so no assertion is made about the grid inside the LitricOS image,
  GPU presentation or audio. No screenshot of vim, less or top was captured.
- The `vim` evidence proves the PTY/SGR path and alternate-screen transitions,
  not every pixel of vim's frame.
- Sixel, full screen selection and GPU acceleration are not implemented. Mouse
  reporting (`1000`/`1002`/`1003`, X10 and SGR `1006`) and bracketed paste
  (`2004`) are implemented. OS clipboard capture is still an input-host
  responsibility: the shell routes `InputEvent::Paste` but does not read the
  clipboard itself.
- `display_width` is a deliberately small dependency-free table, not the whole
  Unicode East Asian Width/grapheme algorithm. Complex emoji ZWJ sequences,
  regional-indicator flags and unusual marks may be approximate.
- Italic is synthetic; no italic font face is bundled.
- The readiness marker is a fixed token and is display state, not a security
  boundary.
- If `litric-pty` is missing, the one-shot fallback remains and full-screen,
  styled-cell behaviour does not apply.
