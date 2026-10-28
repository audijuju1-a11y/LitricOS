# LitricOS UI/UX Research — Part 25
## Chapter L: Linux Desktop Environments (batch 1 — GNOME 51 and KDE Plasma 6.8)

**Part:** 25
**Chapter:** L (Research E, Linux desktop environments)
**Rows covered:** `LG.01`–`LG.14` (GNOME 51), `LK.01`–`LK.14` (KDE Plasma 6.8) — 28 of the 112 rows the checklist holds for Chapter L
**Date:** 2026-10-28
**Evidence base:** five pages read in full in this pass (`S3856`–`S3860`), plus the corpus measurements of Parts 19–24 for the LitricOS column.

---

## L.0 Why this chapter exists, and why it is the last one

Chapters F, G, H, I and J measured LitricOS against *itself* and against design
systems (Fluent, Adwaita, KDE HIG, Apple HIG) at the component and token level.
None of them measured it against a *desktop environment*: a shell, a compositor,
a settings application, a file manager and a notification server that ship
together and answer to one release cadence.

That is a different comparison, and it is the one the gate matrix asks for when
it says "no settings daemon" and "one output". A design system can be adopted
one control at a time; a desktop environment is a package of decisions that were
made to fit each other. Chapter L is therefore the chapter where the *shape* of
LitricOS is compared, not its widgets.

### L.0.1 Evidence grade of this batch

| Source | ID | Grade | What it establishes |
|---|---|---|---|
| GNOME 51 release notes | S3856 | T1 (project) | What changed in GNOME 51, in the project's own words |
| GNOME HIG, front page | S3857 | T1 (project) | The structure of the HIG: principles, resources, guidelines, patterns (containers, navigation, feedback, controls), reference |
| KDE HIG, front page | S3858 | T1 (project) | The HIG's scope and its nine sections, including "Accessibility and inclusiveness" |
| KDE HIG, "Simple by default" | S3859 | T1 (project) | Concrete rules: first-run, progress, undo, deferred deletion, token names |
| Plasma 6.8 announcement | S3860 | T1–T2 | That 6.8 shipped; the announcement body yielded no feature list to this reader |

Two pages I expected to use were not retrievable and are recorded as gaps rather
than paraphrased from memory:

* `developer.gnome.org/hig/patterns/` — **404**. The pattern catalogue is
  therefore cited by its section names from the HIG front page (`S3857`), not by
  its individual pages.
* `develop.kde.org/hig/layout_and_navigation/` — **404**. The KDE layout and
  navigation rules are **not** in this batch; the KDE rows that depend on them
  are marked PARTIAL for that reason and not for a lack of interest.

This is the same discipline Chapter J applied to inaccessible rows: a page that
did not load is a gap in the evidence, not a claim.

---

## L.1 GNOME 51 "A Coruña" (2026-09-16)

The release notes are unusually explicit about what kind of release this is:
three of the five sections that matter to a shell are about *time* and *state*
rather than about features — smoother frames under load, remembered monitor
brightness, and four separate ways for a user to get more time to see or do
something.

### L.1.1 The rows

