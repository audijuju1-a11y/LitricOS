# LITRIC — RESOURCE BUDGET (v2, AGGRESSIVE)

> **STATUS OF EVERY NUMBER ON THIS PAGE: `TARGET`, not `MEASURED`.**
> A target becomes a measurement only when it appears in `docs/EVIDENCE.md` with
> tool, exact command, environment, n, mean and stddev (Appendix §3). Until then
> it is a budget line that blocks a gate, not a claim we may publish.

The budget is the mechanism that keeps LitricOS light. Good intentions are not a
mechanism. `tests/budget/` enforces every line below in CI; exceeding a line
**blocks that phase's gate**, with no "close enough".

---

## 1. WHY THE v1 BUDGET WAS NOT AGGRESSIVE ENOUGH

v1 targeted ≤600 MB desktop idle. That is roughly XFCE-class and only ~2× better
than KDE Plasma 6. It does not earn the word "lightweight" against a 2026 field.

The v2 budget is built bottom-up from what each component *actually needs*, not
top-down from a round number. The single change that makes it reachable is
**ADR-0009: the compositor and the shell are one process, with no GTK/Qt/Electron
anywhere in the graphical path.** In a conventional stack, a compositor, a panel,
a launcher, a notification daemon and a settings app are five processes that each
map a full widget toolkit — GTK4 plus its dependency closure is tens of MB of
resident, largely unshared pages *per process*. Collapsing that into one binary
with a purpose-built renderer is worth more than every sysctl in this document
put together.

---

## 2. PROFILE LADDER

LitricOS ships four build profiles. Each has its own budget and its own gate.

| Profile | Contents | Idle RAM target | Idle CPU target | Boot target |
|---|---|---:|---:|---:|
| `nano` | kernel + init + serial console. CI/embedded. | **≤ 55 MB** | ≤ 0.10 % | ≤ 1.2 s |
| `core` | nano + udev + storage + network + resolved | **≤ 95 MB** | ≤ 0.20 % | ≤ 2.0 s |
| `desktop` | core + shell + audio + defender + portals | **≤ 320 MB** | ≤ 0.50 % | ≤ 3.5 s |
| `gaming` | desktop + gamescope + staged Proton runtime | **≤ 420 MB** | ≤ 0.50 % | ≤ 3.5 s |

`desktop` is the headline profile. **≤ 320 MB** is the number LitricOS is held to.

### Comparison context (external measurements — re-verify before quoting)

Sourced from `15-APPENDIX-TARGETS-MEASUREMENT.md` §2, which sourced them
elsewhere. **These are other people's numbers.** They set the bar; they are not
our results, and they must never be presented as ours.

| System | Idle RAM (as cited) | LitricOS `desktop` target ratio |
|---|---:|---:|
| Windows 11, clean idle | 2.2–3.3 GB | **7–10× lighter** |
| GNOME / KDE Plasma 6 | 1–2 GB | 3–6× lighter |
| XFCE / LXQt | 500–700 MB | 1.6–2.2× lighter |
| Sway / labwc / Hyprland | 300–500 MB | comparable to slightly lighter — but with a full shell, defender and portals, which those do not include |

The honest framing: against Windows the advantage is large and structural.
Against a bare tiling WM the advantage is *features per megabyte*, not raw
megabytes, and we will say it that way.

---

## 3. `desktop` PROFILE LEDGER (the hard budget)

Each line is either a configured `MemoryMax=`/`MemoryHigh=` on a cgroup v2
slice or unit, or an explicit `TARGET (not enforced: ...)` recording that the
limit this budget asks for is **not configured anywhere yet**.
`ci/check-memory-budget.py` reads this table and the units under
`image/mkosi.extra/`, and fails when a cell claims enforcement that does not
exist, when a shipped cap is missing from the table, or when a divergence is not
acknowledged. Cell grammar:

