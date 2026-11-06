#!/usr/bin/env bash
# =============================================================================
# LitricOS — build the QEMU guest toolchain and rootfs without root
#
#   ./tools/qemu-guest-bootstrap.sh
#
# The cache is build/qemu-bootstrap by default. It contains:
#   apt/       private APT lists and .deb cache (never the host package DB)
#   tools/     QEMU, Silicon BIOS, cpio, zstd, kernel and Ubuntu libs
#   rootfs/    BusyBox, glibc, Noto faces, the DRM/input modules and e1000
#
# This script only supports x86_64 Ubuntu/Debian hosts with apt-get and
# dpkg-deb. It does not need root and does not install anything into the host.
# Once the cache is complete it works without network. To use another cache:
#
#   LITRIC_BOOTSTRAP_CACHE=/path ./tools/qemu-guest-bootstrap.sh
#
# If the host cannot reach the Ubuntu archive, the caller can instead provide a
# complete tools tree and rootfs through LITRIC_TOOLS and LITRIC_ROOTFS.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
CACHE="${LITRIC_BOOTSTRAP_CACHE:-${PROJECT_ROOT}/build/qemu-bootstrap}"

# Bump when the guest's contents change shape: an older cache must be refreshed
# instead of being taken for a complete one that merely lacks the new pieces.
SCHEMA=2
SCHEMA_FILE="${CACHE}/.schema"
# The browser is a real engine, pinned and checksummed. A package name in a
# profile is not a browser; this download is what makes the image contain one.
CHROME_VERSION="${LITRIC_CHROME_VERSION:-131.0.6778.204}"
CHROME_SHA256="${LITRIC_CHROME_SHA256:-fba7d5e87cdcf0dc974c9ac8a0a9dbcf5686c935455d5e6beed337c6ada88bc2}"
CHROME_URL="https://storage.googleapis.com/chrome-for-testing-public/${CHROME_VERSION}/linux64/chrome-linux64.zip"
CHROME_CACHE="${CACHE}/chrome"

