# LitricOS UI/UX Research — Part 20 of N

Chapter 41 (G) — Token families and input: the system is complete, the shell
uses part of it, and the border was below the standard it claimed

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P20 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-28 (Asia/Bangkok) |
| Scope of this part | Chapter 41 (`G`): `G.01`–`G.09`, `G.11`–`G.13` and `G.16`–`G.18` — the fifteen token and input items left after `Part 19` measured `G.10`, `G.14` and `G.15`. The whole chapter is now measured on Linux |
| Batch | B3/B6 continuation (`Part1` §2.2), second Linux pass: token values and input modalities |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (15 notes amended; statuses stay `PARTIAL`). No new sources: this part measures the repository against standards already registered in `Part 19`. It also carries a **token repair** (border) and a **gate repair** (the border test asserted 1.3 while its comment claimed WCAG's 3.0) |
| Predecessors | `Part 7` (macOS token evidence), `Part 19` (`G.10` contrast, `G.14` keyboard, `G.15` screen readers) |
| Resume point | none left in `G` on Linux. Windows token tables remain unread for every item; `J` (final register and audit) is the next batch that can close them |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and used here without redefinition.

---

## 0. Scope, method, and the repair that came out of it

### 0.1 Fifteen items, one question each

| Group | Items | The question |
|---|---|---|
| Colour | `G.01`, `G.09` | is there a semantic palette, and does it have a real light/dark pair? |
| Type and geometry | `G.02`, `G.03`, `G.04` | is there a scale, and is it actually one scale? |
| Depth | `G.05`, `G.06` | are elevation and materials tokens or decoration? |
| Drawing | `G.07`, `G.08` | icons and motion: a vocabulary or a set of one-offs? |
| Display | `G.11`, `G.12` | can the interface scale, and can it span two screens? |
| Language | `G.13` | does it survive a translation and a right-to-left locale? |
| Input and feedback | `G.16`, `G.17`, `G.18` | pointer, reduced motion, and what the product says out loud |

### 0.2 The counts (MEASURED from the repository)

| Token family | Value |
|---|---|
| Semantic palette tokens | **11** per theme × 4 themes = **44** values |
| Type steps | **8**: 12, 13, 15, 17, 20, 24, 30, 38 px; line heights 16–44 |
| Space steps | **8**: 2, 4, 8, 12, 16, 24, 32, 48 px (a 4 px grid) |
| Radii | **6**: 6, 10, 14, 20, 28, `Full` (9999 sentinel) |
| Elevation levels | **5**: blur 0–44 px, offset 0–20 px, shadow alpha 0–55 % |
| Motion tokens | **4** springs: settle 120 / 200 / 300 / 450 ms, plus a reduced token |
| Materials | **7** (`Solid`, `Elevated`, `Flat`, `Glass`, `Popover`, `Dialog`, `Scrim`) |
| Glass presets | **3**: blur 32 px, saturation 115–120 %, tint 82 / 85 / 100 % |
| Icons | **32** variants in one module |
| Display outputs in the compositor | **1** (`wl_output` global) |
| RTL / direction handling found | **0** |

### 0.3 The repair: the border token, and the gate that let it through

`Part 19` measured `border` at **1.44:1** on Dark and **1.27:1** on Light against
the base — below the 3:1 that WCAG 1.4.11 asks of the only boundary some
controls have. The repository already had a test named
`borders_are_visible_against_their_own_surface`, whose comment says "WCAG asks
3.0 for non-text UI elements" — and whose assertion was `r >= 1.3`. The gate and
its name disagreed, and the weaker number was the one running.

Both were fixed together:

| Theme | Was | Now | New ratio vs base | vs elevated |
|---|---|---|---|---|
| **Dark** | `#2A2E37` | `#646A75` | **3.60** | **3.29** |
| **Light** | `#D9DDE3` | `#7C828C` | **3.61** | **3.87** |
| HC Dark | `#FFFFFF` | unchanged | 21.00 | 19.80 |
| HC Light | `#000000` | unchanged | 21.00 | 21.00 |

The test now asserts **3.0**, so the token cannot drift back below the standard
without a failing build. The values were chosen to clear 3:1 against **both**
`bg_base` and `bg_elevated`, because a card border sits on the base while an
input border sits on the card.

This is the part's most useful result and it came from measuring a token that
every previous part had used without reading.

---

## G.01 / G.09 — Colour tokens, light and dark

### G.01.1 What exists

`Palette` is semantic, not literal: `bg_base`, `bg_elevated`, `glass_tint`,
`text_primary`, `text_secondary`, `accent`, `on_accent`, `border`, `danger`,
`success`, `warning` — **11 tokens**, resolved per theme. Components ask
`t.text()`, `t.text_dim()`, `t.tone(Tone::Bad)`; no component carries a hex value.

`ThemeKind::all()` returns four themes: **Dark**, **Light**, **High-Contrast
Dark**, **High-Contrast Light**. The accent is a six-value slot layered on top
(`Accent::all`: Default/Sky, Violet, Teal, Amber, Rose), and high-contrast themes
return themselves unchanged — a decorative preference cannot break the AAA
guarantee (`with_accent`, `lib.rs` 278–310).

### G.01.2 Findings

| # | Finding | Where |
|---|---|---|
| **G01.01** | The semantic layer is **complete and enforced by construction**: a theme cannot omit a token, and components cannot bypass one | `Palette`, `lib.rs` 155–180 |
| **G01.02** | **Four themes with a real pair**, not a "dark mode" filter: Light is a separately authored palette (`F6F7F9` base, `0E1116` text), not an inversion | `Theme::of`, `lib.rs` 200–270 |
| **G09.01** | The accent is a **slot, not a hex value**, so the same choice stays readable in both themes; the 12 accent pairs measured in `Part 19` all pass AA | `with_accent`; `Part 19` `G.10` |
| **G09.02** | High-contrast themes **refuse the accent** and say so in the settings UI, rather than offering a control that does nothing | `Accent` docs, `lib.rs` 75–92 |
| **G01.03** | The only colour token that failed its standard was **border**, now repaired (§0.3) | `lib.rs` 206, 223 |

Status: both `PARTIAL` — Linux measured; Windows palette tables unread.

---

## G.02 / G.03 / G.04 — Type, spacing and radii

### G.02.1 Measured

| Step | Size | Line height | Large text (≥ 24 px) |
|---|---|---|---|
| Caption | 12 | 16 | no |
| Footnote | 13 | 18 | no |
| Body | 15 | 22 | no |
| Headline | 17 | 24 | no |
| Title3 | 20 | 26 | no |
| Title2 | 24 | 30 | **yes** |
| Title1 | 30 | 36 | yes |
| Display | 38 | 44 | yes |

`is_large_text()` uses **24 px**, not "18.66 bold", because the system does not
assume bold — a conservative and correct choice for the 3:1 relaxation.

### G.02.2 Findings

| # | Finding | Where |
|---|---|---|
| **G02.01** | **One type scale, eight steps, with line height defined per step** — not a font-size list with a global leading | `TypeStep`, `lib.rs` 500–580 |
| **G02.02** | **One family and one weight axis.** `Style` carries size, colour and bold; there are no separate weight or tracking tokens, so a richer hierarchy would need new tokens rather than new literals | `font::Style` |
| **G03.01** | **An 8-step 4 px grid**, 2–48. It is used for padding and gaps throughout `ui.rs` | `Space`, `lib.rs` 582–635 |
| **G03.02** | **Component heights sit off the grid**: Files rows 24 px (on grid), sidebar rows 30 px, nav buttons 22 px high, breadcrumb bar 30 px, command row 26 px. The system has a grid; the components do not all join it | `views.rs` 1007, 1043, 1158; `ui.rs` sidebar |
| **G04.01** | **Six radii with assigned roles** (6 inputs, 10 buttons/chips, 14 cards/popovers, 20 sheets, 28 windows, `Full`), and the sentinel is clamped at draw time | `Radius`, `lib.rs` 636–684; `Tokens::radius` |
| **G04.02** | Radius is applied through `t.radius()`, so a scale change moves every corner; no component hard-codes a corner | `ui.rs` components |

Status: all three `PARTIAL` — Linux measured; Windows type/spacing tables unread.

---

## G.05 / G.06 — Elevation and materials

### G.05.1 Measured

| Level | Blur | Offset Y | Shadow alpha | Intended for |
|---|---|---|---|---|
| Z0 | 0 | 0 | 0 % | flush with the wallpaper |
| Z1 | 8 | 2 | 18 % | cards resting on the desktop |
| Z2 | 18 | 6 | 28 % | windows, menus |
| Z3 | 30 | 12 | 40 % | focused window, popovers |
| Z4 | 44 | 20 | 55 % | spotlight, sheets, banners |

A test (`higher_elevation_never_casts_less_shadow`) pins monotonicity across all
three dimensions, so the scale cannot invert.

### G.06.1 Measured

`Material` has **7** values. `Glass` is the only material with numbers, and they
are unusually well argued:

| Preset | Blur | Saturation | Tint alpha | Border alpha | Highlight |
|---|---|---|---|---|---|
| Dark | 32 | 120 % | 82 % | 14 % | 10 % |
| Light | 32 | 115 % | 85 % | 10 % | 45 % |
| High contrast | 0 | 100 % | 100 % | 100 % | 0 % |

The static floors are 82 %/85 % because the dim text failed AA below them; the
**adaptive** path samples the wallpaper and solves for the lowest alpha in
**18–92 %** that keeps the text at its target (`adaptive_alpha_pct`). High-contrast
glass is opaque on purpose: translucency and guaranteed AAA are not compatible.

### G.05.2 Findings

| # | Finding | Where |
|---|---|---|
| **G05.01** | **Elevation is a five-level spec, not five shadows copied into components**, and monotonicity is tested | `Elevation`, `lib.rs` 685–760 |
| **G05.02** | **Most surfaces are flat.** The shell's material vocabulary (`Flat`, `Elevated`, `Solid`) carries most depth, and `Z0`–`Z4` are defined and tested but not the main way depth is expressed. That is a design position, not a defect, but it means the elevation scale is under-exercised | `ui.rs` `Material`; `views.rs` surfaces |
| **G06.01** | **Glass is the best-specified material in the product**: three presets, a measured legibility floor, an adaptive solve, and an intentional opaque high-contrast case | `Glass`, `lib.rs` 320–470 |
| **G06.02** | **"Beauty vs contrast" is resolved by measurement, not by a setting**: the tint adapts to the wallpaper so the default is already correct | `adapted_to`, `lib.rs` 438–470 |
| **G06.03** | The material set is **closed and named** (7 values), so a new surface must choose a material rather than invent a blur | `ui.rs` `Material` |

Status: both `PARTIAL` — Linux measured; macOS material values come from Part 7
and were not re-read here.

---

## G.07 / G.08 — Icons and motion

### G.07.1 Measured

**32 icon variants** in one module, drawn by a single `icons::draw` into the
canvas (folder, browser, terminal, settings, activity, trash, environment, grid,
search, bell, network, bluetooth, audio/muted, battery, power, chevrons, close,
minimise, maximise, check, warning, info, lock, user, window, cpu, memory, …).

### G.08.1 Measured

| Token | Stiffness | Damping | Settle budget |
|---|---|---|---|
| INSTANT | 520 | 34 | 120 ms |
| FAST | 380 | 30 | 200 ms |
| NORMAL | 260 | 26 | 300 ms |
| SLOW | 180 | 24 | 450 ms |

Springs, not fixed easing curves, because a spring is interruptible (ADR-0009).
`anim.rs` gives every animated surface a token; `Motion::reduced` snaps instead
of animating.

### G.07.2 Findings

| # | Finding | Where |
|---|---|---|
| **G07.01** | **One icon vocabulary of 32 glyphs**, drawn from vectors rather than loaded as assets, so they scale with the canvas and inherit colour | `icons.rs` 26–120 |
| **G07.02** | **Icons have no labels.** `Part 16` found 9 glyph-only controls and 3 window buttons with no tooltip and no accessible name; the icon set is complete and the naming layer is absent | `icons.rs`; `Part 16` `F.14` |
| **G08.01** | **Four motion tokens plus a reduced token**, each with a settle budget the QA gate can check | `Motion`, `lib.rs` 784–840 |
| **G08.02** | **Motion is actually used**: `anim.rs` assigns INSTANT/FAST/NORMAL/SLOW per surface and honours `reduce_motion` by snapping — a token system that is not shelfware | `anim.rs` 100–150 |
| **G08.03** | **Interruptibility is a stated requirement** (ADR-0009), which is why springs were chosen; a fixed curve would have been cheaper and would fail a drag mid-animation | `lib.rs` 770–783 |

Status: both `PARTIAL` — Linux measured; Windows/macOS motion tables unread here.

---

## G.11 / G.12 — Display scaling and multiple monitors

### G.11.1 Measured

- `Tokens` carries `scale: f32`, and every component asks `t.px()` / `t.sp()` for
  a size. `Shell::set_display(width, height, scale)` replaces all three at once
  (`shell.rs` 4364).
- The layout audit runs every app at **two resolutions** and all built-in themes
  (`e2e::every_app_layout_is_audited_at_both_resolutions_and_all_built_in_themes`).
- There is **no fractional per-output scale**: one scale applies to the whole
  shell.

### G.12.1 Measured

- The compositor registers a **single** `wl_output` global (`dispatch.rs` 16, 109).
- The shell keeps one display size and one scale; there is no arrangement view,
  no per-monitor DPI and no hot-plug path.

### G.11.2 Findings

| # | Finding | Where |
|---|---|---|
| **G11.01** | **Scaling is one number, applied everywhere**, because components never hold raw pixels; that is the right architecture for a UI that must scale at all | `Tokens::px`, `shell.rs` 4364 |
| **G11.02** | **The audit covers two resolutions**, which catches overflow but not fractional or per-monitor scale | `e2e` layout audit |
| **G12.01** | **One output, one display.** Multi-monitor is not "partially implemented"; it has no model at all | `dispatch.rs` 109; `shell.rs` display state |
| **G12.02** | A second screen would need three things the code does not have: per-output tokens, an arrangement model, and a rule for which output owns the bar | — |

Status: both `PARTIAL` — Linux measured; platform scaling rules not re-read.

---

## G.13 — Localization and RTL

### G.13.1 Measured

- A translation dictionary of **367 lines** (`i18n.rs`), including Thai for the
  Settings pages, states and common strings; a `litric-locale` crate supplies
  locale, hour cycle, first day and measurement system.
- `rg -i '\brtl\b|right.to.left|dir='` over `litric-shell` and `litric-locale`
  returns **nothing**.
- No mirrored icon set, no direction-aware layout, no bidi handling in the text
  layer.

### G.13.2 Findings

| # | Finding | Where |
|---|---|---|
| **G13.01** | **Translation exists and is real**: strings resolve through the dictionary, and locale data (hour cycle, first day) is used by the clock and the date column | `i18n.rs`; `views.rs` 2541 |
| **G13.02** | **RTL is absent.** A right-to-left locale would get a left-to-right layout with translated words: no mirrored chevrons, no flipped sidebar, no bidi ordering | census above |
| **G13.03** | The **locale layer is the right foundation** for RTL (it already carries direction-capable data), but nothing consumes a direction bit | `litric-locale` |

Status: `PARTIAL` — Linux measured; platform RTL rules not re-read.

---

## G.16 / G.17 / G.18 — Pointer, reduced motion, feedback

### G.16.1 Measured

- Input is **pointer + wheel + keyboard**: `InputEvent` carries moves, buttons,
  wheel, keys. No touch event type is handled by the shell.
- Hit targets measured in earlier parts: **18 × 18** checkbox (Part 12),
  **20 × 22** nav buttons (Part 17), **30 px** sidebar rows (Part 13), **24 px**
  table rows (Part 18).

### G.17.1 Measured

- `Preferences::reduce_motion` is a real setting, and `anim.rs` snaps when it is
  set (`Motion::reduced`).
- There is **no "reduce transparency"** preference; glass adapts its own tint
  instead (§G.06).
- The Accessibility page exists and is the one place these live.

### G.18.1 Measured

- **No audio output path** in the shell: no sound playback, no system beep, no
  notification sound. `Audio` is an icon and a facts row, not an output.
- Feedback is visual: tone-coded badges, banners, inline errors, state markers,
  and a bell with a count.

### G.16.2 Findings

| # | Finding | Where |
|---|---|---|
| **G16.01** | **Pointer and keyboard only.** A touchscreen produces no touch events, so a tap is a pointer click and a long-press is not a gesture | `litric-input` events; `shell.rs` |
| **G16.02** | **Several targets are below every platform minimum**: 18 px checkbox, 20 × 22 nav buttons, 24 px rows. With no touch support this is a mouse-accuracy issue rather than a touch one, but the numbers fail the 40–44 px targets Windows, Apple and GNOME publish | `Part 12`, `Part 17`, `Part 18` |
| **G17.01** | **Reduced motion is honoured end to end** — preference, token, animation. This is the best-implemented item in the fifteen | `settings.rs` 104; `anim.rs` 26 |
| **G17.02** | **No reduce-transparency setting**; the material solves legibility itself, which is defensible but removes user control | `Glass::adapted_to` |
| **G18.01** | **Feedback is silent.** Nothing in the shell produces sound, so a critical notification delivered while the user is looking elsewhere has only a banner that may expire | `views.rs`; `G.18` |
| **G18.02** | **Visual feedback is not colour-alone** (badges carry counts, states carry markers), so the silent channel is at least redundant in shape as well as hue | `ui.rs` state matrix |

Status: all three `PARTIAL` — Linux measured; platform input/feedback rules not
re-read.

---

## 1. What the fifteen say together

| Family | Verdict |
|---|---|
| Colour, light/dark | **Complete**: 11 semantic tokens × 4 themes; the accent is a slot; border repaired this part |
| Type, space, radii | **Complete as a scale, uneven in use**: 8 type steps, 8 space steps, 6 radii; several component heights sit off the 4 px grid |
| Elevation, materials | **Best-specified part of the system**: 5 tested levels, 7 materials, 3 glass presets with an adaptive legibility solve |
| Icons, motion | **A vocabulary, used**: 32 icons, 4 springs, reduced motion honoured |
| Display | **Single-output by construction**: one scale, one `wl_output`, no multi-monitor model |
| Localization | **Translated, not directional**: a real dictionary, zero RTL |
| Input, feedback | **Mouse-and-keyboard, visual-only**: no touch, no sound, reduced motion yes, reduce transparency no |

The pattern of the whole chapter, in one line: **the token system is better than
the interface that consumes it.** Scales are defined and tested; components
partly ignore them. Glass solves a hard contrast problem; icons have no names.
Motion is interruptible and reduced-motion-aware; the product cannot make a
sound. None of these is a missing token. They are places where a complete system
was not carried all the way into the surface.

The border repair is the exception that proves the rule: one token, one test, and
the standard finally matches the comment that claimed it.

## 2. What this part does not establish

| Claim | Why |
|---|---|
| that the token values match Windows' or macOS' scales | only macOS evidence from `Part 7` exists, and it was not re-read. No Windows token table was fetched |
| that every surface uses the space grid or the elevation scale | the token sets were measured; component-by-component compliance was sampled, not audited exhaustively |
| that no touch event exists anywhere in the system | the census covered `litric-shell`; the compositor and `litric-input` were read only for their event names |
| that the shell cannot produce audio at all | no output path was found in the shell; the Defender or a daemon was not audited for sound |
| that RTL is required by the product's roadmap | only that it is absent. Whether Thai-only is a deliberate scope decision is not recorded in this corpus |
| that the new border values are the visually best choice | they are the values that clear 3:1 against base and elevated with margin; other compliant values exist |
| that raising the border test from 1.3 to 3.0 changes nothing else | the design suite and the shell suite were run; no visual review was performed, because the shell has no display in this environment |
| anything about Windows availability of the fifteen items | untouched. Every item stays `PARTIAL` for that reason |

## 3. Sources

No new URLs were added for this part. It measures the repository against the
standards registered in `Part 19` and the macOS evidence registered in `Part 7`.

| # | Source | Depth | Used for |
|---|---|---|---|
| S3834 | W3C WAI — *SC 1.4.3 Contrast (Minimum)* | full | the 4.5 / 3.0 text thresholds |
| S3835 | W3C WAI — *SC 1.4.11 Non-text Contrast* | snippet | the 3:1 that the border token was failing |
| S3836 | W3C WAI — *SC 2.4.13 Focus Appearance* | full | the focus ring's 2 px / 3:1 requirement |
| S3837 | GNOME HIG — *Accessibility* | full | high contrast, large text, keyboard and screen-reader checks |
| S3838 | Apple HIG — *Accessibility* | summary | accessibility as a product value; no numbers |
| — | `src/litric-design/src/lib.rs` | repository | every token count and value in §0.2, `G.01`–`G.09` |
| — | `src/litric-shell/src/{ui,icons,anim,shell,settings,views}.rs` | repository | material set, icon count, motion wiring, scale, preferences |
| — | `src/litric-compositor/src/dispatch.rs` | repository | one `wl_output` global |

## 4. Status changes and the repair

**No status changed.** All fifteen were `PARTIAL` on macOS evidence; this part
adds the Linux measurement and amends their notes. They stay `PARTIAL` because
**no Windows token table was read for any of them** — the same reason `Part 19`'s
three items stayed open.

| Repair | File | Effect |
|---|---|---|
| Dark border `#2A2E37` → `#646A75`, Light `#D9DDE3` → `#7C828C` | `src/litric-design/src/lib.rs` | 3.60/3.29 and 3.61/3.87 against base/elevated |
| `borders_are_visible_against_their_own_surface` threshold `1.3` → `3.0` | same | the gate now enforces what its own comment claims |

With this part, **all 18 `G` items have Linux evidence** and Chapter 41 is
measured end to end. The remaining gap is uniform and external: the Windows token
and input tables, which belong to the audit batch `J` rather than to another
research pass over this repository.
