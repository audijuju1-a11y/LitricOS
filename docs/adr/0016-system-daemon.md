# ADR-0016 — `litric-sysd`: one process that runs the Layer 2 and Layer 3 code

- **Status:** Accepted, implemented, verified inside a booted image
- **Date:** 2026-08-29
- **Extends:** ADR-0008 (security model), ADR-0009 (unified shell process)

---

## The problem this solves

By 2026-08-28 LitricOS had a capability engine, a storage broker, a manifest
parser, a Windows-compatibility planner, an environment manager, a trash policy
and a design system. All of them had tests. None of them had ever executed
inside the operating system they were written for.

That is a specific and dangerous state to be in, because it looks like progress
from every angle except the one that matters. The scope ledger counted 70 rows
of "real code with passing tests". The image booted to a login prompt with zero
failed units. Both were true, and between them there was nothing: the booting
system was Arch Linux with our configuration on it, and our code was a library
nobody linked.

Layer 4 exists to close exactly that gap.

---

## The decision

**One long-lived process, `litric-sysd`, holds one instance of each decision
engine for the whole machine, and answers questions about them over a Unix
socket.**

```
   /proc, /sys ──► facts ──┐
                           ├─► Sysd ──► line protocol ──► /run/litric/sysd.sock
   litric-caps ────────────┤
   litric-storage ─────────┤            litric-shell, settings, the selftest
   litric-trash ───────────┤            and anything else that needs to know
   litric-env  ────────────┘
```

### Why one process and not one per subsystem

The obvious alternative is a daemon each: `litric-capd`, `litric-envd`,
`litric-filesd`. It is the shape the unit files in `image/mkosi.extra` were
already written for, and it is the shape most systems end up with.

It is wrong here for a reason specific to this project's goals. The headline
constraint is idle memory (PERF-budget §3: ≤ 320 MB for a full desktop). Every
additional daemon costs a process, a socket, a systemd unit, a page table and a
share of the audit and journal machinery, and buys isolation between components
that already trust each other completely — the storage broker cannot resolve a
URI without asking the capability broker, so splitting them puts a socket in the
middle of one decision.

Measured: `litric-sysd` resident set inside the booted nano image is **2.2 MB**.
Four daemons would not be four times that, but they would not be free either,
and the isolation they buy is between two halves of the same answer.

The split we do keep is the one that matters: the **shell** is a separate
process from the **daemon**, because the shell touches the display and input,
which are the parts an attacker reaches first.

### Why a text protocol

`OK <count>` then that many `key=value` lines; `ERR <kind> <detail>` on failure.

A binary protocol would be faster and would mean that when the desktop shows the
wrong memory figure at three in the morning, nobody can see why without writing
a decoder first. The daemon is not on a path where encoding is the bottleneck:
it answers a handful of questions a second, and the numbers in them came from
reading `/proc`, which costs more than formatting them ever will.

The declared line count in the header is not decoration. A client that reads
until it sees something that looks like the end is a client that breaks the
first time a value contains that thing; the count means the framing cannot be
confused by the payload, and `Response::encode` strips newlines from values so
the count is always true.

### Why the caller's identity never comes from the caller

`RESOLVE <app> <uri> <access>` names an application, and the daemon uses that
name to look up which capabilities were granted — it never accepts a capability
in the request. This is the confused-deputy defence from ADR-0008 applied at the
one place it has to hold: the daemon has more authority than any of its callers,
so anything it believes on their word is authority they have borrowed.

---

## What it does not do

Stated because the replies say so and the tests enforce it:

- **The trash is in memory.** Nothing has read `~/.local/share/Trash` from disk
  yet, so `TRASH LIST` on a real boot correctly reports nothing.
- **`ENV SWITCH` moves the state machine, not a virtual machine.** `litric-envd`
  does not exist. The reply says `applied=state-only` rather than reporting a
  switch that did not happen.
- **There is no persistence.** Grants and trash entries do not survive a reboot.

---

## Evidence

From `docs/evidence/`, captured inside a booted nano image on 2026-08-29:

```
litric.sysd.binary=present
litric.sysd.socket=present
litric.sysd.rss_kb=2260
litric.sysd.ping=pong
litric.sysd.environments=3
litric.sysd.denies_ungranted_read=yes
```

The last line is the one worth keeping: a `RESOLVE` for `user://Documents/x` by
an application holding no grant was refused by the running system, not by a
unit test.

---

## Consequences

- Layer 5 has something to ask. `litric-shell` draws numbers it did not invent.
- The capability model is now falsifiable from outside the code: anyone can
  connect to the socket and try to get a path they should not have.
- `litric-capd.service` and `litric-capd.socket` in `image/mkosi.extra` are now
  redundant and refer to a binary that does not exist. They are not enabled, and
  should be removed rather than left to look like they work.
- The daemon is a single point of failure for the desktop's data. It restarts on
  failure, and the shell renders from `/proc` directly when the socket does not
  answer, saying on screen which source it used.