| Cell contains | Meaning |
|---|---|
| `MemoryMax=64M` | A claim. Must be configured for the unit named, and must be the value that actually applies to it. |
| `TARGET (not enforced: ...)` | An acknowledged gap. Printed by the checker on every run, never silent. |
| `... [ACK: reason — BUG-0000]` | Same, for a claim whose configured and effective values differ. The reason must name a review condition. |
| `dbus-activated`, `boot-time measurement`, `—` | Outside unit-file scope. |

A limit is not "configured" because a slice somewhere in the image sets it: it
is configured for a unit when the unit or one of its parent slices sets it,
since systemd's `EffectiveMemoryMax=` is the most stringent value along that
chain. The checker resolves it that way, which is how the `litric-shell` finding
below surfaced.

| # | Component | Budget | Enforced by | Notes |
|---|---|---:|---|---|
| 1 | kernel + slab + page tables | 45 MB | boot-time measurement | trimmed config, `nano`-derived module set |
| 2 | systemd PID 1 | 12 MB | TARGET (not enforced: PID 1 and its cgroup are bounded by nothing in this repository - BUG-0008) | minimum viable unit set (ADR-0011) |
| 3 | journald | 10 MB | TARGET (not enforced: the journald drop-in sets journal config, not a unit limit - BUG-0008) | `Storage=volatile`, `RuntimeMaxUse=16M` |
| 4 | udevd | 8 MB | TARGET (not enforced: no drop-in supplies it - BUG-0008) | trimmed rules |
| 5 | dbus-broker | 5 MB | TARGET (not enforced: no drop-in supplies it - BUG-0008) | broker, not dbus-daemon |
| 6 | iwd + networkd + resolved | 15 MB | TARGET (not enforced: no drop-in supplies it - BUG-0008) | **not** NetworkManager |
| 7 | seatd | 2 MB | TARGET (not enforced: no drop-in supplies it - BUG-0008) | |
| 8 | **litric-shell** (compositor + shell) | **110 MB** | `MemoryHigh=110M`, `MemoryMax=140M` on `litric-shell.slice` joined by the shell service | ADR-0009; measured at 1920×1080 |
| 9 | pipewire + wireplumber | 18 MB | TARGET (not enforced: no drop-in supplies it - BUG-0008) | socket-activated; **0 MB until first stream** |
| 10 | litric-defender | 60 MB | TARGET (not enforced: the scanner has no binary in the image, so its units and its slice are parked under image/units-pending/ - BUG-0008) | ADR-0014; v1 allowed 100 MB; the 64 MB cap is designed into the parked `litric-defender.slice`, not removed |
| 11 | litric-aid (AI, model unloaded) | 8 MB | `MemoryMax=12M` [ACK: `litric-ai.slice` carries it, but no shipped unit joins that slice and no AI service is in the image, so it governs nothing - BUG-0008] | socket-activated; **0 MB until first request** |
| 12 | portals (xdg-desktop-portal) | 0 MB idle | dbus-activated | resident only while a portal dialog is open |
| 13 | headroom / slack | 27 MB | — | absorbs kernel version drift |
| 14 | litric-sysd | unset | `MemoryMax=64M` | enabled at boot, so resident at idle, and it has **no budget line** - BUG-0008 |
| 15 | litric-envd | unset | `MemoryMax=128M` | enabled at boot, so resident at idle, and it has **no budget line** - BUG-0008 |
| 16 | bluetoothd (`bluez`) | 5 MB | TARGET (not enforced: no drop-in supplies it - BUG-0008) | added 2026-10-12 with the wave-1 write half. Enabled at boot, so resident at idle. **The 320 MB TOTAL below was written before this row**, so it is 5 MB optimistic until the headroom row (13) is re-derived; the guest's own `mem.used_mb` in the GATE 01 entry of 2026-10-12 is the measurement, and it is a floor — no radio, no scanning, no traffic |
| | **TOTAL** | **320 MB** | `ci/check-memory-budget.py` — static policy presence, not a runtime measurement | |

#### Correction — 2026-10-09 (later the same day): what the checker found

