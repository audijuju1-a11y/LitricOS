# LitricOS UI/UX Research — Part 19 of N

Chapter 41 (G) — Contrast, keyboard and screen readers: the text passes AA,
the borders do not, and nothing can read the screen aloud

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P19 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-28 (Asia/Bangkok) |
| Scope of this part | Chapter 41 (`G`): `G.10` contrast and high-contrast themes, `G.14` keyboard-only navigation and `G.15` screen readers and accessibility APIs. The other 15 `G` items keep their status and their macOS-only evidence |
| Batch | B3/B6 continuation (`Part1` §2.2) — the tokens/accessibility chapter, first Linux measurement pass |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (3 notes amended; statuses unchanged at `PARTIAL`), `Research-Deep-all/LitricOS Source Register.csv` (5 sources appended, S3834–S3838). This part also carries a **repair**: three `F.09` findings fixed in `views.rs`/`apps.rs`, with tests |
| Predecessors | `Part 7` (macOS tokens), `Part 10` (`F.01` focus states), `Part 14` (`F.12`/`F.13` keyboard routes), `Part 16` (`F.14`), `Part 17` (`F.09`) |
| Resume point | `G.01`–`G.09`, `G.11`–`G.13` and `G.16`–`G.18` still need their Linux half. This part measured three of the eighteen |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and what was repaired

### 0.1 Three questions, three different kinds of evidence

| Item | Question | How it is answered here |
|---|---|---|
| **`G.10`** Contrast | *Can I read it?* | every colour pair in all four themes is computed from the palette literals with the WCAG 2.1 formula, not eyeballed |
| **`G.14`** Keyboard | *Can I reach it without a pointer?* | source census plus the probe findings of Parts 14–17, which drove the real input path |
| **`G.15`** Screen readers | *Can assistive technology see the interface?* | dependency and symbol census: an accessibility API either is linked or is not |

The first is arithmetic and can be exact. The second is a count of routes and a
list of the surfaces with none. The third has a binary answer that decides the
whole item: the shell draws pixels into a canvas, so unless it also publishes a
tree of roles, names and states, no screen reader can see anything — no matter
how good the labels are.

### 0.2 Method (MEASURED)

Contrast is computed with the same formula the design crate implements in
`src/litric-design/src/color.rs` (WCAG 2.1 relative luminance, gamma decode,
`(L1+0.05)/(L2+0.05)`). The palette literals were read from
`Theme::of` in `src/litric-design/src/lib.rs` (lines 200–270) and the six accent
pairs from `Theme::with_accent` (lines 278–310). The run covers **4 themes × 10
pairs = 40 ratios**, **12 accent ratios** and **2 focus-ring composites**.

The keyboard census counts registered hotspots against key bindings:
`Hot` variants in `shell.rs`, bindings in `shortcuts.rs`, and the probes already
recorded in Parts 14 (menus), 15 (dialogs) and 16 (notification centre).

The screen-reader census is `rg -i 'accesskit|atspi'` over the shell's manifest
and source, and a search for any accessible name, role or state store. It
returns **nothing**.

### 0.3 The numbers this part is built on

| Measure | Value |
|---|---|
| Text pairs measured | **36** (9 per theme × 4 themes) |
| Text pairs at or above AA 4.5:1 | **36 of 36**; the lowest is **4.97** (Light `success` on base) |
| High-contrast pairs at or above AAA 7:1 | **20 of 20**; the lowest is **10.00** |
| Border-on-base pairs at or above 3:1 (WCAG 1.4.11) | **2 of 4**; Dark **1.44**, Light **1.27** |
| Accent pairs at or above AA | **12 of 12**; lowest **5.35** (Light accent on base) |
| Accent pairs at or above AAA | **9 of 12**; Light Default/Sky **5.74**, Teal **6.42** |
| Focus ring width / colour | **2 px**, accent at 90 % alpha |
| Focus ring composite against base | **6.59** Dark, **4.47** Light — both clear 3:1 |
| Accessibility-API dependencies in `litric-shell` | **0** |
| Accessible name/role/state store | **0** |

Two sentences follow, and the rest of the part is their evidence: **the text
system is genuinely accessible; the border system is not; and the assistive
layer does not exist.**

