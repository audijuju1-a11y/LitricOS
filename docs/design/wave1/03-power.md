# Wave 1 — power

**Status:** Phase 1 design accepted ([ADR-0023](../adr/0023-wave1-system-providers.md)).
**Facts implemented** in `src/litric-sysd/src/platform/power.rs` (supplies,
thermal zones, cooling devices, backlights), fixture-tested on any host.
**Session actions are implemented, and gated twice**: the quick-settings panel
draws suspend, restart and power off; choosing one opens a confirmation that
names the action; confirming sends `POWER <verb> CONFIRM` to `litric-sysd`,
which checks the capability broker *and* the moment's consent before running
`systemctl <verb>`. Nothing on this machine can end the session on one click.
Idle-inhibit (§4) is still not implemented, and §7 lists what a booted image
would have to show.

## 1. What exists in the tree today

| Thing | Where | State |
|---|---|---|
| Battery percentage and charging flag | `src/litric-shell/src/main.rs` (`read_battery`), `SystemFacts` in `src/litric-shell/src/shell.rs` | implemented, drawn in the status area when a battery exists |
| Battery and thermal sources | `/sys/class/power_supply/*`, `/sys/class/thermal/*` | read by `read_battery` only |
| `power_watts`, `temperature_c` | `SystemFacts` fields | documented as **never populated** |
| Suspend, restart, shutdown | `src/litric-sysd/src/action.rs` (the transition), `src/litric-shell/src/views.rs` (the menu) | implemented 2026-10-11: menu → confirm → `POWER <verb> CONFIRM` → `systemctl <verb>` |
| `logind` | part of `systemd`, `image/mkosi.profiles/core.conf` | shipped, unmanaged |
| Budget | `docs/PERF-budget.md` row 2 | systemd PID 1: `TARGET (not enforced)` |

## 2. The provider, and how it is reached

`systemd-logind` owns sessions and idle policy; PID 1 owns the power transitions.
ADR-0023 decision 2 says the shell executes the base tools that already talk to
them (`systemctl suspend|poweroff|reboot`) rather than reimplementing the
management protocol — the same reasoning as using `write()` to the framebuffer
instead of a compositor: **speak to the thing that owns the interface, in the way
it is meant to be spoken to.**

Facts need no provider at all: sysfs is the kernel's own reporting surface.

**Who runs the tool, added 2026-10-11.** The transition is run by `litric-sysd`
and not by the shell, in `src/litric-sysd/src/action.rs`. Three reasons, in order
of how much they matter:

1. The decision belongs in one place. The daemon owns the capability broker, so
the check and the act are the same transaction there; a shell that asked itself
for permission would be a shell that could grant it.
2. `Runner` is a seam. `SystemRunner` spawns; the tests substitute a recorder and
assert the exact argv without spawning anything. A test suite that had to suspend
the machine to check the argument vector is a test suite nobody runs.
3. The daemon is the process that survives the shell being replaced, which is
what makes an audit entry about a power transition worth having.

The program path is absolute (`/usr/bin/systemctl`). A relative name found
through `PATH` would let anything earlier in `PATH` choose what runs, with the
daemon's privilege, at the moment the user asks to power off — abuse case P7
below. The verb is a value from a closed enumeration and is the only thing a
request contributes to the argv, so no request can turn this into an
arbitrary-command primitive.

## 3. Data surface (facts)

```
power.battery.count <n>
power.supply  name=<str> type=battery status=charging|discharging|full|not-charging|unknown
              capacity=<0-100|-> energy_now_uwh=<n|-> energy_full_uwh=<n|->
              power_now_uw=<n|-> cycles=<n|->
power.ac      name=<str> online=<0|1|unknown>
thermal.zone  name=<str> type=<str|-> temp_mc=<millidegrees|->   (raw, not a rounded claim)
thermal.cooling name=<str> type=<str|-> state=<0-31|->
power.backlight name=<str> brightness=<n|-> max_brightness=<n|->
```

Two of the fields the ledger above used to imply are noteable rather than
implemented:

* **`not-charging` is its own status.** It is what a laptop with a charge
  threshold reports while plugged in; folding it into `full` would be a claim
  the machine did not make.
* **`capacity` can come from energy.** Plenty of batteries publish
  `energy_now`/`energy_full` and no `capacity`. The percentage is then computed,
  and only when `energy_full` is nonzero — dividing by zero full charge is how a
  battery reads 0% at 40%.

The percentage is only ever produced alongside a status by the *consumer*: the
fact is two fields because the number alone cannot tell a charging battery from
a draining one, and the status word is what makes it mean something.

Two rules lifted from the existing shell code and kept: a battery percentage
without a status is not a percentage (a value that is charging and one that is
draining mean opposite things), and a machine without a battery must produce **no
line at all** rather than a zero — the status area's existing behaviour.

## 4. Action surface (capabilities)

| Action | Capability | Note |
|---|---|---|
| Read power/thermal facts | none | machine facts |
| Change a brightness or backlight level | `PowerControl @ backlight://<device>` | reversible, visible |
| Set a power profile (performance/balanced/power-saver) | `PowerControl @ profile://<name>` | Phase 2: needs the provider's interface |
| Suspend, reboot, power off | `PowerControl` with live consent | irreversible for the session; the vocabulary's live-consent flag is what makes a menu item a decision, not a click |
| Inhibit idle/sleep while doing something | `PowerControl @ inhibit://<what>` | an app holding an inhibit is visible in the status area, and the hold expires |

## 5. Threat model