This row previously named tests/budget/test_mem.py as the enforcement for the
320 MB total. `tests/budget/` is empty — the directory exists and contains no
files. So the one number in this document that decides whether the profile is
admissible has never been checked by anything, and the row read as though it
were.

The per-line budgets are real (`MemoryMax=` on the units, which systemd
enforces), and a cgroup cannot exceed them without being killed. What is missing
is the sum: nothing adds the lines up on a running system, so a profile that
keeps every line and still lands over 320 MB would not be noticed.

Two of the three gaps this correction named are now closed, on 2026-10-09:

  * **Static policy presence** — `ci/check-memory-budget.py` resolves every
    claim in this table against the units in `image/mkosi.extra/`, computes the
    effective limit along each unit's slice chain (`EffectiveMemoryMax=` is the
    most stringent value of the unit and its parent slices), verifies that every
    shipped cap appears in this table, checks that the column sums to the
    declared total, and fails on any divergence that the cell does not
    acknowledge. It found fourteen divergences on its first run; each one is
    either fixed above or recorded as `TARGET`/`[ACK: ...]` with a review
    condition. It has a self-test (`--self-test`) proving it fails on a fixture
    with an ungrounded claim.
  * **The ledger's own arithmetic** — the budget column is summed and compared
    to the TOTAL row, so a table that stops adding up cannot pass.

Still open, and the reason this table's total may not be quoted as a
measurement:

  * **Nothing measures the running system.** Reading `memory.current` per slice
    or `systemd-cgtop` from a booted image and failing on the total is runtime
    work, and it is NOT RUN. No static check can substitute for it.
  * **Two resident services have no budget line at all** — `litric-sysd`
    (`MemoryMax=64M`) and `litric-envd` (`MemoryMax=128M`) are both enabled at
    boot and are therefore part of an idle desktop, but the 320 MB above has no
    line for either. Their caps alone are 192 MB. Until someone sets their
    targets, the total is a total of the lines that happen to be written down —
    which is not the same claim. Rows 14 and 15 record them with `unset`
    budgets; the checker prints them as an INFO line rather than silently
    adding or omitting them.

### 3b. Units with a cap that the idle total does not count

These are socket-activated, so their caps bound a burst rather than an idle
baseline. They are listed because a cap that is not written down anywhere is how
a budget stops describing the system it claims to describe; the table's purpose
is accounting, not arithmetic (caps are not usage, and a cap is where the cgroup
OOM killer starts, not what a service consumes).

| Unit | MemoryMax | Why it is not in the idle total |
|---|---:|---|
| `litric-browser@.service` | 1536M | one instance per browser session, socket-activated; resident only while a page is open |
| `litric-files@.service` | 256M | one instance per file-operation session, socket-activated |
| `litric-capd.service` | 24M | socket-activated (`litric-capd.socket`); the screenshot appears when a privileged operation is requested |

`litric-app.slice` also carries no memory cap of its own, deliberately: its
comment says applications are what the machine is for, and per-app limits come
from the manifest's resource class applied by `litric-capd` at launch. That is a
documented absence rather than an oversight, and the checker reads an unlisted
*cap* as an error, not an unlisted absence.

---

Lines 9, 11 and 12 are budgeted at their *running* cost but are **not running at
idle**. A cold-booted, logged-in, untouched desktop should sit nearer **~275 MB**.
That is the number to chase; 320 MB is the number that fails the build.

### Deltas vs v1

| Line | v1 | v2 | How |
|---|---:|---:|---|
| compositor + shell | 360 MB (180 + 180) | **110 MB** | one process, no widget toolkit |
| defender | 100 MB | **60 MB** | mmap'd YARA rules, no in-process signature DB |
| **total** | **600 MB** | **320 MB** | |

---

## 4. CPU AND WAKEUP BUDGET — THE METRIC NOBODY GATES ON