while (($#)); do
    case "$1" in
        --cache)
            [[ $# -ge 2 ]] || { echo "--cache needs a directory" >&2; exit 2; }
            CACHE="$2"
            shift 2
            ;;
        -h|--help)
            sed -n '2,35p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            exit 2
            ;;
    esac
done

TOOLS="${CACHE}/tools"
ROOTFS="${CACHE}/rootfs"
APTROOT="${CACHE}/apt"
COMPLETE="${CACHE}/.complete"
DOWNLOAD_COMPLETE="${CACHE}/.download-complete"
APT_SOURCES_READY="${CACHE}/.apt-sources-ready"
KERNEL_RELEASE_FILE="${CACHE}/kernel-release"

log()  { printf '\033[1;32m[bootstrap]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[bootstrap]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[bootstrap]\033[0m %s\n' "$*" >&2; exit 1; }

need_command() {
    command -v "$1" >/dev/null 2>&1 || die "$1 is required on the host"
}

is_ready() {
    local kernel modules
    [[ -x "${TOOLS}/usr/bin/qemu-system-x86_64" ]] || return 1
    [[ -x "${TOOLS}/usr/bin/cpio" ]] || return 1
    [[ -f "${TOOLS}/usr/share/seabios/bios-256k.bin" ]] || return 1
    kernel="$(find "${TOOLS}/boot" -maxdepth 1 -name 'vmlinuz-*' -print -quit 2>/dev/null || true)"
    [[ -n "${kernel}" ]] || return 1
    [[ -x "${ROOTFS}/bin/busybox" ]] || return 1
    [[ -x "${ROOTFS}/usr/bin/litric-fbpresent" ]] || return 1
    [[ -x "${ROOTFS}/opt/chrome/chrome" ]] || return 1
    [[ -e "${ROOTFS}/usr/lib/x86_64-linux-gnu/libnss3.so" ]] || return 1
    [[ -e "${ROOTFS}/lib64/ld-linux-x86-64.so.2" ]] || return 1
    modules="$(find "${ROOTFS}/lib/modules" -name '*.ko' -print -quit 2>/dev/null || true)"
    [[ -n "${modules}" ]] || return 1
    return 0
}

if is_ready; then
    log "using complete cache at ${CACHE}"
    exit 0
fi

[[ "$(uname -m)" == "x86_64" ]] || die "this guest proof needs an x86_64 host"
for command in apt-get apt-cache dpkg-deb find sort awk curl python3; do need_command "${command}"; done
need_command sha256sum

mkdir -p "${CACHE}" "${APTROOT}/etc/apt/sources.list.d" \
    "${APTROOT}/etc/apt/preferences.d" "${APTROOT}/etc/apt/apt.conf.d" \
    "${APTROOT}/var/lib/apt/lists/partial" \
    "${APTROOT}/var/cache/apt/archives/partial" \
    "${APTROOT}/var/lib/dpkg"

# APT is pointed at private lists and a private archive cache. The host's
# package database is never touched, so this works without sudo.
APT_OPTS=(
    -o "Dir=${APTROOT}"
    -o "Dir::Etc::sourcelist=${APTROOT}/etc/apt/sources.list"
    -o "Dir::Etc::sourceparts=${APTROOT}/etc/apt/sources.list.d"
    -o "Dir::State::status=${APTROOT}/var/lib/dpkg/status"
    -o "Dir::State::lists=${APTROOT}/var/lib/apt/lists"
    -o "Dir::Cache=${APTROOT}/var/cache/apt"
    -o APT::Get::List-Cleanup=0
    -o APT::Get::Download-Only=true
    -o Dpkg::Use-Pty=0
    # Fail the actionable bootstrap error quickly when egress is unavailable;
    # the default retry loop can otherwise hide the real message for minutes.
    -o Acquire::Retries=0
    -o Acquire::http::Timeout=10
    -o Acquire::https::Timeout=10
)

if [[ ! -f "${SCHEMA_FILE}" || "$(cat "${SCHEMA_FILE}" 2>/dev/null)" != "${SCHEMA}" ]]; then
    log "guest schema ${SCHEMA} needs a fresh package set"
    rm -f "${DOWNLOAD_COMPLETE}" "${COMPLETE}"
    printf '%s\n' "${SCHEMA}" > "${SCHEMA_FILE}"
fi

if [[ ! -f "${APT_SOURCES_READY}" ]]; then
    copied=0
    if [[ -f /etc/apt/sources.list ]]; then
        cp /etc/apt/sources.list "${APTROOT}/etc/apt/sources.list"
        copied=1
    else
        : > "${APTROOT}/etc/apt/sources.list"
    fi
    shopt -s nullglob
    for source in /etc/apt/sources.list.d/*.list /etc/apt/sources.list.d/*.sources; do
        cp "${source}" "${APTROOT}/etc/apt/sources.list.d/"
        copied=1
    done
    shopt -u nullglob
    ((copied == 1)) || die "no /etc/apt source list found; install apt sources first"
    touch "${APT_SOURCES_READY}"
fi

if [[ ! -f "${DOWNLOAD_COMPLETE}" ]]; then
    log "updating private APT lists"
    if ! apt-get "${APT_OPTS[@]}" update; then
        die "APT update failed. Network egress to archive.ubuntu.com and
security.ubuntu.com is required for the first bootstrap. Either enable it and
rerun, or supply both LITRIC_TOOLS and LITRIC_ROOTFS (see docs/QEMU-GUEST-PROOF.md).
The cache that would be filled is ${CACHE}."
    fi
    # apt-get update can exit 0 while every index failed; verify that at least
    # one Release index landed before apt-cache is allowed to fail silently.
    if [[ -z "$(find "${APTROOT}/var/lib/apt/lists" -maxdepth 1 -type f \
        \( -name '*_InRelease' -o -name '*_Release' \) -print -quit)" ]]; then
        die "APT update fetched no package indexes. Network egress to
archive.ubuntu.com and security.ubuntu.com is required for the first bootstrap.
Either enable it and rerun, or supply both LITRIC_TOOLS and LITRIC_ROOTFS (see
docs/QEMU-GUEST-PROOF.md). The cache that would be filled is ${CACHE}."
    fi

    if ! kernel_meta="$(apt-cache "${APT_OPTS[@]}" depends linux-image-generic 2>/dev/null \
        | awk '/^[[:space:]]*Depends: linux-image-[0-9]/{print $2; exit}')"; then
        die "could not query linux-image-generic after APT update; check network
access to the Ubuntu archive, or supply both LITRIC_TOOLS and LITRIC_ROOTFS."
    fi
    [[ -n "${kernel_meta}" ]] || die "APT has no linux-image-generic candidate.
The Ubuntu archive may have changed, or the APT lists are incomplete. Retry
with network access, or supply both LITRIC_TOOLS and LITRIC_ROOTFS."
    kver="${kernel_meta#linux-image-}"
    log "kernel package: ${kernel_meta} (release ${kver})"

    packages=(
        qemu-system-x86 qemu-system-data seabios ipxe-qemu cpio zstd
        busybox-static libc6 libgcc-s1 libffi8 fonts-noto-core
        "${kernel_meta}" "linux-modules-${kver}" "linux-modules-extra-${kver}"
        # Chrome for Testing's ldd closure on Ubuntu 24.04, resolved by APT so
        # the transitive dependencies come with them. The t64 names are the
        # 24.04 ones; a 22.04 host names the same libraries without the suffix.
        libnss3 libnspr4 libatk1.0-0t64 libatk-bridge2.0-0t64 libatspi2.0-0t64
        libcairo2 libcups2t64 libpango-1.0-0 libxcomposite1 libxdamage1
        libxfixes3 libxrandr2 libxkbcommon0 libasound2t64 libglib2.0-0t64
        libdbus-1-3 libfontconfig1 libfreetype6 libx11-6 libxcb1 libxext6
        libxi6 libxtst6 libgbm1 libdrm2 libexpat1 libxrender1 libxshmfence1
        libwayland-client0 fontconfig
    )
    log "downloading ${#packages[@]} requested packages and their dependencies"
    if ! DEBIAN_FRONTEND=noninteractive apt-get "${APT_OPTS[@]}" \
        -y --no-install-recommends install "${packages[@]}"; then
        die "package download failed. Check the network/archive and rerun;
the partial APT cache in ${APTROOT} is reused. Alternatively supply
LITRIC_TOOLS and LITRIC_ROOTFS."
    fi
    printf '%s\n' "${kver}" > "${KERNEL_RELEASE_FILE}"
    touch "${DOWNLOAD_COMPLETE}"
else
    [[ -f "${KERNEL_RELEASE_FILE}" ]] || die "cache has .download-complete but no kernel release marker; remove ${CACHE} and retry"
    kver="$(cat "${KERNEL_RELEASE_FILE}")"
    log "using downloaded package cache for kernel ${kver}"
fi

# Every package is extracted below the cache. The rootfs then copies only the
# files the guest actually needs, keeping the initramfs small.
log "extracting packages into ${TOOLS}"
rm -rf "${TOOLS}"
mkdir -p "${TOOLS}"
shopt -s nullglob
debs=("${APTROOT}"/var/cache/apt/archives/*.deb)
shopt -u nullglob
((${#debs[@]} > 0)) || die "no .deb files in ${APTROOT}/var/cache/apt/archives; delete ${DOWNLOAD_COMPLETE} and rerun"
for deb in "${debs[@]}"; do
    dpkg-deb -x "${deb}" "${TOOLS}"
done

# Chrome for Testing is fetched once and kept in the cache. The checksum is the
# pin: an engine that changes under a fixed version is exactly what the hash
# exists to catch.
log "fetching Chrome for Testing ${CHROME_VERSION}"
mkdir -p "${CHROME_CACHE}"
CHROME_ZIP="${CHROME_CACHE}/chrome-linux64.zip"
if [[ -n "${LITRIC_CHROME_ZIP:-}" ]]; then
    cp "${LITRIC_CHROME_ZIP}" "${CHROME_ZIP}"
elif [[ ! -s "${CHROME_ZIP}" ]] || ! printf '%s  %s\n' "${CHROME_SHA256}" "${CHROME_ZIP}" | sha256sum -c - >/dev/null 2>&1; then
    curl -fsSL --retry 3 --max-time 900 -o "${CHROME_ZIP}.part" "${CHROME_URL}" \
        || die "could not download Chrome for Testing from ${CHROME_URL}.
Network egress to storage.googleapis.com is required. To build the image
offline, set LITRIC_CHROME_ZIP to a local chrome-linux64.zip for ${CHROME_VERSION}."
    mv "${CHROME_ZIP}.part" "${CHROME_ZIP}"
fi
printf '%s  %s\n' "${CHROME_SHA256}" "${CHROME_ZIP}" | sha256sum -c - >/dev/null \
    || die "Chrome for Testing checksum mismatch: ${CHROME_ZIP}"
rm -rf "${CHROME_CACHE}/chrome-linux64"
python3 - "${CHROME_ZIP}" "${CHROME_CACHE}" <<'CHROME_UNZIP'
import sys, zipfile
with zipfile.ZipFile(sys.argv[1]) as archive:
    archive.extractall(sys.argv[2])
CHROME_UNZIP
# Python's zipfile writes the contents but not the Unix mode from the archive,
# so the executables are made executable here. Without this the image has an
# engine binary that cannot be spawned.
for executable in chrome chrome-wrapper chrome_crashpad_handler chrome_sandbox; do
    [[ -f "${CHROME_CACHE}/chrome-linux64/${executable}" ]] \
        && chmod 755 "${CHROME_CACHE}/chrome-linux64/${executable}"
done
[[ -x "${CHROME_CACHE}/chrome-linux64/chrome" ]] \
    || die "the Chrome archive did not contain an executable chrome-linux64/chrome"

QEMU="${TOOLS}/usr/bin/qemu-system-x86_64"
CPIO="${TOOLS}/usr/bin/cpio"
KERNEL="$(find "${TOOLS}/boot" -maxdepth 1 -name 'vmlinuz-*' -print -quit)"
[[ -x "${QEMU}" ]] || die "qemu-system-x86_64 was not extracted"
[[ -x "${CPIO}" ]] || die "cpio was not extracted"
[[ -n "${KERNEL}" ]] || die "kernel image was not extracted"
[[ -f "${TOOLS}/usr/share/seabios/bios-256k.bin" ]] || die "SeaBIOS data was not extracted"
kver="${KERNEL##*/vmlinuz-}"
printf '%s\n' "${kver}" > "${KERNEL_RELEASE_FILE}"

log "building minimal guest rootfs at ${ROOTFS}"
rm -rf "${ROOTFS}"
mkdir -p "${ROOTFS}"/{bin,sbin,usr/bin,usr/sbin,usr/lib,lib64} \
    "${ROOTFS}/lib/x86_64-linux-gnu" "${ROOTFS}/usr/share/fonts" \
    "${ROOTFS}/run/litric" "${ROOTFS}/root/.local/share" \
    "${ROOTFS}/var/lib/litric-shell" "${ROOTFS}/var/lib/litric/update" \
    "${ROOTFS}/var/lib/litric/backup" "${ROOTFS}/var/lib/litric-sysd" \
    "${ROOTFS}/dev" "${ROOTFS}/proc" "${ROOTFS}/sys" "${ROOTFS}/tmp"

BUSYBOX="${TOOLS}/bin/busybox"
if [[ ! -x "${BUSYBOX}" ]]; then
    BUSYBOX="$(find "${TOOLS}" -type f -name busybox -perm -111 -print -quit 2>/dev/null || true)"
fi
[[ -n "${BUSYBOX}" && -x "${BUSYBOX}" ]] || die "static busybox was not extracted"
install -m 755 "${BUSYBOX}" "${ROOTFS}/bin/busybox"
while IFS= read -r applet; do
    [[ -n "${applet}" && "${applet}" != "busybox" ]] || continue
    ln -sf /bin/busybox "${ROOTFS}/bin/${applet}"
done < <("${BUSYBOX}" --list)

lib_source() {
    find "${TOOLS}/lib" "${TOOLS}/usr/lib" -path "*/x86_64-linux-gnu/$1" -print -quit 2>/dev/null || true
}
for lib in libc.so.6 libm.so.6 libgcc_s.so.1 libffi.so.8; do
    source_lib="$(lib_source "${lib}")"
    [[ -n "${source_lib}" ]] || die "guest library ${lib} was not extracted"
    cp -L "${source_lib}" "${ROOTFS}/lib/x86_64-linux-gnu/${lib}"
done
loader="$(lib_source ld-linux-x86-64.so.2)"
[[ -n "${loader}" ]] || die "guest ELF loader was not extracted"
cp -L "${loader}" "${ROOTFS}/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2"
ln -sf /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 \
    "${ROOTFS}/lib64/ld-linux-x86-64.so.2"

for font in NotoSans-Regular.ttf NotoSans-Bold.ttf \
    NotoSansThai-Regular.ttf NotoSansThai-Bold.ttf \
    NotoSansMono-Regular.ttf NotoSansMono-Bold.ttf; do
    font_source="$(find "${TOOLS}/usr/share/fonts" -name "${font}" -print -quit 2>/dev/null || true)"
    [[ -n "${font_source}" ]] || die "font ${font} was not extracted; fonts-noto-core/fonts-noto-mono are required"
    cp "${font_source}" "${ROOTFS}/usr/share/fonts/${font}"
done

install -m 755 "${PROJECT_ROOT}/image/mkosi.extra/usr/bin/litric-fbpresent" \
    "${ROOTFS}/usr/bin/litric-fbpresent" \
    || die "image/mkosi.extra/usr/bin/litric-fbpresent is missing"

# The engine, its data files and its library closure. Chrome resolves its .pak
# files and V8 snapshot next to its own binary, so the directory is copied
# whole. Only the shared objects from the package set are copied: headers,
# documentation and package databases are not part of a booting guest.
mkdir -p "${ROOTFS}/opt" "${ROOTFS}/etc" "${ROOTFS}/home/litric" \
    "${ROOTFS}/run/litric-browser" "${ROOTFS}/usr/share/fontconfig"
cp -a "${CHROME_CACHE}/chrome-linux64" "${ROOTFS}/opt/chrome"
for libdir in lib/x86_64-linux-gnu usr/lib/x86_64-linux-gnu; do
    [[ -d "${TOOLS}/${libdir}" ]] || continue
    mkdir -p "${ROOTFS}/${libdir}"
    find "${TOOLS}/${libdir}" -maxdepth 1 \( -type f -o -type l \) -name '*.so*' \
        -exec cp -a {} "${ROOTFS}/${libdir}/" \;
done
[[ -d "${TOOLS}/etc/fonts" ]] && cp -a "${TOOLS}/etc/fonts" "${ROOTFS}/etc/"
[[ -d "${TOOLS}/usr/share/fontconfig" ]] && cp -a "${TOOLS}/usr/share/fontconfig/." "${ROOTFS}/usr/share/fontconfig/"
# Chromium refuses its sandbox as root, so the broker and the engine run as an
# unprivileged user -- the same arrangement the image's systemd unit uses.
printf '%s\n' 'root:x:0:0:root:/root:/bin/sh' \
    'litric:x:1000:1000:Litric Browser:/home/litric:/bin/sh' > "${ROOTFS}/etc/passwd"
printf '%s\n' 'root:x:0:' 'litric:x:1000:' > "${ROOTFS}/etc/group"
chmod 1777 "${ROOTFS}/tmp"

ZSTD="${TOOLS}/usr/bin/zstd"
[[ -x "${ZSTD}" ]] || die "zstd was not extracted"
for module in ttm drm_ttm_helper drm_vram_helper bochs virtio_input e1000; do
    source_module="$(find "${TOOLS}/lib/modules/${kver}" \
        \( -name "${module}.ko.zst" -o -name "${module}.ko" \) -print -quit 2>/dev/null || true)"
    [[ -n "${source_module}" ]] || die "kernel module ${module}.ko was not found for ${kver}"
    relative="${source_module#${TOOLS}/}"
    if [[ "${relative}" == *.zst ]]; then
        relative="${relative%.zst}"
        mkdir -p "$(dirname "${ROOTFS}/${relative}")"
        LD_LIBRARY_PATH="${TOOLS}/usr/lib/x86_64-linux-gnu:${TOOLS}/lib/x86_64-linux-gnu" \
            "${ZSTD}" -q -d -f "${source_module}" -o "${ROOTFS}/${relative}"
    else
        mkdir -p "$(dirname "${ROOTFS}/${relative}")"
        cp "${source_module}" "${ROOTFS}/${relative}"
    fi
done

printf 'kernel=%s\nqemu=%s\nrootfs=%s\ntools=%s\n' \
    "${kver}" "${QEMU}" "${ROOTFS}" "${TOOLS}" > "${COMPLETE}"
log "bootstrap complete: ${CACHE}"
log "kernel ${kver}; QEMU ${QEMU}"
