# BUG-0032 — Two sidebars with two designs, and a Settings list nothing could scroll

```
Bug ID        : BUG-0032
Severity      : S2 (HIGH) — on a short window the last Settings pages existed in
                the model but could not be reached, clicked or drawn; the Files
                and Settings panes also taught two different sidebars
Class         : UI / LAYOUT + COMPONENT (two implementations of one pattern, a
                fixed percentage with no scroll, a 20 px row below every target)
Subsystem     : src/litric-shell/src/views.rs (`draw_files`, `draw_settings`),
                src/litric-shell/src/ui.rs (`ui::sidebar`),
                src/litric-shell/src/apps.rs (`FilesApp::places_scroll`),
                src/litric-shell/src/settings.rs (`Settings::sidebar_scroll`),
                src/litric-shell/src/shell.rs (wheel routing, `Hot` panes)
First Observed: 2026-10-27, F.10 findings 42–44 in `Research-Deep-all` Part 13
Status        : FIXED (seven regression tests)
```

## Why this exists

Two windows drew a sidebar, and they shared nothing:

| | Files places pane | Settings sidebar |
|---|---|---|
| Width | **132 px fixed** | **36 % of the window body** (360 px at 1000, 504 at 1400) |
| Rows | **20 px**, 8 px text, no icon | 30 px, `ui::list_item` with icon and accent marker |
| Selection | an 8 % fill — colour only | fill **plus** a 3 px marker |
| Heading | "Quick access" | none |
| Scroll | none | none |

Both were wrong in a different direction. 132 px is narrower than libadwaita's
180sp minimum and Windows' 320 DIP expanded pane; 36 % is wider than
libadwaita's 280sp maximum at any window this shell opens, and it grew with the
window instead of stopping. The row heights differed by 10 px, and the Files
rows were below every published target, including WCAG's 24 px.

The Settings list is the longer one: 13 pages × 30 px + 12 × 2 px gaps =
**414 px** of rows. At the default 720 × 520 window the body is tall enough, but
any window the manager squeezes under about 440 px of body height drew the last
pages outside the pane. Nothing moved them: no wheel, no scrollbar, no key. The
hotspots were registered from the same loop, so a page drawn past the pane could
even answer a click outside it.

## Fix

**One component: `ui::sidebar`.** Width, row height, gap, heading, scroll
clamping, clipping and the overflow indicator live in one module, and both
windows call it and paint the pane with the same `Material::Elevated` surface
(Settings used to paint 16 % black). The row itself is still `ui::list_item`, so a sidebar row and a
list row cannot drift apart again.

**A width policy, not a number.** `sidebar::width(body_w)` is 25 % of the body,
clamped to libadwaita's **180–280 px**, and it never takes so much that the
content column drops under **320 px** (the width the Files table needs): below
600 px the pane falls back to a third of the body. Files is no longer 132 px at
every width, and Settings is no longer 504 px at 1400.

**A real scroll.** Each pane owns an offset (`FilesApp::places_scroll`,
`Settings::sidebar_scroll`). `sidebar::clamp` bounds it against
`content_height - view`, and the view stores the clamped value, so the offset can
never describe a position that does not exist. The heading stays fixed while the
rows move; the rows are clipped with `Canvas::push_clip`; and a 3 px indicator
appears on the pane's trailing edge only when the rows are taller than the pane.
The first indicator was 2 px at 45 % hard against the divider: the Settings frame
showed it was indistinguishable from the pane's background and read as the
divider itself, so it was widened, inset 4 px and given a visible track.

**The wheel belongs to the pane.** The pane is registered as a hotspot behind its
rows (`Hot::SettingsSidebarScroll`, `Hot::Files(FilesHot::PlacesScroll)`), so the
wheel over the pane — including its empty space — scrolls the list, and a row
under the pointer wins the hit test because it is pushed after the pane.

**A covered pane does not take the wheel.** Hotspots come from the last frame,
which drew every window, including the ones behind. Wheel routing therefore asks
the window manager which window is on top (`wheel_owner`) before scrolling: a
terminal in front of a Settings window now keeps its own wheel events.

