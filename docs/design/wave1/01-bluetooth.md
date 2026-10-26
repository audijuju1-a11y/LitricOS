# Wave 1 — Bluetooth

**Status:** **the two decisions below are answered, and the actions are
implemented as of 2026-10-12.** The *facts* are in
`src/litric-sysd/src/platform/bluetooth.rs` (adapter presence, the adapter's
address, `rfkill` soft and hard blocks, whether `bluetoothd` is installed at
all), fixture-tested; the radio's power, discovery, pairing, connecting and
forgetting are in `src/litric-sysd/src/providers.rs`, run through
`bluetoothctl`. `bluez` and `bluez-utils` are in the desktop profile as of the
same change. Since 2026-10-26 the radio's *power* is a measurement rather than a
memory: `BT STATUS` asks `bluetoothctl`, the daemon caches the answer and carries
it in `FACTS` as `bt.powered`, and the quick-settings panel's Bluetooth row shows
that rather than the switch the user last flicked. **Not verified on anything but
QEMU, which has no radio at all** — §6 says exactly what that does and does not
establish.
**Scope:** adapter presence and state, pairing, connecting a known device, audio
routing of a Bluetooth headset.

## 1. What exists in the tree today

| Thing | Where | State |
|---|---|---|
| `bluez` (daemon) and `bluez-utils` (`bluetoothctl`) | `image/mkosi.profiles/desktop.conf` | **added 2026-10-12**, with `bluetooth.service` enabled in `image/mkosi.postinst.chroot` |
| Budget row | `docs/PERF-budget.md` | added 2026-10-12, with the idle-memory measurement that goes with it recorded in the GATE 01 entry of that date |
| Adapter facts | `/sys/class/bluetooth/*`, `/sys/class/rfkill/*`, `/usr/bin/bluetoothd` | read by `platform::bluetooth` |
| Actions | `bluetoothctl` through `providers::BtAction` | implemented 2026-10-12: `BT POWER`, `BT SCAN`, `BT PAIR`, `BT CONNECT`, `BT FORGET` |
| Radio power, as a fact | `providers::bt_powered`, cached in `Sysd` | **added 2026-10-26**: `BT STATUS` runs `bluetoothctl show` and parses `Powered:`, because the power state is in no file under `/sys`. A question, so no capability; cached, so the twice-a-second `FACTS` poll spawns nothing; refreshed after every applied `BT POWER`. Read back rather than assumed for the same reason the volume is |
| The switch in quick settings | `src/litric-shell/src/views.rs`, `src/litric-shell/src/main.rs` | **added 2026-10-26**, with the three conditions the row needs: a radio, nothing blocking it and `bluetoothd` installed. Where any of those is missing the row says which one, and there is no switch to press |
| A Bluetooth device in the test image | — | **absent**, and not fixable in this VM: QEMU emulates no radio, so every action here is unverified against real hardware |

## 2. The two decisions this note exists to force

1. **Is Bluetooth part of the product?** It costs a daemon (`bluetoothd`),
   a memory-budget row and a pairing UI. **Answered 2026-10-12: yes, in the
   desktop profile.** The daemon is one more package and is enabled; the budget
   row plus its measurement landed in the same change, because a profile whose
   320 MB ledger quietly stops being true is the defect this note warned about.
