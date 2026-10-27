# LitricOS UI/UX Research — Part 14 of N

Chapter 40 (F) — Menus and context menus: a menu bar with no menus, and six
context menus the keyboard cannot open

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P14 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-27 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.12` menus, `F.13` context menus. The other 12 `F` items are untouched and keep their status |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", continued on the two items that share one row vocabulary |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (2 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (sources appended) |
| Predecessors | Parts 10 (action controls), 11 (`F.02`, `F.18`, `F.23`, `F.26`), 12 (`F.04`, `F.05`), 13 (`F.08`, `F.10`, `F.11`) |
| Resume point | `F.14` tooltips and `F.15` dialogs — the two feedback surfaces that remain `PENDING` and both have code behind them |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these two

### 0.1 Two items, one idea of a menu, three implementations of it

A menu is the surface this shell uses most and names least. `F.12` and `F.13` are
adjacent in the checklist because they share a vocabulary — a row, a label, an
action — and the shell splits that vocabulary three ways:

| Surface | Drawing | Row metric | Opened by |
|---|---|---|---|
| The bar's three labels | `draw_top_bar`, `views.rs` line 404 | 13 px text on a 28 px bar | click, Meta tap, Meta+S |
| The two bar menus (LitricOS, Power) | `draw_litro_menu` line 5335, `draw_power_menu` line 5267 | **34 px** rows, 15 px text | click on the mark, the power glyph or the avatar |
| The six context menus | `draw_context_menu_panel`, `views.rs` line 171 | **26 px** rows, 13 px text | right-click, at six surfaces |

There is no `ui::menu`, no menu row type and no menu state. Each of the three is a
loop over literal strings that pushes a hotspot per row.

### 0.2 How each item is recorded

Same four parts as Parts 10–13: platform guidance with source and tier, what the
shell has (`MEASURED`), deltas, status. One addition: this part also compares the
implementation with the shell's **own** specification, `docs/DESIGN-SYSTEM-v3/`,
because for menus that document states a contract the code does not meet.

### 0.3 The numbers these two are made of (MEASURED)

| Measure | Value | Where |
|---|---|---|
| System bar height | **28 px**; labels are 13 px `Footnote`, 18 px apart | `chrome.rs` line 11; `views.rs` lines 415, 527–568 |
| Labels on the bar | **3** — Apps, Workspace, Environment — plus the wordmark | `views.rs` lines 527–530 |
| Labels that open a menu | **0 of 3**: the launcher overlay, the Activity window, the quick-settings panel | `shell.rs` lines 5357–5358, 5423–5424 |
| LitricOS menu | **2 rows**, 34 px, min width 184 px, 76 px tall, no icons | `views.rs` lines 5335–5405 |
| Power menu | **4 rows**, 34 px, 18 px glyph column, 12 px gap, 144 px tall | `views.rs` lines 5267–5330 |
| Bar-menu hover | white at **6 %**, `Radius::Md` rows on a `Radius::Lg` panel | `views.rs` lines 5291, 5379 |
| Context-menu row | **26 px**, 13 px `Footnote`, 4 px panel inset, 8 px text inset | `views.rs` lines 184, 216–238 |
| Context-menu panel | **190 px fixed** width, `Radius::Md` = 10, hover white at **9 %** | `views.rs` lines 185, 194, 232 |
| Context menus | **6 surfaces**, **28 items**, 7 separators, 1 section header | `apps.rs` lines 1702–1747 |
| Menu actions | **27** variants; **24** used by menus, 3 only by the start page's engine chips | `apps.rs` lines 1605–1658; `views.rs` lines 2098–2100 |
| Keyboard routes into a menu | **1 of 3** bar labels (Meta+S for quick settings); **0 of 6** context menus; `Key::Menu` and `F10` are unbound | `shortcuts.rs` lines 563, 681, 690; `keys.rs` line 76 |
| Menu strings in the Thai dictionary | **1 of 6** bar-menu labels ("Lock screen"); **0 of 28** context-menu labels routed through it | `i18n.rs` line 298; `apps.rs` lines 1702–1747 |

### 0.4 What the shell's own specification says

`docs/DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md` line 47 gives the context menu's
worklist as "**E3 shadow + hover state + disabled item** + submenu not in this
phase". `03-SHELL-SYSTEM.md` adds three rules this part measures against:

| Spec line | Requirement |
|---|---|
| §2 (line 22) | the desktop context menu is the same standard as the §02 set: E3 plus hover plus **disabled** |
| §3 (line 28) | the bar's left side is **one** Litric menu: About / Settings / Lock / Restart / Shutdown |
| §3 (line 30) | every bar icon opens an E3 popover on click — **never a new window** |

Both items are therefore measured twice: against the three platforms, and against
a contract written for this shell.

### 0.5 Evidence limits of this part

| Limit | Effect |
|---|---|
| Static reading only; no booted image, no compiled test run | Every claim is a claim about code. No menu was opened on a screen for this part |
| Apple's HIG pages returned titles and summaries to the reader | The macOS column reports the roles, the one context-menu submenu limit and the shortcut-placement rule from indexed text; no Apple size or keyboard binding is reported as verified |
| GNOME's separate popover, menu-button and context-menu paths returned 404 | GNOME guidance comes from the Menus page and the keyboard/pointer pages, not from a page dedicated to context menus |
| No user test | Whether the three bar labels *read* as menus to a person is not measured; only what they do is |
| No width sweep | The claim that a dropped bar label becomes unreachable is read from the layout loop, not from a resized window |

---

## F.12 Menus

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
summary-only; three bar labels that open no menu, and two popover menus that no key
opens).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use a **menu bar** for several always-visible top-level menus at the top of the window, and a **menu flyout** for commands attached to a button. Item kinds are `MenuFlyoutItem`, `MenuFlyoutSubItem`, `SplitMenuFlyoutItem` (App SDK 1.8.6+), `ToggleMenuFlyoutItem`, `RadioMenuFlyoutItem` and `MenuFlyoutSeparator` — that is, a menu is expected to express a toggle and a choice, not only an action. Icons are **16 × 16 px** and used where familiar; separators group. Disabled state exists; the current page gives no design rule for it, and the explicitly legacy Windows 7 guide advises removing inapplicable items from context menus and disabling them in ordinary menus. Touch targets: **40 × 40 epx**, or 32 epx high if at least 120 epx wide; **44 × 44** for touch-optimised UI. Keyboard: Up/Down moves inside a flyout, Tab and Shift+Tab navigate, light-dismiss flyouts trap focus until dismissed, **Esc cancels** and Left closes a submenu; access keys are Alt combinations, and an attached accelerator is **displayed with the item text**. UI Automation exposes a menu item as the **MenuItem** control type and announces its name and its checked or selected state. The legacy guide's numbers, quoted only as history: seven or fewer related items per group, 25 per level, 15 in a context menu, three levels, ten bar categories | learn.microsoft.com — [Menus and context menus](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus), [Menu flyout and menu bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions), [Access keys](https://learn.microsoft.com/en-us/windows/apps/develop/input/access-keys), [Keyboard accelerators](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-accelerators), [Touch interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/touch-interactions), [Guidelines for touch targets](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting), [Expose basic accessibility information](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information), [MenuItem control type](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-supportmenuitemcontroltype), [Windows 7 menus](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus) (T1; the Windows 7 page is flagged legacy and was partially extracted) |
| macOS 27 | The **menu bar** holds the app's top-level menus and its **complete** command set, including commands also offered contextually. A **pop-up button** chooses among mutually exclusive options and shows the current selection; a **pull-down button** offers actions related to the button's purpose. Menu-bar names are the app's own, in a fixed order, and the bar is reachable from the keyboard with **Control-F2** (or Fn-Control-F2), where arrows move, Return opens or chooses, and typing an item's name jumps to it; **Control-F8** reaches the status menu. Full Keyboard Access is the documented reason to keep items such as Minimize and Zoom in the menu bar. The pages returned summaries only: no item height, menu width or spacing was retrievable, and no menu-specific Escape rule was verified | developer.apple.com — [The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar), [Menus](https://developer.apple.com/design/human-interface-guidelines/menus), [Pop-up buttons](https://developer.apple.com/design/human-interface-guidelines/pop-up-buttons), [Pull-down buttons](https://developer.apple.com/design/human-interface-guidelines/pull-down-buttons), [Keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards); support.apple.com — [Mac keyboard shortcuts](https://support.apple.com/en-us/102650), [Navigate your Mac using Full Keyboard Access](https://support.apple.com/guide/mac-help/use-full-keyboard-access-mchlc06d1059/mac) (T1; HIG pages `SNIPPET_ONLY`, the two support pages read in full) |
| GNOME 51 | Three kinds, and the choice between them is about scope: the **primary menu** (the `open-menu-symbolic` hamburger, tooltip and accessible label **"Main Menu"**) for standard app-wide features, a **secondary menu** for the current view's actions, and a **context menu** for the pointed-at thing. A menu holds **3–12 items**, a submenu **3–6**, and submenus are not nested. Group similar items and keep actions, checkbox items and radio items in separate groups; for a setting use a **check or radio item** rather than relabelling one item between opposite actions; verbs for commands, adjectives for settings; add an **ellipsis** when the command needs further input; give every item an access key. **F10** opens a primary or secondary menu, **Menu / Shift+F10** the context menu, **Esc** closes a transient container. Every pointer action must also be keyboard-operable, and a context menu should not exist merely to offer alternative destructive actions. No width, item height, padding or icon rule is published on the Menus page | developer.gnome.org — [Menus](https://developer.gnome.org/hig/patterns/controls/menus.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Pointer & Touch](https://developer.gnome.org/hig/guidelines/pointer-touch.html), [Accessibility](https://developer.gnome.org/hig/guidelines/accessibility.html), [Writing Style](https://developer.gnome.org/hig/guidelines/writing-style.html) (T1; all read in full) |

The three agree on the shape of the surface: a menu is attached to a trigger, it
is reachable from the keyboard, its items can carry state, and it is dismissed
with Escape. They disagree on placement — Windows and macOS put app commands in a
bar inside the window or at the screen edge, GNOME refuses a menu bar entirely.

### What the shell has (MEASURED)

**The bar has three labels that name menus and open none of them.**

| Label | Drawn as | Click does | Opens a menu? | Keyboard | i18n |
|---|---|---|---|---|---|
| Apps | 13 px, dim; text colour + 12 % fill when lit | `Action::ToggleLauncher` | no — a full-screen launcher overlay | bare Meta | literal, not routed, no dictionary entry |
| Workspace | same | `Action::Open(App::Activity)` | no — **a new window** | none | literal |
| Environment | same | `Action::ToggleQuickSettings` | no — a bar popover with tiles | Meta+S | literal |

None of the three draws a chevron, a disclosure glyph or an open state. When they
do not fit, the layout stops drawing labels and draws one `›` — whose hotspot is
`Hot::MenuApps`, so it opens the launcher, not the two labels it replaced
(`views.rs` lines 566–579). The comment there says the dropped labels "are
reachable through the launcher, which is what these menus open anyway"; two of the
three do not open the launcher.

The two surfaces that *are* menus are somewhere else on the bar:

| | LitricOS menu (the wordmark) | Power menu (power glyph, avatar) |
|---|---|---|
| Rows | Session settings, Lock screen — **2** | Lock, Sleep, Shut down, Restart — **4** |
| Row height / text | 34 px, `Body` 15 px | 34 px, `Body` 15 px |
| Width | `max(widest + 30, 184)` px | `pad*2 + 18 + 12 + widest + 12` px |
| Height | **76 px** | **144 px** |
| Icons | none | a lock glyph on row 0, the **same power glyph** on rows 1–3 |
| Grouping | none | none: Lock acts at once, the other three open the confirmation dialog |
| Anchor | left, `x = 8`, `y = bar + 6` | right, `x = screen - w - 12`, `y = bar + 8` |
| State | hover/focus fill only; no check, disabled or destructive tone | same |
| Shortcut | none can open it | none can open it |
| Open state on its trigger | none | none: the power glyph and the avatar look the same while the menu is open |
| i18n | `i18n::text`, but only "Lock screen" is in the dictionary | `i18n::text`, but none of the four labels is in the dictionary |

**The bar's status capsule is drawn as one control and holds up to six.** The comment
on `views.rs` line 612 says the icons "share one capsule: they are one control, the
place the quick settings open from". Inside it: audio, network and battery → quick
settings, shield → scan, power → power menu, bell → notifications — four
destinations in one capsule — and the avatar beside it opens the same power menu
(`views.rs` lines 467–487; `shell.rs` lines 5423–5439).

**There is no menu component.** Nothing in `ui.rs` draws a menu; the two bar menus
are bespoke loops with their own padding, and the context menus are a third loop
with a different row height. The consequences are visible in the numbers: 34 px
rows at 15 px in the bar against 26 px at 13 px on the desktop; 6 % hover against
9 %; `Radius::Lg` panel against `Radius::Md`; an 18 px glyph column in one menu and
none in the other.

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **The menu bar contains no menus** | macOS: the bar lists the app's complete command set and is the route to every command; Windows: a bar of top-level menus, or a flyout attached to its button; GNOME: refuses the bar, and puts app-wide commands in one primary menu | Three labels in menu position do three unrelated things: a launcher, a window, a panel. A user who clicks "Workspace" expecting a menu gets a new window |
| **"Workspace" opens a window from the bar** | the shell's own spec, §3: every bar control opens a popover on click and **never a new window** | This is a specification violation with a one-line cause (`Action::Open(App::Activity)`), not a missing feature |
| The two real menus do not match the spec's item set | spec §3: one Litric menu with About / Settings / Lock / Restart / Shutdown | The shell has a 2-row LitricOS menu without About, Settings, Restart or Shutdown, and a separate 4-row Power menu that holds the shutdown commands. The split is defensible; the omission of About and Settings is not, and "Session settings" is not "Settings" |
| **No keyboard route to either menu** | Windows: Alt access keys, arrows, Esc; macOS: Control-F2, arrows, Return, typing to jump; GNOME: F10 for the menu, Menu/Shift+F10 for the context menu, Esc to close | The LitricOS and Power menus have no binding at all. `TogglePowerMenu` and `ToggleLitroMenu` exist as actions with labels for a shortcut table, and no key resolves to either. `Key::Menu` and `F10` are decoded by the input layer and bound to nothing |
| **No open or selected state on any trigger** | macOS menu titles highlight while their menu is open; Windows and GNOME toggle buttons show their state | Hover is the only state drawn. When the quick-settings panel is open, "Environment" is as dim as when it is closed |
| No checked, radio or disabled items anywhere in the bar menus | Windows: `ToggleMenuFlyoutItem`, `RadioMenuFlyoutItem`, disabled state; GNOME: check and radio items, and a state matrix for menus; the shell's own spec lists a disabled item as part of the component | A menu of four power actions has nothing to check, but the LitricOS menu cannot show, for example, which session settings are in force, and neither menu can disable a row that cannot act |
| Destructive rows carry no weight | macOS: destructive items last, and the system can draw them red; Windows (legacy): separate destructive commands; GNOME: undo or an explicit confirmation | "Shut down" and "Restart" are the 3rd and 4th rows of a 4-row menu, in the same tone as "Lock" and "Sleep", and carry no ellipsis although both open a confirmation |
| One Power glyph for three different actions | Windows: 16 px icons where familiar; macOS: icons used sparingly but meaningfully | Sleep, Shut down and Restart are drawn with the same glyph as the trigger that opened the menu. The icon column is 18 px wide and 12 px from the text, so the space is paid for and not used |
| **No shortcuts shown anywhere** | Windows: an attached accelerator is displayed with the item; macOS: shortcuts belong in the main menus; GNOME: menu items carry access keys | The shell has **43 bindings** and no surface that displays one. There is no menu bar for them to live in, so Ctrl+Shift+N, Ctrl+L and Meta+S are discoverable only by reading `shortcuts.rs` |
| Two menu metrics, two hover alphas, two radii | one component layer; the shell's own design system defines one radius and one motion grammar | 26 px/13 px/190 px/9 % on the desktop; 34 px/15 px/measured/6 % at the bar. Same designer, two answers |
| Bar menus are measured, the context menu is fixed | Windows and GNOME publish no width, so either is defensible | The difference is that only one of the two can elide or grow with a longer label; 190 px is also narrower than a translated German or Thai label of the same command can need |
| **i18n reaches one menu and not the others** | the shell ships a Thai locale (192 mapped strings) and a Thai-shaped font path | The bar menus call `i18n::text` and then miss their own strings: 1 of 6 labels is mapped. The context menus never call it, so 28 rows are English even where a translation exists — `"New folder"` is mapped at `i18n.rs` line 130 and drawn as a literal in six places across three files |
| The status capsule claims to be one control | Windows: one flyout per trigger; macOS: separate status items; the shell's own comment | Up to six hotspots with four destinations in one capsule, and the power glyph opens a menu that the avatar beside it also opens |

---

## F.13 Context menus

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
summary-only; six live surfaces with 28 rows, and no keyboard path to any of them).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | A **context menu** carries secondary commands for the content element it is attached to, and is invoked by right-click, touch press-and-hold, or the keyboard's **Shift+F10** or **Menu** button. It should contain **all** the contextual commands available for that item and provide an access path across input types; for the common ones (Cut, Copy, Paste, Delete, Share) a **CommandBarFlyout** with primary commands in a row is preferred. Frequent commands should also be directly reachable. Use a dialog or flyout — not a menu — for arbitrary content such as a confirmation. Icons **16 × 16 px**. Esc cancels; Esc or Left closes a submenu; a light-dismiss menu traps focus until dismissed. Any acceleration attached to an item is displayed with its text, and UIA exposes MenuItem with its name and state | learn.microsoft.com — [Menus and context menus](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus), [Menu flyout and menu bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus), [Contextual commanding for collections](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/collection-commanding), [Command bar flyout](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar-flyout), [Commanding basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/commanding-basics), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions), [Keyboard accelerators](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-accelerators), [Accessibility testing](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-testing) (T1; keyboard and accelerator pages partially extracted) |
| macOS 27 | A **context menu** is a small set of commands directly related to the clicked or selected item — a shortcut, never the only route to a command, because the menu bar lists everything. Order items to match the menu's position, put **destructive items at the end**, and let the system draw them red where appropriate. Keep shortcuts out of context menus and in the main menus. If submenus are needed, **one level**. A pop-up menu should offer at least **three** items so it is distinguishable from a pull-down. Right-click and Control-click open a context menu; the keyboard route is the menu bar's (Control-F2). The pages returned summaries only; no item height or width was retrievable | developer.apple.com — [Context menus](https://developer.apple.com/design/human-interface-guidelines/context-menus), [Menus](https://developer.apple.com/design/human-interface-guidelines/menus), [The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar), [Pop-up buttons](https://developer.apple.com/design/human-interface-guidelines/pop-up-buttons) (T1, `SNIPPET_ONLY`) |
| GNOME 51 | A **secondary action** opens a context menu for whatever is pointed at, and it **must also be accessible from the keyboard**, which the Keyboard page specifies as **Menu or Shift+F10** for the focused location. Only offer one when there are relevant items; do not use it as a way to offer alternative actions such as Delete or Remove. Structure: **3–12 items**, no nesting, actions and check items and radio items in separate groups, verbs for commands, an **ellipsis** when further input is needed, an access key on every item, and **Esc** closes the transient container | developer.gnome.org — [Menus](https://developer.gnome.org/hig/patterns/controls/menus.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Pointer & Touch](https://developer.gnome.org/hig/guidelines/pointer-touch.html), [Writing Style](https://developer.gnome.org/hig/guidelines/writing-style.html) (T1, read in full) |

All three name a keyboard way in — Shift+F10 or the Menu key — and a keyboard way
out, Escape. Neither exists in this shell.

### What the shell has (MEASURED)

**Six surfaces, 28 rows, one row type.**

| Surface | Rows | Separators | Section | Notes |
|---|---:|---:|---|---|
| `Desktop` (wallpaper) | 6 | 2 | — | New folder, Scan now, Open Trash, Next wallpaper, Display settings |
| `Files` (pane) | 7 | 3 | "Sort by" | New folder, New text file, Paste, then the three sort rows, then Details pane |
| `FilesRow` | 8 | 2 | — | Open, Cut, Copy, Rename, Move to trash, Compress to ZIP, Extract here |
| `Browser` | 2 | 0 | — | Start engine session, Reload |
| `BrowserTab` | 2 | 0 | — | Reload, Close tab |
| `Trash` | 3 | 0 | — | Restore, Delete permanently, Empty trash |

The row vocabulary is one enum: `Section(&'static str)`, `Item(&'static str,
MenuAction)`, `Separator`. There is no icon, no shortcut, no checked flag, no
disabled flag and no submenu — the four things all three platforms describe.

**How a menu opens, closes and is dispatched:**

| Step | Reading |
|---|---|
| Open | A **secondary press** computes the surface from the window under the pointer and calls `ContextMenu::for_surface` at the click point (`shell.rs` lines 5221–5245) |
| Clamp | x and y are clamped so the panel stays on screen (`views.rs` lines 187–190); a menu never opens off an edge |
| Row hit | Each row pushes `Hot::MenuRow(action)` while it is drawn (`views.rs` line 232); a retired menu registers nothing (test at `shell.rs` line 8878) |
| Choose | `Hot::MenuRow` **clears the menu first**, then calls `perform_menu` (`shell.rs` lines 5793–5796) |
| Dismiss | Only two paths clear it: `Hot::Desktop` (a wallpaper click) and choosing a row. A click on any window, toolbar or bar control leaves the menu open, floating above the new focus |
| Second right-click | Re-derives the surface from the window underneath, so a right-click on a row of an open menu opens a *new* menu underneath the pointer — the panel is not part of the surface map (`shell.rs` lines 5841–5879) |
| Keyboard | None in or out: `Key::Menu` and `F10` are unbound, `Shift+F10` resolves to nothing, and Escape resolves to `CloseOverlay`, which sets `overlay = None` and does not touch `context_menu` (`shortcuts.rs` line 690; `shell.rs` line 5900) |
| States | Hover **is** focus (`is_highlighted` covers both), so a keyboard-focused row looks hovered; there is no pressed state and no disabled state |
| Browser page | When a page frame exists and the browser is focused, the page branch returns before the menu branch, so the right-click goes to Chromium (`shell.rs` lines 5199–5202, 1947–1965). The shell's `Browser` menu therefore appears over browser chrome, or over the page when no frame exists — the same gesture has two meanings |

**The dispatch defect.** `perform_menu(NewFolder)` chooses between the Files
dialog and the desktop dialog by reading `self.context_menu`:

```
let in_files = self.context_menu.as_ref().is_some_and(|m| matches!(m.surface,
    MenuSurface::Files | MenuSurface::FilesRow));
