# BUG-0022 — The first-boot prompt flashed and disappeared

```
Bug ID        : BUG-0022
Severity      : S1 (CRITICAL) — no image built without `ncurses` ever reaches
                the desktop on its first boot
Class         : BOOT / FIRST RUN
Subsystem     : image/mkosi.extra/usr/lib/litric/first-run.sh,
                image/mkosi.extra/usr/lib/systemd/system/litric-first-run.service
First Observed: 2026-10-17, on the first boot of an image that had not already
                been through this script
Related       : BUG-0021 — the same class of defect: the firmware boot path of a
                freshly built image had never been executed
Status        : **FIXED 2026-10-17** — the script clears the screen with an ANSI
                escape instead of `clear`, `TERM=linux` is set on the unit, and a
                `litric.first_run=skip` command line provisions the language
                without a prompt. Verified by booting the rebuilt image: first
                run exits 0 and the desktop starts
```

## Symptom

The console shows the same few lines of text, over and over, a couple of seconds
apart; the desktop never appears:

```
[   24.324771] audit: type=1130 ... unit=litric-first-run ... res=failed
[   26.596525] audit: type=1130 ... unit=litric-first-run ... res=failed
[   28.872114] audit: type=1130 ... unit=litric-first-run ... res=failed
```

`systemctl status` inside the guest:

```
● litric-first-run.service - Set the LitricOS desktop password on first boot
     Active: activating (auto-restart) (Result: exit-code) since ...
    Process: 1118 ExecStart=/usr/bin/bash /usr/lib/litric/first-run.sh (code=exited, status=1/FAILURE)
```

## Root cause

Run with `bash -x` inside the running guest:

```
+ set -euo pipefail
++ /usr/bin/getent shadow litric
+ shadow_entry='litric:!*:20326::::::'
+ [[ -z litric:!*:20326:::::: ]]
+ IFS=:
+ read -r _ hash _
+ [[ !* == \!* ]]
+ clear
terminals database is inaccessible
```

`clear` needs a terminfo entry. The `nano` image ships no terminfo database, so
`clear` wrote that message and exited non-zero — and `set -e` then ended the
script before it printed its first line. The unit's `Restart=on-failure` with
`StartLimitIntervalSec=0` restarted it 2.3 seconds later, forever.

`litric-shell.service` has `Requires=litric-first-run.service`, so the desktop
waited behind a script that could never finish.

## Fix

1. `first-run.sh` defines `clear_screen()` as `printf '\033[2J\033[H'` and uses
   it. An ANSI erase needs nothing installed, which is the point for a script
   that runs before anything else exists.
2. `litric-first-run.service` sets `Environment=TERM=linux`: the framebuffer
   console's entry is `linux`, and a program that asks what the terminal is
   should get an answer even where there is no terminfo database.
3. `litric.first_run=skip` on the kernel command line provisions the language
   (default `en`, or `th` with `litric.lang=th`) and exits 0, leaving the account
   locked. This is what an installer-managed or automated boot needs, and it is
   the only path that starts the desktop with no console input at all.

## Verification

Booted the rebuilt image through UEFI with `litric.first_run=skip`:

* `litric-first-run` exits 0 — no restart loop in the journal;
* `litric-shell` starts and owns the framebuffer, reporting
  `litric.shell.frames_over_budget=0`;
* the desktop is on screen and input works through the emulated USB tablet and
  keyboard.

The interactive path (no `litric.first_run=skip`) was verified before the fix by
typing the answer on the graphics console: with a language chosen, the script
reaches `passwd` and the shell starts immediately afterwards.
