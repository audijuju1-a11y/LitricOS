# 27 — The plan the UI/UX research implies

**Created:** 2026-10-26, on top of `565ea4f`.
**Inputs:** `Research-Deep-all/` — the six research parts, the coverage checklist
and the five registers — read together with the phase pack (`00-MASTER-WORKFLOW.md`
… `15-APPENDIX-*`), [the gate matrix](25-beta-1.0-gate-matrix.md),
[the execution plan](26-execution-plan.md), `docs/DESKTOP-2.0.md` and the
current shell.

**Status:** a plan. Nothing here is evidence, and no row may be quoted as a
result.

---

## 1. What the research actually is

The corpus is an **inventory of other systems' surfaces**, prepared to be
implemented from — not a design document and not a specification.

| File | What it holds | Size |
|---|---|---|
| `Research-Deep-all/LitricOS UI UX Research Part1.md` | baseline versions, master coverage checklist, source plan, validation plan skeleton | 915 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 2 Windows 11 26H2 Settings.md` | Windows Settings catalogue `WB.01`–`WB.12`, persistence map, `ms-settings` URI register | 4,587 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 3  Windows 11 Shell and File Explorer.md` | shell + Explorer catalogue `WA.01`–`WA.16`, shortcut register, Win10/7 deltas | 8,646 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 4  Fluent and WinUI Design System.md` | Fluent token/geometry/material/state rules `WC.01`–`WC.13` | 3,460+ lines |
| `Research-Deep-all/LitricOS UI UX Research Part 5  macOS 27 System Settings.md` | macOS Settings catalogue `MD.01`–`MD.28+` | 2,700+ lines |
| `Research-Deep-all/LitricOS UI UX Research Part 6  macOS 27 Desktop Model.md` | menu bar, Dock, Finder, window model, Spaces, Spotlight, Control Center, dialogs, input, session | 5,726 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 10 F Shared Component Catalogue.md` | chapter `F`, the shared component catalogue: the four controls the shell has (`F.01` buttons, `F.03` switches, `F.06` sliders, `F.19` lists) compared against `src/litric-shell/src/ui.rs` | 422 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 11 F Text Progress Search and Errors.md` | chapter `F`, continued: text fields (`F.02`), progress (`F.18`), search fields (`F.23`) and errors/validation (`F.26`), compared against the field renderer, the input path and the five search surfaces | 457 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 12 F Checkboxes and Radio Controls.md` | chapter `F`, selection controls: checkboxes (`F.04`) and radio controls (`F.05`), compared against the hand-drawn Files checkbox and the chip-based single-select surfaces | 331 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 13 F Tabs Sidebars and Toolbars.md` | chapter `F`, layout containers: tabs (`F.08`), sidebars (`F.10`) and toolbars (`F.11`), compared against the browser tab strip, the two sidebars and the three command rows | 329 lines |
| `Research-Deep-all/LitricOS Coverage Checklist.csv` | the ledger behind all of it | 597 items |
| `Research-Deep-all/LitricOS Source Register.csv` | every source URL with tier and status | 3,979 rows |
| `Research-Deep-all/Fluent Token Register.csv` | measured Fluent tokens | 3,553 rows |
| `Research-Deep-all/macOS Design Token Register.csv` | measured macOS tokens | 1,805 rows |
| `Research-Deep-all/Windows Keyboard Shortcut Register.csv` | Windows shortcuts | 365 rows |
| `Research-Deep-all/macOS Keyboard Shortcut Register.csv` | macOS shortcuts | 208 rows |
| `Research-Deep-all/Windows ms-settings URI Register.csv` | Settings deep links | 237 rows |
| `Research-Deep-all/macOS System Settings URL Register.csv` | Settings deep links | 209 rows |

### 1.1 How complete it is — its own numbers

The coverage checklist is the research's own audit, and it is honest about being
unfinished:

| Status | Items | Share |
|---|---:|---:|
| `COMPLETE` | 18 | 3 % |
| `PARTIAL` | 396 | 66 % |
| `PENDING` (not yet researched) | 179 | 30 % |
| `NOT APPLICABLE` | 3 | — |
| `NOT FOUND` | 1 | — |

By section, the shape is: Windows shell (`WA`) 97 partial, Windows Settings
(`WB`) 141 partial, macOS desktop (`MC`) 62 partial, macOS Settings (`MD`) 38
partial, Fluent (`WC`) 12 partial, tokens (`G`) 19 partial — while the whole
Linux-desktop comparison (`LG`–`LR`, eight systems × 15 items = 120), the shared
component catalogue (`F`, 14 of 27), workflows (`H`, 21), technical feasibility
(`I`, 12) and the final audit (`J`, 9) are **still `PENDING`**.

That single table decides the plan's shape: **the research cannot be treated as
a finished specification**, so the build must proceed on the parts that are
settled (Windows shell/settings, macOS desktop/settings, Fluent values) while the
rest is completed in parallel, and no product row may cite a `PENDING` item as
its source.

### 1.2 What the corpus is worth to the build

Three things, and only three:

1. **An inventory of surfaces** — what a person expects to find, page by page
   (`WB.01.01.01` HDR, `MD.10` Accessibility, `WA.07.02` toast lifecycle).
2. **Measured values** — 5,358 design tokens, 573 shortcuts, 446 settings deep
   links. These are numbers, not opinions, and they can be checked.
3. **Behaviour rules** — states, focus visuals, motion timing, empty/error
   states (`WC.10`, `WC.11`, `WC.13`), scroll and grouping behaviour.

What it is **not**: a licence to copy assets, icons, typefaces or trademarks. The
phase pack's hard rule applies unchanged — clean-room behaviour, never
proprietary implementation detail (`00-MASTER-WORKFLOW.md` §8).

---

## 2. The gap this plan closes

| Dimension | Research corpus | LitricOS today | Gap |
|---|---:|---:|---:|
| Settings surfaces | ~237 Windows pages + ~209 macOS panes | 13 pages | ~30× |
| Settings deep links | 446 | 0 | all |
| Shell shortcut actions | 573 across two systems | 36 | ~16× |
| Design tokens | 5,358 | 22 token fields | ~240× |
| Built-in apps | 8 categories in each system | 8 | comparable in count |
| Shell surfaces | ~26 catalogued | desktop, bar, dock, launcher, overview, panels | partial |

The numbers are not a target — a desktop does not need 5,358 tokens or 237 settings
pages. They are a **checklist to decide against**, one row at a time, and the plan
below is built so that every decision is written down rather than implied.

---

## 3. The artefact that makes this tractable: a parity ledger

Building 1,000 controls without a ledger is how a settings application becomes
1,000 special cases. The first deliverable is therefore not a page; it is
`docs/parity/` — one row per research item that LitricOS might implement.

### 3.1 Schema

| Column | Meaning |
|---|---|
| `research_id` | `WB.01.01.03`, `MD.10`, `WA.07.02`, `WC.10`, … the exact checklist id |
| `platform` | `windows-26h2`, `macos-27`, `gnome-51`, … |
| `surface` | the LitricOS surface that would own it (`settings/appearance`, `shell/notifications`, `apps/files`) |
| `decision` | `ADOPT` (same behaviour), `ADAPT` (different because of our constraints), `REJECT` (with a reason), `DEFER` (needs a backend we do not have) |
| `justification` | one sentence. Mandatory for `ADAPT`, `REJECT`, `DEFER` |
| `backend` | the real API or store that makes it functional; `none` is allowed only with `REJECT`/`DEFER` |
| `store_key` | the `settings.conf` / registry key, when it is a setting |
| `evidence` | `STATIC` / `UNIT` / `CONTRACT` / `VM E2E`, with the test name |
| `milestone` | `M1` … `M6` |

Rules:

- A row with `decision = ADOPT` and `backend = none` fails CI. **No inert
  control** — the gate matrix's own condition.
- A `REJECT` without a justification fails CI.
- A `PENDING` research item cannot appear in the ledger at all; it has no
  source. This keeps unfinished research out of the product.
- The ledger is the only place a "we support X" claim may come from.

### 3.2 Where it lives

a machine-checked `ledger.csv` and a human index, both planned under
`docs/parity/`. A new checker, `lint-parity.py` in `ci/` (not written yet),
enforces the three rules above, with a self-test and a negative control, in the
style of the existing `ci/lint-unit-binaries.py`.

---

## 4. Work packages

Each package names the research it draws from, the surface it lands on, and what
proves it. Milestones are in §5.

### WP-01 — Settings foundation (M1)

- **From:** `WB` §11.1 (shell, navigation, search, keyboard), `MD.01` (sidebar),
  `Windows ms-settings URI Register.csv` (deep-link shape).
- **Build:** a typed settings registry (schema version, per-key owner, default,
  validator, store), a searchable index over titles and synonyms, keyboard
  navigation (arrows, Home/End, Enter, Esc), and `litric://settings/<path>` deep
  links that resolve through the registry.
