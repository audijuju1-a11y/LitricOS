# The local backup service: snapshots, manifest, scheduling and verification

Status: implemented in §80 on 2026-10-30. This is a **local copy with a
checksummed manifest**, not a filesystem snapshot, a generation, an off-machine
backup or a restore feature. ADR-0021 keeps live data, snapshots, backups and
generations as separate concepts; this note says exactly which one exists.

## Why the daemon owns it

The state being copied is the application-root state `litric-sysd` already owns:

```text
<application-root>/applications/      desktop entries
<application-root>/litric/apps/       installed executables
```

A shell that copied those trees itself would be a second writer with a
different view of an install in progress. The backup therefore runs inside the
daemon, on the same `Sysd` mutex as install/update requests:

```text
litric-sysd serve loop
  ├─ scheduler thread ── BACKUP tick ── copy + manifest + prune
  └─ socket requests ─── BACKUP STATUS / LIST / CREATE / VERIFY
```

A snapshot cannot observe a half-applied `APP INSTALL` or update through this
process. It is **not** atomic against a program that writes its own files
outside the daemon; such a change either lands before or after a file is read,
and the manifest records what was actually read.

## Root and retention

Production configures:

```text
--backup-root <home>/.local/state/litric/backups
```

The default is derived from `--home`, **outside** the XDG data root being
copied. A root that is the source, or contains the source, is refused with
`ERR unsafe-source` before a directory is created. The root is configuration;
no request can name a path.

Snapshots live under `<root>/snapshots/<id>/`:

```text
snapshots/
  snap-00000000001800000000/
    manifest
    files/applications/...
    files/litric/apps/...
```

The id is `snap-` plus a 20-digit Unix timestamp, with a numeric suffix if two
snapshots have the same second. Retention is **seven valid snapshots**; the
oldest directory names are removed after a successful publish. Pruning refuses
links and non-directories and only removes names that pass the id validator.
Retention is a directory count, not a byte quota.

## Creation and publication

1. `symlink_metadata` the source; it must be a real directory, not a link.
2. Create `<snapshots>` with mode `0700` if needed.
3. Copy into a pid-tagged staging directory, `.snap-….tmp.<pid>`.
4. Walk `applications` and `litric/apps` in sorted order. Directories are
   created mode `0755`; regular files are copied with their permission bits and
   hashed while they are read. Links, sockets, devices and other special files
   are refused, not followed.
5. Refuse unsafe names (tab, newline, NUL), more than 100,000 files, or more
   than 8 GiB total. A path is at most 4,096 bytes.
6. Write `manifest` with `create_new`, mode `0600`, and `sync_all` it.
7. `rename` the staging directory onto the final id. If anything before the
   rename fails, only the staging directory this attempt created is removed.
8. Prune to seven.

The manifest is fsynced before the rename. The containing `snapshots` directory
is **not** fsynced, and there is no power-cut matrix for this path. A crash can
therefore lose the rename even though a completed snapshot's files are
internally consistent. This is a local best-effort copy, not the crash-consistency
claim §76 makes for the update state record.

The backup is outside the copied data root, but it is on the same machine and
normally the same filesystem. Deleting the user's state does not delete it;
deleting the user's home does. There is no encryption, remote copy, deduplication
or incremental transfer.

## Manifest format

The manifest is a UTF-8, tab-separated record, at most 4 MiB:

```text
litric-backup\t1
id\tsnap-00000000001800000000
created\t1800000000
source\tapplications,litric/apps
entry\tapplications/com.example.app.desktop\t142\t<64 hex>\t420
entry\tlitric/apps/com.example.app/app\t8192\t<64 hex>\t493
```

`entry` lines are sorted by relative path, with no duplicates. Each line records
the relative path, byte size, SHA-256 and permission bits (octal semantics in
decimal text). The parser requires the exact protocol/version, the expected
snapshot id, the exact source label, one `id`/`created`/`source` field, sorted
paths, valid relative components and a bounded total. A duplicate field, an
unknown line, an absolute path, `..`, a link, a bad digest or a manifest for
another id is `ERR invalid-manifest`; it is not repaired and not skipped.

The manifest enumerates **regular files**. Copied directories (including empty
ones) are not manifest entries, so verification does not compare empty directory
structure. It does detect every unlisted file and every listed file that is
missing.

## Verification: the delivered proof path

`BACKUP VERIFY <id>` is read-only and re-hashes every file through the same
production entrypoint used by the daemon:

1. Validate the id; no path component is accepted.
2. Inspect the snapshot directory with `symlink_metadata`; a link or non-directory
   is refused.
