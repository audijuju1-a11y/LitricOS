# LitricOS UI/UX Research — Part 26
## Chapter L: Linux Desktop Environments (batch 2 — Xfce 4.20 and Cinnamon 6.6)

**Part:** 26
**Chapter:** L (Research E, Linux desktop environments)
**Rows covered:** `LX.01`–`LX.14` (Xfce 4.20), `LC.01`–`LC.14` (Cinnamon 6.6, Linux Mint 22.3) — **28 new rows**, 56 of the 112 the checklist holds for Chapter L
**Date:** 2026-10-28
**Evidence base:** six pages opened in this pass (`S3861`–`S3866`), plus the corpus measurements of Parts 19–25 for the LitricOS column.

---

## L.0 What this batch is for

Part 25 measured the two environments with the most formal design
documentation. This batch measures the two with the least. Xfce and Cinnamon
both ship enormous surfaces and publish almost no interface rules: Xfce has a
tour and a changelog, Mint has release notes, and neither has a HIG that this
pass could open. That asymmetry is the point. If a design rule only exists when
a project writes it down, then two decades of shipping without one should show
up as behaviour — and it does.

### L.0.1 Evidence grade of this batch

| Source | ID | Grade | What it establishes |
|---|---|---|---|
| Xfce 4.20 tour | S3861 | T1 (project) | User-visible changes, and the project's own Wayland caveats |
| Xfce 4.20 changelog | S3862 | T1 (project) | Component-by-component change list, including Thunar and xfwm4 |
| xfwm4 preferences documentation | S3863 | T1 (project) | The window manager's actual option set: focus, snapping, wrap, double-click |
| Thunar documentation (start) | S3864 | T1 (project) | Thunar's scope, plugins, standards compliance, release line |
| Linux Mint 22.3 "New Features" | S3865 | T1 (project) | Cinnamon 6.6 and Nemo 6.6 behaviour, applets, menu, XSI |
| Linux Mint 22.3 release notes | S3866 | T1 (project) | Known issues, base system, PipeWire, kernel, removed features |

Recorded gaps, not paraphrased:

* **`projects.linuxmint.com` did not resolve** from this environment
  (`ENOTFOUND`). Any Cinnamon developer documentation behind that host is
  therefore **unread**, and the Cinnamon rows that would depend on it are
  structural.
* **No Xfce HIG page was located or opened.** The tour and the component docs
  are behavioural, not prescriptive. `LX.10` is marked PARTIAL for that reason,
  not for lack of trying.
* **No Xfce notification-daemon page** (`xfce4-notifyd`) was opened, so `LX.07`
  is structural: it rests on the panel/tasklist and Thunar's optional
  `libnotify` dependency, not on a notification specification.
* **No Linux Mint user guide** was opened, so the Cinnamon Settings module
  inventory in `LC.04` is incomplete by construction.

---

## L.1 Xfce 4.20 (2024-12-15)

Xfce 4.20 is a release about *portability*: nearly every component was abstracted
away from X11 and onto Wayland through a new library, `libxfce4windowing`, while
the window manager and the compositor stayed behind. The tour says so in the
plainest sentence in this chapter: "So far Xfce does not feature a compositor
which supports Wayland."

### L.1.1 The rows

