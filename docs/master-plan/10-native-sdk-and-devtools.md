# 10 — Native SDK and developer tools

## SDK promise

The SDK makes the safe path ordinary: versioned public interfaces, capability
aware APIs, cancellable operations, explicit ownership, useful errors and stable
debugging symbols. Applications target Litric contracts, not shell internals,
private D-Bus names or physical storage paths.

## v1 SDK deliverables

- Rust bindings first, C ABI/header layer where a stable FFI boundary is needed,
  plus generated reference documentation and runnable examples.
- UI toolkit layer that emits Wayland/accessibility semantics, supports scales,
  themes, localisation and reduced motion without applications reimplementing
  them.
- Package builder/linter, manifest validator, local test runtime, signing tool,
  portal permission inspector, profiler, crash-symbol uploader and system
  diagnostics collector.
- Templates for GUI app, CLI app, background service, media app and test app.

## API requirements

Every SDK operation declares synchronous/asynchronous behaviour, cancellation,
timeout, retryability, resource ownership, thread-safety, capability needed,
error class and fallback when hardware/service support is absent. No API may
hide network access, retain a file grant forever, or opt an app into cloud AI.

## Tooling quality gates

- Build an example app against the oldest supported minor SDK, then run it on
  the newest runtime.
- Generate ABI/API diff reports for every release; additive changes are tested
  against old clients and unknown feature handling.
- Ensure debugger/profiler access is user-approved and cannot bypass sandbox or
  read another app's memory in production.
- Supply reproducible sample builds and source-to-package provenance.

## Deferred decision

A separate Litric executable/package format is `POST`. It requires an ADR that
proves a concrete limitation of Flatpak/runtime integration, a migration path,
toolchain support, update security and ecosystem benefit larger than its cost.

