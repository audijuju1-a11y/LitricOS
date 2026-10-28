# LitricOS UI/UX Research — Part 22 of N

Chapter 42 (H, concluded) — Defaults, files, permissions, updates, recovery,
backups, troubleshooting, accessibility, shutdown, and the failure table the
whole chapter was for

| Field | Value |
|---|---|
| Document ID | LITRIC-RES-P22 |
| Revision | 1.0 |
| Research date (access date for all sources) | 2026-10-28 (Asia/Bangkok) |
| Scope of this part | Chapter 42 (`H`): `H.11`–`H.20` — default apps, file management, permissions, updates, recovery, backups, troubleshooting, accessibility change, shutdown, and the per-workflow failure/cancel/retry/rollback table |
| Batch | B10 second pass (`Part1` §2.2). Chapter `H` closes here |
| Companion files | `LitricOS Coverage Checklist.csv` (10 notes; `H` chapter note), `LitricOS Source Register.csv` (S3845–S3849), `src/litric-shell/src/lock.rs` and `shell.rs` (the `H03.01` repair) |
| Predecessors | `Part 21` (`H.01`–`H.10`, the five-state lens), `Part 11` (errors), `Part 15` (confirmation), `Part 18` (empty/loading states) |
| Resume point | `I` (technical integration) is the next unstarted chapter; `J` (audit) closes the register |

Status vocabulary: `COMPLETE`, `PARTIAL`, `NOT FOUND`, `INACCESSIBLE`, `NOT APPLICABLE`.
`PENDING` means "not researched yet". Evidence tiers (T1–T6), the `MEASURED`/`ESTIMATE`
tags and the confidence rubric are `Part1` §2.1 and used here without redefinition.

---

## 0. Scope, method, and the repair carried in this part

### 0.1 The last ten workflows

| Workflow | The question |
|---|---|
| `H.11` Default apps | does a file type or link type have an owner, and can it be changed? |
| `H.12` File management | can files be copied, moved, deleted and recovered, and can a long operation be stopped? |
| `H.13` Permissions | can an app be granted or refused a capability, and is the decision visible? |
| `H.14` Updates | can the OS update, and can a bad update be undone? |
| `H.15` Recovery | when the desktop cannot start, what does the machine offer? |
| `H.16` Backups | what protects the data when the disk or the user fails? |
| `H.17` Troubleshooting | how does a person find out why something is wrong? |
| `H.18` Accessibility change | can the interface be changed to fit a person, after setup? |
| `H.19` Shutdown | can the machine stop, and does it say what will be lost? |
| `H.20` Failure paths | for every workflow above, is there a named failure, a cancel, and a retry or rollback? |

### 0.2 The repair: `H03.01` is closed

`Part 21` found the unlock had **no attempt limit**: a refusal set a message and
left the field open, so the daemon could be asked an unlimited number of times
at whatever rate the helper allowed. This part ships the fix, because a finding
about a security defect that is only written down is a finding that ships.

The shape of the repair, all of it in `litric-shell`:

| Change | Where | Why that shape |
|---|---|---|
| A wait that doubles from 1 s to a 30 s cap | `LockState::note_failure` | the **first** refusal already waits; a zero-second first step leaves the first run of guesses as cheap as before |
| The countdown is part of the refusal sentence | `LockState::with_countdown` | a field that disappears with no reason reads as a broken screen, not a defence |
| The submit button is drawn disabled **and its hotspot is withheld** | `lock.rs` `entry` | a disabled-looking control that still takes a click is a lie the pointer can tell |
| `take_unlock` refuses while the field is closed | `shell.rs` | the keyboard path and the pointer path share one rule instead of two |
| Countdown advances on the shell clock | `shell.rs` `tick` | a redraw is what makes a countdown move; nothing else ticks a locked surface |
| A success or a new lock clears the run | `clear_failures` | the wait is per lock session, not a permanent penalty on a fat-fingered password |

Tests added: three unit tests in `lock.rs` (the schedule `1, 2, 4, 8, 16, 30, 30,
30`; expiry on the clock; clearing on success) and one end-to-end journey that
checks the field is closed after a refusal, that Enter during the wait produces
no `VerifyPassword` effect and does not consume the typed text, and that the
second refusal waits twice as long as the first.

### 0.3 What was measured (MEASURED from the repository)

