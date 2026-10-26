# LITRIC — COMPLETE SYSTEM SCOPE

> **What "ENTIRE SYSTEM" means, itemised.** Every row is one thing a user or a
> developer can point at. Every row says who builds it and what state it is in
> **today**.
>
> The first version of this document had 160 rows. That was too coarse: it
> collapsed "Settings" into fifteen rows when Windows alone exposes several
> hundred distinct settings, and it left out entire categories — built-in
> applications, accessibility, localisation, printing, enterprise, developer
> tooling. This version is the real decomposition.
>
> Part J was added from the LITRIC Unified Resource, Power, Thermal,
> Environment and Validation Engine specification, and Part J1 from ADR-0015
> (LitricOS as host, Windows and Linux as switchable guests).
>
> Most of the growth from 160 onward is **decomposition, not discovery**. The
> work did not get bigger; it got countable. A row you can finish in a day is a
> row you can schedule. "Build Settings" is not.

**Owner:** `O` = we write it · `U` = upstream, we integrate · `O+U` = upstream
component, our UI and policy on top

**Status:** `D` = done and tested · `P` = partial, real code with passing tests ·
`S` = specified, no code · `T` = not started

---

> **2026-08-27: LitricOS boots.** The `nano` profile builds and boots to a login
> prompt in QEMU. Five rows below moved to `D`, each demonstrated by
> `docs/evidence/gate01-boot.log`.
>
> One row moved *backwards*, from `S` to `T`: row 19, "mainline LTS kernel,
> trimmed config". The booting image uses Arch's **stock** kernel package. Our
> kernel configuration fragment has been linted but has still never been applied
> to an actual kernel build, so none of the symbols it enables — MGLRU,
> sched_ext, ntsync, the LSM stack — are in the running system. The fragment
> looked more finished than it was, and booting the image is what exposed that.

> **2026-08-28: design tokens and trash policy.** Sixteen rows moved from `S`/`T`
> to `P` on the strength of `litric-design` (33 tests) and `litric-trash`
> (17 tests). `P` is the right status and `D` is not, for a specific reason:
> both crates decide *what is true*, and nothing yet draws a pixel or moves a
> byte. The glass material knows the alpha at which text stays readable over a
> given wallpaper; no compositor asks it. The trash knows which file a name
> collision renames and which restore would overwrite; no daemon performs it.
> That gap closes at Layer 5, which does not exist yet.

> **2026-08-29: LitricOS boots to its own desktop.** `litric-sysd` (Layer 4)
> and `litric-shell` (Layer 5) both run inside the image. The screen in
> `docs/evidence/desktop-boot-2026-08-29.png` is the guest's own framebuffer,
> read out through the hypervisor, showing the menu bar, a window, the
> environment switcher and the dock drawn by our renderer in our typeface,
> filled with numbers the daemon measured. Zero failed units, dm-verity
> active, five LSMs live, and every glass surface solved to 18% tint with a
> worst text contrast of 5.19:1.
>
> Seven more rows moved to `P`. None moved to `D`, and that is the point: a
> menu bar that cannot be configured, a clock with no format setting and a
> system monitor with no per-process view are all partial, however good the
> screenshot looks. `D` still means finished.
>
> Two things the screenshot does **not** show, and neither is implied by it:
> there is no input handling of any kind, and no client application can draw
> into that desktop. It is the output half of a compositor. Rows 880 and 1004
> stay at `T`.

> **2026-10-25: a foreign application can draw into that desktop.** The
> sentence above that said "no client application can draw into that desktop"
> is no longer true, and this note is here rather than a quiet edit to the
> history above it. `litric-compositor` is a real Wayland server inside the
> shell's process (ADR-0024): `wl_compositor`, `wl_shm`, `wl_seat`, `wl_output`
> and `xdg_wm_base`, a socket at `$XDG_RUNTIME_DIR/wayland-0`, and client
> surfaces hosted as ordinary windows with LitricOS title bars.
> `weston-simple-shm` — a C client from outside this repository — opens a
> window, draws, resizes, survives input and closes, in
> `artifacts/foreign-client-window.png` and in two test suites that skip
> loudly when the client binary is absent.
>
> The rows are unchanged, and that is deliberate rather than an oversight.
> Row 880 is guest *VM* surface compositing, which is still `T`: a virtual
> machine's framebuffer is not a Wayland client. Row 1004 is native compositor
> frame rate, still `P`. The tally does not move because a capability arrived
> that these rows never described — the scope table's granularity is the
> problem, and inventing a row to move the percentage would be the kind of
> number-keeping this document exists to prevent. What is still absent in the
> same area, and is listed in ADR-0024 rather than implied here: XWayland,
> clipboard and drag-and-drop, input methods, `.desktop` launching, dmabuf and
> any GPU path.

> **2026-09-14: the platform contract widens.** Part K records three architecture decisions from the unified-compatibility and deep-recovery requirement set: one desktop with three application runtimes (ADR-0020), deep recovery with user data as an independent domain (ADR-0021), and the Defender engine whose Phase 0 policy core now exists as `src/litric-defender` (ADR-0022). Ten rows are `P` because a decision function with passing tests exists; the executables that act on those decisions do not, and the statuses say so.

> **2026-08-29 (later): the desktop responds.** Keyboard and mouse reach it
> through `litric-input`, a window manager decides what a click means, a
> shortcut table turns keys into actions, and the compositor repaints only the
> rows that changed. Driven through the hypervisor's own keyboard and mouse and
> photographed at each step:
> `docs/evidence/desktop-interactive-2026-08-29.png`.
>
> Twenty-seven rows moved to `P`. Every one of them is a feature you can now
> use and none of them is finished — Alt+Tab has no previews, the launcher has
> no file search, notifications cannot be snoozed, and the browser has no
> engine. `P` is the honest status for all of that.
>
> Three things the screenshots do **not** show. There is still no client
> application: every window is drawn by the shell. There is no lock screen,
> because there is no authentication subsystem and a lock that accepts anything
> is worse than none. And `Win+1` moves the environment state machine without
> starting a virtual machine — the notification on screen says
> `applied=state-only`, which is the daemon's own word for it.

## TALLY

| Status | Count | Share |
|---|---:|---:|
| `D` done and tested | **9** | 0.8% |
| `P` partial, real code with passing tests | **116** | 10.0% |
| `S` specified, no code | **294** | 25.2% |
| `T` not started | **746** | 64.0% |
| **Total** | **1165** | |

Nine rows are done: the boot chain, proved by `docs/evidence/gate01-boot.log`.
A hundred and four are partial — real code with passing tests, most of it now
running inside the image: the capability engine, the storage broker, the
manifest parser, the Windows compatibility planner and its known-software
table, the environment manager, the design tokens, the trash policy, the input
stack, the window manager, the localisation layer, the browser's model, and the
shell that draws and drives the desktop. Some of those 104 are two views of one
crate, counted once in Part E or F and once in Part I.

Counted honestly, **LitricOS is roughly 5% built**. The 9.3% above is generous
to us: it counts a decision made correctly the same as a decision made
correctly *and* carried out, and it counts a menu bar that is drawn the same
as a menu bar you can configure.

These counts are not typed by hand. `ci/count-scope.py` parses this file and
`ci/verify.sh` fails if the table above disagrees with the rows below — because
a hand-maintained tally drifts, and a drifting tally is how a project starts
believing it is further along than it is. The first version of this table was
written before the rows were finished and claimed 707; the parser said 854.

---

## 0. WHAT WE WRITE VERSUS WHAT WE INTEGRATE

The single most important decision in the project, so it comes first.

| We write | We integrate, never rewrite |
|---|---|
| Shell, compositor, window management | Linux kernel, drivers |
| Settings, capability broker, permission UI | Mesa, Vulkan, DRM/KMS |
| File manager, Trash, indexer, search | PipeWire, WirePlumber |
| Defender, sandbox policy | systemd, iwd, BlueZ, CUPS |
| Update system, A/B slots, rollback | Wine, Proton, DXVK, VKD3D |
| Windows `.exe` subsystem (bottles, planning) | gamescope, GameMode, scx schedulers |
| Native SDK, package format, store | **Browser engine** (Chromium or WebKit) |
| Browser UI, privacy model, capability wiring | llama.cpp, whisper.cpp, piper |
| First-run experience, design system, animation | Flatpak, bubblewrap, XDG portals |
| Built-in applications | Poppler, FFmpeg, libvips, Harfbuzz |

**A browser engine is not something anyone writes.** Edge is Chromium. Opera,
Brave, Vivaldi and Arc are Chromium. Apple started WebKit in 1998. Litric Browser
is our shell, privacy model, capability integration and design, on an existing
engine — which is what every competitor except Apple and Google does.

The same logic governs the kernel. `15-APPENDIX §7` already records the lesson:
SerenityOS rewrote everything and mostly runs in emulators.

---

# PART A — FOUNDATION

## A1. Boot and firmware (18)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 1 | UEFI boot via Unified Kernel Image | O+U | **D** |
| 2 | Secure Boot with our keys | O+U | S |
| 3 | Secure Boot key enrolment flow (MokManager / KEK+db) | O | T |
| 4 | Boot menu with entry selection | U | S |
| 5 | Boot counting with automatic rollback after N failures | U | S |
| 6 | Recovery boot entry | O | S |
| 7 | Safe-graphics boot entry (`nomodeset`) | O | S |
| 8 | Defender-off boot entry | O | S |
| 9 | Verbose/diagnostic boot entry with persistent logs | O | S |
| 10 | TPM 2.0 measured boot, PCR policy | O+U | S |
| 11 | dm-verity root hash embedded in the signed UKI | O | **D** |
| 12 | Fail-closed on a tampered system partition | O+U | S |
| 13 | Firmware update via fwupd + LVFS | U | T |
| 14 | CPU microcode updates | U | T |
| 15 | Boot splash / plymouth-class progress | O | T |
| 16 | Serial console for headless and CI | O+U | **D** |
| 17 | Boot-time measurement and budget enforcement | O | S |
| 18 | Kernel command-line policy, locked against edit at the menu | O | S |

