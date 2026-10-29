# 06 — APP REDESIGN (in-app surfaces, to the standard of Windows and macOS)

> Status: PLAN · 2026-10-09 · extends [UI-PLAN](../UI-PLAN.md) (P0-P9 chrome grammar) · blocks on [BUG-0015](../bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md)

`UI-PLAN` defines the grammar every surface shares: menubar, window chrome, dock,
Spotlight, context menus, motion, notifications. It does not define what is
*inside* each window. Today the applications inside that chrome are the weakest
part of the system -- a Files window that lists names, an Activity window with two
big numbers, a Settings window that is still a TODO. This document is the plan for
the inside: one section per application, each with the reference it is measured
against, the states it must have, and the evidence that closes it.

## 0. The gate that comes first

Frame cost is the precondition, not a parallel track. A frame with one window
measures **206 ms** against a **16.67 ms** budget ([BUG-0015](../bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md));
at that cost, no amount of information architecture in an app is reachable,
because opening it takes a fifth of a second and scrolling it does not track the
pointer. **Activity is redesigned first, and its first job is to make this number
visible on the machine** -- the surface that reports system health has to be the
surface that shows the defect that matters.

## 1. Method: every app closes with an artefact, not a screenshot

Each application follows the same loop, using tooling that runs on any host
(`frame_dump` + `tools/frames-page.py`, both added 2026-10-09):

1. **Frame.** A scene per state in `src/litric-shell/examples/frame_dump.rs`,
   dumped to PPM, converted, and reviewed as one page with the frame's
   fingerprint and draw time.
2. **Journey.** An end-to-end input journey in the shell's e2e harness (real key
   presses and clicks, as the existing 24 journeys do).
3. **States.** Empty, loading, error, denied (capability refused), partial (a fact
   the daemon could not read), and overflow audit 0 -- in both themes.
4. **Numbers.** `pixels_visited` per frame and the frame time, so a regression in
   cost is caught by a test rather than by a person noticing lag.

An application is not "done" until all four exist. This is deliberately stricter
than "screenshots look right": the fingerprints make a silent change visible, and
the visited-pixel count makes a silent slowdown visible.

## 2. Activity -> System Monitor (the user named this one)

> **Shipped 2026-10-09.** The monitor now draws from the facts the machine was
> already reporting and the shell was throwing away: CPU model from
> `/proc/cpuinfo`, live utilisation from two samples of the per-core counters,
> memory available/cache/swap, load and task count, the daemon's own footprint,
> MGLRU and verified-boot state, and the mount table with a plain-language
> explanation on every read-only volume (the EROFS sentence). New code:
> `src/litric-shell/src/monitor.rs` (drawing plus the testable decisions),
> the mount and per-CPU parsers in `src/litric-sysd/src/facts.rs`, and the
> `observe()` step that turns cumulative counters into a live rate.
> The processes table below is **not** implemented: it needs per-process data
> from the daemon that no layer collects yet.

Today: two big meters and an availability note. Reference: the Windows 11 Task
Manager, which is the strongest practical model -- a processes table people
actually diagnose with, plus a Performance tab that explains the machine.

**Processes.** Sortable table, grouped by application, with per-row: icon, name,
CPU % (and total CPU time), memory (working set, plus share of total), disk B/s,
network B/s, energy trend (up/down arrow over a 10 s window), PID, status, user,
thread count. Right-click a row: End task, Show file location, Go to details,
Copy. Values arrive from `litric-sysd`; a fact the daemon does not report renders
as **unavailable**, never as zero (the existing `the_activity_window_shows_unavailable_rather_than_zero`
test is the rule, extended to every new field).