**What is clickable is what is visible.** The hotspot a row registers is its
intersection with the rows area, so a row half-scrolled under the fixed heading
does not answer a click on the heading, and a row clipped at the bottom does not
reach past the pane.

## Regression tests

`src/litric-shell/src/ui.rs`, `ui::tests`:

* `the_sidebar_width_policy_is_bounded_at_both_ends` — 1400 → 280, 1000 → 250,
  740 → 185, 600 → 180, 480 → 160 (a third), 0 → 0, and never wider than the
  body at any width.
* `the_sidebar_scroll_stops_at_the_last_row` — 13 rows in a 200 px pane: a
  negative offset clamps to 0, an over-scroll clamps to `max_scroll`, and at the
  maximum the content's end sits exactly on the pane's padded bottom.
* `a_sidebar_draws_only_the_rows_inside_its_pane` — at a 10,000 px offset only
  the last rows are returned, the last one is among them, and every returned
  rectangle is inside the rows area (the heading is not covered).
* `a_sidebar_says_there_is_more_only_when_there_is` — no indicator pixels in a
  pane that fits its rows; indicator pixels in a pane that does not.

`src/litric-shell/src/shell.rs`, module `sidebar_scroll`:

* `a_squeezed_settings_window_can_reach_its_last_page` — a 640 × 300 Settings
  window: the last category is not a hotspot; the wheel over the pane scrolls
  it; the last category then is a hotspot and a click selects its page.
* `a_squeezed_files_window_can_reach_trash` — the same for Files at 640 × 150:
  Trash is unreachable, the wheel reaches it, and clicking it navigates there.
* `a_covered_sidebar_does_not_steal_the_wheel` — a terminal opened in front of
  the pane: the wheel over the pane leaves `sidebar_scroll` at 0.

## Verification

* `cargo test --offline -p litric-shell --lib sidebar` — 7 passed.
* `cargo test --offline` (the whole workspace) — every suite passed, 0 failed;
  `litric-shell` alone is 585 lib tests plus its 32, 10, 9, 4 and 3-test
  integration suites.
* `cargo clippy --offline -p litric-shell --all-targets` — no warnings.
* `cargo fmt --all -- --check` — clean.
* `ci/verify.sh` — **30 pass, 0 fail, 3 skipped**; the skips are the image build
  (no mkosi), the boot smoke test and the idle RAM/CPU measurement (no
  qemu-system-x86_64) on this host, each with its reason printed in
  `docs/evidence/2026-10-27/verify-f10.log`.
* Golden frames re-dumped with `frame_dump --fonts build/fonts/noto`; the Files
  (`04-files`, fingerprint `0xcd28f3b57732ec2f`) and Settings (`05-settings`,
  `0x2e1156099c2853c7`) frames show a single 180 px pane in the 720/760 px
  windows, with icons, the accent marker, and the last Settings page inside the
  window.
* Log: `docs/evidence/2026-10-27/f10-sidebars.log`.

## Residual risk

* **No compact, minimal or collapsed mode.** Windows has four NavigationView
  modes with an Auto policy at 1008/641/640 px; neither pane here collapses to
  icons, a menu button or an overlay. The width policy stops the pane from
  growing past 280 px, but a 500 px window still shows a 180 px pane beside
  320 px of content.
* **No resize handle.** Apple's split view and GNOME's split view both allow the
  divider to move; this policy is fixed by window width.
* **No arrow-key model.** Each row is still one Tab stop, with Enter to select;
  Up/Down inside the pane are not implemented (`F.10` finding 44 / `G.14`).
* **No sections in the Settings list.** Thirteen pages remain one undifferentiated
  column; Windows and GNOME both support group headers.
* **A page chosen from search or a deep link does not scroll the pane to its row.**
  The page opens correctly, but on a short window its category row can stay
  outside the view until the user scrolls. Nothing indicates the selection is
  above the visible rows.
* **The Files pane's 30 px rows are still below the 40–44 px touch guidance**, in
  line with the rest of the shell's list rows; raising only this pane would
  reintroduce the inconsistency this record removed.
* Everything above is verified on the host by unit tests and frame dumps. No
  LitricOS image was built or booted for this record.