## A2. Kernel and core system (26)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 19 | Mainline LTS kernel, trimmed config | O+U | T |
| 20 | MGLRU page reclaim | U | S |
| 21 | `sched_ext` framework for pluggable schedulers | U | S |
| 22 | `ntsync` NT synchronisation primitives | U | S |
| 23 | fanotify with permission events | U | S |
| 24 | Landlock LSM | U | S |
| 25 | BPF-LSM | U | S |
| 26 | lockdown LSM | U | S |
| 27 | yama LSM (ptrace restriction) | U | S |
| 28 | loadpin LSM (modules only from the verified partition) | U | S |
| 29 | safesetid LSM | U | S |
| 30 | integrity subsystem | U | S |
| 31 | seccomp-bpf syscall filtering | U | S |
| 32 | cgroups v2 resource governance | O+U | S |
| 33 | Per-slice memory, CPU, IO limits | O | S |
| 34 | systemd-oomd PSI-based OOM management | O+U | S |
| 35 | zram swap with selectable algorithm | O+U | S |
| 36 | Transparent huge pages, madvise-only | O | S |
| 37 | Service supervision and restart policy | U | S |
| 38 | Socket activation for every non-essential daemon | O | S |
| 39 | D-Bus broker IPC | U | S |
| 40 | Idle process-count budget, enforced in CI | O | S |
| 41 | Idle wakeup budget, enforced in CI | O | S |
| 42 | Idle disk-write budget, enforced in CI | O | S |
| 43 | Recovery target (shell, storage tools, log extraction) | O | S |
| 44 | Crash reporting with symbolication | O | T |

## A3. Storage and filesystems (34)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 45 | erofs read-only system partition | O+U | **D** |
| 46 | dm-verity integrity over the system partition | O+U | **D** |
| 47 | A/B system slots | O | S |
| 48 | Btrfs user and var partitions | U | S |
| 49 | Btrfs subvolumes (`@home`, `@var`, `@snapshots`, `@games`) | O | S |
| 50 | zstd compression on user data | O | S |
| 51 | `nodatacow` on game and database directories | O | S |
| 52 | Snapshots, automatic pre-update | O | S |
| 53 | Snapshot browse and restore UI | O | T |
| 54 | LUKS2 full-disk encryption with Argon2id | U | S |
| 55 | TPM2-sealed auto-unlock | O+U | S |
| 56 | Recovery key generated at install, shown once | O | S |
| 57 | Passphrase fallback when the PCR seal breaks | O+U | S |
| 58 | `fscrypt` per-user home encryption | U | T |
| 59 | `fs-verity` on application content | U | S |
| 60 | Logical storage namespaces (`user://`, `app://`, `cache://`…) | O | **P** |
| 61 | Storage broker translating namespaces to real paths | O | **P** |
| 62 | Per-app storage quota | O | T |
| 63 | Disk usage analysis by category | O | T |
| 64 | Storage cleanup: caches, old snapshots, orphaned app data | O | T |
| 65 | Low-space warning at 85%, update block at 95% | O | S |
| 66 | External drive mount, unmount, safe eject | O+U | T |
| 67 | Automatic mount policy for removable media | O | T |
| 68 | Filesystem check and repair tooling | U | T |
| 69 | Btrfs scrub scheduling and reporting | O | T |
| 70 | SMART monitoring and failure prediction | O+U | T |
| 71 | TRIM scheduling | U | T |
| 72 | Disk partitioning UI (installer only) | O | T |
| 73 | Format removable media | O | T |
| 74 | NTFS read/write for Windows dual-boot data | U | T |
| 75 | exFAT, FAT32, ext4 support | U | S |
| 76 | Network shares: SMB | U | T |
| 77 | Network shares: NFS | U | T |
| 78 | MTP / phone browsing | U | T |

---

# PART B — SHELL AND DESKTOP

## B1. Panel, taskbar, tray (28)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 79 | Panel with configurable position (top/bottom/left/right) | O | T |
| 80 | Auto-hide with reveal on hover | O | T |
| 81 | Panel transparency and blur | O | **P** |
| 82 | Running-application list with grouping | O | T |
| 83 | Pinned applications | O | T |
| 84 | Window preview on hover | O | T |
| 85 | Jump lists / recent documents per app | O | T |
| 86 | Progress indication on the task button | O | T |
| 87 | Badge counts on the task button | O | T |
| 88 | Middle-click to open a new instance | O | T |
| 89 | Drag to reorder task buttons | O | T |
| 90 | System tray / status notifier items | O | T |
| 91 | Clock with configurable format | O | **P** |
| 92 | Calendar flyout with events | O | T |
| 93 | Volume control with per-app mixer | O | T |
| 94 | Network status and picker | O | T |
| 95 | Bluetooth status and picker | O | T |
| 96 | Battery indicator with time estimate | O | T |
| 97 | Brightness control | O | T |
| 98 | Quick settings panel (toggles grid) | O | **P** |
| 99 | Night light / colour temperature toggle | O | T |
| 100 | Do Not Disturb toggle | O | T |
| 101 | Screen recording / screenshot entry point | O | T |
| 102 | Media playback controls for the active player | O | T |
| 103 | Input method / keyboard layout indicator | O | T |
| 104 | Microphone and camera in-use indicator | O | T |
| 105 | Multi-monitor: panel on each display, or one | O | T |
| 106 | Show desktop button / gesture | O | T |

