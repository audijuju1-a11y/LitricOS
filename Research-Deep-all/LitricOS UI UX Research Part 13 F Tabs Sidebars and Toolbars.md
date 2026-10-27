# LitricOS UI/UX Research — Part 13 of N

Chapter 40 (F) — Tabs, sidebars and toolbars: three containers, five hand-drawn
implementations

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P13 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-26 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.08` tabs, `F.10` sidebars, `F.11` command bars/toolbars. The other 14 `F` items are untouched and remain `PENDING` |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", continued on the three layout containers with live surfaces |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (3 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (sources appended) |
| Predecessors | Parts 10 (action controls), 11 (`F.02`, `F.18`, `F.23`, `F.26`), 12 (`F.04`, `F.05`) |
| Resume point | `F.12` menus — the context menu is live at five surfaces (`MenuSurface`), so it is the next container with code behind it |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet", and nothing in this part changes that for the
14 items it does not touch. Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE` tags and
the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these three

### 0.1 Five implementations of three containers, and no shared component

Parts 10–12 covered controls that live inside `ui.rs`. These three are containers,
and every one of them is drawn inline in `views.rs`:

| Container | Implementation | Surfaces | Shared component |
|---|---|---|---|
| Tabs | `draw_browser`'s tab strip loop, `views.rs` from line 1575 | one: the Browser window | none |
| Sidebars | `draw_files`' places pane (line 957) and `draw_settings`' category list (line 2337) | two: Files and Settings | none; they do not share geometry, background, row height or selection treatment |
| Toolbars | the Browser's command row (line 1676), the Files command row (line 1032), the Trash toolbar (line 4172) | three | none; each has its own overflow policy |

That is the shape this part documents: three containers, five drawings, three
different answers to "what happens when the window is too small".

### 0.2 How each item is recorded

Same four parts as Parts 10–12: platform guidance with source and tier, what the
shell has (`MEASURED`), deltas, status.

### 0.3 The numbers these three are made of (MEASURED)

| Measure | Value | Where |
|---|---|---|
| Tab strip height | **30 px**; tab **150 × 26 px**, 2 px gap, 4 px left inset | `views.rs` lines 1575–1583 |
| Tab title truncation | 20 characters, rendered as 17 + `…` | `views.rs` lines 1601–1607 |
| Tab close button | **15 × 15 px**, inset 3 px, always drawn | `views.rs` lines 1610–1619 |
| New-tab button | **24 × 20 px**, after the last tab | `views.rs` line 1621 |
| Browser toolbar buttons | 24 px tall, width = label + 18 px, 4 px gap; one row of 9 or two rows of 5/4 (controls area 30 or 56 px) | `views.rs` lines 1687–1745 |
| Browser toolbar action list | nine text buttons: Back, Forward, Reload, Stop, Save/Saved, History, Bookmarks, Downloads, Site | `views.rs` line 1676 |
| Files places pane | **132 px** wide, `Material::Elevated`, heading "Quick access", rows **20 px** | `views.rs` lines 957–997 |
| Settings sidebar | **36 %** of the window body width, black at 16 %, **13 pages** at **30 px** rows, 2 px gap, no heading | `views.rs` lines 2337–2360; `settings.rs` line 173 |
| Files command row | 26 px tall; buttons **22 px**; New folder, Paste, Compress, Extract, Details toggle | `views.rs` lines 1032–1136 |
| Trash toolbar | 30 px row: search field, then Empty (76), Restore (76), Delete (88), or confirm (112) + Cancel (70) | `views.rs` lines 4172–4230 |

### 0.4 What these three share with the earlier items

None of the five drawings takes `ui::State` for the container itself: tabs use a
`ui::Material` for the active tab, the panes use raw fills, and the three toolbars
use `ui::button` per item. So the eight-state contract stops at the individual
button, exactly as Parts 10–12 found for the other controls. The difference here
is that the *container* has behaviour of its own — overflow, navigation, selection
— and that behaviour is where most of the deltas are.

### 0.5 Evidence limits of this part

| Limit | Effect |
|---|---|
| Static reading only; no booted image | Every layout claim is a claim about coordinates in the code. The overflow claims are read from the loops, not from a window resized on screen |
| Apple's HIG pages returned titles and summaries only | The macOS column is the weakest again; no Apple tab, sidebar or toolbar number is reported as verified |
| Windows publishes numbers for its controls (320/48 DIP panes, 1008/641/640 breakpoints, 20/16 px icons) and GNOME publishes almost none | The shell can be measured against Windows' sidebar widths and against libadwaita's 25 % / 180–280sp defaults, not against a GNOME HIG rule |
| No keyboard or screen-reader testing ran | The keyboard claims are readings of the hotspot table and `app_key` |
| No tab overflow was exercised | The claim that tabs run off the edge is read from a loop with no clip, no scroll and no width cap; it was not seen on a screen |

---

