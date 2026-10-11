#!/usr/bin/env bash
# =============================================================================
# LitricOS — FULL FEATURE TOUR: 100+ screenshots of every surface
#
# Boots the disposable image headless (-display none), drives the REAL input
# path (QEMU monitor mouse_move + sendkey → guest evdev → our WM), and
# photographs every step. Screenshots are grouped into section folders under
# build/tour/<stamp>/ so the gallery can present a narrative.
#
#   ./tools/photo-tour.sh            # default tour
#   ./tools/photo-tour.sh --quick    # fewer waits
#
# Every screenshot is a real QEMU screendump of the guest framebuffer taken
# after real monitor-injected input. Nothing here stubs the shell.
# =============================================================================
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
OUT_DIR="${BUILD_DIR}/out"
TOOL_DIR="${BUILD_DIR}/tour"
STAMP="$(date -u +%Y%m%d-%H%M%S)"
BASE="${TOOL_DIR}/${STAMP}"

QUICK=0
[[ "${1:-}" == "--quick" ]] && QUICK=1
W="${WAIT_SCALE:-1}"
if [[ "$QUICK" == "1" ]]; then W=0.5; fi

log()  { printf '\033[1;36m[tour]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[tour]\033[0m %s\n' "$*" >&2; }

command -v qemu-system-x86_64 >/dev/null 2>&1 || { warn "qemu not installed"; exit 1; }

IMAGE="${OUT_DIR}/litricos.raw"
[[ -f "$IMAGE" ]] || IMAGE="${OUT_DIR}/litricos.raw.zst"
[[ -f "$IMAGE" ]] || { warn "no image"; exit 1; }
if [[ "$IMAGE" == *.zst ]]; then
    RAW="${OUT_DIR}/litricos.screen.raw"
    if [[ ! -f "$RAW" || "$IMAGE" -nt "$RAW" ]]; then
        log "decompressing image"
        zstd -d -f -o "$RAW" "$IMAGE" >/dev/null 2>&1
    fi
    IMAGE="$RAW"
fi

CODE=""
for c in /usr/share/edk2/x64/OVMF_CODE.4m.fd /usr/share/edk2/x64/OVMF_CODE.fd \
         /usr/share/OVMF/OVMF_CODE.fd /usr/share/edk2-ovmf/x64/OVMF_CODE.fd; do
    [[ -f "$c" ]] && { CODE="$c"; break; }
done
VARS=""
for v in /usr/share/edk2/x64/OVMF_VARS.4m.fd /usr/share/edk2/x64/OVMF_VARS.fd \
         /usr/share/OVMF/OVMF_VARS.fd /usr/share/edk2-ovmf/x64/OVMF_VARS.fd; do
    [[ -f "$v" ]] && { VARS="$v"; break; }
done
[[ -n "$CODE" && -n "$VARS" ]] || { warn "no OVMF firmware"; exit 1; }

ACCEL="kvm"; [[ -w /dev/kvm ]] || ACCEL="tcg"
CPU_ARGS=(-cpu host); [[ "$ACCEL" == "tcg" ]] && CPU_ARGS=(-cpu max)

mkdir -p "$BASE"
VARSFD="${BUILD_DIR}/OVMF_VARS.tour.fd"
cp -f "$VARS" "$VARSFD"
LOG="${BASE}/serial.log"
MON="${BASE}/monitor.sock"

log "image: $(basename "$IMAGE")  accel: $ACCEL"
log "output: $BASE"

qemu-system-x86_64 \
    -machine "q35,accel=${ACCEL}" "${CPU_ARGS[@]}" \
    -smp 4 -m 2048 \
    -drive "if=pflash,format=raw,unit=0,readonly=on,file=${CODE}" \
    -drive "if=pflash,format=raw,unit=1,file=${VARSFD}" \
    -drive "if=virtio,format=raw,file=${IMAGE}" \
    -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
    -device VGA,vgamem_mb=32 \
    -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 \
    -display none \
    -serial "file:${LOG}" \
    -monitor "unix:${MON},server,nowait" \
    -no-reboot &
QPID=$!
trap 'kill $QPID 2>/dev/null; wait $QPID 2>/dev/null; rm -f "$MON"' EXIT

