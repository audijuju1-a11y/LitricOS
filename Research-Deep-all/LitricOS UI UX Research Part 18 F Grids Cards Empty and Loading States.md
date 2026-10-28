# LitricOS UI/UX Research — Part 18 of N

Chapter 40 (F) — Grids, cards, empty states and loading states: the last four
items, and the one distinction the shell gets right

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P18 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-28 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.21` grids/tables, `F.22` cards, `F.24` empty states and `F.25` loading states. These are the last four `PENDING` items; the other 23 keep their status |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", closed on the two container families and the two states that every other component needs |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (4 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (6 sources appended, S3828–S3833). No probe: the evidence is the source census in §0.2 |
| Predecessors | Parts 10–17; `Part 11` (`F.18` progress), `Part 13` (`F.08` tabs, `F.10` sidebars) and `Part 17` (`F.07`, `F.09`, `F.20`) |
| Resume point | Chapter 40 is complete at 27 of 27 researched. The next batch is `G` (design tokens and accessibility) and `H` (workflows), per `Part1` §2.2 |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and what is left

### 0.1 Two containers and two states

| Item | What it is | Why it is last |
|---|---|---|
| **`F.21`** Grids/tables | the multi-column container | it is the shape the files table and the task manager already have, in two different implementations |
| **`F.22`** Cards | a bordered surface grouping one object's content | it exists only as two local inventions (Mission Control, the start page) and no shared component |
| **`F.24`** Empty states | what a surface says when it has nothing | not a component: a message policy applied to eight surfaces |
| **`F.25`** Loading states | what a surface says while it is waiting | not a component: the `State::Loading` enum exists, and almost nothing uses it |

The first two are containers, the last two are sentences the containers say when
they have nothing or are not ready. That is why they close the chapter: unlike
`F.01`–`F.20`, they cannot be measured as a control with a geometry. They are
measured as a count of call sites and a list of the strings those sites draw.

### 0.2 The census (MEASURED, static)

| Measure | Command | Value | What it means |
|---|---|---|---|
| Files table header / row | `views.rs` 1157–1158 | **22 px / 24 px** | the product's densest table: 24 px rows, a 22 px header, one hairline under it and **no vertical rules** |
| Files rows drawn per frame | `views.rs` 1220 | **at most 24** (`take(24)`) | the Files model has no table-scroll field (`apps.rs` 296–320), so a 25th entry has no drawn row |
| Files columns | `files_columns`, `views.rs` 918–950 | **3** (name, size, date) | kept only if the heading fits; the name column is never dropped; freed width goes to name |
| Task-manager columns | `taskmgr.rs` 147–160, 725–750 | **10** in the model, 6 shown by default | the only table with configurable widths and a sort model |
| Card components | `rg -i '\bcard'` | **0** `ui::card`; tokens name cards (`litric-design` 158, 641, 688) but no component | Mission Control cards and start-page tiles are local inventions |
| Empty-state strings | `rg` over `views.rs` | **at least 8** | Files, details, history, bookmarks, downloads, notifications, Trash and the Settings "Unavailable" family |
| `State::Loading` call sites | `rg 'State::Loading'` | **2** (`views.rs` 4190, Defender scan; `lock.rs` 1934, unlock) | the state is defined and tested; almost nothing else uses it |
| Spinner / indeterminate control | `rg -i 'spinner|indeterminate'` | **0** | no spinning or indeterminate widget anywhere |
| Progress bars | `ui::progress` | **1 call site** (Defender) | `Part 11` F.18 already found the scan bar is the only progress in the product |

The two numbers that decide the chapter: **24 rows drawn with no scroll model**
and **one `State::Loading` call site** for a product that copies files, compresses
archives, scans for threats and streams a terminal.

### 0.3 The distinction the shell gets right

Before the findings, the positive one, because it is rare in this corpus. The
shell separates **empty** from **unavailable** and tests the difference:

- A machine with no processes says "Unavailable — litric-sysd did not answer",
  not "0 processes" (`views.rs` 2409; `shell.rs` 7541 has a test named for it).
- Unknown facts are reported as unavailable rather than zero (`shell.rs` 9034;
  `monitor.rs` 795).
- Bluetooth has **four** answers, not two: no radio, blocked, no adapter,
  unavailable (`views.rs` 2968–2978).
- Settings prints "Unavailable - the kernel reported no mounted volumes" rather
  than an empty list (`views.rs` 2614).

Most products collapse all of these into an empty list. This one does not, and
`Part 1` §2.1's rule that a claim needs a measurement is visible in the UI. The
finding of this part is that the **empty** half of that distinction is
under-designed, not that the distinction is wrong.

---

## F.21 — Grids and tables

### F.21.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **Windows 11 26H2 / WinUI 3** | `ListView` "displays data stacked vertically in a single column" and works for text-first collections read top to bottom. **"If you need to display collections in multiple columns or in a table-like format, you should not use ListView. Instead, consider using a DataGrid control."** | Microsoft Learn, *List view and grid view* (T1, full page) |
| Windows | `GridView` presents items in rows and columns, filling across then down, and is for image-first collections "or whose items can be read from side-to-side or are not sorted in a specific order" | same page |
| Windows | Both derive from `ListViewBase` and share interaction, selection and scrolling; `ItemsRepeater` is the building block when neither fits | same page |
| Windows | A **DataGrid is not a built-in WinUI control** — the page points to it as the table option rather than documenting one. This part therefore quotes no Windows DataGrid rule, and no Windows column-width or sorting rule | same page (absence noted) |
| **GNOME 51 / GTK 4** | `GtkColumnView` "presents a large dynamic list of items using multiple columns with headers"; it supports **sorting by clicking headers**, **interactive resizing and reordering** of columns, rubberband selection, and optional row/column separators | GTK 4 API reference, *Gtk.ColumnView* (T1, reference depth) |
| GTK | The accessible role is `TREE_GRID`; headers are `COLUMN_HEADER`, rows `ROW`, cells `GRID_CELL`. A "data table" style class exists (`.data-table`) alongside `.rich-list` and `.navigation-sidebar` | same page |
| **macOS 27** | "Lists and tables present data in one or more columns of rows" | Apple HIG, *Lists and tables* (T1, **summary depth**) |

The Windows sentence is the one to hold against the shell: a table-like collection
should not be a `ListView`, and the shell's Files table is a hand-drawn `ListView`
with three columns and no scroll model.

### F.21.2 What the shell has

**Two tables, two implementations, neither shared.**

**Files** (`views.rs` 1155–1245): a **22 px** header, **24 px** rows, a single
hairline under the header and **no vertical rules** — the comment says vertical
rules "read as a form, not data". Each header cell is a hotspot that sorts by its
key and draws `↑`/`↓` when active. Columns come from `files_columns`
(`views.rs` 918–950): three fractions, each kept only if its heading fits at
`table_w × frac ≥ text + 12 px`; a dropped column's width is given to the name
column, which is never dropped. Cells are elided against their own column, and
the name column is inset **28 px** (for the icon) against **6 px** for the rest.

Three limits fall out of the same function:

1. The draw loop is `vis.iter().enumerate().take(24)` — **24 rows per frame**.
2. `FilesApp` has `places_scroll` but **no table-scroll field** (`apps.rs`
   296–320), so there is nothing for a 25th row to be offset by.
3. No column is resizable or reorderable; the only column interaction is sorting.

**Task manager** (`taskmgr.rs`): ten columns in the model (Name, PID, Parent,
State, User, CPU, CPU time, Memory, Memory share, Threads, Command line), six
shown by default (`Name, Pid, Cpu, Memory, Threads, State` — `taskmgr.rs` 725–730),
natural widths from **64 px** (PID/Parent) to **320 px** (command line)
(`taskmgr.rs` 147–160), width steps per column, alignment rules (numbers right),
a `first_click_desc` map so the first click on a numeric column sorts descending,
and short headers. This is the most complete table in the product, and it shares
nothing with the Files table.

**Defender** draws a verdict table (`views.rs` 4139) of the same hand-rolled kind.

### F.21.3 Findings

| # | Finding | Where |
|---|---|---|
| **F21.01** | **Two table implementations, no shared component.** Files and the task manager sort, size and draw their columns with different models; a fix to one is not a fix to the other | `views.rs` 918–1245; `taskmgr.rs` 147–160 |
| **F21.02** | **The Files table draws at most 24 rows** and the model has no table-scroll offset, so a folder with more entries cannot show the rest through this path | `views.rs` 1220; `apps.rs` 296–320 |
| **F21.03** | **No column can be resized or reordered.** GTK makes both a property of the column view; the shell makes the width a fraction recomputed from the window, so a user cannot make the name column wider | `views.rs` 918–950 |
| **F21.04** | **Columns disappear silently when the window narrows.** The fit test drops a column rather than offering an overflow, so the size or date column vanishes with no menu, no indicator and no way to bring it back except resizing the window | `views.rs` 925–950 |
| **F21.05** | **No grid.** `GridView`'s job (rows and columns of items, image-first) has no surface: the start page's quick links are a fixed row of three, not a collection view | `views.rs` 2150–2170 |
| **F21.06** | **Sorting is Files-only**, and its state is a `(SortKey, bool)` tuple; the task manager has its own sort model and the Defender table has none | `apps.rs` 296–320; `taskmgr.rs` 661–790 |
| **F21.07** | What is right and should be promoted into the shared component: **elide against the column**, not the table; **never drop the name column**; **sort arrow in the header**; numeric columns right-aligned; the task manager's `first_click_desc` rule | `views.rs` 925–1200; `taskmgr.rs` 725–790 |

Status for `F.21`: **PARTIAL** — Windows and GNOME at page depth, Apple
summary-only, and no Windows DataGrid rule exists to cite. The shell side is
measured.

---

## F.22 — Cards

### F.22.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **GNOME 51** | A **boxed list** is "a common type of list that can contain both controls and information", for "relatively small static lists"; "for large or dynamic lists, see list views". Multiple lists can act as sections, each with a heading | GNOME HIG, *Boxed Lists* (T1, full page) |
| GNOME | A row may hold one control, at most two; "when there is a control, clicking the list background should trigger the control"; controls are focusable but the row is not, "allowing direct switching of keyboard focus between the controls and prior/subsequent rows" | same page |
| GNOME | "Lists should have a minimum and maximum width, in order to support adaptive scaling." Predefined rows include switch, action, combo, entry, spin, property and **expander** rows | same page |
| **Windows 11 26H2** | No card rule was retrieved. The list/grid page covers collection containers, not a card pattern; the design-controls lists page returned **404** | Microsoft Learn (two attempts; recorded as not retrieved) |
| **macOS 27** | No card rule was retrieved; the HIG page list read for this part has lists, tables, sidebars and pickers, not cards | Apple HIG (not retrieved) |

Only GNOME gives a card-like container a written contract. That matters, because
the shell's two card surfaces were invented locally and answer GNOME's questions
in two different ways.

### F.22.2 What the shell has

**Two card surfaces, no component.**

**Mission Control** draws each open window as a card (`shell.rs` 306, 3272):
"a window card", ordered by workspace and then by a stable key. The ordering has
a comment explaining why stability matters — cards that reorder themselves as the
user moves between them are a target that moves under the pointer.
`views.rs` 4814 (`quick_cards`) builds the list the switcher draws. The card is
a real object with a click target that brings its window forward
(`shell.rs` 3286).

**The browser start page** draws quick links as tiles: **110 × 48** at a **12 px**
gap, radius `Md`, a 1 px border, and a hover fill (`views.rs` 2150–2170). Three
links, fixed, centred.

**What is missing:** `ui::card` does not exist. The design tokens *name* cards —
`litric-design` line 158 ("Raised surfaces: cards, sheets, sidebars"), line 641
("14 — cards, popovers") and line 688 ("1 — cards resting on the desktop") — but
nothing in `ui.rs` turns those tokens into a component. There is no shared
padding, no elevation rule, no focus/selected/disabled state for a card, and no
card row type.

The start tile has hover only: no focus ring, no pressed state, no disabled
state, and no keyboard route of its own, although it is the only tile on the
page.

### F.22.3 Findings

| # | Finding | Where |
|---|---|---|
| **F22.01** | **No card component.** Two surfaces draw card-like objects with their own geometry; the tokens that name cards are unused by `ui.rs` | `shell.rs` 306, 3272; `views.rs` 2150–2170, 4814; `litric-design` 158, 641, 688 |
| **F22.02** | **The start-page tile is a bordered rectangle with hover only** — 110 × 48, 1 px border, `Md` radius, no focus, pressed, selected or disabled state, and no keyboard model | `views.rs` 2150–2170 |
| **F22.03** | **Mission Control's card ordering is deliberately stable**, and that is the one card decision with a written reason; the visual card itself has no shared geometry with any other surface | `shell.rs` 3272–3286 |
| **F22.04** | **GNOME's boxed-list contract is met by neither surface**: no minimum/maximum width, no "click the background triggers the control" rule, no row-with-one-control model, no expander row | GNOME *Boxed Lists*; `views.rs` 2150–2170 |
| **F22.05** | What should be promoted: Mission Control's **stable ordering** and the start tile's **bordered, hoverable rectangle** are the two halves of a card; they live in different files and never meet | `shell.rs` 3272; `views.rs` 2150 |

Status for `F.22`: **PARTIAL** — one platform at page depth (GNOME), Windows and
macOS not retrieved. The item cannot reach `COMPLETE` until at least one of those
two documents a card, or a decision records the card as a product invention.

---

## F.24 — Empty states

### F.24.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **GNOME 51 / libadwaita** | `AdwStatusPage` is "a page used for empty/error states and similar use-cases". It carries an **icon**, a **title**, a **description** and a **custom widget displayed below them** — the action slot | libadwaita API reference, *Adw.StatusPage* (T1, reference depth) |
| GNOME | With an `AdwSpinnerPaintable` as the paintable the page gains the `.spinner` style class, "for a more compact appearance"; a `.compact` class exists for sidebars and popovers | same page |
| **Windows 11 26H2** | No dedicated empty-state page was retrieved for this part | Microsoft Learn (not retrieved) |
| **macOS 27** | No dedicated empty-state page was retrieved for this part | Apple HIG (not retrieved) |

GNOME's contract is the useful yardstick: an empty state has a **title**, a
**sentence** and a **way out**. The shell's empty states are sentences.

### F.24.2 What the shell has

At least eight, all drawn as text:

| Surface | String | Where | Action offered |
|---|---|---|---|
| Files, empty folder | "Empty folder" | `views.rs` 1297 | none |
| Files, details pane | "Nothing selected" | `views.rs` 1320 | none |
| Browser history | "No pages visited yet" | `views.rs` 1937 | none |
| Browser bookmarks | "No bookmarks saved yet" | `views.rs` 1939 | none |
| Browser downloads | "Nothing has been downloaded yet" | `views.rs` 1989 | none |
| Notification centre | "Nothing waiting" | `Part 16` §F.17 | "Clear all" (disabled-looking) |
| Trash | an empty-list message chosen from the entry count | `views.rs` 4393–4400 | none |
| Settings, storage | "Unavailable - the kernel reported no mounted volumes" | `views.rs` 2614 | none |

The distinction in §0.3 is the good part: "Empty folder" (there is nothing),
"Nothing selected" (you have not chosen) and "Unavailable - ..." (the measurement
failed) are **three different sentences for three different causes**, and the
Bluetooth page goes further with four. A product that says "No items" for all
three is worse than this one.

The missing part is uniform: none of them carries an **icon**, a **second
sentence**, or an **action**. "Empty folder" does not offer "New folder"; "No
bookmarks saved yet" does not offer a way to make one; "Nothing has been
downloaded yet" does not offer the page that would start one. GNOME's
`AdwStatusPage` has slots for exactly those three things, and the shell uses
none of them.

### F.24.3 Findings

| # | Finding | Where |
|---|---|---|
| **F24.01** | **At least eight empty states, all text-only.** None has an icon, a description sentence or an action | `views.rs` 1297, 1320, 1937–1939, 1989, 4393; `Part 16` |
| **F24.02** | **Empty, unselected and unavailable are correctly distinguished** — three sentences for three causes, four on Bluetooth — and the distinction is tested. This is the part's positive finding and should not be flattened | `views.rs` 1297, 1320, 2409, 2614, 2968–2978; `shell.rs` 7541, 9034 |
| **F24.03** | **No empty state offers a way out.** The Files empty state does not offer New folder; the bookmarks state does not offer Add bookmark; the downloads state does not offer the browser. GNOME's `AdwStatusPage` has a child slot for exactly this | `views.rs` 1293–1305, 1933–1995; libadwaita *StatusPage* |
| **F24.04** | **The best empty state in the product is a notification**: `Part 16` found the centre says "Nothing waiting", which names the state in the user's terms rather than the system's. The Files state says "Empty folder", which names the object | `Part 16` §F.17; `views.rs` 1297 |
| **F24.05** | **No surface distinguishes "loading" from "empty" in its words.** The task manager says "Unavailable — litric-sysd did not answer" for a failed measurement, which is right, but a slow daemon and a dead one produce the same sentence | `views.rs` 2409; `shell.rs` 7541 |
| **F24.06** | The empty state is **not a component**: each one is a literal at its call site, so the eight states cannot share a tone, a size or an action slot; `Part 17`'s census has the same shape (`F.24` is a policy, not a widget) | `views.rs` 1293–1995 |

Status for `F.24`: **PARTIAL** — one platform at reference depth, two not
retrieved; the shell measured as a string census.

---

## F.25 — Loading states

### F.25.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **Windows 11 26H2 / WinUI 3** | Two controls, and the choice is about **blocking**: a determinate **ProgressBar** shows known progress and "should be used during an operation whose duration is known, but its progress should not block the user's interaction"; an **indeterminate ProgressBar** "does not block user interaction"; an **indeterminate ProgressRing** "blocks user interaction". Progress controls are **read-only and not interactive** | Microsoft Learn, *Guidelines for progress controls* (T1, full page) |
| Windows | Use text, or no indicator, when the operation is background and of minimal interest; "sometimes it's best to show a notice only when the operation is completed" | same page |
| Windows | The page names the operations each control is for: a bar for "Loading…, Retrieving, Working…", a ring for "Refreshing, Signing in…, Connecting…" | same page |
| **GNOME 51 / libadwaita** | An `AdwStatusPage` with an `AdwSpinnerPaintable` gains a `.spinner` class and a compact appearance, so a loading state and an empty state share one component | libadwaita *Adw.StatusPage* (T1, reference depth) |
| **macOS 27** | "Progress indicators let people know that your app isn't stalled while it loads content or performs lengthy operations" | Apple HIG, *Progress indicators* (T1, **summary depth**) |

The Windows rule the shell breaks is not "use a spinner". It is that **duration
and blocking decide the control**, and that a long operation which blocks the
user needs an indeterminate ring while one that does not needs a bar. The shell
has one bar, used once, and a button state.

### F.25.2 What the shell has

**`State::Loading` exists and is correct.** `ui.rs` 253 dims the base fill;
`ui.rs` 317 and 410 draw a motion marker. The enum's own documentation says the
control "stays visible and draws a motion marker; it never pretends to be idle"
(`ui.rs` 60–64), which is exactly the platform rule.

**It is used twice.** `rg 'State::Loading'` finds three references inside `ui.rs` and **two product call sites**: the Defender scan button, which draws "Scanning" with `State::Loading` (`views.rs` 4190), and the lock screen's unlock button (`lock.rs` 1934; `e2e.rs` 1500). Everything else in the shell never leaves its idle picture.

**No spinner exists.** `rg -i 'spinner|indeterminate'` returns nothing in
`litric-shell` or `litric-design`. `Part 11` (`F.18`) already recorded that the
Defender scan bar is the product's only progress indicator, that there is no
indeterminate form, and that `total == 0` draws an empty track during a live
scan. This part confirms that finding from the loading side and adds the count.

**What works:** `ui::progress` clamps its fraction (`ui.rs` 1325 has a test that
it never overflows its track), the boot intro is driven by a real progress value
(`views.rs` 37–40), and the Defender button does not become a dead grey box — it
keeps its label and adds motion, which is better than most implementations.

**What does not:** copying, compressing, extracting, deleting, scanning and
streaming a terminal are all long operations. The shell's answer to all of them
is either a notification, a streamed line, or nothing. There is no indeterminate
bar for "I do not know how long this will take", no ring for "you cannot use this
window until it finishes", and no skeleton or placeholder for a list that is
still loading.

### F.25.3 Findings

| # | Finding | Where |
|---|---|---|
| **F25.01** | **One loading state, two call sites.** The `State::Loading` enum is well specified and used by the Defender scan button and the lock screen; everything else shows nothing | `ui.rs` 60–64, 253, 317, 410; `views.rs` 4190; `lock.rs` 1934 |
| **F25.02** | **No spinner and no indeterminate form.** A long operation with an unknown duration has no control to show for itself | census; `Part 11` §F.18 |
| **F25.03** | **No loading-versus-blocking distinction.** Windows makes an indeterminate bar non-blocking and an indeterminate ring blocking; the shell has no ring, so it cannot say "wait" as opposed to "working" | `ui.rs`; Microsoft *progress controls* |
| **F25.04** | **No loading state for lists.** Files shows its previous rows until `refresh_files` replaces them (`views.rs` 951); there is no skeleton, no "Loading…" row and no dimmed table | `views.rs` 951–955 |
| **F25.05** | **A slow failure and a fast failure read the same.** "Unavailable — litric-sysd did not answer" is drawn the moment the fact is missing, with no interim "asking…" state; the same string covers a daemon that is starting and one that is dead | `views.rs` 2409; `shell.rs` 7541 |
| **F25.06** | What is right and should be generalised: **the button keeps its label and adds motion**, the progress fraction is clamped by a test, and the boot intro uses a real value rather than a timer | `ui.rs` 1325; `views.rs` 37–40, 4190 |

Status for `F.25`: **PARTIAL** — Windows at page depth, GNOME at reference
depth, Apple summary-only; the shell measured by census.

---

## 1. What the four say together

| | `F.21` Grids/tables | `F.22` Cards | `F.24` Empty | `F.25` Loading |
|---|---|---|---|---|
| Windows expects | a `DataGrid` for table-like data; `GridView` for image-first collections | no rule retrieved | no rule retrieved | duration and blocking decide bar vs ring |
| GNOME expects | `GtkColumnView`: sort, resize, reorder, `TREE_GRID` | boxed list: 1–2 controls per row, min/max width, keyboard per control | `AdwStatusPage`: icon, title, description, action | the same status page with a spinner paintable |
| The shell has | two hand-rolled tables; 24 rows, no scroll, no resize | two local cards; no component | eight text-only states | one enum, two uses; no spinner |
| Missing | scroll, resize, shared model | component, states, keyboard | icon, sentence, action | indeterminate, blocking, list loading |

The chapter closes on a pattern that has held since Part 10. The shell is good at
**the single object** — one table cell elided correctly, one card ordered
stably, one sentence that names the exact cause of an emptiness, one button that
keeps its label while it works. It is thin at **the family**: the second table,
the second card, the eighth empty state and the seventh long operation each
invent their own answer, because there is no shared component to inherit from.

Two of this part's four items are not components at all, which is why they were
left for last. `F.24` and `F.25` are **policies** — what the product says when it
has nothing, and while it waits. The shell has a good policy for the first half
of `F.24` (empty ≠ unselected ≠ unavailable) and no policy at all for `F.25`.

## 2. What this part does not establish

| Claim | Why |
|---|---|
| that the Windows card or empty-state guidance says anything | neither was retrieved. No Windows rule is quoted for `F.22` or `F.24`, and the design-controls lists page returned 404 |
| that Apple's card, empty-state or table rules are as described | the pages that were read returned only their description line (summary depth); no Apple number is quoted |
| that the Files table cannot be scrolled at all | the draw loop stops at 24 and `FilesApp` has no table-scroll field, but a wheel handler elsewhere in `shell.rs` was not traced. The part states the 24-row draw and the missing model, not a user-visible dead end |
| that the task manager's six default columns are the shipped default | read from `taskmgr.rs` 725–730; the preference store can override it and was not traced |
| that `AdwStatusPage` is what GNOME apps use for file-manager empty states | it is the platform component for empty/error states; which apps use it is not measured |
| that the loading state count is exactly two | `State::Loading` was searched in `litric-shell`; other crates (browser, Defender daemon, settings) were not searched |
| that a skeleton or placeholder is required | no platform page read for this part requires one; the finding is the absence of any loading treatment for lists, not a rule that one is mandatory |
| anything about Parts 10–17's items | untouched. `F.21`, `F.22`, `F.24` and `F.25` were the four this part changed |

## 3. Sources

| # | Source | Depth | Used for |
|---|---|---|---|
| S3828 | Microsoft Learn — *List view and grid view* | full | ListView vs GridView, the "use a DataGrid for table-like collections" sentence, ItemsRepeater |
| S3829 | Microsoft Learn — *Guidelines for progress controls* | full | determinate/indeterminate bar and ring, blocking vs non-blocking, read-only controls, when no indicator is needed, keyword mapping |
| S3830 | Apple HIG — *Progress indicators* | summary | "isn't stalled while it loads content or performs lengthy operations"; body not readable |
| S3831 | GNOME HIG — *Boxed Lists* | full | card-like list contract: 1–2 controls per row, background click triggers the control, min/max width, expander rows |
| S3832 | GTK 4 — *Gtk.ColumnView* | reference | columns with headers, sort by header, resize and reorder, rubberband, `TREE_GRID` accessibility |
| S3833 | libadwaita — *Adw.StatusPage* | reference | icon, title, description and child action slots; `.compact` and spinner paintable |
| — | `docs/DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md`, `03-SHELL-SYSTEM.md` | repository | the card tokens and the accent-picker specification carried over from `Part 17` |
| — | `src/litric-shell/src/{views,ui,apps,taskmgr,shell,lock}.rs`, `src/litric-design/src/lib.rs` | repository | every shell-side number, line and absence in §0.2, F.21.2, F.22.2, F.24.2 and F.25.2 |

Pages were fetched on 2026-10-28. Five returned their body in full: two Microsoft
pages, one GNOME HIG page and two GTK/libadwaita references. One Apple page
returned a summary only, and the Microsoft lists page returned **404**; the
card and empty-state pages were not retrieved and are not cited.

## 4. Status changes

| Item | Was | Now | Note |
|---|---|---|---|
| `F.21` Grids/tables | `PENDING` | `PARTIAL` | two table implementations, 24 rows drawn with no table-scroll model, no resize or reorder, columns dropped silently, no grid (F21.01–F21.07) |
| `F.22` Cards | `PENDING` | `PARTIAL` | no card component; Mission Control cards and 110 × 48 start tiles are local inventions; tokens name cards but `ui.rs` does not build one; start tile is hover-only (F22.01–F22.05) |
| `F.24` Empty states | `PENDING` | `PARTIAL` | at least eight, all text-only, none with an icon, sentence or action; empty/unselected/unavailable correctly separated and tested (F24.01–F24.06) |
| `F.25` Loading states | `PENDING` | `PARTIAL` | `State::Loading` used twice, no spinner or indeterminate form, no list loading, no blocking/non-blocking distinction, slow and fast failure read alike (F25.01–F25.06) |

No decisions are recorded in this part. Whether the shell should grow a shared
table, a card component, an empty-state component with an action slot and an
indeterminate control is a product decision with a cost, and belongs in a
decision record rather than in a research note.

**Chapter 40 is now researched end to end: 27 of 27 items, 0 `PENDING`, 0
`COMPLETE`, 27 `PARTIAL`.** No item reaches `COMPLETE` because every one still
has a platform at summary depth (macOS throughout, Windows for cards and empty
states, GNOME for pickers) or a missing platform page. Closing those is a
retrieval problem, not a design problem, and it is the first task of the final
audit batch `J`. The next research batch by `Part1` §2.2 is `G` (design tokens
and accessibility), which is where the contrast, focus and accessible-name
findings from Parts 10–17 become measurable.
