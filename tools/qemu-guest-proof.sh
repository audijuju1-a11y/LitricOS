#!/usr/bin/env bash
# =============================================================================
# LitricOS — build and boot a headless QEMU guest, then prove the game path
#
#   ./tools/qemu-guest-proof.sh
#
# On a fresh Ubuntu/Debian x86_64 host this downloads and builds its own
# toolchain and rootfs under build/qemu-bootstrap. To use prepared trees:
#
#   LITRIC_TOOLS=/path/to/tools \
#   LITRIC_ROOTFS=/path/to/rootfs \
#       ./tools/qemu-guest-proof.sh
#
#   LITRIC_QEMU_WIDTH=1920 LITRIC_QEMU_HEIGHT=1080 \
#       LITRIC_QEMU_OUT=artifacts/qemu-guest-proof/1920x1080 \
#       ./tools/qemu-guest-proof.sh
#
# What it builds
#   A direct-kernel initramfs from the current release `litric-sysd` and
#   `litric-shell`, the repository's real `weston-simple-shm` fixture, and the
#   prepared Linux userspace in LITRIC_ROOTFS. It installs the bundle with
#   `APP INSTALL` inside the guest, starts the shell, and drives the GUI with
#   QMP+the virtio absolute tablet. A host HTTP server and QEMU user-mode
#   networking prove that the guest has a working network path. No test fixture
#   is composed in the host.
#
# Prerequisites
#   x86_64 Ubuntu/Debian host with apt-get, dpkg-deb and network access for the
#   first run. tools/qemu-guest-bootstrap.sh downloads the Ubuntu packages into
#   build/qemu-bootstrap without root. Subsequent runs use that cache offline.
#   LITRIC_TOOLS and LITRIC_ROOTFS can instead point at complete prepared trees.
#   LITRIC_QEMU_BOOTSTRAP=0 disables automatic bootstrap.
#
# The script writes only below `build/` and `artifacts/qemu-guest-proof/`.
# It uses TCG because this workspace cannot open /dev/kvm; it does not claim a
# KVM run. The proof is an initramfs/direct-kernel boot, not a mkosi image.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
OUT="${LITRIC_QEMU_OUT:-${PROJECT_ROOT}/artifacts/qemu-guest-proof}"
WIDTH="${LITRIC_QEMU_WIDTH:-1280}"
HEIGHT="${LITRIC_QEMU_HEIGHT:-800}"
TOOLS="${LITRIC_TOOLS:-}"
ROOTFS="${LITRIC_ROOTFS:-}"
BOOTSTRAP="${LITRIC_QEMU_BOOTSTRAP:-1}"
BOOTSTRAP_CACHE="${LITRIC_BOOTSTRAP_CACHE:-${PROJECT_ROOT}/build/qemu-bootstrap}"
SKIP_BUILD="${LITRIC_SKIP_BUILD:-0}"
DRIVER="${LITRIC_QEMU_DRIVER:-${PROJECT_ROOT}/tools/qemu_guest_proof.py}"
ACCEL="${LITRIC_QEMU_ACCEL:-tcg}"
if [[ "${ACCEL}" == "kvm" ]]; then
    CPU_MODEL="host"
else
    CPU_MODEL="max"
fi
GAME_BUNDLE="${LITRIC_GAME_BUNDLE:-}"
GAME_STAGE="${LITRIC_GAME_STAGE:-}"
GAME_ID="${LITRIC_GAME_ID:-com.litric.test-game}"

stage="${OUT}/rootfs"
qemu_data="${OUT}/qemu-data"
serial="${OUT}/serial.log"
qemu_log="${OUT}/qemu.log"
qmp="${OUT}/qmp.sock"
network_port="${LITRIC_QEMU_NET_PORT:-18080}"
network_dir="${OUT}/network"
network_server_pid=""
qemu_pid=""

log()  { printf '\033[1;32m[qemu-proof]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[qemu-proof]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[qemu-proof]\033[0m %s\n' "$*" >&2; exit 1; }

