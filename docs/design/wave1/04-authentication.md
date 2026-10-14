# Wave 1 — authentication, sessions and the lock screen

**Status:** **a password can be verified, and since 2026-10-12 there is a lock
surface that uses it.** `litric-sysd` answers `AUTH VERIFY` through
`/usr/bin/unix_chkpwd` — the helper `pam_unix` itself runs, setuid root, password
on standard input — and `Verifier::can_verify()` is true when that binary is in
the image. `Verifier::can_lock()` follows `can_verify()`, so a machine with the
helper answers `auth.available=unix-chkpwd` and **omits** `auth.lock`, which is
the claim that `AUTH VERIFY` on this machine can open the lock again; an image
without the helper answers `auth.available=none` **and** `auth.lock=refused` in
the same breath. The shell holds the surface: `src/litric-shell/src/lock.rs`
draws it, and `Shell::can_lock` asks both questions before raising it — a
verifier that can check a password *and* a daemon that did not refuse — so a
screen that could not be unlocked is never shown. The facts are in
`src/litric-sysd/src/platform/session.rs`: the `/run/systemd/sessions/*` records,
and whether a verifier exists — decided by looking for it on the filesystem
rather than by a build-time constant.

## 1. What exists in the tree today

| Thing | Where | State |
|---|---|---|
| An authentication subsystem | — | **absent by decision** |
| The reason it is absent | `docs/SYSTEM-SCOPE.md` Part B6 rows 226–239, note of 2026-08-29 | *"There is no lock screen, because there is no authentication subsystem and a lock that accepts anything is worse than none."* |
| `libpam` on the system | pulled by `openssh`, which `image/mkosi.profiles/core.conf` ships | present as a library, used by nothing of ours |
| Session state | `/run/systemd/sessions/*` (logind writes plain-text session records) | read by nobody |
| Autologin | `image/mkosi.extra/usr/lib/systemd/system/serial-getty@ttyS1.service.d/autologin.conf` | **the serial console logs in without a password** — an existing, deliberate property of the VM image that must not be mistaken for authentication |
| Capability vocabulary | no `AuthSession` capability exists | `docs/SEC-01-capabilities.md` has no row for authentication because no component authenticates |

## 2. The three decisions this note forces

1. **Who verifies a credential?** The candidates, with their real costs:
   - **The distribution's own helper.** **Chosen 2026-10-12.** `pam_unix` does
     not verify a password in-process: it executes `/usr/bin/unix_chkpwd`, a
     setuid-root binary that reads the password on its standard input and
     answers with its exit status. That binary is on the machine already, is
     maintained by the people who maintain `/etc/shadow`, and needs no `unsafe`
     in this workspace, no new process of ours and no password store. The daemon
     runs it with a fixed argument vector (`<user> nonull`) and the password
     written to the child's stdin, so the secret never reaches an argument
     vector; `SystemRunner::run_with_stdin` is the only place a program is fed
     anything, and a runner that cannot supply stdin **refuses** rather than
     running the helper with an empty password.
   - **PAM through a helper process of ours.** `libpam` is already on the system,
     but the workspace forbids `unsafe_code`, so the shell cannot call it and a
     helper would have to. Superseded: it would mean a second implementation of
     what `unix_chkpwd` already is, inside our own privilege boundary.
   - **PAM inside the shell** — rejected: it would put `unsafe` (or a dependency
     that provides it) inside the desktop process, which is the one process that
     parses untrusted content.
   - **Our own password database** — rejected for now: it invents a credential
     store and a hash format and a migration story, all to replace something the
     distribution already maintains.
2. **What is a session?** logind already models sessions, seats and users, and
   writes them to `/run/systemd/sessions/`. The project should *read* that model
   rather than invent a parallel one; where the shell needs richer state (locked,
   which application holds a lock), it keeps its own record and ties it to
   logind's session id.
3. **What does the lock screen lock?** In a framebuffer desktop with no client
   surfaces ([ADR-0017](../adr/0017-framebuffer-shell.md)), "lock" means the shell
   stops drawing content and refuses input until a credential is verified. It does
   **not** mean the machine is safe from a second console — the serial autologin
   above is the proof — so a lock screen claim must be worded for what it is:
   it protects the session on this display, not the machine.

## 3. Data surface (facts, Phase 1)

```
session.count   <n>
session.logind  present|absent
session.id      <id> user=<name|-> uid=<n|-> seat=<seat0|-> type=tty|wayland|unspecified
                     state=online|active|closing|unknown[:<word>] tty=<path|-> display=<:0|serial|-> leader=<pid|->
auth.available  none|unix-chkpwd|pam-helper
auth.lock       refused            (present only while locking cannot work)
```