- **Why the registry first:** it is the only way the later page work stays
  affordable; the shell already proved the pattern in
  `src/litric-shell/src/prefstore.rs`.
- **Acceptance:** every page is reachable from the registry; a search for
  "night light", "wallpaper", "wifi" finds the owning page; a deep link opens
  the exact row; a key with no backend cannot be registered.

### WP-02 — Appearance and personalization (M1)

- **From:** `WB.04`, `MD.11`, `MD.14`, `MD.17`, `WA.01.01` (wallpaper, slideshow,
  accent), `WC.02` (colour tokens).
- **Build:** wallpaper sources (image, folder, solid, accent-derived), slideshow
  interval and order, per-monitor later; accent modes (manual + from-wallpaper);
  theme scheduling (light/dark by time); dock/bar size, position, auto-hide.
- **Acceptance:** each row persists across restart; contrast gate still passes
  for every accent × theme pair; slideshow advances under the shell's real timer.

### WP-03 — Display, sound, power (M2)

- **From:** `WB.01.01` (display, HDR, night light, scale), `WB.01.02` (sound,
  volume mixer), `WB.01.05` (power), `MD.15`, `MD.20`.
- **Dependency:** the DRM/KMS decision in
  [the execution plan](26-execution-plan.md) §4 W8. On framebuffer output, scale
  and night light can be real, HDR and refresh rate cannot — and must say so.
