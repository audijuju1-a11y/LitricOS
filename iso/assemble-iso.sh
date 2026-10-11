#!/usr/bin/env bash
# =============================================================================
# LitricOS — live ISO assembly
#
# Takes the split artifacts produced by mkosi and wraps them into a bootable
# UEFI ISO 9660 image with a GPT-appended EFI System Partition.
#
# INPUTS  (from build/out/, produced by `mkosi --split-artifacts`)
#   litricos_<ver>.efi              signed Unified Kernel Image
#   litricos_<ver>.system.raw       erofs system image
#   litricos_<ver>.verity.raw       dm-verity Merkle tree
#   litricos_<ver>.verity-sig.raw   signed verity root hash
#   litricos_<ver>.roothash         verity root hash, hex
#
# OUTPUT
#   build/out/litricos-<profile>-<ver>-x86_64.iso
#   build/out/litricos-<profile>-<ver>-x86_64.iso.sha256
#
# WHY THIS SHAPE
#   The ISO is not a squashfs-and-hope live image. It carries the *same*
#   verity-protected erofs system image that gets written to a disk slot at
#   install time, so the live session and the installed system are byte-identical
#   and a live-session bug is reproducible on the installed system. It also means
#   the installer is a `dd` of a blob we already verified, not a package install.
#
# REQUIREMENTS: xorriso >= 1.5.4, mtools, sbsigntools, systemd (ukify), coreutils
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${PROJECT_ROOT}/build/out"
STAGE_DIR="${PROJECT_ROOT}/build/iso-stage"
KEYS_DIR="${PROJECT_ROOT}/keys/dev"

PROFILE="${LITRIC_PROFILE:-desktop}"
VERSION="${LITRIC_VERSION:-$(date -u -d "@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y.%m.%d)}"
VOLID="LITRIC_LIVE"

# Reproducibility: every timestamp in the ISO comes from SOURCE_DATE_EPOCH, so
# two builds of the same commit produce the same bytes and the same hash.
: "${SOURCE_DATE_EPOCH:=1756166400}"
export SOURCE_DATE_EPOCH
ISO_MTIME="$(date -u -d "@${SOURCE_DATE_EPOCH}" +%Y%m%d%H%M%S00)"

log()  { printf '\033[1;32m[iso]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[iso]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[iso]\033[0m %s\n' "$*" >&2; exit 1; }