| Row | What Xfce 4.20 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LX.01` Shell components & compositor | Nearly all components ported to Wayland via `libxfce4windowing`; xfwm4, xfdashboard and the screensaver are **not** ported; on Wayland the compositor must be Labwc or Wayfire | S3861 | One Rust shell plus its own compositor (3 globals); no session manager, no external compositor | PARTIAL |
| `LX.02` Launcher | `xfce4-appfinder`: Ctrl+N/Ctrl+P navigation, Ctrl+Enter executes, per-application actions on right-click, optional daemon, single-click launch, generic names | S3861, S3862 | Launcher overlay with `seek` ranking and settings rows; keyboard list navigation; no per-app actions menu | PARTIAL |
| `LX.03` Panel / dock | `xfce4-panel`: configurable border width, plugin discovery in XDG base directories, several objects deleted at once, LCD inactive segments, 24 h analog clock, week numbers, show-desktop on hover, tasklist icon sizing | S3861, S3862 | One fixed top bar plus a morphing dock; app buttons in the dock; no plugin model, no panel configuration | **COMPLETE** |
| `LX.04` Settings inventory | Appearance, Display (profiles, scaling both dimensions, mirror without matching refresh), Mouse (adaptive/flat acceleration, high-resolution scrolling), MIME (multi-select), Keyboard, Workspace | S3861, S3862 | 13 pages, 9 stored keys; Display read-only except the UI scale; Input reports devices | PARTIAL |
| `LX.05` File manager | Thunar 4.20: toolbar overflow menu, view switcher, new tab/window buttons, CSD option, split-view focus and close confirmation with tabs, expandable folders in list view, drag-hover folder opening, bulk renamer size memory, undo asks before deleting, direct-I/O copy verification, queueing on busy disks, IPv6 remote URLs, symlinks on remote | S3861, S3862, S3864 | `apps::FilesApp`: 26 verbs, no cancel/pause, no split, no tabs, no overflow menu, no bulk rename | **COMPLETE** |
| `LX.06` Window behaviour & tiling | xfwm4: click-to-focus or focus-follows-mouse, raise on focus with delay, new-window focus, window snapping with distance, wrap workspaces (pointer and drag) with resistance, opaque move/resize, double-click action | S3863 | `litric-wm`: 36 operations, edge snapping, one output; no focus-follows-mouse, no wrap, no configurable double-click | PARTIAL |
| `LX.07` Notifications | `libnotify` is an optional Thunar dependency for mount notifications; no notification daemon page was read | S3864 (structural) | `litric-notifyd` queue, banners, centre, DND, critical that survives eviction | PARTIAL |
| `LX.08` Workspace model | Wrap workspaces at screen edges; workspace support **missing** on Wayland because the protocol arrived late | S3861, S3863 | Fixed grid of workspaces, Mission Control ordering stable by workspace then open order | PARTIAL |
| `LX.09` Customization model | Themes for window decorations, panel objects and plugins, a shortcut editor with overlap groups, Xfconf as the store, systemd user unit for the session bus | S3861, S3862 | 4 themes × 6 accents × 3 wallpapers; no plugin or panel model; 9 stored keys | PARTIAL |
| `LX.10` Design language / HIG / tokens | No HIG page located. The visible rules are GTK/Adwaita defaults plus per-component options; icon scaling at >1 was fixed across components | S3861, S3862 | 11 semantic colours × 4 themes, 8 type and 8 space steps, 4 motion tokens, enforced by a lint | PARTIAL |
| `LX.11` Accessibility | No accessibility content in the tour, changelog or xfwm4 page | — (gap) | One toggle (reduce motion) honoured everywhere; no a11y tree (G.15) | PARTIAL |
| `LX.12` Limitations & known issues | Wayland is experimental; no Wayland compositor; missing workspaces, tray icons, keyboard/mouse settings, brightness keys, rectangular screenshots; xfwm4, xfdashboard, screensaver and two panel plugins not ported | S3861 | Single output, one scale, no fractional scaling, no capability prompt | **COMPLETE** |
| `LX.13` Integration points | `libxfce4windowing` abstraction, minimal `xdg-desktop-portal` conf (wallpaper, screenshot, background), D-Bus, xfconf systemd user unit, polkit optional, `power-profiles-daemon` support | S3861, S3862 | LFP/1, browser socket, notifyd; no portal, no bus, no profile daemon | **COMPLETE** |
| `LX.14` DE vs kernel vs distro boundary | Xfce owns panel, session, settings, Thunar; the Wayland compositor is external (Labwc/Wayfire); logind, NetworkManager, PipeWire and the kernel are outside; distributions package components separately | S3861, S3862 | One mkosi image contains everything; no package manager, no external compositor | PARTIAL |

### L.1.2 Findings — Xfce

**L26.01 — Xfce is the only environment in the corpus that ports itself and
leaves the compositor to somebody else.** On Wayland it needs Labwc or Wayfire,
and it says so. LitricOS made the opposite trade: it wrote a compositor and no
session manager. Both are partial desktops, and the two halves are complementary
— which is the clearest argument in this chapter that "desktop environment" is
not one thing.

**L26.02 — Thunar's undo asks before it deletes.** "The user now will be asked
before any deletion will be done by an undo operation" (S3861). An undo stack
that can itself destroy files is a real hazard, and Xfce chose a confirmation
inside the undo rather than a smarter undo. LitricOS has no undo at all, so it
has neither problem — and no way back.

**L26.03 — Thunar added the overflow menu that LitricOS still lacks.** "On
shrinking the Thunar window, toolbar items will now be moved to an overflow
menu" (S3861). Part 13 measured LitricOS dropping buttons that would collide
(`F.11`). One of the two is a 2024 fix; the other is a 2026 defect.

**L26.04 — Split view, tabs and expandable folders are all in Thunar and none is
in LitricOS Files.** The close-with-multiple-tabs confirmation implies the exact
state LitricOS has no model for: one window, many locations, one focused pane.

**L26.05 — The file-transfer dialog was made *smaller* and the transfer stopped
stealing focus.** "The file transfer question dialog now provides the same
options while having less buttons" and "Transferring files no longer steals the
current focus" (S3861). LitricOS has no transfer surface at all: a copy is
synchronous or a failure.

**L26.06 — Xfce turned checksums off.** 4.18 validated copies with an MD5 of
source and target; 4.20 compares the files directly, because the checksum was
"rather slow and actually superfluous" (S3861). The corpus has spent five parts
praising verification; this is the counter-example that keeps it honest —
verification is only worth what it costs.

---

## L.2 Cinnamon 6.6 (Linux Mint 22.3 "Zena")

Mint's release notes are the most operationally honest documents in this
chapter: they list what broke, what was removed, and what to do instead. Cinnamon
6.6 itself is a release about **input** — keyboard layouts and input methods
unified, a native on-screen keyboard, a redesigned menu — plus Nemo's first
pause/resume for file operations.

### L.2.1 The rows

| Row | What Cinnamon 6.6 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LC.01` Shell components & compositor | Cinnamon shell on Muffin; Muffin received "a lot of changes to improve its compatibility with Wayland" | S3865 | One Rust shell; own compositor; Wayland globals for pointer, keyboard, surfaces only | PARTIAL |
| `LC.02` Launcher | Application menu redesigned: sidebar with avatar, places and favourites; smaller category list; configurable search-bar and system-button position; optional no-sidebar layout; special directories separated from bookmarks | S3865 | Launcher overlay: query, ranked app rows, settings rows; no sidebar, no favourites, no configurable layout | **COMPLETE** |
| `LC.03` Panel / dock | Panel hosts applets; 6.6 improved the workspace switcher (only visible windows, with icons), the keyboard applet (XKB + IBus), and added a Night Light applet | S3865 | One top bar; dock shows running apps; no applet model | PARTIAL |
| `LC.04` Settings inventory | No Cinnamon Settings page read. Touched by 6.6: keyboard layouts/IM, notification badge, workspace applet, Thunderbolt tool, Night Light, fractional scaling direction, Alt-Tab scope, theme selector grouping | S3865 | 13 pages, 9 stored keys, 8 searchable rows | PARTIAL |
| `LC.05` File manager | Nemo 6.6: template manager, **pause/resume of file operations**, wildcard MIME search with concurrent helpers, regex filename matching, split-pane location preserved, thumbnails | S3865 | `FilesApp`: 26 verbs, 0 cancel or pause, no split pane, no templates | **COMPLETE** |
| `LC.06` Window behaviour & tiling | Muffin: Alt-Tab can be limited to the current monitor; hot corners visible in fullscreen; fractional scaling can prefer scaling up or down; no tiling model documented | S3865 | 36 WM operations, edge snapping, one output | PARTIAL |
| `LC.07` Notifications | A missed notification now shows a badge in the **window list** | S3865 | Notification centre with count badge on the bell (capped `9+`); banners; no per-window badge | PARTIAL |
| `LC.08` Workspace model | Workspace switcher applet redrawn: only visible windows, with their icons | S3865 | Fixed workspace grid; Mission Control cards; no switcher applet | PARTIAL |
| `LC.09` Customization model | Menu element visibility, theme selector grouped by family and variants, applets, fractional-scale direction | S3865 | 4 themes, 6 accents, 3 wallpapers; no add-on surface | PARTIAL |
| `LC.10` Design language / no HIG | XApp Symbolic Icons (XSI) replaces Adwaita's symbolic set, which GNOME restricted to its own apps; all Cinnamon and Mint projects migrated | S3865 | 32 icon variants drawn from one vector module; no icon-theme compatibility layer | PARTIAL |
| `LC.11` Accessibility | On-screen keyboard rewritten natively in Cinnamon, with input-method and layout switching; no accessibility page read | S3865 | One toggle, no on-screen keyboard, no a11y tree | PARTIAL |
| `LC.12` Limitations & known issues | Release notes: HWE 6.14 breaks VirtualBox and the 470 NVIDIA driver; 10 s shutdown timeout; PipeWire replaces PulseAudio with a documented fallback; ZFS removed from the installer; ecryptfs no longer unmounts on logout; guest sessions off; Snap disabled | S3866 | No release-notes surface; failures surface as a generic "Unavailable" | **COMPLETE** |
| `LC.13` Integration points | XApp libraries and XSI, `apt://` URL handling, Update Manager tray, Thunderbolt configuration tool, Night Light; no portal page read | S3865 | Private IPC only; no bus, no portal, no package URL scheme | PARTIAL |
| `LC.14` DE vs kernel vs distro boundary | Cinnamon sits on an Ubuntu 24.04 base with kernel 6.14; PipeWire, systemd, logind, NetworkManager and the package manager belong to the distribution; Mint ships the DE | S3865, S3866 | One image; the boundary is a build decision | **COMPLETE** |

