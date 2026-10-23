#!/usr/bin/bash
# First-boot credential setup. This runs only on tty1 before the desktop takes
# the display, and uses the distribution's passwd tool so the password is
# entered with terminal echo disabled and is written through the normal shadow
# locking/update path. No password is stored in this script, argv or journal.
set -euo pipefail

shadow_entry="$(/usr/bin/getent shadow litric || true)"
if [[ -z "$shadow_entry" ]]; then
    printf '\nLitricOS cannot start setup: the litric account is missing.\n'
    exit 1
fi

# Clearing the screen is a printf, not `clear`. The image ships no terminfo
# database, so `clear` exits 1 with "terminals database is inaccessible" -- and
# with `set -e` that failure killed the whole script before it printed a
# single line. The unit restarted it every two seconds, so the first boot of
# every image this script was in was a prompt that flashed and disappeared, with
# the desktop never starting behind it. An ANSI erase needs nothing installed.
clear_screen() { printf '\033[2J\033[H'; }

# An image that is provisioned by something else -- an installer, an automated
# boot -- says so on the command line and gets no prompt at all: the language
# defaults, no password is set, and the desktop starts. The account stays
# locked, so nothing can log in until a password is set.
if grep -qw 'litric.first_run=skip' /proc/cmdline; then
    language=en
    if grep -qw 'litric.lang=th' /proc/cmdline; then language=th; fi
    settings_dir=/var/lib/litric-shell
    /usr/bin/mkdir -p "$settings_dir"
    /usr/bin/chmod 0700 "$settings_dir"
    printf 'language=%s\n' "$language" > "$settings_dir/settings.conf"
    /usr/bin/chmod 0600 "$settings_dir/settings.conf"
    printf 'LitricOS first run: skipped by litric.first_run=skip (language=%s)\n' "$language"
    exit 0
fi

IFS=: read -r _ hash _ <<< "$shadow_entry"
if [[ "$hash" == '!'* || "$hash" == '*'* || -z "$hash" ]]; then
    clear_screen
    printf '\n  LitricOS first run\n\n'
    printf '  Choose your interface language / เลือกภาษาที่ใช้:\n'
    printf '    1) ไทย\n'
    printf '    2) English\n'
    printf '  Selection [1]: '
    IFS= read -r language_choice || language_choice=1
    case "$language_choice" in
        2|en|EN|english|English) language=en ;;
        *) language=th ;;
    esac
    settings_dir=/var/lib/litric-shell
    /usr/bin/mkdir -p "$settings_dir"
    /usr/bin/chmod 0700 "$settings_dir"
    settings_file="$settings_dir/settings.conf"
    temp_file="$settings_dir/settings.conf.first-run.$$"
    if [[ -f "$settings_file" ]]; then
        /usr/bin/grep -v '^language=' "$settings_file" > "$temp_file" || true
    else
        : > "$temp_file"
    fi
    printf 'language=%s\n' "$language" >> "$temp_file"
    /usr/bin/chmod 0600 "$temp_file"
    /usr/bin/mv -f "$temp_file" "$settings_file"

    printf '  Set a password for the litric desktop account.\n'
    printf '  The password will not appear while you type.\n\n'
    exec /usr/bin/passwd litric
fi

exit 0