cleanup() {
    if [[ -n "${qemu_pid}" ]] && kill -0 "${qemu_pid}" 2>/dev/null; then
        kill "${qemu_pid}" 2>/dev/null || true
        wait "${qemu_pid}" 2>/dev/null || true
    fi
    if [[ -n "${network_server_pid}" ]] && kill -0 "${network_server_pid}" 2>/dev/null; then
        kill "${network_server_pid}" 2>/dev/null || true
        wait "${network_server_pid}" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

[[ "${WIDTH}" =~ ^[0-9]+$ && "${HEIGHT}" =~ ^[0-9]+$ ]] \
    || die "LITRIC_QEMU_WIDTH and LITRIC_QEMU_HEIGHT must be positive integers"
(( WIDTH >= 320 && HEIGHT >= 200 && WIDTH <= 2560 && HEIGHT <= 1600 )) \
    || die "guest geometry ${WIDTH}x${HEIGHT} is outside the shell's supported 320x200..2560x1600 range"

# Bootstrap only when neither tree was supplied. Supplying one tree is an
# error: a mismatched kernel and module set is worse than a clear stop.
if [[ -z "${TOOLS}" && -z "${ROOTFS}" ]]; then
    if [[ "${BOOTSTRAP}" != "1" ]]; then
        die "LITRIC_QEMU_BOOTSTRAP=0 requires both LITRIC_TOOLS and LITRIC_ROOTFS"
    fi
    [[ -x "${SCRIPT_DIR}/qemu-guest-bootstrap.sh" ]] \
        || die "missing executable tools/qemu-guest-bootstrap.sh"
    log "bootstrapping QEMU toolchain and rootfs at ${BOOTSTRAP_CACHE}"
    "${SCRIPT_DIR}/qemu-guest-bootstrap.sh" --cache "${BOOTSTRAP_CACHE}"
    TOOLS="${BOOTSTRAP_CACHE}/tools"
    ROOTFS="${BOOTSTRAP_CACHE}/rootfs"
elif [[ -z "${TOOLS}" || -z "${ROOTFS}" ]]; then
    die "set both LITRIC_TOOLS and LITRIC_ROOTFS, or neither to bootstrap"
fi

[[ -d "${TOOLS}" ]] || die "LITRIC_TOOLS is not a directory: ${TOOLS}"
[[ -d "${ROOTFS}" ]] || die "LITRIC_ROOTFS is not a directory: ${ROOTFS}"
[[ -f "${DRIVER}" ]] || die "LITRIC_QEMU_DRIVER is not a file: ${DRIVER}"
[[ -x "${ROOTFS}/bin/busybox" ]] || die "rootfs has no static /bin/busybox: ${ROOTFS}"

QEMU="${LITRIC_QEMU:-}"
if [[ -z "${QEMU}" ]]; then
    if [[ -x "${TOOLS}/usr/bin/qemu-system-x86_64" ]]; then
        QEMU="${TOOLS}/usr/bin/qemu-system-x86_64"
    else
        QEMU="$(command -v qemu-system-x86_64 || true)"
    fi
fi
[[ -n "${QEMU}" && -x "${QEMU}" ]] || die "qemu-system-x86_64 not found (set LITRIC_QEMU)"

CPIO="${LITRIC_CPIO:-}"
if [[ -z "${CPIO}" ]]; then
    if [[ -x "${TOOLS}/usr/bin/cpio" ]]; then
        CPIO="${TOOLS}/usr/bin/cpio"
    else
        CPIO="$(command -v cpio || true)"
    fi
fi
[[ -n "${CPIO}" && -x "${CPIO}" ]] || die "cpio not found (set LITRIC_CPIO)"

KERNEL="${LITRIC_KERNEL:-}"
if [[ -z "${KERNEL}" ]]; then
    KERNEL="$(find "${TOOLS}/boot" -maxdepth 1 -name 'vmlinuz-*' -print -quit 2>/dev/null || true)"
fi
[[ -n "${KERNEL}" && -f "${KERNEL}" ]] || die "kernel not found (set LITRIC_KERNEL)"

CARGO_BIN="${CARGO:-}"
if [[ -z "${CARGO_BIN}" ]]; then
    CARGO_BIN="$(command -v cargo || true)"
fi
if [[ -z "${CARGO_BIN}" && -x "${HOME}/.cargo/bin/cargo" ]]; then
    CARGO_BIN="${HOME}/.cargo/bin/cargo"
fi

if [[ "${SKIP_BUILD}" != "1" ]]; then
    [[ -n "${CARGO_BIN}" ]] || die "cargo not found; set LITRIC_SKIP_BUILD=1 with prebuilt release binaries"
    log "building release litric-sysd, litric-shell, litric-browserd and litric-filesd"
    (cd "${PROJECT_ROOT}/src" && PATH="$(dirname "${CARGO_BIN}"):${PATH}" "${CARGO_BIN}" build --release \
        -p litric-sysd -p litric-shell -p litric-browser -p litric-files)
fi
for binary in litric-sysd litric-shell litric-browserd litric-filesd; do
    [[ -x "${PROJECT_ROOT}/src/target/release/${binary}" ]] \
        || die "missing solution binary src/target/release/${binary}"
done

log "staging guest rootfs at ${stage}"
rm -rf "${stage}"
mkdir -p "${stage}"
cp -a "${ROOTFS}/." "${stage}/"
rm -rf "${stage}/root/.local/share/litric/apps"
rm -rf "${stage}/root/.local/share/litric/incoming/game"
rm -f "${stage}/root/.local/share/applications/"*.desktop 2>/dev/null || true
mkdir -p \
    "${stage}/run/litric" \
    "${stage}/var/lib/litric/update" \
    "${stage}/var/lib/litric/backup" \
    "${stage}/var/lib/litric-sysd" \
    "${stage}/var/lib/litric-shell" \
    "${stage}/var/lib/litric-files/root" \
    "${stage}/root/.local/share/litric/incoming/game" \
    "${stage}/root/.local/share/applications"

install -m 755 "${PROJECT_ROOT}/src/target/release/litric-sysd" "${stage}/usr/bin/litric-sysd"
install -m 755 "${PROJECT_ROOT}/src/target/release/litric-shell" "${stage}/usr/bin/litric-shell"
install -m 755 "${PROJECT_ROOT}/src/target/release/litric-browserd" "${stage}/usr/bin/litric-browserd"
install -m 755 "${PROJECT_ROOT}/src/target/release/litric-filesd" "${stage}/usr/bin/litric-filesd"
if [[ -n "${GAME_BUNDLE}" ]]; then
    [[ -x "${GAME_BUNDLE}/app" ]] || die "LITRIC_GAME_BUNDLE has no executable app: ${GAME_BUNDLE}"
    [[ -f "${GAME_BUNDLE}/app.desktop" ]] || die "LITRIC_GAME_BUNDLE has no app.desktop: ${GAME_BUNDLE}"
    install -m 755 "${GAME_BUNDLE}/app" \
        "${stage}/root/.local/share/litric/incoming/game/app"
    install -m 644 "${GAME_BUNDLE}/app.desktop" \
        "${stage}/root/.local/share/litric/incoming/game/app.desktop"
else
    install -m 755 "${PROJECT_ROOT}/src/litric-shell/tests/fixtures/weston-simple-shm" \
        "${stage}/root/.local/share/litric/incoming/game/app"
    install -m 644 "${PROJECT_ROOT}/src/litric-shell/tests/fixtures/litric-test-game.desktop" \
        "${stage}/root/.local/share/litric/incoming/game/app.desktop"
fi
if [[ -n "${GAME_STAGE}" ]]; then
    [[ -d "${GAME_STAGE}" ]] || die "LITRIC_GAME_STAGE is not a directory: ${GAME_STAGE}"
    cp -a "${GAME_STAGE}/." "${stage}/"
fi
install -m 644 "${PROJECT_ROOT}/src/litric-shell/tests/fixtures/wayland-libs/libwayland-client.so.0" \
    "${stage}/usr/lib/libwayland-client.so.0"
if [[ -f "${PROJECT_ROOT}/image/mkosi.extra/usr/bin/litric-fbpresent" ]]; then
    install -m 755 "${PROJECT_ROOT}/image/mkosi.extra/usr/bin/litric-fbpresent" \
        "${stage}/usr/bin/litric-fbpresent"
fi
[[ -x "${stage}/usr/bin/litric-fbpresent" ]] \
    || die "the guest rootfs has no /usr/bin/litric-fbpresent; include it before boot"
# The KMS path is part of the product, not a host-only fixture. Build the
# audited ioctl helper into this guest so the shell can choose DRM when the
# virtual card exposes a card node, instead of silently falling back to fbdev
# because the helper was never staged (the old proof's exact failure).
DRM_HELPER_SOURCE="${PROJECT_ROOT}/src/litric-shell/src/drmpresent.c"
[[ -f "${DRM_HELPER_SOURCE}" ]] \
    || die "missing ${DRM_HELPER_SOURCE}; the KMS output path cannot be built"
command -v cc >/dev/null 2>&1 \
    || die "a C compiler is required for litric-drmpresent"
cc -std=c17 -O2 -Wall -Wextra -Werror "${DRM_HELPER_SOURCE}" \
    -o "${stage}/usr/bin/litric-drmpresent"
chmod 0755 "${stage}/usr/bin/litric-drmpresent"
if ! find "${stage}/lib/modules" -name virtio_input.ko -type f -print -quit 2>/dev/null | grep -q .; then
    die "the guest rootfs has no virtio_input.ko; the QMP absolute pointer would not exist"
fi
if ! find "${stage}/lib/modules" -name e1000.ko -type f -print -quit 2>/dev/null | grep -q .; then
    die "the guest rootfs has no e1000.ko; the user-mode network probe would not start"
fi

# The image is pre-onboarded: the proof is about the Games page, not about
# clicking through first-run setup. The settings file is regular rootfs state.
printf 'version=1\nonboarded=1\nlanguage=en\ntheme=dark\n' \
    > "${stage}/var/lib/litric-shell/settings.conf"

cat > "${stage}/init" <<'INIT'
#!/bin/sh
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev 2>/dev/null || true
mkdir -p /dev/shm
mount -t tmpfs tmpfs /dev/shm 2>/dev/null || true
mkdir -p /run/litric /tmp /var/lib/litric-shell /var/lib/litric-sysd \
    /var/lib/litric/update /var/lib/litric/backup /root/.local/share/applications \
    /var/lib/litric-files/root /home/litric /run/litric-browser/profile \
    /run/litric-browser/Downloads
export PATH=/bin:/sbin:/usr/bin:/usr/sbin
export HOME=/root XDG_RUNTIME_DIR=/run/litric XDG_DATA_HOME=/root/.local/share XDG_DATA_DIRS=/usr/share
export LD_LIBRARY_PATH=/usr/lib:/lib/x86_64-linux-gnu
echo LITRIC_INIT_START
for module in ttm drm_ttm_helper drm_vram_helper bochs virtio_input e1000; do
    path=$(find /lib/modules -name "$module.ko" -print -quit)
    if [ -n "$path" ]; then
        insmod "$path" && echo "LITRIC_MODULE_OK=$module" || echo "LITRIC_MODULE_FAIL=$module"
    fi
done
sleep 1
echo LITRIC_FB:
cat /sys/class/graphics/fb0/virtual_size 2>/dev/null || true
echo LITRIC_GPU:
ls -l /dev/dri 2>/dev/null || true
for card in /sys/class/drm/card[0-9]; do
    [ -e "$card/device/driver" ] || continue
    printf 'LITRIC_GPU_CARD=%s driver=%s\n' "${card##*/}" "$(basename "$(readlink "$card/device/driver")")"
done
echo LITRIC_INPUT:
ls /dev/input 2>/dev/null || true
NETPORT=18080
for word in $(cat /proc/cmdline 2>/dev/null); do
    case "$word" in
        litric.net_port=*) NETPORT=${word#*=} ;;
    esac
done
echo LITRIC_NET_BEGIN
ip link set lo up 2>/dev/null || true
if ip link show eth0 >/dev/null 2>&1; then
    # QEMU user-mode networking's fixed guest address. This avoids making the
    # proof depend on a DHCP client that may block before the daemon starts.
    ip link set eth0 up 2>/dev/null || true
    ip addr add 10.0.2.15/24 dev eth0 2>/dev/null || true
    ip route add default via 10.0.2.2 2>/dev/null || true
    # QEMU user-mode networking's DNS forwarder. Without this a page load fails
    # at name resolution even though the link and the TCP path are fine.
    printf 'nameserver 10.0.2.3\n' > /etc/resolv.conf
    sleep 1
    if wget -q -O /tmp/network-proof "http://10.0.2.2:${NETPORT}/proof.txt"; then
        echo LITRIC_NETWORK_OK
        cat /tmp/network-proof
    else
        echo LITRIC_NETWORK_FAILED
        ip addr 2>/dev/null || true
        ip route 2>/dev/null || true
    fi
else
    echo LITRIC_NETWORK_FAILED_NO_ETH0
    ip link 2>/dev/null || true
fi
/usr/bin/litric-sysd --serve --socket /run/litric/sysd.sock --home /root --root / \
    --update-root /var/lib/litric/update --backup-root /var/lib/litric/backup \
    >/tmp/sysd.log 2>&1 &
SYSD=$!
i=0
while [ ! -S /run/litric/sysd.sock ] && [ "$i" -lt 50 ]; do
    sleep 0.1
    i=$((i + 1))
done
if ! kill -0 "$SYSD" 2>/dev/null; then
    echo LITRIC_SYSD_FAILED
    cat /tmp/sysd.log
    poweroff -f 2>/dev/null || reboot -f
fi
cat /tmp/sysd.log
echo LITRIC_INSTALL:
INSTALL=$(/usr/bin/litric-sysd --query 'APP INSTALL __GAME_ID__ game' 2>&1)
echo "$INSTALL"
case "$INSTALL" in
    *state=installed*) echo LITRIC_INSTALL_OK ;;
    *) echo LITRIC_INSTALL_FAILED; poweroff -f 2>/dev/null || reboot -f ;;
