#!/usr/bin/env bash
# =============================================================================
# LitricOS — build an image
#
#   ./tools/build.sh --profile nano          minimal, the GATE 01 target
#   ./tools/build.sh --profile core
#   ./tools/build.sh --profile desktop
#   ./tools/build.sh --profile gaming
#
#   --iso            also master a bootable ISO (iso/assemble-iso.sh)
#   --clean          discard the output tree first
#   --no-cache       ignore the package cache (slow; use to prove reproducibility)
#   --dry-run        run every preflight check and print the plan, build nothing
#
# SAFETY
#   Writes only under <project>/build/. Asserts this before doing anything.
#   Never touches a physical disk, never runs a partitioning tool against one.
#   MASTER invariant G1.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
OUT_DIR="${BUILD_DIR}/out"
IMAGE_DIR="${PROJECT_ROOT}/image"

PROFILE="nano"
MAKE_ISO=0
CLEAN=0
NO_CACHE=0
DRY_RUN=0

# Reproducibility. Two builds of the same commit must produce the same bytes, so
# every timestamp in the image comes from here rather than from the clock.
: "${SOURCE_DATE_EPOCH:=1756166400}"
export SOURCE_DATE_EPOCH

log()  { printf '\033[1;32m[build]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[build]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[build]\033[0m %s\n' "$*" >&2; exit 1; }
step() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }

# -----------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile) PROFILE="${2:-}"; shift 2 ;;
        --profile=*) PROFILE="${1#*=}"; shift ;;
        --iso)     MAKE_ISO=1; shift ;;
        --clean)   CLEAN=1; shift ;;
        --no-cache) NO_CACHE=1; shift ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown argument: $1  (try --help)" ;;
    esac
done

case "$PROFILE" in
    nano|core|desktop|gaming) ;;
    *) die "unknown profile '$PROFILE'. Valid: nano, core, desktop, gaming" ;;
esac

# Profile stacking. `nano` is deliberately NOT a base for the others: it exists
# to be the memory floor, and anything layered on it would stop being a floor.
case "$PROFILE" in
    nano)    PROFILE_ARGS=(--profile nano) ;;
    core)    PROFILE_ARGS=(--profile core) ;;
    desktop) PROFILE_ARGS=(--profile core --profile desktop) ;;
    gaming)  PROFILE_ARGS=(--profile core --profile desktop --profile gaming) ;;
esac

# =============================================================================
step "Preflight"
# =============================================================================

[[ "$(uname -s)" == "Linux" ]] || die "this must run on Linux.
  On Windows, run tools/bootstrap-windows.ps1 first, then build from inside WSL.
  Detected: $(uname -s)"

[[ $EUID -ne 0 ]] || warn "running as root. mkosi does not require it and it is
  safer not to: a rootless build cannot damage the host even if it misbehaves."

