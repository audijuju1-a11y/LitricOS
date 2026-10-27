# LitricOS UI/UX Research — Part 16 of N

Chapter 40 (F) — Tooltips and notifications: one surface the shell does not have,
and one it has four times

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P16 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-27 (Asia/Bangkok) |
| Scope of this part | Chapter 40 (`F`): `F.14` tooltips and `F.17` notifications/banners/toasts. The other 25 `F` items are untouched and keep their status |
| Batch | B9 (`Part1` §2.2) — "F: Component catalog across all systems", continued on the two feedback surfaces the shell needs for every other component |
| Companion files | `Research-Deep-all/LitricOS Coverage Checklist.csv` (2 statuses changed), `Research-Deep-all/LitricOS Source Register.csv` (8 sources appended, S3806–S3813), `docs/evidence/2026-10-27/notification-probe.log` |
| Predecessors | Parts 10–15, and `docs/bugs/BUG-0023`–`BUG-0025` |
| Resume point | `F.07` pickers, `F.09` navigation views and `F.20` trees — the largest `F` items still `PENDING` |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and are used here without redefinition.

---

## 0. Scope, method, and why these two together

### 0.1 The same question from opposite ends

Both items answer one question — *how does this interface say something the user
did not ask for?* — and the shell answers it in two opposite ways.

| Surface | State in the shell | Code |
|---|---|---|
| **Tooltips** | **does not exist**. The word appears once, inside a comment about client applications | `views.rs` line 75 |
| **Notifications** | exists **four times over**: a queue model, a banner layer, a centre panel, and a daemon that persists them across a reboot | `shell.rs` lines 452–646; `views.rs` lines 5393–5576; `litric-notifyd` |

That asymmetry is the finding. A desktop with a queue that survives a reboot and
no way to name an icon button has built the expensive half of feedback and left
the cheap half undone.

### 0.2 The numbers these two are made of (MEASURED)

| Measure | Value | Where |
|---|---|---|
| Occurrences of a tooltip implementation | **0**. One comment, no struct, no timer, no `Hot` variant, no delay constant | `views.rs` line 75 |
| Controls drawn as a glyph with no visible text | **9**: the status row's audio, network, battery, power and bell icons (a sixth, the Defender shield, is added by the same loop) and **3** window buttons (close, minimise, maximise) | `views.rs` lines 481–487, 756–770 |
| Window buttons that reveal their glyph only on hover | **3 of 3** | `views.rs` lines 756–771 |
| Notification model fields | **7**: id, source, title, body, priority, at, expiry | `shell.rs` lines 465–480 |
| Actions, icons, groups, categories on a notification | **0** of each | `shell.rs` lines 465–480 |
| Notification call sites | **32** (23 in `main.rs`, 9 in `shell.rs`) | `grep -c '\.notify('` |
| Priorities | **4**: low, normal, high, critical | `shell.rs` lines 452–461 |
| Queue capacity | **64**, dropping the oldest non-critical first | `shell.rs` lines 495, 581–589 |
| Banner size | **300 × 60** (≤ 60 % of screen width), 12 px from the right edge, 10 px below the bar, 8 px apart | `views.rs` lines 5532, 5547 |
| Banners at once | **3** live, plus any that are still leaving | `views.rs` line 5515 |
| Banner hotspots | **0** — measured, not read | probe |
| Banner dismissal | only through the centre; a critical banner never expires | `views.rs` lines 5511–5515; probe |
| Centre panel | **340** wide (≤ 90 %), rows **56** high with a 2 px gap, height ≤ 70 % of the screen | `views.rs` lines 5396–5405 |
| Centre row, measured | **324 × 56** at x = 936, first row y = 80, next at 138 (58 px pitch) | probe |
| "Clear all" | **72 × 22**, at (1184, 46) | `views.rs` line 5432; probe |
| Text a centre row shows | title + **source**; the **body is not drawn** | `views.rs` lines 5472–5488 |
| Text a banner shows | title + body, body elided to the banner width | `views.rs` lines 5568–5575 |
| Keyboard reachability of the centre | **Escape only**; Down leaves the panel open and moves nothing | `shell.rs` line 4729; probe |
| TTLs chosen by call sites | 6 s ×10, 8 s ×3, 10 s ×3, 9 s, 5 s, 8 s …; **no central default** | `grep` over `main.rs` |
| A notification with no TTL | **never expires**, in the shell and in the daemon | `src/litric-notifyd/src/proto.rs` line 157 |
| Bell badge | a count, capped at `9+` | `views.rs` lines 649–657 |
| Do Not Disturb | one global switch; silences every priority **except** critical | `shell.rs` line 645 |
| Persistence across the shell being down | **yes**, and adoption never banners | `shell.rs` lines 540–575; `litric-notifyd` |

