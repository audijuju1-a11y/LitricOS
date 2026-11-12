#!/usr/bin/env bash
# =============================================================================
# run-qemu-gui.sh — open LitricOS in a visible window (WSLg) for real use.
#
# Difference from the headless verification runs: the window is real, the
# pointer/keyboard are yours, and audio goes through WSLg's PulseAudio server.
# The USB tablet's absolute range is what the guest measures with EVIOCGABS
# (0xFFFF — the same constant photo_tour_lib.py scales to and litric-input
# defaults to), so clicks land exactly where you aim.
#
#   bash tools/run-qemu-gui.sh          # windowed, sound, 4 vCPU / 2 GiB
#   bash tools/run-qemu-gui.sh 4096     # windowed with 4 GiB RAM
#
# While the window is open, the guest's diagnostic console is reachable at
# build/gui-serial.sock (e.g. `socat - UNIX-CONNECT:build/gui-serial.sock`).
# =============================================================================
set -euo pipefail

RAM_MB="${1:-2048}"

cd "$(dirname "$0")/.."
mkdir -p build

IMG="build/out/litricos.raw"
ZST="build/out/litricos.raw.zst"

# Decompress the disposable image once; reuse it across boots.
if [ ! -f "$IMG" ] || [ "$IMG" -ot "$ZST" ]; then
  echo "[gui] decompressing image (once, ~30s)..."
  zstd -d -f "$ZST" -o "$IMG"
fi

SOCK="build/gui-qmp.sock"
[ -e "$SOCK" ] && rm -f "$SOCK"
DSOCK="build/gui-serial.sock"
[ -e "$DSOCK" ] && rm -f "$DSOCK"

# WSLg ships a PulseAudio server at /mnt/wslg/PulseServer; attach QEMU to it
# when present so the guest's bell/notify sounds are audible. Absent (plain
# WSL without WSLg) the flag is simply dropped.
AUD=()
if [ -S /mnt/wslg/PulseServer ]; then
  AUD=(-audiodev pa,id=snd0,server=/mnt/wslg/PulseServer -device intel-hda -device hda-duplex,audiodev=snd0)
fi

# Seed the UEFI vars from the firmware's template once.
VARS="build/OVMF_VARS.gui.fd"
if [ ! -f "$VARS" ]; then
  cp /usr/share/edk2/x64/OVMF_VARS.4m.fd "$VARS"
fi

exec qemu-system-x86_64 \
  -machine q35,accel=kvm \
  -cpu host -smp "$(nproc)" -m "$RAM_MB" \
  -drive if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
  -drive if=pflash,format=raw,unit=1,file="$VARS" \
  -drive if=virtio,format=raw,file="$IMG" \
  -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
  -device VGA,vgamem_mb=32 \
  -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 \
  "${AUD[@]}" \
  -serial file:build/gui-serial.log \
  -chardev "socket,id=diag,path=$DSOCK,server=on,wait=off,logfile=build/gui-serial.log" \
  -serial chardev:diag \
  -qmp "unix:$SOCK,server,nowait" \
  -no-reboot
