# 15 — Power, thermal, and resource engine

## Objective

Treat CPU, memory, storage, GPU, network, display and thermal headroom as one
policy system. Preserve foreground responsiveness and hardware safety before
throughput, background completion, benchmark scores or compatibility guests.

## Resource classes

| Class | Examples | Policy |
|---|---|---|
| interactive | shell, focused app | latency protection and bounded memory |
| realtime | audio, input | no blocking, priority only with strict limits |
| foreground compute | compile/render | work-conserving, user-visible control |
| background | indexer, sync | PSI-aware throttle/pause on battery |
| batch | maintenance, scan | scheduled, cancellable, low I/O priority |
| guest | Windows/Linux VM | host-enforced hard memory/CPU/I/O budget |

## Control loop

Observe cgroup CPU/memory/I/O PSI, active window, battery, charger, thermal
sensors, display state and device capabilities. Decide a bounded policy action:
weight, `MemoryHigh`, I/O limit, GPU preference, pause, resume, or notify. Log
the reason and make manual override visible. Never kill or reclaim active user
work merely to hit a vanity idle target.

## Power rules

- Socket/dbus activation, timer coalescing and idle inhibition are explicit.
- Background indexing/scanning pauses on battery or foreground pressure and
  resumes only when the trigger is satisfied.
- Runtime PM uses device-specific autosuspend with HCL evidence; avoid rapid
  low-power/full-power oscillation.
- Thermal protection overrides performance/game mode; a user may choose quieter
  or faster only within safe boundaries.

## Tests

Inject cgroup pressure, memory exhaustion, slow I/O, high CPU/GPU temperature,
low battery, charger changes, guest overuse and blocked background jobs. Verify
foreground p99, no priority inversion, correct recovery after pressure clears,
and bounded audit/log overhead.

