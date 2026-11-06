# 22 — Unified application compatibility

## Requirement

One desktop, three application ecosystems. Litric, Windows and Linux
applications run directly in the Litric desktop; the user does not choose a
runtime, switch desktops, or boot a second OS to use an installed application.
Full Windows and Linux environments remain available as fallbacks, and
LitricOS remains the primary OS, security boundary and resource owner in every
case. [ADR-0020](../adr/0020-unified-application-compatibility.md) is the
decision record; this document is the implementable contract.

## Runtime selection

The Application Manager is a pure decision function, executed by daemons under
the capability broker's supervision:

```text
                    ┌────────────────────┐
 application file → │ classify + decide  │ → runtime + isolation profile
                    └────────────────────┘      + requested capabilities
                                                + warnings (never suppressed)
```

| Input | Decision | Execution | Status |
|---|---|---|---|
| Litric manifest (`.ilpkg`) | native runtime | native process, sandboxed | planning exists (`litric-manifest`) |
| Windows `.exe` / `.msi` | Windows compatibility runtime | per-app bottle | **built, tested** (`litric-winbridge`) |
| Linux executable | native — sandbox + portals | normal app process | profile resolver/executor **built, tested** (`litric-sysd/src/compat.rs`) |
| Flatpak application | Flatpak runtime | `flatpak run <id>` | profile resolver/executor **built, tested**; real Flatpak is hardware/environment-gated |
| Steam title | Steam runtime, Proton chosen by Steam | `steam -applaunch <id>` | profile resolver/executor **built, tested** with a fake launcher; real Steam/Proton is hardware/environment-gated |
| Needs a foreign kernel | full environment | KVM guest (ADR-0015) | `litric-env` policy **built, tested** |

Escalation to a full VM requires recorded evidence (driver, anti-cheat,
undocumented interface) and states the resource cost before running.

## Invariants

1. The desktop is one compositor surface space; runtime provenance never
   changes how a window is presented (ADR-0009).
2. No compatibility runtime receives ambient authority: no home mount, no
   network, no device access by default (rows 639/640).
3. Cross-environment data access is the ADR-0020 matrix: native apps get
   user-granted scopes; bottles get application-scoped directories; full
   environments get user-approved directories only; **nothing** gets platform
   state, secrets or boot configuration.
4. Removing every compatibility component must leave a working OS
   (master-plan/02 dependency direction).
5. Windows binaries are untrusted regardless of publisher; signature presence
   is never reported as verification.

## Implemented launch seam

`litric-sysd` now owns the resolver and executor in `src/litric-sysd/src/compat.rs`.
`LAUNCHENV` (the shell's Play path) is resolved as a native Linux profile; the
new `LAUNCHPROFILE <linux|flatpak|steam> ...` request names a profile explicitly.
The resolver validates the target, the launcher executable and the bounded
argument vector before handing the plan to the existing `execve` path. Unit
tests run real fixture scripts for all three profiles; the QEMU guest proof
clicks Play and observes a real `litric.client` window and pixels.

This is a route through the vendor's launcher, not a compatibility verdict.
Flatpak and Steam were absent from the build host, so their executor tests use
fake launcher scripts. No game was launched, and no anti-cheat is bypassed.

## Functional work

1. `litric-compatd`: execute the tested install plans — bottle lifecycle,
   component staging, registration (rows 626–629).
2. Linux-app sandbox profile catalogue wired to Landlock/seccomp/namespaces
   (rows 503–508) and XDG portals bound to the capability broker (row 602).
3. VM guest surfaces as windows (row 880) — the step that completes
   "native-looking windows" for full-environment software.
4. Application Manager UI: selection is automatic and *shown* ("this Windows
   program will run in an isolated bottle; it will not see your home
   folder").
5. Per-title compatibility honesty for games: RISK-001 wording, no implied
   anti-cheat support.

## Failure tests

Bottle install interrupted at every step (partial bottle must reset, not
half-run); malformed PE/MSI inputs (fuzz — the planner accepts arbitrary
bytes); capability revocation mid-run; bottle disk exhaustion; guest resource
exhaustion against the host reservation; guest crash while on screen (falls
back to host — already tested in `litric-env`); reset/delete confirming the
exact scoped data lost.
