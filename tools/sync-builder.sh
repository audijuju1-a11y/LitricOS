#!/usr/bin/env bash
# =============================================================================
# LitricOS — copy the working tree onto the native builder, and prove it landed
#
#   ./tools/sync-builder.sh [--dest ~/litricos] [--check-only]
#
# WHY THIS EXISTS
#
# The project lives on a Windows mount, which cannot store Unix ownership or
# permission bits, so tools/build.sh refuses to build there and the real build
# happens from a native copy under $HOME. Every edit therefore has to be carried
# across, and the carrying is the step nobody watches.
#
# It has already gone wrong once, in the way these steps always go wrong: the
# sync used rsync, rsync was not installed, the script was not running under
# `set -e`, and it printed "synced" and continued. The image built cleanly from
# a tree that was two features behind, booted perfectly, and reported the
# absence of code that had in fact been written. Nothing failed. Everything was
# wrong.
#
# So this script does two things a plain copy does not:
#   1. It fails if the copy fails, including when the copying tool is missing.
#   2. It verifies afterwards, by comparing content hashes of every tracked
#      file, that the destination matches the source. A sync that reports
#      success is required to be a sync that happened.
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SRC="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
DEST="${HOME}/litricos"
CHECK_ONLY=0

# Build outputs and caches are per-tree and must never be carried across: a
# stale target/ directory is how a "rebuilt" binary turns out to be yesterday's.
EXCLUDES=(build src/target .git __pycache__ .mypy_cache kernel/artifacts)

log()  { printf '\033[1;32m[sync]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[sync]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[sync]\033[0m %s\n' "$*" >&2; exit 1; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dest) DEST="${2:-}"; shift 2 ;;
        --dest=*) DEST="${1#*=}"; shift ;;
        --check-only) CHECK_ONLY=1; shift ;;
        -h|--help) sed -n '2,28p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown argument: $1" ;;
    esac
done

[[ "$(uname -s)" == "Linux" ]] || die "this runs on the Linux builder, not on the Windows host"
[[ -d "$SRC/image" && -d "$SRC/src" ]] || die "$SRC does not look like the LitricOS tree"
[[ "$SRC" != "$DEST" ]] || die "source and destination are the same directory"

# The list of files that must exist identically on both sides. Generated from
# the source, so a file added today is checked today without editing this list.
# find's predicates are built as an array. Interpolating them from a printf
# leaves the shell free to glob "./build/*" against the current directory, which
# turns one argument into fifty and makes find fail with "paths must precede
# expression". It did exactly that on the first run of this script.
find_args() {
    FIND_ARGS=(-type f)
    local e
    for e in "${EXCLUDES[@]}"; do
        FIND_ARGS+=(-not -path "./${e}/*")
    done
    FIND_ARGS+=(-not -name '*.pyc')
}

manifest() {
    local root="$1"
    find_args
    ( cd "$root" && find . "${FIND_ARGS[@]}" -print0 | sort -z | xargs -0 sha256sum )
}

if (( ! CHECK_ONLY )); then
    mkdir -p "$DEST"
    log "copying $SRC -> $DEST"

    if command -v rsync >/dev/null 2>&1; then
        rsync_args=(-a --delete)
        for e in "${EXCLUDES[@]}"; do rsync_args+=(--exclude "/$e/"); done
        rsync "${rsync_args[@]}" "$SRC"/ "$DEST"/ \
            || die "rsync failed"
    else
        # tar is in base. Not a silent fallback: it says which one ran, because
        # "which tool copied my files" is the first question when they turn out
        # not to have been copied.
        warn "rsync is not installed; using tar"
        tar_args=()
        for e in "${EXCLUDES[@]}"; do tar_args+=(--exclude "./$e"); done
        # --delete has no tar equivalent, so removals are handled explicitly
        # below rather than left to accumulate as phantom files in the copy.
        ( cd "$SRC" && tar "${tar_args[@]}" -cf - . ) \
          | ( cd "$DEST" && tar -xf - ) \
          || die "tar copy failed"

        log "removing files that no longer exist in the source"
        find_args
        ( cd "$DEST" && find . "${FIND_ARGS[@]}" -print )         | while read -r f; do
            [[ -e "$SRC/$f" ]] || { rm -f "$DEST/$f"; log "removed stale $f"; }
        done
    fi
fi

# --- verification -----------------------------------------------------------
# The point of the whole script. A copy is not done until the bytes match.
log "verifying"
src_manifest="$(manifest "$SRC")"
dst_manifest="$(manifest "$DEST")"

if [[ "$src_manifest" == "$dst_manifest" ]]; then
    n="$(printf '%s\n' "$src_manifest" | wc -l)"
    log "OK: $n files identical in $DEST"
    exit 0
fi

printf '\033[1;31m[sync]\033[0m the copy does NOT match the source:\n' >&2
diff <(printf '%s\n' "$src_manifest") <(printf '%s\n' "$dst_manifest") \
    | head -40 >&2 || true
die "destination is not a faithful copy. Do not build from it."
