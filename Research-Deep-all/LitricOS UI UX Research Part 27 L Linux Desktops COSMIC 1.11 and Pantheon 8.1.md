# LitricOS UI/UX Research — Part 27
## Chapter L: Linux Desktop Environments (batch 3 — COSMIC Epoch 1 and Pantheon 8.1)

**Part:** 27
**Chapter:** L (Research E, Linux desktop environments)
**Rows covered:** `LO.01`–`LO.14` (COSMIC Epoch 1), `LP.01`–`LP.14` (Pantheon, elementary OS 8.1.1) — **28 new rows**, 84 of the 112 the checklist holds for Chapter L
**Date:** 2026-10-28
**Evidence base:** eleven pages opened in this pass (`S3867`–`S3877`), plus the corpus measurements of Parts 19–26 for the LitricOS column.

---

## L.0 What this batch is for

These are the two environments in the corpus that were built *as products*:
COSMIC by a hardware vendor that wanted its own desktop, Pantheon by a project
that wanted its own operating system. Both publish design rules — one as a
toolkit book, one as a full HIG — and both are candid about what is unfinished.
The batch is therefore the first in which "what the project says about itself"
is a row of evidence rather than a footnote.

### L.0.1 Evidence grade of this batch

| Source | ID | Grade | What it establishes |
|---|---|---|---|
| COSMIC DE product page | S3867 | T1 (project) | Wayland-native Rust desktop; shell components and compositor share the toolkit; panels, applets, tiling, launcher, workspaces |
| COSMIC UX page | S3868 | T1 (project) | Panel layouts, applet positions and sizing, distro settings pages via cargo features, applet distribution through the store |
| libcosmic book, introduction | S3869 | T1 (project) | The toolkit, its design language, theming, configuration system, cross-platform scope |
| System76 blog, COSMIC 1.0.8 | S3870 | T1 (project) | The Epoch 1 point-release log (1.0.3–1.0.8): Files, Comp, Settings, Portal, Notifications, Session, Store |
| elementary HIG, introduction | S3871 | T1 (project) | Scope of the HIG and its explicit deferral of unwritten sections to the GNOME HIG |
| elementary HIG, index (`llms.txt`) | S3872 | T1 (project) | The complete page list: philosophies, workflow, app launcher, widgets, reference |
| elementary HIG, design philosophy | S3873 | T1 (project) | Four guiding philosophies, named but not enumerated on this page |
| elementary HIG, "Always Provide an Undo" | S3874 | T1 (project) | The undo rule, with buffer time and the condition that the action still happens |
| elementary HIG, "App Launcher" | S3875 | T1 (project) | Launcher metadata rules: Name, GenericName, Comment, Categories, Keywords |
| elementary HIG, "Toast" | S3876 | T1 (project) | A page with a heading and **no body** |
| elementary.io | S3877 | T1 (project) | OS 8.1.1, Secure Session and Dock improvements, accessibility, permissions, Files and Terminal tab models |

**Pages that failed to open, recorded verbatim:**

| URL | Failure |
|---|---|
| `https://9to5linux.com/cosmic-1-11-desktop-environment-released-with-new-touchpad-settings` | **403 Forbidden** |
| `https://system76.com/blog/post/cosmic-1-11-released` | **404 Not Found** |
| `https://blog.elementary.io/elementary-os-8-available-now/` | **404 Not Found** |

Consequence, stated plainly: **no row in this part rests on Epoch 1.11-specific
change text.** The checklist's version baseline says 1.11 is current; the
sources that describe it could not be read, so the COSMIC column describes Epoch
1 through the point releases that *were* readable (1.0.3–1.0.8) and says when a
1.11 claim would be needed.

---

## L.1 COSMIC Epoch 1

COSMIC is the only environment in this corpus whose shell, compositor and
applications are one toolkit: "Even shell components and the compositor use the
same toolkit" (S3867). The same page calls the panel, applets, theming, tiling,
launcher, app library, keyboard shortcuts and workspaces "flexible to bend to
your users needs" — which is the product argument of a vendor that ships to
distributions it does not control.

### L.1.1 The rows