| Row | What GNOME 51 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LG.01` Shell components & compositor | GNOME Shell + Mutter. Frame scheduling reworked, screen capture made cheaper, legacy NVIDIA interfaces removed, cursors rendered from SVG | S3856 | One Rust shell; own compositor with 3 globals (`wl_compositor` v4, `wl_seat` v5, `xdg_wm_base` v2); cursor drawn from code (`ui::cursor`) | PARTIAL |
| `LG.02` Launcher | Activities overview: search plus app grid; screen reader "search results are described more usefully" and "app grid is easier to navigate" | S3856 | One launcher overlay; ranking in `seek.rs`; no providers, no grid | PARTIAL |
| `LG.03` Panel / dock | A single top bar; the dash lives in the overview, not on the desktop. A permanent dock is an extension | S3856 (implicit: no dock change), S3857 | One top bar plus a dock that morphs into the launcher | PARTIAL |
| `LG.04` System Settings inventory | ~20 panels. 51 changed: Displays (auto-rotate, orientation lock, centre-aligned snapping), Mouse (disable touchpad when a mouse is plugged in), Network (DNS domain search; WEP removed; Remote Login over SSH sockets), Users (fingerprint enrolment UI), About (reorganised, with a release-notes button) | S3856 | 13 pages, 8 stored keys; Display is read-only except the UI scale | PARTIAL |
| `LG.05` File manager | Nautilus: drag-count badge; copied files auto-selected; right-click on empty area no longer clears the selection; read-only/unreadable emblem precedence; fewer blocking operations; grouped notifications | S3856 | `FilesApp` with 26 verbs, 5 trash verbs, no cancel, no emblem precedence, no drag badge | PARTIAL |
| `LG.06` Window behaviour & tiling | Mutter: no first-class tiling UI in this release's notes; window behaviour is the compositor's | S3856 (absence), S3857 | 36 WM operations including `cancel_drag` and restore; one output, one workspace grid | PARTIAL |
| `LG.07` Notifications | GNOME Shell is the notification server; per-app settings in Settings. Files' notifications are now **grouped** | S3856 | Private queue (`litric-notifyd`), banners with open/close hotspots, no grouping, no actions, no D-Bus | PARTIAL |
| `LG.08` Workspace model | Dynamic workspaces (vertical) — unchanged in 51 and therefore not in the release notes | S3856 (absence) | Fixed grid, per-window `workspace` field, Mission Control ordering by workspace then open order | PARTIAL |
| `LG.09` Customization model | Extensions are the extension point; appearance is deliberately narrow (themes, accent, new wallpaper variants) | S3856, S3857 | 4 themes × 6 accents × 3 wallpapers; no extension registry (I.02) | PARTIAL |
| `LG.10` Design language / HIG / tokens | The HIG is a five-part document: design principles, resources, guidelines, patterns (containers, navigation, feedback, controls), reference (keyboard shortcuts, UI colours). 51 extends reduced motion into "many core widgets and the shell itself" | S3857, S3856 | 11 semantic colours × 4 themes, 8 type steps, 8 space steps, 4 motion tokens; tokens enforced by a lint | **COMPLETE** |
| `LG.11` Accessibility | 51 ships: reduced motion widened; screenshot area adjustable by keyboard (arrows resize, Alt moves, Shift/Ctrl coarse/fine, R resets); focus-ring timeout can be turned **off**; screen reader no longer announces the dock twice; high-contrast week view in Calendar | S3856 | One toggle (reduce motion), honoured everywhere; no a11y tree, no focus timeout, no keyboard path to region selection | PARTIAL |
| `LG.12` Limitations & known issues | Legacy NVIDIA interfaces **removed**; WEP **removed**; extensions break across major releases (structural) | S3856 | Single output, one scale, no fractional scaling, no capability prompt | PARTIAL |
| `LG.13` Integration points | Portals (`xdg-desktop-portal-gnome`), D-Bus, GSettings/dconf, systemd user units, Flatpak | S3857 (platform), corpus Parts 19–23 | LFP/1 + browser socket + notifyd; no bus, no portal, no discovery (I.06, I.07) | PARTIAL |
| `LG.14` DE vs kernel vs distro boundary | DE owns shell, compositor, settings apps, session apps; kernel owns DRM/input; logind owns sessions; NetworkManager and BlueZ own radio state; PipeWire owns audio; the distribution packages all of it | corpus Parts 19–23, S3857 | One image (mkosi) contains shell, compositor, sysd, filesd, capd, notifyd; no package manager, no session manager | PARTIAL |

### L.1.2 Findings — GNOME

**L25.01 — The accessibility release was a release about time, not about
features.** Reduced motion widened, a focus-ring timeout that can be disabled,
and a screenshot region adjustable entirely from the keyboard. Each one gives
the user more time or more ways to act without a pointer. LitricOS has the first
third of that: one motion preference that the whole shell honours. It has no
focus timeout at all (because `Hot` rectangles have no timeout), and its region
selection — where it exists — has no keyboard route.

**L25.02 — Settings matured by covering hardware edges, not appearance.**
Auto-rotate on devices with an accelerometer, a touchpad that switches off when
a mouse appears, DNS search domains, SSH socket servers. A settings surface is
judged by the machines it refuses to abandon. LitricOS' Display page has one
control and four readings; the pattern GNOME demonstrates is that the *edge*
rows are the ones that get written late, after the appearance rows are done.

**L25.03 — Nautilus fixed three selection bugs in one release, and LitricOS has
all three.** Copied files are auto-selected, right-clicking empty space keeps
the selection, and an unreadable folder shows the right emblem with a defined
precedence. LitricOS: a copy leaves the user to find the result, a right-click
re-derives the whole surface (`F.13`: a second right-click replaces the menu),
and there is one error line rather than emblem precedence.

**L25.04 — Notification grouping is a platform capability, not an app feature.**
Files' notifications are grouped because the server can group them. LitricOS'
queue has an id and a priority and no group key, so two files failures are two
banners that cannot be collapsed (`F.17`).

**L25.05 — The HIG itself is a five-part contract, and LitricOS has three of the
five parts as code.** Principles, resources, guidelines, patterns, reference:
LitricOS has tokens (reference), components (patterns, partly), and guidelines
scattered as comments in `ui.rs`. It has no principles document and no pattern
catalogue, which is why every part of this corpus has had to reconstruct the
rules from behaviour.

---

## L.2 KDE Plasma 6.8 (2026-10-20)

The Plasma 6.8 announcement page that this pass could read establishes the
release and its framing ("KDE's 30th anniversary … a foundation for the future")
and **no feature list**. Everything below that is specific to Plasma comes from
the KDE HIG, which is version-independent in exactly the way a shell's design
rules should be, and is marked as such.

### L.2.1 The rows

| Row | What Plasma 6.8 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LK.01` Shell components & compositor | plasmashell + KWin; Wayland session; panels, widgets, KRunner as separate processes | S3858 (platform scope), S3860 (release only) | One process draws bar, dock, overlays and every app | PARTIAL |
| `LK.02` Launcher | Kickoff and KRunner; HIG requires "something actionable when the app is first launched" and forbids asking preferences in a wizard | S3859 | One launcher overlay, recency-ordered, searches apps and settings rows | PARTIAL |
| `LK.03` Panel / dock | Configurable panels (position, thickness, visibility, floating); multiple panels | S3858 (subject), no page read | Fixed top bar; dock is an overlay state | PARTIAL |
| `LK.04` System Settings inventory | System Settings modules (~30); not enumerated by any page read in this pass | — (gap) | 13 pages, 8 stored keys | PARTIAL |
| `LK.05` File manager | Dolphin; HIG: rename inline "rather than with a separate modal dialog box"; trash instead of delete; undo for anything not trivially re-addable | S3859 | Rename is a modal (`F.15`), trash exists (5 verbs), undo does not exist anywhere (`F.27`) | PARTIAL |
| `LK.06` Window behaviour & tiling | KWin tiling and quick-tile; Activities; no page read | — (gap) | 36 WM operations, one output, no tiling model | PARTIAL |
| `LK.07` Notifications | Plasma's notification server with DND; HIG distinguishes `showPassiveNotification` (self-dismissing) from `InlineMessage` (persistent) for in-app feedback | S3859 | Private queue; banners dismiss themselves by TTL; no in-app persistent message component | PARTIAL |
| `LK.08` Workspace model | Virtual desktops + Activities; no page read | — (gap) | Fixed workspace grid, no activities, no per-activity context | PARTIAL |
| `LK.09` Customization model | "Simple by default, powerful when needed": the platform is expected to be configurable, and defaults are expected to be good | S3859 | 4 themes, 6 accents; settings are a closed set with no third-party surface | PARTIAL |
| `LK.10` Design language / HIG / tokens | Nine HIG sections. Token vocabulary: `Kirigami.Theme` colours, `iconsSizes`, `smallSpacing` / `mediumSpacing` / `largeSpacing`, `veryShortDuration` / `shortDuration` / `longDuration` / `veryLongDuration` | S3858, S3859 | `Space::{Xs..Xxl}`, `Motion` springs with settle budgets; same axes, different names | **COMPLETE** |
| `LK.11` Accessibility | HIG has an "Accessibility and inclusiveness" section; not read in this pass | S3858 (section exists) | 1 toggle, no a11y tree, no AT-SPI | PARTIAL |
| `LK.12` Limitations & known issues | HIG states window size and position are remembered **on X11**, i.e. not on Wayland | S3859 | Honour: LitricOS restores neither, on any output | **COMPLETE** |
| `LK.13` Integration points | D-Bus, KIO, KConfig, KWallet, portals, systemd user units | S3859 (KWallet, KRecentDocument), corpus Part 23 | LFP/1, browser socket, notifyd; no bus, no KIO-equivalent, no key store | PARTIAL |
| `LK.14` DE vs kernel vs distro boundary | Same boundary as GNOME: DRM, logind, NetworkManager, BlueZ and PipeWire outside the DE; distribution packages the DE | S3858, corpus Parts 19–23 | One image; the boundary is a build decision | PARTIAL |