### 0.4 What this part repaired

`Part 17` recorded three navigation findings that were cheap, local and testable.
They were fixed in this pass, before the rest of the chapter was written, because
a corpus that only records defects stops being useful the moment someone is
willing to fix them.

| Finding | Repair | Where |
|---|---|---|
| **F09.03** Back/Forward/Up always looked enabled | `FilesApp::can_go_back/forward/up`; the bar draws `State::Disabled` and **registers no hotspot** when there is nowhere to go | `apps.rs` (methods near `go_up`), `views.rs` (nav block) |
| **F09.04** breadcrumb ran past the bar | `crumb_window()` drops leading crumbs for an ellipsis and keeps the current folder; labels elide against the remaining width | `views.rs` (`crumb_window`, crumb loop) |
| **F09.05** root was painted text with no hotspot | a root crumb `/` is drawn first, always, with `FilesHot::Root` and an action that navigates to `/` | `apps.rs`, `views.rs` |

Four tests were added: `history_buttons_know_when_they_have_nowhere_to_go`,
`the_root_crumb_goes_home_from_any_depth`,
`breadcrumbs_drop_their_head_before_their_tail`, and an addition to the Files
journey test that asserts Back is **absent** with an empty history and appears
once there is one. They pass under `cargo test -p litric-shell --lib`.

---

## G.10 — Contrast and high-contrast themes

### G.10.1 What the standards and platforms require

| Source | Rule |
|---|---|
| **WCAG 2.2, SC 1.4.3** (AA) | Text needs **4.5:1**; large text (≈18 pt, or 14 pt bold) **3:1**. Inactive components, decoration and logotypes are exempt. Thresholds are not rounded: 4.499 does not pass |
| **WCAG 2.2, SC 1.4.11** (AA) | **Non-text** content — the visual information needed to identify a UI component and its states — needs **3:1** against adjacent colours. A 1 px border that is the only boundary of a control is exactly this case |
| **WCAG 2.2, SC 2.4.13** (AAA) | A keyboard focus indicator must cover an area at least equal to a **2 CSS px** perimeter and change contrast by at least **3:1** between focused and unfocused states |
| **GNOME 51** | High-contrast mode is one of the four checks the HIG names for every app: "All parts of the UI should be correctly rendered in the high-contrast style", alongside large text, keyboard navigation and screen reader |
| **macOS 27** | Accessibility: "Accessible user interfaces empower everyone to have a great experience"; page body did not load (summary depth) |

The shell's high-contrast themes claim AAA in their own documentation
(`ThemeKind::requires_aaa`, `lib.rs` 63–66), which makes them testable against a
number rather than against taste. That claim holds; the standard themes' borders
are a different story.

### G.10.2 Measured contrast (MEASURED)

Ratios computed from the palette literals. `bg` is `bg_base`; `elev` is
`bg_elevated`.

| Theme | text/bg | text/elev | dim/bg | dim/elev | on_accent/accent | accent/bg | danger/bg | success/bg | warning/bg | border/bg |
|---|---|---|---|---|---|---|---|---|---|---|
| **Dark** | 17.76 | 16.27 | 9.06 | 8.30 | 8.12 | 7.93 | 7.05 | 11.09 | 12.62 | **1.44** |
| **Light** | 17.64 | 18.91 | 7.03 | 7.54 | 5.74 | 5.35 | 5.26 | 4.97 | 5.53 | **1.27** |
| **HC Dark** | 21.00 | 19.80 | 15.91 | 15.00 | 11.10 | 11.10 | 10.34 | 14.40 | 15.54 | 21.00 |
| **HC Light** | 21.00 | 21.00 | 17.40 | 17.40 | 10.00 | 10.00 | 10.09 | 10.00 | 10.29 | 21.00 |

**Accent pairs** (`on_accent` over `accent`, then `accent` as a text tone over
`bg`):

| Theme | Default/Sky | Violet | Teal | Amber | Rose |
|---|---|---|---|---|---|
| **Dark** | 8.12 / 7.93 | 8.73 / 8.58 | 11.46 / 12.13 | 11.95 / 12.44 | 10.46 / 10.54 |
| **Light** | 5.74 / 5.35 | 7.78 / 7.26 | 6.42 / 5.99 | 7.20 / 6.72 | 7.17 / 6.69 |