- **Acceptance:** every control either changes a measured fact or is labelled
  unavailable with the reason from the probe.

### WP-04 — Notifications, focus and DND (M2)

- **From:** `WA.07` (toast lifecycle, action buttons, history, priority, DND),
  `WB.01.03`, `WB.01.04`, `MD.19`, `MD.21`.
- **Build:** the notifications daemon planned as W3 in the execution plan, with
  the lifecycle states the research documents (banner timeout, action buttons,
  persistence in the centre, per-app priority), plus focus sessions with an end
  time.
- **Acceptance:** a notification raised while the shell is down arrives on
  start; DND suppresses banners and not the centre; every action is keyboard
  reachable.

### WP-05 — Quick settings / Control Centre (M2)

- **From:** `WA.06` (tiles, edit mode, sliders, sub-panels, media), `MD.13`,
  `MC.07`.
- **Build:** volume and brightness sliders over the real providers, network and
  Bluetooth tiles with honest state, media controls driven by the audio
  provider, and edit mode that persists the tile order.
  *Landed:* the volume controls (2026-10-26, session 5), and the network and
  Bluetooth tiles (same day, session 6) — a link switch and a re-read over
  `NET LINK`/`NET RECONFIGURE`, and a radio switch over `BT POWER` that only
  appears where a radio, an unblocked rfkill and a Bluetooth service exist, with
  both readings taken from the machine (`AUDIO LEVEL`, `BT STATUS`) rather than
  remembered. *Remaining:* the brightness slider, which has no provider on this
  image (no backlight, `DEFER` in the ledger), the media controls, which need a
  player to talk to, and edit mode with a persisted tile order.
- **Acceptance:** no tile without a provider; dragging a slider changes the
  measured value; tile order survives restart. The first is now a test —
  `the_panel_offers_no_control_where_the_machine_reports_no_provider` — and the
  panel's height is measured from its sections, so a tile added without room for
  it fails `no_panel_row_is_drawn_outside_the_panel` rather than being drawn off
  the bottom.