```

`Hot::MenuRow` clears that field before calling it, so `in_files` is always false.
Every "New folder" from the Files window or a Files row opens
`DesktopDialog::NewFolder`, and the dialog writes `bridge.mkdir("/Documents", name)`
(`shell.rs` lines 2371–2384, 2652–2663) — the folder lands in `/Documents`
whatever folder the window is showing. The Files toolbar's "New folder" is the
same hotspot (`views.rs` line 1096), so it takes the same path, and
`FilesApp::ask_new_folder` — the only code that opens a dialog in the window's own
folder — is reached from `Ctrl+Shift+N` alone (`shell.rs` line 6104).

**What the row wording and layout say.**

| Element | Reading |
|---|---|
| Label size | 13 px `Footnote`, drawn at `row.y + row.h/2 + size*0.36` with no elision, in a fixed 190 px panel |
| Section header | 12 px `Caption` at `x + 12`, one row tall — "Sort by" is the only one in the product |
| Separators | 1 px rule inset 8 px, half a row tall; 7 of them across the six menus |
| Label casing | Sentence case throughout, which matches GNOME's rule and Windows' current style |
| Ellipsis | none on any of the 28 rows, although four of them open a confirmation or a dialog ("Move to trash", "Delete permanently", "Empty trash", and the bar's "Shut down"/"Restart") |
| Destructive order | "Move to trash" is row 5 of 8, between Rename and Compress; Trash's two destructive rows are rows 2 and 3 with no separator and no danger tone |
| Icons | none; the shell has an icon set, and 16 px is the Windows menu size |
| Disabled | none: Paste with an empty clipboard, Extract with no archive selected, Restore with nothing selected and Reload with no page are all drawn and clickable |
| Checked | none: the current sort key is not marked, and "Details pane" is a plain row for a toggle |
| i18n | Labels are `&'static str` and never pass through `i18n::text`; `"New folder"` has a Thai mapping and is drawn in English at all five call sites |

