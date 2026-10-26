# ADR-0025 — `litric-notifyd`: the notification queue is a daemon, not a list

- **Status:** Accepted, implemented, unit-tested; not yet exercised inside a
  booted image
- **Date:** 2026-10-26
- **Extends:** ADR-0016 (one system daemon), ADR-0008 (security model)

---

## The problem this solves

The shell has had a notification centre since the desktop existed, and it was
honest about its limit: the queue lived in the compositor's memory. So a
notification raised while the shell was restarting, or before it started at all,
was lost — and nothing could tell the user it had ever existed.

That is a small defect with a specific shape: *the record of a message is the
process that displays it*. Every consequence follows from that. A shell crash
discards everything unread. A program with something to report has to be running
after the desktop is up, and has to find it. And "you missed four notifications
while the desktop restarted" is a sentence no component can honestly say.

---

## The decision

**A separate process, `litric-notifyd`, owns the queue and writes it to disk
before it answers. The shell takes the queue at startup and empties it.**

```
  any program ──► POST ──►  Queue ──► /var/lib/litric-notifyd/queue.conf
                              │
   litric-shell ──► TAKE ─────┘        (taken notes are removed)
```

### Why a separate daemon, when ADR-0016 argued against one per subsystem

ADR-0016's argument was against splitting a **stateless decision** across two
processes: the storage broker cannot answer without the capability broker, so a
socket between them would sit in the middle of one decision. That argument does
not reach this case, and three differences decide it:

1. **The lifetime requirement is different, and inverted.** `litric-sysd` is
   useful only while something is asking it questions. The queue must be able to
   accept a post *while the desktop is not running* — that is its entire purpose.
   Putting it inside the shell would put the record in the process whose absence
   it exists to survive; putting it inside `sysd` would tie "can I tell you
   something" to "can I measure memory", and a wedged facts reader would silently
   stop accepting notifications.
2. **Its state is authoritative and durable.** `sysd` answers from `/proc` and
   `/sys`; if it restarts, it has lost nothing. This daemon holds the only copy
   of what was queued, so it owns a file and a `StateDirectory`, and it is the
   first LitricOS daemon whose restart has a cost.
3. **The failure domains should not be shared.** The queue accepts text from any
   program on the machine. That is a wider input surface than the rest of the
   system daemon, which reads kernel files. Keeping the parser that accepts
   arbitrary strings in its own process, with an empty capability set and no
   network, means a bug there cannot reach authentication or power.

The cost is one process. The unit caps it at 32 MB and 16 tasks; the queue itself
is 64 notes of a few hundred bytes.

### Why the queue has no Do Not Disturb flag

Because the shell already has one, and two answers to "should this interrupt?"
can disagree. The switch lives with the thing that decides when to draw, and the
queue is deliberately dumb about it: a note taken while the desktop is quiet
still appears in the centre, and only the banner is suppressed. This is also why
the protocol has no `DND` request — not an omission, a decision.

### Why `TAKE` and not `LIST`

`LIST` exists for inspection. `TAKE` is the delivery primitive, and it returns
the notes **and empties the queue in one request**. A read followed by a separate
delete has a window in between; a shell that crashes inside it delivers the same
notification twice. One request cannot be half done.

The consequence is intentional: a note the desktop has taken is gone from the
record. This is a notification queue, not a mailbox.

### Why the format is `key=value` lines

The same reason `settings.conf` and the sysd protocol use it. The runtime crates
take no third-party dependencies (the hard rule at the top of `src/Cargo.toml`),
so JSON would mean a hand-written parser with hand-written escapes — and an
escape bug in a parser that accepts strings from every program on the machine.
A line-oriented format has no escapes to get wrong, and the queue file can be
read with `cat` while the desktop is broken.

Control characters are therefore *refused* rather than escaped: a title with a
newline is a second line of protocol, not a longer title.

### Why the write is atomic, and what a failed write means

Temporary file, `fsync`, `rename`, `fsync` the directory — the same sequence
`prefstore` uses, for the same reason: the file on disk is always a complete
queue, either before the change or after it.

A write that fails does **not** drop the notification. It stays in memory, the
reply says `persisted=0`, and the daemon logs the reason to the journal. The
promise that is broken is "this survives a reboot", and breaking it silently is
the failure this project refuses; breaking it loudly is a disk-full message.

### Why expiry is absolute, and why critical never expires

A `ttl_ms` from a sender is converted to an absolute expiry the moment it is
accepted, and expired notes are dropped on load. A five-second toast posted
before a reboot must not appear afterwards as though it had just arrived.

`Priority::Critical` never gets an expiry, whatever the sender asked for. This is
the shell's existing rule; the queue repeats it so that a reboot cannot quietly
age out the one class of message that is never supposed to age.

---

## What this does not do

| Not done | Why |
|---|---|
| Live push while the desktop is running | the shell drains on the facts cadence (2 s), which bounds the delay instead of adding a subscription channel; a verified push path needs the shell's event loop to own a socket, which is a change to the compositor, not to this daemon |
| Action buttons, inline replies | they need an action model and a way to run the action after the user's session has changed; the queue carries text |
| Per-application badges and priority editing | needs an application registry, which the shell does not have (ledger `WA.07.04`, `DEFER`) |
| Rate limiting a misbehaving sender | the queue is capped at 64 and evicts oldest-non-critical, so a flood costs other notifications rather than memory; a per-sender policy belongs with the capability broker, which does not yet have a persisted grant store |

Everything above is recorded in `docs/parity/ledger.csv` as `ADAPT` or `DEFER`
with the reason, so a reader can tell a decision from an oversight.

---

## Consequences

- `image/mkosi.extra/usr/lib/systemd/system/litric-notifyd.service` runs it,
  enabled by `image/mkosi.postinst.chroot`, staged by `tools/build.sh`.
- The shell links `litric-notifyd` for the client half, so there is exactly one
  implementation of the wire format — two is how the browser's frame header came
  to disagree with itself.
- A notification that was raised while the desktop was down appears in the centre
  and **does not raise a banner**. Adoption is "you missed this", not "stop what
  you are doing".
- The status file carries `inbox=<n>`, so "four arrived while the desktop was
  down" is checkable from outside the shell.
