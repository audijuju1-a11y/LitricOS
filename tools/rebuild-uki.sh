#!/usr/bin/env bash
set -euo pipefail

UKI_DIR=/tmp/esp-mount/EFI/Linux
UKI_SRC=$(ls "$UKI_DIR"/*.efi 2>/dev/null | head -1)
INITRD=/root/litricos/build/out/initrd.cpio.zst
WORK=/tmp/uki-work
mkdir -p "$WORK"

echo "UKI: $UKI_SRC"
echo "Size: $(ls -lh "$UKI_SRC" | awk '{print $5}')"

# Extract kernel
echo "Extracting kernel..."
ukify inspect "$UKI_SRC" --section=.linux -o "$WORK/vmlinuz"
echo "Kernel: $(ls -lh "$WORK/vmlinuz" | awk '{print $5}')"

# Get cmdline
CMDLINE=$(ukify inspect "$UKI_SRC" --show-cmdline 2>/dev/null || echo "")
echo "Cmdline: ${CMDLINE:0:100}..."

# Get os-release content
ukify inspect "$UKI_SRC" --show-os-release > "$WORK/os-release" 2>/dev/null || true

# Rebuild UKI with trimmed initrd
echo "Rebuilding UKI with trimmed initrd..."
BUILD_DIR=/root/litricos
SECUREBOOT_KEY=${BUILD_DIR}/keys/dev/secureboot.key
SECUREBOOT_CERT=${BUILD_DIR}/keys/dev/secureboot.crt

UKIFY_ARGS=(ukify build
    --linux="$WORK/vmlinuz"
    --initrd="$INITRD"
    --os-release="$WORK/os-release"
    --output="$WORK/litricos-trimmed.efi")

[[ -n "$CMDLINE" ]] && UKIFY_ARGS+=(--cmdline="$CMDLINE")

if [[ -f "$SECUREBOOT_KEY" ]]; then
    UKIFY_ARGS+=(--efi-boot-key="$SECUREBOOT_KEY" --efi-boot-certificate="$SECUREBOOT_CERT" --sign-kernel=yes)
fi

"${UKIFY_ARGS[@]}" 2>&1
echo "New UKI: $(ls -lh "$WORK/litricos-trimmed.efi" | awk '{print $5}')"

# Replace in ESP
cp "$WORK/litricos-trimmed.efi" "$UKI_SRC"
echo "UKI replaced in ESP!"

# Verify
echo "Final UKI: $(ls -lh "$UKI_SRC" | awk '{print $5}')"
