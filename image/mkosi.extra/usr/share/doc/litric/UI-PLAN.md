# UI Plan — Standard (Windows-behavior) × Premium (macOS-look) for LitricOS

Status: PLAN · 2026-09-19 · feeds from docs/UI-RESEARCH-MACOS.md
Renderer reality: software canvas @1280×800, frame budget 50ms, no GPU blur —
all "glass" is alpha+tint approximation (honest per research §3).

## North star

One sentence: **every surface reads as a system material, not a colored skin** —
glass alphas + hairline highlight + 4-level elevation + smoothstep dock + centered
Spotlight, while behavior stays predictable-standard (double-click opens, right-click
menus, Esc cancels, F2 rename, Del→trash).

## Phases (each ships green + E2E-tested; no phase breaks previous)

### P0 — Design tokens (litric-design)
Add `glass` module: tier alphas (0.72/0.86/0.55/0.92/0.80), radius scale (8/10/14/22),
elevation z0–z4 + shadow specs, type roles (body 13 / title 17 / caps 11 / large 28),
accent set (#0A84FF/#30D158/#FF9F0A/#FF453A), hairline colors.
Accept: tokens compile; overflow audit still 0; PNG render unchanged until P1.

### P1 — Menu bar 24px glass + app menus
24px tall, glass tier 0.72 + wallpaper tint; left: app name + menus (File Edit View
Window Help) per HIG order; right: status cluster (Defender shield, net, clock 16px
glyphs, 35% when disabled). Dropdown menus: row 26px, shortcut right-aligned,
disabled 35%, dividers between groups.
Accept: E2E — click File → New Folder works; menus never under cursor; overflow=0.

### P2 — Window chrome: traffic lights + hairline
12px orbs at (12,12), gap 8, glyphs appear on hover only, doc-dirty dot on Files
when dialog open; window radius 10, hairline top inset highlight, shadow-md on
focused, none+dim on unfocused; green = maximize toggle.
Accept: E2E close/min/max via orbs; inactive window visually distinct; hit-testing
matches paint exactly (Hotspots audit).

### P3 — Dock: centered glass pill with smoothstep magnification
Bottom-center pill (alpha 0.55, radius 22, pad 6, gap 6); icons 44px base,
smoothstep(s)=s²(3−2s), range 88px, mag +28px; running-dot indicator; Trash at
right end separated by divider. Icons grow into free space above the dock —
row never clips (compute row height = base+mag).
Accept: E2E synthetic pointer sweep → sizes follow smoothstep exactly (golden
values); click launches; running dot toggles.

### P4 — Spotlight (centered launcher)
Ctrl+Space opens centered panel 640×56 + results (radius 14, tier 0.80, shadow-lg);
fuzzy search over apps/files/settings; ↑↓ navigate, Enter opens, Esc closes;
results ≤8 with kind badges.
Accept: E2E — type "def" → Defender first; Enter opens app; Esc restores focus.

### P5 — Files as Finder
Left sidebar 168px (Favorites: Home/Documents/Downloads/Desktop; Locations: volume)
tier 0.55; toolbar: back/forward, view toggles (icon/list), sort, action ⋯; path
breadcrumb + item count status bar; double-click navigates, single select, rename
F2, Del→Trash with undo banner.
Accept: E2E full journey create→rename→trash→restore; sidebar navigation equals
path navigation.

### P6 — Context menu polish
+1px right +1px down from cursor, flip at edges; groups separated by dividers;
no default item; 35% disabled; every item exists in menubar equivalent.
Accept: E2E — right-click desktop/icon/row/trash; menu never opens under cursor;
all actions perform.

### P7 — Motion within frame budget
Menu/popover open 120ms ease-in-out, window open 220ms, dock magnify instant
(cursor-driven, no tween); banner slide 160ms. All animations = pure functions of
t in [0,1] over monotonic clock — deterministic for tests.
Accept: PNG capture mid-animation = golden frame; frame budget ≤50ms verified.

### P8 — Notification banners → macOS style
Top-right stacked glass cards (radius 14, shadow-lg), app icon + title + body,
auto-dismiss 6s (critical persists), swipe-up-equivalent drag not needed v1.
Accept: E2E Defender critical detection shows card; expiry hides; click opens app.

### P9 — Settings + About (premium close-out)
Settings app window: sidebar+detail layout; About window: centered, glass, version
stamp. Theme switch (light/dark) flips token set, not code.
Accept: E2E toggle dark↔light; overflow audit 0 both themes.

### P10 — Lock Liquid Glass (ported 2026-10-12)
The prototype's lock screen (`screens/lock/liquid` in the design pack) as a real
surface: three colour fields drifting on 28/34/40 s keyframes over the
wallpaper; a glass clock that blurs what is behind it in software (two box-blur
passes through `canvas::blur_region`, the renderer's honest stand-in for
`backdrop-filter`), with **glow and bloom adjustable 0–100**; a clock that can
breathe; a widget row (uptime, network, battery) drawn from measured system
facts; and a customize sheet whose four tabs — Look, Effects, Glass, Widgets —
edit all of it live, with a reset. Motion is a preset (Still / Slow / Normal /
Fast) rather than a free number, and "Still" leaves the fields where they are
instead of snapping them home.

Accept: E2E — Space raises the password field, Enter submits once, the sheet
changes the look live, a machine that cannot verify a password refuses to lock
and says why; a locked frame is composed in full every tick, so its cost is
measured by `src/litric-shell/examples/render_bench.rs` in four states.

Status: **shipped on the host** — `src/litric-shell/src/lock.rs`. The numbers
are in `docs/EVIDENCE.md`; the booted-image half is still open
(`docs/design/wave1/04-authentication.md` §7).

### P11 — Dock that grows into the launcher (ported 2026-10-12)
The prototype's Desktop B (`screens/desktop/b-dock-morph`) as one surface rather
than two: the resting pill (launcher button, pinned icons, search bubble, clock)
morphs into a 900 px panel on a spring — same width centre, same bottom edge, so
the icons the user was aiming at do not move — carrying a title row, **a row of
action chips** (Terminal, Files, Settings, Activity, Trash, Defender: the
addition this system makes to the prototype, which put a single link to its lock
screen at the top right), the app grid, and the dock's own row at the bottom of
the panel.

Search is the same surface contracted rather than a second panel: typing filters
the grid, the pinned icons fold away, and an X collapses it — a launcher with no
matches still offers a way back.

Accept: E2E — the pill grows, a tile opens its app, a chip performs what its
label says, typing filters, Esc and the X collapse the spring back to rest.

Status: **shipped on the host** — `views::dock` + `shell::SHORTCUT_CHIPS`.

## Sequencing & risk

- P0→P1→P2 first: they define the visual grammar everything else reuses.
- Dock (P3) and Spotlight (P4) are the two "wow" items users will name — do them
  only after chrome is right.
- Risk: alpha-over-alpha banding on canvas → mitigate with dithering-free tone
  selection in litric-design (existing tone ramp).
- Risk: magnification math in integer canvas → keep float internally, round once
  at blit; test golden values at cursor x = {0, 22, 44, 88, 132}.
- Every phase ends: cargo test --workspace + overflow audit 0 + PNG evidence.

## Non-goals v1
GPU blur/refraction (renderer constraint — P10 approximates the glass with two
box-blur passes, and the tint and specular layers are what carry the look when
it is off), multiple Spaces, window snapping grid beyond maximize, Dock on
screen edges other than bottom.
