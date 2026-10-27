# ADR-0026 — The screen is scanned out over KMS, and the raster stays software

- **Status:** Accepted, implemented on the host; **not** verified on a booted
  image and never run against a real card
- **Date:** 2026-10-27
- **Extends:** ADR-0017 (framebuffer shell), ADR-0005 (compositor)
- **Answers:** W8 in [the execution plan](../master-plan/26-execution-plan.md),
  "Graphics decision (P05)"

---

## The requirement this settles

The execution plan left one decision open: stay on the framebuffer for Beta, or
move to DRM/KMS with the `unsafe` ioctls confined to one audited module. It also
stated the consequence of not deciding: GATE 05 cannot pass on framebuffer-only
output, and `unsafe_code = "forbid"` in the shell rules the move out without a
decision on record.

This ADR takes the second road, and it is deliberately narrower than "use the
GPU".

## What was actually wrong

Three separate things were being described by one sentence, "the desktop draws
to `/dev/fb0`":

1. **fbdev is a compatibility shim.** It has no notion of a frame: there is no
   point at which the desktop says "show this now", so a frame appears whenever
   the driver decides to notice.
2. **`write(2)` on fbdev is not a promise.** QEMU's DRM fbdev bridge accepts
   the write and can leave the scanout unchanged. The shell's own fb module
   records this and works around it with a helper that `mmap`s the framebuffer
   and pokes `FBIOPAN_DISPLAY`. That is a workaround for a path that should not
   be the primary one.
3. **Every repaint crossed a pipe.** The mapped helper receives complete,
   fixed-size frames over stdin, so a pointer move that changes two rectangles
   still moved 4 MB at 1280×800. `present_full` measures 3.7 ms on the
   development host, which sounds fine until it is the *cheapest* part of a
   71 ms frame.

## The decisions

### 1. KMS is the primary output path; fbdev is the fallback

The shell opens `/dev/dri/cardN` through a native helper, finds a connected
connector, takes its preferred mode, allocates two dumb buffers and sets the
CRTC. A repaint is then a set of row bands written into memory the card is
already scanning out, followed by `DRM_IOCTL_MODE_PAGE_FLIP` with
`DRM_MODE_PAGE_FLIP_EVENT`. The display controller switches which buffer it
scans out at the next vertical blank.

This is the point of the change: the flip is performed by the hardware that
owns the scanout, at the display's own cadence, and the CPU never copies a
finished frame to the screen.

### 2. A partial update is written into *both* buffers

This looks wasteful and is the only simple way to stay correct. A band is
applied to the front and back buffers before the flip, so whichever buffer the
card is scanning out already contains the newest band. Without it, flipping
after a small update shows the previous *frame* with one new rectangle on it —
a stale-frame bug that only appears under partial damage, which is exactly the
path the pointer uses.

### 3. The unsafe boundary is one C file, and it is smaller than the old one

`src/litric-shell/src/drmpresent.c` is the only place in the output path that
issues an ioctl. Rust stays `#![forbid(unsafe_code)]`, and the shell keeps
pixels, geometry, damage, timing and every policy decision. The helper accepts
three commands (`B` band, `P` flip, `Q` quit), has no filesystem access beyond
the one card node it was handed, no network, and no command that moves, scales
or restyles the picture. It prints one handshake line with the mode it actually
set, and the shell refuses to use it if that line is not exactly this protocol.

### 4. The choice is stated, never inferred

- `--presenter auto|drm|fbdev`, defaulting to `auto`.
- The status file always contains one `present=` line: `present=drm`,
  `present=fbdev`, or `present=drm-skipped(<reason the machine gave>)` followed
  by the path that was used.
- `litric-drmpresent --check /dev/dri/cardN` reports connector, mode and pitch
  without taking DRM master or touching the screen, and the image's selftest
  prints both `drm.presenter` and `drm.mode`.

A desktop that fell back looks identical on screen to one that worked. The
difference has to be readable from outside the process, which is why these
lines exist rather than a comment.

### 5. Nothing in this path may hang

