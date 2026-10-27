# LitricOS UI/UX Research — Part 15 of N

Chapter 40 (F) — Dialogs and confirmation flows: ten dialogs, three
implementations, and a confirmation the keyboard cannot answer

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P15 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-27 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.15` dialogs and `F.27` confirmation flows. The other 10 `F` items are untouched and keep their status |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", continued on the pair where one is the mechanism and the other is the decision |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (2 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (15 sources appended) |
| Predecessors | Parts 10–14; and `docs/bugs/BUG-0023`, `BUG-0024`, `BUG-0025` — the three repair records that came out of Part 14 and made a dialog with no button visible |
| Resume point | `F.14` tooltips and `F.17` notifications/banners/toasts — the feedback surfaces that remain `PENDING` and have code behind them |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these two

### 0.1 One mechanism, ten presentations, three implementations

A dialog is the only surface in this shell that is allowed to interrupt. It is
also the surface with the least code behind it: there is no `ui::dialog`, no
dialog model and no shared action vocabulary. There are three implementations and
two button metrics.

| Surface | Kinds | Drawn by | Panel | Buttons | How it ends |
|---|---|---|---|---|---|
| **Files dialogs** | 5: new folder, new text file, rename, confirm, error | `draw_files_dialog_body`, `views.rs` line 1378 | **360** wide, **160** tall with a field / **150** without | **76 × 26**, gap 8 | removed in one frame |
| **Desktop dialogs** | 3: new folder, power confirm, error | `draw_desktop_dialog_panel`, `views.rs` line 256 | **360** wide (≤ 90 % screen), **150** with a field / **170** without | **100 × 28**, gap 8 | spring out; the leaving panel is kept as a snapshot |
| **Trash confirmation** | 2 states: empty, purge one | the Trash toolbar, `views.rs` lines 4194–4224 | none — the toolbar's buttons are replaced | **112 + 70**, toolbar height | the buttons swap back |

The Files dialog's panel is centred on the *window*; the desktop's is centred on
the *screen*; the Trash confirmation is not a panel at all. Nothing is shared but
the scrim material and `Radius::Lg`.

### 0.2 The numbers these two are made of (MEASURED)

| Measure | Value | Where |
|---|---|---|
| Dialog presentations | **10** (5 Files kinds + 3 desktop kinds + 2 Trash states) | `apps.rs` lines 166, 201; `views.rs` lines 1378, 256, 4194 |
| Dialog implementations | **3** | `views.rs` lines 1378, 256, 4194 |
| Dialog strings | **23** distinct literals across titles, details and buttons | `views.rs` lines 1378–1478; `shell.rs` lines 731–745 |
| Dialog strings through the dictionary | **0 of 23**, though **4** are already translated there (`New folder`, `Create`, `Confirm`, `Cancel`) | `views.rs`; `i18n.rs` lines 130, 143–145 |
| Files panel | **360 × 160** with a field, **360 × 150** without | `views.rs` lines 1409–1411 |
| Files buttons | **76 × 26**; affirmative at `right − 88`, cancel 8 px to its left | `views.rs` lines 1439–1452 |
| Desktop panel | **360 × 150** with a field, **360 × 170** without | `views.rs` lines 311–316 |
| Desktop buttons | **100 × 28**; affirmative at `right − 112`, cancel 8 px to its left | `views.rs` lines 360–371 |
| Affirmative label | one per kind, never per action: `Create` / `Confirm` / `OK` / `Close` / the power verb | `views.rs` lines 1437–1440, 281–309 |
| Confirmation call sites | **11** (9 in `shell.rs`, 2 in `apps.rs`), **7** distinct prompts | `shell.rs` lines 2376, 2387, 2593, 4896, 4910, 4924, 4950, 6205, 6221; `apps.rs` lines 1544, 1566 |
| Confirmation titles naming the object | **3 of 7** (`Move /path to the trash?`, `Extract /path…`, `Compress … as name.zip`) | `shell.rs` line 2389; `apps.rs` lines 1546, 1571 |
| Undo actions in the shell | **0**; `notify` takes source, title, body, priority and TTL, no action | `shell.rs` line 6316 |
| Enter as the default action | **0 of 2** dialog implementations | probe, `dialog-probe.log` |
| Escape dismissing | **1 of 2** — Files only | `shell.rs` line 4867; probe |
| Pointer modality | **0 of 2** — the Files scrim registers no hotspot; the desktop's panel *is* its cancel target | `views.rs` lines 1378–1452, 331; probe |

The four keyboard and pointer rows are not readings of the code: a temporary test
drove the real input path and printed the four results below, then was removed.
The log is `docs/evidence/2026-10-27/dialog-probe.log`:

```
PROBE files-enter: dialog=Some(NewFolder) path=/Documents/Pictures
PROBE desktop-open: dialog=Some(NewFolder)
PROBE desktop-escape: dialog=Some(NewFolder)
PROBE desktop-enter: dialog=Some(NewFolder)
PROBE files-click-behind: selection=Some(1) dialog=Some(NewFolder)
PROBE desktop-click-panel: dialog=None
PROBE desktop-click-outside: dialog=Some(NewFolder)
```

### 0.3 What the shell's own specification says

`docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` §8 is a specification for a dialog
component that does not exist:

| Spec requirement | Measured here |
|---|---|
| Ten dialog types, all drawn by **one** helper `dialog(kind, title, body, actions)` | 10 presentations, **3** helpers, no `dialog()` |
| Destructive button in the danger colour, object named in the text ("ลบ 'รายงาน.pdf' ถาวร?") | danger tone yes; object named in 3 of 7 prompts |
| Cancel always the secondary, Esc cancels, **no timeout** | cancel always present (except errors); Esc only in Files; no timeout — correct |
| Focus trap, primary receives Enter | no trap, no default button; Enter does something else (below) |

`03-SHELL-SYSTEM.md` §6 adds the other half: a destructive action in a
notification must offer **undo for 5 s**. There is no undo mechanism to offer.

So the two items are measured against the platforms *and* against a contract this
shell wrote for itself.

### 0.4 The four questions a dialog has to answer

Both items reduce to the same four, and the shell answers them differently
depending on which window the dialog belongs to:

1. **Who owns the keyboard while it is open?** (Enter, Escape, Tab)
2. **Who owns the pointer while it is open?** (a scrim that blocks, or one that
   only paints)
3. **What does the affirmative button say, and what happens when it is pressed
   by accident?** (label, default, consequence)
4. **Can the action be taken back?** (undo, trash, confirmation only)

### 0.5 Evidence limits of this part

| Limit | Effect |
|---|---|
| Static reading plus one probe; no booted image | The probe drives the shell's own input path; nothing here was seen on a screen |
| Apple's HIG body did not load; full apple text came from a mirror | macOS rules are quoted from a secondary copy and marked as such; `NSAlert` keyboard behaviour came from Apple's own documentation JSON |
| No accessibility tree exists to inspect | Role, name and announcement requirements are reported as absent, not as failures of a role implementation |
| No user test | Whether a confirmation is read before it is clicked is not measured |
| No long-string test | The title is not elided (only the detail line is), but no title longer than the panel was driven |

---

## F.15 Dialogs

Status after this part: **PARTIAL** (Windows and GNOME read at page depth; macOS
from a mirror plus Apple's `NSAlert` reference; ten presentations, three
implementations, and a keyboard that reaches neither dialog's affirmative button).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 (WinUI 3 / Windows App SDK) | A dialog **blocks** interaction and should be reserved for "situations where you want the user to drop everything"; a flyout is for attention that may be ignored. Dialogs are for the cases that matter — "When the user is about to permanently alter a valuable asset", "When the user is about to delete a valuable asset". Validation that belongs to a field stays inline. Buttons: **the "do it" action is the leftmost; the safe, nondestructive action is the rightmost**, and `CloseButton` is required. A default button is optional, receives Enter and the accent treatment; **if no default is set, none responds to Enter**. Escape (and system Back, and gamepad B) closes via the safe action. The title is the main instruction and is optional; the body is required and must not repeat the title. No dialog width, height or type size is published, and **no destructive-button colour rule** is published. The Fluent 2 React page (a different component) adds the rules Learn does not: alert dialogs only for potential loss, titles mandatory, focus moves into the dialogs and **traps**, and on close returns to the trigger | learn.microsoft.com — [Dialogs and flyouts](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/), [Dialog controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs), [Flyout controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/flyouts); fluent2.microsoft.design — [React Dialog](https://fluent2.microsoft.design/components/web/react/core/dialog/usage) (T1; Learn pages read in full; the Fluent page is React guidance, not WinUI, and is marked as such) |
| macOS 27 | "An alert gives people critical information they need right away", and alerts should be used **sparingly**. An alert displays "a title, optional informative text, and up to three buttons"; titles should not wrap past two lines; the informative text explains consequences. **Default button trailing, Cancel leading**; `NSAlert` puts the default at index 0, which is the **rightmost** button for a left-to-right language, and Return invokes it; a button titled **Cancel** gets Escape. Avoid **OK** except in informational alerts; use specific labels ("Delete", "Erase"). **Avoid alerts for common, undoable actions, even when they are destructive**; alert when the action is uncommon and **cannot be undone**. Modality is "a separate, dedicated mode that prevents interaction with the parent view and requires an explicit action to dismiss". No alert size, margin or type size is published | developer.apple.com — [Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts), [Action sheets](https://developer.apple.com/design/human-interface-guidelines/action-sheets), [Modality](https://developer.apple.com/design/human-interface-guidelines/modality), [NSAlert.buttons](https://developer.apple.com/documentation/appkit/nsalert/buttons) (T1 for the AppKit page; the HIG bodies returned summaries, so the quoted alert text is from a [mirror of the HIG content](https://github.com/JHKits/Apple_HIG_Korean/blob/main/translated-markdown/components/presentation/alerts.md), marked secondary) |
| GNOME 51 (GTK 4 / Libadwaita) | Two kinds: **alert dialogs** ("present a message or question, along with between one and three buttons") and **action dialogs** (options about an action before it happens). "Confirmation dialogs are a standard type of alert dialog… They have two buttons: one to confirm… and one to cancel." "Never pop up a dialog window unexpectedly"; prefer inline controls. **Cancel appears first, before the affirmative** (left in LTR). Return activates the affirmative **unless its action is "irreversible, destructive or otherwise inconvenient"**, in which case no default is set. **Esc activates cancel.** "Dialogs should always have a parent window, to which they are modal"; give initial focus to the component the user will operate first. Destructive actions need a confirmation **or** an undo, and "undo is typically a better option than a confirmation dialog" because people click through prompts. Destructive buttons use a strong colour and each view should have **at most one** suggested or destructive button. No dialog width or padding is published; `AdwDialog` cannot exceed its parent window | developer.gnome.org — [Dialogs](https://developer.gnome.org/hig/patterns/feedback/dialogs.html), [Keyboard](https://developer.gnome.org/hig/guidelines/keyboard.html), [Buttons](https://developer.gnome.org/hig/patterns/controls/buttons.html), [Toasts](https://developer.gnome.org/hig/patterns/feedback/toasts.html), [Writing Style](https://developer.gnome.org/hig/guidelines/writing-style.html); gnome.pages.gitlab.gnome.org — [Adw.Dialog](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.6/class.Dialog.html) (T1; HIG pages read in full) |

The three platforms agree on the contract and disagree on two details. They agree
that a dialog is for consequential interruption, that the safe action must always
exist, that the affirmative carries the action's own verb, and that Escape is the
way out. They disagree on **button order** (Windows puts the doing action leftmost
and the safe one rightmost; GNOME puts Cancel first; macOS puts the default last)
and on **whether Return may run a destructive action** (GNOME says no default in
that case; Windows leaves it to the app; macOS makes the default the rightmost
button, which the developer chooses).

### What the shell has (MEASURED)

**The presentations.** Five Files kinds and three desktop kinds, drawn by two
functions, plus the Trash confirmation that is not a dialog at all:

| Kind | Title | Detail | Field | Affirmative | Tone |
|---|---|---|---|---|---|
| Files `NewFolder` | New folder | — | yes, "Name" | Create | Accent |
| Files `NewFile` | New text file | — | yes, "Name" | Create | Accent |
| Files `Rename` | Rename | `from <name>` | yes, pre-filled | Create | Accent |
| Files `Confirm` | the caller's title | the caller's detail | no | **Confirm** | Warning |
| Files `Error` | Could not finish | the error's `Display` | no | OK | Bad |
| Desktop `NewFolder` | New folder | — | yes, "Folder name" | Create | Accent |
| Desktop `PowerConfirm` | `Suspend the machine?` / `Restart the machine?` / `Power the machine off?` | the matching consequence sentence | no | Suspend / Restart / Power off | Accent for suspend, Bad otherwise |
| Desktop `Error` | Could not finish | the error's `Display` | no | Close | Bad |

**What is right, and should survive any refactor.** The dialogs are a scrim and a
popover with `Radius::Lg`; a confirmation is never the default focus (the text
field takes focus in the three naming dialogs); Cancel is present on every dialog
that can do something; there is no timeout on any of them; the desktop's error
dialog and the Files error dialog both have exactly one way out; and the power
confirmation names the consequence in a sentence rather than repeating the
question.

**What is not.** The keyboard, the pointer, the wording, and the fact that the
same role is drawn two ways:

| Defect | Evidence |
|---|---|
| **Enter does not accept a Files dialog.** With New folder open over a listing whose first row is `Pictures`, Enter navigated the window into `/Documents/Pictures` and left the dialog open | probe; `shell.rs` line 4967 (the Files `Key::Enter` arm has no dialog guard) |
| **Desktop dialogs answer no key.** Escape and Enter both leave the new-folder dialog open. Escape is bound to `CloseOverlay`, which closes overlays only | probe; `shell.rs` lines 5971–5978; `shortcuts.rs` line 690 |
| **The Files dialog is not modal to the pointer.** Clicking row 1 behind the scrim selected it while the dialog stayed. The scrim is painted; it registers no hotspot | probe; `views.rs` lines 1385–1390 |
| **The desktop dialog's dismissal is inverted.** Clicking the dialog's own panel cancels it; clicking the wallpaper outside does not | probe; `views.rs` line 331 |
| **One affirmative label per kind, not per action.** Eleven confirmation sites share the word **Confirm** | `views.rs` line 1438; `shell.rs` lines 2376–4950 |
| **No string reaches the dictionary**, including four that are already in it | `views.rs` lines 1378–1478; `i18n.rs` lines 130, 143–145 |
| **The same title role is two styles.** Files: bold `Title3` at y+28; desktop: `style_label()` at y+30 | `views.rs` lines 1418, 333 |
| **The error body is a developer string.** `DialogKind::Error(e.to_string())` and `DesktopDialog::Error(msg)` put `FsError`'s display text under "Could not finish" | `shell.rs` lines 2228, 2586, 2664; `views.rs` line 1405 |

### Deltas

| # | Delta | Platform rule it misses | Where |
|---|---|---|---|
| 1 | Enter runs the action *behind* the dialog instead of the dialog's affirmative | Windows: the default button answers Enter; GNOME: Return activates the affirmative unless destructive; macOS: Return invokes the default | `shell.rs` 4967 |
| 2 | A desktop dialog cannot be answered, cancelled or dismissed from the keyboard | all three: Escape cancels; Enter activates the default | `shell.rs` 5971; probe |
| 3 | A click outside a Files dialog reaches the window it covers | Windows: modal "blocks interactions with the app window"; GNOME: "modal"; macOS: "prevents interaction with the parent view" | `views.rs` 1385; probe |
| 4 | The desktop dialog cancels when its body is clicked and stays when the desktop is clicked | none of the three make the panel itself the cancel target; click-away is either a dismissal or not, and this is both | `views.rs` 331; probe |
| 5 | The affirmative says `Confirm` for every confirmation | "Use specific responses to the main instruction"; "label the affirmative button with a specific imperative verb"; use a specific label such as Delete | `views.rs` 1438 |
| 6 | Dialog text is English only | not a platform rule, but the shell's own bilingual contract (a Thai dictionary with these four words already in it) | `i18n.rs` 130, 143–145 |
| 7 | Two title styles and two button metrics for one role | no platform publishes the numbers; the shell's own §8 asks for one helper | `views.rs` 256, 1378 |
| 8 | Error text is the error type's `Display` | Windows asks for plain language and the user's objective; Apple: specific and useful, not an error code | `views.rs` 1405 |

Status after this part: **PARTIAL**. Windows and GNOME were read at page depth,
macOS through a mirror and Apple's `NSAlert` reference; the shell side is measured
including four probe results. What keeps it from `COMPLETE` is that the item has
no decision: whether the ten types become one component, and whether that
component grows a keyboard model, is a design decision this record does not make.

---

## F.27 Confirmation flows

Status after this part: **PARTIAL** (seven prompts across eleven call sites, all
confirmation and no undo, while every platform says undo is the better answer for
the actions most of those prompts guard).

### Platform guidance

| Platform | Guidance | Source (tier) |
|---|---|---|
| Windows 11 26H2 | Confirm when the user is "about to permanently alter a valuable asset" or "about to delete a valuable asset". For frequent actions: "consider providing a way for the user to undo the action if it was a mistake, rather than forcing users to confirm the action every time." The sample destructive dialog names the object and the consequence: title `Delete file permanently?`, body "If you delete this file, you won't be able to recover it. Do you want to delete it?", buttons `Delete` and `Cancel` | learn.microsoft.com — [Dialogs and flyouts](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/), [Dialog controls](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs) (T1) |
| macOS 27 | **"Avoid displaying alerts for common, undoable actions, even when they're destructive."** Deleting a file is the example. Alert for "an uncommon destructive action that they can't undo". Identify the destructive button visually and always include Cancel; label it with the action ("Delete", "Erase"), not OK. The caution symbol is for unexpected loss, not for emptying the Trash | developer.apple.com — [Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts) (summary body; quoted text from the HIG mirror, secondary) |
| GNOME 51 | "Destructive actions should always be accompanied by either a confirmation dialog or an offer to undo the action (typically using a toast)." **"Since users will often habitually click through confirmation dialogs without fully reading or considering them, undo is typically a better option than a confirmation dialog."** If undo is impossible, confirm — "to alert the user to the risk, to clarify which action will be taken, and to give them the opportunity to cancel." Destructive buttons carry a strong colour; at most one per view; label with the verb, and add an ellipsis when the command needs further input | developer.gnome.org — [Dialogs](https://developer.gnome.org/hig/patterns/feedback/dialogs.html), [Toasts](https://developer.gnome.org/hig/patterns/feedback/toasts.html), [Buttons](https://developer.gnome.org/hig/patterns/controls/buttons.html), [Writing Style](https://developer.gnome.org/hig/guidelines/writing-style.html) (T1) |

All three say the same thing in different words: **a confirmation is a cost, and
undo is the cheaper way to make a destructive action safe.** Windows and GNOME
say it for frequent actions; Apple says it even for destructive ones that can be
undone; GNOME names the failure mode — people click through prompts.

### What the shell has (MEASURED)

Eleven call sites, seven distinct prompts. The table is the whole confirmation
surface of the shell as it stands:

| Prompt (as drawn) | Sites | Action | Reversible after the fact? |
|---|---|---|---|
| `Paste` / `Move <path> into <dir>?` / `Copy <path> into <dir>?` | 3 — menu Paste, Ctrl+V, `FilesPaste` | copy or move one entry | yes — the copy or move can be undone by the opposite action; the source is not destroyed by a copy |
| `Move to trash` — `Move <path> to the trash?` / `Move N selected item(s) to the trash?` | 3 — row menu, Delete key, `FilesTrash` | move to Trash | **yes, built in**: Trash → Restore |
| `Delete permanently` — `Delete N selected item(s) permanently?` | 2 — Delete with Shift/Ctrl | delete for ever | no |
| `Extract` — `Extract <path> into a folder beside it?` | 2 — open a zip, Extract row | unpack an archive | yes — the output folder can be deleted |
| `Compress` — `Archive N selected items as <name>.zip?` | 1 — Compress row | create an archive | yes — the archive can be deleted |
| `Empty permanently?` (Trash toolbar) | 1 state | empty the Trash | no |
| `Delete forever?` (Trash toolbar) | 1 state | purge one entry | no |

Outside `ask_confirm` there are three more destructive confirmations, all in the
power dialog: suspend (reversible), restart and power off (not).

**What is right.** Every irreversible action does ask first: permanent delete,
empty Trash, purge, restart, power off. The single-row Trash and Extract prompts
name the object. Dialogs cannot be dismissed by clicking the scrim by accident
(the Files scrim ignores outside clicks entirely — a defect in the other
direction, but it means a confirmation cannot be *performed* by a stray click).
There is no timeout on any confirmation, so none can resolve itself. The Trash is
a real recovery path and the prompt says "move to the trash", not "delete".

**What is not.** The shell pays the cost of confirmation **and** the cost of no
undo:

| Defect | Evidence |
|---|---|
| **Routine, reversible work is confirmed on every invocation.** Copy, paste, move, extract and compress are 6 of the 11 call sites and 5 of the 7 prompts; none of them destroys anything, and all can be undone by deleting the result | `shell.rs` 2376, 4950, 6205; `apps.rs` 1544, 1566 |
| **The irreversible prompts do not name what they will destroy.** `Delete N selected item(s) permanently?` gives a count; `Delete forever?` and `Empty permanently?` give nothing at all | `shell.rs` 4897; `views.rs` 4197–4198 |
| **The Trash confirmation is not a dialog.** It replaces the toolbar's buttons, is not modal, takes no focus, and disappears if the toolbar redraws for another reason; its label is a question on a button ("Delete forever?") rather than an action with an object | `views.rs` 4194–4224 |
| **No undo exists anywhere.** `notify` posts a source, title, body, priority and TTL — no action, no callback; nothing keeps a reversible handle to a completed operation | `shell.rs` 6316 |
| **The confirmation is the only safety net for permanent deletion**, which is the one action that cannot be recovered from Trash | `shell.rs` 4896; `apps.rs` 700 |
| **Labels are generic**: `Confirm` for all seven, so the prompt's verb is in the title and the button says nothing | `views.rs` 1438 |

### Deltas

| # | Delta | Platform rule it misses | Where |
|---|---|---|---|
| 9 | Copy, paste, move, extract and compress ask every time | Windows: undo rather than confirm frequent actions; GNOME: undo is typically better; Apple: do not alert for common undoable actions | `shell.rs` 2376–6205; `apps.rs` 1544–1566 |
| 10 | Permanent delete and Trash purge do not name the items | Windows' sample names the file; Apple: specific, useful information; GNOME: clarify which action will be taken | `shell.rs` 4897; `views.rs` 4197 |
| 11 | The Trash confirmation is inline and transient, not a dialog | GNOME: destructive actions get a confirmation dialog or an undo; Apple: identify the destructive button and include Cancel (here Cancel exists, but the confirmation is a toolbar state) | `views.rs` 4194 |
| 12 | No undo, so every reversible action still pays a prompt | all three prefer undo; the shell's own §6 asks for a 5 s undo in notifications | `shell.rs` 6316 |
| 13 | The destructive verb is on the panel, not the button | "Use specific responses"; "specific label"; "specific imperative verb" | `views.rs` 1438 |

Status after this part: **PARTIAL**. The prompt surface is fully measured and
compared; what keeps it from `COMPLETE` is the decision the platforms are really
asking for — **which actions get a prompt, which get an undo, and where the undo
lives** — and that decision needs a backend the shell does not have (a
notification action or an operation handle).

---

## 5. Findings this part files without deciding

Numbering continues from Part 14 (finding 69). Every row is a measurement a
reader can open and check; none is a decision.

| # | Finding | Evidence | Item |
|---|---|---|---|
| 70 | Ten dialog presentations across three implementations; no `dialog()` helper; two panel metrics (76 × 26 against 100 × 28 buttons) and two exit behaviours (instant against spring) | `views.rs` lines 256, 1378, 4194 | `F.15` |
| 71 | **Enter does not accept a Files dialog** — with New folder open it navigated the window into the selected folder (`/Documents/Pictures`) and left the dialog open | probe; `shell.rs` line 4967 | `F.15` |
| 72 | **Desktop dialogs answer no key**: Escape and Enter leave the dialog open; Escape is bound to `CloseOverlay`, which closes overlays only | probe; `shell.rs` line 5971 | `F.15` |
| 73 | **The Files dialog is not modal to the pointer**: a click on row 1 behind the scrim selected it while the dialog stayed | probe; `views.rs` line 1385 | `F.15` |
| 74 | **The desktop dialog cancels when its own panel is clicked and not when the desktop outside it is** — the panel is registered as the cancel target | probe; `views.rs` line 331 | `F.15` |
| 75 | One affirmative label per dialog *kind*, not per action: 11 confirmation sites share `Confirm`; there is no default-button concept at all | `views.rs` line 1438 | `F.15` / `F.27` |
| 76 | 23 distinct dialog strings, **0** through the dictionary, including the 4 the dictionary already translates | `views.rs` 1378–1478; `i18n.rs` 130, 143–145 | `F.15` |
| 77 | Permanent delete names a count and Trash's purge names nothing; 3 of 7 prompts name the object | `shell.rs` 4897; `views.rs` 4197 | `F.27` |
| 78 | 11 confirmation call sites, 7 prompts, and **5 of the 7 guard routine work** (paste/copy/move, extract, compress) that all three platforms say should not need a prompt | `shell.rs` 2376–6205; `apps.rs` 1544–1566 | `F.27` |
| 79 | **No undo exists**: `notify` carries no action, so the only recovery is Trash restore — itself behind a confirmation | `shell.rs` 6316 | `F.27` |
| 80 | No focus trap and no focus restore: Tab walks the global hotspot ring, and closing a dialog leaves keyboard focus where it was | `shell.rs` `move_keyboard_focus`; probe | `F.15` |
| 81 | Error dialogs show the error type's `Display` under "Could not finish", with no cause and no next step | `views.rs` 1405; `shell.rs` 2228 | `F.15` |
| 82 | The title role is drawn two ways: bold `Title3` at y+28 (Files) against `style_label()` at y+30 (desktop) | `views.rs` 1418, 333 | `F.15` |

## 6. What this part does not establish

| Claim | Why not |
|---|---|
| that the dialogs look right | no image was booted and no frame was captured. Every geometry number is read from the drawing code, and the probe prints state, not pixels |
| that a user reads a confirmation before clicking it | no user was asked; the finding is that the prompt exists, not that it works |
| that the macOS alert rules are current | Apple's HIG body did not load; the quoted text is a mirror of the HIG content, and the `NSAlert` button and Escape behaviour is Apple's own documentation. The two are labelled separately |
| that a long title overflows | the title is not elided and the panel is fixed at 360, but no title longer than the panel was driven |
| that the dialog is unreachable by assistive technology | there is no accessibility tree in this shell to inspect; the absence of roles and announcements is described, not measured against a screen reader |
| that `F` is nearly done | 9 of 27 items remain `PENDING`; 18 are `PARTIAL`; none is `COMPLETE` |

## 7. Coverage added, and what would finish these two items

| Item | Before | After | What is still missing |
|---|---|---|---|
| `F.15` Dialogs | `PENDING` | `PARTIAL` | macOS at body depth; a decision on one dialog component and its ten types; a keyboard model (Enter, Escape, focus trap, focus restore); pointer modality; one title style and one button metric |
| `F.27` Confirmation flows | `PENDING` | `PARTIAL` | a decision on confirm-versus-undo per action; an undo backend (notification action or operation handle); object naming in the destructive prompts; a home for the Trash confirmation |

What would move each to `COMPLETE`: Apple's pages read in full, one decision per
item with a backend and a milestone, and — for `F.15` — the dialog component the
shell's own §8 already specifies.

## Sources read for this part

| Source | What it was used for | Tier |
|---|---|---|
| [Dialogs and flyouts — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/) | dialog vs flyout; when to confirm; undo for frequent actions | T1, full text |
| [Dialog controls — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs) | button order, default button, Enter and Escape, title/body rules, the delete example | T1, full text |
| [Flyout controls — Windows apps](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/flyouts) | light dismiss, for contrast with dialogs | T1, full text |
| [React Dialog — Fluent 2](https://fluent2.microsoft.design/components/web/react/core/dialog/usage) | the rules Learn does not publish: title mandatory, focus trap, focus restore, 24 px footer padding | T1, full text — but React guidance, marked as not WinUI |
| [Dialogs — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/dialogs.html) | two kinds, button order, Return and Escape, initial focus, destructive guidance, undo preferred | T1, full text |
| [Keyboard — GNOME HIG](https://developer.gnome.org/hig/guidelines/keyboard.html) | Escape closes a transient container | T1, full text |
| [Buttons — GNOME HIG](https://developer.gnome.org/hig/patterns/controls/buttons.html) | destructive and suggested styles; one strong button per view | T1, full text |
| [Toasts — GNOME HIG](https://developer.gnome.org/hig/patterns/feedback/toasts.html) | undo in a toast after a destructive action | T1, full text |
| [Writing Style — GNOME HIG](https://developer.gnome.org/hig/guidelines/writing-style.html) | specific verbs; ellipsis when more input is needed | T1, full text |
| [Adw.Dialog — Libadwaita 1.6](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1.6/class.Dialog.html) | dialog cannot exceed its parent; not resizable | T1, API reference |
| [Alerts — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/alerts) | title/body/buttons, default trailing, Cancel and Escape, undo-over-alert rule | T1 title and summary only; quoted text from the mirror below |
| [Action sheets — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/action-sheets) | when choices belong in a sheet, and that macOS adds no rules | T1, summary only |
| [Modality — Apple HIG](https://developer.apple.com/design/human-interface-guidelines/modality) | modal blocks its parent and requires an explicit dismissal | T1, summary and indexed excerpts |
| [NSAlert.buttons — Apple](https://developer.apple.com/documentation/appkit/nsalert/buttons) | index 0 is the rightmost/default button; Cancel gets Escape | T1, full text via the documentation JSON |
| [Apple HIG alert text — mirror](https://github.com/JHKits/Apple_HIG_Korean/blob/main/translated-markdown/components/presentation/alerts.md) | the alert body Apple's own page would not serve | T6, secondary — quoted only where marked |

Access date for all sources: 2026-10-27.