**Performance.** One pane per resource, each with a live graph and a breakdown:
- **CPU** -- model name, physical cores / logical threads, live clock and base
  clock, per-core utilisation grid (Windows' small-multiples view), load average,
  top-5 consumers, uptime, context switches. The model string comes from the
  daemon, not from a hard-coded table.
- **Memory** -- total, used, available, cached, committed, swap, with a stacked
  composition bar and a "memory pressure" state, plus the top consumers.
- **Disk** -- per volume: label, filesystem, mount, capacity, free, read/write
  rates, IOPS, queue depth.
- **Network** -- per interface: link state, addresses, throughput graph, totals.
- **GPU / power** when the daemon reports them; otherwise the pane is absent, not
  empty.

**Every figure has a "what is this" affordance.** This is the answer to "EROFS is
what?": the filesystem row in the Disk pane carries a one-line plain explanation
the user can open, and here is the text it renders for the OS volume:

> **EROFS (Enhanced Read-Only File System).** The operating system image is a
> read-only filesystem, so it cannot be modified while running -- not by you, not
> by a program, not by root, not by a bad update. It is compressed, which is why
> the image is small, and it is paired with dm-verity, which checks every read
> against a hash tree, so a corrupted or tampered block is refused instead of
> executed. This is why the volume shows 0% free and warns on no writes: nothing
> on it will ever need the space, and nothing on it can be changed.

Each figure in the table gets the same treatment: the number, the unit, and one
sentence about what it means and what a bad value looks like. Reference:
[EROFS in the kernel documentation](https://docs.kernel.org/filesystems/erofs.html).

**Fix list that this pane exists to expose** (each becomes a task with a number):
frame time and pixels painted per frame, blur cost, wakeups per second at idle,
memory of the shell itself, and the over-budget frame warning count.

## 3. Settings -> the quality bar of the system

> **Partially shipped 2026-10-09.** The Settings window already had a sidebar of
> eight categories and one real page (Region and Language); the rest of the pane
> said "not implemented". It now also renders a **System** page (version, root
> filesystem, processor model, memory, uptime, load, verified boot, MGLRU, the
> daemon's footprint, and which of those the daemon or a direct read measured)
> and a **Storage** page (each real volume, its filesystem, and the EROFS
> explanation on the read-only one). Both are read-only by construction: every
> row is a measurement from `SystemFacts`, so there is no control that could
> disagree with the machine. The remaining categories are still named at the
> bottom of the pane instead of being shown as switches that do nothing.

Reference: macOS System Settings' sidebar+detail, Windows' searchable
settings, and one thing neither does well -- every control states its effect.

**Structure** (sidebar with search, detail pane on the right):
Appearance (theme, accent, wallpaper, glass intensity, density) · Accessibility
(scale, contrast, reduced motion, larger text, screen reader state) · Language &
Input (locale, keyboard layout, formats, RTL preview) · Displays (resolution,
scale, refresh) · Sound (output, volume, per-app) · Network (interfaces, Wi-Fi,
proxy) · Storage (volumes, EROFS explanation, what is writable and where) · Power
(battery, thermal, suspend) · Notifications (per-app, do-not-disturb) ·
**Privacy & Permissions** (the capability broker: which app holds which capability,
revoke, audit log) · Windows Environments (the environment switch: install state,
what it runs, resource share) · Applications (default handlers, autostart,
uninstall) · Updates & About (version, image hash, verified boot state).

**Rules that make it not "กาก":**
1. **No dead control.** Every toggle either applies immediately with a visible
   effect or is labelled "applies after restart", and shows the restart affordance.
2. **Search finds it.** Typing matches setting names *and* their help text, and
   lands on the control, not on a page.
3. **Revert.** Per-section "reset to defaults" and a visible state for "changed
   from default".
4. **The audit shows the truth.** A setting the daemon cannot honour says so, with
   the reason, rather than silently keeping its old value.
5. **Themes are tokens, not code.** Light and dark both pass the overflow audit
   and the contrast tests with no per-theme drawing branches.

## 4. The other applications, in priority order

| App | Reference | Redesign (the parts that matter) | Closes with |
|---|---|---|---|
| **Files** | Finder / Explorer | Sidebar (Favorites, Locations, Trash), back/forward, icon+list+column views, sort, breadcrumb + item count, rename F2, Del->Trash with undo, properties panel (size on disk, EROFS vs writable volume, permissions), real trash window with restore | Journey create->rename->trash->restore; grid and empty frames |
| **Defender** | Windows Security | Status header (protection state, last scan, definitions age), scan pane with live progress and findings, quarantine list with restore/delete, exclusions, per-tier explanation (Hash/Pattern/Heuristic) -- honest about what each tier can and cannot catch | Detection journey + quarantine frame |
| **Browser** | Edge/Chrome skeleton | Tab strip with per-tab state, address bar with security state, back/forward/reload, history and downloads, per-tab capability banner when a site is denied something | Journey load->navigate->deny->banner |
| **Terminal** | Windows Terminal | Real command execution over the file service, scrollback, tab title, exit-code display, clear, Ctrl+C cancelling a running command, monospace grid with the shipped font | Journey run command->see output->exit code |
| **Trash** | Recycle Bin | Reads the real freedesktop trash: items with original path and deletion time, restore, empty, per-item delete | Journey trash->list->restore->empty |
| **Environments** | -- | Install state per environment, what it runs, resources it takes, and the honest "not installed" path | Frames for all three states |
| **Activity** | Task Manager | Section 2 above | All panes framed; BUG-0015 visible in the UI |

## 5. Sequencing

1. **BUG-0015**: bound drawing and measure it. Nothing else can be judged until
   a frame is under budget.
2. **Activity** (with the EROFS/Storage explanation): it is the diagnostic surface
   and the one the user named.
3. **Settings**: it is where quality is visible, and it unblocks theme and
   accessibility work that every other app depends on.
4. **Files -> Trash -> Terminal -> Defender -> Browser**: in that order, because
   each uses the previous one's patterns.
5. **Motion and notification polish** last, per `UI-PLAN` P7/P8, with the frame
   budget now a real gate instead of an assumption.

## 6. Definition of done (per app)

- [ ] Every state in section 1 has a frame with a fingerprint in the review page.
- [ ] The e2e journey passes and asserts the behaviour, not the pixels.
- [ ] Overflow audit 0 at 1280x800 and at the smallest supported scale, both themes.
- [ ] Frame time and `pixels_visited` recorded for the app's heaviest frame.
- [ ] No dead control, no invented number: every figure comes from the daemon or
      reads "unavailable".

## Non-goals

- GPU blur/refraction, and any visual world that needs one (renderer constraint,
  `UI-PLAN` non-goals).
- Window snapping grids, multiple Spaces, third-party app store.
- Replacing the token system: this plan consumes `litric-design`, it does not add
  a second source of truth for colour, type or spacing.

## Shipped copy rule (2026-10-29)

The direction above is kept, with one change decided against the running shell:
the *concept* is explained, the *token* is not printed. Storage rows read
"Read-only" or "Read and write", and the sentence beside a protected volume says
the system storage cannot change while Litric runs and that every block is
checked as it is read; the filesystem's own name no longer appears in any frame.
Network rows read "Connected", "Not connected" or "Connecting" -- interface
names, hardware addresses and `/dev` paths stay out of the default rows.

The vocabulary lives in `src/litric-shell/src/copy.rs`; `copy::BANNED_TOKENS` is
the single forbidden list, and the journey
`journey_no_machine_token_reaches_the_system_surfaces` renders the real Tasks,
Network and Diagnostics frames and fails if any of those tokens is drawn.

## Shipped copy rule (2026-10-29)

The direction above is kept, with one change decided against the running shell:
the *concept* is explained, the *token* is not printed. Storage rows read
"Read-only" or "Read and write", and the sentence beside a protected volume says
the system storage cannot change while Litric runs and that every block is
checked as it is read; the filesystem's own name no longer appears in any frame.
Network rows read "Connected", "Not connected" or "Connecting" -- interface
names, hardware addresses and `/dev` paths stay out of the default rows.

The vocabulary lives in `src/litric-shell/src/copy.rs`; `copy::BANNED_TOKENS` is
the single forbidden list, and the journey
`journey_no_machine_token_reaches_the_system_surfaces` renders the real Tasks,
Network and Diagnostics frames and fails if any of those tokens is drawn.
