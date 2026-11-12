#!/usr/bin/env bash
# LitricOS as an application window.
#
# WHAT THIS IS
# ------------
# One command that opens the real operating system in a window on this machine:
# UEFI firmware, the signed image, systemd, litric-sysd, and the shell drawing
# into the emulated framebuffer. Mouse and keyboard are USB HID devices the
# guest enumerates, so input travels the path a physical mouse and keyboard
# would take -- through evdev into the shell's own event loop. There is no
# remote-desktop shim and no synthetic input inside the guest.
#
# The window itself is a GTK window served by WSLg, which is why this runs
# inside WSL rather than on the Windows side: the host has no X server.
#
# WHAT IS REAL AND WHAT IS NOT
# ----------------------------
# LAN: real. A virtio-net NIC on QEMU's user-mode network gets a DHCP lease
# from the guest's own systemd-networkd, and traffic is NATed by QEMU. The
# LitricOS network panel shows the lease the guest actually holds.
#
# Wi-Fi and Bluetooth: no hardware. QEMU 11 has no wireless or Bluetooth
# controller model -- there is no register-compatible emulation of a radio to
# give the guest, and inventing one would be a driver for a device that does
# not exist. The guest therefore reports no adapter, and the panels say so;
# that is the honest answer, and it is the same answer a desktop with the radio
# switched off in firmware gives.
#
# Real radios can still be used: a USB Wi-Fi or Bluetooth dongle attached to
# WSL (usbipd-win on the Windows side) shows up as a /dev/bus/usb device, and
# LITRIC_USB="vendorid=0x0bda,productid=0x8771" hands it to the guest as a real
# USB device. The guest drives it with its own driver and its own iwd/bluez, so
# what the panels show is then whatever that hardware actually does. See
# docs/DESKTOP-APP.md.
#
# RAM: the guest is capped at 1.5 GiB, inside the 4 GiB WSL ceiling this
# machine is configured with. The shell's own slice limit is 140 MB, so the
# rest is page cache and the browser engine.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$here/build/out"

disk="${LITRIC_DISK:-/tmp/litricos-app.raw}"
memory="${LITRIC_MEMORY:-1536}"
smp="${LITRIC_SMP:-2}"
display="${LITRIC_DISPLAY:-gtk}"
qmp="${LITRIC_QMP:-/tmp/litricos-qmp.sock}"
serial="${LITRIC_SERIAL:-/tmp/litricos-app-serial.log}"
firmware="${LITRIC_FIRMWARE:-/usr/share/edk2/x64/OVMF_CODE.4m.fd}"
vars="${LITRIC_VARS:-/tmp/litricos-app-vars.fd}"
share="${LITRIC_SHARE:-}"
debug_sock="${LITRIC_DEBUG_SOCK:-/tmp/litricos-ttyS1.sock}"