**Focus ring**: 2 px at 90 % accent over base — composite `#6097e7` on Dark
(**6.59:1** against the base), `#336eda` on Light (**4.47:1**). Both clear the
3:1 that 2.4.13 and 1.4.11 ask for.

### G.10.3 What the shell has

The token system is semantic, not literal: components ask for `t.text()`,
`t.text_dim()`, `t.tone(...)` and never for a hex value (`Palette`, `lib.rs`
155–180). Four themes exist and the settings surface enumerates them; the accent
is a slot with six values and high-contrast themes refuse it by design
(`Accent` docs, `lib.rs` 75–92).

The measurement confirms the two claims the crate makes for itself:

1. **Every text pair is AA in every theme.** The worst is 4.97, and it is a
   success-green on the light base — a status colour, not body text.
2. **Both high-contrast themes are AAA across the board**, minimum 10.00,
   including the border.

And it contradicts one assumption nobody wrote down: **the 1 px border is not a
visible boundary in either standard theme.** At 1.44:1 (Dark) and 1.27:1
(Light) it is below the 3:1 that 1.4.11 requires for the only thing identifying
a text field, a card or a sidebar edge. The border token is doing decorative work
while the components use it structurally.

### G.10.4 Findings

| # | Finding | Where |
|---|---|---|
| **G10.01** | **All 36 text pairs pass AA**, in all four themes. The palette's text half is correct and should not be touched | `lib.rs` 200–270; measurement above |
| **G10.02** | **Both high-contrast themes pass AAA on all 20 pairs**, minimum 10.00. The `requires_aaa` claim is true, and it is true of borders too | `lib.rs` 63–66, 232–266 |
| **G10.03** | **The border fails 1.4.11 in both standard themes**: 1.44 on Dark, 1.27 on Light. It is the only visual boundary of a text field (`text_field`, `ui.rs`), a card and a sidebar edge. The failure is structural, not cosmetic | `lib.rs` 210, 226; `ui.rs` text_field |
| **G10.04** | **Every accent passes AA** for text on the accent and as a text tone; the weakest is Light's 5.35. Nine of twelve also pass AAA, so the picker cannot produce an unreadable selection | `lib.rs` 278–310 |
| **G10.05** | **The focus ring is correct by the standard**: 2 px, accent at 90 %, 6.59:1 Dark and 4.47:1 Light against the base. It is one of the few components that meets a AAA criterion | `ui.rs` 258–266; measurement |
| **G10.06** | **The ring is often not drawn.** `Part 10` found `State::Focused` unreachable in the button path because one function merges pointer and keyboard. A correct indicator that no control reaches is a specification, not a feature | `Part 10` §F.01; `ui.rs` 260 |
| **G10.07** | **Glass and scrim are not measured.** The contrast table uses opaque surfaces; text over a translucent panel depends on the wallpaper behind it, and the glass tint's floor is enforced by a different test than this one | `Glass` docs, `lib.rs` 330–470; limitation in §2 |
| **G10.08** | The theme set is **enumerable and stable** (`ThemeKind::all`, `Accent::all`), so a future contrast gate can iterate exactly what the settings UI offers instead of a hand-written list | `lib.rs` 47–58, 100–120 |

