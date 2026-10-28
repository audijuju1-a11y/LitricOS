# LitricOS progress tracker

Living record of verified engineering batches. A line is only here if the
command that produced it ran in this workspace; unrun work is listed under
**Open**, never as done.

- Repository: `audijuju1-a11y/LitricOS`
- Work branch: `freebuff/d7991b8c41a570fd94f0a9bc` (tracks `origin/main`)
- Rust workspace root: `src/` (23 crates, toolchain 1.99.1)
- Date of this entry: 2026-10-28

## 1. Repository identity check

The mission asked for a separate "Deep-research-all" GitHub repository. The
authenticated account (`audijuju1-a11y`) has no such repository: the full
repository list contains LitricOS, hyper, Hypercheck, Basehider_Pro, In-Ear-Pro,
sentinel-shield, audisurround-license-server, AudiSurround-Releases,
audismp-shop and Basehider. The LitricOS research corpus lives inside this
repository at `Research-Deep-all/` (Parts 1–32, coverage checklist and source
register). Public search finds three unrelated repositories with similar names
(`hades905/deep-all-research`, `we11as22/all_included_deep_research`, one
unrelated thesis project); none is this project. Work therefore continued in
`Research-Deep-all/` rather than guessing a target.

## 2. Batch: frame-loop hardening and CI enforcement (verified)

| Change | Evidence |
|---|---|
| `crumb_window` no longer indexes a slice in the frame loop; `skip` is `usize` with `saturating_add` | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 |
| Breadcrumb "last item" arithmetic no longer overflows on a future edit | same clippy run |
| Dead `filled` flag removed from the scale-banner button closure (both branches were `Material::Popover`, so the parameter could only mislead) | same clippy run; visual material unchanged |
| GitHub Actions workflow runs fmt, clippy (`-D warnings`), workspace tests and the research validators | `.github/workflows/ci.yml` (new) |
| Part 23 no longer attributes a Windows-forum article (`S0930`) to registry documentation | Part 32 §3 updated; citation removed rather than repointed |

### Commands and results

```text
$ cargo fmt --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
TESTS passed=1634 failed=0
```

## 3. Benchmark corpus (verified, not a performance claim)

`litric-bench` drives the real `litric-defender` engine; it is a security
benchmark, not a frame-time benchmark.

```text
$ cargo run -q -p litric-bench -- generate --output /tmp/lb --count 32 --seed 7
generated 32 samples (76135 bytes) in /tmp/lb/corpus
exit 0

$ cargo run -q -p litric-bench -- validate --output /tmp/lb
determinism (seed 928374): OK
digest agreement: OK
corpus: 1000 samples, 980 hostile, 20 benign
exit 0
```

The 1,000-sample corpus is generated deterministically by `validate`; the
`--count 32` generation is a separate, smaller artifact. Both are correctness
evidence, not throughput.

## 4. Open, with the reason

| Item | State | Blocker |
|---|---|---|
| Frame time, RAM, startup time, GPU use | **PARTIAL — host-side numbers in §7, desktop not measured** | The binaries compose and serve headlessly and were measured (§7). Present-to-display, GPU use and whole-machine idle RAM still need a booted image and a display; no QEMU, no image and no root in this workspace |
| `VP-01`–`VP-27` device halves | **PARTIAL / NOT RUN** | Need Windows 26H2, macOS, a Linux desktop session or a mkosi image |
| Chapter L rows observed live | **0 / 112** | Same; the corpus remains documentation-derived until `VP-23` runs |
| Remaining "Reused" citation list in Part 23 | **DONE** | Every entry checked in this batch; decisions in §6 |
| Installer/image build and QEMU boot | **NOT RUN** | This workspace has no image, no `qemu-system-x86_64`/`qemu-img`/`mkosi`, and no way to install them: `sudo` is refused (`no new privileges` flag) and the package index has no `qemu-system-x86` (§7 Phase 1 quotes both) |

