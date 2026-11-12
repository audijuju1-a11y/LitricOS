#!/usr/bin/env bash
# =============================================================================
# LitricOS — provision a Linux build host
#
# Runs as root inside WSL (or on any Arch machine) and installs everything
# tools/build.sh needs. Idempotent: safe to run repeatedly.
#
# WHY ARCH
#
#   mkosi builds an Arch image by driving `pacman` ON THE HOST. There is no way
#   around that: bootstrapping a distribution requires that distribution's
#   package manager. Ubuntu does not ship pacman, so an Ubuntu build host would
#   need the whole build to happen inside an Arch container -- an extra layer,
#   extra disk, and an extra failure mode, on a machine that is already short of
#   space.
#
#   Using Arch as the build host makes the toolchain native. It also means the
#   host and the target run the same package manager and the same package
#   versions, which is what makes the reproducibility pin in mkosi.conf mean
#   anything.
#
#   The image distribution is Arch because LitricOS needs a kernel new enough for
#   ntsync (6.14+) and sched_ext, per 00-MASTER-WORKFLOW.md section 3.
#
# Usage (inside WSL):
#   sudo bash tools/provision-builder.sh
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

log()  { printf '\033[1;32m[provision]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[provision]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[provision]\033[0m %s\n' "$*" >&2; exit 1; }

[[ $EUID -eq 0 ]] || die "must run as root: sudo bash tools/provision-builder.sh"

# -----------------------------------------------------------------------------
# 1. Identify the host distribution
# -----------------------------------------------------------------------------
ID=""
[[ -r /etc/os-release ]] && ID="$(. /etc/os-release && printf '%s' "${ID:-}")"
log "host distribution: ${ID:-unknown}"

if [[ "$ID" != "arch" && "$ID" != "archarm" ]]; then
    cat >&2 <<EOF

$(printf '\033[1;31m[provision]\033[0m') This build host is '${ID:-unknown}', not Arch Linux.

  mkosi bootstraps the LitricOS image with pacman, which only exists on Arch.
  Building from another distribution would require running the whole build in an
  Arch container -- more disk, more moving parts, and a path this project does
  not currently test. Rather than ship a half-supported fallback, this stops.

  Install the Arch WSL distribution instead:

      wsl --install --distribution archlinux
      wsl -d archlinux

  then re-run this script from inside it.

  If you are deliberately building on another distribution, the container route
  is documented in docs/DEV-02-build.md and is not automated.

EOF
    exit 1
fi

# -----------------------------------------------------------------------------
# 2. Packages
# -----------------------------------------------------------------------------
# Grouped by what they are for, so that a future reader can tell which of these
# is load-bearing and which is convenience.
PACKAGES=(
    # --- image construction -------------------------------------------------
    arch-install-scripts     # pacstrap, used by mkosi to populate the root tree
    erofs-utils              # mkfs.erofs -- the system partition format
    squashfs-tools           # fallback image format, and initrd inspection
    dosfstools               # mkfs.vfat for the ESP
    mtools                   # rootless FAT manipulation for the ESP image
    libisoburn               # xorriso -- ISO mastering
    cryptsetup               # veritysetup, LUKS2
    btrfs-progs              # the user partition
    e2fsprogs
    gptfdisk                 # sgdisk, for partition inspection

    # --- boot and signing ---------------------------------------------------
    systemd-ukify            # Unified Kernel Image assembly
    sbsigntools              # Secure Boot signing
    efitools
    openssl

    # --- virtualization for testing -----------------------------------------
    qemu-base                # qemu-system-x86_64
    qemu-img
    edk2-ovmf                # UEFI firmware for the guest
    swtpm                    # software TPM, needed for the Phase 03 TPM2 path

    # --- toolchain ----------------------------------------------------------
    base-devel
    git
    rust                     # the native workspace in src/
    python
    python-pip

    # --- measurement (PERF-budget is enforced, so these are product, not extra)
    sysstat                  # pidstat
    strace
    bc
)

# -----------------------------------------------------------------------------
# 2a. Package signing keyring
# -----------------------------------------------------------------------------
# A freshly installed Arch WSL image ships WITHOUT an initialised pacman keyring.
# Every package install then fails with "keyring is not writable" followed by
# "required key missing from keyring", which reads like a network or mirror
# problem and is neither.
#
# This is not optional hardening: pacman verifies every package signature, so an
# uninitialised keyring means either nothing installs, or somebody disables
# signature checking to make it work -- and that turns the build host into a
# machine that installs unverified packages.
# The guard counts actual keys rather than looking for trustdb.gpg. gnupg 2.4
# stores keys in keyboxd/pubring.kbx and may never create trustdb.gpg, so a
# file-existence check reports "not initialised" forever on a working keyring
# and "initialised" on a broken one. Counting public keys is the property we
# actually care about.
keycount() { pacman-key --list-keys 2>/dev/null | grep -c '^pub' || true; }

if [[ "$(keycount)" -lt 10 ]]; then
    log "initialising the pacman keyring (fresh WSL images ship without one)"
    pacman-key --init              || die "pacman-key --init failed"
    pacman-key --populate archlinux || die "pacman-key --populate failed"
    have="$(keycount)"
    [[ "$have" -ge 10 ]] || die "keyring still has only ${have} keys after populate.
  Without a populated keyring pacman cannot verify package signatures, and the
  only ways forward are to fix this or to disable signature verification.
  Disabling it would make this a build host that installs unverified packages,
  so this stops here instead."
    log "keyring populated: ${have} keys"
else
    log "pacman keyring already populated ($(keycount) keys)"
fi