| Artifact | Value |
|---|---|
| Settings pages | **13**; pages that change a workflow setting: **4** (Appearance, Power, Region, Accessibility) |
| File-service verbs | **26**, of which **5** are trash operations (`TRASH`, `TRASHLIST`, `RESTORE`, `PURGE`, `EMPTYTRASH`) |
| Cancel verbs in the file protocol | **0** |
| Conflict query (`CONFLICTS`) | **1** verb, before `COPY`/`MOVE` |
| Capability broker public methods | **15** (register, grant, revoke, revoke_all, check, check_with_consent, would_allow, grants_for, active_capabilities, audit_log, audit_for, purge_audit, and three constructors/queries) |
| Permission prompts in the shell | **0** |
| MIME / handler / association code | **0** |
| Default-apps page | **0** of 13 |
| Update actions in the daemon | **0** |
| Update surfaces in the shell | **0** |
| Backup components | **0** |
| Recovery states | **1** (`LifeState::Recovery`, never drawn) |
| Accessibility controls on the page | **1** (reduce motion), plus contrast in Appearance |
| Power commands | **3** (Suspend, Restart, Power Off) inside a **4**-entry menu |
| Power confirmation strings | **2** per command (prompt, consequence) |

---

## H.11 — Default apps

### H.11.1 What the platforms require

Windows has a dedicated page: **Settings → Apps → Default apps**, where a
default can be set by file extension or protocol (`.txt`, `http`) or per
application, and a browser's "Set default" is a single button ([S3845]). macOS
puts the same decision in Finder's Get Info → Open with → Change All, and GNOME
in Settings → Default Applications. All three name the same two directions:
**from the type** and **from the app**.

### H.11.2 What the shell has

Nothing. There is no MIME table, no `xdg-mime` call, no `handlers` list and no
"Open with" surface; the string `handler` appears in the tree only as a comment
about routing. Settings has **13 pages and none of them is Default apps**.

### H.11.3 Findings

- **`H11.01`** — No association model exists, so every file opens in the app the
  shell chooses. This is invisible until a person installs a second editor, at
  which point the OS has no way to let them choose.
- **`H11.02`** — There is no "Open with" either, which is the smaller half of the
  same feature and the half that needs no registry: a context menu entry listing
  installed apps that declare the type. `Part 14` established the context menu;
  this is a missing item in it, not a missing component.

Status: **PARTIAL** — shell measured; Windows inspected; macOS and GNOME at
guideline level.

---

## H.12 — File management

### H.12.1 What the platforms require

Explorer and Finder both promise four things beyond copy and move: a **trash**
that can restore, a **conflict decision** before overwriting, **progress** with a
**cancel** for long operations, and an **undo** for the last destructive action.

### H.12.2 What the shell has

A separate file service with **26 verbs**: `LIST`, `STAT`, `DU`, `MKDIR`,
`NEWFILE`, `WRITETEXT`, `IMPORT`, `RENAME`, `CONFLICTS`, `COPY`, `MOVE`,
`TRASH`, `TRASHLIST`, `RESTORE`, `PURGE`, `EMPTYTRASH`, `DELETE`, `COMPRESS`,
`EXTRACT`, `SEARCH`, `RECENT`, `VOLUMES`, `READTEXT`, plus `HELLO` and `QUIT`.
Long operations take a `progress` callback that is framed as `PROGRESS` lines;
the trash writes `.trashinfo` so `RESTORE` knows where an item came from; the
archive code implements zip/tar/gzip in-tree.

There is **no cancel verb**: the only way to stop a copy is to close the
connection, which the protocol treats as the peer going away rather than as a
decision. `DELETE` exists next to `TRASH`, so permanence is a UI choice.

### H.12.3 Findings

- **`H12.01`** — The trash is real: put, list, restore, purge and empty, with the
  original path recorded, and a journey test that restores after a trash. This is
  the only undo in the whole chapter.
- **`H12.02`** — **No cancel.** A 40 GB copy has progress and no stop button. The
  absence is structural (no verb), so a UI cannot add one without a protocol
  change.
- **`H12.03`** — `CONFLICTS` is asked *before* the transfer, which is the right
  order; whether the shell's Files window uses the answer for a replace/skip/keep
  both dialog is `Part 15`'s question, and the protocol at least makes it
  possible.
- **`H12.04`** — `DELETE` and `TRASH` are both exposed with no marking of which
  is permanent. A UI that binds one key to the wrong verb has no second line of
  defence.

