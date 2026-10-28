# LitricOS UI/UX Research — Part 23 of N

**Chapter I — Technical integration and LitricOS feasibility**

Status: **PARTIAL** — all eleven rows measured; two decisions are still open by design.
Prepared: 2026-10-28. Targets: Windows 11 26H2, macOS 27, GNOME 51 / KDE Plasma 6.8, LitricOS at this tree.

---

## 0. Scope, method, and why Chapter I asks a different question

Chapters W, M, F, G and H asked *what does the reference do to a person*. They
measured screens, flows, wording, and failure paths. Chapter I asks a question
that is one layer down and much less flattering:

> **What can somebody outside the OS attach to, and what would LitricOS have to
> publish before such a person could attach at all?**

That question cannot be answered by reading Settings pages. It is answered by
counting what each system exposes as a contract — a toolkit, an extension point,
a settings database, a permission model, a notification API, an IPC substrate, a
compositor protocol — and by comparing that against what this tree actually
implements.

### 0.1 The seven surfaces this chapter measures

| Row | Surface | Why it decides whether an ecosystem can exist |
|---|---|---|
| I.01 | Public UI framework | Without one, every app is a separate visual language |
| I.02 | Shell extension points | Without them, the shell can only ever show what it shipped with |
| I.03 | Settings persistence | The store is where an OS either offers a namespace or forces every app to invent one |
| I.04 | Permission model | Determines whether "deny by default" is usable or merely restrictive |
| I.05 | Notification API | The most-used OS API in ordinary software, and the easiest to make private by accident |
| I.06 | IPC / events | One substrate is a platform; four is four products |
| I.07 | Compositor protocols | Decides what an existing toolkit can do unmodified |

### 0.2 What was measured, and how

Code claims below come from this tree, counted mechanically:

```text
workspace crates                        21
compositor globals created              3   (wl_compositor v4, wl_seat v5, xdg_wm_base v2)
wl_seat capabilities advertised         2   (pointer, keyboard)
preference keys under schema version 1  8   (+ the version line)
capability rules enforced by litric-caps 6
capability names a manifest may carry   bounded by Manifest::MAX_CAPABILITIES, unknown = hard error
notification daemon contract            text in, text out; no rendering, no DND, no duration
LFP/1 requests retried after the line   0
D-Bus references in litric-notifyd      0
UI scale steps this build offers        3   (1.00, 1.25, 1.50)
revert window for a scale change        15 s
tests in litric-shell                   596
```

Platform claims come from seven sources inspected for this part (S3850–S3855, S3893).
They are cited where they carry a number or a rule, not as background.

### 0.3 What this part does not measure

* **No third-party application exists**, so nothing here is an observation of an
  ecosystem. Every I.01–I.08 finding is about a *contract*, not about an app.
* **No macOS API reference was inspected in this part.** The macOS rows use the
  AppKit and System Settings sources already inspected for Parts 5–6 (54 AppKit
  rows). They support structural claims (AppKit is the toolkit, `defaults` is
  the store) and nothing finer.
* **TCC and XPC appear zero times in the whole register.** That is a gap this
  part records rather than papers over: the macOS permission and IPC columns of
  the comparison matrix are structural, not API-level.
* **Windows shell extension handlers were not fetched.** The Windows extension
  claim rests on MSIX identity (S3851) and UAC (S3854), not on the shell
  extension documentation.

---

## I.01 — Public UI frameworks per OS

### I.01.1 What the platforms offer

| OS | Toolkit a third party writes against | Consequence for consistency |
|---|---|---|
| Windows 11 | WinUI / Fluent (already inspected for Part 4) alongside Win32 | A packaged app gets the system controls, theme resources and motion for free |
| macOS 27 | AppKit (54 sources inspected for Parts 5–6) | The toolkit *is* the look; an app that uses it cannot easily look foreign |
| Linux | GTK 4 and Qt 6 over Wayland (11 GTK sources inspected) | Two toolkits, two looks, one protocol underneath |
| LitricOS | **none** | An app can draw pixels and nothing else |

### I.01.2 What LitricOS has

The shell's own interface is built from `ui.rs` (1,615 lines), `views.rs` (6,603
lines), `canvas.rs` and a stroke font. Applications are *variants of an enum*:
`App::ALL` is a static table, and every window body in `views.rs` is a `match`
arm. There is no widget crate, no theme-resource lookup for a foreign process,
and no header a third party could compile against.

