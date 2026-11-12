#!/usr/bin/env bash
# =============================================================================
# LitricOS — boot the built image with a real, interactive window
#
#   ./tools/show-desktop.sh                  # window on your screen
#   ./tools/show-desktop.sh --mem 2048       # more RAM for the guest
#   ./tools/show-desktop.sh --backend sdl    # different UI backend (default: gtk)
#   ./tools/show-desktop.sh --tcg            # force software emulation
#
# WHY THIS EXISTS
#
# tools/boot-test.sh proves the boot and tools/screen-test.sh photographs the
# screen, but neither lets a human touch anything. This one opens QEMU as a
# real window on the host desktop (WSLg presents it on Windows like any other
# app) with an absolute pointer, so the LitricOS desktop, its dock, its
# switcher and its windows can be used with the machine's own mouse and
# keyboard.
#
# Safety is the same as every launcher here: the image lives under the
# project's build/ tree, no host device is ever opened, and the guest network
# is user-mode only.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${PROJECT_ROOT}/build/out"
LOG_DIR="${PROJECT_ROOT}/build"

MEM=2048
SMP=4
BACKEND="gtk"
ACCEL="kvm"
IMAGE=""

log()  { printf '\033[1;32m[show]\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31m[show]\033[0m %s\n' "$*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --mem)     MEM="${2:?}"; shift 2 ;;
        --smp)     SMP="${2:?}"; shift 2 ;;
        --backend) BACKEND="${2:?}"; shift 2 ;;
        --tcg)     ACCEL="tcg"; shift ;;
        --image)   IMAGE="${2:?}"; shift 2 ;;
        -h|--help) sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *)         die "unknown argument: $1 (try --help)" ;;
    esac
done

command -v qemu-system-x86_64 >/dev/null 2>&1 || die "qemu-system-x86_64 not found"

# --- image -------------------------------------------------------------------
if [[ -z "$IMAGE" ]]; then
    IMAGE="${OUT_DIR}/litricos.raw"
    if [[ ! -f "$IMAGE" && -f "${IMAGE}.zst" ]]; then
        log "decompressing $(basename "${IMAGE}.zst")"
        zstd -d -f -o "$IMAGE" "${IMAGE}.zst" >/dev/null 2>&1 || die "zstd failed"
    fi
fi
[[ -f "$IMAGE" ]] || die "no image at ${IMAGE} -- run tools/build.sh first"

# --- firmware ----------------------------------------------------------------
find_fw() {
    local name="$1" c
    for c in "/usr/share/edk2/x64/${name}" "/usr/share/edk2-ovmf/x64/${name}" \
             "/usr/share/OVMF/${name}"; do
        [[ -f "$c" ]] && { printf '%s' "$c"; return 0; }
    done
    return 1
}
CODE="$(find_fw OVMF_CODE.4m.fd)" || die "OVMF firmware not found"
VARS_TEMPLATE="$(find_fw OVMF_VARS.4m.fd)" || die "OVMF variable store not found"

STAMP="$(date -u +%Y%m%d-%H%M%S)"
VARS="${LOG_DIR}/OVMF_VARS.show.fd"
cp -f "$VARS_TEMPLATE" "$VARS"
LOG="${LOG_DIR}/show-${STAMP}.log"

if [[ "$ACCEL" == "kvm" && ! -w /dev/kvm ]]; then
    log "/dev/kvm not writable -- using software emulation (slower)"
    ACCEL="tcg"
fi

# The display device, chosen by what this QEMU actually has.
#
# virtio-gpu-pci is the right device for a real machine and is what this script
# asked for unconditionally -- and a QEMU built without it (Arch's qemu-base,
# which is what the WSL builder has) exits with "'virtio-gpu-pci' is not a valid
# device model name" before the firmware even runs, which reads like a broken
# image rather than a missing device. VGA is the Bochs adapter every kernel can
# drive through bochs-drm, and the guest kernel here has the fbdev emulation for
# it, so the desktop lands on the same framebuffer either way.
GPU=(-device virtio-gpu-pci,xres=1280,yres=800)
if ! qemu-system-x86_64 -device help 2>/dev/null | grep -q '"virtio-gpu-pci"'; then
    log "virtio-gpu-pci is not in this QEMU; using VGA for the guest display"
    GPU=(-device VGA,vgamem_mb=32)
fi
CPU_ARGS=(-cpu host); [[ "$ACCEL" == "tcg" ]] && CPU_ARGS=(-cpu max)

# WSLg quirk: the Wayland/EGL path has shown black or invisible windows here
# (Mesa logs a DRI3 error and the surface never maps). XWayland with software
# rendering is the reliable route, so both toolkits are pinned to X11 before
# any window exists.
export GDK_BACKEND=x11
export SDL_VIDEODRIVER=x11

log "image   : $(basename "$IMAGE")"
log "display : ${BACKEND} (a window should open on your desktop)"
log "accel   : ${ACCEL}   cpus: ${SMP}   ram: ${MEM}M"
log "serial  : ${LOG}"

# The window is the product here. The tablet device is what makes the pointer
# absolute -- without it the guest sees a relative PS/2 mouse and the desktop
# feels like a remote desktop from 2003. The serial console is logged but not
# attached to the terminal: the terminal is free for Ctrl+C to actually mean
# "stop the VM". A monitor socket is bound next to the log so scripts can
# screendump or query the guest without touching the interactive window.
#
# vmport=off: without it the guest's vmmouse driver binds the i8042 AUX
# port as absolute pointing devices (range 0-4095) alongside the USB
# tablet (0-32767). Two pointers feed one InputState; the vmmouse stream
# never moves, so the desktop cursor sticks in a corner. Killing vmport
# removes the phantom devices; the tablet alone is the single pointer.
# (The comment lives ABOVE the command: a `#` inside a backslash-continuation
# would truncate the exec line mid-argument.)
exec qemu-system-x86_64 \
    -name LitricOS \
    -machine "q35,accel=${ACCEL},vmport=off" \
    "${CPU_ARGS[@]}" \
    -smp "$SMP" \
    -m "$MEM" \
    -boot order=c \
    -drive "if=pflash,format=raw,unit=0,readonly=on,file=${CODE}" \
    -drive "if=pflash,format=raw,unit=1,file=${VARS}" \
    -drive "if=virtio,format=raw,file=${IMAGE}" \
    -device virtio-net-pci,netdev=n0,addr=0x0a -netdev user,id=n0 \
    "${GPU[@]}" \
    -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 -device usb-kbd,bus=ehci.0 \
    -display "$BACKEND" \
    -monitor "unix:${LOG_DIR}/show-monitor.sock,server,nowait" \
    -serial "file:${LOG}" \
    -no-reboot
