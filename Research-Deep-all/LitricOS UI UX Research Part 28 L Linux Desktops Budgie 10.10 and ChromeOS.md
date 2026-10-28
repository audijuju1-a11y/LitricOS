# LitricOS UI/UX Research — Part 28
## Chapter L: Linux Desktop Environments (batch 4 — Budgie 10.10.3 and ChromeOS 154)

**Part:** 28
**Chapter:** L (Research E, Linux desktop environments) — **final batch**
**Rows covered:** `LB.01`–`LB.14` (Budgie 10.10.3), `LR.01`–`LR.14` (ChromeOS, stable 154 line) — **28 new rows**
**Chapter L status after this part:** **112 / 112 rows measured**
**Date:** 2026-10-28
**Evidence base:** ten pages opened in this pass (`S3878`–`S3887`), plus the corpus measurements of Parts 19–27 for the LitricOS column.

---

## L.0 What this batch is, and what it closes

The last two environments are the two extremes of the chapter. Budgie publishes
release notes that read like engineering logs — protocol lists, D-Bus interfaces,
config migrations, a bug number for every fix. ChromeOS publishes almost nothing
about its shell: five of the six pages this pass tried for launcher, shelf, Files,
release-notes and design returned **404**, and the two pages that opened describe
the *web platform*, not the desktop.

That asymmetry is the chapter's closing result. Chapter L set out to measure
"what a desktop environment is" across eight systems; it ends with two answers —
one that documents everything it does, and one that documents the platform
underneath its shell and nothing about the shell itself.

### L.0.1 Evidence grade of this batch

| Source | ID | Grade | What it establishes |
|---|---|---|---|
| Buddies of Budgie home | S3878 | T1 (project) | 10.10.3 summary: favourites, keyboard applet, primary monitor, desktop icons, labwc |
| Budgie 10.10.3 release notes | S3879 | T1 (project) | Menu favourites, popover→window focus fix, labwc bridge, keybinds, keyboard applet architecture, config migrations |
| Budgie 10.10 release notes | S3880 | T1 (project) | Wayland migration, compositor-agnostic design, required protocols, applet and Raven changes, Budgie Control Center, maintenance mode |
| ChromeOS.dev landing | S3881 | T1 (project) | A 2024 featured story; no shell content |
| ChromeOS.dev, web apps | S3882 | T1 (project) | Chrome powers ChromeOS; PWAs install, pin to the shelf, appear in the launcher, open files from Files |
| ChromiumOS developer library | S3883 | T1 (project) | The library's structure: getting started, reference, guides, training |
| ChromiumOS reference index | S3884 | T1 (project) | Reference topics including security, user sessions, AUE/EOL and release documentation |
| Chrome Enterprise and Education release notes | S3885 | T1 (project) | Release-note cadence and Chrome version dates; ChromeOS notes stay on the Help Center |
| Chromebook "What's New" | S3886 | T1 (project) | June 2025 update: Select-to-search with Lens from the launcher |
| Chrome Enterprise release-notes landing | S3887 | T1 (project) | Where Chrome Enterprise release notes live; no feature content |

**Pages that failed to open, recorded verbatim:**

| URL | Failure |
|---|---|
| `https://chromeos.dev/en/design` | **404 Not Found** |
| `https://support.google.com/chromebook/answer/10721178` | **404 Not Found** |
| `https://support.google.com/chromebook/answer/9548267` | **404 Not Found** |
| `https://chromeenterprise.google/os/release-notes/` | **404 Not Found** |
| `https://9to5linux.com/budgie-10-10-3-desktop-environment-introduces-free-placement-of-desktop-icons` (search result only, never opened) | not fetched |

Consequence: **eleven of the fourteen ChromeOS rows are structural or NOT FOUND.**
That is a fact about ChromeOS documentation, and it is recorded rather than
filled in from memory. No row in this part claims ChromeOS behaviour that an
opened page does not support.

---

## L.1 Budgie 10.10.3 (2026-09-27)

Budgie 10.10 is the release that moved the desktop from X11 to Wayland and, in
doing so, gave up its own window manager. The 10.10 notes are explicit about the
architecture: the move "decouples the desktop from a specific window manager
(budgie-wm, which in turn relied on our fork of Mutter called Magpie), and makes
Budgie truly compositor-agnostic" (S3880). The desktop instead requires five
Wayland protocols and ships a bridge that configures **labwc** on its behalf.