Status: **PARTIAL** — protocol and archive measured; platform behaviour from
`Part 3`/`Part 6`.

---

## H.13 — Permissions grant and revoke

### H.13.1 What the platforms require

Windows asks through UAC for elevation and through capability prompts for
hardware; macOS asks through TCC with a purpose string per resource and lists
the result in Privacy & Security; Linux asks through polkit for system actions.
The common shape: **a named requester, a named resource, a decision, and a place
to see and revoke it later.**

### H.13.2 What the shell has

`litric-caps` is a broker with **15 public operations**: register an app, grant a
capability, revoke one or all, check (with and without a consent requirement),
a dry-run `would_allow`, list grants and active capabilities, and an **audit log**
with a bounded capacity and a purge. The manifest crate refuses an unknown
capability as a hard error so the install dialog cannot understate a request.

The shell has **no permission prompt**: capability checks happen in the broker's
library API, and the Privacy page reports "Per-site prompts are not connected
yet" and the persistent file-store path.

### H.13.3 Findings

- **`H13.01`** — The decision model is unusually complete for a system at this
  stage: grants carry a reason, an issuer and an expiry, and checks are audited.
  What is missing is the **surface**: no prompt, no per-app list, no revoke
  button.
- **`H13.02`** — The audit log is the right primitive for a "recent decisions"
  view and has no reader outside tests.
- **`H13.03`** — The Privacy page is honest that prompts are not connected, which
  is better than a toggle that changes nothing; it is still a page whose only
  control is a path.

Status: **PARTIAL** — broker measured; platform behaviour from `Part 8` and this
part's sources.

---

## H.14 — Updates

### H.14.1 What the platforms require

Windows Update downloads, stages, installs on restart, and offers **uninstall an
update** and **go back to the previous version** as recovery options ([S3847]).
macOS Software Update does the same. GNOME Software shows updates and asks for a
restart. The requirement is not the downloader; it is that an update is
**reversible** and that its state is visible.

### H.14.2 What the shell has

No update surface, no update action in the daemon protocol, and no update state
in Settings. The only "update" in the shell is the settings save state
(`Idle → Saving → Saved/Failed`), which is a preference write, not an OS update.

### H.14.3 Findings

- **`H14.01`** — Updates are **absent from the interface and from the protocol**.
  There is no version row, no "checking", no staged-update concept, and no
  "restart to finish".
- **`H14.02`** — Because there is no uninstall-an-update path, the first update
  mechanism that ships will ship without its rollback unless the state is
  designed with one. The recovery article's ordering — reinstall, uninstall the
  update, point-in-time restore, reset — is the shape to copy ([S3847]).

Status: **PARTIAL** — shell and daemon measured; Windows recovery article
inspected; the Windows Update article returned 404 and is registered as
`INACCESSIBLE` (S3849).

---

## H.15 — Recovery

### H.15.1 What the platforms require

Windows offers an ordered ladder, least disruptive first: a troubleshooter,
reinstall-in-place, uninstall the update, **point-in-time restore** (24H2+), then
reset keeping files, then reset removing everything; from WinRE the same ladder
minus the troubleshooter, and BitLocker recovery keys gate most of it ([S3847]).
Quick Machine Recovery detects repeated boot failures and applies a fix from
Windows Update with no user action.

### H.15.2 What the shell has

A `LifeState::Recovery { reason }` — described as "something failed badly enough
that only the recovery surface is shown" — and a daemon outcome
`SwitchOutcome::NeedsRecovery`. The state exists in the life-state model and in
its diagnostic string; **nothing draws it**, and no key, menu or boot path enters
it.

### H.15.3 Findings

- **`H15.01`** — Recovery is a **named state with no surface**. That is the
  inverse of `H.06`'s read-only page: there, a control was replaced by a
  sentence; here, a state that the shell already models produces no frame at all.
- **`H15.02`** — The ladder is missing entirely: no safe mode, no reinstall, no
  rollback, no point-in-time restore. The daemon can say "needs recovery" and the
  shell cannot offer the first rung.
- **`H15.03`** — Because `H.14` has no staged update, there is also nothing to
  uninstall, so the two most useful recovery rungs are blocked on the same
  missing state.

Status: **PARTIAL** — state measured; Windows inspected; macOS Recovery not
inspected.

---

## H.16 — Backups

### H.16.1 What the platforms require