**Tests.** `journey_context_menu_rows_all_do_something`
(`e2e.rs` lines 1371–1393) claims "every menu row the desktop can draw must be an
action the shell implements: the menu can never contain a dead entry", and then
asserts `let _ = s.perform_menu(*action)` — no panic. A row that silently does
nothing passes: `Restore` with `trash.selected == None` is exactly that. No test
opens a menu from the keyboard, presses Escape, clicks away, or checks the folder
a "New folder" row targets.

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **No keyboard path in or out** | Windows: Shift+F10 or the Menu key, arrows, Esc; macOS: Control-F2 into the menu bar, Esc; GNOME: Menu/Shift+F10, Esc, and "every pointer action must be keyboard-operable" | Six surfaces out of six are pointer-only. `Key::Menu` is decoded (`keys.rs` line 76) and bound to nothing; Shift+F10 resolves to nothing; Escape closes overlays and context menus not at all. This is the largest finding of the item, and the one the shell's own accessibility contract cannot survive |
| **Dismissal is incomplete** | all three: light-dismiss on click-away, Esc, and choosing an item | Two exits exist: the wallpaper and a row. Clicking a Files row, a toolbar button or a bar control leaves the menu over the new surface. A second right-click re-derives the surface instead of dismissing, so the menu can be replaced by another menu under the pointer |
| **No disabled state** | Windows: a disabled menu item leaves the tab order and cannot be invoked; GNOME: invalid actions are insensitive; the shell's own spec names a disabled item explicitly | Four rows are drawn live when they cannot act. The shell *has* disabled buttons in the browser toolbar, so this is a per-surface omission, not a missing capability |
| **No checked or radio state** | Windows: Toggle and Radio items; GNOME: check and radio items for settings, and "do not relabel one item between opposite actions" | "Sort by" offers three rows and marks none of them; "Details pane" toggles with nothing to show whether it is on. The menu cannot express the state it changes |
| **Destructive rows sit in the middle, unseparated, untinted** | macOS: destructive last, may be red; Windows (legacy): separate destructive commands; GNOME: undo or an explicit confirmation | Files' "Move to trash" is 5th of 8; Trash offers Restore, Delete permanently and Empty trash as one unbroken block. The confirmations that follow are the right mechanism, placed after the wrong layout |
| **No ellipsis on rows that ask again** | GNOME: an ellipsis when the command needs further input; the same convention in Windows dialogs and macOS sheets | Four rows open a prompt without saying so. "Empty trash" reads like a one-click action and raises a confirmation row instead |
| **Dispatch reads state the click already destroyed** | no platform; this is internal consistency | Files and FilesRow "New folder" always opens the desktop dialog and writes `/Documents`. The visible folder is ignored, the toolbar button inherits the bug, and the only correct path is a shortcut |
| Fixed 190 px width with no elision | Windows and GNOME publish no width; both expect the label to fit or be ellipsized | The label is drawn with `font::draw` and never measured against the panel. A longer label, a translation, or a larger text scale overflows the panel rather than shortening |
| 26 px rows | Windows: 40 × 40 epx general target, 32 epx high when at least 120 wide, 44 × 44 for touch; Apple: 44 pt; GNOME: qualitative | A menu row is a target. 26 px is below the Windows general minimum and well below the touch figure; the bar menus' 34 px are also below it |
| No icons, no shortcuts, no access keys | Windows: 16 px icons where familiar, accelerators shown; macOS: shortcuts in the main menus; GNOME: access key on every item | The 190 px panel is text-only while a 16 px icon column would fit. No row shows the key that does the same thing (Ctrl+Shift+N, Ctrl+C, Delete), and no Alt mnemonic exists |
| The Browser page menu is conditional on a frame | all three: the menu belongs to the content, and the content is the page | With a frame, Chromium gets the right-click (correct) and the shell's page menu is unreachable; without one, the shell menu appears in the same place. Neither is wrong alone; together they are one gesture with two outcomes and no indication which |
| Test name promises more than the assertion | — | `journey_context_menu_rows_all_do_something` proves no panic, not an effect. The property worth testing is "each row changes something the surface can observe", and the test does not state it |
| Dismissal of the **bar** panels works | — | Escape and click-away both close the launcher, quick settings, power and LitricOS panels. The contrast is the finding: the shell knows how to dismiss a popover and does not apply it to a context menu |

