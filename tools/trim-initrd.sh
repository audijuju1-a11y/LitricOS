#!/usr/bin/env bash
# =============================================================================
# Trim initrd for nano profile — remove binaries not needed for boot
# =============================================================================
set -euo pipefail

OUT_DIR="$1"

say()  { printf '\033[1;36m[trim-initrd]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[trim-initrd]\033[0m %s\n' "$*" >&2; }

INITRD="${OUT_DIR}/initrd.cpio.zst"
[[ -f "$INITRD" ]] || { warn "initrd not found"; exit 0; }

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Extract
say "extracting initrd..."
zstd -d -c "$INITRD" | cpio -idm -D "$TMP" 2>/dev/null

BEFORE=$(du -sb "$TMP" | awk '{print $1}')
say "before: $(( BEFORE / 1024 ))KB, $(find "$TMP" -type f | wc -l) files"

# Remove unnecessary binaries
for bin in \
    sqlite3 sqlite3_rsync sqlite3_expert sqldiff showstat4 showdb \
    index_usage dbhash dbdump captree \
    btrfs btrfs-convert mkfs.btrfs btrfstune btrfs-image \
    btrfs-select-super btrfs-map-logical btrfs-find-root \
    xfs_repair xfs_db mkfs.xfs \
    openssl \
    gettext xgettext msgfmt msgmerge msgcat msgunfmt msgconv msgfilter \
    strace ltrace perf pahole \
    tar cpio gzip bzip2 xz zstd lz4 unzip zip \
    curl wget \
    svim ed ex rvim rview view \
    ; do
    rm -f "$TMP/usr/bin/$bin" 2>/dev/null || true
done

# Remove directories
rm -rf "$TMP/usr/lib/udev/hwdb.bin" "$TMP/usr/lib/udev/hwdb.d" 2>/dev/null || true
rm -rf "$TMP/usr/share/file" 2>/dev/null || true
rm -rf "$TMP/usr/lib/systemd/boot/efi" 2>/dev/null || true
rm -rf "$TMP/usr/share/locale" "$TMP/usr/share/man" "$TMP/usr/share/info" \
       "$TMP/usr/share/doc" "$TMP/usr/share/bash-completion" 2>/dev/null || true

# Remove empty dirs
find "$TMP" -type d -empty -delete 2>/dev/null || true

AFTER=$(du -sb "$TMP" | awk '{print $1}')
SAVED=$(( (BEFORE - AFTER) / 1024 ))
say "after: $(( AFTER / 1024 ))KB, $(find "$TMP" -type f | wc -l) files (saved ${SAVED}KB)"

# Recompress
say "recompressing..."
(cd "$TMP" && find . -print0 | cpio -o -0 --quiet 2>/dev/null | zstd -19 -T0 > "${OUT_DIR}/initrd.trimmed.zst")
mv "${OUT_DIR}/initrd.trimmed.zst" "$INITRD"
say "final initrd: $(( $(stat -c%s "$INITRD") / 1024 ))KB"
