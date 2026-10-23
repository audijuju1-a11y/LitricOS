# BUG-0009 — The trash `Path=` field carried a URI, not the original location

```
Bug ID        : BUG-0009
Severity      : S3 (MEDIUM) — interoperability, not loss: another tool reading
                Litric's trash restores the file to the wrong place
Class         : SPEC-CONFORMANCE (Freedesktop Trash 1.0, `Path=`)
Subsystem     : src/litric-files/src/{trash,server,trashinfo}.rs,
                src/litric-shell/src/fsbridge.rs
Status        : **FIXED 2026-10-15** — one format, one owner, legacy entries
                still restore; see "Fix" and "Verification" below
First Observed: 2026-10-09, source inspection against the specification text
```

## Reproducer

```bash
grep -n 'file://' src/litric-files/src/trash.rs src/litric-shell/src/fsbridge.rs
sed -n '/Path=/,+2p' docs/adr/0019-litric-files.md
```

## Expected

Trash Specification 1.0:

> The key "Path" contains the original location of the file/directory, as either
> an absolute pathname (starting with the slash character "/") or a relative
> pathname (starting with any other character). A relative pathname is to be from
> the directory in which the trash directory resides … it MUST not include a ".."
> directory.

`docs/adr/0019-litric-files.md` §4 states the intent: "`Path=` written as a
percent-encoded absolute path, which is what the specification requires and what
other tools read".

## Actual

Both writers stored `Path=file%3A///Documents/report.odt`, i.e. the decoded value
`file:///Documents/report.odt`. That is neither an absolute pathname nor a usable
relative one: it starts with `f`, so the specification's reader treats it as a
path relative to the trash directory's parent and restores the file to something
like `$XDG_DATA_HOME/file:/Documents/report.odt`. Note also that a relative
spelling of the true location (`../Documents/…` from `<root>/.local/share`) is
forbidden by the same clause, so only the absolute form is available.

Litric's own round trip worked, because both its reader and the shell's stripped
the `file://` prefix and parsed what remained as a logical path. The ADR's claim
about other tools was therefore false in the code, and the promise in the ADR is
the reason this is a bug rather than a style choice.

## Impact

A user who trashes a file with Litric and restores it with `gio trash`, KDE, or
any tool that reads the same trash directory gets the file in the wrong place (or
a refusal), and the original location is not the one recorded. No data is lost by
Litric itself.

## Root Cause

Two things, and the second is why the first was possible:

1. The logical path (`/Documents/x`) was available and the host path was not
   needed for Litric's own reader, so the URI form was chosen as
   "absolute-looking". The specification's definition of the field was not
   consulted at the point of writing it — the same failure class as BUG-0001's
   fabricated config and BUG-0008's unenforced limits: a document making a claim
   the code does not implement.
2. **The record's format had three owners.** `litric-files::trash` (the daemon's
   writer and reader), `litric-shell::fsbridge` (the shell's writer and reader,
   with its own private `pct_path`/`unpct_path` copies), and
   `litric_trash::TrashedItem::trashinfo`, which rendered
   `Path=user://Documents/x` — a storage URI, not a pathname, and a third
   instance of the same defect. Half-fixing the four I/O sites would have been
   worse than not fixing it at all, which is why the previous session deferred
   it.

## Fix

**One format, one owner.** The `.trashinfo` body and its percent-encoding moved
into a new module, `litric-files::trashinfo`, used by both writers. It is
deliberately **not** `#[cfg(unix)]`: `trash.rs`, `ops.rs` and `server.rs` are all
Unix-only, so on the Windows development host nothing about the trash format had
ever compiled, let alone been tested. String handling belongs where every host
can test it.

What the module provides:

| Item | Meaning |
|---|---|
| `record(host_path, deleted_iso)` | the whole `[Trash Info]` body; `Path=` is the **absolute host pathname**, percent-encoded |
| `pct_encode` / `pct_decode` | the field codec, keeping `/` literal as GLib does |
| `strip_root(host_root, absolute)` | maps an absolute location back to a logical path, on a **path boundary** — `/var/home/litric2/x` is not under `/var/home/litric` |
| `logical_location(host_root, recorded)` | current absolute form → logical; legacy `file://…` → logical; anything else → **unchanged**, so the caller's `LogicalPath::parse` refuses it as `invalid-name` |

Applied to the four I/O sites as the previous session's plan ordered:

1. `DiskTrash` now keeps its `host_root` (the daemon's reader needs to know where
   the absolute form begins).
