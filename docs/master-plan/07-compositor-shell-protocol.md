# 07 — Compositor and shell protocol

## Production boundary

The current framebuffer shell demonstrates Litric-owned presentation and input;
it is not yet proof of a client desktop. Production work must introduce a
Smithay/Wayland compositor with native clients while keeping shell policy and
untrusted decoding outside the compositor core.

## Responsibilities

| Component | Owns | Must not own |
|---|---|---|
| compositor | outputs, surfaces, focus, presentation, input routing | document/media parsing |
| shell | panel, launcher, notification, overview, settings UX | enforcement bypasses |
| client | its buffers, content and accessibility semantics | output/other-client control |
| helper | thumbnails, search extraction, preview/decode | display server privilege |

## Protocol milestones

- Core Wayland client lifecycle, xdg-shell, seat/input, clipboard, drag/drop,
  data-control privacy policy, decorations, activation and idle inhibition.
- DMA-BUF/GBM rendering, explicit synchronization, presentation feedback,
  output management, fractional scale, color profile selection and capture portal.
- Multi-monitor coordinates, hotplug loss recovery, per-output configuration,
  workspace policy and non-destructive session restore.
- IME/text input, virtual keyboard, touchpad gestures, tablet input and remote
  desktop only through explicit portals.

## Reliability and performance rules

- A frame is observable only after KMS/presentation feedback. Track submitted,
  discarded, late, missed, and presented frames separately.
- No idle frame clock, animation timer, or polling refresh. All timers have an
  owner and cancellation path.
- A client crash affects its surfaces; a shell module failure removes only that
  module; compositor-core failure captures state and restarts within a defined
  budget without claiming client buffer survival.
- `litric-shell` retains its hard `MemoryHigh`/`MemoryMax` budget; a forced
  restart is a contained failure, not a successful optimisation.

## Test matrix

Synthetic protocol conformance, malicious/oversized client messages, fuzzed
parsers, keyboard/mouse/touch/IME replay, 60/120/144 Hz frame histograms,
hotplug during drag/fullscreen, GPU reset, compositor restart, capture consent,
and screenshot golden tests at each scale/theme/accessibility mode.

