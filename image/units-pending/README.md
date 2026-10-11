# Units waiting for a daemon that exists

These unit files are **not installed into the image**. They were moved here from
`image/mkosi.extra/usr/lib/systemd/system/` on 2026-10-11 because each service
among them starts a binary that no crate in `src/` produces (and one slice here
belongs to a service that does). A unit that names a
missing binary is a promise the image cannot keep: `systemd` reports it as a
failed unit at boot, or — worse, in the socket-activated case — fails only when
a user first asks for the service, which looks like a bug in the thing they were
using.

They are kept here rather than deleted because this working copy has **no
version control history** to recover them from (see
[docs/audit/2026-10-11-beta-1.0-stage-a-audit.md](../../docs/audit/2026-10-11-beta-1.0-stage-a-audit.md) §1).
The sandbox profiles in these files are real design work and must not be lost.

| Unit | `ExecStart` | Binary target in `src/`? | What it is for |
|---|---|---|---|
| `litric-capd.service` / `.socket` | `/usr/bin/litric-capd` | **none** | the capability broker as a separate process. [ADR-0016](../../docs/adr/0016-system-daemon.md) decided the broker lives **inside** `litric-sysd`, and says these units "should be removed rather than left to look like they work" |
| `litric-defender.service` | `/usr/bin/litric-defender` | **none** (`litric-defender` is a library) | on-access scanning daemon. ADR-0022 Phase 0 is the policy core only; the fanotify transport is later work |
| `litric-files@.service` / `.socket` | `/usr/bin/litric-filesd` | **none** (`litric-files` is a library) | the Files daemon of [ADR-0019](../../docs/adr/0019-litric-files.md) |
| `litric-defender.slice` | — (a slice, not a service) | n/a | the resource slice that gives row 10 of the memory ledger its `MemoryMax=64M`. It travels with `litric-defender.service`: a slice sets limits on the units that join it, and with the service parked nothing joins it, so the limits would govern nothing while still reading as though they were in force. `ci/check-memory-budget.py` says exactly that when it finds a slice no unit joins. Until the daemon exists, `docs/PERF-budget.md` row 10 records 60 MB as a target rather than a configured cap |

## The one that was worse than dead

`litric-files.socket` was **enabled** by `image/mkosi.postinst.chroot`. The socket
unit itself starts fine — a socket needs no binary — so the boot looks healthy
with zero failed units, and the failure appears only when the shell connects and
`systemd` cannot spawn `litric-filesd`. That enable line has been removed with a
comment saying so.

## What must exist before a unit returns here

1. A crate that builds the named binary (`[[bin]]` in its manifest, or a
   `src/main.rs`).
2. `ci/lint-unit-binaries.py` passing — it walks every installed unit, resolves
   each `ExecStart` absolute path against the binaries the workspace can produce
   and the files the image ships, and fails the gate on a mismatch. This check
   exists so that the state described above cannot come back silently.
3. Where the unit configures a security boundary, a review of the sandbox
   section against what the new binary actually needs.