### 0.3 The probe, and what it says about the banner layer

A temporary test drove the real input path and printed the states in
`docs/evidence/2026-10-27/notification-probe.log`. The load-bearing lines:

```
probe.banner_pixel_changed=true
probe.banner_hotspots=0
probe.click_on_banner: overlay=None count 4 -> 4
probe.centre_row0=Some(Rect { x: 936.0, y: 80.0, w: 324.0, h: 56.0 })
probe.clear_all=Some(Rect { x: 1184.0, y: 46.0, w: 72.0, h: 22.0 })
probe.dnd_normal_banners=false dnd_critical_banners=true
probe.expiry_removed=2
probe.arrow_kept_overlay_open=Notifications
```

Two readings matter. A banner **is** drawn (the sampled pixel changes), and it is
**not** interactive (no hotspot, and a click on it changes neither the queue nor
the overlay). Everything else about banners — tone stripe, icon, title, body — is
paint on a surface that cannot be clicked, dismissed or focused.

### 0.4 What the shell's own specification says

`docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` §6 describes the notification system
the shell is supposed to have:

| Spec requirement | Measured here |
|---|---|
| A toast for transient confirmations, a centre for history | both exist |
| Banners carry an action, and a destructive action offers **undo for 5 s** | **no action mechanism at all**, so no undo |
| Priority decides interruption | yes, four levels, and critical is never auto-dismissed |
| Do Not Disturb suppresses banners and not the centre | yes, and critical still banners |
| Every surface is keyboard reachable | **no**: Escape closes the centre and nothing else moves |

The spec's undo requirement cannot be met by the current model at all: `notify`
takes five arguments and none of them is an action.

---

## F.14 — Tooltips

### F.14.1 What the platforms require

| Platform | Rule | Source |
|---|---|---|
| **Windows 11 26H2 / WinUI 3** | "Tooltips display automatically when the user moves focus to, presses and holds, or hovers the pointer over the associated control." **Keyboard focus raises a tooltip** — the pointer is not the only trigger | Microsoft Learn, *Tooltips* (T1, full page) |
| Windows | "Toolbar controls and command buttons showing only icons need tooltips." And: "If a control does not have a text label, use a tooltip to provide the label." | same page |
| Windows | "Keyboard accelerators are displayed in tooltips by default." A custom tooltip must still carry them | same page |
| Windows | "Don't put interactive controls inside the tooltip." "Users can't interact with tips because moving the mouse makes them disappear." | same page |
| Windows | "Use tooltips sparingly (or not at all). Tooltips are an interruption." | same page |
| Windows | Since Windows 11 21H2, "a tooltip can also be dismissed by pressing the CTRL key" | same page |
| Windows | Default placement is "centered above the pointer", and the tooltip is **not** constrained to the app window | same page |
| **GNOME 51** | "Controls in the header bars of primary windows should all have tooltips. Elsewhere, only use tooltips when they are really useful" | GNOME HIG, *Tooltips* (T1, full page) |
| GNOME | "If a tooltip is provided for one control in a container, all other controls in that container should also have tooltips." | same page |
| GNOME | "Tooltip labels should be short and written in header capitalization." Examples: `Grid View`, `Fullscreen`, `Back`, `Search` | same page |
| GNOME | When the control already has a label, "avoid repeating the control label and try to provide useful supplementary information" | same page |
| GNOME | Tooltips "aren't always accessible (such as when using a touch device). Don't rely on tooltips to communicate essential information." | same page |
| **macOS 27** | Help tags: Apple's page returned as a **summary only** to this scraper. Its own summary: "you can provide contextual help when necessary" | Apple HIG, *Offering help* (T1, summary depth) |

The two full pages agree on the three rules that matter here, and the shell breaks
all three: icon-only controls need a tooltip, the tooltip text is the control's
name plus optional supplementary information, and a keyboard user must be able to
reach the same information without a pointer.

### F.14.2 What the shell has

Nothing. `tooltip` (any case) appears in `src/litric-shell/` exactly once, in a
comment: *"Menus and tooltips from client applications sit above every window"*
(`views.rs` line 75). That comment is about drawing order if such a window ever
exists; no tooltip is ever created.