Idle RAM is the metric everyone quotes. Idle *CPU* is the metric users feel, as
battery life and fan noise, and it is where a stock Windows install loses badly to
a curated Linux stack. LitricOS gates on it.

| Metric | Target (`desktop`, idle, 60 s settle, no user input) | Tool |
|---|---:|---|
| Aggregate CPU | **≤ 0.5 %** | `pidstat 1 60` |
| Wakeups/second, whole system | **≤ 25 /s** | `powertop --time=60` |
| Wakeups/second, `litric-shell` alone | **≤ 2 /s** | `powertop` |
| Timer interrupts (idle) | ≤ 260 /s | `/proc/interrupts` delta |
| Processes at idle | **≤ 32** | `ps -e --no-headers` |
| Threads at idle | ≤ 220 | `ps -eLf --no-headers` |
| Open file descriptors, system | ≤ 1200 | `/proc/sys/fs/file-nr` |
| Disk writes at idle, 10 min | **≤ 2 MB** | `iotop -ao` |

The last line matters more than it looks: a desktop that writes at idle keeps the
storage device awake, defeats SSD low-power states, and shortens battery life.
`Storage=volatile` journald plus no telemetry gets this near zero.

A shell that idles at ≤ 2 wakeups/second is only possible if it is **fully event
driven** — no polling timers, no animation ticks when nothing is animating, no
"refresh the clock every 100 ms". That is a design constraint on `litric-shell`,
enforced here as a number.

### Frame budget (from Phase 05, restated as a gate)

| Refresh | Budget | Gate |
|---|---:|---|
| 60 Hz | 16.67 ms | p99 within budget |
| 120 Hz | 8.33 ms | p99 within budget |
| 144 Hz | 6.94 ms | p99 within budget |
| 240 Hz | 4.17 ms | p99 within budget |

Also gated: **input-to-present p99 ≤ 2 frames**, and **zero missed flips** over a
60 s animation. `litric-shell` measures composition-to-presentation against the
life state's own interval and logs the first ten over-budget frames and every
hundredth thereafter (one line per frame would make the journal the largest cost
in a loop that is already over budget); `frames_drawn=` and
`frames_over_budget=` go into the status file, and that count is the CI metric.
Added 2026-10-14 — the warning this paragraph described did not exist before
then, so the metric it named was unobservable.

#### Measured on the development host — 2026-10-12

First measurements of this path. Tool: `cargo run --release -p litric-shell
--example render_bench`, 15 runs per line, release build, 1280x800, WSL2 on the
development machine (8 vCPU), software renderer, no GPU in the path. The raw
lines are in `docs/EVIDENCE.md`; the two bugs are [BUG-0015](bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md)
and [BUG-0018](bugs/BUG-0018-a-locked-frame-costs-half-a-second.md).

| Frame | median | mean ± stddev | n | Budget | Status |
|---|---:|---:|---:|---:|---|
| Desktop, two windows open | 185.339 ms | 188.834 ± 11.291 ms | 15 | 16.67 ms | **NOT MET** — 11.1x (BUG-0015) |
| After one pointer move | 183.525 ms | 185.027 ± 8.807 ms | 15 | 16.67 ms | **NOT MET** |
| After opening a window | 423.675 ms | 427.174 ± 11.557 ms | 15 | 16.67 ms | **NOT MET** — 25.4x (BUG-0015) |
| Locked, shipped look (interval 33 ms) | 569.664 ms | 608.179 ± 90.267 ms | 15 | 16.67 ms | **NOT MET** — 34.2x (BUG-0018) |
| Locked, customize sheet open | 804.543 ms | 820.711 ± 49.188 ms | 15 | 16.67 ms | **NOT MET** — 48.3x (BUG-0018) |
| Full-screen blur, two passes | 91.202 ms (r10) | 92.329 ± 5.928 ms | 15 | — | the renderer's largest single cost |
| Present a full frame (4 096 000 B) | 3.471 ms | 3.552 ± 0.428 ms | 15 | — | not the bottleneck |
| Present one damage band (163 840 B) | 0.104 ms | 0.113 ± 0.026 ms | 15 | — | not the bottleneck |