## F.08 Tabs

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
summary-only; one live tab strip, and it has no overflow or keyboard model).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use **document tabs** (`TabView`) when users can open, close, rearrange or move documents between windows; static tabs suit a few fixed pages, and more than a few navigation choices belong in `NavigationView`. Each tab has a descriptive header; closable tabs show a close button, and `CloseButtonOverlayMode="OnPointerOver"` keeps the selected tab's X visible while revealing others on pointer-over. An **add-tab (+)** button and reordering are part of the pattern; if tabs no longer fit, **scroll bumpers** move the strip to reach hidden tabs. Width modes: **Equal** (default), **SizeToContent**, **Compact** (icons only when unselected); per-item `Width` is ignored, and no fixed tab width or maximum count is specified. Keyboard: focus enters at the selected tab; Left/Right moves focus without selecting; Space/Enter selects; Ctrl+Tab / Ctrl+Shift+Tab cycle and wrap; Ctrl+F4 requests close; recommended browser-style additions are Ctrl+T, Ctrl+W, Ctrl+Shift+T, Ctrl+1–8 and Ctrl+9. Use meaningful text headers, not icon-only identifiers | learn.microsoft.com — [Tab View](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/tab-view), [Navigation design basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/navigation-basics), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) (T1) |
| macOS 27 | Apple defines a **tab view** as multiple mutually exclusive content panes switched with a tabbed control, a **tab bar** as navigation among an app's top-level sections, and a **segmented control** as a linear set of two or more button-like segments. The only verified number is "two or more segments". No tab width, close-button rule, overflow rule or keyboard binding was retrievable | developer.apple.com — [Tab views](https://developer.apple.com/design/human-interface-guidelines/tab-views), [Tab bars](https://developer.apple.com/design/human-interface-guidelines/tab-bars), [Segmented controls](https://developer.apple.com/design/human-interface-guidelines/segmented-controls) (T1, `SNIPPET_ONLY`: titles and summaries only) |
| GNOME 51 | Use tabs for a **mutable set of content items** (pages, documents, images); editors and browsers are the named cases. Tabs are distinct from a view switcher (a small fixed set, 3–5 views) and from a sidebar (too many views for a switcher). A tab may carry an icon, a label, or both when necessary; keep labels short and meaningful when ellipsized; the HIG specifies **no tab width**. Provide a context menu per tab: **Close last**, and where supported Duplicate, Pin/Unpin, Mute, Reload, **Move to New Window**; avoid left/right wording for RTL. The HIG specifies no overflow algorithm; libadwaita's `AdwTabBar` scrolls and keeps pinned tabs outside the scroll area. The HIG does not require a + button but names **Ctrl+T**; `AdwTabBar` allows action widgets before/after the tabs. Shortcuts: Ctrl+T new, Ctrl+W close, Ctrl+PageUp/PageDown next/previous, Ctrl+Home/End first/last, Ctrl+Tab switch, Alt+1–9 jump. Pointer guidance says not to rely on hover to reveal actions | developer.gnome.org — [Tabs](https://developer.gnome.org/hig/patterns/nav/tabs.html), [View Switchers](https://developer.gnome.org/hig/patterns/nav/view-switchers.html), [Navigation](https://developer.gnome.org/hig/guidelines/navigation.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Pointer & Touch](https://developer.gnome.org/hig/guidelines/pointer-touch.html); libadwaita — [Adw.TabBar](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/class.TabBar.html), [Adw.TabView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.7/class.TabView.html) (T1; libadwaita pages are T2 implementation detail) |

All three name a way to reach tabs that do not fit, a key set for switching, and
a close affordance. Windows gives the most numbers and GNOME the most menu
structure; macOS gives neither.

### What the shell has (MEASURED)

`draw_browser` lays out one 150 px tab per open tab, left to right, with no clip
and no cap:

| Element | Reading |
|---|---|
| Strip | 30 px tall; tabs 150 × 26 at 2 px gaps; the plus button follows the last tab |
| Width policy | **fixed 150 px**: not equal-width-to-content, not size-to-content, not compact. Two tabs and twenty tabs each get 150 |
| Overflow | **none**. No scrolling, no scroll bumpers, no "more tabs" affordance, no minimum width. The loop adds 152 px per tab and never checks the body width, so past roughly `body.w / 152` tabs the strip runs off the window edge; the plus button is drawn after them and goes with them. There is no clip set in the function, so the window's own clip is the only bound |
| Active tab | `ui::Material::Elevated` versus `Flat` — a fill difference only. Same 13 px text, same weight, no underline, no accent bar, no bold |
| Title | Falls back to the URL when a page has no title; truncated by **character count** (20 → 17 + `…`), not by measured width |
| Close | A 15 × 15 px ✕ hotspot inside the tab, **always visible**, including on the inactive tabs and the last tab |
| New tab | A 24 × 20 px `+` hotspot after the last tab; no shortcut |
| Tab context menu | Right-click a tab opens `MenuSurface::BrowserTab` with exactly two items: **Reload**, **Close tab** | 
| Keyboard | `app_key` for `com.litric.browser` handles **Escape and Enter only**. There is no Ctrl+T, Ctrl+W, Ctrl+Tab, Ctrl+PageUp/Down, Ctrl+1–9, Alt+1–9 or arrow-key movement in the strip. Tabs and their close buttons are ordinary hotspots, so Tab visits **two stops per tab** and Enter activates whichever has focus. **Ctrl+T is Settings' "cycle theme"** (`theme` preference), not a new tab |
| Reorder / tear-out / pin / mute | none of them exist: no drag state, no pinned flag, no mute flag, no move-to-window action |
| Closing the last tab | creates a fresh tab (Chrome's behaviour), so the window never has zero tabs |
| Switching tabs | clears the engine frame and `last_frame` before activating the new tab, so one site's pixels are not shown under another's URL |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **No overflow policy at all** | Windows: scroll bumpers when tabs do not fit; GNOME/libadwaita: a scrolling tab bar with pinned tabs outside the scroll | The strip is a plain accumulation. A browser with enough tabs loses the right-hand tabs and the + button with no way to reach them. This is the largest finding of the item |
| **No keyboard model for tabs** | Windows: arrows move focus, Space/Enter select, Ctrl+Tab and Ctrl+1–9 jump; GNOME: Ctrl+T/W/PageUp/PageDown/Home/End/Alt+1–9 | Every tab switch is a pointer click. The keyboard reaches tabs only as two Tab stops per tab, with the close button as its own stop — the opposite of the platforms' "one stop in the strip, arrows inside" |
| **Ctrl+T means "cycle theme"** in Settings | Every platform and every browser: Ctrl+T is new tab | The one shortcut a browser user will try first does something unrelated in the window next door. This is a key-binding collision, not merely a missing shortcut |
| Active tab is distinguished only by surface material | Windows and GNOME connect the selected tab to its content; the shell's own `ui.rs` contract says every state changes more than a hue | `Elevated` versus `Flat` is a subtle fill difference. There is no text weight, underline, accent edge or icon, and no accessible state |
| Close button is 15 × 15 and always visible | Windows: close on the selected tab, others on pointer-over; GNOME: close is the last item in the tab menu, and hover-only actions are discouraged; general targets are 40 px (Windows) / 44 pt (Apple) | The target is a third of the Windows touch guidance. Being always visible is defensible on a desktop; being 15 px is not |
| The tab context menu has two items | GNOME: Close last, plus Duplicate, Pin, Mute, Reload, Move to New Window where supported; Windows: reorder and tear-out | Reload and Close are the minimum. There is no duplicate, no pin, no mute, no move-to-window |
| Fixed 150 px width | Windows: Equal/SizeToContent/Compact modes; GNOME: no width rule, scroll instead | Fixed width wastes space on short titles and truncates long ones at 17 characters, while giving every tab the same 150 px. `AdwTabBar` expands or scrolls |
| Title truncation by character count | All three ask for a meaningful ellipsized label; none asks for a character cap | A Latin title of 17 characters fits; a title of 17 wide glyphs (CJK) can exceed the 150 px tab. The measured width is available at the call site and unused |
| No pinned, muted or audio state | Windows and GNOME both document tab states beyond selected | The shell has no per-tab state except active and URL |
| Closing the last tab leaves a fresh one | — | Correct: matches Chrome and the Windows rule that a strip always has an active tab |
| Switching tabs clears the stale frame | — | Correct, and the comment says why: a frame from the old tab must not sit under the new URL |

---

## F.10 Sidebars

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
summary-only; two live sidebars with two different designs).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use `NavigationView` for consistent **top-level app navigation**, adapting to window width. **Top** suits five or fewer equally important categories when all must be visible or icons do not describe them; **Left** suits **5–10** equally important categories when navigation should be prominent. Left navigation generally suits more than five top-level items that are not switched frequently; keep the hierarchy to **two levels** and use breadcrumbs beyond that. Modes: **Left** (expanded pane), **LeftCompact** (icons, overlay when opened), **LeftMinimal** (menu button only), **Auto** (default) adapting at **1008 px** (expanded), **641–1007 px** (compact) and **≤640 px** (minimal). Default expanded width **320 DIP**, compact **48 DIP**; header **52 px**; content margins **12 px** in minimal and **24 px** otherwise. Pane content can include items, separators, group headers, search and a Settings entry; footer items have their own collections. The selected item shows **one** indicator, on the left edge in Left mode and the bottom edge in Top mode. Keyboard: Up/Down in Left, Left/Right in Top; Space/Enter invokes; Escape closes a flyout. Compact items need clear icons and tooltips | learn.microsoft.com — [NavigationView](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/navigationview), [Navigation design basics](https://learn.microsoft.com/en-us/windows/apps/design/basics/navigation-basics), [Keyboard interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions), [OpenPaneLength](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.navigationview.openpanelength?view=windows-app-sdk-2.0), [CompactPaneLength](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.navigationview.compactpanelength?view=windows-app-sdk-2.0) (T1; the NavigationView page was partially extracted) |
| macOS 27 | Apple describes a sidebar as appearing on the **leading side** of a view and helping people navigate among app areas or top-level collections (folders, playlists). `NavigationSplitView` describes two- or three-column presentations in which a leading column's selection controls the next. No sidebar width, row height, section rule, collapse behaviour or keyboard binding was retrievable | developer.apple.com — [Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars), [NavigationSplitView](https://developer.apple.com/documentation/swiftui/navigationsplitview) (T1, `SNIPPET_ONLY`) |
| GNOME 51 | A sidebar is **flat navigation among locations or views**, for when a view switcher cannot hold the number of views, when destinations are dynamic, or when users switch back and forth often; avoid it for rich immersive content. Order destinations by usefulness; rows may hold text or images. Put sidebar-affecting controls **above the list**, and with a sidebar put the **primary menu above the list** rather than at the far end of the header bar. The HIG gives **no fixed width**; container widths must stay appropriate as the window changes, and a sidebar should not look excessively wide or narrow. libadwaita's split-view defaults (implementation, not HIG): **25 %**, minimum **180sp**, maximum **280sp**, with an example **400sp** collapse breakpoint. Keyboard: all actions keyboard-operable, arrows move directionally where possible, descriptive accessible names | developer.gnome.org — [Sidebars](https://developer.gnome.org/hig/patterns/nav/sidebars.html), [Navigation](https://developer.gnome.org/hig/guidelines/navigation.html), [Menus](https://developer.gnome.org/hig/patterns/controls/menus.html), [Scaling & Adaptiveness](https://developer.gnome.org/hig/guidelines/adaptive.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html); libadwaita — [Adw.NavigationSplitView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/class.NavigationSplitView.html), [Adw.OverlaySplitView](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.4/class.OverlaySplitView.html) (T1; libadwaita pages are T2) |

Windows is the only platform with a full mode/width/breakpoint model. GNOME gives
a proportionate rule and libadwaita gives a range. Apple gives the role only.

### What the shell has (MEASURED)

Two sidebars, drawn by two functions, sharing nothing:

| | Files places pane | Settings sidebar |
|---|---|---|
| Width | **132 px fixed** | **36 % of the window body** (360 px at 1000, 504 at 1400) |
| Background | `Material::Elevated` | black at 16 % |
| Heading | "Quick access" in `style_label` (12 px, tracked) | **none** |
| Rows | **20 px** tall, 4 px inset, 8 px text, no icon | **30 px** tall, `ui::list_item` with icon, 2 px gap |
| Count | the place list (Home, Desktop, Documents, Downloads, Pictures, Trash by default) | **13 pages** (`Page::ALL`) |
| Selection | `fill text at 8 %` — **one channel**, colour only | `State::Selected`: fill **plus** a 3 px accent marker from `list_item` — two channels |
| Selection rule | `place_index == Some(i)` **or** `place == app.path` | `page == shell.settings.page` |
| Resize / collapse | none | none |
| Scroll | none (fixed list) | none; 13 × 30 + 12 × 2 = **414 px** of rows |
| Keyboard | one Tab stop per place; no Up/Down for the pane (Files' Up/Down move the table selection) | one Tab stop per page; the Settings keys are Ctrl+L/T/A/W/M, none of which moves the category |
| Responsive behaviour | none: 132 px at every window width | none: 36 % at every width, so the sidebar is 180 px at 500 and 504 px at 1400 |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **Two sidebars, two designs, no shared component** | Windows has one NavigationView with modes; GNOME has one sidebar pattern; the shell's own component layer exists to prevent exactly this | Width policy (fixed vs proportional), background, row height (20 vs 30), icons (none vs icon), selection channels (one vs two), headings (one has, one does not). A user learns two different sidebars in two adjacent windows |
| Settings' sidebar is **36 %** with no collapse or resize | Windows: 320 DIP expanded, 48 DIP compact, modes at 1008/641/640; GNOME/libadwaita: 25 % target, 180–280sp range, collapse around 400sp | At 1400 px the Settings sidebar is 504 px — 80 % wider than libadwaita's maximum and 184 px wider than Windows' default. At 500 px it is 180 px, at libadwaita's minimum, with the content squeezed into 320 |
| No compact, minimal or collapsed mode anywhere | Windows has four modes and an Auto policy; GNOME collapses to a navigation view | Neither sidebar changes with width. There is no icon-only strip, no menu button, no overlay |
| Files places rows are **20 px** | Windows/GNOME publish no row height, but the general target is 40 px (Windows) / 44 pt (Apple); Windows' compact pane is 48 DIP wide with icon rows | A 20 px row is below every published target except WCAG's 24 px web minimum, which it also misses. Settings' 30 px rows are larger but still below the touch guidance |
| Selection is colour-only in the places pane | Windows draws a selection indicator on the pane edge; the shell's own `list_item` uses fill **and** a 3 px marker | The Settings sidebar gets the two-channel treatment; the Files pane gets an 8 % fill and nothing else |
| No headings or sections in the Settings sidebar; one heading in Files | Windows: group headers and separators; GNOME: sectioned sidebar; Apple: sections | Thirteen pages in one undifferentiated column is the largest list in the shell with the least structure |
| No sidebar scroll | Windows and GNOME both scroll the pane when it exceeds the window | 414 px of Settings rows with no scroll. At a body height under ~430 px the last pages are drawn outside the pane with no way to reach them |
| No keyboard model for either pane | Windows: Up/Down (Left mode), Left/Right (Top mode), Space/Enter; GNOME: directional arrows where possible | One Tab stop per row and Enter. Files' Up/Down keys move the file table, not the places pane |
| No resize handle | Windows and GNOME both allow the pane width to adapt; Apple's split view allows resizing | Fixed values in both windows |

---

## F.11 Command bars and toolbars

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
summary-only; three toolbars with three different overflow policies).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Use a `CommandBar` for quick access to the app's or page's **most common** commands, ordered by importance. Infrequent or grouped commands go in **SecondaryCommands** (overflow) or a menu. The bar has optional content, primary and secondary commands; primary commands can move into overflow dynamically (`IsDynamicOverflowEnabled`); a **"see more" (…)** button opens the bar and reveals labels, or the overflow menu. States: **Compact** (closed: content, primary icons, `…`), **Minimal** (a thin bar that opens when pressed), **Hidden** (contextual; warned against). Opening overflow defaults above the commands and flips below when there is no room; light-dismiss is expected, and sticky bars are discouraged. Icons: **20 × 20 px** in the primary area, **16 × 16 px** in overflow; app bar buttons carry an icon and a text label, and labels are hidden while compact and shown when open. On the smallest screens (320 epx) at most **four** primary commands fit. Touch targets: at least 40 × 40 epx, or 32 epx tall if at least 120 epx wide; 44 × 44 for touch-optimised UI. Access keys (Alt) and accelerators (Ctrl) are recommended and should be documented in labels or tooltips. No fixed CommandBar height is specified | learn.microsoft.com — [Command bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar), [Menus and context menus](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus), [Menu flyout and menu bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus), [Touch interactions](https://learn.microsoft.com/en-us/windows/apps/develop/input/touch-interactions), [Guidelines for touch targets](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting), [Access keys](https://learn.microsoft.com/en-us/windows/apps/develop/input/access-keys), [Keyboard accelerators](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-accelerators) (T1) |
| macOS 27 | Apple describes a toolbar as convenient access to **frequently used commands, controls, navigation and search**. The summary gives no grouping, priority, overflow, icon-versus-label, placement, customisation, keyboard or size rule; the toolbar-items page returned 404 | developer.apple.com — [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars), [The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar) (T1, `SNIPPET_ONLY`) |
| GNOME 51 | The current HIG has no separate "toolbar" pattern; the equivalent is the **header bar**, which supports dragging and window management and hosts a **small number of relevant controls**. Arrange controls at **start** (main actions such as new, add, open, back), **center** (heading, sometimes a view switcher) and **end** (menus). Use the **backgroundless** button style (icon-only, icon+label, split), generally avoid label-only and suggested/destructive buttons in primary header bars, and group related controls with spacing rather than linked buttons. Keep controls relevant to the current view and leave draggable blank space. The primary menu holds app-wide actions, the secondary menu view-specific ones (3–12 items, 3–6 in a submenu, avoid nesting). All controls in a primary header bar should have **tooltips**; the primary menu's label is "Main Menu", and tooltips must not carry essential information. **No standard height, padding, gap, button size or touch-target number**; the 48dp figure often quoted is not in the HIG. Keyboard: F10 opens menus, Menu/Shift+F10 the context menu, every action keyboard-operable | developer.gnome.org — [Header Bars](https://developer.gnome.org/hig/patterns/containers/header-bars.html), [Menus](https://developer.gnome.org/hig/patterns/controls/menus.html), [Buttons](https://developer.gnome.org/hig/patterns/controls/buttons.html), [Tooltips](https://developer.gnome.org/hig/patterns/feedback/tooltips.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Pointer & Touch](https://developer.gnome.org/hig/guidelines/pointer-touch.html); GTK — [Gtk.HeaderBar](https://docs.gtk.org/gtk4/class.HeaderBar.html) (T1; the GTK page is T2) |

All three put frequent actions in the bar and infrequent ones behind a menu or
overflow. Windows gives the overflow model and the numbers; GNOME gives the
icon-first, tooltip-bearing header-bar style; Apple gives the purpose only.

### What the shell has (MEASURED)

Three toolbars, three policies:

| | Browser command row | Files command row | Trash toolbar |
|---|---|---|---|
| Items | 9 text buttons: Back, Forward, Reload, Stop, Save/Saved, History, Bookmarks, Downloads, Site | New folder, Paste, Compress, Extract, and a Details toggle at the right | search field + Empty / Restore / Delete, or confirm + Cancel |
| Size | 24 px tall; width = label + 18 px; 4 px gap | 22 px tall; width = label + 12 px; 4 px gap | full 30 px row; fixed widths 76/88/112/70 |
| Overflow | **wrap**: one row if the measured total fits, otherwise two rows of 5 and 4 (`div_ceil(2)`), controls area 56 px; if a row is still too wide, every button in it is **scaled** by `available / row_width` and its label is elided against the scaled width | **drop**: a button that would reach the Details toggle is **not drawn at all**, and the rest follow because the cursor does not advance | none: the layout reserves a fixed width for the actions and gives the rest to the search field |
| Disabled state | real: Back/Forward disabled when there is no history, Stop/Reload when they would act on nothing | **none**: every button is `State::Normal`, including Paste with an empty clipboard and Compress/Extract with no selection | **none**: Empty is enabled with an empty Trash, Restore/Delete depend on the selection |
| Selected state | real: the open panel's button (History/Bookmarks/Downloads/Site) and Save/Saved | Details toggle carries a ✕ in its label when open; no selected fill | none |
| Icons / labels | labels only, no icons, no tooltips, no access keys, no separators between navigation, page and panel groups | labels only; the ✕ is a text glyph in the Details label | labels only; destructive actions use `Tone::Bad` |
| Labelling | "Site" is the only ambiguous label; "Save/Saved" doubles as a state | "New folder" is the one two-word label | "Empty permanently?" is 18 characters on a fixed 112 px button, and `ui::button` does not elide |
| Keyboard | each button is a Tab stop; Enter activates; no accelerators | same, but the buttons that are not drawn are also not in the tab order | same |

### Deltas

| Delta | Against | Direction and size |
|---|---|---|
| **Three overflow policies, none of them an overflow menu** | Windows: primary/secondary commands with dynamic overflow and a `…` button; GNOME: put the excess in the primary or secondary menu | Browser wraps and then shrinks labels; Files silently drops controls; Trash assumes they fit. There is no surface on which an overflowed command remains discoverable |
| **Files removes buttons without telling the user** | Windows keeps commands reachable in overflow; GNOME routes them to a menu | In a narrow Files window, Paste and Compress simply are not on screen. The comment explains the collision it avoids and notes the context menu and shortcuts still work, which is a mitigation, not an affordance |
| **Buttons look enabled when the action cannot succeed** | GNOME: invalid buttons should be insensitive; Windows: disabled commands leave the tab order and are not actionable | Paste with an empty clipboard, Compress/Extract with no selection, Empty with an empty Trash: all drawn as live buttons. The shell *does* implement disabled states (the browser uses them), so this is a per-surface omission, not a missing capability |
| Buttons are **22–24 px** tall | Windows: 40 × 40 epx general target, 44 × 44 for touch; Apple: 44 pt; GNOME: qualitative | The browser's 24 px buttons and Files' 22 px buttons are half the Windows target. The whole toolbar is shorter than one target |
| **No icons anywhere in a toolbar** | Windows: icon + label, 20 px primary / 16 px overflow; GNOME: backgroundless icon buttons in header bars; Apple: toolbar items are typically iconic | Nine words in a row at 13 px is the widest possible toolbar. The shell has an icon set (`icons.rs`) and uses it in the sidebars and the omnibox, so this is a toolbar decision, not a missing asset |
| No tooltips, access keys or accelerators | Windows: access keys and accelerators documented in labels/tooltips; GNOME: tooltips on every primary header-bar control, F10 for menus | The only way to learn "Site" is to click it. There is no Alt mnemonic and no shortcut list for the browser's commands |
| No separators or grouping | Windows: primary/secondary and grouping; GNOME: start/center/end zones and spacing groups | The browser's nine commands are one flat sequence: navigation (Back/Forward/Reload/Stop), page state (Save), and panels (History/Bookmarks/Downloads/Site) run together |
| Browser overflow splits by **half the item count** | Windows: order by importance so the most important stay visible; GNOME: a small number of relevant controls | The split is `div_ceil(len/2)`, not priority: Back, Forward, Reload, Stop and Save stay on row one while History, Bookmarks, Downloads and Site move to row two. That ordering happens to be sensible, but it is an accident of list order, not a rule |
| Scaling shrinks the target, not just the label | Windows drops the label (compact) and keeps the 20/16 px icon; GNOME keeps icon buttons | A scaled 24 px button can become narrower than its own elided label, and the target shrinks with it. A pointer user gets a smaller target precisely when the window is small |
| Fixed-width destructive label | Any platform: a label belongs inside its control | "Empty permanently?" on a 112 px button at 13 px type is wider than the button; `ui::button` centres it and lets it overhang. The same pattern repeats on "Delete forever?" |
| Selected state is a tint only | Windows: selection indicator; GNOME: backgroundless selected style; the shell's own contract | `State::Selected` on a browser toolbar button is an accent fill at 22 %, with no glyph or edge |
| Disabled states exist where they matter | — | The browser disables Back/Forward/Stop/Reload correctly, and does not register a hotspot for a disabled button. This is the best part of the three toolbars |

---

## 5. Findings this part files without deciding

Continuing Parts 10–12's numbering. None of these is a ledger row: a row needs a
decision with a backend and a justification that a research note does not have.

| # | Finding | Where it is visible | Owner |
|---|---|---|---|
| 38 | The tab strip has no overflow: fixed 150 px tabs, no scroll, no cap, no clip; the + button can leave the window | `views.rs` lines 1575–1621 | `F.08` decision |
| 39 | No tab keyboard model: two Tab stops per tab, no Ctrl+Tab/1–9/Page keys, no arrow movement; Ctrl+T is Settings' theme cycle | `shell.rs` lines 4691–4720, 4699; `views.rs` lines 1608, 1618 | `F.08`, with `G.14` |
| 40 | The active tab is a material change only: same text, weight and size, no accent edge | `views.rs` lines 1584–1592 | `F.08`, with `G.07` |
| 41 | The tab context menu has two items; no duplicate, pin, mute, move-to-window, reorder or tear-out | `apps.rs` lines 1738–1741 | `F.08` / `F.13` |
| 42 | Two sidebars, two designs: 132 px/20 px rows/fill-only vs 36 %/30 px rows/icon+marker | `views.rs` lines 957, 2337 | `F.10` decision |
| 43 | The Settings sidebar is 36 % with no collapse or resize, and no scroll for 414 px of rows | `views.rs` lines 2337–2360 | `F.10` |
| 44 | No arrow-key model for either sidebar; one Tab stop per row | `shell.rs` line 445; `views.rs` lines 980, 2341 | `F.10` / `G.14` |
| 45 | Three toolbars, three overflow policies: wrap+scale, silent drop, fixed widths | `views.rs` lines 1702–1723, 1083, 4172 | `F.11` decision |
| 46 | Files drops buttons that do not fit, and draws inapplicable actions as enabled | `views.rs` lines 1083, 1102–1136 | `F.11` / `F.26` |
| 47 | Toolbars are label-only, 22–24 px tall, with no tooltips, access keys or separators | `views.rs` lines 1676–1755, 1032–1136 | `F.11` |
| 48 | Browser overflow splits by half the item count and shrinks target width | `views.rs` lines 1702, 1723 | `F.11` |
| 49 | Fixed-width confirm labels ("Empty permanently?") overhang their buttons | `views.rs` lines 4189–4200; `ui::button` | `F.11` / `F.27` |

## 6. What this part does not establish

| Claim | Why not |
|---|---|
| that tabs actually run off the screen | the loop was read, not a resized window. The claim is that nothing in the code bounds the strip; the visual result follows from the window clip |
| that a sidebar should be 320 px | Windows' 320 DIP is a default, not a rule; GNOME's 180–280sp is a libadwaita default, not HIG. The finding is that the shell has no policy at all, in either direction |
| that the toolbar should be icon-first | GNOME's header bar prefers backgroundless icons; Windows keeps icon + label and hides the label when compact. The finding is that the shell has no overflow, not that labels are wrong |
| that macOS agrees with any of this | every Apple page returned a title and a summary. No macOS tab, sidebar or toolbar rule is reported as verified |
| that the tab or sidebar behaviour was keyboard-tested | no keyboard was used. The claims are readings of `app_key` and the hotspot table |
| that `F` is nearly done | 14 of 27 items remain `PENDING`, thirteen are `PARTIAL`, none is `COMPLETE` |

## 7. Coverage added, and what would finish these three items

| Item | Before | After | What is still missing |
|---|---|---|---|
| `F.08` Tabs | `PENDING` | `PARTIAL` | macOS at any depth; a decision on overflow (scroll, compress, or a tab list), a decision on the keyboard model, and a decision on active-tab emphasis |
| `F.10` Sidebars | `PENDING` | `PARTIAL` | macOS at any depth; one sidebar component with a width and collapse policy, a row-height decision, and an arrow-key model shared by both windows |
| `F.11` Command bars/toolbars | `PENDING` | `PARTIAL` | macOS at any depth; one toolbar component with an overflow menu, a disabled-state rule per action, and a decision on icons versus labels |

What would move each to `COMPLETE`: Apple's pages read in full, one decision per
item with a backend and a milestone, and — for all three — a layout test at two
window widths, because the current overflow behaviours are exactly the kind of
thing that only a resized frame shows.

## Sources read for this part

Access date for all rows: 2026-10-26 (Asia/Bangkok). Tier definitions: `Part1` §2.1.

| Source | Tier | Status | Limitations |
|---|---|---|---|
| [Tab View — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/tab-view) | T1 | INSPECTED | No fixed width; width modes instead |
| [NavigationView — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/navigationview) | T1 | INSPECTED | Partial extraction; modes, widths and keyboard sections present |
| [Navigation design basics — Windows](https://learn.microsoft.com/en-us/windows/apps/design/basics/navigation-basics) | T1 | INSPECTED | Five-item and two-level rules |
| [Command bar — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar) | T1 | INSPECTED | No fixed height |
| [Menus and context menus — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus-and-context-menus) | T1 | INSPECTED | — |
| [Menu flyout and menu bar — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/menus) | T1 | INSPECTED | — |
| [Keyboard interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions) | T1 | INSPECTED | Partial extraction |
| [Access keys — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/access-keys) | T1 | INSPECTED | — |
| [Keyboard accelerators — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-accelerators) | T1 | INSPECTED | Partial extraction |
| [Touch interactions — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/touch-interactions) | T1 | INSPECTED | General targets |
| [Guidelines for touch targets — Windows](https://learn.microsoft.com/en-us/windows/apps/develop/input/guidelines-for-targeting) | T1 | INSPECTED | Density-dependent |
| [NavigationView.OpenPaneLength — Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.navigationview.openpanelength?view=windows-app-sdk-2.0) | T1 | INSPECTED | Default value only |
| [NavigationView.CompactPaneLength — Windows App SDK](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.navigationview.compactpanelength?view=windows-app-sdk-2.0) | T1 | INSPECTED | Default value only |
| [Tab views — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/tab-views) | T1 | SNIPPET_ONLY | Title and summary only |
| [Tab bars — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/tab-bars) | T1 | SNIPPET_ONLY | as above |
| [Segmented controls — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/segmented-controls) | T1 | SNIPPET_ONLY | "Two or more segments" only |
| [Sidebars — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/sidebars) | T1 | SNIPPET_ONLY | as above |
| [Toolbars — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/toolbars) | T1 | SNIPPET_ONLY | as above |
| [Windows — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/windows) | T1 | SNIPPET_ONLY | as above |
| [Layout — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/layout) | T1 | SNIPPET_ONLY | as above |
| [The menu bar — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar) | T1 | SNIPPET_ONLY | as above |
| [NavigationSplitView — SwiftUI](https://developer.apple.com/documentation/swiftui/navigationsplitview) | T1 | SNIPPET_ONLY | API summary only |
| [Tabs — GNOME HIG](https://developer.gnome.org/hig/patterns/nav/tabs.html) | T1 | INSPECTED | No tab width; no overflow algorithm; close-button policy not prescribed |
| [Sidebars — GNOME HIG](https://developer.gnome.org/hig/patterns/nav/sidebars.html) | T1 | INSPECTED | No width, row height or selection styling |
| [Header Bars — GNOME HIG](https://developer.gnome.org/hig/patterns/containers/header-bars.html) | T1 | INSPECTED | No height, padding or target number |
| [Navigation — GNOME HIG](https://developer.gnome.org/hig/guidelines/navigation.html) | T1 | INSPECTED | — |
| [View Switchers — GNOME HIG](https://developer.gnome.org/hig/patterns/nav/view-switchers.html) | T1 | INSPECTED | 3–5 views rule of thumb |
| [Menus — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/menus.html) | T1 | INSPECTED | 3–12 items, 3–6 per submenu |
| [Tooltips — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/tooltips.html) | T1 | INSPECTED | Tooltips must not carry essential information |
| [Scaling & Adaptiveness — GNOME HIG](https://developer.gnome.org/hig/guidelines/adaptive.html) | T1 | INSPECTED | 1024 × 600 desktop target |
| [Keyboard — GNOME HIG](https://developer.gnome.org/hig/guidelines/keyboard.html) | T1 | INSPECTED | F10, arrows, Ctrl+Tab family |
| [Pointer & Touch — GNOME HIG](https://developer.gnome.org/hig/guidelines/pointer-touch.html) | T1 | INSPECTED | Qualitative; no 48dp figure |
| [Adw.TabBar — libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/class.TabBar.html) | T2 (implementation detail) | INSPECTED | Widget behaviour, not HIG policy |
| [Adw.TabView — libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.7/class.TabView.html) | T2 (implementation detail) | INSPECTED | as above |
| [Adw.NavigationSplitView — libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.9/class.NavigationSplitView.html) | T2 (implementation detail) | INSPECTED | 25 % / 180–280sp defaults; 400sp example breakpoint |
| [Adw.OverlaySplitView — libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.4/class.OverlaySplitView.html) | T2 (implementation detail) | INSPECTED | Same defaults |
| [Gtk.HeaderBar — GTK 4](https://docs.gtk.org/gtk4/class.HeaderBar.html) | T2 (implementation detail) | INSPECTED | Widget API only |
| [Reinventing tabs — GNOME developer blog (2021)](https://blogs.gnome.org/alicem/2021/03/13/reinventing-tabs/) | T4 (historical) | INSPECTED | Not current HIG; used only as history for close-button placement |

Repository sources read (no tier): `src/litric-shell/src/ui.rs`,
`src/litric-shell/src/views.rs`, `src/litric-shell/src/apps.rs`,
`src/litric-shell/src/shell.rs`, `src/litric-shell/src/settings.rs`,
`src/litric-shell/src/e2e.rs`, `src/litric-design/src/lib.rs`.