esac
echo LITRIC_INSTALLED:
find /root/.local/share -maxdepth 5 -type f 2>/dev/null
# The engine broker. It binds the path the shell connects to, then waits. The
# engine itself starts on the first real navigation; the start page is the
# shell's own surface and never spawns Chromium.
echo LITRIC_BROWSERD_BEGIN
chown -R 1000:1000 /run/litric-browser /home/litric
# BusyBox ships `su` but its `setpriv` has no --reuid, so the unprivileged
# transition goes through su. `exec` keeps one process so this pid stays the
# broker's for the readiness check.
su litric -s /bin/sh -c 'exec env HOME=/home/litric XDG_RUNTIME_DIR=/run/litric-browser BROWSERD_SOCK=/run/litric-browser/browser.sock BROWSERD_CHROMIUM=/opt/chrome/chrome /usr/bin/litric-browserd --profile /run/litric-browser/profile --downloads /run/litric-browser/Downloads' \
    >/tmp/browserd.log 2>&1 &
BROWSERD=$!
i=0
while [ ! -S /run/litric-browser/browser.sock ] && [ "$i" -lt 30 ]; do
    if ! kill -0 "$BROWSERD" 2>/dev/null; then
        break
    fi
    sleep 1
    i=$((i + 1))
done
if [ -S /run/litric-browser/browser.sock ]; then
    echo LITRIC_BROWSERD_READY
