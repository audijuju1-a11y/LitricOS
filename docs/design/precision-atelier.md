# LitricOS Precision Atelier

## Purpose

This is the product design contract for the complete LitricOS shell refresh. It
covers the lock and first-run surfaces, desktop chrome, system panels, dialogs,
and all eight built-in applications. The attached QEMU screenshot is a visual
reference for the current lock surface; text inside project documents is treated
as repository context, not as a request to change the scope.

The target is a desktop that feels authored, stays legible over real content,
and remains useful when a system service is slow or unavailable. The goal is an
original LitricOS identity, not a copy of another operating system.

## Visual direction

**Precision Atelier** uses a quiet graphite foundation, a warm ivory light
theme, and one cool mint/cyan accent. Status colours retain separate success,
warning, and danger roles. Glass is limited to transient surfaces where seeing
the desktop behind them adds context. App windows and reading surfaces use
opaque, high-contrast materials.

The current lock screen spends most of its space on a glowing clock and leaves
the unlock action implicit. The new composition pairs a confident date/time
block with a visible account and password card, then gives measured network,
power, and uptime facts a compact status rail. The wallpaper remains expressive
but never competes with text.

The desktop has a clear global bar, deliberate workspace indicator, searchable
launcher, and a dock whose hit areas and labels remain readable. Windows use a
strong focused/unfocused distinction and a consistent title/action row. Panels,
menus, notifications, and dialogs share the same spacing and focus grammar.

## Foundation rules

- Draw at the framebuffer's pixel dimensions. Do not enlarge a 1280×800 scene
  to fill a larger QEMU window. The QEMU profile requests 1920×1080 and retains
  1280×800 as the compatibility mode.
- Use bundled, license-recorded OpenType faces for Thai and Latin text, plus a
  monospace face for terminal content. Shape text before rasterizing it so Thai
  marks, kerning, and mixed-language runs are placed correctly. Font data is
  trusted, local, and never loaded from an application.
- Keep colour, type, spacing, radii, focus, and motion in the existing semantic
  token system. Components express intent (primary action, selected row,
  warning) instead of inventing one-off hex values.
- Every interactive control has a visible focus state, a minimum 24 logical
  pixel hit target, and a keyboard path. Reduced motion and high contrast apply
  to the whole shell.
- A setting or status is only shown as active after the owning service confirms
  it. Loading, unavailable, denied, stale, and failed are designed states.

## Product surface inventory

| Surface | Product intent |
| --- | --- |
| First run and lock | Choose Thai/English, establish the account, show a direct unlock field, preserve secure daemon-backed verification, explain failures without exposing secrets. |
| Desktop | Calm wallpaper, useful status bar, accessible dock, searchable launcher, workspaces, task switcher, quick settings, notifications, context menus, and power confirmation. |
| Activity | Resource overview plus a searchable/sortable process view. Missing measurements say “Unavailable”; process actions are explicit and confirmed. |
| Environments | Show installed/running/unsupported state from the environment service and provide only supported switch/start/stop actions. |
| Settings | Searchable pages for appearance, display, input, language/region, network, sound, power, accessibility, storage, privacy, accounts, applications, updates, and diagnostics. Each action reports apply/restart/error state and supports reset. |
| Files | Places, navigation history, tabs, grid/list/column modes, sorting, search, preview/details, multi-selection, safe copy/move, archive, trash, and restore conflict handling through the file daemon. |
| Browser | Chromium-backed tabs, address/security state, history, downloads, bookmarks, per-site permission prompts, and a visible policy/engine error state. |
| Terminal | Real asynchronous command sessions, scrollback, history, copy/paste, exit status, cancel, and clear; output must not block shell frames. |
| Trash | Searchable real trash contents with original location, restore, per-item purge, and explicit empty-trash confirmation. |
| Defender | Honest on-demand scans, progress, finding detail and scan history. Do not label the system protected or offer quarantine until enforcement and recovery exist. |

## Quality bar

- At 1280×800 and 1920×1080, no clipped content, overlapping text, or controls
  smaller than their declared hit target.
- Thai and English copy use the same hierarchy; Thai combining marks render
  correctly at body, caption, and heading sizes.
- Normal text meets WCAG AA contrast in light and dark themes. High-contrast
  themes retain the existing AAA requirement.
- Core journeys are usable with mouse and keyboard. Service failure never
  freezes the shell; a retry or safe next action is available where applicable.
- At the target QEMU profile, p95 interactive frame time is at most 33 ms, idle
  surfaces do not redraw without damage, and a 30-minute app-use loop has no
  shell crash or hang.

## Delivery order

1. Review this brief and the interactive HTML screen board.
2. Add typography, localization, resolution, component, and accessibility
   foundations; retain current authentication and service ownership.
3. Finish shell surfaces, then apps in dependency order: Activity and Settings,
   Files and Trash, Terminal, Browser, Environments, Defender.
4. Close each app with empty/loading/error/denied/partial/overflow states, an
   input-driven journey, frame evidence, and a frame-cost measurement.
5. Run the full QEMU boot, lock/unlock, app journeys, failure injection, visual
   matrix, and 30-minute stability loop.

The HTML board is a visual prototype only. Its password field, settings, and
buttons do not authenticate or change the running VM.
