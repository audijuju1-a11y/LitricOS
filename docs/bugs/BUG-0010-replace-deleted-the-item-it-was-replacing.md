# BUG-0010 — `Policy::Replace` deleted the item it was replacing before the new copy existed

```
Bug ID        : BUG-0010
Severity      : S2 (HIGH) — user data loss on a failed copy/move; the documented
                contract promised the opposite
Class         : DATA-INTEGRITY (failure semantics of a public operation)
Subsystem     : src/litric-files/src/ops.rs
First Observed: 2026-10-09, source inspection of `copy_item` / `move_item`
```

## Reproducer

```bash
sed -n '/pub fn copy_item/,/^    }/p' src/litric-files/src/ops.rs      # the staged path
sed -n '/Replace the existing item/,+2p' src/litric-files/src/lib.rs   # the promise
rustfmt --check --edition 2021 src/litric-files/src/ops.rs             # parses; not executed
```

Not run: the crate is not compiled in this mode (no cargo), so the Rust unit tests
added for it are **NOT VERIFIED**. The defect itself is provable by reading the
sequence of filesystem calls.

## Expected

`src/litric-files/src/lib.rs`, `Policy::Replace`:

> Replace the existing item, but only once the new one is complete.

## Actual

Both `copy_item` and `move_item` did the opposite:

```rust
let dest = dest_dir.join(&name);
if action == NameAction::Replace {
    self.remove_host(&dest)?;   // the old item is destroyed here
}
self.copy_into(&src, &dest, 0)?; // ... and the copy can still fail
```

Every failure between those two lines loses the destination for nothing: a full
disk, a permission error, a source tree containing a socket or FIFO
(`copy_into` returns `unsupported-archive` for anything that is not a file, a
directory or a link), or a tree deeper than `COPY_DEPTH_CAP`. `move_item` had the
same shape and additionally lost the occupant whenever the rename failed.

## Impact

The user asks to replace a file, the replace fails, and both the old and the new
content are gone. Nothing in the reply distinguishes this from an ordinary
failure, and no rollback exists anywhere in the layer.

## Root Cause

The code was written as "make room, then write". The contract in `lib.rs` was
written from the user's expectation ("replace once complete") and nothing
connected the two: no test covered a *failing* replace, only a successful one, and
a check that passes with the destination already deleted cannot notice the window.

## Fix

The new item is staged next to the destination and swapped in:

1. `stage_copy` copies to `.litric-staged-N` in the destination directory. On
   failure it removes its own staging entry and returns; the destination was
   never touched.
2. `install_staged` installs the completed item. One `rename(2)` is atomic for a
   file or a link. When the destination is a directory (which no single rename
   can replace) it is moved aside, the staged item renamed in, and the old tree
   removed only afterwards; if the second rename fails, the old item is renamed
   back and the error says where it is preserved if even that fails.
3. `move_staged` uses the same two steps for `MOVE`: a plain rename first (atomic
   replace), the staged copy across filesystems, and the source deleted only once
   the copy is in place.

Rejected alternatives: deleting first and hoping (the bug); copying into a
temporary *file* and renaming over (breaks for directories); refusing `Replace`
on directories (changes the product's behaviour for a case users rely on).

## Regression Test

`src/litric-files/src/ops.rs`, `mod tests` (added this session, **not executed**):

* `replace_keeps_the_destination_when_the_copy_fails` — the source tree contains a
  UNIX socket, which needs no `unsafe` code to create and which `copy_into`
  refuses; the destination must still hold its original bytes afterwards, and no
  `.litric-staged-*` entry may be left behind. This test fails against the old
  code.
* `replace_installs_the_new_item_and_removes_the_old` — the success path.
* `move_onto_an_occupied_directory_swaps_instead_of_deleting_first` — the
  directory case, where the old tree must survive until the swap.

## Verification

| Check | Result |
|---|---|
| `rustfmt --check --edition 2021 src/litric-files/src/ops.rs` | **PASS** (exit 0) — the file parses and matches the repo's formatting |
| `bash ci/verify.sh` (cargo hidden from `PATH`) | **PASS for the permitted subset** — 14 pass, 0 fail, 6 skipped; the Rust check is SKIPPED, so it says nothing about this change |
| Rust unit tests above | **NOT RUN** — compilation is prohibited in this task |

## Residual Risk

* The staging name is chosen by a check-then-act step, the same accepted window as
  the rest of the module (ADR-0019, "Residual risk accepted"). It narrows the
  failure mode to "the staging entry is not ours"; the *destination* is still only
  removed after the new item is complete.
* If `remove_host` on the vacated backup fails, a hidden `.litric-staged-N` entry
  is left behind and the operation still reports success. Deleting it later is the
  correct action; the operation cannot fail at that point without lying about
  what happened.
* Nothing here has been executed. A build phase must run the three tests above.
