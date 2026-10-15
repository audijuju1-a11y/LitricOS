# BUG-0022 — Trash purge left stale `.trashinfo` metadata

```
Bug ID        : BUG-0022
Severity      : S3 (MEDIUM) — disk-state consistency
Class         : CLEANUP / TRASH-STATE-DRIFT
Subsystem     : src/litric-shell/src/fsbridge.rs
Status        : IMPLEMENTED on audit branch 2026-10-15; regression assertion added, CI pending
First Observed: 2026-10-15, review of purge lifecycle
```

## Expected

Permanent purge removes both the payload in `Trash/files/` and its metadata
record in `Trash/info/`.

## Actual

`trash_purge()` delegated only to `delete("/.local/share/Trash/files/<name>")`.
The payload was deleted, but the matching `.trashinfo` file remained. The
listing code hid the orphan because it shows entries only when their payload
exists, so a test asserting only an empty visible list did not detect the disk
residue.

## Impact

Orphaned records accumulate and can influence name collision/reservation
behavior on later deletions. They also make the on-disk state disagree with the
visible Trash list.

## Fix (implemented on `audit/comprehensive-2026-10-15`)

Purge now verifies the Trash directory tree, removes the payload without
following a symlink, then deletes the matching metadata. Missing metadata is
tolerated; other metadata cleanup failures are returned because the payload has
already been removed and the operation is partial. The regression test verifies
the actual metadata path is gone after purge.

## Verification

| Check | Result |
|---|---|
| Source review | **DONE** |
| Metadata cleanup regression assertion | **ADDED, CI pending** |
