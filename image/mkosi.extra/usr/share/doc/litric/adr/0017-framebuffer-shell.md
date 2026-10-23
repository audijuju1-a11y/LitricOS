# ADR-0017 — The desktop is drawn in software, on `/dev/fb0`, with no `unsafe`

- **Status:** Accepted, implemented, verified on a booted image
- **Date:** 2026-08-29
- **Extends:** ADR-0005 (compositor), ADR-0009 (unified shell process)
- **Does not supersede** either: this is the first stage of the shell they
  describe, not a replacement for it.

---

## The requirement

Power on the machine and see a LitricOS desktop. Not a login prompt, not a
console, not a splash image: the actual interface, drawn from the actual design
system, filled with the actual state of the machine.

## The constraint that shaped every decision

The workspace sets `unsafe_code = "forbid"` and carries **zero third-party
crates**, and the build runs offline. That rules out, in order:

- **Smithay / wlroots / any Wayland compositor library.** Not on the machine and
  not fetchable.
- **`mmap` on the framebuffer.** `unsafe`.
- **`ioctl` for `FBIOGET_VSCREENINFO` or DRM mode setting.** `unsafe`.
- **Any font crate, and any embedded TTF.** No crate, and a TTF would mean
  shipping someone else's file plus writing a glyph interpreter to read it.

The lazy conclusion is that a desktop is therefore impossible without relaxing
the rules. It is not. Each of those turns out to have an answer that is not a
compromise.

---

## The decisions

### 1. Geometry from sysfs, pixels through `write()`

The kernel already publishes what an `ioctl` would tell us:

```
/sys/class/graphics/fb0/virtual_size     "1280,800"
/sys/class/graphics/fb0/bits_per_pixel   "32"
/sys/class/graphics/fb0/stride           "5120"
```

and fbdev implements `write()`, so a frame is delivered with `seek` and
`write_all`. The whole display path is ordinary safe Rust over ordinary files.

The cost is one memory copy per frame — 4 MB at 1280×800 — which is nothing for
a desktop that repaints when something changes rather than sixty times a second.
If a compositing path ever needs the frame rate, `mmap` can be added then,
measured, and confined to one module with the `unsafe` visible in one place.

**What is assumed:** sysfs does not publish the channel offsets, so the byte
order is taken to be `XRGB8888` little-endian, which is what every 32-bit fbdev
on x86 uses in practice. The assumption is named in `PixelFormat`, overridable
with `--format`, and would show up immediately as a desktop with red and blue
swapped. It is checked by looking at the first screenshot from any new machine.

### 2. Signed-distance rendering rather than scanline filling

Shapes are drawn from a signed distance function: every pixel knows how far it
is from the shape's edge, so edge pixels get partial coverage. That is one
`sqrt` per pixel of a shape's bounding box, and it is the difference between an
interface that looks drawn and one that looks computed.

### 3. The typeface is a stroke font, authored here

`LitricOS Sans` describes each letter as the path a pen takes — polylines, arcs
and dots on a grid with cap height 10, x-height 7, baseline 0, descender −3 —
and rasterises it with the same anti-aliased line drawer used for everything
else.

This is better than the alternatives rather than merely available: it is
resolution-independent, so one description serves an 11-pixel label and a
60-pixel heading; it is a few kilobytes; it needs no parser; and it is
unambiguously ours. A monoline geometric sans is also the right design for this
interface — circles and straight lines, uniform weight, single-storey `a` and
`g` — so the constraint and the intent point the same way.

### 4. The glass is solved against the pixels actually behind it

Every translucent surface reads back the **brightest** pixel under its own
footprint, asks `litric_design::Glass::adaptive_alpha_pct` for the most
transparent tint that keeps both text roles at WCAG AA over that backdrop, and
uses it. Measured on the real composition: all four panels settle at **18%**,
the aesthetic floor, over the shipped wallpaper — and the worst text contrast
anywhere on screen is **5.19:1** against a required 4.5.

Brightest, not average, is load-bearing. Text has to stay readable over the
bright corner of a wallpaper, and an average would quietly excuse exactly that
corner.

### 5. `fbcon` is released through sysfs, not `KDSETMODE`

The kernel's framebuffer console owns the same pixels. Left bound, it repaints
the text console over the desktop the moment anything logs, which looks exactly
like the shell crashing. The usual fix is a `KD_GRAPHICS` `ioctl`; the same
decision is also a plain text write to `/sys/class/vtconsole/vtconN/bind`, so
that is what the shell does. Best effort: where it is refused, the desktop still
draws and the status file says so.

---

## What this is not

It is **not a Wayland compositor**. There is no input handling, no client
applications, no window management and no protocol implementation. The window on
screen is drawn by the shell, not hosted by it.

That is the honest description, and it is also the right first stage: the output
path, the design system, the typeface, the geometry and the glass all have to
exist and be correct before there is anywhere for a client surface to go. A
machine that boots to this is a machine with a desktop on it. The next layer has
somewhere to draw into.

---

## Consequences

- The shell needs write access to `/dev/fb0`, which is write access to the
  screen. A compromised shell can draw a convincing password prompt. That is
  inherent to any display server; the mitigation is that it holds no
  capabilities, has no network, and cannot read the user's files, so it can lie
  on the screen and cannot act on what it learns.
- Rendering is single-threaded software. At 1280×800 a full frame is a few
  milliseconds; at 4K it will need measuring, and that is when `mmap` and
  damage tracking earn their keep.
- `--png` renders the same composition to a file, so the desktop can be looked
  at on a build host with no framebuffer. Every screenshot in `docs/evidence`
  was produced either that way or by photographing the guest's display through
  the hypervisor.