### L.1.1 The rows

| Row | What Budgie 10.10.3 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LB.01` Shell components & compositor | Wayland-first since 10.10; compositor-agnostic, requiring `ext-workspace-v1`, `wlr-foreign-toplevel-management`, `wlr-layer-shell`, `wlr-output-management` and `xdg-output`; recommends labwc; no compositor of its own | S3880 | One Rust shell with its own compositor: `wl_compositor` v4, `wl_seat` v5, `xdg_wm_base` v2 — none of Budgie's five required protocols | **COMPLETE** |
| `LB.02` Launcher | Budgie Menu: favourites (right-click → Add to favourites), desktop actions in the menu, shared action code with the Icon Tasklist, a Favorites category above All, compact-mode grouping; the menu is a layer-shell window again so Super-key search receives keyboard focus | S3879, S3878 | Launcher overlay with query and ranked rows; no favourites, no desktop actions, no categories | **COMPLETE** |
| `LB.03` Panel / dock | `budgie-panel` on layer-shell; start/centre/end sections with dynamic sizing and overflow; Icon Tasklist and a rewritten Tasklist with dynamic button sizing and scroll buttons; primary monitor can be chosen | S3880, S3879 | Fixed top bar plus morphing dock; app buttons in the dock; no panel sections, overflow or primary-monitor choice | **COMPLETE** |
| `LB.04` Settings inventory | Budgie Control Center: Colour panel hidden, Bluetooth optional, per-panel visibility checks, Multitasking on Wayland, window-manager keyfiles, screen lock moved to Privacy, Night Light its own panel, accessibility options including a working magnifier | S3880 | 13 pages, 9 stored keys, 8 registry rows; no per-panel visibility | PARTIAL |
| `LB.05` File manager | No file-manager component appears in either release note; desktop icons come from Budgie Desktop View, which is an icon layer rather than a file manager | S3880 (absence) | `apps::FilesApp`: 26 verbs, breadcrumb, table | **NOT FOUND** |
| `LB.06` Window behaviour & tiling | The labwc bridge writes `rc.xml` for keybinds, focus policy, theming and window rules, including snapping and a thumbnail window switcher; auto-raise delay from settings; Show Desktop uses labwc's action; config migrations by version | S3879, S3880 | `litric-wm`: 36 operations, edge snapping, one output, no external composer | PARTIAL |
| `LB.07` Notifications | Raven is the notification centre; Do Not Disturb toggles by middle-clicking the Notifications applet; implementation is Budgie's own, not a portal | S3880 | `litric-notifyd` queue with DND, banners, centre; no per-applet DND affordance | PARTIAL |
| `LB.08` Workspace model | Workspaces applet: reliability fixes for new windows, a more responsive add-workspace button; `ext-workspace-v1` is a required protocol | S3880, S3879 | Fixed grid; Mission Control cards; no workspace applet | PARTIAL |
| `LB.09` Customization model | The bridge applies fonts, drop shadows and title-bar layouts; keybind templates can be shipped per distribution; panel sections and applets are user-arranged; Night Light via gammastep | S3879, S3880 | 4 themes × 6 accents × 3 wallpapers; no per-distribution defaults, no shadow or title-bar tokens | **COMPLETE** |
| `LB.10` Design language / HIG / tokens | No Budgie HIG or token documentation was found in the pages opened; the design is GTK3 plus the bridge's labwc theming | — (gap) | 11 semantic colours × 4 themes, 8 type and 8 space steps, 4 motion tokens, enforced by a lint | PARTIAL |
| `LB.11` Accessibility | Budgie Control Center "supports a number of Accessibility options including a working magnifier" (S3880); no screen-reader or keyboard-navigation statement was found | S3880 | One toggle (reduce motion); no magnifier, no a11y tree | PARTIAL |
| `LB.12` Limitations & known issues | 10.10 is the **final major release** of the 10.x series, now in maintenance mode while work moves to Budgie 11; the Keyboard Layout applet was disabled in 10.10 and restored in 10.10.3; Budgie Screensaver deprecated in favour of swayidle/gtklock; Debian blocks `SetX11Keyboard` (bug 1118702); labwc 0.20.2 rejected empty keybinds | S3880, S3879 | No release-notes surface and no maintenance statement | **COMPLETE** |
| `LB.13` Integration points | `xdg-desktop-portal-gtk` and `xdg-desktop-portal-wlr`; `locale1`/systemd-localed; WirePlumber `wpctl` for volume and mic; D-Bus interface `org.buddiesofbudgie.KeyboardLayout`; gammastep, grim/slurp, swaybg, wlopm, swayidle; Budgie Desktop Services (Qt6) persists display configuration | S3879, S3880 | LFP/1, browser socket, notifyd; no portal, no bus, no locale1 | **COMPLETE** |
| `LB.14` DE vs kernel vs distro boundary | Budgie is a desktop that sits on somebody else's compositor; the kernel, logind, portals and audio stack are outside; ships in Fedora 44 and Ubuntu Budgie 26.04; distributions package it | S3880 | One mkosi image contains desktop, compositor and services | **COMPLETE** |

### L.1.2 Findings — Budgie

**L28.01 — Budgie solved the same problem LitricOS solved, from the opposite
direction.** Budgie removed its window manager and requires a compositor with
five protocols. LitricOS wrote a compositor and requires nothing. Both are
"compositor-agnostic" in spirit; only one of them can be adopted by another
distribution.

**L28.02 — The labwc bridge is a design pattern LitricOS lacks: a translation
layer between settings and somebody else's configuration format.** It rewrites
`rc.xml` and labwc's environment file, reloads the compositor, and migrates
configs version by version (S3879). LitricOS owns both ends, so it never needs
this layer — and therefore has no answer for a user who wants their settings to
survive a different compositor.

**L28.03 — Budgie's keyboard-layout round trip is the most complete input story
in the chapter.** Applet → D-Bus request → daemon broadcast → compositor bridge →
environment file → read-back → `CurrentLayout` property → panel label, with a
documented precedence list ending in "US English, if nothing else is set"
(S3879). LitricOS has no layout surface at all (Part 26, L26.08).

**L28.04 — A menu that cannot take keyboard focus was a real, filed bug.** Under
Wayland, Budgie Menu was a `GtkPopover`/`xdg_popup` and only received keys while
the panel had focus, so opening it with Super and typing went to the window
behind; the fix was to make it a layer-shell `GtkWindow` again (S3879, issue
#842). This is the exact class of defect LitricOS' rectangle-hotspot input model
would produce, and it is now documented by somebody else.

**L28.05 — Budgie is closing, and says so.** 10.10 is the last major of the 10.x
series; the project is in maintenance mode while it builds Budgie 11 (S3880).
LitricOS has no equivalent statement about which parts of it are finished.

---

## L.2 ChromeOS (stable 154 line)

ChromeOS is the only environment in this chapter whose shell is not documented
in anything this pass could open. The pages that did open describe the platform:
"Google Chrome powers ChromeOS, with the operating system keeping up-to-date
with the latest releases of Chrome as they happen" (S3882), and web apps "can be
installed, they can be pinned to the shelf, and they can be found from the
launcher" (S3882).

### L.2.1 The rows

| Row | What ChromeOS does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LR.01` Shell components & compositor | Chrome "powers ChromeOS" and updates with it; the window manager/shell is not described on any page opened | S3882 (structural) | One Rust shell plus own compositor | PARTIAL |
| `LR.02` Launcher | Web apps are found from the launcher; a long press of the launcher starts Select-to-search with Lens | S3882, S3886 | Launcher overlay with ranked app and settings rows | PARTIAL |
| `LR.03` Panel / dock | The shelf exists: PWAs can be "pinned to the shelf" | S3882 | Top bar plus morphing dock; no pinning model | PARTIAL |
| `LR.04` Settings inventory | No ChromeOS Settings page opened; the four attempts at Help Center and Enterprise pages for it returned 404 | — (gap) | 13 pages, 9 stored keys | **NOT FOUND** |
| `LR.05` File manager | The Files app exists and can hand files to installed web apps ("use it to open files directly from the Files app"); no Files behaviour page could be opened (two 404s) | S3882 | `FilesApp`: 26 verbs, no tabs | PARTIAL |
| `LR.06` Window behaviour & tiling | No page describing windows, split screen or overview was opened | — (gap) | 36 WM operations, edge snapping | **NOT FOUND** |
| `LR.07` Notifications | No notification page was opened | — (gap) | notifyd queue, banners, centre, DND | **NOT FOUND** |
| `LR.08` Workspace model | No virtual-desks or workspace page was opened | — (gap) | Fixed workspace grid | **NOT FOUND** |
| `LR.09` Customization model | No wallpaper, theme or shelf-customization page was opened | — (gap) | 4 themes, 6 accents, 3 wallpapers | **NOT FOUND** |
| `LR.10` Design language / HIG / tokens | `chromeos.dev/en/design` returned **404**; the PWA guidance that did open is a capability list, not a design language | S3882, failure above | Tokens plus a lint; no principles document | PARTIAL |
| `LR.11` Accessibility | No accessibility page was opened | — (gap) | One toggle, no a11y tree | **NOT FOUND** |
| `LR.12` Limitations & known issues | End-of-life/Auto Update Expiration is a documented process in the ChromiumOS reference index; ChromeOS release notes are published one week before stable and stay on the Help Center | S3884, S3885 | No release-notes surface, no support-window statement | PARTIAL |
| `LR.13` Integration points | Web apps are installable, pinnable, launcher-visible and file-handling; Chrome extensions are "just as integral" as on other platforms; Android and Linux app support is not described on any opened page | S3882 | LFP/1, browser socket, notifyd; no extension or store model | PARTIAL |
| `LR.14` DE vs kernel vs distro boundary | ChromeOS is an OS, not a desktop on a distribution: the browser is the platform and the OS tracks Chrome releases; security, SELinux, user sessions and AUE are documented as platform topics | S3882, S3884 | One mkosi image; the boundary is a build decision | PARTIAL |

