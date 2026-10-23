# ADR-0023 — Wave 1 reaches system providers through files, sysfs and the project's own socket, and takes no D-Bus dependency

**Date:** 2026-10-11
**Status:** ACCEPTED for Phase 1 (facts, configuration, provider management) — Phase 2 (Bluetooth
and authentication) is decided in principle and blocked on the helper decisions named below.
**Implemented 2026-10-11:** the facts half of decisions 2 and 3, and one action.
`litric-sysd::platform` reads network, audio, power, Bluetooth and session facts
from files only (`src/litric-sysd/src/platform/`), the five capabilities exist in
`litric-caps` with their scope grammar, and the three places where this ADR was
wider than the kernel's file surface are recorded in the notes below rather than
left to be discovered.

**Added 2026-10-12, decision 2 carried through to the write half:** the four
remaining providers now have actions, all of them through a base tool that owns
its interface rather than through a protocol client of ours — `wpctl` (volume and
mute), `ip`/`networkctl`/`iwd`'s credential file (link state, re-read, join and
forget a network), `bluetoothctl` (radio, scan, pair, connect, forget) and
`unix_chkpwd` (verify a password, on stdin, with the exit status as the answer).
They live in `src/litric-sysd/src/providers.rs`, behind the same capability
broker as the power transition and with every argument validated before a plan
exists. `bluez` and `bluez-utils` join the desktop profile in the same change
(with a budget row and a measurement), because a Bluetooth client without a
daemon is the stub this ADR refused to ship. What decision 1 deferred was a
D-Bus client for *runtime control of `iwd`/BlueZ*; the actions that are
implementable without one are implemented, and the ones that are not — Wi-Fi
scanning, a passkey pairing agent — are still named as gaps rather than
approximated.

**Added later, decision 3 made real:** a power transition is the
first action that executes. `litric-sysd/src/action.rs` runs
`systemctl suspend|reboot|poweroff` through a `Runner` seam, `state.rs` checks
it against the broker with `Broker::check_with_consent` — so a destructive scope
without consent from the user *in that moment* is refused by the broker rather
than by convention — the shell draws the session rows and the confirmation, and
the protocol carries `POWER <verb> CONFIRM`. The network drop-in writer, the
`iwd` credential file, the mixer helper and the authentication helper are still
unimplemented, and each note's §8 says so.

**One correction this work forced on decision 3:** the daemon cannot take the
caller's identity from the request. `RESOLVE` names an application and the
daemon looks its grants up, which is the confused-deputy defence ADR-0016
states — but a power transition has no application to name, so `Sysd` holds the
*one* local client identity it was configured with (`LOCAL_CLIENT` in
`src/litric-sysd/src/state.rs`) and grants it the actions that exist. The socket
is `0600`; a peer-credential check that would let this be per-connection is the
authentication helper's job and is not claimed here.

## The problem this solves

Five capabilities a desktop OS is expected to have — network, Bluetooth, audio,
power management and authentication — have **no provider in this tree at all**.
The Stage A audit recorded it (`docs/audit/2026-10-11-beta-1.0-stage-a-audit.md`
§5): no crate takes a D-Bus dependency, no crate talks to a service, and there is
no authentication subsystem of any kind. `SYSTEM-SCOPE.md` carries the rows
(`S`/`T`) and the image ships providers nobody currently asks anything of:

| Area | What the image already ships | Where it is declared |
|---|---|---|
| Network | `iwd`, `nftables`, `systemd-resolvconf`, and a wired `systemd-networkd` configuration | `image/mkosi.profiles/core.conf`, `image/mkosi.extra/usr/lib/systemd/network/20-wired.network` |
| Audio | `pipewire`, `pipewire-pulse`, `pipewire-alsa`, `pipewire-jack`, `wireplumber` | `image/mkosi.profiles/desktop.conf` |
| Power | `systemd-logind` as part of `systemd`; battery and thermal state in sysfs | `image/mkosi.profiles/core.conf` (systemd), `/sys/class/power_supply` |
| Bluetooth | **nothing.** `bluez` is not in any profile | — |
| Authentication | `openssh` (which pulls `libpam`), no service of ours that uses it | `image/mkosi.profiles/core.conf` |