---

## 5. Findings this part files without deciding

Continuing Parts 10–13's numbering. None of these is a ledger row: a row needs a
decision with a backend and a justification that a research note does not have.

| # | Finding | Where it is visible | Owner |
|---|---|---|---|
| 50 | Three bar labels in menu position open a launcher, a window and a panel — none opens a menu | `views.rs` lines 527–568; `shell.rs` lines 5357–5358 | `F.12` decision |
| 51 | "Workspace" opens a window from the bar, which the shell's own spec forbids | `shell.rs` line 5423; `DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` line 30 | `F.12` |
| 52 | Bar labels that do not fit are replaced by one `›` that opens the launcher, so the dropped labels become unreachable | `views.rs` lines 566–579 | `F.12` / `F.11` |
| 53 | The status capsule is drawn as one control and holds up to six hotspots with four destinations | `views.rs` lines 467–487, 612–645; `shell.rs` lines 5423–5439 | `F.12` |
| 54 | No open or selected state on any bar trigger; hover is the only state drawn | `views.rs` lines 551, 623–645 | `F.12` / `G.07` |
| 55 | The LitricOS and Power menus have no keyboard binding; `Key::Menu` and `F10` are decoded and unbound; Escape closes panels but not context menus | `shortcuts.rs` lines 563, 681, 690; `keys.rs` line 76 | `F.12` / `F.13` / `G.14` |
| 56 | The Power menu draws one glyph for sleep, shut down and restart, groups nothing, tints nothing and marks no row as needing confirmation | `views.rs` lines 5267–5330 | `F.12` |
| 57 | The LitricOS menu has two rows and the spec asked for five (About, Settings, Lock, Restart, Shutdown) | `views.rs` lines 5335–5405; spec §3 line 28 | `F.12` |
| 58 | Files "New folder" — toolbar and both menus — writes `/Documents` regardless of the folder on screen, because dispatch reads a field the click cleared | `shell.rs` lines 5793–5796, 2371–2384, 2652–2663; `views.rs` line 1096 | `F.13` / `F.27` |
| 59 | Context menus cannot be opened, navigated or dismissed from the keyboard; only a wallpaper click or a chosen row closes one | `shell.rs` lines 5221–5245, 5782–5796, 5900 | `F.13` / `G.14` |
| 60 | No disabled, checked or radio state in any of the 28 rows; no menu can show a current sort, a toggle or an unavailable action | `apps.rs` lines 1595–1601, 1702–1747; `views.rs` lines 216–238 | `F.13` decision |
| 61 | Four rows that end in a confirmation carry no ellipsis | `apps.rs` lines 1710–1745; `views.rs` lines 5271 | `F.13` / `F.27` |
| 62 | Destructive rows are unseparated and in the middle: "Move to trash" 5th of 8; Trash's two destructive rows directly under Restore | `apps.rs` lines 1724–1733, 1742–1746 | `F.13` / `F.27` |
| 63 | Two menu metrics and two hover alphas: 26 px/13 px/9 % on the desktop against 34 px/15 px/6 % at the bar | `views.rs` lines 184–232, 5291, 5379 | `F.12` / `F.13` |
| 64 | Context-menu labels bypass i18n entirely; `"New folder"` is translated in the dictionary and drawn in English in six places, and the bar menus miss five of their six labels | `apps.rs` lines 1702–1747; `i18n.rs` lines 109–130, 298 | `F.12` / `F.13` / `G` |
| 65 | `journey_context_menu_rows_all_do_something` asserts no panic rather than an effect, so a silent no-op row passes | `e2e.rs` lines 1371–1393 | `F.13` / `J` |
| 66 | A right-click on a live page goes to Chromium; the same gesture opens the shell's Browser menu when there is no frame | `shell.rs` lines 5199–5202, 1947–1965 | `F.13` |