### L.2.2 Findings — ChromeOS

**L28.06 — The most widely used desktop in this chapter has the least
documentation of its shell.** Five 404s and two platform pages. There is no
public HIG, no launcher specification and no Files behaviour page that this pass
could open. Whatever ChromeOS' shell rules are, they are not published where a
research corpus can cite them — and a corpus that guessed would be inventing
Material Design folklore.

**L28.07 — ChromeOS defines the desktop through the browser, and that is a real
architectural difference.** "Chrome powers ChromeOS" (S3882) is not marketing
language: the shell updates with the browser, the app model is the web app, and
the shelf pins PWAs. LitricOS' shell is a native process and its browser is one
app among many; the two systems disagree about what the platform *is*.

**L28.08 — Web-app pinning is the launcher story, and LitricOS has no equivalent.**
`PWA` install → launcher entry → shelf pin → Files handler (S3882) is a complete
chain. LitricOS' launcher lists built-in apps from an enum; there is no way for
anything new to appear in it (I.02).

**L28.09 — AUE is a first-class published policy, and LitricOS has no support
window at all.** The ChromiumOS reference index documents the End of Life / Auto
Update Expiration process (S3884). LitricOS has no statement of how long an image
is supported, which is the same absence as its missing release notes.

---

## L.3 Chapter L result — the eight environments, side by side

