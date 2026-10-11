# BUG-0009 — The trash `Path=` field carried a URI, not the original location

```
Bug ID        : BUG-0009
Severity      : S3 (MEDIUM) — interoperability, not loss: another tool reading
                Litric's trash restores the file to the wrong place
Class         : SPEC-CONFORMANCE (Freedesktop Trash 1.0, `Path=`)
Subsystem     : src/litric-files/src/trash.rs, src/litric-shell/src/fsbridge.rs
Status        : **OPEN — deliberately not fixed in this session** (see Fix)
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

Both writers store `Path=file%3A///Documents/report.odt`, i.e. the decoded value
`file:///Documents/report.odt`. That is neither an absolute pathname nor a usable
relative one: it starts with `f`, so the specification's reader treats it as a
path relative to the trash directory's parent and restores the file to something
like `$XDG_DATA_HOME/file:/Documents/report.odt`. Note also that a relative
spelling of the true location (`../Documents/…` from `<root>/.local/share`) is
forbidden by the same clause, so only the absolute form is available.

Litric's own round trip works, because both its reader and the shell's strip the
`file://` prefix and parse what remains as a logical path. The ADR's claim about
other tools is therefore false in the code, and the promise in the ADR is the
reason this is a bug rather than a style choice.

## Impact

A user who trashes a file with Litric and restores it with `gio trash`, KDE, or
any tool that reads the same trash directory gets the file in the wrong place (or
a refusal), and the original location is not the one recorded. No data is lost by
Litric itself.

## Root Cause

The logical path (`/Documents/x`) was available and the host path was not needed
for Litric's own reader, so the URI form was chosen as "absolute-looking". The
specification's definition of the field was not consulted at the point of writing
it — the same failure class as BUG-0001's fabricated config and BUG-0008's
unenforced limits: a document making a claim the code does not implement.

## Fix (NOT APPLIED — and why)

The correct value is the absolute host path of the original location,
percent-encoded, and the readers must map it back to a logical path (with a
fallback for entries already written in the URI form).

That change touches four sites in two crates — the daemon's writer and reader, the
shell's writer and reader — plus `src/litric-files/src/server.rs`, which currently
strips `file://` before parsing. Half-fixing it is worse than not fixing it: the
two writers share one on-disk layout, so a daemon writing absolute paths while the
shell still expects `file://` would make the shell restore daemon-trashed files
into `/` (its `unwrap_or_else` default). Nothing in this mode can compile either
crate or run either test suite, so the change is specified here for one atomic
patch rather than landed blind.

Ordered plan for that patch:

1. `DiskTrash` keeps the host root (`DiskTrash::open` already receives it) so the
   reader can strip it. `read_one` returns the logical path, as today.
2. Writer: `Path=` = `pct_path(<absolute host path>)`.
3. Reader: if the value starts with the root, strip it → logical path; else if it
   starts with `file://`, strip that (legacy entries) → logical path; else keep
   the raw value, which then fails `LogicalPath::parse` and surfaces as
   `invalid-name`.
4. `server.rs::restore` parses `item.original_path` directly; the `file://`
   strip and its `corrupt-archive` branch go away.
5. `fsbridge.rs::trash` and `fsbridge.rs::trash_list` get the same two changes.
6. Acceptance: a Trash round trip in both layers, one legacy `.trashinfo` fixture
   written in the URI form still restoring correctly, and `gio trash --list` on a
   booted image showing the original path (the last one needs a booted image).

## Verification

| Check | Result |
|---|---|
| Source inspection of both writers and both readers | **DONE** — the four sites and the two spellings are enumerated above |
| Specification text read | **DONE** — `https://specifications.freedesktop.org/trash/latest` (v1.0, 2014-01-02) |
| Any behavioural test | **NOT RUN** — the fix is not applied, and neither crate compiles in this mode |
