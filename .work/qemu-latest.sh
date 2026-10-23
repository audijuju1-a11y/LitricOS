#!/usr/bin/env bash
# Boot the freshly built LitricOS image in a WSLg window.
set -euo pipefail
cd /root/litric-latest
mkdir -p build

IMG=build/out/litricos.raw
VARS=build/OVMF_VARS.gui.fd
[ -f "$VARS" ] || cp /usr/share/edk2/x64/OVMF_VARS.4m.fd "$VARS"
rm -f build/gui-qmp.sock build/gui-serial.sock

exec qemu-system-x86_64 \
  -machine q35,accel=kvm \
  -cpu host -smp "$(nproc)" -m 4096 \
  -drive if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
  -drive if=pflash,format=raw,unit=1,file="$VARS" \
  -drive if=virtio,format=raw,file="$IMG" \
  -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
  -device VGA,vgamem_mb=32 \
  -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 \
  -serial file:build/gui-serial.log \
  -chardev "socket,id=diag,path=build/gui-serial.sock,server=on,wait=off,logfile=build/gui-serial.log" \
  -serial chardev:diag \
  -qmp "unix:build/gui-qmp.sock,server,nowait" \
  -no-reboot