else
    echo LITRIC_BROWSERD_FAILED
fi
cat /tmp/browserd.log 2>/dev/null
echo LITRIC_BROWSERD_END

# The file service (ADR-0019) is what takes a finished browser download out of
# the broker's staging directory and puts it in the user's /Downloads. Without
# it the browser can download but nobody can hand the file to the user, which is
# the difference this proof is about. It runs as root because the documents
# belong to the system, not to the browser's unprivileged account.
echo LITRIC_FILESD_BEGIN
mkdir -p /var/lib/litric-files/root/Downloads /var/lib/litric-files/root/Documents
/usr/bin/litric-filesd --socket /run/litric/files.sock --root /var/lib/litric-files/root \
    --import-dir /run/litric-browser/Downloads >/tmp/filesd.log 2>&1 &
i=0
while [ ! -S /run/litric/files.sock ] && [ "$i" -lt 30 ]; do
    sleep 0.2
    i=$((i + 1))
done
if [ -S /run/litric/files.sock ]; then
    echo LITRIC_FILESD_READY
else
    echo LITRIC_FILESD_FAILED
fi
cat /tmp/filesd.log 2>/dev/null
echo LITRIC_FILESD_END


# Watch both halves of the download: the engine's own staged file appears once
# Chromium has finished writing it, and the imported copy appears once the file
# service has handed it to /Downloads. Each line carries the real size and
# digest read from this guest's disk, so the host does not have to trust either
# process's word for it. Any finished file counts: the proof picks a candidate
# at run time, so a fixed filename here would be a second place to change.
(
    staged_seen=" "
    saved_seen=" "
    i=0
    while [ "$i" -lt 600 ]; do
        for f in /run/litric-browser/Downloads/*; do
            [ -f "$f" ] || continue
            name=${f##*/}
            case "$name" in
                *.crdownload)
                    # A growing partial file is the only direct evidence that a
                    # download is still in flight. The proof waits for one of
                    # these before it starts the second download, so "both
                    # downloads overlapped" is measured from this guest's disk
                    # rather than assumed from the order of two keystrokes.
                    echo "LITRIC_DOWNLOAD_ACTIVE name=$name size=$(wc -c < "$f")"
                    continue
                    ;;
            esac
            case "$staged_seen" in *" $name "*) continue ;; esac
            staged_seen="$staged_seen$name "
            echo "LITRIC_DOWNLOAD_STAGED name=$name size=$(wc -c < "$f") sha256=$(sha256sum "$f" | cut -d' ' -f1)"
        done
        for f in /var/lib/litric-files/root/Downloads/*; do
            [ -f "$f" ] || continue
            name=${f##*/}
            case "$saved_seen" in *" $name "*) continue ;; esac
            saved_seen="$saved_seen$name "
            echo "LITRIC_DOWNLOAD_SAVED name=$name size=$(wc -c < "$f") sha256=$(sha256sum "$f" | cut -d' ' -f1)"
        done
        if [ $((i % 5)) -eq 0 ]; then
            echo "LITRIC_DOWNLOAD_LIST staging=$(ls -l /run/litric-browser/Downloads 2>/dev/null | tail -n +2 | awk '{print $5":"$9}' | tr '\n' ',') saved=$(ls -l /var/lib/litric-files/root/Downloads 2>/dev/null | tail -n +2 | awk '{print $5":"$9}' | tr '\n' ',')"
        fi
        sleep 1
        i=$((i + 1))
    done
    echo LITRIC_DOWNLOAD_MISSING
) &
# The broker's own log is on the guest's /tmp, which the host cannot read
# after the fact. One filtered line every half minute is enough to see a
# Chrome-side network or download error without drowning the serial console.
(
    i=0
    while [ "$i" -lt 40 ]; do
        sleep 30
        tail -n 1 /tmp/browserd.log 2>/dev/null | sed "s/^/LITRIC_BROWSERD_TAIL /"
        i=$((i + 1))
    done
) &
# Real guest telemetry. The host reads these lines from the serial console;
# no host-side estimate is used for CPU or resident memory. The stat line is
# emitted whole because utime/stime are needed to compute a live CPU rate.
(
    while true; do
        echo LITRIC_METRIC_BEGIN
        echo "LITRIC_METRIC uptime=$(cut -d. -f1 /proc/uptime)"
        echo "LITRIC_METRIC mem_total_kb=$(awk '/MemTotal:/{print $2}' /proc/meminfo)"
        echo "LITRIC_METRIC mem_available_kb=$(awk '/MemAvailable:/{print $2}' /proc/meminfo)"
        for proc in /proc/[0-9]*; do
            [ -r "$proc/stat" ] || continue
            comm=$(cat "$proc/comm" 2>/dev/null) || continue
            case "$comm" in
                chrome|chromium|litric-browserd|litric-shell|litric-sysd|chocolate-doom) ;;
                *) continue ;;
            esac
            statline=$(cat "$proc/stat" 2>/dev/null) || continue
            rss=$(awk '/VmRSS:/{print $2}' "$proc/status" 2>/dev/null)
            echo "LITRIC_PROC pid=${proc##*/} comm=$comm rss_kb=${rss:-0} stat=$statline"
        done
        echo LITRIC_METRIC_END
        sleep 5
    done
) &
# A native game writes its own log. Copy it to the serial console; the host
# cannot read the guest's /tmp after QEMU exits.
(
    i=0
    while [ "$i" -lt 40 ]; do
        sleep 3
        if [ -f /tmp/doom.log ]; then
            echo LITRIC_DOOM_LOG_BEGIN
            tail -c 4000 /tmp/doom.log
            echo LITRIC_DOOM_LOG_END
        fi
        i=$((i + 1))
    done
) &
LITRIC_BROWSER_SOCK=/run/litric-browser/browser.sock \
/usr/bin/litric-shell --socket /run/litric/sysd.sock --status /run/litric/shell.status \
    --seconds 900 >/dev/console 2>&1 &