These are **host numbers, not target numbers**, and they are presented as the
failures they are: every frame path on this list is over the gate, and the two
bugs above are open. They are recorded here because a budget that is only ever
compared against nothing is a wish.

#### Measured again after the lock fix — 2026-10-15

[BUG-0018](bugs/BUG-0018-a-locked-frame-costs-half-a-second.md) is fixed: the frame
that opens a locked session fell from 293.727 ms to **35.819 ms**, and the
customize sheet from 477.656 ms to **185.718 ms**. Those rows were re-measured on a
**second host** (Windows 11, `x86_64-pc-windows-gnu`, same release profile), not
on the WSL2 machine above — which is why the "before" column here reads 293.727 ms
where the table above reads 569.664 ms for the same frame. The two hosts differ by
roughly 2x in absolute terms, so the ratio and the pixel counts are the parts that
carry.

| Frame | Before | After | Speedup | Its interval | Status |
|---|---:|---:|---:|---:|---|
| Locked, shipped look | 293.727 ms | **35.819 ms** | 8.2x | 33 ms | **NOT MET** — 1.08x over (was 8.9x) |
| Locked, entry sheet out | 311.362 ms | **37.983 ms** | 8.2x | 33 ms | **NOT MET** — 1.15x over |
| Locked, motion "Still" | 306.114 ms | **36.843 ms** | 8.3x | 33 ms | **NOT MET** — 1.12x over |
| Locked, customize sheet open | 477.656 ms | **185.718 ms** | 2.6x | 33 ms | **NOT MET** — 5.6x; 4.06M of its 7.27M pixels are one soft shadow |

The 16.67 ms gate above is the desktop's. A locked session asks its own loop for
33 ms (`lock.interval_ms=33`, which `render_bench` prints) and is measured against
that. Pixels: a locked frame paints 1 909 744 where it painted 10 426 249. Raw
lines: [`docs/evidence/2026-10-15/bug0018-bench-final.log`](evidence/2026-10-15/bug0018-bench-final.log).

#### Measured through the loop's own repaint path — 2026-10-18

The rows above came from a benchmark that drew the way the loop drew *at the
time it was written*: restore the wallpaper, then draw the whole desktop
unclipped. The loop has drawn through `Compositor::repaint` since — each damage
region restored and drawn over with the clip set to it — so the benchmark was
timing a path nothing runs. It now draws through the compositor, exactly as
`main.rs` does, and prints where each frame's pixels went
(`*.sites_per_frame=` lines).

What putting the benchmark on the real path exposed, in one sentence: **a
spring that moved was a full redraw** (`Shell::tick`, BUG-0019's damage at the
wrong granularity). Switching windows repainted all 1 024 000 pixels of a
1280x800 desktop to move one shadow. `anim::ShellAnims::step` now reports *what*
moved — an overlay reveal (still a whole-frame repaint: overlays are large and
can move, the same call `overlay_damage` makes) or the ids of windows whose
focus elevation moved, whose frames are damaged with the shadow's reach. The
proof that targeted damage is not a shortcut is
`main::tests::a_partial_redraw_is_pixel_identical_to_a_full_redraw`, which
compares partial frames to full redraws pixel by pixel across the journeys it
drives; it is green on every gate in this row.

Tool: `cargo run --release -p litric-shell --example render_bench`, 15 runs per
line, release build, 1280x800, WSL2 on the development machine (8 vCPU),
software renderer, no GPU in the path. The rows are this host only; the two
hosts in the rows above differ by roughly 2x, so compare levels within a table,
never across.

