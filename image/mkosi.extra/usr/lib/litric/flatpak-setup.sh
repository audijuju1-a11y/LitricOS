#!/usr/bin/bash
# Provision the Flathub remote on first boot, idempotently.
#
# The desktop profile ships `flatpak` (and a portal backend), but the package
# carries no remotes: a fresh image therefore has an empty App Store catalog and
# `STORE SEARCH` answers with no results, which looks like a broken store rather
# than an unconfigured one. Adding the remote at image build time would bake a
# network fetch into the reproducible build; adding it here runs after the
# network is up and is safe to repeat on every boot.
#
# Failure is not a boot failure. A machine that boots without network keeps a
# working desktop; the unit retries on the next boot because the remote is still
# absent. Nothing is installed by this script -- it only declares where apps can
# come from, and the App Store still asks for confirmation before a write.
set -uo pipefail

FLATPAK=/usr/bin/flatpak
REMOTE=flathub
URL=https://flathub.org/repo/flathub.flatpakrepo

if [[ ! -x "$FLATPAK" ]]; then
    printf 'litric-flatpak-setup: flatpak is not installed; nothing to do\n'
    exit 0
fi

# `flatpak remotes` prints one name per line. A missing repo directory is an
# empty list, not an error, so this is also the first-boot path.
if "$FLATPAK" remotes --system --columns=name 2>/dev/null \
        | /usr/bin/grep -qx "$REMOTE"; then
    printf 'litric-flatpak-setup: %s is already configured\n' "$REMOTE"
    exit 0
fi

if "$FLATPAK" remote-add --system --if-not-exists "$REMOTE" "$URL"; then
    printf 'litric-flatpak-setup: added %s (%s)\n' "$REMOTE" "$URL"
else
    printf 'litric-flatpak-setup: could not add %s; will retry on the next boot\n' \
        "$REMOTE" >&2
fi

# Always zero: the desktop is up either way, and a failed unit here would turn a
# missing network into a boot defect on a machine that is otherwise fine.
exit 0
