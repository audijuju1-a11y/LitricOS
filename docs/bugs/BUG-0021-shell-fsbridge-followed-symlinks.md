# BUG-0021 — Shell filesystem bridge did not enforce the no-follow contract

```
Bug ID        : BUG-0021
Severity      : S2 (HIGH) — filesystem boundary
Class         : SYMLINK-TRAVERSAL / PATH-CONFINEMENT
Subsystem     : src/litric-shell/src/fsbridge.rs
Status        : IMPLEMENTED on audit branch 2026-10-15; regression tests added, CI pending
First Observed: 2026-10-15, comprehensive source review
```

## Expected

ADR-0019 §3 requires every intermediate path component to be a real directory and
requires reads to treat symlinks as links, not follow them. The shell bridge
module header makes the same promise.

## Actual

`host_path()` originally joined logical path segments after checking only
`..`. Existing symlinked directory components were not rejected. Read methods
then opened the resulting path by name, including `read_text()` reading after
metadata inspection; the Trash directory setup used `create_dir_all` on
`.local/share/Trash`, which follows existing symlink components.

## Impact

Operations could resolve through an existing symlink outside the intended
logical directory, contrary to the stated filesystem confinement contract. The
practical impact depends on the bridge's root and process permissions. This is a
confirmed contract violation, not a claim that a remote attacker can exploit it.

## Fix (implemented on `audit/comprehensive-2026-10-15`)

The bridge now:
- rejects `.`/`..` segments and refuses existing symlink or non-directory
  intermediate components;
- checks that parent/destination directory arguments name real directories;
- refuses final symlinks in text reads and archive reads, and lists children using
  non-following metadata;
- checks or creates each component of the Trash directory tree individually,
  refusing symlinks instead of using `create_dir_all`.

Final symlinks can still be moved, trashed, renamed or unlinked as link objects.
That behavior does not require following the target.

## Residual risk

Check-then-use remains vulnerable to a same-user TOCTOU race. Closing that race
requires a descriptor-relative implementation (for example, directory file
descriptors with `openat2`/beneath/no-symlink constraints) and its own ADR,
reviewed platform boundary and adversarial tests.

## Verification

| Check | Result |
|---|---|
| Source review | **DONE** |
| Regression tests | **ADDED, CI pending** |
| Race-resistant descriptor-relative implementation | **NOT IMPLEMENTED** |
