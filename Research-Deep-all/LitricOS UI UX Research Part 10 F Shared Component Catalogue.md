# LitricOS UI/UX Research — Part 10 of N

Chapter 40 (F) — Shared component catalogue: the four controls the shell already has

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P10 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-26 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.01` buttons, `F.03` switches, `F.06` sliders, `F.19` lists. The other 23 `F` items are untouched and remain `PENDING` |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", entered early and partially, on the four components the shell has a live implementation of |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (4 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (sources appended) |
| Resume point | `F.02` text fields — the next component with a live implementation in the shell's component layer |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet", and nothing in this part changes that for the
23 items it does not touch. Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE` tags and the
confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and what "already has" means

### 0.1 Why these four, and not the other 23

The `F` chapter is a catalogue of components across every system in the research
scope (Windows 11 26H2, macOS 27, GNOME 51, Plasma 6.8, Xfce 4.20, Cinnamon,
COSMIC 1.11, Pantheon, Budgie, ChromeOS). It has 27 items and, before this part,
one of them (`F.16` sheets, on macOS HIG evidence alone) had any prose behind it.

The four chosen here are the four the shell actually draws. `src/litric-shell/src/ui.rs`
defines the component layer — one place where a button decides what a button is —
and `button`, `toggle` (the switch), `slider` and `list_item` are four of its
exports:

| Component | Export | Definition | Call sites in the shell |
|---|---|---|---|
| Button (text) | `ui::button` | `src/litric-shell/src/ui.rs` line 273 | 28 |
| Button (icon) | `ui::icon_button` | `src/litric-shell/src/ui.rs` line 351 | **0** |
| Switch | `ui::toggle` | `src/litric-shell/src/ui.rs` line 545 | 5 (`views.rs` ×4, `lock.rs` ×1) |
| Slider | `ui::slider` | `src/litric-shell/src/ui.rs` line 585 | **0** |
| List row | `ui::list_item` | `src/litric-shell/src/ui.rs` line 379 | 5 (`views.rs` ×4, `e2e.rs` ×1) |

Starting here rather than at `F.01` in order is deliberate: a component catalogue
whose first four entries can be compared against running code is falsifiable
today, and the four comparisons below each produced at least one finding that
would not have surfaced from the platform documentation alone. The two zeroes in
that table are the clearest example: the shell's component layer defines a slider
and an icon button, and no surface draws either.

### 0.2 How each item is recorded

Every item has the same four parts, and the fourth is the one that is worth
arguing about:

1. **Platform guidance** — what the three primary platforms document, with the
   source and its tier. No paraphrase of a paraphrase: if the page a reader used
   returned only a summary, that is stated in the item's evidence limitation.
2. **What the shell has** — read out of the source in this repository, tagged
   `MEASURED` when a value comes from a token or a call site, `ESTIMATE` when it
   comes from looking at a rendering.
3. **Deltas** — where the shell differs, in either direction. A delta is not a
   defect until a decision says so; several here are deliberate and are recorded
   as such.
4. **Status** — one of the five final statuses, with the reason.

### 0.3 The numbers the shell's tokens define (MEASURED)