### L.2.2 Findings — Cinnamon

**L26.07 — Nemo pauses file operations; LitricOS has zero cancel verbs.** Pause
is strictly stronger than cancel: a paused copy can be resumed, and the user who
paused because the disk was busy does not have to start over. Part 22 measured
`0 cancel verbs` and this is the first source in the corpus that shows what the
next step after cancel looks like.

**L26.08 — Cinnamon unified XKB layouts and IBus input methods into one list.**
"Traditional layouts and IBus input methods will be presented in the keyboard
settings alongside each other, as if they were the same" (S3865). LitricOS has
no keyboard-layout surface at all: its Input page reports devices, and the
layout comes from the environment.

**L26.09 — The launcher was rebuilt as a layout, not a list.** Sidebar sections,
optional favourites-only mode, configurable search position, a no-sidebar
variant. LitricOS' launcher has one layout and a query; the ranking is real, the
*frame* is fixed.

**L26.10 — Mint's diagnosis tool grew four device pages while LitricOS has none.**
USB grouped by controller with speed and power against the controller's maximum,
GPU acceleration, PCI IDs and drivers, BIOS and Secure Boot. LitricOS'
Diagnostics page reports shell and daemon facts and its Activity window lists
processes; there is no device inventory anywhere. Part 22 called this "0 guided
troubleshooters"; this is the raw material those troubleshooters read.