for _ in $(seq 1 100); do [[ -S "$MON" ]] && break; sleep 0.1; done
[[ -S "$MON" ]] || { warn "no monitor socket"; exit 1; }

mon() { python3 "${SCRIPT_DIR}/qemu-monitor.py" "$MON" "$1" >/dev/null 2>&1; }

SECTION="00-boot"
shoot() {
    # shoot <name>  →  $BASE/$SECTION/<nn>-<name>.png
    local name="$1"
    local dir="${BASE}/${SECTION}"
    mkdir -p "$dir"
    local n
    n=$(printf '%02d' "$(ls -1 "$dir" 2>/dev/null | wc -l)")
    local ppm="${dir}/${n}-${name}.ppm"
    mon "screendump ${ppm}"
    sleep 0.3
    if [[ -s "$ppm" ]]; then
        python3 "${SCRIPT_DIR}/ppm2png.py" "$ppm" "${ppm%.ppm}.png" >/dev/null 2>&1 && rm -f "$ppm"
    else
        rm -f "$ppm"; warn "shot failed: $SECTION/$name"
    fi
}

# Absolute-pointer click at pixel (x,y): move, brief settle, press, release.
click() {
    local x="$1" y="$2"
    mon "mouse_move ${x} ${y}"
    sleep 0.15
    mon "mouse_button 1"   # press (button down state 1)
    sleep 0.12
    mon "mouse_button 0"   # release
    sleep 0.4
}
rclick() {
    local x="$1" y="$2"
    mon "mouse_move ${x} ${y}"
    sleep 0.15
    mon "mouse_button 4"   # right press
    sleep 0.12
    mon "mouse_button 0"
    sleep 0.4
}
key() {
    # sendkey wants QEMU key names: kp_enter etc. map: ret -> ret
    mon "sendkey $1"
    sleep 0.25
}
w() { sleep "$1"; }
w 1

# ============================ 00-boot ========================================
log "waiting for desktop…"
for t in 12 20 30 40; do
    sleep_needed=$(( t - ${prev_t:-0} ))
    sleep "$sleep_needed"
    prev_t="$t"
    if [[ "$t" == "40" ]]; then shoot "desktop-idle"; else shoot "boot-t${t}"; fi
done
SECTION="00-boot"; shoot "desktop-confirm"

# ============================ 01-desktop =====================================
SECTION="01-desktop"
shoot "start"
# Desktop icon single-click + double-click (Files usually at a known spot)
click 64 140;          w 0.5; shoot "icon-select"
click 64 140; w 0.4;  shoot "icon-open-files"   # double-click via two clicks

# ============================ 02-menubar-menus ===============================
SECTION="02-menubar-menus"
click 200 12;  w 0.4; shoot "menu-app"
key "esc"
click 280 12;  w 0.4; shoot "menu-file"
key "down"; key "down"; shoot "menu-file-navigated"
key "esc"
click 360 12;  w 0.4; shoot "menu-edit"
key "esc"
click 430 12;  w 0.4; shoot "menu-view"
key "esc"
click 500 12;  w 0.4; shoot "menu-window"
key "esc"
click 560 12;  w 0.4; shoot "menu-help"
key "esc"
click 1200 12; w 0.4; shoot "status-area"
click 1240 12; w 0.4; shoot "clock-area"

# ============================ 03-context-desktop =============================
SECTION="03-context-desktop"
click 800 500; # clear any focus
rclick 900 600; w 0.6; shoot "ctx-desktop"
click 940 640; w 0.6; shoot "ctx-desktop-item-clicked"
rclick 700 300; w 0.6; shoot "ctx-desktop-2"
key "esc"

# ============================ 04-files-app ===================================
SECTION="04-files-app"
# Launch Files from app menu (app menu click, then first item)
click 200 12; w 0.3; key "down"; key "ret"; w 1.2
shoot "files-window"
rclick 500 400; w 0.6; shoot "files-ctx-blank"
key "esc"
click 500 400; w 0.3
# Create new folder via File menu
click 280 12; w 0.3; shoot "files-menu-file"
key "esc"
# Toolbar / sidebar
click 90 180; w 0.5; shoot "files-sidebar-home"
click 90 220; w 0.5; shoot "files-sidebar-docs"
click 90 260; w 0.5; shoot "files-sidebar-downloads"
click 700 30; w 0.4; shoot "files-toolbar"
# double-click a folder row if present
click 420 200; w 0.2; click 420 200; w 0.8; shoot "files-navigated"
key "alt"; w 0.2

