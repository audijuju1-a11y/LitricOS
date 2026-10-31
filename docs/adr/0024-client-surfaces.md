# ADR-0024 — Client surfaces: a Wayland server inside the shell

- **Status:** Accepted
- **Date:** 2026-10-25
- **Supersedes in part:** nothing. It fills in the half of ADR-0009 that was
  written down but never built: "ordinary applications remain fully isolated
  Wayland clients".
- **Drives:** SYSTEM-SCOPE rows about client windows, APP-COMPAT phase B.

---

## Context

ADR-0009 decided the compositor and the shell would be one process, with
Smithay as the protocol foundation (ADR-0005). ADR-0017 then built the *output*
half of that in software: a framebuffer, a canvas, a text renderer, a window
manager, input decoding and a shell that draws everything itself.

The input half of the claim was false in one specific way: **no foreign process
could put a pixel on that desktop.** Every window was drawn by the shell. A
Linux application, however well it ran, had nowhere to go: there was no
`wayland-0` socket, no `wl_compositor`, no `xdg-shell`, and therefore no
application window. That is the difference between a desktop and a mock-up of
one, and it was the largest single gap in the system.

## Decision

**A real Wayland server, inside the desktop process's session, with client
surfaces hosted as ordinary windows.**

1. **A new crate, `litric-compositor`.** It owns the wire protocol, the object
   graph and the file descriptors. It is safe Rust (`#![forbid(unsafe_code)]`);
   the audited `sendmsg`/`SCM_RIGHTS` path belongs to
   `wayland-backend`, which is the dependency the workspace header already
   sanctioned ("you do not hand-write a Wayland compositor").
2. **Its own thread.** `wayland-server`'s display is not `Send`, and the shell's
   loop blocks in `poll` on input. The compositor thread sleeps in `poll(2)` on
   three descriptors — the client socket, the listening socket and a wake pipe —
   so an idle desktop with no clients is genuinely asleep, and a client commit
   reaches the shell at once instead of at the next frame budget.
3. **Pixels by shared memory readback.** `wl_shm` only, `ARGB8888` and
   `XRGB8888`, read out with `pread` after each commit. No GPU import, no
   dmabuf, no mmap. It is not the fast path and it is honest about what it is:
   the cost of a frame is bounded by the window's area and every buffer is
   bounds-checked against the pool the client actually passed.
4. **Client windows are window-manager windows.** `litric-wm::Content::Client`
   already existed for exactly this; `litric-shell::clients` maps a surface to a
   window, paints the body, and reserves the title bar for LitricOS. Maximise,
   snap, drag, focus, workspaces, the switcher and the dock therefore work on a
   foreign application without a second window system.
5. **Input goes to the application, not around it.** A key reaches the focused
   client unless it carries Meta, so Win+E still opens Files while a game has
   focus — a desktop a person cannot leave is not a desktop. Pointer events are
   z-ordered through the window manager before they are forwarded, so a click on
   a buried window does not reach the application under it.
6. **Popups are drawn, not ignored.** `xdg_popup` surfaces are painted above
   their parent at the positioner's anchor rect plus offset, and dismissed on a
   click outside. A menu that opens and cannot be seen is worse than one that
   fails to open, because the user cannot tell which happened.
7. **A crash is a window that goes away.** Every failure path — a killed client,
   a malformed buffer, a socket closed mid-commit — turns into `Gone`, the
   window closes, and the session continues.

## What is actually advertised

| Global | Version | Note |
|---|---|---|
| `wl_compositor` | 4 | surfaces, regions, buffer scale, `damage_buffer` |
| `wl_shm` | 1 | `ARGB8888`, `XRGB8888` only |
| `wl_seat` | 5 | pointer + keyboard; no touch capability is claimed |
| `wl_output` | 4 | one output at the shell's size, scale 1 |
| `xdg_wm_base` | 2 | toplevels and popups |

Deliberately absent, and reported as absent rather than half-supported:
`wl_data_device_manager` (clipboard and drag-and-drop), `wl_subcompositor`,
`linux-dmabuf`, `presentation-time`, `wp_viewporter`,
`wp_fractional_scale`, `xdg_decoration`, `zwp_text_input_v3` (IME), XWayland.

The keyboard keymap is a resolved `us` layout shipped as `src/keymap.xkb`,
generated once from `xkb_keymap_get_as_string`. A client decodes keycodes with
it. **A user who switches the shell's layout does not switch the client's**:
that is a real limitation, listed below, not a detail.

## Evidence

- `litric-compositor/tests/client.rs` — a foreign C client (`weston-simple-shm`,
  libwayland-client) connects, maps a window, commits frames with real content,
  honours a `configure` to 320x200, survives pointer and key events without a
  protocol error, exits when the toplevel is closed, and a killed client is
  reported `Gone` without taking the server with it.
- `litric-shell/tests/foreign_clients.rs` — the desktop hosts that client as a
  window with a LitricOS title bar, paints the client's pixels in the body,
  forwards a click and a keystroke, closes the client when the window closes,
  and launches a GUI application by typing its name into the Terminal.
- `artifacts/foreign-client-window.png` — the rendered desktop frame.
- Unit tests for pool bounds, stride handling, scaling, `XRGB` opacity and the
  XKB modifier bits.

## What this does not do

- **No XWayland.** X11 applications have no display. That is phase C and needs a
  Wayland server with the protocols XWayland expects, plus the X server itself.
- **No clipboard, no drag-and-drop, no IME.** `wl_data_device_manager` and
  `zwp_text_input_v3` are not implemented, so copying between applications and
  typing Thai into one does not work yet.
- **No package manager or AppImage installer.** S82 adds validated local
  bundle installation and `.desktop` discovery/launching (ADR-0027), including
  the display environment the daemon must pass. There is still no network
  package source, signature verifier or AppImage extractor.
- **No GPU path.** dmabuf import and hardware overlays are absent; every frame
  is a copy through the CPU.
- **Popups are anchored, not positioned.** Size, anchor, gravity and constraint
  adjustment are recorded but only the anchor rect's origin is used, so a menu
  near the screen edge is clamped by the shell rather than flipped by the
  protocol.
- **Not verified on the image.** The socket, the unit's `XDG_RUNTIME_DIR` and
  the sandbox are wired, but no QEMU boot has been run from this change.

## Revisit if

- A toolkit turns out to need a protocol listed as absent before it will start;
  that protocol moves from "absent" to "implemented and tested", not to
  "advertised and hoped for".
- The readback cost becomes the bound on large-window animation; that is the
  signal to add dmabuf import rather than to micro-optimise the copy.
- Multi-seat or touch hardware arrives; the seat is a state machine here, not a
  constant, so a second seat is a change of topology rather than a rewrite.