What a foreign process gets is `clients.rs`: a Wayland surface mapped to a
window whose title bar the shell draws and whose body is entirely the client's.

### I.01.3 Findings

**I.01.1 — Every visual guarantee in Parts 4–20 stops at the process boundary.**
Severity: high. Evidence: T3 (code). The contrast floors (Part 19), the token
families (Part 20), the focus ring, the 3 px border fix — all of it lives inside
the shell binary. A third-party app that opens a window inherits exactly one
thing: the title bar. Its body can violate every rule this corpus measured, and
the OS has no mechanism to notice.

**I.01.2 — "Draw your own pixels" is a complete contract and a poor platform.**
Severity: high. Evidence: T3. The contract is honest: a shared-memory buffer, a
size, a title. It is also the contract of a remote desktop, not of a desktop
environment: no menus in the system bar, no theme, no high-contrast adaptation,
no screen-reader names. A toolkit is not a convenience — it is the only way a
platform can promise that an app it did not write is still usable by a person
who needs contrast, keyboard access or a screen reader.

**I.01.3 — The shell's own apps are the only measurable design conformance.**
Severity: medium. Evidence: T3. Because apps are enum arms, every journey test in
`e2e.rs` measures first-party drawing. That is why the 596 tests can assert
contrast and hit sizes at all. It also means the corpus has no measurement of a
window the shell did not draw, which is the case that will dominate a real
ecosystem.

---

## I.02 — Shell extension points

### I.02.1 What the platforms offer

Windows exposes shell extensions as registered handlers; macOS as app extensions
and Services; Linux as `.desktop` entries, D-Bus services and portals. In each
case the OS publishes a *place* where a foreign component can appear — a context
menu item, a share target, a settings pane, a search provider.

### I.02.2 What LitricOS has

One documented extension point: `litric-manifest`, which declares an application
and the capabilities it needs. Its parser treats an **unknown capability as a
hard error** and refuses a manifest that repeats one or exceeds
`MAX_CAPABILITIES`.

There is no extension registry. The launcher reads `App::ALL`; the context menu
is a `match` on window content; the settings pages are thirteen static `Page`
variants; quick settings is a fixed list of toggles.

### I.02.3 Findings

**I.02.1 — The manifest is a permission declaration, not an extension point.**
Severity: high. Evidence: T3. It answers "what may this app do", never "where
does this app appear". A file manager that wants a context-menu verb, a search
provider that wants to answer queries, a settings pane that wants a page: none
of these has anywhere to register. The only integration a third-party app can
perform is "be launched".

**I.02.2 — Refusing an unknown capability is the right failure, and it is rare.**
Severity: none (a strength). Evidence: T3, and S3851 for why the alternative is
worse: on Windows a package's manifest declares capabilities too, and a manifest
that is *partly* understood is how an app ends up with less authority than its
installer claimed. LitricOS refuses the whole manifest instead, which is a
strictly better failure for a security boundary and a worse one for forward
compatibility — a manifest from a newer build will not install at all.

---

## I.03 — Settings persistence

### I.03.1 What the platforms offer

| OS | Store | Property that matters |
|---|---|---|
| Windows | Registry (S3893, inspected) | A system-defined database every app reads and writes through the registry API |
| macOS | `defaults` / property lists (2 inspected sources) | Per-app domain, atomic-ish, readable by `defaults` |
| Linux | GSettings/dconf, KConfig | Schema with typed keys, change notification, per-user overlays |
| LitricOS | `prefstore.rs` | 9 keys, `key=value`, version 1, atomic write, `0600` |

### I.03.2 What LitricOS has

A line-oriented file: `#` comments, one `key=value` per line, `version` first.
Parsing never fails — an unknown key is ignored, an unparsable value keeps the
default, a newer version still parses every known key. Writing is atomic
(temp file, `fsync`, `rename`, directory `fsync`) and the file is `0600` in a
`0700` directory.

### I.03.3 Findings

**I.03.1 — There is no settings namespace for anyone but the shell.**
Severity: high. Evidence: T3. The store has exactly the eight keys the shell
owns. A third-party app that wants to remember a preference has no service to
call: it will write its own file, in its own format, with its own permissions,
and the OS cannot audit, migrate, back up or reset it. Every platform in the
table solved this by owning the store; LitricOS solved it for one process.