Windows Backup signs in with a Microsoft account, syncs chosen user folders to
OneDrive, and optionally remembers installed apps, accessibility settings,
accounts and Wi-Fi passwords, personalisation and language preferences; restoring
is signing in on the new PC ([S3846]). Time Machine makes hourly backups for 24
hours, daily for a month and weekly beyond that, can encrypt the backup with a
password, shows progress in the menu bar, and restores any or all files to the
same or another Mac ([S3848]).

Both make three things explicit: **what is included**, **when it runs**, and
**how it is restored**.

### H.16.2 What the shell has

No backup subsystem, no schedule, no destination picker, no restore browser. The
only recovery of user data is the trash's `RESTORE`.

### H.16.3 Findings

- **`H16.01`** — Backups are absent. `H.12`'s trash is the only path back to a
  deleted file, and it covers exactly one operation.
- **`H16.02`** — The two platform articles agree on the minimum useful scope:
  user folders, settings, and a restore path that does not require the old
  machine. A first implementation that backs up only the persistent store would
  still meet that minimum; one that backs up nothing does not.

Status: **PARTIAL** — shell measured; Windows and macOS inspected.

---

## H.17 — Troubleshooting

### H.17.1 What the platforms require

Windows: Settings → System → Troubleshoot → Other troubleshooters runs a guided
fixer per problem class ([S3847]); macOS has Console and Disk Utility; GNOME has
Logs and a problem-reporting tool. The common requirement is that a person can
answer "what is wrong" without a terminal.

### H.17.2 What the shell has

A **Diagnostics** page of read-only rows (desktop state, facts source, shell and
daemon facts), an **Activity** window with a live process table and machine
meters, and a diagnostics overlay. `Part 11` measured the error component; this
part measures where errors are *looked at*.

### H.17.3 Findings

- **`H17.01`** — The diagnostics surfaces are real and honest: they report what
  the daemon said and what the kernel said, and distinguish "unavailable" from
  "none". This is the strongest read-only surface in the chapter.
- **`H17.02`** — There are **no guided troubleshooters and no log viewer**. A
  person can see that something is wrong and cannot follow a fix.
- **`H17.03`** — The Activity window is the natural home for the first
  troubleshooter ("this app is using all the CPU" → end task), and `Part 15`'s
  confirmation dialog already exists for the destructive half.

Status: **PARTIAL** — shell measured; Windows inspected; macOS not inspected.

---

## H.18 — Accessibility change

### H.18.1 What the platforms require

Windows: Accessibility settings for text size, contrast themes, magnifier,
Narrator and more, plus a per-app accessibility model exposed to assistive
technology. macOS: the same through System Settings → Accessibility. GNOME:
large text, high contrast, screen reader, and a keyboard-only path to every
control ([S3837]).

### H.18.2 What the shell has

One control on the Accessibility page — **Reduce motion** — and a sentence
pointing at Appearance for high-contrast themes, where the four themes live.
`Part 19` measured the contrast and found the text pairs pass; `Part 20` measured
the reduced-motion token and found it honoured everywhere. `Part 19` also found
**no accessibility tree at all**: the shell draws pixels, so a screen reader sees
nothing.

### H.18.3 Findings

- **`H18.01`** — Reduced motion is the one accessibility preference that is both
  settable and honoured across the whole system. That is a complete feature, and
  it is the model the others should copy.
- **`H18.02`** — Text size, magnification, and per-app accessibility are absent;
  contrast is a theme choice rather than an accessibility setting, which is
  defensible but leaves the page with one control.
- **`H18.03`** — The missing accessibility tree (`G.15`) is not a workflow defect,
  but it makes this workflow's ceiling low: a person who needs a screen reader
  cannot use the settings page that would help them.

Status: **PARTIAL** — shell measured; Windows and GNOME inspected; macOS at
guideline level.

---

## H.19 — Shutdown

### H.19.1 What the platforms require

Windows: power menu with Sleep, Shut down, Restart, and **Update and shut down**
when an update is staged; unsaved-work prompts come from applications. macOS:
Apple menu with Sleep, Restart, Shut Down, each with a confirmation only when
something would be lost. GNOME: a single power menu with the same three and a
confirmation that names the consequence.

### H.19.2 What the shell has