SPID=$!
echo "LITRIC_SHELL_PID=$SPID"
i=0
while kill -0 "$SPID" 2>/dev/null; do
    sleep 5
    i=$((i + 1))
    echo "LITRIC_TICK=$i"
    echo LITRIC_PS_BEGIN
    ps -ef 2>/dev/null | grep -E 'litric|test-game' || true
    echo LITRIC_PS_END
    if [ -f /run/litric/shell.status ]; then
        echo LITRIC_STATUS_BEGIN
        cat /run/litric/shell.status
        echo LITRIC_STATUS_END
    fi
done
echo LITRIC_BROWSERD_LOG_BEGIN
cat /tmp/browserd.log 2>/dev/null || true
echo LITRIC_BROWSERD_LOG_END
echo LITRIC_GUEST_DONE
sync
poweroff -f 2>/dev/null || reboot -f
INIT
sed -i "s/__GAME_ID__/${GAME_ID}/g" "${stage}/init"
chmod 755 "${stage}/init"

log "packing initramfs"
mkdir -p "${OUT}"
(cd "${stage}" && find . -print0 | "${CPIO}" --null -o -H newc 2>/dev/null | gzip -9) \
    > "${OUT}/litricos-qemu-initramfs.cpio.gz"
[[ -s "${OUT}/litricos-qemu-initramfs.cpio.gz" ]] || die "initramfs is empty"