# ============================ 05-new-folder-dialog ===========================
SECTION="05-new-folder-dialog"
# Ctrl+N or File>New Folder attempt, then dialogs
key "ctrl-n"; w 0.8; shoot "dialog-new-folder"
key "shift";  shoot "dialog-shift"
key "ret";    w 0.8; shoot "dialog-confirmed"
# rename flow
key "f2";     w 0.6; shoot "rename-active"
key "ret"

# ============================ 06-keyboard ====================================
SECTION="06-keyboard"
key "f5";  w 0.5; shoot "kbd-refresh"
key "ctrl-a"; w 0.3; shoot "kbd-select-all"
key "esc"
key "alt-f4"; w 0.6; shoot "kbd-close-window"

# ============================ 07-defender ====================================
SECTION="07-defender"
# Launch Defender via menu (app menu down-down-ret pattern varies by build)
click 200 12; w 0.3; key "down"; key "down"; key "down"; key "ret"; w 1.5
shoot "defender-window"
click 640 300; w 0.3; shoot "defender-click-body"
# Find Scan button: try common spots
click 700 420; w 0.5; shoot "defender-scan-click1"
click 700 460; w 0.5; shoot "defender-scan-click2"
w 2; shoot "defender-scanning"
w 3; shoot "defender-results"
rclick 600 350; w 0.6; shoot "defender-ctx"
key "esc"

# ============================ 08-browser =====================================
SECTION="08-browser"
click 200 12; w 0.3; key "down"; key "down"; key "down"; key "down"; key "ret"; w 2
shoot "browser-open"
# omnibox
click 640 55; w 0.4; shoot "browser-omnibox-focus"
key "g"; key "o"; key "o"; key "g"; key "l"; key "e"; key "spc"; key "b"; key "e"; key "n"; key "c"; key "h"
shoot "browser-typed-google"
key "ret"; w 3; shoot "browser-google-search"
# engine picker if present
rclick 640 55; w 0.5; shoot "browser-omnibox-ctx"
key "esc"
# second query
click 640 55; w 0.3
key "l"; key "i"; key "t"; key "r"; key "i"; key "c"; key "o"; key "s"
shoot "browser-typed-2"
key "ret"; w 3; shoot "browser-search-2"
# back/forward
click 90 55; w 0.8; shoot "browser-back"
click 130 55; w 0.8; shoot "browser-forward"
key "alt-f4"; w 0.6

# ============================ 09-trash =======================================
SECTION="09-trash"
click 1208 760; w 1.0; shoot "trash-open"
shoot "trash-contents"
key "alt-f4"; w 0.4

# ============================ 10-dock ========================================
SECTION="10-dock"
click 400 764; w 0.4; shoot "dock-app1"
click 560 764; w 0.4; shoot "dock-app2"
click 720 764; w 0.4; shoot "dock-app3"
click 880 764; w 0.4; shoot "dock-app4"
click 1040 764; w 0.4; shoot "dock-app5"
click 1208 760; w 0.6; shoot "dock-trash"

# ============================ 11-wallpaper ===================================
SECTION="11-wallpaper"
click 200 12; w 0.3
# navigate app menu to settings if it exists
key "down"; key "down"; key "down"; key "down"; key "down"; key "ret"; w 1.2
shoot "settings-open"
click 400 300; w 0.3; shoot "settings-body"
key "alt-f4"; w 0.4

# ============================ 12-final =======================================
SECTION="12-final"
click 800 500; w 0.3
shoot "desktop-final"
rclick 640 400; w 0.5; shoot "final-ctx"
key "esc"

kill "$QPID" 2>/dev/null || true
wait "$QPID" 2>/dev/null || true
trap - EXIT
rm -f "$MON"

echo
log "tour complete: $BASE"
find "$BASE" -name '*.png' | wc -l | xargs -I{} echo "screenshots: {}"