usage() {
    sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Options (environment variables, because this is usually launched by a shortcut):
  LITRIC_DISPLAY=gtk|none   gtk opens a window; none is headless for a screendump
  LITRIC_MEMORY=1536        guest RAM in MiB
  LITRIC_SHARE=/mnt/d       expose a host directory to the guest over 9p
  LITRIC_USB=vendorid=0x...,productid=0x...   pass a real USB device through
  LITRIC_FULLSCREEN=1       open the window full screen
  LITRIC_DEBUG_CONSOLE=1    also attach a login prompt on ttyS1 to a unix
                            socket ($debug_sock) for host-side guest checks
  LITRIC_DISK=...           disk image to boot (default: a scratch copy in /tmp)
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

command -v qemu-system-x86_64 >/dev/null || {
    echo "desktop-app: qemu is not installed in this WSL distribution" >&2
    exit 1
}
[[ -f "$firmware" ]] || {
    echo "desktop-app: no UEFI firmware at $firmware" >&2
    exit 1
}
[[ -f /usr/share/edk2/x64/OVMF_VARS.4m.fd ]] || {
    echo "desktop-app: no UEFI variable store template" >&2
    exit 1
}

# The built disk is stored compressed. Decompress once into /tmp: 3.2 GB of
# sparse file, and re-extracting on every launch would put a 1.4 GB zstd
# decompression in front of the window appearing.
if [[ ! -f "$disk" ]]; then
    if [[ -f "$out/litricos.raw" ]]; then
        disk="$out/litricos.raw"
    elif [[ -f "$out/litricos.raw.zst" ]]; then
        echo "desktop-app: decompressing $out/litricos.raw.zst -> $disk (once)"
        zstd -dc -T0 --long=27 "$out/litricos.raw.zst" > "$disk"
    else
        echo "desktop-app: no disk image; run tools/build.sh first" >&2
        exit 1
    fi
fi

# A fresh variable store per launch. Reusing one keeps the previous boot order
# and any half-written efibootmgr entry, which reads as "the image stopped
# booting" on the second run only.
cp -f /usr/share/edk2/x64/OVMF_VARS.4m.fd "$vars"
rm -f "$qmp" "$serial"

# The display device is chosen from what this QEMU has. bochs-display is the
# one that gave the guest a DRM framebuffer with an EDID on this host, and it
# takes an explicit mode, so the shell draws at the size its layout was
# designed for instead of the 1280x800 default.
disp_args=(-device bochs-display,edid=on,xres=1920,yres=1080)
if [[ "$display" == "gtk" ]]; then
    gtk_opts="zoom-to-fit=on,show-tabs=off"
    [[ "${LITRIC_FULLSCREEN:-0}" == "1" ]] && gtk_opts="$gtk_opts,full-screen=on"
    disp_args+=(-display "gtk,$gtk_opts")
else
    disp_args+=(-display none)
fi

net_args=()
if [[ "${LITRIC_NET:-1}" != "0" ]]; then
    net_args=(
        -device virtio-net-pci,netdev=n0
        -netdev "user,id=n0"
    )
fi

usb_args=()
if [[ -n "${LITRIC_USB:-}" ]]; then
    # A real USB device, attached to the emulated EHCI controller. usb-host
    # passes the device through untouched: the guest's kernel binds its own
    # driver, and nothing is translated.
    IFS=',' read -r -a usb_spec <<< "$LITRIC_USB"
    usb_args+=(-device "usb-host,bus=ehci.0,${usb_spec[*]}")
fi

# A second serial port on a Unix socket. ttyS1 has a getty on it, so this is a
# real login prompt inside the guest, reachable from the host -- the way to
# check what the running system actually reports (ip addr, iwctl, systemctl
# status) instead of guessing it from the outside. Off by default: a login
# prompt on a socket is a door, and this one is only opened on request.
serial_args=(-serial "file:$serial")
if [[ "${LITRIC_DEBUG_CONSOLE:-0}" == "1" ]]; then
    serial_args+=(
        -chardev "socket,id=dbg,path=$debug_sock,server=on,wait=off"
        -serial chardev:dbg
    )
fi

share_args=()
if [[ -n "$share" ]]; then
    share_path="$(cd "$share" && pwd)"
    share_name="$(basename "$share_path")"
    share_args=(
        -virtfs "local,path=$share_path,mount_tag=$share_name,security_model=none,readonly=on"
    )
fi

echo "desktop-app: booting $disk"
echo "desktop-app: window=$display ram=${memory}MiB smp=$smp net=${LITRIC_NET:-1}"

exec qemu-system-x86_64 \
    -name "LitricOS" \
    -machine q35,accel=kvm:tcg,vmport=off \
    -cpu max \
    -smp "$smp" \
    -m "$memory" \
    -boot order=c \
    -drive "if=pflash,format=raw,unit=0,readonly=on,file=$firmware" \
    -drive "if=pflash,format=raw,unit=1,file=$vars" \
    -drive "if=virtio,format=raw,file=$disk" \
    "${disp_args[@]}" \
    -device usb-ehci,id=ehci \
    -device usb-tablet,bus=ehci.0 \
    -device usb-kbd,bus=ehci.0 \
    "${net_args[@]}" \
    "${share_args[@]}" \
    "${usb_args[@]}" \
    -qmp "unix:$qmp,server,nowait" \
    "${serial_args[@]}" \
    -no-reboot