There is no delay constant, no hover timer, no `Hot::Tooltip`, no `ui::tooltip`
helper, and no accessible-name store to source the text from. Hover is already a
first-class state (`is_highlighted`, hover springs, pointer tracking), so the
machinery a tooltip would ride on exists; the tooltip does not.

### F.14.3 Findings

| # | Finding | Where |
|---|---|---|
| **F14.01** | The shell has **no tooltip system**. The one occurrence of the word is a comment about client windows | `views.rs` line 75 |
| **F14.02** | The three window buttons draw as **unlabelled coloured dots** whose glyph appears only on hover. This is the exact case Microsoft names as needing a tooltip, and GNOME's header-bar rule covers it too | `views.rs` lines 756–771 |
| **F14.03** | The status-bar controls are icon-only, with no tooltip and no accessible name. The bell is the only one that carries information beyond its glyph (a count badge) | `views.rs` lines 481–487, 649–657 |
| **F14.04** | There is no keyboard path to any of it: hover is the only trigger, so a keyboard-only user cannot learn what an icon means at all. Windows shows a tooltip **on focus** | `views.rs` lines 756–771; platform rule above |
| **F14.05** | When the shell wants to explain something, it uses a **notification** instead — 32 call sites, several of them informational ("Input: No keyboard found"). That is the opposite of the platform advice to keep essential text in the UI and tooltips supplemental | `main.rs` lines 1101, 1139; `shell.rs` line 6316 |
| **F14.06** | There is no label source to build tooltips from: button labels are literals at call sites, and no control carries a name/description pair | `views.rs`; no name field in `Hot` |

Status for `F.14`: **PARTIAL** — researched at page depth on two platforms and at
summary depth on one; the shell side is measured (absent). What is missing is not
research but a component.

---

## F.17 — Notifications, banners and toasts

### F.17.1 What the platforms require

| Platform | Element / rule | Source |
|---|---|---|
| **GNOME 51** | Elements: **App Icon** ("indicates which app sent the notification"), **Title** ("short and concise … written in header capitalization. The notification should be understandable from the title alone"), **Body** ("optional sentence … sentence capitalization"), **Default Action** ("triggered when the notification body is clicked … dismiss the notification and show the relevant part of the source app window"), **Actions** ("Each notification can include up to three buttons") | GNOME HIG, *Notifications* (T1, full page) |
| GNOME | "If a message is only relevant while the sender app is being used, toasts are a better option." | same page |
| GNOME | "Don't rely solely on notifications for communicating something … The app window should independently communicate all the information contained in notifications." | same page |
| GNOME | "Ensure that your app removes notifications that are no longer valid." | same page |
| GNOME | Rate-limit chatty sources: "instead of showing a notification for each new message, periodic notifications can show a summary" | same page |
| GNOME | Actions "should not replace user interface controls elsewhere — it should be possible to take the same actions from your app's windows" and "should not duplicate the default action" | same page |
| **Windows 11 26H2 / Windows App SDK** | "App notifications are UI popups that appear outside of your app's window, delivering timely information or actions to the user. Notifications can be purely informational, can launch your app when clicked, or can trigger a background action" | Microsoft Learn, *App notifications overview* (T1, full page, but an index) |
| Windows | Documented limitation: "Apps running with administrator privileges (elevated) cannot send or receive app notifications." | same page |
| **macOS 27** | "A notification gives people timely, high-value information they can understand at a glance." Page body not readable to this scraper | Apple HIG, *Notifications* (T1, **summary depth**) |

Microsoft's design-detail page (`App notification UX guidance`) is linked from the
overview but its URL returned 404 to this scraper on three attempts; the overview
is therefore cited for what it states and **not** for timing or layout rules.
No Windows timing number is claimed anywhere in this part.

### F.17.2 What the shell has

**Model** (`shell.rs` lines 452–492): `Priority` is four levels; `Notification` is
`id, source, title, body, priority, at_ms, expires_ms`. `source` is a string, not an
application identity: nothing joins it to an installed app, an icon, or a window.
`Notifications` is a `VecDeque` with a capacity of 64 and a `do_not_disturb` flag;
eviction removes the oldest **non-critical** entry first.

**Posting**: `Shell::notify(source, title, body, priority, ttl_ms)` (`shell.rs` line
6316). The TTL is the caller's decision, every time: 6 s is used ten times, 8 s
three times, 10 s three times, and some call sites pass `None`, which means the
notification never expires. The daemon applies the same rule (`src/litric-notifyd/src/proto.rs`
line 157), so "no TTL" is persistent in both places.