## B2. Window management (42)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 107 | Move, resize, minimise, maximise, restore, close | O | **P** |
| 108 | Keyboard-driven move and resize | O | T |
| 109 | Snap to left/right half | O | **P** |
| 110 | Snap to quarters | O | T |
| 111 | Snap to thirds (ultrawide) | O | T |
| 112 | Snap layout picker on hover over maximise | O | T |
| 113 | Snap groups: restore a snapped set together | O | T |
| 114 | Snap assist: suggest what fills the other half | O | T |
| 115 | Custom snap zones | O | T |
| 116 | Tiling mode as a first-class alternative | O | T |
| 117 | Per-workspace tiling/floating choice | O | T |
| 118 | Always-on-top | O | T |
| 119 | Sticky (visible on all workspaces) | O | T |
| 120 | Fullscreen, distinct from maximised | O | T |
| 121 | Picture-in-picture / minimised video overlay | O | T |
| 122 | Window opacity control | O | T |
| 123 | Window shading (roll up to title bar) | O | T |
| 124 | Remember size and position per application | O | T |
| 125 | Remember placement per monitor arrangement | O | T |
| 126 | Restore windows sensibly when a monitor disappears | O | T |
| 127 | Alt-Tab switcher with live previews | O | **P** |
| 128 | Alt-Tab scoped to the current workspace, or global | O | T |
| 129 | Alt-` to cycle windows of one application | O | T |
| 130 | Overview showing every window laid out (Mission Control) | O | T |
| 131 | Overview scoped per workspace | O | T |
| 132 | Type-to-filter inside the overview | O | T |
| 133 | Drag a window between workspaces in the overview | O | T |
| 134 | Expose-style "show windows of this app" | O | T |
| 135 | Focus-follows-mouse option | O | T |
| 136 | Click-to-focus with raise-on-click option | O | **P** |
| 137 | Focus stealing prevention | O | T |
| 138 | Urgency hint / attention request | O | T |
| 139 | Window rules engine (match by app, title, class) | O | T |
| 140 | Per-application default workspace | O | T |
| 141 | Server-side decorations with a consistent look | O | T |
| 142 | Client-side decoration support | O | T |
| 143 | Rounded corners with correct antialiasing | O | S |
| 144 | Drop shadows with correct occlusion | O | S |
| 145 | Interruptible, physics-based window animations | O | S |
| 146 | Reduced-motion honoured throughout | O | S |
| 147 | Session save and restore across logout | O | T |
| 148 | Kill a hung window, with a "not responding" state | O | T |

## B3. Workspaces and multi-monitor (24)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 149 | Multiple virtual desktops | O | T |
| 150 | Dynamic workspace creation and removal | O | T |
| 151 | Fixed workspace count option | O | T |
| 152 | Name a workspace | O | T |
| 153 | Per-workspace wallpaper | O | T |
| 154 | Move a window to another workspace | O | **P** |
| 155 | Move a window to another workspace and follow it | O | T |
| 156 | Switch workspace by keyboard | O | **P** |
| 157 | Switch workspace by trackpad gesture | O | T |
| 158 | Workspace switch animation | O | S |
| 159 | Workspace grid or linear arrangement | O | T |
| 160 | Per-monitor workspaces, or shared across monitors | O | T |
| 161 | Multi-monitor: arrange displays by dragging | O | T |
| 162 | Per-monitor resolution and refresh rate | O | T |
| 163 | Per-monitor scale factor | O | T |
| 164 | Fractional scaling | O | S |
| 165 | Per-monitor rotation | O | T |
| 166 | Primary monitor selection | O | T |
| 167 | Mirror displays | O | T |
| 168 | Extend displays | O | T |
| 169 | Use one display only | O | T |
| 170 | Hotplug without crash or state loss | O | S |
| 171 | Move a window to the next monitor by keyboard | O | T |
| 172 | Remember per-arrangement layout, restore on reconnect | O | T |

## B4. Launcher, search, command palette (26)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 173 | Application launcher with grid and list views | O | **P** |
| 174 | Application categories and folders | O | T |
| 175 | Pin and reorder favourites | O | T |
| 176 | Type-to-search applications | O | **P** |
| 177 | Fuzzy matching with sensible ranking | O | T |
| 178 | Recently used and frequently used | O | T |
| 179 | Search files by name | O | T |
| 180 | Search file contents (full-text index) | O | T |
| 181 | Search settings by name and by synonym | O | T |
| 182 | Search browser history and bookmarks | O | T |
| 183 | Search contacts, calendar, mail | O | T |
| 184 | Inline calculator | O | T |
| 185 | Unit and currency conversion | O | T |
| 186 | Run a command | O | T |
| 187 | Open a URL | O | T |
| 188 | Dictionary and definitions | O | T |
| 189 | Emoji search and insert | O | T |
| 190 | Clipboard history search | O | T |
| 191 | Application actions from search (Spotlight-style intents) | O | T |
| 192 | Window switching from search | O | T |
| 193 | Search provider plugin API | O | T |
| 194 | Per-provider privacy toggle | O | T |
| 195 | Indexer respects capability scopes | O | T |
| 196 | Indexer runs at low priority, pauses on battery | O | T |
| 197 | Index exclusion list | O | T |
| 198 | Rebuild index / clear index | O | T |

## B5. Notifications (20)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 199 | Notification popups with app identity | O | **P** |
| 200 | Actions and inline reply | O | T |
| 201 | Notification centre with history | O | **P** |
| 202 | Grouping by application | O | T |
| 203 | Per-app enable/disable | O | T |
| 204 | Per-app priority and preview settings | O | T |
| 205 | Do Not Disturb with schedule | O | T |
| 206 | Focus modes with per-mode allow lists | O | T |
| 207 | Automatic suppression in fullscreen and Game Mode | O | S |
| 208 | Urgent notifications bypass suppression | O | **P** |
| 209 | Persistent vs transient notifications | O | **P** |
| 210 | Progress notifications | O | T |
| 211 | Sound per notification category | O | T |
| 212 | Notification position configurable | O | T |
| 213 | Snooze a notification | O | T |
| 214 | Lock-screen notification visibility policy | O | T |
| 215 | Notification requires the `notify` capability | O | S |
| 216 | Rate limiting to prevent notification spam | O | T |
| 217 | Badge counts propagated to the panel | O | T |
| 218 | Notification history search | O | T |

## B6. Desktop, lock screen, session (22)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 219 | Wallpaper: static image | O | **P** |
| 220 | Wallpaper: slideshow | O | T |
| 221 | Wallpaper: per-monitor | O | T |
| 222 | Wallpaper: dynamic (time of day) | O | T |
| 223 | Desktop icons, optional | O | T |
| 224 | Desktop right-click menu | O | T |
| 225 | Widgets on the desktop | O | T |
| 226 | Lock screen with clock and date | O | T |
| 227 | Lock screen notifications with a privacy policy | O | T |
| 228 | Lock screen media controls | O | T |
| 229 | Automatic lock on idle | O | T |
| 230 | Lock on lid close | O | T |
| 231 | Login screen (greeter) | O | T |
| 232 | Password login | O+U | T |
| 233 | Fingerprint login | U | T |
| 234 | Multiple user accounts | O+U | T |
| 235 | Fast user switching | O+U | T |
| 236 | Guest session | O | T |
| 237 | Log out, restart, shut down, suspend, hibernate | O+U | T |
| 238 | Confirm-before-shutdown when apps have unsaved work | O | T |
| 239 | Session restore after restart | O | T |
| 240 | Screen saver / display blanking policy | O | T |

## B7. Design system, animation, theming (24)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 241 | Design tokens: colour, type, spacing, radius, motion | O | **P** |
| 242 | Light theme | O | **P** |
| 243 | Dark theme | O | **P** |
| 244 | Automatic light/dark by time or ambient light | O | T |
| 245 | High-contrast theme | O | **P** |
| 246 | Accent colour selection | O | **P** |
| 247 | Accent extracted from the wallpaper | O | T |
| 248 | Consistent corner radii across every surface | O | **P** |
| 249 | Real shadows with correct elevation model | O | S |
| 250 | Blur and translucency (glass) with a performance budget | O | **P** |
| 251 | Spring/physics animation curves, interruptible | O | **P** |
| 252 | Animations run at display refresh, never fixed 60 | O | S |
| 253 | Animation duration scale, user-adjustable | O | T |
| 254 | Reduced motion disables non-essential animation | O | **P** |
| 255 | Icon theme, coherent metaphors | O | T |
| 256 | Cursor theme and size | O | **P** |
| 257 | Font selection with a coherent type scale | O | **P** |
| 258 | Font hinting and subpixel rendering settings | O+U | T |
| 259 | UI density option (compact / comfortable) | O | T |
| 260 | Sound theme | O | T |
| 261 | Per-app theme override | O | T |
| 262 | Theme API for first-party shell plugins | O | T |
| 263 | No GTK/Qt/Electron anywhere in the graphical path | O | S |
| 264 | `ldd` allowlist enforced in CI so no toolkit creeps in | O | S |

---

# PART C — FILES

## C1. File manager (48)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 265 | Browse directories | O | T |
| 266 | Icon view | O | T |
| 267 | List view with sortable columns | O | T |
| 268 | Compact / column view | O | T |
| 269 | Tree sidebar | O | T |
| 270 | Bookmarks and pinned locations | O | T |
| 271 | Breadcrumb path bar with editable path | O | T |
| 272 | Tabs | O | T |
| 273 | Split / dual pane | O | T |
| 274 | Back, forward, up navigation with history | O | T |
| 275 | New folder, new file from template | O | T |
| 276 | Rename, including inline rename | O | T |
| 277 | Batch rename with pattern and preview | O | T |
| 278 | Copy, cut, paste | O | T |
| 279 | Drag and drop within and between windows | O | T |
| 280 | Drag and drop to and from other applications | O | T |
| 281 | Copy progress with speed, ETA, pause and cancel | O | T |
| 282 | Conflict resolution dialog (skip, replace, keep both) | O | T |
| 283 | Undo of file operations | O | T |
| 284 | Delete to Trash | O | T |
| 285 | Permanent delete, with confirmation | O | T |
| 286 | Select all, invert selection, select by pattern | O | T |
| 287 | Sort by name, size, type, date | O | T |
| 288 | Group by type or date | O | T |
| 289 | Filter / search within the current folder | O | T |
| 290 | Recursive search with live results | O | T |
| 291 | Saved searches | O | T |
| 292 | Show and hide hidden files | O | T |
| 293 | Properties: size, dates, type, location | O | T |
| 294 | Properties: permissions editor | O | T |
| 295 | Properties: open-with default | O | T |
| 296 | Thumbnails: images | O | T |
| 297 | Thumbnails: video | O | T |
| 298 | Thumbnails: PDF and documents | O | T |
| 299 | Thumbnails generated out of process, sandboxed | O | S |
| 300 | Thumbnail cache with size limit | O | T |
| 301 | Preview pane | O | T |
| 302 | Quick Look: space bar to preview any file | O | T |
| 303 | Quick Look: images, video, audio, PDF, text, archives | O | T |
| 304 | Archive: extract here / extract to | O+U | T |
| 305 | Archive: create zip, tar, 7z | O+U | T |
| 306 | Archive: browse without extracting | O+U | T |
| 307 | Free-space indicator | O | T |
| 308 | Folder size calculation on demand | O | T |
| 309 | Disk usage treemap | O | T |
| 310 | Open terminal here | O | T |
| 311 | Compute checksum of a file | O | T |
| 312 | File operations respect capability scopes | O | S |

## C2. Trash / Recycle Bin (12)

Implements the freedesktop.org Trash specification, so trashed files remain
interoperable with every other Linux tool.

`src/litric-trash` is the policy half and is tested (17 tests): collision
naming, refusal to overwrite on restore, refusal to accept an item larger than
the whole trash, and an eviction plan that is returned for inspection before
anything is destroyed. It performs no I/O and has no view; those rows stay `T`.

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 313 | Move to Trash instead of deleting | O | **P** |
| 314 | Trash view listing deleted items | O | T |
| 315 | Restore to original location | O | **P** |
| 316 | Restore to a chosen location | O | T |
| 317 | Empty Trash | O | **P** |
| 318 | Delete a single item permanently from Trash | O | **P** |
| 319 | Original path and deletion time recorded and shown | O | **P** |
| 320 | Trash on removable media, per-volume | O | T |
| 321 | Size limit with oldest-first eviction | O | **P** |
| 322 | Automatic emptying after N days, configurable | O | **P** |
| 323 | Trash size shown in Settings > Storage | O | T |
| 324 | Trash portal so sandboxed apps can trash without file access | O+U | T |

---

# PART D — SETTINGS

## D1. System and display (34)

| # | Setting | Owner | Status |
|---:|---|:---:|:---:|
| 325 | Display: resolution | O | T |
| 326 | Display: refresh rate | O | T |
| 327 | Display: scale, including fractional | O | T |
| 328 | Display: orientation | O | T |
| 329 | Display: arrangement by drag | O | T |
| 330 | Display: primary selection | O | T |
| 331 | Display: mirror / extend / single | O | T |
| 332 | Display: HDR enable and tone mapping | O | T |
| 333 | Display: colour profile / ICC | O | T |
| 334 | Display: night light with schedule | O | T |
| 335 | Display: VRR / adaptive sync | O | T |
| 336 | Display: overscan correction | O | T |
| 337 | Display: identify displays | O | T |
| 338 | Brightness per display | O | T |
| 339 | Automatic brightness from ambient light | O+U | T |
| 340 | Text size / global UI scale | O | T |
| 341 | Notifications: master and per-app | O | T |
| 342 | Focus assist / Do Not Disturb rules | O | T |
| 343 | Power: profile selection | O+U | T |
| 344 | Power: screen-off timeout | O | T |
| 345 | Power: sleep timeout | O | T |
| 346 | Power: lid-close action | O | T |
| 347 | Power: power-button action | O | T |
| 348 | Power: battery percentage display | O | T |
| 349 | Power: battery health | O | T |
| 350 | Power: per-app energy usage | O | T |
| 351 | Power: charge limit for battery longevity | O+U | T |
| 352 | Storage: usage by category | O | T |
| 353 | Storage: cleanup recommendations | O | T |
| 354 | Storage: per-app data size | O | T |
| 355 | Multitasking: snap behaviour options | O | T |
| 356 | Multitasking: Alt-Tab scope | O | T |
| 357 | About: hardware, OS version, build hash | O | T |
| 358 | About: copy diagnostics for a bug report | O | T |

## D2. Sound (16)

| # | Setting | Owner | Status |
|---:|---|:---:|:---:|
| 359 | Output device selection | O+U | T |
| 360 | Input device selection | O+U | T |
| 361 | Master volume and mute | O | T |
| 362 | Per-application volume mixer | O | T |
| 363 | Per-application output routing | O | T |
| 364 | Input level with live meter | O | T |
| 365 | Input noise suppression | O+U | T |
| 366 | Echo cancellation | O+U | T |
| 367 | Sample rate and bit depth | O+U | T |
| 368 | Buffer size / quantum (latency) | O+U | T |
| 369 | Channel configuration (stereo, 5.1, 7.1) | O+U | T |
| 370 | Spatial audio | O+U | T |
| 371 | Balance and per-channel trim | O | T |
| 372 | Test each speaker | O | T |
| 373 | Bluetooth codec selection (SBC, AAC, aptX, LDAC, LC3) | U | T |
| 374 | System sound theme and volume | O | T |

## D3. Network (26)

| # | Setting | Owner | Status |
|---:|---|:---:|:---:|
| 375 | Wi-Fi: scan and list networks with signal | O+U | T |
| 376 | Wi-Fi: connect with WPA2/WPA3 | U | S |
| 377 | Wi-Fi: hidden SSID | U | T |
| 378 | Wi-Fi: enterprise 802.1X | U | T |
| 379 | Wi-Fi: saved networks, forget, priority | O | T |
| 380 | Wi-Fi: metered connection flag | O | T |
| 381 | Wi-Fi: randomised MAC per network | O+U | T |
| 382 | Wi-Fi: QR code to share a network | O | T |
| 383 | Ethernet: DHCP and static | U | S |
| 384 | IPv6: SLAAC, DHCPv6, privacy extensions | U | S |
| 385 | DNS: custom servers | U | S |
| 386 | DNS: DNS-over-TLS | U | S |
| 387 | DNS: DNSSEC validation | U | S |
| 388 | Proxy configuration | O+U | T |
| 389 | VPN: WireGuard | U | S |
| 390 | VPN: OpenVPN | U | T |
| 391 | VPN: kill switch | O | T |
| 392 | VPN: split tunnelling | O | T |
| 393 | Hotspot / connection sharing | O+U | T |
| 394 | Airplane mode | O | T |
| 395 | Firewall: default-deny inbound | O+U | S |
| 396 | Firewall: per-application rules | O | T |
| 397 | Per-app network usage statistics | O | T |
| 398 | Captive portal detection and sign-in | O | T |
| 399 | Network diagnostics (ping, traceroute, speed) | O | T |
| 400 | Connection priority / interface metric | O | T |

## D4. Bluetooth and devices (24)

| # | Setting | Owner | Status |
|---:|---|:---:|:---:|
| 401 | Bluetooth: on/off | U | T |
| 402 | Bluetooth: scan and pair | U | T |
| 403 | Bluetooth: trusted devices, auto-reconnect | U | T |
| 404 | Bluetooth: audio (A2DP) | U | T |
| 405 | Bluetooth: LE Audio | U | T |
| 406 | Bluetooth: headset profile with microphone | U | T |
| 407 | Bluetooth: input devices (keyboard, mouse) | U | T |
| 408 | Bluetooth: file transfer | U | T |
| 409 | Bluetooth: battery level of connected devices | U | T |
| 410 | Mouse: pointer speed and acceleration profile | O+U | T |
| 411 | Mouse: natural vs traditional scrolling | O+U | T |
| 412 | Mouse: button remapping | O | T |
| 413 | Touchpad: tap to click | U | T |
| 414 | Touchpad: two-finger scroll | U | T |
| 415 | Touchpad: pinch zoom | U | T |
| 416 | Touchpad: three- and four-finger gestures | O | T |
| 417 | Touchpad: palm rejection | U | T |
| 418 | Touchpad: disable while typing | U | T |
| 419 | Pen and tablet: pressure, mapping, buttons | U | T |
| 420 | Touchscreen calibration | U | T |
| 421 | Game controller: detection and remapping | O+U | T |
| 422 | Game controller: rumble and LED | U | T |
| 423 | USB device list and per-device permission | O | T |
| 424 | Thunderbolt / USB4 device approval | U | T |

## D5. Personalisation, apps, accounts, time (30)

| # | Setting | Owner | Status |
|---:|---|:---:|:---:|
| 425 | Wallpaper picker | O | **P** |
| 426 | Theme: light / dark / auto | O | T |
| 427 | Accent colour | O | T |
| 428 | Font family and size | O | T |
| 429 | Icon theme | O | T |
| 430 | Cursor theme and size | O | **P** |
| 431 | Panel position, size, auto-hide | O | T |
| 432 | Launcher layout | O | T |
| 433 | Lock screen appearance | O | T |
| 434 | Animation speed | O | T |
| 435 | Sound theme | O | T |
| 436 | Apps: installed list with size and last used | O | T |
| 437 | Apps: uninstall | O | T |
| 438 | Apps: per-app permissions | O | T |
| 439 | Apps: per-app storage and cache clearing | O | T |
| 440 | Apps: default application per file type | O | T |
| 441 | Apps: default application per URL scheme | O | T |
| 442 | Apps: startup / autostart list with impact | O | T |
| 443 | Apps: per-app resource limits | O | T |
| 444 | Accounts: create, edit, delete users | O+U | T |
| 445 | Accounts: administrator vs standard | O+U | T |
| 446 | Accounts: password change and policy | O+U | T |
| 447 | Accounts: fingerprint enrolment | U | T |
| 448 | Accounts: avatar | O | T |
| 449 | Accounts: parental controls / screen time | O | T |
| 450 | Time: automatic via NTP | U | T |
| 451 | Time: manual set | O+U | T |
| 452 | Time: time zone, automatic by location | O+U | T |
| 453 | Time: 12/24-hour format | O | T |
| 454 | Region: number, date, currency formats | O | T |

## D6. Expert tier (14)

Three visible tiers — Simple, Advanced, Expert — per Phase 06 §4. Every Expert
knob shows current value, default, what it changes, measured effect where known,
security cost, and a revert button.

| # | Setting | Owner | Status |
|---:|---|:---:|:---:|
| 455 | CPU governor and energy preference | O | S |
| 456 | CPU core isolation and pinning | O | S |
| 457 | Split-lock detection mode | O | S |
| 458 | zram algorithm and size | O | S |
| 459 | Swappiness and watermark tuning | O | S |
| 460 | Transparent huge page policy | O | S |
| 461 | GPU power level and clock offsets | O | S |
| 462 | IO scheduler per device | O | S |
| 463 | IRQ affinity | O | S |
| 464 | Network: busy-poll, coalescing, congestion control | O | S |
| 465 | Scheduler selection (EEVDF / scx_lavd / scx_bpfland) | O | S |
| 466 | `mitigations=off`, behind an explicit warning dialog | O | S |
| 467 | Arbitrary sysctl editor with revert | O | S |
| 468 | Export and import an Expert profile | O | T |

---

# PART E — SECURITY AND PRIVACY

## E1. Permissions (34)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 469 | Capability model: deny by default | O | **P** |
| 470 | Capability model: resource-scoped grants | O | **P** |
| 471 | Capability model: path-traversal-proof scopes | O | **P** |
| 472 | Runtime revocation, effective immediately | O | **P** |
| 473 | Time-boxed grants with expiry | O | **P** |
| 474 | Audit log of every allow and deny | O | **P** |
| 475 | Confused-deputy defence (caller identity, never claimed) | O | **P** |
| 476 | Capability broker daemon (`litric-capd`) | O | **P** |
| 477 | Camera permission | O+U | T |
| 478 | Microphone permission | O+U | T |
| 479 | Location permission | O+U | T |
| 480 | Screen capture permission | O+U | T |
| 481 | Screen recording permission, separate from capture | O+U | T |
| 482 | File access via picker (the choice IS the grant) | O+U | T |
| 483 | Pictures / albums access | O | T |
| 484 | Documents access | O | T |
| 485 | Downloads access | O | T |
| 486 | Removable media access | O | T |
| 487 | Network access | O | **P** |
| 488 | Clipboard read | O | T |
| 489 | Clipboard write | O | T |
| 490 | Notification permission | O | T |
| 491 | Background execution permission | O | T |
| 492 | Autostart permission | O | T |
| 493 | USB device permission, per device | O+U | T |
| 494 | Bluetooth device permission | O+U | T |
| 495 | Global shortcut registration permission | O+U | T |
| 496 | Keyring / secret access permission | O+U | T |
| 497 | Printing permission | U | T |
| 498 | Contacts and calendar permission | O | T |
| 499 | Live indicator whenever mic or camera is active | O | T |
| 500 | Indicator cannot be suppressed by an application | O | S |
| 501 | Per-app permission page in Settings | O | T |
| 502 | Permission prompt with plain-language wording | O | **P** |

## E2. Sandbox and system integrity (26)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 503 | Every application sandboxed by default | O+U | S |
| 504 | Landlock filesystem confinement per app | O+U | S |
| 505 | seccomp syscall filter per app profile | O+U | S |
| 506 | Namespace isolation (mount, pid, net, user) | U | S |
| 507 | Per-app cgroup limits | O | S |
| 508 | No ambient authority for any application | O | S |
| 509 | Systemd sandboxing on every system service | O | S |
| 510 | W^X memory policy where practical | O | S |
| 511 | ASLR, stack protection, CFI on our binaries | O | S |
| 512 | Signed system image, verified at boot | O+U | S |
| 513 | `fs-verity` on application content | U | S |
| 514 | Signed application packages | O | S |
| 515 | Signature verification before install | O | S |
| 516 | Revocation list checking | O | T |
| 517 | Reproducible builds, independently verifiable | O | S |
| 518 | TUF metadata for the repository | O+U | S |
| 519 | Anti-downgrade enforcement | O | S |
| 520 | Secure Boot chain from firmware to userspace | O+U | S |
| 521 | Measured boot with TPM PCRs | O+U | S |
| 522 | Encrypted secrets store | O+U | T |
| 523 | Hardware-backed keys where a TPM exists | O+U | T |
| 524 | Developer mode, explicitly enabled, never default | O | T |
| 525 | Developer mode visibly indicated while active | O | T |
| 526 | Plugin isolation: separate identity and signature | O | S |
| 527 | High-risk plugins out of process | O | S |
| 528 | Security-relevant events surfaced to the user | O | S |

## E3. Defender (18)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 529 | On-access scanning via fanotify permission events | O | S |
| 530 | Verdict cache keyed on hash and verity state | O | S |
| 531 | Hard timeout, fail-open with a logged warning | O | S |
| 532 | Hash denylist | O | S |
| 533 | YARA rule engine | O+U | S |
| 534 | Behavioural monitoring via BPF-LSM | O | S |
| 535 | Ransomware signal: mass file rename or encrypt | O | S |
| 536 | Unexpected outbound connection signal | O | S |
| 537 | Scoring with evidence, not silent killing | O | S |
| 538 | Encrypted quarantine with restore | O | S |
| 539 | Never delete silently; every action reversible | O | S |
| 540 | On-demand full scan (ClamAV, started then stopped) | O+U | S |
| 541 | Scheduled scan | O | T |
| 542 | Exclusion list | O | T |
| 543 | False-positive reporting path | O | S |
| 544 | Allowlist by verity hash | O | S |
| 545 | Resident memory hard-capped at 60 MB | O | S |
| 546 | Honest UI wording about detection limits | O | S |

---

# PART F — APPLICATIONS

## F1. Built-in applications (44)

| # | Application | Owner | Status |
|---:|---|:---:|:---:|
| 547 | Files (file manager) | O | T |
| 548 | Settings | O | T |
| 549 | Terminal, GPU-rendered, full Unicode | O | T |
| 550 | Text editor | O | T |
| 551 | Litric Browser (our UI on an existing engine) | O+U | **P** |
| 552 | Mail client | O | T |
| 553 | Calendar | O | T |
| 554 | Contacts | O | T |
| 555 | Photos / image viewer | O | T |
| 556 | Photo editor (crop, rotate, adjust) | O | T |
| 557 | Video player | O+U | T |
| 558 | Music player | O | T |
| 559 | Camera application | O | T |
| 560 | Screenshot tool (region, window, full screen) | O | T |
| 561 | Screen recorder | O | T |
| 562 | Calculator (basic, scientific, programmer) | O | T |
| 563 | Clock (alarm, timer, stopwatch, world clock) | O | T |
| 564 | Weather | O | T |
| 565 | Maps | O | T |
| 566 | Notes | O | T |
| 567 | PDF viewer with annotation | O+U | T |
| 568 | Document viewer (office formats, read-only) | O+U | T |
| 569 | Archive manager | O+U | T |
| 570 | Disk utility (partition, format, SMART) | O+U | T |
| 571 | System monitor (CPU, memory, disk, network, per-process) | O | **P** |
| 572 | Task manager with end-task | O | T |
| 573 | Log viewer | O | T |
| 574 | Device manager | O | T |
| 575 | Software store / app manager | O | T |
| 576 | Update manager UI | O | T |
| 577 | Backup and restore | O | T |
| 578 | Character / emoji picker | O | T |
| 579 | Colour picker | O | T |
| 580 | Magnifier | O | T |
| 581 | On-screen keyboard | O | T |
| 582 | Voice recorder | O | T |
| 583 | Remote desktop client | O+U | T |
| 584 | Remote desktop server | O+U | T |
| 585 | Network diagnostics tool | O | T |
| 586 | Font viewer and manager | O | T |
| 587 | Printer queue and management | O+U | T |
| 588 | Scanner application | O+U | T |
| 589 | Bluetooth file transfer UI | O+U | T |
| 590 | Help and documentation browser | O | T |

## F2. Application platform (26)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 591 | Native package format (`.ilpkg` / `.ilapp`) | O | T |
| 592 | Native executable format and loader | O | T |
| 593 | Manifest with declared capabilities | O | **P** |
| 594 | Install, showing capabilities before consent | O | T |
| 595 | Atomic install with rollback on failure | O | T |
| 596 | Update per application | O | T |
| 597 | Rollback per application to a previous version | O | T |
| 598 | Uninstall leaving no orphaned state | O | T |
| 599 | Dependency resolution | O | T |
| 600 | Content-addressed runtime layers, deduplicated | O | T |
| 601 | Flatpak support | U | S |
| 602 | XDG portals wired to the capability broker | O+U | T |
| 603 | AppImage runnable, but not the platform | U | T |
| 604 | Application store front end | O | T |
| 605 | Store: search, categories, screenshots, ratings | O | T |
| 606 | Store: per-app capability disclosure before install | O | T |
| 607 | Store: verified publisher badges | O | T |
| 608 | Store: update channels (stable, beta) | O | T |
| 609 | Web applications installable as first-class apps | O | T |
| 610 | Application lifecycle: suspend and resume | O | T |
| 611 | Application crash isolation | O | S |
| 612 | Application crash reporting with user consent | O | T |
| 613 | Background service registration and policy | O | T |
| 614 | File type association registry | O | T |
| 615 | URL scheme registry | O | T |
| 616 | Share sheet between applications | O | T |

## F3. Windows compatibility (30)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 617 | PE inspection: architecture, subsystem, .NET, hardening | O | **P** |
| 618 | Reject unrunnable binaries with a specific reason | O | **P** |
| 619 | Installer family detection | O | **P** |
| 620 | Unattended install with the correct silent switches | O | **P** |
| 621 | Never guess a silent switch for an unknown installer | O | **P** |
| 622 | Per-application bottle | O | **P** |
| 623 | Bottle lifecycle: create, suspend, reset, repair, delete | O | **P** |
| 624 | Bottle isolation policy, deny by default | O | **P** |
| 625 | Install planning with ordered, describable steps | O | **P** |
| 626 | Bottle execution daemon (`litric-compatd`) | O | T |
| 627 | Double-click a `.exe` and it installs | O | T |
| 628 | Installed Windows apps appear in the launcher | O | T |
| 629 | Windows apps uninstall cleanly | O | T |
| 630 | MSI via msiexec | O+U | T |
| 631 | .NET Framework staging | O+U | T |
| 632 | Visual C++ redistributable staging | O+U | T |
| 633 | Font staging so text is not boxes | O+U | T |
| 634 | Registry virtualisation, per bottle | U | T |
| 635 | Path virtualisation (`C:\` mapped per bottle) | U | T |
| 636 | DirectX to Vulkan translation | U | S |
| 637 | Windows app windows integrate with our window manager | O | T |
| 638 | Windows app file dialogs route through our portal | O | T |
| 639 | Windows apps get no network by default | O | **P** |
| 640 | Windows apps get no real home directory by default | O | **P** |
| 641 | Per-bottle Windows version presentation | O | **P** |
| 642 | Bottle disk usage shown in Settings | O | T |
| 643 | Import an existing Wine prefix | O | T |
| 644 | Compatibility report per application | O | T |
| 645 | Windows binaries treated as untrusted regardless of publisher | O | S |
| 646 | Native version preferred over Windows version when both exist | O | S |

---

# PART G — INPUT, ACCESSIBILITY, LOCALISATION

## G1. Input and text (24)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 647 | Keyboard layout selection | O+U | T |
| 648 | Multiple layouts with fast switching | O+U | T |
| 649 | Per-window layout memory | O | T |
| 650 | Compose key and dead keys | U | T |
| 651 | Input method framework (`text-input-v3`) | O | T |
| 652 | Thai input and word breaking | O+U | T |
| 653 | Chinese, Japanese, Korean input methods | U | T |
| 654 | Indic input methods | U | T |
| 655 | Right-to-left text and UI mirroring | O | T |
| 656 | Bidirectional text rendering | O+U | T |
| 657 | Complex script shaping | U | T |
| 658 | Emoji picker with search | O | T |
| 659 | Spell checking, system-wide | O+U | T |
| 660 | Autocorrect and text replacement | O | T |
| 661 | Clipboard history | O | T |
| 662 | Clipboard: image, rich text, file list | O | T |
| 663 | Global keyboard shortcuts, user-remappable | O | **P** |
| 664 | Per-application shortcut overrides | O | T |
| 665 | Shortcut conflict detection | O | **P** |
| 666 | Trackpad gesture customisation | O | T |
| 667 | Touchscreen gestures | O | T |
| 668 | Stylus input with pressure and tilt | U | T |
| 669 | Dictation / speech to text | O+U | T |
| 670 | Text to speech | O+U | T |

## G2. Accessibility (26)

Treated as gate items, not wishes (Phase 06 §5).

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 671 | Screen reader | O+U | T |
| 672 | Accessibility tree exposed from our scene graph (AT-SPI) | O | T |
| 673 | Keyboard navigation for every action | O | S |
| 674 | Visible focus indicator | O | **P** |
| 675 | Screen magnifier with follow-focus | O | T |
| 676 | Text size scaling | O | T |
| 677 | UI scaling independent of text | O | T |
| 678 | High-contrast theme | O | **P** |
| 679 | Colour filters for colour blindness | O | T |
| 680 | Invert colours | O | T |
| 681 | Reduce transparency | O | T |
| 682 | Reduce motion | O | S |
| 683 | Cursor size and high-visibility cursor | O | T |
| 684 | Sticky keys | O+U | T |
| 685 | Slow keys | O+U | T |
| 686 | Bounce keys | O+U | T |
| 687 | Mouse keys | O+U | T |
| 688 | Click assist / dwell click | O | T |
| 689 | Double-click speed adjustment | O | T |
| 690 | Visual bell instead of audible | O | T |
| 691 | Mono audio | O+U | T |
| 692 | Closed captions styling | O | T |
| 693 | Switch access | O | T |
| 694 | Eye tracking support | O | T |
| 695 | Accessibility quick-toggle panel | O | T |
| 696 | Accessibility settings survive login and reboot | O | T |

## G3. Localisation (12)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 697 | UI translation framework | O | T |
| 698 | Language selection at first run | O | T |
| 699 | Language change without reinstall | O | T |
| 700 | Per-user language | O | T |
| 701 | Region-specific formats independent of language | O | **P** |
| 702 | Translated system messages and errors | O | T |
| 703 | Locale-aware sorting and collation | O | T |
| 704 | Locale-aware date, time, number, currency | O | **P** |
| 705 | Measurement units | O | T |
| 706 | First day of week | O | T |
| 707 | Translation contribution workflow | O | T |
| 708 | Thai, English, and CJK verified at first release | O | T |

---

# PART H — GAMING, AI, UPDATE, DEVELOPER

## H1. Gaming (32)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 709 | Game Mode toggle changing real system state | O | S |
| 710 | Separate controller-driven game session | O | T |
| 711 | gamescope micro-compositor integration | U | S |
| 712 | Direct scanout, fewer copies | U | S |
| 713 | HDR output in games | U | S |
| 714 | VRR / adaptive sync | U | S |
| 715 | Per-game FPS cap, with the latency trade-off stated | O+U | S |
| 716 | FSR upscaling | U | S |
| 717 | NIS upscaling | U | S |
| 718 | Integer scaling | U | S |
| 719 | Resolution spoofing | U | S |
| 720 | Tearing control | O+U | S |
| 721 | CPU governor to performance under Game Mode | O+U | S |
| 722 | GPU performance level forced high | O+U | S |
| 723 | IO priority raised for the game | O+U | S |
| 724 | Notification suppression during play | O | S |
| 725 | `scx_lavd` scheduler, switchable live | U | S |
| 726 | Scheduler fallback to EEVDF if the swap fails | O+U | S |
| 727 | `ntsync` enabled | U | S |
| 728 | Proton with DXVK and VKD3D | U | S |
| 729 | Shader pre-caching (Fossilize) | U | S |
| 730 | Shader cache as a managed system path with a budget | O | S |
| 731 | Per-title profiles: Proton, scheduler, upscaler, knobs | O | S |
| 732 | Profile import and export | O | T |
| 733 | In-game performance overlay | U | S |
| 734 | Frame-time and 1% low capture | O+U | S |
| 735 | Game Mode off restores every value exactly | O | S |
| 736 | Watchdog restores defaults if the daemon dies | O | S |
| 737 | Controller detection, remapping, calibration | O+U | T |
| 738 | Suspend and resume a running game | U | T |
| 739 | Per-title anti-cheat compatibility status, honest | O | S |
| 740 | Game library scanning across launchers | O | T |

## H2. AI (18)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 741 | Local inference runtime | U | S |
| 742 | Model management: download, verify, remove | O | T |
| 743 | Models content-addressed and signature-verified | O | S |
| 744 | Capability/model router with logged decisions | O | S |
| 745 | Route by privacy class of the request | O | S |
| 746 | Route by free memory and battery state | O | S |
| 747 | Model unloaded when idle, to hold the RAM budget | O | S |
| 748 | `ai.local` and `ai.cloud` as separate capabilities | O | **P** |
| 749 | Cloud calls impossible without `ai.cloud` | O | S |
| 750 | No ambient file access for AI | O | S |
| 751 | **No silent screen capture, ever** | O | S |
| 752 | Speech to text | U | S |
| 753 | Text to speech | U | S |
| 754 | Wake word, off by default | U | S |
| 755 | Microphone indicator during wake-word listening | O | S |
| 756 | User-readable audit log of every AI request | O | S |
| 757 | Audit retention configurable, one-click purge | O | S |
| 758 | NPU acceleration where genuinely available, silent CPU fallback | O+U | S |

## H3. Update and recovery (22)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 759 | Background download of updates | O+U | S |
| 760 | A/B slot write and verify | O+U | S |
| 761 | Activation on next boot, single reboot | O+U | S |
| 762 | Boot counting with automatic rollback | U | S |
| 763 | Health checks before commit | O | S |
| 764 | Boot-time regression guard | O | S |
| 765 | Pre-update snapshot of user data | O | S |
| 766 | "Undo update" for 7 days | O | S |
| 767 | Signature verification before writing anything | O | S |
| 768 | Anti-downgrade via monotonic counter | O | S |
| 769 | Power-cut safe at every stage | O | S |
| 770 | ESP free-space check before staging | O | S |
| 771 | Never interrupt a game or fullscreen application | O | S |
| 772 | Active hours / deferral policy | O | T |
| 773 | Changelog and download size shown before applying | O | S |
| 774 | Update history with rollback per entry | O | T |
| 775 | Per-application updates, independent of the system | O | T |
| 776 | Firmware updates through the same discipline | O+U | T |
| 777 | Recovery environment bootable from the ESP | O | S |
| 778 | Recovery: verify and repair filesystems | O | S |
| 779 | Recovery: roll back to the other slot | O | S |
| 780 | Recovery: re-flash a slot from removable media | O | S |

## H4. Developer platform (24)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 781 | Native SDK with stable headers and contracts | O | T |
| 782 | C bindings | O | T |
| 783 | Rust bindings | O | T |
| 784 | Scripting language bindings | O | T |
| 785 | `hello-litric` template that builds and runs sandboxed | O | T |
| 786 | Build tool (`ilbuild`) | O | T |
| 787 | Package builder and signer | O | T |
| 788 | Package validator | O | T |
| 789 | ABI compatibility checker | O | T |
| 790 | Debugger | O+U | T |
| 791 | Profiler (CPU) | O+U | T |
| 792 | Profiler (GPU) | O+U | T |
| 793 | Frame-time analyser | O | T |
| 794 | Audio latency diagnostics | O | T |
| 795 | IPC tracing | O | T |
| 796 | Filesystem access tracing | O | T |
| 797 | Crash analyser with symbolication | O | T |
| 798 | UI toolkit and designer for native apps | O | T |
| 799 | API documentation generator | O | T |
| 800 | Test environment / emulator | O | T |
| 801 | Reproducible package builds | O | T |
| 802 | Developer mode with explicit, indicated relaxations | O | T |
| 803 | Native/compatibility overhead attribution in the profiler | O | T |
| 804 | Sandbox escape regression test suite | O | T |

---

# PART I — FIRST RUN, ENTERPRISE, DIFFERENTIATORS

## I1. First-run experience (16)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 805 | Welcome screen | O | T |
| 806 | Language selection, before anything else | O | T |
| 807 | Region selection | O | T |
| 808 | Keyboard layout selection with a test field | O | T |
| 809 | Accessibility options available *during* setup | O | T |
| 810 | Network connection during setup | O | T |
| 811 | Time zone, automatic where possible | O | T |
| 812 | User account creation | O | T |
| 813 | Password or biometric enrolment | O | T |
| 814 | Disk encryption choice with recovery key display | O | T |
| 815 | Privacy choices, all defaulting to the private option | O | T |
| 816 | Theme and accent choice | O | T |
| 817 | Optional data import from another system | O | T |
| 818 | Suggested applications, none pre-installed without consent | O | T |
| 819 | Quick tour of the interaction model | O | T |
| 820 | Setup resumable if interrupted | O | T |

## I2. Multi-user and enterprise (14)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 821 | Multiple local users with separate data | O+U | T |
| 822 | Fast user switching | O+U | T |
| 823 | Per-user encryption keys | O+U | T |
| 824 | Administrator elevation prompt | O | T |
| 825 | Policy management for managed fleets | O | T |
| 826 | Remote enrolment | O | T |
| 827 | Directory integration (LDAP/Kerberos) | U | T |
| 828 | Remote desktop / remote support | O+U | T |
| 829 | Centralised update control | O | T |
| 830 | Disk encryption key escrow | O | T |
| 831 | Kiosk / single-application mode | O | T |
| 832 | Screen time and application limits | O | T |
| 833 | Content filtering | O | T |
| 834 | Audit log export | O | T |

## I3. Litric-only differentiators (20)

Parity plus strengths still only makes a good OS. These are the reasons to
switch, and none of them is a feature a competitor can add without rebuilding.

| # | Feature | Why nobody else has it | Status |
|---:|---|---|:---:|
| 835 | ≤320 MB full-desktop idle, enforced in CI | Everyone else carries a legacy stack | S |
| 836 | ≤0.5% idle CPU as a hard gate | Nobody gates on it, so nobody optimises it | S |
| 837 | ≤25 wakeups/sec as a hard gate | Same | S |
| 838 | ≤2 MB idle disk writes per 10 min | Same | S |
| 839 | Compositor and shell in one process, no toolkit | The 250 MB saving that makes 835 reachable | **P** |
| 840 | Zero-daemon idle: everything socket-activated | Requires designing for it from the start | S |
| 841 | Every performance claim carries its command and variance | Vendors publish numbers, not methods | S |
| 842 | RAM budget failure blocks the build | Requires wanting to be told no | S |
| 843 | `.exe` double-click → installed, isolated, uninstallable app | Wine exists; nobody made it an OS subsystem | **P** |
| 844 | Per-app disposable Windows environment | A shared prefix cannot be uninstalled | **P** |
| 845 | One capability broker for every permission decision | Windows and Linux each have several | **P** |
| 846 | Runtime revocation effective on a running app | Windows needs a restart; Linux mostly cannot | **P** |
| 847 | Every permission decision auditable by the user | — | **P** |
| 848 | Scoped grants proof against path traversal | Most systems grant whole directories | **P** |
| 849 | Defender ≤60 MB, sandbox-first, honest about limits | Commercial AV is 500 MB–1 GB | S |
| 850 | Update survivable at any power-cut instant | Windows update famously does not | S |
| 851 | On-device AI with no ambient access, no silent capture | Recall did the opposite and was withdrawn | S |
| 852 | Expert tier: every knob, measured cost, revert button | Windows hides them; Linux scatters them | S |
| 853 | Reproducible builds anyone can verify bit-for-bit | No consumer OS offers this | S |
| 854 | Honest per-title anti-cheat status, never implied support | Vendors imply; we state | S |

---

# PART J — RESOURCE, POWER, THERMAL, ENVIRONMENT AND VALIDATION ENGINE

From the LITRIC Unified Engine specification. The governing principle is that
LitricOS does not maximise performance at all times, nor minimise power at all
times: it selects the correct state for the current workload, and can explain
why it chose it.

```
OBSERVE → CLASSIFY → PREDICT → PLAN → AUTHORIZE → ALLOCATE
        → APPLY → MEASURE → VERIFY → ADJUST → REPEAT
