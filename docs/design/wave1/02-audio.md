# Wave 1 — audio

**Status:** Phase 1 design accepted ([ADR-0023](../adr/0023-wave1-system-providers.md)).
**State and control are both implemented.** `src/litric-sysd/src/platform/audio.rs`
reports cards, PCM devices, sound-server presence and which tool can change a
level, fixture-tested; since 2026-10-12 the default sink's **volume and mute are
settable** through `wpctl` (`src/litric-sysd/src/providers.rs`), which is
WirePlumber's own client. This note was corrected on 2026-10-11 because it
previously claimed a mixer read "through the same file surface as the rest",
which is not true of the ALSA control API — and the 2026-10-12 implementation is
the resolution of exactly that: **not** the control API, but the provider's own
tool.

## 1. What exists in the tree today

| Thing | Where | State |
|---|---|---|
| `pipewire`, `pipewire-pulse`, `pipewire-alsa`, `pipewire-jack`, `wireplumber` | `image/mkosi.profiles/desktop.conf` | shipped, unmanaged |
| Budget | `docs/PERF-budget.md` row 9 | 18 MB, `TARGET (not enforced: no drop-in supplies it)` |
| Audio capability | `Microphone`, `AudioOutput` in `src/litric-caps` | vocabulary exists |
| Kernel interfaces | `/proc/asound/cards`, `/proc/asound/pcm`, `/proc/asound/*/…` | read by nobody |
| A mixer | — | absent: nothing in this tree changes a volume |

## 2. The provider, and how it is reached

PipeWire is the sound server; WirePlumber is its session policy. Both speak the
PipeWire native protocol over `/run/user/<uid>/pipewire-0`, and that protocol is
not a text protocol — implementing it in-tree is a subsystem, not a task, and
taking `libpipewire` as a crate violates the dependency policy
(`ci/lint-deps.py`, `src/Cargo.toml`).

ADR-0023 decision 2 therefore splits the work by what the kernel already exposes:

- **Volume, mute, per-card mixer state**: the kernel's mixer interface (ALSA
  control API), which `alsa-utils` drives with `alsactl`/`amixer`. `alsa-utils`
  is **not in any profile today** — a gap this note records; PipeWire's ALSA
  compatibility is a client, not a mixer editor.
  **Correction, 2026-10-11:** the control API is not a file surface. Reading or
  writing it is an `ioctl` on `/dev/snd/controlC<card>`, which needs `unsafe`
  (forbidden in the daemon) or `libasound` (a dependency this tree reviews).
  **Resolved, 2026-10-12:** neither is used, and `alsa-utils` is not added. The
  sound server has a client of its own — `wpctl`, shipped by `wireplumber`, which
  is already in the profile — and ADR-0023 decision 2 asks for precisely that:
  the tool that owns the interface rather than a second implementation of its
  protocol. `providers::AudioAction` runs `/usr/bin/wpctl set-volume --limit 1.0
  @DEFAULT_AUDIO_SINK@ <0.00-1.00>` or `set-mute ... 0|1`, with a fixed argv and
  a percentage that is clamped in code. `--limit 1.0` is not optional: without
  it WirePlumber amplifies past 100%.
  What is still **not** reachable this way is the **capture** level, which
  `wpctl` addresses by node id rather than by name, and a **default-sink
  switch**, which is policy. Both are §8 work; the `capture://` scope stays in
  the vocabulary for the first of them.
- **Device selection and the "who is playing" view**: PipeWire's own state, read
  without a client — `/run/user/<uid>/pipewire-0` presence, and WirePlumber's
  device set from `/proc/asound/*` plus sysfs. A *switch* of the default sink is
  PipeWire policy and is **Phase 2**, blocked on the same D-Bus/native-protocol
  decision as Bluetooth. Phase 1 shows the state; it does not pretend to own it.

## 3. Data surface (facts)

```
audio.card.count <n>
audio.card        index=<n> id=<str> name=<str> driver=<str>
audio.pcm         card=<n> device=<n> name=<str> playback=<0|1> capture=<0|1>
audio.state       server=running|none|unknown socket=<path|->
audio.control     wpctl|none
audio.level       volume=<0-100> muted=<0|1>
```

`audio.level` is the one fact here that is not read from a file: it is the answer
to `AUDIO LEVEL`, which runs `wpctl get-volume` and parses it. It is *cached*,
because `FACTS` is polled twice a second and a poll must not spawn a process; the
daemon refreshes the cache when it is asked directly and after every audio action
it applies. It is absent until something has been measured — `volume=0` over a
machine nobody has asked is a mute machine this daemon invented — and a tool that
cannot be run is `ERR audio-unavailable` with the reason, never a zero. Since
2026-10-26 the shell asks on the one occasion somebody is about to look: opening
quick settings sends `AUDIO LEVEL` (and `BT STATUS`), so the panel shows a
measured level from the first open instead of only after a change. Closing the
panel asks nothing.

`audio.state` is *presence*: whether `/run/user/<uid>/pipewire-0` exists. The
socket is never opened, so "running" means the socket is there and says nothing
about whether it answers. `server=unknown` is the third answer and not a
synonym for `none`: it means `/run/user` was not there to look at, which is a
different claim about the machine.