**Banner layer** (`views.rs` lines 5500–5576): at most three live banners, each
300 × 60 at 12 px from the right edge, 10 px under the bar, 8 px apart. A 3 px
tone stripe, a 16 px priority icon, the title in footnote, the body in caption,
elided to the banner's width. It arrives and leaves on a spring, sliding from the
screen edge because the software canvas has no alpha compositing.

**Centre** (`views.rs` lines 5393–5490): one panel 340 wide, rows 56 high, header
64, "Clear all" 72 × 22, empty state "Nothing waiting". A row is an icon (warning
or info, chosen by priority), the title, and the **source**. Clicking a row
**dismisses** it (`shell.rs` lines 5843–5848).

**Interruption rules**: banners appear for anything with an expiry, and for
criticals, which never have one (`views.rs` lines 5511–5515). Do Not Disturb
silences every priority except critical (`shell.rs` line 645) — a deliberate rule
with a test behind it.

**Persistence**: `litric-notifyd` queues notifications and hands them over when the
shell starts; adoption sets the original timestamp and never raises a banner
(`shell.rs` lines 540–575). This is better than most of what is measured in this
corpus: a notification raised while the desktop was down is not lost, and the
recovery does not interrupt.

### F.17.3 Findings

| # | Finding | Where |
|---|---|---|
| **F17.01** | The model has **no action, no app icon, no grouping, no category, no default action** — five of the six elements GNOME defines. The missing one is the one every platform treats as the point of a notification: what happens when you click it | `shell.rs` lines 465–480 |
| **F17.02** | **Banners are not interactive.** Measured: 0 hotspots on the banner layer, and a click on a banner changes neither the queue nor the overlay. Microsoft's and GNOME's whole action model — click the body to open the source, buttons to act — has no surface here | probe; `views.rs` lines 5500–5576 |
| **F17.03** | **A critical banner cannot be dismissed where it is.** Criticals never expire by design, the banner draws no close control, and it has no hotspot. It sits at the top right until the centre is opened and the row or "Clear all" is clicked | `views.rs` lines 5511–5515; `shell.rs` line 645; probe |
| **F17.04** | **The centre drops the body.** A row shows title and source, so the sentence that explains the notification appears only in a banner that may already be gone. This contradicts "the notification should be understandable from the title alone" combined with "the app window should independently communicate all the information" — the shell has neither the second sentence nor the body in its history | `views.rs` lines 5472–5488 |
| **F17.05** | **No default action.** Clicking a row destroys it. Nothing opens the app that sent it, and after the click there is no record that it existed — the opposite of "dismiss the notification and show the relevant part of the source app window" | `shell.rs` lines 5843–5848 |
| **F17.06** | **The centre is pointer-only.** Escape closes it; Down leaves it open and moves nothing. Every other keyboard-reachable surface in the shell (launcher, switcher, menus) has a selection index; the centre has none | `shell.rs` line 4729; probe |
| **F17.07** | **TTL is a per-call-site decision with no default.** 6 s appears ten times, 8 s three times, 10 s three times, and `None` is used for anything the caller wants to keep. Two notifications of the same priority can therefore behave differently, and there is no place that states the policy | `main.rs`; `src/litric-notifyd/src/proto.rs` line 157 |
| **F17.08** | **No app identity.** `source` is a free string drawn where a secondary line would be. There is no icon, no colour, no link to a window, and two applications that choose the same string are indistinguishable | `shell.rs` line 469 |
| **F17.09** | **No rate limiting or grouping**, so a chatty source fills the centre until the 64-entry cap evicts other applications' notifications. GNOME's summary-instead-of-each-message rule has no equivalent | `shell.rs` lines 495, 581–589 |
| **F17.10** | What **is** right, and should not be undone: persistence across the desktop being down, adoption without a banner, critical surviving DND, critical surviving eviction, and a count badge on the bell capped at `9+` | `shell.rs` lines 540–589, 645; `views.rs` lines 649–657 |
| **F17.11** | The banner and the centre use **different text**: the banner shows title + body, the centre shows title + source. The same notification reads differently depending on where it is seen | `views.rs` lines 5568–5575, 5472–5488 |

Status for `F.17`: **PARTIAL** — three platforms researched (two at page depth, one
at summary depth), the shell side measured with code and a probe, and the gap
between them recorded. The item becomes `COMPLETE` only when the Windows UX page
and the macOS body can be read, and when the missing elements have a decision
(adopt / adapt / reject) rather than only a finding.