| Row | What COSMIC Epoch 1 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LO.01` Shell components & compositor | Wayland-native, written in Rust; shell components and the compositor share the toolkit; cosmic-comp on smithay with damage-tracking and `wl_fixes` work; greeter, session, idle and settings are separate projects | S3867, S3870 | One Rust shell plus its own compositor; no greeter, session manager or idle daemon | **COMPLETE** |
| `LO.02` Launcher | A launcher and an app library as separate surfaces; the workspaces overview can be configured to open either on typing | S3867, S3870 | One launcher overlay with `seek` ranking; no separate app library | PARTIAL |
| `LO.03` Panel / dock | Single or dual panels on any edge; applets placed at beginning, centre or end; size, opacity, square or rounded corners; no extension API — applets are apps packaged as Flatpak for the store | S3868 | One fixed top bar plus a morphing dock; no applet API, no store | **COMPLETE** |
| `LO.04` System Settings inventory | Settings pages are extensible per distribution through cargo feature flags (Pop!_OS adds "Upgrades and Restore"); pages touched by the point releases include appearance/themes, network, language, default apps, startup apps, shortcuts, workspaces, touchpad | S3868, S3870 | 13 pages, 9 stored keys, 8 registry rows | PARTIAL |
| `LO.05` File manager | Files: tabs, opening several directories in separate tabs, tab drag-and-drop, Move to / Copy to, pasting images, video and text from the clipboard, Recents toggle with clear history, copy-path on Shift, ellipsized names, preview open button | S3870 | `apps::FilesApp`: 26 verbs, no tabs, no named Move to / Copy to; clipboard cut/copy/paste is wired through one shell path (`src/litric-shell/src/shell.rs` line 2413) | **COMPLETE** |
| `LO.06` Window behaviour & tiling | Tiling is a first-class flexible element; point releases fix tiled-window state on maximize/snap and Super+Click on a tiled window, and make the cursor follow focus across monitors | S3867, S3870 | `litric-wm`: 36 operations, edge snapping, one output | PARTIAL |
| `LO.07` Notifications | cosmic-notifications accepts `file://` icons; the status-area applet runs a socket-activated StatusNotifierWatcher, so tray icons survive client crashes | S3870 | `litric-notifyd` queue, banners, centre, DND; no tray or status area | PARTIAL |
| `LO.08` Workspace model | Workspaces can be dynamic or pinned; the overview's action on typing is a setting | S3867, S3870 | Fixed grid; Mission Control cards; no overview typing action | **COMPLETE** |
| `LO.09` Customization model | A theming system built for brand palettes without losing contrast, RON theme files applicable from the CLI, Qt theming through qt5ct/qt6ct, panel and shortcut defaults shipped by the distribution | S3867, S3868, S3870 | 4 themes × 6 accents × 3 wallpapers; no theme file format, no per-distro defaults | **COMPLETE** |
| `LO.10` Design language / HIG / tokens | libcosmic is "based on COSMIC's design language", with consistent interface guidelines, a standardized configuration system and cross-desktop theming; no public HIG with token values was opened | S3869 | 11 semantic colours × 4 themes, 8 type and 8 space steps, 4 motion tokens, enforced by a lint | PARTIAL |
| `LO.11` Accessibility | No accessibility page, screen-reader statement or on-screen keyboard was found in the pages opened; the one accessibility-adjacent change visible in search results is a touchpad setting in 1.11, which could not be opened (403) | — (gap) | One toggle (reduce motion) honoured everywhere; no a11y tree | PARTIAL |
| `LO.12` Limitations & known issues | The point-release log is a public list of fixed failure classes: a tiling-placeholder panic in cosmic-comp, an image-copy memory leak in smithay, a portal backend abort when dma-buf is missing, password handling in the terminal, duplicate wallpaper entries | S3870 | No public failure log; failures are single-sentence notifications | PARTIAL |
| `LO.13` Integration points | `xdg-desktop-portal-cosmic` (screencast, clipboard, background, screenshot), StatusNotifierWatcher, AccountsService language handling, Qt theme generation, Flatpak applets, distro feature flags | S3870, S3868 | LFP/1, browser socket, notifyd; no portal, no bus, no status area | **COMPLETE** |
| `LO.14` DE vs kernel vs distro boundary | COSMIC runs on Pop!_OS 24.04 LTS and on other distributions; the distribution chooses settings pages at build time and ships tagged releases or rolling QA updates; the kernel, logind and audio stack stay outside | S3867, S3868, S3870 | One mkosi image; the boundary is a build decision | **COMPLETE** |