## 5. Next verified step

Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`
on every change; the new workflow does this on push. The next measurement that
would move the tracker is a QEMU boot with `tools/measure-ui.py` and
`tools/measure-idle.py`, because every whole-machine performance row is blocked
on the same missing image. What *can* be measured without one is in §7, and is
reproduced by `python3 tools/measure-host.py`.

## 6. Batch: Part 23 citation audit and the trash-name guard (verified)

### Citation decisions, one per entry

| Citation | Decision | Reason |
|---|---|---|
| `S0941` (`defaults`) | **removed** | Windows Central "14 defaults worth changing" is Windows 11 settings, not the macOS `defaults` store. The macOS group is now `S3079` + `S0112b`, and the body's "3 inspected sources" was corrected to 2 |
| `S3079` | kept | macOS 27 `defaults write` report; supports the macOS store claim |
| `S0112b` | kept | macOS defaults/window-appearance issue; supports the same claim |
| `S1740` | kept, label fixed | Syncfusion WinUI compact sizing; "family" implied more rows than exist (one) |
| `S3716` | kept, label fixed | GTK 4 `Gtk.Entry`; one row, not a family |
| `S3042` | kept, label fixed | WWDC26 AppKit session; one row, not a family |
| `S0130` / `S0131` | kept, label fixed | Both are Microsoft Support notification/DND pages, not API references; the label now says settings and DND |
| `S3852` | kept | XDG Desktop Portal documentation; supports the portal claim |
| `S3845`–`S3849` | kept | Chapter H sources: default apps, Windows Backup, recovery, Time Machine, Windows Update |
| `S0930` in the I.03 table | **replaced** | The table still cited the Windows-forum search article as "Registry". It now cites `S3893` (Microsoft Learn, *Registry - Win32 apps*), opened this turn, and the claim was narrowed to what that page actually states: a system-defined database read and written through the registry API |

The register gained one inspected source: `S3893`, 4,208 rows, last id `S3893`,
1,303 inspected, 382 domains.

### Code defect: an empty trash name addressed the whole trash

`DiskTrash::purge` and `restore` validated `/` and a leading `.` but not the
empty string. `files_dir.join("")` is `files/` itself, so `purge("")` reached
`remove_dir_all(files/)` and `restore("")` would have moved the entire trash
directory to the destination. `get("")` happened to return `None`, which hid the
hole from the server path but not from the public API.

| File | Change |
|---|---|
| `src/litric-files/src/trash.rs` | one `valid_name` rule (non-empty, no `/`, no leading `.`) used by `get`, `restore` and `purge`; regression test asserts the entry and `files/` survive three empty-name calls |
| `src/litric-files/src/server.rs` | `RESTORE` and `PURGE` reject an empty decoded field with `bad-request` before reaching the disk layer |

```text
$ cargo fmt --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ cargo test --workspace
TESTS passed=1635 failed=0
```

### Research validators

```text
$ python3 ci/check-python-syntax.py
OK: 60 python file(s) parse (no bytecode written)          exit 0
$ python3 ci/lint-line-endings.py
OK: 351 file(s) checked, all LF                            exit 0
$ python3 ci/lint-doc-refs.py
OK: 748 citation(s) across 151 document(s) resolve        exit 0
$ python3 tools/validate_chapter45.py --check
8 checks: 2 PASS, 6 PARTIAL, 0 FAIL                       exit 0
$ python3 tools/fix_source_id_keys.py --check
no-op: 4208 rows, 4208 distinct ids, 0 duplicates; nothing written
```

## 7. Batch: host-side measurement — real numbers, no image

Starting point: `HEAD` and `@{u}` were both
`52ba41ce21b361f91e9ea2372b9a6e5e8d58bc43`, working tree clean.

### 7.1 Feasibility probe (every line run in this workspace)

```text
$ uname -a
Linux freestyle-vm 6.1.102 #1 SMP PREEMPT_DYNAMIC Wed Aug 26 20:12:24 UTC 2026 x86_64
$ nproc
8
$ free -m
               total        used        free      shared  buff/cache   available