The platform guidance below is mostly qualitative about size (this part's sources
contain exactly one hard number each: Windows' 120 px minimum button width and its
7.5 mm touch target, and Apple's 44 × 44 pt hit region). The shell's own numbers are
exact, because they come from `src/litric-design/src/lib.rs` and not from a
screenshot:

| Scale | Values (logical pixels) | Source |
|---|---|---|
| `Space` | `Xxs` 2, `Xs` 4, `Sm` 8, `Md` 12, `Lg` 16, `Xl` 24, `Xxl` 32, `Xxxl` 48 | `src/litric-design/src/lib.rs` |
| `Radius` | `Sm` 6, `Md` 10, `Lg` 14, `Xl` 20, `Window` 28 | `src/litric-design/src/lib.rs` |
| `TypeStep` | `Caption` 12 / 16 line, `Footnote` 13 / 18, `Body` 15 / 22 | `src/litric-design/src/lib.rs` |

Every value in the "what the shell has" sections below is one of these, or a
`t.px(...)` literal at a named call site. Nothing is estimated.

### 0.4 The state model these four share

`ui::State` (`src/litric-shell/src/ui.rs` line 56) has eight variants, and the
module's own contract, in its header comment, is that every state changes more
than a hue: hover changes the fill, pressed changes the fill *and* insets the
content, focus adds a ring, selected adds a fill and a marker, disabled removes
contrast and adds transparency. Four of the eight are exercised by the two
controls this part finds are covered by a rendering test; the others are not.

Where the states come from at runtime is one function: `Shell::is_highlighted`
(`src/litric-shell/src/shell.rs` line 3216) returns true when the hotspot is
under the pointer **or** holds the keyboard focus, so a surface that asks it draws
`State::Hover` for both. The focus ring variant, `State::Focused`, has no call
site outside `ui.rs` and its own test — a fact this part measured by searching the
whole shell for the identifier, and one that matters to all four items below.

### 0.5 Evidence limits of this part

| Limit | Effect |
|---|---|
| Static reading only. No booted image, no rendering harness for `toggle` and `slider` | The claims about what a control draws are claims about the code. Two of the four are not covered by a rendering test, so no hash comparison confirms them |
| The platform pages for three of the four components were read for this part, not for a version-pinned build | Microsoft publishes WinUI 3 guidance, not build 26300.9711 behaviour; the macOS pages are the current HIG and are not pinned to macOS 27.0.1 |
| Apple's component pages returned summaries to the reader used | The macOS column is the weakest of the three and is marked so item by item; where detail came from a third-party mirror it is marked T6 |
| No platform documents its complete rest/hover/pressed/disabled/focus matrix in the pages read | The shell's eight-state model cannot be checked against an external specification for any of the three platforms. This is a gap in the sources, not a pass for the shell |
| No keyboard or accessibility testing ran | The keyboard claims are readings of the input handlers, not results from a keyboard user. Accessibility stays a `G`-chapter question (see the item statuses) |

---

## F.01 Buttons

Status after this part: **PARTIAL** (Windows 26H2 and GNOME 51 guidance read
against the implementation; macOS read at summary depth only; no keyboard or
rendering test).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | A button initiates an immediate action. Navigation is a `HyperlinkButton`; a repeated action is a `RepeatButton`; a two-state action may be a `ToggleButton`, otherwise prefer `ToggleSwitch`/`CheckBox`/`RadioButton`. Recommended minimum width **120 px** for short command text; button text **26 characters maximum**. The general touch target is about **7.5 mm ≈ 40 × 40 px at 135 PPI / 1.0×**. A focusable control needs a visible focus indicator; disabled controls leave the tab order by default. A focused button activates with **Enter or Space**. `ClickMode="Hover"` cannot raise `Click` from keyboard or touch, so it is not usable as a standard action. Give the control an accessible name, and keep the label a specific verb | learn.microsoft.com — [Buttons](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/buttons), [Guidelines for touch targets](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting), [Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility), [Accessibility overview](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-overview) (T1) |
| macOS 27 | A button initiates an instantaneous action; the standard macOS button is a push button. A lasting two-state choice is a toggle, not a button. A button that opens another window or view carries a trailing ellipsis. Roles: primary (app accent), cancel, destructive (system red), normal. A custom button needs a press state. Hit region guidance: **44 × 44 pt**; image buttons keep about **10 px** between image and edge. The primary button can respond to **Return**. macOS can show a tooltip after hover | developer.apple.com — [Buttons](https://developer.apple.com/design/human-interface-guidelines/buttons), [Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards) (T1, summary depth: the live Buttons page returned a summary to the reader used) |
| GNOME 51 | Use a button to initiate an action. Outside header bars use an icon **or** a label, not both. Toggle buttons show set/unset as pushed in / popped out. `suggested-action` marks the affirmative action, `destructive-action` warns; at most one of either in a view, and a destructive action must be visually distinct. Invalid buttons should be insensitive. Keep labels short; use no more than one or two button widths in a window, and make adjacent buttons the same width; pill buttons invite the primary action in open space. Keyboard: Tab/Shift+Tab move focus, Return activates, accessible controls are named and tested with a screen reader | developer.gnome.org — [Buttons](https://developer.gnome.org/hig/patterns/controls/buttons.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Pointer & Touch](https://developer.gnome.org/hig/guidelines/pointer-touch.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html) (T1) |

All three agree on: an action button does an action immediately; a two-state
control is not a button; every button needs a name and a visible focus state; and
none of them documents its full state matrix as a specification.

### What the shell has (MEASURED)

`ui::button` (`src/litric-shell/src/ui.rs` line 273) draws a filled rounded
rectangle at `Radius::Md` (10), a 1 px border, the label centred in
`TypeStep::Footnote` (13 px), and takes `(state, tone)` as parameters. The eight
states render as:

| State | What changes |
|---|---|
| Normal | neutral: white at alpha 0.10; accent tone: accent at 0.85 |
| Hover | neutral: white at 0.16; accent: 0.92 |
| Pressed | neutral: white at 0.22; accent: 1.0, **and** the label is inset 1 px vertically |
| Focused | the fill steps up by 0.05 and a 2 px accent ring is drawn outside the rect (`ui::focus_ring`, line 260) |
| Selected | accent at 0.22 — deliberately weaker than the filled action states, so a *choice* never looks like the *call to action* |
| Disabled | fill at 0.4 of the base alpha, label at `text` alpha 0.4 |
| Loading | the label is replaced by three marching dots; motion is the whole message |
| Error | danger border in `Tone::Bad` |

`ui::icon_button` (line 351) draws a square at `Radius::Sm` (6) with the glyph
inset 22 % of the shorter side, a fill only when the state is not `Normal`, and
disabled glyph alpha 0.35. The identifier appears in exactly one place in the
shell — its own definition — so every icon-only control in the product is drawn
by hand at its call site instead: the title-bar buttons, the panel toggles and
the row affordances all call `icons::draw` directly. `F.01` therefore covers one
button component that 28 surfaces use, and one that none does.

The interaction model, read from the input handlers:

| Behaviour | Where | Reading |
|---|---|---|
| Pointer activation | `ui::Hotspots` (line 745) — hit-testing the rectangle that was drawn | A click lands on the drawn rect, not a second coordinate set |
| Keyboard traversal | `Shell::move_keyboard_focus` (`src/litric-shell/src/shell.rs` line 3225), driven by Tab/Shift+Tab, over hotspots filtered by `Hot::keyboard_focusable` (line 445) | Tab and Shift+Tab wrap. Whole-surface catchers, window bodies, the desktop and the lock wake target are deliberately skipped |
| Keyboard activation | `Key::Enter => self.activate(hot)` in the setup, lock and dialog paths (`src/litric-shell/src/shell.rs` line 3821 and the lock path) | **Enter** activates. No Space-activation path for a control exists: the only `Key::Char(' ')` handling in the shell is the lock screen's wake (`src/litric-shell/src/shell.rs` line 3933). Text entry never needs a literal space arm, because it consumes characters as text |
| Keyboard appearance | `Shell::is_highlighted` (line 3216) merges pointer hover and keyboard focus; call sites then ask for `State::Hover` | A keyboard user sees the hover picture, not the focus ring |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| A focused button is 34 px wide and 22 px tall | Windows' 7.5 mm ≈ 40 px target; macOS' 44 pt hit region | Below both. `src/litric-shell/src/views.rs`, Quick settings volume row: step buttons `t.px(34.0)` × `t.px(22.0)`, mute `t.px(50.0)` × `t.px(22.0)` |
| The largest button in the shell is 110 px wide | Windows' 120 px minimum for short command text | Below by 10 px. The Defender scan button, `src/litric-shell/src/views.rs`, `bw = t.px(110.0)` |
| Keyboard focus is drawn as hover; the focus ring is unreachable | Windows ("a focusable control needs a visible focus indicator", with a two-outline high-visibility template) and GNOME ("Tab/Shift+Tab move focus" plus screen-reader testing) | The shell has both halves of a keyboard model — a focus order and Enter — and draws the target as if the pointer were on it. `State::Focused` has no call site outside `ui.rs`; the ring code is live but never reached |
| No Space activation | Windows documents Enter **or** Space for a focused button; GNOME documents Space for toggles | Absent, not merely undocumented. This matters most for `F.03`, where Space is the documented toggle key |
| No per-surface default button | macOS: the primary button can respond to Return | Unverified rather than absent — several call sites bind Enter to a specific action (dialogs, the unlock path), but nothing models "the default button of this surface" as a component |
| Label length is unbounded in the component | Windows' 26-character cap; all three ask for short, specific verbs | `ui::button` measures the label and centres it; a long label simply overflows the rect. Current call sites are short ("Scan now", "Mute", "Unmute"), so this is a missing guard rather than a live defect |
| Eight states, and only one tone family is specified per state | Windows/GNOME publish roles (suggested, destructive, primary, cancel) | The shell's `Tone` (line 89) is richer than the roles the platforms name, and has no "destructive" semantics: `Tone::Bad` is a *state*, not a promise to warn. Delete flows that should be visually distinct (GNOME) have no tone of their own |
| Icon-only controls do not go through the component layer | Windows and GNOME both treat an icon button as a button with an accessible name; the shell's title-bar, panel and row affordances call `icons::draw` at their own call sites | `ui::icon_button` has no call site, so its fill, its focus ring and its disabled alpha 0.35 apply to nothing. A hand-drawn icon target inherits none of the eight states, and any future change to icon-button behaviour changes nothing on screen |
| Rendering is covered by a test | — | `every_control_state_renders_differently` (`src/litric-shell/src/e2e.rs` line 1165) hashes the pixels of all eight states for `button` and `list_item` and fails if two states draw the same picture. Buttons are the best-covered control in the layer |

---

## F.03 Switches

Status after this part: **PARTIAL** (Windows 26H2 and GNOME 51 guidance read
against the implementation; macOS read at summary depth only; disabled-state
rendering is a defect this part records and does not fix).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use a toggle switch for a binary setting that takes effect **immediately** when changed. Label it with one or two words, preferably nouns, naming the feature; keep the On/Off labels unless a custom label makes the action clearer. If the change needs a later Submit, or several optional items are being selected, use a CheckBox. Disabled appearance is not specified in the guidance (the default styles live in the SDK's `generic.xaml` and vary by version); the same page set does not state the key used to toggle it | learn.microsoft.com — [Toggle switches](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/toggles), [Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility), [ToggleSwitch API](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.toggleswitch?view=windows-app-sdk-2.0) (T1) |
| macOS 27 | A toggle is for a **pair of opposing states**, on and off. It must not trigger a one-time command, and must not be used to choose among several alternatives. The two states have different appearances | developer.apple.com — [Toggles](https://developer.apple.com/design/human-interface-guidelines/toggles) (T1, summary depth) |
| GNOME 51 | Use a switch for features, settings or hardware with clear on/off logic. The HIG generally prefers switches to checkboxes, because a switch offers a **larger click target** and fits modern layouts; use another control if the label cannot make both states clear. On/off is shown by **position and background colour**, and **the background may change independently of the position** — for example to say that a feature is on but not yet active. Make unavailable switches **insensitive**; give the label an access key; Space toggles | developer.gnome.org — [Switches](https://developer.gnome.org/hig/patterns/controls/switches.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html) (T1) |

Two of the three name a case the shell's switch cannot currently express:
Windows' "immediate effect, or it is a checkbox", and GNOME's "on, but not yet
active" background state. The third (macOS) is the least detailed here.

### What the shell has (MEASURED)

`ui::toggle` (`src/litric-shell/src/ui.rs` line 545) draws a pill of radius
`h / 2` with a knob of diameter `h − 4`:

| State | What changes |
|---|---|
| Off | track white at alpha 0.16, knob at the left, no tick |
| On | track accent at alpha 0.95, knob at the right, **and** a white tick on the left of the track — the state is carried by position, fill and glyph, never by colour alone |
| Focused | a 2 px accent ring (`ui::focus_ring`); the track does not otherwise change |
| Disabled, off | **identical to Off.** The fill is chosen by the on/off branch first; `Disabled` only lowers the *on* alpha from 0.95 to 0.4 |
| Disabled, on | as On, at alpha 0.4 |
| Error / Loading / Selected | ignored: the switch draws as Off or On |

Call sites and how they behave:

| Call site | What it is | Note |
|---|---|---|
| `src/litric-shell/src/views.rs` line 4977 | Quick settings, action row | `State::Normal`; the row's own text says the switch records a preference only when that is the truth |
| `src/litric-shell/src/views.rs` line 5037 | Settings, applied row | state from the measured facts |
| `src/litric-shell/src/views.rs` line 5052 | a **disabled** switch, off | drawn for a setting the backend cannot honour |
| `src/litric-shell/src/views.rs` line 5111 | Settings row bound to a measurement | `measured.unwrap_or(false)` |
| `src/litric-shell/src/lock.rs` line 2212 | Lock screen option | `State::Normal` |

The volume row's mute control is a *button* whose label flips to "Unmute"
(`src/litric-shell/src/views.rs` line 5127 onwards), not a switch — on macOS that
would be a toggle candidate only if it were a setting rather than an action, and
on Windows it is a two-state action, which the guidance routes to `ToggleButton`
at most. Recorded as observed, not as an error.

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| A **disabled off switch is pixel-identical to an enabled off switch** | The module's own contract in `ui.rs` ("disabled removes contrast and adds transparency"); Windows' template exposes a disabled-opacity visual state; GNOME says make unavailable switches insensitive | The shell draws a control the user cannot use to look exactly like one they can. This is the strongest defect this part found, and it is in the component, not in a call site |
| `Error`, `Loading` and `Selected` render as `Normal` | The eight-state contract; no platform documents a switch loading state, so this half is a shell-internal inconsistency | `Error` on a switch is arguably not a state a switch needs. The gap is that the component accepts the state and silently ignores it rather than not offering it |
| No "on but not yet active" background | GNOME 51: the background can change independently of the position | The three dots of `State::Loading` exist for buttons and rows; the switch has no equivalent vocabulary |
| Disabled state is reachable and used | — | The disabled switch at `src/litric-shell/src/views.rs` line 5052 is drawn for a setting no backend can honour, which is the honest use of it. The *rendering* of that state is what is wrong |
| No Space-to-toggle, no state announced beyond the fill and glyph | Windows keeps On/Off labels; GNOME: labels carry access keys, Space toggles; macOS: do not use a toggle for a command | Keyboard handling for a switch is the same as for a button: Enter, not Space, and only where the surface implements it |
| No transition between Off and On | WinUI's ToggleSwitch animates thumb and track | `G.08` (motion timing) is the chapter that owns this; the switch has no motion now |

---

## F.06 Sliders

Status after this part: **PARTIAL**, and the item's note says plainly why: the
component exists, nothing uses it, and its state handling is incomplete.

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|---|
| Windows 11 26H2 | Use a slider for a **changeable relative quantity** — volume, brightness — or a range with **four or more** values, especially where immediate feedback helps. Do not use one as a progress indicator, and do not use one for a value the user cannot change. For a binary setting use a toggle switch, for fewer choices radio buttons, for an exact known value a numeric text box. Keep both endpoints inside the view; **do not change the thumb from its default size**; disable the associated labels and feedback with it; a large touch target (~7.5 mm / 40 px) applies | learn.microsoft.com — [Sliders](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/slider), [Guidelines for touch targets](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting) (T1) |
| macOS 27 | Use a slider to adjust a value along a minimum-to-maximum range; the control is a horizontal track with a thumb, and on macOS can carry **tick marks**; optional stepper buttons beside the slider change the value by a predefined amount | developer.apple.com — [Sliders](https://developer.apple.com/design/human-interface-guidelines/sliders) (T1, summary depth; no slider-specific size, spacing or key binding was verifiable in the page text retrieved) |
| GNOME 51 | Use a slider to choose quickly from a **fixed, ordered range**, especially where relative adjustment, real-time rate of change or many values matter — volume, seeking, image adjustments. Prefer a spin button if the range has no fixed minimum and maximum. Provide real-time feedback while it changes, and mark significant values with text or tick marks | developer.gnome.org — [Sliders](https://developer.gnome.org/hig/patterns/controls/sliders.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html) (T1) |

### What the shell has (MEASURED)

`ui::slider` (`src/litric-shell/src/ui.rs` line 585) draws a 4 px track through
`ui::progress`, filled in `Tone::Accent` to the value, with a 7 px white knob —
9 px when `State::Pressed` — and returns the hit rectangle. A focused slider gets
a 2 px accent ring.

| Finding | Evidence | Consequence |
|---|---|---|
| **No surface draws it.** The identifier `slider` appears in exactly two places in the shell: its own doc comment and its definition | `src/litric-shell/src/ui.rs` lines 584–585, and no call site anywhere under `src/litric-shell/src/` | "The shell has a slider" is true of the component layer and false of the product. `F.06` cannot be judged against a use |
| Volume is two step buttons and a mute button | `src/litric-shell/src/views.rs` line 5127 onwards: `−` and `+` at 34 × 22, mute at 50 × 22, all disabled when no level has been measured | The most obviously slider-shaped control in the system is steps instead. Windows and GNOME both name volume as the canonical slider; macOS adds steppers as a *supplement*, not a replacement |
| `Disabled`, `Error`, `Loading` and `Selected` draw exactly as `Normal` | The only state branches in `ui::slider` are `Pressed` (9 px knob) and `Focused` (ring) | A slider nobody can move looks exactly like one they can, against the same module contract that the disabled button honours. Because there is no call site, the defect is latent rather than visible |
| No tick marks, no value label, no stepper, no keyboard step | Nothing in the function or in the shell's key handlers implements them; `Home`/`End`/arrow stepping exists for lists (launcher, menus, Trash) and not for values | macOS (tick marks, steppers) and GNOME (significant-value marks, real-time feedback) both ask for something here. Windows asks for range or value feedback when the exact value matters |
| The one progress bar in the shell is a progress bar | `ui::progress` (`src/litric-shell/src/ui.rs` line 530), called once, from the Defender scan bar at `src/litric-shell/src/views.rs` line 4072 | Correct against Windows' "do not use a slider as a progress indicator" |
| No rendering test covers it | `every_control_state_renders_differently` — see `src/litric-shell/src/e2e.rs` line 1165 — covers `button` and `list_item`; `slider` and `toggle` are not in it | The two-state defects above were found by reading, not by a failing test. Adding them to that test's list is the smallest change that would have caught the slider one as soon as the states diverged |

---

## F.19 Lists

Status after this part: **PARTIAL** (Windows 26H2 and GNOME 51 guidance read
against the implementation; macOS read at summary depth only; arrow-key behaviour
differs by list and is recorded, not decided).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | `ListView` is for text-heavy, vertically stacked, single-column collections with a shared item structure and interaction (messages, search results, contacts); not for multi-column or table data (use `DataGrid`) or image-led content (use `GridView`). `ListBox` suits 3–9 visibly present alternatives, one line of text per item, avoid scrolling. Item containers document a normal state, a **light-gray hover rectangle**, a **light-blue selection rectangle**, and a **high-visibility keyboard-focus visual drawn over the item template**; the default presenter exposes a disabled-opacity resource but the guidance gives no numeric value. Example item-template heights are **44, 64, 84, 48 px** — examples, not a required row height. Keyboard: arrows move; in single-selection mode arrows move focus **independently of selection**; Space selects the focused item; Ctrl/Shift give deselection and range selection; set `AutomationProperties.Name` on template roots that would otherwise announce `ToString()` | learn.microsoft.com — [Collections and lists](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/lists), [List view and grid view](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/listview-and-gridview), [Combo box and list box](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/combo-box), [Item containers and templates](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/item-containers-templates), [Item templates for list view](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/item-templates-listview), [Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) (T1) |
| macOS 27 | Lists and tables suit data in rows, including grouped or hierarchical data, and support selecting, adding, deleting and reordering. Collections are generally for image-based content; Apple recommends standard row or grid layouts and suggests a table for text-heavy content. Give images enough padding that focus or hover effects stay visible, and consider animation feedback for insertions, deletions and reordering. VoiceOver focus is separate from keyboard focus; group related elements; interactions reachable only by pointer hover may be inaccessible | developer.apple.com — [Lists and tables](https://developer.apple.com/design/human-interface-guidelines/lists-and-tables), [Collections](https://developer.apple.com/design/human-interface-guidelines/collections), [Focus and selection](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection), [WWDC25: Make your Mac app more accessible to everyone](https://developer.apple.com/videos/play/wwdc2025/229/) (T1 for the first two at summary depth; the WWDC25 session was read in full) |
| GNOME 51 | Use a **boxed list** for relatively small, static collections such as preferences or recent documents, and a **list view** for large or dynamic content. Keep rows simple: a boxed-list row should generally contain one control and at most two. A row that navigates shows a trailing `go-next-symbolic` arrow. Predefined row patterns: switch, action, combo, entry, spin, property, expander. In a boxed list, controls should be focusable but **the row itself should not be**, so focus moves directly among controls and neighbouring rows. Reorderable lists must offer "move up"/"move down" menu items. No row height, padding or inter-row spacing is stated; lists need minimum *and* maximum widths for adaptive scaling | developer.gnome.org — [Boxed Lists](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html), [List & Column Views](https://developer.gnome.org/hig/patterns/containers/list-column-views.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html) (T1) |

### What the shell has (MEASURED)

`ui::list_item` (`src/litric-shell/src/ui.rs` line 379) draws one row and returns
it for hit-testing:

| Element | Value (MEASURED) |
|---|---|
| Padding, left and right | `Space::Md` = 12 px |
| Leading icon | sized at half the row height, then a `Space::Sm` = 8 px gap before the text |
| Title | `TypeStep::Body` 15 px, **bold** — a row's title is the row's identity |
| Subtitle | `TypeStep::Caption` 12 px, dim |
| Trailing label | `TypeStep::Caption` 12 px, dim, right-aligned |
| Row height | **not** set by the component: the caller passes the rect. Measured call sites: 30 px (Settings sidebar, gap 2 px), a computed `row_h` (window switcher, notifications), `(panel height − label) / rows` (Quick settings environments) |
| Selected | fill **plus** a 3 px accent marker at the left edge, a quarter of the row height tall — two channels |
| Hover / Focused | fill (`State::Hover` from `is_highlighted`), or a 2 px ring when `Focused` is asked for |
| Error | danger border in `Tone::Bad`, in addition to whatever the row already said |
| Loading | three dots at the trailing edge, inset 18 px from the right |
| Disabled | title, subtitle, icon and trailing label drop to alpha 0.4 / 0.35 |
| Overflow | the title and subtitle are **elided against the row** (`ui::elide`, line 974). The header comment records the defect that caused it: "Environments" reached into the page and was read on top of the "Keyboard" row |

Call sites: Settings sidebar (`src/litric-shell/src/views.rs` line 2347, `Selected`
for the current page), window switcher (line 4620, `Selected`/`Hover` by index),
Quick settings environments (line 5252, trailing hotkey), notifications
(line 5467, subtitle = source, priority icon), and the state-matrix test
(`src/litric-shell/src/e2e.rs` line 1197).

Keyboard behaviour by list, read from the handlers:

| List | Up/Down | Source |
|---|---|---|
| Launcher / app grid | yes, wrapping, moves `selected` | `Shell::overlay_key`, `src/litric-shell/src/shell.rs` line 4559 |
| Menus | yes | `Shell::app_key`, same file |
| Trash | yes, **clamped** (not wrapping) over the visible indices | `Shell::app_key`, same file, line 4937 |
| Settings sidebar, window switcher, notifications, Quick settings environments | no arrow-key navigation; reachable by Tab, activated by Enter | the focus model in §0.4 |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| Rows are 30 px in the Settings sidebar | Windows' example item heights start at 44 px; Apple's 44 × 44 pt hit region for controls; Windows' ~40 px touch target | A pointer-only desktop can defend a 30 px row; it cannot defend claiming touch parity. This is a deliberate density choice that has never been written down as one |
| Three different keyboard models in one shell: arrows for launcher/menus/Trash, Tab for panels, nothing for selecting without focusing | Windows: arrows move focus independently of selection, Space selects; GNOME: the row must not be a focus stop in a boxed list | The shell's `list_item` behaves as a GNOME *boxed-list row* (focus passes among the row's controls) in Settings and as a Windows `ListView` row (the row is the unit) in the switcher, because one component serves both. Honest reading: this is a genuine architecture question the ledger has not decided |
| Selection is not announced to assistive technology | Windows (`AutomationProperties.Name`) and Apple (VoiceOver) both require it; the shell has no accessibility tree at all | Out of scope for `F.19` alone: this is `G.15`, and the component layer would be the place a name would live |
| Hover and keyboard focus draw the same picture | Windows draws a distinct high-visibility keyboard-focus visual over the item template; Apple distinguishes VoiceOver focus from keyboard focus | Same root as `F.01`: `is_highlighted` merges the two, and `State::Focused` is unreachable from a call site |
| Navigable rows carry no trailing chevron | GNOME: a boxed-list row that navigates shows a trailing `go-next-symbolic` arrow | The Settings sidebar navigates and shows selection instead of an arrow. The trailing slot exists in the component (used for text labels); the affordance does not |
| Elision against the row | All three platforms assume text that fits its row; none states the rule because it is assumed | The shell implements it and documents the bug it fixed. This is a place where the implementation is stricter than the sources |
| Row height is the caller's decision, and the component does not enforce a minimum | — | Consequence of the above: nothing in the component stops a 12 px row. The measured call sites are reasonable (30 px and up) |
| Rendering is covered by a test | — | `every_control_state_renders_differently` exercises `list_item` across all eight states, so unlike `toggle` and `slider`, this component's state matrix cannot silently collapse |

---

## 5. Findings this part files without deciding

None of these is a ledger row: a row needs a decision, and the ledger's rules
(see `docs/parity/README.md`) require a backend and a justification that a
research note does not have. They are recorded here so the decision is made against
evidence rather than from memory.

| # | Finding | Where it is visible | Owner |
|---|---|---|---|
| 1 | A disabled **off** switch is pixel-identical to an enabled off switch | `ui::toggle`, `src/litric-shell/src/ui.rs` line 545; live at `src/litric-shell/src/views.rs` line 5052 | `F.03` decision, then the component |
| 2 | `slider` ignores `Disabled`/`Error`/`Loading`/`Selected`, and has no call site to make it visible | `ui::slider`, line 585 | `F.06` decision |
| 3 | Keyboard focus is drawn as hover; `State::Focused` is unreachable from any surface | `Shell::is_highlighted` (`src/litric-shell/src/shell.rs` line 3216) vs `ui::focus_ring` (line 260) | `G.14` keyboard-only navigation, with `F.01`/`F.19` |
| 4 | The state-matrix test covers two of the four controls | `every_control_state_renders_differently`, `src/litric-shell/src/e2e.rs` line 1165 | Test change, one line per control |
| 5 | Buttons below both platforms' minimum targets (34 × 22, 110 px) | `src/litric-shell/src/views.rs`, Quick settings and Defender | `F.01` decision; affects `G.16` pointer/touch |
| 6 | Volume is steps, not a slider, though two of three platforms name volume as the canonical slider | `src/litric-shell/src/views.rs` line 5127 onwards | `F.06` decision, on top of `WP-05` (Quick settings) |
| 7 | No "on, but not yet active" vocabulary | `ui::toggle` | `F.03` / `F.25` loading states |
| 8 | `icon_button` has no call site: every icon-only control is drawn by hand and inherits no state | `ui::icon_button`, `src/litric-shell/src/ui.rs` line 351 | `F.01` decision, then the call sites |

## 6. What this part does not establish

| Claim | Why not |
|---|---|
| that any of the four components is correct | This part compares guidance with an implementation. It found seven things; it decided none of them, and a decided-differently outcome is possible for every one |
| that the shell's component layer is complete | Four of the layer's exports were reviewed. `text_field`, `badge`, `progress`, `section_label`, `separator`, `Column`, `Hotspots`, `cursor`, `wrap` and `elide` were read as context and are not catalogued |
| that `F` is now mostly researched | 23 of 27 items are still `PENDING` and untouched. The chapter is 4 items further along, not largely done |
| that the platform columns are equally strong | The macOS column is the weakest: three of four Apple component pages returned summaries, `F.01`'s detail came through a third-party mirror (T6), and no Apple page states a state matrix |
| that the state matrix matches any platform | None of the three publishes a complete rest/hover/pressed/disabled/focus specification for these controls in the pages read. The shell's eight states have no external yardstick in this corpus |
| that anything here was seen on screen | No image was booted, no pixel was measured, and no keyboard or screen reader was used. Every claim about rendering is a claim about the code that draws it |

## 7. Coverage added, and what would finish these four items

| Item | Before | After | What is still missing |
|---|---|---|---|
| `F.01` Buttons | `PENDING` | `PARTIAL` | macOS at T1 depth (the Buttons page read as a summary only); a decision on the icon button (adopt it at the call sites or delete it); a rendering test that shows the focus ring is reachable |
| `F.03` Switches | `PENDING` | `PARTIAL` | macOS at T1 depth; a decision on the disabled-off rendering; a motion spec (`G.08`) |
| `F.06` Sliders | `PENDING` | `PARTIAL` | A use. The item cannot be judged fully while no surface draws it — the decision is arguably to delete the component or to adopt it in Quick settings |
| `F.19` Lists | `PENDING` | `PARTIAL` | macOS at T1 depth; a decision on which of the two list keyboard models the shell intends; a row-height rule written down rather than repeated per call site |

What would move each to `COMPLETE`: the macOS column at the same depth as the
other two (that is one careful pass over the HIG pages with a reader that returns
full text, not summaries), one decision per item with a backend and a milestone,
and — for the two components with no rendering test — the one-line addition that
makes the state matrix enforceable.

## Sources read for this part

Access date for all rows: 2026-10-26 (Asia/Bangkok). Tier definitions: `Part1` §2.1.

| Source | Tier | Status | Limitations |
|---|---|---|---|
| [Buttons — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/buttons) | T1 | INSPECTED | Automatic extraction; re-read verbatim before a final citation |
| [Toggle switches — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/toggles) | T1 | INSPECTED | Does not specify disabled or hover appearance; does not state the toggle key |
| [Sliders — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/slider) | T1 | INSPECTED | Does not state slider key bindings |
| [Collections and lists — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/lists) | T1 | INSPECTED | — |
| [List view and grid view — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/listview-and-gridview) | T1 | INSPECTED | — |
| [Combo box and list box — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/combo-box) | T1 | INSPECTED | — |
| [Item containers and templates — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/item-containers-templates) | T1 | INSPECTED | States that template brushes come from theme resources; no exact colours |
| [Item templates for list view — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/item-templates-listview) | T1 | INSPECTED | Row heights are examples |
| [Guidelines for touch targets — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting) | T1 | INSPECTED | 7.5 mm figure is density- and scale-dependent |
| [Keyboard accessibility — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) | T1 | INSPECTED | — |
| [Accessibility overview — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-overview) | T1 | INSPECTED | — |
| [ToggleSwitch API — Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.toggleswitch?view=windows-app-sdk-2.0) | T1 | INSPECTED | Default styles vary by SDK version |
| [Slider API — Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.slider?view=windows-app-sdk-2.0) | T1 | INSPECTED | as above |
| [Buttons — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/buttons) | T1 | SNIPPET_ONLY | Reader returned title and summary only |
| [Toggles — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/toggles) | T1 | SNIPPET_ONLY | as above; no macOS state treatment verifiable |
| [Sliders — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/sliders) | T1 | SNIPPET_ONLY | as above; no size, spacing or key binding |
| [Lists and tables — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/lists-and-tables) | T1 | SNIPPET_ONLY | as above |
| [Collections — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/collections) | T1 | SNIPPET_ONLY | Qualitative only |
| [Focus and selection — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection) | T1 | SNIPPET_ONLY | as above |
| [Keyboards — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/keyboards) | T1 | SNIPPET_ONLY | as above |
| [Accessibility — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/accessibility) | T1 | SNIPPET_ONLY | as above |
| [Make your Mac app more accessible to everyone — WWDC25 session 229](https://developer.apple.com/videos/play/wwdc2025/229/) | T1 | INSPECTED | A session transcript, not component specification |
| [HIG text mirror (Buttons)](https://sodiffmi.fyi/HIG/2026-07-17/) | T6 | INSPECTED | A third-party mirror of an Apple page. Used only to confirm detail already implied by the T1 summary, and never as the sole source for a material claim |
| [Buttons — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/buttons.html) | T1 | INSPECTED | States no exact dimensions or padding |
| [Switches — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/switches.html) | T1 | INSPECTED | No numeric size or spacing |
| [Sliders — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/sliders.html) | T1 | INSPECTED | No hover/focus/disabled appearance |
| [Boxed Lists — GNOME HIG](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html) | T1 | INSPECTED | No row height or spacing |
| [List & Column Views — GNOME HIG](https://developer.gnome.org/hig/patterns/containers/list-column-views.html) | T1 | INSPECTED | as above |
| [Keyboard — GNOME HIG](https://developer.gnome.org/hig/guidelines/keyboard.html) | T1 | INSPECTED | — |
| [Pointer & Touch — GNOME HIG](https://developer.gnome.org/hig/guidelines/pointer-touch.html) | T1 | INSPECTED | Qualitative |
| [Accessibility — GNOME HIG](https://developer.gnome.org/hig/guidelines/accessibility.html) | T1 | INSPECTED | — |

Repository sources read (no tier): `src/litric-shell/src/ui.rs`,
`src/litric-shell/src/views.rs`, `src/litric-shell/src/shell.rs`,
`src/litric-shell/src/lock.rs`, `src/litric-shell/src/e2e.rs`,
`src/litric-design/src/lib.rs`.
