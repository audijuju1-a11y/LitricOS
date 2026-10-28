# LitricOS UI/UX Research — Part 17 of N

Chapter 40 (F) — Pickers, navigation views and trees: the shell can go places,
but it cannot choose a value or show what is inside what

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P17 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-28 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.07` pickers, `F.09` navigation views and `F.20` trees. The other 24 `F` items are untouched and keep their status |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", on the three items `Part 16` named as its resume point: the largest surfaces still `PENDING` |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (3 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (14 sources appended, S3814–S3827). No probe: the evidence here is a source census, and §0.2 carries the commands and the counts |
| Predecessors | Parts 10–16; `Part 13` (`F.08`, `F.10`, `F.11`), `Part 15` (`F.15`) and `Part 16` (`F.14`) |
| Resume point | `F.21` grids/tables, `F.22` cards, `F.24` empty states and `F.25` loading states — the last four `PENDING` |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these three

### 0.1 Three questions, and the shell answers one

| Item | The question it answers | The shell's answer | Where |
|---|---|---|---|
| **`F.07`** Pickers | *What value is this, and how do I change it?* | nothing to change it with; the values are printed or cycled | `settings.rs` lines 140–187; `views.rs` line 2541 |
| **`F.09`** Navigation views | *Where am I, and how do I go somewhere else?* | a places pane, a breadcrumb bar and a page list — three idioms, no container | `views.rs` lines 961–1039, 2476–2505 |
| **`F.20`** Trees | *What is inside what?* | a flat table and a path string | `apps.rs` lines 149–158, 296–320 |

These are the three surfaces where a desktop lets a person **choose one thing out
of many**: a value out of a range, a place out of a location list, a parent out of
a hierarchy. The shell has built the middle one, partly, three times. It has not
built the first or the third at all.

That makes this part different from Parts 10–16. Those found components that
exist and misbehave. This one finds two component families that do not exist, in
a product whose job is to show files — a hierarchy — and to change settings — a
set of known values. The absence is the finding, and the census below is how it
is measured.

### 0.2 The census (MEASURED, static)

| Measure | Command | Value | What the value means |
|---|---|---|---|
| `picker` occurrences in the shell | `rg -i -c 'picker' src/litric-shell/src` | **8** | every one names an inline chooser, not a picker control: `i18n.rs` 59 and 353 (a language list), `desktop.rs` 1370, 1445, 1479 (the wallpaper list), `views.rs` 2198 (three search-engine chips), `e2e.rs` 857, 863 (tests for those chips) |
| `calendar` occurrences | `rg -i -c 'calendar' src/litric-shell/src` | **2** | both comments: the top bar's date (`views.rs` 537) and the lock screen's month (`lock.rs` 27). No calendar widget |
| `tree` occurrences | `rg -i -c 'tree' src/litric-shell/src` | **42** | `BTreeSet`/`BTreeMap`, `copy_tree`, "folder tree" comments, and the task manager's "End process tree". **0** tree controls |
| Fields on a file row | `apps.rs` 149–158 | **4** | name, kind, size, mtime. No parent, no children, no depth, no expanded flag |
| List surfaces in the product | Files table, places pane, Settings pages, Trash, Task manager, browser history | **6** | all flat |
| Navigation containers | — | **0** | no `NavigationView`-equivalent, no split-view controller, no adaptive mode |
| Sidebar width policies | `ui.rs` 1013–1076 | **1** | quarter of the body clamped to 180–280 px, 30 px rows, 2 px gaps — one component, since `cf6b523` |

Two of these numbers are the part's spine. **Eight occurrences of "picker" and
zero pickers** is the cleanest absence in the corpus so far: the word is only
ever used for a horizontal row of options that is already on screen. **Four
fields on a file row** is the other: a file manager that cannot represent a
child cannot draw a tree, whatever the view code does.

### 0.3 Two absences of different kinds

A missing component is only a finding if a platform says it should be there, so
the three are not equal:

- **Files without a tree is defensible.** Microsoft's tree-view page says,
  verbatim, "For most drill-in scenarios, a regular list view is appropriate."
  The shell drills in with a breadcrumb and a flat table. That is a supported
  design, and Part 13 already measured the breadcrumb as one of the better
  surfaces in the product.
- **Settings without a picker is not defensible.** The clock format is already
  known (`HourCycle::H12`/`H24`, printed at `views.rs` 2541), the accent is a
  seven-value choice the design system asks for by name
  (`docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` line 77: "accent picker 7 สี
  (จาก palette เดิมเท่านั้น)"), and the wallpaper is an enumerated set. Three
  known value spaces, three surfaces that either print the value or cycle it.
- **The tree is defensible for navigation and not for moving.** Drill-in gets a
  person to a folder. It does not let them drop a file into a folder they cannot
  see, which is the one operation a file manager exists for. Windows' own
  picker documentation is explicit that a picker shows "a tree of locations that
  the user can browse to".

---

## F.07 — Pickers (date, time, colour, file)

### F.07.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **Windows 11 26H2 / WinUI 3** | There are **four** date and time controls and the choice between them is the design decision: **CalendarView** for a single date or a range with the calendar always visible, **CalendarDatePicker** for a single date "where contextual information like the day of the week … is important", **DatePicker** for "a single known date when contextual info isn't important", **TimePicker** for "a single time value" | Microsoft Learn, *Calendar, date, and time controls* (T1, full page) |
| Windows | A **CalendarDatePicker** "displays placeholder text if a date has not been set". The *unset* state is part of the control, not an edge case | same page |
| Windows | A picker "overlays other UI; it doesn't push other UI out of the way" — the surface is transient and the invoking context stays visible | same page |
| Windows | A colour picker browses a spectrum or takes **RGB, HSV or hexadecimal** input. "For the most accurate color picking experience, use the square spectrum and ensure it is at least 256x256px, or include the text input fields" | Microsoft Learn, *Color picker* (T1, full page) |
| Windows | In a flyout, a tap on the spectrum "should not commit the colour selection": commit either through explicit OK/Cancel or on dismissal, and the choice is stated, not implied | same page |
| Windows | A file picker shows "the current location", "the item or items that the user picked" and **"a tree of locations that the user can browse to"** — file-system folders plus apps that implement the picker contract. It also carries filters and a thumbnail/list view mode | Microsoft Learn, *Open files and folders with a picker* (T1, full page; the page states that WinUI 3 apps should use `Microsoft.Windows.Storage.Pickers` instead of the UWP API it documents) |
| **GNOME 51 / GTK 4** | The file chooser is a toolkit object, `GtkFileDialog` (since 4.10): asynchronous, with `open`, `save`, `select_folder`, multiple selection, filters, an accept label, an initial file/folder/name, and a modal flag | GTK 4 API reference, *Gtk.FileDialog* (T1, reference depth) |
| GNOME | **No HIG rule is quoted for date, time or colour pickers**: none was located in the pattern pages read for this part. The platform statement here is the toolkit's, not a design guideline's | `developer.gnome.org/hig/patterns/nav/*`; GTK docs |
| **macOS 27** | "A picker displays one or more scrollable lists of distinct values that people can choose from" | Apple HIG, *Pickers* (T1, **summary depth** — the page body did not load) |
| macOS | "A colour well displays a colour picker when people tap or click it. This colour picker can be the system-provided one or a custom interface that you design" | Apple HIG, *Color wells* (T1, **summary depth**) |

The two Windows pages that decide this item are unusually concrete, and the shell
fails the first test of each: there is no control to be in an unset state, no
spectrum or text field, and no location tree.

### F.07.2 What the shell has

**Nothing that picks.** The census in §0.2 is the whole inventory: eight uses of
the word, zero controls. The four picker families each have a different
near-miss:

| Family | What exists instead | Where |
|---|---|---|
| **Date and time** | formatted output only: the top bar's clock and date, the lock screen's clock and month, a file's mtime column, a Trash entry's deletion date | `views.rs` lines 123–139, 442, 536–607; `lock.rs` lines 896–897, 1259–1270 |
| **Colour** | the accent is an `Accent` **enum** selected by a row that **cycles** on click ("Change accent" beside the current name); the theme is a four-value `ThemeKind`; the wallpaper is an enumerated `Wallpaper` | `settings.rs` lines 95–101; `views.rs` lines 2781–2801 |
| **File** | no open or save dialog at all. The five Files dialogs are new folder, new text file, rename, confirm and error (`Part 15`); none asks for a path outside the current folder | `views.rs` line 1378; `Part 15` §0.1 |
| **Any other value** | the search engine is three chips (`views.rs` line 2198); the task manager's time column is three chips (`taskmgr.rs` lines 1239–1241); Settings' Region page prints Clock, First day, Units and Time zone as **label/value pairs** | `views.rs` lines 2541–2572 |

The Region page is the clearest single piece of evidence. It prints
`Clock: 12-hour` or `Clock: 24-hour` from the locale and gives the reader no way
to change it; the page's own comment below says the rows are "Read-only by
design: every row here is a measurement". A measurement is the right thing for
"Processor" and "Memory". It is the wrong thing for a preference the same
locale layer can already express both ways.

The design system is explicit that this was meant to be a picker: the accent is
specified as "accent picker 7 สี (จาก palette เดิมเท่านั้น)", and the component
table specifies a segmented control for "theme picker, view mode"
(`docs/DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md` line 54). The shell shipped the
cycling row instead.

### F.07.3 Findings

| # | Finding | Where |
|---|---|---|
| **F07.01** | The shell has **no date, time, colour or file picker**. The word appears eight times and every use means "a row of options already on screen" | census, §0.2 |
| **F07.02** | The three settings that are literally picker problems are not pickers: the **accent cycles** through an enum, the **wallpaper** is a fixed list, and the **clock format is printed** from the locale with no control | `views.rs` 2781–2801, 2541–2542 |
| **F07.03** | The design system asks for a **7-colour accent picker** and a segmented control for theme and view mode. The shell has a cycling row; the spec and the code disagree about whether the value space is visible | `03-SHELL-SYSTEM.md` line 77; `02-COMPONENTS-STATES.md` line 54; `views.rs` 2789 |
| **F07.04** | **No control can be unset**, because no value is ever chosen. Windows builds the placeholder into `CalendarDatePicker`; the shell has no control that needs one | `views.rs` 2541–2572 |
| **F07.05** | There is **no file open/save dialog**, which is consistent with a shell whose apps never ask for a path — and means the desktop cannot hand a file to an application, or take one back | `Part 15` §0.1; `apps.rs` 1643–1651 |
| **F07.06** | The **data a date picker needs already exists**: hour cycle, first day of week, measurement system and time zone are all parsed, printed and translated (`i18n.rs` 209–210). The missing piece is the control, not the locale work | `views.rs` 2538–2558; `i18n.rs` 209–210 |
| **F07.07** | What is right and should be reused: the search-engine chips are a **real, labelled, tested choice** (three engines, the selection persists and drives `search_url`), and the accent row at least names the current value in words | `views.rs` 2198; `e2e.rs` 857–863; `Part 12` §F.05 |

Status for `F.07`: **PARTIAL** — three platforms researched (two at page depth,
one summary-only; no GNOME design rule located), the shell side measured by
census. It becomes `COMPLETE` when a picker exists or the product records a
decision not to have one, and when the macOS page bodies can be read.

---

## F.09 — Navigation views

### F.09.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **Windows 11 26H2 / WinUI 3** | `NavigationView` "provides top-level navigation" and "adapts to a variety of screen sizes and supports both top and left navigation styles" | Microsoft Learn, *NavigationView* (T1, full page) |
| Windows | **Top** for "5 or fewer top-level navigation categories that are equally important"; **Left** for "5–10 equally important" categories that should be prominent; **LeftCompact** shows icons until opened and overlays the content; **LeftMinimal** shows only the menu button until opened; **Auto** (the default) moves LeftMinimal → LeftCompact → Left as the window widens | same page |
| Windows | The pane holds items, separators, headers, an optional search box and an optional settings entry; the **menu button** toggles the pane; a **back button** is placed top-left but "does not automatically handle backwards navigation and add content to the back stack" — the app must wire it | same page |
| Windows | The header is a **fixed 52 px** and holds the page title; it is "docked to the top of the page and acts as a scroll clipping point". Content margins are **12 px** in Minimal mode and **24 px** otherwise | same page |
| **GNOME 51** | A view switcher holds **three to five** views; "If you have more views, a sidebar might be a more appropriate choice". Labels use header capitalisation and nouns, and should be a similar length. A switcher "should switch to the bottom window edge if the window becomes too narrow" | GNOME HIG, *View Switchers* (T1, full page) |
| GNOME | A sidebar is a vertical panel of locations, for "a larger number of views than can be accommodated in a standard view switcher", for dynamic locations, and where "frequent switching back and forth" is common. Order by usefulness; "header bar controls which affect the sidebar list should be placed above the list" | GNOME HIG, *Sidebars* (T1, full page) |
| GNOME | Tabs hold "a mutable set of content items"; labels short with the useful part first; a context menu per tab with **Close last** and Move to New Window, Duplicate, Pin, Mute, Reload; the standard shortcuts are Ctrl+T, Ctrl+W, Ctrl+PageUp/PageDown, Ctrl+Home/End, Ctrl+Tab and Alt+1–9 | GNOME HIG, *Tabs* (T1, full page) |
| **macOS 27** | "A sidebar appears on the leading side of a view and lets people navigate between areas of your app or top-level collections of content, like folders and playlists" | Apple HIG, *Sidebars* (T1, **summary depth**) |

Two rules matter most for what follows. Windows makes the **back button a
control the app has to wire**, and GNOME makes the **number of destinations
decide the container** (three to five → switcher, more → sidebar). Settings alone has
13 destinations, no switcher, and no back button outside the browser.

### F.09.2 What the shell has

**Three navigation idioms, and no shared model.**

**Files** (`views.rs` 961–1039): a places pane, then a 30 px bar that holds
three 20 × 22 glyph buttons `‹ › ↑` at a 24 px pitch, then the breadcrumb. Each
crumb is `Footnote` text plus 8 px of padding, 22 px tall, separated by a drawn
`/` with 8 px on each side, and each one is a hotspot that navigates to
`breadcrumb_path(i)` (`apps.rs` 453–476). The root is **not** drawn: the
segment list drops the empty root, and the empty case paints the literal
`/ (root)` **with no hotspot** (`views.rs` 1035–1038). The bar has **no
overflow handling**: the loop advances `cx` by the measured width of each crumb
and never compares it with `bar.right()`, so a deep path runs past the bar.

**Settings** (`views.rs` 2476–2505): the same sidebar component, 13 pages
(`settings.rs` 173–187), a `Title3` page title beside it, and no history at
all. Switching pages replaces the pane; there is no back affordance, although
Windows' `NavigationView` supplies one and GNOME's split-view pattern expects
one.

**Browser** (`views.rs` 1812–1872): the one navigation surface with proper
states. Back and Forward are real buttons whose `disabled` flag is computed from
`can_go_back()`/`can_go_forward()`, and the history model trims the forward stack
on a new navigation (`apps.rs` 2425 has a test for exactly that). Part 13 covered
the tab strip (`F.08`); it is not re-measured here.

**One sidebar, finally.** `ui::sidebar` (`ui.rs` 1006–1076) is a quarter of the
body clamped to **180–280 px**, with a floor of a third and a 320 px content
minimum, **30 px** rows at a **2 px** gap, a fixed 26 px heading, and a shared
scroll model. Both Files and Settings call it. That closes the delta `F.10`
recorded and `BUG-0032`/`cf6b523` repaired; it is the strongest navigation work
in the shell and this part does not re-open it.

**What is absent is the container.** There is no display mode (no
LeftCompact/LeftMinimal, no hamburger, no bottom-edge switcher), no shared
"current destination + history" model between the three idioms, no page header
in Files, and no disabled state on the three Files buttons: they are drawn with
`State::Normal` unconditionally (`views.rs` 1011–1015), so Back looks pressable
when the history is empty.

### F.09.3 Findings

| # | Finding | Where |
|---|---|---|
| **F09.01** | **No navigation container**: three idioms (places + breadcrumb, page list, tab strip + history) with three state models and no shared notion of "current place and how I got here" | `views.rs` 961, 2476; `apps.rs` 296 |
| **F09.02** | **No adaptive mode.** The sidebar keeps 180–280 px at every width. Windows would collapse to LeftMinimal or LeftCompact; GNOME would move a view switcher to the bottom edge. The shell has one mode and no breakpoint | `ui.rs` 1013–1076; platform tables above |
| **F09.03** | **Files' Back, Forward and Up are always enabled.** They are drawn `State::Normal` regardless of history, so two of the three are dead controls that look live. The browser, in the same shell, computes its disabled state correctly | `views.rs` 1011–1015 vs 1871–1872 |
| **F09.04** | **The breadcrumb has no overflow policy.** Crumb widths are summed with no clamp against the bar, so a path deeper than the window runs under the details pane or off the edge. Contrast the top bar, which deliberately **drops the date first** when it runs out of room (`views.rs` 536) | `views.rs` 1023–1033 |
| **F09.05** | **The root is not a control.** `breadcrumbs()` drops the empty root, and the fallback paints `/ (root)` as text with no hotspot, so the one destination a deep path most needs is the one that cannot be clicked | `views.rs` 1035–1038; `apps.rs` 453–476 |
| **F09.06** | **No per-crumb keyboard state.** Each crumb is a pointer hotspot with an index and nothing else; there is no crumb cursor, no arrow-key walk, no focus ring. The bar is a picture of a path that happens to be clickable | `views.rs` 1023–1033 |
| **F09.07** | **Settings has no back.** Thirteen destinations, one level deep, no history and no back button — the destination list is the only way out of a page | `views.rs` 2476–2505; `settings.rs` 173–187 |
| **F09.08** | The three nav buttons are **glyphs with no label** (`‹ › ↑`), which is the icon-only case `F.14` found has no tooltip and no accessible name | `views.rs` 1011–1013; `Part 16` F14.03 |
| **F09.09** | What is right and should be kept: **one sidebar component** with published bounds and a real scroll model (`F.10`'s delta closed), and the browser's **disabled-aware history** with a test behind it | `ui.rs` 1006–1076; `views.rs` 1871–1872; `apps.rs` 2425 |

Status for `F.09`: **PARTIAL** — two platforms at page depth, one summary-only;
the shell measured by code. It becomes `COMPLETE` when a navigation container
exists (or a decision records the three-idiom design as intended) and when the
macOS sidebar body can be read.

---

## F.20 — Trees

### F.20.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **Windows 11 26H2 / WinUI 3** | A tree view is "a hierarchical list with expanding and collapsing nodes that contain nested items"; it "uses a combination of indentation and icons to represent the nested relationship". Collapsed nodes use a **right** chevron, expanded nodes a **down** chevron | Microsoft Learn, *Tree view* (T1, full page) |
| Windows | Supports **N-level nesting**, single or multiple selection, data binding, arbitrary item content and drag and drop between trees | same page |
| Windows | `HasChildren` and `HasUnrealizedChildren` decide whether the chevron is shown at all, and `HasUnrealizedChildren` is how a node is filled **when it expands** — lazy loading is part of the contract | same page |
| Windows | When to use it, verbatim: "Use a TreeView when items have nested list items, and if it is important to illustrate the hierarchical relationship of items to their peers and nodes." And: "For most drill-in scenarios, a regular list view is appropriate." | same page |
| **GNOME 51 / GTK 4** | `GtkTreeExpander` "provides an expander for a tree-like list", placed as the bottommost child of a `GtkListView` and driven by a `GtkTreeListModel`; it supplies "the common UI elements, gestures and keybindings" | GTK 4 API reference, *Gtk.TreeExpander* (T1, reference depth) |
| GTK | Keyboard: **`+` or `*` expands**, **`-` or `/` collapses**, **Shift+Left/Right** (locale direction) expand or collapse, **Ctrl+Space toggles**. Actions `listitem.expand`, `listitem.collapse`, `listitem.toggle-expand` exist for custom UI | same page |
| GTK | `indent-for-depth`, `indent-for-icon` and `hide-expander` are properties, so indentation and the expander's presence are decided per row. Since GTK 4.12 the accessible role is `BUTTON` with an `EXPANDED` state | same page |
| **macOS 27** | "An outline view looks like a table view, but includes disclosure triangles for exposing nested levels of data" | Apple HIG, *Lists and tables* (T1, **summary depth** — the page body did not load) |

All three platforms put the same three things in the control: a **chevron** that
says whether a node is open, an **indent** that says how deep it is, and a
**keyboard model** that can open and close it without a pointer. The shell has
the chevron glyph (`icons.rs` 62–64) and none of the rest.

### F.20.2 What the shell has

**No tree, and no state to build one from.**

The file model is four fields (`apps.rs` 149–158): `name`, `kind`, `size`,
`mtime`. There is no parent, no children, no depth and no expanded flag; a row is
a leaf by construction. The listing is `rows: Vec<FileRow>` (`apps.rs` 310) for
one directory, and moving between directories is `navigate()` plus a `path:
String` (`apps.rs` 479), with `back`/`forward` vectors beside it. The only
hierarchy the product ever draws is that string, rendered as breadcrumbs.

The other five list surfaces are flat in the same way: the places pane is
`Vec<(String, String)>` (`apps.rs` 300), Settings is 13 flat pages
(`settings.rs` 173–187), Trash is a flat row list, the browser's history is a
flat stack, and the task manager is a table.

The task manager is the interesting near-miss. It has a **`Parent` (PPID)
column** (`taskmgr.rs` 50, 1608) and an action called **"End process tree"**
(`taskmgr.rs` 1033, 1568) that ends everything under a process. So the product
already *reasons* about a tree, tells the user it is ending one, and draws the
processes as a flat list with a number in a column. The word "tree" names an
operation, not a view.

`Icon::ChevronDown` and `Icon::ChevronRight` exist (`icons.rs` 62–64, 411–412)
and are used for menu affordances and scroll bumpers (`views.rs` 566, 1737),
never for a row.

### F.20.3 Findings

| # | Finding | Where |
|---|---|---|
| **F20.01** | **There is no tree control anywhere in the shell**, and no model that could feed one: a `FileRow` has no parent, child, depth or expanded state | `apps.rs` 149–158, 310 |
| **F20.02** | **The filesystem is shown flat.** Folder-in-folder is represented only by the path string and the breadcrumb, so a person can go *into* a folder but cannot see, or drop something into, a folder they are not already inside | `apps.rs` 479; `views.rs` 1021–1033 |
| **F20.03** | **No expander state exists** — no expanded set, no lazy-load flag, no depth indent. The three things every platform puts in a tree (chevron, indent, keyboard) have zero implementation, although the chevron glyphs exist | `apps.rs` 296–320; `icons.rs` 62–64 |
| **F20.04** | **The task manager ends process trees without drawing one.** It shows PPID and offers "End process tree" while the table has no hierarchy, so the user is asked to trust a relationship the interface never shows | `taskmgr.rs` 50, 1033, 1568, 1608 |
| **F20.05** | **The sidebar is flat too**: places are a `(path, label)` vector with no groups, no nesting and no reordering, though GNOME's sidebar guidance allows multi-line rows ordered by usefulness and Windows' pane supports headers and separators | `apps.rs` 300; GNOME *Sidebars* |
| **F20.06** | What is right and should be reused: the `list_item` component already carries selection, hover, focus, disabled, error and loading states, and `Part 12` measured it. A tree would be a composition of existing rows plus an indent and a chevron — not a new visual language | `ui.rs` (list item); `Part 12` §F.04 |

Status for `F.20`: **PARTIAL** — two platforms at page depth and one
summary-only; the shell measured by census and code. It becomes `COMPLETE` only
with a decision: either a tree for move/copy targets, or a recorded decision that
drill-in is the whole navigation model and the missing target picker is accepted.

---

## 1. What the three say together

| Capability | Pickers | Navigation views | Trees |
|---|---|---|---|
| Windows expects | 4 date/time controls, a spectrum or RGB/HSV/hex field, a location tree in the file picker | an adaptive `NavigationView` with 5 display modes and an app-wired back button | chevrons, N levels, lazy children, multi-select |
| GNOME expects | the toolkit's `GtkFileDialog`; no HIG picker rule located | 3–5 view switcher, else sidebar; tabs with a full shortcut set | `GtkTreeExpander` with `+`/`-`/Shift+arrows/Ctrl+Space |
| The shell has | none | three idioms, one shared sidebar, no container | none |
| What it costs | a preference cannot be **stated**, only cycled | a user cannot get **back** out of Settings; a deep path **overflows**; dead nav buttons look live | a file cannot be **put into** a folder that is not on screen |

The pattern across the three is not "the shell is missing components". It is that
the shell has built the **forward** direction of every interaction and not the
**sideways or upward** one. It goes into a folder but cannot show the tree above
it. It goes into a Settings page but cannot go back. It cycles a preference but
cannot show the values it is cycling through. Every one of those is the same
missing idea: a **chooser that shows the space of choices**.

The cheapest of the three to fix is not the tree. It is the picker for three
settings whose value spaces already exist and are already enumerated: a 7-colour
accent, a fixed wallpaper set, and a two-value clock. The most expensive is the
tree, because it needs a model change before it needs a view change.

## 2. What this part does not establish

| Claim | Why |
|---|---|
| that the macOS picker, sidebar, outline-view and colour-well rules are as Apple states them | all four pages returned only their title, description and summary sentence; the body did not load. They are cited at summary depth and no macOS number is quoted |
| that the GNOME HIG has no picker guidance | only that no such rule was located in the pages read for this part. The HIG was not crawled end to end, and the lists page returned 404 |
| that WinUI 3 file pickers behave as the UWP page describes | the WinUI 3 picker page 404'd; the UWP quickstart is cited for the picker's **information model**, and it states WinUI 3 should use `Microsoft.Windows.Storage.Pickers` instead. No Windows App SDK API behaviour is claimed |
| that 8 is the total count of the word "picker" in the product | the census was `src/litric-shell/src` only. Other crates (settings app, browser, Defender) were not counted |
| that a deep breadcrumb *visually* overflows | read from the loop, which has no clamp. No frame was captured, so overlap with the details pane is described from geometry, not from an image |
| that Alt+Left or any key reaches Files Back | not measured. The part describes the three buttons and their handlers, not the shortcut table |
| that a tree is the right answer for Files | a product decision, not a research finding. Windows explicitly endorses drill-in for navigation; this part only records that drill-in cannot serve as a move target |
| that the task manager could show a tree cheaply | its model was read for `Parent` and `EndTree` only; the rest of the process table was not measured |
| anything about Parts 10–16's items | untouched. `F.07`, `F.09`, `F.20` were the three this part changed |

## 3. Sources

| # | Source | Depth | Used for |
|---|---|---|---|
| S3814 | Microsoft Learn — *NavigationView* (windows/apps/develop/ui/controls/navigationview) | full | display modes and when each applies, pane anatomy, 52 px header, back button that does not navigate, content margins |
| S3815 | Microsoft Learn — *Tree view* | full | chevrons, N-level nesting, selection, `HasChildren`/`HasUnrealizedChildren`, drill-in exception |
| S3816 | Microsoft Learn — *Calendar, date, and time controls* | full | the four date/time controls and the choice between them, placeholder state, overlay behaviour |
| S3817 | Microsoft Learn — *Color picker* | full | spectrum and RGB/HSV/hex, 256×256 guidance, commit rules in a flyout |
| S3818 | Microsoft Learn — *Open files and folders with a picker* (UWP) | full | location, picked items, **tree of locations**, filters, view mode; the WinUI 3 substitution note |
| S3819 | Apple HIG — *Pickers* | summary | "one or more scrollable lists of distinct values"; body not readable |
| S3820 | Apple HIG — *Lists and tables* | summary | outline view as a table with disclosure triangles; body not readable |
| S3821 | Apple HIG — *Color wells* | summary | a colour well opens a colour picker; body not readable |
| S3822 | Apple HIG — *Sidebars* | summary | leading-side navigation between areas or top-level collections; body not readable |
| S3823 | GNOME HIG — *View Switchers* | full | 3–5 views, sidebar beyond, labels, bottom-edge behaviour |
| S3824 | GNOME HIG — *Sidebars* | full | when a sidebar beats a switcher, ordering, controls above the list |
| S3825 | GNOME HIG — *Tabs* | full | tab context menu contents, standard shortcuts |
| S3826 | GTK 4 — *Gtk.FileDialog* | reference | async file chooser, open/save/folder, filters, labels, modal |
| S3827 | GTK 4 — *Gtk.TreeExpander* | reference | expander in a list, `GtkTreeListModel`, indent properties, keyboard and a11y |
| — | `docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` line 77 | repository | the specified 7-colour accent picker |
| — | `docs/DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md` line 54 | repository | the segmented control specified for theme and view mode |
| — | `src/litric-shell/src/{apps,views,ui,settings,taskmgr,icons}.rs` | repository | every shell-side number, line and absence in §0.2, F.07.2, F.09.2 and F.20.2 |

Pages were fetched on 2026-10-28. Ten returned their body in full: five Microsoft
pages, three GNOME HIG pages and two GTK references. The four Apple pages
returned a title, a description and one summary sentence; they are recorded at
summary depth, not page depth. The WinUI 3 file-picker page returned **404** and is not cited; the
UWP page stands in for the information model with that limitation stated in §2.

## 4. Status changes

| Item | Was | Now | Note |
|---|---|---|---|
| `F.07` Pickers (date/time/color/file) | `PENDING` | `PARTIAL` | no picker of any kind (8 word occurrences, all inline choosers); clock format printed read-only, accent cycled, wallpaper enumerated; no unset state and no open/save dialog (F07.01–F07.07) |
| `F.09` Navigation views | `PENDING` | `PARTIAL` | three idioms and no container; no adaptive mode; Files' nav buttons never disabled; breadcrumb has no overflow, no root hotspot and no keyboard cursor; Settings has no back (F09.01–F09.09) |
| `F.20` Trees | `PENDING` | `PARTIAL` | no tree control and no model for one: `FileRow` is four fields; all six list surfaces are flat; the task manager ends process trees without drawing them (F20.01–F20.06) |

No decisions are recorded in this part. Whether the shell should grow a tree for
move/copy targets, and whether Settings should grow real pickers for the three
preferences it already knows how to express, are product decisions with a cost,
and belong in a decision record rather than in a research note.

With these three, Chapter 40 has **23 of 27** items researched and **4** left
`PENDING`: `F.21` grids/tables, `F.22` cards, `F.24` empty states and `F.25`
loading states. None of the four has a component in the shell; `F.21` and `F.22`
have surfaces (the files table, the start page's cards), while `F.24` and `F.25`
are states rather than components and will need a different method from the one
this part used.