2. **How is D-Bus reached?** BlueZ is D-Bus-only, and this is still true.
   **Answered 2026-10-12, for the actions in scope: not by a D-Bus client of
   ours.** The daemon runs `/usr/bin/bluetoothctl` — BlueZ's own client, from
   `bluez-utils` — the way it runs `systemctl` for power, `wpctl` for sound and
   `iwctl` for Wi-Fi. The alternative was a protocol implementation inside the
   privileged daemon (ADR-0023 decision 1's first option) for one provider out of
   four, which would also have made the other three inconsistent with it.
   **What that costs, stated plainly:** everything `bluetoothctl` can express is
   available and nothing else is. A pairing that needs a passkey shown and
   confirmed cannot be completed from a non-interactive invocation — it fails
   rather than being silently accepted, which is the fail-closed direction, but
   it is a real limitation and it is why §7 ticket 7 asks for a pairing agent
   before any pairing UI is drawn.

## 3. If the answer is yes: the shape

- **Facts without BlueZ**: `/sys/class/bluetooth/*` (adapters), `/sys/class/rfkill/*`
  (`type=bluetooth`, soft/hard block) and the presence of `usr/bin/bluetoothd`.
  **Implemented**, and the shapes came out as three states rather than one:
  `bt.state=no-adapter` (this machine has no radio), `bt.state=blocked` (a radio
  exists and something stops it), `bt.state=no-daemon` (a radio is present and
  unblocked, but this image has nothing to pair with), `bt.state=ready`. Without
  the third word a Settings row would read "available" on an image where pairing
  cannot work. `hard` and `soft` are reported separately for the same reason: a
  firmware block is not a software switch the user can flip. `/proc/net/bluetooth`
  is deliberately *not* read: it is a socket table, not adapter state, and
  nothing in this wave has a use for it.
- **Actions**: the capability is `BluetoothAdmin`, scoped as `bt://` for
  radio-level changes (power, scanning — a grant over every Bluetooth resource on
  the machine, which is what "turn the radio on" means) and `bt://<bdaddr>` for a
  device. Containment is segment-aware, so a grant for one device is not a grant
  for another and a device grant is not a radio grant; both directions are
  asserted. Each action becomes a `bluetoothctl` argument vector with a validated
  address (`aabbccddeeff` in colon form, and never the all-zero address a caller
  that filled in nothing would send).

  **A pairing reply says `requested`, not `paired`.** The tool's exit status is
  the tool's answer, and the device's state is what the next facts read says; a
  UI that printed "paired" from the exit status alone would be inventing a fact
  at the one moment a user is watching for one. `Plan::notes` carries
  `state=from-next-read` for exactly that reason, and a test asserts it is there.
- **Audio**: a paired headset is a PipeWire sink, so the audio path is
  [02-audio.md](02-audio.md), not this one. The two notes must agree on who
  selects the sink — the answer here: Bluetooth sets it as *the* preferred sink
  when the user pairs a headset, and the audio layer is what does the switching.

## 4. Threat model (for the shape above)

| # | Abuse case | Mitigation |
|---|---|---|
| B1 | An app pairs with a device the user did not choose, in a pairing window opened by someone else | pairing needs `BluetoothAdmin` scoped to `bt://<bdaddr>`, which no unprivileged app holds, and every request is audited. **Not** live-consented, and the reason is recorded rather than glossed: `BluetoothAdmin` is not a live-consent capability in the vocabulary, so the check is a grant rather than a prompt. What stops a silent pairing today is that a passkey pairing cannot complete without an agent (decision 2), and what will stop it once one exists is ticket 7 below: the prompt is the agent's, and it must be the shell's |
| B2 | A paired device silently becomes an input device (HID) and types on the user's behalf | HID profiles are refused by policy in Phase 1; a device that offers HID is listed with that fact, not paired silently |
| B3 | Audio is routed to an attacker's headset so the user hears silence and the attacker hears the call | sink selection is user-visible in the status area (existing audio indicator) and a device-initiated sink switch raises a notification |
| B4 | The pairing agent accepts a passkey without showing it | passkey confirmation is rendered by the shell, never auto-accepted; an unanswered prompt expires and refuses |
| B5 | A blocked radio is unblocked without the user knowing | `rfkill` soft-block changes go through `BluetoothAdmin`, are audited, and the indicator reflects hard-block (firmware) separately from soft-block (policy) |
| B6 | `bluetoothd` runs with more privilege than it needs | unit sandbox is written by us, not inherited: `CapabilityBoundingSet`, `PrivateNetwork` off (it needs the bus), `ProtectSystem=strict`, a memory slice — and a budget row before it lands |

## 5. What is verifiable today

- That the image does **not** ship BlueZ and has no budget row for it: one `grep`.
- The `/sys/class/bluetooth` and `rfkill` parsers, on any host, with fixtures.
  **Implemented**: `platform::bluetooth` asserts the four states, that a blocked
  *wlan* radio does not make the Bluetooth verdict read `blocked`, and that
  `hard` wins over `soft` when both are set.
- The `Powered:` parse and the cache, on any host: a recording runner answers
  `bluetoothctl show`, and the tests assert that `FACTS` carries no `bt.powered`
  until something has asked, that a facts poll runs no tool, and that an applied
  `BT POWER` is followed by a readback rather than by the requested value.

## 6. NOT verified, and what it would take

| Claim | Needs |
|---|---|
| a paired headset plays audio | a real adapter (QEMU has none by default; USB passthrough or `-device usb-bt-dongle` is not emulation of the host radio) |
| pairing prompts cannot be bypassed | the implementation, then an adversarial test against it |
| `bluetoothd` fits the idle budget | BlueZ in a booted image with the memory ledger updated first — **done 2026-10-12**, and the guest's own number is in the GATE 01 entry of that date. What it does *not* include is a radio: no adapter, no traffic, no scanning, which is a floor and not a working figure |
| the radio actions do what they say | a machine with an adapter. **NOT verified**, and the booted image says so out loud: `bt.state=no-adapter` and the action's own `action-failed` |
| the quick-settings switch shows a real radio's power | a machine with an adapter; in this VM `BT STATUS` answers `ERR bt-unavailable` because there is no `bluetoothctl` to run, which is the honest path and is what a host test records |

## 7. Tickets this note implies

1. Decide (product): is Bluetooth in Beta 1.0? If no, the Settings row says so and
   this note is the record of the decision rather than a plan.
2. Decide (architecture): the D-Bus question from ADR-0023.
3. ~~Facts: `/sys/class/bluetooth` + `rfkill` readers, fixture-tested — useful now.~~
   **Done** — `src/litric-sysd/src/platform/bluetooth.rs`.
4. ~~Add `bluez` to a profile and a `docs/PERF-budget.md` row, in one change.~~
   **Done 2026-10-12** — desktop profile, `bluetooth.service` enabled in
   `mkosi.postinst.chroot`, budget row and measurement together.
7. A pairing agent the shell owns, so a passkey is shown and confirmed rather
   than the pairing failing. **Not done** — and no pairing UI may be drawn before
   it exists, or the interface would offer a flow that cannot complete.
5. ~~Capability `BluetoothAdmin` with `bt://` scope grammar.~~ **Done** — token
   `device.bluetooth.admin`, scopes `bt://<adapter>` and `bt://<adapter>/<bdaddr>`,
   segment-aware so `bt://hci0` cannot cover `bt://hci0x`.
6. Pairing UI with live consent and the passkey rendered by the shell. Blocked on
   ticket 1 and ticket 4.
