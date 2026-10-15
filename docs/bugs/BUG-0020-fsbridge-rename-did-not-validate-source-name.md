# BUG-0020 — FsBridge rename did not validate the source name

```
Bug ID        : BUG-0020
Severity      : S2 (HIGH) — filesystem boundary validation
Class         : PATH-VALIDATION / PATH-TRAVERSAL
Subsystem     : src/litric-shell/src/fsbridge.rs
Status        : IMPLEMENTED on audit branch 2026-10-15; regression test added, CI pending
First Observed: 2026-10-15, source review during comprehensive audit
```

## Expected

`FsBridge::rename(parent, old, new)` renames one direct child of `parent`.
Both `old` and `new` are file-name segments and must be validated before either
is joined to a host filesystem path.

## Actual

The method validated `new` but not `old`. It then evaluated
`dir.join(old)`. On Unix, joining an absolute path discards the prefix in
`dir`, and a name containing parent-directory components can resolve outside
the intended parent. This contradicts the method's documented contract and the
bridge module's logical-path confinement rules.

## Impact

A caller that supplies a malformed source name could cause a rename operation
to select a host path other than a direct child of the requested directory.
Whether a particular file can be moved still depends on the process's filesystem
permissions, but the input validation boundary was incomplete.

## Fix (implemented on `audit/comprehensive-2026-10-15`)

The method now calls `valid_name(old)` and `valid_name(new)` before resolving
the parent or constructing source/destination paths. Regression coverage checks
that parent traversal and absolute source names return `FsError::BadName`, and
that the original file remains present after the rejected calls.

## Verification

| Check | Result |
|---|---|
| Source-level validation | **DONE** |
| Regression test | **ADDED, CI pending** |
| Rust workspace test suite | **NOT YET GREEN** — see the audit report and workflow status |