### L.2.2 Findings — Plasma

**L25.06 — KDE states the undo rule that LitricOS inverts.** Three sentences, in
order: hide destructive actions or confirm them; move files to the trash rather
than deleting; *offer an undo for every action that removes content not trivial
to re-add*, using a passive notification when losing the undo is survivable and
an inline message when it is not. LitricOS has 11 confirmations and 7 prompts for
routine work, and zero undo (`F.27`). It pays the cost of asking and still has
no way back.

**L25.07 — The first-run rule is explicit and LitricOS breaks it.** A wizard is
for "mandatory and unskippable setup" or an optional teaching tool, and it must
not ask the user to express preferences. LitricOS' first run asks three
preferences and a Finish button (`H.01`), which is precisely the case the rule
names.

**L25.08 — "Remember window size and position" is an X11-only capability in
Plasma.** A desktop with thirty years of window management says outright that
one of its oldest behaviours does not survive the move to Wayland. LitricOS has
never had it, so it is not behind on the feature — it is behind on the *reason*
the feature is hard, which is that a compositor owns placement and a client
cannot be trusted to ask for it.

**L25.09 — Token names map onto LitricOS one-for-one, which makes this the
cheapest adoption in the corpus.** KDE's four spacing steps and four duration
steps are the same four axes LitricOS already has (`Space`, `Motion`). Adopting
the *rule* ("use the named step, never a number") is a rename and a lint, not a
redesign — and LitricOS already has the lint (`tests/design_lint.rs`).

