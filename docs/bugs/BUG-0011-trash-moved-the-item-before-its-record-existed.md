# BUG-0011 — The trash moved the item before its record existed, and never reserved the name

```
Bug ID        : BUG-0011
Severity      : S2 (HIGH) — a trashing that reports failure can still remove the
                item from its place, and a later same-name trash could overwrite
                an earlier trashed copy
Class         : DATA-INTEGRITY / SPEC-CONFORMANCE (Freedesktop Trash 1.0)
Subsystem     : src/litric-files/src/trash.rs
First Observed: 2026-10-09, source inspection against the specification text
```

## Reproducer

Source inspection (no build, no boot):

```bash
sed -n '/pub fn put/,/fn reserve_info/p' src/litric-files/src/trash.rs
grep -n "free_name_for\|fs::write(self.info_dir" src/litric-files/src/trash.rs
```

## Expected

Freedesktop Trash Specification 1.0, *Contents of a trash directory*:

> When trashing a file or directory, the implementation MUST create the
> corresponding file in `$trash/info` **first**. Moreover, it MUST try to do this
> in an atomic fashion, so that if two processes try to trash files with the same
> filename this will result in two different trash files. On Unix-line systems
> this is done by generating a filename, and then opening with O_EXCL.

and, on the `files/` side:

> Even if a file with the same name and location gets trashed many times, each
> subsequent trashing **must not overwrite** a previous copy.

## Actual

`DiskTrash::put` did, in this order: pick a name by checking
`info/<name>.trashinfo` with `exists()`, `fs::rename` the item into
`files/<name>`, then `fs::write` the record. Three consequences:

1. **Record after the move.** A failure writing the record (full disk, permission,
   an interrupted process) leaves the item inside `files/` with no readable
   record. `list()`/`read_one()` skip entries without a `Path=`, so the item
   vanishes from the trash window and cannot be restored from it — while the
   original location is empty and the reply was an error. The user's data is not
   destroyed, but the state on disk no longer matches either the report or the UI.
2. **No exclusive reservation.** The `exists()` check and the `fs::write` that
   follows are not atomic, and `fs::write` truncates. Two processes (the socket is
   `Accept=yes`, so one connection is one process) trashing the same name can pick
   the same name and overwrite each other's `files/` entry — the exact case the
   specification forbids. `litric-shell`'s own trash writer in
   `src/litric-shell/src/fsbridge.rs` has the same shape, plus `rename` into
   `files/` replacing an existing entry.
3. **Orphan reuse.** Because the name check only looked at `info/`, a leftover
   item in `files/` from an earlier failure (1) was invisible to the check, and
   the next trashing of the same name renamed **onto** it. Single-process,
   no concurrency needed: the earlier copy was silently destroyed.

## Root Cause

The natural order is "move it, then describe it". The specification requires the
reverse because the record is what makes the name unique and what makes a crash
recoverable. Nothing in the module tested a failure of the record write, so the
ordering was never challenged.

## Fix

`put` now builds the record first and reserves the name by creating it with
`create_new` (`O_EXCL`): a name already claimed fails that syscall and the next
candidate is tried, so two trashing processes cannot share an entry. A candidate
counts as taken when **either** half exists — `info/<name>.trashinfo` or
`files/<name>` — which closes the orphan-reuse path. If the move then fails, the
reservation is removed again and the error is returned, so the reply matches the
disk. `write_all` is where a full disk or a permission error surfaces; the record
is removed if it fails, because a `File` drop cannot report a close error.

`restore` now applies the same one-segment name rule `get` and `purge` already
applied. The server resolves the name through `get` before calling it, so this is
a second independent check rather than the only one — the module should not depend
on its caller for its own invariant.

Rejected alternative: keeping move-first and adding a rollback (rename back on a
record failure). It works, but it is strictly more code in the failure path and
still leaves the exclusive-reservation hole that the spec's `O_EXCL` rule exists
to close.

## Regression Test

Rust unit tests for this module are **NOT VERIFIED** (no compilation in this
mode). The behavioural claims above are provable from the call order, and the
specification clauses are quoted in place. A build phase should add, at minimum:
a trashing that fails before the move leaves `files/` and `info/` unchanged; an
orphan in `files/<name>` makes that name unavailable to the next trashing of the
same name; two `put` calls with the same base name produce two entries.

## Verification

| Check | Result |
|---|---|
| `rustfmt --check --edition 2021 src/litric-files/src/trash.rs` | **PASS** (exit 0) |
| `bash ci/verify.sh` (cargo hidden) | permitted subset **PASS**, Rust stage **SKIPPED** |
| Any Rust test execution | **NOT RUN** — prohibited |

## The second implementation (same session)

`src/litric-shell/src/fsbridge.rs` writes the same on-disk trash from the shell
side, and it had the same three defects: the name was taken with an `exists()`
check, the item was moved before the record was written, and `trash_list` showed a
record whose item was missing. It now uses the same shape as the daemon —
`reserve_trash_name` creates the record with `create_new` (`O_EXCL`), treats a name
as taken when either `info/<n>.trashinfo` or `files/<n>` exists, rolls the
reservation back if the move fails, and refuses to overwrite a half-written entry.
`trash_list` skips a record with no item behind it, which is what the daemon's
reader already did.

Both implementations keep writing `Path=file://<logical>`; the format change is
BUG-0009 and is deliberately separate, because it has to land in all four sites at
once.

## Residual Risk

* Neither this module's tests nor the shell bridge's tests were executed: no
  compilation is permitted in this mode. The claims above are provable from the
  call order and from the specification clause quoted at the top.
* `reserve_trash_name` tries up to 10 000 candidates and then fails with `Io`
  rather than inventing a name; the daemon has the same bound.
* The specification's `directorysizes` cache is not implemented; it is a SHOULD,
  and nothing here reads it.
* `Path=` is still `file://<logical path>` rather than the original location the
  specification asks for — **BUG-0009**, deliberately not changed in this session.
