#!/usr/bin/env bash
# =============================================================================
# LitricOS — boot with a real display and photograph the screen
#
#   ./tools/screen-test.sh [--profile nano] [--timeout 90] [--at 25]
#
# WHY THIS EXISTS
#
# tools/boot-test.sh runs QEMU with -nographic, which means the guest has no
# display device at all. That is the right harness for "did it boot", and it is
# useless for "is there a desktop on the screen", because there is no screen.
#
# A shell that prints "drew the desktop" on the serial console has proved that
# it reached the end of its own code. It has not proved that a single pixel
# changed. The only evidence that settles it is a picture of the guest's
# framebuffer, taken by the hypervisor, which is what this does:
#
#   -device VGA          give the guest a display adapter, so /dev/fb0 exists
#   -display none        do not open a window on the build host
#   -monitor unix:...    a control socket to ask QEMU for a screenshot
#   screendump           QEMU writes the guest's screen to a PPM
#
# The PPM is converted to PNG by tools/ppm2png.py, which has no dependencies,
# so the picture can be looked at anywhere.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
OUT_DIR="${BUILD_DIR}/out"
SHOT_DIR="${BUILD_DIR}/screens"

PROFILE="nano"
TIMEOUT=120
MEM=1024
SMP=2
POINTER_DEVICE="usb-mouse"
# When to photograph, in seconds after QEMU starts. Several shots are taken so
# that a desktop which appears and then dies is distinguishable from one that
# never appeared -- a single well-timed screenshot can hide both.
SHOT_AT="12 20 30 45"

# Monitor commands to run after the timed screenshots, one per line, as
# `command|label`. A label makes a screenshot; an empty one does not.
INPUT_SCRIPT=""

log()  { printf '\033[1;32m[screen]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[screen]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[screen]\033[0m %s\n' "$*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile) PROFILE="${2:-}"; shift 2 ;;
        --timeout) TIMEOUT="${2:-}"; shift 2 ;;
        --at)      SHOT_AT="${2:-}"; shift 2 ;;
        --input)   INPUT_SCRIPT="${2:-}"; shift 2 ;;
        --mem)     MEM="${2:-}"; shift 2 ;;
        --smp)     SMP="${2:-}"; shift 2 ;;
        # An absolute pointer. With it, the monitor's `mouse_move X Y` places
        # the pointer at screen pixel (X, Y) instead of moving it relatively, so
        # an input script can click a known control rather than guess.
        --tablet)  POINTER_DEVICE="usb-tablet"; shift ;;
        -h|--help) sed -n '2,25p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown argument: $1" ;;
    esac
done

command -v qemu-system-x86_64 >/dev/null 2>&1 || die "qemu-system-x86_64 is not installed"

IMAGE="${OUT_DIR}/litricos.raw"
[[ -f "$IMAGE" ]] || IMAGE="${OUT_DIR}/litricos.raw.zst"
[[ -f "$IMAGE" ]] || die "no image in ${OUT_DIR}. Run tools/build.sh first."

# QEMU cannot boot a zstd stream. Decompress once, next to the image.
if [[ "$IMAGE" == *.zst ]]; then
    RAW="${OUT_DIR}/litricos.screen.raw"
    if [[ ! -f "$RAW" || "$IMAGE" -nt "$RAW" ]]; then
        log "decompressing $(basename "$IMAGE")"
        zstd -d -f -o "$RAW" "$IMAGE" >/dev/null 2>&1 || die "zstd failed"
    fi
    IMAGE="$RAW"
fi

CODE=""
for c in /usr/share/edk2/x64/OVMF_CODE.4m.fd /usr/share/edk2/x64/OVMF_CODE.fd \
         /usr/share/OVMF/OVMF_CODE.fd /usr/share/edk2-ovmf/x64/OVMF_CODE.fd; do
    [[ -f "$c" ]] && { CODE="$c"; break; }
done
[[ -n "$CODE" ]] || die "no OVMF firmware found"

VARS_TEMPLATE=""
for v in /usr/share/edk2/x64/OVMF_VARS.4m.fd /usr/share/edk2/x64/OVMF_VARS.fd \
         /usr/share/OVMF/OVMF_VARS.fd /usr/share/edk2-ovmf/x64/OVMF_VARS.fd; do
    [[ -f "$v" ]] && { VARS_TEMPLATE="$v"; break; }
done
[[ -n "$VARS_TEMPLATE" ]] || die "no OVMF variable store found"

ACCEL="kvm"
[[ -w /dev/kvm ]] || { warn "/dev/kvm not writable; using software emulation"; ACCEL="tcg"; }
CPU_ARGS=(-cpu host); [[ "$ACCEL" == "tcg" ]] && CPU_ARGS=(-cpu max)