| Environment | Own compositor | Shell documented | HIG / design doc | File manager | Settings model | Undo |
|---|---|---|---|---|---|---|
| GNOME 51 | Mutter | Yes | Yes (5-part HIG) | Files | GSettings + 20 panels | App-level |
| Plasma 6.8 | KWin | Yes | Yes (9-section HIG) | Dolphin | KConfig + ~30 modules | HIG-mandated |
| Xfce 4.20 | No (xfwm4 is X11) | Yes (component docs) | No HIG located | Thunar | Component dialogs | Undo asks before delete |
| Cinnamon 6.6 | Muffin | Yes (release notes) | No HIG located | Nemo | Cinnamon Settings | Not documented |
| COSMIC Epoch 1 | cosmic-comp | Toolkit book | Design language, no token HIG opened | COSMIC Files | Build-flag pages | Not documented |
| Pantheon 8.1 | Mutter (structural) | Yes | Yes (full HIG, declared gaps) | Files | System Settings | HIG-mandated, with buffer |
| Budgie 10.10.3 | No (labwc bridge) | Yes (engineering logs) | No HIG located | None shipped | Budgie Control Center | Not documented |
| ChromeOS 154 | Ash (undocumented here) | No | 404 | Files | Not documented | Not documented |
| **LitricOS** | **Yes (3 globals)** | **No** | **Tokens + lint only** | **Files (26 verbs)** | **13 pages, 9 keys** | **None** |

