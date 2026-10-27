# LitricOS UI/UX Research — Part 12 of N

Chapter 40 (F) — Checkboxes and radio controls: the one control the shell draws,
and the one it does not have

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P12 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-26 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.04` checkboxes, `F.05` radio controls. The other 17 `F` items are untouched and remain `PENDING` |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", continued on the two selection controls |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (2 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (sources appended) |
| Predecessors | Parts 10 (action controls) and 11 (`F.02`, `F.18`, `F.23`, `F.26`) |
| Resume point | `F.08` tabs — the browser tab strip is hand-drawn and is the next control with a live implementation |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet", and nothing in this part changes that for the
17 items it does not touch. Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE` tags and
the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these two follow the last six

### 0.1 One of the two exists, once, and the other does not exist at all

`F.04` and `F.05` are the last pair of selection controls before the catalogue moves
into containers (tabs, sidebars, menus). They are also the pair with the starkest
inventory in the product:

| Control | Export in `ui.rs` | Call sites | Surfaces |
|---|---|---|---|
| Checkbox | **none** | 0 | one: the Files row's multi-select box, drawn by hand |
| Radio | **none** | 0 | **none**: no dot-in-ring is drawn anywhere |
| Single-select chips (the shell's stand-in for a radio group) | none | 4 hotspots per engine list | two: the browser start page and Settings' engine picker |
| Boolean as a state-in-label button | none | 3 | first-run setup, Settings' Accessibility page, Settings' Appearance page |

The shell therefore has a checkbox vocabulary in exactly one place, no radio
vocabulary at all, and two different chip designs standing in where a radio group
would go. That is the shape this part documents.

### 0.2 How each item is recorded

Same four parts as Parts 10 and 11, so they can be read side by side:

1. **Platform guidance** — what the three primary platforms document, with the
   source and its tier. Where the reader returned only a summary, the item says so.
2. **What the shell has** — read out of the source in this repository, tagged
   `MEASURED` when a value comes from a token or a call site.
3. **Deltas** — where the shell differs, in either direction. A delta is not a
   defect until a decision says so.
4. **Status** — one of the five final statuses, with the reason.

### 0.3 The numbers these two are made of (MEASURED)

Token values are the ones Parts 10–11 tabulated (`Space` 2/4/8/12/16/24/32/48,
`Radius` 6/10/14/20/28, `TypeStep` 12/13/15). The geometry below is read from the
draw code and its call site, not from a screenshot:

| Measure | Value | Where |
|---|---|---|
| Files checkbox box | **12 × 12 px**, border 1 px, corner radius **`Radius::Sm` = 6** | `src/litric-shell/src/views.rs` line 1243 |
| Corner radius against the box | 6 on a 12 px box is **half the side**, so the box is a circle, not a square | `views.rs` line 1250, `t.radius(Radius::Sm)` |
| Filled state | accent fill + accent border + a hand-drawn check: two strokes, **1.5 px**, in `t.text()` (white) | `views.rs` lines 1250–1275 |
| Unchecked state | no fill, `t.text_dim()` 1 px border | `views.rs` lines 1252–1256 |
| Checkbox hit target | the box inflated by 3 px = **18 × 18 px** | `views.rs` line 1281, `check.inflate(t.px(3.0))` |
| Row height around it | **24 px**; the box is vertically centred, 8 px from the table's left edge; the first column's text starts at 28 px | `views.rs` lines 1147, 1243, 1230 |
| Start-page engine chip | **96 × 26 px**, `Radius::Md`, bold 13 px label; selected = accent fill at 22 % + 1 px accent border | `views.rs` lines 1986–2010 |
| Settings engine chip | height **18 px**, `Radius::Sm`, caption 12 px label; selected = accent fill at 18 % + accent text, **no border change** | `views.rs` lines 2064–2100 |
| Settings boolean row | accent button **230 × 30 px**, label carries the state (`"Reduce motion · On"`) | `views.rs` lines 2564–2586, 2930–2945 |
| Setup boolean row | accent button, pane width × **32 px**, label carries the state | `views.rs` lines 5673–5695 |

### 0.4 The state model, and how little of it reaches these two

Parts 10 and 11 documented `ui::State`'s eight variants and the fact that
`Shell::is_highlighted` (`src/litric-shell/src/shell.rs` line 3216) merges pointer
hover with keyboard focus. Neither selection control uses the state enum:

| Control | States it can draw | States the platforms describe |
|---|---|---|
| Files checkbox | unchecked, checked, hover/focus ring; **nothing else** | unchecked, checked, indeterminate, hover, pressed, disabled, focus |
| Engine chips | unselected, selected; hover only where a call site remembers to ask | unselected, selected, hover, pressed, disabled, focus, and (Windows) a group focus |
| Boolean button | rest, hover/pointer, "focused" from a settings link ring | on, off, hover, pressed, disabled, focus |

So the selection half of the interface has the same structural gap Part 11 found in
the input half: the eight-state contract is not reachable from these controls.

### 0.5 Evidence limits of this part

| Limit | Effect |
|---|---|
| Static reading only. No booted image and no rendering harness for the checkbox or the chips | What they look like is a claim about the code. The checkbox is not in the state-matrix test; it is covered by one behavioural test |
| **Apple's checkbox and radio pages were not retrievable.** The reader returned titles and summaries for `toggles`, `accessibility`, `keyboards`, `voiceover`, `selection-and-input`, `menus`, `buttons` and `controls`, and short API summaries for `NSButton.allowsMixedState`, `NSButton.state` and `NSButton.ButtonType.switch` | The macOS column is effectively thin, and is marked `SNIPPET_ONLY` throughout. No macOS checkbox or radio rule is reported as verified; the only usable facts are that Apple's toggles page describes opposing states and that AppKit exposes a mixed state for buttons |
| Windows' current checkbox and radio pages describe use, labels and keyboard, but **not** a per-control visual state recipe | The hover/pressed/disabled appearance cannot be compared against a published Windows rule; the pages say focus is a highlighted border, nothing more specific |
| GNOME gives no checkbox or radio dimensions | The shell can be measured against Windows' 40 × 40 epm touch guidance and against no Linux number. WCAG's 24 × 24 target is quoted once, explicitly marked as a web-content criterion and not a platform rule |
| No keyboard or screen-reader testing ran | The keyboard claims are readings of the hotspot table and the key handlers |

---

## F.04 Checkboxes

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
unavailable beyond summaries; one live surface, and it is drawn as a circle).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use a checkbox for a yes/no choice or a group where users may select **any combination** of non-exclusive options. A checkbox indicates **status** and may wait for a submit; a toggle switch indicates an **action** that takes effect immediately, so an immediate setting is a switch, not a checkbox. A single binary choice is one checkbox, not two radios. Three selection states: unselected, selected and **indeterminate**, the last meaning "some, but not all, subordinate options are selected" — an aggregate, never a third answer the user picks; the subordinate boxes stay visible. The label sits next to the box (`Content`), no more than **two lines**; child options in the page's example are indented 24 px (an example, not a rule). General touch target **7.5 mm square ≈ 40 × 40 px at 135 PPI**; group labels are recommended; **Space** activates the focused control; disabled controls leave the tab order; give it a meaningful accessible name and use `AutomationProperties.Name` where text cannot supply it | learn.microsoft.com — [Check boxes](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/checkbox), [Toggle switches](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/toggles), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions), [Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility), [Guidelines for touch targets](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting), [Expose basic accessibility information](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information) (T1; a legacy Win32 checkbox page was also read but is marked Windows 7-era and is not used for current rules) |
| macOS 27 | Apple's checkbox page could not be read: the reader returned only the title and summary, and the same for the controls index. What is verifiable is adjacent: Apple's toggles page describes a control for **opposing states** (such as on and off) with a different appearance for each, and AppKit's `NSButton` exposes `allowsMixedState`, `state` and a `switch` button type, i.e. the **mixed state exists as an API capability**. No macOS checkbox size, spacing, label rule or keyboard binding was retrievable | developer.apple.com — [Toggles](https://developer.apple.com/design/human-interface-guidelines/toggles), [Controls](https://developer.apple.com/design/human-interface-guidelines/controls), [NSButton.allowsMixedState](https://developer.apple.com/documentation/appkit/nsbutton/allowsmixedstate?language=objc), [NSButton.state](https://developer.apple.com/documentation/appkit/nsbutton/state?language=objc) (T1, `SNIPPET_ONLY`: summaries only; no checkbox-specific rule verified) |
| GNOME 51 | A checkbox represents a binary option or property. GNOME **generally prefers a switch** for clear on/off settings, because a switch offers a larger click target and fits modern layouts; the checkbox remains appropriate where a switch does not fit. For a list where choices are independent and more than one may be selected, use separate checkboxes — not a radio group. Labels should make both the checked and unchecked effect clear, and negative labels are discouraged. States: checked, unchecked and **mixed**, where a property differs across selected objects; clicking a mixed box checks it for all, clicking again unchecks for all, and a third click restores the original values and returns to mixed. **No numeric size, spacing or hit-target rule**; pointer guidance is qualitative. **Space** toggles; accessible names are required; test keyboard-only, screen-reader, high-contrast and large-text use; GTK exposes true/false/mixed | developer.gnome.org — [Checkboxes](https://developer.gnome.org/hig/patterns/controls/checkboxes.html), [Switches](https://developer.gnome.org/hig/patterns/controls/switches.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html), [Pointer & Touch](https://developer.gnome.org/hig/guidelines/pointer-touch.html); GTK — [Gtk.CheckButton](https://docs.gtk.org/gtk4/class.CheckButton.html), [AccessibleRole](https://docs.gtk.org/gtk4/enum.AccessibleRole.html) (T1; the GTK pages are T2 implementation detail and are marked as such) |

Both platforms that could be read name the indeterminate/mixed state and Space.
Neither publishes a complete visual-state matrix in the pages read.

### What the shell has (MEASURED)

There is no `ui::checkbox`. The only checkbox in the product is drawn inline in the
Files table, once per row:

| Element | Reading |
|---|---|
| Shape | a 12 × 12 px rounded rectangle at `Radius::Sm` = 6, i.e. **a circle** |
| Unchecked | 1 px `text_dim()` border, no fill |
| Checked | accent fill, accent border, and a two-stroke check in white at 1.5 px — fill *and* glyph, so the state does not rest on colour |
| Hover / focus | `ui::focus_ring` (2 px accent) around the circle, from `is_highlighted`; the two inputs are merged |
| Disabled | no path: the box is drawn wherever a row is |
| Indeterminate | no path |
| Label | none. The filename is in the row, but it is not associated with the box in any structure, and the shell has no accessibility tree |
| Hit target | the circle inflated to **18 × 18 px** |
| Semantics | `FilesHot::ToggleRow(i)` adds or removes that row from a `BTreeSet<String>`; the row itself has a second hotspot (`Row(i)`) that replaces the selection. Both are in the tab order |

Input and selection model:

| Behaviour | Evidence |
|---|---|
| Click the circle toggles one row without disturbing the others | `FilesHot::ToggleRow` → `FilesApp::toggle_row` (`src/litric-shell/src/apps.rs` lines 621, 1485) |
| Click the row selects it as the single selection | `FilesHot::Row` → `select_row`; `selected_rows` is a `BTreeSet` (`apps.rs` line 255) |
| `Ctrl+A` selects every visible row; there is no shift-range and no ctrl-click distinction | `select_all_visible` (`apps.rs` line 653); `Ctrl+A` in `shell.rs` line 4900; the click path takes no modifiers |
| The keyboard reaches every row **and** every box by Tab, and Enter activates whichever has focus | `Hot::keyboard_focusable` (`shell.rs` line 445) excludes only window bodies, the desktop and the lock wake; `Shell::activate` (`shell.rs` line 5351) |
| **Space does nothing to a checkbox.** The only `Space` handling in the shell is the lock screen's wake | `shell.rs` line 3933 |
| A checkbox cannot be disabled or marked mixed, because neither is an argument to the draw code | `views.rs` lines 1243–1280 |

The one line of test coverage is behavioural, not visual: `e2e.rs` line 317 finds
every `ToggleRow` rectangle, clicks its centre, and asserts two rows are selected.
`every_control_state_renders_differently` (`e2e.rs` line 1165) does not include the
checkbox.

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **The product's only multi-select checkbox is drawn as a circle.** A 12 px box at radius 6 is a disc | Every platform's checkbox is a square; a disc is the radio vocabulary, and this product uses discs for nothing else | The control's shape says "choose one", its behaviour says "choose several". The Files row is the worst place for that ambiguity, because the same rows also have a single-selection state |
| The hit target is 18 × 18 px | Windows' 7.5 mm ≈ 40 × 40 px; Apple's 44 × 44 pt (from Part 10's reading of the buttons page); WCAG 2.2's 24 × 24 CSS px, quoted here as a web criterion rather than a platform rule | Below all three yardsticks. The 24 px row it sits in is smaller than the WCAG target before the box is even drawn |
| **No indeterminate/mixed state**, and no header checkbox to aggregate a selection | Windows: indeterminate means "some but not all"; GNOME: mixed across selected objects, with the three-click cycle | The shell has multi-select over one folder and no way to show "all/none/some" in one control. `Ctrl+A` is keyboard-only and has no visible control |
| **Space does not toggle.** Enter activates; clicking toggles | Windows and GNOME both name Space as the checkbox key | A keyboard user must Tab to the box and press Enter — the key that also opens a row when focus is on the row. The two actions are one Tab apart and share one key |
| Two Tab stops per row: the row and its box | Windows: arrows move within the list, Space selects; GNOME: in a boxed list the row itself should not be a focus stop, the controls should be | A 24-row folder inserts 48 Tab stops. The keyboard model is "tab through every control", not "move through the list" |
| No shift-click range and no ctrl-click distinction | Windows documents Ctrl and Shift selection; macOS and GNOME list multi-select as the reason for checkboxes | Selection is toggled one box at a time. There is no way to select a run |
| No label on the box; no accessible name or state | Windows: label next to the box, `AutomationProperties.Name`; GNOME: accessible names, screen-reader test; Apple: VoiceOver | The filename is adjacent but unassociated. With no accessibility tree, the box announces nothing at all |
| Hover draws the focus ring | Windows draws a distinct keyboard-focus visual; the shell's `State::Focused` remains unreachable (Part 10, finding 3) | Same root cause as every other control: one function merges pointer and keyboard |
| The same boolean ("Reduce motion") is a **switch** in Quick settings, a **state-in-label button** in Settings, and a **state-in-label button** in first-run | Windows: checkbox/switch distinction by immediacy; GNOME: prefer a switch for on/off; Apple: a toggle is for opposing states | Three surfaces, two control types, one setting. Only the Quick settings copy is a real toggle (`ui::toggle`, Part 10's `F.03`); the Settings row says "Reduce motion · On" on an accent button, so the state lives in text rather than in a control |
| No pressed or disabled rendering | All three publish those states (even where the exact pixels are not documented) | There is nothing to press-but-not-release on a checkbox, so pressed matters less; disabled matters, and there is no way to express "this box cannot be changed" |
| Behaviour is tested; appearance is not | — | The e2e test proves the toggle works, not that the circle reads as a checkbox. A rendering test would not help while the shape is the defect |

---

## F.05 Radio controls

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
unavailable beyond summaries; **no radio control exists in the shell**, so the item
is judged on the single-select surfaces that stand in for one).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use radio buttons for **two or more related, mutually exclusive** choices when the options should be visible; only one can be selected, selecting another moves the selection, and selecting the current one again does not clear it. A single binary choice is one checkbox or switch, not two radios. For **more than eight options** use a combo box; consider radios when there are **fewer than five** single-select items. Each label is **one line**. Prefer the `RadioButtons` control, which handles group layout, keyboard and accessibility; for individual buttons, group them in one parent or by `GroupName`, and match spacing with `Margin="0,0,7,3"`. Keyboard: Tab enters the group at the selected item (or the first when nothing is selected); arrows move within the group (Up/Down in a column, Left/Right in a row), **selection follows focus**, Ctrl+arrow moves focus without selecting, Space selects, and navigation does not wrap. `RadioButtons.Header` is the group label ("Background color" in the example); make the group's purpose and current state explicit and do not put two groups side by side. Each option needs a meaningful accessible name. The general touch target is the same 7.5 mm ≈ 40 × 40 px | learn.microsoft.com — [Guidelines for radio buttons](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/radio-button), [Combo box and list box](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/combo-box), [Toggle switches](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/toggles), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) (T1; the legacy Win32 radio page is Windows 7-era and is not used for current rules) |
| macOS 27 | Apple's radio-button page could not be read: the reader returned only titles and summaries for `controls`, `selection-and-input`, `menus` and `buttons`. A "about five options" threshold appeared in a search preview and is **not** treated as established guidance. No macOS radio size, group-label, arrow-key or VoiceOver rule is reported here as verified | developer.apple.com — [Controls](https://developer.apple.com/design/human-interface-guidelines/controls), [Selection and input](https://developer.apple.com/design/human-interface-guidelines/selection-and-input), [Menus](https://developer.apple.com/design/human-interface-guidelines/menus) (T1, `SNIPPET_ONLY`: titles and summaries only) |
| GNOME 51 | Use radios for a set of **mutually exclusive** options: choosing one selects it in place of another. Radios suit small sets because every option is visible; longer sets often suit a dropdown, and the dropdown guidance says **do not use a dropdown with fewer than three items** — for two choices, use radio or toggle buttons. A **mixed** state is allowed when the property differs across selected objects. Space toggles under the general keyboard rule; GTK groups check buttons with a `radio` indicator, supports directional movement through the group and activates the newly focused option; each option needs a descriptive accessible name. The current HIG discusses option labels but **does not prescribe a group heading**; the archived (retired) HIG did — a descriptive heading above or left, a frame only if needed, options aligned vertically. That archived rule is cited as history, not as current policy. No numeric size or hit target | developer.gnome.org — [Radio Buttons](https://developer.gnome.org/hig/patterns/controls/radio-buttons.html), [Drop-Down Lists](https://developer.gnome.org/hig/patterns/controls/drop-downs.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html); GTK — [Gtk.CheckButton](https://docs.gtk.org/gtk4/class.CheckButton.html); archived HIG — [Mutually Exclusive Options](https://wiki.gnome.org/Design(2f)HIG(2f)Planning(2f)MutuallyExclusiveOptions.html) (T1 current; the archived page is marked retired) |

Windows is the only platform of the three that documents the full keyboard model
for a group. GNOME documents Space and points at GTK for arrows. macOS contributes
nothing verifiable here.

### What the shell has (MEASURED)

**There is no radio control.** The strings `radio`/`Radio` in the shell refer to the
Bluetooth radio, never to a control. There is no dot-in-ring drawing at all:
`Canvas` has no `stroke_circle` method, and `fill_circle` is used for toggle knobs,
badge dots and the pointer — never as a selection glyph.

The mutually exclusive choices the product does have are drawn as **chips**, and
there are two different chip designs for the same three-way choice:

| Surface | Geometry | Selected appearance | Non-colour channel | Group label |
|---|---|---|---|---|
| Browser start page, "Search engine" chips | 96 × 26 px, `Radius::Md`, bold 13 px | accent fill 22 % + 1 px accent border | **none**, despite the comment saying "the check is its own glyph" (`views.rs` line 1985) — no check is drawn in the loop | none |
| Settings, engine picker | 18 px tall, `Radius::Sm`, caption 12 px | accent fill 18 % + accent text; border unchanged | **none** | `"Search engine:"` label at `views.rs` line 2070 |

Both sets register one hotspot per option (`Hot::EngineChip(i)` on the start page,
`Hot::MenuRow(MenuAction::Engine*)` in Settings). Both are ordinary hotspots, so:

| Behaviour | Reading |
|---|---|
| Arrow keys within the group | none; there is no group object and no arrow handler for either set |
| Tab | stops on **every chip**: a three-way choice costs three Tab presses, and Tab leaves the group only after the last one |
| Selection follows focus | no; focus and selection are independent, and the selection changes only on Enter or click |
| Group focus ring | none; each chip can draw its own focus from `is_highlighted` where the call site asks, and neither call site asks on the start page |
| Disabled chip | none |
| Current choice visible after leaving the start page | no. The toolbar shows no engine; the Settings label shows the chosen chip only while the page is open |
| Persistence | the engine is a stored preference and survives a restart |

The other mutually exclusive choices are **cycling rows**, not option sets:

| Surface | What it is | Where |
|---|---|---|
| Settings · Appearance | "Change theme · Dark" on an accent button; `Ctrl+T` cycles | `views.rs` line 2628; shortcut list line 3091 |
| Settings · Language | "Change language" on an accent button; `Ctrl+L` cycles | `views.rs` line 2692 |
| First-run setup | the same two choices as accent buttons with the value in the label | `views.rs` lines 5645–5652 |

A cycling row shows the current value and hides the alternatives. That is the
GNOME "action row" pattern, not a radio group; it is recorded as observed, not as
an error.

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **No radio control and no radio vocabulary.** The shell's single-select choices are chips or cycling rows | Windows: radios for 2–8 visible exclusive options; GNOME: radios for small sets; Apple: the selection controls page exists and could not be read | "Radio controls" in this catalogue describes a control the product does not have. That is a finding about the product, not a gap in the research |
| The one multi-select checkbox is **circular**, while the single-select choice is a **chip** | Every platform draws a checkbox square and a radio as a disc | The visual grammar is inverted: the multi-select control looks like a radio, and the single-select control looks like a tag |
| Selected chips rest on colour alone, in both designs | Windows: do not rely on colour alone; GNOME: accessible names and high-contrast testing; the shell's own comment in `ui.rs` says every state changes more than a hue | The start-page comment claims a check glyph that the code does not draw. This is the same class of defect Part 10 found in the switch: the contract is written down and the drawing does not follow it |
| Two designs for the same choice | One component per control is the premise of the layer (`ui.rs`), and Parts 10–11 found the same divergence in fields and search | The start page's chips are 96 × 26, bordered and bold; Settings' chips are 18 px, borderless and caption-sized. A user who learned one does not recognise the other |
| **Three Tab stops for a three-way choice**, no arrow navigation, no selection-follows-focus | Windows: Tab enters the group once, arrows move, selection follows focus; GNOME/GTK: directional movement within the group | The shell has the options but not the group. A keyboard user presses Tab three times and Enter once, and cannot tell where the group begins or ends |
| No group label on the start page | Windows: `RadioButtons.Header`; archived GNOME: a descriptive heading; Apple: scope/title as context | The start page's chips sit under a search field with no "Search engine" text; the Settings copy has the label. The same group has a label in one place and none in the other |
| No focus ring on the start-page chips | Windows: a focusable control needs a visible focus indicator; the group has a focused-item visual | `is_highlighted` is not consulted in that loop, so a keyboard user's current chip is indistinguishable from the others |
| No disabled state | All three list disabled among the states | A chip that cannot be used cannot be drawn as unusable |
| Theme and language hide their alternatives in a cycling button | Windows: radios when the options should be visible; GNOME: dropdown for three or more (and never a dropdown for fewer than three); Apple: pop-up for many | A three-way theme choice is exactly the Windows radio case. The cycling row is defensible as an action row, but it is not a selection control and has no keyboard model beyond "press again" |
| The engine is a stored preference and the choice is real | — | This part's one positive finding for the item: the selection persists, and `search_url` uses it (Part 11, `F.23`) |

---

## 5. Findings this part files without deciding

Continuing Parts 10–11's numbering. None of these is a ledger row: a row needs a
decision with a backend and a justification that a research note does not have.

| # | Finding | Where it is visible | Owner |
|---|---|---|---|
| 25 | The only multi-select checkbox is drawn as a 12 px **circle** (radius = half the side), so the shape says radio and the behaviour says checkbox | `views.rs` lines 1243–1256 | `F.04` decision, then the component layer |
| 26 | Its hit target is 18 × 18 px | `views.rs` line 1281 | `F.04`, with `G.16` pointer/touch |
| 27 | No indeterminate/mixed state and no header/select-all checkbox | `views.rs` lines 1243–1280 | `F.04` |
| 28 | Space does not toggle a checkbox; only Enter and click do | `shell.rs` line 3933 (the only Space) | `F.04`, with `G.14` |
| 29 | Two Tab stops per Files row (row + box); a 24-row folder is 48 stops | `shell.rs` line 445; `views.rs` lines 1242, 1277 | `F.04` / `F.19` / `G.14` |
| 30 | No shift-range or ctrl-click selection | `apps.rs` line 621; `shell.rs` line 4900 | `F.04` |
| 31 | No label or accessible name on the box; no accessibility tree | `views.rs` line 1243 | `G.15`, with `F.04` |
| 32 | "Reduce motion" is a switch in Quick settings and a state-in-label button in Settings and setup | `views.rs` lines 2930, 4907, 5654 | `F.03` / `F.04` |
| 33 | No radio control exists at all | grep: no `radio` control, no `stroke_circle` | `F.05` decision |
| 34 | The start-page chip comment claims a check glyph; none is drawn, so the selected chip is colour-only | `views.rs` line 1985 vs lines 1986–2010 | `F.05`, and the same class as Part 10 finding 1 |
| 35 | The same three-way engine choice has two chip designs and only one has a group label | `views.rs` lines 1986, 2064 | `F.05` |
| 36 | Three chips are three Tab stops, with no arrow navigation or selection-follows-focus | `shell.rs` line 445; both chip loops | `F.05`, with `G.14` |
| 37 | Theme and language hide their alternatives in cycling rows | `views.rs` lines 2628, 2692 | `F.05` / `F.09` |

## 6. What this part does not establish

| Claim | Why not |
|---|---|
| that the checkbox is wrong to be a circle | This part compares shapes with what platforms draw and with the behaviour in the code. It records the mismatch and decides nothing; a deliberate disc-shaped multi-select is possible, just undocumented |
| that the engine choice should be radios | Chips are a legitimate single-select pattern where the options are short labels. The finding is that this is the *only* single-select vocabulary the shell has, not that chips are illegitimate |
| that `F.04` and `F.05` are researched to the same depth as `F.01` | They are not. Apple's checkbox and radio pages were unreadable, so the macOS column is a summary and an API capability, nothing more |
| that `F` is nearly done | 17 of 27 items remain `PENDING`; ten are `PARTIAL`; none is `COMPLETE` |
| that anything was seen on screen | no image was booted and no pixel measured. Every rendering claim is a claim about the code that draws it |
| that a screen reader would announce the checkbox | there is no accessibility tree; the claim is that it cannot, not that it was observed failing |

## 7. Coverage added, and what would finish these two items

| Item | Before | After | What is still missing |
|---|---|---|---|
| `F.04` Checkboxes | `PENDING` | `PARTIAL` | macOS at any depth (the page did not load), a decision on the shape and the hit target, a decision on indeterminate and select-all, and a keyboard decision (Space and the two-stops-per-row problem) |
| `F.05` Radio controls | `PENDING` | `PARTIAL` | macOS at any depth; a decision on whether the product gets a radio control or keeps chips (and then which chip design is the one), a group-label rule, and the arrow-key model for a group |

What would move each to `COMPLETE`: a reader that returns Apple's checkbox and
radio pages in full, one decision per item with a backend and a milestone, and — for
`F.04` — a rendering test that would have caught a disc-shaped checkbox the moment
it was drawn.

## Sources read for this part

Access date for all rows: 2026-10-26 (Asia/Bangkok). Tier definitions: `Part1` §2.1.

| Source | Tier | Status | Limitations |
|---|---|---|---|
| [Check boxes — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/checkbox) | T1 | INSPECTED | Use, labels, three selection states and Space; no hover/pressed/disabled pixel recipe |
| [Guidelines for radio buttons — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/radio-button) | T1 | INSPECTED | Group keyboard model and Header; no visual-state recipe |
| [Toggle switches — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/toggles) | T1 | INSPECTED | Status-versus-action distinction |
| [Combo box and list box — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/combo-box) | T1 | INSPECTED | The fewer-than-five and more-than-eight thresholds |
| [Keyboard interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) | T1 | INSPECTED | Partial extraction; Space activation and radio arrows were present |
| [Keyboard accessibility — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) | T1 | INSPECTED | Focus indicator and tab order |
| [Expose basic accessibility information — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information) | T1 | INSPECTED | Accessible names |
| [Guidelines for touch targets — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting) | T1 | INSPECTED | 7.5 mm / 40 px, density-dependent |
| [Check boxes — Win32 UX guide](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-check-boxes) | T1 (legacy) | INSPECTED | Marked Windows 7-era by Microsoft; not used for current rules |
| [Radio buttons — Win32 UX guide](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-radio-buttons) | T1 (legacy) | INSPECTED | as above |
| [Toggles — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/toggles) | T1 | SNIPPET_ONLY | Title and summary only |
| [Controls — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/controls) | T1 | SNIPPET_ONLY | as above |
| [Selection and input — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/selection-and-input) | T1 | SNIPPET_ONLY | Title only; extraction returned "There." |
| [Menus — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/menus) | T1 | SNIPPET_ONLY | as above |
| [Buttons — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/buttons) | T1 | SNIPPET_ONLY | as above |
| [Accessibility — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/accessibility) | T1 | SNIPPET_ONLY | as above |
| [Keyboards — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/keyboards) | T1 | SNIPPET_ONLY | as above |
| [VoiceOver — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/voiceover) | T1 | SNIPPET_ONLY | as above |
| [NSButton.allowsMixedState — AppKit](https://developer.apple.com/documentation/appkit/nsbutton/allowsmixedstate?language=objc) | T1 | SNIPPET_ONLY | API summary only |
| [NSButton.state — AppKit](https://developer.apple.com/documentation/appkit/nsbutton/state?language=objc) | T1 | SNIPPET_ONLY | API summary only |
| [NSButton.ButtonType.switch — AppKit](https://developer.apple.com/documentation/appkit/nsbutton/buttontype/switch) | T1 | SNIPPET_ONLY | API summary only |
| [Checkboxes — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/checkboxes.html) | T1 | INSPECTED | No size or spacing; mixed-state cycle described |
| [Radio Buttons — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/radio-buttons.html) | T1 | INSPECTED | No group heading in the current page; no numbers |
| [Switches — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/switches.html) | T1 | INSPECTED | Preference for switches over checkboxes |
| [Drop-Down Lists — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/drop-downs.html) | T1 | INSPECTED | Never fewer than three items in a dropdown |
| [Keyboard — GNOME HIG](https://developer.gnome.org/hig/guidelines/keyboard.html) | T1 | INSPECTED | Space toggles; directional navigation where possible |
| [Accessibility — GNOME HIG](https://developer.gnome.org/hig/guidelines/accessibility.html) | T1 | INSPECTED | Accessible names and test matrix |
| [Pointer & Touch — GNOME HIG](https://developer.gnome.org/hig/guidelines/pointer-touch.html) | T1 | INSPECTED | Qualitative only |
| [Mutually Exclusive Options — archived GNOME HIG](https://wiki.gnome.org/Design(2f)HIG(2f)Planning(2f)MutuallyExclusiveOptions.html) | T1 (retired) | INSPECTED | Explicitly retired; used only as history for group headings |
| [Gtk.CheckButton — GTK 4](https://docs.gtk.org/gtk4/class.CheckButton.html) | T2 (implementation detail) | INSPECTED | Indicator names, grouping, Space/Enter, mixed state |
| [AccessibleRole — GTK 4](https://docs.gtk.org/gtk4/enum.AccessibleRole.html) | T2 (implementation detail) | INSPECTED | Checkbox state exposure |
| [GtkCheckButton source — GTK](https://codebrowser.dev/gtk/gtk/gtk/gtkcheckbutton.c.html) | T2 (implementation detail) | INSPECTED | Partial extraction; grouping and focus-navigation portions readable |
| [WAI-ARIA radio group pattern](https://www.w3.org/WAI/ARIA/apg/patterns/radio/) | T3 (standard) | INSPECTED | A web pattern, not a desktop platform; used only for the group-label question |
| [Understanding target size minimum — WCAG 2.2](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) | T3 (standard) | INSPECTED | Web-content criterion, labelled as such; not a platform rule |

Repository sources read (no tier): `src/litric-shell/src/ui.rs`,
`src/litric-shell/src/views.rs`, `src/litric-shell/src/apps.rs`,
`src/litric-shell/src/shell.rs`, `src/litric-shell/src/e2e.rs`,
`src/litric-design/src/lib.rs`.