Status for `G.10`: **PARTIAL** — the Linux side is now measured exactly; Windows
and GNOME platform token tables (as opposed to WCAG/GNOME's accessibility rule)
were not re-read in this part.

---

## G.14 — Keyboard-only navigation

### G.14.1 What the platforms require

| Source | Rule |
|---|---|
| **GNOME 51** | "Use the app using only the keyboard. Can every part of the app be navigated to and interacted with? Does the UI follow the keyboard navigation guidelines?" Accessibility names and high contrast are listed beside it; keyboard is one of four checks, not an optional extra |
| **WCAG 2.2, SC 2.4.13** | A visible focus indicator is required to be visible; its minimum size and contrast are the ones measured in `G.10` |
| **macOS 27** | Accessibility summary only: accessible interfaces are for everyone; no keyboard-specific rule was retrievable |

### G.14.2 What the shell has

**A real focus vocabulary.** `State::Focused` exists with its own fill
(`ui.rs` 248–258) and every interactive component calls `focus_ring`
when it is focused (`button`, `icon_button`, `list_item`, `text_field`, `slider`).
The ring is the same shape and colour everywhere, which is the right decision:
keyboard focus is one thing a user learns once.

**A real shortcut layer.** `shortcuts.rs` is 1,108 lines and binds the launcher,
tabs, workspace switching, Settings search, the lock screen and the power menu.
There is a shortcut table drawn in Settings (`views.rs` 3230). This is not a
shell that ignored the keyboard.

**Gaps that survived Parts 14–17, and are re-stated here because they are
accessibility findings and not component findings:**

| Surface | Gap | Evidence |
|---|---|---|
| Menus and context menus | `Key::Menu` and `F10` are decoded and bound to nothing; Shift+F10 resolves to nothing; Escape does not close a context menu | `Part 14` §F.12/F.13 |
| Tooltips | hover is the only trigger, so a keyboard user cannot learn what an icon means | `Part 16` §F.14 |
| Notification centre | Escape closes it; Down moves nothing | `Part 16` §F.17 |
| Breadcrumbs | no crumb cursor; each crumb is a pointer hotspot. The repair in §0.4 made the root clickable, not focusable | `Part 17` §F.09 |
| Dialogs | Enter does not accept a Files dialog (it navigates the folder behind it); desktop dialogs answer no key at all | `Part 15` §F.15 |
| Tab order | two Tab stops per Files row; one per sidebar row; the centre has none | `Part 12`, `Part 13`, `Part 16` |

The common cause is architectural and was named in `Part 10`: the shell has
**hotspots**, not **focusable controls**. A hotspot is a rectangle that answers a
pointer; focus is a property of a control that can be moved with Tab and
activated with Space or Enter. The shell can draw a focus ring but has no
focusable object to attach it to.

### G.14.3 Findings

| # | Finding | Where |
|---|---|---|
| **G14.01** | **The focus vocabulary exists and is consistent** — one 2 px accent ring at 90 %, drawn by every component that accepts a `State` | `ui.rs` 258–266 |
| **G14.02** | **There is no focusable-control model.** Interaction is a list of rectangle hotspots with actions; nothing has a tab index, a focus order or an activation key contract | `shell.rs` `Hot`; `views.rs` hotspots |
| **G14.03** | **Six surface families have no keyboard route**: menus, context menus, tooltips, the notification centre, breadcrumbs and the non-Files dialogs. Each is a pointer-only island | Parts 14–17 |
| **G14.04** | **Where a keyboard model exists it is good**: the launcher, tab strip, Settings search, lock screen and power menu are fully operable, and the shortcut table is user-visible | `shortcuts.rs`; `views.rs` 3230 |
| **G14.05** | **Tab order is per-surface and inconsistent**: two stops per file row, one per sidebar row, none in the notification centre. There is no global order to learn | Parts 12, 13, 16 |
| **G14.06** | **A pointer click can focus a control that Tab cannot reach**, because the same function merges hover and focus (`Part 10`). The ring therefore proves keyboard reach only where the code path happens to be shared | `Part 10` §F.01 |

Status for `G.14`: **PARTIAL** — Linux is measured; the Windows and macOS
keyboard-requirement pages were not re-read in this part.

---

## G.15 — Screen readers and accessibility APIs

### G.15.1 What the platforms require

| Source | Rule |
|---|---|
| **GNOME 51** | "All interface elements should have descriptive, accessible names. These provide the text that is read aloud by screen readers." And the test is concrete: "Can you turn the display off and still use the app?" |
| **macOS 27** | Accessibility summary only: accessible interfaces "empower everyone"; no API rule was retrievable |
| **Windows 11 26H2** | Not retrieved in this part; no UI Automation rule is quoted |

GNOME's test is the one that decides the item. With the display off, the
interface is whatever the accessibility tree says it is.

### G.15.2 What the shell has

**No accessibility layer at all.**

- `litric-shell/Cargo.toml` lists twelve path dependencies and `png`. There is no
  `accesskit`, no `atspi`, no `zbus` bridge, no platform UI Automation crate.
- `rg -i 'accesskit|atspi'` over `src/litric-shell` and `src/litric-design`
  returns nothing.
- There is no accessible-name, role, state or relationship store. Labels are
  string literals drawn through `font::draw`; they exist as pixels, not as
  objects.
- There is no focus, selection or value-changed event to publish, because there
  is no tree to publish it from.

The shell is a software-rendered desktop: it draws into a canvas and (since
`893592e`) presents over KMS. Nothing in that pipeline carries semantics. This
is not a missing label on a button; it is a missing **tree**.

What does exist is the raw material:

- A translation dictionary with the shell's strings (`i18n.rs`, 367 lines), so
  names could be sourced rather than invented.
- State that never rests on colour alone (the design system's rule, visible in
  `list_item`: selected is a fill **and** a marker; error is a border **and** a
  glyph; pressed insets). That is the visual half of what an accessibility tree
  would expose.
- `ThemeKind::requires_aaa` and a high-contrast theme that genuinely passes
  (`G.10`), which serves low-vision users even with no screen reader.

None of that reaches assistive technology.

### G.15.3 Findings

| # | Finding | Where |
|---|---|---|
| **G15.01** | **No accessibility API is linked.** Zero AccessKit/AT-SPI/UI Automation dependencies in the shell | `litric-shell/Cargo.toml`; census |
| **G15.02** | **No accessible name, role or state store exists.** The UI is drawn text and rectangles; a screen reader has nothing to enumerate | `views.rs`; `ui.rs` |
| **G15.03** | **No focus/selection/value events.** Even a future tree would have nothing to announce until the hotspot model becomes a control model (`G.14`) | `shell.rs`; `G14.02` |
| **G15.04** | The **raw material is better than average**: a translation dictionary, non-colour-only states and a real high-contrast theme. The gap is the semantic layer, not the visual one | `i18n.rs`; `ui.rs`; `G.10` |
| **G15.05** | This is a **release-level** gap, not a polish item. GNOME's "turn the display off and still use the app" cannot be attempted at all | GNOME *Accessibility* |

Status for `G.15`: **PARTIAL** — measured as an absence; the Windows and macOS
API requirements were not retrieved, so the item cannot be closed by this part.

---

## 1. What the three say together

| | `G.10` Contrast | `G.14` Keyboard | `G.15` Screen readers |
|---|---|---|---|
| Standard that decides it | WCAG 1.4.3 / 1.4.11 / 2.4.13 | GNOME's four checks; 2.4.13 | GNOME accessible names; "display off" test |
| The shell's state | text AA everywhere, HC AAA everywhere, **border 1.27–1.44** | focus ring correct, **no focusable model**, six pointer-only families | **no API, no tree, no names** |
| What it would take | a border token at 3:1, or a second boundary | a control model with tab order and activation keys | an accessibility tree and a bridge |
| Cost | one token per theme | architectural | architectural, second |

The pattern is the same one Parts 10–18 kept finding, stated for the last time in
this chapter: the shell is excellent at **what a person sees** and has not built
**the layer that tells anything else what is there**. Its text contrast is better
than most shipping desktops; its high-contrast theme is genuinely AAA; its focus
ring meets a AAA criterion. And a screen reader cannot find the window, and a
1 px border at 1.27:1 is the only thing separating a text field from the page.

The border is the cheapest fix in this part and the most visible: the token is
one value per theme, and the components already use it. The accessibility tree is
the most expensive, and it cannot be faked by adding labels to a canvas — the
control model has to come first.

## 2. What this part does not establish

| Claim | Why |
|---|---|
| that text over glass meets contrast | the table models opaque surfaces only. Glass composites over the wallpaper; the crate enforces a tint floor with a different test, not a ratio against an arbitrary backdrop |
| that disabled text is readable | `State::Disabled` dims to 40 % alpha; WCAG exempts inactive components from 1.4.3, and the effective ratio over a surface was not computed |
| that every interactive component draws a focus ring | the components that take `State` do. The hotspot rectangles that are not components (crumbs, banner areas, table headers) do not |
| that the keyboard gaps are the complete list | they are the ones Parts 14–17 measured. A full tab-order audit was not run |
| that no accessibility work exists elsewhere in the product | the census covered `litric-shell` and `litric-design`, not the browser, Defender or the daemons |
| that Windows or macOS require what this part quotes | neither platform's accessibility API page was retrieved; only GNOME's rule is quoted in full |
| that the repair in §0.4 is the right product decision | it fixes three recorded findings; whether a disabled Back button should instead remain a target that explains itself is a design choice |
| anything about the other 15 `G` items | untouched. Only `G.10`, `G.14` and `G.15` were measured here |

## 3. Sources

| # | Source | Depth | Used for |
|---|---|---|---|
| S3834 | W3C WAI — *Understanding SC 1.4.3 Contrast (Minimum)* | full | 4.5:1 text, 3:1 large text, exemptions, no rounding of thresholds |
| S3835 | W3C WAI — *Understanding SC 1.4.11 Non-text Contrast* | snippet | the 3:1 rule for UI component boundaries; the page returned only an example figure, so no further 1.4.11 detail is quoted |
| S3836 | W3C WAI — *Understanding SC 2.4.13 Focus Appearance* | full | 2 CSS px perimeter, 3:1 change between focused and unfocused |
| S3837 | GNOME HIG — *Accessibility* | full | accessible names for all elements, and the four tests: high contrast, large text, keyboard navigation, screen reader with the display off |
| S3838 | Apple HIG — *Accessibility* | summary | accessible interfaces are for everyone; body not readable |
| — | `docs/DESIGN-SYSTEM-v3/01-FOUNDATION-TOKENS.md`, `02-COMPONENTS-STATES.md` | repository | the state matrix the components implement |
| — | `src/litric-design/src/{lib,color}.rs` | repository | the palette literals, the WCAG formula, `requires_aaa`, the accent table |
| — | `src/litric-shell/src/{ui,views,apps,shortcuts,shell}.rs` | repository | the focus ring, the hotspot census, the shortcut layer, the repair |
| — | `src/litric-shell/Cargo.toml` | repository | the dependency census for `G.15` |

Pages were fetched on 2026-10-28. Three returned their body in full; one returned
a fragment only; one returned a summary only. The contrast numbers are not quoted
from any page — they were computed from this repository's own palette.

## 4. Status changes and the repair

**No `G` status changed.** `G.10`, `G.14` and `G.15` were already `PARTIAL` on
macOS evidence; this part adds the Linux measurement and amends their notes in the
checklist. They stay `PARTIAL` because each still has a platform page unread
(Windows for all three, macOS for `G.14`/`G.15`).

| Item | Status | Note now says |
|---|---|---|
| `G.10` Contrast | `PARTIAL` | 36/36 text pairs AA, 20/20 HC pairs AAA (min 10.00), border 1.27–1.44 fails 1.4.11, focus ring 6.59/4.47 |
| `G.14` Keyboard | `PARTIAL` | focus ring correct; no focusable-control model; six pointer-only surface families; tab order inconsistent |
| `G.15` Screen readers | `PARTIAL` | zero accessibility-API dependencies and no name/role/state tree; the shell is invisible to assistive technology |

| Repair | File | Test |
|---|---|---|
| disabled Back/Forward/Up, no hotspot when dead | `apps.rs`, `views.rs` | `history_buttons_know_when_they_have_nowhere_to_go`, Files journey |
| breadcrumb overflow drops the head, keeps the current folder | `views.rs` | `breadcrumbs_drop_their_head_before_their_tail` |
| root crumb clickable from any depth | `apps.rs`, `views.rs` | `the_root_crumb_goes_home_from_any_depth` |

With this part, Chapter 41 has three of eighteen items measured on Linux. The
remaining fifteen are token families (`G.01`–`G.09`, `G.11`–`G.13`) and input
modalities (`G.16`–`G.18`) that still carry macOS evidence only; they are the next
pass of batch G, and they are cheaper than this one because most of their values
are already in `litric-design`.