`audio.control` is written by the reader itself rather than left out, so a caller
cannot mistake presence for control. Since 2026-10-12 it is read from the image
(`usr/bin/wpctl` exists or it does not) rather than being the constant `none` it
was, on the same rule as the credential verifier in `session.rs`: a fact about
what this build can do is looked up, never assumed. `none` still means "no way to
change a level", and the action then answers `action-failed` rather than
reporting a volume that did not move.

## 4. Action surface (capabilities)

| Action | Capability | Note | State |
|---|---|---|---|
| Change output volume or mute | `AudioSet @ audio://default-sink` | the sink this machine plays through unless told otherwise | **implemented** (`AUDIO VOLUME <0-100>`, `AUDIO MUTE ON\|OFF`) |
| Read the output level and mute flag | none — a question | changes nothing, so it needs no capability, the same rule `FACTS` follows | **implemented** (`AUDIO LEVEL` → `audio.level volume= muted=`); after an applied change the level is read back from the sink rather than echoed from the request |
| Change capture (microphone) level | `AudioSet @ capture://<card>` | separate from output, because the abuse case is different | not implemented |
| Open a capture stream | `Microphone` | already exists, already live-consented | unchanged |
| Play audio | `AudioOutput` | already exists | unchanged |
| Change the default sink | **not in Phase 1** | needs the provider's protocol; listed as Phase 2 | not implemented |

`audio://` is a new scheme in `Scope::parse_resource` (2026-10-12), which is why
the scope is spelled as a resource: the vocabulary's resource schemes are the
ones a grant can be narrowed by, and a volume with no scope at all would be a
grant that covers every future audio action at once. A volume change is not
destructive (`Scope::is_destructive` covers `power://system/*` only), so it needs
no live consent, and a slider that opened a dialog would be a worse interface for
the same safety.

## 5. Threat model

| # | Abuse case | Mitigation |
|---|---|---|
| A1 | An app sets the volume to zero and the user cannot hear a notification or an alarm | `AudioSet` is a capability; a *mute* by an unprivileged app is refused; muting by the shell is shown in the status area (the indicator exists) |
| A2 | An app opens the microphone and listens | `Microphone` already requires live consent and an active indicator; the capture level is a separate capability so raising the gain does not imply permission to open a stream |
| A3 | An app unmutes the microphone after the user muted it | the mixer write is refused while a user-set mute is in force; the user's mute is state the system owns, not the app's last write |
| A4 | The mixer writer is given a card/control name with a path separator or a shell metacharacter | the writer resolves the control through the kernel interface it opened, never through a constructed path or a shell; a name that is not in the current control list is a refusal |
| A5 | Audio state is read from a user-controlled socket path and reports another user's session | the socket path is derived from the *shell's own* uid, never from a request field; facts about another uid are not produced |
| A6 | `pipewire`/`wireplumber` are given more privilege than they need | their units are ours and get sandboxing plus the 18 MB budget made real (`TARGET` → configured), which `ci/check-memory-budget.py` already watches |

## 6. What is verifiable today

- `/proc/asound/cards` and `pcm` parsers, against fixtures, on any host.
  **Implemented**: `platform::audio` asserts the five fields of a card, the
  playback/capture split, and the presence verdict for zero, one and many
  candidate sockets.
- That the image ships PipeWire and **not** `alsa-utils`: one `grep`, and the gap
  is stated rather than assumed.
- That no code in this tree changes a volume — `grep` for a mixer write — and
  that the reply says so out loud: `audio.control=none`.

## 7. NOT verified, and what it would take

| Claim | Needs |
|---|---|
| sound comes out | a booted image with a QEMU audio device (`-audiodev`), and a listener that is not us; QEMU's emulated HDA is enough to prove the path, not the quality. **NOT verified.** The booted image of 2026-10-12 has no sound device (`audio.card.count=0`) |
| the volume action reaches the sound server in a booted image | a booted image with a running PipeWire **for the session asking**. **NOT verified**: the in-guest self test runs `AUDIO MUTE ON` and records `unavailable -- no sound server in this VM`, which is the honest answer and is *not* evidence that a laptop's slider works |
| PipeWire reaches the budget | a booted image, cgroup reader, `docs/PERF-budget.md` row 9 turned into a configured cap |
| the default-sink switch | a node-level client, which `wpctl` provides by id; §8 |
| the capture level | the same, addressed by node; §8 |

## 8. Tickets this note implies

1. ~~`alsa-utils` into a profile (with the ISO-budget consequence recorded), or
   an explicit decision that the kernel mixer is driven without it.~~
   **Decided 2026-10-12: neither.** The sound server's own client is used, and no
   package is added.
2. ~~`litric-sysd::platform`: `audio.card`, `audio.pcm`, `audio.state` facts + tests.~~
   **Done** — `src/litric-sysd/src/platform/audio.rs`. `audio.mixer` was dropped for the reason in §2.
3. ~~Capability `AudioSet` with the output/capture split.~~ **Done** — token
   `audio.set`, scope `capture://<card>` for the microphone side.
4. Drop-in for `pipewire`/`wireplumber` that makes row 9 of the memory ledger true.
   **Not done.**
5. ~~Phase 2: the mixer helper (`alsactl` or a small `libasound` helper with the
   same fail-closed shape as the authentication helper)~~. **Superseded**: `wpctl`
   is the helper, and it needs no unsafe, no dependency and no new process of
   ours.
6. Capture level and default-sink switching, by node id, through `wpctl`.
   **Not done** — and the Settings pane that would use any of this is not drawn
   either (see `00-network.md` §8 ticket 4, which is the same gap).