```

## J1. Multi-environment platform (32)

LitricOS is the host; Windows and Linux are KVM guests. See
[ADR-0015](adr/0015-multi-environment-switching.md), which also records the
constraints — single-GPU performance, Windows licensing, memory — that decide
what may be promised here.

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 855 | LitricOS boots as host and hypervisor | O+U | S |
| 856 | LITRIC login screen at power-on | O | T |
| 857 | Password requirement, user-toggleable in Settings | O | T |
| 858 | Setting states plainly what turning it off means | O | T |
| 859 | Auto-enter LitricOS by default | O | T |
| 860 | Environment model: states, health, budgets | O | **P** |
| 861 | Lifecycle transitions validated, illegal ones refused | O | **P** |
| 862 | **Host can never be suspended, hibernated or stopped** | O | **P** |
| 863 | Host CPU/memory reservation enforced against guests | O | **P** |
| 864 | Guest budgets computed against each other, not independently | O | **P** |
| 865 | Hibernate frees memory; suspend does not | O | **P** |
| 866 | Switch reports mechanism and latency class, never just "ok" | O | **P** |
| 867 | Compromised environment cannot switch silently | O | **P** |
| 868 | Losing the active environment falls back to the host | O | **P** |
| 869 | Failed environment is reset, never resumed | O | **P** |
| 870 | Hotkey binding model with conflict detection | O | **P** |
| 871 | `Win+1` / `Win+2` / `Win+3` default bindings | O | **P** |
| 872 | Bindings remappable | O | **P** |
| 873 | Reserved escape binding always reaches the host | O | **P** |
| 874 | Host grabs the switch chord before the guest sees it | O | T |
| 875 | Windows marked non-distributable; user supplies media | O | **P** |
| 876 | `litric-envd`: VM lifecycle execution | O | T |
| 877 | Windows 11 guest: TPM 2.0 via swtpm | O+U | S |
| 878 | Windows 11 guest: Secure Boot via OVMF secboot | O+U | S |
| 879 | Linux guest: download, verify signature, install | O | T |
| 880 | Guest surface compositing in `litric-shell` | O | T |
| 881 | Switch = surface swap when target is running (~1 frame) | O | T |
| 882 | GPU passthrough (VFIO) where a second GPU exists | U | T |
| 883 | virtio-gpu Venus paravirtual Vulkan | U | T |
| 884 | Honest statement of single-GPU guest limits in the UI | O | T |
| 885 | Guest disks owned by the host, never raw disk access | O | S |
| 886 | Guests cannot modify LitricOS boot configuration | O | S |

## J2. Resource manager and power domains (26)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 887 | Unified Resource Manager across all subsystems | O | T |
| 888 | HAL / platform abstraction over firmware and ACPI | O | T |
| 889 | Power domain: CPU package | O | T |
| 890 | Power domain: CPU cores | O | T |
| 891 | Power domain: GPU | O | T |
| 892 | Power domain: GPU engines | O | T |
| 893 | Power domain: VRAM | O | T |
| 894 | Power domain: system RAM | O | T |
| 895 | Power domain: storage devices | O | T |
| 896 | Power domain: Ethernet | O | T |
| 897 | Power domain: Wi-Fi | O | T |
| 898 | Power domain: Bluetooth | O | T |
| 899 | Power domain: display | O | T |
| 900 | Power domain: audio | O | T |
| 901 | Power domain: USB | O | T |
| 902 | Power domain: PCIe devices | O | T |
| 903 | Power domain: virtualized environments | O | T |
| 904 | Per-domain: current / requested / effective state | O | T |
| 905 | Per-domain: utilization, power estimate, frequency | O | T |
| 906 | Per-domain: temperature, latency, performance level | O | T |
| 907 | Per-domain: min safe, max allowed, policy, owner | O | T |
| 908 | Per-domain: dependencies, transition state, errors | O | T |
| 909 | **Unavailable capability reports UNKNOWN, never a fabricated value** | O | S |
| 910 | Abstract performance states P0–P6 | O | T |
| 911 | P-state abstraction maps onto real platform capability | O | T |
| 912 | No direct manipulation of undocumented registers from user space | O | S |

## J3. CPU management (22)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 913 | Dynamic detection of cores and logical processors | O | T |
| 914 | **Never hard-code a maximum core count** | O | S |
| 915 | Distinguish cores / logical processors / software threads | O | S |
| 916 | CPU affinity control | O+U | T |
| 917 | Process and thread priority | O+U | T |
| 918 | Scheduling class selection | O+U | T |
| 919 | CPU performance preference (EPP) | O+U | T |
| 920 | Boost policy | O+U | T |
| 921 | Idle policy | O+U | T |
| 922 | Core parking / online state | O+U | T |
| 923 | Utilization and frequency measurement | O | T |
| 924 | CPU bandwidth / rate limits per cgroup | O+U | T |
| 925 | Per-application CPU policy | O | T |
| 926 | Per-environment CPU policy | O | T |
| 927 | Dynamic reallocation as environments become CPU-bound | O | T |
| 928 | Inactive environment deprioritised or suspended by policy | O | **P** |
| 929 | Control plane retains guaranteed minimum capacity | O | **P** |
| 930 | Split-lock detection policy | O+U | S |
| 931 | Core isolation and pinning | O | S |
| 932 | scx scheduler selection, live and reversible | U | S |
| 933 | Scheduler fallback if a live swap fails | O+U | S |
| 934 | Honest claim boundary: cannot universally prevent thread creation | O | S |

## J4. Memory management (22)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 935 | Memory pressure states: LOW / MEDIUM / HIGH / CRITICAL | O | T |
| 936 | Track total, used, available, cached | O | T |
| 937 | Track anonymous, file-backed, compressed | O | T |
| 938 | Track working sets per process | O | T |
| 939 | Track per-environment memory usage | O | **P** |
| 940 | Track swap / zram usage and ratio | O | T |
| 941 | Track fragmentation and allocation failures | O | T |
| 942 | MEDIUM: reclaim caches and inactive memory | O | T |
| 943 | HIGH: compress and reclaim safe inactive memory | O | T |
| 944 | CRITICAL: suspend or hibernate inactive environments | O | T |
| 945 | **Never blindly reclaim active user memory** | O | S |
| 946 | **No memory operation may silently corrupt user data** | O | S |
| 947 | Demand-based memory sharing between environments | O | **P** |
| 948 | Rebalance when an environment becomes memory-heavy | O | T |
| 949 | Allocation observable and attributable | O | T |
| 950 | MGLRU enabled and verified live | O+U | **D** |
| 951 | zram with selectable algorithm | O+U | S |
| 952 | Transparent huge pages, madvise-only | O | S |
| 953 | systemd-oomd PSI policy per slice | O+U | S |
| 954 | Deterministic handling of allocation failure | O | T |
| 955 | Zero use-after-free / double-free / OOB in our code | O | S |
| 956 | Unbounded kernel heap growth detection | O | T |

## J5. GPU and VRAM (24)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 957 | GPU utilization telemetry | O+U | T |
| 958 | Per-engine utilization (graphics, compute, decode, encode) | O+U | T |
| 959 | VRAM usage and pressure tracking | O+U | T |
| 960 | Memory bandwidth, clocks, power, temperature | O+U | T |
| 961 | Queue depth and context activity | O+U | T |
| 962 | Desktop idle → efficient GPU state | O | T |
| 963 | Video playback → hardware decode path | O | T |
| 964 | Gaming → high or maximum performance | O | T |
| 965 | AI → compute-focused state | O | T |
| 966 | Rendering → throughput mode | O | T |
| 967 | GPU selection on multi-GPU systems | O+U | T |
| 968 | GPU priority per application | O | T |
| 969 | Power vs performance preference | O+U | T |
| 970 | Per-environment GPU policy | O | T |
| 971 | VRAM treated as distinct from system RAM | O | T |
| 972 | Track dedicated vs shared GPU memory | O+U | T |
| 973 | Track committed vs resident memory | O+U | T |
| 974 | Track eviction and allocation failure | O+U | T |
| 975 | **Never fake a hard VRAM limit that the platform cannot enforce** | O | S |
| 976 | Report `UNSUPPORTED_HARD_LIMIT` and keep monitoring | O | S |
| 977 | Pressure response by workload priority | O | T |
| 978 | GPU driver hang → compositor restart without session loss | O | S |
| 979 | Explicit sync (`linux-drm-syncobj-v1`) | O | S |
| 980 | HDR and colour management | O+U | S |

## J6. Power profiles and the adaptive engine (22)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 981 | Profile: ECO | O | T |
| 982 | Profile: BALANCED | O | T |
| 983 | Profile: PERFORMANCE | O | T |
| 984 | Profile: HIGH PERFORMANCE | O | T |
| 985 | Profile: MAXIMUM PERFORMANCE | O | T |
| 986 | Profile: ADAPTIVE | O | T |
| 987 | Profile: CUSTOM | O | T |
| 988 | Profiles affect the whole platform, not only CPU | O | T |
| 989 | **Never exceed hardware safety limits to satisfy a profile** | O | S |
| 990 | Closed-loop governor over workload signals | O | T |
| 991 | Input: user activity, foreground app, app category | O | T |
| 992 | Input: CPU, GPU, RAM, storage, network, display activity | O | T |
| 993 | Input: battery and thermal state | O | T |
| 994 | Input: environment state and system priorities | O | **P** |
| 995 | Output: CPU, GPU, memory, display, storage, network policy | O | T |
| 996 | Output: background priority and environment state | O | T |
| 997 | **Contextual classification, not "5 minutes idle = lower CPU"** | O | S |
| 998 | Race-to-idle evaluated by measured energy, not assumption | O | T |
| 999 | Energy = Power × Time, measured not inferred | O | T |
| 1000 | Power budgeting across domains with reserve | O | T |
| 1001 | Respect firmware, package, charger, battery, thermal limits | O | S |
| 1002 | **No assumption of watt-for-watt transfer between domains** | O | S |

## J7. Adaptive idle and AFK behaviour (20)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 1003 | States: ACTIVE / IDLE / DEEP_IDLE / SLEEP | O | **P** |
| 1004 | ACTIVE: native compositor frame rate | O | **P** |
| 1005 | IDLE: compositor target 30 FPS | O | **P** |
| 1006 | DEEP_IDLE: 5–15 FPS where visually safe | O | T |
| 1007 | SLEEP: OS sleep policy | O | T |
| 1008 | **Only rendering slows — never the whole OS clock** | O | S |
| 1009 | Input remains full rate in every idle state | O | **P** |
| 1010 | Audio remains full rate in every idle state | O | S |
| 1011 | System timers and network unaffected | O | S |
| 1012 | Activity detection: keyboard, mouse, touch, controller | O | **P** |
| 1013 | Activity detection: fullscreen, gaming, media playback | O | T |
| 1014 | Activity detection: recording, downloads, presentations | O | T |
| 1015 | Fullscreen video does not enter rendering reduction | O | T |
| 1016 | Wake on any meaningful interaction | O | T |
| 1017 | Wake restores compositor target and GPU/CPU policy | O | **P** |
| 1018 | Wake verifies frame delivery before continuing | O | T |
| 1019 | Wake does not restart applications | O | S |
| 1020 | All idle timings user-configurable | O | T |
| 1021 | Reduced-motion and accessibility respected in idle | O | T |
| 1022 | Adaptive-idle acceptance test with artifact | O | T |

## J8. Thermal engine (15)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 1023 | CPU temperature tracking | O+U | T |
| 1024 | GPU temperature tracking | O+U | T |
| 1025 | Per-device temperature tracking | O+U | T |
| 1026 | Thermal headroom computation | O | T |
| 1027 | Fan speed observation | O+U | T |
| 1028 | Throttling detection | O+U | T |
| 1029 | Power limit observation | O+U | T |
| 1030 | Recent thermal history | O | T |
| 1031 | Large headroom → performance may increase | O | T |
| 1032 | Little headroom → performance reduced | O | T |
| 1033 | **Thermal protection overrides performance preference** | O | S |
| 1034 | **Never sacrifice hardware safety for a benchmark** | O | S |
| 1035 | Sustained-load stability measurement (minute 1 vs 30) | O | T |
| 1036 | Thermal profile / target where the platform permits | O | T |
| 1037 | Thermal state exposed in observability output | O | T |

## J9. Per-subsystem power (26)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 1038 | Network: efficient state when idle | O+U | T |
| 1039 | Network: performance mode under heavy transfer | O+U | T |
| 1040 | Network: preserve required wake capability in sleep | O+U | T |
| 1041 | **Never drop a required connection merely because the desktop is idle** | O | S |
| 1042 | Network: busy-poll and coalescing as measured knobs | O | S |
| 1043 | Storage: IOPS, throughput, latency, queue depth | O | T |
| 1044 | Storage: active/idle time and power state | O+U | T |
| 1045 | Storage: read/write amplification | O | T |
| 1046 | Storage: latency priority during game loading | O | T |
| 1047 | Storage: throughput mode for large sequential I/O | O | T |
| 1048 | **Never trade filesystem integrity for power** | O | S |
| 1049 | Display: brightness control | O+U | T |
| 1050 | Display: refresh rate adaptation | O | T |
| 1051 | Display: VRR control | O+U | T |
| 1052 | Display: HDR control | O+U | T |
| 1053 | Display: external monitor policy | O | T |
| 1054 | Display: panel power and idle policy | O | T |
| 1055 | Display: no visible instability from adaptation | O | S |
| 1056 | Audio: low power when nothing is playing | O+U | T |
| 1057 | Audio: efficient state during normal playback | O+U | T |
| 1058 | Audio: latency-priority policy on demand | O+U | T |
| 1059 | **Real-time audio path: no allocation, no blocking locks** | O | S |
| 1060 | USB: ACTIVE / IDLE / SUSPENDED / WAKE_CAPABLE states | O+U | T |
| 1061 | **Never suspend input hardware needed for wake** | O | S |
| 1062 | Peripheral wake capability tracking | O+U | T |
| 1063 | Hardware capability discovery at startup, never hard-coded | O | T |

## J10. Security, AI policy, transactions, observability (24)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 1064 | Every resource request passes the capability layer | O | **P** |
| 1065 | Untrusted applications get no unlimited hardware control | O | **P** |
| 1066 | **Power management cannot bypass security** | O | S |
| 1067 | **AI cannot bypass security** | O | S |
| 1068 | **Environment managers cannot bypass security** | O | S |
| 1069 | AI expresses policy intent, never touches hardware | O | S |
| 1070 | Natural language → intent → validated policy → permission check | O | T |
| 1071 | AI actions explainable, auditable, reversible | O | S |
| 1072 | Resource changes transactional where practical | O | T |
| 1073 | Validate all, then apply; rollback on partial failure | O | T |
| 1074 | **Never leave the system partially applied** | O | S |
| 1075 | Safe high-level user controls, no raw registers | O | S |
| 1076 | Expose current profile and workload classification | O | T |
| 1077 | Expose requested / allocated / effective resource | O | T |
| 1078 | Expose power, thermal, environment and idle state | O | T |
| 1079 | Expose policy decisions and transition reasons | O | T |
| 1080 | **Explain WHY**: "GPU raised to P0 because…" | O | T |
| 1081 | Operation results: SUCCESS / PARTIAL / UNSUPPORTED / DENIED | O | T |
| 1082 | Operation results: TIMEOUT / FAILED / ROLLED_BACK | O | T |
| 1083 | **Never return SUCCESS when nothing happened** | O | S |
| 1084 | Report actual state when the requested state is unreachable | O | S |
| 1085 | Power measurement from hardware where exposed | O+U | T |
| 1086 | **Never invent a measurement; UNKNOWN is the answer** | O | **P** |
| 1087 | Energy-per-task accounting for benchmarks | O | T |

## J11. Validation, artifacts and regression (30)

| # | Feature | Owner | Status |
|---:|---|:---:|:---:|
| 1088 | `artifacts/performance/` tree with per-subsystem directories | O | T |
| 1089 | Machine-readable artifacts (JSON, CSV, LOG) | O | T |
| 1090 | Artifact carries build id, commit, profile, hardware | O | T |
| 1091 | Artifact carries OS/kernel version, test config, workload | O | T |
| 1092 | Artifact carries measurements, baseline, thresholds, result | O | T |
| 1093 | Artifact carries reproducibility information | O | T |
| 1094 | BASELINE / CURRENT / DELTA / THRESHOLD / RESULT comparison | O | T |
| 1095 | **Comparison engine never silently ignores a regression** | O | T |
| 1096 | Boot regression threshold 5% | O | T |
| 1097 | Idle RAM regression threshold 5% | O | T |
| 1098 | Idle CPU regression threshold +0.2pp | O | T |
| 1099 | UI latency regression threshold 10% | O | T |
| 1100 | IPC regression threshold 10% | O | T |
| 1101 | Power regression threshold 5% | O | T |
| 1102 | Critical security regression blocks release immediately | O | S |
| 1103 | Fuzz: syscalls, IPC, filesystem and network parsers | O | T |
| 1104 | Fuzz: configuration, package, update and boot metadata | O | T |
| 1105 | Fuzz: driver, environment and compatibility interfaces | O | T |
| 1106 | Fuzz: malformed images, manifests, device data | O | T |
| 1107 | Fuzz: resource requests, suspend/resume state, policy inputs | O | T |
| 1108 | Every crash produces a reproducible case before being closed | O | S |
| 1109 | 24-hour desktop soak, zero kernel crashes | O | T |
| 1110 | 72-hour release-candidate CI soak | O | T |
| 1111 | 1000 suspend/resume cycles, zero unrecovered failures | O | T |
| 1112 | 1000 app start/close cycles, no leak trend | O | T |
| 1113 | 1000 environment start/suspend/resume cycles | O | T |
| 1114 | **A skipped test reports SKIPPED, never PASSED** | O | **D** |
| 1115 | **A build that was skipped is a FAILED build** | O | **D** |
| 1116 | **Stale artifacts are never reported as newly built** | O | **D** |
| 1117 | Build records commit, profile, toolchain, config and artifact hashes | O | **P** |


---

## K1. Unified application compatibility (13)



One desktop, three application ecosystems. Decided in

[ADR-0020](adr/0020-unified-application-compatibility.md); the

implementable contract is [master-plan/22](master-plan/22-unified-compatibility.md).



| # | Feature | Owner | Status |

|---:|---|:---:|:---:|

| 1118 | Windows `.exe` runtime selection: bottle vs full VM, decided on evidence | O | **P** |

| 1119 | Unified Application Manager selecting the runtime for all three ecosystems automatically | O | S |

| 1120 | Selection surfaced before first launch, with isolation and resource cost explained | O | S |

| 1121 | Per-app windows from every runtime in the one desktop (completes row 880) | O | S |

| 1122 | Cross-environment data-access matrix enforced by the capability broker | O | S |

| 1123 | Bottles: application-scoped directories only; no home, no network by default | O | **P** |

| 1124 | Full environments: user-approved directories only, never platform state | O | S |

| 1125 | Escalation from bottle to full Windows VM only on recorded evidence | O | S |

| 1126 | Linux applications run natively: sandbox profile + portals, no ABI translation layer | O | S |

| 1127 | Removing every compatibility component leaves a fully working OS, verified in CI | O | S |

| 1128 | Per-title game compatibility honesty: anti-cheat status shown, never implied | O | S |

| 1129 | Application Manager decision function is pure, fuzzed, offline-testable | O | **P** |

| 1130 | Runtime provenance never changes compositor treatment of a window | O | S |



## K2. Deep recovery and data protection (19)



Platform authority, generations, the user-data domain and the recovery

decision engine. Decided in [ADR-0021](adr/0021-deep-recovery-data-protection.md);

the implementable contract is [master-plan/23](master-plan/23-deep-recovery.md).



| # | Feature | Owner | Status |

|---:|---|:---:|:---:|

| 1131 | Four-concept separation (live data / snapshot / backup / generation) enforced in storage policy | O | S |

| 1132 | User data is an independent domain: own permissions, encryption, versioning, recovery metadata | O | S |

| 1133 | System rollback never rolls back user data; separate lifecycles, tested | O | S |

| 1134 | Snapshots are bounded and recovery-managed; never a substitute for backup | O | S |

| 1135 | Immutable, integrity-verified backups; local and external/offline targets | O | S |

| 1136 | Restore testing is automated; an untested backup is reported as untested | O | S |

| 1137 | Restore never silently overwrites newer files; a conflict plan is produced | O | S |

| 1138 | Independent recovery environment: separate signed boot entry, minimal toolset | O | S |

| 1139 | Recovery reachable with all three environments broken; no OS required first | O | S |

| 1140 | Recovery never offers "disable verification and continue" as an ordinary path | O | S |

| 1141 | Recovery priority structurally enforced: data, keys, repair, rollback, snapshot, reinstall, destructive | O | S |

| 1142 | Destructive recovery requires explicit multi-step confirmation, every time | O | S |

| 1143 | Recovery decision engine: diagnose before choosing the operation | O | S |

| 1144 | Recovery operations are transactional; partial failure rolls the recovery back | O | S |

| 1145 | Every generation switch records slot, reason and user-data-integrity result | O | S |

| 1146 | Per-environment watchdog heartbeats with containment actions | O | S |

| 1147 | Health-check gate before generation activation: session, storage, critical services | O | S |

| 1148 | Watchdogs never reboot the machine when a single environment can be recovered | O | S |

| 1149 | The recovery environment itself has a watchdog and a removable-media fallback | O | S |



## K3. Litric Defender engine (16)



Layered detection with a pure policy core. Decided in

[ADR-0022](adr/0022-defender-architecture.md); Phase 0 is implemented as

`src/litric-defender` (zero third-party dependencies, std-only).



| # | Feature | Owner | Status |

|---:|---|:---:|:---:|

| 1150 | Defender policy core: verdicts from signals as a pure function, table-tested | O | **P** |

| 1151 | Layered engine: hash, patterns, rules, heuristics, behaviour, on-demand deep scan | O | **P** |

| 1152 | Fail-open with a logged warning on scanner timeout; a hung scanner never blocks the desktop | O | **P** |

| 1153 | Fail-closed reserved for exact denylist hash matches | O | **P** |

| 1154 | Verdict cache keyed (content hash, file class, definition generation) | O | **P** |

| 1155 | Definition generation bump invalidates cached verdicts atomically | O | **P** |

| 1156 | On-access scanning is recursion-proof: the scanner never gates its own I/O, deadlock-tested | O | S |

| 1157 | ClamAV runs in a transient scope; its footprint never counts against the resident budget | O | S |

| 1158 | Ransomware canary files: cheap, low false positives, seconds-scale detection | O | S |

| 1159 | Definition updates: signed, schema-validated, canary rollout, rollback retained | O | S |

| 1160 | Definition parser fuzzed; malformed definitions refused before activation | O | S |

| 1161 | Verdict cache persisted across reboots; unchanged files are not rescanned after boot | O | S |

| 1162 | Defender signals feed the recovery decision engine and mark environments Compromised | O | S |

| 1163 | EICAR and a test corpus detected in CI with real counts reported | O | S |

| 1164 | Scan scheduling: idle-time, low I/O priority, paused on battery | O | S |

| 1165 | ML scoring, if ever added, never sits on the blocking path | O | S |



## THE RULE THIS DOCUMENT EXISTS TO ENFORCE

A row moves to `D` when it is implemented **and** a test proves it, and that test
appears in `docs/EVIDENCE.md`. Not when the code is written. Not when it
compiles. Not when it looked right once.

854 rows at mostly `T` is an honest position. 854 rows claimed as done would not
be, and the difference between those two documents is the whole project.

### Order of work — fixed by the layer rule, not by convenience

```
Layer 1  Kernel + boot          A1, A2          → GATE 01   ← blocked on a build host
Layer 2  Core services          A2, A3, E1      → GATE 02   ← capability engine lives here
Layer 3  System API             C, D, E2        → GATE 03/04
Layer 4  Runtime + SDK          H4, F2
Layer 5  Compositor + shell     B, G            → GATE 05/06
Layer 6  Applications + compat  F1, F3, H1      → GATE 07-12
```

Part B (the macOS-class feel: animation, radii, Quick Look, Mission Control) is
**Layer 5**. It cannot be built before the compositor exists, and building it
earlier means building it twice.