`auth.available` exists so the shell can be honest: a lock screen that cannot
verify a credential must say so and refuse to pretend, which is the rule the
2026-08-29 note already set. `session.display` is folded into the `session.id`
line rather than being a second line keyed by session, because a client that
joins two lines by id has to handle a line that never arrives.

`session.logind=absent` is a real state and not an error: a container or an
image without `systemd-logind` has no session records, and reporting zero
sessions for it would be a claim about a machine nobody looked at.

## 4. Action surface (capabilities)

| Action | Capability | Note | State |
|---|---|---|---|
| Read session facts | none | machine facts | implemented |
| Verify a password | `AuthSession @ lock://<user>`, **live consent** | the request must carry `CONFIRM`: `AuthSession` is a live-consent capability, and the password the user just typed is the answer to that prompt. Without the word the broker refuses and the helper is never reached | **implemented** (`AUTH VERIFY <user> <hex-password> CONFIRM`) |
| Lock the display | `AuthSession @ lock://<session-id>` | the shell holds it; no app may lock the screen to hide its own activity | **implemented** — `Shell::lock_now()` raises the surface, which clears the frame before it draws, and `Shell::take_unlock()` is the only way out |
| Switch user | `AuthSession @ switch://<uid>` | Phase 2 |
| Change a credential | **not ours** | `passwd`/PAM tooling stays the distribution's business; we add no second way to set a password |

## 5. Threat model

