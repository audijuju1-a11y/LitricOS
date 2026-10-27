# BUG-0031 — The shell took any resolution, and the image flushed caches to look lean

```
Bug ID        : BUG-0031
Severity      : S2 (HIGH) — a 4K mode allocated past the shell's own cgroup
                ceiling, and the advertised idle-RAM figure came from a cache flush
Class         : MEMORY BUDGET (unbounded allocation; a measurement that measured
                the wrong thing)
Subsystem     : src/litric-shell/src/present.rs (`surface_budget`, `open_*`),
                src/litric-sysd/src/facts.rs (`read_processes`, `cpu_shares`),
                image/mkosi.extra/usr/lib/systemd/system/litric-drop-caches.service,
                image/mkosi.postinst.chroot, ci/check-memory-snapshot.py
First Observed: 2026-10-27, static performance audit of `main` (`b270aa3`)
Status        : FIXED (seven regression tests + a checker self-test)
```

## 1. The desktop accepted a mode it could not hold in memory

The compositor keeps three full-screen 32-bit canvases (wallpaper, scene,
cached glass backdrop) and the lock screen can add two more. At 1920×1080 that
is about 40 MiB; at 3840×2160 it is about 158 MiB **before** the text atlas, the
browser frame and allocator overhead — and `litric-shell.slice` has
`MemoryMax=140M`. Nothing checked the mode against the budget: the framebuffer
parser only rejected zero and unsupported depths, so a 4K machine was accepted
and then killed by its own cgroup at the first lock screen.

**Fix.** `present::surface_budget(width, height)` states one limit,
`MAX_SURFACE_PIXELS = 2560 × 1600` (4,096,000 px), and explains it: five
canvases at that size cost about 78 MiB, which leaves room inside 140 MiB. The
check runs in both output paths:

* **DRM**: after the helper negotiates a mode. A refusal under `auto` records
  `present=drm-skipped(<mode> is N pixels, past …)` and falls back to fbdev; a
  required `--presenter drm` is an error that names the mode, its pixel count,
  the budget and the cap.
* **fbdev**: before `Framebuffer::open`, so no allocation happens first. The
  error reaches the status file as `mode=headless(...)` rather than a boot that
  silently draws nothing.

**Tests** (`present::tests`): the limit itself is accepted and one pixel past it
is not; a 4K framebuffer is refused before opening; a 4K card mode falls back
with the reason in the status line; a required 4K card mode is an error.

## 2. The 320 MB figure was a cache flush, not a memory budget

`litric-drop-caches.service` ran `echo 3 > /proc/sys/vm/drop_caches` after boot
and the optimization document quoted the resulting `used` as "~295 MB idle
RAM". Page cache is reclaimable by the kernel; forcing it out makes every first
launch re-read from disk, and `used` then reads low without the system needing
less memory. Two of the three services enabled at boot (`litric-sysd`,
`litric-envd`, plus the newer `litric-notifyd`) had no line in the budget at all.

**Fix.**

* The unit and its enablement are deleted; `docs/OPTIMIZATION.md` (and the copy
  shipped in the image) now says the 295 MB figure was post-flush, that the unit
  was removed, and that the current image has no measured idle figure.
* `docs/PERF-budget.md` gained §3c: what a runtime measurement reads, and the
  exact commands. The TOTAL row now points at the runtime checker instead of
  implying the static checker measures memory.
* `image/mkosi.extra/usr/lib/litric/budget-snapshot` writes a
  `litric-budget/1` snapshot **inside a booted image**: `MemTotal`,
  `MemAvailable`, `MemFree`, `Cached`, `Slab`, swap, task count, and for each
  resident unit its cgroup `memory.current`/`memory.peak` and PSS from
  `smaps_rollup`.
* `ci/check-memory-snapshot.py` turns a snapshot into a verdict: used =
  `MemTotal − MemAvailable` against `--limit 320`; it fails when a resident
  unit is missing or stopped, when PSS or `memory.current` is absent, when
  `memory.peak` is below current, or when the protocol is unknown. Its
  `--self-test` proves each of those without a machine, and `ci/verify.sh` runs
  it.

**Tests:** `ci/check-memory-snapshot.py --self-test` — 8 checks, including "a
resident unit missing from the snapshot fails" and "over the budget fails".

## 3. Two bounded lists that were not bounded the way they claimed

* `read_processes` collected every PID on the host, sorted it and then
  truncated to 512. The intermediate list grew with the machine, not the limit.
  It now keeps the lowest `limit` PIDs in a max-heap while walking, and reports
  nothing for `limit == 0`.
* `cpu_shares` searched the earlier sample from the start for each current
  process — about 262,000 PID comparisons per sample at the 512-row cap. It now
  builds a `HashMap` of earlier ticks once and looks each process up.
* **Test:** `the_process_list_is_bounded_while_it_is_collected` (six PID
  directories, limit 3, the three lowest sorted ids).

## Verification

| Command | Result |
|---|---|
| `cargo test --offline -p litric-shell present::tests` | 11 passed |
| `cargo test --offline -p litric-sysd` | 176 passed |
| `python3 ci/check-memory-snapshot.py --self-test` | 8 passed |
| `python3 ci/check-memory-budget.py` | PASS (static policy) |
| `python3 ci/lint-doc-refs.py` | 729 citations resolve |
| `cargo test --offline --workspace` | see `docs/evidence/2026-10-27/performance-audit.log` |

## What is still not established

* **No booted image was measured.** The snapshot reader exists and is tested;
  running it needs the image this host cannot build (`mkosi`) or boot (`qemu`).
  Until then the 320 MB total remains a target over the written lines, not a
  whole-system measurement, and `litric-sysd`/`litric-envd`/`litric-notifyd`
  still have `unset` targets in the ledger.
* **A 4K display is refused, not supported.** The honest alternatives are a
  measured cgroup increase or tile/region caches; neither was chosen without a
  measurement to justify it.
* **The lock-screen caches still retain two full canvases after unlock.** They
  are deliberate speed/memory tradeoffs, and the 2560×1600 ceiling is what keeps
  them inside the slice.
* **The audit's other items are untouched:** the `Activity`/`Browser` model
  clones, per-damage-region view logic, the dock icon cache and the canvas
  allocation-failure abort.
