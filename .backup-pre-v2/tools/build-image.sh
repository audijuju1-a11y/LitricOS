#!/usr/bin/env bash
# =============================================================================
# ILTRIC OS — Image Build Script
# Builds the Iltric disk image from mkosi configuration
#
# Prerequisites:
#   - Linux host (or WSL2)
#   - mkosi (systemd ≥ 256)
#   - qemu-img (for qcow2 conversion)
#   - sbsigntool (for UKI signing, optional for dev)
#
# Usage:
#   ./tools/build-image.sh              # full build
#   ./tools/build-image.sh --rebuild    # clean + full build
#   ./tools/build-image.sh --quick      # skip verity for dev testing
#
# STATUS: DESIGN-ONLY — not yet tested against real build
# =============================================================================

set -euo pipefail

# Project root (one level up from tools/)
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
IMAGE_DIR="${PROJECT_ROOT}/image"
OUTPUT_RAW="${BUILD_DIR}/iltric.raw"
OUTPUT_QCOW2="${BUILD_DIR}/iltric.qcow2"
LAST_GOOD="${BUILD_DIR}/iltric.last-good.qcow2"
OVMF_CODE="${BUILD_DIR}/OVMF_CODE_4M.fd"
OVMF_VARS="${BUILD_DIR}/OVMF_VARS_4M.fd"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[BUILD]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
err() { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

# =============================================================================
# PREFLIGHT CHECKS
# =============================================================================

preflight() {
    log "Running preflight checks..."
    
    # Check we're on Linux
    if [[ "$(uname -s)" != "Linux" ]]; then
        err "This script must run on Linux (or WSL2). Detected: $(uname -s)"
    fi
    
    # Check for mkosi
    if ! command -v mkosi &>/dev/null; then
        err "mkosi not found. Install: sudo pacman -S mkosi (or from systemd-stable)"
    fi
    
    # Check for qemu-img
    if ! command -v qemu-img &>/dev/null; then
        warn "qemu-img not found. Install: sudo pacman -S qemu-system-x86"
    fi
    
    # Verify project root
    if [[ ! -f "${PROJECT_ROOT}/00-MASTER-WORKFLOW.md" ]]; then
        err "Not running from project root. Expected 00-MASTER-WORKFLOW.md at ${PROJECT_ROOT}"
    fi
    
    log "Preflight OK"
}

# =============================================================================
# DOWNLOAD OVMF FIRMWARE (4 MB split variant)
# =============================================================================

download_ovmf() {
    log "Checking OVMF firmware..."
    
    if [[ -f "${OVMF_CODE}" && -f "${OVMF_VARS}" ]]; then
        log "OVMF already present"
        return 0
    fi
    
    log "Downloading OVMF 4MB split firmware..."
    mkdir -p "${BUILD_DIR}"
    
    # Arch Linux package — extract OVMF files
    if command -v pacman &>/dev/null; then
        local tmppkg
        tmppkg=$(mktemp -d)
        pacman -Sw --noconfirm ovmf --cachedir="${tmppkg}" 2>/dev/null || true
        local ovmf_pkg
        ovmf_pkg=$(find "${tmppkg}" -name "ovmf*.pkg.tar*" | head -1)
        if [[ -n "${ovmf_pkg}" ]]; then
            tar -xf "${ovmf_pkg}" -C "${tmppkg}"
            cp "${tmppkg}/usr/share/ovmf/x64/OVMF_CODE_4M.fd" "${OVMF_CODE}"
            cp "${tmppkg}/usr/share/ovmf/x64/OVMF_VARS_4M.fd" "${OVMF_VARS}"
            rm -rf "${tmppkg}"
            log "OVMF extracted from Arch package"
            return 0
        fi
    fi
    
    # Fallback: download from TianoCore / Arch mirrors
    warn "Could not extract OVMF from local packages"
    warn "Download manually from: https://archlinux.org/packages/extra/any/ovmf/"
    warn "Place OVMF_CODE_4M.fd and OVMF_VARS_4M.fd in ${BUILD_DIR}/"
    return 1
}

# =============================================================================
# BUILD IMAGE WITH MKOSI
# =============================================================================

build_image() {
    log "Building image with mkosi..."
    mkdir -p "${BUILD_DIR}"
    
    # Save last-good if it exists
    if [[ -f "${OUTPUT_QCOW2}" ]]; then
        cp "${OUTPUT_QCOW2}" "${LAST_GOOD}"
        log "Saved last-good: ${LAST_GOOD}"
    fi
    
    cd "${IMAGE_DIR}"
    
    # Build the image
    mkosi build \
        --output "${OUTPUT_RAW}" \
        2>&1 | tee "${BUILD_DIR}/build-$(date +%Y%m%d-%H%M%S).log"
    
    cd "${PROJECT_ROOT}"
    
    log "Image built: ${OUTPUT_RAW}"
}

# =============================================================================
# CONVERT TO QCOW2
# =============================================================================

convert_qcow2() {
    log "Converting to qcow2..."
    
    if [[ ! -f "${OUTPUT_RAW}" ]]; then
        err "Raw image not found: ${OUTPUT_RAW}"
    fi
    
    qemu-img convert -f raw -O qcow2 "${OUTPUT_RAW}" "${OUTPUT_QCOW2}"
    
    log "Converted: ${OUTPUT_QCOW2}"
    
    # Record hash for reproducibility
    local hash
    hash=$(sha256sum "${OUTPUT_QCOW2}" | cut -d' ' -f1)
    echo "${hash}  iltric.qcow2" > "${BUILD_DIR}/iltric.qcow2.sha256"
    log "SHA256: ${hash}"
}

# =============================================================================
# RECORD MEASUREMENTS
# =============================================================================

record_measurements() {
    log "Recording measurements..."
    
    {
        echo "# Build: $(date -Iseconds)"
        echo "# Host: $(uname -n) / $(uname -s) $(uname -r)"
        echo ""
        
        if [[ -f "${OUTPUT_QCOW2}" ]]; then
            echo "Image size (qcow2): $(du -h "${OUTPUT_QCOW2}" | cut -f1)"
        fi
        if [[ -f "${OUTPUT_RAW}" ]]; then
            echo "Image size (raw):   $(du -h "${OUTPUT_RAW}" | cut -f1)"
        fi
        if [[ -f "${BUILD_DIR}/iltric.qcow2.sha256" ]]; then
            echo "SHA256: $(cat "${BUILD_DIR}/iltric.qcow2.sha256")"
        fi
        
    } > "${BUILD_DIR}/measurements.txt"
    
    log "Measurements written to ${BUILD_DIR}/measurements.txt"
}

# =============================================================================
# MAIN
# =============================================================================

main() {
    local mode="${1:-}"
    
    log "========================================="
    log "ILTRIC OS — Image Build Pipeline"
    log "========================================="
    log "Project root: ${PROJECT_ROOT}"
    log "Build dir:    ${BUILD_DIR}"
    log "Mode:         ${mode:-full}"
    
    preflight
    
    if [[ "${mode}" == "--rebuild" ]]; then
        log "Cleaning build directory..."
        rm -rf "${BUILD_DIR}"
        mkdir -p "${BUILD_DIR}"
    fi
    
    download_ovmf
    build_image
    convert_qcow2
    record_measurements
    
    log "========================================="
    log "BUILD COMPLETE"
    log "========================================="
    log "Image: ${OUTPUT_QCOW2}"
    log "Run with: ./tools/run-qemu.sh"
    log "========================================="
}

main "$@"
