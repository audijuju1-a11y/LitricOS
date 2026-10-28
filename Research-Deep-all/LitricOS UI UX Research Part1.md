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
| 1 | Master coverage checklist | 8 | PUBLISHED (597 rows; every row carries a final status, 0 PENDING) |
| 2 | Research and source plan | 8 | PUBLISHED |
| 3 | Initial release-delta findings (26H2, macOS 27, GNOME 51, Plasma 6.8, COSMIC 1.11) | 1 | PARTIAL |
| 4 | Source register status | 8 | IN PROGRESS (94 URLs / 29 domains) |
| 5 | Missing, uncertain, contradictory evidence log | 9 | IN PROGRESS |
| 6 | Validation plan skeleton | 10 | PUBLISHED (skeleton VP-01-VP-07; full plan in Chapter 45 / Part 30) |
| 10 | Windows 11 shell and system UI (WA) | 1, 5 | COMPLETE for coverage (Part 3; every WA row final, several PARTIAL where rollout flags are unverified) |
| 11 | Windows 11 Settings catalogue (WB) | 2 | COMPLETE for coverage (Parts 2, 12-15 and 29; WB.12.08 is PARTIAL because only the policy mechanism is documented) |
| 12 | Fluent / WinUI design system (WC) | 3, 4 | COMPLETE for coverage (Part 4; 13 rows, 11 PARTIAL because Mica/shadow constants live in closed code) |
| 20 | macOS 27 desktop model (MC) | 1, 5 | COMPLETE for coverage (Part 6; 63 rows, 61 PARTIAL pending VP-18-VP-22 geometry) |
| 21 | macOS 27 System Settings catalogue (MD) | 2 | COMPLETE for coverage (Part 5; 38 rows, 37 PARTIAL pending VP-13/VP-17) |
| 22 | macOS HIG / Liquid Glass design system (MG) | 3, 4 | COMPLETE for coverage (Part 7; 10 rows, all PARTIAL: Apple publishes no token values) |
| 30–37 | GNOME, KDE Plasma, Xfce, Cinnamon, COSMIC, Pantheon, Budgie, ChromeOS (LG–LR) | 1, 2 | COMPLETE for coverage (Parts 25-28: GNOME 51, Plasma 6.8, Xfce 4.20, Cinnamon 6.6, COSMIC Epoch 1, Pantheon 8.1, Budgie 10.10 and ChromeOS 154 - all 112 rows measured; quality is PARTIAL where pages were unavailable) |
| 40 | Shared component catalog (F) | 3 | PARTIAL (Parts 10–18 research `F.01`, `F.03`, `F.06`, `F.19`, `F.02`, `F.18`, `F.23`, `F.26`, `F.04`, `F.05`, `F.08`, `F.10`, `F.11`, `F.12`, `F.13`, `F.15`, `F.27`; Part 16 `F.14`, `F.17`; Part 17 `F.07`, `F.09`, `F.20`; Part 18 `F.21`, `F.22`, `F.24`, `F.25`; and Part 6 `F.16`; 0 items `PENDING`) |
| 41 | Design tokens and accessibility (G) | 4 | PARTIAL (Part 7 macOS; Parts 19–20 Linux measurement of all 18 items: contrast, keyboard, screen readers, token families, scaling, RTL and input; Windows token and input tables still unread) |
| 42 | User workflows and edge cases (H) | 5 | PARTIAL (Parts 21–22 measure all 20 workflows; `H03.01` unlocked-attempt limit repaired in Part 22; macOS evidence thin in `H.01`, `H.02`, `H.15`, `H.17`) |
| 43 | Technical integration and LitricOS feasibility (I) | 6, 7 | PARTIAL (Part 23: all 11 rows measured; 12 decisions, revert window implemented; no public toolkit) |
| 44 | Final source register and coverage audit (J) | 8, 9 | COMPLETE (Part 24 and addenda J.11-J.14: 4,205 rows, 382 domains, see J.14 for the re-derived percentages, 48 duplicate URLs, 5 contradictions) |
| 45 | Validation plan (full) | 10 | COMPLETE and executed (Part 30 plan; Part 31 wave 1: 8 repo-side checks; Part 32 repaired the register key, so VP-28 and VP-28b both PASS, 6 PARTIAL, 0 FAIL; 21 points still NOT RUN without hardware) |

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
| WA.01 | Desktop | PARTIAL | Group researched in Part 3; see child items. |
| WA.01.01 | &nbsp;&nbsp;&nbsp;&nbsp;Wallpaper/slideshow/spotlight | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.01.02 | &nbsp;&nbsp;&nbsp;&nbsp;Desktop icons, arrange, auto-arrange, grid | PARTIAL | Only T4/T5 evidence (3 sources); no official Windows 11 article describes this feature. Low–medium confidence. |
| WA.01.03 | &nbsp;&nbsp;&nbsp;&nbsp;Desktop context menu (Win11 compact vs 'Show more options') | PARTIAL | 1 T1/T2 and 3 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.02 | Start | PARTIAL | Group researched in Part 3; see child items. |
| WA.02.01 | &nbsp;&nbsp;&nbsp;&nbsp;Pinned grid, folders | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.02.02 | &nbsp;&nbsp;&nbsp;&nbsp;Recommended / Recent section | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.02.03 | &nbsp;&nbsp;&nbsp;&nbsp;All apps view(s) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.02.04 | &nbsp;&nbsp;&nbsp;&nbsp;Start size Small/Large & section hiding (26H2) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.02.05 | &nbsp;&nbsp;&nbsp;&nbsp;Power menu & account menu | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.02.06 | &nbsp;&nbsp;&nbsp;&nbsp;Start keyboard navigation | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03 | Taskbar | PARTIAL | Group researched in Part 3; see child items. |
| WA.03.01 | &nbsp;&nbsp;&nbsp;&nbsp;Alignment/position top/left/right (26H2) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.02 | &nbsp;&nbsp;&nbsp;&nbsp;Smaller taskbar option | PARTIAL | 3 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.03 | &nbsp;&nbsp;&nbsp;&nbsp;Grouping, labels, combine behaviour | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.04 | &nbsp;&nbsp;&nbsp;&nbsp;Jump lists & app-specific actions | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.05 | &nbsp;&nbsp;&nbsp;&nbsp;Thumbnail previews & hover | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.06 | &nbsp;&nbsp;&nbsp;&nbsp;Overflow menu | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.07 | &nbsp;&nbsp;&nbsp;&nbsp;Taskbar corner icons / system tray | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.08 | &nbsp;&nbsp;&nbsp;&nbsp;Badges, flash, progress | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.03.09 | &nbsp;&nbsp;&nbsp;&nbsp;AI agent monitoring | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.04 | Search | PARTIAL | Group researched in Part 3; see child items. |
| WA.04.01 | &nbsp;&nbsp;&nbsp;&nbsp;Search box modes | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.04.02 | &nbsp;&nbsp;&nbsp;&nbsp;Result preview & type labels | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.04.03 | &nbsp;&nbsp;&nbsp;&nbsp;Web/Store suggestion toggles | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.04.04 | &nbsp;&nbsp;&nbsp;&nbsp;Indexing dependency | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.05 | Widgets board & notification controls | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.06 | Quick Settings | PARTIAL | Group researched in Part 3; see child items. |
| WA.06.01 | &nbsp;&nbsp;&nbsp;&nbsp;Tiles, edit, sliders | PARTIAL | 3 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.06.02 | &nbsp;&nbsp;&nbsp;&nbsp;Wi-Fi/Bluetooth sub-panels | PARTIAL | 1 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.06.03 | &nbsp;&nbsp;&nbsp;&nbsp;Media controls | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.07 | Notifications | PARTIAL | Group researched in Part 3; see child items. |
| WA.07.01 | &nbsp;&nbsp;&nbsp;&nbsp;Notification Center | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.07.02 | &nbsp;&nbsp;&nbsp;&nbsp;Toasts lifecycle | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.07.03 | &nbsp;&nbsp;&nbsp;&nbsp;Do not disturb / Focus | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.07.04 | &nbsp;&nbsp;&nbsp;&nbsp;Badges & priority | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.08 | Task View & virtual desktops | PARTIAL | Group researched in Part 3; see child items. |
| WA.08.01 | &nbsp;&nbsp;&nbsp;&nbsp;Create/rename/reorder/close | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.08.02 | &nbsp;&nbsp;&nbsp;&nbsp;Per-desktop wallpaper | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.08.03 | &nbsp;&nbsp;&nbsp;&nbsp;Alt+Tab scope | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.08.04 | &nbsp;&nbsp;&nbsp;&nbsp;Timeline legacy (deprecated) | NOT APPLICABLE | Timeline is not part of the Windows 11 shell. Legacy behaviour is recorded here and in WA.16. |
| WA.09 | Window management | PARTIAL | Group researched in Part 3; see child items. |
| WA.09.01 | &nbsp;&nbsp;&nbsp;&nbsp;Snap Layouts flyout | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.09.02 | &nbsp;&nbsp;&nbsp;&nbsp;Snap Groups | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.09.03 | &nbsp;&nbsp;&nbsp;&nbsp;Snap assist | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.09.04 | &nbsp;&nbsp;&nbsp;&nbsp;Drag-to-edge, shake, maximize/restore | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.09.05 | &nbsp;&nbsp;&nbsp;&nbsp;Title bar controls & caption buttons | PARTIAL | 4 T1/T2 and 0 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.09.06 | &nbsp;&nbsp;&nbsp;&nbsp;Win+arrow shortcuts | COMPLETE | Every Windows key + arrow combination is listed in Microsoft's keyboard shortcuts article (Appendix 3A). Live-build c... |
| WA.09.07 | &nbsp;&nbsp;&nbsp;&nbsp;Multi-monitor window restore | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.10 | Lock, sign-in, session | PARTIAL | Group researched in Part 3; see child items. |
| WA.10.01 | &nbsp;&nbsp;&nbsp;&nbsp;Lock screen status & widgets | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.10.02 | &nbsp;&nbsp;&nbsp;&nbsp;Sign-in (PIN, Hello, password, passkey) | PARTIAL | 2 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.10.03 | &nbsp;&nbsp;&nbsp;&nbsp;User switching & Ctrl+Alt+Del screen | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.10.04 | &nbsp;&nbsp;&nbsp;&nbsp;Shutdown/restart/sleep/hibernate | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.10.05 | &nbsp;&nbsp;&nbsp;&nbsp;Update-and-restart flows | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.11 | OOBE / first-run | PARTIAL | Group researched in Part 3; see child items. |
| WA.11.01 | &nbsp;&nbsp;&nbsp;&nbsp;Region/keyboard | PARTIAL | 3 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.11.02 | &nbsp;&nbsp;&nbsp;&nbsp;Network requirement | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.11.03 | &nbsp;&nbsp;&nbsp;&nbsp;Microsoft account requirement & local account paths | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.11.04 | &nbsp;&nbsp;&nbsp;&nbsp;Privacy pages | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.11.05 | &nbsp;&nbsp;&nbsp;&nbsp;Restore from backup | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.12 | UAC & permission prompts | PARTIAL | Group researched in Part 3; see child items. |
| WA.12.01 | &nbsp;&nbsp;&nbsp;&nbsp;Consent vs credential prompts | PARTIAL | 2 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.12.02 | &nbsp;&nbsp;&nbsp;&nbsp;Secure desktop | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.12.03 | &nbsp;&nbsp;&nbsp;&nbsp;Admin protection (if available) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.12.04 | &nbsp;&nbsp;&nbsp;&nbsp;App capability prompts (camera/mic/location) | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13 | File Explorer | PARTIAL | Group researched in Part 3; see child items. |
| WA.13.01 | &nbsp;&nbsp;&nbsp;&nbsp;Home, Gallery, Quick Access | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.02 | &nbsp;&nbsp;&nbsp;&nbsp;Navigation pane | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.03 | &nbsp;&nbsp;&nbsp;&nbsp;Address bar/breadcrumbs | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.04 | &nbsp;&nbsp;&nbsp;&nbsp;Tabs | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.05 | &nbsp;&nbsp;&nbsp;&nbsp;Command bar & menus | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.06 | &nbsp;&nbsp;&nbsp;&nbsp;Views, sort, group | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.07 | &nbsp;&nbsp;&nbsp;&nbsp;Selection model | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.08 | &nbsp;&nbsp;&nbsp;&nbsp;Details/preview pane | PARTIAL | 2 T1/T2 and 4 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.09 | &nbsp;&nbsp;&nbsp;&nbsp;Search in Explorer | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.10 | &nbsp;&nbsp;&nbsp;&nbsp;Copy/move progress dialog | PARTIAL | Only T4/T5 evidence (2 sources); no official Windows 11 article describes this feature. Low–medium confidence. |
| WA.13.11 | &nbsp;&nbsp;&nbsp;&nbsp;Conflict resolution | PARTIAL | Only T4/T5 evidence (3 sources); no official Windows 11 article describes this feature. Low–medium confidence. |
| WA.13.12 | &nbsp;&nbsp;&nbsp;&nbsp;Rename (incl. voice typing 26H2) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.13 | &nbsp;&nbsp;&nbsp;&nbsp;Delete & Recycle Bin | PARTIAL | Only T4/T5 evidence (5 sources); no official Windows 11 article describes this feature. Low–medium confidence. |
| WA.13.14 | &nbsp;&nbsp;&nbsp;&nbsp;Context menus (modern/legacy) | PARTIAL | 2 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.15 | &nbsp;&nbsp;&nbsp;&nbsp;Drag & drop semantics | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.16 | &nbsp;&nbsp;&nbsp;&nbsp;Archive formats | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.17 | &nbsp;&nbsp;&nbsp;&nbsp;AI actions | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.13.18 | &nbsp;&nbsp;&nbsp;&nbsp;Properties dialog | PARTIAL | Only T4/T5 evidence (2 sources); no official Windows 11 article describes this feature. Low–medium confidence. |
| WA.14 | Common dialogs | PARTIAL | Group researched in Part 3; see child items. |
| WA.14.01 | &nbsp;&nbsp;&nbsp;&nbsp;Open/Save (IFileDialog) | PARTIAL | 2 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.14.02 | &nbsp;&nbsp;&nbsp;&nbsp;Print dialog (modern/legacy) | PARTIAL | 2 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.14.03 | &nbsp;&nbsp;&nbsp;&nbsp;Share sheet | PARTIAL | 3 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.14.04 | &nbsp;&nbsp;&nbsp;&nbsp;Folder picker | PARTIAL | 2 T1/T2 and 1 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.15 | System-wide tools | PARTIAL | Group researched in Part 3; see child items. |
| WA.15.01 | &nbsp;&nbsp;&nbsp;&nbsp;Clipboard history (Win+V) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.15.02 | &nbsp;&nbsp;&nbsp;&nbsp;Snipping Tool / PrtScn | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.15.03 | &nbsp;&nbsp;&nbsp;&nbsp;Emoji & symbols panel (Win+.) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.15.04 | &nbsp;&nbsp;&nbsp;&nbsp;Touch keyboard/IME | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.15.05 | &nbsp;&nbsp;&nbsp;&nbsp;Voice typing (Win+H) | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.15.06 | &nbsp;&nbsp;&nbsp;&nbsp;Run dialog | PARTIAL | 1 T1/T2 and 2 T4/T5 sources relevant. Official sources are not specific to build 26300.9711; live-build check pending. |
| WA.16 | Legacy Windows (10/7) interface differences | PARTIAL | Only T4/T5 evidence (2 sources); no official Windows 11 article describes this feature. Low–medium confidence. |