### L.1.2 Findings — COSMIC

**L27.01 — COSMIC is the only other system in the corpus that can say its shell
and its compositor are the same toolkit.** LitricOS can say the same thing, and
should: it is the strongest architectural claim the project has, and until now it
has been a fact about the tree rather than a statement in the design record.

**L27.02 — COSMIC has no extension API, and calls that a feature.** Applets are
applications, packaged as Flatpak and listed in the COSMIC Store (S3868). GNOME
solves the same problem with extensions and a shell that can be broken by them.
LitricOS solves it with a manifest and a capability list — but has neither a
store nor a way to ship an applet.

**L27.03 — Settings pages are a build flag.** `cargo build --features pop-os`
selects which settings pages exist (S3868). LitricOS' 13 pages are also compiled
in, but there is no per-image tailoring: every LitricOS image has the same
Settings, including pages for hardware it may not have.

**L27.04 — Files shipped tabs, tab drag-and-drop and clipboard paste before
LitricOS shipped a cancel verb.** The 1.0.3–1.0.8 log adds tabs, then tab drag
and drop, then Move to / Copy to, then paste of images, video and text (S3870).
Part 26 already named tabs and split as the gap; this is the same gap measured on
a second codebase, four point releases deep.

**L27.05 — The point-release log is a public record of what broke.** A tiling
placeholder panic, an image-copy leak, a portal abort. LitricOS has no equivalent
document, and Part 24 found the same absence from the other direction: the
research corpus has a failure log, the product does not.

**L27.06 — Accessibility is the row nobody documents.** Eleven pages opened, and
not one of them describes COSMIC's screen reader, keyboard navigation or
contrast behaviour. That is not evidence that COSMIC lacks accessibility; it is
evidence that a desktop can ship for a year without publishing the rules for it.
LitricOS is in the same position from the other side: one toggle, no tree.

---

## L.2 Pantheon (elementary OS 8.1.1 "Circe")

elementary is the only project in this corpus with a HIG that starts by saying
what it is not: "this is a set of guidelines, not a rule book", and "for sections
that have not yet been written, please refer to The GNOME HIG" (S3871). That
sentence does more for the credibility of the document than a hundred rules
would.

### L.2.1 The rows

| Row | What Pantheon 8.1 does | Evidence | LitricOS today | Verdict |
|---|---|---|---|---|
| `LP.01` Shell components & compositor | Pantheon shell with a Dock, Multitasking View, picture-in-picture and Do Not Disturb; 8.1 improves the Secure Session and the Dock. The compositor is not described on any page opened | S3877 (structural) | One Rust shell; own compositor; no secure session | PARTIAL |
| `LP.02` Launcher | The HIG specifies launcher metadata as UI: Name without descriptive words, GenericName, a Comment beginning with a verb, Categories, Keywords — the same fields the applications menu and Dock display | S3875 | Launcher searches app names; no generic names, comments, categories or keywords | **COMPLETE** |
| `LP.03` Panel / dock | A Dock, improved in 8.1; no dock configuration page was opened | S3877 | One top bar plus a morphing dock; no dock settings | PARTIAL |
| `LP.04` Settings inventory | System Settings is where app permissions can be revoked; no module list was opened | S3877 | 13 pages, 9 stored keys | PARTIAL |
| `LP.05` File manager | Files: smart pathbar with breadcrumbs, search and path completion; column view; "browser-class tabs" with tab history | S3877 | `FilesApp`: breadcrumb, table, 26 verbs, no tabs | **COMPLETE** |
| `LP.06` Window behaviour & tiling | Multitasking View and workspaces are advertised; no window-manager or tiling page was opened | S3877 (structural) | 36 WM operations, edge snapping | PARTIAL |
| `LP.07` Notifications | Do Not Disturb "stops notifications in their tracks"; the HIG's Toast page exists but has **no body** | S3877, S3876 | Queue with DND; banners; centre; no toast component | PARTIAL |
| `LP.08` Workspace model | Workspaces "help organize your work by task", reachable from the Multitasking View | S3877 | Fixed grid; Mission Control cards | PARTIAL |
| `LP.09` Customization model | No theme, accent or layout customization page was opened; the HIG covers iconography and text as references | S3872 (index) | 4 themes, 6 accents, 3 wallpapers; no add-on surface | PARTIAL |
| `LP.10` Design language / HIG / tokens | A full HIG: four design philosophies (What Design Is Not, Concision, Accessible Configuration, Minimal Documentation), user workflow, app launcher, widgets, and reference sections for iconography and text; unwritten sections explicitly defer to the GNOME HIG | S3871, S3872, S3873 | Tokens plus a lint; no principles document, no pattern catalogue | **COMPLETE** |
| `LP.11` Accessibility | The HIG names "Accessible Configuration" as a philosophy; 8.1 "enhances accessibility"; no specific accessibility feature, shortcut or contrast rule was described on a page opened | S3872, S3873, S3877 | One toggle, no a11y tree | PARTIAL |
| `LP.12` Limitations & known issues | No known-issues page and no release announcement could be opened: the OS 8 announcement URL returned **404**, and no other page lists defects | — (gap) | No release-notes surface | **NOT FOUND** |
| `LP.13` Integration points | AppCenter reviews apps for permission use and System Settings can revoke permissions; the store is the distribution channel; no portal page was opened | S3877 | Private IPC; no store, no permission UI | PARTIAL |
| `LP.14` DE vs kernel vs distro boundary | elementary ships its own OS with its own apps and AppCenter; the package base is not stated on any page opened | S3877 (structural) | One mkosi image | PARTIAL |