The prompt set that drove this work assumes each of these is "connect Settings to
the existing service", and the audit refused that framing for a reason worth
repeating: every one of these services is reachable only over D-Bus, and a
D-Bus client is a dependency — which `src/Cargo.toml` forbids for the core and
runtime crates in prose and `ci/lint-deps.py` now enforces.

## The decisions

### 1. No D-Bus dependency in this wave. The three options were weighed, not skipped.

| Option | What it costs | Why it was not chosen now |
|---|---|---|
| Write a minimal D-Bus client in-tree (auth, marshalling, signals) | ~2,000 lines of protocol code inside the privilege boundary, and the most attractive target in the system | It is a real option and it is the one to take if runtime control of `iwd`/`BlueZ` becomes a requirement. It is not Phase 1 work: it is a subsystem with its own threat model, and doing it speculatively is what this ADR exists to prevent |
| Take a third-party crate (`zbus`) | one line in a manifest; `ci/dep-policy-allow.txt` is the mechanism | It breaks the property the core and runtime crates are valued for — compile and test anywhere, offline — and it puts a large transitive tree inside a process that holds system privilege. That is an architecture decision, not a detail, and it is not needed for Phase 1 |
| Use the interfaces that are already files | nothing | **Chosen.** See below |

### 2. Phase 1 works on interfaces that need no client library

Every Phase 1 capability is reachable through a file, a sysfs attribute, or a base
tool that ships in the image and speaks to its own daemon:

- **Facts** (what the machine *is*): `/sys/class/net/*`, `/proc/net/dev`,
  `/proc/net/route`, `/proc/net/if_inet6`, `/sys/class/rfkill/*`,
  `/sys/class/bluetooth/*`, `/proc/asound/cards`, `/proc/asound/pcm`,
  `/sys/class/power_supply/*`, `/sys/class/thermal/*`,
  `/sys/class/backlight/*`, `/run/systemd/sessions/*`. All read-only, all plain
  text, all parsed by `litric-sysd::platform` with fixture tests, none of it
  requiring privilege.

  **One documented exception, added 2026-10-11.** IPv4 *addresses* are published
  only through netlink, and a netlink client needs `unsafe`, which the daemon
  forbids; they are read by running `ip -o -4 addr show` with a fixed argument
  vector and no shell. `iproute2` is a hard dependency of `systemd` on this
  distribution, so this is a base tool in the same sense as `systemctl` in the
  power bullet below. When it is missing the reply says `net.addr.v4=none`
  instead of reporting an empty address list as if the machine had none.
- **Network configuration**: `systemd-networkd` reads drop-ins, so the action is
  *write a file under `/run/systemd/network/` and ask networkd to re-read it*.
  Wi-Fi credentials are `iwd`'s own files under `/var/lib/iwd/`, so joining a
  **known** network is a file write plus a reload, with no scanning. Interactive
  scanning is Phase 2 and is exactly the work that needs the D-Bus decision.
- **Audio**: PipeWire's volume and default-sink state is ALSA mixer state, which
  `alsa-utils` writes through the kernel's mixer interface; the graph itself stays
  the provider's business.
  **Correction, 2026-10-11:** the kernel mixer interface is an `ioctl` on
  `/dev/snd/controlC<card>`, not a file, so there is no `audio.mixer` fact and no
  volume write in this wave. The reply carries `audio.control=none` and
  `docs/design/wave1/02-audio.md` §2 records the helper decision this needs, the
  same shape as the authentication helper below.
- **Power**: `systemctl suspend`/`poweroff` are base tools talking to PID 1 over
  systemd's own socket. The shell executes them the way a person would, rather
  than reimplementing the management protocol.
- **Authentication**: facts only in Phase 1 — which sessions exist, from
  `/run/systemd/sessions/`. No credential verification is implemented, because a
  lock screen that accepts anything is worse than no lock screen
  (`SYSTEM-SCOPE.md` Part B6 note of 2026-08-29).