**I.03.2 — The failure policy is better than the reference, and better than it
looks.** Severity: none (a strength). Evidence: T3. A registry that cannot be
parsed can leave a profile unusable; a dconf database that is corrupt is a
support case. Here a corrupt line costs one setting and the desktop still
starts, and the file can be repaired with `cat` while the desktop is broken —
which is the same argument ADR-0021 makes for the recovery story.

**I.03.3 — No change notification.** Severity: medium. Evidence: T3. GSettings
and KConfig emit a signal when a key changes; the shell re-reads on its own
actions only. With one writer that is invisible. The moment a second process can
write preferences — which I.03.1 says is coming — a stale in-memory copy becomes
a correctness bug, not a refresh bug.

---

## I.04 — Permission models

### I.04.1 What the platforms offer

* **Windows** — UAC: a split token, and a consent prompt when an operation needs
  the full one (S3854). The same source records a trap worth copying into any
  design review: "Never notify" no longer disables UAC; it grants the split
  token *and* auto-elevates, so the setting named "off" is the least safe one.
* **macOS** — TCC prompts per resource class, per app, with a record the user can
  revoke. (No TCC source was inspected for this part; see 0.3.)
* **Linux** — polkit (S3855): a *mechanism* asks an *authority* whether a
  *subject* may perform a declared *action*; a per-session authentication agent
  proves the user is who they say, and actions are declared in XML under
  `/usr/share/polkit-1/actions`.
* **Linux (sandboxed apps)** — xdg-desktop-portal (S3852): D-Bus interfaces that
  mediate file access, opening URIs and printing, with a per-desktop backend.

### I.04.2 What LitricOS has

`litric-caps` is a pure decision engine and `litric-capd` is the single
authority. Six rules are stated and enforced: deny by default, scope per
resource not per class, revocation effective on the next decision, expiry
enforced, every decision recorded (allow and deny), and no confused deputy —
authority is looked up from the *caller's* identity, never from a capability the
caller presents. Time is injected, never read, so expiry is testable without
sleeping and the audit log is reproducible.

### I.04.3 Findings

**I.04.1 — The authority exists; the conversation does not.**
Severity: high. Evidence: T3 vs S3854/S3855. All three reference systems put a
human in the loop at the moment of escalation: a UAC consent prompt, a TCC
dialog, a polkit authentication agent drawn by the session. LitricOS has no
prompt surface and no authentication agent. A grant is issued out of band, which
means the only moment a user can consent is install time — and a user who is
asked once, at install, for everything, will say yes.

**I.04.2 — A folder grant is not a file grant.**
Severity: high. Evidence: T3 vs S3852. Portals hand out access to the *thing the
user chose*, for as long as that choice lasts. LitricOS hands out a scope, and a
scope does not end when the task does. "Least privilege by scope" is true and
still coarser than a picker: the Files app holding a grant on a directory holds
it for every file that will ever be created there.

**I.04.3 — The audit trail is stronger than any reference default.**
Severity: none (a strength). Evidence: T3. UAC and polkit both log, and neither
puts the record in front of the user as a first-class surface. LitricOS records
every decision, allow and deny, with injected time — which is exactly what a
"which app read my files" page would need. That page does not exist yet (H.13),
so the strength is currently unrealised.

---

## I.05 — Notification APIs

### I.05.1 What the platforms offer

* **Linux** — the Desktop Notifications Specification, version 1.3 (S3850): a
  D-Bus service with actions, hints, urgency levels, categories, markup and
  icon/image fields. Its own scope note is precise: passive popups only, not
  modal dialogs, not window decorations, not window-list annotations.
* **Windows** — toast notifications are a WinRT platform API, and MSIX requires
  **package identity** for push notifications, background tasks and live tiles
  (S3851). An unpackaged app cannot use the modern notification surface.
* **macOS** — the user-notification framework, gated by a per-app authorization
  request (AppKit sources from Parts 5–6).

### I.05.2 What LitricOS has