## 6. What this part does not establish

| Claim | Why not |
|---|---|
| that the bar labels *read* as menus | no user was asked. The finding is what they do, not how they look |
| that a bar label is actually dropped at a plausible width | the layout loop was read, not a resized window; the claim is that nothing in the loop preserves the label |
| that Escape leaves the menu open in every focus state | the call sites that clear `context_menu` are exhaustive for the code as written; the claim is that no key path clears it, not that a particular frame was observed |
| that menus were visually checked | no image was booted and no frame was captured. Every geometry number is read from the drawing code |
| that the Files "New folder" bug is reproducible on a booted image | the code path is unambiguous, but no test asserts it and no image was run. It is recorded as a reading of `perform_menu` and `submit_desktop_dialog`, not as a captured failure |
| that macOS agrees with any of this | every Apple HIG page returned a title and a summary. The macOS column is roles plus two indexed rules (`SNIPPET_ONLY`) |
| that `F` is nearly done | 11 of 27 items remain `PENDING`; 16 are `PARTIAL`; none is `COMPLETE` |

## 7. Coverage added, and what would finish these two items

| Item | Before | After | What is still missing |
|---|---|---|---|
| `F.12` Menus | `PENDING` | `PARTIAL` | macOS at any depth; a decision on what the bar's three labels are (menus, buttons, or a primary menu), a decision on the item set of the session menu, and a keyboard route to every menu |
| `F.13` Context menus | `PENDING` | `PARTIAL` | macOS at any depth; a decision on the row state model (disabled, checked, radio), a keyboard model (Shift+F10, arrows, Esc), a dismissal rule, and a rule for dispatch that does not depend on the surface that opened the menu |