2. `DiskTrash::put` writes `trashinfo::record(&host, …)` — the resolved absolute
   host path — and reports the **logical** path in `OnDiskItem::original_path`.
3. `DiskTrash::read_one` decodes with `pct_decode` and maps back with
   `logical_location`.
4. `server.rs::restore` parses `item.original_path` directly. The
   `strip_prefix("file://")` and its `corrupt-archive` branch are gone; a value
   the root cannot claim is now `invalid-name` from the parser.
5. `fsbridge.rs::trash` and `fsbridge.rs::trash_list` get the same two changes and
   its private `pct_path`/`unpct_path` copies are deleted.
6. `litric_trash::TrashedItem::trashinfo` is **removed**. It was unused, it
   emitted a storage URI under a name that promised the freedesktop file, and
   keeping a third formatter — even a corrected one — re-creates exactly the
   drift that produced this bug. The ledger stays what its header says it is:
   the layer that decides what a delete *means*, with no I/O.

**Legacy recovery is preserved.** `logical_location` handles the `file://` form
first-class, so an entry an older LitricOS wrote still lists with its real
location and still restores. That is a test, not a claim:
`a_legacy_uri_entry_still_restores_to_the_right_place` (format module),
`an_absolute_location_under_the_root_maps_back_to_a_logical_path`,
`a_record_written_in_the_legacy_uri_form_still_lists_and_restores` (shell), and
`trashlist_reports_the_original_logical_location_for_a_legacy_entry` (daemon
server).

**A refused restore beats a guessed one.** A `Path=` outside the root cannot be
mapped to a logical path, so it is passed through verbatim and the parser rejects
it. The alternative — inventing a location — is the bug this record is about.

## Regression tests added

| Where | Test | Runs on |
|---|---|---|
| `litric-files::trashinfo` | 12 tests: the body's exact bytes; `Path=` is absolute and never a URI; the root-is-`/` case; a trailing slash on the root; a sibling sharing the root's text prefix; the root itself; unchanged pass-through for unknown values; encode/decode inverses for awkward names | **any host** (ran here) |
| `litric-files::trash` | the on-disk record is the absolute location; a legacy-URI record still lists and restores; a `Path=` outside the root is listed but refused by the parser | Unix only |
| `litric-files::server` | `TRASHLIST` reports the logical original and never the word `file`; a legacy-URI entry restores | Unix only |
| `litric-shell::fsbridge` | the shell's record holds the absolute location and the shared reader maps it back; a legacy-URI record still lists with the right `restore_dir` and restores | Unix only |

`the_record_never_writes_a_uri` is the guard that would have failed before the
fix: the old writer's value was `file%3A///Documents/x`, which contains `file`
and is not the host path.

## Verification

| Check | Result |
|---|---|
| `cargo check --workspace --all-targets --target x86_64-unknown-linux-gnu` | **PASS (exit 0)** — this is what compiles the `cfg(unix)` code the Windows host otherwise skips, so both writers, both readers, `server.rs` and every new Unix test are type-checked |
| `cargo clippy --workspace --all-targets --target x86_64-unknown-linux-gnu` | **PASS** — no diagnostics (workspace lints: `unwrap_used`/`expect_used`/`panic`/`indexing_slicing` denied) |
| `cargo test --workspace` (host, `stable-x86_64-pc-windows-gnu`) | **PASS** — 1193 passed, 0 failed; 1181 before the change, +12 new `trashinfo` tests |
| The `cfg(unix)` end-to-end tests | **NOT EXECUTED HERE** — written and type-checked, but this host has no Unix runtime (no WSL, no container, no Linux target toolchain). They must run in a Linux CI stage |
| `gio trash --list` on a booted image showing the original path | **NOT RUN** — needs a booted image |

The honest summary: the format is proven on this host; the wiring that calls it is
compiled for Linux but its tests have not executed anywhere yet.

## Remaining work

* Run the `cfg(unix)` tests on a Linux host. Acceptance: the four lists above all
  pass, and `the_recorded_path_is_the_absolute_location_not_a_uri` fails if the
  `record` call is reverted to the old `file://` construction.
* `gio trash --list` / a KDE restore against a `litric-filesd`-written trash on a
  booted image — the acceptance the ADR promises other tools.
* Unify the two collision-name rules the audit's next-work table lists
  (`name.2.ext` in the daemon vs `name - Copy.ext` in the shell). Both are unique,
  so nothing breaks; it is a naming decision, not a safety one.