### 3. Actions go through a capability request, never through ambient authority

The shell is the only component that holds these capabilities in Phase 1, and it
holds them by grant, checked by the broker the way every other capability is
(`docs/SEC-01-capabilities.md`). The vocabulary additions this ADR requires are
`NetConfigure`, `BluetoothAdmin`, `AudioSet`, `PowerControl` and `AuthSession`,
each with the prompt, risk class and scope the vocabulary file already defines for
its neighbours.

### 4. Bluetooth and authentication are not stubbed, they are scheduled

**As implemented, 2026-10-11:** the *facts* of both are real and tested. A
Bluetooth verdict of `bt.state=no-daemon` is what an image with a radio and no
`bluetoothd` reports, which is this image; `bt.state=ready` is not reachable
until `bluez` ships. Authentication reports `auth.available=none` and
`auth.lock=refused`, and `Verifier::can_lock()` is `false` for every value this
build can produce — asserted by a test, so a lock screen that appears to work
while nothing can verify a password is a test failure rather than a discovery.

**Answered 2026-10-12, both.** Bluetooth got its `bluez` package and a budget
row, and reaches the daemon through `bluetoothctl` — BlueZ's own client — rather
than through a D-Bus client of ours; the member of decision 1 that was deferred
is a *client library in this tree*, and it is still deferred. Authentication got
a verifier that is not ours: `/usr/bin/unix_chkpwd`, the helper `pam_unix`
already executes, setuid root, password on standard input. That answer is better
than the helper this ADR sketched, on the same reasoning as every other row here:
the tool that owns the interface, spoken to the way it is meant to be spoken to,
and no `unsafe`, no dependency and no second password store. Neither is faked: a
Settings pane that shows a switch it cannot honour is the defect class this
project has already recorded four times, and no such pane has been drawn yet —
which is now the largest gap left in this wave (`docs/design/wave1/00-network.md`
§8 ticket 4).

## What it does not do

- It does not implement all of the write half. **Superseded in part on
  2026-10-12**: the `iwd` credential file, the audio control and the
  authentication path are implemented, and each note's §8 says what is left.
  Still unimplemented: the `networkd` drop-in writer, Wi-Fi scanning, a capture
  level, a default-sink switch, a passkey pairing agent, and every display, input
  and time-and-locale action.
- It does not let the broker's consent rule be bypassed by the protocol. A
  request without the confirmation word is refused before any tool is run, which
  is asserted at the parser, at the broker and at the daemon, and probed in a
  booted image by the self test (`litric.sysd.refuses_unconfirmed_poweroff`).
- It does not add, remove or pin a package: `bluez` and `alsa-utils` are named as
  required by a later change. Nothing checks the profile's package budget today —
  `image/mkosi.profiles/desktop.conf` used to claim a budget linter
  (lint-budget.sh, in ci/) that has never existed, and that claim is corrected in
  the profile itself rather than left to be believed.
- It does not implement any privileged action. This is the decision that has to
  exist before they are written, which is what the audit asked for.
- It does not decide the D-Bus question. It records the three options, the
  criteria, and the fact that Phase 1 does not need an answer.

## Evidence

The provider list above is not from memory: `image/mkosi.profiles/core.conf` and
`image/mkosi.profiles/desktop.conf` were read, and `bluez`, `polkit`, `upower`
and `pam` appear in neither of them. The fact sources are the ones a Linux kernel
publishes; each parser added under `litric-sysd::platform` is tested against
fixture text so the claim "this is what `/proc/asound/cards` looks like" is a test,
not a recollection.

## Consequences

- Settings grows a provider layer that is honest about what it cannot do yet: the
  Bluetooth row says "not in this image", not "off".
- The five design notes under `docs/design/wave1/` carry the per-area threat
  models, because one shared ADR cannot state the abuse cases for a pairing
  dialog and a lock screen in the same paragraph.
- Phase 2 work starts with a D-Bus decision, a helper design for authentication,
  and a memory-budget row for BlueZ — in that order.