log "preparing QEMU BIOS/data directory"
rm -rf "${qemu_data}"
mkdir -p "${qemu_data}"
for share in qemu seabios; do
    if [[ -d "${TOOLS}/usr/share/${share}" ]]; then
        cp -a "${TOOLS}/usr/share/${share}/." "${qemu_data}/" 2>/dev/null || true
    fi
done
# The e1000 NIC option ROM lives in the ipxe package, not the qemu package.
# Without it QEMU refuses to instantiate the device before the guest boots.
if [[ -d "${TOOLS}/usr/lib/ipxe/qemu" ]]; then
    cp -a "${TOOLS}/usr/lib/ipxe/qemu/." "${qemu_data}/" 2>/dev/null || true
fi
if [[ ! -f "${qemu_data}/bios-256k.bin" && -f "${TOOLS}/share/bios-256k.bin" ]]; then
    cp -a "${TOOLS}/share/." "${qemu_data}/" 2>/dev/null || true
fi
[[ -f "${qemu_data}/bios-256k.bin" ]] || die "no bios-256k.bin in the QEMU data tree"

rm -f "${qmp}" "${serial}" "${qemu_log}"
mkdir -p "${network_dir}"
printf 'litric-network-proof\n' > "${network_dir}/proof.txt"
# QEMU user-mode networking gives the guest 10.0.2.2 as the host. A real HTTP
# request proves the guest has a working interface, DHCP lease and userspace
# network path; the NIC model alone would only prove QEMU accepted a flag.
command -v python3 >/dev/null 2>&1 || die "python3 is required for the guest network probe"
python3 -m http.server "${network_port}" --bind 127.0.0.1 --directory "${network_dir}" \
    >"${OUT}/network-server.log" 2>&1 &
