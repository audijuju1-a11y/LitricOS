# LitricOS UI/UX Research Encyclopedia — Part 1 of N

Baseline verification, master coverage checklist, and research/source plan

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P01 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-25 (Asia/Bangkok) |
| Scope of this part | Chapter 0 (version baseline), Chapter 1 (master coverage checklist), Chapter 2 (research and source plan), Chapter 3 (initial release-delta findings), Chapter 4 (source register status), Chapter 5 (gaps and contradictions found so far), Chapter 6 (validation plan skeleton) |
| Companion files | `litricos_coverage_checklist.csv` (568 rows), `litricos_source_register.csv` (94 URLs) |
| Resume point | Part 2 → Batch B1 (WB: Windows 11 26H2 Settings, full inventory) |

Status vocabulary used throughout: COMPLETE, PARTIAL, NOT FOUND, INACCESSIBLE, NOT APPLICABLE. Items that research has not reached yet are marked PENDING. PENDING means "not yet researched". It is not a sixth final status, and every PENDING item must become one of the five final statuses before its subsystem can be declared complete.

---

## Master Index

| Ch. | Title | Deliverable # | Status |
|---|---|---|---|
| 0 | Version baseline verification | 8 | COMPLETE (10 items), PARTIAL (2 items) |
| 1 | Master coverage checklist | 8 | PUBLISHED (structure only; research items PENDING) |
| 2 | Research and source plan | 8 | PUBLISHED |
| 3 | Initial release-delta findings (26H2, macOS 27, GNOME 51, Plasma 6.8, COSMIC 1.11) | 1 | PARTIAL |
| 4 | Source register status | 8 | IN PROGRESS (94 URLs / 29 domains) |
| 5 | Missing, uncertain, contradictory evidence log | 9 | IN PROGRESS |
| 6 | Validation plan skeleton | 10 | DRAFT |
| 10 | Windows 11 shell and system UI (WA) | 1, 5 | PENDING (Part 3) |
| 11 | Windows 11 Settings catalogue (WB) | 2 | PENDING (Part 2) |
| 12 | Fluent / WinUI design system (WC) | 3, 4 | PENDING |
| 20 | macOS 27 desktop model (MC) | 1, 5 | PENDING |
| 21 | macOS 27 System Settings catalogue (MD) | 2 | PENDING |
| 22 | macOS HIG / Liquid Glass design system (MG) | 3, 4 | PENDING |
| 30–37 | GNOME, KDE Plasma, Xfce, Cinnamon, COSMIC, Pantheon, Budgie, ChromeOS (LG–LR) | 1, 2 | PENDING |
| 40 | Shared component catalog (F) | 3 | PARTIAL (Parts 10–12: `F.01`, `F.03`, `F.06`, `F.19`, `F.02`, `F.18`, `F.23`, `F.26`, `F.04`, `F.05` researched; 17 items `PENDING`) |
| 41 | Design tokens and accessibility (G) | 4 | PENDING |
| 42 | User workflows and edge cases (H) | 5 | PENDING |
| 43 | Technical integration and LitricOS feasibility (I) | 6, 7 | PENDING |
| 44 | Final source register and coverage audit (J) | 8, 9 | PENDING |
| 45 | Validation plan (full) | 10 | PENDING |

---

## Chapter 0 — Version Baseline (verified 2026-10-25)

Rule: the "selected release" for each platform is the latest generally available (GA) stable release. Betas, release candidates and Insider builds are recorded but are not the research target.