### WP-06 — Shell chrome parity (M3)

- **From:** `WA.02` (Start/launcher), `WA.03` (taskbar: grouping, jump lists,
  previews, overflow, badges), `MC.01`, `MC.02`.
- **Rule:** this is where `ADAPT`/`REJECT` will be most common. No live window
  thumbnails until the canvas has render-to-texture — the shell already refuses
  to fake them (`docs/DESKTOP-2.0.md` §1).
- **Acceptance:** grouping, jump lists, badges and overflow each have a written
  decision; whatever is adopted has a test at the shell level.

### WP-07 — Window management (M3)

- **From:** `WA.09`, `MC.04`, `MC.05`, and `docs/DESKTOP-2.0.md` §5 item 1.
- **Build:** snap layouts picker, per-application window cycling, stage-manager
  style set, rules engine for placement.
- **Acceptance:** the picker produces the geometry the manager already supports;
  keyboard-only path exists for every placement.

### WP-08 — Files and Explorer parity (M3)

- **From:** `WA.13`, `WA.14` (dialogs), `MC.03` (Finder).
- **Build:** tabs, breadcrumb editing, drag and drop, details pane, properties
  sheet, sort/filter, archive operations, conflict dialogs — over the files
  daemon from W1 of the execution plan, never over in-process writes.
- **Acceptance:** every operation crosses the daemon; a failure shows the real
  errno class; conflicts never overwrite silently.

### WP-09 — OOBE, accounts, lock (M4)

- **From:** `WA.10`, `WA.11`, `MD.23`, `MD.25`, `MD.26`.
- **Build:** first-run parity beyond the current wizard (language, account,
  password, privacy choices, network), lock screen options, session switching.
- **Acceptance:** a clean image reaches the desktop through the OOBE without a
  terminal; the lock cannot be bypassed; no default credential exists.

### WP-10 — Accessibility and input (M4)

- **From:** `MD.10` (the longest single pane in the corpus), the accessibility
  section of `WA.09`, `G` token/accessibility rows, `H` workflows (needs Track R).
- **Build:** a semantic accessibility tree, a screen-reader bridge decision,
  keyboard-only path per surface, IME/Thai typing, sticky keys, zoom, colour
  filters, reduced motion honouring `prefers-reduced-motion` equivalent.
- **Acceptance:** "every user-facing action is reachable by keyboard alone"
  (GATE 06) becomes a test, not a claim.

### WP-11 — Design system consolidation (M1, then continuous)

- **From:** `WC.01`–`WC.13`, both token registers, `G`.
- **Build:** map the 5,358 measured tokens to the 22 LitricOS token fields with
  a written decision per family (adopt value, adapt to our scale, ignore with
  reason); add the control states, focus visuals and loading/empty/error
  patterns the research found; a golden frame per surface.
- **Acceptance:** no shell colour/space/radius literal outside the token file
  (CI check); every new surface ships a golden frame in the same commit.

### WP-12 — System tools and permission prompts (M5)

- **From:** `WA.12` (UAC analogue), `WA.14` (common dialogs), `WA.15` (system
  tools: Task Manager, Terminal, clipboard history).
- **Build:** a capability consent prompt modelled on UAC's structure but backed
  by `litric-caps`, dialog components, clipboard history over the real store.
- **Acceptance:** a denied capability produces a prompt that names the app, the
  capability and the consequence; allow-once / allow-always / deny are all
  real decisions.

---

## 5. Milestones

| Milestone | Contents | Exit criterion | Needs builder VM |
|---|---|---|---|
| **M1** | WP-01, WP-02, WP-11 foundation | registry + search + deep links + appearance rows, all persisted and contrast-gated | no |
| **M2** | WP-04, WP-05, WP-03 | notifications daemon, control centre over real providers | partly (sliders need a guest) |
| **M3** | WP-06, WP-07, WP-08 | chrome, window management and Files over the files daemon | partly |
| **M4** | WP-09, WP-10 | OOBE + accessibility tree + keyboard-only proof | yes |
| **M5** | WP-12 + remaining settings families | capability prompts and tools | yes |
| **M6** | research close-out + parity audit | every ledger row decided and evidenced; research `PENDING` = 0 | no |

