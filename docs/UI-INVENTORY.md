# UI inventory — every file the desktop is made of, how it is wired, what it configures

Written 2026-10-13, on a machine that had the desktop running in QEMU at the time
(see *Seeing it* at the end). It is a **map**, not a specification: it says which
file holds which decision and which files talk to each other, so a change can be
aimed rather than searched for.

Line counts are the real ones at the time of writing and are there for scale, not
as a budget.

**Updated 2026-10-22:** Mission Control, system-wide search and the window
placement shortcuts were added after this map was written. See
[DESKTOP-2.0.md](DESKTOP-2.0.md) for the capabilities, the keybindings and the
measured gaps.

---

## 1. What is on the screen

| Surface | Where it is drawn | Where its state lives |
|---|---|---|
| Top bar (Apps, **focused application** in the centre, Environment, status, clock, user) | [views.rs](../src/litric-shell/src/views.rs) | [shell.rs](../src/litric-shell/src/shell.rs) (`Overlay`, hotspots) |
| Desktop icon grid | [apps.rs](../src/litric-shell/src/apps.rs) (models), [views.rs](../src/litric-shell/src/views.rs) (pixels) | [shell.rs](../src/litric-shell/src/shell.rs) |
| The dock **and** the launcher it morphs into (one surface, two states) | [views.rs](../src/litric-shell/src/views.rs) `draw_dock` | [shell.rs](../src/litric-shell/src/shell.rs), `SHORTCUT_CHIPS` |
| Windows, focus, z-order | [litric-wm](../src/litric-wm/src/) | [litric-wm](../src/litric-wm/src/manager.rs), mirrored in `Shell` |
| Overlays: switcher, quick settings, notifications, Mission Control | [views.rs](../src/litric-shell/src/views.rs) `draw_overlay` | `Overlay` enum, [shell.rs:206](../src/litric-shell/src/shell.rs#L206) |
| Mission Control (every window, every workspace) | [views.rs](../src/litric-shell/src/views.rs) `draw_overview` | [overview.rs](../src/litric-shell/src/overview.rs) layout, `Overlay::Overview` |
| System-wide search (apps, actions, settings pages, windows) | [views.rs](../src/litric-shell/src/views.rs) `launcher_rows` | [seek.rs](../src/litric-shell/src/seek.rs) |
| Lock screen (Liquid Glass) | [lock.rs](../src/litric-shell/src/lock.rs) | `lock.rs`, verified through `litric-sysd` |
| Eight applications | [apps.rs](../src/litric-shell/src/apps.rs) + [views.rs](../src/litric-shell/src/views.rs) | [shell.rs](../src/litric-shell/src/shell.rs) |
| Context menus (one list, drawn = works) | [views.rs](../src/litric-shell/src/views.rs), [apps.rs:1283](../src/litric-shell/src/apps.rs#L1283) | `MenuItem`/`MenuAction` |

The eight built-in applications are listed once, in launcher order, in
[shortcuts.rs:117](../src/litric-shell/src/shortcuts.rs#L117) (`App::ALL`):
Activity, Environments, Settings, Files, Litric Browser, Terminal, Trash,
Defender — each with a `title()` for the user and an `app_id()` (`com.litric.*`)
for the window manager.

---

## 2. The shell crate — every source file

[src/litric-shell](../src/litric-shell) is the desktop. 34 168 lines of Rust.

| File | Lines | What it decides |
|---|---:|---|
| [src/overview.rs](../src/litric-shell/src/overview.rs) | 147 | Mission Control's card grid and keyboard navigation, tested without a display. |
| [src/seek.rs](../src/litric-shell/src/seek.rs) | 312 | The system-wide search index and its ranking. |
| [src/shell.rs](../src/litric-shell/src/shell.rs) | 4881 | All state and all input consequences. **Deliberately free of drawing**: which window has focus, what a click means, what an action does. |
| [src/views.rs](../src/litric-shell/src/views.rs) | 3751 | Drawing from that state: every function reads `Shell` and writes pixels; none of them decide anything. |
| [src/taskmgr.rs](../src/litric-shell/src/taskmgr.rs) | 3403 | The process table as a model (see [monitor.rs](../src/litric-shell/src/monitor.rs) for why it is a model and not a draw call). |
| [src/canvas.rs](../src/litric-shell/src/canvas.rs) | 2088 | The software raster canvas. Every pixel LitricOS puts on a screen goes through here; there is no GPU in the path. |
| [src/font.rs](../src/litric-shell/src/font.rs) | 2067 | LitricOS Sans — a stroke-outline typeface *drawn*, not loaded. `GLYPHS` at [font.rs:1010](../src/litric-shell/src/font.rs#L1010), lookup at [font.rs:1540](../src/litric-shell/src/font.rs#L1540). |
| [src/lock.rs](../src/litric-shell/src/lock.rs) | 1870 | The lock screen: scrim, Liquid Glass panel, password field, and the rule that a locked frame leaves no pixel of what was underneath. |
| [src/apps.rs](../src/litric-shell/src/apps.rs) | 1751 | The application models: Files, Browser, Defender, the icon grid, terminal buffer, menus. |
| [src/main.rs](../src/litric-shell/src/main.rs) | 1657 | Startup, the event loop, framebuffer discovery, sysd queries, damage bands, the status file. |
| [src/e2e.rs](../src/litric-shell/src/e2e.rs) | 1449 | Journeys driven by synthetic input — key presses, clicks, right-clicks, typed text — the way a user produces them. |
| [src/fsbridge.rs](../src/litric-shell/src/fsbridge.rs) | 1197 | The one bridge to the filesystem, working in three places: a booted LitricOS, the development host, and tests. |
| [src/desktop.rs](../src/litric-shell/src/desktop.rs) | 1193 | The composition itself (wallpaper, bar, window), tokens from `litric-design`, numbers from `litric-sysd`. |
| [src/ui.rs](../src/litric-shell/src/ui.rs) | 1132 | The component layer: one place where a button decides what a button is. |
| [src/shortcuts.rs](../src/litric-shell/src/shortcuts.rs) | 1050 | Global shortcuts as a table (policy as data), and the `App` enum. |
| [src/activity.rs](../src/litric-shell/src/activity.rs) | 661 | The Activity window: meters and the process table, drawn *from* `TaskManager`. |
| [src/icons.rs](../src/litric-shell/src/icons.rs) | 634 | The icon set, described as pen paths like the typeface. |
| [src/monitor.rs](../src/litric-shell/src/monitor.rs) | 588 | What the Activity window draws and why — the honest-numbers layer (`Option`, "Unavailable" instead of a fabricated zero). |
| [src/fb.rs](../src/litric-shell/src/fb.rs) | 582 | The Linux framebuffer, reached without a single `ioctl`. |
| [src/textedit.rs](../src/litric-shell/src/textedit.rs) | 559 | Single-line text editing: the browser address bar, Files' rename/path/search fields. |
| [src/link.rs](../src/litric-shell/src/link.rs) | 543 | The background connection to a socket-activated daemon: the desktop must never block on one. |
| [src/anim.rs](../src/litric-shell/src/anim.rs) | 408 | The single animation integrator — the one place springs are stepped, against `Motion` tokens. |
| [src/domkey.rs](../src/litric-shell/src/domkey.rs) | 332 | Keyboard events named the way the web platform names them (`key`, `code`, `keyCode`). |
| [src/textaudit.rs](../src/litric-shell/src/textaudit.rs) | 297 | Finding text drawn over other text (the "ตัวอักษรทับกัน" complaint, as a check). |
| [src/lib.rs](../src/litric-shell/src/lib.rs) | 55 | Crate root and layer statement. |
| [examples/frame_dump.rs](../src/litric-shell/examples/frame_dump.rs) | 483 | One frame per scene, to a PPM, for looking at. |
| [examples/render_bench.rs](../src/litric-shell/examples/render_bench.rs) | 426 | 15-run medians of full frames, damage frames, blur and present. |
| [tests/design_lint.rs](../src/litric-shell/tests/design_lint.rs) | 270 | The design system's rules, asserted. |
| [tests/overflow_audit.rs](../src/litric-shell/tests/overflow_audit.rs) | 168 | Nothing draws outside what it was given. |
| [tests/font_standards.rs](../src/litric-shell/tests/font_standards.rs) | 309 | Typeface standards, including that every printable ASCII character has a glyph. |
| [tests/text_layout_audit.rs](../src/litric-shell/tests/text_layout_audit.rs) | 364 | Text does not collide with text. |

### What it depends on ([Cargo.toml](../src/litric-shell/Cargo.toml))

`litric-design`, `litric-sysd`, `litric-input`, `litric-wm`, `litric-locale`,
`litric-files`, `litric-defender`, `litric-core` — no third-party GUI toolkit
anywhere. Everything the desktop renders, it renders itself.

### The crates that are the UI's other halves

| Crate | Lines | Role for the UI |
|---|---:|---|
| [litric-design](../src/litric-design) | 1542 | Tokens: colour, motion, spacing, radii. Pure data and pure functions; [examples/tokens.rs](../src/litric-design/examples/tokens.rs) prints them. |
| [litric-input](../src/litric-input) | 3082 | evdev discovery and event decoding, pointer state, key tables. |
| [litric-wm](../src/litric-wm) | 1885 | Window manager: geometry, focus, z-order, damage. |
| [litric-locale](../src/litric-locale) | 1877 | Region detection, clock and date formatting, timezone — what the top bar's clock prints. |
| [litric-sysd](../src/litric-sysd) | 8657 | The numbers and the system actions behind the UI (`facts`, `providers`, `state`, `proto`). |
| [litric-files](../src/litric-files) | 7375 | The Files application's backing store: paths, ops, trash, archives, and its server. |
| [litric-defender](../src/litric-defender) | 1643 | The Defender application's scan engine, cache and verdicts. |
| [litric-browser](../src/litric-browser) | 5626 | Litric Browser: tabs, URL handling, policy, image decoding. |
| [litric-winbridge](../src/litric-winbridge) | 2537 | Windows-compatibility inspection and install planning (the Environments window). |
| [litric-trash](../src/litric-trash) | 700 | Trash, as the Trash window shows it. |
| [litric-caps](../src/litric-caps) | 2307 | Capability checks the UI's actions pass through. |
| [litric-core](../src/litric-core) | 1562 | Ids, errors, handles, percentage arithmetic. |
| [litric-manifest](../src/litric-manifest) | 953 | Application manifests (the app model behind the grid). |
| [litric-env](../src/litric-env) | 1249 | The environment manager behind the Environments window. |

---

## 3. The wiring (การเชื่อม)

### 3.1 Boot

```
kernel → systemd → litric-sysd.service → litric-shell.service → desktop
```

[litric-shell.service](../image/mkosi.extra/usr/lib/systemd/system/litric-shell.service)
is worth reading before touching the shell's environment, because it states its
own scope: `After=`/`Wants=litric-sysd.service`, `Conflicts=getty@tty1.service`
(the shell unbinds fbcon; two programs owning tty1 is a fight neither wins),
`ConditionPathExists=/dev/fb0` (a headless CI boot gets a skip, not a failed
unit), `RuntimeDirectory=litric` with `RuntimeDirectoryPreserve=yes` (otherwise
whichever unit stops first deletes the other's socket), `MemoryMax=96M`,
`TasksMax=32`, `DeviceAllow=/dev/fb0 rw`, `DeviceAllow=char-input r`
(**read-only**: the shell must see input and must never be able to synthesise it),
`RestrictAddressFamilies=AF_UNIX`, `IPAddressDeny=any`.

### 3.2 Pixels

```
litric-sysd numbers ─┐
litric-design tokens ─┼→ shell.rs state → views.rs → ui.rs → canvas.rs → fb.rs → /dev/fb0
litric-input events ─┘                                    (damage bands only)
```

* [fb.rs](../src/litric-shell/src/fb.rs) opens the framebuffer without `ioctl`.
* [canvas.rs](../src/litric-shell/src/canvas.rs) is the only rasteriser.
* [views.rs](../src/litric-shell/src/views.rs) draws and registers hotspots in
  the same pass — a thing that is drawn is a thing that can be clicked. This is
  why the context menu and the shortcut chips cannot drift from their behaviour.
* Only changed rows are written back ([main.rs](../src/litric-shell/src/main.rs)),
  which is where "an idle desktop presents nothing at all" comes from.

### 3.3 Input

[litric-input](../src/litric-input) discovers and decodes devices;
[main.rs](../src/litric-shell/src/main.rs) runs the loop; [shell.rs](../src/litric-shell/src/shell.rs)
decides; [litric-wm](../src/litric-wm/src/manager.rs) owns window geometry and
damage. The `e2e` module feeds the same path with synthetic events, so a journey
in a test is a journey a user could actually perform.

### 3.4 Daemons

| Path | Who listens | Who connects |
|---|---|---|
| `/run/litric/sysd.sock` | [litric-sysd](../src/litric-sysd/src/main.rs) | [main.rs:53](../src/litric-shell/src/main.rs#L53) constant, queries at [main.rs:191](../src/litric-shell/src/main.rs#L191) |
| `/run/litric/browser.sock` | `litric-browser.socket` + `litric-browser@.service` | [shell.rs:1205](../src/litric-shell/src/shell.rs#L1205), over [link.rs](../src/litric-shell/src/link.rs) |
| `/run/litric/shell.status` | the shell writes it | [main.rs:56](../src/litric-shell/src/main.rs#L56) — what a probe reads to know the shell came up |
| `/run/litric/env` | [litric-envd](../src/litric-envd/src/lib.rs) | the Environments window |

The rule [link.rs](../src/litric-shell/src/link.rs) enforces: the desktop never
blocks on a daemon. A browser engine that takes a second to answer must not stop
the clock.

### 3.5 Animation, type, icons, locale

* [anim.rs](../src/litric-shell/src/anim.rs) is the only place a spring is
  stepped; the motion numbers are `litric-design`'s `Motion` tokens.
* [font.rs](../src/litric-shell/src/font.rs) draws the typeface; [icons.rs](../src/litric-shell/src/icons.rs)
  draws the icons the same way. Neither loads a file at runtime.
* [litric-locale](../src/litric-locale) supplies what the clock and date print.

### 3.6 The lock screen

[lock.rs](../src/litric-shell/src/lock.rs) owns the scrim, the glass panel and
the field. Verification goes through `litric-sysd`'s `AUTH` path rather than a
password comparison in the shell — see
[docs/design/wave1/04-authentication.md](design/wave1/04-authentication.md).
`e2e.rs` asserts that a locked frame contains **no pixel** of what was underneath
and that every other surface's hotspots are gone with it.

### 3.7 Checks that exist because something went wrong

* [tests/design_lint.rs](../src/litric-shell/tests/design_lint.rs) — design-system rules.
* [tests/text_layout_audit.rs](../src/litric-shell/tests/text_layout_audit.rs) + [src/textaudit.rs](../src/litric-shell/src/textaudit.rs) — "ตัวอักษรทับกัน".
* [tests/font_standards.rs](../src/litric-shell/tests/font_standards.rs) — typeface standards.
* [tests/overflow_audit.rs](../src/litric-shell/tests/overflow_audit.rs) — nothing draws outside its box.
* [src/e2e.rs](../src/litric-shell/src/e2e.rs) — the journeys.
* [tools/scan-nonascii.py](../tools/scan-nonascii.py) — which characters the shell
  prints that the typeface may not have. **See §6 for a defect in this tool.**

---

## 4. Settings (การตั้งค่า)

### 4.1 In the shell

[shell.rs:647](../src/litric-shell/src/shell.rs#L647) is the whole of quick
settings, and its doc comment is the honest part: *every field here is a
request*.

| Field | Meaning |
|---|---|
| `power` | requested power profile |
| `do_not_disturb` | notifications off |
| `adaptive_idle` | the shell may reduce its frame rate |
| `reduce_transparency` | readability |
| `reduce_motion` | accessibility |
| `applied` | which of the above actually reached the system |

`is_applied(key)` ([shell.rs:681](../src/litric-shell/src/shell.rs#L681)) exists so
a toggle that only records a preference is not presented as though it
reconfigured the machine.

Entry points: the top bar's environment/power areas
([shell.rs:3121](../src/litric-shell/src/shell.rs#L3121)), the user area opens
`App::Settings` ([shell.rs:3131](../src/litric-shell/src/shell.rs#L3131)), and the
desktop's context menu carries `Display settings`
([apps.rs:1283](../src/litric-shell/src/apps.rs#L1283) →
[shell.rs:1552](../src/litric-shell/src/shell.rs#L1552)).

### 4.2 What the machine can actually be asked to do

[litric-sysd](../src/litric-sysd) is where a settings change becomes a system
change:

* [providers.rs](../src/litric-sysd/src/providers.rs) — the four Wave-1 providers
  (network, audio, Bluetooth, power) plus session; a Settings pane can say what
  the machine can do, and one toggle is checked without consent.
* [platform/](../src/litric-sysd/src/platform/) — `net.rs`, `audio.rs`,
  `bluetooth.rs`, `session.rs`: the real backends.
* [state.rs](../src/litric-sysd/src/state.rs), [proto.rs](../src/litric-sysd/src/proto.rs),
  [action.rs](../src/litric-sysd/src/action.rs) — the state machine, the wire
  format, and the permitted mutations.
* [facts.rs](../src/litric-sysd/src/facts.rs) — the numbers the desktop displays.

Design notes for these live in
[docs/design/wave1/](design/wave1/) (network, Bluetooth, audio, power,
authentication) and [ADR 0023](adr/0023-wave1-system-providers.md).

### 4.3 Configuration files that ship in the image

| File | Sets |
|---|---|
| [sysctl.d/10-litric-mm.conf](../image/mkosi.extra/usr/lib/sysctl.d/10-litric-mm.conf) | memory management |
| [sysctl.d/20-litric-net.conf](../image/mkosi.extra/usr/lib/sysctl.d/20-litric-net.conf) | networking |
| [sysctl.d/30-litric-security.conf](../image/mkosi.extra/usr/lib/sysctl.d/30-litric-security.conf) | hardening |
| [sysctl.d/40-litric-sched.conf](../image/mkosi.extra/usr/lib/sysctl.d/40-litric-sched.conf) | scheduler |
| [journald.conf.d/00-litric.conf](../image/mkosi.extra/usr/lib/systemd/journald.conf.d/00-litric.conf) | logging |
| [oomd.conf.d/00-litric.conf](../image/mkosi.extra/usr/lib/systemd/oomd.conf.d/00-litric.conf) | OOM policy |
| [zram-generator.conf](../image/mkosi.extra/usr/lib/systemd/zram-generator.conf), [modules-load.d/litric-zram.conf](../image/mkosi.extra/usr/lib/modules-load.d/litric-zram.conf) | swap |
| [network/20-wired.network](../image/mkosi.extra/usr/lib/systemd/network/20-wired.network) | wired networking |
| [tmpfiles.d/](../image/mkosi.extra/usr/lib/tmpfiles.d/) | `litric-home`, `litric-dns`, `litric-iwd` |
| [sysusers.d/litric.conf](../image/mkosi.extra/usr/lib/sysusers.d/litric.conf) | accounts |
| [litric-shell.slice](../image/mkosi.extra/usr/lib/systemd/system/litric-shell.slice), `litric-app.slice`, `litric-ai.slice`, `litric-game.slice` | cgroup budgets per class of work |
| [selftest](../image/mkosi.extra/usr/lib/litric/selftest) + [litric-selftest.service](../image/mkosi.extra/usr/lib/systemd/system/litric-selftest.service) | what the image proves about itself at boot |

### 4.4 Image profiles and partitions

[image/mkosi.profiles/](../image/mkosi.profiles) (`nano`, `core`, `desktop`,
`gaming`) select what a build contains; `nano` is deliberately not a base for the
others. Partitioning is [image/mkosi.repart/](../image/mkosi.repart) (ESP, root,
verity, verity signature, var). [tools/build.sh](../tools/build.sh) refuses to
build the `desktop` profile without a shell binary, and refuses to build on a
Windows mount at all — both because the resulting image would look like a kernel
fault rather than a build mistake.

### 4.5 Design tokens

[litric-design](../src/litric-design) (`lib.rs`, `color.rs`) is the single source
of colour, spacing, radius and motion; the written form is
[docs/DESIGN-SYSTEM-v3/01-FOUNDATION-TOKENS.md](DESIGN-SYSTEM-v3/01-FOUNDATION-TOKENS.md),
the component states are [02-COMPONENTS-STATES.md](DESIGN-SYSTEM-v3/02-COMPONENTS-STATES.md),
and the shell's own system is [03-SHELL-SYSTEM.md](DESIGN-SYSTEM-v3/03-SHELL-SYSTEM.md).

---

## 5. Design references and documentation

| Document | What it is for |
|---|---|
| [docs/UI-PLAN.md](UI-PLAN.md) | The UI plan and its phases. |
| [docs/UI-RESEARCH-MACOS.md](UI-RESEARCH-MACOS.md) | What was borrowed from macOS, and why. |
| [docs/DESIGN-SYSTEM-v3/](DESIGN-SYSTEM-v3/) | Audit, tokens, components, shell system, QA/lint governance, roadmap, app redesign. |
| [docs/PERF-budget.md](PERF-budget.md) | The frame budget the UI is measured against. |
| [docs/EVIDENCE.md](EVIDENCE.md) | Measurements, with the commands that produced them. |
| [docs/master-plan/07-compositor-shell-protocol.md](master-plan/07-compositor-shell-protocol.md) | Where the shell is headed. |
| [docs/master-plan/08-accessibility-i18n-input.md](master-plan/08-accessibility-i18n-input.md) | Accessibility, i18n, input. |
| [docs/master-plan/19-first-run-settings-builtins.md](master-plan/19-first-run-settings-builtins.md) | First-run and built-in settings. |
| [ADR 0017](adr/0017-framebuffer-shell.md) / [ADR 0009](adr/0009-unified-shell-process.md) | Why the shell is one framebuffer process. |
| [docs/bugs/](bugs/) | Every defect record. UI-relevant: [BUG-0015](bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md) (frame cost), [BUG-0016](bugs/BUG-0016-the-typeface-printed-boxes-and-the-grid-collided.md) (typeface and layout), [BUG-0018](bugs/BUG-0018-a-locked-frame-costs-half-a-second.md) (lock frame). |
| `C:\Users\audim\Downloads\litricos-ui-x` | The HTML prototypes the shell was ported from, **outside the repo**: `screens/desktop/{a-panel,b-dock-morph,c-orb}`, `screens/lock/{a-corner,b-center,c-giant-clock,customize,d-users,liquid}`, `screens/settings`, `screens/apps/liquid-open`, `vfx/{glow-bloom,goo-open}`, `shared/{base.css,litric-runtime.js}`. |

---

## 6. Measured gaps (things this inventory is honest about)

1. **Frame cost.** [BUG-0015](bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md):
   a frame with a window has been measured at 206 ms against a 16.67 ms budget,
   still open. [BUG-0018](bugs/BUG-0018-a-locked-frame-costs-half-a-second.md): a
   locked frame cost half a second, and is **fixed** as of 2026-10-15 — 293.727 ms
   to 35.819 ms (8.2x), 10.43M pixels to 1.91M, with the customize sheet still at
   185.718 ms. A second host measured a windowed frame at 107 ms and a locked
   frame at 335 ms on 2026-10-14 ([docs/EVIDENCE.md](EVIDENCE.md)); the defect was
   the surface, not the machine.
2. **[BUG-0019](bugs/BUG-0019-a-snap-that-changed-the-value-reported-no-frame.md):
   a spring that snapped to its target reported "nothing moved", so a frame that
   had to be drawn was not requested and the pixels it drives — a window's focus
   shadow — stayed stale. Fixed, with regression tests; found by the
   partial-redraw oracle, which had been passing vacuously.
2. **The typeface has no Thai glyphs, and the shell prints Thai anyway.**
   Observed live on the running desktop: the terminal's first line,
   [apps.rs:1426](../src/litric-shell/src/apps.rs#L1426)
   (`"LitricOS Terminal — พิมพ์คำสั่ง Linux แล้วกด Enter"`), renders as boxes
   (`▯▯▯▯▯`) because `font::glyph` has nothing for U+0E01–U+0E4C.
   [tools/scan-nonascii.py](../tools/scan-nonascii.py) exists to catch exactly
   this and reports the characters correctly when run with its default argument
   (Thai characters appear in `anim.rs`, `apps.rs`, `shell.rs`, `lock.rs`,
   `fsbridge.rs`, `textedit.rs`, `textaudit.rs`, `domkey.rs`).
   **Its argument handling is wrong**: `root.glob("*.rs")` does not recurse, so
   `python3 tools/scan-nonascii.py src/litric-shell` — the crate root, and the
   obvious thing to type — scans zero files and prints
   `no non-ASCII characters in string literals`. A clean report from a check
   that checked nothing. Fixing the glob (`rglob`) makes the tool honest; adding
   Thai coverage to the typeface is the larger decision. This is BUG-0016's
   territory and is not yet recorded there.
3. **Some quick-settings toggles only record a preference.** That is what
   `applied` and `is_applied` are for, and the UI says so — but it means the
   pane is currently narrower than it looks.
4. **Boot to desktop measured under QEMU is ~75 s**, while the desktop profile's
   budget is 3.5 s ([PERF-budget.md](PERF-budget.md)). A QEMU boot is not the
   hardware the budget describes, but the discrepancy is unexplained.

---

## 7. Seeing it

The desktop is a framebuffer program with no display server, so there are two
ways to look at it: dump frames, or attach a screen.

**Frames** (from `src/`, on the development host):

```bash
cargo run --release -p litric-shell --example render_bench   # 15-run medians
cargo run --release -p litric-shell --example frame_dump -- ../build/frames
python3 tools/frames-page.py                                  # browse them
```

**A live, usable screen**, on the Linux builder (WSL). QEMU's VNC server speaks
websocket itself, so a browser reaches the VM with noVNC and no proxy:

```bash
# once: sync, then build the image
wsl.exe -d archlinux -u root -- bash tools/sync-builder.sh
wsl.exe -d archlinux -u root -- bash build/wsl-image.sh        # detached, logs to build/image-build.log

# every time: boot it
wsl.exe -d archlinux -u root -- bash build/wsl-boot-vnc.sh
```

The scripts are [build/wsl-image.sh](../build/wsl-image.sh),
[build/wsl-boot-vnc.sh](../build/wsl-boot-vnc.sh), with the viewer at
[build/litric-viewer.html](../build/litric-viewer.html) (`ws://localhost:5700`;
ports are bound on `0.0.0.0` so WSL's forwarding carries them to Windows). Serial
logs land in `build/vm-serial0.log` (kernel + systemd) and `build/vm-serial1.log`
(the login shell — there is deliberately no getty on ttyS0, see
[BUG-0017](bugs/BUG-0017-smoke-test-listened-to-the-wrong-console.md)).