**L26.11 — A missed notification is shown where the user is looking.** Cinnamon
puts the badge in the window list, not only in a notification tray. LitricOS
puts the count on the bell in the top bar — one place, and not the place a user
who is working in a window is looking.

**L26.12 — Mint writes its failures down as release-note content.** A 10-second
shutdown timeout, ZFS removed, ecryptfs regression, PipeWire fallback commands,
Snap disabled. LitricOS has no release-notes surface and no known-issues
document in the product; its failures arrive as one sentence at the moment they
happen and are never collected.

---

## L.3 Cross-batch comparison

| Question | Xfce 4.20 | Cinnamon 6.6 | LitricOS |
|---|---|---|---|
| Wayland compositor | External (Labwc/Wayfire); xfwm4 is X11-only | Muffin, improving | Own, 3 globals |
| Launcher | Appfinder, keyboard-first, per-app actions | Menu applet, sidebar layout, configurable | Overlay, ranked list, fixed frame |
| File manager | Thunar: tabs, split, overflow, bulk rename, undo-with-confirm | Nemo: pause/resume, templates, split panes | Flat table, 26 verbs, no cancel |
| Settings | Component dialogs (Appearance, Display, Mouse, MIME, Keyboard) | Cinnamon Settings modules (not enumerated this pass) | 13 pages, 9 keys |
| Window manager | xfwm4 with focus models and snap distance | Muffin with per-monitor Alt-Tab | 36 ops, edge snapping |
| Notifications | libnotify optional; daemon page unread | Badge in the window list | Queue, banners, centre, DND |
| Design documentation | Tour + changelog, no HIG located | Release notes, no HIG; XSI icon project | Tokens + lint, no principles document |
| What it will not do | Ship a Wayland compositor | Ship a tiling model | Ship a second process for anything |