Milestones are ordered by dependency, not by how visible they are. M1 is first
because every later row depends on the registry existing.

---

## 6. Track R — finishing the research

The corpus is 30 % unresearched and 66 % partial. The build may not cite it, so
completion is work, scheduled beside the build:

### Coverage today (2026-10-26)

The checklist holds 597 items; 414 are `COMPLETE` or `PARTIAL` and therefore
citable, and `docs/parity/ledger.csv` decides 389 of them -- every one that is a
surface. Per chapter:

| Chapter | Citable | Decided |
|---|---:|---:|
| `WB` Windows Settings (26H2) | 143 | 143 |
| `WA` Windows 11 shell & system UI | 98 | 98 |
| `MC` macOS 27 desktop model | 64 | 64 |
| `MD` macOS 27 System Settings | 39 | 39 |
| `G` tokens & accessibility | 19 | 19 |
| `WC` Fluent / WinUI design system | 14 | 14 |
| `MG` macOS HIG | 11 | 11 |
| `F` shared component catalogue | 14 | 1 (13 newly citable; their decisions are not written, 14 are `PENDING`) |
| `CH00` version baseline | 12 | 0 (not a surface; see the ledger README) |

The 389 rows split 169 `ADAPT`, 126 `DEFER`, 88 `REJECT`, 6 `ADOPT`. `DEFER` is
a promise with a milestone on it, and the promises are the build's own backlog:
`M3` carries the settings families that need the files daemon, `M4` the session
and accessibility ones, `M5` the capability and privacy ones, and `M6` the ones
that will have to become `REJECT` or be built.

What remains outside the ledger is research, not decisions: the `PENDING`
chapters (`F` 14, `H` 21, `I` 12, `J` 9 and eight Linux desktops of 15) cannot
be cited until their status is final, so they are Track R's work and not rows.

### Track R queue, first pass (2026-10-26)

179 checklist items are `PENDING`, and none of them may be cited. The first pass
counted any heading that *contained* an item's title and found 56; that number was
wrong, and the chapter `F` review below is how it was found out. Counting only
headings that name the item -- its id, or its whole title -- leaves 24:

| Chapter | `PENDING` | A section names it |
|---|---:|---:|
| `F` shared component catalogue | 14 | 5 |
| `H` workflows & edge cases | 21 | 2 |
| `I` technical feasibility | 12 | 1 |
| `J` source register & audit | 9 | 1 |
| each of `LB`–`LX` (8 desktops) | 15 | 2 |

A section is **not** a status either: it says someone wrote something under that
name, not that it answers the item with a source. So the queue is 15 reviews, and the other 164 items are research that has not
been done.

#### Chapter `F`, reviewed (2026-10-26)

`F` is the case the substring count got wrong: it reported 12 of 28 items as
written up, and the review found 1. The corpus's `F` section is the checklist
table itself (`Part1` §"F — Research F — Shared component catalog"): 27 rows of
`PENDING` and no prose. What matched were other chapters' headings that happened
to contain the word -- "Buttons" in a toast-lifecycle heading, "Menus" in a File
Explorer heading -- which is why a substring is not evidence.

The one real section is `F.16 Sheets`: `Part6` §MC.04.06 carries Apple's HIG
guidance with eight elements and four behavioural guidelines. That is one
platform of the three this catalogue needs, so the item is `PARTIAL` with that
note, and its row is in the ledger. The other 27 carry the note
"the corpus has no section for this item"; a review that ends by writing down
what is missing is worth more than one that guesses.

#### Part 10: the four components the shell already has (2026-10-26)

Batch B9 entered early and partially, on the four `F` items that can be checked
against running code today rather than against another system's documentation.
`Part 10` compares Windows 26H2, macOS 27 and GNOME 51 guidance with
`src/litric-shell/src/ui.rs` and sets `F.01` buttons, `F.03` switches, `F.06`
sliders and `F.19` lists to `PARTIAL`.

