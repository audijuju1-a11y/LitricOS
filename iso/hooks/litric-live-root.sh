#!/usr/bin/env bash
# =============================================================================
# LitricOS — live root assembly (runs inside the initrd)
#
# Turns the ISO payload into a mounted, writable, integrity-verified /sysroot
# that systemd can switch-root into.
#
#   ISO (iso9660, ro)
#     └─ /litric/system.erofs   ──loop──┐
#        /litric/system.verity  ──loop──┤
#        /litric/system.roothash        │
#                                       ▼
#                            veritysetup open  →  /dev/mapper/litric-sys  (ro, verified)
#                                       │
#                     lowerdir ─────────┘
#                     upperdir ── tmpfs (RAM, volatile)
#                                       ▼
#                                   overlayfs → /sysroot
#
# INTEGRITY: the root hash comes from the kernel command line, which lives inside
# the SIGNED UKI. It is never read from the ISO. That is the whole point -- an
# attacker who rewrites the ISO payload cannot also rewrite the signed hash, so
# a tampered payload fails to open the verity device and the boot stops.
#
# FAILURE POLICY: fail CLOSED and land in an emergency shell with a printed
# reason. Never continue with an unverified root. Never silently fall back.
# =============================================================================
set -uo pipefail

LIVE_MNT=/run/litric/live
LOWER=/run/litric/lower
UPPER=/run/litric/upper
SYSROOT=/sysroot
DM_NAME=litric-sys

log()  { printf 'litric-live: %s\n' "$*" > /dev/kmsg 2>/dev/null || printf 'litric-live: %s\n' "$*"; }
fail() {
    printf '\n' >&2
    printf 'litric-live: FATAL: %s\n' "$*" >&2
    printf 'litric-live: refusing to continue with an unverified or missing root.\n' >&2
    printf 'litric-live: boot the "recovery console" entry to investigate.\n' >&2
    exit 1
}

# --- 0. Read parameters from the signed command line -------------------------
kcmd() {
    local key="$1" tok
    for tok in $(cat /proc/cmdline); do
        case "$tok" in "$key"=*) printf '%s' "${tok#*=}"; return 0 ;; esac
    done
    return 1
}

[[ "$(kcmd litric.live || echo 0)" == "1" ]] || { log "not a live boot; nothing to do"; exit 0; }

VOLID="$(kcmd litric.volid || echo LITRIC_LIVE)"
ROOTHASH="$(kcmd roothash || true)"
[[ -n "$ROOTHASH" ]] || fail "no roothash= on the kernel command line.
  The UKI was not re-stamped after the system image was built.
  See iso/assemble-iso.sh restamp_uki()."
[[ "$ROOTHASH" =~ ^[0-9a-f]{64}$ ]] || fail "roothash is malformed: '$ROOTHASH'"

log "live boot: volid=${VOLID} roothash=${ROOTHASH:0:16}..."

# --- 1. Find the boot medium -------------------------------------------------
# Poll rather than assume: USB enumeration is slow and racy, and a hard-coded
# sleep is either too short on slow hardware or wasted time on fast hardware.
find_medium() {
    local deadline=$(( SECONDS + 30 )) dev
    while (( SECONDS < deadline )); do
        dev="$(blkid -L "$VOLID" 2>/dev/null || true)"
        if [[ -n "$dev" && -b "$dev" ]]; then printf '%s' "$dev"; return 0; fi
        udevadm settle --timeout=2 >/dev/null 2>&1 || sleep 0.25
    done
    return 1
}

MEDIUM="$(find_medium)" || fail "could not find a block device labelled ${VOLID} within 30 s.
  If this is a USB stick, the write may be incomplete or the medium may be failing.
  Verify with: sha256sum -c litricos-*.iso.sha256"
log "medium: ${MEDIUM}"

mkdir -p "$LIVE_MNT" "$LOWER" "$UPPER" "$SYSROOT"
mount -o ro,noatime "$MEDIUM" "$LIVE_MNT" \
    || fail "could not mount ${MEDIUM} at ${LIVE_MNT}"

SYS_IMG="${LIVE_MNT}/litric/system.erofs"
VER_IMG="${LIVE_MNT}/litric/system.verity"
[[ -f "$SYS_IMG" ]] || fail "missing payload: /litric/system.erofs"
[[ -f "$VER_IMG" ]] || fail "missing payload: /litric/system.verity"

# --- 2. Loop-mount the payload ----------------------------------------------
# Read-only loop devices: nothing in the live session can write to the system
# image even before verity is in the path.
SYS_LOOP="$(losetup --find --show --read-only "$SYS_IMG")" \
    || fail "losetup failed for system.erofs"
VER_LOOP="$(losetup --find --show --read-only "$VER_IMG")" \
    || fail "losetup failed for system.verity"
log "loop: system=${SYS_LOOP} verity=${VER_LOOP}"

# --- 3. Open the verity device ----------------------------------------------
# This is the integrity gate. If a single byte of system.erofs differs from what
# was signed, veritysetup either refuses here or the kernel returns EIO on the
# first bad block -- either way the system does not run tampered code.
if ! veritysetup open "$SYS_LOOP" "$DM_NAME" "$VER_LOOP" "$ROOTHASH" \
        --panic-on-corruption 2>/dev/null; then
    # --panic-on-corruption is not supported on every kernel; retry without it,
    # but say so, because the failure mode is then EIO rather than a clean panic.
    log "note: --panic-on-corruption unsupported; falling back to restart-on-corruption"
    veritysetup open "$SYS_LOOP" "$DM_NAME" "$VER_LOOP" "$ROOTHASH" \
        || fail "dm-verity refused the system image.
  The root hash on the signed command line does not match the payload on the medium.
  This means the ISO is damaged or has been modified. Do not boot it."
fi
log "verity: /dev/mapper/${DM_NAME} open and verified"

mount -o ro,noatime "/dev/mapper/${DM_NAME}" "$LOWER" \
    || fail "could not mount the verified system image"

# --- 4. Writable layer in RAM ------------------------------------------------
# size=50% of RAM. On a 4 GB machine that is 2 GB of writable space, which is
# enough to install packages and run the installer. It is tmpfs, so it is backed
# by zram-compressed swap once zram comes up in the real root.
mount -t tmpfs -o "size=50%,mode=0755,nosuid,nodev" litric-live-rw "$UPPER" \
    || fail "could not create the writable tmpfs layer"
mkdir -p "$UPPER/upper" "$UPPER/work"

# --- 5. Overlay --------------------------------------------------------------
mount -t overlay litric-live \
    -o "lowerdir=${LOWER},upperdir=${UPPER}/upper,workdir=${UPPER}/work,redirect_dir=on,metacopy=on" \
    "$SYSROOT" \
    || fail "overlayfs assembly failed"

# Keep the medium reachable from the real root so the installer can read the
# payload it is about to write, without re-mounting anything.
mkdir -p "${SYSROOT}/run/litric/live"
mount --bind "$LIVE_MNT" "${SYSROOT}/run/litric/live" || true

# Mark the session as live so first-boot logic, the installer, and the shell can
# all behave differently without guessing.
mkdir -p "${SYSROOT}/run/litric"
{
    printf 'LITRIC_LIVE=1\n'
    printf 'LITRIC_MEDIUM=%s\n' "$MEDIUM"
    printf 'LITRIC_ROOTHASH=%s\n' "$ROOTHASH"
} > "${SYSROOT}/run/litric/live-session"

log "root assembled at ${SYSROOT}; handing off to switch-root"
exit 0