The comparison row that matters for LitricOS is the first and the last: both
environments delegate something LitricOS internalises (compositor in Xfce,
session/base system in Mint), and LitricOS' single-image design is what makes its
gaps read as missing services rather than missing features.

---

## L.4 What to change in LitricOS because of this part

| # | Change | Why (row) | Size |
|---|---|---|---|
| 1 | Add a toolbar overflow menu to Files and Trash instead of dropping buttons | `LX.05`, L26.03 | S |
| 2 | Add pause/resume to file operations, and a cancel verb before that | `LC.05`, L26.07 | L |
| 3 | Add a split view with a focused pane and a close confirmation when tabs would be lost | `LX.05`, L26.04 | L |
| 4 | Add tabs to Files, restoring them with the window | `LX.05` | M |
| 5 | Add a device inventory page (USB, GPU, PCI, BIOS) under Diagnostics | `LC.04`, L26.10 | M |
| 6 | Show a notification badge on the dock's app button, not only on the bell | `LC.07`, L26.11 | S |
| 7 | Add a keyboard-layout surface that lists layouts and input methods together | `LC.08`, L26.08 | M |
| 8 | Keep a known-issues document in the image and surface it from Diagnostics | `LC.12`, L26.12 | S |
| 9 | Confirm inside undo, the way Thunar does, when undo would delete | `LX.05`, L26.02 | — (needs undo first) |

Two of these are small and independent (1, 6). Two are the same large piece of
work seen from different environments (2, 3), and Part 22 already named it: the
Files application cannot stop what it started.

---

## L.5 Limitations of this batch

* **Four environments of eight.** COSMIC, Pantheon, Budgie and ChromeOS remain
  PENDING; Chapter L is 56 of 112 rows.
* **No HIG for either environment.** Xfce publishes behaviour, Mint publishes
  release notes. `LX.10` and `LC.10` are therefore measured from icons, themes
  and changelog entries, not from interface rules.
* **No Cinnamon Settings inventory.** The `projects.linuxmint.com` host did not
  resolve, and no user guide was opened, so `LC.04` is a list of touched modules,
  not a catalogue.
* **Nothing was run.** No Xfce or Cinnamon session was started; the DE column is
  documentation, the LitricOS column is code in this repository.
* **Xfce's version line is unusual.** 4.20 is the stable release, while Thunar's
  own documentation page advertises a 4.21.7 development release; this part cites
  4.20 behaviour and says so.

---

## L.6 Sources read in this part

| ID | URL | What it gave |
|---|---|---|
| S3861 | https://www.xfce.org/about/tour420/ | Xfce 4.20 user-visible changes and Wayland caveats |
| S3862 | https://www.xfce.org/download/changelogs/4.20/ | Component change list: Thunar, panel, settings, session, xfconf |
| S3863 | https://docs.xfce.org/xfce/xfwm4/preferences | xfwm4 style, keyboard, focus and advanced options |
| S3864 | https://docs.xfce.org/xfce/thunar/start | Thunar scope, plugins, standards, release line |
| S3865 | https://linuxmint.com/rel_zena_whatsnew.php | Cinnamon 6.6, Nemo 6.6, applets, menu, XSI, known changes |
| S3866 | https://linuxmint.com/rel_zena.php | Mint 22.3 known issues, base system, removed features |

Unreachable, recorded and not cited: `projects.linuxmint.com` (`ENOTFOUND`).
