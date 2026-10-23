# LitricOS running as an application window, 2026-10-17

What these are: frames grabbed from the emulated framebuffer of a real boot of
the built disk image, opened with `tools/desktop-app.sh`. Input was injected
through QEMU's emulated USB keyboard and tablet, so every effect below travelled
the path a physical keyboard and mouse travel: HID → evdev → the shell's own
event loop. Nothing here is a mock-up and nothing was drawn by the host.

| Frame | Input that produced it | What it shows |
| --- | --- | --- |
| `01-desktop-at-rest.png` | none | the desktop with nothing open |
| `02-launcher-open.png` | Super | the launcher, opened from the keyboard |
| `03-launcher-typed.png` | `f` `i` `l` `e` `s` | the launcher filtering on what was typed |
| `04-files-open.png` | Enter | the result of the search, opened |

Each frame differs from the one before it; the comparison was made on the raw
pixels, not by eye: `launcher changed frame: True`, `typing changed frame: True`,
`opening changed frame: True`.

`../app-desktop/` holds the same exercise on the `desktop` profile image, which
is the one worth running as an application: it ships `iwd`, `bluetoothd`,
`bluetoothctl`, `chromium` and the radio firmware, and it comes up at 1920x1080
rather than the 1280x800 `nano` gets. Frames there are `01-desktop-at-rest`,
launcher opened with Super, filtered by typing `settings`, and the result opened
with Enter — each one compared against the last on raw pixels.

`serial-app-boot.log` is the same boot's kernel and service log, filtered to the
lines that say what the guest had to say about itself:

```
litric-shell[419]: litric.shell.frames=8
litric-shell[419]: litric.shell.frames_drawn=7
litric-shell[419]: litric.shell.frames_over_budget=0
```

## How it was taken

```bash
tools/desktop-app.sh                 # the window; QMP socket for screendumps
python3 build/qmp.py shot out.png    # a frame
python3 build/qmp.py key ret         # a key
```

The screen is 1280x800 here, which is what this host's `bochs-display` gives the
guest's DRM driver — it is an EDID property of the emulated card, not a choice
the shell makes. The shell scales its layout to what the kernel exposes; the
supported minimum is 960 wide. On a host whose QEMU has `virtio-gpu`, the
kernel's `video=Virtual-1:1920x1080` is honoured and the same image draws at
1920x1080.