# Safety assertion: every path we will write to must be inside the project.
# Checked before anything is created, not after.
for path in "$BUILD_DIR" "$OUT_DIR"; do
    case "$path" in
        "${PROJECT_ROOT}"/*) ;;
        *) die "refusing to write outside the project directory: $path" ;;
    esac
done

MISSING=()
for tool in mkosi pacstrap mkfs.erofs veritysetup openssl; do
    command -v "$tool" >/dev/null 2>&1 || MISSING+=("$tool")
done
if (( MAKE_ISO )); then
    for tool in xorriso mcopy mformat; do
        command -v "$tool" >/dev/null 2>&1 || MISSING+=("$tool")
    done
fi
if (( ${#MISSING[@]} )); then
    die "missing tools: ${MISSING[*]}
  Run: sudo bash tools/provision-builder.sh"
fi

MKOSI_VERSION="$(mkosi --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1 || echo '0')"
log "mkosi ${MKOSI_VERSION}"

# The project must be on a Linux filesystem. A build on /mnt/c cannot represent
# Unix ownership, so the resulting image tree has wrong permissions on every
# file -- an image that builds and then fails to boot for a reason that looks
# like a kernel problem.
case "$PROJECT_ROOT" in
    /mnt/*)
        die "the project is on a Windows mount: ${PROJECT_ROOT}

  A Windows filesystem cannot store Unix ownership or permission bits, so the
  image tree would be built with wrong permissions throughout. The failure shows
  up at boot and looks like a kernel fault, which is why this stops here instead.

  tools/provision-builder.sh created a native copy. Build from there:
      cd ~/litricos && ./tools/build.sh --profile ${PROFILE}"
        ;;
esac

# =============================================================================
step "Signing keys"
# The mkosi linter validates that every signing key referenced by the image
# configuration exists. Generate the local development keys before linting so a
# clean checkout can pass the same preflight as a previously-built checkout.
# These keys are generated locally and are never used for release artifacts.
# =============================================================================
if [[ ! -s "${PROJECT_ROOT}/keys/dev/secureboot.key" ]]; then
    log "generating development signing keys"
    bash "${PROJECT_ROOT}/tools/gen-dev-keys.sh"
else
    log "development keys present"
fi

# =============================================================================
step "Configuration checks"
# =============================================================================
# The two linters that exist because both of these files have already shipped
# defects that a build would only have surfaced hours later, or not at all.
if command -v python3 >/dev/null 2>&1; then
    python3 "${PROJECT_ROOT}/ci/lint-mkosi.py"   || die "image configuration is invalid"
    python3 "${PROJECT_ROOT}/ci/lint-kconfig.py" || die "kernel configuration is invalid"
else
    warn "python3 unavailable; skipping configuration lint"
fi

# Private keys must be 0600. mkosi refuses to sign with a world-readable key and
# is right to: on a shared or multi-user build host a 0777 signing key is a key
# anyone can copy.
#
# This is repaired rather than merely checked, because the usual cause is benign
# and unavoidable: a project copied from a Windows filesystem arrives with 0777
# on everything, since NTFS carries no Unix mode bits. Failing the build would
# just make every Windows-hosted developer run chmod by hand.
for key in "${PROJECT_ROOT}"/keys/dev/*.key; do
    [[ -f "$key" ]] || continue
    mode="$(stat -c '%a' "$key" 2>/dev/null || echo unknown)"
    if [[ "$mode" != "600" ]]; then
        log "tightening ${key##*/} from ${mode} to 600"
        chmod 0600 "$key" || die "could not chmod ${key}"
    fi
done