| ID | Platform | Selected release | Build / identifier | GA date | Evidence | Status |
|---|---|---|---|---|---|---|
| CH00.01 | Windows 11 | Version 26H2 ("Windows 11 2026 Update"), General Availability Channel | OS build 26300.9711 (KB5127648, 2026-10-13) | 2026-09-29 | [Microsoft release information](https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information); [26H2 update history](https://support.microsoft.com/en-us/servicing/os/windows-11/2026/09/windows-11-version-26h2-update-history) | COMPLETE |
| CH00.02 | Windows 11 26H1 | Not selected: limited to new devices | 28000.3231 | 2026-02-10 | [Microsoft release information](https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information) says 26H1 "is not offered as an in-place update from 24H2 or 25H2" | COMPLETE |
| CH00.03 | macOS | macOS 27.0.1 "Golden Gate" | 26A434 | 2026-09-28 (27.0 was 2026-09-14) | [Apple Developer releases](https://developer.apple.com/news/releases/); [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) | COMPLETE |
| CH00.04 | macOS (marketing) | — | — | — | The fetched copy of [apple.com/macos](https://www.apple.com/macos/) still described macOS Tahoe (26), so it is treated as stale and is not cited for macOS 27 claims | PARTIAL |
| CH00.05 | GNOME | GNOME 51 "A Coruña" (51.1 point release) | GTK 4.23, Libadwaita 1.10 | 2026-09-16 | [GNOME 51 release notes](https://release.gnome.org/51/); [GNOME calendar](https://release.gnome.org/calendar/) | COMPLETE |
| CH00.06 | KDE Plasma | Plasma 6.8.1 | Frameworks 6.31.0; Gear 26.08.2 | 6.8.0 on 2026-10-14; 6.8.1 on 2026-10-20 | [KDE announcements](https://kde.org/announcements/); [Plasma 6.8](https://kde.org/announcements/plasma/6/6.8.0/) | COMPLETE |
| CH00.07 | Xfce | Xfce 4.20 | — | 2024-12-15 | [xfce.org download](https://xfce.org/download/?layout=normal) (snippet: "4.20 is the most recent version") | COMPLETE (re-fetch planned) |
| CH00.08 | Cinnamon | Linux Mint 22.3 (current); Mint 27 "Adrien" scheduled for Christmas 2026 | Cinnamon version not yet verified | — | [Linux Mint blog](https://blog.linuxmint.com/) | PARTIAL |
| CH00.09 | COSMIC | COSMIC Epoch 1.11.0 | tag epoch-1.11.0, commit e98f828 | 2026-10-20 | [pop-os/cosmic-epoch releases](https://github.com/pop-os/cosmic-epoch/releases/) | COMPLETE |
| CH00.10 | Pantheon | elementary OS 8.1.1 "Circe" | gala 8.6.2, files 7.3.3, dock 8.3.3, wingpanel 8.0.4, settings 8.0.3 | 2026-02-26 | [elementary releases tracker](https://releases.elementary.io/) | COMPLETE |
| CH00.11 | Budgie | Budgie 10.10.3 | v10.10.3 (ec56162) | 2026-09-27 | [Buddies of Budgie](https://buddiesofbudgie.org/) | COMPLETE (re-fetch planned) |
| CH00.12 | ChromeOS | Stable 16805.33.0 | Browser 154.0.8037.151 | 2026-10-09 | [Chrome Releases](https://chromereleases.googleblog.com/2026/10/stable-channel-update-for-chromeos.html). A third-party report says ChromeOS 156 reached Chromebooks on 2026-10-20 ([ChromeGeek](https://www.chromegeek.com/chrome-156-stable-whats-new/)); this is not yet confirmed by an official source | COMPLETE (with a version-drift note) |

Baseline notes:
- Windows 26H2 shares a servicing foundation with 24H2 and 25H2. It is delivered as an enablement package that "enables selected features," according to [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2). Because builds 26100, 26200 and 26300 receive the same cumulative update, many features differ by enablement state or rollout rather than by binary. Every Windows Settings item will therefore record a rollout/SKU flag.
- macOS 27 runs only on Apple silicon and Macs with the A18 Pro chip, according to [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate). Settings that only existed on Intel Macs are therefore NOT APPLICABLE for the selected release.
- Plasma 6.8 retires Plasma X11 sessions ([KDE](https://kde.org/announcements/plasma/6/6.8.0/)). KDE research targets Plasma Wayland only; X11 is recorded as a legacy difference.

---
## Chapter 2 — Research and Source Plan

### 2.1 Evidence hierarchy and confidence rubric

| Tier | Evidence type | Examples | Default confidence |
|---|---|---|---|
| T1 | Official documentation, UI guidelines, API references, release notes | learn.microsoft.com, support.microsoft.com, developer.apple.com, support.apple.com, help.gnome.org, developer.gnome.org, userbase.kde.org, develop.kde.org, docs.xfce.org, docs.elementary.io, support.google.com/chromebook | High |
| T2 | Source code and issue trackers | github.com/microsoft (WinUI, PowerToys, microsoft-ui-xaml), gitlab.gnome.org, invent.kde.org, gitlab.xfce.org, github.com/linuxmint, github.com/pop-os, github.com/elementary, github.com/BuddiesOfBudgie, chromium.googlesource.com | High (implementation behavior); Medium (intent) |
| T3 | Standards and protocols | wayland.app, freedesktop.org specs, W3C WCAG/ARIA, Unicode bidi | High |
| T4 | Reproducible independent demonstrations | Phoronix, Ars Technica, OSnews, MacRumors, Windows Latest, Neowin, 9to5Mac, OMG! Ubuntu, YouTube walkthroughs with visible build numbers | Medium |
| T5 | Community and forums | Microsoft Q&A, Apple Community, GNOME Discourse, KDE Discuss, Reddit, Stack Exchange | Low (used to locate contradictions; never the only source for a material claim) |
| T6 | Search snippet only | — | Not counted as inspected; never cited as sole evidence |

Confidence is High when there is T1/T2 evidence for the exact version, Medium when there is T1 evidence for an adjacent version or two concordant T4 sources, and Low when there is a single T4/T5 source. Measured values (from documentation or code tokens) are tagged MEASURED. Values derived from screenshots or pixel inspection are tagged ESTIMATE together with the method used (e.g. "pixel count at 100% scale from a cited screenshot").

### 2.2 Batch plan

| Batch | Part | Workstream | Primary inputs | Exit criteria |
|---|---|---|---|---|
| B1 | 2 | WB: Windows Settings full inventory (11 categories + Beyond Settings) | Microsoft Support "Settings" articles per page, ms-settings URI table, Windows IT-pro docs, Policy CSP docs | Every WB leaf has page path, controls, URI, SKU/rollout flag, status |
| B2 | 3 | WA: Windows shell, Explorer, dialogs, OOBE, UAC | Microsoft Support, Win32 docs (IFileDialog, UAC), Insider blog, PowerToys (third-party boundary) | Every WA leaf has behavior, state transitions, shortcuts |
| B3 | 4 | WC + G (Windows): Fluent 2 / WinUI 3 tokens, motion, materials | learn.microsoft.com/windows/apps/design, microsoft-ui-xaml theme resources (source), Fluent 2 site | Token tables MEASURED vs ESTIMATE |
| B4 | 5 | MD: macOS 27 System Settings inventory | Mac User Guide (27) per pane, Apple Platform Deployment, MDM payload docs, URL schemes | Sidebar enumerated; every pane leaf status |
| B5 | 6 | MC: macOS desktop model | Mac User Guide, HIG, AppKit docs, keyboard-shortcut KB | Every MC leaf |
| B6 | 7 | MG + G (macOS): HIG, Liquid Glass, SF, motion | developer.apple.com/design, AppKit/SwiftUI API | Token tables |
| B7 | 8 | LG + LK: GNOME 51, Plasma 6.8 | help.gnome.org, GNOME HIG, Libadwaita docs, gnome-control-center source; KDE UserBase, KDE HIG, kcm source | Each DE: 14 checklist rows resolved |
| B8 | 9 | LX, LC, LO, LP, LB, LR | docs.xfce.org, Mint/Cinnamon docs and GitHub, COSMIC repos, elementary HIG/docs, Budgie docs, ChromeOS Help Center | Same |
| B9 | 10 | F: Component catalog across all systems | Outputs of B3, B6, B7 and B8 + control API docs | One spec sheet per component |
| B10 | 11 | H: Workflows and edge cases | Per-OS support articles, recovery documentation | Success/failure/cancel/retry/rollback for each workflow |
| B11 | 12 | I: Integration, comparison matrix, ADOPT/ADAPT/AVOID/INVESTIGATE decisions | API docs, Wayland protocols, portals, polkit, TCC, UAC | Every decision has evidence, cost, security, test |
| B12 | 13 | J + validation plan: final register and audit | All | Counts reconciled; every leaf has a final status |

Each batch ends with a completeness audit table. The audit lists leaf IDs, final statuses, reasons, and newly opened gaps.

### 2.3 Domain-coverage strategy toward the 400-domain target

The target is at least 400 distinct registrable domains (eTLD+1). The register counts these separately from unique URLs and from inspected pages. This is a target only: it will be reported as met only when the register shows 400 distinct domains, and the number of inspected domains will be reported alongside it. Planned domain pools:

| Pool | Planned examples (to be verified as they are used) | Rationale |
|---|---|---|
| OS vendors and projects | microsoft.com, windows.com, apple.com, gnome.org, kde.org, xfce.org, linuxmint.com, system76.com, elementary.io, buddiesofbudgie.org, google.com, chromium.org, freedesktop.org, wayland.freedesktop.org | T1 |
| Distributions (to separate distro additions from DE features) | ubuntu.com, fedoraproject.org, debian.org, archlinux.org, opensuse.org, manjaro.org, pop.system76.com, zorin.com, kubuntu.org, xubuntu.org, ubuntubudgie.org, solus-project.com, nixos.org, endlessos.org | Boundary evidence |
| Code hosts | github.com, gitlab.gnome.org, invent.kde.org, gitlab.freedesktop.org, gitlab.xfce.org, googlesource.com, sourceforge.net | T2 |
| Standards and accessibility | w3.org, unicode.org, section508.gov, a11yproject.com, webaim.org, levelaccess.com, deque.com, tpgi.com, nvaccess.org, freedomscientific.com | Accessibility research (G) |
| Hardware OEM support | dell.com, hp.com, lenovo.com, asus.com, acer.com, framework.com, logitech.com, wacom.com, intel.com, nvidia.com, amd.com | Hardware-dependent settings |
| Independent press | arstechnica.com, theverge.com, phoronix.com, osnews.com, omgubuntu.co.uk, 9to5linux.com, 9to5mac.com, macrumors.com, appleinsider.com, macworld.com, windowscentral.com, windowslatest.com, neowin.net, bleepingcomputer.com, howtogeek.com, pcworld.com, zdnet.com, tomshardware.com, itsfoss.com, lwn.net | T4 independent checks |
| Reference and community | wikipedia.org, stackexchange.com, superuser.com, askubuntu.com, reddit.com, discourse.gnome.org, discuss.kde.org, discussions.apple.com | Contradictions and undocumented behavior |
| Design systems and research | fluent2.microsoft.design, m3.material.io, nngroup.com, baymard.com, smashingmagazine.com, uxdesign.cc, acm.org, arxiv.org | Design principles and evidence |
| Localization/RTL and enterprise | unicode.org, mozilla.org, intune docs, jamf.com, kandji.io, mosyle.com, workspaceone | RTL, MDM settings |

Reaching 400 domains will require broad independent corroboration, particularly from OEM, accessibility-vendor, regional/localized and enterprise-MDM sources. Domain counts will be audited after each batch. If, after B12, the target is not reachable with relevant sources, the shortfall will be reported as-is; irrelevant domains will not be added to make up the count.

### 2.4 Separation rules applied to every finding

Each finding carries a `layer` tag:
- `SYS-UI`: system shell UI
- `FP-APP`: first-party application, e.g. Snipping Tool, Finder, Dolphin
- `DEV-GUIDE`: developer guideline
- `CONFIG`: configuration utility (Settings, Control Panel, Tweaks, dconf-editor)
- `3P`: third-party tool, e.g. PowerToys extensions, GNOME Shell extensions, Rectangle
- `DE`/`KERNEL`/`DISTRO`: Linux boundary

Example: COSMIC's new touchpad tap-and-drag and drag-lock toggles live in `cosmic-settings` (CONFIG) and depend on libinput (a userspace library, not the kernel) ([COSMIC Epoch 1.11](https://github.com/pop-os/cosmic-epoch/releases/)). The libinput attribution is a planned verification item, not yet evidenced.

### 2.5 Register schema (`litricos_source_register.csv`)

`source_id, url (normalized), domain (eTLD+1), host, title, publisher, pub_or_update_date, access_date, target_os_version, subsystem, evidence_type, primary_or_independent, status (INSPECTED | SNIPPET_ONLY | INACCESSIBLE), findings, confidence, limitations, checklist_ids`

URL normalization: https scheme, lowercase host, `www.` stripped, trailing slash and fragment removed, and query dropped unless it identifies a document (e.g. `?id=`). Domain counts use eTLD+1 (e.g. learn.microsoft.com and support.microsoft.com both count as microsoft.com).

---

## Chapter 3 — Initial Release-Delta Findings (inspected sources)

These findings come from the version-verification pass. They will be expanded and re-verified verbatim in the subsystem batches; until then each subsystem's checklist item remains PARTIAL or PENDING.

### 3.1 Windows 11 26H2 (target build 26300.9711)

| ID | Finding | Layer | Conditions / SKU | Source | Conf. |
|---|---|---|---|---|---|
| R-W-001 | 26H2 installs via an enablement package on 24H2/25H2, with a single restart in most scenarios | SYS | Eligible 24H2/25H2 devices | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-002 | Users opt in via Settings > Windows Update > "Get the latest updates as soon as they're available" | CONFIG | Consumer path | [Windows Experience Blog](https://blogs.windows.com/windowsexperience/2026/09/29/how-to-get-windows-11-2026-update/) | High |
| R-W-003 | The taskbar can be positioned bottom, top, left or right | SYS-UI | 26H2 | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-004 | A smaller-taskbar option reduces icon size and taskbar height | SYS-UI | 26H2 | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-005 | App-specific actions are available from the taskbar; enabled by default for commercial devices | SYS-UI | Was under temporary commercial controls | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-006 | AI agent monitoring from the taskbar; Researcher in Microsoft Copilot is the first supported experience, with progress and a completion notification | SYS-UI + FP-APP | Requires Copilot | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-007 | File Explorer: AI actions (edit images, summarize documents); hover actions "Open file location" and "Ask Copilot" on Home for work/school accounts | FP-APP (Explorer) | Summarize requires M365 + Copilot license | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-008 | Explorer adds archive formats uu, cpio, xar, nupkg | FP-APP | 26H2 | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-009 | View and Sort preferences are preserved in Downloads/Documents when apps launch Explorer directly to those folders | FP-APP | 26H2 | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-010 | Voice typing is available when renaming a file | FP-APP | 26H2 | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-011 | Windows settings backup is on by default for eligible commercial devices; first-sign-in restore supports Entra hybrid joined, Cloud PC and multi-user | CONFIG | Admin policies honored | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-26h2) | High |
| R-W-012 | Start has Small/Large sizes, and the Pinned, Recent or All sections can be hidden | SYS-UI | Independent report; official confirmation pending in B2 | [Windows Latest](https://www.windowslatest.com/2026/10/06/microsoft-reveals-whats-actually-new-in-windows-11-26h2-and-how-to-verify-it/) | Medium |
| R-W-013 | Search previews results, tolerates typos, labels result types, and web/Store suggestions can be turned off | SYS-UI | Same | [Windows Latest](https://www.windowslatest.com/2026/10/06/microsoft-reveals-whats-actually-new-in-windows-11-26h2-and-how-to-verify-it/) | Medium |
| R-W-014 | Settings has an AI agent for natural-language find/change | CONFIG | Hardware/region dependency unverified | [Windows Latest](https://www.windowslatest.com/2026/10/06/microsoft-reveals-whats-actually-new-in-windows-11-26h2-and-how-to-verify-it/) | Medium |
| R-W-015 | Task Manager tracks NPU usage | CONFIG | NPU hardware | [Windows Latest](https://www.windowslatest.com/2026/10/06/microsoft-reveals-whats-actually-new-in-windows-11-26h2-and-how-to-verify-it/) | Medium |
| R-W-016 | Point-in-time restore turns on automatically only if the OS volume is ≥200 GB; on for managed systems from 26H2 | CONFIG / recovery | Volume size condition | [Windows Latest](https://www.windowslatest.com/2026/10/06/microsoft-reveals-whats-actually-new-in-windows-11-26h2-and-how-to-verify-it/) | Medium (official doc pending) |
| R-W-017 | `ms-settings:` URIs open specific Settings pages via `LaunchUriAsync`. The Microsoft Learn table has 233 URI rows (229 distinct) in 20 categories; availability varies by version and SKU | DEV-GUIDE | Some URIs are legacy (Cortana, Mixed reality, Surface Hub) | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings-app) | High |

### 3.2 macOS 27.0.1 Golden Gate

| ID | Finding | Layer | Source | Conf. |
|---|---|---|---|---|
| R-M-001 | Open/Save panel Recents list is reachable with Command-Shift-F | SYS-UI (AppKit panel) | [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes) | High |
| R-M-002 | Window > Move & Resize and Window > Full Screen Tile apply to open/save panels that aren't shown as sheets | SYS-UI | Same | High |
| R-M-003 | `NSMenu` hides menu item symbol images by default (reverting the macOS 26 style); `NSMenuItem.preferredImageVisibility` overrides this; Settings, Share and Print keep default images | DEV-GUIDE / SYS-UI | Same | High |
| R-M-004 | `NSSegmentedControlRole.tabs`: VoiceOver reads it as "tabs", with a distinct appearance from value-selection segmented controls | DEV-GUIDE / a11y | Same | High |
| R-M-005 | `NSRefreshController` adds pull-to-refresh to `NSScrollView` | DEV-GUIDE | Same | High |
| R-M-006 | `NSTextSelectionManager` provides click/drag/shift-click/double- and triple-click selection via gesture recognizers | DEV-GUIDE | Same | High |
| R-M-007 | Titlebar accessories may draw outside their bounds (shadows, interactive glass) for apps linked on 27 | DEV-GUIDE | Same | High |
| R-M-008 | Traffic-light controls were redesigned; Dock context menus show background processes; menu bar items show no icons by default | SYS-UI | [Wikipedia](https://en.wikipedia.org/wiki/MacOS_Golden_Gate) (tertiary; to be confirmed with Apple sources) | Medium |
| R-M-009 | System Settings uses sidebar navigation; available settings "may vary depending on your Mac and the apps you installed"; app settings live in each app's menu | CONFIG | [Mac User Guide (27)](https://support.apple.com/en-gb/guide/mac-help/mh15217/27/mac/27) | High |

### 3.3 GNOME 51, Plasma 6.8, COSMIC 1.11

| ID | Finding | Layer | Source | Conf. |
|---|---|---|---|---|
| R-G-001 | Settings > Displays: Auto Rotate plus an orientation lock when rotation is enabled; center-aligned snapping in the arrangement view | CONFIG | [GNOME 51](https://release.gnome.org/51/) | High |
| R-G-002 | Settings > Mouse: option to disable the touchpad automatically when a mouse is plugged in | CONFIG | Same | High |
| R-G-003 | Settings > Network: WEP support removed; DNS search-domain settings added | CONFIG | Same | High |
| R-G-004 | Files (Nautilus): a badge shows the dragged-item count; copied files are auto-selected; right-clicking empty space keeps the selection; notifications are grouped | FP-APP | Same | High |
| R-G-005 | GTK 4.23: GtkSpinner, GtkStack and GtkRevealer respect the reduced-motion preference; `xdg-session-management-v1` session saving; fractional-scale pixel snapping | DEV-GUIDE | [GNOME 51 developers](https://release.gnome.org/51/developers/index.html) | High |
| R-K-001 | Plasma X11 session retired; X11-only apps continue via XWayland | SYS | [Plasma 6.8](https://kde.org/announcements/plasma/6/6.8.0/) | High |
| R-K-002 | Meta+Ctrl+PgUp/PgDn cycles tasks; adding Shift moves the active task | SYS-UI | Same | High |
| R-K-003 | Autostart entries can be enabled/disabled without being deleted; Display Configuration moves screens pixel-by-pixel with the arrow keys; new `kscreenctl` CLI | CONFIG | Same | High |
| R-K-004 | KWin window rules can add shadows, outlines and rounded corners to client-side-decorated windows | SYS-UI | Same | High |
| R-O-001 | COSMIC: `xdg-dialog-v1` keeps modal dialogs from being hidden; Escape no longer captured by text inputs (menus close on the first press) | SYS-UI / toolkit | [COSMIC Epoch releases](https://github.com/pop-os/cosmic-epoch/releases/) | High |
| R-O-002 | COSMIC Settings: touchpad tap-and-drag and drag-lock toggles | CONFIG | Same | High |

---

## Chapter 4 — Source Register Status (after Part 1)

| Metric | Count |
|---|---|
| Unique normalized URLs | 94 |
| Unique registrable domains (eTLD+1) | 29 |
| Pages actually inspected (full fetch + extraction) | 20 |
| Domains with at least one inspected page | 11 |
| Search-snippet-only URLs | 74 |
| Inaccessible | 0 |
| Primary sources among inspected | 18 |
| Independent sources among inspected | 2 |
| Progress toward the 400-domain target | 29 / 400 (7.3%) — target not met |

Limitation: inspected pages were read through prompted extraction. Before final citation in the subsystem chapters, every quoted label will be re-read from the full page text.

---

## Chapter 5 — Missing, Uncertain, and Contradictory Evidence (open log)

| Gap ID | Type | Description | Planned resolution |
|---|---|---|---|
| GAP-001 | Contradiction (staleness) | The fetched copy of apple.com/macos shows Tahoe content, while Apple Developer and the Mac User Guide show macOS 27 | Refetch uncached in B5; rely on support.apple.com/27 pages |
| GAP-002 | Missing | Exact Cinnamon version in Mint 22.3 | Fetch linuxmint/cinnamon GitHub tags and the Mint 22.3 release notes (B8) |
| GAP-003 | Version drift | ChromeOS 156 rollout (2026-10-20) is reported by a third party only; the official stable post seen is 154 (16805.33.0) | Check Chrome Releases for any later ChromeOS stable post (B8) |
| GAP-004 | Uncorroborated | Start Small/Large, Search previews, Settings Agent and point-in-time restore ≥200 GB come from an independent source only | Find the Microsoft Support / Insider blog originals (B1/B2) |
| GAP-005 | Rollout ambiguity | 26H2 features may be enabled by Controlled Feature Rollout rather than by build | Record a per-feature enablement-state flag; note this in the validation plan |
| GAP-006 | Uncertain | Whether the macOS 27 sidebar contains a separate "Menu Bar" pane (Tahoe introduced one) | Enumerate the sidebar from Mac User Guide 27 (B4) |
| GAP-007 | Uncorroborated | macOS 27 traffic-light and menu-icon changes rely on Wikipedia for the UI side; AppKit notes confirm only the menu-image behavior | HIG / Apple newsroom (B5/B6) |
| GAP-008 | Limitation | Live OS screenshots and measurements cannot be produced in this environment; pixel-level values will rely on documented tokens, source code (e.g. WinUI theme resources, Libadwaita CSS, Breeze) or cited screenshots | Flag ESTIMATE; validation plan covers measurement on real hardware |

---

## Chapter 6 — Validation Plan (skeleton; full version in Chapter 45)

| VP ID | What is validated | Method | Environment | Pass criterion |
|---|---|---|---|---|
| VP-01 | Settings inventories (WB, MD, DE settings) | Walk every sidebar/page; screenshot; diff labels against the catalogue | Clean VM or hardware at the exact build (26300.9711, 26A434, GNOME 51.1, Plasma 6.8.1…) | 100% label match, or a documented delta |
| VP-02 | ms-settings / x-apple.systempreferences / settings URIs | Script-launch each URI; record the page opened or a failure | Same | Each URI resolves or is marked absent |
| VP-03 | Keyboard shortcuts | Scripted key injection plus observation | Same | Behavior matches the catalogue |
| VP-04 | Motion timing | 120/240 fps screen capture; frame-count the transitions | Hardware with reduced motion on and off | ESTIMATE ±1 frame, labeled |
| VP-05 | Accessibility semantics | Accessibility Insights (Windows), Accessibility Inspector (macOS), Accerciser (AT-SPI) | Same | Role/name/state as specified |
| VP-06 | Workflows (H) | Scripted success / failure / cancel / retry / rollback runs | Same | Observed states match the state diagrams |
| VP-07 | Rollout/SKU flags | Compare Home vs Pro vs Enterprise; managed vs unmanaged; region | Multiple VMs | Flags confirmed |

---

## Appendix A — Master Coverage Checklist (full hierarchy)

Stable IDs: `WA` Windows shell, `WB` Windows Settings, `WC` Fluent design, `MC` macOS desktop, `MD` macOS System Settings, `MG` macOS design, `LG` GNOME, `LK` KDE Plasma, `LX` Xfce, `LC` Cinnamon, `LO` COSMIC, `LP` Pantheon, `LB` Budgie, `LR` ChromeOS, `F` components, `G` tokens/accessibility, `H` workflows, `I` integration, `J` register/audit.

Leaf titles under WB and MD are research targets taken from the prompt and from the expected structure of each release. They are not yet verified page names. The batch for each item will confirm, rename, or mark NOT FOUND. The same list is available as `litricos_coverage_checklist.csv`.


### CH00 — Version baseline verification

| ID | Item | Status | Note |
|---|---|---|---|
| CH00.01 | Windows 11 GA version/build | COMPLETE | 26H2, OS build 26300.9711 (KB5127648, 2026-10-13) |
| CH00.02 | Windows 11 26H1 scope (new-device only) | COMPLETE | Not offered in-place from 24H2/25H2 |
| CH00.03 | macOS latest GA version/build | COMPLETE | macOS 27.0.1 Golden Gate (26A434, 2026-09-28); 27.2 RC 26B118 excluded |
| CH00.04 | macOS 27 marketing page (apple.com/macos) | PARTIAL | Fetched page still showed Tahoe; treated as stale cache |
| CH00.05 | GNOME stable | COMPLETE | GNOME 51 'A Coruña' (2026-09-16); 51.1 point release |
| CH00.06 | KDE Plasma stable | COMPLETE | Plasma 6.8.1 (2026-10-20); Frameworks 6.31.0; Gear 26.08.2 |
| CH00.07 | Xfce stable | COMPLETE | Xfce 4.20 (2024-12-15) still current per xfce.org |
| CH00.08 | Cinnamon / Linux Mint | PARTIAL | Mint 22.3 current; Mint 27 due Christmas 2026; exact Cinnamon version not yet verified |
| CH00.09 | COSMIC Epoch | COMPLETE | Epoch 1.11.0 (20 Oct 2026) |
| CH00.10 | elementary OS / Pantheon | COMPLETE | OS 8.1.1 (2026-02-26); component versions from releases.elementary.io |
| CH00.11 | Budgie | COMPLETE | Budgie 10.10.3 (2026-09-27) |
| CH00.12 | ChromeOS stable | COMPLETE | 16805.33.0 / browser 154.0.8037.151 (2026-10-09); later M156 rollout reported by third party only |

### WA — Research A — Windows 11 shell & system UI

| ID | Item | Status | Note |
|---|---|---|---|
| WA.01 | Desktop | PENDING |  |
| WA.01.01 | &nbsp;&nbsp;&nbsp;&nbsp;Wallpaper/slideshow/spotlight | PENDING |  |
| WA.01.02 | &nbsp;&nbsp;&nbsp;&nbsp;Desktop icons, arrange, auto-arrange, grid | PENDING |  |
| WA.01.03 | &nbsp;&nbsp;&nbsp;&nbsp;Desktop context menu (Win11 compact vs 'Show more options') | PENDING |  |
| WA.02 | Start | PENDING |  |
| WA.02.01 | &nbsp;&nbsp;&nbsp;&nbsp;Pinned grid, folders | PENDING |  |
| WA.02.02 | &nbsp;&nbsp;&nbsp;&nbsp;Recommended / Recent section | PENDING |  |
| WA.02.03 | &nbsp;&nbsp;&nbsp;&nbsp;All apps view(s) | PENDING |  |
| WA.02.04 | &nbsp;&nbsp;&nbsp;&nbsp;Start size Small/Large & section hiding (26H2) | PENDING |  |
| WA.02.05 | &nbsp;&nbsp;&nbsp;&nbsp;Power menu & account menu | PENDING |  |
| WA.02.06 | &nbsp;&nbsp;&nbsp;&nbsp;Start keyboard navigation | PENDING |  |
| WA.03 | Taskbar | PENDING |  |
| WA.03.01 | &nbsp;&nbsp;&nbsp;&nbsp;Alignment/position top/left/right (26H2) | PENDING |  |
| WA.03.02 | &nbsp;&nbsp;&nbsp;&nbsp;Smaller taskbar option | PENDING |  |
| WA.03.03 | &nbsp;&nbsp;&nbsp;&nbsp;Grouping, labels, combine behaviour | PENDING |  |
| WA.03.04 | &nbsp;&nbsp;&nbsp;&nbsp;Jump lists & app-specific actions | PENDING |  |
| WA.03.05 | &nbsp;&nbsp;&nbsp;&nbsp;Thumbnail previews & hover | PENDING |  |
| WA.03.06 | &nbsp;&nbsp;&nbsp;&nbsp;Overflow menu | PENDING |  |
| WA.03.07 | &nbsp;&nbsp;&nbsp;&nbsp;Taskbar corner icons / system tray | PENDING |  |
| WA.03.08 | &nbsp;&nbsp;&nbsp;&nbsp;Badges, flash, progress | PENDING |  |
| WA.03.09 | &nbsp;&nbsp;&nbsp;&nbsp;AI agent monitoring | PENDING |  |
| WA.04 | Search | PENDING |  |
| WA.04.01 | &nbsp;&nbsp;&nbsp;&nbsp;Search box modes | PENDING |  |
| WA.04.02 | &nbsp;&nbsp;&nbsp;&nbsp;Result preview & type labels | PENDING |  |
| WA.04.03 | &nbsp;&nbsp;&nbsp;&nbsp;Web/Store suggestion toggles | PENDING |  |
| WA.04.04 | &nbsp;&nbsp;&nbsp;&nbsp;Indexing dependency | PENDING |  |
| WA.05 | Widgets board & notification controls | PENDING |  |
| WA.06 | Quick Settings | PENDING |  |
| WA.06.01 | &nbsp;&nbsp;&nbsp;&nbsp;Tiles, edit, sliders | PENDING |  |
| WA.06.02 | &nbsp;&nbsp;&nbsp;&nbsp;Wi-Fi/Bluetooth sub-panels | PENDING |  |
| WA.06.03 | &nbsp;&nbsp;&nbsp;&nbsp;Media controls | PENDING |  |
| WA.07 | Notifications | PENDING |  |
| WA.07.01 | &nbsp;&nbsp;&nbsp;&nbsp;Notification Center | PENDING |  |
| WA.07.02 | &nbsp;&nbsp;&nbsp;&nbsp;Toasts lifecycle | PENDING |  |
| WA.07.03 | &nbsp;&nbsp;&nbsp;&nbsp;Do not disturb / Focus | PENDING |  |
| WA.07.04 | &nbsp;&nbsp;&nbsp;&nbsp;Badges & priority | PENDING |  |
| WA.08 | Task View & virtual desktops | PENDING |  |
| WA.08.01 | &nbsp;&nbsp;&nbsp;&nbsp;Create/rename/reorder/close | PENDING |  |
| WA.08.02 | &nbsp;&nbsp;&nbsp;&nbsp;Per-desktop wallpaper | PENDING |  |
| WA.08.03 | &nbsp;&nbsp;&nbsp;&nbsp;Alt+Tab scope | PENDING |  |
| WA.08.04 | &nbsp;&nbsp;&nbsp;&nbsp;Timeline legacy (deprecated) | PENDING |  |
| WA.09 | Window management | PENDING |  |
| WA.09.01 | &nbsp;&nbsp;&nbsp;&nbsp;Snap Layouts flyout | PENDING |  |
| WA.09.02 | &nbsp;&nbsp;&nbsp;&nbsp;Snap Groups | PENDING |  |
| WA.09.03 | &nbsp;&nbsp;&nbsp;&nbsp;Snap assist | PENDING |  |
| WA.09.04 | &nbsp;&nbsp;&nbsp;&nbsp;Drag-to-edge, shake, maximize/restore | PENDING |  |
| WA.09.05 | &nbsp;&nbsp;&nbsp;&nbsp;Title bar controls & caption buttons | PENDING |  |
| WA.09.06 | &nbsp;&nbsp;&nbsp;&nbsp;Win+arrow shortcuts | PENDING |  |
| WA.09.07 | &nbsp;&nbsp;&nbsp;&nbsp;Multi-monitor window restore | PENDING |  |
| WA.10 | Lock, sign-in, session | PENDING |  |
| WA.10.01 | &nbsp;&nbsp;&nbsp;&nbsp;Lock screen status & widgets | PENDING |  |
| WA.10.02 | &nbsp;&nbsp;&nbsp;&nbsp;Sign-in (PIN, Hello, password, passkey) | PENDING |  |
| WA.10.03 | &nbsp;&nbsp;&nbsp;&nbsp;User switching & Ctrl+Alt+Del screen | PENDING |  |
| WA.10.04 | &nbsp;&nbsp;&nbsp;&nbsp;Shutdown/restart/sleep/hibernate | PENDING |  |
| WA.10.05 | &nbsp;&nbsp;&nbsp;&nbsp;Update-and-restart flows | PENDING |  |
| WA.11 | OOBE / first-run | PENDING |  |
| WA.11.01 | &nbsp;&nbsp;&nbsp;&nbsp;Region/keyboard | PENDING |  |
| WA.11.02 | &nbsp;&nbsp;&nbsp;&nbsp;Network requirement | PENDING |  |
| WA.11.03 | &nbsp;&nbsp;&nbsp;&nbsp;Microsoft account requirement & local account paths | PENDING |  |
| WA.11.04 | &nbsp;&nbsp;&nbsp;&nbsp;Privacy pages | PENDING |  |
| WA.11.05 | &nbsp;&nbsp;&nbsp;&nbsp;Restore from backup | PENDING |  |
| WA.12 | UAC & permission prompts | PENDING |  |
| WA.12.01 | &nbsp;&nbsp;&nbsp;&nbsp;Consent vs credential prompts | PENDING |  |
| WA.12.02 | &nbsp;&nbsp;&nbsp;&nbsp;Secure desktop | PENDING |  |
| WA.12.03 | &nbsp;&nbsp;&nbsp;&nbsp;Admin protection (if available) | PENDING |  |
| WA.12.04 | &nbsp;&nbsp;&nbsp;&nbsp;App capability prompts (camera/mic/location) | PENDING |  |
| WA.13 | File Explorer | PENDING |  |
| WA.13.01 | &nbsp;&nbsp;&nbsp;&nbsp;Home, Gallery, Quick Access | PENDING |  |
| WA.13.02 | &nbsp;&nbsp;&nbsp;&nbsp;Navigation pane | PENDING |  |
| WA.13.03 | &nbsp;&nbsp;&nbsp;&nbsp;Address bar/breadcrumbs | PENDING |  |
| WA.13.04 | &nbsp;&nbsp;&nbsp;&nbsp;Tabs | PENDING |  |
| WA.13.05 | &nbsp;&nbsp;&nbsp;&nbsp;Command bar & menus | PENDING |  |
| WA.13.06 | &nbsp;&nbsp;&nbsp;&nbsp;Views, sort, group | PENDING |  |
| WA.13.07 | &nbsp;&nbsp;&nbsp;&nbsp;Selection model | PENDING |  |
| WA.13.08 | &nbsp;&nbsp;&nbsp;&nbsp;Details/preview pane | PENDING |  |
| WA.13.09 | &nbsp;&nbsp;&nbsp;&nbsp;Search in Explorer | PENDING |  |
| WA.13.10 | &nbsp;&nbsp;&nbsp;&nbsp;Copy/move progress dialog | PENDING |  |
| WA.13.11 | &nbsp;&nbsp;&nbsp;&nbsp;Conflict resolution | PENDING |  |
| WA.13.12 | &nbsp;&nbsp;&nbsp;&nbsp;Rename (incl. voice typing 26H2) | PENDING |  |
| WA.13.13 | &nbsp;&nbsp;&nbsp;&nbsp;Delete & Recycle Bin | PENDING |  |
| WA.13.14 | &nbsp;&nbsp;&nbsp;&nbsp;Context menus (modern/legacy) | PENDING |  |
| WA.13.15 | &nbsp;&nbsp;&nbsp;&nbsp;Drag & drop semantics | PENDING |  |
| WA.13.16 | &nbsp;&nbsp;&nbsp;&nbsp;Archive formats | PENDING |  |
| WA.13.17 | &nbsp;&nbsp;&nbsp;&nbsp;AI actions | PENDING |  |
| WA.13.18 | &nbsp;&nbsp;&nbsp;&nbsp;Properties dialog | PENDING |  |
| WA.14 | Common dialogs | PENDING |  |
| WA.14.01 | &nbsp;&nbsp;&nbsp;&nbsp;Open/Save (IFileDialog) | PENDING |  |
| WA.14.02 | &nbsp;&nbsp;&nbsp;&nbsp;Print dialog (modern/legacy) | PENDING |  |
| WA.14.03 | &nbsp;&nbsp;&nbsp;&nbsp;Share sheet | PENDING |  |
| WA.14.04 | &nbsp;&nbsp;&nbsp;&nbsp;Folder picker | PENDING |  |
| WA.15 | System-wide tools | PENDING |  |
| WA.15.01 | &nbsp;&nbsp;&nbsp;&nbsp;Clipboard history (Win+V) | PENDING |  |
| WA.15.02 | &nbsp;&nbsp;&nbsp;&nbsp;Snipping Tool / PrtScn | PENDING |  |
| WA.15.03 | &nbsp;&nbsp;&nbsp;&nbsp;Emoji & symbols panel (Win+.) | PENDING |  |
| WA.15.04 | &nbsp;&nbsp;&nbsp;&nbsp;Touch keyboard/IME | PENDING |  |
| WA.15.05 | &nbsp;&nbsp;&nbsp;&nbsp;Voice typing (Win+H) | PENDING |  |
| WA.15.06 | &nbsp;&nbsp;&nbsp;&nbsp;Run dialog | PENDING |  |
| WA.16 | Legacy Windows (10/7) interface differences | PENDING |  |

### WB — Research B — Windows Settings (26H2) full inventory

| ID | Item | Status | Note |
|---|---|---|---|
| WB.01 | System | PENDING |  |
| WB.01.01 | &nbsp;&nbsp;&nbsp;&nbsp;Display | PENDING |  |
| WB.01.02 | &nbsp;&nbsp;&nbsp;&nbsp;Sound | PENDING |  |
| WB.01.03 | &nbsp;&nbsp;&nbsp;&nbsp;Notifications | PENDING |  |
| WB.01.04 | &nbsp;&nbsp;&nbsp;&nbsp;Focus | PENDING |  |
| WB.01.05 | &nbsp;&nbsp;&nbsp;&nbsp;Power & battery | PENDING |  |
| WB.01.06 | &nbsp;&nbsp;&nbsp;&nbsp;Storage | PENDING |  |
| WB.01.07 | &nbsp;&nbsp;&nbsp;&nbsp;Nearby sharing | PENDING |  |
| WB.01.08 | &nbsp;&nbsp;&nbsp;&nbsp;Multitasking | PENDING |  |
| WB.01.09 | &nbsp;&nbsp;&nbsp;&nbsp;For developers / Advanced | PENDING |  |
| WB.01.10 | &nbsp;&nbsp;&nbsp;&nbsp;Activation | PENDING |  |
| WB.01.11 | &nbsp;&nbsp;&nbsp;&nbsp;Troubleshoot | PENDING |  |
| WB.01.12 | &nbsp;&nbsp;&nbsp;&nbsp;Recovery | PENDING |  |
| WB.01.13 | &nbsp;&nbsp;&nbsp;&nbsp;Projecting to this PC | PENDING |  |
| WB.01.14 | &nbsp;&nbsp;&nbsp;&nbsp;Remote Desktop | PENDING |  |
| WB.01.15 | &nbsp;&nbsp;&nbsp;&nbsp;Clipboard | PENDING |  |
| WB.01.16 | &nbsp;&nbsp;&nbsp;&nbsp;Optional features (location) | PENDING |  |
| WB.01.17 | &nbsp;&nbsp;&nbsp;&nbsp;About | PENDING |  |
| WB.01.18 | &nbsp;&nbsp;&nbsp;&nbsp;Settings Agent (AI) | PENDING |  |
| WB.02 | Bluetooth & devices | PENDING |  |
| WB.02.01 | &nbsp;&nbsp;&nbsp;&nbsp;Devices | PENDING |  |
| WB.02.02 | &nbsp;&nbsp;&nbsp;&nbsp;Printers & scanners | PENDING |  |
| WB.02.03 | &nbsp;&nbsp;&nbsp;&nbsp;Phone Link / mobile devices | PENDING |  |
| WB.02.04 | &nbsp;&nbsp;&nbsp;&nbsp;Cameras | PENDING |  |
| WB.02.05 | &nbsp;&nbsp;&nbsp;&nbsp;Mouse | PENDING |  |
| WB.02.06 | &nbsp;&nbsp;&nbsp;&nbsp;Touchpad | PENDING |  |
| WB.02.07 | &nbsp;&nbsp;&nbsp;&nbsp;Pen & Windows Ink | PENDING |  |
| WB.02.08 | &nbsp;&nbsp;&nbsp;&nbsp;Keyboard / Text input | PENDING |  |
| WB.02.09 | &nbsp;&nbsp;&nbsp;&nbsp;AutoPlay | PENDING |  |
| WB.02.10 | &nbsp;&nbsp;&nbsp;&nbsp;USB | PENDING |  |
| WB.02.11 | &nbsp;&nbsp;&nbsp;&nbsp;Shared Audio | PENDING |  |
| WB.03 | Network & internet | PENDING |  |
| WB.03.01 | &nbsp;&nbsp;&nbsp;&nbsp;Wi-Fi | PENDING |  |
| WB.03.02 | &nbsp;&nbsp;&nbsp;&nbsp;Ethernet | PENDING |  |
| WB.03.03 | &nbsp;&nbsp;&nbsp;&nbsp;VPN | PENDING |  |
| WB.03.04 | &nbsp;&nbsp;&nbsp;&nbsp;Mobile hotspot | PENDING |  |
| WB.03.05 | &nbsp;&nbsp;&nbsp;&nbsp;Airplane mode | PENDING |  |
| WB.03.06 | &nbsp;&nbsp;&nbsp;&nbsp;Proxy | PENDING |  |
| WB.03.07 | &nbsp;&nbsp;&nbsp;&nbsp;Dial-up | PENDING |  |
| WB.03.08 | &nbsp;&nbsp;&nbsp;&nbsp;Advanced network settings | PENDING |  |
| WB.03.09 | &nbsp;&nbsp;&nbsp;&nbsp;Data usage | PENDING |  |
| WB.04 | Personalization | PENDING |  |
| WB.04.01 | &nbsp;&nbsp;&nbsp;&nbsp;Background | PENDING |  |
| WB.04.02 | &nbsp;&nbsp;&nbsp;&nbsp;Colors | PENDING |  |
| WB.04.03 | &nbsp;&nbsp;&nbsp;&nbsp;Themes | PENDING |  |
| WB.04.04 | &nbsp;&nbsp;&nbsp;&nbsp;Dynamic Lighting | PENDING |  |
| WB.04.05 | &nbsp;&nbsp;&nbsp;&nbsp;Lock screen | PENDING |  |
| WB.04.06 | &nbsp;&nbsp;&nbsp;&nbsp;Text input | PENDING |  |
| WB.04.07 | &nbsp;&nbsp;&nbsp;&nbsp;Start | PENDING |  |
| WB.04.08 | &nbsp;&nbsp;&nbsp;&nbsp;Taskbar | PENDING |  |
| WB.04.09 | &nbsp;&nbsp;&nbsp;&nbsp;Fonts | PENDING |  |
| WB.04.10 | &nbsp;&nbsp;&nbsp;&nbsp;Device usage | PENDING |  |
| WB.05 | Apps | PENDING |  |
| WB.05.01 | &nbsp;&nbsp;&nbsp;&nbsp;Installed apps | PENDING |  |
| WB.05.02 | &nbsp;&nbsp;&nbsp;&nbsp;Advanced app settings | PENDING |  |
| WB.05.03 | &nbsp;&nbsp;&nbsp;&nbsp;Default apps | PENDING |  |
| WB.05.04 | &nbsp;&nbsp;&nbsp;&nbsp;Offline maps | PENDING |  |
| WB.05.05 | &nbsp;&nbsp;&nbsp;&nbsp;Optional features | PENDING |  |
| WB.05.06 | &nbsp;&nbsp;&nbsp;&nbsp;Apps for websites | PENDING |  |
| WB.05.07 | &nbsp;&nbsp;&nbsp;&nbsp;Video playback | PENDING |  |
| WB.05.08 | &nbsp;&nbsp;&nbsp;&nbsp;Startup | PENDING |  |
| WB.05.09 | &nbsp;&nbsp;&nbsp;&nbsp;Actions | PENDING |  |
| WB.06 | Accounts | PENDING |  |
| WB.06.01 | &nbsp;&nbsp;&nbsp;&nbsp;Your info | PENDING |  |
| WB.06.02 | &nbsp;&nbsp;&nbsp;&nbsp;Email & accounts | PENDING |  |
| WB.06.03 | &nbsp;&nbsp;&nbsp;&nbsp;Sign-in options | PENDING |  |
| WB.06.04 | &nbsp;&nbsp;&nbsp;&nbsp;Passkeys | PENDING |  |
| WB.06.05 | &nbsp;&nbsp;&nbsp;&nbsp;Family | PENDING |  |
| WB.06.06 | &nbsp;&nbsp;&nbsp;&nbsp;Windows backup | PENDING |  |
| WB.06.07 | &nbsp;&nbsp;&nbsp;&nbsp;Other users | PENDING |  |
| WB.06.08 | &nbsp;&nbsp;&nbsp;&nbsp;Access work or school | PENDING |  |
| WB.07 | Time & language | PENDING |  |
| WB.07.01 | &nbsp;&nbsp;&nbsp;&nbsp;Date & time | PENDING |  |
| WB.07.02 | &nbsp;&nbsp;&nbsp;&nbsp;Language & region | PENDING |  |
| WB.07.03 | &nbsp;&nbsp;&nbsp;&nbsp;Typing | PENDING |  |
| WB.07.04 | &nbsp;&nbsp;&nbsp;&nbsp;Speech | PENDING |  |
| WB.08 | Gaming | PENDING |  |
| WB.08.01 | &nbsp;&nbsp;&nbsp;&nbsp;Game Bar | PENDING |  |
| WB.08.02 | &nbsp;&nbsp;&nbsp;&nbsp;Captures | PENDING |  |
| WB.08.03 | &nbsp;&nbsp;&nbsp;&nbsp;Game Mode | PENDING |  |
| WB.09 | Accessibility | PENDING |  |
| WB.09.01 | &nbsp;&nbsp;&nbsp;&nbsp;Text size | PENDING |  |
| WB.09.02 | &nbsp;&nbsp;&nbsp;&nbsp;Visual effects | PENDING |  |
| WB.09.03 | &nbsp;&nbsp;&nbsp;&nbsp;Mouse pointer & touch | PENDING |  |
| WB.09.04 | &nbsp;&nbsp;&nbsp;&nbsp;Text cursor | PENDING |  |
| WB.09.05 | &nbsp;&nbsp;&nbsp;&nbsp;Magnifier | PENDING |  |
| WB.09.06 | &nbsp;&nbsp;&nbsp;&nbsp;Color filters | PENDING |  |
| WB.09.07 | &nbsp;&nbsp;&nbsp;&nbsp;Contrast themes | PENDING |  |
| WB.09.08 | &nbsp;&nbsp;&nbsp;&nbsp;Narrator | PENDING |  |
| WB.09.09 | &nbsp;&nbsp;&nbsp;&nbsp;Audio | PENDING |  |
| WB.09.10 | &nbsp;&nbsp;&nbsp;&nbsp;Captions | PENDING |  |
| WB.09.11 | &nbsp;&nbsp;&nbsp;&nbsp;Speech / Voice access | PENDING |  |
| WB.09.12 | &nbsp;&nbsp;&nbsp;&nbsp;Keyboard | PENDING |  |
| WB.09.13 | &nbsp;&nbsp;&nbsp;&nbsp;Mouse | PENDING |  |
| WB.09.14 | &nbsp;&nbsp;&nbsp;&nbsp;Eye control | PENDING |  |
| WB.10 | Privacy & security | PENDING |  |
| WB.10.01 | &nbsp;&nbsp;&nbsp;&nbsp;Windows Security | PENDING |  |
| WB.10.02 | &nbsp;&nbsp;&nbsp;&nbsp;Find my device | PENDING |  |
| WB.10.03 | &nbsp;&nbsp;&nbsp;&nbsp;Device encryption / BitLocker | PENDING |  |
| WB.10.04 | &nbsp;&nbsp;&nbsp;&nbsp;For developers | PENDING |  |
| WB.10.05 | &nbsp;&nbsp;&nbsp;&nbsp;Windows permissions (general, speech, inking, diagnostics, activity, search) | PENDING |  |
| WB.10.06 | &nbsp;&nbsp;&nbsp;&nbsp;App permissions (location, camera, mic, etc.) | PENDING |  |
| WB.11 | Windows Update | PENDING |  |
| WB.11.01 | &nbsp;&nbsp;&nbsp;&nbsp;Updates & pause | PENDING |  |
| WB.11.02 | &nbsp;&nbsp;&nbsp;&nbsp;Update history | PENDING |  |
| WB.11.03 | &nbsp;&nbsp;&nbsp;&nbsp;Advanced options | PENDING |  |
| WB.11.04 | &nbsp;&nbsp;&nbsp;&nbsp;Windows Insider Program | PENDING |  |
| WB.11.05 | &nbsp;&nbsp;&nbsp;&nbsp;Restart workflows & active hours | PENDING |  |
| WB.12 | Beyond Settings | PENDING |  |
| WB.12.01 | &nbsp;&nbsp;&nbsp;&nbsp;ms-settings URI register (229 distinct URIs on MS Learn) | PENDING |  |
| WB.12.02 | &nbsp;&nbsp;&nbsp;&nbsp;Control Panel applets | PENDING |  |
| WB.12.03 | &nbsp;&nbsp;&nbsp;&nbsp;Windows Security app | PENDING |  |
| WB.12.04 | &nbsp;&nbsp;&nbsp;&nbsp;Device Manager | PENDING |  |
| WB.12.05 | &nbsp;&nbsp;&nbsp;&nbsp;Task Manager (incl. NPU) | PENDING |  |
| WB.12.06 | &nbsp;&nbsp;&nbsp;&nbsp;Disk Management | PENDING |  |
| WB.12.07 | &nbsp;&nbsp;&nbsp;&nbsp;Legacy dialogs (sysdm.cpl etc.) | PENDING |  |
| WB.12.08 | &nbsp;&nbsp;&nbsp;&nbsp;Group Policy / MDM-only settings | PENDING |  |
| WB.12.09 | &nbsp;&nbsp;&nbsp;&nbsp;SKU differences (Home/Pro/Enterprise/Education) | PENDING |  |

### WC — Research A3 — Fluent / WinUI design system

| ID | Item | Status | Note |
|---|---|---|---|
| WC.01 | Design principles | PENDING |  |
| WC.02 | Color & theming tokens | PENDING |  |
| WC.03 | Typography ramp (Segoe UI Variable) | PENDING |  |
| WC.04 | Spacing & layout grid | PENDING |  |
| WC.05 | Geometry & corner radius | PENDING |  |
| WC.06 | Elevation & shadows | PENDING |  |
| WC.07 | Materials: Mica, Mica Alt, Acrylic, Smoke | PENDING |  |
| WC.08 | Iconography (Segoe Fluent Icons) | PENDING |  |
| WC.09 | Motion: timing & easing | PENDING |  |
| WC.10 | Control states (rest/hover/pressed/disabled/focus) | PENDING |  |
| WC.11 | Focus visuals | PENDING |  |
| WC.12 | Win32 / common controls conventions | PENDING |  |
| WC.13 | Loading, error, empty states | PENDING |  |

### MC — Research C — macOS 27 desktop model

| ID | Item | Status | Note |
|---|---|---|---|
| MC.01 | Menu bar | PENDING |  |
| MC.01.01 | &nbsp;&nbsp;&nbsp;&nbsp;Apple menu | PENDING |  |
| MC.01.02 | &nbsp;&nbsp;&nbsp;&nbsp;App menu & standard menus | PENDING |  |
| MC.01.03 | &nbsp;&nbsp;&nbsp;&nbsp;Menu item image visibility (27 change) | PENDING |  |
| MC.01.04 | &nbsp;&nbsp;&nbsp;&nbsp;Status menus / menu extras | PENDING |  |
| MC.01.05 | &nbsp;&nbsp;&nbsp;&nbsp;Live Activities | PENDING |  |
| MC.01.06 | &nbsp;&nbsp;&nbsp;&nbsp;Menu bar hiding & customization | PENDING |  |
| MC.02 | Dock | PENDING |  |
| MC.02.01 | &nbsp;&nbsp;&nbsp;&nbsp;Placement/size/magnification | PENDING |  |
| MC.02.02 | &nbsp;&nbsp;&nbsp;&nbsp;App indicators | PENDING |  |
| MC.02.03 | &nbsp;&nbsp;&nbsp;&nbsp;Context menu incl. background processes (27) | PENDING |  |
| MC.02.04 | &nbsp;&nbsp;&nbsp;&nbsp;Stacks | PENDING |  |
| MC.02.05 | &nbsp;&nbsp;&nbsp;&nbsp;Minimize effects | PENDING |  |
| MC.03 | Finder | PENDING |  |
| MC.03.01 | &nbsp;&nbsp;&nbsp;&nbsp;Windows/tabs/views | PENDING |  |
| MC.03.02 | &nbsp;&nbsp;&nbsp;&nbsp;Sidebar | PENDING |  |
| MC.03.03 | &nbsp;&nbsp;&nbsp;&nbsp;Tags | PENDING |  |
| MC.03.04 | &nbsp;&nbsp;&nbsp;&nbsp;Search & smart folders | PENDING |  |
| MC.03.05 | &nbsp;&nbsp;&nbsp;&nbsp;Quick Look | PENDING |  |
| MC.03.06 | &nbsp;&nbsp;&nbsp;&nbsp;Get Info | PENDING |  |
| MC.03.07 | &nbsp;&nbsp;&nbsp;&nbsp;Copy/move/conflict | PENDING |  |
| MC.03.08 | &nbsp;&nbsp;&nbsp;&nbsp;Trash & recovery | PENDING |  |
| MC.03.09 | &nbsp;&nbsp;&nbsp;&nbsp;Desktop & stacks | PENDING |  |
| MC.04 | Window model | PENDING |  |
| MC.04.01 | &nbsp;&nbsp;&nbsp;&nbsp;App vs window relationship | PENDING |  |
| MC.04.02 | &nbsp;&nbsp;&nbsp;&nbsp;Active/inactive appearance | PENDING |  |
| MC.04.03 | &nbsp;&nbsp;&nbsp;&nbsp;Traffic-light controls (27 redesign) | PENDING |  |
| MC.04.04 | &nbsp;&nbsp;&nbsp;&nbsp;Full screen | PENDING |  |
| MC.04.05 | &nbsp;&nbsp;&nbsp;&nbsp;Window tiling & Move & Resize | PENDING |  |
| MC.04.06 | &nbsp;&nbsp;&nbsp;&nbsp;Sheets & panels | PENDING |  |
| MC.04.07 | &nbsp;&nbsp;&nbsp;&nbsp;Window restoration/persistence | PENDING |  |
| MC.05 | Spaces / Mission Control / Stage Manager | PENDING |  |
| MC.05.01 | &nbsp;&nbsp;&nbsp;&nbsp;Spaces | PENDING |  |
| MC.05.02 | &nbsp;&nbsp;&nbsp;&nbsp;Mission Control | PENDING |  |
| MC.05.03 | &nbsp;&nbsp;&nbsp;&nbsp;App Exposé | PENDING |  |
| MC.05.04 | &nbsp;&nbsp;&nbsp;&nbsp;Stage Manager | PENDING |  |
| MC.06 | Spotlight | PENDING |  |
| MC.06.01 | &nbsp;&nbsp;&nbsp;&nbsp;Search | PENDING |  |
| MC.06.02 | &nbsp;&nbsp;&nbsp;&nbsp;Actions & quick keys | PENDING |  |
| MC.06.03 | &nbsp;&nbsp;&nbsp;&nbsp;Intelligence/Siri integration | PENDING |  |
| MC.07 | Control Center & Notification Center | PENDING |  |
| MC.07.01 | &nbsp;&nbsp;&nbsp;&nbsp;Modules & customization | PENDING |  |
| MC.07.02 | &nbsp;&nbsp;&nbsp;&nbsp;Third-party controls | PENDING |  |
| MC.07.03 | &nbsp;&nbsp;&nbsp;&nbsp;Notification grouping | PENDING |  |
| MC.07.04 | &nbsp;&nbsp;&nbsp;&nbsp;Widgets on desktop | PENDING |  |
| MC.08 | Dialogs & panels | PENDING |  |
| MC.08.01 | &nbsp;&nbsp;&nbsp;&nbsp;Open/Save panels (Recents Cmd-Shift-F, 27) | PENDING |  |
| MC.08.02 | &nbsp;&nbsp;&nbsp;&nbsp;Print | PENDING |  |
| MC.08.03 | &nbsp;&nbsp;&nbsp;&nbsp;Share | PENDING |  |
| MC.08.04 | &nbsp;&nbsp;&nbsp;&nbsp;Services menu | PENDING |  |
| MC.08.05 | &nbsp;&nbsp;&nbsp;&nbsp;Alerts | PENDING |  |
| MC.09 | Input | PENDING |  |
| MC.09.01 | &nbsp;&nbsp;&nbsp;&nbsp;Keyboard shortcuts | PENDING |  |
| MC.09.02 | &nbsp;&nbsp;&nbsp;&nbsp;Trackpad gestures | PENDING |  |
| MC.09.03 | &nbsp;&nbsp;&nbsp;&nbsp;Full Keyboard Access | PENDING |  |
| MC.09.04 | &nbsp;&nbsp;&nbsp;&nbsp;Screenshots (Cmd-Shift-3/4/5) | PENDING |  |
| MC.10 | Session | PENDING |  |
| MC.10.01 | &nbsp;&nbsp;&nbsp;&nbsp;Login window | PENDING |  |
| MC.10.02 | &nbsp;&nbsp;&nbsp;&nbsp;Fast user switching | PENDING |  |
| MC.10.03 | &nbsp;&nbsp;&nbsp;&nbsp;Lock | PENDING |  |
| MC.10.04 | &nbsp;&nbsp;&nbsp;&nbsp;Force Quit | PENDING |  |
| MC.10.05 | &nbsp;&nbsp;&nbsp;&nbsp;Startup/shutdown/restart | PENDING |  |
| MC.10.06 | &nbsp;&nbsp;&nbsp;&nbsp;Setup Assistant | PENDING |  |

### MD — Research D — macOS 27 System Settings inventory

| ID | Item | Status | Note |
|---|---|---|---|
| MD.01 | Sidebar enumeration (exact order, hardware-dependent items) | PENDING |  |
| MD.02 | Apple Account | PENDING |  |
| MD.03 | Family | PENDING |  |
| MD.04 | Wi-Fi | PENDING |  |
| MD.05 | Bluetooth | PENDING |  |
| MD.06 | Network | PENDING |  |
| MD.07 | VPN | PENDING |  |
| MD.08 | Battery / Energy | PENDING |  |
| MD.09 | General (About, Software Update, Storage, AppleCare, AirDrop & Handoff, Login Items & Extensions, Language & Region, Date & Time, Sharing, Time Machine, Transfer or Reset, Startup Disk, Device Management) | PENDING |  |
| MD.10 | Accessibility | PENDING |  |
| MD.11 | Appearance | PENDING |  |
| MD.12 | Apple Intelligence & Siri | PENDING |  |
| MD.13 | Control Center | PENDING |  |
| MD.14 | Desktop & Dock | PENDING |  |
| MD.15 | Displays | PENDING |  |
| MD.16 | Spotlight | PENDING |  |
| MD.17 | Wallpaper | PENDING |  |
| MD.18 | Screen Saver | PENDING |  |
| MD.19 | Notifications | PENDING |  |
| MD.20 | Sound | PENDING |  |
| MD.21 | Focus | PENDING |  |
| MD.22 | Screen Time | PENDING |  |
| MD.23 | Lock Screen | PENDING |  |
| MD.24 | Privacy & Security | PENDING |  |
| MD.25 | Touch ID & Password / Login Password | PENDING |  |
| MD.26 | Users & Groups | PENDING |  |
| MD.27 | Internet Accounts | PENDING |  |
| MD.28 | Game Center | PENDING |  |
| MD.29 | Wallet & Apple Pay | PENDING |  |
| MD.30 | Keyboard | PENDING |  |
| MD.31 | Mouse | PENDING |  |
| MD.32 | Trackpad | PENDING |  |
| MD.33 | Game Controllers | PENDING |  |
| MD.34 | Printers & Scanners | PENDING |  |
| MD.35 | Menu Bar (if separate in 27) | PENDING |  |
| MD.36 | App Settings conventions (Cmd-,) | PENDING |  |
| MD.37 | x-apple.systempreferences URL schemes | PENDING |  |
| MD.38 | Configuration profiles / MDM | PENDING |  |

### MG — macOS design system (HIG)

| ID | Item | Status | Note |
|---|---|---|---|
| MG.01 | Liquid Glass materials | PENDING |  |
| MG.02 | Color & semantic colors | PENDING |  |
| MG.03 | SF Pro typography & Dynamic Type equivalents | PENDING |  |
| MG.04 | SF Symbols | PENDING |  |
| MG.05 | Layout & spacing | PENDING |  |
| MG.06 | Corner radii/concentricity | PENDING |  |
| MG.07 | Motion | PENDING |  |
| MG.08 | Control sizes (mini/small/regular/large/extra-large) | PENDING |  |
| MG.09 | Dark mode & increased contrast | PENDING |  |
| MG.10 | Accessibility semantics (NSAccessibility) | PENDING |  |

### LG — Research E — GNOME 51

| ID | Item | Status | Note |
|---|---|---|---|
| LG.01 | Shell components & compositor | PENDING |  |
| LG.02 | Launcher | PENDING |  |
| LG.03 | Panel / dock | PENDING |  |
| LG.04 | System Settings inventory | PENDING |  |
| LG.05 | File manager | PENDING |  |
| LG.06 | Window behaviour & tiling | PENDING |  |
| LG.07 | Notifications | PENDING |  |
| LG.08 | Workspace model | PENDING |  |
| LG.09 | Customization model | PENDING |  |
| LG.10 | Design language / HIG / tokens | PENDING |  |
| LG.11 | Accessibility | PENDING |  |
| LG.12 | Limitations & known issues | PENDING |  |
| LG.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LG.14 | DE vs kernel vs distro boundary | PENDING |  |

### LK — Research E — KDE Plasma 6.8

| ID | Item | Status | Note |
|---|---|---|---|
| LK.01 | Shell components & compositor | PENDING |  |
| LK.02 | Launcher | PENDING |  |
| LK.03 | Panel / dock | PENDING |  |
| LK.04 | System Settings inventory | PENDING |  |
| LK.05 | File manager | PENDING |  |
| LK.06 | Window behaviour & tiling | PENDING |  |
| LK.07 | Notifications | PENDING |  |
| LK.08 | Workspace model | PENDING |  |
| LK.09 | Customization model | PENDING |  |
| LK.10 | Design language / HIG / tokens | PENDING |  |
| LK.11 | Accessibility | PENDING |  |
| LK.12 | Limitations & known issues | PENDING |  |
| LK.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LK.14 | DE vs kernel vs distro boundary | PENDING |  |

### LX — Research E — Xfce 4.20

| ID | Item | Status | Note |
|---|---|---|---|
| LX.01 | Shell components & compositor | PENDING |  |
| LX.02 | Launcher | PENDING |  |
| LX.03 | Panel / dock | PENDING |  |
| LX.04 | System Settings inventory | PENDING |  |
| LX.05 | File manager | PENDING |  |
| LX.06 | Window behaviour & tiling | PENDING |  |
| LX.07 | Notifications | PENDING |  |
| LX.08 | Workspace model | PENDING |  |
| LX.09 | Customization model | PENDING |  |
| LX.10 | Design language / HIG / tokens | PENDING |  |
| LX.11 | Accessibility | PENDING |  |
| LX.12 | Limitations & known issues | PENDING |  |
| LX.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LX.14 | DE vs kernel vs distro boundary | PENDING |  |

### LC — Research E — Cinnamon (Mint 22.3 / 27)

| ID | Item | Status | Note |
|---|---|---|---|
| LC.01 | Shell components & compositor | PENDING |  |
| LC.02 | Launcher | PENDING |  |
| LC.03 | Panel / dock | PENDING |  |
| LC.04 | System Settings inventory | PENDING |  |
| LC.05 | File manager | PENDING |  |
| LC.06 | Window behaviour & tiling | PENDING |  |
| LC.07 | Notifications | PENDING |  |
| LC.08 | Workspace model | PENDING |  |
| LC.09 | Customization model | PENDING |  |
| LC.10 | Design language / HIG / tokens | PENDING |  |
| LC.11 | Accessibility | PENDING |  |
| LC.12 | Limitations & known issues | PENDING |  |
| LC.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LC.14 | DE vs kernel vs distro boundary | PENDING |  |

### LO — Research E — COSMIC Epoch 1.11

| ID | Item | Status | Note |
|---|---|---|---|
| LO.01 | Shell components & compositor | PENDING |  |
| LO.02 | Launcher | PENDING |  |
| LO.03 | Panel / dock | PENDING |  |
| LO.04 | System Settings inventory | PENDING |  |
| LO.05 | File manager | PENDING |  |
| LO.06 | Window behaviour & tiling | PENDING |  |
| LO.07 | Notifications | PENDING |  |
| LO.08 | Workspace model | PENDING |  |
| LO.09 | Customization model | PENDING |  |
| LO.10 | Design language / HIG / tokens | PENDING |  |
| LO.11 | Accessibility | PENDING |  |
| LO.12 | Limitations & known issues | PENDING |  |
| LO.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LO.14 | DE vs kernel vs distro boundary | PENDING |  |

### LP — Research E — Pantheon (elementary OS 8.1)

| ID | Item | Status | Note |
|---|---|---|---|
| LP.01 | Shell components & compositor | PENDING |  |
| LP.02 | Launcher | PENDING |  |
| LP.03 | Panel / dock | PENDING |  |
| LP.04 | System Settings inventory | PENDING |  |
| LP.05 | File manager | PENDING |  |
| LP.06 | Window behaviour & tiling | PENDING |  |
| LP.07 | Notifications | PENDING |  |
| LP.08 | Workspace model | PENDING |  |
| LP.09 | Customization model | PENDING |  |
| LP.10 | Design language / HIG / tokens | PENDING |  |
| LP.11 | Accessibility | PENDING |  |
| LP.12 | Limitations & known issues | PENDING |  |
| LP.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LP.14 | DE vs kernel vs distro boundary | PENDING |  |

### LB — Research E — Budgie 10.10

| ID | Item | Status | Note |
|---|---|---|---|
| LB.01 | Shell components & compositor | PENDING |  |
| LB.02 | Launcher | PENDING |  |
| LB.03 | Panel / dock | PENDING |  |
| LB.04 | System Settings inventory | PENDING |  |
| LB.05 | File manager | PENDING |  |
| LB.06 | Window behaviour & tiling | PENDING |  |
| LB.07 | Notifications | PENDING |  |
| LB.08 | Workspace model | PENDING |  |
| LB.09 | Customization model | PENDING |  |
| LB.10 | Design language / HIG / tokens | PENDING |  |
| LB.11 | Accessibility | PENDING |  |
| LB.12 | Limitations & known issues | PENDING |  |
| LB.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LB.14 | DE vs kernel vs distro boundary | PENDING |  |

### LR — Research E — ChromeOS 154+

| ID | Item | Status | Note |
|---|---|---|---|
| LR.01 | Shell components & compositor | PENDING |  |
| LR.02 | Launcher | PENDING |  |
| LR.03 | Panel / dock | PENDING |  |
| LR.04 | System Settings inventory | PENDING |  |
| LR.05 | File manager | PENDING |  |
| LR.06 | Window behaviour & tiling | PENDING |  |
| LR.07 | Notifications | PENDING |  |
| LR.08 | Workspace model | PENDING |  |
| LR.09 | Customization model | PENDING |  |
| LR.10 | Design language / HIG / tokens | PENDING |  |
| LR.11 | Accessibility | PENDING |  |
| LR.12 | Limitations & known issues | PENDING |  |
| LR.13 | Integration points (portals, D-Bus, APIs) | PENDING |  |
| LR.14 | DE vs kernel vs distro boundary | PENDING |  |

### F — Research F — Shared component catalog

| ID | Item | Status | Note |
|---|---|---|---|
| F.01 | Buttons | PENDING |  |
| F.02 | Text fields | PENDING |  |
| F.03 | Switches | PENDING |  |
| F.04 | Checkboxes | PENDING |  |
| F.05 | Radio controls | PENDING |  |
| F.06 | Sliders | PENDING |  |
| F.07 | Pickers (date/time/color/file) | PENDING |  |
| F.08 | Tabs | PENDING |  |
| F.09 | Navigation views | PENDING |  |
| F.10 | Sidebars | PENDING |  |
| F.11 | Command bars/toolbars | PENDING |  |
| F.12 | Menus | PENDING |  |
| F.13 | Context menus | PENDING |  |
| F.14 | Tooltips | PENDING |  |
| F.15 | Dialogs | PENDING |  |
| F.16 | Sheets | PENDING |  |
| F.17 | Notifications/banners/toasts | PENDING |  |
| F.18 | Progress indicators | PENDING |  |
| F.19 | Lists | PENDING |  |
| F.20 | Trees | PENDING |  |
| F.21 | Grids/tables | PENDING |  |
| F.22 | Cards | PENDING |  |
| F.23 | Search fields | PENDING |  |
| F.24 | Empty states | PENDING |  |
| F.25 | Loading states | PENDING |  |
| F.26 | Errors/validation | PENDING |  |
| F.27 | Confirmation flows | PENDING |  |

### G — Research G — Tokens & accessibility

| ID | Item | Status | Note |
|---|---|---|---|
| G.01 | Color tokens | PENDING |  |
| G.02 | Typography | PENDING |  |
| G.03 | Spacing | PENDING |  |
| G.04 | Radii | PENDING |  |
| G.05 | Shadows/elevation | PENDING |  |
| G.06 | Materials/blur | PENDING |  |
| G.07 | Icons | PENDING |  |
| G.08 | Motion timing/easing | PENDING |  |
| G.09 | Light/dark | PENDING |  |
| G.10 | Contrast & high-contrast themes | PENDING |  |
| G.11 | Display scaling/HiDPI | PENDING |  |
| G.12 | Multi-monitor | PENDING |  |
| G.13 | Localization & RTL | PENDING |  |
| G.14 | Keyboard-only navigation | PENDING |  |
| G.15 | Screen readers & a11y APIs | PENDING |  |
| G.16 | Pointer/touch/trackpad | PENDING |  |
| G.17 | Reduced motion/transparency | PENDING |  |
| G.18 | Sound & visual feedback | PENDING |  |

### H — Research H — Workflows

| ID | Item | Status | Note |
|---|---|---|---|
| H.01 | First boot & setup | PENDING |  |
| H.02 | Account creation | PENDING |  |
| H.03 | Unlock | PENDING |  |
| H.04 | Launch/switch apps | PENDING |  |
| H.05 | Window management | PENDING |  |
| H.06 | Display change | PENDING |  |
| H.07 | Audio change | PENDING |  |
| H.08 | Network change | PENDING |  |
| H.09 | Device pairing | PENDING |  |
| H.10 | Install/uninstall software | PENDING |  |
| H.11 | Default apps | PENDING |  |
| H.12 | File management | PENDING |  |
| H.13 | Permissions grant/revoke | PENDING |  |
| H.14 | Updates | PENDING |  |
| H.15 | Recovery | PENDING |  |
| H.16 | Backups | PENDING |  |
| H.17 | Troubleshooting | PENDING |  |
| H.18 | Accessibility change | PENDING |  |
| H.19 | Shutdown | PENDING |  |
| H.20 | Failure/cancel/retry/rollback paths (per workflow) | PENDING |  |

### I — Research I — Technical integration & LitricOS feasibility

| ID | Item | Status | Note |
|---|---|---|---|
| I.01 | Public UI frameworks per OS | PENDING |  |
| I.02 | Shell extension points | PENDING |  |
| I.03 | Settings persistence (registry, defaults/plist, GSettings/dconf, KConfig) | PENDING |  |
| I.04 | Permission models (UAC/TCC/polkit/portals) | PENDING |  |
| I.05 | Notification APIs | PENDING |  |
| I.06 | IPC/events | PENDING |  |
| I.07 | Compositor protocols (Wayland) | PENDING |  |
| I.08 | Undocumented vs public boundary | PENDING |  |
| I.09 | Comparison matrix | PENDING |  |
| I.10 | ADOPT/ADAPT/AVOID/INVESTIGATE decisions | PENDING |  |
| I.11 | Validation tests per decision | PENDING |  |

### J — Research J — Source register & audit

| ID | Item | Status | Note |
|---|---|---|---|
| J.01 | Register schema | PENDING |  |
| J.02 | URL/domain dedupe | PENDING |  |
| J.03 | Domain count toward 400 target | PENDING |  |
| J.04 | Inspected vs snippet-only counts | PENDING |  |
| J.05 | Primary vs independent counts | PENDING |  |
| J.06 | Inaccessible log | PENDING |  |
| J.07 | Contradictions log | PENDING |  |
| J.08 | Per-subsystem coverage audit | PENDING |  |

### Checklist status totals (Part 1)

| Status | Items |
|---|---|
| COMPLETE | 10 |
| PARTIAL | 2 |
| PENDING | 536 |
| Total items (excluding block headers) | 548 |

---

## Resume Point

Part 2 starts with Batch B1, item WB.01.01 (Windows 11 26H2 Settings › System › Display).