### WB — Research B — Windows Settings (26H2) full inventory

| ID | Item | Status | Note |
|---|---|---|---|
| WB.01 | System | PARTIAL | Category researched in Part 2; see child items. |
| WB.01.01 | &nbsp;&nbsp;&nbsp;&nbsp;Display | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.02 | &nbsp;&nbsp;&nbsp;&nbsp;Sound | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.03 | &nbsp;&nbsp;&nbsp;&nbsp;Notifications | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.04 | &nbsp;&nbsp;&nbsp;&nbsp;Focus | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.05 | &nbsp;&nbsp;&nbsp;&nbsp;Power & battery | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.06 | &nbsp;&nbsp;&nbsp;&nbsp;Storage | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.07 | &nbsp;&nbsp;&nbsp;&nbsp;Nearby sharing | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.08 | &nbsp;&nbsp;&nbsp;&nbsp;Multitasking | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.09 | &nbsp;&nbsp;&nbsp;&nbsp;For developers / Advanced | COMPLETE | Official Advanced settings page enumerates the full page contents (updated 2026-02-26); live-build check pending (VP-... |
| WB.01.10 | &nbsp;&nbsp;&nbsp;&nbsp;Activation | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.11 | &nbsp;&nbsp;&nbsp;&nbsp;Troubleshoot | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.12 | &nbsp;&nbsp;&nbsp;&nbsp;Recovery | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.13 | &nbsp;&nbsp;&nbsp;&nbsp;Projecting to this PC | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.14 | &nbsp;&nbsp;&nbsp;&nbsp;Remote Desktop | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.15 | &nbsp;&nbsp;&nbsp;&nbsp;Clipboard | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.16 | &nbsp;&nbsp;&nbsp;&nbsp;Optional features (location) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.17 | &nbsp;&nbsp;&nbsp;&nbsp;About | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.01.18 | &nbsp;&nbsp;&nbsp;&nbsp;Settings Agent (AI) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02 | Bluetooth & devices | PARTIAL | Category researched in Part 2; see child items. |
| WB.02.01 | &nbsp;&nbsp;&nbsp;&nbsp;Devices | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.02 | &nbsp;&nbsp;&nbsp;&nbsp;Printers & scanners | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.03 | &nbsp;&nbsp;&nbsp;&nbsp;Phone Link / mobile devices | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.04 | &nbsp;&nbsp;&nbsp;&nbsp;Cameras | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.05 | &nbsp;&nbsp;&nbsp;&nbsp;Mouse | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.06 | &nbsp;&nbsp;&nbsp;&nbsp;Touchpad | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.07 | &nbsp;&nbsp;&nbsp;&nbsp;Pen & Windows Ink | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.08 | &nbsp;&nbsp;&nbsp;&nbsp;Keyboard / Text input | NOT FOUND | No 'Keyboard' page under Bluetooth & devices was documented in the sources inspected; keyboard options are covered un... |
| WB.02.09 | &nbsp;&nbsp;&nbsp;&nbsp;AutoPlay | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.10 | &nbsp;&nbsp;&nbsp;&nbsp;USB | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.02.11 | &nbsp;&nbsp;&nbsp;&nbsp;Shared Audio | PARTIAL | Only independent or preview-era evidence; no official Microsoft documentation found for this page. |
| WB.03 | Network & internet | PARTIAL | Category researched in Part 2; see child items. |
| WB.03.01 | &nbsp;&nbsp;&nbsp;&nbsp;Wi-Fi | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.02 | &nbsp;&nbsp;&nbsp;&nbsp;Ethernet | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.03 | &nbsp;&nbsp;&nbsp;&nbsp;VPN | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.04 | &nbsp;&nbsp;&nbsp;&nbsp;Mobile hotspot | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.05 | &nbsp;&nbsp;&nbsp;&nbsp;Airplane mode | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.06 | &nbsp;&nbsp;&nbsp;&nbsp;Proxy | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.07 | &nbsp;&nbsp;&nbsp;&nbsp;Dial-up | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.03.08 | &nbsp;&nbsp;&nbsp;&nbsp;Advanced network settings | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.03.09 | &nbsp;&nbsp;&nbsp;&nbsp;Data usage | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04 | Personalization | PARTIAL | Category researched in Part 2; see child items. |
| WB.04.01 | &nbsp;&nbsp;&nbsp;&nbsp;Background | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.02 | &nbsp;&nbsp;&nbsp;&nbsp;Colors | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.03 | &nbsp;&nbsp;&nbsp;&nbsp;Themes | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.04 | &nbsp;&nbsp;&nbsp;&nbsp;Dynamic Lighting | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.05 | &nbsp;&nbsp;&nbsp;&nbsp;Lock screen | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.06 | &nbsp;&nbsp;&nbsp;&nbsp;Text input | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.04.07 | &nbsp;&nbsp;&nbsp;&nbsp;Start | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.08 | &nbsp;&nbsp;&nbsp;&nbsp;Taskbar | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.09 | &nbsp;&nbsp;&nbsp;&nbsp;Fonts | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.04.10 | &nbsp;&nbsp;&nbsp;&nbsp;Device usage | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.05 | Apps | PARTIAL | Category researched in Part 2; see child items. |
| WB.05.01 | &nbsp;&nbsp;&nbsp;&nbsp;Installed apps | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.05.02 | &nbsp;&nbsp;&nbsp;&nbsp;Advanced app settings | PARTIAL | Only community Q&A evidence; official documentation not found. |
| WB.05.03 | &nbsp;&nbsp;&nbsp;&nbsp;Default apps | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.05.04 | &nbsp;&nbsp;&nbsp;&nbsp;Offline maps | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.05.05 | &nbsp;&nbsp;&nbsp;&nbsp;Optional features | NOT APPLICABLE | Optional features sits under System > Optional features in current documentation (see WB.01.16). |
| WB.05.06 | &nbsp;&nbsp;&nbsp;&nbsp;Apps for websites | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.05.07 | &nbsp;&nbsp;&nbsp;&nbsp;Video playback | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.05.08 | &nbsp;&nbsp;&nbsp;&nbsp;Startup | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.05.09 | &nbsp;&nbsp;&nbsp;&nbsp;Actions | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.06 | Accounts | PARTIAL | Category researched in Part 2; see child items. |
| WB.06.01 | &nbsp;&nbsp;&nbsp;&nbsp;Your info | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.02 | &nbsp;&nbsp;&nbsp;&nbsp;Email & accounts | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.03 | &nbsp;&nbsp;&nbsp;&nbsp;Sign-in options | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.04 | &nbsp;&nbsp;&nbsp;&nbsp;Passkeys | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.05 | &nbsp;&nbsp;&nbsp;&nbsp;Family | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.06 | &nbsp;&nbsp;&nbsp;&nbsp;Windows backup | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.07 | &nbsp;&nbsp;&nbsp;&nbsp;Other users | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.06.08 | &nbsp;&nbsp;&nbsp;&nbsp;Access work or school | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.07 | Time & language | PARTIAL | Category researched in Part 2; see child items. |
| WB.07.01 | &nbsp;&nbsp;&nbsp;&nbsp;Date & time | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.07.02 | &nbsp;&nbsp;&nbsp;&nbsp;Language & region | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.07.03 | &nbsp;&nbsp;&nbsp;&nbsp;Typing | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.07.04 | &nbsp;&nbsp;&nbsp;&nbsp;Speech | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.08 | Gaming | PARTIAL | Category researched in Part 2; see child items. |
| WB.08.01 | &nbsp;&nbsp;&nbsp;&nbsp;Game Bar | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.08.02 | &nbsp;&nbsp;&nbsp;&nbsp;Captures | PARTIAL | Only community Q&A evidence; official documentation not found. |
| WB.08.03 | &nbsp;&nbsp;&nbsp;&nbsp;Game Mode | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.09 | Accessibility | PARTIAL | Category researched in Part 2; see child items. |
| WB.09.01 | &nbsp;&nbsp;&nbsp;&nbsp;Text size | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.02 | &nbsp;&nbsp;&nbsp;&nbsp;Visual effects | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.03 | &nbsp;&nbsp;&nbsp;&nbsp;Mouse pointer & touch | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.04 | &nbsp;&nbsp;&nbsp;&nbsp;Text cursor | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.05 | &nbsp;&nbsp;&nbsp;&nbsp;Magnifier | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.06 | &nbsp;&nbsp;&nbsp;&nbsp;Color filters | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.07 | &nbsp;&nbsp;&nbsp;&nbsp;Contrast themes | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.08 | &nbsp;&nbsp;&nbsp;&nbsp;Narrator | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.09 | &nbsp;&nbsp;&nbsp;&nbsp;Audio | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.10 | &nbsp;&nbsp;&nbsp;&nbsp;Captions | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.11 | &nbsp;&nbsp;&nbsp;&nbsp;Speech / Voice access | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.12 | &nbsp;&nbsp;&nbsp;&nbsp;Keyboard | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.09.13 | &nbsp;&nbsp;&nbsp;&nbsp;Mouse | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.09.14 | &nbsp;&nbsp;&nbsp;&nbsp;Eye control | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.10 | Privacy & security | PARTIAL | Category researched in Part 2; see child items. |
| WB.10.01 | &nbsp;&nbsp;&nbsp;&nbsp;Windows Security | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.10.02 | &nbsp;&nbsp;&nbsp;&nbsp;Find my device | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.10.03 | &nbsp;&nbsp;&nbsp;&nbsp;Device encryption / BitLocker | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.10.04 | &nbsp;&nbsp;&nbsp;&nbsp;For developers | NOT APPLICABLE | For developers moved to System > Advanced in 25H2 and later (see WB.01.09). |
| WB.10.05 | &nbsp;&nbsp;&nbsp;&nbsp;Windows permissions (general, speech, inking, diagnostics, activity, search) | PARTIAL | Children WB.10.05.01–.05 documented. |
| WB.10.06 | &nbsp;&nbsp;&nbsp;&nbsp;App permissions (location, camera, mic, etc.) | PARTIAL | Children WB.10.06.01–.03 documented. |
| WB.11 | Windows Update | PARTIAL | Category researched in Part 2; see child items. |
| WB.11.01 | &nbsp;&nbsp;&nbsp;&nbsp;Updates & pause | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.11.02 | &nbsp;&nbsp;&nbsp;&nbsp;Update history | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.11.03 | &nbsp;&nbsp;&nbsp;&nbsp;Advanced options | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.11.04 | &nbsp;&nbsp;&nbsp;&nbsp;Windows Insider Program | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.11.05 | &nbsp;&nbsp;&nbsp;&nbsp;Restart workflows & active hours | PARTIAL | Restart scheduling and active hours are covered in WB.11.01 and WB.11.03. |
| WB.12 | Beyond Settings | PARTIAL | Category researched in Part 2; see child items. |
| WB.12.01 | &nbsp;&nbsp;&nbsp;&nbsp;ms-settings URI register (229 distinct URIs on MS Learn) | COMPLETE | Full MS Learn URI table parsed (Appendix 2B). Live resolution test pending (VP-02). |
| WB.12.02 | &nbsp;&nbsp;&nbsp;&nbsp;Control Panel applets | PARTIAL | Official source confirms the page exists but documents few or none of its controls. |
| WB.12.03 | &nbsp;&nbsp;&nbsp;&nbsp;Windows Security app | PARTIAL | See WB.10.01 (Windows Security app). |
| WB.12.04 | &nbsp;&nbsp;&nbsp;&nbsp;Device Manager | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.12.05 | &nbsp;&nbsp;&nbsp;&nbsp;Task Manager (incl. NPU) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.12.06 | &nbsp;&nbsp;&nbsp;&nbsp;Disk Management | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.12.07 | &nbsp;&nbsp;&nbsp;&nbsp;Legacy dialogs (sysdm.cpl etc.) | PARTIAL | Controls documented by official 'Windows 11' support content that isn't build-specific; not yet verified against a li... |
| WB.12.08 | &nbsp;&nbsp;&nbsp;&nbsp;Group Policy / MDM-only settings | PARTIAL | Part 29 mechanism (S3888-S3890); Part 31 enumeration method (S3891-S3892); list not extracted |
| WB.12.09 | &nbsp;&nbsp;&nbsp;&nbsp;SKU differences (Home/Pro/Enterprise/Education) | PARTIAL | SKU/hardware conditions captured per page where sources state them; no consolidated Home/Pro/Enterprise matrix yet. |

### WC — Research A3 — Fluent / WinUI design system

| ID | Item | Status | Note |
|---|---|---|---|
| WC.01 | Design principles | PARTIAL | Principles documented by Microsoft Learn and Fluent 2; qualitative only, no exhaustive list. |
| WC.02 | Color & theming tokens | PARTIAL | 83 core Color tokens × 3 themes enumerated from source; SystemAccentColor shades are computed at runtime (not in XAML... |
| WC.03 | Typography ramp (Segoe UI Variable) | COMPLETE | Type ramp sizes (source) + line heights/weights/axes (docs) agree; naming difference recorded as GAP-B3-04. |
| WC.04 | Spacing & layout grid | PARTIAL | 4 epx base unit, spacing guidance and breakpoints documented; no exhaustive spacing token set exists in source. |
| WC.05 | Geometry & corner radius | COMPLETE | ControlCornerRadius 4 / OverlayCornerRadius 8 (source) match docs; Win32 DWMWCP enum documented. |
| WC.06 | Elevation & shadows | PARTIAL | Elevation depths documented; shadow blur/offset/opacity of ThemeShadow are not published. |
| WC.07 | Materials: Mica, Mica Alt, Acrylic, Smoke | PARTIAL | Acrylic recipe constants from source; Mica/Mica Alt tint parameters live in closed WinAppSDK/DWM code. |
| WC.08 | Iconography (Segoe Fluent Icons) | PARTIAL | Glyph table counted (1,574 unique code points); count conflicts across pages (GAP-B3-01). |
| WC.09 | Motion: timing & easing | PARTIAL | Duration/easing tokens from source; docs disagree on exit curve and legacy durations (GAP-B3-02/03). |
| WC.10 | Control states (rest/hover/pressed/disabled/focus) | PARTIAL | State brushes for 21 resource files (Button, CheckBox, ToggleSwitch, TextBox, etc.) captured; other controls not down... |
| WC.11 | Focus visuals | PARTIAL | Two-tone focus visual (2px/1px/1px margin) documented; framework defaults not in downloaded XAML. |
| WC.12 | Win32 / common controls conventions | PARTIAL | Win32 UX guide (legacy), ComCtl32 v6 visual styles, DWM attributes and dark mode guidance captured; per-control Win32... |
| WC.13 | Loading, error, empty states | PARTIAL | ProgressRing/ProgressBar/InfoBar/ContentDialog guidance and dimensions captured; no official empty-state pattern found. |

### MC — Research C — macOS 27 desktop model

| ID | Item | Status | Note |
|---|---|---|---|
| MC.01 | Menu bar | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.01.01 | &nbsp;&nbsp;&nbsp;&nbsp;Apple menu | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.01.02 | &nbsp;&nbsp;&nbsp;&nbsp;App menu & standard menus | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.01.03 | &nbsp;&nbsp;&nbsp;&nbsp;Menu item image visibility (27 change) | COMPLETE | SDK-dependent menu-image rules are documented in the macOS 27 release notes (T1) and corroborated by 5 independent so... |
| MC.01.04 | &nbsp;&nbsp;&nbsp;&nbsp;Status menus / menu extras | PARTIAL | Apple documents Command-drag rearranging; macOS 27 overflow control and MenuBarAgent changes are third-party evidence... |
| MC.01.05 | &nbsp;&nbsp;&nbsp;&nbsp;Live Activities | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.01.06 | &nbsp;&nbsp;&nbsp;&nbsp;Menu bar hiding & customization | PARTIAL | Menu Bar pane options captured (Automatically hide and show the menu bar: Always/On Desktop Only/In Full Screen Only/... |
| MC.02 | Dock | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.02.01 | &nbsp;&nbsp;&nbsp;&nbsp;Placement/size/magnification | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.02.02 | &nbsp;&nbsp;&nbsp;&nbsp;App indicators | PARTIAL | Background-process indicator only in independent beta reports; release notes reference a fixed bug in it (GAP-B5-02). |
| MC.02.03 | &nbsp;&nbsp;&nbsp;&nbsp;Context menu incl. background processes (27) | PARTIAL | Dock shortcut menu items documented; the new background-activity menu item name is unverified (GAP-B5-02). |
| MC.02.04 | &nbsp;&nbsp;&nbsp;&nbsp;Stacks | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.02.05 | &nbsp;&nbsp;&nbsp;&nbsp;Minimize effects | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03 | Finder | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.03.01 | &nbsp;&nbsp;&nbsp;&nbsp;Windows/tabs/views | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.02 | &nbsp;&nbsp;&nbsp;&nbsp;Sidebar | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.03 | &nbsp;&nbsp;&nbsp;&nbsp;Tags | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.04 | &nbsp;&nbsp;&nbsp;&nbsp;Search & smart folders | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.05 | &nbsp;&nbsp;&nbsp;&nbsp;Quick Look | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.06 | &nbsp;&nbsp;&nbsp;&nbsp;Get Info | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.07 | &nbsp;&nbsp;&nbsp;&nbsp;Copy/move/conflict | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.08 | &nbsp;&nbsp;&nbsp;&nbsp;Trash & recovery | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.03.09 | &nbsp;&nbsp;&nbsp;&nbsp;Desktop & stacks | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04 | Window model | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.04.01 | &nbsp;&nbsp;&nbsp;&nbsp;App vs window relationship | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.02 | &nbsp;&nbsp;&nbsp;&nbsp;Active/inactive appearance | PARTIAL | Corner radius and shadow changes reported qualitatively; no numeric values published (GAP-B5-07). |
| MC.04.03 | &nbsp;&nbsp;&nbsp;&nbsp;Traffic-light controls (27 redesign) | PARTIAL | Redesign corroborated by 6 independent sources; no Apple T1 description (GAP-B5-01, supersedes Part 1 GAP-007). |
| MC.04.04 | &nbsp;&nbsp;&nbsp;&nbsp;Full screen | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.05 | &nbsp;&nbsp;&nbsp;&nbsp;Window tiling & Move & Resize | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.06 | &nbsp;&nbsp;&nbsp;&nbsp;Sheets & panels | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.04.07 | &nbsp;&nbsp;&nbsp;&nbsp;Window restoration/persistence | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.05 | Spaces / Mission Control / Stage Manager | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.05.01 | &nbsp;&nbsp;&nbsp;&nbsp;Spaces | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.05.02 | &nbsp;&nbsp;&nbsp;&nbsp;Mission Control | PARTIAL | Guide captured; third-party reports of changed Dock accessibility notifications and animation bugs (GAP-B5-06). |
| MC.05.03 | &nbsp;&nbsp;&nbsp;&nbsp;App Exposé | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.05.04 | &nbsp;&nbsp;&nbsp;&nbsp;Stage Manager | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.06 | Spotlight | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.06.01 | &nbsp;&nbsp;&nbsp;&nbsp;Search | PARTIAL | Spotlight guide captured; Spotlight menu-bar icon status conflicts (GAP-B5-05). |
| MC.06.02 | &nbsp;&nbsp;&nbsp;&nbsp;Actions & quick keys | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.06.03 | &nbsp;&nbsp;&nbsp;&nbsp;Intelligence/Siri integration | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07 | Control Center & Notification Center | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.07.01 | &nbsp;&nbsp;&nbsp;&nbsp;Modules & customization | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.02 | &nbsp;&nbsp;&nbsp;&nbsp;Third-party controls | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.03 | &nbsp;&nbsp;&nbsp;&nbsp;Notification grouping | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.07.04 | &nbsp;&nbsp;&nbsp;&nbsp;Widgets on desktop | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08 | Dialogs & panels | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.08.01 | &nbsp;&nbsp;&nbsp;&nbsp;Open/Save panels (Recents Cmd-Shift-F, 27) | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.02 | &nbsp;&nbsp;&nbsp;&nbsp;Print | PARTIAL | Mac User Guide print page captured; HIG Printing page inaccessible (GAP-B5-11). |
| MC.08.03 | &nbsp;&nbsp;&nbsp;&nbsp;Share | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.04 | &nbsp;&nbsp;&nbsp;&nbsp;Services menu | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.08.05 | &nbsp;&nbsp;&nbsp;&nbsp;Alerts | PARTIAL | Only HIG guidance (developer, not OS behaviour) and the NSAlert release-note item. |
| MC.09 | Input | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.09.01 | &nbsp;&nbsp;&nbsp;&nbsp;Keyboard shortcuts | COMPLETE | 208 system shortcuts parsed from Apple KB 102650 (published 2026-09-14) and the tiling-shortcut page; app-specific sh... |
| MC.09.02 | &nbsp;&nbsp;&nbsp;&nbsp;Trackpad gestures | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.09.03 | &nbsp;&nbsp;&nbsp;&nbsp;Full Keyboard Access | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.09.04 | &nbsp;&nbsp;&nbsp;&nbsp;Screenshots (Cmd-Shift-3/4/5) | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10 | Session | PARTIAL | Researched in Part 6 (Chapter 14). |
| MC.10.01 | &nbsp;&nbsp;&nbsp;&nbsp;Login window | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.02 | &nbsp;&nbsp;&nbsp;&nbsp;Fast user switching | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.03 | &nbsp;&nbsp;&nbsp;&nbsp;Lock | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.04 | &nbsp;&nbsp;&nbsp;&nbsp;Force Quit | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.05 | &nbsp;&nbsp;&nbsp;&nbsp;Startup/shutdown/restart | PARTIAL | Apple guide elements/behaviours captured; geometry, timing and defaults not live-verified (VP-18). |
| MC.10.06 | &nbsp;&nbsp;&nbsp;&nbsp;Setup Assistant | PARTIAL | Only a model-specific guide (MacBook Pro) describes Setup Assistant steps (GAP-B5-09). |

### MD — Research D — macOS 27 System Settings inventory

| ID | Item | Status | Note |
|---|---|---|---|
| MD.01 | Sidebar enumeration (exact order, hardware-dependent items) | PARTIAL | Pane set evidenced from 27 guide navigation paths; canonical sidebar order not published by Apple (GAP-B4-01). |
| MD.02 | Apple Account | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.03 | Family | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.04 | Wi-Fi | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.05 | Bluetooth | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.06 | Network | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.07 | VPN | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.08 | Battery / Energy | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.09 | General (About, Software Update, Storage, AppleCare, AirDrop & Handoff, Login Items & Extensions, Language & Region, Date & Time, Sharing, Time Machine, Transfer or Reset, Startup Disk, Device Management) | PARTIAL | 12 of 13 General subpages from the 27 guide; AppleCare & Warranty and AutoFill & Passwords only from support articles... |
| MD.10 | Accessibility | PARTIAL | 25 Accessibility subpages captured; label conflicts with independent 27 reports (GAP-B4-03, 04). |
| MD.11 | Appearance | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.12 | Apple Intelligence & Siri | PARTIAL | Guide documents pane as "Siri"; Apple Intelligence master switch reported removed (GAP-B4-05, 06). |
| MD.13 | Control Center | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.14 | Desktop & Dock | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.15 | Displays | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.16 | Spotlight | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.17 | Wallpaper | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.18 | Screen Saver | PARTIAL | Screen Saver page captured; whether it is a separate sidebar item or reached via Wallpaper is unverified (GAP-B4-09). |
| MD.19 | Notifications | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.20 | Sound | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.21 | Focus | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.22 | Screen Time | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.23 | Lock Screen | PARTIAL | Guide lacks the user-name/photo visibility switch reported by geeksmodo (GAP-B4-08). |
| MD.24 | Privacy & Security | PARTIAL | Privacy list captured; "Accessibility" vs "Device Control and Data Access" conflict (GAP-B4-02). |
| MD.25 | Touch ID & Password / Login Password | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.26 | Users & Groups | PARTIAL | Users & Groups captured; legacy login-options topic redirects (GAP-B4-12). |
| MD.27 | Internet Accounts | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.28 | Game Center | PARTIAL | Game Center settings now documented in the Games User Guide (1.0 / macOS 27), not the Mac User Guide. |
| MD.29 | Wallet & Apple Pay | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.30 | Keyboard | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.31 | Mouse | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.32 | Trackpad | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.33 | Game Controllers | PARTIAL | Only Games User Guide coverage; MDM identifier com.apple.Game-Controller-Settings.extension listed by Apple (GAP-B4-10). |
| MD.34 | Printers & Scanners | PARTIAL | Apple Mac User Guide 27 controls captured; defaults rarely stated; not live-verified (VP-13). |
| MD.35 | Menu Bar (if separate in 27) | PARTIAL | Menu Bar confirmed as its own sidebar pane in 27 (resolves GAP-006). |
| MD.36 | App Settings conventions (Cmd-,) | COMPLETE | Command-Comma and settings-window rules documented by Apple HIG and the Mac shortcuts page; HIG detail continues in P... |
| MD.37 | x-apple.systempreferences URL schemes | PARTIAL | 209 URL rows; extension IDs listed by Apple for MDM, URL-scheme use undocumented and 27 validity untested (GAP-B4-13). |
| MD.38 | Configuration profiles / MDM | PARTIAL | SystemPreferences payload (deprecated), Restrictions keys and 27 payload removals captured; DDM replacements only par... |

### MG — macOS design system (HIG)

| ID | Item | Status | Note |
|---|---|---|---|
| MG.01 | Liquid Glass materials | PARTIAL | API surface and design rules are verified at T1. The slider has no public API or stored-value documentation, there is... |
| MG.02 | Color & semantic colors | PARTIAL | Semantic names, availability and settings are verified at T1. Numeric values for the macOS 26/27 system colors were n... |
| MG.03 | SF Pro typography & Dynamic Type equivalents | PARTIAL | Text styles, tracking and minimum sizes are verified at T1. The HIG typography revision is from 2025, and no macOS 27... |
| MG.04 | SF Symbols | PARTIAL | Library facts and APIs are verified at T1. Per-symbol inventories were not taken, because copying glyphs is excluded... |
| MG.05 | Layout & spacing | PARTIAL | Layout rules and APIs are verified at T1. Apple publishes no numeric spacing scale for macOS 26/27. |
| MG.06 | Corner radii/concentricity | PARTIAL | The concentricity model and APIs are verified at T1. Numeric window and control radii are unpublished; macOS 26 value... |
| MG.07 | Motion | PARTIAL | Framework animation defaults are verified at T1 with exact parameters. System-shell motion timings are unpublished. |
| MG.08 | Control sizes (mini/small/regular/large/extra-large) | PARTIAL | The size ladder, shapes and compatibility flag are verified at T1. Heights are unpublished, and the SwiftUI extraLarg... |
| MG.09 | Dark mode & increased contrast | PARTIAL | Appearance names, APIs and glass behaviour are verified at T1. How the settings are coupled is known only from indepe... |
| MG.10 | Accessibility semantics (NSAccessibility) | PARTIAL | The role, subrole and protocol inventory is complete for the fetched pages. Per-control role mapping and the notifica... |

### LG — Research E — GNOME 51

| ID | Item | Status | Note |
|---|---|---|---|
| LG.01 | Shell components & compositor | PARTIAL | GNOME Shell + Mutter; 51 reworked frame scheduling, cheaper capture, legacy NVIDIA removed, SVG cursors (S3856); Litr... |
| LG.02 | Launcher | PARTIAL | Activities overview: search + app grid; 51 made screen-reader result descriptions and the app grid easier to navigate... |
| LG.03 | Panel / dock | PARTIAL | One top bar; the dash lives in the overview and a persistent dock is an extension (S3856, S3857); LitricOS: one bar p... |
| LG.04 | System Settings inventory | PARTIAL | ~20 panels; 51 changed Displays (auto-rotate, orientation lock, centre snapping), Mouse, Network (DNS search, WEP rem... |
| LG.05 | File manager | PARTIAL | Nautilus 51: drag-count badge, copied files auto-selected, right-click on empty area keeps the selection, emblem prec... |
| LG.06 | Window behaviour & tiling | PARTIAL | No first-class tiling in the 51 notes; window behaviour is Mutter's (S3856, absence); LitricOS: 36 WM ops, one output... |
| LG.07 | Notifications | PARTIAL | Shell is the notification server; Files notifications grouped in 51 (S3856); LitricOS: private queue, banner open/clo... |
| LG.08 | Workspace model | PARTIAL | Dynamic vertical workspaces, unchanged in 51 and therefore absent from its notes (S3856); LitricOS: fixed grid, Missi... |
| LG.09 | Customization model | PARTIAL | Extensions are the extension point; appearance deliberately narrow, with new wallpaper variants in 51 (S3856, S3857);... |
| LG.10 | Design language / HIG / tokens | COMPLETE | HIG is five parts: principles, resources, guidelines, patterns, reference (S3857); 51 extends reduced motion into wid... |
| LG.11 | Accessibility | PARTIAL | 51: motion widened, screenshot region adjustable by keyboard, focus-ring timeout can be turned off, dock announced on... |
| LG.12 | Limitations & known issues | PARTIAL | Legacy NVIDIA interfaces and WEP removed (S3856); extensions break across major releases; LitricOS: one output, one s... |
| LG.13 | Integration points (portals, D-Bus, APIs) | PARTIAL | Portals, D-Bus, GSettings/dconf, systemd user units, Flatpak (S3857); LitricOS: LFP/1 + browser socket + notifyd, no... |
| LG.14 | DE vs kernel vs distro boundary | PARTIAL | DE owns shell, compositor, settings and session apps; kernel, logind, NetworkManager, BlueZ and PipeWire are outside,... |

### LK — Research E — KDE Plasma 6.8

| ID | Item | Status | Note |
|---|---|---|---|
| LK.01 | Shell components & compositor | PARTIAL | plasmashell + KWin on Wayland; the 6.8 announcement yielded no feature list (S3860); LitricOS: one process draws bar,... |
| LK.02 | Launcher | PARTIAL | Kickoff and KRunner; HIG forbids asking preferences in a first-run wizard (S3859); LitricOS: one overlay, no wizard,... |
| LK.03 | Panel / dock | PARTIAL | Configurable panels; no KDE page read in this batch (gap); LitricOS: fixed top bar, dock is an overlay state |
| LK.04 | System Settings inventory | PARTIAL | System Settings modules not enumerated by any page read in this batch (gap); LitricOS: 13 pages, 9 stored keys |
| LK.05 | File manager | PARTIAL | Dolphin; HIG requires inline rename, trash instead of delete, and undo for content that is not trivial to re-add (S38... |
| LK.06 | Window behaviour & tiling | PARTIAL | KWin tiling and quick tile; no page read in this batch (gap); LitricOS: 36 WM ops, no tiling model |
| LK.07 | Notifications | PARTIAL | Plasma notification server; HIG separates passive notification from persistent inline message (S3859); LitricOS: TTL... |
| LK.08 | Workspace model | PARTIAL | Virtual desktops plus Activities; no page read in this batch (gap); LitricOS: fixed workspace grid, no activities |
| LK.09 | Customization model | PARTIAL | Simple by default, powerful when needed (S3859); LitricOS: closed theme and accent set, no third-party surface |
| LK.10 | Design language / HIG / tokens | COMPLETE | Nine HIG sections; tokens Kirigami.Theme, iconsSizes, small/medium/largeSpacing, veryShort/short/long/veryLongDuratio... |
| LK.11 | Accessibility | PARTIAL | HIG has an Accessibility and inclusiveness section; the page was not read in this batch (gap); LitricOS: 1 toggle, no... |
| LK.12 | Limitations & known issues | COMPLETE | HIG states window size and position are remembered on X11 only, i.e. not on Wayland (S3859); LitricOS restores neithe... |
| LK.13 | Integration points (portals, D-Bus, APIs) | PARTIAL | D-Bus, KIO, KConfig, KWallet, KRecentDocument, portals (S3859); LitricOS: no bus, no key store, no KIO equivalent |
| LK.14 | DE vs kernel vs distro boundary | PARTIAL | Same boundary as GNOME: DRM, logind, NetworkManager, BlueZ and PipeWire outside the DE, distribution packages the DE... |

### LX — Research E — Xfce 4.20

| ID | Item | Status | Note |
|---|---|---|---|
| LX.01 | Shell components & compositor | PARTIAL | Components ported to Wayland through libxfce4windowing; xfwm4, xfdashboard and the screensaver are not, and Labwc or... |
| LX.02 | Launcher | PARTIAL | xfce4-appfinder: Ctrl+N/P navigation, Ctrl+Enter executes, per-app actions on right-click, optional daemon (S3861, S3... |
| LX.03 | Panel / dock | COMPLETE | xfce4-panel: border width, XDG plugin discovery, multi-delete, LCD/analog clock options, show desktop on hover, taskl... |
| LX.04 | System Settings inventory | PARTIAL | Appearance, Display (profiles, both-dimension scaling, mirror without matching refresh), Mouse (acceleration profile,... |
| LX.05 | File manager | COMPLETE | Thunar 4.20: toolbar overflow, view switcher, tabs, CSD, split view, expandable folders, bulk renamer, undo asks befo... |
| LX.06 | Window behaviour & tiling | PARTIAL | xfwm4: click-to-focus or focus-follows-mouse, raise delay, snapping with distance, workspace wrap with resistance, op... |
| LX.07 | Notifications | PARTIAL | libnotify is an optional Thunar dependency for mount notifications (S3864); no xfce4-notifyd page read; LitricOS: not... |
| LX.08 | Workspace model | PARTIAL | Workspace wrap at screen edges (S3863); workspace support missing on Wayland because the protocol arrived late (S3861... |
| LX.09 | Customization model | PARTIAL | Window-decoration themes, panel plugins, shortcut editor with overlap groups, Xfconf, systemd user unit (S3861, S3862... |
| LX.10 | Design language / HIG / tokens | PARTIAL | No Xfce HIG page located; rules are GTK/Adwaita defaults plus per-component options, and icon scaling above 1 was fix... |
| LX.11 | Accessibility | PARTIAL | No accessibility content in the tour, changelog or xfwm4 page (gap); LitricOS: 1 toggle honoured everywhere, no a11y... |
| LX.12 | Limitations & known issues | COMPLETE | Wayland is experimental: no compositor, missing workspaces, tray icons, keyboard/mouse settings, brightness keys and... |
| LX.13 | Integration points (portals, D-Bus, APIs) | COMPLETE | libxfce4windowing, minimal xdg-desktop-portal conf, D-Bus, xfconf systemd user unit, optional polkit, power-profiles-... |
| LX.14 | DE vs kernel vs distro boundary | PARTIAL | Xfce owns panel, session, settings and Thunar; the Wayland compositor is external; kernel, logind, NetworkManager and... |

### LC — Research E — Cinnamon (Mint 22.3 / 27)

| ID | Item | Status | Note |
|---|---|---|---|
| LC.01 | Shell components & compositor | PARTIAL | Cinnamon on Muffin; 6.6 improved Muffin's Wayland compatibility (S3865); LitricOS: one shell, own compositor, 3 globals |
| LC.02 | Launcher | COMPLETE | Menu redesigned: sidebar with avatar, places and favourites, smaller categories, configurable search and system-butto... |
| LC.03 | Panel / dock | PARTIAL | Panel applets: workspace switcher, XKB+IBus keyboard applet, Night Light (S3865); LitricOS: fixed bar and dock, no ap... |
| LC.04 | System Settings inventory | PARTIAL | No Cinnamon Settings page read; 6.6 touched keyboard layouts, fractional-scale direction, Alt-Tab scope and theme gro... |
| LC.05 | File manager | COMPLETE | Nemo 6.6: template manager, pause/resume of file operations, wildcard MIME and regex search, split-pane location kept... |
| LC.06 | Window behaviour & tiling | PARTIAL | Muffin: Alt-Tab limited to the current monitor, hot corners in fullscreen, fractional scaling up or down (S3865); Lit... |
| LC.07 | Notifications | PARTIAL | A missed notification shows a badge in the window list (S3865); LitricOS: count badge on the bell only, no per-window... |
| LC.08 | Workspace model | PARTIAL | Workspace switcher shows only visible windows with their icons (S3865); LitricOS: fixed grid and Mission Control card... |
| LC.09 | Customization model | PARTIAL | Menu element visibility, themes grouped by family and variant, applets, fractional-scale direction (S3865); LitricOS:... |
| LC.10 | Design language / HIG / tokens | PARTIAL | XSI replaces Adwaita symbolic icons after GNOME restricted them; all Cinnamon and Mint projects migrated (S3865); no... |
| LC.11 | Accessibility | PARTIAL | Native on-screen keyboard with input-method and layout switching (S3865); no accessibility page read; LitricOS: 1 tog... |
| LC.12 | Limitations & known issues | COMPLETE | Known issues: HWE 6.14 breaks VirtualBox and the 470 NVIDIA driver, 10 s shutdown timeout, PipeWire with a PulseAudio... |
| LC.13 | Integration points (portals, D-Bus, APIs) | PARTIAL | XApp and XSI, apt:// handling, Update Manager tray, Thunderbolt tool, Night Light (S3865); no portal page read; Litri... |
| LC.14 | DE vs kernel vs distro boundary | COMPLETE | Ubuntu 24.04 base with kernel 6.14; PipeWire, systemd, logind and the package manager are distribution-owned (S3865,... |

### LO — Research E — COSMIC Epoch 1.11

| ID | Item | Status | Note |
|---|---|---|---|
| LO.01 | Shell components & compositor | COMPLETE | Wayland-native Rust desktop; shell components and compositor share one toolkit; cosmic-comp on smithay with damage tr... |
| LO.02 | Launcher | PARTIAL | Launcher and app library are separate surfaces; the workspaces overview can open either on typing (S3867, S3870); 1.1... |
| LO.03 | Panel / dock | COMPLETE | Single or dual panels on any edge, applets at start/centre/end, size, opacity, square or rounded corners; applets are... |
| LO.04 | System Settings inventory | PARTIAL | Settings pages selectable per distribution by cargo feature flags, e.g. Pop!_OS Upgrades and Restore; pages touched b... |
| LO.05 | File manager | COMPLETE | Files: tabs, several directories in separate tabs, tab drag-and-drop, Move to / Copy to, clipboard paste of images, v... |
| LO.06 | Window behaviour & tiling | PARTIAL | Tiling is a first-class flexible element; point releases fix tiled state on maximize/snap, Super+Click on tiled windo... |
| LO.07 | Notifications | PARTIAL | cosmic-notifications accepts file:// icons; the status area runs a socket-activated StatusNotifierWatcher so tray ico... |
| LO.08 | Workspace model | COMPLETE | Workspaces dynamic or pinned, and the overview's action on typing is a setting (S3867, S3870); LitricOS: fixed grid,... |
| LO.09 | Customization model | COMPLETE | Theming built for brand palettes without losing contrast, RON theme files from the CLI, Qt theming via qt5ct/qt6ct, d... |
| LO.10 | Design language / HIG / tokens | PARTIAL | libcosmic is based on COSMIC's design language with consistent guidelines, a standardized configuration system and cr... |
| LO.11 | Accessibility | PARTIAL | No accessibility page, screen-reader statement or on-screen keyboard in the eleven pages opened; the only accessibili... |
| LO.12 | Limitations & known issues | PARTIAL | The point-release log publicly lists fixed failure classes: tiling-placeholder panic, smithay image-copy leak, portal... |
| LO.13 | Integration points (portals, D-Bus, APIs) | COMPLETE | xdg-desktop-portal-cosmic for screencast, clipboard, background and screenshot; StatusNotifierWatcher; AccountsServic... |
| LO.14 | DE vs kernel vs distro boundary | COMPLETE | Runs on Pop!_OS 24.04 LTS and other distributions; distributions choose settings pages at build time and ship tagged... |

### LP — Research E — Pantheon (elementary OS 8.1)

| ID | Item | Status | Note |
|---|---|---|---|
| LP.01 | Shell components & compositor | PARTIAL | Pantheon shell with Dock, Multitasking View, picture-in-picture and Do Not Disturb; 8.1 improves Secure Session and D... |
| LP.02 | Launcher | COMPLETE | The HIG specifies launcher metadata as UI: Name without descriptive words, GenericName, a Comment beginning with a ve... |
| LP.03 | Panel / dock | PARTIAL | A Dock, improved in 8.1 (S3877); no dock configuration page opened; LitricOS: one bar plus morphing dock, no dock set... |
| LP.04 | System Settings inventory | PARTIAL | System Settings is where app permissions can be revoked (S3877); no module list opened; LitricOS: 13 pages, 9 stored... |
| LP.05 | File manager | COMPLETE | Files: smart pathbar with breadcrumbs, search and path completion, column view, browser-class tabs with tab history (... |
| LP.06 | Window behaviour & tiling | PARTIAL | Multitasking View and workspaces are advertised (S3877); no window-manager or tiling page opened; LitricOS: 36 WM ops... |
| LP.07 | Notifications | PARTIAL | Do Not Disturb stops notifications (S3877); the HIG Toast page exists with a heading and no body (S3876); LitricOS: q... |
| LP.08 | Workspace model | PARTIAL | Workspaces organise work by task and are reached from the Multitasking View (S3877); LitricOS: fixed grid, Mission Co... |
| LP.09 | Customization model | PARTIAL | No theme, accent or layout customization page opened; the HIG covers iconography and text as references (S3872); Litr... |
| LP.10 | Design language / HIG / tokens | COMPLETE | Full HIG: four design philosophies (What Design Is Not, Concision, Accessible Configuration, Minimal Documentation),... |
| LP.11 | Accessibility | PARTIAL | The HIG names Accessible Configuration as a philosophy and 8.1 enhances accessibility (S3872, S3873, S3877); no speci... |
| LP.12 | Limitations & known issues | NOT FOUND | No known-issues page and no release announcement could be opened: the OS 8 announcement returned 404 and no other pag... |
| LP.13 | Integration points (portals, D-Bus, APIs) | PARTIAL | AppCenter reviews permission use and System Settings can revoke permissions (S3877); no portal page opened; LitricOS:... |
| LP.14 | DE vs kernel vs distro boundary | PARTIAL | elementary ships its own OS with its own apps and AppCenter (S3877); the package base is not stated on any page opene... |

### LB — Research E — Budgie 10.10

| ID | Item | Status | Note |
|---|---|---|---|
| LB.01 | Shell components & compositor | COMPLETE | Wayland-first since 10.10 and compositor-agnostic: requires ext-workspace-v1, wlr-foreign-toplevel-management, wlr-la... |
| LB.02 | Launcher | COMPLETE | Budgie Menu: favourites via right-click, desktop actions in the menu, shared action code with Icon Tasklist, Favorite... |
| LB.03 | Panel / dock | COMPLETE | budgie-panel on layer-shell with start/centre/end sections, dynamic sizing and overflow; Icon Tasklist plus a rewritt... |
| LB.04 | System Settings inventory | PARTIAL | Budgie Control Center: Colour panel hidden, Bluetooth optional, per-panel visibility checks, Multitasking on Wayland,... |
| LB.05 | File manager | NOT FOUND | No file-manager component appears in either release note; desktop icons come from Budgie Desktop View, an icon layer... |
| LB.06 | Window behaviour & tiling | PARTIAL | The labwc bridge writes rc.xml for keybinds, focus, theming, snapping and a thumbnail window switcher, sets auto-rais... |
| LB.07 | Notifications | PARTIAL | Raven is the notification centre and Do Not Disturb toggles by middle-clicking the Notifications applet (S3880); per-... |
| LB.08 | Workspace model | PARTIAL | Workspaces applet reliability fixes and a more responsive add-workspace button; ext-workspace-v1 is a required protoc... |
| LB.09 | Customization model | COMPLETE | Bridge applies fonts, drop shadows and title-bar layouts; keybind templates per distribution; panel sections and appl... |
| LB.10 | Design language / HIG / tokens | PARTIAL | No Budgie HIG or token documentation found in the pages opened; design is GTK3 plus the bridge's labwc theming (S3879... |
| LB.11 | Accessibility | PARTIAL | Budgie Control Center supports accessibility options including a working magnifier (S3880); no screen-reader or keybo... |
| LB.12 | Limitations & known issues | COMPLETE | 10.10 is the final major of the 10.x series and is in maintenance mode while Budgie 11 is built; Keyboard Layout appl... |
| LB.13 | Integration points (portals, D-Bus, APIs) | COMPLETE | xdg-desktop-portal-gtk and -wlr, locale1/systemd-localed, WirePlumber wpctl, D-Bus org.buddiesofbudgie.KeyboardLayout... |
| LB.14 | DE vs kernel vs distro boundary | COMPLETE | Budgie sits on somebody else's compositor; kernel, logind, portals and audio outside; ships in Fedora 44 and Ubuntu B... |

### LR — Research E — ChromeOS 154+

| ID | Item | Status | Note |
|---|---|---|---|
| LR.01 | Shell components & compositor | PARTIAL | Chrome powers ChromeOS and the OS tracks Chrome releases; the shell and window manager are not described on any page... |
| LR.02 | Launcher | PARTIAL | Web apps are found from the launcher and a long press of the launcher starts Select-to-search with Lens (S3882, S3886... |
| LR.03 | Panel / dock | PARTIAL | The shelf exists: PWAs can be pinned to it (S3882); no shelf behaviour page opened; LitricOS: fixed top bar plus morp... |
| LR.04 | System Settings inventory | NOT FOUND | No ChromeOS Settings page could be opened: the Help Center and Enterprise URLs attempted returned 404 (recorded in Pa... |
| LR.05 | File manager | PARTIAL | The Files app can hand files to installed web apps (S3882); two Help Center Files URLs returned 404, so no Files beha... |
| LR.06 | Window behaviour & tiling | NOT FOUND | No page describing windows, split screen or overview was opened; nothing to cite; LitricOS: 36 WM ops with edge snapping |
| LR.07 | Notifications | NOT FOUND | No ChromeOS notification page was opened; nothing to cite; LitricOS: notifyd queue, banners, centre, DND |
| LR.08 | Workspace model | NOT FOUND | No virtual-desks or workspace page was opened; nothing to cite; LitricOS: fixed workspace grid and Mission Control |
| LR.09 | Customization model | NOT FOUND | No wallpaper, theme or shelf-customization page was opened; nothing to cite; LitricOS: 4 themes, 6 accents, 3 wallpapers |
| LR.10 | Design language / HIG / tokens | PARTIAL | chromeos.dev/en/design returned 404; the PWA page that opened lists capabilities, not a design language (S3882 plus r... |
| LR.11 | Accessibility | NOT FOUND | No ChromeOS accessibility page was opened; nothing to cite; LitricOS: 1 toggle, no a11y tree |
| LR.12 | Limitations & known issues | PARTIAL | End of Life / Auto Update Expiration is a documented process in the ChromiumOS reference index, and ChromeOS release... |
| LR.13 | Integration points (portals, D-Bus, APIs) | PARTIAL | Web apps install, pin to the shelf, appear in the launcher and handle files; Chrome extensions are integral (S3882);... |
| LR.14 | DE vs kernel vs distro boundary | PARTIAL | ChromeOS is an OS whose browser is the platform and whose releases track Chrome; security, SELinux, user sessions and... |

### F — Research F — Shared component catalog

| ID | Item | Status | Note |
|---|---|---|---|
| F.01 | Buttons | PARTIAL | Part 10 §F.01. Windows 26H2 and GNOME 51 guidance compared against ui.rs::button and ui::icon_button; macOS pages rea... |
| F.02 | Text fields | PARTIAL | Part 11 §F.02. Windows 26H2, GNOME 51 and macOS 27 (summary depth) guidance compared against ui::text_field and the s... |
| F.03 | Switches | PARTIAL | Part 10 §F.03. Windows 26H2 and GNOME 51 guidance compared against ui.rs::toggle; macOS pages read at summary depth o... |
| F.04 | Checkboxes | PARTIAL | Part 12 §F.04. Windows 26H2 and GNOME 51 read at page depth; macOS 27 unavailable beyond summaries. Found: the only m... |
| F.05 | Radio controls | PARTIAL | Part 12 §F.05. Windows 26H2 and GNOME 51 read at page depth; macOS 27 unavailable beyond summaries. Found: no radio c... |
| F.06 | Sliders | PARTIAL | Part 10 §F.06. Windows 26H2 and GNOME 51 guidance compared against ui.rs::slider; macOS pages read at summary depth o... |
| F.07 | Pickers (date/time/color/file) | PARTIAL | Part 17 §F.07. Windows 26H2 date/time and colour controls at page depth; the WinUI 3 file-picker page returned 404, s... |
| F.08 | Tabs | PARTIAL | Part 13 §F.08. Windows 26H2 and GNOME 51 read at page depth; macOS 27 summary-only. Found: fixed 150 px tabs with no... |
| F.09 | Navigation views | PARTIAL | Part 17 §F.09. Windows NavigationView read at page depth (five display modes, 52 px header, back button that does not... |
| F.10 | Sidebars | PARTIAL | Part 13 §F.10. Windows 26H2 and GNOME 51 read at page depth; macOS 27 summary-only. Found: two sidebars with two desi... |
| F.11 | Command bars/toolbars | PARTIAL | Part 13 §F.11. Windows 26H2 and GNOME 51 read at page depth; macOS 27 summary-only. Found: three toolbars with three... |
| F.12 | Menus | PARTIAL | Part 14 §F.12. Windows 26H2 and GNOME 51 read at page depth; macOS 27 summary-only. Found: the bar's three labels (Ap... |
| F.13 | Context menus | PARTIAL | Part 14 §F.13. Windows 26H2 and GNOME 51 read at page depth; macOS 27 summary-only. Found: six surfaces with 28 rows... |
| F.14 | Tooltips | PARTIAL | Part 16 §F.14. Windows 26H2 and GNOME 51 at page depth, macOS at summary depth, measured against 9 glyph-only control... |
| F.15 | Dialogs | PARTIAL | Part 15 §F.15. Windows 26H2 and GNOME 51 read at page depth; macOS 27 from a HIG mirror plus Apple's NSAlert referenc... |
| F.16 | Sheets | PARTIAL | macOS sheets: Apple HIG, 8 elements and 4 guidelines, Part 6 §MC.04.06. Windows and Linux sheets were not compared, s... |
| F.17 | Notifications/banners/toasts | PARTIAL | Part 16 §F.17. Windows 26H2 and GNOME 51 at page depth, macOS at summary depth, measured against the queue model, the... |
| F.18 | Progress indicators | PARTIAL | Part 11 §F.18. Windows 26H2, GNOME 51 and macOS 27 (summary depth) guidance compared against ui::progress and its one... |
| F.19 | Lists | PARTIAL | Part 10 §F.19. Windows 26H2 and GNOME 51 guidance compared against ui.rs::list_item; macOS pages read at summary dept... |
| F.20 | Trees | PARTIAL | Part 17 §F.20. Windows TreeView and GTK GtkTreeExpander read at page depth (chevrons, lazy children, plus/minus and C... |
| F.21 | Grids/tables | PARTIAL | Part 18 §F.21. Windows ListView/GridView read at page depth (a table-like collection should use a DataGrid rather tha... |
| F.22 | Cards | PARTIAL | Part 18 §F.22. GNOME boxed lists read in full (one or two controls per row, background click triggers the control, mi... |
| F.23 | Search fields | PARTIAL | Part 11 §F.23. Windows 26H2, GNOME 51 and macOS 27 (summary depth) guidance compared against the product's five searc... |
| F.24 | Empty states | PARTIAL | Part 18 §F.24. libadwaita AdwStatusPage read as a reference (icon, title, description and child action slots, compact... |
| F.25 | Loading states | PARTIAL | Part 18 §F.25. Windows progress controls read at page depth (determinate bar non-blocking, indeterminate bar non-bloc... |
| F.26 | Errors/validation | PARTIAL | Part 11 §F.26. Windows 26H2, GNOME 51 and macOS 27 (summary depth) guidance compared against fsbridge::valid_name and... |
| F.27 | Confirmation flows | PARTIAL | Part 15 §F.27. Same platform reading. Found: 11 confirmation call sites and 7 prompts; 5 of the 7 guard routine work... |

### G — Research G — Tokens & accessibility

| ID | Item | Status | Note |
|---|---|---|---|
| G.01 | Color tokens | PARTIAL | Part 20: 11 semantic palette tokens x 4 themes; components ask semantic names, never hex; border was the only token b... |
| G.02 | Typography | PARTIAL | Part 20: 8 type steps 12-38 px with line heights 16-44; the large-text threshold is 24 px, chosen conservatively. One... |
| G.03 | Spacing | PARTIAL | Part 20: 8 space steps 2-48 px on a 4 px grid. Component heights do not all join it: 24 px table rows, 30 px sidebar... |
| G.04 | Radii | PARTIAL | Part 20: 6 radii 6/10/14/20/28/Full with assigned roles; Full is a 9999 sentinel clamped at draw time and every corne... |
| G.05 | Shadows/elevation | PARTIAL | Part 20: 5 elevation levels, blur 0-44 px, offset 0-20 px, alpha 0-55 percent, monotonic by test. Most shell surfaces... |
| G.06 | Materials/blur | PARTIAL | Part 20: 7 materials and 3 glass presets (blur 32, saturation 115-120, tint 82/85/100 percent); the adaptive solve pi... |
| G.07 | Icons | PARTIAL | Part 20: 32 icon variants drawn from one vector module that inherits colour. Icon-only controls still have no label,... |
| G.08 | Motion timing/easing | PARTIAL | Part 20: 4 spring tokens with settle budgets 120/200/300/450 ms plus a reduced token; anim.rs assigns them per surfac... |
| G.09 | Light/dark | PARTIAL | Part 20: 4 themes (Dark, Light, HC Dark, HC Light); Light is separately authored, not an inversion, and the 6-slot ac... |
| G.10 | Contrast & high-contrast themes | PARTIAL | macOS evidence in Part 7; Linux measured in Part 19. 36 of 36 text pairs pass AA (lowest 4.97, Light success), 20 of... |
| G.11 | Display scaling/HiDPI | PARTIAL | Part 20: one scale factor in Tokens, set by Shell::set_display(width, height, scale); the layout audit runs every app... |
| G.12 | Multi-monitor | PARTIAL | Part 20: the compositor registers a single wl_output and the shell keeps one display size and scale; there is no arra... |
| G.13 | Localization & RTL | PARTIAL | Part 20: a 367-line translation dictionary including Thai, and a locale crate feeding hour cycle and first day. Zero... |
| G.14 | Keyboard-only navigation | PARTIAL | macOS evidence in Part 7; Linux measured in Part 19 as a route census. The focus ring is consistent and meets a AAA c... |
| G.15 | Screen readers & a11y APIs | PARTIAL | macOS evidence in Part 7; Linux measured in Part 19. Zero accessibility-API dependencies (no AccessKit, AT-SPI, UI Au... |
| G.16 | Pointer/touch/trackpad | PARTIAL | Part 20: pointer, wheel and keyboard only; no touch event type is handled. Hit targets include 18x18 checkboxes, 20x2... |
| G.17 | Reduced motion/transparency | PARTIAL | Part 20: reduce_motion is honoured end to end (preference, token, animation). There is no reduce-transparency setting... |
| G.18 | Sound & visual feedback | PARTIAL | Part 20: no audio output path in the shell; feedback is visual only (tone-coded badges, banners, inline errors, a cou... |

### H — Research H — Workflows

| ID | Item | Status | Note |
|---|---|---|---|
| H.01 | First boot & setup | PARTIAL | 3 preferences, no identity/network/time/keyboard |
| H.02 | Account creation | PARTIAL | no creation path; provisioning out of band |
| H.03 | Unlock | PARTIAL | 0 attempt limit; refuses to lock without verifier |
| H.04 | Launch/switch apps | PARTIAL | no failure or progress surface |
| H.05 | Window management | PARTIAL | 36 ops, cancel_drag, no undo for close |
| H.06 | Display change | PARTIAL | read-only page, no revert timer |
| H.07 | Audio change | PARTIAL | 4 verbs, 3 controls, no device picker |
| H.08 | Network change | PARTIAL | kernel facts only, no connect |
| H.09 | Device pairing | PARTIAL | 5 verbs, no bluetoothd in image |
| H.10 | Install/uninstall software | PARTIAL | trust chain designed, no installer |
| H.11 | Default apps | PARTIAL | no association model |
| H.12 | File management | PARTIAL | 26 verbs, trash, no cancel |
| H.13 | Permissions grant/revoke | PARTIAL | broker complete, no prompts |
| H.14 | Updates | PARTIAL | absent, no rollback |
| H.15 | Recovery | PARTIAL | state exists, never drawn |
| H.16 | Backups | PARTIAL | absent |
| H.17 | Troubleshooting | PARTIAL | diagnostics yes, guidance no |
| H.18 | Accessibility change | PARTIAL | one control, honoured |
| H.19 | Shutdown | PARTIAL | prompt + consequence, no unsaved guard |
| H.20 | Failure/cancel/retry/rollback | PARTIAL | 9 entries, 2 cancels, 0 revert timers |

### I — Research I — Technical integration & LitricOS feasibility

| ID | Item | Status | Note |
|---|---|---|---|
| I.01 | Public UI frameworks per OS | PARTIAL | no public UI toolkit; apps are enum arms, foreign apps draw their own pixels |
| I.02 | Shell extension points | PARTIAL | manifest + capabilities only; unknown capability is a hard error, no extension registry |
| I.03 | Settings persistence (registry, defaults/plist, GSettings/dconf, KConfig) | PARTIAL | 9 keys in a private key=value file (ui_scale added 2026-10-28), atomic write 0600; no namespace for other apps, no ch... |
| I.04 | Permission models (UAC/TCC/polkit/portals) | PARTIAL | capd authority, deny-by-default, audit; no consent prompt, no picker, scope coarser than a portal |
| I.05 | Notification APIs | PARTIAL | litric-notifyd private queue, text in/out, no DND in the daemon; no D-Bus, no replace-by-id, no actions |
| I.06 | IPC/events | PARTIAL | LFP/1 + browser socket + notifyd; at-most-once mutations documented; no bus, no discovery, no event stream |
| I.07 | Compositor protocols (Wayland) | PARTIAL | wl_compositor v4, wl_seat v5, xdg_wm_base v2; pointer+keyboard only; wl_output imported but not advertised |
| I.08 | Undocumented vs public boundary | PARTIAL | ADRs argue the boundary; LFP/1 has no specification outside its own module |
| I.09 | Comparison matrix | COMPLETE | comparison matrix across 10 surfaces in Part 23 |
| I.10 | ADOPT/ADAPT/AVOID/INVESTIGATE decisions | COMPLETE | 12 ADOPT/ADAPT/AVOID/INVESTIGATE decisions; revert window implemented in the shell |
| I.11 | Validation tests per decision | PARTIAL | 5 of 12 decisions validated by tests; 4 unimplemented, 3 undecided |

### J — Research J — Source register & audit

| ID | Item | Status | Note |
|---|---|---|---|
| J.01 | Register schema | COMPLETE | 17 columns; 39 evidence labels, 23 without a tier suffix |
| J.02 | URL/domain dedupe | COMPLETE | 47 duplicate URLs, 52 surplus rows, no duplicate_of column |
| J.03 | Domain count toward 400 target | COMPLETE | 382 of 400 domains (95.5%) |
| J.04 | Inspected vs snippet-only counts | COMPLETE | 1,300 inspected, 2,899 snippet-only, 6 inaccessible |
| J.05 | Primary vs independent counts | COMPLETE | 3,079 primary / 1,091 independent; independent used only as corroboration |
| J.06 | Inaccessible log | COMPLETE | 6 inaccessible rows, none cited for a claim; no retry reason stored |
| J.07 | Contradictions log | COMPLETE | 5 contradictions logged, 2 resolved in LitricOS' favour |
| J.08 | Per-subsystem coverage audit | COMPLETE | 272 subsystem labels (31 singletons); 0 Linux rows PENDING, 120 measured |

### Checklist status totals (Part 1)

| Status | Items |
|---|---|
| COMPLETE | 57 |
| PARTIAL | 528 |
| NOT FOUND | 9 |
| NOT APPLICABLE | 3 |
| PENDING | 0 |
| Total items (excluding block headers) | 597 |

---

## Resume Point

Part 2 starts with Batch B1, item WB.01.01 (Windows 11 26H2 Settings › System › Display).