The handshake is read on its own thread with a five-second bound; a helper that
never speaks is killed and reported, and the shell presents through fbdev
instead of stopping before the first pixel. A page flip that does not complete
within 100 ms is logged and the next frame proceeds. A wedged display must not
turn a compositor bug into a machine that looks like it failed to boot.

### 6. The raster is unchanged, and this ADR does not make it faster

The full-frame compose is still CPU SDF rasterisation in `canvas.rs`, and it is
still the number that matters: 71.4 ms for a full frame at 1280×800 against a
16.7 ms budget. This ADR moves the *present* cost, not the *draw* cost. Saying
otherwise would be the exact kind of claim the project has had to retract
before.

## What this is not

- **Not a GPU rasteriser.** A texture upload and a fullscreen quad would not
  make the shell faster, because the shell draws with the CPU into a canvas
  first. Moving the SDF raster, the glyph raster and the blur to GL/Vulkan is a
  separate architecture with its own ADR, its own GPU matrix and its own
  failure modes; nothing in this change pretends to do it.
- **Not `dmabuf` import.** Client surfaces (ADR-0024) still arrive as shared
  memory and are composed by the shell.
- **Not multi-monitor, VRR, HDR or plane composition.** One connector, one
  CRTC, one plane. Those need atomic modesetting, which needs more ioctls and a
  reason.
- **Not verified on hardware.** See the evidence section.

## Consequences

- The image build needs the kernel's DRM uapi headers (`linux-api-headers` on
  Arch). `tools/build.sh` probes for them: when they are absent it does not
  guess, it warns in the build log that `litric-drmpresent` will not be in the
  image and that the shell will present through fbdev.
- `litric-shell.service` no longer requires `/dev/fb0`. The condition is an OR
  over `/dev/fb0` and `/dev/dri/card0|1`, because a machine with KMS and no
  fbdev node is exactly the machine this path is for. The unit already allowed
  `char-drm rw`; nothing wider was opened.
- Damage-only presents are now genuinely cheap in bytes: a pointer move sends
  the rows it touched instead of 4 MB. The status file's `rows_presented`
  counts what was sent, so the claim is checkable.
- If a driver refuses page flips on dumb buffers, the helper logs it once and
  keeps scanning out in place. The picture stays correct because both buffers
  receive every band; the downgrade is visible in the journal rather than
  silent.
- The draw path is untouched, so BUG-0015 (a frame with a window open costs
  151.9 ms) is still open and still the reason the desktop feels slow.

## Evidence, and what it is worth

| Check | Result |
|---|---|
| `cc -std=c17 -O2 -Wall -Wextra -Werror drmpresent.c` | compiles clean |
| `litric-drmpresent --check /dev/dri/card0` on a host with no card | exits 1 with `open card: No such file or directory` |
| `litric-drmpresent /tmp/fake-card0` (a regular file) | exits 1 with `get resources: Inappropriate ioctl for device`, no hang, no master taken |
| `cargo test -p litric-shell --lib drm::` | 14 pass: handshake parsing and refusals, band merging and clipping, pitch-correct encoding, protocol capture against a stand-in helper, a silent helper timing out, an exiting helper and a nonsense handshake being refused, card discovery |
| `cargo test -p litric-shell --lib present::` | 7 pass: DRM preferred when card and helper exist, fbdev fallback with the recorded reason, a required path that is missing being an error, a broken helper degrading, a forced framebuffer never starting the card |
| fixture root, `--presenter auto --once` | status contains `present=drm-skipped(no card node under /dev/dri)` then `present=fbdev`, and 256,000 bytes of real pixels reach the fixture device |
| `cargo test -p litric-shell` | 555 lib + 32 + 4 + 10 + 3 pass |
| booted image on a real card | **NOT RUN.** No card node exists on the build host and no image was booted with this path |

The protocol is verified end to end against a stand-in process, which is a real
test of everything the shell controls. The ioctl sequence is *compiled*, not
run. Until someone boots a desktop profile on a machine with a DRM card and
reads `present=drm` plus `drm.mode` from the status file, the KMS path is
`NOT VERIFIED` in this repository's sense, and GATE 05 stays open.