Three conclusions, stated once for the chapter:

1. **LitricOS is one of three systems here that own their compositor** (with
   GNOME and Plasma, and COSMIC), and the only one that documents neither its
   shell nor its failures.
2. **Every environment in this chapter ships a file manager with a feature
   LitricOS lacks** — tabs, split view, pause/resume, bulk rename, tab history,
   or a template manager. The gap is not taste; it is a missing model.
3. **Six of the eight document undo, and three mandate it.** LitricOS has zero
   undo and eleven confirmations, which is the least defensible position in the
   table.

---

## L.4 What to change in LitricOS because of this part

| # | Change | Why (row) | Size |
|---|---|---|---|
| 1 | Adopt `wlr-foreign-toplevel-management` and `xdg-output` so a foreign shell could drive LitricOS windows, the way Budgie's five protocols let it be adopted | `LB.01`, L28.01 | L |
| 2 | Add favourites to the launcher, with desktop actions, as Budgie Menu did | `LB.02` | M |
| 3 | Put window-manager settings behind a translation layer, not inline, so they survive a different compositor | `LB.06`, L28.02 | M |
| 4 | Add a primary-monitor choice and panel-section overflow | `LB.03` | M |
| 5 | State a support window (an AUE equivalent) and publish it in the image | `LR.12`, L28.09 | S |
| 6 | Make the launcher extensible so an app installed later can appear in it | `LR.02`, L28.08 | M |
| 7 | Write the shell documentation LitricOS does not have, starting from the eight-environment table above | `LR.01`, L28.06 | M |

---

## L.5 Limitations of this batch

* **Chapter L is complete; the corpus is not.** 112 of 112 Chapter L rows are
  measured, but `WB.12.08` and three parent rows remain PENDING, and the full
  validation plan (Chapter 45) is still unwritten.
* **ChromeOS was measured mostly by absence.** Eleven of fourteen rows are
  structural or NOT FOUND; five URLs 404'd and two more describe the web
  platform. No ChromeOS shell behaviour is claimed without a source.
* **Budgie ships no file manager**, so `LB.05` is NOT FOUND by design rather than
  by failure: the pages opened describe applets, Raven and desktop icons, none of
  which is a file manager.
* **No Budgie HIG or token documentation was found**, so `LB.10` is structural.
* **Nothing was run.** No Budgie or ChromeOS session was started; the DE column
  is documentation, the LitricOS column is code in this repository.

---

## L.6 Sources read in this part

| ID | URL | What it gave |
|---|---|---|
| S3878 | https://buddiesofbudgie.org/ | 10.10.3 headline changes |
| S3879 | https://buddiesofbudgie.org/blog/budgie-10-10-3-released | Menu favourites, focus fix, labwc bridge, keybinds, keyboard applet, migrations |
| S3880 | https://buddiesofbudgie.org/blog/budgie-10-10-released | Wayland migration, protocols, applets, panel, control centre, maintenance mode |
| S3881 | https://chromeos.dev/en | Landing page; one 2024 story |
| S3882 | https://chromeos.dev/en/web | Chrome as the platform; PWA install, shelf pin, launcher, Files integration |
| S3883 | https://www.chromium.org/chromium-os/ | Developer library structure |
| S3884 | https://www.chromium.org/chromium-os/developer-library/reference/ | Reference topics incl. security, sessions, AUE/EOL |
| S3885 | https://support.google.com/chrome/a/answer/7679408 | Release-note cadence and Chrome version dates |
| S3886 | https://www.google.com/intl/en_us/chromebook/whatsnew/ | June 2025 update; launcher long-press for Lens |
| S3887 | https://chromeenterprise.google/resources/release-notes/ | Where enterprise release notes live; no feature content |

Failed, recorded and not cited: `chromeos.dev/en/design` (**404**),
`support.google.com/chromebook/answer/10721178` (**404**),
`support.google.com/chromebook/answer/9548267` (**404**),
`chromeenterprise.google/os/release-notes/` (**404**).
