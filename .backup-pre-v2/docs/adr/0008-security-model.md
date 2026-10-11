# ADR-0008: Security Model
**Status:** Accepted | **Date:** 2026-08-26

## Context
Iltric needs a layered security model that is deny-by-default without being unusable.

## Decision
Layered approach: Landlock + seccomp-bpf + bubblewrap + cgroups v2 + systemd sandboxing

## Consequences
+ Landlock: unprivileged, low overhead, primary per-app confinement
+ seccomp-bpf: syscall filtering per app profile
+ bubblewrap: namespace isolation (mount/pid/net/user)
+ cgroups v2: resource limits per app
+ systemd directives for system services
- No single mechanism is sufficient (defense-in-depth)
- Configuration complexity
- SELinux not included by default (available as high-security mode)

## Revisit-if
- Landlock cannot enforce required filesystem restrictions
- A kernel exploit bypasses all LSM layers
- SELinux becomes mandatory for compliance requirements
