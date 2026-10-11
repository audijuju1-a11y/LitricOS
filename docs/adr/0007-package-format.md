# ADR-0007: Package Format
**Status:** Accepted | **Date:** 2026-08-26

## Context
Litric needs a sandboxed app platform with atomic updates and a capability model.

## Decision
Use Flatpak + XDG portals as the primary app platform, bound to Litric's capability model.

## Consequences
+ Bubblewrap sandbox (per-app isolation)
+ Portal-mediated access (file picker, camera, etc.)
+ OSTree deduplication
+ Large existing ecosystem
+ Atomic install/update/rollback
- Disk overhead of runtimes (measured, published)
- Flatpak's sandbox may need Litric-specific capability bindings
- Vendor-centric aspects (Flathub)

## Alternatives considered
- Snap: vendor-centric backend, rejected
- AppImage: no sandbox, no update story, rejected
- Native packages only: too small an ecosystem

## Revisit-if
- Flatpak runtime disk overhead exceeds budget
- Litric's capability model cannot be cleanly bound to Flatpak portals