What would move each to `COMPLETE`: Apple's pages read in full, one decision per
item with a backend and a milestone, and — for both — one shared menu component in
`ui.rs`, because the three drawings in this part are the reason the same gesture
has three different row heights.

## Sources read for this part

Access date for all rows: 2026-10-27 (Asia/Bangkok). Tier definitions: `Part1` §2.1.

| Source | Tier | Status | Limitations |
|---|---|---|---|
| [Menus and context menus — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus) | T1 | INSPECTED | — |
| [Menu flyout and menu bar — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus) | T1 | INSPECTED | No item-count, width or height rule |
| [Contextual commanding for collections — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/collection-commanding) | T1 | INSPECTED | — |
| [Command bar flyout — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar-flyout) | T1 | INSPECTED | — |
| [Commanding basics — Windows](https://learn.microsoft.com/en-us/windows/apps/design/basics/commanding-basics) | T1 | INSPECTED | Undo-versus-confirm guidance only |
| [Keyboard interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) | T1 | INSPECTED | Partial extraction |
| [Access keys — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/access-keys) | T1 | INSPECTED | — |
| [Keyboard accelerators — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-accelerators) | T1 | INSPECTED | Partial extraction |
| [Touch interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/touch-interactions) | T1 | INSPECTED | General targets |
| [Guidelines for touch targets — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting) | T1 | INSPECTED | Density-dependent |
| [Expose basic accessibility information — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information) | T1 | INSPECTED | — |
| [Accessibility testing — Windows](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-testing) | T1 | INSPECTED | — |
| [MenuItem control type — Windows](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-supportmenuitemcontroltype) | T1 | INSPECTED | Semantics only |
| [Windows 7 menus — Design basics](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus) | T1 (legacy) | INSPECTED | Page itself says it is not updated for newer Windows; partial extraction; quoted as history only |
| [Context menus — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/context-menus) | T1 | SNIPPET_ONLY | Title and indexed text only |
| [Menus — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/menus) | T1 | SNIPPET_ONLY | as above |
| [The menu bar — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar) | T1 | SNIPPET_ONLY | as above |
| [Pop-up buttons — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/pop-up-buttons) | T1 | SNIPPET_ONLY | as above |
| [Pull-down buttons — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/pull-down-buttons) | T1 | SNIPPET_ONLY | as above |
| [Keyboards — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/keyboards) | T1 | SNIPPET_ONLY | as above |
| [Mac keyboard shortcuts — Apple Support](https://support.apple.com/en-us/102650) | T1 | INSPECTED | Control-F2 and Control-F8 menu-bar access |
| [Navigate your Mac using Full Keyboard Access — Apple Support](https://support.apple.com/guide/mac-help/use-full-keyboard-access-mchlc06d1059/mac) | T1 | INSPECTED | — |
| [Menus — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/menus.html) | T1 | INSPECTED | 3–12 items, 3–6 per submenu, no nesting; no sizes |
| [Keyboard — GNOME HIG](https://developer.gnome.org/hig/guidelines/keyboard.html) | T1 | INSPECTED | F10, Menu/Shift+F10, Esc |
| [Pointer & Touch — GNOME HIG](https://developer.gnome.org/hig/guidelines/pointer-touch.html) | T1 | INSPECTED | Context menus must be keyboard-accessible |
| [Accessibility — GNOME HIG](https://developer.gnome.org/hig/guidelines/accessibility.html) | T1 | INSPECTED | — |
| [Writing Style — GNOME HIG](https://developer.gnome.org/hig/guidelines/writing-style.html) | T1 | INSPECTED | Ellipsis rule only |

Not found: `developer.gnome.org/hig/patterns/controls/popovers.html`,
`.../menu-buttons.html` and `.../context-menus.html` returned 404, so no GNOME page
dedicated to context menus or popovers is cited.

Repository sources read (no tier): `src/litric-shell/src/views.rs`,
`src/litric-shell/src/apps.rs`, `src/litric-shell/src/shell.rs`,
`src/litric-shell/src/shortcuts.rs`, `src/litric-shell/src/i18n.rs`,
`src/litric-shell/src/e2e.rs`, `src/litric-shell/src/chrome.rs`,
`src/litric-input/src/keys.rs`, `src/litric-design/src/lib.rs`, and the shell's own
specification in `docs/DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md` and
`docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md`.