| Frame | median | mean ± stddev | pixels visited | Status |
|---|---:|---:|---:|---|
| Desktop, full redraw | 41.084 ms | 40.789 ± 3.333 ms | 3 704 646 | **NOT MET** — 2.5x over 16.67 ms |
| Pointer move, settled | 16.179 ms | 14.290 ± 5.228 ms | 23 428 | mixed by design: frames spent while a spring flies repaint one window (1.2M px); a settled one repaints 23k px in 0.997 ms |
| Open a window | 65.996 ms | 67.529 ± 4.489 ms | 5 495 632 | **NOT MET** — its regions cover more than half the screen, so the plan is a full repaint |
| Locked, one due frame | 8.399 ms | 12.272 ± 14.293 ms | 1 684 139 | meets the lock's 33 ms interval |
| Locked, customize sheet | 36.695 ms | 36.669 ± 1.393 ms | 3 199 475 | **NOT MET** — 1.11x over 33 ms |
| Locked, still, nothing owed | 0.000 ms | — | 0 | no frame is drawn; the benchmark asserts it |
| Full-screen blur, two passes (r10) | 66.141 ms | 70.493 ± 10.379 ms | — | still the largest single primitive |
| Present a full frame (4 096 000 B) | 3.488 ms | 3.568 ± 0.169 ms | — | not the bottleneck |
| Present one damage band (46 080 B) | 0.141 ms | 0.143 ± 0.007 ms | — | not the bottleneck |

The second and sixth rows are the two things this measurement changed about the
story. Switching windows now costs one window's shadow, not the desktop: the
same tool, same host and same scenario measured 43.922 ms median before the
fix and 16.179 ms after, and the settled frame draws 23 428 pixels where the
bug drew 2.8M. And the shipped lock look — Still motion, the configuration
every machine boots into — owes *no* frame between a keypress and the next
clock change: the tick draws nothing at all. The four lock rows above are each
one **due** frame, which is what those events cost.

The customize sheet's row also carries the soft-shadow rewrite: one shadow went
from 4 060 000 pixel writes per frame (the 2026-10-15 note) to 373 120 — the
same squared falloff, composed from one signed-distance pass instead of twelve
rings over the bounding box. That is the largest part of the sheet's fall from
185.718 ms (on the other host) to 36.695 ms here; the level is not comparable
across the two hosts, the pixel counts are.

Raw lines: [`docs/evidence/2026-10-18/render-bench-realpath.log`](evidence/2026-10-18/render-bench-realpath.log).

---

## 5. DISK BUDGET

| Artifact | Target |
|---|---:|
| `litricos-nano.iso` | ≤ 90 MB |
| `litricos-desktop.iso` | ≤ 850 MB |
| system erofs (desktop, zstd) | ≤ 700 MB |
| kernel (vmlinuz, compressed) | ≤ 12 MB |
| initrd (UKI-embedded) | ≤ 18 MB |
| A/B slot overhead | 2 × system size — accepted, see ADR-0004 |

---

## 6. THE FOUR LEVERS

In descending order of expected value. **These are hypotheses to test, not
results.** Each estimate is an engineering expectation that the harness must
confirm or refute.

1. **One graphical process, no widget toolkit** (ADR-0009) — expected −250 MB.
2. **Zero-daemon idle: everything socket- or dbus-activated** (ADR-0011) —
   expected −60 MB and most of the wakeup budget.
3. **Trimmed kernel and module set** (`kernel/config/`) — expected −25 MB and
   −0.6 s boot.
4. **erofs read-only system + zram** — page cache is shared, clean and instantly
   evictable; nothing in the system image is ever dirtied. Expected −30 MB
   effective.

Everything after these four is single-digit MB. Do the four first. Do not spend a
week on sysctls before the shell is one process.

---

## 7. RULES OF ENGAGEMENT

1. **Any RAM saving must state what capability was removed.** "Optimised" without
   a named removal is a regression in disguise.
2. **Every number is tagged with its environment.** VM numbers are never presented
   as metal numbers.
3. **The harness runs the full set every time.** A partial run is not a result.
4. **CI fails on any regression >10 %** in any line, absent an approved note.
5. **`NOT RUN` is a valid, respectable value.** An estimate wearing a number's
   clothes is not.
