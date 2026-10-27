# LitricOS UI/UX Research — Part 11 of N

Chapter 40 (F) — Text fields, progress, search fields and error presentation: the
rest of the component layer the shell draws

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P11 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-26 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.02` text fields, `F.18` progress indicators, `F.23` search fields, `F.26` errors/validation. The other 19 `F` items are untouched and remain `PENDING` |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", continued on the four items that have a live implementation or a live surface |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (4 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (sources appended) |
| Predecessor | `Research-Deep-all/LitricOS UI UX Research Part 10 F Shared Component Catalogue.md` (`F.01`, `F.03`, `F.06`, `F.19`) |
| Resume point | `F.04` checkboxes — the Files row and the Settings rows draw check marks by hand, so the next component with code behind it is the checkbox, not a placeholder item |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet", and nothing in this part changes that for the
19 items it does not touch. Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE` tags and
the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these four follow the last four

### 0.1 The layer had ten exports; Part 10 covered four of them

`src/litric-shell/src/ui.rs` is the shell's component layer, and Part 10 reviewed
the four *action* controls it exports. The remaining two visual primitives in the
same file are what this part starts with, because they are the two the shell
actually calls:

| Component | Export | Definition | Call sites in the shell |
|---|---|---|---|
| Text field | `ui::text_field` | `src/litric-shell/src/ui.rs` line 599 | 4 (2 dialogs, 2 filters) |
| Progress bar | `ui::progress` | `src/litric-shell/src/ui.rs` line 530 | 1 (Defender scan) |
| Status pill | `ui::badge` | `src/litric-shell/src/ui.rs` line 509 | **0** |

`F.23` and `F.26` have no export at all, which is itself the finding: the shell
searches in three bespoke shapes (the omnibox, the launcher bubble, and the
generic field wearing a magnifier), and it presents errors through a dialog kind
and one inline line rather than through a component. A catalogue that only read
`ui.rs` would report both items as "absent"; the product has both, four times
over, drawn by hand.

That is why these four are the honest next step: two of them are the remaining
primitives, and two of them are the places where the absence of a primitive is
visible in the product.

### 0.2 How each item is recorded

Same four parts as Part 10, so the two can be read side by side:

1. **Platform guidance** — what the three primary platforms document, with the
   source and its tier. No paraphrase of a paraphrase: where the reader returned
   only a summary, the item says so.
2. **What the shell has** — read out of the source in this repository, tagged
   `MEASURED` when a value comes from a token or a call site.
3. **Deltas** — where the shell differs, in either direction. A delta is not a
   defect until a decision says so; several here are deliberate density choices.
4. **Status** — one of the five final statuses, with the reason.

### 0.3 The numbers these four are made of (MEASURED)

The token values are the same ones Part 10 tabulated (`Space` 2/4/8/12/16/24/32/48,
`Radius` 6/10/14/20/28, `TypeStep` 12/13/15). The geometry below is read from
`ui.rs` and the call sites, not from a screenshot:

| Measure | Value | Where |
|---|---|---|
| Field radius | `Radius::Md` = 10 px | `ui::text_field` |
| Field inner padding | `Space::Md` = 12 px; icon at 42 % of field height; 8 px gap before text | `ui::text_field` |
| Field text | `TypeStep::Body` 15 px; placeholder in `text_dim` | `ui::text_field` |
| Caret | 1.5 px wide, 1.1 × 15 = **16.5 px** tall, accent, **no blink** | `ui::text_field`; no timer for a caret exists anywhere in the shell |
| Field heights in use | **32 px** (desktop New Folder dialog), **30 px** (Files dialog, Trash toolbar), **28 px** (Activity process search) | `src/litric-shell/src/views.rs` lines 359, 1460, 2220, 4179 |
| Omnibox height | toolbar 30 px minus 8 = **22 px**, border only, no fill | `src/litric-shell/src/views.rs` line 1641 |
| Start-page search field | **40 px**, `Radius::Lg`, 14 px padding, no magnifier | `src/litric-shell/src/views.rs` line 1936 |
| Launcher bubble | pill, height from the dock row, 13 px text, elided against the bubble, caret at the end | `src/litric-shell/src/views.rs` line 3633 |
| Progress bar | **6 px** tall at its only call site, track white at 12 %, fill = tone, radius = height / 2 | `src/litric-shell/src/views.rs` line 4072 |
| Status pill | 18 px tall, 4 px dot, `TypeStep::Caption` 12 px bold, 8 px padding, tone fill at 16 % | `ui::badge` |
| Error dialog | 360 × 150 px, `Title3` title, `Footnote` 13 px detail **elided to one line**, 76 × 26 px acknowledge button | `src/litric-shell/src/views.rs` line 1372 onwards |

### 0.4 The state model these four do not share

Part 10's §0.4 documented `ui::State`'s eight variants and the fact that hover and
keyboard focus are merged by `Shell::is_highlighted`
(`src/litric-shell/src/shell.rs` line 3216). None of this part's four controls can
express those eight states:

| Control | State parameter | States it can draw |
|---|---|---|
| `ui::text_field` | `focused: bool` | rest, focused |
| `ui::progress` | none (`fraction`, `tone`) | one |
| `ui::badge` | none (`tone`) | one |
| Omnibox / launcher bubble / error dialog | ad hoc per call site | rest, focused (omnibox); rest (error dialog) |

So the layer's eight-state contract covers the "things you press" half of the
interface and stops at the "things that show you something" half. That is the
structural observation this part is built on, and every item below is a
consequence of it.

### 0.5 Evidence limits of this part

| Limit | Effect |
|---|---|
| Static reading only. No booted image and no rendering harness for these four | What a field or bar draws is a claim about the code. `text_field` and `progress` are not in the state-matrix test, so no hash confirms any of it |
| The Apple pages returned titles and one-line descriptions to the reader used | The macOS column is the weakest of the three, again, and is marked `SNIPPET_ONLY` in the source table. No Apple numeric value is reported here |
| Windows' Forms page gives control dimensions (80 px height, 24 px gaps); GNOME gives none | The shell can be measured against Windows' form layout and against no Linux number at all. That asymmetry is stated per item rather than smoothed over |
| No IME, clipboard or accessibility testing ran | The claims about composition, paste and announcement are readings of the input handlers and of the absence of an accessibility tree |
| No text field is driven by a real keyboard in a test | The strongest claim below (mid-text editing is absent) is a claim about which keys the handlers match, and every branch was read |