| # | Abuse case | Mitigation |
|---|---|---|
| P1 | An app suspends the machine to hide activity (or to interrupt a long verification) | suspend needs live consent, is audited, and an in-flight inhibit from a *signed* holder is honoured first. **Enforced since 2026-10-11**: the broker denies a destructive scope without consent (`Reason::NeedsLiveConsent`), so this is a decision rather than a habit of whoever draws the dialog |
| P2 | An app holds an idle-inhibit forever so the machine never sleeps (battery drain) | holds expire; the status area shows the holders; a hold from a process that has exited is dropped |
| P3 | An app reads thermal/power state and infers what else runs on the machine (a side channel) | accepted and recorded, not hidden: facts are machine-wide by design, and `docs/RISK-00.md` RISK-009 is the register entry for side channels |
| P4 | A malicious app sets brightness to zero and looks like a dead machine | the write is a capability; brightness is restored on session start; the UI path (a slider with a floor) is not the API |
| P5 | The power-menu entry is triggered by a keystroke the user did not mean | the destructive items require live consent (a prompt that names the action), never a single default-focused click. **Implemented**: the quick-settings rows open a confirmation whose affirmative button says the action, whose cancel comes first, and whose click-away is a cancellation |
| P6 | Facts are read from a user-supplied root | same rule as network N4: the root parameter is a test seam; the daemon passes `/` |
| P7 | Something earlier in `PATH` supplies the tool that performs the transition | the program path is absolute and the argv is fixed; no shell, no string interpolation, and the only variable is a verb from a closed enumeration |
| P8 | A client that is not the shell asks for a transition over the daemon's socket | the socket is `0600`, and the daemon acts for the one local client identity it was configured with (the `LOCAL_CLIENT` constant in `src/litric-sysd/src/state.rs`) rather than for an identity named in a request; a peer-credential check is the authentication helper's job and is not claimed here |
| P9 | A refusal is indistinguishable from a success in the reply | an unconfirmed or ungranted request is `ERR refused <reason>` with a stable reason token, a transition that was authorised but did not run is `ERR action-failed <detail>`, and only a transition that really ran answers `OK … applied=true` |

## 6. What is verifiable today

- Every parser above, with fixture text, on any host. **Implemented**:
  `platform::power` reads a whole laptop fixture (AC, battery, thermal zone,
  cooling device, backlight) and asserts the percentage, the absent attributes
  written as `-`, and the rule that a machine with no battery produces no
  battery line at all.
- The whole action path, on any host, without a `systemctl` and without ending
  anyone's session. `cargo test -p litric-sysd` covers: a request with no
  consent is refused *and runs nothing*; a confirmed request runs exactly
  `["/usr/bin/systemctl", "<verb>"]` once; a missing tool is reported as
  `ERR action-failed` rather than as a success; a revoked grant is refused; and
  every request lands in the capability audit log.
- The shell's half, driven by synthetic pointer input. `cargo test -p litric-shell`
  includes a journey that opens quick settings, clicks *Power off*, asserts that
  no effect came out of that click, cancels, reopens, confirms, and asserts that
  exactly `Effect::Power(PowerOff)` came out.
- That the confirm step is not bypassable from the protocol: `POWER POWEROFF`
  without the `CONFIRM` word is refused, asserted at the parser, at the broker
  and at the daemon.
- That `power_watts` and `temperature_c` are still unpopulated: `grep`, and the
  fact that filling them requires an attribute most machines do not publish
  (`power_now` is optional; the honest answer to "how many watts" on many laptops
  is "the kernel does not say", which is a line we can print).

## 7. NOT verified, and what it would take

| Claim | Needs |
|---|---|
| suspend works | a booted image under QEMU (`systemctl suspend` needs ACPI; QEMU supports it) and a serial log showing the transition and resume |
| the in-guest daemon refuses an unconfirmed power-off | a booted image; the self test asks `POWER POWEROFF` (the unconfirmed form, so the request itself cannot end the boot) and reports `litric.sysd.refuses_unconfirmed_poweroff` |
| the battery indicator is right | a machine with a battery, or a fixture that mirrors a real `power_supply` tree (the parser tests do this; the claim that a *laptop* is right needs a laptop) |
| idle policy holds the budget | a booted image, cgroup reader, and the `TARGET` rows in `docs/PERF-budget.md` turned into configured caps |

## 8. Tickets this note implies

1. ~~`litric-sysd::platform`: `power.supply`, `power.ac`, `thermal.zone` facts +
   fixtures.~~ **Done** — `src/litric-sysd/src/platform/power.rs`, served as part
   of `FACTS`.
   Replacing the shell's own `read_battery` with the shared reader is **not done**:
   the shell still has its own copy, and two readers that can disagree is exactly
   the state this ticket exists to end.
2. ~~Capability `PowerControl`, with live consent on the destructive scopes.~~
   **Done** — `power://system/suspend|reboot|poweroff` are `is_destructive()` and
   [`Capability::requires_live_consent`] consults the scope as well as the kind,
   so dimming a backlight and suspending the machine are not the same grant.
3. ~~Power menu in the shell: suspend / restart / power off, with a confirm step
   that names the action.~~ **Done** — the SESSION section of the
   quick-settings panel, `DialogKind`-style confirmation in
   `src/litric-shell/src/apps.rs` (`DesktopDialog::PowerConfirm`), the effect in
   `src/litric-shell/src/shell.rs`, the request line in
   `PowerCommand::request_line`, and the daemon call in
   `src/litric-shell/src/main.rs`.
4. Idle-inhibit registry with expiry and a status-area indicator. **Not done.**
5. Replacing the shell's own `read_battery` with `platform::power`'s reader, so
   the status area and the facts reply cannot disagree. **Not done** — still the
   one duplicate reader left, and this ticket is now the only one in this note
   that changes code already on screen.