It found eight things, and none of them is a ledger row yet: a disabled **off**
switch draws exactly like an enabled one; `ui::slider` and `ui::icon_button` have
no call site at all, so the component layer contains a slider the product does not
have and an icon button every icon-only control bypasses; keyboard focus is drawn
as the hover picture while `State::Focused` stays unreachable; the state-matrix
test covers two of the four controls; and the shell's Settings rows are 30 px
tall against every platform's minimum target.

Why this is `PARTIAL` and not `COMPLETE`: Apple's component pages return titles
and summaries to a reader, so the macOS column is the weakest of the three and one
item's detail came through a third-party mirror. Why it is citable at all: the
comparison is against code in this repository, so a reader can open the file and
disagree with a specific sentence.

#### Part 11: text, progress, search and errors (2026-10-26)

Batch B9 continued on the four `F` items that are either the last two primitives in
the component layer or the places where a primitive is missing from the product.
`Part 11` compares Windows 26H2, macOS 27 and GNOME 51 guidance with
`ui::text_field`, `ui::progress`, the shell's input path and its five search
surfaces, and sets `F.02` text fields, `F.18` progress indicators, `F.23` search
fields and `F.26` errors/validation to `PARTIAL`.

It found sixteen things, and none of them is a ledger row yet:

| Finding | Evidence |
|---|---|
| The shell has a text editor and does not use it. `TextEdit` (caret, selection, Home/End, word delete, 8192-char limit) reaches **one Backspace in one dialog**; every other field is a `String` appended at the end | `apps.rs` line 1832, `shell.rs` line 4774, `push_str` at six sites |
| The field renderer takes `(text, placeholder, focused)` and cannot show a caret index or a selection, so the one dialog that has the model still draws its caret at the end | `ui.rs` line 599 |
| A magnifier is drawn in the New Folder and Rename fields, and the generic field is the only search affordance the two filter fields have | `views.rs` lines 339, 1450 |
| No filter has a clear button; the launcher's ✕ is `Hot::DockCollapse` and closes the launcher instead of clearing the query | `shell.rs` line 5521 |
| `Ctrl+F` in Files focuses the table, not the filter | `shell.rs` line 4896 |
| Progress is one determinate bar and a `total == 0` guard that draws an empty track during a live scan; there is no indeterminate form and no spinner anywhere | `views.rs` line 4072 |
| Validation is a round-trip, and a failure replaces the form, loses the typed name, and shows an elided one-line detail with a danger-toned OK button | `views.rs` lines 1405, 1435 |

Why these are `PARTIAL` and not `COMPLETE`: the macOS column is again the
weakest (every Apple page returned a title and a summary), and none of the four
has a decision, a backend or a test. Why they are citable at all: each item's
claims name a line in this repository that a reader can open.

#### Part 12: checkboxes and radio controls (2026-10-26)

The last pair of selection controls, and the pair with the strangest inventory in
the product. `Part 12` compares Windows 26H2 and GNOME 51 at page depth (Apple's
checkbox and radio pages did not load, so the macOS column is an API summary and
nothing more) with the one hand-drawn checkbox in the shell and the chip-based
single-select surfaces that stand in for a radio group. `F.04` checkboxes and
`F.05` radio controls become `PARTIAL`.

| Finding | Evidence |
|---|---|
| The product's only multi-select checkbox is **a circle**: a 12 x 12 box at radius 6, with an 18 x 18 hit target | `views.rs` lines 1243, 1250, 1281 |
| No indeterminate/mixed state and no header/select-all box; `Ctrl+A` is keyboard-only and has no control | `views.rs` lines 1243-1280; `apps.rs` line 653 |
| Space does not toggle a checkbox — the only Space in the shell is the lock screen's wake; Enter activates | `shell.rs` line 3933 |
| Two Tab stops per Files row (the row and its box), so a 24-row folder is 48 stops | `shell.rs` line 445; `views.rs` lines 1242, 1277 |
| **No radio control exists at all**, and no dot-in-ring drawing; single-select choices are chips | grep: no radio control, no `stroke_circle` |
| The same three-way choice has two chip designs, and the selected chip is colour-only even though the start-page comment claims a check glyph | `views.rs` line 1985 vs lines 1986-2010; lines 2064-2100 |
| Three chips are three Tab stops, with no arrow navigation and no selection-follows-focus | both chip loops; `shell.rs` line 445 |
| "Reduce motion" is a switch in Quick settings and a state-in-label button in Settings and first-run | `views.rs` lines 2930, 4907, 5654 |