STAMP="$(date -u +%Y%m%d-%H%M%S)"
mkdir -p "$SHOT_DIR"
VARS="${BUILD_DIR}/OVMF_VARS.screen.fd"
cp -f "$VARS_TEMPLATE" "$VARS"
LOG="${BUILD_DIR}/screen-${STAMP}.log"
MON="${BUILD_DIR}/qemu-mon-${STAMP}.sock"

log "image  : $(basename "$IMAGE")"
log "shots  : ${SHOT_AT} seconds"
log "log    : ${LOG}"

# -device VGA is the standard Bochs-compatible adapter. The guest kernel binds
# bochs-drm to it, and DRM's fbdev emulation then provides /dev/fb0. virtio-gpu
# would also work and is what a real LitricOS machine would use; VGA is chosen
# here because it needs no guest driver beyond what any kernel already has, so a
# blank screen means the shell failed, not that a driver was missing.
timeout "$TIMEOUT" qemu-system-x86_64 \
    -machine "q35,accel=${ACCEL}" \
    "${CPU_ARGS[@]}" \
    -smp "$SMP" \
    -m "$MEM" \
    -drive "if=pflash,format=raw,unit=0,readonly=on,file=${CODE}" \
    -drive "if=pflash,format=raw,unit=1,file=${VARS}" \
    -drive "if=virtio,format=raw,file=${IMAGE}" \
    -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
    -device VGA,vgamem_mb=32     -device usb-ehci,id=ehci -device "${POINTER_DEVICE},bus=ehci.0" \
    -display none \
    -serial "file:${LOG}" \
    -monitor "unix:${MON},server,nowait" \
    -no-reboot &
QEMU_PID=$!

cleanup() {
    kill "$QEMU_PID" 2>/dev/null || true
    wait "$QEMU_PID" 2>/dev/null || true
    rm -f "$MON"
}
trap cleanup EXIT

# Wait for the monitor socket to exist before trying to talk to it.
for _ in $(seq 1 100); do [[ -S "$MON" ]] && break; sleep 0.1; done
[[ -S "$MON" ]] || die "QEMU never created its monitor socket"

# The monitor client is a separate file rather than an inline heredoc: a Python
# program nested inside a shell heredoc inside a build script is three levels of
# quoting, and the escape that breaks is never the one you check.
monitor() {
    python3 "${SCRIPT_DIR}/qemu-monitor.py" "$MON" "$1"
}

shoot() {
    # Two statements, not one. `local a="$1" b="...$a..."` declares both names
    # before either assignment runs, so under `set -u` the second one reads an
    # unset variable and the whole script dies with "secs: unbound variable"
    # after QEMU has already booted. It did exactly that on the first run.
    local secs="$1"
    local ppm="${SHOT_DIR}/${STAMP}-t${secs}.ppm"
    monitor "screendump ${ppm}" >/dev/null 2>&1 || warn "monitor unreachable at t=${secs}s"
    sleep 0.5
    if [[ -s "$ppm" ]]; then
        python3 "${SCRIPT_DIR}/ppm2png.py" "$ppm" "${ppm%.ppm}.png" \
            && rm -f "$ppm" \
            && log "captured ${STAMP}-t${secs}.png"
    else
        warn "no image at t=${secs}s"
        rm -f "$ppm"
    fi
}

prev=0
for at in $SHOT_AT; do
    sleep "$(( at - prev ))"
    prev="$at"
    kill -0 "$QEMU_PID" 2>/dev/null || { warn "QEMU exited before t=${at}s"; break; }
    shoot "$at"
done

# --- input injection ---------------------------------------------------------
#
# The screenshots above prove the desktop draws. They prove nothing at all about
# whether it responds, and a desktop that draws but does not respond is exactly
# what this whole layer was built to stop being.
#
# QEMU's monitor can press keys and move the emulated PS/2 mouse. Driving the
# guest through it means the input path being tested is the real one: the
# kernel's evdev nodes, our device discovery, our scancode tables, our shortcut
# resolver, our window manager. Nothing is stubbed and nothing is simulated on
# our side of the wire.
if [[ -n "$INPUT_SCRIPT" ]]; then
    log "driving input"
    step=0
    while IFS='|' read -r command label; do
        [[ -z "$command" ]] && continue
        step=$(( step + 1 ))
        case "$command" in
            wait\ *) sleep "${command#wait }" ;;
            *) monitor "$command" >/dev/null 2>&1 || warn "monitor refused: $command" ;;
        esac
        if [[ -n "$label" ]]; then
            sleep 1
            shoot "in${step}-${label}"
        fi
    done <<< "$INPUT_SCRIPT"
fi

kill "$QEMU_PID" 2>/dev/null || true
wait "$QEMU_PID" 2>/dev/null || true
trap - EXIT
rm -f "$MON"

echo
log "serial log tail:"
tail -40 "$LOG" 2>/dev/null | sed 's/^/    /'
echo
log "screens in ${SHOT_DIR}:"
ls -1 "${SHOT_DIR}"/${STAMP}-*.png 2>/dev/null | sed 's/^/    /' || warn "none"
