# BUG-0009 — The trash `Path=` field carried a URI, not the original location

```
Bug ID        : BUG-0009
Severity      : S3 (MEDIUM) — interoperability, not loss: another tool reading
                Litric's trash restores the file to the wrong place
Class         : SPEC-CONFORMANCE (Freedesktop Trash 1.0, `Path=`)
Subsystem     : src/litric-files/src/trash.rs, src/litric-shell/src/fsbridge.rs
Status        : **IMPLEMENTED on audit branch 2026-10-15; tests added, not yet executed**
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

## Fix (implemented on `audit/comprehensive-2026-10-15`)

Both writers now percent-encode the absolute host path in `Path=`. The daemon
normalizes it back into a validated logical path beneath its configured root;
the shell performs the equivalent root-relative mapping. The restore handler
consumes the normalized logical path rather than assuming every entry is a URI.
Historical `file://` records remain readable for backward compatibility.

Regression tests now assert that newly written records use an absolute encoded
path and that legacy URI entries still restore in both the daemon and shell test
paths. These Rust tests have been added but have **not been executed in this
audit**. The patch remains unmerged until CI confirms the Rust tests and
formatting checks pass. End-to-end compatibility with `gio trash --list` still
requires a booted image and an external interoperability check.

## Verification

| Check | Result |
|---|---|
| Source inspection of both writers and both readers | **DONE** — the four sites and the two spellings are enumerated above |
| Specification text read | **DONE** — `https://specifications.freedesktop.org/trash/latest` (v1.0, 2014-01-02) |
| New regression tests | **ADDED, NOT RUN** — execution depends on the CI workflow added on this branch |
