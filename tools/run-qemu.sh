#!/usr/bin/env bash
# =============================================================================
# LITRIC OS — QEMU Launcher (Linux/Mac Host)
# Launches Litric in QEMU with KVM acceleration
#
# Prerequisites:
#   - QEMU (qemu-system-x86_64)
#   - OVMF_CODE_4M.fd and OVMF_VARS_4M.fd in build/
#   - litric.qcow2 in build/
#
# Usage:
#   ./tools/run-qemu.sh              # Normal boot
#   ./tools/run-qemu.sh --debug      # GDB stub (frozen at start)
#   ./tools/run-qemu.sh --tcg        # TCG fallback (no KVM)
#   ./tools/run-qemu.sh --smp 1      # Single core
#   ./tools/run-qemu.sh --mem 8G     # 8 GB RAM
#
# SAFETY: Never touches host physical disks. All paths relative to project dir.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
LOGS_DIR="${SCRIPT_DIR}/logs"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
LOG_FILE="${LOGS_DIR}/boot-${TIMESTAMP}.log"

# Defaults
SMP=4
MEM="4G"
DEBUG=false
TCG=false
SSH_PORT=2222
IMAGE=""

# =============================================================================
# PARSE ARGUMENTS
# =============================================================================

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)   DEBUG=true; shift ;;
        --tcg)     TCG=true; shift ;;
        --smp)     SMP="$2"; shift 2 ;;
        --mem)     MEM="$2"; shift 2 ;;
        --port)    SSH_PORT="$2"; shift 2 ;;
        --image)   IMAGE="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--debug] [--tcg] [--smp N] [--mem SIZE] [--port PORT] [--image PATH]"
            exit 0
            ;;
        *)         echo "Unknown option: $1"; exit 1 ;;
    esac
done

# =============================================================================
# PREFLIGHT
# =============================================================================

# Verify project root
if [[ ! -f "${PROJECT_ROOT}/00-MASTER-WORKFLOW.md" ]]; then
    echo "[ERROR] Not running from project root." >&2
    exit 1
fi

# Image path
if [[ -z "$IMAGE" ]]; then
    IMAGE="${BUILD_DIR}/litric.qcow2"
fi

# Safety: assert image is inside project dir
RESOLVED_IMAGE="$(cd "$(dirname "$IMAGE")" && pwd)/$(basename "$IMAGE")"
if [[ ! "$RESOLVED_IMAGE" == "$PROJECT_ROOT"* ]]; then
    echo "[ERROR] SAFETY: Image must be inside project directory!" >&2
    echo "  Image: $RESOLVED_IMAGE" >&2
    echo "  Project: $PROJECT_ROOT" >&2
    exit 1
fi

if [[ ! -f "$RESOLVED_IMAGE" ]]; then
    echo "[ERROR] Image not found: $RESOLVED_IMAGE" >&2
    echo "  Run: ./tools/build-image.sh" >&2
    exit 1
fi

# OVMF firmware
OVMF_CODE="${BUILD_DIR}/OVMF_CODE_4M.fd"
OVMF_VARS="${BUILD_DIR}/OVMF_VARS_4M.fd"

if [[ ! -f "$OVMF_CODE" || ! -f "$OVMF_VARS" ]]; then
    echo "[ERROR] OVMF firmware not found in $BUILD_DIR/" >&2
    echo "  Download OVMF_CODE_4M.fd and OVMF_VARS_4M.fd" >&2
    exit 1
fi

# Create logs directory
mkdir -p "$LOGS_DIR"

# =============================================================================
# BUILD QEMU COMMAND
# =============================================================================

QEMU_ARGS=(
    -machine "q35,accel=kvm,kernel-irqchip=off"
    -cpu max
    -smp "$SMP"
    -m "$MEM"
    
    # Firmware (4 MB split OVMF)
    -drive "if=pflash,format=raw,unit=0,readonly=on,file=${OVMF_CODE}"
    -drive "if=pflash,format=raw,unit=1,file=${OVMF_VARS}"
    
    # Disk (virtio)
    -drive "if=virtio,format=qcow2,file=${RESOLVED_IMAGE}"
    
    # Network (user-mode with SSH forwarding)
    -device "virtio-net-pci,netdev=n0"
    -netdev "user,id=n0,hostfwd=tcp::${SSH_PORT}-:22"
    
    # Serial console
    -serial mon:stdio
    
    # No display (headless)
    -nographic
)

# TCG fallback
if [[ "$TCG" == "true" ]]; then
    QEMU_ARGS+=(-accel tcg)
    SMP=1
    QEMU_ARGS[4]="1"  # Update -smp
    echo "[LITRIC] Using TCG fallback (SMP=$SMP)"
fi

# Debug mode
if [[ "$DEBUG" == "true" ]]; then
    QEMU_ARGS+=(-s -S)
    echo "[LITRIC] Debug mode: GDB stub on :1234, VM frozen"
    echo "[LITRIC] Connect: gdb vmlinux -ex 'target remote :1234'"
    echo "[LITRIC] Boot with nokaslr for stable breakpoints"
fi

# =============================================================================
# LAUNCH
# =============================================================================

echo "========================================="
echo "LITRIC OS — QEMU Launcher"
echo "========================================="
echo "Image:    $RESOLVED_IMAGE"
echo "Firmware: $OVMF_CODE"
echo "SMP:      $SMP"
echo "Memory:   $MEM"
echo "SSH:      localhost:$SSH_PORT -> guest:22"
echo "Log:      $LOG_FILE"
echo "========================================="

# Run QEMU, tee output to log
qemu-system-x86_64 "${QEMU_ARGS[@]}" 2>&1 | tee "$LOG_FILE"

echo ""
echo "[LITRIC] QEMU exited. Log: $LOG_FILE"