Mem:           15993         536       11230           1        4557       15456
$ grep -m1 'model name' /proc/cpuinfo
model name	: 19/01
$ ls -l /dev/kvm
crw-rw---- 1 root kvm 10, 232 Oct 28 04:04 /dev/kvm
$ command -v qemu-system-x86_64 qemu-img mkosi systemd-nspawn /usr/bin/time
(no output: none of the five exists)
$ apt-get install -s --no-install-recommends qemu-system-x86
E: Unable to locate package qemu-system-x86
$ sudo -n true
sudo: The "no new privileges" flag is set, which prevents sudo from running as root.
$ ls *.img
ls: cannot access '*.img': No such file or directory
```

`/dev/kvm` exists but is `root:kvm 0660`, this account is not in `kvm`, and
`sudo` is refused, so neither the group nor a package index can be obtained.
`tools/measure-idle.py` needs `build/out/litricos.raw` plus OVMF, and
`.gitignore` excludes `*.raw`/`*.img`: no image exists in this checkout. The
QEMU/image path is therefore **blocked with evidence**, not skipped. The CPU
model string is the masked `19/01` a virtual machine reports, so no clock or
microarchitecture claim is available from it.

### 7.2 Measured on this host (headless, no display, no image)

Both commands below were run in this turn; the JSON is the machine-readable
record (`validation/perf/host-measurements.json`).

```text
$ (cd src && cargo build --release --workspace)
BUILD_EXIT=0