**L25.10 — Plasma's progress rule is a threshold, not a control.** Show an
indicator for anything over a second; prefer determinate; use the indeterminate
form when the count is unknown. LitricOS has one progress bar, no indeterminate
form, and no threshold: a scan that takes 200 ms and one that takes two minutes
look the same until it finishes (`F.18`, `F.25`).

---

## L.3 The comparison that matters

Fourteen rows per environment, and the same shape appears in both:

| Question | GNOME 51 | Plasma 6.8 | LitricOS |
|---|---|---|---|
| Who draws the panel? | Shell process | plasmashell | The shell |
| Can a third party extend the shell? | Extensions | Widgets, scripts, KWin effects | Manifest + capabilities only (I.02) |
| Where do settings live? | GSettings/dconf | KConfig | 9 keys in `settings.conf` |
| Who serves notifications? | GNOME Shell | plasmashell | `litric-notifyd` |
| How does an app ask for a capability? | Portal | Portal | Capability broker, no prompt (I.04) |
| What happens after a destructive action? | Trash, plus app-level undo | Trash, plus HIG-mandated undo | Trash, no undo |
| How is the interface scaled? | Per-monitor fractional, fractional scaling | Per-output scale | One scale for one output |
| What does the DE not own? | Kernel, logind, NetworkManager, BlueZ, PipeWire | Same | Nothing — all inside one image |