### L.2.2 Findings — Pantheon

**L27.07 — elementary states the undo rule more strongly than anyone else in the
corpus.** "Rather than present the user with a warning, apps should let the user
undo the action for an appropriate amount of time", with a buffer between showing
the result and performing it, and the action still performed if the window closes
(S3874). That is a design for the exact case LitricOS handles with eleven
confirmations and zero undos.

**L27.08 — The HIG's Toast page is empty, while another HIG page tells apps to
use a toast.** S3876 contains a heading and nothing else. A documented pattern
whose reference page has not been written is a real state of documentation, and
it is honest: the index lists it, the body does not pretend. LitricOS has the
inverse problem — a banner component with no written rule at all.

**L27.09 — elementary publishes its own incompleteness.** "For sections that have
not yet been written, please refer to The GNOME HIG" (S3871). No other project
in this corpus says where its documentation stops. LitricOS says nothing about
its design record anywhere in the product.

**L27.10 — Launcher metadata *is* the launcher.** Name, GenericName, Comment,
Categories and Keywords are specified as interface, because the menu and the Dock
display them (S3875). LitricOS' launcher ranks names; it has no comment, no
category and no keyword, so "browser" cannot find Web and "control panel" cannot
find Settings — the registry solved this for settings rows only.

**L27.11 — Tab history is a stronger model than tabs.** Files advertises
"browser-class tabs with smart features like tab history" (S3877). LitricOS has
no tabs; when it gets them, back/forward per tab is the part that makes them
useful rather than decorative.

**L27.12 — Two of the three failures in this batch are release notes.** The
COSMIC 1.11 post and the elementary OS 8 announcement both 404'd. A research
corpus that only reads what is currently online inherits the vendor's publishing
decisions, and this part says so rather than filling the gaps from memory.

---

## L.3 Cross-batch comparison

| Question | COSMIC Epoch 1 | Pantheon 8.1 | LitricOS |
|---|---|---|---|
| Toolkit | libcosmic for shell, apps and applets | GTK/Vala stack (not described on opened pages) | One Rust crate set for shell and apps |
| Extension model | Applets as Flatpak apps in a store | AppCenter apps, no shell extension API documented | Manifest + capabilities, no store |
| Panel | Single or dual, any edge, applets, opacity, corners | Dock, improved in 8.1 | One bar, morphing dock |
| File manager | Tabs, tab drag-and-drop, Move to/Copy to, clipboard paste | Tabs with tab history, column view, smart pathbar | Flat table, 26 verbs |
| Settings | Pages selected per distro by build flag | System Settings, permissions revocable | 13 fixed pages, 9 keys |
| Undo | Not specified in opened pages | HIG-mandated, with a buffer window | None |
| Design docs | Toolkit book, no token-level HIG opened | Full HIG with named philosophies and declared gaps | Tokens + lint, no principles |
| Failure log | Public point-release log | Not published / not reachable | None |