$ python3 tools/measure-host.py --runs 5
component                      n   ready/wall ms (med)   rss kB (med)   peak kB (med)   idle cpu % (med)
--------------------------------------------------------------------------------------------------------
litric-shell --once 1280x800   5        129.5 [126.3-133.1]          -         32044            -
litric-shell --once 1920x1080  5        227.5 [222.5-232.3]          -         59644            -
litric-filesd --serve          5         10.6 [10.5-10.6]       1284          1284         0.00
litric-sysd --serve            5         20.7 [20.7-20.8]       2840          2840         0.00
litric-notifyd --serve          5         10.6 [10.5-10.7]       1244          1244         0.00
render_bench                   5      10067.6 [10023.7-10135.0]          -         31412            -
```

`render_bench` (the repository's own 15-frame-per-scene example, median of the
per-run medians over 5 processes):

| Scene | Median | What it is |
|---|---|---|
| `full_frame` | **86.14 ms** | wallpaper restore + whole desktop draw, 1280x800 |
| `pointer_move_frame` | 87.75 ms | the benchmark draws unclipped; the shipping path clips (see 7.3) |
| `open_window_frame` | 184.17 ms | first frame of a new window |
| `blur_full_screen_r3_2passes` | 35.34 ms | one full-screen box blur, two passes |
| `present_full` | 3.87 ms | blit to a file standing in for `/dev/fb0` |
| `present_pointer_damage` | 0.15 ms | 32 damage rows |
| `lock_frame` / `_entry` / `_still` | 6.11 / 6.10 / 6.11 ms | locked shell, no wallpaper restore |
| `lock_frame_customize` | 71.70 ms | locked shell with the customize sheet open |

One-frame wall time scales with pixels the way a scalar compositor plus PNG
encode should: 1.024 Mpx → 129.5 ms and 2.074 Mpx → 227.5 ms, i.e. about
**93 ms per megapixel plus 34 ms of process startup** (two points, so this is a
linear fit, not a model). Peak RSS over the same two sizes is 32.0 MB and
59.6 MB.

### 7.3 What these numbers do and do not say

* **Valid:** process start, one composed frame, daemon ready latency, daemon
  steady RSS and idle CPU on this machine. Peak RSS comes from `wait4(2)`
  `ru_maxrss` / `VmHWM`, never from an estimate.
* **Not valid as a desktop claim:** there is no `present()` to a DRM/KMS
  device, no GPU, no systemd and no boot. `full_frame` is composition only.
* **A caveat found while reading the benchmark:** `render_bench`'s `frame()`
  calls `views::draw` on the whole scene, while the shipping
  `Compositor::repaint` (`src/litric-shell/src/desktop.rs:253-262`) sets a clip
  to each damage region. So `pointer_move_frame` (87.4 ms) measures the
  *unclipped* worst case, not what a pointer move costs on the machine. The
  `full_frame` and `present_*` rows are not affected by this.
* 86 ms for a full 1280x800 frame is 5.4x the 16 ms active budget. That is
  consistent with BUG-0015/BUG-0018 ("the renderer is over budget today") and
  with `docs/DESKTOP-APP.md`'s 204 over-budget frames in 518; it is **not** new
  evidence that real hardware misses the budget.

### 7.4 Code change decision

No product code was changed. The measurement produced no small, low-risk
correctness or performance defect with an obvious fix; the one large finding
(full-frame composition cost) is already tracked as BUG-0015/BUG-0018 and
cannot be validated here without a display. Churning the renderer against a
number this host cannot confirm would be guessing, so it was not done.

Added instead: `tools/measure-host.py`, the harness that produced 7.2. It is
read-only with respect to the product, writes its JSON under `validation/perf/`,
and exits 1 when a component cannot be run, with the reason recorded rather
than a substituted number.

### 7.5 Still blocked

| Item | State | Concrete blocker |
|---|---|---|
| `tools/measure-idle.py` / `measure-ui.py` | **NOT RUN** | no image, no QEMU, no root (7.1) |
| Boot time, whole-machine idle RAM, idle CPU | **NOT RUN** | same |
| GPU use, present-to-display frame time | **NOT RUN** | no DRM device in this VM |
| `VP-01`–`VP-27`, Chapter L live rows | **NOT RUN** | need Windows/macOS/DE sessions or the image |

## 8. Batch: profiled the full frame, fixed the shadow (verified)

Baseline on `8248002`, re-measured rather than trusted: five processes of
`render_bench`, each timing 15 frames per scene.

```text
$ src/target/release/examples/render_bench <dir>   (5 processes)
full_frame        median 86.93 ms   spread 86.10-88.30
open_window_frame median 183.70 ms  spread 183.21-183.97
```

### 8.1 Profiling method (what this host actually has)

```text
$ command -v perf valgrind heaptrack bpftrace
(all MISSING)
$ cat /proc/sys/kernel/perf_event_paranoid
2
```

No sampling profiler and no way to install one (no root, no package index —
§7.1). So the attribution is Rust-level instrumentation in the bench path:

1. `Canvas` already bills every pixel loop by `Site`; `render_bench` now prints
   `full_frame.sites_per_frame` for the full frame.
2. `render_bench` now times the primitives the frame is built from, on the same
   1280x800 canvas: full-screen blit, a 600x420 panel fill, a 600x420
   `soft_shadow`, a 600x420 `glass_backdrop`, and ten 51-character text lines.

```text
full_frame.sites_per_frame=draw=1772848 glass=462944 shadow=392576 text=101278 brightest=19583
primitive_soft_shadow_panel      median 46.46 ms
primitive_glass_backdrop_panel   median  1.98 ms
primitive_fill_panel             median  1.19 ms
primitive_text_10_lines          median  3.45 ms
primitive_blit_full_screen       median  0.17 ms
```

**Attribution:** 46.46 ms of an 86.93 ms frame — 53 % — was `soft_shadow`,
which blends up to twelve rings into every pixel it covers. Everything else the
frame draws is 1–3 ms per primitive. The remaining `draw` site is large in
pixels but cheap per pixel (the panel fill measures 4.7 ns/px), which is why the
shadow was the first target and the draw loops were not touched.

### 8.2 The change (multiple files)

| File | Change |
|---|---|
| `src/litric-shell/src/canvas.rs` | `soft_shadow` composes the rings a pixel is *fully* inside as one table lookup: `chain[n][v]` is what the outermost `n` rings do to channel value `v`, built by calling the same `blend` in the same order, so the arithmetic and rounding are the loop's own. Rings straddling the edge are still blended one at a time, in the original outermost-first order. |
| `src/litric-shell/examples/render_bench.rs` | the per-site split for `full_frame` and the five primitive timings above, so the next person can re-derive the attribution instead of trusting this table |

Pixel equality is asserted by the repository's own reference test, which renders
each case twice — once through `soft_shadow`, once through the literal
twelve-ring implementation — and compares every pixel. It covers four real
panel shapes including `radius = 0` and a fractional origin, and passes:

```text
$ cargo test -p litric-shell --lib canvas::
test result: ok. 49 passed; 0 failed
  incl. the_one_pass_shadow_equals_the_twelve_ring_shadow
