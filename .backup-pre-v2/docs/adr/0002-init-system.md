# ADR-0002: Init System
**Status:** Accepted | **Date:** 2026-08-26

## Context
Iltric needs service supervision, cgroups integration, A/B update support, and sandboxing.

## Decision
Use systemd.

## Consequences
+ A/B update via systemd-sysupdate
+ OOM handling via systemd-oomd + PSI
+ Sandboxing directives built-in
+ UKI + boot counting support
+ Large ecosystem, well-maintained
- Resource overhead (~30-50 MB RAM)
- Complex dependency graph

## Revisit-if
- systemd overhead exceeds budget consistently
- A simpler init (e.g., runit) proves sufficient with A/B update hack
