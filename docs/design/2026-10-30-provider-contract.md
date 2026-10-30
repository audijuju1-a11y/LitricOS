# C1 — the provider contract between the shell and the system daemon

Date: 2026-10-30
Batch: C1 (master plan Stage C)
Status: landed as a **contract layer**; the real iwd/BlueZ integration is still
`INTEGRATION` work for a builder.

## The gap

The shell already reached network, Bluetooth and audio through
`litric-sysd`'s Unix socket. There was no named contract behind that path:
`Sysd` called `providers::perform`, `providers::audio_level` and
`providers::bt_powered` directly. A test could replace the process runner, but
it could not answer a provider request without also modelling the tool that the
request would spawn. There was no fake for the three families C1 names.

The risk C1 exists to prevent is architectural: someone connects the shell to
`zbus`/`libdbus` "just for the panel", putting a protocol client and its parser
inside the process that draws the lock screen. ADR-0023 decision 1 rejects that
move. The shell must keep asking the daemon.

Separately, §65 already made the secret-bearing action types render through
redacting `Debug`, and the socket process test proved a sentinel Wi-Fi
passphrase never reaches the reply, journal or audit. C1 does not redo that; it
adds a provider-call record that uses the same redaction and tests it at the new
seam.

## What changed

| File | Change |
|---|---|
| `src/litric-sysd/src/provider_contract.rs` | `Provider` trait, `ToolProvider`, `FakeProvider`, `AudioLevel` and contract tests |
| `src/litric-sysd/src/state.rs` | `Sysd` holds `Box<dyn Provider>`; `with_provider`; all audio/network/Bluetooth/auth dispatch and readbacks go through it |
| `src/litric-sysd/src/lib.rs` | the module is part of the public library |
| `ci/lint-provider-boundary.py` | rejects a D-Bus dependency or import in `src/litric-shell`, and requires the trait/fake |
| `ci/verify.sh` | `[3m]` runs the checker and its self-test |

`ToolProvider` delegates to the existing `providers` functions, so the exact
argument vectors, capability checks, timeouts, `applied=` lines and secret
handling are unchanged. `FakeProvider` is the contract test double: it records
the redacted action rendering and returns scripted audio, Bluetooth and action
results without invoking `Runner`. `Sysd::with_provider` is the injection seam.

No new dependency, no `unsafe`, and no change to the IPC schema or an error
code. The shell still sends its existing LFP/1-style request lines; it simply
has no reason or route to own a D-Bus client.

## Precedence

| Layer | Rule |
|---|---|
| ADR-0023 decision 1 | no D-Bus client library in this tree's core/runtime path; the shell must not add one either |
| C1 `Provider` | the daemon's only dispatch seam for audio, network, Bluetooth and single-plan authentication |
| `providers` plans | unchanged: validated arguments, base tools, bounded timeouts, secret-free replies |
| `litric-shell` | keeps only the socket request/response path; the `[3m]` gate rejects a D-Bus dependency or import |
| tests | `FakeProvider` may answer without a tool; production always constructs `ToolProvider` |

## Failure and degradation

* A provider error is returned as the existing error reply (`audio-unavailable`,
  `bt-unavailable`, `action-failed`); it is never converted into `applied=true`.
* A failed audio or Bluetooth readback leaves the previous measurement in place.
  Nothing invents zero or "off".
* A fake provider is a test implementation only. `Sysd::new` constructs
  `ToolProvider`; there is no request field that selects a provider.
* The boundary checker is static. It cannot prove what a built binary links; it
  proves that the shell manifest and sources contain no D-Bus dependency/import
  and that the contract exists. The dependency policy remains the separate
  `[3h]` gate.

## Evidence

`python3 ci/lint-provider-boundary.py` reads the real shell manifest and source,
and requires `src/litric-sysd/src/provider_contract.rs`. Its self-test does three
things in a temporary tree: a clean shell passes, a planted `zbus` dependency is
rejected, and a planted `use zbus::Connection` is rejected. Before C1 no such
checker existed, so the planted dependency and import would have passed.

The contract tests are in `provider_contract.rs` and `state.rs`:

* the fake answers audio and Bluetooth without running anything;
* the fake covers audio, network and Bluetooth outcomes;
* a provider call record with a sentinel password contains `<redacted>`, not the
  sentinel;
* `ToolProvider` still runs the audited `wpctl` argument vector;
* `Sysd` with a fake serves `AUDIO LEVEL`, `BT STATUS` and a `NET RECONFIGURE`
  action without a tool.

## Limits

* No sound server, Bluetooth radio, iwd or real network was exercised. This is
  contract evidence, not integration evidence.
* Only a caller holding the `Sysd` value can inject the fake; no request field
  always uses `ToolProvider`.
* This does not implement Wi-Fi scanning, a passkey agent, a default-sink switch,
  or any capability beyond the actions already present.
* `SYSTEM-SCOPE` rows moved: **none**. C1 creates no new machine capability; the
  existing audio, network and Bluetooth rows remain `T`, and the provider work
  already recorded as `S`/`P` is unchanged.