---

## F.02 Text fields

Status after this part: **PARTIAL** (all three platforms read; macOS at summary
depth; the editor model and the renderer are compared and do not meet).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use a `TextBox` for short, unformatted, editable text; a specialized control (password, search, date) when one fits, and a `TextBlock` for text that never changes. Label a field whose purpose is not self-evident; in forms Microsoft recommends a **label above the input** and treats a placeholder as a format hint, never the only persistent identification. On tapping populated text the default is an insertion point with no selection; select all on focus only when replacing the whole value is the common action. `IsReadOnly` still allows selection and copy and looks the same as editable (discouraged); a disabled control leaves the tab order. `MaxLength` does **not** limit pasted text. Form layout: **80 px** control height, **24 px** between controls, **48 px** between groups; touch target **40 × 40 epx**, or **32 epx high when at least 120 epx wide**, **44 × 44** for touch-optimized UI. Associate the visible label with `AutomationProperties.LabeledBy`; `InputScope` suggests a keyboard and validates nothing | learn.microsoft.com — [Text box](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/text-box), [Text controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/text-controls), [Forms](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/forms), [Touch interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/touch-interactions), [Keyboard accessibility](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) (T1) |
| macOS 27 | A text field is for entering or editing a **small, specific** piece of text; longer-form input belongs in an editing control made for it. A placeholder may suggest an expected value but disappears, so a separate label can carry the field's purpose. Validate against the field's real requirements (the indexed example is a digits-only field containing other characters) and give a way to correct the value rather than discarding what was typed. Support Full Keyboard Access; keep a logical focus order; make labels, values, validation states and errors available to VoiceOver | developer.apple.com — [Text fields](https://developer.apple.com/design/human-interface-guidelines/text-fields), [Focus and selection](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection), [Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards), [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) (T1, `SNIPPET_ONLY`: titles and summaries only; no size, limit or state rule was verifiable) |
| GNOME 51 | Use a text field for single-line text; entry rows are often preferable in forms and preferences. Size the field to the likely content. Give it a label **or** a placeholder, and use a label when the purpose would not be clear once the field holds text; a placeholder may be an instruction with an ellipsis ("Search for a city…"). Assign an access key to a label. For constrained formats, give feedback; prefer positive feedback while the value is valid over negative feedback during editing, and update as the user types where possible. Embedded symbolic icons and buttons (a spinner, a clear action) are allowed. The HIG prescribes **no** field height, padding, character limit or invalid-field treatment. Keyboard: Tab/Shift+Tab move focus; Ctrl+Tab applies inside a text entry where Tab has another function; every action must be keyboard-operable; controls need short descriptive accessible names | developer.gnome.org — [Text Fields](https://developer.gnome.org/hig/patterns/controls/text-fields.html), [Boxed Lists](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html) (T1) |

All three treat the persistent identification of a field as a requirement in
forms, and none publishes a complete caret, selection or invalid-state
specification in the pages read.

### What the shell has (MEASURED)

`ui::text_field` (`src/litric-shell/src/ui.rs` line 599) takes
`(rect, text, placeholder, focused)` and draws: a filled rounded rectangle
(white at 8 %), a 1 px border (accent at 80 % when focused, white at 12 %
otherwise), **a magnifier icon unconditionally**, then the text or the
placeholder, then a 1.5 px accent caret. It returns the rect.

| Behaviour | Reading | Consequence |
|---|---|---|
| Caret position | Always at the end of the string, or at the field's left edge when empty. The function receives no caret index | A caller cannot show an insertion point anywhere else |
| Selection | No way to express one. The function receives `&str` | A selection, if the model has one, is invisible |
| Overflow | The text is drawn from a fixed x with `font::draw` and **no clip is pushed for the field**. It is bounded only by whatever clip the caller happens to have set | A long filter or name runs over the field's rounded edge and into its neighbours. The Trash search field sits in a toolbar with no field-level clip |
| Hover | Call sites pass `shell.is_highlighted(hot)` as `focused`; that is true for a pointer hover as well as keyboard focus | **Hovering a search field draws an accent border and a caret**, i.e. a picture of a focused field that will not receive a keystroke |
| Blink | None. No timer for a caret exists in the shell | The caret is a static bar |
| Disabled / read-only / error | No parameter for any of them | A field cannot say it is unavailable or invalid. The one disabled-looking field in the product is drawn by hand elsewhere |
| Label | No label parameter. All four call sites rely on a placeholder or on the dialog's title | The Files rename field's placeholder is the **current name**, so the field says nothing about what it wants once the user starts typing |
| Limit | No limit. The live input path is `String::push_str` in every field | See the model note below: the editor model has a limit the input path never consults |
| Icon | `Icon::Search`, every time | The New Folder and Rename fields are magnifier fields |

The four call sites:

| Call site | What it is | Height | Pushed hotspot |
|---|---|---|---|
| `src/litric-shell/src/views.rs` line 359 | Desktop "New folder" dialog field, placeholder from `field_hint` | 32 px | **none** — the field is not hit-testable; it is drawn always-focused |
| `src/litric-shell/src/views.rs` line 1460 | Files dialog field (New Folder, New File, Rename) | 30 px | `Hot::Files(FilesHot::Filter)` |
| `src/litric-shell/src/views.rs` line 2220 | Activity (Task Manager) process search, 300 px max width | 28 px | `Hot::ActivityFilter` |
| `src/litric-shell/src/views.rs` line 4179 | Trash search, 96–260 px wide, in the command toolbar | 30 px | `Hot::TrashFilter` |

The editor model, and where it stops:

| Fact | Evidence |
|---|---|
| `src/litric-shell/src/textedit.rs` implements a full single-line editor: caret, anchor/selection, `select_all`, `select_range`, `insert`, `backspace(word)`, `delete_forward(word)`, `move_left/right(select, word)`, `home`, `end`, `handle_key`, `handle_text`, and a default limit of **8192 chars** | that file, public API from line 57 |
| It is wired to exactly **one key of one dialog**: `dialog_field_key` (`src/litric-shell/src/apps.rs` line 1832) is called once, for `Key::Backspace`, in the Files dialog path (`src/litric-shell/src/shell.rs` line 4774) | grep of `dialog_field_key` returns the definition and one call |
| Every typed character in every field is appended: `push_str` at `src/litric-shell/src/shell.rs` lines 4982 (desktop dialog), 5030 (Files dialog), 5033 and 5044 (omnibox), 5048 (terminal), 5057 (Activity filter) | the same file |
| No field ever receives Left, Right, Home, End, Delete, Ctrl+A or a paste: `Key::Left`/`Right` in the shell are overview navigation, `Delete` is file/Trash deletion guarded by `files.dialog.is_none()`, and there is **no text clipboard** — `Ctrl+V` is the Files *file* paste | `src/litric-shell/src/shell.rs` lines 4624, 4823, 4879, 4965 |
| The Files dialog's `dialog_selection` field can only be set from an `EditOutcome` the one wired key never produces with a non-empty selection | `dialog_field_key` writes `editor.selection()`; Backspace with no selection leaves it `None` |
| The model's own header states the caret can stop between a Thai base character and its combining mark, because it edits per `char` and not per grapheme cluster | `src/litric-shell/src/textedit.rs` module documentation |
| No composition/preedit string exists anywhere in the shell | grep for `preedit`/`Ime` across `src/litric-shell/src` finds no input-side match |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **The shell has a text editor and does not use it.** Caret and selection live in a model that one key of one dialog reaches; six other fields append at the end of a `String` | All three platforms assume ordinary editing semantics: an insertion point, a selection, Home/End/arrow movement, and select-all where replacement is common (Windows names select-on-focus explicitly) | This is the largest finding of the part. A user can place the pointer mid-name and type, and the character lands at the end. A user can press Ctrl+A in a name field and nothing happens |
| The renderer cannot show a caret position or a selection even where the model has one | Windows and macOS both treat selection visibility as part of editing; GNOME's HIG says nothing about it because it is assumed | The Files dialog can move its caret with Home/End only through the model, but the caret is drawn at the end regardless. The visual caret is a lie as soon as it moves |
| The omnibox's "fully selected" address is an invisible flag | macOS and Windows both present a selected address as selected | `omnibox_fresh` clears on the first keystroke, which is correct behaviour, and the user is shown a caret at the end instead of a highlight |
| A magnifier is drawn in a name field | Windows and GNOME both route "search" to a search control and "name" to a text box; a search glyph in a rename field names the wrong action | Live at two call sites (`views.rs` lines 359 and 1460) |
| No clip on the text | Every platform assumes text stays inside its control; the shell word-wraps and elides in lists (`F.19`) but not here | A 60-character filter in a 260 px Trash toolbar can draw across the Empty/Delete buttons |
| Hover draws focus | Windows draws a distinct keyboard-focus visual; the shell's own `State::Focused` ring is unreachable (Part 10, finding 3) | A pointer user is told the field is ready for typing 100 % of the time they are over it, and a keyboard user is not told at all |
| Fields are 28–32 px tall | Windows' form layout: 80 px control height; touch: 44 × 44 | Far below, in both readings. GNOME states no number; Apple's is not retrievable. The density is a deliberate desktop choice that has never been written down |
| No limit tied to the filesystem's 255-byte segment | Windows: set `MaxLength` where the store has a limit, plus a live remaining count; GNOME: size to expected content | `fsbridge::valid_name` refuses >255 bytes, `litric-files::path::MAX_SEGMENT` is 255, the editor model defaults to 8192 and the live path has no limit at all. The refusal arrives after a submit, in a different surface |
| No permanent label on a field whose placeholder is the old name | Windows wants a label above the input; GNOME wants a label when the purpose is unclear once filled | The Rename dialog's title and its "from …" line carry the context, so this is thin rather than absent. The filter fields have no label and their placeholder is the only identification |
| No IME/preedit surface, and per-`char` editing of Thai | Not a rule any of the three states in the pages read; it is a correctness property of the script | Thai vowel and tone marks are separate `char`s. Without a preedit string there is nothing to show while composing, and the caret can be placed inside a cluster |
| No rendering test | — | `every_control_state_renders_differently` (`src/litric-shell/src/e2e.rs` line 1165) covers `button` and `list_item` only. None of this item's claims would fail a test today |

---

## F.18 Progress indicators

Status after this part: **PARTIAL** (all three platforms read; one call site; the
component has no indeterminate form and the shell has no spinner at all).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use a determinate `ProgressBar` when progress is known or predictable and the user can keep working; an indeterminate one when completion time is unknown and the user is not blocked. A `ProgressRing`'s indeterminate state is for an unknown wait that blocks interaction; its determinate form is an alternative visual. Sometimes text alone is enough. Progress controls are **read-only and non-interactive**; pair them with enough text to explain a non-obvious operation. `ProgressRing` minimum **20 × 20 epx**, equal width and height; the `ProgressBar` range defaults to **0–100**. The API exposes `ShowPaused` and `ShowError` visual states (defaults `false`); the design page gives no rule for when to use them. The legacy Win32 UX guide says not to put text on a bar, and says itself that it is not updated for newer Windows | learn.microsoft.com — [Progress controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls), [ProgressBar API](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.progressbar?view=windows-app-sdk-2.0), [Progress Bars — Win32 UX guide](https://learn.microsoft.com/en-us/windows/win32/uxguide/progress-bars) (T1; the Win32 page is legacy and marked as such; the ProgressBar API page was truncated in extraction) |
| macOS 27 | Use a progress indicator to show that work is happening during loading or a lengthy operation; determinate when progress can be measured, indeterminate when it cannot. macOS indeterminate indicators can be **bar or circular**. Keep displayed progress representative; do not imply a precise estimate that does not exist. Show a percentage or time remaining only when it is useful and credible. Expose an accessible name and the current value; announce status changes | developer.apple.com — [Progress indicators](https://developer.apple.com/design/human-interface-guidelines/progress-indicators), [NSProgressIndicator](https://developer.apple.com/documentation/appkit/nsprogressindicator) (T1, `SNIPPET_ONLY`; no size or label number was retrievable) |
| GNOME 51 | Show progress when an operation takes **more than about 3 seconds**; below that it is unnecessary. A **spinner** says work is happening but not how much, so it suits shorter operations; a **determinate bar** suits longer ones — **over 30 seconds** is the rule of thumb. When progress cannot be estimated, an activity-mode bar moves back and forth; avoid it where possible and change mode rarely. A progress bar should carry a **text label saying how much is done**, in task units ("13 of 19 images rotated"), not necessarily a percentage; a long task may add an "about" time estimate. Place it inline near the task; one spinner at a time; offer pause/cancel for destructive or resource-intensive work; a thin bar at the bottom of the header bar is the pattern for background progress. No pixel sizes | developer.gnome.org — [Progress Bars](https://developer.gnome.org/hig/patterns/feedback/progress-bars.html), [Spinners](https://developer.gnome.org/hig/patterns/feedback/spinners.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html) (T1) |

Two of the three name an indeterminate form and all three name the distinction;
GNOME is the only one that gives thresholds (3 s, 30 s) and the only one that asks
for a text label in task units.

### What the shell has (MEASURED)

`ui::progress` (`src/litric-shell/src/ui.rs` line 530) draws a rounded track
(white at 12 %), then a rounded fill of width `r.w * fraction`, floored at the
track height, in the caller's tone. It clamps the fraction to 0–1. It takes no
state, no label and no direction. There is **no** indeterminate variant, and the
shell contains **no spinner**: a grep for `spinner`/`indeterminate` across
`src/litric-shell/src` returns nothing.

| Surface | What it draws | Reading |
|---|---|---|
| Defender scan | a 6 px bar in `Tone::Accent`, then a `TypeStep::Caption` dim line `"{done} of {total}"` below | The only determinate progress in the product, and it does carry a task-unit label — GNOME's requirement, met by the call site rather than the component |
| Scan with `total == 0` | `fraction = 0.0`, because the call site guards the division | An **empty track** while a scan is running: the one state that most needs an activity mode is the one that draws nothing |
| Loading a folder, saving a file, a build | a button label changes to a verb ("Scanning") and `State::Loading` draws three marching dots inside a button or a row | No indicator outside a button. There is no spinner to use |
| Status pill | `ui::badge` (line 509): 18 px pill, tone dot, caption, tone fill at 16 % | **Zero call sites.** A status word with a tone has no surface |
| Error/paused progress | nothing | `ShowPaused`/`ShowError` have no analogue; the bar only knows a fraction |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **No indeterminate or activity form, and no spinner** | Windows (indeterminate bar and ring), macOS (bar or circular indeterminate), GNOME (spinner for short work, activity mode for unknown) | Every operation whose duration is unknown must either lie with a frozen determinate bar or show nothing. The shell currently shows nothing |
| `total == 0` renders an empty bar during a live scan | Windows: indeterminate when completion time is unknown; GNOME: activity mode | The guard prevents a division by zero and produces a picture of "no progress". The honest picture exists in all three platforms' vocabularies and not in the component's |
| The bar's label is a raw count, not task units, and lives outside the component | GNOME: "13 of 19 images rotated"; Windows: pair with enough text | "12 of 400" with no noun is weaker than GNOME's example, and the component cannot carry a label at all, so every future progress needs its own text layout |
| A fill is floored at the track height | No platform states a minimum fill; this is a shell decision | At 1 of 5 000 the bar shows a 6 × 6 px dot rather than a hairline. Defensible; undocumented |
| 6 px bar | Windows gives no thickness; GNOME's thin header-bar variant is "smaller than a regular bar", no number | No external yardstick. Recorded as measured, not as a delta |
| No paused/error progress | Windows exposes both as visual states; neither platform's design page says when | The scan cannot say it failed in place; the failure becomes a dialog (`F.26`) |
| No accessible value | All three require an accessible name/value | The shell has no accessibility tree; `G.15` owns the answer, and this component would be where the value is exposed |
| `badge` unused | — | A tone-coded status word is exactly what a scan result row or a folder state wants; nothing uses it, so its contrast and its 12 px caption are untested by the product |

---

## F.23 Search fields

Status after this part: **PARTIAL** (all three platforms read; macOS at summary
depth; the product's five search surfaces are compared, and no search component
exists to compare).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | `AutoSuggestBox` is for text search with suggestions, or when the user should choose from suggestions as they type. The control separates **filtering suggestions** (`TextChanged`) from **running the query** (`QuerySubmitted`). Suggestions populate as the user types; a **clear-all** affordance appears. A query button is **not shown by default**; set `QueryIcon` (for example `Find`) for a search-box appearance. If nothing matches, show a single-line **"No results"** suggestion so the user knows the request ran. Enter submits from the field and selects a suggestion when the list has focus; keyboard navigation updates the text to the highlighted suggestion. No size is prescribed | learn.microsoft.com — [Auto-suggest box](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/auto-suggest-box), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) (T1; the keyboard page was truncated in extraction) |
| macOS 27 | Use a search field to find content by entering terms; a filter narrows the current set by known criteria (this distinction is a synthesis of the page summary, marked as such). Common elements: **search icon, clear button, placeholder**; a **scope bar** when there are clearly defined categories, with the current scope visible. Search-as-you-type is a useful default where results are responsive. Provide a helpful no-results state and a way to revise or clear the query. Command-F is the familiar macOS search shortcut. No sizes retrieved | developer.apple.com — [Search fields](https://developer.apple.com/design/human-interface-guidelines/search-fields), [Searching](https://developer.apple.com/design/human-interface-guidelines/searching), [Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards) (T1, `SNIPPET_ONLY`) |
| GNOME 51 | Search filters a collection to locate items, and is distinct from *find*, which moves to and highlights a match. Provide it when there are more textual items than users can scan, but not as the only way to find things. The standard pattern is a search bar that slides down under the header bar, activated by typing while unfocused, by **Ctrl+F**, or by a search toggle; results are live and incremental; when nothing matches, show feedback in the content view — a simple **"No results"** label is often enough. Global search, tolerant matching and relevant ordering are recommended. The HIG prescribes no placeholder wording and no size. GTK's `SearchEntry` (an implementation detail, not HIG) shows a find icon when empty and a clear icon when filled, and clicking it clears | developer.gnome.org — [Search](https://developer.gnome.org/hig/patterns/nav/search.html), [Text Fields](https://developer.gnome.org/hig/patterns/controls/text-fields.html); GTK — [Gtk.SearchEntry](https://docs.gtk.org/gtk4/class.SearchEntry.html) (T1; the GTK page is T2-level implementation detail and is labelled as such) |

All three name a clear affordance and a no-results message. Windows and GNOME
name a keyboard route into search (Enter / Ctrl+F); macOS names Command-F.

### What the shell has (MEASURED)

There is no search component. Search is drawn in **four** shapes plus a fifth
surface that has no field at all, and the two filter fields are the generic text
field from `F.02`, which bakes in the magnifier:

| Surface | Where | Clear affordance | No-results state | Keyboard route |
|---|---|---|---|---|
| Activity filter | `ui::text_field`, 28 px, placeholder "Search processes" | none | table empties; the count line above changes | Tab or click only |
| Trash filter | `ui::text_field`, 30 px, placeholder "Search trash", in the command toolbar | none | **"No items match this search"** (`views.rs` line 4259) | Tab or click only |
| Files filter | a readout only: `"filter: {text} ({n} items · {m} selected)"` in the command row; the text itself is typed into the window, not a field | none | **"Empty folder"** (`views.rs` line 1287) — the same message the folder shows when it really is empty | `Ctrl+F` sets `FilesFocus::Table`, **not** the filter (`shell.rs` line 4896) |
| Browser omnibox | bespoke: 22 px border-only field in the toolbar (line 1641), plus a second bespoke 40 px field on the Litric Search start page (line 1936) | none | the start page's engine chips | click; the field is `Hot::BrowserOmnibox` |
| Launcher search | bespoke pill: magnifier, 13 px text elided against the bubble, caret at the end | an **✕** that is `Hot::DockCollapse` → `Action::CloseOverlay` (`shell.rs` line 5521) — it closes the launcher instead of clearing the query | **"Nothing matches that"** (`views.rs` line 3797) | typing while the launcher is open; Ctrl+F not bound |

Additional measured facts:

| Fact | Evidence |
|---|---|
| Search URLs are percent-encoded per engine and empty queries are refused | `SearchEngine::search_url` uses `litric_core::pct::encode`; `commit_omnibox` trims and returns `None` on empty (`apps.rs` lines 851, 1040) |
| The user can choose among **Google, DuckDuckGo and Bing**, and the start page draws the choice as chips with an accent fill and a check glyph, so the scope is not carried by colour alone | `apps.rs` line 806; `views.rs` from line 1986 |
| The start page's placeholder is hardcoded **"Search the web with Google, or type a URL"** | `views.rs` line 1957, while the engine can be DuckDuckGo or Bing |
| The toolbar has no engine indicator. The chosen engine is visible only while the start page is up | `views.rs` line 1633 onwards |
| `Hot::LauncherSearch` is a **dead hotspot**: it is declared, counted as a text field by `keyboard_focus_is_text_field`, and its activation returns an empty vector — but no surface ever pushes it | `shell.rs` lines 302, 3259, 5470; no `push(…, Hot::LauncherSearch)` exists |
| When the launcher is searching, the bubble pushes no hotspot at all (`Hot::DockSearch` is only pushed when not searching), and its clear-looking ✕ collapses the overlay | `views.rs` line 3682 |
| The browser's caret is drawn at the end of the address even though focus "selects" it | `views.rs` lines 1661–1672; `omnibox_fresh` in `shell.rs` line 5762 |
| No `Ctrl+F` (or any other shortcut) focuses a filter field anywhere | `shell.rs` line 4896 is the only Ctrl+F, and it selects the file table |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **No clear affordance in any filter field** | Windows (clear-all appears), macOS (clear button), GNOME/GTK (clear icon when filled) | Three platforms, three implementations, zero in the shell. Clearing is either Escape (Activity only) or selecting and deleting, and the field has no selection |
| The launcher's ✕ closes the launcher | The visual grammar all three platforms use: an ✕ inside a field clears the field | The one search surface with a clear-looking control gives it the wrong behaviour. A user who wants to refine a query loses the whole launcher |
| `Ctrl+F` in Files focuses the table | GNOME names Ctrl+F as the search route; the Files window calls its own filter "Search" nowhere in the UI | The shortcut that means "find" everywhere else means "focus the list" here. There is no keyboard route to the filter at all |
| **Two vocabularies for "nothing matched"**: Trash and the launcher say so; Files says "Empty folder" | Windows: show "No results" so the user knows the request ran; GNOME: feedback in the content view | In Files, a filter that hides every row is reported as an empty directory. The command row's `filter:` readout is the only clue |
| The magnifier is baked into the generic field | Windows and macOS treat search as a distinct control with its own affordances | It appears in two rename/create dialogs, and it is the *only* search affordance in the two filters — so the shell has the icon everywhere and the control nowhere |
| The placeholder names Google while the engine may be DuckDuckGo or Bing | All three platforms ask for the current scope to be visible; honesty about where a query goes is the shell's own privacy rule | A user who picked DuckDuckGo is told, in the field they type in, that the query goes to Google. The start page's chips contradict the placeholder one row below |
| The engine is not visible once the start page is gone | Apple: make the current scope evident via placeholder, scope bar or title | The only engine cue left is the resulting URL |
| `Hot::LauncherSearch` is dead and counted | — | It appears in `keyboard_focus_is_text_field`, so the shell believes a search field is focusable that is not drawn. A future reader would wire behaviour to it and see nothing happen |
| No suggestions, no incremental result count in the field, no scope bar | Windows (`AutoSuggestBox`), Apple (scope bar), GNOME (incremental results) | The filters are literal substring filters over a row list; the browser has no suggestion surface. This is a capability gap, not a defect in what exists |
| Empty query is refused | — | Correct against all three: nothing is sent when the field is empty |
| Percent-encoding is per engine | — | Correct: `pct::encode` before the query is placed in the URL |

---

## F.26 Errors and validation

Status after this part: **PARTIAL** (all three platforms read; macOS at summary
depth; the shell's two error surfaces and its one validation function are
compared).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Put a **field error next to the invalid control**, mark the field invalid, and include text — not colour alone. A validation popup is also mentioned for a character limit. An **InfoBar** is for a visible, non-intrusive **app-wide** state (connectivity, an automatic save failure), takes layout space, and must not carry field-specific errors; severity and text must agree; do not rely on colour or icon alone. A **ContentDialog** is for important or blocking information and is explicitly *not* for contextual validation errors such as an invalid password field. Error text should identify the issue in the first line and be direct ("Name is required", "Enter a valid email address"); keep server-side errors distinct from local format errors; preserve the user's input; if the failing field is off-screen, move focus or provide a summary. Bind the error to `AutomationProperties.HelpText`; changing an open InfoBar's message does not notify a screen reader, so close and reopen it if announcement matters | learn.microsoft.com — [Build a data-entry form with validation](https://learn.microsoft.com/en-us/windows/apps/get-started/line-of-business/build-validated-form), [InfoBar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/infobar), [Dialog controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs), [Forms](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/forms), [Expose basic accessibility information](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information) (T1) |
| macOS 27 | Reserve **alerts** for important, immediate information — especially problems and consequences — rather than routine field feedback. An alert can have a title, optional informative text and **up to three buttons**; button titles should be **one or two words**, describe the result, and preferably be verbs. Field-specific, correctable errors are better shown near the field (synthesis of the page summaries, marked as such). Make the problem and the next step understandable; preserve what was entered; return the user to a useful correction point such as the invalid field; keep the error available to VoiceOver and never colour-only | developer.apple.com — [Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts), [Text fields](https://developer.apple.com/design/human-interface-guidelines/text-fields), [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) (T1, `SNIPPET_ONLY`; the only numbers retrieved are "up to three buttons" and one- or two-word titles) |
| GNOME 51 | For a field with a required format, give feedback in the field's context; prefer positive feedback while the value is valid over negative feedback during editing, and update while typing where practical. For a **simple, non-critical** app error, avoid a disruptive dialog where possible — a **toast** is a good alternative; toasts are transient event feedback, banners are for important ongoing states, and neither is for one-off messages. An alert dialog is for something the user must see and respond to; error dialogs should be avoided where possible. Set initial keyboard focus on the component the user is expected to operate. The HIG prescribes no error wording template and no invalid-field visual treatment; GTK exposes `.error`/`.warning` style classes (implementation detail) | developer.gnome.org — [Text Fields](https://developer.gnome.org/hig/patterns/controls/text-fields.html), [Dialogs](https://developer.gnome.org/hig/patterns/feedback/dialogs.html), [Toasts](https://developer.gnome.org/hig/patterns/feedback/toasts.html), [Banners](https://developer.gnome.org/hig/patterns/feedback/banners.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html) (T1) |

The three agree on: a correctable field problem belongs at the field; a modal is
for something that must be seen; the message must say what to do; and the input
must survive the failure. They disagree on the preferred non-modal channel
(InfoBar / alert / toast), and only Windows and GNOME give wording rules.

### What the shell has (MEASURED)

Validation exists in one function, far from the field:

| Fact | Evidence |
|---|---|
| `fsbridge::valid_name` refuses an empty name, a name longer than 255 bytes, a leading `.`, `/`, `\`, `..`, `://` or any control character | `src/litric-shell/src/fsbridge.rs` line 94 |
| It runs **after** the submit. Nothing checks the field as the user types, and nothing disables the Create/Confirm button while the name is empty | `submit_files_dialog` (`shell.rs` line 2303) calls the bridge; the dialog's OK button is drawn with `State::Normal` unconditionally (`views.rs` line 1442) |
| On a booted image the bridge delegates to the Files daemon **before** the local check, so the refusal is the daemon's | `fsbridge.rs` line 412: `if let Some(daemon) = &self.daemon { return daemon.mkdir(parent, name); }` precedes `valid_name` |
| A failure **replaces** the dialog with `DialogKind::Error(String)`, which has no field, no Cancel, one acknowledge button, and a title of "Could not finish" | `shell.rs` lines 2293, 2360, 2604; `views.rs` line 1372 |
| The typed name does not survive: `DialogKind::Error` closes to nothing, and the next New Folder / Rename resets the field (`start_new_folder` clears it; `start_rename` sets the current name) | `apps.rs` lines 691, 716 |
| The error detail is **elided to one line** at `TypeStep::Footnote` in a 360 px panel | `views.rs` line 1422: `ui::elide(&detail, &s_detail, panel.w - Space::Md * 2)` |
| The acknowledge button is drawn in `Tone::Bad` — the same tone as "Delete forever?" and "Empty permanently?" | `views.rs` line 1442; the destructive confirms at line 4195 use the same tone for the destructive action |
| The shell *has* an inline error line for the Files table: `app.error` is drawn in `Tone::Bad` `Footnote` where the rows would be, and the rows are cleared | `views.rs` line 1193; `apps.rs::apply_error` |
| That inline line is not elided or wrapped | `views.rs` line 1195 draws the raw string |
| Success is a notification ("Files", "Done"); failure is a modal. No toast is used for a simple failure | `shell.rs` line 2287 (success) vs 2293 (failure); the notification system exists and is used for the success case |
| The status pill that could carry a failure tone has no call site (`F.18`) | `ui::badge`, 0 uses |
| There is no error state on the text field, so the field cannot be marked invalid even if a caller wanted to | `ui::text_field` signature |
| No accessibility tree, no announcement, no focus return | grep: no AT-SPI/D-Bus accessibility surface in the shell; the dialog sets no focus after the error |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **Validation is a round-trip, not a field state**: the user learns a name is empty or too long only after submitting, from a different surface | Windows (mark the field invalid, text next to it; disable submit when the reason is clear), GNOME (feedback in the field's context, ideally while typing), Apple (validate against the field's real requirements and give a way to correct) | All three place the message at the field. The shell cannot: the component has no error state, and the check lives in the bridge |
| **The typed name is destroyed by the failure** | Windows: preserve the user's input; Apple: preserve and return to a correction point | In Files, a failed rename closes the form and loses the new name. The one input a user most wants back is the one the error path discards |
| The error is elided to a single line in a 360 px dialog | Windows: identify the issue in the first line; Apple: make the problem and next step understandable; GNOME: short and unambiguous | `"the filesystem is read-only, so nothing can be written there"` does not fit; it is cut, and there is no second line, no expansion and no copy |
| The acknowledge button is danger-toned | Apple: button titles describe the result, verbs, one or two words; the destructive colour is for destructive actions | A red "OK" tells the user they are about to destroy something. They are closing a message |
| A simple failure becomes a modal | GNOME: avoid a dialog for a simple non-critical error; a toast is a good alternative. Windows: InfoBar for app-wide state | The shell has notifications and uses them for success only. A failed delete interrupts the whole desktop, and the Files window behind it is still usable |
| The two error surfaces disagree about how to show text | Windows: inline for fields, InfoBar for app-wide, dialog only for blocking | The Files table has an inline red line; the dialog has an elided grey line; neither is the component's error state. The table's line can overflow the window |
| No "Name is required" case at all | Windows names it as the canonical message; all three treat required-field feedback as basic | Pressing Create with an empty name asks the bridge or the daemon, and the answer arrives as a failure. `valid_name` would refuse it locally, but only after the round-trip and only without a daemon |
| The error copy is human, not a code | — | `FsError`'s `Display` writes sentences ("an item with that name already exists"). This is the one part of the error path that is already right, and it is worth saying because the daemon path's `e.to_string()` also produces text, not a token, for `FsError` |
| No announcement, no focus return | Windows (HelpText plus focus movement), Apple (VoiceOver, return to the field), GNOME (initial focus in a dialog) | `G.15` owns the accessibility answer. The focus half is local: closing the error leaves focus wherever it was |
| No test | — | No test asserts that a failed create leaves the name in the field, because it does not |

---

## 5. Findings this part files without deciding

Continuing Part 10's numbering. None of these is a ledger row: a row needs a
decision with a backend and a justification that a research note does not have.

| # | Finding | Where it is visible | Owner |
|---|---|---|---|
| 9 | The editor model (`TextEdit`) reaches one key of one dialog; every other field appends at the end, with no caret movement, selection, Home/End/Delete or Ctrl+A | `apps.rs` line 1832; `shell.rs` line 4774; `push_str` at 4982/5030/5033/5044/5048/5057 | `F.02` decision, then the input path |
| 10 | The renderer cannot show a caret index or a selection | `ui::text_field` signature, `ui.rs` line 599 | `F.02`, with 9 |
| 11 | A magnifier is drawn in New Folder and Rename fields | `views.rs` lines 359, 1460 | `F.02` / `F.23` |
| 12 | No clip on field text: overflow draws over neighbours | `ui::text_field` | `F.02` |
| 13 | Hovering a field draws the focus border and a caret | `is_highlighted` → `focused`, e.g. `views.rs` line 2216 | `G.14` keyboard-only navigation, with Part 10 finding 3 |
| 14 | No error, disabled or read-only state on a field | `ui::text_field` | `F.02` / `F.26` |
| 15 | No limit tied to the 255-byte name rule; the live path is unbounded | `fsbridge::valid_name`, `textedit::DEFAULT_MAX_CHARS` | `F.02`, with `F.26` |
| 16 | No indeterminate progress and no spinner; `total == 0` renders an empty bar | `ui::progress`; `views.rs` line 4072 | `F.18` |
| 17 | `badge` has no call site | `ui::badge`, `ui.rs` line 509 | `F.18` / `F.26` |
| 18 | No clear affordance in filter fields; the launcher's ✕ closes the launcher instead of clearing | `views.rs` line 3682; `shell.rs` line 5521 | `F.23` |
| 19 | `Ctrl+F` in Files focuses the table, not the filter | `shell.rs` line 4896 | `F.23` |
| 20 | Files says "Empty folder" when a filter hides every row | `views.rs` line 1287 vs line 4259 (Trash) | `F.23` / `F.24` |
| 21 | The start page placeholder names Google regardless of the selected engine | `views.rs` line 1957 | `F.23` |
| 22 | A failed name round-trip replaces the form and loses the typed name | `shell.rs` line 2360; `apps.rs` lines 691, 716 | `F.26` |
| 23 | The error detail is elided to one line; the acknowledge button is danger-toned | `views.rs` lines 1422, 1442 | `F.26` |
| 24 | Neither `text_field` nor `progress` appears in the state-matrix test | `e2e.rs` line 1165 | Test change |

## 6. What this part does not establish

| Claim | Why not |
|---|---|
| that any of the four items is correct | This part compares guidance with an implementation. It found sixteen things, decided none, and a decided-differently outcome is possible for every one |
| that the component layer is now catalogued | Part 10 covered four exports and this part three. `Column`, `Hotspots`, `cursor`, `wrap`, `elide`, `section_label`, `separator` and `focus_ring` are read as context and are not items |
| that `F` is mostly researched | 19 of 27 items remain `PENDING`. Eight are now `PARTIAL`, none is `COMPLETE` |
| that the macOS column is as strong as the other two | every Apple page returned a title and a summary. No Apple number, state rule or wording rule is reported as verified |
| that a user can type Thai | The model's per-`char` editing is documented in its own header, and there is no preedit surface. No keyboard, IME or composing test ran |
| that anything was seen on screen | no image was booted, no pixel measured, no field clicked. Every rendering claim is a claim about the code that draws it |
| that the error copy is good because `FsError` is | one error path is human-readable. The daemon path's text and the elision of long messages are separate facts, and the second is a defect |

## 7. Coverage added, and what would finish these four items

| Item | Before | After | What is still missing |
|---|---|---|---|
| `F.02` Text fields | `PENDING` | `PARTIAL` | macOS at T1 depth; a decision on the editor (wire `TextEdit` into the input path and pass caret/selection to the renderer, or delete it and own append-only); a limit policy; an IME answer |
| `F.18` Progress indicators | `PENDING` | `PARTIAL` | macOS at T1 depth; a decision on whether the shell gets an indeterminate form (and whether `badge` is adopted or deleted); a paused/error policy |
| `F.23` Search fields | `PENDING` | `PARTIAL` | macOS at T1 depth; a decision on one search component versus four surfaces; a clear-affordance and Ctrl+F policy; engine visibility in the toolbar |
| `F.26` Errors and validation | `PENDING` | `PARTIAL` | macOS at T1 depth; a decision on inline-versus-modal and on preserving input; a field error state in the component; a wording rule (the shell has good sentences in `FsError` and no rule for new ones) |

What would move each to `COMPLETE`: the macOS column at the same depth as the
other two, one decision per item with a backend and a milestone, and the two
lines that add `text_field` and `progress` to the state-matrix test — which would
not catch findings 9 or 10, because those are about the input path and the
function signature, not about a state collapsing.

## Sources read for this part

Access date for all rows: 2026-10-26 (Asia/Bangkok). Tier definitions: `Part1` §2.1.

| Source | Tier | Status | Limitations |
|---|---|---|---|
| [Text box — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/text-box) | T1 | INSPECTED | No general field height or padding; the page's `MaxLength`-does-not-limit-paste caveat is explicit |
| [Text controls — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/text-controls) | T1 | INSPECTED | Overview only |
| [Forms — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/forms) | T1 | INSPECTED | 80/24/48 px layout values are this page's; they are form layout, not a TextBox rule |
| [Build a data-entry form with validation — WinUI 3](https://learn.microsoft.com/en-us/windows/apps/get-started/line-of-business/build-validated-form) | T1 | INSPECTED | Sample messages are examples |
| [Auto-suggest box — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/auto-suggest-box) | T1 | INSPECTED | No sizing rule |
| [Progress controls — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/progress-controls) | T1 | INSPECTED | No bar thickness; no design rule for paused/error |
| [ProgressBar API — Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.progressbar?view=windows-app-sdk-2.0) | T1 | INSPECTED | Partial extraction; the 0–100 default and the visual-state properties were present |
| [Progress Bars — Win32 UX guide](https://learn.microsoft.com/en-us/windows/win32/uxguide/progress-bars) | T1 (legacy) | INSPECTED | Microsoft marks it as Windows 7-era; used only for the "no text on the bar" note, never as a current rule |
| [InfoBar — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/infobar) | T1 | INSPECTED | — |
| [Dialog controls — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs) | T1 | INSPECTED | — |
| [Keyboard accessibility — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/keyboard-accessibility) | T1 | INSPECTED | — |
| [Keyboard interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) | T1 | INSPECTED | Partial extraction; the AutoSuggestBox case was present |
| [Touch interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/touch-interactions) | T1 | INSPECTED | Target sizes are general, not TextBox-specific |
| [Expose basic accessibility information — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information) | T1 | INSPECTED | — |
| [Text fields — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/text-fields) | T1 | SNIPPET_ONLY | Title and description only |
| [Progress indicators — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/progress-indicators) | T1 | SNIPPET_ONLY | as above |
| [Search fields — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/search-fields) | T1 | SNIPPET_ONLY | as above |
| [Searching — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/searching) | T1 | SNIPPET_ONLY | as above |
| [Alerts — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/alerts) | T1 | SNIPPET_ONLY | "Up to three buttons" and the one- or two-word title guidance survived in the summary |
| [Focus and selection — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection) | T1 | SNIPPET_ONLY | as above |
| [Keyboards — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/keyboards) | T1 | SNIPPET_ONLY | as above |
| [Accessibility — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/accessibility) | T1 | SNIPPET_ONLY | as above |
| [NSProgressIndicator — AppKit](https://developer.apple.com/documentation/appkit/nsprogressindicator) | T1 | SNIPPET_ONLY | as above |
| [Text Fields — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/text-fields.html) | T1 | INSPECTED | No height, padding, limit or invalid-field treatment |
| [Progress Bars — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/progress-bars.html) | T1 | INSPECTED | 3 s / 30 s rules; no pixel sizes |
| [Spinners — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/spinners.html) | T1 | INSPECTED | as above |
| [Search — GNOME HIG](https://developer.gnome.org/hig/patterns/nav/search.html) | T1 | INSPECTED | No placeholder wording, no size |
| [Dialogs — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/dialogs.html) | T1 | INSPECTED | — |
| [Toasts — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/toasts.html) | T1 | INSPECTED | — |
| [Banners — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/banners.html) | T1 | INSPECTED | — |
| [Boxed Lists — GNOME HIG](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html) | T1 | INSPECTED | Entry rows described; no dimensions |
| [Keyboard — GNOME HIG](https://developer.gnome.org/hig/guidelines/keyboard.html) | T1 | INSPECTED | Ctrl+F named as the search route |
| [Accessibility — GNOME HIG](https://developer.gnome.org/hig/guidelines/accessibility.html) | T1 | INSPECTED | — |
| [Gtk.Entry — GTK 4](https://docs.gtk.org/gtk4/class.Entry.html) | T2 (implementation detail) | INSPECTED | Partial extraction; `.error`/`.warning` and the icon-accessibility caveat were present |
| [Gtk.SearchEntry — GTK 4](https://docs.gtk.org/gtk4/class.SearchEntry.html) | T2 (implementation detail) | INSPECTED | Describes behaviour, not HIG policy |
| [Gtk.ProgressBar — GTK 4](https://docs.gtk.org/gtk4/class.ProgressBar.html) | T2 (implementation detail) | INSPECTED | Accessibility role only |
| [Gtk.Spinner — GTK 4](https://docs.gtk.org/gtk4/class.Spinner.html) | T2 (implementation detail) | INSPECTED | Accessibility role only |

Repository sources read (no tier): `src/litric-shell/src/ui.rs`,
`src/litric-shell/src/textedit.rs`, `src/litric-shell/src/shell.rs`,
`src/litric-shell/src/apps.rs`, `src/litric-shell/src/views.rs`,
`src/litric-shell/src/fsbridge.rs`, `src/litric-shell/src/e2e.rs`,
`src/litric-files/src/lib.rs`, `src/litric-files/src/path.rs`,
`src/litric-design/src/lib.rs`.
