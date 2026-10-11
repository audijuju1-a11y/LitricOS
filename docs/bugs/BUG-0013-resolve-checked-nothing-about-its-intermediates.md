# BUG-0013 — `Root::resolve` checked nothing about the components it walked through

```
Bug ID        : BUG-0013
Severity      : S3 (MEDIUM) — a documented confinement rule was not implemented;
                the failure surfaced as the wrong error for the wrong reason
Class         : CONFINEMENT / DOCUMENTATION-INTEGRITY (ADR-0019 §3)
Subsystem     : src/litric-files/src/ops.rs
First Observed: 2026-10-09, source inspection of `Root::resolve`
```

## Reproducer

```bash
sed -n '/pub fn resolve/,/Ok(current)/p' src/litric-files/src/ops.rs
grep -n 'Every intermediate component is checked' docs/adr/0019-litric-files.md
```

## Expected

ADR-0019 §3: "Every intermediate component is checked with `symlink_metadata` and
**must be a real directory**."

## Actual

The symlink half was implemented. The directory half was a no-op:

```rust
// Intermediate components must be directories. ...
let _ = meta.is_dir();
```

`let _ = …` evaluates and discards the result, so no component was ever required
to be a directory. A path such as `/Documents/note.txt/inner` — where `note.txt`
is a file — failed later, when `current.join(segment)` produced a path whose
`symlink_metadata` could not succeed, and the caller was told `not-found` with the
message "no such item". The stored path was not a boundary break (a symlink is
still refused, so the walking rule held), but the error named the wrong problem,
and the code and its comment disagreed about what was checked.

## Impact

Confined to error reporting and to the truthfulness of the ADR — a wrong
`not-found` for a path whose real problem is a file in the middle. It matters
because the same class of claim (a document describing a check the code does not
make) is what produced BUG-0001, BUG-0008 and the fabricated citation behind GATE
01 condition 7. This is the same defect in code rather than in a table.

## Fix

The walk is now `peekable()` and the rule is enforced where it is described:
a component with another one behind it must be a real directory
(`not-a-directory`), and the last component may be anything, which is what `STAT`,
`DELETE` and `RENAME` require. `peekable` rather than collecting the segments
keeps `resolve` allocation-free — it runs on every operation.

## Regression Test

Added to `src/litric-files/src/ops.rs`'s new test module (**not executed**):
`resolve_reports_a_file_standing_in_the_middle` asserts `not-a-directory` for
`/Documents/note.txt/inner` and that `/Documents/note.txt` itself still resolves.
Verified by inspection that no existing test asserted `not-found` for a
path with a file in the middle:

```bash
grep -rn "NotFound\|not-found" src/litric-files/src/*.rs src/litric-shell/src/*.rs
```

## Verification

| Check | Result |
|---|---|
| `rustfmt --check --edition 2021 src/litric-files/src/ops.rs` | **PASS** (exit 0) |
| `bash ci/verify.sh` (cargo hidden) | permitted subset **PASS**, Rust stage **SKIPPED** |
| Behaviour change (`not-found` → `not-a-directory`) | **NOT VERIFIED** — needs a build |

## Residual Risk

The error kind for this case changes, which a client that switches on `kind` could
notice. ADR-0019's error table is the contract, and `not-a-directory` is what it
assigns to "a directory was required"; no client code in this repository matches
on that token for this path.