The shape mismatch is the item's headline: a control whose form says "choose one"
and whose behaviour says "choose several", in a table that also has a
single-selection state. It is recorded, not changed.

#### Part 13: tabs, sidebars and toolbars (2026-10-26)

The three containers the shell draws most often, and the three with five separate
implementations between them. `Part 13` compares Windows 26H2 and GNOME 51 at page
depth with the browser's tab strip, the Files places pane, the Settings category
list and the three command rows, and sets `F.08` tabs, `F.10` sidebars and `F.11`
command bars/toolbars to `PARTIAL`.

| Finding | Evidence |
|---|---|
| The tab strip has **no overflow policy**: fixed 150 px tabs, no scroll, no cap, no clip, so the `+` button can leave the window | `views.rs` lines 1575–1621 |
| **No tab keyboard model**: no Ctrl+T/W, Ctrl+Tab, Ctrl+1–9 or arrow movement; Ctrl+T is Settings' theme cycle | `shell.rs` line 4699; `views.rs` lines 1608, 1618 |
| The active tab differs by fill only — same text, weight and size, no accent edge | `views.rs` lines 1584–1592 |
| The tab context menu has two items; no duplicate, pin, mute, move-to-window, reorder or tear-out | `apps.rs` lines 1738–1741 |
| **Two sidebars, two designs**: 132 px / 20 px rows / fill-only vs 36 % / 30 px rows / icon + marker | `views.rs` lines 957, 2337 |
| The Settings sidebar is 36 % with no collapse, resize or scroll — 414 px of rows | `views.rs` lines 2337–2360 |
| Neither sidebar has an arrow-key model; one Tab stop per row | `shell.rs` line 445 |
| **Three toolbars, three overflow policies**: wrap + scale, silent drop, fixed widths | `views.rs` lines 1702–1723, 1083, 4172 |
| Files drops buttons that do not fit and draws inapplicable actions as enabled | `views.rs` lines 1083, 1102–1136 |
| Toolbars are label-only at 22–24 px with no tooltips, access keys or separators | `views.rs` lines 1676–1755 |
| Fixed-width confirm labels ("Empty permanently?") overhang their 112 px button | `views.rs` lines 4189–4200 |

The container finding is structural: the shell's component layer exists to stop two
windows from inventing two sidebars, and both pairs did it anyway. Why `PARTIAL`:
Apple's pages returned titles and summaries again, and no layout test at two window
widths has been run, so each overflow claim is a reading of a loop rather than a
resized frame.

Each batch produces ledger-ready rows, not prose: a research batch is done when
its checklist status is final and its rows carry evidence tiers.

---

## 7. Governance

1. **One source of truth per claim.** A surface's behaviour is described in the
   ledger row; the research file is the evidence, not the requirement.
2. **No inert control.** The planned parity checker fails an `ADOPT` row with
   no backend, and the gate runs it with a self-test once it exists.
3. **No token literal outside the token file.** A new CI check extends the
   existing design lint to the shell.
4. **Every surface ships with a golden frame** in the same commit, so visual
   regressions are caught by pixels and not by memory.
5. **No copy of assets.** Behaviour may be adopted; icons, typefaces, sounds,
   names and layouts that are trade dress are not.
6. **The matrix row is corrected in the commit that changes the fact.** The
   gate matrix already carries two stale rows (lock/auth, browser); drift is a
   defect.

---

## 8. Risks