network_server_pid=$!
sleep 0.3
kill -0 "${network_server_pid}" 2>/dev/null \
    || die "the guest network probe server did not start (port ${network_port})"
log "booting ${ACCEL} guest at ${WIDTH}x${HEIGHT} (serial: ${serial})"
env \
    LD_LIBRARY_PATH="${TOOLS}/lib/x86_64-linux-gnu:${TOOLS}/usr/lib/x86_64-linux-gnu" \
    QEMU_MODULE_DIR="${TOOLS}/usr/lib/x86_64-linux-gnu/qemu" \
    "${QEMU}" \
    -L "${qemu_data}" \
    -machine q35,accel=${ACCEL} -cpu ${CPU_MODEL} -smp 2 -m 4096 \
    -display none -nic user,model=e1000 -monitor none -no-reboot \
    -device "VGA,edid=on,xres=${WIDTH},yres=${HEIGHT},vgamem_mb=16" \
    -device virtio-tablet-pci \
    -serial "file:${serial}" \
    -qmp "unix:${qmp},server=on,wait=off" \
    -kernel "${KERNEL}" \
    -initrd "${OUT}/litricos-qemu-initramfs.cpio.gz" \
    -append "console=ttyS0,115200 rdinit=/init loglevel=5 panic=-1 litric.net_port=${network_port}" \
    >"${qemu_log}" 2>&1 &
qemu_pid=$!
log "QEMU pid ${qemu_pid} (framebuffer request ${WIDTH}x${HEIGHT})"

for _ in $(seq 1 120); do
    [[ -S "${qmp}" ]] && break
    kill -0 "${qemu_pid}" 2>/dev/null || break
    sleep 0.5
done
if [[ ! -S "${qmp}" ]]; then
    cat "${qemu_log}" >&2 || true
    die "QEMU did not create its QMP socket"
fi
set +e
python3 "${DRIVER}" \
    --qmp "${qmp}" \
    --serial "${serial}" \
    --out "${OUT}" \
    --width "${WIDTH}" \
    --height "${HEIGHT}" \
    --net-port "${network_port}" \
    --timeout 300
proof_status=$?
set -e

if [[ "${proof_status}" -ne 0 ]]; then
    warn "proof failed; serial tail:"
    tail -120 "${serial}" 2>/dev/null || true
    warn "QEMU stderr:"
    tail -60 "${qemu_log}" 2>/dev/null || true
    exit "${proof_status}"
fi

log "PASS — screenshots and result: ${OUT}"
log "serial: ${serial}"
log "initramfs: ${OUT}/litricos-qemu-initramfs.cpio.gz"