`litric-notifyd` is a queue, not a presenter. Its module documentation states
three deliberate refusals: it does not decide whether to disturb the user (no
Do Not Disturb flag — the shell decides at display time), it does not decide how
long a notification stays on screen (the sender's `ttl_ms` becomes an absolute
expiry so a reboot cannot resurrect a five-second toast), and it does not render
(text in, text out; "a daemon that drew would be a second implementation of the
design system, reachable by anyone who can open a socket"). Every mutating write
is atomic and a failed write is reported, because the queue's promise is that a
notification raised while the desktop is down still arrives.

### I.05.3 Findings

**I.05.1 — The API is private, so no third-party app can notify anyone.**
Severity: high. Evidence: T3 vs S3850. The daemon's wire protocol is its own;
the register finds **zero** references to `org.freedesktop` or D-Bus in it. A
Linux app posts a notification through a documented D-Bus call and works on
every desktop. Here an app must link LitricOS-specific code, which means ports
carry a `#ifdef` and a maintenance burden — the classic reason small platforms
lose application support.

**I.05.2 — "Who decides to interrupt" is separated correctly, and the reference
does not.** Severity: none (a strength). Evidence: T3 vs S3850. On Linux the
*daemon* implements DND, so every daemon must implement it and two daemons can
disagree. LitricOS puts the policy in the shell and the record in the daemon, so
there is one answer to "should this interrupt" — and a notification taken while
the desktop is quiet still appears in the centre.

**I.05.3 — None of the specification's delivery semantics exist.** Severity:
medium. Evidence: T3 vs S3850. No replace-by-id (a chat app posting "3 messages"
then "4 messages" produces two banners), no actions, no hints, no urgency
mapping beyond the shell's own three priorities, no markup. Each of those is a
small feature and a large compatibility gap: a ported app does not degrade
gracefully, it degrades silently.

---

## I.06 — IPC and events

### I.06.1 What the platforms offer

One substrate each: COM/RPC and named pipes on Windows, XPC and Mach ports on
macOS, D-Bus on Linux. One substrate is what makes discovery, introspection and
policy possible; four substrates is four products with four failure modes.

### I.06.2 What LitricOS has

Three line protocols and no bus:

| Path | Protocol | Notable rule |
|---|---|---|
| Files | LFP/1 over a Unix socket, one connection per process | **Never retried** once the request line is written |
| Browser | `LITRIC_BROWSER_SOCK` | Frames, not requests |
| Notifications | `litric-notifyd` queue file + socket | Text in, text out |

The LFP/1 rule is stated as a correctness property, not an optimisation: if the
connection breaks after the request line, the daemon may already have performed
the rename or the delete, and a retry would do it twice.

### I.06.3 Findings

**I.06.1 — Three protocols, no discovery, no schema, no common error model.**
Severity: medium. Evidence: T3. Each service defines its own framing and its own
error vocabulary. Nothing can enumerate what is running. A user cannot ask "what
is talking to what", which is precisely the question a permission audit (I.04.3)
would need to answer.

**I.06.2 — At-most-once mutations are documented, which is rarer than it
should be.** Severity: none (a strength). Evidence: T3. Most IPC tutorials show
a retry loop; most retry loops are wrong for mutations. Writing the rule down in
the module that implements it is the difference between a property and a habit.

**I.06.3 — No event stream.** Severity: medium. Evidence: T3. The shell polls
the machine (its facts are sampled) and has no subscription to "the network came
up" or "a file appeared". That is why H.08's network page is read-only and why
the Files window re-reads a folder after an operation instead of being told.
Polling is a defensible choice at this size; it stops being one when a second
process is expected to react to the first.

---

## I.07 — Compositor protocols (Wayland)

### I.07.1 What the protocol offers

The core Wayland protocol (S3853) is a list of interfaces: `wl_display`,
`wl_registry`, `wl_compositor`, `wl_surface`, `wl_shm`, `wl_seat`, `wl_pointer`,
`wl_keyboard`, `wl_output`, `wl_data_device_manager`, `wl_subcompositor`, and the
`xdg_*` family in the extended protocols. The same document describes the
compositor as a program that can run on KMS and evdev directly — which is
exactly what LitricOS does.

### I.07.2 What LitricOS has

Three globals, with versions: `wl_compositor` v4, `wl_seat` v5, `xdg_wm_base`
v2. `wl_seat` advertises pointer and keyboard only. The shell's own crate docs
list what is deliberately absent: XWayland, clipboard, drag-and-drop, an input
method, and a GPU import path.

### I.07.3 Findings

**I.07.1 — A toolkit can open a window, and then stop.** Severity: high.
Evidence: T3 vs S3853. Three globals are enough for GTK or Qt to map a toplevel
and paint into it. They are not enough for copy/paste (`wl_data_device_manager`),
for IMEs (text-input protocols), for accessibility (AT-SPI runs over its own
bus, not Wayland), or for decorations negotiated by the client. A ported toolkit
will open a window that looks alive and behaves like a screenshot.

**I.07.2 — `wl_output` is imported but never advertised.** Severity: medium.
Evidence: T3. The protocol module names `wl_output`, and the registry creates
only three globals; no output global is among them. A client therefore cannot
learn the output's size, scale or refresh rate — which is the information a
toolkit uses to choose buffer scale. The practical result is a client that draws
at 1× on a 1.5× desktop, and the shell's own revert-timer work (I.10) makes that
mismatch more visible, not less.

**I.07.3 — No clipboard is a policy, and the policy is not written down as a
user-facing limit.** Severity: medium. Evidence: T3. The crate docs are honest;
nothing a user sees says "copy and paste do not work between these windows". A
person who selects text in a foreign app and presses Ctrl+C gets silence, which
is indistinguishable from a bug.

---

## I.08 — The undocumented-versus-public boundary

### I.08.1 What the platforms do

Windows has a large documented surface and a larger undocumented one; macOS
draws the line with notarisation and rejects private API use; Linux has stable
protocols beside compositor-private ones. In every case the boundary is
*enforced* by a mechanism, not by convention.

### I.08.2 What LitricOS has

The boundary is explicit and unusually well argued — ADR-0019 (files daemon),
ADR-0021 (recovery), ADR-0024 (compositor) — but it is almost empty. The only
contracts a foreign process can use are the Wayland globals and the manifest.

### I.08.3 Findings

**I.08.1 — LFP/1 has no specification outside its own source.** Severity:
medium. Evidence: T3. The module documentation is good: connection policy,
at-most-once, one request and one reply. It is also inside the crate that
implements the client. A second implementation — a test harness, a recovery
tool, a different shell — would be reverse-engineering a private file. Anything
meant to be public needs a document outside the code that produces it.

**I.08.2 — The opposite failure from Windows, with the same outcome.**
Severity: high. Evidence: T3 vs S3851. Windows' problem is that real interfaces
exist and are undocumented. LitricOS' problem is that interfaces are documented
and do not exist. Both end in the same place: an application developer cannot
ship without guessing.

---

## I.09 — Comparison matrix

Legend: **●** public and implemented · **◐** public but partial · **○** absent ·
**–** not applicable.

| Surface | Windows 11 | macOS 27 | Linux (GNOME/KDE) | LitricOS |
|---|---|---|---|---|
| UI toolkit for third parties | ● WinUI/Fluent | ● AppKit | ● GTK 4 / Qt 6 | ○ |
| Shell extension points | ● handlers, share, search | ● extensions, Services | ● .desktop, D-Bus, portals | ◐ manifest only |
| Settings namespace | ● registry | ● defaults/plist | ● GSettings/KConfig | ◐ 8 shell keys |
| Consent prompt at escalation | ● UAC | ● TCC | ● polkit agent | ○ |
| Per-selection access | ● file pickers | ● powerbox | ● portals | ○ |
| Public notification API | ● WinRT (needs MSIX) | ● UN framework | ● D-Bus spec 1.3 | ○ private queue |
| IPC substrate | ● COM/RPC/pipes | ● XPC/Mach | ● D-Bus | ◐ three private protocols |
| Compositor protocol | – (DWM) | – (WindowServer) | ● Wayland core + xdg | ◐ 3 globals |
| Accessibility bus | ● UIA | ● AX API | ● AT-SPI | ○ |
| Audit of permission decisions | ◐ event log | ◐ log | ◐ journal | ● recorded, no UI |

**What the matrix says in one line:** LitricOS is at or above the reference on
*authority* (capability model, audit) and at or below it on *conversation*
(prompts, pickers, public APIs). The gap is not security. It is everything a
person and a developer need in order to use the security.

---

## I.10 — ADOPT / ADAPT / AVOID / INVESTIGATE

| # | Decision | Item | Why |
|---|---|---|---|
| 1 | **ADOPT** | DND decided by the shell, not the daemon | One answer to "should this interrupt"; the reference duplicates it per daemon |
| 2 | **ADOPT** | Unknown capability = hard error | A partly understood manifest is a security bug that looks like a feature |
| 3 | **ADOPT** | At-most-once mutations over IPC | The failure is already documented; keep it as a rule, not a habit |
| 4 | **ADOPT** | Revert window for display changes | Implemented this part: 15 s, keep/revert, no stacking, clock owned by the shell |
| 5 | **ADOPT** | Every capability decision recorded | The only path to a "which app read my files" page |
| 6 | **ADAPT** | polkit-style declared actions | Declare actions in the manifest so policy can name them, instead of granting a whole scope |
| 7 | **ADAPT** | portal-style per-selection access | A grant should end with the task; the file picker is the natural issuer |
| 8 | **AVOID** | A renderer reachable from a socket | notifyd already refuses; that refusal is the design rule for every daemon |
| 9 | **AVOID** | A single untyped settings blob | Registry-shaped stores are how preferences become unrecoverable |
| 10 | **INVESTIGATE** | Whether to publish a toolkit or a widget protocol | A toolkit is years; a protocol is a spec and a conformance suite |
| 11 | **INVESTIGATE** | `wl_output` global and buffer-scale negotiation | Without it every HiDPI client draws at the wrong size |
| 12 | **INVESTIGATE** | An `org.freedesktop.Notifications` compatibility shim | Cheapest possible way to make existing Linux apps notify correctly |

### I.10.1 The revert window, as implemented

The decision numbered 4 is not a proposal; it is in this tree.

| Property | Value | Where |
|---|---|---|
| Steps offered | 1.00×, 1.25×, 1.50× | `Shell::UI_SCALES` |
| Window | 15,000 ms | `Shell::REVERT_WINDOW_MS` |
| Applied | Immediately, before the countdown | `Shell::apply_ui_scale` |
| Second change while pending | Refused, not stacked | `Shell::cycle_ui_scale` |
| Kept | Value stays, clock stops, `tick` reports `Kept` forever, and the step is written | `revert::RevertWindow::keep`, `Shell::keep_pending_change` |
| Expired | Previous value restored on the shell's own clock | `Shell::advance_pending_change` |
| Drawn | Banner below the top bar: title, countdown, Keep, Revert | `views::draw_revert_banner` |
| Persistence | A kept step is stored as `ui_scale` and read back at startup; an expired or reverted step is never written | `prefstore` 9th key, `Shell::load_preferences` |

The last two rows are the part that had to change after this chapter was
written. The step a user keeps is now a stored preference, so 1.50× survives a
restart — but only the *interface* step is stored, because a display mode still
has no service to persist it to (Part 22, H.06). Writing happens on keep and
never on apply: a provisional value in the file would come back after a crash
as the size the user refused, which is the one outcome the window exists to
prevent.

---

## I.11 — Validation tests per decision

| Decision | Test that would validate it | State |
|---|---|---|
| 1. DND in the shell | A quiet-hours notification is queued, not dropped | Exists (notification queue tests) |
| 2. Unknown capability | Manifest with an unknown token is rejected whole | Exists (`litric-manifest`) |
| 3. At-most-once | A broken connection after the request line is not retried | Exists (`fsclient` policy) |
| 4. Revert window | Expiry restores; keep survives the clock; no stacking; the kept step round-trips through the store | **Added this part**: 3 unit tests + 1 journey; the journey now covers the record and the restart |
| 5. Decision audit | Allow and deny both recorded with injected time | Exists (`litric-caps`) |
| 6. Declared actions | Policy names an action, not a scope | **No test — not implemented** |
| 7. Per-selection access | A picker-issued grant ends with the task | **No test — not implemented** |
| 8. No socket renderer | notifyd exposes no drawing verb | Exists by construction (no API) |
| 9. Typed settings | A one-line corruption costs one key | Exists (`prefstore` tests) |
| 10. Toolkit vs protocol | A conformance suite for a widget protocol | **No test — undecided** |
| 11. `wl_output` | A client reads output scale and draws at 1.5× | **No test — global absent** |
| 12. Notification shim | A D-Bus client's notification appears in the centre | **No test — not implemented** |

Five of twelve decisions are validated today. Four are unimplemented and three
are undecided; none of those three can be closed by writing a test first.

---

## 1. What this chapter establishes

1. **LitricOS has authority without a conversation.** Deny-by-default, scoped
   grants, immediate revocation, expiry and a full audit trail are stronger than
   any reference default — and none of them is visible to the person they
   protect, because there is no prompt, no picker and no audit page.
2. **The public surface is the compositor and nothing else.** Three Wayland
   globals let an existing toolkit open a window; no toolkit, no notification
   API, no settings namespace and no extension point let it behave like an app
   on this desktop.
3. **The private surface is well argued and under-published.** ADRs explain the
   files daemon, recovery and the compositor; LFP/1 still has no specification
   outside its own module, so a second implementation would reverse-engineer it.
4. **The strongest reusable pattern is a refusal.** notifyd refuses to decide
   DND, refuses to decide duration and refuses to render. Those three refusals
   are why its promise — "a notification raised while the desktop is down still
   arrives" — is keepable.
5. **The revert window is the first display decision that can be taken back.**
   Fifteen seconds, two buttons, no stacking, and the old value restored by the
   shell's own clock. It is also the first implementation of a pattern Chapter H
   found missing everywhere (H.20: zero revert timers).

---

## 2. What this part does not establish

* **Whether a toolkit is the right next step.** I.10 leaves it undecided on
  purpose: a toolkit and a widget protocol are different five-year commitments,
  and this part measured the gap, not the choice.
* **Anything about TCC or XPC at API level.** Zero sources. The macOS column is
  structural.
* **That a Wayland client currently fails.** No foreign client was run against
  this compositor in this part; I.07.1 and I.07.2 are protocol-level gaps read
  from the dispatch table, not observed crashes.
* **That the revert window is pleasant to use.** It is tested for behaviour, not
  for legibility at 1.50× — which is the one scale where its own banner is the
  control at risk.

---

## 3. Sources

### New for this part

| ID | Title | Publisher | Status | Used for |
|---|---|---|---|---|
| S3850 | Desktop Notifications Specification, v1.3 (2024-08-18) | freedesktop.org | INSPECTED | I.05 scope, D-Bus protocol, urgency/hints/replace semantics |
| S3851 | What is MSIX? | Microsoft Learn | INSPECTED | Package identity as a precondition for notifications, background tasks, live tiles |
| S3852 | XDG Desktop Portal | freedesktop.org | INSPECTED | D-Bus portal interfaces, per-desktop backends, file access mediation |
| S3853 | The Wayland Protocol | freedesktop.org | INSPECTED | Core interface list; compositor on KMS + evdev |
| S3854 | User Account Control (Authorization) | Microsoft Learn | INSPECTED | Split token, consent, the "Never notify" trap |
| S3855 | polkit Reference Manual | freedesktop.org | INSPECTED | Mechanism/subject/authority, per-session agent, declared actions |

### Reused

S3079/S0112b (`defaults`), S1740 (WinUI compact sizing),
S3716 (GTK 4 Entry), S3042 (AppKit, WWDC26), S0130/S0131 (notification settings and DND),
S3852 (XDG Desktop Portal), S3845–S3849 (Chapter H).

---

## 4. Status changes

| Row | Before | After | Note |
|---|---|---|---|
| I | PENDING | PARTIAL | 11 of 11 measured; toolkit decision open |
| I.01 | PENDING | PARTIAL | No public toolkit; enum apps |
| I.02 | PENDING | PARTIAL | Manifest only |
| I.03 | PENDING | PARTIAL | 9 keys, no namespace |
| I.04 | PENDING | PARTIAL | Authority yes, prompt no |
| I.05 | PENDING | PARTIAL | Private queue |
| I.06 | PENDING | PARTIAL | Three protocols, no bus |
| I.07 | PENDING | PARTIAL | 3 globals, no wl_output |
| I.08 | PENDING | PARTIAL | Boundary argued, under-published |
| I.09 | PENDING | COMPLETE | Matrix above |
| I.10 | PENDING | COMPLETE | 12 decisions; 1 implemented this part |
| I.11 | PENDING | PARTIAL | 5 of 12 validated |