# -----------------------------------------------------------------------------
# 1. Preflight -- fail loudly and early, never half-build
# -----------------------------------------------------------------------------
preflight() {
    log "preflight"
    local missing=()
    for tool in xorriso mcopy mmd mformat sha256sum; do
        command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
    done
    if (( ${#missing[@]} )); then
        die "missing tools: ${missing[*]}
  Debian/Ubuntu: apt install xorriso mtools
  Arch:          pacman -S libisoburn mtools
  Fedora:        dnf install xorriso mtools"
    fi

    [[ -d "$OUT_DIR" ]] || die "no build output at $OUT_DIR -- run tools/build.sh first"

    UKI="$(find "$OUT_DIR" -maxdepth 1 -name '*.efi' -print -quit)"
    SYSTEM="$(find "$OUT_DIR" -maxdepth 1 -name '*.system.raw' -print -quit)"
    VERITY="$(find "$OUT_DIR" -maxdepth 1 -name '*.verity.raw' -print -quit)"
    VERITYSIG="$(find "$OUT_DIR" -maxdepth 1 -name '*.verity-sig.raw' -print -quit)"
    ROOTHASH_FILE="$(find "$OUT_DIR" -maxdepth 1 -name '*.roothash' -print -quit)"

    [[ -n "$UKI"    ]] || die "no UKI (*.efi) in $OUT_DIR -- was Bootable=yes set?"
    [[ -n "$SYSTEM" ]] || die "no system image (*.system.raw) in $OUT_DIR"
    [[ -n "$VERITY" ]] || die "no verity tree (*.verity.raw) -- was Verity=signed set?"
    [[ -n "$ROOTHASH_FILE" ]] || die "no roothash file in $OUT_DIR"

    ROOTHASH="$(tr -d '[:space:]' < "$ROOTHASH_FILE")"
    [[ "$ROOTHASH" =~ ^[0-9a-f]{64}$ ]] \
        || die "roothash is not 64 hex chars: '${ROOTHASH}'"

    log "  uki       $(basename "$UKI")  ($(numfmt --to=iec < <(stat -c%s "$UKI")))"
    log "  system    $(basename "$SYSTEM")  ($(numfmt --to=iec < <(stat -c%s "$SYSTEM")))"
    log "  verity    $(basename "$VERITY")"
    log "  roothash  ${ROOTHASH:0:16}..."
}

# -----------------------------------------------------------------------------
# 2. Re-stamp the UKI with the live command line
#
# The verity root hash is only known AFTER the system image is built, and the
# kernel command line lives INSIDE the signed UKI. So the UKI must be rebuilt and
# re-signed here with the real hash. Phase 03 failure mode: "verity root hash
# mismatch after a legit build -> the pipeline must embed the hash automatically;
# never hand-copy." This function is that automation.
# -----------------------------------------------------------------------------
restamp_uki() {
    log "re-stamping UKI with live cmdline + roothash"

    local cmdline
    cmdline="quiet loglevel=4 console=ttyS0,115200 console=tty0"
    cmdline+=" litric.live=1"
    cmdline+=" litric.volid=${VOLID}"
    cmdline+=" roothash=${ROOTHASH}"
    cmdline+=" systemd.verity=yes"
    cmdline+=" lsm=landlock,lockdown,yama,integrity,bpf"
    cmdline+=" rd.udev.log_level=3 nowatchdog random.trust_cpu=on"

    if ! command -v ukify >/dev/null 2>&1; then
        warn "ukify not found -- shipping the UKI as built."
        warn "The live boot will fail to find its root unless the cmdline already"
        warn "carries roothash=. This is a WARNING, not a silent pass."
        return 0
    fi

    # Extract the kernel and initrd back out of the existing UKI so we rebuild
    # with identical payloads and only the cmdline changes.
    local work="${STAGE_DIR}/uki-work"
    rm -rf "$work"; mkdir -p "$work"
    objcopy -O binary --only-section=.linux  "$UKI" "$work/vmlinuz" 2>/dev/null || true
    objcopy -O binary --only-section=.initrd "$UKI" "$work/initrd"  2>/dev/null || true

    if [[ ! -s "$work/vmlinuz" ]]; then
        warn "could not extract .linux from the UKI (objcopy unavailable?)"
        warn "shipping the UKI unmodified -- see the warning above"
        return 0
    fi

    printf '%s' "$cmdline" > "$work/cmdline"

    local sign_args=()
    if [[ -f "${KEYS_DIR}/secureboot.key" && -f "${KEYS_DIR}/secureboot.crt" ]]; then
        sign_args=(--signtool=sbsign
                   --secureboot-private-key="${KEYS_DIR}/secureboot.key"
                   --secureboot-certificate="${KEYS_DIR}/secureboot.crt")
    else
        warn "no dev Secure Boot keys in ${KEYS_DIR} -- producing an UNSIGNED UKI."
        warn "It will not boot with Secure Boot enabled. Run tools/gen-dev-keys.sh."
    fi

    ukify build \
        --linux="$work/vmlinuz" \
        ${work:+--initrd="$work/initrd"} \
        --cmdline="@$work/cmdline" \
        --os-release="@${PROJECT_ROOT}/image/mkosi.extra/usr/lib/os-release" \
        --uname="$(basename "$UKI" .efi)" \
        "${sign_args[@]}" \
        --output="$work/litricos.efi"

    UKI="$work/litricos.efi"
    log "  UKI re-stamped: cmdline embeds roothash=${ROOTHASH:0:16}..."
}

# -----------------------------------------------------------------------------
# 3. Stage the ISO filesystem tree
# -----------------------------------------------------------------------------
stage_tree() {
    log "staging ISO tree"
    rm -rf "$STAGE_DIR/root"
    mkdir -p "$STAGE_DIR/root/EFI/BOOT" \
             "$STAGE_DIR/root/EFI/Linux" \
             "$STAGE_DIR/root/litric" \
             "$STAGE_DIR/root/loader/entries"

    # The UKI is its own bootloader: firmware can execute it directly as
    # BOOTX64.EFI. Shipping it twice (as the fallback path AND as a
    # systemd-boot entry) means the ISO boots on firmware that ignores
    # /loader/ as well as on firmware that honours it.
    install -Dm0644 "$UKI" "$STAGE_DIR/root/EFI/BOOT/BOOTX64.EFI"
    install -Dm0644 "$UKI" "$STAGE_DIR/root/EFI/Linux/litricos-${VERSION}.efi"

    # If systemd-boot is available, use it as the primary loader so the user gets
    # a menu (live / safe graphics / recovery) instead of an unconditional boot.
    local sdboot=/usr/lib/systemd/boot/efi/systemd-bootx64.efi
    if [[ -f "$sdboot" ]]; then
        install -Dm0644 "$sdboot" "$STAGE_DIR/root/EFI/BOOT/BOOTX64.EFI"
        log "  bootloader: systemd-boot (UKI available at /EFI/Linux/)"
    else
        warn "systemd-boot not found -- booting the UKI directly, no menu"
    fi

    install -Dm0644 "$SCRIPT_DIR/loader/loader.conf" "$STAGE_DIR/root/loader/loader.conf"
    for entry in "$SCRIPT_DIR"/loader/entries/*.conf; do
        [[ -f "$entry" ]] || continue
        sed -e "s|@VERSION@|${VERSION}|g" \
            -e "s|@ROOTHASH@|${ROOTHASH}|g" \
            -e "s|@VOLID@|${VOLID}|g" \
            "$entry" > "$STAGE_DIR/root/loader/entries/$(basename "$entry")"
    done

    # Payload. These are what the initrd loop-mounts and verity-verifies.
    install -Dm0644 "$SYSTEM"    "$STAGE_DIR/root/litric/system.erofs"
    install -Dm0644 "$VERITY"    "$STAGE_DIR/root/litric/system.verity"
    [[ -n "$VERITYSIG" ]] && install -Dm0644 "$VERITYSIG" "$STAGE_DIR/root/litric/system.verity.sig"
    printf '%s\n' "$ROOTHASH" > "$STAGE_DIR/root/litric/system.roothash"

    cat > "$STAGE_DIR/root/litric/version" <<EOF
LITRIC_VERSION=${VERSION}
LITRIC_PROFILE=${PROFILE}
LITRIC_ROOTHASH=${ROOTHASH}
LITRIC_BUILD_EPOCH=${SOURCE_DATE_EPOCH}
EOF

    # A plain-text manifest so a user can verify the payload without our tooling.
    ( cd "$STAGE_DIR/root/litric" && sha256sum ./* > SHA256SUMS ) || true
}

# -----------------------------------------------------------------------------
# 4. Build the EFI System Partition image (FAT), appended as a GPT partition
#
# Built with mtools so the whole ISO build stays rootless -- no loop device, no
# mount, no sudo. That matters: a build that needs root is a build that cannot
# run in an unprivileged CI container.
# -----------------------------------------------------------------------------
build_efiboot() {
    log "building EFI system partition image"
    local img="$STAGE_DIR/efiboot.img"
    rm -f "$img"

    # Size it to the actual content plus 20% slack, rounded to 1 MiB, with a
    # 32 MiB floor (some firmware dislikes very small FAT volumes).
    local content_kb
    content_kb="$(du -sk "$STAGE_DIR/root/EFI" | cut -f1)"
    local size_kb=$(( content_kb * 12 / 10 + 2048 ))
    (( size_kb < 32768 )) && size_kb=32768
    size_kb=$(( (size_kb + 1023) / 1024 * 1024 ))

    mformat -i "$img" -C -f "$size_kb" -F -v LITRIC_EFI ::
    mmd -i "$img" ::/EFI ::/EFI/BOOT ::/EFI/Linux ::/loader ::/loader/entries
    mcopy -i "$img" -s "$STAGE_DIR/root/EFI/BOOT/"*   ::/EFI/BOOT/
    mcopy -i "$img" -s "$STAGE_DIR/root/EFI/Linux/"*  ::/EFI/Linux/
    mcopy -i "$img" "$STAGE_DIR/root/loader/loader.conf" ::/loader/
    mcopy -i "$img" -s "$STAGE_DIR/root/loader/entries/"* ::/loader/entries/ 2>/dev/null || true

    log "  efiboot.img: $(numfmt --to=iec < <(stat -c%s "$img"))"
}

# -----------------------------------------------------------------------------
# 5. Master the ISO
#
# `-append_partition ... -appended_part_as_gpt` puts the ESP OUTSIDE the ISO 9660
# filesystem and references it from El Torito by interval. This is the modern
# approach: the same file is not stored twice, the GPT is valid so the image can
# also be dd'd straight to a USB stick, and no isohybrid post-processing is
# needed.
# -----------------------------------------------------------------------------
master_iso() {
    local iso="${OUT_DIR}/litricos-${PROFILE}-${VERSION}-x86_64.iso"
    log "mastering $(basename "$iso")"

    xorriso -as mkisofs \
        -iso-level 3 \
        -full-iso9660-filenames \
        -joliet -joliet-long \
        -rational-rock \
        -volid "$VOLID" \
        -appid "LitricOS ${PROFILE} ${VERSION}" \
        -publisher "LitricOS" \
        -preparer "iso/assemble-iso.sh" \
        -volume_date all_file_dates "=${SOURCE_DATE_EPOCH}" \
        -volume_date uuid "$ISO_MTIME" \
        -append_partition 2 C12A7328-F81F-11D2-BA4B-00A0C93EC93B "$STAGE_DIR/efiboot.img" \
        -appended_part_as_gpt \
        -eltorito-alt-boot \
        -e --interval:appended_partition_2:all:: \
        -no-emul-boot \
        -partition_offset 16 \
        -output "$iso" \
        "$STAGE_DIR/root"

    ( cd "$OUT_DIR" && sha256sum "$(basename "$iso")" > "$(basename "$iso").sha256" )

    local size
    size="$(numfmt --to=iec < <(stat -c%s "$iso"))"
    log "ISO:    $iso  (${size})"
    log "SHA256: $(cut -d' ' -f1 < "${iso}.sha256")"

    # Budget check (docs/PERF-budget.md section 5). A warning, not a failure --
    # tests/budget/ owns the hard gate; this is the early signal.
    local bytes budget
    bytes="$(stat -c%s "$iso")"
    case "$PROFILE" in
        nano)    budget=$((  90 * 1024 * 1024 )) ;;
        core)    budget=$(( 250 * 1024 * 1024 )) ;;
        desktop) budget=$(( 850 * 1024 * 1024 )) ;;
        gaming)  budget=$(( 1400 * 1024 * 1024 )) ;;
        *)       budget=0 ;;
    esac
    if (( budget > 0 && bytes > budget )); then
        warn "ISO is over the ${PROFILE} disk budget: ${size} > $(numfmt --to=iec <<< "$budget")"
        warn "See docs/PERF-budget.md section 5. This blocks the gate."
    fi
}

# -----------------------------------------------------------------------------
main() {
    mkdir -p "$STAGE_DIR" "$OUT_DIR"
    preflight
    restamp_uki
    stage_tree
    build_efiboot
    master_iso
    log "done"
}

main "$@"