---

## 1. What these two say together

| Mechanism | Tooltips | Notifications |
|---|---|---|
| Exists in the shell | no | yes, four layers |
| Reachable by keyboard | n/a (absent) | Escape only |
| Carries an action | n/a | none |
| Names its source | n/a | a free string |
| Stated in the design system | not mentioned | §6, including an undo that cannot exist |
| Tests behind it | none | queue, expiry, DND, adoption, banner exit |

The shell built a durable, tested queue and then gave it no way to be acted on,
and refused to build a one-line hover label for the ten controls that have no text.
The next `F` part that touches either should start from this asymmetry: the missing
half of the notification model is **actions**, the missing half of the tooltip
system is **the whole system**.

## 2. What this part does not establish

| Claim | Why |
|---|---|
| that the Windows notification layout or timing is as described | the UX-guidance page 404'd to this scraper and the overview is an index. No Windows timing number is quoted anywhere above |
| that Apple's notification and help-tag rules are as described | both pages returned only their own summary sentence; the body did not load. Marked summary depth, not page depth |
| that the shell's banners overlap a window or the centre | the probe read one pixel and one hotspot set, not the whole composition; no image was captured |
| that a critical banner is *always* permanent | it is permanent under the current rules (no expiry, no hotspot). Whether it is ever replaced depends on the three-live-banner cap and on user action in the centre, which was not driven |
| that notification text is translated | not measured in this part; `F.15` found dialog strings bypassing the dictionary and the same suspicion applies here, but it is untested |
| that `9+` is the badge cap in every theme and scale | read from the code at one scale; the layout audit covers text overflow but not this specific string |
| that the notification centre is unusable with a screen reader | there is no accessibility tree in this shell to inspect; the absence of keyboard navigation is described, not measured against assistive technology |

## 3. Sources

| # | Source | Depth | Used for |
|---|---|---|---|
| S3806 | Microsoft Learn — *Tooltips* (windows/apps/develop/ui/controls/tooltips) | full | focus/hover trigger, Ctrl dismissal, icon-only rule, accelerator rule, no interactive content, placement |
| S3807 | Microsoft Learn — *App notifications overview* | full (index) | what a notification can do (inform, launch, background action), elevated-app limitation |
| S3808 | GNOME HIG — *Tooltips* | full | header-bar rule, consistency within a container, header capitalisation, no essential information |
| S3809 | GNOME HIG — *Notifications* | full | app icon, title/body rules, default action, up to three actions, rate limiting, remove invalid ones |
| S3810 | Apple HIG — *Offering help* | summary | help tags exist as contextual help; body not readable |
| S3811 | Apple HIG — *Notifications* | summary | the "timely, high-value information … at a glance" definition; body not readable |
| S3812 | GTK 4 `GtkTooltip` API (linked from the HIG page) | reference | that the HIG's tooltip advice maps to a toolkit primitive; not read in depth |
| S3813 | GIO `GNotification` API (linked from the HIG page) | reference | same, for notifications |
| — | `docs/DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md` §6 | repository | the shell's own toast/centre/undo contract |
| — | `docs/evidence/2026-10-27/notification-probe.log` | repository | the measured banner/centre/keyboard states |
| — | `src/litric-shell/src/{shell,views}.rs`, `src/litric-notifyd/src/proto.rs` | repository | every shell-side number and line in §0.2 |

Platform pages were fetched on 2026-10-27. Four returned their body to the
scraper in full (both Microsoft pages and both GNOME pages); the two Apple pages
returned only a title and their own summary sentence; the two GTK/GIO API pages
are recorded from the links on the HIG pages and were not opened further.

## 4. Status changes

| Item | Was | Now | Note |
|---|---|---|---|
| `F.14` Tooltips | `PENDING` | `PARTIAL` | researched on two platforms at page depth and macOS at summary depth; the shell has no tooltip system at all (F14.01–F14.06) |
| `F.17` Notifications/banners/toasts | `PENDING` | `PARTIAL` | the model, banner layer, centre, DND and persistence measured; five of GNOME's six elements are absent, banners are not interactive, and the centre is pointer-only (F17.01–F17.11) |

No decisions are recorded in this part. It states what the platforms require and
what the shell does; adopting an action model, an app identity and a keyboard path
is a product decision with a cost, and belongs in a decision record rather than in
a research note.
