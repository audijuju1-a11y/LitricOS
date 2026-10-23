# ADR-0009 — The compositor and the shell are one process

- **Status:** Accepted (v2)
- **Date:** 2026-08-26
- **Supersedes in part:** ADR-0005 (compositor foundation) — Smithay is retained;
  the process topology above it changes.
- **Drives:** `docs/PERF-budget.md` line 8, GATE 05, GATE 06.

---

## Context

The `desktop` profile budget allows **110 MB** for the entire graphical stack:
compositor, panel, dock, launcher, overview, notifications, settings and file
manager. A conventional Wayland desktop cannot fit in that, and the reason is not
inefficiency in any one component — it is process topology.

A conventional stack looks like this:

```
compositor        (wlroots/Smithay)     60–120 MB
panel/dock        (GTK4 layer-shell)     45–90 MB
launcher          (GTK4)                 40–80 MB
notification dmn  (GTK4)                 25–45 MB
settings app      (GTK4)                 60–120 MB
------------------------------------------------
                                       230–455 MB
```

Each toolkit client maps GTK4, GLib, GIO, Pango, Cairo, HarfBuzz, GdkPixbuf,
librsvg, and their transitive closure. Shared libraries share their *text* pages
across processes, but every process pays its own relocations, GObject type
registrations, theme parse, icon cache, font cache, style context, and per-process
heap arenas. In practice a "small" GTK4 utility lands at 35–60 MB RSS before it
draws anything of its own.

Multiply that by five, add a Wayland IPC round trip and a separate buffer
allocation for every panel repaint, and the topology — not the code — is the cost.

## Requirements

- `desktop` idle graphical stack ≤ 110 MB (budget line 8).
- `litric-shell` idle ≤ 2 wakeups/second (budget §4).
- p99 frame time within the display's refresh budget (budget §4).
- A crash in a shell component must not destroy the user's session (Phase 06 gate:
  auto-restart within 2 s, session intact).
- Every user-facing action reachable by keyboard alone (Phase 06 gate).

## Options considered

### A. Conventional multi-process, GTK4 shell
- **Pros:** fastest to build; enormous ecosystem; accessibility comes free via
  ATK/AT-SPI; theming and i18n solved.
- **Cons:** 230–455 MB, i.e. **2–4× over the entire desktop budget** before the
  kernel is counted. Non-starter. Also inherits GTK's per-process wakeup
  behaviour, which alone would blow the 25 wakeups/second target.

### B. Multi-process shell on a minimal toolkit (e.g. custom Rust + wgpu clients)
- **Pros:** true fault isolation between panel, launcher and compositor; each
  component independently restartable; conventional Wayland client model.
- **Cons:** each client still needs its own GPU context, its own swapchain, its
  own font atlas and its own shaper. A GPU context plus a font atlas is ~15–25 MB
  before any content. At four clients that is 60–100 MB *of pure duplication*,
  and it lands right at the budget ceiling with nothing left for actual windows.
  It also reintroduces a compositor↔client round trip on every panel update.

### C. **Single process: compositor and shell share one address space** ← selected
- The Wayland compositor, the DRM/KMS presentation loop, and every shell surface
  live in one binary, on one GPU device, with one font atlas, one shaper, one
  glyph cache and one scene graph.
- Shell surfaces are composited directly as scene-graph nodes. They never become
  Wayland clients, so a panel repaint costs a buffer update in the same process
  rather than a protocol round trip, a buffer allocation, and a commit.
- Ordinary applications remain fully isolated Wayland clients, exactly as before.
  **This decision is about the shell, not about applications.**

## Decision

**Option C.** `litric-shell` is one binary containing:

```
litric-shell
├── core/          Smithay: wl_compositor, xdg-shell, DRM/KMS, libinput, dmabuf,
│                  linux-drm-syncobj-v1, presentation-time, fractional-scale
├── render/        one wgpu/Vulkan device, one glyph atlas (cosmic-text/swash),
│                  one batched 2D renderer, one scene graph
├── modules/       panel · dock · launcher · overview · notifications ·
│                  settings · file browser — each a scene-graph subtree
└── ipc/           litric.gfx / litric.caps endpoints for external clients
```