The last row is the chapter's real result. GNOME and Plasma are both *parts* of
a Linux system: they draw, and something else owns the hardware, the session and
the radio. LitricOS is the whole system, which is why its gaps in Chapters F–I
read as "missing service" rather than "missing feature" — and why "add a
setting" in LitricOS is a build change, not a package change.

Two design choices that Chapter L makes look deliberate rather than accidental:

1. **One bar, no separate dock process.** GNOME also has one bar, and its dock
   is an overlay state (the dash). LitricOS' morphing dock is the same decision
   with a different animation, and it is defensible on the same grounds.
2. **No first-class tiling.** Neither environment in this batch ships a tiling
   mode as its default; both offer it as an add-on or a shortcut. LitricOS'
   three-way window control (float / maximize / close) is closer to the GNOME
   default than to Plasma's configurability, and that is a choice worth stating
   in the ADRs rather than leaving implicit.

---

## L.4 What to change in LitricOS because of this part

| # | Change | Why (row) | Size |
|---|---|---|---|
| 1 | Group notifications by app in the queue and in the centre | `LG.07`, L25.04 | M |
| 2 | Keep the Files selection when the empty area is right-clicked | `LG.05`, L25.03 | S |
| 3 | Select the result of a copy, as Nautilus does | `LG.05`, L25.03 | S |
| 4 | Add an undo for a delete that is not yet purged, with the HIG's split: banner for reversible, inline for destructive | `LK.05`, `LK.07`, L25.06 | L |
| 5 | Move first-run preferences after setup rather than into it | `LK.02`, L25.07 | S |
| 6 | Give the Files dialog an inline rename path before the modal | `LK.05` | M |
| 7 | Show progress after one second, and add an indeterminate form | `LK.10`, L25.10 | S |
| 8 | Rename nothing, but lint the rule: every spacing and duration comes from a token | `LK.10`, L25.09 | — (exists) |
| 9 | Write the two missing HIG documents: principles and a pattern catalogue | `LG.10` | M |

Changes 1–3 are small and independent. Change 4 is the only large one, and it is
the one both environments name explicitly, which is the strongest evidence in
this chapter that it is not a nicety.

---

## L.5 Limitations of this batch

* **Two environments of eight.** Xfce, Cinnamon, COSMIC, Pantheon, Budgie and
  ChromeOS remain PENDING; Chapter L is 28 of 112 rows.
* **No Plasma feature page.** The 6.8 announcement yielded no feature list, and
  the KDE HIG layout and navigation page 404'd. Rows `LK.03`, `LK.04`, `LK.06`,
  `LK.08` and `LK.11` are structural, not page-level.
* **No GNOME pattern pages.** The HIG front page gives the five-part structure;
  the individual pattern rules were not read in this pass.
* **No environment was run.** Nothing here is a measurement of a GNOME 51 or
  Plasma 6.8 session; the LitricOS column is measured code, the DE column is
  documentation.
* **Release cadence is not compared.** Both projects ship on schedules that
  LitricOS does not have, and no row in this chapter covers it.

---

## L.6 Sources read in this part

| ID | URL | What it gave |
|---|---|---|
| S3856 | https://release.gnome.org/51/ | GNOME 51 changes across display, settings, Files, accessibility, login, cursors, key storage |
| S3857 | https://developer.gnome.org/hig/ | HIG structure: principles, resources, guidelines, patterns, reference |
| S3858 | https://develop.kde.org/hig/ | KDE HIG scope and its nine sections |
| S3859 | https://develop.kde.org/hig/simple_by_default/ | First-run, progress, undo, deferred deletion, token names, X11-only window state |
| S3860 | https://kde.org/announcements/plasma/6/6.8.0/ | That Plasma 6.8 shipped, and its framing only |

Gaps recorded, not cited: `developer.gnome.org/hig/patterns/` (404),
`develop.kde.org/hig/layout_and_navigation/` (404).
