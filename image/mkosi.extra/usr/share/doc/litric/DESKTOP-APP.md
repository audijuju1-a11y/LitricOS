# Running LitricOS as an application

Double-click `LitricOS.cmd` at the root of the checkout. It opens the operating
system in its own window: UEFI firmware, the signed disk image, systemd, the
LitricOS daemons, and the shell drawing at 1920x1080.

The same thing from a terminal, inside WSL:

```bash
tools/desktop-app.sh                 # window, LAN, mouse, keyboard
LITRIC_FULLSCREEN=1 tools/desktop-app.sh
LITRIC_DISPLAY=none tools/desktop-app.sh   # headless, for screenshots
```

## What the guest really has

| Device | How | Real? |
| --- | --- | --- |
| Mouse | `usb-tablet` — absolute pointer | Yes. HID → evdev → the shell's pointer path. |
| Keyboard | `usb-kbd` | Yes. HID → evdev → the shell's key handling. |
| LAN | `virtio-net-pci` on QEMU user networking | Yes. The guest runs its own DHCP client and holds `10.0.2.15/24`, route via `10.0.2.2`, DNS `10.0.2.3`. |
| Wi-Fi | none | No adapter. QEMU 11 has no wireless controller model. |
| Bluetooth | none | No adapter. QEMU 11 has no Bluetooth controller model. |
| Disk | the built GPT image, read/write | Yes. The root is erofs behind dm-verity; `/var` is btrfs and writable. |

The network is not a stub: `litric-sysd` reports the lease the guest actually
holds — `ip -br addr` inside the guest prints `enp0s4 UP 10.0.2.15/24`, and the
route table has `default via 10.0.2.2 dev enp0s4` — and traffic is NATed by QEMU
to the host's connection.

`nano` is the CI profile and does not ship `iwd` or `bluez`; `core` ships `iwd`
and `desktop` adds `bluez` + `bluez-utils`. Build the one whose software you
want to exercise:

```bash
tools/build.sh --profile desktop     # iwd, bluez, bluetoothctl, chromium, pipewire
```

The `desktop` image is the one to use as an application. Built 2026-10-17 it
carries `iwd`, `bluetoothd` + `bluetoothctl`, `chromium`, the Wi-Fi firmware for
the Intel, Atheros, MediaTek and Realtek radios, `pipewire`, `libinput` and
`seatd` — 388 packages against `nano`'s 94. It also gives the guest a 1920x1080
framebuffer where `nano` came up at 1280x800, so the shell draws at the size its
layout was measured at.

### Wi-Fi and Bluetooth

There is no radio, so the Wi-Fi and Bluetooth panels say so. That is the point
of those panels: they report what the machine has, and this machine has none.
Nothing in the UI invents a network list.

To use a real radio, attach a USB Wi-Fi or Bluetooth dongle to WSL and pass it
through. On the Windows side (once, elevated):

```powershell
winget install usbipd
usbipd list
usbipd bind --busid 2-1        # the dongle
usbipd attach --wsl --busid 2-1
```

Then in WSL, give the device to the guest:

```bash
lsusb                              # find idVendor:idProduct
LITRIC_USB=vendorid=0x0bda,productid=0x8771 tools/desktop-app.sh
```

The guest's own driver binds the device, `iwd` manages the Wi-Fi association and
`bluez` the pairing. The panels then show real hardware.

## Measured on this host, 2026-10-17

`tools/measure-idle.py` boots the built image, logs in over the guest's own
console and reads the machine's `/proc`, so these are the guest's numbers rather
than the host's view of `vmmem`:

```
accel            kvm
boot_to_login    22.0 s   (login prompt on serial; UEFI, UKI, systemd)
idle RAM         452 MB used of 1449 MB
  process RSS    164 MB total, 16 MB litric-shell
idle CPU         3.7 %
```

Those numbers were taken on the `nano` image. On the `desktop` image the shell
is busier — a 6-minute session reported `litric.shell.frames=519`,
`frames_drawn=518`, `frames_over_budget=204` — and under an emulated display
without a GPU that over-budget count is a fact about this host's rendering cost
as much as about the shell. It is recorded here rather than smoothed over: the
frame budget exists to be exceeded visibly, and 39 % of frames exceeding it on
an emulated machine is a number someone should look at on real hardware next.

Read them for what they are. The boot time and the idle CPU are an **emulated**
machine: QEMU's software display and emulated devices cost CPU that real
hardware does not pay, and the profile budgets in `docs/PERF-budget.md`
(1.2-3.5 s boot, 0.1-0.5 % idle CPU) are hardware targets that this host cannot
confirm or refute. The RAM figure is a measurement of the guest as it really
was: 452 MB used, of which 164 MB is process resident memory. It is above the
320 MB `desktop` target, and that is a finding rather than a rounding error --
the budget is a target precisely so that a measurement like this can disagree
with it.

## Memory

The guest is started with `-m 1536`. The WSL VM it runs inside is capped at
4 GiB by `.wslconfig` (`memory=4GB`, `swap=2GB`, `autoMemoryReclaim=gradual`),
which is what keeps `vmmem` from sitting at 7-8 GB with nothing running. The
shell's own slice limit is 140 MB (`litric-shell.slice`), so the rest of the
guest's memory is page cache and the browser engine.

`free -m` inside WSL should read `Mem: 3917` total. If it reads 8192, the
`.wslconfig` was written after the VM started: `wsl --shutdown` and start again.

## Powering off

Close the window and stop the launcher (Ctrl+C in the console `LitricOS.cmd`
opened). That is a power cut from the guest's point of view: `/var` is btrfs and
journalled, so it comes back, but the clean way is the desktop's own shutdown.

## First boot

A freshly built disk has no password for the `litric` account. On first boot the
shell is not started: `litric-first-run.service` owns tty1 and asks for a
language and a password, exactly as a new machine would. That prompt is on the
graphics console, i.e. in this window — type `1` (Thai) or `2` (English), then
choose a password. The disk in `/tmp` keeps that setting, so the second boot
goes straight to the desktop.