**No GTK, no Qt, no Electron, no embedded browser, no JavaScript anywhere in the
graphical path.** Text shaping is `cosmic-text` + `swash`; there is no Pango, no
Cairo, no librsvg.

## Consequences

### Positive
- Budget line 8 becomes reachable: one GPU context, one atlas, one heap.
- Panel and notification updates skip the Wayland round trip entirely.
- Input latency drops: input events reach shell UI without a client hop.
- The idle wakeup target becomes achievable — one event loop, fully event-driven,
  with no per-client timers.
- One design-token source of truth is *structurally* enforced (Phase 06 task 3):
  every module reads the same `tokens/` values in-process. There is no second
  toolkit that could disagree.

### Negative — and this is the real cost
**A panic in a shell module kills the whole session**, including every client
window, because Wayland clients cannot survive their compositor. This is a
genuine regression against the multi-process option and it must be engineered
against, not hand-waved.

Mitigations, all mandatory:

1. **The compositor core is panic-free by construction.** `#![deny(clippy::unwrap_used,
   clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` on the `core/`
   module. Enforced in CI, not by review.
2. **Module isolation via `catch_unwind`.** Each module's update and draw entry
   points are wrapped. A panicking module is disabled, logged with its backtrace,
   and its scene subtree is removed; the compositor and every client window
   survive. The binary therefore builds with `panic = "unwind"`, *not* `abort` —
   these are mutually exclusive and unwind is the correct trade here.
3. **Untrusted and unbounded work is out-of-process, always.** Thumbnail
   generation, search indexing, media decode, image decode of user files, and
   archive inspection run in separate sandboxed helpers and communicate over IPC.
   The shell process must never parse a user-supplied file format.
4. **Protocol parsing is fuzzed.** The Wayland protocol surface is the shell's
   attack surface; `cargo-fuzz` targets are a GATE 05 requirement, not a nicety.
5. **Session-preserving restart.** On unrecoverable failure the shell records open
   client identities, exits, and is restarted by systemd within 2 s. Client state
   survives only to the extent applications implement their own state restoration
   — **we do not claim window contents survive, because they cannot.** The Phase 06
   gate is met for *shell component* crashes (mitigation 2), not for compositor-core
   crashes.
6. **The memory budget is a hard cap, not a target.** `MemoryMax=140M` means a
   shell leak gets OOM-killed and restarted rather than eating the machine.

### Accepted, documented limitations
- Accessibility does not come free. AT-SPI must be implemented against our own
  scene graph. This is real work and it is a GATE 06 blocker, not deferrable — it
  is listed here so it cannot be forgotten when the budget looks good.
- No third-party GTK/Qt *shell extensions*. Applications are unaffected; shell
  extensibility is a first-party plugin API or nothing. That is a deliberate
  ecosystem cost.
- Right-to-left text, complex scripts and IME are our responsibility. `cosmic-text`
  covers shaping; IME needs `text-input-v3` implemented by us.

## Verification plan

| Claim | How it will be proven |
|---|---|
| ≤ 110 MB idle | `smem -P litric-shell` PSS, n=5, 60 s settle |
| ≤ 2 wakeups/s idle | `powertop --time=60` |
| p99 frame within budget | in-compositor `presentation-time` histogram, 60 s |
| Module panic does not kill session | fault injection: a test module that panics on demand; assert clients survive |
| No toolkit crept in | `ldd litric-shell` asserted against an allowlist in CI |

The last row is the one that keeps this decision honest over time. A single
`gtk4` in `ldd` output fails the build.

## Revisit if

- Measurement shows the unified shell exceeds 110 MB anyway, in which case the
  premise was wrong and Option B deserves a second look.
- Shell-core crash frequency in real use exceeds ~1 per 1000 hours, in which case
  fault isolation is worth more than the megabytes.
- A memory-safe toolkit appears that idles under ~15 MB per client with a shared
  GPU context, which would make Option B viable within budget.