Three commands — `Suspend`, `Restart`, `PowerOff` — inside a four-entry menu
whose first entry is Lock. Each command carries **two strings**: a prompt
("Suspend the machine?") and a consequence ("The session stays open and this
machine can be woken again."). Choosing one opens a confirmation dialog. The
daemon executes through `systemctl` with a **30-second timeout**, and a failure
is reported as `exit` (with the code) or `spawn-failed`.

### H.19.3 Findings

- **`H19.01`** — The prompt-and-consequence pair is the best-written dialog
  content in the shell: it says what will happen, not "are you sure?". That is
  the pattern `Part 15` asked for everywhere else.
- **`H19.02`** — No **unsaved-work guard**: the shell has no way for an app to
  say "I have unsaved changes", so a restart can discard work silently.
- **`H19.03`** — No **Update and shut down**, because there is no update state
  (`H.14`). No hibernate either; the three commands are the whole vocabulary.
- **`H19.04`** — The 30-second timeout turns a hung `systemctl` into a named
  failure instead of a frozen screen, which is exactly the `H.20` property the
  rest of the chapter lacks.

Status: **PARTIAL** — shell and daemon measured; platform behaviour from
`Part 15` and this part's sources.

---

## H.20 — Failure, cancel, retry and rollback, per workflow

### H.20.1 The table

| Workflow | UI entry | Failure named in the UI | Cancel | Undo / retry | Revert timer |
|---|---|---|---|---|---|
| `H.01` Setup | yes (first run) | n/a | no | no | n/a |
| `H.02` Accounts | no | — | — | — | — |
| `H.03` Unlock | yes | yes (daemon's sentence + countdown) | yes (Esc) | retry, **now throttled** | n/a |
| `H.04` Launch | yes | **no** | n/a | no | n/a |
| `H.05` Windows | yes | n/a | yes (`cancel_drag`) | snap/maximise restore; close has none | n/a |
| `H.06` Display | no | n/a (read-only explains) | — | no | **none** |
| `H.07` Audio | yes | yes (`audio_note`) | n/a | manual | none |
| `H.08` Network | no | — | — | — | — |
| `H.09` Pairing | no | yes (why it cannot) | — | — | — |
| `H.10` Install | no | — | — | — | — |
| `H.11` Default apps | no | — | — | — | — |
| `H.12` Files | yes | yes (`ERR` line) | **no** | trash restore; delete permanent | none |
| `H.13` Permissions | no | — | — | revoke exists in the broker only | n/a |
| `H.14` Updates | no | — | — | **no uninstall-an-update** | none |
| `H.15` Recovery | **state only** | — | — | — | — |
| `H.16` Backups | no | — | — | — | — |
| `H.17` Troubleshoot | yes (diagnostics) | reports, does not diagnose | n/a | no | n/a |
| `H.18` Accessibility | yes | n/a | n/a | toggle back | n/a |
| `H.19` Shutdown | yes | yes (`exit`, `spawn-failed`) | dialog cancel | no | n/a |

### H.20.2 The counts

| Property | Workflows with it (of 20) |
|---|---|
| A UI entry point | **9** — `H.01`, `H.03`, `H.04`, `H.05`, `H.07`, `H.12`, `H.17`, `H.18`, `H.19` |
| A failure named in the interface | **5** — `H.03`, `H.07`, `H.09` (a state, not an error), `H.12`, `H.19` |
| A cancel | **2** — `H.03` (back), `H.05` (abandoned drag) |
| An undo or retry | **4** — `H.03` (retry), `H.05` (restore), `H.12` (trash restore), `H.13` (revoke, API only) |
| A revert timer for a change that can lose the display | **0** |

### H.20.3 Findings

- **`H20.01`** — The chapter's central defect — **an unthrottled unlock** — is
  fixed in this part (§0.2). Of everything measured across `H.01`–`H.20`, it was
  the only item where the absence was a security hole rather than a missing
  workflow.
- **`H20.02`** — **Cancel and undo are the two rarest properties**: two cancels
  and four undo/retry paths across twenty workflows, and half of the undo paths
  are the same trash. A shell can be honest about what it cannot do and still be
  unable to let a person stop it.
- **`H20.03`** — **No revert timer exists anywhere.** `H.06` is the workflow that
  needs it, and the failure-path table shows the gap is general: any setting that
  can make the machine unusable has no "keep this change?" step to borrow.
- **`H20.04`** — Silence is the default failure: `H.04` (launch), `H.12`
  (long copy), `H.17` (a problem with no guided fix) all leave the person
  without a next action. The surfaces that do it well — lock, power, Bluetooth,
  sound — share one property: **they name the thing that refused and what it
  means**, in the daemon's own words.
- **`H20.05`** — The pattern to keep: when a workflow cannot be implemented, the
  shell says so where the control would be (`H.06`, `H.08`, `H.09`, `H.13`). The
  pattern to stop: modelling a state and never drawing it (`H.15`).

---

## 1. What Chapter H says as a whole

Twenty workflows, ranked by what a person would hit first:

1. **`H03.01`** — the unthrottled unlock. **Closed here.**
2. **`H01.01` + `H08.01`** — setup without identity, network or time; a Network
   page that cannot connect.
3. **`H10.02` + `H02.01` + `H16.01`** — the three flows that make an OS
   installable, shareable and survivable are absent, each with its data model
   already written.
4. **`H04.01` + `H12.02`** — silent launch failure and uncancellable long
   operations: the two places where the machine is busy and says nothing.
5. **`H20.03`** — no revert timer, the prerequisite for ever shipping a display
   or network change safely.
6. **`H15.01`** — a recovery state that exists and never renders.

## 2. What this part does not establish

- **No browser or visual check.** The lock throttle was verified by unit and
  journey tests, not by watching the countdown on a display; this environment has
  no session to watch.
- **PAM delay is still unmeasured.** The shell now waits; whether the helper also
  delays is not established, and the finding is scoped to the shell.
- **The file protocol's cancel is "absent", not "impossible"**: a peer can close
  the connection, which the server treats as a disconnect. Whether a partial copy
  is cleaned up on disconnect was not measured.
- **macOS Recovery, Console, and Software Update** were not inspected; `H.15` and
  `H.17` compare Windows and the shell only.
- **`H.20`'s counts are counts of code paths and drawn controls**, not of user
  sessions: "no cancel" means no verb and no control, not that a person cannot
  kill the process from a terminal.

## 3. Sources

| ID | Source | Status | Used for |
|---|---|---|---|
| S3845 | [Change Default Apps in Windows](https://support.microsoft.com/en-us/windows/apps/change-default-apps-in-windows) — Microsoft Support | INSPECTED | `H.11` |
| S3846 | [Back up and restore with Windows Backup](https://support.microsoft.com/en-us/windows/experience/backup-recovery/back-up-and-restore-with-windows-backup) — Microsoft Support | INSPECTED | `H.16` |
| S3847 | [Recovery options in Windows](https://support.microsoft.com/en-us/windows/experience/backup-recovery/recovery-options-in-windows) — Microsoft Support | INSPECTED | `H.14`, `H.15`, `H.17` |
| S3848 | [Back up your Mac with Time Machine](https://support.apple.com/en-us/104984) — Apple Support, 2026-10-16 | INSPECTED | `H.16` |
| S3849 | [Update Windows](https://support.microsoft.com/en-us/windows/experience/windows-update/update-windows) — Microsoft Support | INACCESSIBLE (HTTP 404) | `H.14` (not used for a claim) |

Reused: `S3837` (GNOME accessibility), `S3839` (OOBE), `S3841` (uninstall),
`S3842` (Gatekeeper), `S3843` (Bluetooth pairing).

## 4. Status changes

| Item | Before | After | Why |
|---|---|---|---|
| `H.11` | PENDING | PARTIAL | no association model measured; Windows inspected |
| `H.12` | PENDING | PARTIAL | 26 verbs, trash measured, no cancel; platform from `Part 3` |
| `H.13` | PENDING | PARTIAL | broker measured, no prompts; platform from `Part 8` |
| `H.14` | PENDING | PARTIAL | absent in shell and daemon; update article inaccessible |
| `H.15` | PENDING | PARTIAL | state exists, no surface; Windows inspected |
| `H.16` | PENDING | PARTIAL | absent; Windows and macOS inspected |
| `H.17` | PENDING | PARTIAL | diagnostics measured; Windows inspected |
| `H.18` | PENDING | PARTIAL | one control measured; Windows and GNOME inspected |
| `H.19` | PENDING | PARTIAL | three commands measured; platform from `Part 15` |
| `H.20` | PENDING | PARTIAL | table complete for all 20; macOS columns missing |
| `H` (chapter) | PENDING | PARTIAL | all 20 workflows have a status and a note |