```

The golden-frame gate could not be used as a second proof here: `frame_dump`
reports `no Latin text face is installed` in this sandbox (the image's fonts are
not present), so it wrote no frames and `ci/check-golden-frames.py` reported
`no frame directory at build/frames`. That is a missing fixture, not a pass.

### 8.3 Measured result (5 processes, same host, same protocol)

| Scene | Before | After | Change |
|---|---|---|---|
| `full_frame` | 86.93 ms | **35.45 ms** | **−59.2 %** |
| `pointer_move_frame` | 88.38 ms | 35.62 ms | −59.7 % |
| `open_window_frame` | 183.70 ms | 71.56 ms | −61.0 % |
| `lock_frame_customize` | 72.05 ms | 22.15 ms | −69.3 % |
| `primitive_soft_shadow_panel` | 46.46 ms | **3.83 ms** | **−91.8 %** |
| `blur_full_screen_r3_2passes` | 35.49 ms | 35.83 ms | unchanged (not touched) |
| `present_full` | 3.93 ms | 3.90 ms | unchanged |
| `lock_frame` | 6.12 ms | 6.11 ms | unchanged |

`full_frame.sites_per_frame` is byte-for-byte the same split after the change
(shadow still bills 392,576 pixel visits), which is what a pure shortcut should
look like: the same pixels, the same arithmetic, fewer operations.

### 8.4 What is left toward the 16 ms budget

35.45 ms is 2.2× the active-frame budget on this 8-vCPU VM with no GPU. The
next cost is the `draw` site (1.77 M visits) but it is cheap per pixel; splitting
it further needs a profiler that can attribute *inside* `views::draw`, which
this host does not have. No second change was made, because the primitive
timings did not justify one — guessing at the draw loop would be exactly the
kind of unfounded optimisation this batch exists to avoid.

## 9. Batch: call-level attribution of the draw site (instrumented, one revert)

Ground truth re-measured on `9419eb0` before touching anything: `full_frame`
**35.36 ms** median (spread 35.23–35.52) over five `render_bench` processes.

### 9.1 Instrumentation added

`src/litric-shell/src/canvas.rs` gained `canvas::profile`: opt-in, thread-local
call timers. Off by default — a call pays one flag read and nothing is
allocated until a name is first recorded. Seven drawing entry points are timed:
`blit_from`, `fill_round_rect`, `stroke_round_rect`, `stroke_line`,
`draw_argb`, `glass_backdrop`, `soft_shadow`. `render_bench` enables them for
one extra frame *after* the timed loop, so the reported frame times pay no
instrumentation cost, and prints calls, ms and share per name.

### 9.2 Attribution of a 35.2 ms full frame (real output)

| Entry point | Calls | ms | Share |
|---|---|---|---|
| `glass_backdrop` | 3 | 20.49 | **58.2 %** |
| `fill_round_rect` | 75 | 5.40 | 15.3 % |
| `soft_shadow` | 3 | 5.04 | 14.3 % |
| `stroke_line` | 2,239 | 2.23 | 6.3 % |
| `stroke_round_rect` | 5 | 1.57 | 4.5 % |
| `blit_from` | 1 | 0.25 | 0.7 % |
| `draw_argb` | 10 | 0.23 | 0.7 % |
| **timed total** | | **35.21** | 100 % |

(Figures from `render_bench`'s `profile.*` lines; the shadow primitive is
already down from 46.46 ms to 3.8 ms after §8, and the 75 fills plus 2,239
stroke calls are the icons and window chrome.)

### 9.3 Experiment: core fast path in `glass_backdrop` — reverted

Since `fill_round_rect` already skips the signed-distance square root for
pixels the rounded outline covers completely, the same core rule was added to
`glass_backdrop`'s attached-backdrop branch with the same coverage function.
The pixels are bit-identical by construction (same `coverage(d)` value of 1.0),
and all 49 canvas tests — including the reference tests — passed.

| Measurement | Before | After | Spread | Verdict |
|---|---|---|---|---|
| `primitive_glass_backdrop_panel` | 1.96 ms | 1.61 ms | outside spread | real, −18 % |
| `full_frame` | 35.36 ms | 35.15 ms | 34.68–35.57 (inside) | **noise** |

Because the frame gain is inside its own spread, the change was **reverted**
and the revert is recorded in the code comment where it was tried. What the
experiment proves is useful: the remaining 20 ms of glass is *not* the distance
function, it is the per-pixel backdrop lookup and blend that both paths share.
Attacking that needs a different design (e.g. blitting glass from a cached
source), which is not a small exact-equivalence change and is not justified by
this batch's evidence.

### 9.4 Where the frame stands

35.4 ms on this VM, 2.2× the 16 ms active budget, with `glass_backdrop` (58 %),
`fill_round_rect` (15 %) and `soft_shadow` (14 %) as the measured order. The
next real step would be the glass design change above, or a GPU/DRM present
path — neither can be validated on this host, which has no display device.

## 10. Batch: glass aggregate confirmed, per-panel split not delivered

Ground truth re-measured on `3bed3ef` before touching anything: `full_frame`
**35.36 ms** median (spread 35.23–35.52) over five `render_bench` processes.
§9's aggregate reproduced: `glass_backdrop` **20.08 ms / 57.8 %** in 3 calls,
`fill_round_rect` 5.28 ms, `soft_shadow` 5.14 ms, `stroke_line` 2.22 ms over
2,239 calls, `stroke_round_rect` 1.53 ms (`profile.timed_ms` 34.45–34.72).

### 10.1 Per-panel split: attempted, not delivered

`glass_backdrop` was instrumented with area buckets (`glass_panel_s/l/xl`,
pixel area recorded via `profile::note_pixels`) so the three calls could be
separated. The bucket rows **never appeared** in the bench output — the outer
`glass_backdrop` row reported `px=0` and no `glass_panel_*` line was printed,
even after `touch`ing the crate and rebuilding. Rather than present an
aggregate as if it were a per-panel table, the bucket code was removed and this
paragraph records the failure. Per-panel attribution is **NOT DELIVERED**.

### 10.2 Hot path: two attempts, both reverted

| Attempt | Before | After | Spread | Verdict |
|---|---|---|---|---|
| Row-offset direct read/write loop in the attached-backdrop branch (skip three per-pixel bounds checks, same `blend`) | 35.36 ms | 35.57 ms | 35.34–36.05 | **reverted**, no gain |
| Hoist `y + 0.5` out of the inner loop | — | — | — | reverted with the row loop; below resolution |

All 49 canvas tests (including the shadow reference test) passed with both
attempts in place, so they were exact — they were simply not faster. **No
hot-path change is kept.**

### 10.3 What the numbers say, and the next concrete blocker

`glass_backdrop` is 20 ms for 3 calls, ~58 % of a 35 ms frame, and §9 already
showed the distance function is not the cost (the core fast path moved the
primitive but not the frame). The two exact per-pixel shortcuts tried here also
did not move the frame. The remaining hypotheses are structural: the panels are
large and every pixel is blended from the backdrop with no reuse between
overlapping panels, which needs a cached glass source keyed on backdrop and
panel geometry — a design change, not an exact micro-edit. The concrete blocker
for validating any of it further is the same as §7: no display/GPU path and no
sampler on this host, so frame-time attribution beyond these entry points is
inference, not measurement.

## 11. Batch: why the buckets vanished, and the stage table at last

### 11.1 Root cause (code evidence, not a guess)

`render_bench`'s `build()` makes the scene with `let scene = wallpaper.clone();`
and never calls `set_backdrop` — the only `set_backdrop` in the example is at
line 537, for the *primitive* glass benchmark, long after the profiled frame.
So the frame under test runs `glass_backdrop`'s **live-blur branch**, and §10's
buckets plus `note_pixels` sat inside the `if let Some(source) = ...` branch
that never executes. That is why no `glass_panel_*` row printed even though the
code was present and rebuilt.

**This also corrects §9.3.** The "core fast path" tried there was in the same
attached-backdrop branch, so it never ran in the measured frame; its 35.36 →
35.15 ms result measured nothing but noise. The revert was right for the wrong
reason, and the record now says so.

### 11.2 What prints now (raw, one instrumented frame, 5 processes of run 1)

```text
profile.glass_backdrop.calls=3  ms=20.405  px=0    ns_per_px=0.00
profile.glass_panel_m.calls=3   ms=16.485  px=268395 ns_per_px=61.42
profile.glass_stage_blur.calls=3        ms=14.920
profile.soft_shadow.calls=3             ms=7.302
profile.fill_round_rect.calls=75        ms=5.166
profile.glass_panel_s.calls=2   ms=3.919   px=41540  ns_per_px=94.34
profile.glass_stage_mask_blend.calls=3  ms=2.761
profile.stroke_line.calls=2239          ms=2.241
profile.glass_stage_saturate.calls=3    ms=1.897
profile.glass_stage_copy.calls=3        ms=0.803
profile.glass_panel_m.dims=435x355
profile.glass_panel_s.dims=670x62
profile.timed_ms=77.844
```

Timed frame on this commit: `full_frame` **35.45 ms** median (spread
35.40–37.68), `open_window_frame` 71.76 ms, primitive glass 1.94 ms.

### 11.3 Sums and the one thing that does not add up

* Stages: 14.920 + 2.761 + 1.897 + 0.803 = **20.381 ms** against the outer
  **20.405 ms** — a 0.024 ms (0.1 %) gap, inside the stated 0.05 ms tolerance.
* Panels: 16.485 + 3.919 = **20.404 ms** against the same outer row (0.001 ms).
* **But** the panel rows report 5 calls (3 + 2) where the outer row reports 3.
  The times agree exactly and the counts do not; the cause was not established
  in this batch, so the counts are reported as printed and not interpreted. This
  is the next thing to fix in the instrumentation, not in the renderer.

The instrumented frame is slower (77.8 ms timed) because timers are on for it;
shares in the raw output are relative to that, not to the 35.45 ms frame.

### 11.4 Hot path: no change, and why

The blur is **14.92 ms of 20.41 ms (73 %)**. It is already a separable
two-pass sliding-window box blur, over a working region already cut to the clip
(§8's BUG-0015 note). No exact equivalent was found that removes per-pixel work
— the cost is the pixels themselves. Per the batch rule, **no renderer change
was made** rather than a speculative one. Remaining: 35.45 ms against the 16 ms
budget, 2.2×; the next named blocker is that the blur's cost is structural
(large translucent panels blurred per frame) and would need a different design
plus a display path to validate, neither of which this host has.

### 11.5 The 5-vs-3 count mystery: solved

**Confirmed cause, from the code.** `profile::note_pixels` created a missing row
with a placeholder `calls: 1`:

```rust
            } else {
                rows.push(Row {
                    name,
                    calls: 1,      // <- placeholder, counted as a real call
                    nanos: 0,
                    pixels,
                });
```

and the timer for that same call then added one more:

```rust
    fn record(name: &'static str, nanos: u64) {
            if let Some(row) = rows.iter_mut().find(|row| row.name == name) {
                row.calls = row.calls.saturating_add(1);
```

So each bucket that `note_pixels` created read exactly one call high: the two
real medium panels reported 3 and the one small panel reported 2. It was a
bookkeeping bug in the instrumentation, not a second draw and not a reset.
Seeding with `calls: 0` fixes it. No renderer code was involved.

Raw instrumented frame after the fix (run 1 of 5):

```text
profile.glass_backdrop.calls=3   ms=20.329
profile.glass_panel_m.calls=2    ms=16.440  px=268395  ns_per_px=61.25
profile.glass_panel_s.calls=1    ms=3.889   px=41540   ns_per_px=93.62
profile.glass_stage_blur.calls=3        ms=14.892
profile.glass_stage_mask_blend.calls=3  ms=2.768
profile.glass_stage_saturate.calls=3    ms=1.868
profile.glass_stage_copy.calls=3        ms=0.796
profile.glass_panel_m.dims=435x355
profile.glass_panel_s.dims=670x62
profile.timed_ms=75.305
```

Panels: 2 + 1 = **3 calls**, matching the outer row, and 16.440 + 3.889 =
**20.329 ms**, equal to the outer 20.329. Stages sum 14.892 + 2.768 + 1.868 +
0.796 = **20.324 ms**, 0.005 ms from the outer row. Nothing is left unreconciled.

Timed scenes, five processes on this commit: `full_frame` **35.85 ms** median
(spread 35.42–38.63, min 35.27), `open_window_frame` 72.55 ms, lock customize
22.00 ms, primitive glass 1.93 ms. **No hot-path change**: the blur is 14.892 of
20.329 ms (73.3 %) and already a separable two-pass sliding-window blur over
the clipped region, so the finding stands that this is per-pixel work. 35.85 ms
remains 2.2× the 16 ms budget; next blocker unchanged — a structural design
change plus a display path to validate it.
