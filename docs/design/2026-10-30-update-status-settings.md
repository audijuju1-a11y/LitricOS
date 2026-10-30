# §78 — Update status in Settings

Date: 2026-10-30
Batch: §78, continuing the Settings work named by §73
Status: landed as a **read-only status page**. It is not an updater UI and does
not claim that an update can be downloaded, staged or committed from Settings.

## The gap

`litric-sysd` has answered `UPDATE STATUS 3` since §75, and §77 made the
system-written health evidence part of that answer. No shell surface consumed
it. A person could not see which slot was active, whether a rollback existed,
or whether the last boot had health evidence — even though the daemon was
already reporting all of it.

§73 is the Settings family wave. This batch adds one page to that wave; it does
not close the wave, and the remaining settings rows are unchanged.

## What changed

| File | Change |
|---|---|
| `src/litric-shell/src/updatestatus.rs` | typed `Status`, safe parser, one socket query |
| `src/litric-shell/src/settings.rs` | `Page::Updates` appended after Diagnostics, so existing sidebar indices do not move |
| `src/litric-shell/src/shell.rs` | cached status, `refresh_update_status`, category and refresh hotspots |
| `src/litric-shell/src/views.rs` | the Updates page: state, active generation, rollback, candidate, health, last error, schema |
| `src/litric-shell/src/a11y.rs` | the refresh control's name |
| `src/litric-shell/src/e2e.rs` | a real daemon, a real socket, a sidebar click and the real paint path |

The query is an action: it runs when the page is opened or the refresh row is
pressed. There is no timer, no I/O in the paint path, and no cached "up to
date" that can outlive the answer.

## Parsing rules

The shell understands only the fields it draws:

* `state` is one of `ready`, `uninitialized`, `disabled`, `busy`, `corrupt`,
  `incomplete` or `unavailable`. Anything else is unavailable.
* `active_slot` is `A` or `B`; a version is accepted only from the engine's
  small character set and is capped at 64 bytes.
* `health_evidence` is `healthy`, `booting` or `failed`; `health_matches` is
  `0` or `1`.
* `last_error` is reduced to its first short token. A path or free-form reason
  is not drawn.
* `root=` and unknown fields are ignored.

The page therefore cannot turn a corrupt record into an invented generation.
An absent update root is shown as **No update record**, and a socket that does
not answer as **Unavailable** with a short reason.

## What the page shows

| Row | Source |
|---|---|
| Update status | `state`, in plain words |
| Active system | `active_slot` and `active_version` |
| Rollback | `rollback_slot` and `rollback_version` |
| Staged update | `candidate_version` and `candidate_attempts` |
| Health evidence | `health_evidence` and `health_matches` |
| Last error | `last_error`, token only |
| Status protocol | `schema` |

The only control is **Check for changes**. It asks again; it does not perform
an update.

## Evidence

* `updatestatus` unit tests: six, covering a ready record, an uninitialized
  root, a typed refusal, ignored paths/unknown fields, a missing state, and an
  invalid version that must not be echoed.
* `journey_settings_updates_reads_the_real_daemon_status`: a real
  `litric-sysd` on a real Unix socket, an absent update root, a click on the
  Updates sidebar row, and the real canvas. It asserts `uninitialized`, no
  `Ready` row, and the refresh action querying the same socket.
* No test claims a booted update or a generation switch. The e2e daemon is a
  host process with a temporary root, not an image.

## Limits

* Read-only: no download, stage, commit or rollback control is drawn.
* No health probe is invented; the page displays only evidence the daemon
  wrote.
* No real booted image, systemd update unit or `/var/lib/litric/update` was
  exercised.
* §73 remains open. Auto-backup, Linux application installation and C2/KMS are
  separate batches; this page does not imply them.
* `SYSTEM-SCOPE` rows moved: **none**. This is a view over an existing daemon
  fact, not a new machine capability.