log "synchronising package databases"
pacman -Sy --noconfirm >/dev/null

# Refresh the keyring package itself before anything else. Arch rotates signing
# keys, and a WSL image that has sat on a CDN for months can be older than the
# keys the current packages are signed with.
log "refreshing archlinux-keyring"
pacman -S --needed --noconfirm archlinux-keyring >/dev/null 2>&1 ||     warn "could not refresh archlinux-keyring; continuing with the existing one"

log "installing ${#PACKAGES[@]} packages (this is the slow part)"
# --needed makes this idempotent; a second run installs nothing.
pacman -S --needed --noconfirm "${PACKAGES[@]}"

# -----------------------------------------------------------------------------
# 3. mkosi
# -----------------------------------------------------------------------------
# From the distribution package, not from PyPI.
#
# The original version of this script installed mkosi with pip, reasoning that a
# distribution package might lag the MinimumVersion=25.3 that image/mkosi.conf
# declares. On Arch that reasoning is simply wrong: extra/mkosi is 26-5, ahead of
# the floor. It is also signed and verified by pacman, whereas the pip path
# needed --break-system-packages to defeat PEP 668 and pulled an unverified
# wheel. Fewer moving parts and a signature beats a version worry that turned out
# not to exist.
#
# The version floor is still enforced below, so if the distribution ever falls
# behind, this fails loudly instead of producing confusing schema errors at
# build time.
MKOSI_MIN="25.3"

log "installing mkosi from the distribution"
pacman -S --needed --noconfirm mkosi || die "could not install mkosi"

command -v mkosi >/dev/null 2>&1 || die "mkosi installed but is not on PATH"

MKOSI_HAVE="$(mkosi --version 2>/dev/null | grep -oE '[0-9]+(\.[0-9]+)*' | head -1 || echo 0)"
# sort -V so that 26 and 25.10 compare correctly against 25.3.
if [[ "$(printf '%s
%s
' "$MKOSI_MIN" "$MKOSI_HAVE" | sort -V | head -1)" != "$MKOSI_MIN" ]]; then
    die "mkosi ${MKOSI_HAVE} is older than the required ${MKOSI_MIN}.
  image/mkosi.conf declares MinimumVersion=${MKOSI_MIN}. An older mkosi rejects
  the configuration with a schema error that looks like a broken config rather
  than a stale tool, which is why this is checked here instead."
fi
log "mkosi ${MKOSI_HAVE} satisfies >= ${MKOSI_MIN}"

# -----------------------------------------------------------------------------
# 4. Working copy inside the Linux filesystem
# -----------------------------------------------------------------------------
# Building on /mnt/c is a trap. Every file operation crosses WSL's 9p bridge,
# which is 5-20x slower, and the Windows filesystem cannot represent Unix
# ownership or the setuid bits an image tree needs. An image built there is both
# slow to produce and wrong.
WORK="${SUDO_USER:+/home/${SUDO_USER}}/litricos"
[[ -n "${SUDO_USER:-}" ]] || WORK="/root/litricos"

if [[ "$PROJECT_ROOT" == /mnt/* ]]; then
    log "project is on a Windows mount (${PROJECT_ROOT})"
    if [[ ! -d "$WORK" ]]; then
        log "creating a native working copy at ${WORK}"
        mkdir -p "$WORK"
        # -a preserves what can be preserved; the Windows side has no Unix
        # metadata to preserve, which is precisely the problem being solved.
        cp -a "${PROJECT_ROOT}/." "$WORK/"
        [[ -n "${SUDO_USER:-}" ]] && chown -R "${SUDO_USER}:${SUDO_USER}" "$WORK"
        log "working copy created"
    else
        warn "${WORK} already exists; leaving it alone"
        warn "sync manually with: rsync -a --delete '${PROJECT_ROOT}/' '${WORK}/'"
    fi
else
    WORK="$PROJECT_ROOT"
    log "project is already on a Linux filesystem"
fi

# -----------------------------------------------------------------------------
# 5. Verify the toolchain is actually usable
# -----------------------------------------------------------------------------
log "verifying toolchain"
MISSING=()
for tool in mkosi pacstrap mkfs.erofs veritysetup sbsign ukify xorriso mcopy \
            qemu-system-x86_64 qemu-img openssl cargo python3; do
    command -v "$tool" >/dev/null 2>&1 || MISSING+=("$tool")
done

if (( ${#MISSING[@]} )); then
    die "these tools are still missing after provisioning: ${MISSING[*]}"
fi

# OVMF firmware is a file, not a command, and its path differs per distribution.
OVMF_CODE=""
for candidate in \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd \
    /usr/share/edk2-ovmf/x64/OVMF_CODE.fd \
    /usr/share/OVMF/OVMF_CODE_4M.fd
do
    [[ -f "$candidate" ]] && { OVMF_CODE="$candidate"; break; }
done
[[ -n "$OVMF_CODE" ]] || warn "OVMF firmware not found; tools/run-qemu.sh will report the paths it looked in"

printf '\n'
log "build host ready"
printf '  mkosi   : %s\n' "$(mkosi --version 2>/dev/null | head -1)"
printf '  qemu    : %s\n' "$(qemu-system-x86_64 --version 2>/dev/null | head -1)"
printf '  rust    : %s\n' "$(cargo --version 2>/dev/null)"
printf '  ovmf    : %s\n' "${OVMF_CODE:-NOT FOUND}"
printf '  workdir : %s\n' "$WORK"
printf '\n'
printf '  Next:\n'
printf '    cd %s\n' "$WORK"
printf '    ./tools/build.sh --profile nano\n'
printf '\n'