The pattern across the three batches is now clear enough to state as a Chapter L
result: **every environment in the corpus publishes more about its behaviour than
LitricOS does, and none of them publishes as much as LitricOS could** — because
LitricOS has the tokens, the lint and the measured layouts, and has never written
the principles or the failure record.

---

## L.4 What to change in LitricOS because of this part

| # | Change | Why (row) | Size |
|---|---|---|---|
| 1 | Give Files tabs with per-tab history, then Move to / Copy to | `LO.05`, `LP.05`, L27.11 | L |
| 2 | Add a launcher metadata layer: generic name, comment, category, keywords | `LP.02`, L27.10 | M |
| 3 | Replace the routine-work confirmations with an undo buffer and a toast/banner | `LP.07`, L27.07 | L |
| 4 | Write the two missing design documents: principles and a pattern catalogue | `LO.10`, `LP.10` | M |
| 5 | Keep a public failure log per release, in the image and in the repo | `LO.12`, L27.05 | S |
| 6 | Make the Settings page set selectable per image, as COSMIC does per distro | `LO.04`, L27.03 | M |
| 7 | Add a status area / tray path for StatusNotifier items | `LO.07` | M |
| 8 | State the architecture in the design record: shell and compositor share one toolkit | `LO.01`, L27.01 | S |

---

## L.5 Limitations of this batch

* **Two environments of eight remain.** Budgie and ChromeOS are still PENDING;
  Chapter L is 84 of 112 rows.
* **Epoch 1.11 could not be read.** Its blog post 404'd and the independent
  article 403'd, so no row rests on 1.11-specific text and `LO.02`, `LO.06` and
  `LO.11` are marked accordingly.
* **No COSMIC accessibility surface was found.** Eleven pages, none of them about
  screen readers, keyboard navigation or contrast.
* **Pantheon's compositor, settings module list and package base were not on any
  page opened.** `LP.01`, `LP.04` and `LP.14` are structural, and `LP.12` is
  NOT FOUND because no known-issues page was reachable.
* **Nothing was run.** No COSMIC or Pantheon session was started; the DE column
  is documentation, the LitricOS column is code in this repository.

---

## L.6 Sources read in this part

| ID | URL | What it gave |
|---|---|---|
| S3867 | https://system76.com/cosmic | COSMIC product claims: Rust, Wayland, shared toolkit, flexible panels/applets/tiling/workspaces |
| S3868 | https://system76.com/cosmic/ux | Panel layouts and applet options, distro settings pages by cargo feature, Flatpak applets |
| S3869 | https://pop-os.github.io/libcosmic-book/introduction.html | libcosmic scope, design language, configuration system, cross-platform targets |
| S3870 | https://system76.com/blog/post/cosmic-1-0-8-released | Epoch 1 point-release log 1.0.3–1.0.8 across Files, Comp, Settings, Portal, Notifications |
| S3871 | https://docs.elementary.io/hig/ | HIG introduction and its deferral to the GNOME HIG |
| S3872 | https://docs.elementary.io/hig/llms.txt | Complete HIG page index |
| S3873 | https://docs.elementary.io/hig/design-philosophy.md | The philosophies section |
| S3874 | https://docs.elementary.io/hig/user-workflow/always-provide-an-undo.md | The undo rule, buffer time and the still-performed condition |
| S3875 | https://docs.elementary.io/hig/app-launcher.md | Launcher metadata rules |
| S3876 | https://docs.elementary.io/hig/widgets/ui-toolkit-elements/toast.md | A heading with no body |
| S3877 | https://elementary.io/ | OS 8.1.1, Dock and Secure Session, DND, Files tabs with history, permissions |

Failed, recorded and not cited: 9to5linux COSMIC 1.11 article (**403**),
`system76.com/blog/post/cosmic-1-11-released` (**404**),
`blog.elementary.io/elementary-os-8-available-now/` (**404**).