3. Parse the manifest and check that it names this id and the expected source.
4. For every entry, inspect the file (not a link, regular, exact size and mode),
   read it again, hash it, and compare the digest.
5. Walk the snapshot's `files/` tree and refuse any regular file that the
   manifest did not list.
6. Return `state=verified` and the id, created time, entry count and byte count.

A manifest with a valid format but a changed digest is **not** verified from its
own claims: step 4 fails with `ERR checksum`. A missing snapshot is
`ERR not-found`; a damaged manifest is `ERR invalid-manifest`; a symlink or
special file is `ERR unsafe-source`.

There is deliberately **no restore command**. ADR-0021 requires restore to be
conflict-aware and to be designed as its own operation; copying files back over
live application state without that design would be a data-loss path. Verification
proves the copy is complete and unchanged. It does not put it back.

## Schedule

`BackupService` defaults to 86,400 seconds (once a day). `serve` starts a named
background thread when the interval is non-zero:

* first tick immediately;
* wake at `min(interval, 60)` seconds and ask the service whether it is due;
* use the real system clock and the same `Sysd` mutex as requests;
* record a failure in `BACKUP STATUS` and log it, without stopping the socket.

The schedule is not persisted. Restarting `litric-sysd` makes the first tick
due immediately, which may create an extra snapshot; retention still caps the
directory count at seven. A zero interval means manual-only. Manual creation is
`BACKUP CREATE CONFIRM` and goes through the capability broker; the scheduler is
configuration, not a client request, so it does not ask the broker for consent.

## Protocol and authorization

```text
BACKUP STATUS
BACKUP LIST
BACKUP CREATE CONFIRM
BACKUP VERIFY <id>
```

`STATUS` is `state=disabled|uninitialized|ready|corrupt`, interval, next-due,
snapshot count, and optional last id/time/count/bytes/error. It never echoes the
root path. `LIST` is one line per valid snapshot, newest first. `CREATE` requires
the exact confirmation word and a `BackupControl` grant scoped to
`backup://create`, checked with live consent. Read-only status/list/verify do not
need a grant, matching the existing socket boundary (mode `0600`, local session).

| Failure | Stable token |
|---|---|
| no root configured | `backup-disabled` |
| malformed snapshot id | `bad-id` |
| snapshot/source absent | `not-found` |
| malformed or mismatched manifest | `invalid-manifest` |
| size/mode/digest changed | `checksum` |
| link or unsupported source/snapshot entry | `unsafe-source` |
| entry/byte/path limit | `limit` |
| id already taken and no suffix available | `exists` |
| filesystem operation failed | `backup-io` |

Errors carry a short sentence without a filesystem path. The daemon never
panics on a malformed backup; every refusal is a typed reply.

## Shell status

The Updates page asks `BACKUP STATUS` once when it opens or the user presses
**Check for changes**. `litric_shell::backupstatus` keeps only the state word, interval,
snapshot count, safe id/counts and a short error token. The root, free-form
reason and unknown fields are not displayed. The shell does not run the
scheduler, poll in the paint path, create a backup or claim a restore. The shell
e2e reads the same reply over the real socket after the automatic timer fires.

## Evidence

* `cargo test -p litric-sysd backup --lib` — unit and protocol tests for
  creation, re-hashing, retention, missing snapshot, corrupt/mismatched
  manifests, checksum refusal, links/bad ids and the interval.
* `cargo test -p litric-shell e2e_backup` — a real `litric-sysd` socket, a real
  timer-created snapshot, independent manifest checks, `LIST`, the shell status
  reader, verify after the originals are changed/deleted, tamper refusal, repair,
  and manual create.
* `frame_dump` scene `25-settings-updates` — the real Settings paint path with
  update and backup rows. The generated 25-frame review set is under
  `build/ui-review-2026-10-30/`, which is ignored build output.

## Limits

* No restore, conflict resolution or rollback to a snapshot. This is the main
  missing half of a backup feature.
* No off-machine, encrypted or authenticated backup; no remote storage.
* No filesystem/Btrfs snapshot and no atomic point-in-time view of files being
  written by another process.
* No byte quota, manual prune, retention policy UI or schedule persistence.
* No root-owned system-state backup; the source is the daemon user's application
  root.
* No booted-image proof in this batch. The e2e uses the real daemon binary path
  in-process, a real Unix socket and real files.
* §73 remains open; this adds status rows to its Updates page, not the full
  settings wave. C2/KMS remains separate.