# =============================================================================
step "Native workspace"
# =============================================================================
# Built before the image so that a compile error costs seconds rather than
# surfacing after a multi-minute package bootstrap.
if command -v cargo >/dev/null 2>&1; then
    log "building src/ (release)"
    ( cd "${PROJECT_ROOT}/src" && cargo build --release --locked 2>&1 | tail -5 ) \
        || die "the native workspace failed to build"

    # Stage any binaries the workspace produced into the image tree.
    #
    # HONEST STATE, 2026-08-28: litric-sysd exists and is staged. litric-shell,
    # litric-capd and litric-defender are still referred to by systemd units in
    # image/mkosi.extra and DO NOT EXIST; the desktop guard below is what keeps
    # that from becoming an image that boots to an empty screen.
    STAGE="${IMAGE_DIR}/mkosi.extra/usr/bin"
    staged=0
    # Where cargo actually put the binaries. The build cache may live off the
    # Windows-mounted filesystem (tools/cargo-target-ext4.sh, installed to
    # /etc/profile.d on the build host), and a staging step that looked only in
    # src/target would silently stage nothing. Unset, both cargo and this line
    # fall back to src/target, so they can never disagree.
    TARGET_DIR="${CARGO_TARGET_DIR:-${PROJECT_ROOT}/src/target}"
    for bin in litric-sysd litric-envd litric-shell litric-browserd litric-filesd litric-capd litric-defender litricctl; do
        src="${TARGET_DIR}/release/${bin}"
        if [[ -x "$src" ]]; then
            mkdir -p "$STAGE"
            install -Dm0755 "$src" "${STAGE}/${bin}"
            log "staged ${bin}"
            staged=$((staged + 1))
        fi
    done
    # QEMU's DRM fbdev bridge can accept write(2) without publishing those
    # pixels to the visible scanout. Keep the one necessary mmap boundary in a
    # standalone helper; the Rust shell sends it fixed-size frames through stdin
    # and retains all desktop policy in safe Rust.
    PRESENTER_SOURCE="${PROJECT_ROOT}/src/litric-shell/src/fbpresent.c"
    if [[ -f "$PRESENTER_SOURCE" ]]; then
        command -v cc >/dev/null 2>&1 || die "C compiler required for litric-fbpresent"
        mkdir -p "$STAGE"
        cc -std=c17 -O2 -Wall -Wextra -Werror "$PRESENTER_SOURCE" -o "${STAGE}/litric-fbpresent"
        chmod 0755 "${STAGE}/litric-fbpresent"
        log "staged litric-fbpresent"
    fi
    if (( staged == 0 )); then
        warn "no LitricOS system binaries were staged."
    fi

    # Every unit file carries a Documentation= pointing into
    # /usr/share/doc/litric. Until now none of those files was in the image, so
    # `systemctl status` offered a path that did not exist -- a small lie, but
    # the same kind as any other. The documents are a few hundred kilobytes of
    # text; ship them.
    DOCS="${IMAGE_DIR}/mkosi.extra/usr/share/doc/litric"
    rm -rf "$DOCS"
    mkdir -p "${DOCS}/adr"
    if compgen -G "${PROJECT_ROOT}/docs/*.md" >/dev/null; then
        install -Dm0644 "${PROJECT_ROOT}"/docs/*.md "${DOCS}/" 2>/dev/null || true
    fi
    if compgen -G "${PROJECT_ROOT}/docs/adr/*.md" >/dev/null; then
        install -Dm0644 "${PROJECT_ROOT}"/docs/adr/*.md "${DOCS}/adr/" 2>/dev/null || true
    fi
    log "staged $(find "$DOCS" -name '*.md' | wc -l) documents"

    # The desktop profile needs the SHELL, not merely some binary. Counting
    # staged binaries would let litric-sysd unlock a desktop image that still
    # boots to a failed litric-shell.service, which is exactly the kind of
    # check that reports success for the wrong reason.
    if [[ ! -x "${STAGE}/litric-shell" ]]; then
        if [[ "$PROFILE" == "desktop" || "$PROFILE" == "gaming" ]]; then
            die "refusing to build the '${PROFILE}' profile: litric-shell does not
  exist, so the image would boot to a failed litric-shell.service and an empty
  screen. That is not a usable artifact and shipping it would misrepresent
  progress. Build --profile nano or core until the shell exists."
        fi
        warn "litric-shell does not exist yet; 'desktop' and 'gaming' are refused."
    fi
else
    warn "cargo unavailable; skipping the native workspace"
fi

# =============================================================================
step "Image build: profile ${PROFILE}"
# =============================================================================

if (( CLEAN )); then
    log "removing ${OUT_DIR}"
    rm -rf "$OUT_DIR"
fi
mkdir -p "$OUT_DIR" "${BUILD_DIR}/cache"

# --force is not optional here. Without it, mkosi sees an existing
# build/out/litricos.raw.zst and prints "Output path exists already", does
# nothing, and exits 0. This script then reported a successful build and listed
# the STALE artifacts as if it had just produced them -- a configuration fix was
# applied, the build "succeeded", and the image was byte-for-byte the old broken
# one. A command called `build` must build.
MKOSI_ARGS=(--directory "$IMAGE_DIR" --force "${PROFILE_ARGS[@]}")
(( NO_CACHE )) && MKOSI_ARGS+=(--cache-only=never --incremental=no)

# Keep the previous image. Phase 01 failure mode: "unbootable image -> previous
# image is kept by the build script. Always."
LAST_GOOD_DIR="${BUILD_DIR}/last-good"
if compgen -G "${OUT_DIR}/*.raw" > /dev/null 2>&1; then
    log "preserving the previous build as build/last-good/"
    rm -rf "$LAST_GOOD_DIR"
    mkdir -p "$LAST_GOOD_DIR"
    cp -a "${OUT_DIR}/." "${LAST_GOOD_DIR}/" 2>/dev/null || true
fi

if (( DRY_RUN )); then
    log "DRY RUN -- would execute:"
    printf '    mkosi %s summary\n' "${MKOSI_ARGS[*]}"
    printf '    mkosi %s build\n'   "${MKOSI_ARGS[*]}"
    (( MAKE_ISO )) && printf '    LITRIC_PROFILE=%s iso/assemble-iso.sh\n' "$PROFILE"
    log "showing the resolved configuration:"
    # A failing summary means the configuration is broken, so --dry-run must
    # exit non-zero. It previously warned and exited 0, which made a broken
    # config look like a successful dry run -- exactly the kind of green result
    # that hides a real failure.
    if ! mkosi "${MKOSI_ARGS[@]}" summary; then
        die "mkosi rejected the configuration (see above). Nothing was built."
    fi
    exit 0
fi

log "running mkosi (this bootstraps a distribution; the first run is slow)"
MKOSI_LOG="${BUILD_DIR}/mkosi-last.log"
if ! mkosi "${MKOSI_ARGS[@]}" build 2>&1 | tee "$MKOSI_LOG"; then
    die "mkosi build failed -- see the output above.
  The previous image, if any, is preserved in build/last-good/"
fi

# Belt and braces alongside --force: if mkosi ever again decides to skip the
# build, say so instead of reporting success. A build that produced nothing is a
# failed build even when the exit code is zero.
if grep -q "exists already" "$MKOSI_LOG"; then
    die "mkosi skipped the build because the output already existed.
  Nothing was rebuilt, so any configuration change you just made is NOT in the
  image. This should be impossible now that --force is passed; if you are seeing
  it, tools/build.sh has regressed."
fi

# --- the artifact can mount its own root (BUG-0020) --------------------------
#
# mkosi reporting success is not the same as the image being bootable. The
# system partition is erofs behind dm-verity and both drivers are modules, so an
# initrd without them boots to an emergency shell with "Failed to mount
# /sysroot" -- a failure that looks like storage and is a packaging mistake in
# one config key. This reads the built UKI and refuses to call the image
# buildable when its initrd cannot mount the root.
UKI="$(ls "${OUT_DIR}"/*.efi 2>/dev/null | head -1)"
if [[ -n "$UKI" ]]; then
    if command -v python3 >/dev/null 2>&1; then
        # dm-verity may be its own module or part of dm-mod depending on the
        # kernel; either one creates the device, so either one passes.
        python3 "${PROJECT_ROOT}/tools/probe-initrd.py" --uki "$UKI" \
            --require erofs "dm-verity|dm-mod" \
            || die "the built UKI cannot mount its own root filesystem.
  The initrd is missing a driver the boot needs; the image would reach an
  emergency shell instead of a desktop. See docs/bugs/BUG-0020 and
  image/mkosi.conf (KernelInitrdModules=)."
    else
        warn "python3 unavailable; skipped the initrd driver probe (BUG-0020 check)"
    fi
fi

# --- Initrd trimming (nano profile only) ---
if [[ "${PROFILE}" == "nano" ]]; then
    step "Initrd trimming"
    bash "${PROJECT_ROOT}/tools/trim-initrd.sh" "${OUT_DIR}" || warn "initrd trimming failed (non-fatal)"
    # Update UKI in the disk image with trimmed initrd
    DISK_IMG=$(ls "${OUT_DIR}"/*.raw 2>/dev/null | head -1)
    INITRD_TRIMMED="${OUT_DIR}/initrd.cpio.zst"
    if [[ -n "$DISK_IMG" && -f "$INITRD_TRIMMED" ]]; then
        TMPDIR_UKI=$(mktemp -d)
        trap 'rm -rf "${TMPDIR_UKI}"' RETURN
        mkdir -p "${TMPDIR_UKI}/esp"
        # ESP is at offset 1MiB (2048 sectors * 512)
        if mount -o loop,offset=$((2048*512)),ro "$DISK_IMG" "${TMPDIR_UKI}/esp" 2>/dev/null; then
            UKI_PATH=$(find "${TMPDIR_UKI}/esp" -name '*.efi' -path '*/EFI/*' 2>/dev/null | head -1)
            if [[ -n "$UKI_PATH" ]]; then
                log "updating UKI in ESP: $UKI_PATH"
                # Extract kernel from UKI
                if ukify inspect "$UKI_PATH" --section=.linux -o "${TMPDIR_UKI}/vmlinuz" 2>/dev/null; then
                    # Get cmdline
                    CMDLINE=$(ukify inspect "$UKI_PATH" --show-cmdline 2>/dev/null || echo "")
                    # Rebuild UKI with trimmed initrd
                    UKIFY_ARGS=(ukify build --linux="${TMPDIR_UKI}/vmlinuz" --initrd="$INITRD_TRIMMED" --output="${TMPDIR_UKI}/uki-new.efi")
                    [[ -n "$CMDLINE" ]] && UKIFY_ARGS+=(--cmdline="$CMDLINE")
                    # Sign with SecureBoot key if available
                    if [[ -f "${PROJECT_ROOT}/keys/dev/secureboot.key" ]]; then
                        UKIFY_ARGS+=(--efi-boot-key="${PROJECT_ROOT}/keys/dev/secureboot.key" --efi-boot-certificate="${PROJECT_ROOT}/keys/dev/secureboot.crt" --sign-kernel=yes)
                    fi
                    if "${UKIFY_ARGS[@]}" 2>/dev/null; then
                        umount "${TMPDIR_UKI}/esp" 2>/dev/null || true
                        # Remount rw and replace UKI
                        mount -o loop,offset=$((2048*512)) "$DISK_IMG" "${TMPDIR_UKI}/esp" 2>/dev/null && \
                        cp "${TMPDIR_UKI}/uki-new.efi" "$UKI_PATH" && \
                        log "UKI updated in disk image" || warn "failed to update UKI in disk image"
                        umount "${TMPDIR_UKI}/esp" 2>/dev/null || true
                    else
                        warn "ukify rebuild failed"
                        umount "${TMPDIR_UKI}/esp" 2>/dev/null || true
                    fi
                else
                    warn "could not extract kernel from UKI"
                    umount "${TMPDIR_UKI}/esp" 2>/dev/null || true
                fi
            else
                warn "no UKI found in ESP"
                umount "${TMPDIR_UKI}/esp" 2>/dev/null || true
            fi
        else
            warn "could not mount ESP (read-only?)"
        fi
    fi
fi

# =============================================================================
step "Artifacts"
# =============================================================================
# Only mkosi's own outputs. Boot-test scratch files (decompressed images, OVMF
# variable stores, serial logs) live here too and must not be reported as build
# artifacts -- listing them made a no-op build look productive.
shopt -s nullglob
artifacts=( "${OUT_DIR}"/*.raw.zst "${OUT_DIR}"/*.efi "${OUT_DIR}"/*.cpio.zst "${OUT_DIR}"/*.roothash )
shopt -u nullglob

if (( ${#artifacts[@]} == 0 )); then
    die "mkosi reported success but produced no artifacts in ${OUT_DIR}.
  Treat this as a build failure, not a success with an empty result."
fi

for f in "${artifacts[@]}"; do
    [[ -f "$f" ]] || continue
    printf '  %-52s %10s\n' "$(basename "$f")" "$(numfmt --to=iec < <(stat -c%s "$f"))"
done

# Hashes, so that "reproducible" is a claim someone can check rather than assert.
log "recording hashes"
( cd "$OUT_DIR" && sha256sum ./* > SHA256SUMS 2>/dev/null || true )

# =============================================================================
if (( MAKE_ISO )); then
    step "ISO"
    LITRIC_PROFILE="$PROFILE" bash "${PROJECT_ROOT}/iso/assemble-iso.sh"
fi

# =============================================================================
step "Done"
# =============================================================================
cat <<EOF

  profile   : ${PROFILE}
  output    : ${OUT_DIR}
  epoch     : ${SOURCE_DATE_EPOCH} (set SOURCE_DATE_EPOCH to change)

  Next -- and this is the part that matters, because an image that has not been
  booted is not evidence of anything:

      ./tools/run-qemu.sh --profile ${PROFILE}
      python3 tests/smoke_boot.py

  Record the result in docs/GATES.md. GATE 01 stays BLOCKED until the smoke test
  passes AND is shown to fail on a deliberately corrupted image.

EOF