| Risk | Why it bites | Mitigation |
|---|---|---|
| 1,000-control scope | every row looks small; together they are years | the ledger's `DEFER`/`REJECT` decisions are first-class, and M1 ships the registry before any family |
| copying one vendor's model | the corpus is 60 % Windows by volume | Track R's Linux batch, and an explicit rule that a behaviour needs a reason beyond "Windows does it" |
| research mistaken for spec | `WB`/`WA` are `PARTIAL` by their own audit | `PENDING` rows are barred from the ledger; `PARTIAL` rows need a second source before `ADOPT` |
| no builder VM | no gate can move | M1 is builder-free by design; M2+ items are split into "code here / verify there" |
| Thai typography and IME | the research corpus is English-centric | WP-10 treats Thai as a first-class case; the existing text audit stays a gate |
| design tokens as decoration | 5,358 values with no mapping is noise | WP-11 requires a decision per token family, not per token |

---

## 9. First three sessions

1. **Session 1 — the ledger and its checker.** The schema document, a seeded
   ledger covering the 13 existing settings pages and the shell surfaces that
   already exist, the `lint-parity.py` checker with its self-test, and the two
   stale gate-matrix rows corrected. None of these exists yet; this session
   creates them.
2. **Session 2 — settings registry + search + deep links (WP-01).** Typed
   registry over `prefstore`, search index with synonyms, keyboard navigation,
   `litric://settings/...` resolution, tests for each.
3. **Session 3 — appearance family (WP-02).** Wallpaper sources and slideshow,
   accent from wallpaper, theme schedule, dock size/position — every row recorded
   in the ledger with its store key and test.

### 9.1 What has landed since this plan was written

| Session | State | Where |
|---|---|---|
| 1 — ledger and checker | **done** (2026-10-26): 41 seeded rows, `ci/lint-parity.py` with a 16-case self-test, wired into `ci/verify.sh`; the three stale gate-matrix rows corrected | `docs/parity/ledger.csv`, `ci/lint-parity.py` |
| 2 — registry, search, deep links | **done**: 7 rows, a two-way test against the store, `litric://settings/<page>/<key>`, and the launcher search reaches rows as well as pages | `src/litric-shell/src/registry.rs` |
| 3 — appearance family | **partial**: the automatic light/dark schedule is in (`theme_schedule`, resolved on every tick); wallpaper sources, slideshow, accent-from-wallpaper and dock size remain `DEFER` in the ledger | `src/litric-shell/src/settings.rs` |
| 4 — browser controls | **done**: Stop and Reload are in the toolbar over the engine's own `STOP`/`RELOAD`, each disabled when it would act on nothing; the broker reports `stopped` instead of `loaded` when a stop interrupted a page | `src/litric-shell/src/views.rs`, `src/litric-browser/src/bin/browserd.rs` |
| 5 — quick settings volume | **done** (WP-05, first control): the panel reads the sink's level through `litric-sysd` (`AUDIO LEVEL`, read back after every change) and offers quieter/louder/mute; the two `DEFER` rows it answers (`WB.01.02`, `WA.06`) are `ADAPT` with the constraint written down | `src/litric-sysd/src/providers.rs`, `src/litric-shell/src/main.rs` |
| 6 — network and Bluetooth tiles | **done** (WP-05, second and third controls): the network row names the interface the kernel reports and offers a link switch and a re-read; the Bluetooth row shows the radio's state and a switch, and both exist only where a provider does. The radio's power became a measurement (`BT STATUS` → `bt.powered`) because no file under `/sys` carries it, and the panel asks for both readings when it opens instead of waiting for a change | `src/litric-sysd/src/state.rs`, `src/litric-sysd/src/providers.rs`, `src/litric-shell/src/views.rs` |

Session 3 is partial on purpose: the three rows it deferred are deferred for a
reason the ledger records, not because there was no time. Slideshow and
per-monitor wallpaper need an image decoder and a monitor model; dock geometry is
one measured strip inside the layout pass.

## 10. What this plan refuses to do

- It does not declare parity with Windows or macOS. It uses them as evidence of
  what people expect.
- It does not implement a control whose backend does not exist, and does not
  hide a missing backend behind a disabled-looking row.
- It does not treat the research as complete. 179 items are unread, and the
  build may not use them until they are not.