| # | Abuse case | Mitigation in this design |
|---|---|---|
| U1 | The lock screen accepts anything (the failure mode the project already named) | two separate facts, because they are two separate questions: `auth.available` says whether a password can be checked, `auth.lock` says whether content can be hidden, and **the lock action refuses while the second is false even though the first is true**. The shell asks both before it raises the surface — `Shell::can_lock` is `verifier().can_lock() && auth_lock == Some(false)` — and a refused lock reports the daemon's own sentence rather than a paraphrase. A verification that fails (a wrong password, an account whose password is locked, a helper that is not there) is a failure at every level: the helper's non-zero status becomes `ERR action-failed`, never an `OK` |
| U2 | An app impersonates the helper and harvests credentials | the helper's socket is a fixed path with `0600` ownership by the helper's user; the client checks peer credentials (`SO_PEERCRED`) before writing a byte, and refuses a peer with a different euid |
| U3 | The helper is fed an oversized or malformed request and crashes, leaving the session unlocked | a byte ceiling, one request per connection, a timeout, and a refusal on any parse error; a crashed helper is a *denied* verification, never an accepted one — fail closed is the whole design |
| U4 | Brute force against the verifier | the helper counts consecutive failures per session and applies an increasing delay (PAM's own `pam_faildelay` is not enough on its own); the delay lives in the helper so a restarted shell cannot reset it |
| U5 | The password is captured in a log, a notification or an audit entry | the password travels hex-encoded on a `0600` socket, is written only to the child's stdin, and never appears in an argument vector, a reply line, a plan's `command_line()`, or an audit entry. A test asserts that the reply to a successful verification does not contain the password, and the test runner records only the *length* of what it was fed so that a printed call list cannot leak one either. `unix_chkpwd`'s stderr is discarded rather than inherited |
| U6 | The serial console autologin is mistaken for a secure login path | recorded here and in the image's `autologin.conf`; the VM image is a development artifact, and no claim about authentication may be made from it |
| U7 | A lock screen shows the previous session's content in a frame buffer that is not cleared | the lock path clears the framebuffer before drawing the lock surface, and a test asserts the order (draw-after-clear, not clear-after-draw) |
| U8 | An unlock attempt succeeds from a different seat/display than the one being locked | `AuthSession` is scoped to a session id; the helper verifies the session belongs to the caller's seat |

## 6. What is verifiable today

- The session-record parser, with fixtures — including a fixture for the
  record shape logind actually writes, so the claim "this is what logind writes"
  is a test rather than a recollection. **Implemented**: the fixture is the
  twelve-line record logind writes, and the tests cover the `ACTIVE=0 /
  STATE=online` case, an unrecognised state reported as `unknown:<word>`, and a
  record with nothing but an id.
- That nothing in the tree holds a password store, a hash format or a `libpam`
  binding: `grep`, and the fact that the only credential code is a fixed argument
  vector plus a pipe.
- The whole verification path, on any host, without a password that works:
  `cargo test -p litric-sysd` covers an unconfirmed request being refused by the
  broker *and the helper never being reached*, a confirmed request being fed the
  password on stdin while the argument vector holds only the account and
  `nonull`, a wrong password (exit 7) reported as a failure rather than a login,
  a missing helper reported as `action-failed`, and a reply that does not contain
  the password. **Implemented 2026-10-12.**
- That the serial autologin exists: the unit file, read directly.
- The lock surface, driven the way a user drives it: `cargo test -p litric-shell`
  runs end-to-end journeys through synthetic input. A machine that cannot check a
  password refuses to lock and says why; locking hides the session — the frame is
  drawn over a sentinel colour and **no pixel of what was underneath survives**,
  which is U7's clear-then-draw order as a test; only a password brings the
  session back, and the typed secret leaves the shell once, as
  `Effect::VerifyPassword`; a lock that could not collect a password shows the
  refusal instead of a field; the customize sheet changes the look live; and the
  dock grows into the launcher, whose tiles and shortcut row perform what their
  labels say. **Implemented 2026-10-12.**
- The cost of the surface: `src/litric-shell/examples/render_bench.rs` times a locked frame in
  four states, because a locked shell composes in full on every tick at 30 FPS
  and is therefore the shell's most expensive steady state. The numbers, and what
  they were measured on, are in `docs/EVIDENCE.md`. **Measured 2026-10-12** on the
  development host, not on the target.

## 7. NOT verified, and what it would take

| Claim | Needs |
|---|---|
| a *correct* password verifies | an account with a known password in an isolated image, and a credential the test may hold. **NOT verified, by choice**: putting a known password into a shipped image would be a worse defect than the missing evidence, so what the booted image proves is the refusal direction (below) and the helper's presence |
| a wrong password does not verify, in a booted image | a booted image — **done 2026-10-12**: the in-guest self test asks `AUTH VERIFY root <hex of "wrong"> CONFIRM` and records `litric.sysd.refuses_wrong_password=yes`, and separately that the unconfirmed form is refused with `needs-live-consent` |
| the helper is what the image runs | a booted image — **done 2026-10-12**: the self test records `/usr/bin/unix_chkpwd` present with mode `4755` |
| the lock cannot be bypassed | the implementation plus an adversarial test (kill the helper, malformed input, oversized frame). **Done on the host**: `cargo test -p litric-sysd` covers a request that is refused before any helper is reached, a missing helper, and a reply that does not carry the password; `cargo test -p litric-shell` covers the surface refusing to appear at all. **Not done on a booted image**, so "the lock cannot be bypassed *on this machine*" is not yet a claim this note makes |
| locking is meaningful on this machine | a booted image where the serial console's autologin is *also* removed, or an explicit statement that the lock protects the display and not the machine. **Still open**: the surface stops the shell drawing and accepting input, and the serial console is outside it |
| the surface looks the way the design specifies, on the target display | a booted image and a screenshot. The design is a port of the prototype pack, and `docs/UI-PLAN.md` P10–P11 record what was ported. **NOT verified**: every frame measured so far is the development host's |

## 8. Tickets this note implies

1. ~~Facts: `/run/systemd/sessions/*` parser + fixture tests (Phase 1, useful now).~~
   **Done** — `src/litric-sysd/src/platform/session.rs`.
2. ~~Capability `AuthSession` with `lock://`, `switch://` and live-consent rules.~~
   **Done** — `auth.session`, live consent on both scopes.
3. ~~**Decision ticket:** the helper~~. **Answered 2026-10-12**: no helper of
   ours; the distribution's own `unix_chkpwd`, run with a fixed argv and the
   password on stdin, by the daemon that already holds the capability broker.
4. ~~Helper implementation~~. **Done 2026-10-12** in
   `src/litric-sysd/src/providers.rs` (`AuthAction::Verify`), fail-closed: a
   non-zero status, a missing helper and a runner that cannot supply stdin are
   all failures. **Not done:** the increasing delay per session (U4) — a
   brute-force limiter is `pam_faildelay`'s job today and needs a decision about
   where the counter lives, since the daemon restarts.
5. ~~Lock screen: clear-then-draw, refuse while `auth.lock=refused`, and a test
   that asserts the refusal (a lock that cannot verify must not appear to work).~~
   **Done 2026-10-12** — `src/litric-shell/src/lock.rs` is the surface (Liquid
   Glass: three drifting colour fields, a glass clock whose glow and bloom are
   adjustable, a breathing clock, a widget row, and a customize sheet whose four
   tabs edit the look live), `views::draw_lock` clears the frame before it draws,
   `Shell::lock_now` refuses unless the verifier fact and the daemon's answer
   agree, and the journeys in §6 are the tests. **What is left** is the
   booted-image half: the same journeys have not been driven on the VM, and the
   frame cost is a host measurement.
