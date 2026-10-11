#!/usr/bin/env bash
# =============================================================================
# LitricOS — boot the built image in QEMU and capture the serial console
#
#   ./tools/boot-test.sh                  boot, 180 s cap
#   ./tools/boot-test.sh --timeout 300    longer cap
#   ./tools/boot-test.sh --secureboot     boot with Secure Boot firmware
#   ./tools/boot-test.sh --tcg            software emulation, no KVM
#
# Writes the full serial log to build/boot-<timestamp>.log and prints a verdict.
#
# This exists as a script rather than an inline command because every attempt to
# run QEMU through nested shell quoting lost the output redirect to MSYS path
# translation. A file on disk has no quoting problem.
# =============================================================================
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${PROJECT_ROOT}/build/out"
LOG_DIR="${PROJECT_ROOT}/build"

TIMEOUT=180
SECUREBOOT=0
ACCEL="kvm"
MEM=1024
SMP=2
IMAGE=""

log()  { printf '\033[1;32m[boot]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[boot]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[boot]\033[0m %s\n' "$*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --timeout) TIMEOUT="${2:?}"; shift 2 ;;
        --secureboot) SECUREBOOT=1; shift ;;
        --tcg) ACCEL="tcg"; shift ;;
        --mem) MEM="${2:?}"; shift 2 ;;
        --smp) SMP="${2:?}"; shift 2 ;;
        --image) IMAGE="${2:?}"; shift 2 ;;
        *) die "unknown argument: $1" ;;
    esac
done

command -v qemu-system-x86_64 >/dev/null 2>&1 || die "qemu-system-x86_64 not found"

# --- locate the image --------------------------------------------------------
if [[ -z "$IMAGE" ]]; then
    IMAGE="${OUT_DIR}/litricos.raw"
    if [[ ! -f "$IMAGE" ]]; then
        [[ -f "${IMAGE}.zst" ]] || die "no image at ${IMAGE}[.zst] -- run tools/build.sh first"
        log "decompressing $(basename "${IMAGE}.zst")"
        zstd -d -f -o "$IMAGE" "${IMAGE}.zst" >/dev/null 2>&1 || die "zstd failed"
    fi
fi
[[ -f "$IMAGE" ]] || die "image not found: $IMAGE"

# --- firmware ----------------------------------------------------------------
find_fw() {
    local name="$1" c
    for c in "/usr/share/edk2/x64/${name}" "/usr/share/edk2-ovmf/x64/${name}" \
             "/usr/share/OVMF/${name}"; do
        [[ -f "$c" ]] && { printf '%s' "$c"; return 0; }
    done
    return 1
}

if (( SECUREBOOT )); then
    CODE="$(find_fw OVMF_CODE.secboot.4m.fd)" || die "Secure Boot firmware not found"
    log "firmware: Secure Boot enabled"
else
    CODE="$(find_fw OVMF_CODE.4m.fd)" || die "OVMF firmware not found"
fi
VARS_TEMPLATE="$(find_fw OVMF_VARS.4m.fd)" || die "OVMF variable template not found"

# A writable copy of the variable store, per run. Reusing one across runs means
# a Secure Boot enrolment from a previous boot silently changes this one.
VARS="${LOG_DIR}/OVMF_VARS.run.fd"
cp -f "$VARS_TEMPLATE" "$VARS" || die "could not copy the OVMF variable store"

# --- accel -------------------------------------------------------------------
if [[ "$ACCEL" == "kvm" && ! -w /dev/kvm ]]; then
    warn "/dev/kvm is not writable; falling back to software emulation (slower)"
    ACCEL="tcg"
fi
CPU_ARGS=(-cpu host)
[[ "$ACCEL" == "tcg" ]] && CPU_ARGS=(-cpu max)

STAMP="$(date -u +%Y%m%d-%H%M%S)"
LOG="${LOG_DIR}/boot-${STAMP}.log"
mkdir -p "$LOG_DIR"

log "image   : $(basename "$IMAGE") ($(numfmt --to=iec < <(stat -c%s "$IMAGE")))"
log "accel   : ${ACCEL}   cpus: ${SMP}   ram: ${MEM}M"
log "timeout : ${TIMEOUT}s"
log "log     : ${LOG}"

# -no-reboot so a panic-triggered reboot ends the run instead of looping.
# -no-shutdown is deliberately NOT set: a clean poweroff should end the run.
timeout "$TIMEOUT" qemu-system-x86_64 \
    -machine "q35,accel=${ACCEL}" \
    "${CPU_ARGS[@]}" \
    -smp "$SMP" \
    -m "$MEM" \
    -drive "if=pflash,format=raw,unit=0,readonly=on,file=${CODE}" \
    -drive "if=pflash,format=raw,unit=1,file=${VARS}" \
    -drive "if=virtio,format=raw,file=${IMAGE}" \
    -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
    -nographic -serial mon:stdio -no-reboot \
    > "$LOG" 2>&1
QEMU_RC=$?

LINES=$(wc -l < "$LOG" 2>/dev/null || echo 0)
log "qemu exit ${QEMU_RC}, ${LINES} lines captured"

# --- verdict -----------------------------------------------------------------
# Ordered most-severe first: a panic is a panic even if a shell prompt scrolled
# past earlier.
verdict="UNKNOWN"
if   grep -qiE 'Kernel panic|BUG: unable to handle|Oops:' "$LOG"; then verdict="PANIC"
elif grep -qiE 'Attempted to kill init'                     "$LOG"; then verdict="INIT-DIED"
elif grep -qiE 'You are in emergency mode|emergency\.target' "$LOG"; then verdict="EMERGENCY"
elif grep -qiE 'login:|# $|Welcome to|Reached target'        "$LOG"; then verdict="USERSPACE"
elif grep -qE  '\[    [0-9]+\.[0-9]+\]'                      "$LOG"; then verdict="KERNEL-ONLY"
elif grep -qiE 'BdsDxe: starting'                            "$LOG"; then verdict="BOOTLOADER-ONLY"
elif grep -qiE 'PXE|No valid offer'                          "$LOG"; then verdict="NO-BOOTABLE-MEDIA"
fi

printf '\n\033[1;36m==> VERDICT: %s\033[0m\n\n' "$verdict"

case "$verdict" in
    USERSPACE)  log "userspace was reached" ;;
    KERNEL-ONLY) warn "the kernel ran but userspace was not observed on the serial console" ;;
    BOOTLOADER-ONLY) warn "the bootloader started but the kernel produced no output" ;;
    NO-BOOTABLE-MEDIA) warn "the firmware found nothing bootable -- check the ESP" ;;
    PANIC|INIT-DIED|EMERGENCY) warn "boot failed: ${verdict}" ;;
    *) warn "could not classify the boot" ;;
esac

echo "--- last 40 lines ---"
tail -40 "$LOG"

# A verdict of USERSPACE is the only success.
[[ "$verdict" == "USERSPACE" ]]
