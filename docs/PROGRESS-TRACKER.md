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
| Frame time, RAM, startup time, GPU use | **PARTIAL — the shipped compose path is measured in §12, §16, §17, §18, §19, §20, §21, §22 and §23 (6.612 ms median; lock-customize 7.998 ms and open-window 11.904 ms are inside the 16 ms budget); display, GPU and idle RAM are not** | §12 corrected the benchmark so it measures the glass path the shell actually runs. Present-to-display, GPU use and whole-machine idle RAM still need a booted image and a display; no QEMU, no image and no root in this workspace |
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
reproduced by `python3 tools/measure-host.py`. On the composition side, §12
replaced the benchmark's drifted scene with the one `Compositor::new` builds;
the next measurable target it exposed is `fill_round_rect` (5.31 ms, 27.1 % of
the profiled shipped frame) and the lock-customize frame (19.65 ms).

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

## 12. Batch: the benchmark was timing a glass path the shell does not run

### 12.1 The overlap question, answered with the real rectangles

`profile::note_rect` records each glass call's rectangle while profiling is on,
and the bench prints them in call order together with every pairwise
intersection:

```text
profile.glass_panel_m.rect=780.8,92.0,435.2x262.5
profile.glass_panel_m.rect=64.0,92.0,435.2x355.1
profile.glass_panel_s.rect=305.0,726.0,670.0x62.0
profile.glass_rects.count=3 profile.glass_rects.overlapping_pairs=0 profile.glass_rects.overlap_px=0 profile.glass_rects.area_sum_px=310320
```

| Rect | Which panel | x range | y range | px |
|---|---|---|---|---|
| `64.0,92.0,435.2x355.1` | Activity window | 64.0–499.2 | 92.0–447.1 | 154,540.7 |
| `780.8,92.0,435.2x262.5` | Environments window | 780.8–1216.0 | 92.0–354.5 | 114,240.0 |
| `305.0,726.0,670.0x62.0` | dock strip | 305.0–975.0 | 726.0–788.0 | 41,540.0 |

The two medium panels sit at opposite top corners: `499.2 < 780.8`, so their x
ranges are disjoint. Both end at or above `y = 447.1`, while the dock starts at
`y = 726.0`. **0 overlapping pairs, 0 shared pixels of 310,320.** No strip is
blurred twice, and no blurred result is reused across panels.

### 12.2 The larger finding: `full_frame` was not the shipped frame

`Compositor::new` blurs and saturates the wallpaper once and attaches it
(`scene.set_backdrop(Some(source))`); every panel then copies out of that
pre-blurred source. `render_bench::build()` built `let scene =
wallpaper.clone();` and attached nothing, so its three glass calls fell into
`glass_backdrop`'s live-blur fallback. Every `full_frame` number in §7–§11
therefore timed a path the desktop does not take.

The same binary already showed the inconsistency: `full_frame` spent **20.3 ms**
in `glass_backdrop`, while `primitive_glass_backdrop_panel` — which *does*
attach a backdrop — costs **1.92 ms** for a 600×420 panel. §11's 73 % blur share
was real, but it belonged to the fallback, not to the shipped frame.

### 12.3 What changed

| File | Change |
|---|---|
| `src/litric-shell/examples/render_bench.rs` | `build()` attaches `wallpaper.glass_source(...)` through `desktop::glass_params`, exactly as `Compositor::new` does; new `full_frame_live_blur` scene detaches the backdrop and times the same frames; the profile block prints per-call rectangles and pairwise overlap |
| `src/litric-shell/src/desktop.rs` | `glass_params` is `pub` so the bench cannot drift from the compositor again; three tests pin the scene's backdrop and which branch each scene profiles |
| `src/litric-shell/src/canvas.rs` | `profile::note_rect` / `take_rects`: one push per glass call while profiling, nothing on a timed frame |

No renderer code changed. The duplicate-work question is moot on the shipped
path (there is no per-frame blur), and the rectangles show no shared strip, so
there is nothing to reuse and nothing to tighten.

### 12.4 Raw profile, shipped path (run 1 of 5)

```text
profile.fill_round_rect.calls=75         ms=5.309  share=27.1 %
profile.glass_backdrop.calls=3           ms=2.499  share=12.8 %
profile.glass_stage_attached.calls=3     ms=2.497  share=12.7 %
profile.soft_shadow.calls=3              ms=2.492  share=12.7 %
profile.stroke_line.calls=2239           ms=2.242  share=11.4 %
profile.glass_panel_m.calls=2            ms=2.163  px=268395  ns/px=8.06
profile.stroke_round_rect.calls=5        ms=1.554  share=7.9 %
profile.glass_panel_s.calls=1            ms=0.335  px=41540   ns/px=8.07
profile.blit_from.calls=1                ms=0.313
profile.draw_argb.calls=10               ms=0.195
profile.timed_ms=19.598
```

Panels 2.163 + 0.335 = **2.498 ms** against the outer 2.499 ms hero row;
`glass_stage_attached` is 2.497 ms of it and **`glass_stage_blur` does not
appear at all** — the shipped frame blurs nothing per frame. The four live-blur
stage rows in §11 remain correct for the fallback, which `full_frame_live_blur`
still measures.

### 12.5 Before and after, five processes

`cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/final > run$i.txt; done`,
then the median of the five `median_ms` lines; min/max are of those medians.

| Scene | Before (§11) | After | Change | Spread (after) |
|---|---|---|---|---|
| `full_frame` (shipped path) | 35.597 ms | **15.076 ms** | −57.6 % | 14.949–15.121 |
| `full_frame_live_blur` (new) | — | 35.367 ms | reproduces the old row | 35.122–35.755 |
| `open_window_frame` | 71.806 ms | 29.654 ms | −58.7 % | 29.596–29.699 |
| `lock_frame_customize` | 22.145 ms | 19.647 ms | −11.3 % | 19.549–19.755 |
| `primitive_glass_backdrop_panel` | 1.924 ms | 1.929 ms | unchanged (always had a backdrop) | 1.924–1.951 |
| `present_full` | 3.907 ms | 3.909 ms | unchanged | 3.652–3.940 |

The live-blur row landing within 0.24 ms of the old `full_frame` is the control:
the only thing that changed in the timed scene is which branch glass takes.
This is a **measurement correction, not an optimization** — the shipped desktop
already did the fast thing; the benchmark had drifted.

### 12.6 Where the frame stands

The shipped composition is **15.076 ms median (14.949–15.121, all five runs
under budget)** against the 16 ms active budget at 1280×800 on this 8-vCPU VM
with no GPU. `present_full` is a separate **3.909 ms** and is not part of that
compose figure. `lock_frame_customize` at **19.647 ms** is still over budget and
is the next named target, with `fill_round_rect` (5.309 ms, 27.1 %) the largest
single entry in the active frame. The golden-frame gate remains **BLOCKED** on
the missing Latin font face fixture — not passing.

Caveats: the profiled frame runs instrumented (19.598 ms against a 15.076 ms
timed frame), so its shares are relative to 19.598 ms; this is composition, not
present-to-display; no GPU or DRM device was measured.

§16 supersedes the `fill_round_rect` part of this section: the same scene now
measures **14.404 ms** median after the run-based fill. The 15.076 ms figure
above is §12's measurement as it stood; §16 records the new runs. §17 lowers the same scene to 12.054 ms by
replacing the translucent core blend with an exact table. §18 adds the missing
per-row attribution for `lock_frame_customize` itself and records three
attempted optimizations that were measured and reverted; `full_frame` stays at
11.927 ms and `lock_frame_customize` at 16.223 ms.

## 13. Batch: browser origin canonicalisation and the queue's load bound

Security pass over the layer that reads untrusted text: the address bar and the
one persisted service file. Two real defects, both fixed with regression tests;
no renderer or shell behaviour changed.

### 13.1 BUG-0033 — a default port was a second origin

`Policy::permission` keys every decision on `url.origin()`, and `Url::parse`
kept an explicit default port. `https://example.com:443/` therefore produced
the origin `https://example.com:443`, not `https://example.com`, so a stored
`Deny` (e.g. Camera) was not found for that spelling and the decision fell back
to `Ask` — the refusal was re-asked instead of enforced. The same parser also
folded a non-numeric port into the host: `https://tracker.test:abc/pixel`
became host `tracker.test:abc`, which `Policy::is_tracker` could not match, so
the navigation was allowed.

Fix in `src/litric-browser/src/url.rs`: a bare IPv6 literal is the only
authority allowed to end in colons; every other port must be non-empty digits
inside `u16` (`BadPort` otherwise), and a port equal to the scheme's default is
dropped, making `origin()` canonical for every caller.

Tests: `a_default_port_is_the_same_origin_as_no_port`,
`a_bad_port_is_refused`, `an_ipv6_literal_keeps_its_colons_and_still_takes_a_port`,
`policy::a_default_port_cannot_step_around_a_refusal`,
`policy::a_tracker_cannot_hide_behind_a_default_port_or_a_bad_one`.

Full record: `docs/bugs/BUG-0033-a-default-port-made-a-second-origin.md`.

### 13.2 BUG-0034 — the queue file ignored `CAPACITY` on load

`Queue::post` enforced the 64-note ceiling, `parse` did not: every readable
record in the file was pushed, so a hand-edited, restored or buggily written
queue loaded an unbounded list. `parse` now truncates to `Queue::CAPACITY` and
adds the dropped records to the skipped count, which the daemon logs. Because
`encode` writes newest first and `parse` pushes in file order, the truncation
drops the oldest — the same order `post` evicts in — and `next_id` still keeps
the highest id seen.

Test: `queue::a_file_holding_more_notes_than_capacity_loads_only_the_capacity`.
Full record: `docs/bugs/BUG-0034-the-notification-queue-file-was-not-capped-on-load.md`.

### 13.3 Verification and what was not fixed

```text
$ cargo test -p litric-browser -p litric-notifyd --lib
litric-browser: 89 passed, 0 failed
litric-notifyd: 47 passed, 0 failed
$ cargo test --workspace
TESTS passed=1643 failed=0        (was 1638; +5 tests in this batch)
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ cargo fmt --check
exit 0
$ python3 ci/check-python-syntax.py        OK: 61 file(s); exit 0
$ python3 ci/lint-line-endings.py          OK: 352 file(s), all LF; exit 0
$ python3 ci/lint-doc-refs.py              OK: 762 citations / 151 docs; exit 0
$ python3 tools/validate_chapter45.py --check   8 checks: 2 PASS, 6 PARTIAL, 0 FAIL; exit 0
$ python3 tools/fix_source_id_keys.py --check   no-op: 4208 rows, 0 duplicates; exit 0
```

**Found and not fixed in §13, fixed in §14:** `url::interpret` returned
`Intent::Search` for an address that carries a supported scheme but fails to
parse, while the comment above it said the user should get an error about the
address instead. That is BUG-0035, closed in the next batch through an explicit
intent and outcome plus the shell's omnibox path.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture. No claim of zero vulnerabilities is made: this pass covered the
address-bar parser, the browser permission/tracker policy and the notification
queue file, not every crate.

## 14. Batch: malformed addresses are reported, not searched (VERIFIED)

BUG-0033's residual risk is closed. `url::interpret` now returns
`Intent::AddressError { input, error }` for text that carries a supported
scheme and fails `Url::parse`; the comment above it and the code agree, and the
choice was to fix the code, not the comment, because searching for a broken
address asks a question nobody asked and sends the text to a provider.

### 14.1 What changed

| File | Change |
|---|---|
| `src/litric-browser/src/url.rs` | `Intent::AddressError`; `interpret` returns it for a supported scheme that does not parse; `validate_host` refuses an unmatched bracket, so `https://[::1` is `BadHost` instead of host `[:` with port 1. Two tests |
| `src/litric-browser/src/lib.rs` | `Outcome::InvalidAddress { input, reason }`; `Session::go` reports it, marks the tab `LoadState::Failed` with `that address is not valid: …`, and does not enter history. Caller-level test |
| `src/litric-shell/Cargo.toml` | `litric-browser` added as a dependency (the shell already depends on `litric-defender`, `litric-files`, `litric-notifyd`) |
| `src/litric-shell/src/apps.rs` | `commit_omnibox` validates every address form through `litric_browser::url::Url::parse` before navigating: `://` input and host-like `example.com:abc` both refused, status line set, tab marked, `None` returned so no engine request is sent. Search URLs are not parsed. Caller-level test |

What the user sees: the browser status line reads
`that address is not valid: <the parser's reason>` and the tab is marked as
failed; the address stays in the omnibox for correction, no navigation is
recorded, and no request leaves for the engine or a search provider.

### 14.2 Behaviour kept exactly as it was

* Bare words and phrases still search (`litric docs` → the configured engine).
* `example.com` still becomes `https://example.com`, byte-for-byte.
* `file:///etc/hostname` and `litric://about` still resolve as before.
* `javascript:alert(1)` and `note: remember this` are still searches: an
  unsupported scheme is text first and an address never.
* Whitespace-only input still returns `None`/`Search("")`.

### 14.3 Verification

```text
$ cargo fmt --check                                exit 0
$ cargo clippy --workspace --all-targets -- -D warnings   exit 0
$ cargo test -p litric-browser --lib              92 passed; 0 failed
$ cargo test -p litric-shell --lib -- malformed_address   1 passed; 0 failed
$ cargo test --workspace                          TESTS passed=1647 failed=0
$ python3 ci/check-python-syntax.py               OK: 61 file(s); exit 0
$ python3 ci/lint-line-endings.py                 OK: 352 file(s), all LF; exit 0
$ python3 ci/lint-doc-refs.py                     OK: 765 citations / 153 docs; exit 0
```

Full record: `docs/bugs/BUG-0035-a-malformed-address-was-searched-for-instead-of-reported.md`.
The corresponding residual-risk bullet was removed from BUG-0033.

### 14.4 Residual

* The shell validated but did not canonicalise: a valid address kept the
  spelling the user typed, because history and bookmark keys were strings.
  **Resolved in section 15 (BUG-0036).**
* The error sentence is English-only, like the rest of the browser status lines;
  it is not in `litric-locale`.
* `Intent` is not `#[non_exhaustive]`, so a future `match` fails to compile
  rather than silently searching — deliberate, and recorded in BUG-0035.
* No pixel test: the shell's error path is proven at the model/status level, not
  by a rendered screenshot.
* The golden-frame gate remains **BLOCKED** on the missing Latin font face
  fixture. This batch makes no performance claim.


## 15. Batch: one site has one spelling in history and bookmarks (VERIFIED)

Closes the residual in section 14.4 and BUG-0035. The shell used the typed text
as the history and bookmark key, so `https://EXAMPLE.com`, `https://example.com`
and `https://example.com:443` were three entries for one page, three Back steps,
and a page could read as unbookmarked right after being bookmarked from another
spelling. `Url::parse` already normalised the parts (host case, a scheme's
default port, an empty path); the shell validated with it and then threw the
result away. The saved file had the same defect: `load_browser_data` deduplicated
the decoded text as written.

### 15.1 What changed

| File | Change |
|---|---|
| `src/litric-browser/src/url.rs` | `to_url` is documented as the canonical text and writes the `//` an empty-host but rooted URL needs, so `file:///etc/hostname` no longer came back as `file:/etc/hostname`. Three tests (two new, one round-trip list extended) |
| `src/litric-shell/src/apps.rs` | `BrowserApp::canonical_address` (parser text, or the input unchanged when it is not an address); `commit_omnibox` stores and returns `parsed.to_url()`; `navigate_active_to` normalises on entry so every stored-address writer shares one key. Two caller-level tests |
| `src/litric-shell/src/main.rs` | `load_browser_data` canonicalises each decoded entry before deduplicating, so a store written before this fix loads as one row per page. One test |
| `src/litric-shell/src/e2e.rs`, `apps.rs` | Four expectations that asserted the old non-canonical text updated to the intended canonical output (not loosened) |
| `docs/bugs/BUG-0036-…md`, this section, BUG-0035 | Evidence, fix, tests, verification and residual risk; BUG-0035's canonicalisation bullet marked resolved here |

No renderer, network, permission or storage behaviour changed. The change is
confined to the text form used as a stored key and returned to the caller.

### 15.2 Behaviour, measured as text

| Typed | Stored, shown and returned |
|---|---|
| `https://example.com` | `https://example.com/` |
| `https://EXAMPLE.com/path` | `https://example.com/path` |
| `https://example.com:443` | `https://example.com/` |
| `file:///etc/hostname` | `file:///etc/hostname` |
| `litric://about` | `litric://about` |
| `litric docs` | the search URL, unchanged |

The path keeps its case: only the host is case-folded, because only the host is
case-insensitive. A search URL and a non-address token are returned unchanged —
they have no canonical address form.

### 15.3 Verification (real output)

```text
$ cargo fmt --check
FMT_EXIT=0
$ cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile; CLIPPY_EXIT=0
$ cargo test --workspace
TOTAL passed=1652 failed=0
  test tests::saved_addresses_load_as_one_canonical_spelling ... ok
$ python3 ci/check-python-syntax.py
OK: 61 python file(s) parse (no bytecode written); EXIT=0
$ python3 ci/lint-line-endings.py
OK: 352 file(s) checked, all LF; EXIT=0
$ python3 ci/lint-doc-refs.py
OK: 774 citation(s) across 155 document(s) resolve; EXIT=0
```

The count rose from 1647 to 1652: two URL tests, two shell tests and one load
test. The four updated expectations are not new tests.

### 15.4 Residual and next

* Percent-encoding is not normalised (`%7E` and `~` stay different keys):
  decoding an encoding can change which resource is addressed, so that is a
  separate decision with its own tests, not a silent addition here.
* A saved file is canonicalised in memory on load and rewritten in canonical
  form on the next save; loading never edits the user's file, which is the
  existing store rule.
* The golden-frame gate remains **BLOCKED** on the missing Latin font face
  fixture; no pixel test covers the bookmark star, and this batch makes no
  performance claim.
* Frame time is unchanged from section 12 (15.08 ms median on the shipped
  compose path); the next measured renderer target remains `fill_round_rect`
  (5.31 ms, 27.1 % of the profiled frame), and the lock-customize frame at
  19.65 ms.


## 16. Batch: the rounded-rect fill derives its geometry once and writes runs (VERIFIED)

`fill_round_rect` was the largest single entry in the shipped compose frame
(§12.4: **5.290 ms over 75 calls, 26.9 %** of a 19.670 ms instrumented frame).
The baseline was re-measured on `1a05e45` before any edit — five separate
`render_bench` processes, 15 frames per scene — and agreed with §12:
`full_frame` medians 14.982 / 15.004 / 15.026 / 15.243 / 15.586 ms.

### 16.1 Root cause (code evidence)

Inside `fill_round_rect`'s pixel loop, every visited pixel:

* called `sd_round_rect`, which re-derived `r.center()` and divided
  `r.w / 2.0`, `r.h / 2.0` and `radius.min(hw).min(hh).max(0.0)` **per pixel**;
* decided its own case with `in_rows && x >= core_x0 && x < core_x1`, even
  though that condition is constant along a row apart from two boundaries;
* wrote through `blend_in`, which paid `index()` (two range checks and a
  multiply), then `px.get_mut` (another check), per pixel;
* in the core, ran the full three-channel float blend per pixel even for an
  opaque colour, although `blend` already returns an opaque source unchanged at
  coverage 1.0.

The cost was therefore not the anti-aliasing: it was per-pixel geometry and
per-pixel addressing, plus a blend that could have been a fill.

### 16.2 What changed

One file, `src/litric-shell/src/canvas.rs`:

* the centre, both half-extent offsets, the clamped radius, and the per-row
  `py` / `qy` / `ay` terms are now derived once (per call / per row);
* each row is split into exactly the three runs the old per-pixel condition
  selected — left edge, core, right edge — so the same pixels are blended once
  each, in the same left-to-right order, with the same arguments;
* writes go through a row slice and the `blend` free function, removing the
  repeated `index()` address arithmetic and bounds check;
* an opaque core is written with `slice::fill(color)`, which is exactly what
  `blend(dst, color, 1.0)` returns for an opaque source;
* the function now reports its visited pixels through `profile::note_pixels`,
  so the attribution row prints `px` and `ns/px` instead of zeros.

No API, allocation or behavioural change. Degenerate and clipped rectangles are
handled by empty runs, not by special cases.

### 16.3 Identical output, proven by the existing reference test

`canvas::tests::the_filled_core_skips_the_distance_and_changes_no_pixel` already
rendered `fill_round_rect` and a verbatim per-pixel reference side by side. It
was extended rather than replaced: 10 rectangles (plain, pill, fractional,
radius larger than half-extent, tiny, zero-width, negative-width, off-canvas),
2 colours (opaque and alpha 0.37) and 4 clips (none, fractional partial,
one-pixel, empty) are compared pixel for pixel. It passes.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture; it is not passing and was not worked around.

### 16.4 Measured result (same protocol, same host)

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/<phase> > run$i.txt; done
```

Values are medians of the five per-process medians; ranges are min–max of those
medians. Raw logs: `evidence/bench/fill-round-rect-2026-10-28/{before,after}/`.

| Scene | Before | After | Change | Verdict |
|---|---|---|---|---|
| `full_frame` (shipped) | **15.026 ms** (14.982–15.586) | **14.404 ms** (14.219–14.577) | −4.1 % | kept; ranges disjoint |
| `open_window_frame` | 30.087 ms (29.510–30.287) | 27.800 ms (27.756–29.090) | −7.6 % | kept; ranges disjoint |
| `lock_frame_customize` | 19.875 ms (19.561–19.920) | 18.950 ms (18.919–19.176) | −4.7 % | kept; ranges disjoint |
| `full_frame_live_blur` | 35.939 ms (35.540–36.409) | 35.324 ms (34.711–35.492) | −1.7 % | kept; ranges disjoint |
| `primitive_fill_panel` (600×420 opaque) | **1.197 ms** (1.193–1.207) | **0.231 ms** (0.227–0.232) | −80.7 % | kept; the direct measure of the change |
| `present_full` | 3.949 ms (3.928–4.026) | 3.918 ms (3.896–3.946) | −0.8 % | **not claimed**: inside the overlapping spread |

The attribution row moved from 5.290 ms with `px=0` to **4.749 ms with
`px=380,749` and 12.47 ns/px**. The primitive result and the frame result agree:
the opaque core stopped being a float blend, and the remaining time is the edge
square root plus the per-pixel blend that translucent fills still need.

Conditions: release build, 1280×800, 8-vCPU VM, no GPU, no display, same host
and session for both phases. The profiled frame runs instrumented (19.231 ms
against the 14.404 ms timed frame), so its shares are relative to 19.231 ms.

### 16.5 Status and remaining distance

**VERIFIED** for the change: output is proven identical by the reference test,
all six scenes were re-measured with the same protocol, and every kept number
comes from ranges that do not overlap.

`full_frame` is now **1.596 ms inside the 16 ms budget** (14.404 ms).
`lock_frame_customize` is still **2.950 ms over** at 18.950 ms. The largest
remaining entries in the instrumented frame are `fill_round_rect` 4.749 ms
(24.7 %), `glass_backdrop` 2.580 ms (13.4 %), `soft_shadow` 2.531 ms (13.2 %)
and `stroke_line` 2.259 ms (11.7 %, 2239 calls). The next unblocked target is
the translucent path of `fill_round_rect` — the opaque core is now a fill, so
what is left is the edge square root and the per-pixel blend for alpha colours
— or `stroke_line`, which has never been attributed below its entry point.

§17 takes this further (12.054 ms) by replacing the translucent core blend
with an exact table; the numbers above are §16's measurement.


## 17. Batch: the translucent rounded fill blends through an exact table (VERIFIED)

After §16 the residual `fill_round_rect` row was **4.638 ms over 380,749 px
(12.18 ns/px)**. This batch measured where that went before changing anything,
and the answer was the per-pixel blend, not the edge square root.

### 17.1 Evidence: a control that isolates the blend

`render_bench` now has `primitive_fill_panel_translucent`: the same 600×420
panel, radius 16 and call site as the existing opaque primitive, painted at
alpha 0.37. Same geometry, same edge coverage, same square roots — only the
core's colour differs.

| Primitive (5 processes, same session) | Median | Range |
|---|---|---|
| `primitive_fill_panel` (opaque) | 0.229 ms | 0.228–0.231 |
| `primitive_fill_panel_translucent` (alpha 0.37) | **2.756 ms** | 2.747–2.763 |

The difference is **2.527 ms over 220,384 core pixels (568×388) ≈ 11.5 ns/px**:
the three-channel float blend at full coverage. Edge work is present in both rows,
so hoisting or approximating the square root could not have moved this number.

Baseline on `a9049f8`, before any renderer edit, with the new primitive present:
`full_frame` **14.709 ms** (14.413–15.385), `open_window_frame` 28.006 ms
(27.733–28.583), `lock_frame_customize` 18.961 ms (18.902–19.039).

### 17.2 What changed

`src/litric-shell/src/canvas.rs`:

* for a translucent fill whose clipped core is at least 1024 px, build a
  256-entry table once per call:
  `table[d] = blend(0xFF00_0000 | d << 16 | d << 8 | d, color, 1.0)`;
* the core loop takes the R byte of `table[r]`, the G byte of `table[g]` and
  the B byte of `table[b]`, then ORs the opaque alpha;
* small translucent cores keep the direct `blend`, so the table build cannot
  dominate a tiny fill; opaque cores keep the `slice::fill` path from §16.

Why the pixels are identical: at full coverage, `blend`'s per-channel result
depends only on that channel's destination value, and every table entry was
produced by calling `blend` itself with that value. Same f32 operations, same
`round_half_up`, same output integer — a table of the function, not an
approximation of it. There is no allocation (a 1 KiB stack array) and no API
change.

`src/litric-shell/examples/render_bench.rs` gained the translucent primitive as
a permanent control, so the next person can re-run this attribution.

### 17.3 Proof of identical output

`canvas::tests::the_filled_core_skips_the_distance_and_changes_no_pixel` was
extended again: 11 rectangles (including a 130×100 rect clipped by the canvas,
zero-width, negative-width and off-canvas), 7 colours (opaque, alpha 0.001,
0.05, 0.37, 0.5, 0.999 and a fully transparent source) and 4 clips (none,
fractional partial, one-pixel, empty), each compared pixel for pixel with the
verbatim per-pixel reference. It passes.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture; it is not passing and was not worked around.

### 17.4 Measured result

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/res-<phase> > run$i.txt; done
```

Medians of the five per-process medians; ranges are min–max of those medians.
Raw logs: `evidence/bench/fill-round-rect-blend-table-2026-10-28/{before,after}/`.

| Scene | Before | After | Change | Verdict |
|---|---|---|---|---|
| `full_frame` (shipped) | **14.709 ms** (14.413–15.385) | **12.054 ms** (11.867–13.422) | −18.1 % | kept; ranges disjoint |
| `open_window_frame` | 28.006 ms (27.733–28.583) | 21.727 ms (21.421–21.840) | −22.4 % | kept; ranges disjoint |
| `lock_frame_customize` | 18.961 ms (18.902–19.039) | 16.418 ms (16.222–16.616) | −13.4 % | kept; ranges disjoint |
| `primitive_fill_panel_translucent` | **2.756 ms** (2.747–2.763) | **0.801 ms** (0.794–0.809) | −70.9 % | kept; direct measure |
| `primitive_fill_panel` (opaque) | 0.229 ms (0.228–0.231) | 0.222 ms (0.221–0.224) | −3.1 % | **not claimed**: the opaque path was not touched; this is layout/noise |

The attribution row moved from **4.638 ms / 12.18 ns/px / 24.3 %** to
**2.135 ms / 5.61 ns/px / 12.9 %**, same 380,749 px.

Conditions: release build, 1280×800, 8-vCPU VM, no GPU, no display, same host
and session for both phases. The profiled frame runs instrumented (16.578 ms
against the 12.054 ms timed frame), so its shares are relative to 16.578 ms.

### 17.5 Status and remaining distance

**VERIFIED**: the measured bottleneck was isolated by a control, the change is
proven pixel-identical by the existing reference pattern, and every kept number
comes from before/after ranges that do not overlap.

`full_frame` is now **3.946 ms inside the 16 ms budget** (12.054 ms).
`lock_frame_customize` is now **0.418 ms over** at 16.418 ms and is the next
target. The largest rows left in the instrumented frame are `glass_backdrop`
2.548 ms (15.4 %), `soft_shadow` 2.436 ms (14.7 %), `stroke_line` 2.244 ms
(13.5 %, 2239 calls), `fill_round_rect` 2.135 ms (12.9 %) and
`stroke_round_rect` 1.553 ms (9.4 %).

## 18. Batch: the lock screen attributed, three optimizations measured and reverted

### 18.1 Why the scene needed its own attribution

`render_bench` timed `lock_frame_customize`, but every call-level profile so far
came from the shipped full-frame scene. Those are different scenes with
different paint: the lock sheet is 62 rounded fills, 1261 short stroke segments,
four glass backdrops and one soft shadow, and none of the earlier row tables
say how they split. Guessing from the full-frame rows would have been a guess.

The bench now keeps the lock scenes timed exactly as before and, after the last
one, turns on the existing `canvas::profile`, sets `lock.edit = true` and
`lock.stage = Entry`, ticks one frame, and prints the sorted rows as
`lock_profile.*`. The profiled frame runs after every timed lock scene, so no
measured frame is disturbed. A `primitive_fill_chip_translucent` control (24x24,
radius 6, alpha 0.12) was added next to the panel control so the small-fill path
is a direct measurement rather than an inference. Both changes are in
`src/litric-shell/examples/render_bench.rs`; no renderer code changed in this
batch.

### 18.2 The measured attribution

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/<phase> > run$i.txt; done
```

Release, 1280x800, 8-vCPU VM, no GPU and no display, 15 frames per scene, five
separate processes per phase; each figure is the median of the five
per-process medians, ranges are min-max of those medians. Raw logs:
`evidence/bench/lock-frame-customize-2026-10-28/`.

Rows from one instrumented customize frame (`evidence/bench/lock-frame-customize-2026-10-28/baseline-2-final/run1.txt`); the
printed row sum is 20.582 ms, and the shares are shares of that sum. The table
includes nested rows (`glass_stage_attached`, `glass_panel_l`,
`glass_panel_s` are inside `glass_backdrop`), so the rows cannot be added up.
The timed frame is 16.223 ms; the difference is instrumentation overhead.

| Row | Calls | ms | Share | Pixels | ns/px |
|---|---|---|---|---|---|
| `fill_round_rect` | 62 | **4.829** | 23.5 % | 573,051 | 8.43 |
| `stroke_line` | 1261 | 3.252 | 15.8 % | - | - |
| `glass_backdrop` | 4 | 2.599 | 12.6 % | - | - |
| `glass_stage_attached` (inner) | 4 | 2.595 | 12.6 % | - | - |
| `stroke_round_rect` | 20 | 2.570 | 12.5 % | - | - |
| `glass_panel_l` (inner) | 1 | 2.263 | 11.0 % | 295,960 | 7.65 |
| `soft_shadow` | 1 | 1.976 | 9.6 % | - | - |
| `glass_panel_s` (inner) | 3 | 0.335 | 1.6 % | 38,160 | 8.79 |
| `blit_from` | 1 | 0.163 | 0.8 % | - | - |

Scene medians on the final tree (no renderer change kept):

| Scene | Median | Range |
|---|---|---|
| `full_frame` (shipped) | **11.927 ms** | 11.819-12.201 |
| `open_window_frame` | 21.593 ms | 21.557-22.387 |
| `lock_frame_customize` | **16.223 ms** | 16.155-16.436 |
| `primitive_fill_panel` (opaque 600x420) | 0.222 ms | 0.221-0.227 |
| `primitive_fill_panel_translucent` | 0.799 ms | 0.796-0.801 |
| `primitive_fill_chip_translucent` | 0.011 ms | 0.011-0.012 |

A first baseline of the same renderer, taken before the chip control existed,
measured `lock_frame_customize` at 16.398 ms (16.237-16.544), so the scene sits
within about 0.2 ms of the budget across independent batches, not at a stable
single value.

### 18.3 Three changes: each attempted, measured, reverted

Every attempt used the same five-process protocol in the same session, against
the baseline taken immediately before it, and was then removed with
`git checkout -- src/litric-shell/src/canvas.rs`. This commit therefore changes
no renderer code: `canvas.rs` is byte-identical to e7530b0.

**A - reject the empty part of each `stroke_line` bounding box.** A per-pixel
capsule test before the projection and square root. `lock_frame_customize`
16.398 -> **16.891 ms** (16.608-17.182) and the `stroke_line` row 3.302 ->
3.623 ms. The lock's segments are short, so their boxes are already mostly
capsule and the extra test costs more than it skips. Reverted.

**B - cache the exact 256-entry blend table by colour.** This would remove the
1024-core-pixel gate that §17 introduced, so a small translucent fill would
reuse the table instead of building it per call or blending directly.
`lock_frame_customize` 16.223 -> **16.174 ms** (16.136-16.442) - the ranges
overlap, the `fill_round_rect` row moved 4.829 -> 4.822 ms (inside noise), and
only the direct chip control moved (0.011 -> 0.010 ms). No disjoint gain, so
reverted rather than kept.

**C - reject provably empty edge pixels without the square root.** In the edge
run, `sqrt(ax^2 + ay^2) >= max(ax, ay)`, so a bound built from `max` can prove a
pixel has zero coverage. `lock_frame_customize` 16.223 -> **16.407 ms**
(16.386-16.799) and the `fill_round_rect` row 4.829 -> 5.154 ms. This is a
regression: the check is paid on every edge pixel, and the edge runs that matter
are short enough that the skipped work does not repay it. Reverted.

### 18.4 What the numbers establish, and what they do not

The overage is **0.223 ms** on the second baseline (16.223 ms) and **0.398 ms**
on the first (16.398 ms); the honest reading is that the scene is about a
quarter to four tenths of a millisecond over, not that 16.223 is exact.

The `fill_round_rect` row is 62 mixed fills whose bounding boxes total 573,051
pixels, 8.43 ns/px. It is not explained by table construction: the isolated
large translucent panel measures 0.799 ms over 252,000 bounding-box pixels
(3.17 ns/px; 3.63 ns per core pixel), and the chip's table is amortised over one
call at 0.011 ms. Sixty-two calls at even 20 us of setup would be 1.2 ms, far
short of 4.829 ms. Attempts B and C rule out two mechanical explanations - the
small-fill table build and the empty stretches of the edge runs - and both made
the number worse, not better. The remaining cost is spread across the per-pixel
edge coverage and blending of many differently sized, mostly translucent fills;
a direct core-versus-edge pixel counter is not emitted, so this batch does
**not** claim a single root cause for the row. That counter is the next
measurement, not a conclusion.

### 18.5 Status and remaining distance

**NOT VERIFIED** for the target: `lock_frame_customize` is still over the 16 ms
budget and no renderer change was kept. **VERIFIED** for the instrumentation:
the attribution above is produced by the shipped bench, reproducible with the
commands in 18.2, and its raw logs are committed.

`full_frame` is **4.073 ms inside** the budget (11.927 ms).
`lock_frame_customize` is **0.223 ms over** at 16.223 ms. The next named target
is the `stroke_line` row (3.252 ms, 15.8 %): the cheap bounding-box reject
failed, so the next attempt must start from a different mechanism (for example
per-segment capsule bounds rather than the inflated box, or an exact
precomputed cap table) and be measured the same way. `glass_backdrop`
(2.599 ms) and `stroke_round_rect` (2.570 ms) follow. The golden-frame gate
remains **BLOCKED** on the missing Latin font face fixture; it is not passing.
No GPU, VRAM, display or present-to-display measurement exists on this host,
and none is claimed.

§19 changes the `fill_round_rect` conclusion in 18.4: the counter it asked
for showed that 49.7 % of that row's pixels were on the square-root edge path,
and a per-row split now writes every full-coverage pixel through the fast fill.
The row falls from 6.245 to 1.564 ms and the lock scene from 16.394 to
12.527 ms.

## 19. Batch: the rounded fill writes full rows, not a radius-shaped edge (VERIFIED)

### 19.1 The counter §18 asked for

§18 could not say where `fill_round_rect`'s 8.43 ns/px went, so this batch
added disjoint buckets to the existing opt-in profile: `interior` (written by
the fast full-coverage path), `edge_scanned` (pixels the distance loop visited),
`edge_painted` and `edge_zero` (the split of the scanned band by whether the
coverage was above zero), `outside_shape` (inside the clipped bounding box but
outside the rounded shape), and `clipped` (cut by the canvas or the clip), plus
counters for rows walked, zero-pixel calls and the largest call. The buckets are
filled by an untimed second pass that uses the same `edge_coverage` expression
as the fill, so a bucket cannot disagree with the painted frame; a timed frame
pays one flag read and takes no timestamps or counting pass.

One instrumentation defect was found and fixed while measuring: the bench's
`take()` empties the timer rows but the new buckets are a separate accumulator,
so the first lock dump inherited the full-frame profile's 380,749 pixels. The
bench now clears buckets and counters before enabling the lock profile, and the
numbers below come from running the pre-change renderer again with that fix
(`before-clean`). The earlier contaminated split (553,321 interior / 400,479
edge) is not used anywhere.

Measured on the pre-change renderer, lock frame only:

| Bucket | Pixels | ns | ns/px |
|---|---|---|---|
| `interior` | 288,272 | 393,685 | **1.37** |
| `edge_scanned` | **284,779** | **4,561,466** | **16.02** |
| of which `edge_painted` | 237,238 | - | - |
| of which `edge_zero` | 47,541 | - | - |
| `outside_shape` | 0 | - | - |
| `clipped` | 0 | - | - |

The two measured slices add up exactly: 288,272 + 284,779 = 573,051, the row's
reported pixel count. Half the row's pixels were paying 16.02 ns/px in the
distance loop while the other half paid 1.37 ns/px, and 83.3 % of the expensive
half was actually painted.

### 19.2 Root cause

The old decomposition called "core" only the rectangle inset by the corner
radius on **all four sides**. For a rounded rectangle that is far too small: the
straight-edge bands, and the middle of the top and bottom bands, already have
`d <= -0.5`, so their coverage is exactly 1 and they do not need a square root
at all. A 600x420 panel with radius 16 ran 16 pixels of each straight edge, and
the middle of each corner band, through `sqrt` and `blend` even though the
answer was the same `blend(..., 1.0)` the fast path writes.

The distance for a pixel whose centre is `u = |x + 0.5 - cx|` from the middle
line is `d(u) = sqrt(max(u - qx_off, 0)^2 + ay^2) - rad`, where `ay` is the
row's distance outside the vertical core. Solving `d = -0.5` and `d = 0.5`
gives two per-row boundaries: one square root per row instead of one per edge
pixel.

### 19.3 What changed

`src/litric-shell/src/canvas.rs`:

* each row is split into a full-coverage span, a partial band on each side, and
  nothing beyond the coverage ramp;
* the full span is written by the existing fast path (`slice::fill` for opaque,
  the exact 256-entry table for large translucent fills, `blend(..., 1.0)`
  otherwise);
* only the partial band runs the distance loop;
* the boundaries carry a quarter-pixel margin in the safe direction: the full
  span is shrunk and the partial band grown, so a boundary pixel that is not
  provably full still goes through the distance test, and only an exactly-full
  pixel can be written by the fast path;
* degenerate cases are unchanged and covered by the existing reference test:
  zero and negative extents, radius greater than half the extent, a circle
  (radius equal to half the extent), rectangles clipped by the canvas, a clip
  that cuts a shape, one-pixel rows, off-canvas rectangles, alpha 0 and 255.

`src/litric-shell/examples/render_bench.rs` prints the buckets and counters in
the existing `lock_profile` dump and clears them before the lock profile.

Why the pixels are identical: the fast span is only used where the coverage is
exactly 1, and `blend(source, colour, 1.0)` returns the source unchanged. For a
translucent colour the table's entries were built by calling `blend` with that
same full coverage, per channel, so the result is the same integer. Pixels on
the boundary of the fast span go through the original distance loop, in the
same left-to-right order, written once each.

### 19.4 Proof of identical output

`canvas::tests::the_filled_core_skips_the_distance_and_changes_no_pixel` is the
existing verbatim per-pixel reference test. It now covers 12 rectangles (the
previous 11 plus a circle of radius 20 in a 40x40 rect, whose full-coverage
span is a disc), 7 colours (opaque, alpha 0.001, 0.05, 0.37, 0.5, 0.999 and
0.0) and 4 clips (none, fractional partial, one-pixel, empty). Every pixel
equals the reference.

`canvas::tests::the_fill_buckets_account_for_every_pixel_it_walks` checks the
new counters against the reference: every pixel whose reference coverage is
above zero is either in `interior` or in `edge_painted`, the buckets add up to
the walked area, and the test rectangle exercises both paths. It passes.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture; it is not passing and was not worked around.

### 19.5 Measured result

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/<phase> > run$i.txt; done
```

Release, 1280x800, 8-vCPU VM, no GPU and no display, 15 frames per scene, five
separate processes per phase; medians of the five per-process medians, ranges
min-max. Raw logs: `evidence/bench/fill-round-rect-row-split-2026-10-28/`
(`before/`, `before-clean/`, `after/`).

| Scene | Before | After | Change | Verdict |
|---|---|---|---|---|
| `full_frame` (shipped) | 11.987 ms (11.813-12.162) | **10.470 ms** (10.373-10.839) | -12.7 % | kept; ranges disjoint |
| `lock_frame_customize` | **16.394 ms** (16.191-16.975) | **12.527 ms** (12.499-12.884) | -23.6 % | kept; ranges disjoint, now under 16 ms |
| `open_window_frame` | 21.556 ms (21.443-22.145) | 20.136 ms (19.955-20.709) | -6.6 % | kept; ranges disjoint |
| `primitive_fill_panel` (opaque) | 0.221 ms (0.221-0.224) | **0.051 ms** (0.050-0.053) | -76.9 % | kept; direct measure |
| `primitive_fill_panel_translucent` | 0.808 ms (0.806-0.810) | **0.327 ms** (0.326-0.332) | -59.5 % | kept; direct measure |
| `primitive_fill_chip_translucent` | 0.011 ms | 0.008 ms | -27 % | kept; direct measure, ranges 0.011 vs 0.008 |

The attribution row moved from **6.245 ms / 573,051 px / 10.90 ns/px** to
**1.564 ms / 527,459 px / 2.96 ns/px**. The pixel count fell because pixels
outside the coverage ramp are no longer visited at all: after the change,
510,782 interior + 16,677 edge_scanned + 45,592 outside_shape = 573,051, and
the row's visited pixels are the first two terms. The after buckets are
`interior` 510,782 at 1.33 ns/px and `edge_scanned` 16,677 at 28.64 ns/px, with
`edge_painted` 14,728 and `edge_zero` 1,949.

The share of the instrumented lock frame changes too: the row is 1.564 ms of a
17.6 ms instrumented sum (8.9 %), down from 6.245 ms of 22.3 ms (28.0 %).

Conditions: release build, 1280x800, 8-vCPU VM, no GPU, no display, same host
and session for all phases. The profiled frame runs instrumented, so its shares
are relative to the printed row sum, not the timed frame.

### 19.6 Status and remaining distance

**VERIFIED**: the counter identified the dominant slice, the change targets that
slice, the existing reference test proves identical pixels on every case
including degenerate and clipped shapes, and every kept number comes from
before/after ranges that do not overlap.

`full_frame` is **5.530 ms inside** the 16 ms budget (10.470 ms).
`lock_frame_customize` is **3.473 ms inside** at 12.527 ms; it was 0.223 ms over
before this batch. `open_window_frame` is 20.136 ms and remains over 16 ms if it
is held to the same budget; it is the next scene with a measured overage.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture. No GPU, VRAM, display or present-to-display measurement exists on this
host, and none is claimed.

§20 applies the same per-row split to the attached glass path: the window
frame's largest row, `glass_backdrop`, falls from 5.430 to 0.625 ms, and
`open_window_frame` from 20.337 to 15.127 ms.

## 20. Batch: attached glass copies its interior instead of blending it (VERIFIED)

### 20.1 Why the window scene needed its own attribution

`render_bench` timed `open_window_frame` but every call-level profile so far
came from the full-frame or lock scenes. The bench now composes one untimed
frame in the same open-window state immediately after the timed loop, with the
existing call timers on, and prints rows, the disjoint pixel buckets and the
frame's damage facts. Buckets and counters are cleared before the profile, the
defect §19 fixed for the lock dump, so this scene cannot inherit the earlier
full-frame profile's pixels.

### 20.2 The measured attribution

Before the change, one instrumented open-window frame (printed row sum
30.462 ms; the timed frame is 20.337 ms, the difference is instrumentation
overhead):

| Row | Calls | ms | Share | Pixels | ns/px |
|---|---|---|---|---|---|
| `glass_backdrop` | 4 | **5.430** | 17.8 % | - | - |
| `glass_stage_attached` (inner) | 4 | 5.427 | 17.8 % | - | - |
| `soft_shadow` | 4 | 4.190 | 13.8 % | - | - |
| `stroke_round_rect` | 16 | 3.657 | 12.0 % | - | - |
| `stroke_line` | 3094 | 3.166 | 10.4 % | - | - |
| `fill_round_rect` | 102 | 2.716 | 8.9 % | 841,611 | 3.23 |
| `glass_panel_l` (inner) | 1 | 2.981 | 9.8 % | 380,000 | **7.84** |
| `glass_panel_m` (inner) | 2 | 2.109 | 6.9 % | 268,395 | **7.86** |
| `glass_panel_s` (inner) | 1 | 0.339 | 1.1 % | 41,540 | **8.16** |
| `blit_from` | 1 | 0.256 | 0.8 % | - | - |
| `draw_argb` | 10 | 0.190 | 0.6 % | - | - |

The four glass calls add up to 5.429 ms, 689,935 panel pixels, all on the
attached path at 7.84-8.16 ns/px. The fill row is the control that shows this is
not the new fill split: its buckets are already 737,035 interior pixels at
1.33 ns/px plus only 104,576 edge pixels at 8.96 ns/px. The dominant contributor
is therefore the attached glass copy, not the fill, shadow or strokes.

### 20.3 Root cause

The attached branch called `sd_round_rect` for every pixel of the panel's
bounding box. That function recomputes the centre, both half-extents, the clamped
radius and two divides per pixel, then takes a square root; the result is fed to
`blend_in`, which recomputes the pixel index and runs the three-channel float
blend. At full coverage, however, `blend(dst, source, 1.0)` returns the source
with an opaque alpha whatever is underneath, so the whole interior is an opaque
copy of the backdrop and needs neither the distance nor the blend.

§9 tried a smaller version of this (skip the distance in the covered core)
and reverted it because the frame gain sat inside the spread. That attempt still
paid a per-pixel lookup and a per-pixel blend. This one writes whole runs from
the backdrop and hoists the geometry to the row, so it does not repeat it.

### 20.4 What changed

`src/litric-shell/src/canvas.rs`:

* the attached branch derives the centre, half-extents and clamped radius once
  per call and splits each row by the same `d = -0.5` and `d = 0.5` boundaries
  as the fill: a full-coverage span, a partial band on each side, and nothing
  beyond the coverage ramp;
* the full span is copied from the backdrop as `0xFF000000 | (source & 0xFFFFFF)`,
  which is exactly what `blend(..., 1.0)` writes;
* only the partial band runs `edge_coverage` and `blend`;
* the quarter-pixel margins shrink the full span and grow the partial band, so a
  boundary pixel that is not provably full still takes the original distance
  test;
* profiling reports `glass_attached.full`, `.partial_scanned`,
  `.partial_painted`, `.partial_zero` and `.skipped` as disjoint buckets, plus a
  rows counter, filled by an untimed pass that reuses `edge_coverage`.

`src/litric-shell/examples/render_bench.rs` prints the window scene's rows,
buckets and counters in the existing dump, after clearing the accumulators.

The degenerate cases are the ones the fill already covers and the new test
exercises: zero and negative extents, radius larger than half the extent, a
circle, fractional panels, panels partly off the canvas, and four clip cases
including an empty clip.

### 20.5 Proof of identical output

`canvas::tests::the_attached_glass_matches_the_per_pixel_distance` compares the
new path with the old branch verbatim (one `sd_round_rect`, one `blend_in` per
pixel inside the shape) over 10 panels and 4 clips: plain, fractional, pill,
radius 500, circle, 0.4-px wide, zero-size, negative-width, off-canvas and
canvas-sized. Every pixel is equal.

The existing 51 canvas tests still pass, including
`glass_with_a_backdrop_ignores_what_is_under_it`, the fill reference test from
§19 and the bucket-accounting test. The golden-frame gate remains
**BLOCKED** on the missing Latin font face fixture; it is not claimed.

### 20.6 Measured result

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/<phase> > run$i.txt; done
```

Release, 1280x800, 8-vCPU VM, no GPU and no display, 15 frames per scene, five
separate processes per phase; medians of the five per-process medians, ranges
min-max. Raw logs: `evidence/bench/open-window-glass-2026-10-28/` (`before/`,
`after/`).

| Scene | Before | After | Change | Verdict |
|---|---|---|---|---|
| `open_window_frame` | **20.337 ms** (20.196-21.232) | **15.127 ms** (14.930-15.502) | -25.6 % | kept; ranges disjoint, now under 16 ms |
| `full_frame` (shipped) | 10.687 ms (10.501-11.199) | **8.178 ms** (8.061-8.543) | -23.5 % | kept; ranges disjoint |
| `lock_frame_customize` | 12.746 ms (12.609-14.295) | **10.060 ms** (10.056-10.164) | -21.1 % | kept; ranges disjoint |
| `primitive_fill_panel` | 0.050 ms | 0.051 ms | +2 % | **not claimed**: the fill path was not changed; inside the spread |
| `primitive_fill_panel_translucent` | 0.335 ms (0.331-0.342) | 0.335 ms (0.332-0.336) | 0 | not claimed; unchanged |
| `primitive_fill_chip_translucent` | 0.008 ms | 0.008 ms | 0 | not claimed; unchanged |

The `glass_backdrop` row falls from **5.430 ms to 0.625 ms** (-88.5 %); the four
panels from 2.981/2.109/0.339 to 0.271/0.314/0.040 ms (0.71/1.17/0.96 ns/px of
panel area). After the change one instrumented window frame prints (row sum
15.975 ms):

| Bucket | Pixels | ns | ns/px |
|---|---|---|---|
| `glass_attached.full` | **686,910** | 336,840 | **0.49** |
| `glass_attached.partial_scanned` | 5,565 | 106,139 | 19.07 |
| of which `partial_painted` | 4,154 | - | - |
| of which `partial_zero` | 1,411 | - | - |
| `glass_attached.skipped` | 11,440 | - | - |

686,910 + 5,565 + 11,440 = 703,915 pixels of bounding box, and the copy path
covers 97.5 % of the glass area. The remaining window rows are now
`soft_shadow` 4.171 ms (26.1 %), `stroke_round_rect` 3.630 ms (22.7 %),
`stroke_line` 3.158 ms (19.8 %) and `fill_round_rect` 2.724 ms (17.1 %).

### 20.7 Status and remaining distance

**VERIFIED**: the dominant row was identified by measurement, the change
targets it, the old implementation is the reference in a per-pixel equality
test, and every kept number comes from before/after ranges that do not overlap.

`open_window_frame` is **0.873 ms inside** the 16 ms budget at 15.127 ms.
`full_frame` is **7.822 ms inside** at 8.178 ms, and `lock_frame_customize`
**5.940 ms inside** at 10.060 ms. All three profiled scenes are now under the
budget. The largest remaining rows in the window frame are the four named
above; none is over budget on its own, and this batch does not change them.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture. No GPU, VRAM, display or present-to-display measurement exists on this
host, and none is claimed. The instrumented sums (30.462 ms before, 15.975 ms
after) are not frame times; the timed frames are 20.337 and 15.127 ms.

## 21. Batch: hoisting the shadow geometry out of its pixel loop (partially verified)

### 21.1 Attribution, measured before the change

The `window_profile` dump from §20 was extended with a shadow classification
pass. It walks the same rectangle with the same expression as the painter, in an
untimed frame, and reports disjoint buckets. One window frame walks
**832,918** pixels for the four `soft_shadow` calls and paints only 137,271 of
them (16.5 %):

| Bucket | Pixels | Share of walked | What it costs in the painter |
|---|---|---|---|
| `hidden` | **605,923** | **72.7 %** | one bounds test per ring; no blend |
| `mixed` | **108,587** | **13.0 %** | full rings plus one partial blend; the dominant painted class |
| `under_panel` | 76,508 | 9.2 % | one distance test, skipped before the ring loop |
| `all_full` | 22,008 | 2.6 % | every ring is full: ring-chain walk only |
| `zero` | 13,216 | 1.6 % | coverage is 0: no blend |
| `partial_only` | 6,676 | 0.8 % | every ring is partial |
| `walked` | 832,918 | 100 % | the six buckets sum to it exactly |

The row itself measures **6.001 ms** in this session's baseline (5.788-6.219
across five processes), against 137,271 painted pixels: **43.7 ns per painted
pixel**, and 79.1 % of those are `mixed`. The skip classes are not the cost;
the painted pixel is, and every painted pixel ran `sd_round_rect` twice - once
against the panel to decide whether the shadow is hidden, once against the
offset shadow rectangle - and each call recomputed the centre, both
half-extents, the clamped radius (two divides) and a square root.

The four bench calls come from `views::draw_windows`: radius
`t.px(chrome.radius)`, spread `t.px(mix(30.0, 18.0))`, offset
`t.px(mix(12.0, 7.0))`, alpha `mix(0.60, 0.40)`, where `mix` interpolates
between the focused and unfocused endpoints by the focus spring's elevation.
The dump does not record the resolved per-call values, which is why they are
named as expressions here rather than as invented numbers.

### 21.2 Root cause and change

`soft_shadow` derived the same geometry inside the pixel loop. The fix hoists
`rad`, the centre, the quarter-pixel offsets and the shadow centre out of the
loop, computes `qx`/`ax` once, and shares that horizontal term between the
panel test and the shadow distance; the base rectangle is built once. Two square
roots remain - they are the distance itself - but the four divides, the centre
arithmetic and the radius clamping are now per call instead of per pixel.

Files: `src/litric-shell/src/canvas.rs` (the loop and its tests), this tracker and
`validation/perf/host-measurements.json`; the shadow classification pass already
shipped in `f1c174d`'s `render_bench.rs` and is used unchanged.

### 21.3 Pixel-identical proof

`canvas::tests::the_hoisted_shadow_geometry_equals_the_distance_function` compares
the inlined expression against `sd_round_rect` with `f32::to_bits()` equality for
both the panel test and the shadow test, over eight geometries - plain,
fractional, radius equal to half the extent, radius 500, radius 0, zero-size,
negative-width and off-canvas - and a 250x200 sample grid each
(**~640,000 bit-exact comparisons**). `the_one_pass_shadow_equals_the_twelve_ring_shadow`
still compares whole canvases, and `the_shadow_buckets_account_for_every_walked_pixel`
asserts the six buckets sum to `walked`.

The extended matrix briefly included a negative-width rectangle in the
twelve-ring comparison and it failed: the one-pass shadow and the twelve-ring
*reference* already disagree for negative extents at `f1c174d`. That is a
pre-existing divergence between the two implementations, unrelated to the
hoist, and is recorded here as an observation rather than fixed in this batch.

### 21.4 Measured result

Identical protocol: release, 1280x800, 8-vCPU VM, no GPU and no display, 15
frames per scene, five separate processes per phase, medians of the five
per-process medians with min-max ranges. Raw logs:
`evidence/bench/soft-shadow-2026-10-28/{before,after}/`.

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/<phase> > run$i.txt; done
```

| Item | Before | After | Verdict |
|---|---|---|---|
| `soft_shadow` row | 6.001 ms (5.788-6.219) | 5.805 ms (5.772-5.864) | **not claimed**: the ranges overlap |
| `open_window_frame` | 15.815 ms (15.509-16.213) | **15.092 ms (15.060-15.195)** | ranges disjoint, 0.908 ms inside budget |
| `full_frame` | 8.602 ms (8.154-9.447) | 8.262 ms (8.111-8.546) | ranges overlap; not claimed |
| `lock_frame_customize` | 10.856 ms (10.251-11.196) | 10.096 ms (10.011-10.698) | ranges overlap; not claimed |
| `primitive_fill_panel` | 0.052 ms | 0.050 ms | not claimed; inside the spread |
| `primitive_fill_panel_translucent` | 0.329 ms | 0.334 ms | not claimed; inside the spread |
| `primitive_fill_chip_translucent` | 0.008 ms | 0.008 ms | unchanged |

The targeted row improves by 0.196 ms (3.3 %) at the median, but its ranges
overlap, and the unchanged scenes moved in the same direction between the two
blocks. The scene-level disjointness is therefore consistent with a block-wide
drift and cannot be attributed to this change alone.

### 21.5 Status and remaining distance

**PARTIALLY VERIFIED.** The change is pixel-identical, removes real redundant
work from the dominant painted class, and regresses nothing; the whole-scene
number is inside budget with disjoint ranges, but the targeted row's ranges
overlap and the unchanged scenes also moved, so no row-level win is claimed.

`open_window_frame` is 0.908 ms inside the 16 ms budget in this session;
`full_frame` is 7.738 ms inside and `lock_frame_customize` 5.904 ms inside. The
remaining largest rows in the window frame are `stroke_round_rect`,
`stroke_line`, `fill_round_rect` and the now-5.8 ms `soft_shadow`; this batch
changes none of them beyond the hoist. The golden-frame gate remains
**BLOCKED** on the missing Latin font face fixture and is not claimed.

## 22. Batch: walking only the stroke's annulus (verified)

### 22.1 Attribution, measured before the change

The `window_profile` dump gained a profiling-only classification pass for
`stroke_round_rect`, using the identical distance and coverage expression as the
painter. One open-window frame walks **832,335** pixels across its 16 stroke
calls:

| Bucket | Pixels | Share of walked | Meaning |
|---|---|---|---|
| `zero` | **812,711** | **97.6 %** | coverage 0; the blend does nothing |
| `partial` | 19,616 | 2.4 % | the anti-aliased outline |
| `full` | 8 | 0.0 % | coverage 1 |
| `walked` | 832,335 | 100 % | the three buckets sum to it exactly |

The row measured **6.883 ms** (6.828-7.416) in this session's baseline. The
stroke walked `r.inflate(width + 1.5)` - the whole rectangle, including every
interior pixel - to paint a 19,616-pixel outline, so 97.6 % of its distance
tests could not change a pixel.

### 22.2 Root cause and change

A rounded rectangle grown by `g` is exactly the same shape's signed distance
shifted by `g`: growing the half-extents and the radius by `g` leaves
`q = |p - c| - (half - radius)` unchanged. That makes the two edges of a stroke
rounded rectangles with every extent and the radius moved by `half + 0.5`.

`stroke_round_rect` now computes, per row, the horizontal extent of the grown
shape and of the eroded one with **one square root each** instead of one
distance test per interior pixel, then walks only the annulus between them,
with a one-pixel outward margin and a two-pixel inward one. Every pixel inside
that annulus still runs the original distance and coverage math, with the
geometry hoisted out of the loop. Rows whose grown shape does not reach the row
are skipped without a test. Negative or non-finite widths, radii and extents
take the original full walk, so degenerate inputs behave exactly as before.

Files: `src/litric-shell/src/canvas.rs` (the stroke and the new
`rounded_row_half_width` helper), this tracker, the measurement record and the
raw logs. `soft_shadow`'s `f32::to_bits` equivalence test and the fill/glass
reference tests are unchanged.

### 22.3 Pixel-identical proof

`canvas::tests::the_stroke_band_walk_equals_the_full_distance_walk` keeps the
pre-change implementation verbatim as `stroke_round_rect_reference` and compares
whole canvases after every draw. The matrix is 13 geometries (fractional,
radius 0, radius equal to half the extent, radius 500, 0.5 px and 9 px pens,
a circle, a one-pixel rectangle, zero and negative extents, off-canvas, a
canvas-sized rectangle) x 4 clips (none, fractional, one-pixel, empty) x 3
colours (opaque, translucent, alpha 0) = **156 full-canvas comparisons**. All 54
canvas tests pass, including the shadow bucket and distance-equality tests.

### 22.4 Measured result

Identical protocol: release, 1280x800, 8-vCPU VM, no GPU and no display, 15
frames per scene, five separate processes per phase, medians of the five
per-process medians with min-max ranges. Raw logs:
`evidence/bench/stroke-round-rect-2026-10-28/{before,after}/`.

```text
cd src && cargo build --release --example render_bench
cd src && for i in 1 2 3 4 5; do ./target/release/examples/render_bench /tmp/perf/<phase> > run$i.txt; done
```

| Item | Before | After | Verdict |
|---|---|---|---|
| `open_window_frame` | 15.444 ms (15.246-15.914) | **12.137 ms (12.074-12.508)** | ranges disjoint; **3.863 ms inside** the 16 ms budget |
| `full_frame` | 8.483 ms (8.165-8.696) | **7.155 ms (6.952-7.507)** | ranges disjoint |
| `lock_frame_customize` | 10.381 ms (10.106-11.295) | **8.376 ms (8.188-8.877)** | ranges disjoint |
| `stroke_round_rect` row | 6.883 ms (6.828-7.416) | **3.790 ms (3.764-3.823)** | ranges disjoint |
| `primitive_fill_panel` | 0.050 ms | 0.051 ms | unchanged within spread |
| `primitive_fill_panel_translucent` | 0.340 ms | 0.341 ms | unchanged within spread |

The row timer includes the profiling-only classification pass, which still
walks the old 832,335-pixel rectangle to produce comparable buckets; a timed
frame does not run it. The timed scene numbers above are the honest measure of
the change. The buckets are intentionally the pre-change classification so the
before and after reports describe the same walk.

### 22.5 Status and remaining distance

**VERIFIED.** The dominant bucket was 97.6 % zero-coverage pixels; the fix
removes them by construction, preserves every painted pixel against the
pre-change implementation, and every kept number comes from before/after ranges
that do not overlap. No profiled scene regressed.

At 12.137 ms, `open_window_frame` is 3.863 ms inside the budget; `full_frame` is
8.845 ms inside at 7.155 ms and `lock_frame_customize` 7.624 ms inside at
8.376 ms. The largest window rows after this change are `soft_shadow` (about
5.8 ms before this batch) and `stroke_line`; this batch does not change them.
The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture and is not claimed.

## 23. Batch: bounding the soft shadow's ring scan (verified)

### 23.1 Attribution, measured before the change

The soft-shadow classification pass gained two profiled counters: how many
times the painter enters its partial-ring loop, and how many of those
iterations actually blend. One open-window frame reports:

| Bucket | Count | Meaning |
|---|---|---|
| `partial_iters` | **949,831** | partial-ring loop iterations |
| `partial_blends` | **64,056** | iterations whose coverage was above zero |
| `mixed` | 108,587 px | pixels with both full and partial rings |
| `all_full` | 22,008 px | every ring covers the pixel |
| `under_panel` | 76,508 px | distance test, then skipped |
| `zero` | 13,216 px | no ring coverage |
| `partial_only` | 6,676 px | every ring partial |
| `hidden` | 605,923 px | row-skipped under the glass backdrop |

**93.3 % of the partial-ring iterations compute coverage, find zero and return
from `blend_in` without touching a byte.** The row measured 7.176 ms
(7.068-7.468) in this session's baseline; the whole open-window frame measured
11.971 ms (11.728-12.247).

### 23.2 Root cause and change

The twelve rings are `spread / 12` apart and coverage ramps over one pixel, so
at most `ceil(12 / spread) + 1` rings can straddle the edge at any pixel. Every
other ring in `0..full_from` has `g <= d - 0.5`, which makes
`coverage(d - g)` exactly zero. The loop now starts at
`full_from - (ceil(1 / step) + 3)`; the three-ring margin keeps it conservative
under floating-point rounding, and a zero `step` (spread 0) keeps the original
full scan. Nothing about which pixels blend, in which order, or with which
coverage changed: the removed iterations were the ones `blend_in` discarded.

Files: `src/litric-shell/src/canvas.rs` (the bound, the profiled counters, the
test). No buffer was added; the change is arithmetic and frees work.

### 23.3 Pixel-identical proof

The pre-change `soft_shadow` is kept **verbatim** in the test module as
`Canvas::soft_shadow_reference`, classification pass included. The new
`the_bounded_ring_scan_equals_the_unbounded_one` compares whole canvases over
12 geometries x 3 clips (none, fractional, empty) x 2 backdrop modes = **72
full-canvas comparisons**, covering spreads 0.5, 2.5, 12, 30, 30.5, 120,
alpha 0 and 1, radius 0, radius larger than the half-extent, fractional
geometry and an off-canvas rectangle. All 55 canvas tests pass, and the
original four-case twelve-ring test is untouched.

A separate observation: the one-pass shadow and the twelve-ring reference
already disagree at `spread 30.5` at `ed689ee` - a one-ulp difference in the
grown rectangle's centre crosses a coverage boundary. That was verified by
running the same case against the unbounded loop, which fails identically; it
is a pre-existing property of the twelve-ring comparison, not of this batch,
and is recorded rather than hidden.

### 23.4 Measured result

Identical protocol: release, 1280x800, 8-vCPU VM, no GPU and no display, 15
frames per scene, five separate processes per phase, medians of the five
per-process medians with min-max ranges. Raw logs:
`evidence/bench/soft-shadow-ring-bound-2026-10-28/{before,after}/`.

| Item | Before | After | Verdict |
|---|---|---|---|
| `soft_shadow` row | 7.176 ms (7.068-7.468) | **6.726 ms (6.618-6.791)** | ranges disjoint; **-6.3 %** |
| `open_window_frame` | 11.971 ms (11.728-12.247) | 11.904 ms (11.453-11.955) | ranges overlap; scene change not claimed |
| `full_frame` | 7.018 ms (6.826-7.109) | 6.612 ms (6.588-6.831) | ranges overlap; not claimed |
| `lock_frame_customize` | 8.128 ms (8.082-8.250) | 7.998 ms (7.922-8.186) | ranges overlap; not claimed |
| `primitive_fill_panel` | 0.051 ms | 0.051 ms | unchanged |
| `primitive_fill_panel_translucent` | 0.336 ms | 0.336 ms | unchanged |

The row improvement is disjoint and directly attributable to the removed
iterations. The scene-level numbers moved down together across the whole block,
including scenes this change does not touch, so no whole-frame win is claimed.

### 23.5 Status and remaining distance

**VERIFIED** for the targeted row: the dominant waste was measured at 93.3 % of
the loop iterations, removed by an exact bound, and the output is identical to a
verbatim copy of the old implementation. Scene-level improvement is inside the
block spread and is not claimed; no scene regressed.

All three profiled scenes remain inside 16 ms: `open_window_frame` 11.904 ms
(4.096 ms inside), `full_frame` 6.612 ms (9.388 ms inside) and
`lock_frame_customize` 7.998 ms (8.002 ms inside). The remaining largest window
rows are `soft_shadow` (6.726 ms), `stroke_line` and `fill_round_rect`; this
batch changes only the ring scan. The golden-frame gate remains **BLOCKED** on
the missing Latin font face fixture and is not claimed.

## 24. Batch: measuring stroke_line and rejecting a row-band walk (negative result)

### 24.1 Ground truth and attribution

`HEAD == upstream == remote tip == 7002db9`, tree clean, `origin/main = c2822ba`
(untouched). A profiled pass was added to `stroke_line` and two call counters to
`fill_round_rect`; both run only while profiling is enabled, so timed frames pay
nothing. One open-window frame reports:

| `stroke_line` | Value |
|---|---|
| calls | **3,094** |
| axis-aligned calls (`dx == 0` or `dy == 0`) | 836 (27.0 %) |
| zero-length calls | 0 |
| fully hidden by the clip | 0 |
| total segment length | 9,599 px (3.1 px per call) |
| walked pixels | 155,310 |
| zero-coverage pixels | **118,663 (76.4 %)** |
| partial / full coverage | 32,753 / 3,894 |
| row, profiled frame | 5.110 ms (5.072-6.373) |

| `fill_round_rect` | Value |
|---|---|
| calls / translucent calls | 102 / 51 |
| walked pixels | 841,611 |
| interior | 737,035 at 1.31 ns/px |
| edge band | 104,576 scanned at 8.93 ns/px, 102,038 painted |
| outside shape / clipped | 39,182 / 1,728 |
| row, profiled frame | 2.747 ms (2.706-4.051) |

The fill is already split at the per-row level and its interior runs at
1.31 ns/px; this batch found no defensible cut there. The line looked different:
76.4 % of its walked pixels paint nothing, all within the rectangular bbox of a
diagonal capsule.

### 24.2 The change that was tried

Each row's painted interval was bounded by the union of the segment's
perpendicular band (within `half + 0.5` of the infinite line) and the two end
discs. That union is a superset of the capsule, and one-pixel margins kept
rounding from excluding a painted pixel. The exact distance and coverage test
for every pixel inside the interval was unchanged.

Parity: the pre-change loop was kept verbatim as `stroke_line_reference` and
compared over 14 geometries (horizontal, vertical, both diagonals, shallow,
steep, fractional, zero length, 1 px, thick and thin pens, off-canvas) x 4 clips
(none, fractional, one-pixel, empty) x 3 colours (opaque, translucent, alpha 0)
= **168 full-canvas comparisons**. All 56 canvas tests passed, so the rewrite
was pixel-identical.

### 24.3 Measured result and why it was reverted

Identical five-process protocol, same session, after the parity tests:

| Item | Before | After (band walk) | Verdict |
|---|---|---|---|
| `stroke_line` row | 5.110 ms (5.072-6.373) | **5.641 ms (5.614-5.802)** | slower at the median |
| `open_window_frame` | 11.613 ms (11.560-12.150) | 12.116 ms (11.932-12.360) | slower |
| `full_frame` | 6.989 ms (6.709-7.091) | 7.018 ms (6.906-7.240) | no gain |
| `lock_frame_customize` | 8.183 ms (7.966-8.281) | 8.235 ms (8.206-8.525) | no gain |
| `fill_round_rect` row (control) | 2.747 ms | 2.746 ms | unchanged, as expected |
| `primitive_fill_panel` (control) | 0.053 ms | 0.051 ms | unchanged |

The change is **reverted**. The measurement explains it: the 118,663 zero
pixels are spread over 3,094 calls (38 per call), so a row band saves only a few
pixels per row while adding a division and up to two square roots per row. With
3.1 px average segment length the setup cost dominates. This is the opposite of
§22, where the same idea removed 97.6 % of 832,335 pixels across 16 large
rectangles, and of §23, where the removed iterations were pure no-ops inside a
loop that was already running.

### 24.4 Status and remaining distance

**NOT VERIFIED for the optimisation; the change was rejected by measurement and
reverted.** The attribution counters are kept because they are prof-only and
make the next attempt cheaper. No speed claim is made, and no profiled scene
changed: `open_window_frame` 11.613 ms (4.387 ms inside 16), `full_frame`
6.989 ms (9.011 ms inside) and `lock_frame_customize` 8.183 ms (7.817 ms
inside).

The largest remaining window rows are `soft_shadow` (~6.7 ms), `stroke_line`
(~5.1 ms, 3,094 mostly-short calls) and `fill_round_rect` (~2.7 ms, 1.31 ns/px
interior). Any further stroke work needs to reduce **per-call** cost, not bbox
area. The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture and is not claimed.

## 25. Batch: the shadow's panel interior skipped as a row interval (verified)

### 25.1 What was measured first

This batch started where §24 stopped: `stroke_line`'s per-call cost. The row was
split so the diagnostic pass no longer hides inside the paint number, and calls
were counted by class. One profiled `open_window_frame` (1280x800, the same
benchmark scene):

| Item | Value |
|---|---|
| `stroke_line` calls | 3,094 |
| axis-aligned (horizontal / vertical) | 836 (287 / 549) |
| zero length | 0 |
| width <= 1 px | 103 |
| fully clipped / partly clipped | 0 / 0 |
| segment length actually drawn | 9,599 px (3.1 px per call) |
| pixels walked | 155,310 |
| of those: zero / partial / full coverage | 118,663 / 32,753 / 3,894 |
| `stroke_line` row, instrumented frame | 5.700 ms |
| of which the diagnostic pass itself | 2.392 ms |

So the row is 42 % instrumentation and about 3.31 ms of painting, i.e. 21 ns per
walked pixel. With 3.1 px average length and 76 % of the box painting nothing,
§24 already measured that cutting the box costs more than it saves; this batch
therefore left the stroke loop unchanged and kept the split, the class counters
and a permanent verbatim `stroke_line_reference` harness (144 full-canvas
comparisons: 12 geometries x 4 clips x 3 colours, including zero length, 1 px,
fractional, off-canvas and a NaN endpoint).

The shadow was the next row, and its buckets named the waste:

| `soft_shadow` bucket (one frame, 4 calls) | px |
|---|---|
| walk box | 832,918 |
| skipped by the inset hide rectangle | 605,923 |
| **under the panel, past the first pixel** | **76,508** |
| mixed (rings straddle the edge) | 108,587 |
| all rings full | 22,008 |
| only partial rings | 6,676 |
| no ring paints | 13,216 |
| real ring blends | 64,056 |

### 25.2 The change

76,508 pixels paid a full rounded-rectangle distance (a square root) only to be
discarded by `panel < -1.0`. That region is the panel's interior eroded by one
pixel, which is exactly a rounded rectangle with both half-extents and the
radius reduced by one, so its half-width per row is one
`rounded_row_half_width` call. The painter now computes that interval once per
row, shrinks it by a pixel on each side, and jumps the columns inside it; the
boundary columns keep the per-pixel test. No pixel's colour arithmetic changed.

### 25.3 Parity

The pre-change implementation is kept verbatim as `soft_shadow_reference`
(per-pixel panel test for every pixel). The new path was compared against it
over 11 geometries x 3 clips (none, fractional, empty) x 2 backdrop states
(detached, attached -- attached is what hides a shadow under its own panel)
= **66 full-canvas comparisons**, including radius 0, radius 500, spread 0 /
0.5 / 1 / 36, zero and negative extents, an off-canvas rectangle and alpha 0.
All matched on every pixel. Canvas tests: 63 passed, 0 failed.

The bucket split is kept exact as well: the interval's overlap with the inset
rectangle is still reported as `hidden`, the rest as `under_panel`, so the
disjoint buckets still sum to the walked box (asserted by
`the_shadow_buckets_account_for_every_walked_pixel`).

### 25.4 Measured result

Same protocol as §23/§24: `cargo build --release --example render_bench`, five
separate processes, 1280x800, 15 frames per scene, median of the five
per-process medians, min-max across processes. Raw logs:
`evidence/bench/stroke-shadow-rings-2026-10-28/{before,after}/`.

| Item | Before | After | Verdict |
|---|---|---|---|
| `primitive_soft_shadow_panel` (600x420) | 1.463 ms (1.443-1.494) | **1.384 ms (1.381-1.394)** | **disjoint, 5.4 % faster** |
| `open_window_frame` | 11.668 ms (11.413-11.912) | 11.462 ms (11.409-12.513) | ranges overlap: no claim |
| `full_frame` | 6.553 ms (6.546-6.794) | 6.651 ms (6.446-6.958) | +0.10 ms at the median, inside the spread |
| `lock_frame_customize` | 7.931 ms (7.848-7.999) | 7.691 ms (7.610-7.888) | ranges overlap: no claim |
| `primitive_fill_panel` (control) | 0.050 ms | 0.050 ms | unchanged |
| `primitive_fill_panel_translucent` (control) | 0.337 ms | 0.336 ms | unchanged |

The claim this batch makes is the primitive row only: every after median is
below every before median. At frame level the change is inside the spread and
is reported as such; `full_frame` is the one row that read slightly worse at the
median while its ranges overlap, which is stated rather than hidden.

### 25.5 Status and remaining distance

**VERIFIED for the shadow interval on the primitive row; the frame-level gain is
NOT VERIFIED.** All profiled scenes are inside the 16 ms budget: open window
11.462 ms (4.538 ms of headroom), full frame 6.651 ms, lock 7.691 ms.

Remaining window rows, measured this session: `soft_shadow` 6.365 ms
(instrumented frame, 2 calls per panel and 4 per frame), `stroke_line` 5.700 ms
of which 2.392 ms is the diagnostic pass (3,094 calls, 3.1 px each), and
`fill_round_rect` 2.783 ms (1.31 ns/px interior). The stroke's remaining cost is
per-pixel and per-call, not bbox area: 76 % of its walked pixels paint nothing
but they are spread 38 per call, so both a row band (§24) and a
closed-form axis specialisation cost more than they save at this length
distribution.

The golden-frame gate remains **BLOCKED** on the missing Latin font face
fixture and is not claimed.

## 26. Batch: the file broker's containment, tested verb by verb (verified)

### 26.1 Why this row, and not another frame

§25 closed the last profiled frame row: every scene in `render_bench` is inside
the 16 ms budget (`open_window_frame` 11.462 ms, `full_frame` 6.651 ms,
`lock_frame_customize` 7.691 ms), and two independent attempts to cut the
stroke's remaining 3.31 ms of paint were rejected by measurement (§24) or
inside the spread (§25). The gate matrix's P0 list is therefore where the next
work is, not the renderer.

The one P0 row in that matrix with a self-contained, verifiable boundary is the
file broker: the shell holds no writable path and every operation crosses LFP/1
to `litric-filesd` (ADR-0019). Its confinement lives in exactly one place —
`LogicalPath::parse` — but only a handful of verbs had adversarial coverage:
`WRITETEXT` and `IMPORT` had a traversal test each, everything else relied on
the shared parser without a test that named it.

Gate [4] of `ci/verify.sh` reports FAIL in a clean checkout (development signing
keys absent). That is setup, not a defect: `tools/gen-dev-keys.sh` exists,
`keys/.gitignore` excludes what it writes, and `docs/GATES.md` already records
the gate as PASS *after* generation. Committing key material to make a gate
green is the one thing the mission forbids, so the gate was left alone.

### 26.2 What was added

`every_verb_refuses_a_path_that_leaves_the_root` in
`src/litric-files/src/server.rs` drives **25 hostile requests** through
`Connection::dispatch`, the same entry point the socket uses:

| Verb | Hostile input | Expectation |
|---|---|---|
| `MKDIR`, `NEWFILE`, `WRITETEXT` | `..%2Fescape`, `%2E%2E%2Fescape`, `a%2F..%2Fb` as the name | `InvalidName` |
| `STAT`, `LIST`, `DU`, `TRASH` | `/Documents/../../escape`, `/..`, `/Documents/..`, `/a/./b` as the path | `InvalidName` |

After the refusals the test inspects the host tree: no `escape` anywhere, and
`Documents` still empty — a refusal that creates an empty directory is still a
write.

### 26.3 What the matrix found

`..%5Cescape` was **accepted**, and that is correct: a backslash is a literal
name byte on Linux, so the request creates one directory whose name contains
it, inside the folder that was asked for. It is safe because it is one
segment; it was simply undocumented and untested. The test now pins both
halves of the rule — encoded separators and dot segments are refused, a
backslash is a name byte that stays inside the root — and contrasts it with
`archive.rs`, which refuses backslashes because an archive entry name may have
been authored on Windows. That asymmetry is deliberate and is now written down
rather than implied.

**No production behaviour changed in this batch.** It is a test-and-document
batch: the boundary already held, and now a regression that moves it fails a
named test instead of passing silently. The mission's rule is that a claim
needs evidence, and "the broker is confined" had less evidence than the
renderer rows did.

### 26.4 Checks

```text
cargo fmt --check                                    exit 0
cargo clippy --workspace --all-targets -- -D warnings exit 0
cargo test --workspace     55 suites, 1,661 passed, 0 failed
python3 ci/check-python-syntax.py                    exit 0 (61 files)
python3 ci/lint-line-endings.py                      exit 0 (352 files, all LF)
python3 ci/lint-doc-refs.py                          exit 0 (787 citations)
bash ci/verify.sh            21 pass, 2 fail, 5 skipped
```

The two failures are gate [4]'s absent development keys (setup: run
`tools/gen-dev-keys.sh`, which is not committed by design) and nothing else;
the five skips are the image build, boot, idle-RAM, cargo and golden-frame
checks that need mkosi, QEMU, a toolchain on PATH or the font fixture.

### 26.5 Limits

Composition and host-level tests only: no QEMU, no image build, no display and
no GPU in this environment, so nothing here is a boot, install or hardware
claim, and no RAM, GPU or present-to-display number was measured. The
golden-frame gate remains **BLOCKED** on the missing Latin font face fixture.

## 27. Batch: the file socket's per-user boundary (verified)

### 27.1 Why this row

§26 closed the broker's containment: every verb refuses a path that leaves
the root, proven by 25 hostile requests. That left the transport itself — how
a client reaches the daemon, and who is allowed to. In the standalone path
(`--socket`), `serve_unix` did `let _ = std::fs::remove_file(socket)`
unconditionally before binding, and bound with whatever the umask allowed.

Two consequences, both real:

* A second daemon unlinked a **live** daemon's socket, bound the same name,
  and the first carried on accepting on an inode no client could reach.
* The socket was created with the caller's umask. A group- or world-accessible
  mode on `/run/litric/files.sock` lets another local user speak LFP/1, which
  is the one boundary ADR-0019 assumes the kernel is holding.

### 27.2 What changed

| Behaviour | Before | After |
|---|---|---|
| Path exists, is a socket, nothing listening | unlinked, then bound | unlinked, then bound (stale socket) |
| Path exists, is a socket, something listening | unlinked, then bound — the live daemon was displaced | `AddrInUse`, live socket untouched |
| Path exists, is not a socket | unlinked — a file the daemon did not create was deleted | `AlreadyExists`, nothing deleted |
| Socket mode | umask default | `0600`; if `chmod` fails the daemon removes its own socket and exits |
| Socket owner | daemon's uid | daemon's uid (unchanged, now asserted) |
| Request buffers | fresh `Vec` + `String` per request | one `Vec` reused per connection, borrowed as `&str` when it is UTF-8 |

`bind_listener` is the one place that decides this; `serve_unix` calls it. The
production path (systemd `Accept=yes`, `--stdio`) does not bind at all, and
`image/mkosi.extra/usr/lib/systemd/system/litric-files.socket` already sets
`SocketMode=0600` — so both paths now state the same rule in the place that
actually creates the socket.

### 27.3 Why the boundary is the kernel's, not a userspace uid check

The first design compared `peer_cred().uid()` with the daemon's uid before
reading a request. It cannot be built here:

* `std::os::unix::net::UnixStream::peer_cred` is unstable on this toolchain
  (1.99.1).
* `getsockopt(SO_PEERCRED)` needs `libc`/`rustix`, and `litric-files` is a
  runtime crate covered by the dependency policy: `ci/dep-policy-allow.txt` is
  empty and the crate is `#![forbid(unsafe_code)]`.

So the check would have cost the file layer its first third-party dependency
or its first `unsafe` block, to duplicate a decision the kernel already makes
for a `0600` socket: `connect` from another uid fails with `EACCES` before a
byte of the request exists. The trade is recorded rather than silently taken.
**Gap:** when `peer_cred` stabilises, an explicit check plus a real cross-uid
integration test belongs here; that is a next step, not a claim.

### 27.4 Tests (all executed)

| Test | What it proves |
|---|---|
| `the_socket_is_owner_only_and_a_live_daemons_socket_is_not_stolen` | mode is exactly `0600`; the socket's uid is the daemon's; a second `bind_listener` on it returns `AddrInUse`; the first listener still answers `connect` |
| `a_stale_socket_file_is_replaced` | a socket file left by a killed process is replaced, and the new socket is also `0600` |
| `a_path_that_is_not_a_socket_is_never_deleted` | a plain file at the path is reported as `AlreadyExists` and its bytes are intact |
| `a_connection_from_the_daemons_own_user_is_served_over_a_real_socket` | end to end over a real `UnixStream`: `HELLO 1` in, `OK ... version=1` out |

**Limit, stated plainly:** the mismatch case could not be *executed*
in-process. Producing a second uid needs root or a second account, which this
environment does not provide, so no `EACCES` was observed. What is executed is
the pair of facts the kernel's decision rests on: the socket is `0600` and it
belongs to the daemon's uid. The refusal is Linux's, not ours, and the unit and
the standalone path now agree on the mode.

### 27.5 The file-safety gate caught the new deletion

The first draft added `remove_file` twice inside `bind_listener` and gate
`[3f]` failed with `2 unreviewed deletion site(s) in src/`. That is the gate
working as designed. The fix was the review step it asks for, not a
suppression: `ci/lint-file-safety.py` now records the site with its reason —
unlinks a stale socket file before bind (after a `symlink_metadata` refusal),
and removes the socket it just bound when making it owner-only failed; never a
user path.

### 27.6 Checks

```text
cargo fmt --check                                      exit 0
cargo clippy --workspace --all-targets -- -D warnings  exit 0
cargo test --workspace         55 suites, 1,665 passed, 0 failed
python3 ci/check-python-syntax.py                      exit 0 (61 files)
python3 ci/lint-line-endings.py                        exit 0 (352 files, all LF)
python3 ci/lint-doc-refs.py                            exit 0
python3 ci/lint-file-safety.py                         exit 0
bash ci/verify.sh              27 pass, 2 fail, 4 skipped
```

1,665 is four above §26's 1,661: the four socket tests.

The two failures have one cause and it is setup, not a defect. `keys/dev/` is
absent because `keys/.gitignore` excludes it and `tools/gen-dev-keys.sh`
generates it; `ci/verify.sh` gate [4] says exactly that, and the mkosi lint
fails on the same absent `SecureBootKey`/`VerityKey` paths named in
`image/mkosi.conf`. Generating key material to turn them green was not done:
the mission forbids committing keys, and a lint that passes only with local
material checks the checkout, not the code.

### 27.7 Limits

No QEMU, no image build, no display and no GPU in this environment: nothing
here is a boot, install or hardware claim, and no RAM, GPU or
present-to-display number was measured. The buffer reuse removes two
allocations per request but was **not** measured — no heap or RSS figure is
claimed for it, only the code change. `read_until` still grows the line buffer
until a newline arrives; `proto::MAX_LINE` (16,384) is enforced *after* the
read, so a same-user client can still make the daemon buffer before the
refusal. With the socket owner-only that is self-DoS only, and it is written
down rather than fixed by changing reader semantics in this batch. The
golden-frame gate remains **BLOCKED** on the missing Latin font fixture, and
ADR-0016's capability-grant persistence is still an open architectural
decision, not part of this batch.
## 28. Batch: LFP/1 frames bounded before they are buffered (verified)

### 28.1 Why this row

§27 made the socket owner-only, so only the daemon's own user can reach it
at all. That removed the cross-user exposure, not the read loop's shape:
`read_until(b'\n', &mut line)` grew the buffer to whatever the peer sent and
only then compared the length with `proto::MAX_LINE`. A client that never
sent a newline made the daemon hold every byte it received, and the refusal
arrived only once the client stopped — memory proportional to input, for a
request the framing rules were going to refuse anyway.

### 28.2 What changed

`read_frame` reads through `BufRead::fill_buf` and stores at most
`FRAME_LIMIT = proto::MAX_LINE + 2` bytes (the 16,384-byte payload plus its
optional `\r` and the `\n` that ends it). Once a line passes that, it stops
storing and drains to the newline without keeping the bytes, so the *next*
request still starts at a line boundary. The reply is unchanged and is still
followed by a served connection.

| Case | Before | After |
|---|---|---|
| 4 MiB line with no newline | buffer grew to 4 MiB | stores at most 16,386 bytes |
| Reply to an over-long line | after the peer stops sending | the same `ERR`, after draining |
| Connection after an over-long line | keeps serving | keeps serving (tested with `HELLO 1` next) |
| Payload exactly `MAX_LINE`, `\n` or `\r\n` | accepted | accepted (both tested) |
| Payload `MAX_LINE + 1`, `\n` | refused | refused |
| Allocation per request | one `Vec`, unbounded growth | same `Vec`, bounded growth |

### 28.3 The one narrowing, stated precisely

The old rule was: strip trailing `\r`/`\n`, then require at most `MAX_LINE`
bytes. The new rule adds: the *raw* frame may not exceed `MAX_LINE + 2`
bytes. The only shape refused now that was accepted before is a line whose
raw framing exceeds 16,386 bytes while its trimmed payload is still within
16,384 — in practice a maximal line followed by two or more `\r` bytes. LFP/1
defines a line ending in `\n`, with `\r\n` tolerated; a run of trailing CRs is
outside the protocol. It is recorded here rather than glossed over.

### 28.4 Tests (executed)

| Test | What it proves |
|---|---|
| `a_request_at_the_line_limit_is_framed_and_one_byte_over_is_refused` | a 16,384-byte payload with `\n` and with `\r\n` is framed (no length refusal); 16,385 is refused; the `HELLO 1` after it is answered `OK 1`, so the drain resynchronised |
| `an_endless_line_is_discarded_instead_of_buffered` | 4 MiB with no newline yields `Frame::Line { overlong: true }`, `line.len() <= MAX_LINE + 2`, and the source is fully consumed |

The first draft of these assertions searched the raw reply for
`"request line too long"`, but LFP/1 percent-encodes the message, so that
string never appears and the negative assertion could not fail. The test now
compares against `pct::encode_field("request line too long")`; the check
started failing against the real wire text, which is how the mistake was
caught.

### 28.5 Checks

```text
cargo fmt --check                                      exit 0
cargo clippy --workspace --all-targets -- -D warnings  exit 0
cargo test --workspace         55 suites, 1,667 passed, 0 failed
python3 ci/check-python-syntax.py                      exit 0 (61 files)
python3 ci/lint-line-endings.py                        exit 0 (352 files, all LF)
python3 ci/lint-doc-refs.py                            exit 0 (798 citations)
python3 ci/lint-file-safety.py                         exit 0
bash ci/verify.sh              27 pass, 2 fail, 4 skipped
```

1,667 is two above §27's 1,665: these two tests. The two verifier failures are
the same absent `keys/dev/` material as §26 and §27 (`tools/gen-dev-keys.sh`
generates it, `keys/.gitignore` excludes it); nothing was generated, weakened
or committed to change that.

### 28.6 Limits, and next steps that need a decision

No QEMU, no image build, no display and no GPU here: no boot or hardware claim
is made. The buffer bound is proven by construction plus the `line.len()`
assertion in the 4 MiB test; **no RSS or heap figure was measured**, so none is
claimed. Draining an over-long line still costs wall time proportional to that
line — memory is bounded, bandwidth is not; closing the connection instead
would change what the shell observes, so it was not done.

Two items are decisions rather than work:

* A userspace peer-uid check (`SO_PEERCRED`) needs `libc`/`rustix`, which the
  dependency policy currently forbids (`ci/dep-policy-allow.txt` is empty).
  Either that policy changes or the kernel's `0600` boundary stays the
  enforcement point.
* ADR-0016 (capability grants are in memory and lost on reboot) is an
  architectural decision, not a patch.

The golden-frame gate remains **BLOCKED** on the missing Latin font fixture.
## 29. Batch: every daemon's socket bound the same way (verified)

### 29.1 Why this row

§27 and §28 fixed `litric-filesd`. An audit of every `UnixListener::bind` in
`src/` found the same two questions unasked in the other daemons:

| Daemon | Live daemon's socket | Non-socket at the path | Mode |
|---|---|---|---|
| `browserd` | **unlinked it** | **unlinked it** | **caller's umask** |
| `notifyd` | **unlinked it** | refused, intact | `0600` |
| `sysd` | **unlinked it** | refused, intact | `0600` |

`browserd` was the worst: the standalone path (`BROWSERD_SOCK`) removed
whatever was at the path before binding, so a second broker displaced a live
one, and it never set a mode, so `/run/browserd.sock` took the umask.

### 29.2 What changed

All three now answer the three cases the same way `litric-filesd` does:

* a socket something still answers on → `AddrInUse`, nothing unlinked (the
  live daemon keeps its clients);
* a path that is not a socket → `AlreadyExists`, nothing deleted;
* a leftover socket nothing answers on → unlinked, then bound, then `0600`;
  if the `chmod` fails the socket just made is removed and the error returned
  rather than serving with wider permissions than intended.

`browserd` gained a `bind_listener` inside `mod broker` and `run` now calls
it; it also propagates "cannot create the socket's directory" instead of
ignoring it, since binding would fail anyway with a less useful error.

### 29.3 Tests (executed)

| Crate | Test | What it proves |
|---|---|---|
| `litric-notifyd` | `a_live_daemons_socket_is_not_replaced_and_the_socket_is_owner_only` | second `bind` is `AddrInUse`; the first still answers `connect`; mode `0600` |
| `litric-notifyd` | `a_stale_socket_is_replaced_and_a_file_is_never_deleted` | stale replaced at `0600`; a plain file is `AlreadyExists` and its bytes intact |
| `litric-sysd` | `a_live_daemons_socket_is_not_replaced_and_the_socket_is_owner_only` | same, over `sysd::server::bind` |
| `litric-sysd` | `a_stale_socket_is_replaced_and_a_file_is_never_deleted` | same |
| `litric-browserd` | `a_live_brokers_socket_is_not_replaced_and_the_socket_is_owner_only` | same, over `broker::bind_listener` (this socket had no mode test before) |
| `litric-browserd` | `a_stale_brokers_socket_is_replaced_and_a_file_is_never_deleted` | same |

Two things this batch learned by measuring rather than assuming:

* The workspace total rose by **4**, not 6: `litric-browserd` is declared with
  `required-features = ["broker"]`, so `cargo test --workspace` never compiled
  the bin and its two tests did not run. They were then run explicitly with
  `-p litric-browser --features broker --bin litric-browserd` (12 passed). The
  first draft of this row would have claimed six tests while running four.
* `cargo clippy --features broker` (also never run before) failed on a denied
  `expect()` in the pre-existing `staging` test helper. The crate denies
  `expect_used`, `unwrap_used` and `panic` crate-wide, so both that helper and
  the new `scratch` helper now use `assert!(…is_ok(), …)`; test-only change, no
  suppression added.

### 29.4 Checks

```text
cargo fmt --check                                                  exit 0
cargo clippy --workspace --all-targets -- -D warnings              exit 0
cargo clippy -p litric-browser --features broker --all-targets -D  exit 0
cargo test --workspace                 55 suites, 1,671 passed, 0 failed
cargo test -p litric-browser --features broker --bin litric-browserd  12 passed
python3 ci/check-python-syntax.py                                  exit 0 (61 files)
python3 ci/lint-line-endings.py                                    exit 0 (352 files, all LF)
python3 ci/lint-file-safety.py                                     exit 0
bash ci/verify.sh                      27 pass, 2 fail, 4 skipped
```

The two verifier failures are unchanged and are setup: absent `keys/dev/`
material (`tools/gen-dev-keys.sh`, excluded by `keys/.gitignore`), which also
fails the mkosi lint on the same missing paths. Nothing was generated,
weakened or committed for them.

### 29.5 Limits, and one finding for the next round

No QEMU, image build, display or GPU in this environment: no boot or hardware
claim, and no RAM/GPU number was measured for this batch. The cross-uid case
is still not executed anywhere — it needs a second uid — so the boundary is
still asserted as "owner + `0600`", which is what the kernel enforces.

One finding is recorded rather than fixed because it is shipped as pending:
`image/units-pending/litric-capd.socket` sets `SocketMode=0666`. The socket is
not installed by the image yet, and the capability broker's own protocol may
authenticate the peer, but a world-connectable socket on a capability broker
deserves a deliberate decision before that unit leaves `units-pending`. That is
the next non-UI row.

The golden-frame gate remains **BLOCKED** on the missing Latin font fixture.
## 30. Batch: the capability socket's mode, and a gate for every socket (verified)

### 30.1 What the audit found

Every `*.socket` under `image/`, with the daemon that serves it:

| Unit | Installed? | `ListenStream` | `Accept` | `SocketMode` before |
|---|---|---|---|---|
| `litric-browser.socket` | yes (`mkosi.extra`) | `/run/litric/browser.sock` | `yes` | `0600` |
| `litric-files.socket` | yes (`mkosi.extra`) | `/run/litric/files.sock` | `yes` | `0600` |
| `litric-capd.socket` | no (`units-pending`) | `/run/litric/capd.sock` | `no` | **`0666`** |

`litric-capd.socket` said the mode was deliberate: "every process on the system
must be able to ASK ... Authorisation happens inside the broker using
SO_PEERCRED plus the caller's cgroup-derived application identity". That
authorisation does not exist anywhere in `src/`:

* there is no `litric-capd` binary — `src/litric-caps` is a pure library whose
  own doc comment says it "contains no I/O" and that a daemon would own the
  socket, persistence and transport;
* `image/units-pending/README.md` records that ADR-0016 decided the broker lives
  **inside** `litric-sysd`, and that these capd units "should be removed rather
  than left to look like they work";
* the broker that does exist serves on `sysd`'s socket, which `litric-sysd`
  binds `0600` (§29) and which has no peer-credential check either — it does not
  need one, because the socket is owner-only.

A world-connectable socket in front of a broker that cannot attribute a peer is
a door with no lock, and a comment that describes a check is not a check.

Two further contradictions were found in the same audit:

* `litric-sysd.service` asked for `RuntimeDirectoryMode=0755` while
  `litric-shell.service` and `litric-notifyd.service` both ask for `0700`;
  whichever unit creates `/run/litric` first sets the mode, and the directory
  holds the shell's Wayland socket — an unauthenticated door to the compositor.
* No unit named `SocketUser=`/`SocketGroup=` for the `Accept=no` listener, so its
  ownership was whatever systemd happened to run as.

### 30.2 What changed

| File | Before | After |
|---|---|---|
| `image/units-pending/litric-capd.socket` | `SocketMode=0666`, no user/group, comment claiming SO_PEERCRED | `SocketMode=0600`, `SocketUser=root`, `SocketGroup=root`, comment now states that the claimed authorisation does not exist and lists what widening would require |
| `image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service` | `RuntimeDirectoryMode=0755` | `RuntimeDirectoryMode=0700`, matching the two units that share the directory |
| `ci/lint-unit-sockets.py` | — | new gate: every socket unit must be `0600`, listen under `/run/litric/`, state `Accept=`, name `SocketUser`/`SocketGroup` when `Accept=no`, and `RemoveOnStop=yes` |
| `ci/verify.sh` | — | runs the new gate and its self-test as `[3i]` |
| `.github/workflows/ci.yml` | — | runs the new gate in CI |

**Path chosen for peer credentials: none added.** The sockets this tree ships
are owner-only, so a userspace uid check would duplicate a decision the kernel
already makes (`EACCES` on `connect`) for the price of the file layer's first
`libc`/`rustix` dependency, which `ci/dep-policy-allow.txt` forbids. The
`0600` mode is therefore the enforcement point and the unit says so; widening
the capd socket now requires the binary, the credential check, tests that prove
the refusal, and a reviewed exception in the new linter — in that order.

### 30.3 Tests and gates that actually ran

```text
python3 ci/lint-unit-sockets.py
  OK: 3 socket unit(s) are owner-only, explicit and removed on stop
python3 ci/lint-unit-sockets.py --self-test
  OK: checker rejects a planted 0666 unit with 4 finding(s)

bash ci/verify.sh
  [3i] socket units declare an owner-only socket
    PASS  socket units are owner-only and explicit
    PASS  socket-unit linter detects a world-connectable socket
  summary: 29 pass, 2 fail, 4 skipped

cargo fmt --check                                                  exit 0
cargo clippy --workspace --all-targets -- -D warnings              exit 0
cargo clippy -p litric-browser --features broker --all-targets -D  exit 0
cargo test --workspace                 55 suites, 1,671 passed, 0 failed
cargo test -p litric-browser --features broker --bin litric-browserd  12 passed
python3 ci/check-python-syntax.py                                  exit 0 (62 files)
python3 ci/lint-line-endings.py                                    exit 0 (353 files, all LF)
python3 ci/lint-doc-refs.py                                        exit 0 (802 citations)
python3 ci/lint-file-safety.py                                     exit 0
```

The two verifier failures remain the absent `keys/dev/` material, which also
fails the mkosi lint on the same missing paths. No key was generated and the
gate was not weakened.

### 30.4 Limits

The new lint reads declarations, not a running system: no QEMU and no image
build here, so it proves what the units say, not what systemd applies at boot.
That is why the daemon-side bind tests from §27–§29 exist alongside it — unit
and code now make the same statement (`0600`), and each is checked where it
lives. No cross-uid test was run (no second uid), so no `EACCES` was observed;
no RAM, GPU or present-to-display number was measured for this batch, and no
claim is made. `litric-capd` remains parked with no binary; the next evidence-
based non-UI gap is to either delete those pending units per ADR-0016 or build
the broker as a separate process with the credential check above — that choice
is architectural, not a patch.

The golden-frame gate remains **BLOCKED** on the missing Latin font fixture.
## 31. Batch: a ceiling on concurrent connections (verified)

### 31.1 What was actually unbounded

Every point where a peer's input causes allocation or queueing in the four
installed daemons, with the cap that applied before this batch:

| Daemon | Peer-controlled allocation | Where | Cap before |
|---|---|---|---|
| `litric-filesd` | request line buffer | `src/litric-files/src/server.rs`, `read_frame` (line 1080), `FRAME_LIMIT` (line 1047) | **16,386 bytes** (`MAX_LINE + 2`) |
| `litric-filesd` | file body read for `READTEXT` | same file, `read_file_capped` (line 865) | cap + 1 byte, caller's `cap` |
| `litric-filesd` | handler thread + its buffers | same file, `serve_unix` (line 1188) | **none** |
| `litric-notifyd` | request line buffer | `src/litric-notifyd/src/server.rs`, `read_line` (line 281) | `MAX_LINE + 1` |
| `litric-notifyd` | note block lines | `src/litric-notifyd/src/proto.rs`, `MAX_BLOCK_LINES` | fixed count |
| `litric-notifyd` | pending queue | `src/litric-notifyd/src/queue.rs`, `Queue::CAPACITY` (line 56) | **64**, evicts oldest |
| `litric-notifyd` | handler thread + its buffers | `src/litric-notifyd/src/server.rs`, `serve` (line 260) | **none** |
| `litric-sysd` | request line buffer | `src/litric-sysd/src/server.rs`, `handle` (line 118) | `MAX_LINE + 1` |
| `litric-sysd` | handler thread + its buffers | same file, `serve` (line 87) | **none** |
| `litric-browserd` | command line buffer | `src/litric-browser/src/bin/browserd.rs`, `serve` (line 1523) | `take(64 * 1024)` |
| `litric-browserd` | frame buffer | same file, line 641 | sized from the engine's pixels, not a peer field |
| `litric-browserd` | concurrent clients | same file, `run` (line 1545) | **1** — the accept loop serves serially |

Framing, bodies and queues were already bounded. The genuinely unbounded path
was concurrency: `filesd`, `notifyd` and `sysd` each spawned one thread per
accepted connection with nothing counting the live handlers, so a client that
opened connections faster than it closed them multiplied a per-handler line
buffer (up to `MAX_LINE + 1` bytes) plus its stack without limit — the one
thing in these daemons whose memory was a function of the peer's behaviour
rather than a constant.

### 31.2 What changed

| File | Change |
|---|---|
| `src/litric-files/src/server.rs` | `MAX_CONNECTIONS = 16` and a `Connections`/`ConnectionSlot` atomic gate; a connection over the cap is answered `ERR limit connection%20limit%20reached` and closed; the slot is held for the handler's lifetime |
| `src/litric-sysd/src/server.rs` | the same gate and `Response::err("limit", …)` refusal |
| `src/litric-notifyd/src/server.rs` | the same gate; this protocol has no error frame, so an over-cap connection is closed without allocating a handler |
| `src/litric-browserd` | unchanged: it already serves one connection at a time |

No new dependency; the gate is `std::sync::atomic` only. Well-behaved clients
see no change: the cap is far above what the shell opens (one LFP/1 connection
per session, plus a probe), and a refusal is a single line then EOF.

### 31.3 Tests that ran

| Test | Where | Result |
|---|---|---|
| `the_connection_cap_serves_exactly_max_connections` | `litric-files` | ok |
| `a_burst_above_the_connection_cap_is_refused_and_the_daemon_keeps_serving` | `litric-files` | ok |
| `the_connection_cap_serves_exactly_max_connections` | `litric-sysd` | ok |
| `the_connection_cap_serves_exactly_max_connections` | `litric-notifyd` | ok |

The unit tests assert against `MAX_CONNECTIONS` itself, so raising the constant
without changing the test fails it. The burst test holds `MAX_CONNECTIONS`
handlers open with an unterminated line, then asserts the next connection gets
`ERR limit`, then frees one slot and asserts a `HELLO 1` is answered `OK 1`.

### 31.4 Measured memory

From the burst test, run with `--nocapture`:

```text
cargo test -p litric-files --lib a_burst_above_the_connection_cap -- --nocapture
connection cap: 16 served + 1 refused, RSS 1324 KiB -> 1476 KiB
```

The RSS reading is `/proc/self/statm` resident pages. **What this does and does
not show, plainly:** it measures the capped daemon under a burst of 17
connections (16 served, 1 refused) — a 152 KiB rise including the test's own
buffers. It is *not* a before/after comparison: measuring the pre-fix tree would
mean rebuilding the previous commit, and the old behaviour has no bound to
measure (its memory is a function of how many connections the peer opens, so any
single number would describe the burst, not the design). What is proven instead
is the bound itself: at most `MAX_CONNECTIONS` handlers exist, each holding at
most `FRAME_LIMIT` bytes for its line, so the daemon's per-peer memory is
`16 × (16,386 + stack)` regardless of how many connections arrive. No
system-wide RAM, boot footprint or cross-uid number was measured or claimed.

### 31.5 Gates (exit statuses enforced, not piped)

```text
cargo fmt --check                                                  exit 0
cargo clippy --workspace --all-targets -- -D warnings              exit 0
cargo clippy -p litric-browser --features broker --all-targets -D  exit 0
cargo test --workspace                 55 suites, 1,675 passed, 0 failed
cargo test -p litric-browser --features broker --bin litric-browserd  12 passed
python3 ci/check-python-syntax.py                                  exit 0 (62 files)
python3 ci/lint-line-endings.py                                    exit 0 (353 files, all LF)
python3 ci/lint-doc-refs.py                                        exit 0 (808 citations)
python3 ci/lint-file-safety.py                                     exit 0 (71 sites, all reviewed)
python3 ci/lint-unit-sockets.py                                    exit 0 (3 units)
python3 ci/lint-unit-sockets.py --self-test                        exit 0
bash ci/verify.sh                      29 pass, 2 fail, 4 skipped
```

1,675 is four above §30's 1,671: these four tests. The two failures are the
known absent `keys/dev/` material (mkosi config and dev key pairs); nothing was
generated or weakened.

### 31.6 Limits and next gap

No QEMU, image build, display or GPU here, so no boot or hardware claim. The cap
bounds *memory*, not throughput: a client that opens 17 connections and keeps
them open still occupies all 16 slots, and the 17th is refused rather than
queued — deliberate, because queueing would reintroduce the unbounded buffer the
cap exists to remove. The concurrency cap is per process; with `Accept=yes`
(`filesd`, `browserd`) systemd starts one process per connection, so on that
path the effective bound is `MaxConnections=` in the socket unit, which is not
set for `litric-files.socket` — the next evidence-based non-UI row is to decide
and test that unit limit, because it is the real ceiling on the installed
socket-activated path.

The golden-frame gate remains **BLOCKED** on the missing Latin font fixture.
## 32. Batch: the socket-activated ceiling, made explicit and enforced (verified)

### 32.1 Correction to §31.6

§31.6 said `MaxConnections=` "is not set for `litric-files.socket`". **That was
wrong**, and this batch's audit is what found it: both shipped `Accept=yes`
sockets already declare the value. The §31 audit grepped a directive list that
did not include `MaxConnections`, so the absence was in the check, not the
units. The error is recorded here rather than edited out of §31.

### 32.2 Every shipped socket and the service behind it

| Unit | Line of `MaxConnections` | `Accept` | `MaxConnections` | `SocketMode` | Service | `TasksMax` | `MemoryMax` |
|---|---|---|---|---|---|---|---|
| `image/mkosi.extra/usr/lib/systemd/system/litric-browser.socket` | 18 | `yes` | **1** | `0600` (line 12) | `litric-browser@.service` | 512 | 1536M |
| `image/mkosi.extra/usr/lib/systemd/system/litric-files.socket` | 14 | `yes` | **8** | `0600` (line 11) | `litric-files@.service` | 16 | 256M |
| `image/units-pending/litric-capd.socket` | — (absent) | `no` (line 24) | not required | `0600` (line 9) | `litric-capd.service` (parked, no binary) | 32 | 24M |

`Restart=` is absent from both `@` units: they are socket-activated, so systemd
starts one process per connection and there is no long-lived unit to restart.
Both `@` services carry `StandardInput=socket` / `StandardOutput=socket`, which
is the per-connection path the previous batch capped in code.

**Before/after for every value: unchanged.** The audit found no unit that lacked
a cap, so no limit was invented. What was missing was any check that this stays
true.

### 32.3 Why those two numbers, and what they cost

* `litric-browser.socket`: one connection, because `litric-browser@.service`
  declares `MemoryMax=1536M` and one instance drives one Chromium engine; a
  second simultaneous instance would double the largest memory reservation on
  the machine for a broker the shell opens once.
* `litric-files.socket`: eight, because each instance carries
  `MemoryMax=256M` and the daemon's own `MAX_CONNECTIONS` is 16 — the standalone
  path's ceiling. Eight is a deliberate half of that: the worst case on this
  path is 8 × 256M = 2 GiB if every instance reaches its own cap, which is a
  number worth having in the record rather than discovering at a bad moment.
* `litric-capd.socket`: `Accept=no`, and systemd states `MaxConnections=` has no
  effect for such sockets, so requiring it there would be cargo cult. The lint
  does check the value if one is ever added.

### 32.4 The new gate

`ci/lint-socket-connection-limits.py` fails when:

* a socket has no `Accept=`;
* an `Accept=yes` socket has no `MaxConnections=` (systemd would apply its
  default of 64, which is what `man systemd.socket` documents and what this
  environment's systemd 255 reports);
* the value is not a plain integer, or is outside `1..=16` (`MAX_CONNECTIONS` in
  `src/litric-files/src/server.rs` — a unit may promise less concurrency than the
  daemon serves, never more).

`Accept=no` sockets are exempt from the requirement, not from the range check.
The `--self-test` writes its own fixtures in a temp directory: one conforming
`Accept=yes` unit, one `Accept=no` unit, one missing the cap, one above it.

It is wired into `ci/verify.sh` as `[3j]` (check plus self-test, exit status
enforced by the same `check` helper as every other gate) and into
`.github/workflows/ci.yml`.

### 32.5 What actually ran

```text
python3 ci/lint-socket-connection-limits.py
  OK: 3 socket unit(s) state their activation mode and concurrency cap
python3 ci/lint-socket-connection-limits.py --self-test
  OK: checker rejects a missing cap and an out-of-range cap (1 + 2 finding(s))
python3 ci/lint-unit-sockets.py
  OK: 3 socket unit(s) are owner-only, explicit and removed on stop
python3 ci/lint-unit-sockets.py --self-test
  OK: checker rejects a planted 0666 unit with 4 finding(s)

systemd-analyze verify image/.../litric-files.socket image/.../litric-browser.socket
  (no output, rc=0)

bash ci/verify.sh
  [3j] socket-activated units bound their concurrency
    PASS  per-connection sockets declare MaxConnections
    PASS  socket-limit linter detects a missing or oversized cap
  summary: 31 pass, 2 fail, 4 skipped

cargo fmt --check                                                  exit 0
cargo clippy --workspace --all-targets -- -D warnings              exit 0
cargo clippy -p litric-browser --features broker --all-targets -D  exit 0
cargo test --workspace                 55 suites, 1,675 passed, 0 failed
cargo test -p litric-browser --features broker --bin litric-browserd  12 passed
python3 ci/check-python-syntax.py                                  exit 0 (63 files)
python3 ci/lint-line-endings.py                                    exit 0 (354 files, all LF)
python3 ci/lint-doc-refs.py                                        exit 0 (818 citations)
python3 ci/lint-file-safety.py                                     exit 0
```

The two failures remain the absent `keys/dev/` material (mkosi config and dev
key pairs); nothing was generated or weakened.

### 32.6 Limits

`systemd-analyze verify` proves the units parse and that systemd accepts the
`MaxConnections=` values; it does **not** run the units. No boot, no QEMU, no
image build here, so no runtime observation of systemd refusing the ninth
connection is claimed — that behaviour is documented in `man systemd.socket` and
encoded in the unit, not measured. No RAM number was measured or claimed this
round. The lint reads declarations; the daemon-side behaviour (one process per
connection, `0600`, connection cap in the standalone path) is covered by the
tests recorded in §27–§31.

The golden-frame gate remains **BLOCKED** on the missing Latin font fixture.
## 33. Batch: the socket-activated worst case is derived by the budget gate (verified)

### 33.1 The gate as found

`ci/check-memory-budget.py` reads `docs/PERF-budget.md` and the units under
`image/mkosi.extra/usr/lib/systemd/system/`. Its idle table's `Enforced by` cell
is a claim (`MemoryMax=140M`) that must be configured for the named unit and be
the value that actually applies along that unit's slice chain
(`EffectiveMemoryMax=` semantics); `TARGET (not enforced: …)` and
`[ACK: reason — BUG-0000]` are printed as `ACKNOWLEDGED`; the budget column is
summed against the `**TOTAL**` row; every shipped `Memory*` cap must appear
somewhere in the ledger; a slice whose limits govern no shipped unit is an
error. Exit 0 on no errors, 1 otherwise; `--strict` also fails on acknowledged
gaps; `--self-test` runs ten fixtures. It is wired at `ci/verify.sh:202` and
`ci/verify.sh:204` (check and self-test).

**What it could not see:** the loader only reads `*.service` and `*.slice`, so
every `*.socket` was invisible, and the `| Unit | MemoryMax |` table (§3b) was
read only to mark those units as "listed" — never to multiply anything. The
installed-path ceiling `MaxConnections= × MemoryMax=` could therefore change
without any checked number moving.

### 33.2 The inputs, with their real locations

| Input | Value | Where |
|---|---:|---|
| `litric-files.socket` `MaxConnections=` | 8 | `image/mkosi.extra/usr/lib/systemd/system/litric-files.socket:14` |
| `litric-files@.service` `MemoryMax=` | 256M | `image/mkosi.extra/usr/lib/systemd/system/litric-files@.service:78` |
| `litric-browser.socket` `MaxConnections=` | 1 | `image/mkosi.extra/usr/lib/systemd/system/litric-browser.socket:18` |
| `litric-browser@.service` `MemoryMax=` | 1536M | `image/mkosi.extra/usr/lib/systemd/system/litric-browser@.service:64` |
| `litric-capd.socket` `Accept=` | `no` | `image/units-pending/litric-capd.socket:24` |
| daemon's standalone handler cap | 16 | `src/litric-files/src/server.rs:1180` (`MAX_CONNECTIONS`) |
| per-handler line ceiling | 16,386 B | `src/litric-files/src/server.rs:1049` (`FRAME_LIMIT`) |

Neither socket declares `Service=`, so systemd's defaults apply:
`<stem>@.service` for `Accept=yes`, `<stem>.service` otherwise. The gate encodes
that rule rather than assuming a name.

Derived worst case, from those numbers only:

```text
litric-browser.socket   1 x 1536M = 1536M
litric-files.socket     8 x  256M = 2048M
                          TOTAL   = 3584M
```

These are ceilings, not usage, and they are not part of the 320 MB idle total.
The 3584M is now written into the ledger (§3b.1) and recomputed on every run.

### 33.3 What changed

| File | Change |
|---|---|
| `ci/check-memory-budget.py` | parses `*.socket` (`Accept`, `MaxConnections`, `Service`), resolves each socket's service and its effective `MemoryMax`, computes `MaxConnections × MemoryMax`, and compares it with the new ledger table row by row and in total. It fails when an `Accept=yes` socket is missing from the table, when the recorded `MaxConnections`/service/`MemoryMax`/worst case disagrees with the shipped units, when a socket-activated service has no `MemoryMax` at all, when the table names a socket the repository does not ship, and when the socket TOTAL is absent or differs. |
| `docs/PERF-budget.md` | new §3b.1 with the socket table above, the derivation rule, and the statement that these are ceilings rather than usage. |
| Self-test | three fixtures added: consistent (passes), `MaxConnections` raised without the ledger (fails), socket missing from the ledger (fails). All thirteen cases pass. |

A real defect was found while doing this, by the change itself: `total_row()`
searched the whole document for `**TOTAL**`, so once a second table had its own
TOTAL the idle check compared 320 MB with 3584 MB. It is now scoped to the idle
table, and the failure it produced is quoted below rather than quietly fixed.

### 33.4 The check actually biting

Temporarily raising the shipped cap, with the real output and exit status:

```text
$ sed -i 's/^MaxConnections=8$/MaxConnections=9/' image/.../litric-files.socket
$ python3 ci/check-memory-budget.py ; echo rc=$?
ERROR          PERF-budget.md (socket TOTAL row): declares 3584M but the shipped MaxConnections x effective MemoryMax sums to 3840M
ERROR          PERF-budget.md:197 (litric-files.socket): the socket-activated worst case and the ledger disagree: recorded MaxConnections=8, shipped 9; recorded worst case 2048 MB, derived 2304M
ERROR            : 2
RESULT: FAIL -- static policy check only; says nothing about measured runtime memory
rc=1
```

Restored to `MaxConnections=8`, same command:

```text
$ python3 ci/check-memory-budget.py ; echo rc=$?
RESULT: PASS -- static policy only: every ledger claim is configured, every shipped cap is accounted for, and every divergence is acknowledged with a review condition
rc=0
```

`git status` after the demo shows only the intended edits (this checker and the
ledger); the unit file is byte-identical to what was shipped.

### 33.5 Gates (exit statuses enforced)

```text
python3 ci/check-memory-budget.py                    rc=0 (PASS)
python3 ci/check-memory-budget.py --self-test        rc=0 (13 PASS, 0 FAIL)
python3 ci/lint-unit-sockets.py                      rc=0 (3 units)
python3 ci/lint-unit-sockets.py --self-test          rc=0
python3 ci/lint-socket-connection-limits.py          rc=0 (3 units)
python3 ci/lint-socket-connection-limits.py --self-test rc=0
cargo fmt --check                                                  exit 0
cargo clippy --workspace --all-targets -- -D warnings              exit 0
cargo clippy -p litric-browser --features broker --all-targets -D  exit 0
cargo test --workspace                 55 suites, 1,675 passed, 0 failed
cargo test -p litric-browser --features broker --bin litric-browserd  12 passed
python3 ci/check-python-syntax.py                                  exit 0 (63 files)
python3 ci/lint-line-endings.py                                    exit 0 (354 files, all LF)
python3 ci/lint-doc-refs.py                                        exit 0 (825 citations)
python3 ci/lint-file-safety.py                                     exit 0
bash ci/verify.sh                      31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material (mkosi config and dev
key pairs); no key was created and no gate was relaxed.

### 33.6 Limits and the decision left to the user

The gate proves the ledger and the units agree; it does not measure a machine.
No boot, QEMU, image build or cgroup reader exists here, so the 3584M is a
configured ceiling, not observed usage, and no RAM number is claimed. No shipped
limit was changed: `MaxConnections=8` for files and `1` for the browser stand.
Whether files should be 4 instead of 8 — halving the socket-activated ceiling
from 2 GiB to 1 GiB — is a product decision, and the number is now written where
that decision can be made against it rather than guessed at.

The golden-frame gate remains **BLOCKED** on the missing Latin font fixture.
## 34. Batch: Defender offers two scans that really cover different folders (verified)

### 34.1 What was wrong

The Defender had one button, "Scan now", and one hard-coded walk in
`start_scan`: the six document folders, depth 8, a 2,000-file cap. There was no
way to ask for a whole-home scan, and no code path that would walk one — so the
application could not do the thing a scan window exists to offer. The brief asks
for **Fast scan / Deep scan** as usable features, not as labels.

### 34.2 What changed

| File | Change |
|---|---|
| `src/litric-shell/src/apps.rs` | `ScanMode` (`Fast`, `Deep`) owns the promise: `roots()`, `depth()`, `file_cap()`, `label()` and a `coverage()` sentence written in user words. `DefenderApp` records the mode it is running in; `begin(mode, total)` starts a run and records it. `MenuAction::DeepScan` added; the desktop menu now offers "Fast scan" and "Deep scan". |
| `src/litric-shell/src/shell.rs` | `start_scan()` is the fast scan; `start_scan_mode(mode)` is the one implementation both modes use. `collect_for_mode(bridge, mode)` walks the mode's roots with the mode's depth and file caps and returns the entries the caps refused, so the result sentence can still say what was not looked at. `collect_files` takes `max_depth`/`max_files` instead of their old constants, so no cap is duplicated in two places. |
| `src/litric-shell/src/views.rs` | Two buttons, "Fast scan" and "Deep scan", the running one accented, both inert while a scan runs; a line under them states what the selected mode covers. |
| `ci/lint-file-safety.py` | the new journey's fixture-root removal is a reviewed entry with its reason, as the gate requires. |

The bounds are unchanged in kind and explicit in value: fast = 6 roots, depth 8,
2,000 files (exactly the old behaviour); deep = home root, depth 24, 40,000
files. Both are path lists, not file contents, and `MAX_SCAN_FILE_BYTES` (64 MiB)
still caps each read, so a scan still cannot grow without a bound.

### 34.3 Tests that ran, by name

| Test | Proves |
|---|---|
| `apps::scan_mode_tests::the_two_modes_promise_different_coverage` | the modes differ in roots, depth, cap, label and coverage sentence |
| `apps::scan_mode_tests::the_scan_queue_caps_are_pinned` | the caps are 2,000 / 40,000 by value, so raising one is a deliberate edit |
| `e2e::journey_defender_fast_and_deep_scans_cover_different_folders` | on a real temp tree, the fast walk does **not** reach a threat nested eleven folders down and the deep walk does; the engine denies that file; clicking the real "Deep scan" button starts the deep mode and the worker finishes with `scanned: 2, threats: 1` |

Measured output:

```text
cargo test -p litric-shell --lib   614 passed; 0 failed
cargo test --workspace            passed=1678 failed=0   (was 1,675)
```

### 34.4 Gates (exit status enforced)

```text
cargo fmt --check                                         0
cargo clippy --workspace --all-targets -- -D warnings      0
cargo clippy -p litric-browser --features broker ... -D     0
cargo test --workspace                                     0 (1,678 passed, 0 failed)
cargo test -p litric-browser --features broker --bin litric-browserd  0 (12 passed)
python3 ci/check-python-syntax.py                          0
ci/lint-line-endings.py                                    0
ci/lint-doc-refs.py                                        0
ci/lint-file-safety.py                                     0 (72 sites reviewed)
ci/lint-unit-sockets.py (+ --self-test)                     0 / 0
ci/lint-socket-connection-limits.py (+ --self-test)         0 / 0
ci/check-memory-budget.py (+ --self-test)                   0 / 0
bash ci/verify.sh                                          31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material. No gate was
relaxed, no test deleted, no dependency added.

### 34.5 Limits, recorded rather than glossed

There is no display, GPU, QEMU or second uid in this environment, so nothing
here claims a rendered screenshot or a booted image. The window evidence is what
a headless frame can prove: both buttons are drawn, register distinct hotspots,
and clicking the deep one runs the deep scan over the real files of a fixture
root. No RAM number is claimed for the deep scan; its memory is bounded by the
path-queue cap and the existing 64 MiB per-file read ceiling, which are the
numbers that were actually changed and asserted.

Next evidence-based non-UI/UX gap, left for a decision rather than guessed at:
the storage surfaces still print the raw filesystem token (`erofs`) beside a
volume. `docs/DESIGN-SYSTEM-v3/06-APP-REDESIGN.md` argues for naming it **and**
explaining it; the brief says the word should not reach the user at all. That is
a copy/design decision with tests on both sides, so it is recorded here instead
of being changed unilaterally.
## 36. Batch: Files create → rename → move → trash, all through the window (verified)

### 36.1 What was broken, with the code that proves it

The toolbar Paste button did nothing. `Hot::Files(h)` in `shell.rs` sent every
Files hotspot through `crate::apps::files_action`, and that function deliberately
returns `None` for `FilesHot::Paste` (`apps.rs`, the
`FilesHot::New | FilesHot::Filter | FilesHot::Paste => None` arm): paste needs the
clipboard and the destination folder, which the model does not own. The button
was drawn and registered, and the click was swallowed.

The second defect was in the row menu: `MenuAction::Paste` always built
`FilesOp::PasteCopy`, even when the clipboard held a **cut**. A cut followed by
menu-paste would therefore try to copy and fail with `NotFound`, so "move" could
never happen from the menu. Ctrl+V and the action list already chose correctly,
which is why the same operation appeared to work from one surface and not another.

The trash row was not broken: `MenuAction::Trash` already asked for confirmation
and `perform_file_op` moves the path through the bridge synchronously. The earlier
headless attempt failed because it clicked a row before the listing had settled,
so the selection was empty and no confirm dialog was ever raised.

### 36.2 The fix

| File | Change |
|---|---|
| `src/litric-shell/src/shell.rs` | one `Shell::paste_clipboard()` is now the only paste path: it reads `(cut, path)` from the clipboard, chooses `FilesOp::PasteCut` or `PasteCopy`, and asks for confirmation with the item and destination named in plain words. The toolbar `Hot::Files(FilesHot::Paste)`, the row-menu `MenuAction::Paste`, `Action::FilesPaste` and Ctrl+V all call it. |
| `src/litric-shell/src/copy.rs` | `place_name()` names a place or item the way the window does (`/` → "This Litric", `/Documents/Journey` → "Journey"), so the confirmation reads `Move “Journey” into “Documents”?` instead of printing internal paths. |
| `src/litric-shell/src/e2e.rs` | the journey is now `journey_this_litric_creates_renames_moves_and_trashes`: create → rename → cut → open Documents → paste → confirm → trash → confirm, each step a real click or key event. |
| `ci/lint-file-safety.py` | the reviewed deletion entry follows the renamed journey. |

No test-only hook, no fixture special case, no polling, no thread, no new
per-frame work: paste is still the event-driven dialog → `perform_file_op` path,
and the diff adds only two small strings per user action.

### 36.3 Tests, with the state each assertion observed

| Step | Assertion on the fixture's real disk, through `FsBridge` |
|---|---|
| create | `/Trip` exists |
| rename | `/Journey` exists and `/Trip` does not |
| cut → Documents → paste → confirm | `/Documents/Journey` exists and `/Journey` does not |
| row menu → Move to trash → confirm | `/Documents/Journey` no longer exists, and `trash_list()` contains an entry named Journey |

```text
e2e::journey_this_litric_creates_renames_moves_and_trashes   ok
apps::tests::a_files_place_click_navigates_and_pins           ok (root row pinned by name)
copy::tests::places_are_named_the_way_the_window_names_them    ok
cargo test --workspace        passed=1686 failed=0   (was 1,685)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material (mkosi config, dev key
pairs). No gate relaxed, no test deleted, no dependency added.
## 37. Batch: Defender Fast and Deep scans, proven through the window (verified)

### 37.1 What was already wired at `17f7c85`, and what was not

Both buttons existed and both really scanned. `MenuAction::ScanNow` and
`MenuAction::DeepScan` (`apps.rs`) reach `Shell::start_scan_mode` (`shell.rs`),
which walks the mode's roots and depth through `collect_for_mode`, resolves host
paths on the bridge thread, and hands the jobs to the `litric-defender-scan`
worker. `Shell::scan_step` drains eight finished verdicts per tick into
`DefenderApp`, so a scan cannot lengthen a frame. What was **not** ready for a
user:

- Every verdict was an engine token. `DefenderApp::record` produced `Clean`,
  `Unknown (hash-match-unavailable)` and `DETECTED [Hash] eicar-test` — exactly
  the internals the brief forbids in a window.
- A scan could not be stopped. Both mode buttons became inert `Loading`
  controls while the worker ran and nothing replaced them, so the only exit was
  to wait.
- The result sentence did not say which mode produced it, and the only journeys
  asserted the queue depth and one deep result — neither proved that a clean
  file is left unflagged, nor that the row the window draws is the scanner's
  real verdict for a planted threat in a real folder.

### 37.2 The fix

| File | Change |
|---|---|
| `src/litric-shell/src/apps.rs` | `record()` now writes the three human verdicts (`Safe`, `Threat found`, `Not checked`) and drops the tier, signature id and reason from the row; the scan report still carries them for the audit trail. `MenuAction::CancelScan` added. |
| `src/litric-shell/src/shell.rs` | `ScanShared::cancelled` is checked by the worker between files, so a stop costs at most the file in hand. `Shell::cancel_scan()` raises it and says `Stopping the scan…`; the finish sentence is prefixed with the mode (`Fast scan: 1 threat found in 2 files`) and a stopped scan says `… stopped after N files`. |
| `src/litric-shell/src/views.rs` | While a scan runs, a live `Stop` control is drawn beside the inert mode buttons and registered as `MenuAction::CancelScan`. |
| `src/litric-shell/src/e2e.rs` | `journey_defender_scans_a_real_folder_fast_and_deep` (one session: open Defender → Fast → Deep, every step a click) and `journey_a_running_defender_scan_can_be_stopped_from_the_window` (the Stop control is drawn and its click reaches the worker). |
| `ci/lint-file-safety.py` | the new journey's fixture-root removal is a reviewed entry with its reason. |

No test-only hook, no fixture special case, no bypass: the journeys plant real
files and click the controls the window draws.

### 37.3 Tests, with the state each one observed

| Test | Observed |
|---|---|
| `e2e::journey_defender_scans_a_real_folder_fast_and_deep` | Clicking Fast scan queued exactly the two top-level files (`Scanning { done: 0, total: 2 }`) and finished `Done { scanned: 2, threats: 1 }`; the planted `eicar.com` row read `Threat found` with `detected`, `clean.txt` read `Safe` and was not flagged, the threat nested ten folders down was absent from the fast rows, and the sentence read `Fast scan: 1 threat found in 2 files`. Clicking Deep scan queued three files and finished `Done { scanned: 3, threats: 2 }`; `eicar-nested.com` was now listed and flagged, the sentence began `Deep scan`, and every rendered verdict was one of `Safe`, `Threat found`, `Not checked` with no `Hash`, `DETECTED`, `Deny`, `Allow`, `skipped`, `stat:` or `erofs`. |
| `e2e::journey_a_running_defender_scan_can_be_stopped_from_the_window` | With a worker attached and a scan running, the frame registered `MenuAction::CancelScan`; clicking it set the worker's `cancelled` flag and the status line read `Stopping the scan…`. |
| `e2e::journey_defender_scans_and_reports_eicar_end_to_end`, `e2e::journey_defender_fast_and_deep_scans_cover_different_folders` | still pass unchanged. |

```text
cargo test --workspace        passed=1688 failed=0   (was 1,686; +2 journeys)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material (mkosi key layout,
dev key pairs). No gate relaxed, no test deleted, no dependency added.

### 37.4 Resources

The scan worker already existed and is unchanged in kind: one thread per scan,
bounded result rows (every detection plus at most 200 clean samples), one file
in memory at a time under the unchanged 64 MiB cap, eight verdicts drained per
tick. The diff adds one `AtomicBool` load per file for cancellation, one button
while a scan runs, and no polling, no `thread::spawn`, no unbounded buffer and no
idle per-frame work.

### 37.5 Limit

There is no display, GPU or QEMU here, so nothing claims a screenshot or a boot.
Cancellation is proven at the window and worker-flag level; it was not observed
stopping a multi-minute scan mid-file on a real machine.
## 38. Batch: a real local page opens, renders and scrolls in the Browser (verified)

### 38.1 What the Browser could actually do at `93117b7`

Nothing rendered a page. `litric-browser/src/engine.rs` has exactly one engine,
`NoEngine`, whose `load` returns a sentence naming the missing engine; the shell's
real engine path is Chromium over `litric-browserd`, and this image has no
Chromium binary, so `views.rs` drew the honest placeholder ("No engine session
yet…"). What *was* real: URL parsing and policy, the tab/history model, and a
complete baseline JPEG decoder (`litric-browser/src/jpeg.rs`) with no caller that
put its pixels on screen.

That meant the mission's comic-page claim had no evidence behind it: no text was
laid out, no picture was decoded into a frame, and no wheel moved a page.

### 38.2 What changed

| File | Change |
|---|---|
| `src/litric-browser/src/html.rs` | New bounded parser: headings, paragraphs, `<br>`, `<img src alt width height>`, `<hr>`, `<title>`; entities decoded; script/style/head contents removed; unknown tags keep their text. Limits: 4 MiB input, 600 blocks, 8,000 characters per block. |
| `src/litric-shell/src/reader.rs` | New `Reader`: reads a `file://` page with `std::fs`, lays it out at the page width (heading/body styles from the design tokens), decodes pictures with the existing JPEG decoder **only when a picture's block is inside the viewport**, clamps scrolling, and draws text, rules and pixels. Picture cache: at most 4 decoded pictures and 8,000,000 decoded pixels (32 MiB), at most 2 new decodes per frame, 16 MiB per picture file. |
| `src/litric-shell/src/canvas.rs` | `blit_argb`: nearest-neighbour blit of `0xFFRRGGBB` pixels, so a decoded picture costs the area it covers on screen, not its own size. |
| `src/litric-shell/src/shell.rs` | `send_go` routes `file://` to the reader instead of the broker (`open_local_page`), all other addresses still go to the engine; the wheel over the page surface scrolls the reader (`reader_scroll`), clamped. Status line reads `Opened <title>` or `Could not open this page` — no path, format or code. |
| `src/litric-shell/src/views.rs` | The page area draws the reader when its address is the focused tab's; engine frames are untouched. |
| `src/litric-shell/src/e2e.rs` | `journey_browser_opens_a_real_comic_page_and_renders_it`: one session — open Browser, click the omnibox, type the address, Enter, assert drawn text pixels, panel pixels and cache state, wheel down, assert the second chapter and second panel. |
| `ci/lint-file-safety.py` | the journey's fixture-root removal is a reviewed entry. |

No test-only hook, no fixture special case: the journey writes a real HTML file
and copies a real JPEG from the decoder's corpus beside it, then drives the real
input path.

### 38.3 What the journey observed

| Step | Observation |
|---|---|
| type `file:///tmp/…/page.html`, Enter | `reader` is present, status is exactly `Opened Comic Page`, tab title is `Comic Page` |
| first frame | heading pixels differ from the page background in the drawn band; `visible_text` contains `Chapter One` and not `Chapter Two`; exactly **1** picture decoded (the visible one) |
| first panel | sampled pixels differ from the background, so decoded panel bytes reached the frame |
| wheel down 10 lines (400 px) | scroll = 400; `visible_text` now contains `Chapter Two`; **2** pictures cached, still ≤ 4 and ≤ 8,000,000 pixels; the second panel's pixels are in the frame |
| status text | contains no `file://`, `jpeg`, `html`, `erofs` or `error` |

```text
e2e::journey_browser_opens_a_real_comic_page_and_renders_it   ok
litric-browser::html tests (4)                                 ok
litric-shell::reader tests (3)                                 ok
cargo test --workspace        passed=1696 failed=0   (was 1,688; +8)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material (mkosi key layout, dev
key pairs). No gate relaxed, no test deleted, no dependency added.

### 38.4 Resources, stated exactly

* One picture file at a time in memory, capped at 16 MiB.
* Cache: `MAX_CACHED_IMAGES = 4`, `MAX_CACHED_PIXELS = 8_000_000` (32 MiB),
  `MAX_DECODES_PER_FRAME = 2`; eviction is oldest-first.
* No polling, no `thread::spawn`, no unbounded buffer. The decode pass allocates
  nothing when no new picture becomes visible; the draw pass borrows the layout
  instead of cloning it.
* A frame's picture cost is the area on screen (nearest-neighbour blit), never
  the decoded picture's size.

### 38.5 The limit, plainly

This is the **local document path**, not a web engine. A real `https://` fetch
needs Chromium through `browserd` (absent in this image) or a TLS stack this
project does not ship; no dependency was added to fake either. The proven claim
is: a real file on disk is read, parsed, laid out, its baseline JPEGs decoded
and drawn, and scrolled by the wheel, through the window — with the web path
unchanged and still honestly reporting that no engine is integrated. Image
support is the decoder's subset (baseline/extended sequential JPEG; progressive,
CMYK and arithmetic are refused by name), and there is no display or GPU here, so
no screenshot or compositor presentation is claimed.

## 39. Batch: a finished download opens in Files and is scanned (verified)

### 39.1 What was disconnected

The Browser's Downloads panel listed the real files the file service had
imported into `/Downloads`, but every row was inert: the panel said a
download "can be opened from either window" while only the Files window could
open it. The list was drawn (`src/litric-shell/src/views.rs`, the Downloads
branch of `draw_browser`) with no hotspot at all, so a user who clicked a
finished download got nothing. Files and Defender were each reachable on their
own, but the workflow the mission asks for — download, manage, scan — had no
bridge at the point where the user starts it.

### 39.2 The connection

| File | Change |
|---|---|
| `src/litric-shell/src/shell.rs` | `Hot::BrowserDownloadItem(usize)` opens the Files window at `/Downloads` and asks it to select the clicked file by name. No second copy of the download list and no invented state: the panel already reads that folder through the Files bridge. |
| `src/litric-shell/src/apps.rs` | `FilesApp::select_name` selects the row when it is already listed; otherwise it records `pending_selection`, which `apply_entries` spends when the next real directory read arrives. The listing always clears the selection (rows moved), so a caller in another window needs this hand-off rather than writing `selected_rows` before the refresh wipes it. |
| `src/litric-shell/src/views.rs` | Each Downloads row is a real target with the standard hover tint, so the affordance matches the row's behaviour. |
| `src/litric-shell/src/e2e.rs` | `journey_a_download_opens_in_files_and_is_scanned`: one session — open Browser, click Downloads, click the row for a real file on disk, see Files open that folder with the file selected, then run the Defender's Fast scan and find a planted threat beside it. |
| `ci/lint-file-safety.py` | the journey's fixture cleanup is a reviewed entry. |

The scan needs no new scope code: `/Downloads` is already one of the fast
scan's documented roots, so the downloaded file is scanned by the existing
worker and the existing verdict model.

### 39.3 What the journey observed

| Step | Observation |
|---|---|
| click Downloads | two real files listed (`report.txt`, `eicar.com`) from the fixture's `/Downloads` |
| click the `report.txt` row | `files.path == "/Downloads"`, `selected_rows` contains `report.txt`, and the Files window is focused |
| click Fast scan | the scan finishes, `eicar.com` is detected, `report.txt` is listed and not flagged |

```text
e2e::journey_a_download_opens_in_files_and_is_scanned   ok
cargo test --workspace        passed=1697 failed=0   (was 1,696; +1)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material. No test deleted,
no gate relaxed, no dependency added.

### 39.4 Resources and limits

The click does one window open, one folder navigation and one selection set —
no polling, no thread, no per-frame work. The scan is the existing worker.

This proves the local workflow (a real file on disk in `/Downloads` opened from
the Browser panel, then scanned). It does not prove a network download: this
image still has no Chromium engine and no TLS stack, so no `https://` page was
fetched and the broker's staging path could not be exercised end to end here.

## 40. Batch: Defender contains a finding and restores it (verified)

### 40.1 What was missing

"Quarantine" and restore existed only as plans: a designed API
(`docs/API-00-contracts.md`, `defender_quarantine` / `defender_restore`), a
phase goal asking for quarantine with restore, and a mention of a planner. No
move, record on disk, list or restore existed in the shell. The Defender window
drew each finding as text with no hotspot (`draw_defender` in
`src/litric-shell/src/views.rs`), so a finding could be reported but not acted
on.

### 40.2 The one service both halves use

| File | Change |
|---|---|
| `src/litric-shell/src/quarantine.rs` | The single owner of containment: `quarantine`, `restore`, `list`, `in_scope`. It moves through `FsBridge` — the same validated backend Files uses — and writes one record per entry. |
| `src/litric-shell/src/shell.rs` | `refresh_quarantine` reads the list when Defender opens and after every move; `quarantine_finding` / `restore_quarantined` run one service call, then refresh from what the filesystem actually contains. Wired to `MenuAction::QuarantineFinding` and `MenuAction::RestoreQuarantined`. |
| `src/litric-shell/src/views.rs` | A real `Quarantine` button on each in-scope detection, and a `QUARANTINED` section listing the entries read from disk, each with a real `Restore` button. Both are ordinary hotspots; the hover tint comes from the shared button. |
| `src/litric-shell/src/apps.rs` | `DefenderApp::quarantined` holds what the records say; `MenuAction` carries the two actions. |
| `ci/lint-file-safety.py` | The fixture cleanups of the new tests are reviewed entries. |

Rules the service enforces, all with a plain sentence for the window:

* only paths under the Fast scan's roots (`/Documents`, `/Downloads`,
  `/Pictures`, `/Music`, `/Videos`, `/Projects`) — Deep scan findings in system
  folders are reported but not moved, because moving the running OS is not a
  repair;
* links and folders are refused; the path is checked with `symlink_metadata`
  and canonicalised against the real root before anything moves;
* a move never overwrites: the name is numbered until free, the bridge's
  `write_text` is create-new and its `rename` refuses an existing name, and the
  name the bridge actually used is verified against the one chosen;
* a record is written before the move is reported as done. If it cannot be
  written, the file is moved back and the window says the containment failed —
  or says plainly that the file is in the Quarantine folder and was not
  recorded, if even that rollback fails;
* `restore` takes the original path only from the record, refuses a record that
  names a path outside the protected folders, refuses an occupied original
  path, and refuses a missing original folder. Any refusal leaves the
  quarantined copy intact;
* `list` reads records from disk and drops a record whose file is gone, so the
  list is what is contained now. A folder that exists but cannot be read is an
  error, not "nothing quarantined".

The quarantine folder is `/Quarantine` at the storage root, outside every scan
root, so a scan does not re-report what is already contained. `/Downloads` is
still in the protected scope, so a downloaded file can be contained.

### 40.3 What the journeys observed

`journey_defender_quarantines_and_restores_a_real_threat` drives only the
window: open Defender, click Fast scan on a fixture with `clean.txt` and
`eicar.com`, click `Quarantine` on the finding row, then `Restore` on the
contained row. It asserts, against the real filesystem and the window's own
state:

* the source is gone from `/Documents`, the file is in `/Quarantine`, and
  `eicar.com.quarantine` exists;
* the window's list has one entry naming `/Documents/eicar.com`, the message is
  "Quarantined …", and the finding row is no longer listed as loose;
* the clean file is untouched;
* a second shell opening Defender, with no shared memory, sees the same one
  entry — the list comes from disk;
* after Restore the file is back at its original path with identical bytes, the
  quarantine file and record are gone, the list is empty and the message is
  "Restored …".

`journey_defender_refuses_to_quarantine_a_file_that_is_gone` removes the file
after the scan and clicks the same control: the window says
"Could not quarantine: that file could not be found", no quarantine folder is
created, the finding stays listed (the user is not told a stale finding was
contained), and the clean file is untouched.

The unit tests in `quarantine.rs` cover the cases a window cannot reach: an
out-of-scope path (source untouched), two containments of the same name (two
separate entries, never an overwrite), restore onto an occupied original path
(copy kept), a record naming a system path (refused), and a link (refused, both
link and target untouched).

```text
cargo test --workspace        passed=1704 failed=0   (was 1,697; +7)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the known absent `keys/dev/` material (the mkosi key
layout and the dev key pairs). No test deleted, no gate relaxed, no dependency
added.

### 40.4 Resources and limits

A click does one directory listing (when the window opens) or one move plus one
listing (after a move) — no polling, no `thread::spawn`, no per-frame work, no
unbounded buffer. The list is capped by what is in the folder and the view
draws at most 20 entries. Containment is a rename on one filesystem; the
existing bridge falls back to a copy only across devices.

This proves the local containment path with real files. It does not prove a
network download being contained (no Chromium engine and no TLS stack in this
image), and it does not quarantine system files found by Deep scan: those are
reported only, by design.

## 41. Batch: quarantine records are tamper-evident (verified)

### 41.1 What was missing

The v1 record held the original path, time, reason and size, but no digest.
`restore` checked the name, scope, the original path and its folder, then moved
the contained file back — so bytes altered or swapped inside `/Quarantine` would
be put back silently. `list` rebuilt entries from the same record and had no
concept of integrity.

The workspace already had a hash implementation: `litric-defender`'s
`sha256` module (streaming `Sha256::update`/`finish`, an 8 KiB read pattern, and
FIPS known-answer vectors). No dependency was added; the shell uses that module.

### 41.2 What changed

| File | Change |
|---|---|
| `src/litric-shell/src/quarantine.rs` | Record format 2 adds the file's SHA-256. `quarantine` hashes the real bytes before the move; `verify` hashes a contained file on demand; `restore` verifies before moving and again after, refuses a mismatch and keeps the contained copy, and reports a legacy record as unverified instead of pretending. |
| `src/litric-shell/src/views.rs` | Each contained row shows its real state (`Verified`, `No integrity record`, `Does not match`, `Not checked`) and has a real `Check` control beside `Restore`. |
| `src/litric-shell/src/shell.rs` | `MenuAction::VerifyQuarantined` runs one check and updates that entry; restore messages distinguish a verified restore from a legacy one and from a file put back but unconfirmed. |
| `src/litric-shell/src/apps.rs` | `Quarantined` carries the digest and integrity state; the new action is in `MenuAction`. |
| `src/litric-shell/src/e2e.rs` | The existing quarantine journey now also asserts the on-disk digest and clicks `Check`; two new journeys cover tampering and legacy records. |

How the record is written (v2):

```text
LITRIC-QUARANTINE 2
<64 lowercase hex characters>
<size in bytes>
<time in seconds>
<reason, one line>
<original path>
```

A v1 record (`LITRIC-QUARANTINE 1`, no digest line) is still parsed. It appears
as `No integrity record`, restores successfully, and the window says
"Restored <name> (no integrity record)" — it does not claim a check that never
happened. A record with an unreadable digest line is dropped rather than
guessed at.

### 41.3 What the journeys observed

* `journey_defender_quarantines_and_restores_a_real_threat` — after clicking
  `Quarantine`, the record on disk contains
  `275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f`, the
  published SHA-256 of the EICAR test string and therefore independent of the
  product's own hasher. Clicking `Check` shows `Verified`; `Restore` puts back
  identical bytes and empties the list.
* `journey_defender_refuses_to_restore_a_tampered_quarantine_entry` — after
  quarantining EICAR, the test replaces the contained bytes with
  `swapped bytes`. `Restore` shows "Could not restore: the contained file no
  longer matches what was quarantined, so it was not restored"; the original
  path stays empty; the tampered copy is still in `/Quarantine` with its bytes
  intact; the entry stays listed. Clicking `Check` then shows `Does not match`.
* `journey_defender_restores_a_legacy_quarantine_entry_unverified` — a v1 record
  lists as `No integrity record`, restores the file, and shows "Restored old.bin
  (no integrity record)".

The new unit tests cover the same rules without a window: a correct digest
round-trips and verifies, tampered bytes are refused and kept, a legacy record
restores unverified, and the existing refusal tests still pass (the tampered
record in the overwrite test is now explicitly a legacy record).

```text
cargo test --workspace        passed=1708 failed=0   (was 1,704; +4)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the unchanged absent `keys/dev/` material (the mkosi key
layout and the dev key pairs). No test deleted, no gate relaxed, no dependency
added.

### 41.4 Hashing: where, on which thread, and the honest cost

Hashing happens in exactly two places: `quarantine` before it moves a file, and
`verify` / `restore` on the same thread as the click. It is never in the draw
path, never a poll and never a background task. `hash_path` streams the file
through a fixed 8 KiB stack buffer with `Sha256::update`, so peak memory is the
buffer, not the file size. Time is O(file size): a very large contained file
would occupy the UI thread for the duration of the check or restore. That is a
real limit, not a measurement claim; no worker thread was added for it.

Integrity is recorded for new entries only. Files and records are still stored
in plain form — this batch adds detection of tampering, not encryption.

### 41.5 Limits

Unchanged and stated plainly: no Chromium engine and no TLS stack, so a network
download still cannot be exercised here; no QEMU/display, so the proof is the
headless frame and the real filesystem; Deep-scan findings in system folders
remain report-only. A v1 record cannot be retroactively verified; it is reported
as unverified rather than upgraded with a digest.

## 42. Batch: Activity can select and end a real process (verified)

### 42.1 What the audit found

The Activity window drew a process table with no row targets and no action. The
Task Manager model had `Change::EndTask`, `EndTree` and `EndGroup`, and an
`act()` that said "Ending process <pid>" — but nothing outside `taskmgr.rs`
called it, nothing sent a signal, and `litric-sysd` has no process-signal
request. The model advertised a capability the product did not have.

No TODO/FIXME/unimplemented markers were found in `src`. The cross-app paths
already verified in §§36-41 (Files, Defender, downloads, Browser, quarantine)
are unchanged by this batch.

### 42.2 The wiring

| File | Change |
|---|---|
| `src/litric-shell/src/process.rs` | New same-user, same-session termination service. Reads `/proc/<pid>/status` for the owner and `/proc/<pid>/stat` for the kernel session, refuses pid 0, pid 1, this desktop and anything in another user or session, then runs the absolute `/bin/kill` (or `/usr/bin/kill`) with `-TERM` and the validated numeric pid. No shell, no `PATH` lookup. |
| `src/litric-shell/src/activity.rs` | `row_bands` publishes the hit rectangle of every process row the table draws, using the same geometry as the drawing code. Group headings are skipped. |
| `src/litric-shell/src/shell.rs` | `Hot::ActivityRow(pid)` selects by pid (keeping the model's selection), `Hot::ActivityEndTask` runs one confirmation then the service, and `activity_end_confirm` holds the armed pid. Clicking another row disarms it. |
| `src/litric-shell/src/views.rs` | An **End task** button beside the process search, disabled with no selection and labelled **Confirm end** while armed. The model's live message is drawn beside it, and every process row is a real target. |
| `src/litric-shell/src/e2e.rs` | `journey_activity_ends_a_real_process_after_confirmation`: spawns a real `sleep 30`, inserts its real `/proc` identity, clicks its row, clicks End task once (asserts the child is still alive), clicks again, then waits for the real child to exit. |

### 42.3 The rule, stated precisely

The kernel already decides who may signal whom. This batch adds only what makes
the button's promise honest: the target must have the same effective uid **and**
the same kernel session id as the desktop. That refuses the machine's services
even when the shell itself runs as root, and refuses another logged-in user.
`SIGTERM` is asked for, not `SIGKILL`; the window says "Asked <name> to stop",
never that it exited. If the process ignores the request, the next facts sample
still shows it and the table is the truth.

Ending a whole tree was **not** exposed in this batch; §43 supersedes this
paragraph with a real, bounded tree action and removes the unused model
variants. Privileged process control (another user, another session) remains
unsupported.

### 42.4 Resources

No polling, no thread, no background work: one confirmed click reads two small
`/proc` files and runs one `kill`. The row hit bands are one bounded vector of
the rows already on screen, built only while an Activity window draws. Nothing
is computed when Activity is closed.

### 42.5 Gates

```text
cargo test --workspace        passed=1712 failed=0   (was 1,708; +4)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the unchanged absent `keys/dev/` material. No test deleted,
no gate relaxed, no dependency added.


## 43. Batch: Activity ends a real process tree (verified)

### 43.1 What the audit found

§42 gave the Activity window one real action and left the Task Manager model with
three process-action variants (`EndTask`, `EndTree`, `EndGroup`). None had a UI
consumer; `act()` returned a sentence and performed no work. `ConfirmEndTask` was
a preference no window read, because the confirmation is unconditional. This
batch implements the tree action, then removes every unused variant instead of
leaving the model advertising capabilities the product does not have.

Process-group termination was removed rather than guessed at: a kernel process
group can contain processes started by another user or session, and signalling
one as a shortcut would bypass the per-node rule below. There is no group button
to press and no group implementation to trust.

### 43.2 The wiring

| File | Change |
|---|---|
| `src/litric-shell/src/process.rs` | Adds `EndTarget`, `TreeEnd`, a bounded post-order walk over `/proc/<pid>/task/*/children`, and `terminate_tree`. `terminate` and `terminate_tree` share one `standing` rule: same effective uid and the same kernel session as the desktop, never pid 0 or 1, never this desktop, never another user or session, never a zombie. The tree is signalled deepest first, then the selected process. |
| `src/litric-shell/src/shell.rs` | `activity_end_confirm` now carries the pid **and** the promise (`EndTarget`), so arming End task cannot confirm End tree. `Hot::ActivityEndTree` dispatches through the same handler; selecting another row disarms both. The message reports the real count: asked, plus left alone and could-not-be-asked when they are non-zero. |
| `src/litric-shell/src/views.rs` | A second real control, **End tree**, beside **End task**. Both are disabled with no selection, each shows **Confirm end** / **Confirm tree** while armed, and the message is drawn beside them. |
| `src/litric-shell/src/taskmgr.rs` | Removes `Change::EndTask`, `EndTree`, `EndGroup` and the unconsumed `ConfirmEndTask` preference with their registry entries, `act`/`apply`/reset arms and test references. The model no longer claims process actions it cannot perform. |
| `src/litric-shell/src/e2e.rs` | `journey_activity_ends_a_real_process_tree_after_confirmation`: spawns one real parent (`sh`) with two real `sleep` children, inserts the parent's real `/proc` identity, clicks its row, asserts the first click only arms and every process is still running, clicks again, then observes the parent exit and both children stop running. `journey_activity_refuses_a_process_outside_its_scope`: selects init and proves the tree request is refused in plain language with `/proc/1` untouched. |

Unit coverage also gained `a_tree_is_terminated_from_the_leaves_up` and
`a_tree_will_not_start_from_init`.

### 43.3 The rule, the honest count and the bounds

Every node is checked individually; there is no shortcut for “the tree of a
process I was allowed to signal”. A setuid child, a child that changed session,
or init appearing in a child list is counted and left running, never signalled.
`SIGTERM` is requested, never `SIGKILL`; the window says the processes were
*asked* to stop, not that they exited.

The walk visits at most **256** processes, stops at depth **32**, and reads at
most 256 entries from any one children file. A waiting parent can exit by itself
once its children are stopped; in that case it is not counted as asked, because
the window reports what the system accepted, not what the walk intended.

### 43.4 Resources

No polling, no thread and no background work. A tree read and its signals happen
only after the second confirmation click, on the UI thread: bounded `/proc`
reads, one absolute `kill -TERM` per node that is still there. A single-process
end still reads two small `/proc` files and runs one `kill`. Nothing is computed
while Activity is closed.

### 43.5 Gates

```text
cargo test --workspace        passed=1716 failed=0   (was 1,712; +4)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the unchanged absent `keys/dev/` material. No test deleted,
no gate relaxed, no dependency added, no privileged verb introduced. A process
that ignores `SIGTERM` remains running and the next facts sample still shows it;
there is no forced kill and no claim of one.


## 44. Batch: Activity shows an end request's outcome on the next frame (verified)

### 44.1 What the audit found

The table was owned by `Shell::sample`, but the only sampler was `gather_facts`
in `main.rs`, called once at startup and then every `FACTS_INTERVAL_MS`
(2,000 ms). The confirmed End task / End tree handler ran the signal, wrote its
message and asked for a redraw, but did not sample: the rows always waited for
the next two-second tick. A second audit found two stale-path defects in the
same area: `TaskManager::observe` kept the old selection index when the selected
process vanished, so a later action could target a different row, and a
previous action's message survived a selection change.

### 44.2 The wiring

| File | Change |
|---|---|
| `src/litric-shell/src/facts.rs` | New home for the one facts reader. `gather_facts` moves here as `gather(socket, root)` with the reply parsers, GPU/root/network/battery readers and `ask`/`field`; `DEFAULT_SOCKET` names the system socket. Nothing about what is read changed. |
| `src/litric-shell/src/shell.rs` | `Shell` owns `sysd_socket` and `facts_root` (set by the runner, defaulting to the system socket and `/`) and exposes `refresh_facts()`, which calls the same `gather` and then `sample`. The confirmed End task / End tree handler calls it once after an accepted signal, before the message and redraw. Selecting another process row clears the old message. |
| `src/litric-shell/src/main.rs` | Sets the two fields from `--root` and the daemon socket; startup and the periodic loop call `shell.refresh_facts()` instead of the removed local sampler. The interval, the loop and its budget are unchanged. |
| `src/litric-shell/src/taskmgr.rs` | `observe` resolves the selection by pid after replacing the rows and clears it when that pid is gone, instead of leaving an index that now points at another process. New unit test covers the vanished selection. |
| `src/litric-shell/src/e2e.rs` | The tree journey asserts the fixture starts with exactly one known row, then, immediately after the confirming click and before any periodic tick, asserts the table is a real `/proc` sample: more than one row and the test process itself present. The refusal journey asserts the refused action did not claim a sample. |
| `src/litric-shell/src/lib.rs` | Publishes the new module. |

### 44.3 Cost and honesty

One extra sample per accepted confirmed action: one daemon query or one direct
`/proc` read, on the UI thread, through the reader the periodic loop already
uses. Nothing is scheduled by it; the two-second interval, the idle budget and
the absence of polling threads are unchanged. A refused action takes no sample.
The window still says the processes were *asked* to stop, and the refreshed
table is the machine's answer: a process that ignores `SIGTERM` remains visible.

### 44.4 Gates

```text
cargo test --workspace        passed=1717 failed=0   (was 1,716; +1)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the unchanged absent `keys/dev/` material. No test deleted,
no gate relaxed, no dependency added.

### 44.5 Limits

The CI host has no `litric-sysd`, so the proven immediate sample is the direct
`/proc` fallback; the daemon reply path is the same parser code, unchanged, and
runs when a daemon is present. No QEMU or display is available, so the proof is
the headless frame and real `/proc`. No RAM/CPU/GPU profile was taken for this
change, so no performance number is claimed.


## 45. Batch: a move in Defender or Trash refreshes the open Files table (verified)

### 45.1 What the audit found

`refresh_files` re-reads a folder only when `files.listed` differs from
`files.path`. The filesystem-changing actions that live outside the Files
window did not tell it: Defender's `quarantine_finding` and
`restore_quarantined` moved a real file and refreshed only the contained list,
and Trash's `MenuAction::Restore` put a file back and refreshed only the Trash
list. A Files window open on that folder kept drawing a file that had moved, or
stayed empty after a restore, until the user navigated away. Two journeys were
written before the fix and both failed on exactly that: Files still showed
`["clean.txt", "eicar.com"]` after the quarantine had removed the file from
disk, and `[]` after the restore had put it back.

### 45.2 The wiring

| File | Change |
|---|---|
| `src/litric-shell/src/shell.rs` | `files_changed()` marks the open listing stale (`files.listed = None`) and asks for a redraw. It reads nothing; the next frame's view pass does the single directory read. Called on the successful branch of Defender Quarantine, Defender Restore and Trash Restore. A refusal does not call it because nothing moved. |
| `src/litric-shell/src/e2e.rs` | Two click-driven journeys over real files: open Files on the folder, scan, quarantine, assert the file is gone from disk and from the still-open table, restore, assert it is back in both; and trash a real file, open Files on its folder, select the Trash row, click Restore, assert the file is on disk and in the open table. |
| `ci/lint-file-safety.py` | The two new journeys' fixture cleanups are recorded in the review allowlist with their reason, like the existing journey beside them. |

### 45.3 Cost

One directory read on the next frame after a successful move, through the same
FilesBridge every other listing uses. Nothing is read in the click handler,
nothing is scheduled, and the frame loop, its budget and Activity's samples are
untouched.

### 45.4 Gates

```text
cargo test --workspace        passed=1719 failed=0   (was 1,717; +2)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the unchanged absent `keys/dev/` material: the mkosi
configuration check and the dev key-pair check. Every skip is the documented
missing host tool (fonts, mkosi, qemu). No test deleted, no gate relaxed, no
dependency added.

### 45.5 Limits

No QEMU or display is available, so the proof is the headless frame driven by
the same input path a person drives, over real files. The Trash journey is
Unix-only where it needs a real freedesktop trash move; the Defender journey
runs on every host the tests build for. No performance number is claimed.


## 46. Batch: Files can re-read a folder on demand (verified)

### 46.1 What the audit found

There was no Files refresh control at all: no `FilesHot` variant, no context
row, no shortcut, and F5 was unbound. The only re-read was the implicit
staleness check in `views.rs` (`refresh_files` reads when `files.listed`
differs from `files.path`), so a folder changed by a program outside LitricOS
stayed as last drawn until the user navigated away and back. The command row
drew New folder, Paste, Compress, Extract and Details; nothing there could ask
for a fresh read.

### 46.2 The wiring

| File | Change |
|---|---|
| `src/litric-shell/src/apps.rs` | `FilesHot::Refresh` names the control; the model arm returns `None` because the shell owns the filesystem read. |
| `src/litric-shell/src/shell.rs` | `request_files_refresh()` clears `files.listed` and asks for a redraw; the existing `refresh_files` performs one `list_dir` through the same `FilesBridge` navigation uses. Called by the `Hot::Files(FilesHot::Refresh)` click and by F5 while Files is focused, so both routes are one behaviour. `read_error()` turns the bridge error into a plain sentence ("This folder no longer exists…", permission, not-a-folder, service unavailable); the technical cause is appended only with diagnostics on. |
| `src/litric-shell/src/views.rs` | A Refresh button is pinned beside Details, and the dropping command buttons and the item readout now stop before it, so the control survives the narrowest window that draws Details. A failed read goes through `read_error`, and `apply_error` clears the stale rows. |
| `src/litric-shell/src/e2e.rs` | One journey over real files: an external create and delete are absent/pre-click stale, Refresh makes the table match the disk, F5 does the same on a later change, and a folder removed under the window produces "no longer exists" with no rows left behind. |
| `ci/lint-file-safety.py` | The journey's fixture cleanup is in the review allowlist with its reason, like its siblings. |

### 46.3 Cost

One directory read per user action, on the next frame, through the bridge every
listing already uses. Nothing is read in the click handler, nothing is
scheduled, and there is still no watcher and no polling.

### 46.4 Gates

```text
cargo test --workspace        passed=1720 failed=0   (was 1,719; +1)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped
```

The two failures are the unchanged absent `keys/dev/` material; the four skips
are the documented missing host tools. No test deleted, no gate relaxed, no
dependency added.

### 46.5 Limits

There is still no filesystem watcher: an outside change appears only when the
user asks. A failed read clears the table and shows the reason rather than
keeping rows that are no longer true. No QEMU or display is available, so the
proof is the headless frame over real files, and no performance number is
claimed.

## 47. Batch: Files watches its open folder for outside changes (verified)

### 47.1 Audit

There was no watcher of any kind. The only automatic read was the staleness
check in `views::refresh_files`, which runs during the draw and reads only when
`files.listed` differs from `files.path`; `docs/PROGRESS-TRACKER.md` §46 and the
journey it added both said so explicitly. Files has no refresh timer, no
inotify subscription and no notify dependency, and it cannot have the last two:
`src/litric-shell/src/lib.rs` and `src/litric-shell/src/main.rs` are
`#![forbid(unsafe_code)]`,
so there is no safe `inotify_init1` call here, and the workspace has no watcher
crate. The honest option is a bounded check through the bridge that already
owns the directory, not a second filesystem backend and not an unsafe syscall
wrapper.

### 47.2 The connection

| File | What it does |
|---|---|
| `src/litric-shell/src/filewatch.rs` (new) | `Watch` keeps the path and a 64-bit FNV-1a digest of the listing the last real read produced, the earliest time the next check may run, and a read counter for diagnostics. `INTERVAL_MS = 1_000`. A different digest, or a readable folder that stopped being readable, reports one change; repeats of the same failure are not new changes. Four unit tests cover digest sensitivity, the interval, deferral and loss/recovery. |
| `src/litric-shell/src/shell.rs` | `watch_open_folder(now_ms)` is called from `tick`, so it runs on the wake-up the loop already performs. It returns unless the watcher is armed for the listed path and the interval has elapsed; it defers without reading while the session is locked, no Files window is visible, the listing is waiting for its own read, or the bridge is unavailable. Otherwise it calls the existing `FilesBridge::list_dir`, compares the listing, and on a change calls `files_changed()` — which marks the listing stale, so the next frame's existing read draws real rows. The diagnostics add `files.watch=<path> reads=<n>`. |
| `src/litric-shell/src/views.rs` | `refresh_files` seeds the watcher after every real read (success or failure) with `note_read`/`note_failure`, so the first check compares against what the table is actually showing and does not duplicate the frame's read. |
| `src/litric-shell/src/e2e.rs` | One new headless journey over real files: an outside create and delete are absent from the open table two frames before the interval; `tick_after(1_000)` plus one frame shows both without any click; removing the folder and advancing the clock shows the plain-language error with no stale rows. The diagnostics counter proves zero reads inside the interval and exactly one per interval. The existing Refresh journey is unchanged apart from its now-inaccurate "no watcher" comment. |
| `ci/lint-file-safety.py` | The new journey's fixture cleanup is in the reviewed allowlist with its reason, like its siblings. No rule was weakened. |

### 47.3 Cost

One directory read per second at most, only while a Files window is visible and
the session is unlocked, through the same bridge every listing and file
operation already uses. No new thread, timer, dependency, privilege or polling
interval: the shell's idle wake cap is already one second, so the check rides
the wake-up the clock causes. A digest is computed from the entries of that one
read; nothing else is read and nothing is drawn from the digest.

### 47.4 Gates

```text
cargo test --workspace        passed=1725 failed=0   (was 1,720; +4 unit, +1 journey)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             31 pass, 2 fail, 4 skipped   (baseline unchanged)
```

The two failures are the unchanged absent `keys/dev/` material; the four skips
are the documented missing host tools (fonts, mkosi, QEMU). No test deleted, no
gate relaxed, no dependency added.

### 47.5 Limits

This is not inotify. A change made and reverted between two checks is not
observed, and an entry whose bytes change without its size or modification
second changing produces the same digest — the table would not draw that
difference either. Only the currently listed folder is watched (the shell has
one Files model), and a minimized window is not read until it is visible again.
No performance number is claimed: the only measured cost is the read count in
the journey. No QEMU or display is available, so the proof is headless frames
over real files.

## 48. Batch: "Reduce Transparency" is a real material, and the quick switches persist (verified)

### 48.1 Audit

The switch was already drawn and already claimed to be applied, but nothing
rendered from it. `QuickSettings::reduce_transparency` existed in `shell.rs`,
defaulted to `false`, and was listed in `QuickSettings::applied`; the quick
panel drew it through `views::draw_quick_settings` using
`quick.is_applied("reduce_transparency")`; and `Hot::QuickToggle(2)` did
exactly one thing: flip the bool. No renderer read the field. The material had
two independent reasons a tint value alone could not fix it:

* `desktop::glass` recomputed alpha from the backdrop with
  `Glass::adaptive_alpha_pct`, ignoring the theme's own `tint_alpha_pct`; a
  theme set to 100% would still be drawn translucently.
* `desktop::glass_at` used the caller's alpha (88 for popovers), so a popover
  would stay translucent whatever the theme said.
* `desktop::glass_params` forced a minimum blur of one pixel, so even a
  zero-blur theme still paid for a backdrop blur.

The preference record had no such key either: `Preferences` had no
`reduce_transparency` field, `prefstore` neither parsed nor wrote it, and the
registry had eight entries with Accessibility containing only Reduce motion.
A restart therefore discarded the choice. Its neighbour had a smaller version
of the same problem: the quick Reduce Motion switch flipped
`quick.reduce_motion` directly instead of calling the existing
`toggle_reduced_motion`, so it changed the running shell but never persisted.

### 48.2 The connection

| File | What it does |
|---|---|
| `src/litric-design/src/lib.rs` | `Theme::with_reduced_transparency` replaces the glass preset with `blur_px: 0`, `saturation_pct: 100`, `tint_alpha_pct: 100` while keeping the theme's border and specular values. The `Glass::blur_px` contract now says zero means opaque. |
| `src/litric-shell/src/desktop.rs` | `glass` and `glass_at` take one early path through `solid_glass` when the theme is opaque: full-strength tint, the same edge and highlight, no `backdrop_brightest` read and no `glass_backdrop` call. `glass_params` returns `(0, 1.0)` for an opaque or zero-blur theme. This also corrects the high-contrast themes, which declared 100% but were still being adapted down by the solver. |
| `src/litric-shell/src/settings.rs` | `Preferences::reduce_transparency`, default off. |
| `src/litric-shell/src/prefstore.rs` | Parses `reduce_transparency` with the existing boolean parser and writes it in the same line-oriented format. No schema version bump: the format's own rule is that adding a key does not change the meaning of an existing one. |
| `src/litric-shell/src/registry.rs` | A ninth entry, `reduce_transparency`, on Accessibility with the synonyms `transparency`, `blur`, `glass`, `readability`, `opaque`, so search and `litric://settings/...` links reach it. |
| `src/litric-shell/src/shell.rs` | `resolved_theme` applies the flag to every theme it builds (startup, save, rollback, schedule). `load_preferences`, `save_preferences` and `finish_preferences_save` keep `quick.reduce_transparency` in step with the record. `toggle_reduced_transparency` goes through `save_preferences`, and `Hot::QuickToggle(2)` and `Hot::QuickToggle(3)` now call the transparency and motion toggles instead of mutating live flags. |
| `src/litric-shell/src/views.rs` | The Accessibility page draws a real Reduce transparency row next to Reduce motion, using the same `settings_button` and focus key as every other Settings row. |
| `src/litric-shell/src/e2e.rs` | One new headless journey opens Quick Settings through the real command, waits for its reveal spring, clicks the actual `Hot::QuickToggle(2)` hotspot, and asserts the state, the persisted effect, the opaque material, a real frame difference, and a restart from the saved record. |

### 48.3 Cost

One preference write per confirmed toggle, through the existing atomic
preference store; no new dependency, thread, timer, polling interval, daemon
verb or privilege. In the opaque state the renderer does strictly less work per
panel: it skips the brightest-backdrop scan and the blur pass, and the
compositor is told not to build a blurred copy. No runtime performance number is
claimed because none was measured.

### 48.4 Tests and gates

New tests: `desktop::tests::reduced_transparency_is_opaque_and_does_not_read_the_backdrop`,
`shell::tests::reduce_transparency_changes_the_real_material_and_persists`,
`shell::tests::the_quick_motion_switch_persists_like_the_settings_row`, and
`e2e::journey_reduce_transparency_changes_the_rendered_frame_and_survives_a_restart`.

```text
cargo test --workspace        passed=1729 failed=0   (was 1,725; +4)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             32 pass, 2 fail, 3 skipped
```

The two failures are the unchanged absent `keys/dev/secureboot` and
`keys/dev/verity`; the three skips are the documented missing host tools and
the measurement that needs them (mkosi, QEMU, idle RAM/CPU). No test deleted, no
gate relaxed, no dependency added.

The golden-frame gate ran for the first time here because the image fonts are
now installed; until then it had been skipped as missing fonts. Two frames,
`03-windows-overlap` and `04-files`, were over threshold from Files work that
landed after the last recording (`This Litric`, Refresh/New folder and the size
column). Both were reviewed against the committed evidence, and only those two
fingerprints were re-recorded. The material in this batch is unreachable in that
dumper: it composes the Dark theme only (82% glass, 32 px blur) and never loads
preferences, so the opaque branch cannot execute there. The inherited citation in §47 was also corrected to
`src/litric-shell/src/main.rs`.

### 48.5 Limits

There is no display and no QEMU here, so the proof is a headless frame over the
real draw path, not a photograph of a monitor. The setting is global: every
glass surface becomes opaque together, and there is no per-app override. The
high-contrast themes are now actually opaque rather than adapted down; that is
the behaviour their own comments always claimed, but it is a visible change.
No performance metric is claimed. There is no executable research-deep-all
runner in this repository, so none was run.

## 49. Batch: hidden-file visibility is a persisted, service-backed Files choice (verified)

### 49.1 Audit

The request says the corpus is `research-deep-all` with 15 files. The only
directory with that role is `Research-Deep-all` (capitalised), and it is tracked
on every branch. The exact searches were:

```text
find /workspace/chats/d7991b8c41a570fd94f0a9bc /tmp "$HOME" -maxdepth 6 \
  -type d \( -iname 'research-deep-all' -o -iname 'research_deep_all' \
  -o -iname 'research-deepall' -o -iname 'deep-research' \) -print
  -> only .../Research-Deep-all

git ls-tree -r --name-only origin/main | rg -c -i '^Research-Deep-all/'
  -> 37

for ref in $(git for-each-ref --format='%(refname)'); do
  git ls-tree -r --name-only "$ref" | rg -c -i 'research'
done
  -> 41 main/freebuff refs, 18 redesign/audit refs, 2 tag refs
     (the extra four are docs/research, docs/UI-RESEARCH-*.md)

git log --all --name-only --pretty=format: | sort -u | rg -i 'research'
  -> the same paths; no 15-file variant exists in history

git ls-remote origin
  -> main, audit/comprehensive-2026-10-15, redesign/shell-v4,
     freebuff/d7991b8c41a570fd94f0a9bc, refs/tags/LitricOS
```

The corpus is 30 Markdown research parts plus 7 CSV registers, not 15 files.
That is a count mismatch in the request, not a missing dependency; the corpus
was used as the requirements source. The relevant rows are `WA.13.05.e16`
("Hidden items — displays hidden files and folders") and `WB.01.09.a04`
(File Explorer, "Show hidden and system files") in
`Research-Deep-all/LitricOS UI UX Research Part 3  Windows 11 Shell and File Explorer.md`
and `Research-Deep-all/LitricOS UI UX Research Part 2 Windows 11 26H2 Settings.md`.

The code gap was concrete. `FsBridge::list_dir` filtered every name beginning
with `.` and `fsclient::Daemon::list_dir` always sent plain `LIST <path>`. The
file service already supported the capability: `src/litric-files/src/server.rs`
parsed a second `hidden=1` field and `Root::list_dir` applied it. The shell had
no preference, no registry row, no Settings control and no Files key for it, so
the backend capability was unreachable. This is a wired capability, not a new
filter over rows already in memory.

### 49.2 The connection

| File | What it does |
|---|---|
| `src/litric-shell/src/settings.rs` | `Preferences::show_hidden` defaults to `false`; hidden entries stay hidden until the user asks. |
| `src/litric-shell/src/prefstore.rs` | Parses and writes `show_hidden=0/1` in the existing line-oriented, atomic record. No schema bump: adding a key does not change an existing key's meaning. |
| `src/litric-shell/src/registry.rs` | A tenth entry, `show_hidden`, on Privacy with the synonyms `hidden files`, `hidden items`, `dotfiles`, `show hidden` and `files`; search and `litric://settings/privacy/show_hidden` reach it. |
| `src/litric-shell/src/fsclient.rs` | The real service request is `LIST <path> hidden=1` only when the preference is on; otherwise the wire request is unchanged. |
| `src/litric-shell/src/fsbridge.rs` | `list_dir_with_hidden` is the single read path. The daemon receives the flag; the local fallback applies the same rule. `list_dir` remains the hidden-excluded wrapper for every caller that did not ask. |
| `src/litric-shell/src/views.rs` | `refresh_files` reads through `list_dir_with_hidden` with the stored preference; the Privacy page draws a real Show hidden files button and a line describing the current state. `settings_rows_from` places those rows below the button without duplicating row layout. |
| `src/litric-shell/src/shell.rs` | `Hot::SettingsShowHidden` calls `toggle_show_hidden`, which goes through `save_preferences`; a change invalidates the listing and resets the watcher. `Ctrl+H` is scoped to the Files window. `finish_preferences_save` re-invalidates on a failed write so the table returns to the stored visibility. The watcher reads with the same flag, so its digest cannot oscillate. |
| `src/litric-shell/src/filewatch.rs` | `Watch::reset` forgets a digest recorded under the old visibility; the next real read arms it again. |
| `src/litric-files/src/server.rs` | A test proves `LIST /Documents hidden=0` omits `.secret` and `LIST /Documents hidden=1` returns it. |
| `src/litric-shell/src/e2e.rs` | One headless journey over real temp files drives `Ctrl+H` through the input path, asserts the dot entry is absent before and present after the next composed frame, measures the frame delta, stores the exact record the shell asked for, and reloads it into a fresh shell against the same directory. |
| `docs/evidence/golden/fingerprints.json` | Only `15-search` is re-baselined (old hash `0x965b93930f291ecb`, new `0x1f5a70ea6ad1bfde`). The new frame shows the intended second result, "Show hidden files — Settings · Privacy", for the query `fil`. The other 23 fingerprints are unchanged. |

### 49.3 Cost

One preference write per confirmed toggle through the existing atomic store. A
toggle marks the listing stale; the next frame performs one directory read with
the new flag. While the preference is off, the request string, local read and
rendering are the same as before. There is no polling change, no watcher
interval change, no new thread, dependency, daemon verb or privilege. The
watcher still performs at most one read per second while Files is visible, and
`Watch::reset` prevents an extra read from being interpreted as a change. No
performance number is claimed because none was measured.

### 49.4 Tests and gates

New tests: `prefstore::tests::hidden_file_visibility_round_trips_through_the_file`,
`fsbridge::tests::hidden_entries_are_listed_only_when_the_caller_asks`,
`shell::tests::the_hidden_files_switch_persists_and_marks_the_listing_stale`,
`server::tests::list_hidden_one_returns_dot_entries_and_zero_does_not`, and
`e2e::journey_hidden_files_toggle_is_a_real_read_and_survives_a_restart`.

```text
cargo test --workspace        passed=1734 failed=0   (was 1,729; +5)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
python3 ci/lint-file-safety.py   OK: every deletion site is reviewed
python3 ci/check-golden-frames.py --frames build/frames   PASS
bash ci/verify.sh             32 pass, 2 fail, 3 skipped
```

The two failures are the unchanged absent local dev-key material: the mkosi
schema check reports the secureboot, pcr and verity key files, and the key-pair
check reports secureboot and verity. They are not caused by this batch and were
not papered over; no keys were generated or committed. The three skips are
mkosi, QEMU and the idle measurement that needs them.

### 49.5 Limits

There is no display and no QEMU here, so the proof is headless frames over the
real draw and filesystem paths, not a photograph of a monitor. Hidden means the
Linux dot-prefix convention; there is no xattr or system/attribute hidden bit,
and the choice is global to Files rather than per-folder. The corpus count is
37, not the 15 stated in the request. No performance metric is claimed. This
batch used the Research-Deep-all corpus and its gates; no separate executable
research-deep-all runner exists in this repository.

## 50. Batch: Do Not Disturb is a persisted, queue-backed setting (verified)

### 50.1 Requirement and audit

The requirement source is the `Research-Deep-all` corpus tracked on `main`
(30 Markdown parts and 7 CSV registers; the "15 files" count in the request is
the mismatch recorded in §49.1). The chosen row is `WB.01.03.01`,
"System > Notifications > Do not disturb", whose quoted control text is
"Turn on do not disturb automatically", with the parent row `WA.07.03`,
"Do not disturb / Focus". The implemented slice is the global switch and its
real effect; the automatic schedule and per-app priority list are not claimed.

The audit found that Do Not Disturb existed only in memory. `QuickSettings` and
`Notifications` each held a `do_not_disturb` bool; the quick-settings arm
previously flipped the two fields in place and emitted no
`Effect::PersistPreferences` (the new handler is at
`src/litric-shell/src/shell.rs:6315`). There was no store key, no
registry row, no Settings page and no saved value to reload, so a restart always
came back with banners enabled. The daemon boundary was already correct and was
not changed: `src/litric-notifyd/src/lib.rs` deliberately has no DND flag
because the shell owns the display-time decision. The queue still records every
note; DND only decides whether a banner interrupts.

### 50.2 The connection

| File | What it does |
|---|---|
| `src/litric-shell/src/settings.rs:189` | `Preferences::do_not_disturb` defaults to `false` at line 225, preserving banners for every existing record. The new `Page::Notifications` variant is at line 259, in `Page::ALL` at lines 272/282, with title line 302 and slug line 333. |
| `src/litric-shell/src/prefstore.rs:117` | Parses `do_not_disturb=0/1` (and the file's existing `true/false/yes/no` forms). Line 192 writes it through the same atomic temporary-file/rename path as every other key. |
| `src/litric-shell/src/registry.rs:180` | An eleventh row, `do_not_disturb`, on Notifications, with the synonyms `focus`, `quiet`, `silence`, `notifications`, `dnd` and `banners`. Search and `litric://settings/notifications/do_not_disturb` reach it. |
| `src/litric-shell/src/views.rs:3376` | The Notifications page draws the real Do Not Disturb button and three factual rows: banners silenced except critical, critical always interrupts, and the centre still records notes. The sidebar icon is line 2898; save/rollback status includes the page at line 3542. |
| `src/litric-shell/src/shell.rs:3465` | `toggle_do_not_disturb` flips the preference and goes through `save_preferences`, which applies the value to both the quick switch and the notification queue (lines 3301-3302). `load_preferences` applies a stored value at startup (3249-3250) and `finish_preferences_save` restores the queue after a failed write (3335-3336). |
| `src/litric-shell/src/shell.rs:6262` | `Hot::SettingsDoNotDisturb` is the Settings button's real handler. `Hot::QuickToggle(0)` at line 6315 is the quick-settings control and now takes the same persisted path instead of mutating two fields directly. |
| `src/litric-shell/src/i18n.rs:174` | The new Notifications page title has its Thai translation, so the sidebar does not fall back to English. |
| `src/litric-shell/src/e2e.rs:4408` | One headless journey drives the real quick-settings hotspot with a real pointer click, measures the before/after frame difference, reads the exact `PersistPreferences` record back through a real store file, and starts a fresh shell from it. |

The display rule is unchanged and single-sourced:
`Notifications::should_banner` at `src/litric-shell/src/shell.rs:680` returns
true for critical alerts even while quiet, and the queue itself is untouched.
No daemon verb, second backend, thread, polling interval, dependency or
privilege was added.

### 50.3 Cost

One existing preference write per confirmed toggle, through the atomic store.
Nothing is added to the frame loop: the banner pass already asks
`should_banner` for each queued notification, and that is one boolean test. When
the feature is off the rendering and queue work are identical to before. The
queue remains capped at 64 notes and is unchanged. No speed or RAM number is
claimed because none was measured.

### 50.4 Tests and gates

New tests:
`prefstore::tests::do_not_disturb_round_trips_through_the_file_and_defaults_off`,
`shell::tests::do_not_disturb_is_stored_applied_to_the_queue_and_rolled_back`,
`shell::tests::a_stored_do_not_disturb_is_applied_at_load_and_only_silences_banners`,
and the journey
`e2e::journey_do_not_disturb_persists_silences_the_banner_and_survives_a_restart`.
The registry search and round-trip tests exercise the new row and deep link.

```text
cargo test --workspace        1738 passed, 0 failed, 0 ignored   (was 1734; +4)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             32 pass, 2 fail, 3 skipped
```

`docs/evidence/golden/fingerprints.json` re-baselines only `15-search`
(old `0x1f5a70ea6ad1bfde`, new `0x1328a77eeba088f2`). The reviewed frames show
the intended extra results: the new Notifications page and the Do not disturb
row match the scene's `fi` query through the registry's existing page/detail
matching. The other 23 fingerprints are byte-identical to the committed ones;
`--update` was not accepted wholesale because it rewrote exact hashes for frames
that were still within threshold. The two `keys/dev/` failures remain the
unchanged absent secureboot and verity material.

The journey first draws a real normal notification and asserts banner ink,
clicks the real quick-settings control, asserts the queue still holds the note
while the banner is suppressed, and requires more than 200 differing pixels
between the before and after frames. It then writes the exact record the shell
asked to persist, reloads it into a fresh shell, and asserts the stored quiet
mode survives and still suppresses a normal banner while leaving critical
alerts alone. The OS boundary is not mocked.

### 50.5 Limits

There is no display and no QEMU on this host, so the proof is headless frames
over the real draw path and a real preference file. This batch implements the
global switch and its persisted effect, not the Windows row's automatic
schedule, per-app priority list or calls/reminders exceptions; those remain
unimplemented and are not presented as working. Do Not Disturb is a single
session-wide choice, not a per-app or per-notification policy. The daemon still
does not own DND by design, so a notification posted while the shell is down is
not retroactively suppressed as a banner (adoption never banners). No
performance metric is claimed. This batch used the Research-Deep-all corpus and
the repository's named gates; no separate executable `research-deep-all` runner
exists in this repository.

## 51. Batch: the Browser opens a page served on this machine over real HTTP (verified)

### 51.1 Requirement and audit

The requirement source is the `Research-Deep-all` corpus tracked on `main` (30
Markdown parts and 7 CSV registers; the "15 files" count in the request is the
mismatch recorded in §49.1). The chosen row is `LR.13`, "Integration points
(portals, D-Bus, APIs)" (PARTIAL), whose checklist text is:

> Web apps install, pin to the shelf, appear in the launcher and handle files;
> Chrome extensions are integral (S3882); Android and Linux app support was not
> on any opened page; LitricOS: no extension or store model

The implemented slice is the first step of that chain and nothing more: a page
served by a web server on this machine can be opened by the Browser through a
real HTTP request, laid out and drawn by the window's own reader, with its
pictures fetched over the same socket. Installation, shelf pinning, launcher
entries and file handlers are not implemented and are not claimed.

The audit found that the window had exactly one page path: `reader::open`
accepted `file://` and refused everything else with "this page is not a local
file". `browserd` is a real Chromium-over-CDP broker, but this image has no
Chromium (no `chromium`, `chromium-browser`, `google-chrome` or `chrome` on
`PATH`, and no Playwright browser cache), and `Policy::check_navigation` refused
every plain-HTTP address with an upgrade offer. There was therefore no way to
load a page from a server even when that server was on loopback — the one web
path that needs neither Chromium nor a TLS stack.

App loading, the other half of the request, was audited and is **not**
implemented: `shortcuts::App::ALL` has exactly eight built-in applications,
nothing parses freedesktop `.desktop` entries, and there is no app catalog or
install path. `litric-sysd` spawns specific programs for specific verbs, not
arbitrary user applications. This batch did not invent a launcher that would
list applications it cannot run; the gap is recorded under limits.

### 51.2 The connection

| File | What it does |
|---|---|
| `src/litric-browser/src/lib.rs:45` | `pub mod http;` |
| `src/litric-browser/src/http.rs:58` | `is_loopback_host` accepts exactly `localhost`, `::1` and `127.0.0.0/8`; a name that merely contains `localhost`, or a private address, is another machine. |
| `src/litric-browser/src/http.rs:104` | `get_with_limit` builds one `GET`, connects with a 2 s timeout, bounds the head to 16 KiB and the body to the caller's cap, refuses a chunked response by name, and reports a short body as an error rather than a smaller page. |
| `src/litric-browser/src/url.rs:299` | `Url::join` resolves a page's relative or root-relative picture reference against the page address; `normalize_path` at 377 collapses `.` and `..`. |
| `src/litric-browser/src/policy.rs:302` | Plain HTTP is allowed for loopback only. Every other `http:` address is still refused with the same upgrade offer, and `https` remains the only way off the machine. |
| `src/litric-shell/src/reader.rs:127` | Module-level `serves` (mirrored as `Reader::serves` at 159) is the one predicate the window uses to choose reader versus engine: `file://`, or loopback `http://`. |
| `src/litric-shell/src/reader.rs:199` | `Reader::open` fetches a loopback page with `litric_browser::http::get`, refuses a non-2xx answer, parses the body with the existing HTML reader and uses the page's own `<title>`. |
| `src/litric-shell/src/reader.rs:88` | `Base::image` resolves `<img src>`: a file beside a file page, or a URL against a served page, and `Unavailable` for anything that would leave this machine. `ImageSource` is at 115; `decode` at 499 fetches a JPEG with the existing 16 MiB picture cap. |
| `src/litric-shell/src/shell.rs:1977` | `send_go` routes a served page to the reader instead of the engine; `browser_navigate` does the same at 2056, so an omnibox commit, a bookmark and history take one path. |
| `src/litric-shell/src/shell.rs:2025` | A failed read keeps the user-facing status "Could not open this page" and writes the reason to the journal (`litric-shell: browser: …`), not to the page. |
| `src/litric-shell/src/e2e.rs:4508` | `save_shot` writes a frame as a real PPM when `LITRIC_E2E_SHOTS` names a directory, so a reviewer can see the frame; it is inert otherwise. |
| `src/litric-shell/src/e2e.rs:4542` | The journey: a real `TcpListener` serves one HTML page and one JPEG, the omnibox types the address, Enter loads it, pixels are asserted, the wheel scrolls to the second heading, then a closed port is loaded and must report the failure with no page left on screen. |

### 51.3 Cost

One socket per page fetch and one per picture that becomes visible, each closed
immediately. A page is capped at 4 MiB (the HTML reader's own limit), a picture
at the existing 16 MiB, and a response head at 16 KiB; a declared length over
the cap is refused before any body is read. Picture decoding keeps the §38
bounds (4 pictures, 8,000,000 pixels, 2 new decodes per frame). The reader path
is entered only for `file://` and loopback `http://`; every other address takes
the unchanged engine path. Nothing polls, nothing spawns a thread in product
code, and no per-frame allocation was added: the pending-decode vector existed
and is reused. No speed or RAM number is claimed because none was measured.

### 51.4 Tests and gates

New tests: ten `litric-browser::http` tests over real loopback sockets (status,
content type, close-delimited body, 404, chunked refusal, oversized length,
short body, refused connection, non-local and TLS refusal, the loopback
predicate, target encoding);
`policy::tests::a_server_on_this_machine_may_use_plain_http_and_nothing_else_may`;
`reader::tests::the_reader_serves_this_machines_files_and_web_server_only`;
`reader::tests::a_page_reference_resolves_only_to_this_machines_server`; and the
journey
`e2e::journey_browser_loads_a_page_from_a_local_web_server_and_reports_a_failed_fetch`.

```text
cargo test --workspace        1752 passed, 0 failed, 0 ignored   (was 1738; +14)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
bash ci/verify.sh             32 pass, 2 fail, 3 skipped
```

The two failures are the unchanged absent local dev-key material (mkosi reports
`secureboot`, `pcr` and `verity`; the key-pair check reports `secureboot` and
`verity`). No keys were generated or committed, no gate was weakened and no
golden fingerprint changed: no desktop frame's pixels are touched by this batch.

**Sensitivity, stated as evidence.** With `reader::serves` temporarily forced to
return `false`, the new journey fails at its first assertion — "the served page
loaded; the status line says: browser engine not reachable at
/run/litric/browser.sock" — and passes again once the wiring is restored. The
test is therefore sensitive to this connection, not passing for an unrelated
reason. The journey also asserts the two request lines the server actually
received (`GET / HTTP/1.1`, `GET /panel.jpg HTTP/1.1`), so the picture in the
frame is proven to have arrived over the same socket as the page.

### 51.5 Limits

This is a **local-server path, not a web engine**. There is no Chromium binary in
this image, so `https://` pages still fail honestly at the broker, and no TLS
stack was added to fake one. Only loopback HTTP is fetchable; a private or public
address is still refused by policy. Redirects are not followed (a 3xx is reported
as "the server answered 3xx"), and chunked or compressed responses are refused by
name rather than half-read. There are no cookies, no scripts and no forms. A
served page's pictures are decoded only when they are baseline JPEG, which is the
decoder the product ships.

**App loading remains unimplemented.** The launcher lists eight built-in
applications; there is no `.desktop` parser, app catalog, install path or
arbitrary program launch, and this batch does not claim one. The images produced
for this turn are headless frames of the real draw path, saved under the
untracked `artifacts/` directory, not a photograph of a monitor: this host has no
display and no QEMU.

## 52. Batch: Linux applications are discovered from the desktop and started through litric-sysd (verified)

### 52.1 Requirement and audit

The requirement source is the tracked `Research-Deep-all` corpus and this
tracker. The chosen row is `H.04`, "Launch/switch apps" (PARTIAL), whose
checklist text is:

> Recency switcher exists; no failure or progress surface for a launch (Part 21)

and the gap §51.5 recorded in this same file:

> **App loading remains unimplemented.** The launcher lists eight built-in
> applications; there is no `.desktop` parser, app catalog, install path or
> arbitrary program launch, and this batch does not claim one.

The audit confirmed both statements and found the exact shape of the gap:

* `shortcuts::App::ALL` is a fixed array of eight applications; the launcher
  grid, the dock's expanded panel and `seek::search` each iterated it directly,
  and `seek::Hit` had no variant that could name anything else.
* `litric-sysd` had typed verbs for power, actions, resolution and the wave-1
  providers, but no verb that starts a program, and `litric-caps` had no
  `process.spawn` capability at all — a launch could not be authorised even in
  principle.
* nothing in the tree read `$XDG_DATA_HOME` or `$XDG_DATA_DIRS`; a repository
  search for `applications`, `.desktop` and `TryExec` found no parser.

This batch closes that gap end to end: discovery, validation, a capability-gated
daemon verb, the launcher/dock/search surface and a failure line. It does not
add an installer, a store, D-Bus activation or an icon-theme renderer.

### 52.2 The connection

| File | Line | What it connects |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 33, 36, 39 | the three bounds: 512 files per directory, 64 KiB per file, 128 applications |
| `src/litric-shell/src/desktop_entry.rs` | 62 | `data_dirs`: `$XDG_DATA_HOME/applications` then each `$XDG_DATA_DIRS/applications`, defaulting to `/usr/local/share:/usr/share` |
| `src/litric-shell/src/desktop_entry.rs` | 91 | `discover_in`: deterministic order, first directory wins an id, dangling and oversized entries skipped, no file contents retained |
| `src/litric-shell/src/desktop_entry.rs` | 157 | `parse_entry`: `[Desktop Entry]` only, `Type=Application`, refuses `NoDisplay`/`Hidden`/`Terminal`, honours `TryExec`, `Name[locale]` then `Name` |
| `src/litric-shell/src/desktop_entry.rs` | 307 | `parse_exec`: quoting, `%%`, field codes stripped, shell metacharacters refused |
| `src/litric-shell/src/desktop_entry.rs` | 337 | `resolve_program`: an absolute executable only, or a bare name resolved on `PATH` |
| `src/litric-sysd/src/launch.rs` | 24, 96 | `MAX_ARGV` and `validate`: an absolute regular executable, at most 32 arguments of at most 512 bytes |
| `src/litric-sysd/src/launch.rs` | 128 | `launch`: `Command` with no shell, stdin/out/err null, one reaper thread per child |
| `src/litric-sysd/src/launch.rs` | 74, 153 | `is_executable` (mode bits) and `hex_encode` for the wire |
| `src/litric-sysd/src/proto.rs` | 134, 497 | `Request::LaunchApp` and its `LAUNCH <hex>…` parser; NUL refused |
| `src/litric-sysd/src/state.rs` | 247 | `launch_app`: the `ProcessSpawn` capability is checked with the existing broker before `launch::launch` runs; the reply carries `pid=` or a typed `ERR` |
| `src/litric-caps/src/capability.rs` | 103, 142, 181, 221, 294, 337, 355, 491 | the new `ProcessSpawn` capability: token, parse, `all()`, scope class, prompt, risk, and the `app://` resource scheme |
| `src/litric-sysd/src/state.rs` | 79 | the local session client's grant of `ProcessSpawn` scoped to `app://` |
| `src/litric-shell/src/shell.rs` | 7232 | `load_desktop_apps`: the validated list the shell owns |
| `src/litric-shell/src/shell.rs` | 7240, 7246, 7256 | `launcher_len`, `launcher_item`, `launcher_name`: built-ins first, then the machine's applications |
| `src/litric-shell/src/shell.rs` | 7272 | `launch_desktop`: one `LAUNCH` request to `litric-sysd`, the outcome in `app_status`, the reason in the journal |
| `src/litric-shell/src/shell.rs` | 7349 | `seek_open` dispatches `Hit::Desktop` to that launch, so keyboard and pointer share one path |
| `src/litric-shell/src/seek.rs` | 160 | discovered applications are ranked in the same search as the built-ins |
| `src/litric-shell/src/views.rs` | 3800, 3907 | the dock's row count and the last launch's outcome line |
| `src/litric-shell/src/views.rs` | 4216, 4230, 4364 | the launcher tiles: one per slot, a generated glyph for a discovered program, the same icon in the search rows |
| `src/litric-shell/src/icons.rs` | 46, 109, 339 | the `Application` glyph: a rounded tile with a centre dot, not a borrowed application icon |
| `src/litric-shell/src/main.rs` | 1201 | discovery runs once at startup, before the first frame |
| `src/litric-shell/src/e2e.rs` | 4746 | the journey: a real `.desktop` tree, a real daemon on a real socket, a real marker file |

### 52.3 Cost

Discovery happens **once**, at startup, before the first frame: it reads at most
512 directory entries per applications directory, refuses any file over 64 KiB
without opening it, and keeps at most 128 applications. The only thing retained
is the `Vec<DesktopApp>` (name, vector, icon name, source path); file contents
are dropped as soon as one entry is parsed. There is no watcher, no polling, no
background thread and no cache beyond that list.

The draw path does no discovery. `launcher_len()` is an addition of two lengths,
and the tile loop runs over `8 + N` slots only while the dock is expanded; the
status line borrows `Option<&str>` from the shell and allocates nothing. A launch
costs one socket round trip per user action and one reaper thread per launched
program. No RSS or CPU number is claimed: none was measured for this change.

### 52.4 Tests and gates

* `desktop_entry` unit tests: a plain entry, quotes/`%%`/field codes, the eight
  refusal classes, XDG precedence with a shadowed id, an oversized file, and a
  pinned-bounds test.
* `launch` unit tests: an empty or relative vector refused, the bounds, a
  directory and a non-executable file refused, a real script's marker file, an
  argument vector that never reaches a shell, a program that disappears after
  validation, and the hex round trip.
* `litric-sysd` suite: the `LAUNCH` parser and the capability-gated verb.
* `src/litric-shell/src/e2e.rs:4746`
  `journey_launcher_lists_a_real_desktop_app_and_starts_it_through_sysd`: a
  temporary `XDG_DATA_HOME` tree with one runnable entry and one entry whose
  program does not exist; the launcher is opened with a real pointer press, the
  runnable app is listed, the missing one is not; the name is typed and Enter is
  pressed; the program the daemon started writes its argument to a marker file
  and the test reads it from disk; the status line says `Started Zeta Marker`.
  Then the program is deleted and the same keyboard path reports
  `Could not start Zeta Marker` instead of claiming success.
* Frames: `artifacts/browser-shots/52-launcher-real-app.png` (the launcher with
  `Zeta Marker` beside the eight built-ins) and
  `artifacts/browser-shots/52-app-launched.png` (the same panel with
  `Started Zeta Marker (pid …)`), written by the existing `save_shot` mechanism;
  the PPM originals sit beside them. They are untracked, as `artifacts/` always
  is.

```text
cargo test --workspace        1766 passed, 0 failed, 0 ignored   (was 1752; +14)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
python3 ci/lint-doc-refs.py   OK: 951 citations across 155 documents
python3 ci/lint-file-safety.py   OK: 93 deletion sites, all inside reviewed functions
bash ci/verify.sh             32 pass, 2 fail, 3 skipped
```

The two failures are the unchanged absent local dev-key material (mkosi reports
`secureboot`, `pcr` and `verity`; the key-pair check reports `secureboot` and
`verity`). No key was generated or committed, no gate was weakened and no golden
fingerprint changed: no existing desktop frame's pixels are touched by this
batch, and no frame was re-recorded. The file-safety linter flagged the two
deletion sites in the new journey; they were added to `ci/lint-file-safety.py`'s
reviewed list with the fixture-scoped reason, exactly as the sibling journeys
are, rather than by relaxing the rule.

### 52.5 Sensitivity

Both directions were disabled and the journey re-run, then the code restored:

* With `Shell::launch_desktop` returning before it sends the request, the
  journey fails with `assertion left == right failed: the launched program wrote
  its argument to the marker` (`0 passed; 1 failed`). The marker is written by
  the program the daemon starts, so this can only pass when the whole chain ran.
* With `discover_in` returning an empty list, the journey fails earlier with
  `assertion left == right failed: only a runnable entry is offered: []`.
* Restored, the same journey passes (`1 passed; 0 failed`).

### 52.6 Limits

* **Icons are not decoded.** `Icon=` is stored, but this image has no icon-theme
  reader and no SVG/PNG decoder for themes, so every discovered application is
  drawn with the generated `Application` glyph. This is stated, not faked with a
  borrowed icon.
* **No D-Bus activation and no desktop actions.** Only the `[Desktop Entry]`
  group is read; `Actions=`, `DBusActivatable=` and `StartupNotify=` are ignored.
* **`Terminal=true` entries are refused.** The image hosts no terminal emulator
  to attach them to, so offering them would be offering a window that cannot
  exist.
* **Field codes are dropped, not filled.** `%f`, `%F`, `%u`, `%U` and the rest
  are removed because the launcher supplies no file or URL. An application that
  requires a file argument will start without one.
* **Discovery is a snapshot.** An application installed after the shell starts
  appears after the next start; there is deliberately no watcher and no polling.
* **Locale coverage is one language prefix** (`LANGUAGE`, `LC_ALL` or `LANG`),
  not the full `Name[lang_COUNTRY@modifier]` fallback chain.
* **No sandbox around the launched program.** It runs with the daemon's
  environment and credentials; the guarantees are the validated absolute path,
  the bounded vector and the absence of a shell, not isolation.
* **No measured performance claim.** The bounds above are static; RSS, CPU and
  latency were not measured for this batch.

## 53. Batch: real theme icons, and a page whose values can be read (verified)

### 53.1 Requirement and audit

Two sources, both already recorded: §52.6's own limit,

> **Icons are not decoded.** `Icon=` is stored, but this image has no icon-theme
> reader and no SVG/PNG decoder for themes, so every discovered application is
> drawn with the generated `Application` glyph.

and the reported page defect: a served page could not be read. The audit found
the exact causes:

* `reader.rs` laid body text out at `TypeStep::Footnote` (13 px), scaled headings
  by hand (`1.12`/`1.0`/`0.9`), and drew on the window's bare panel with no page
  sheet — a wall of small text on chrome.
* `html.rs` had no table block at all. `table`, `tr` and `td` were only block
  boundaries, so a row of metrics collapsed into one paragraph —
  `Metric Value CPU 42% Memory 1.2 GB` — which is precisely a value nobody can
  read because the columns are gone.
* `appicons.rs` already decoded PNG through the `png` crate (the dock artwork),
  but nothing looked an `Icon=` name up in a theme: `desktop_entry.rs` kept the
  string and never used it.

This batch fixes all three: a real icon-theme lookup with a bounded decode
cache, structural table parsing with a column layout, body typography at the
Body step, and a page surface. It does not add an SVG renderer, CSS or a
scripting engine.

### 53.2 The connection

| File | Line | What it connects |
| --- | --- | --- |
| `src/litric-browser/src/html.rs` | 24, 27, 30 | the table bounds: 60 rows, 6 columns, 120 characters per cell |
| `src/litric-browser/src/html.rs` | 60 | `Block::Table`: a table is kept as a table |
| `src/litric-browser/src/html.rs` | 116 | `segments`: `<table>` runs are cut out of the prose pass |
| `src/litric-browser/src/html.rs` | 223 | `table`: `<tr>`/`<td>`/`<th>` become header and body cells; any other tag keeps its text |
| `src/litric-browser/src/html.rs` | 310 | `push_cell`: one cell cannot grow past its bound |
| `src/litric-shell/src/reader.rs` | 69 | `LaidKind::Table`: the laid-out columns |
| `src/litric-shell/src/reader.rs` | 295 | `visible_text` reports header and rows, so the text a person reads is the text the reader exposes |
| `src/litric-shell/src/reader.rs` | 399 | layout: Table arm (equal columns, 1.9× body row height) |
| `src/litric-shell/src/reader.rs` | 455 | the page surface: a sheet a shade above the panel, with its own edge |
| `src/litric-shell/src/reader.rs` | 503 | draw: header rule, zebra rows, each value in its own column |
| `src/litric-shell/src/reader.rs` | 760 | unit test: a page with a table lays out `Metric  Value` and `CPU  42%` |
| `src/litric-shell/src/themeicons.rs` | 25, 31 | bounds: 512 KiB per icon, 48 cached scaled copies |
| `src/litric-shell/src/themeicons.rs` | 38 | `theme_dirs`: `$XDG_DATA_HOME/icons`, `~/.icons`, each `$XDG_DATA_DIRS/icons`, `/usr/share/pixmaps` |
| `src/litric-shell/src/themeicons.rs` | 73 | `find_in`: flat PNG, then each theme (`hicolor` first), nearest size directory, five standard categories |
| `src/litric-shell/src/themeicons.rs` | 125, 134 | `usable` (regular file, size bound) and `size_dirs` (nearest edge first) |
| `src/litric-shell/src/themeicons.rs` | 165 | `draw`: decode once, scale once, blit |
| `src/litric-shell/src/themeicons.rs` | 183, 205, 254 | `scaled` (bounded cache), `decode` (RGBA/RGB, edge bound), `scale` (nearest neighbour) |
| `src/litric-shell/src/desktop_entry.rs` | 53 | `DesktopApp::icon_path`: the resolved PNG |
| `src/litric-shell/src/desktop_entry.rs` | 89, 95, 104, 151 | `discover` resolves against the machine's themes; `discover_in` stays hermetic; `discover_in_with` takes explicit icon dirs and resolves each name once |
| `src/litric-shell/src/views.rs` | 4273 | the launcher tile draws the resolved icon; the generated glyph is the fallback |
| `src/litric-shell/src/views.rs` | 4389 | the search row draws the same icon |
| `src/litric-shell/src/lib.rs` | 80 | the module is part of the shell |
| `src/litric-shell/src/e2e.rs` | 4874 | icon journey: a real theme PNG, discovery, launcher frame, icon removed, frame compared |
| `src/litric-shell/src/e2e.rs` | 4961 | table journey: a real loopback server, the real omnibox, column assertions and pixel ink |

### 53.3 Cost

Discovery resolves each entry's icon once, at startup: a handful of `stat` calls
per application, at most 64 theme directories inspected per lookup. Pixels are
decoded only when a tile is drawn, once per (path, size), into a cache bounded at
48 scaled copies; a file over 512 KiB or an edge over 512 px is refused before it
is read. Scaling is nearest-neighbour, not a filter: icons are drawn at 16–82 px
and the cost of a filter is not repaid there.

The reader's table cells are cloned once per layout, not per frame. The draw loop
allocates only the elided strings the visible cells need, exactly as it already
did for paragraphs. There is no polling, no watcher and no new thread anywhere in
this batch. No RSS, CPU or frame-time number is claimed: none was measured.

### 53.4 Tests and gates

* `html` unit tests: a table keeps its header and rows; a table without `<th>` is
  all body and is truncated at the row bound; the three new bounds are pinned.
* `themeicons` unit tests: a real PNG is found, decoded and drawn; a name that
  could escape the theme is refused; an oversized file is not an icon; the same
  icon is decoded once and the cache bound is pinned.
* `reader` unit test: a file page with a table lays out `Metric  Value` and
  `CPU  42%` and draws it.
* `journey_launcher_draws_the_icon_the_desktop_entry_names`:
  `src/litric-shell/src/e2e.rs:4874` — a temporary XDG tree with a real
  `hicolor/48x48/apps` PNG and a real `.desktop` entry; the launcher is opened by
  clicking Apps; the icon tile is compared with the same frame after the icon is
  removed, and at least a third of the sampled pixels must change.
* `journey_reader_lays_out_a_table_of_values_in_columns`:
  `src/litric-shell/src/e2e.rs:4961` — a real `TcpListener` on loopback serves a
  page with a three-row table, opened by typing the address into the real
  omnibox; the reader reports `Metric  Value`, `CPU  42%`, `Memory  1.2 GB` as
  separate lines, and both the label and value columns must carry ink in the
  rendered frame.
* Frames: `artifacts/browser-shots/53-launcher-real-icon.png` (the launcher with
  the theme's own folder artwork in the discovered app's tile) and
  `artifacts/browser-shots/53-web-table-values.png` (the served page with its
  table drawn in columns). PPM originals sit beside them; `artifacts/` stays
  untracked.

```text
cargo test --workspace        1775 passed, 0 failed, 0 ignored   (was 1752; +23)
cargo clippy --workspace --all-targets -- -D warnings   exit 0
cargo fmt --all --check       exit 0
python3 ci/lint-doc-refs.py   OK: 976 citations across 155 documents
python3 ci/lint-file-safety.py   OK: 94 deletion sites, all inside reviewed functions
bash ci/verify.sh             32 pass, 2 fail, 3 skipped
```

The two failures are the unchanged absent local dev-key material (mkosi reports
`secureboot`, `pcr` and `verity`; the key-pair check reports `secureboot` and
`verity`). No key was generated or committed, no gate was weakened, and no golden
fingerprint changed: this batch touches no desktop frame that the golden set
records (its frames are the launcher and a browser page rendered inside a
journey, saved by `save_shot` under `artifacts/`, not the golden dump). The
file-safety linter flagged the new journey's temporary-root cleanup; it was added
to `ci/lint-file-safety.py`'s reviewed list with the fixture-scoped reason, the
same way the sibling journeys are, rather than by relaxing the rule.

### 53.5 Sensitivity

Each direction was disabled, the journey re-run, then the code restored:

* With `html::table` returning `None` (tables dropped, the behaviour before this
  batch), the table journey fails:
  `the header keeps its cells: ["System values", "Read from a real page served on this machine."]`
  — the rows are gone. Restored, it passes.
* With `themeicons::draw` returning `false`, the icon journey fails:
  `the themed icon replaced the glyph: 0/361 pixels differ`. Restored, it passes.

The icon journey also fails if the path is not resolved: it asserts
`apps.first().and_then(|a| a.icon_path.as_deref()).is_some()` before it draws.

### 53.6 Limits

* **SVG icons are still not decoded.** A theme that ships only `scalable/*.svg`
  falls back to the generated glyph. There is no SVG renderer in this image and
  none was invented.
* **No `Inherits=` theme chain.** Lookup tries the flat directory, then each
  theme directory with `hicolor` first, at the nearest size; it does not follow a
  theme's declared parents.
* **Nearest-neighbour scaling.** A 16 px icon drawn at 82 px is blocky; the tile
  is still the real artwork, not a fabricated one.
* **Table support is structural.** No `colspan`, `rowspan`, `caption`, footers,
  alignment or per-cell styling; at most 6 columns and 60 rows.
* **Still no CSS, scripts, forms or links.** The reader lays out headings,
  paragraphs, tables, pictures and rules. A page that is mostly a script-driven
  application still reads as its static text.

## 54. Batch: a scalable-only icon theme renders its real artwork (verified)

### 54.1 Requirement and audit

Sources: this file's own §53.6 limits (the remaining list it left) and the
checklist row `G.07,G,Icons,PARTIAL` — "32 icon variants drawn from one vector
module that inherits colour. Icon-only controls still have no label, tooltip or
accessible name (Part 16 F.14)." The three limits §53.6 stated are quoted here
verbatim:

> **SVG icons are still not decoded.** A theme that ships only `scalable/*.svg`
> falls back to the generated glyph. There is no SVG renderer in this image and
> none was invented.
>
> **No `Inherits=` theme chain.** Lookup tries the flat directory, then each
> theme directory with `hicolor` first, at the nearest size; it does not follow a
> theme's declared parents.
>
> **Nearest-neighbour scaling.** A 16 px icon drawn at 82 px is blocky; the tile
> is still the real artwork, not a fabricated one.

The audit found, in this order:

* `find_in` never probed a `scalable/` directory: only `<size>x<size>` PNG
  directories and a flat `<name>.png` were tried, so an SVG-only theme resolved
  to nothing and the tile fell back to the generated glyph in `icons.rs`.
* There was no SVG reader in the tree and no `index.theme` reader either: a
  theme's `Inherits=` list was never opened.
* `scale` was nearest neighbour — right for a downscale, visibly blocky for the
  16 px → 82 px case the launcher and the dock produce.

This batch adds (1) `scalable/*.svg` to the lookup, with a PNG at an equal or
better size still winning, (2) a bounded SVG rasteriser written in the module —
no dependency was added, because the workspace has no SVG crate and this batch
did not bring one in, (3) an `Inherits=` walk with a depth cap and a cycle guard,
and (4) bilinear resampling for upscales only.

The first end-to-end run of the rasteriser **failed on a real bug this batch
wrote**: `rect_polygon` returned four corner points and `fill_polygons` walked
only the `windows(2)` edges, so the closing edge was missing, exactly one
crossing was found, and the blue field of the test document was never painted —
0 blue pixels of 4356. The filler now closes every ring implicitly (SVG fills a
subpath as if it ended with `Z`), and a `<polyline>` stays open because it is only
ever stroked. The pixel assertion caught what the compiler had accepted.

### 54.2 The connection

| File | Line | What it connects |
| --- | --- | --- |
| `src/litric-shell/src/themeicons.rs` | 14 | module doc: what the reader supports and what it approximates |
| `src/litric-shell/src/themeicons.rs` | 48, 51, 54, 57, 60, 63, 66, 69, 72, 75 | the bounds: 512 KiB PNG, 256 KiB SVG, 512 px edge, 48 cached copies, 64 themes, depth 8, 400 elements, 20 000 points, 128 px raster side, 40 000 000 samples |
| `src/litric-shell/src/themeicons.rs` | 108, 118 | `find` / `find_in`: the one lookup entry the shell calls |
| `src/litric-shell/src/themeicons.rs` | 137, 243 | a flat `<name>.svg`, then `<theme>/scalable/apps/<name>.svg` |
| `src/litric-shell/src/themeicons.rs` | 163, 191, 213 | the `Inherits=` walk: breadth-first over declared parents, depth-capped, names deduplicated so a cycle terminates |
| `src/litric-shell/src/themeicons.rs` | 271, 280 | `ceiling` picks 256 KiB for an SVG; `is_svg` is the one extension test |
| `src/litric-shell/src/themeicons.rs` | 319 | `draw`: the only path from an icon file to pixels |
| `src/litric-shell/src/themeicons.rs` | 337, 359 | `scaled` (bounded cache) and `decode_at` (file size checked before reading, SVG or PNG) |
| `src/litric-shell/src/themeicons.rs` | 417, 419 | `resample`: bilinear for an upscale, average for a downscale |
| `src/litric-shell/src/themeicons.rs` | 786 | `rasterize_svg`: elements → polygons → fills, bounded by the constants above |
| `src/litric-shell/src/themeicons.rs` | 1379 | `fill_polygons`: nonzero/even-odd spans, every ring closed implicitly |
| `src/litric-shell/src/desktop_entry.rs` | 151 | `icon_path` = `themeicons::find_in(icon_dirs, icon, 48)`: discovery resolves once, at startup |
| `src/litric-shell/src/views.rs` | 4273 | the launcher tile draws that path; the generated glyph is only the fallback |
| `src/litric-shell/src/views.rs` | 4389 | the launcher search row draws the same path — one drawing path, not two |
| `src/litric-shell/src/e2e.rs` | 5069 | journey: a real XDG applications tree plus an SVG-only theme, discovery, the real launcher frame, colours asserted, path removed, frames compared |
| `src/litric-shell/src/e2e.rs` | 5117, 5158 | the two frames saved for review: SVG-drawn and glyph fallback |
| `ci/lint-file-safety.py` | 142 | the journey's temporary-root cleanup, recorded with its reason |

### 54.3 Cost

Measured, not estimated:

* **Rasterisation:** 0.411 ms per 48 px icon. Method: the unit test clears the
  cache and rasterises the same document 20 times inside one `Instant`, then
  divides by 20 (`rasterising_a_theme_sized_icon_is_quick_and_the_cache_is_bounded`).
* **Cache ceiling:** 3 145 728 bytes = 48 entries × 128 px × 128 px × 4 bytes,
  asserted in the same test. An icon is rasterised only when a tile is drawn and
  only once per (path, size).
* **Skip when off:** no SVG is parsed unless a tile resolves to one. A PNG path
  never touches the SVG code, and a tile with no icon never opens a file.

No per-frame work was added: the draw loop calls `draw`, which hits the cache,
exactly as §53 did. The rasteriser is a one-off per (path, size) and its worst
case is bounded by the constants (256 KiB file, 400 elements, 20 000 points,
40 000 000 sample budget, 128 px side). An oversized file is refused before it
is read; a path with a huge `d` runs out of budget instead of hanging, which the
test `an_svg_over_its_bounds_is_refused_and_a_huge_path_does_not_hang` asserts.

### 54.4 Tests and gates

Tests added: 8 unit tests (`an_svg_is_found_and_rasterised_when_the_theme_has_no_png`,
`a_png_at_the_size_directory_wins_over_an_svg`,
`a_theme_that_names_no_icon_falls_back_to_its_parent`, `an_inherit_cycle_terminates`,
`an_svg_over_its_bounds_is_refused_and_a_huge_path_does_not_hang`,
`bilinear_scaling_is_not_nearest_neighbour`,
`rasterising_a_theme_sized_icon_is_quick_and_the_cache_is_bounded`,
`the_bounds_are_pinned`) and 1 e2e journey
(`journey_launcher_draws_an_svg_only_theme_icon`), 9 in total.

Commands run, with their real results:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | 1783 passed, 0 failed |
| `python3 ci/lint-doc-refs.py` | OK: 993 citations across 155 documents |
| `python3 ci/lint-file-safety.py` | exit 0, with the new journey's cleanup site recorded at line 142 and its reason printed |
| `bash ci/verify.sh` | `summary: 32 pass, 2 fail, 3 skipped` |

The two failures are the unchanged absent local dev-key material (`secureboot`,
`pcr`, `verity`); the three skips are the QEMU-only checks, which this host
cannot run. No gate was weakened, no key was generated or committed, and no
golden fingerprint changed: the golden dump records desktop frames, and this
batch changes a frame only where a real theme icon resolves, which the golden
fixtures do not have.

### 54.5 Sensitivity

Each direction was disabled, the journey re-run, then the code restored:

* With `.svg` lookups disabled (`if false && usable(...)` at all three probe
  sites), the journey fails before it draws:
  `panicked at litric-shell/src/e2e.rs:5100:10: the SVG icon resolved`.
* With the SVG decode disabled (`if false && is_svg(path)`, so a scalable icon
  falls through to the PNG decoder), it fails on the pixels:
  `panicked at litric-shell/src/e2e.rs:5149:5: the SVG's shapes are not on
  screen: 39 blue, 0 amber of 676` — the glyph fallback has neither the
  document's blue field nor its amber circle.
* Restored, both runs pass (`test result: ok. 1 passed`), and `rg 'false &&'`
  over the module returns nothing.

### 54.6 Limits

* **One sample per pixel, no antialiasing.** Edges are hard; at 82 px a curve
  shows stair-steps where librsvg would be smooth. The artwork, colours and
  shapes are the document's own.
* **Attributes, not stylesheets.** `fill`/`stroke` come from presentation
  attributes; `<style>`, `class` selectors and CSS are not read.
* **Gradients are approximated** by their first stop, drawn flat. `mask`,
  `clipPath`, `filter` and `pattern` are skipped, so a themed icon that depends
  on a mask renders its unmasked shapes.
* **No `<text>`, `<use>`/`<symbol>` instancing, or external images.** A document
  that needs them loses those parts.
* **Stroke geometry is a quad strip** with butt ends, not mitered or rounded
  joins, so a very thick stroked icon can differ by a pixel at a corner.
* **The theme list is read once per process.** A theme or application installed
  while the shell is running needs a restart to appear, unchanged from §52.
* **Discovery is still a startup cost.** It runs on the shell's start path, not
  in the background; a machine with hundreds of `applications` directories pays
  it once, before the first frame.

## 55. Batch: an application or theme installed while the shell runs appears without a restart (verified)

### 55.1 Requirement and audit

Source: this file's own §54.6 limits, which ended with

> **The theme list is read once per process.** A theme or application installed
> while the shell is running needs a restart to appear, unchanged from §52.
>
> **Discovery is still a startup cost.** It runs on the shell's start path, not
> in the background; a machine with hundreds of `applications` directories pays
> it once, before the first frame.

and the checklist row `G.07,G,Icons,PARTIAL`, whose remaining gap was that the
shell had no way to notice the machine changing under it.

The audit found exactly three places that made the list static:

* `main.rs` line 1203 called `desktop_entry::discover()` once, at
  `start()`, and handed the result to `Shell::load_desktop_apps`; nothing kept
  the directories it read.
* `themeicons` kept decoded icons in a thread-local `HashMap` with no way to
  drop it, and `find_in` resolved a name against the tree as it was at that
  moment. A theme installed later could never be found by a name that had
  already resolved to nothing.
* There was no watcher for either. The Files window has had one since §45
  (`filewatch`), and nothing equivalent existed for installed applications.

This batch adds `desktop_entry::DesktopWatch`: the modification time and entry
count of the directories an install would touch, compared at most once a second,
with the shell reloading the list, dropping the decoded icon cache and redrawing
only when the comparison says something changed. No `inotify` call (the crate is
`#![forbid(unsafe_code)]`) and no dependency was added.

**A real finding shaped the design.** The first version compared only directory
mtimes. Its test failed on this machine, and the cause was measured: this
filesystem reports a directory's mtime at **one-second granularity** — `stat`
after a write 0.2 s later showed a delta of `0 ns` — so an entry created in the
same second as the last look was invisible, permanently. The stamp therefore
carries a bounded entry count beside the mtime, which changes the moment a file
or directory is added or removed. Both signals are documented in the module.

### 55.2 The connection

| File | Line | What it connects |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 172 | `WATCH_INTERVAL_MS`: one look a second, the clock's own resolution |
| `src/litric-shell/src/desktop_entry.rs` | 179, 186 | bounds: at most 96 directories stamped, at most 512 entries counted per directory |
| `src/litric-shell/src/desktop_entry.rs` | 202 | `stamp_of`: one `stat` plus a capped `read_dir` count per directory |
| `src/litric-shell/src/desktop_entry.rs` | 218 | the watched set: the applications directories, the icon roots, and each theme under them |
| `src/litric-shell/src/desktop_entry.rs` | 272, 287 | `DesktopWatch` and the recorded tree it is armed with |
| `src/litric-shell/src/desktop_entry.rs` | 306 | `poll`: returns before touching the disk until the interval elapses; compares every stamp otherwise |
| `src/litric-shell/src/shell.rs` | 3096 | `Shell::desktop_watch`: `None` for a fixture's list, armed for a real one |
| `src/litric-shell/src/shell.rs` | 7254 | `load_desktop_apps_from`: discover, resolve icons, arm the watch |
| `src/litric-shell/src/shell.rs` | 7275 | `refresh_desktop`: on a change, drop the icon cache and reload the list |
| `src/litric-shell/src/shell.rs` | 4710 | the tick that calls it: one batch of `stat` calls a second, none per frame |
| `src/litric-shell/src/themeicons.rs` | 321, 329 | `clear_cache` (the icon tree moved) and `cache_len` (the measured ceiling) |
| `src/litric-shell/src/main.rs` | 1203 | the session starts watched, over `data_dirs()` and `theme_dirs()` |
| `src/litric-shell/src/e2e.rs` | 5069 | journey: an empty temporary tree, an entry and its SVG written while the shell runs, the launcher drawn, the entry removed |
| `ci/lint-file-safety.py` | 142 | the journey's temporary-root cleanup, recorded with its reason |

`Shell::load_desktop_apps` still exists and now clears the watch: a list placed by
a fixture is never watched, so no test stats the host and no golden frame can
depend on the machine it ran on.

### 55.3 Cost

Measured, not estimated:

* **One check: 0.254 ms** over 2 directories in the unit fixture (an
  `applications` directory with 200 entries plus its root), 2 `stat` calls and 2
  bounded `read_dir` counts per check. Method:
  `one_check_costs_a_bounded_number_of_directory_reads` polls 20 times inside one
  `Instant` with the interval forced forward each time, and divides.
* **One check a second at most**, and **none per frame**: the journey asserts the
  `stats` counter is unchanged across 30 frames (480 ms) with the launcher open.
  `poll` returns before any syscall while the interval has not elapsed.
* **Skip when not armed:** a fixture list (`load_desktop_apps`) leaves the watch
  `None`, so the tick does one `Option` test and nothing else.
* **Icon cache:** unchanged ceiling, 48 entries = 3 145 728 bytes; a change drops
  it whole (bounded, one-off) because the name-to-file mapping may have moved.
* **RAM:** the watch keeps the directory paths (one `PathBuf` each, at most 96)
  and one `Stamp` (an `Option<SystemTime>` and a `u32`) per directory. No
  directory entries are retained.

### 55.4 Tests and gates

Tests added: 5 unit tests in `desktop_entry` (`the_watch_reads_nothing_before_its_interval`,
`an_installed_entry_and_a_new_theme_are_both_a_change`, `the_watched_paths_are_bounded`,
`a_missing_directory_is_not_a_change_every_second`,
`one_check_costs_a_bounded_number_of_directory_reads`) and 1 e2e journey
(`journey_launcher_notices_an_app_installed_while_it_runs`), 6 in total.

Commands run, with their real results:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | 1789 passed, 0 failed |
| `python3 ci/lint-doc-refs.py` | OK, all citations resolve |
| `python3 ci/lint-file-safety.py` | exit 0, with the new journey's cleanup site recorded at line 142 |
| `bash ci/verify.sh` | `summary: 32 pass, 2 fail, 3 skipped` |

The two failures are the unchanged absent local dev-key material; the three skips
are the QEMU-only checks. No gate was weakened, no key was generated, and no
golden fingerprint changed: the golden dump records desktop frames with no
discovered application in them, and every fixture list is unwatched.

### 55.5 Sensitivity

With invalidation disabled — `if true || now_ms < self.next_ms`, so `poll` never
compares — the journey fails on the assertion that matters:

```
panicked at litric-shell/src/e2e.rs:5116:5:
assertion `left == right` failed: the entry written while the shell ran is not in the list; status: None
  left: 0
 right: 1
```

Restored, the same journey passes (`test result: ok. 1 passed`) and
`rg 'true \|\| now_ms'` over the module returns nothing.

### 55.6 Limits

* **One second of latency.** A change is seen on the next tick at or after the
  interval; there is no event, so an install is not instant.
* **A change beyond the count cap and inside the same second can be missed.**
  The count stops at 512 entries per directory; past that only the mtime is
  compared, and on a filesystem with one-second mtime granularity an add and a
  removal inside the same second would cancel out. Real directories are far
  below the cap.
* **An icon file added inside an existing theme is not noticed.** The watch
  compares directory stamps; `<theme>/scalable/apps/foo.svg` appearing does not
  change the theme directory's stamp. Installing the theme, or changing which
  theme a name resolves in, is noticed.
* **A change reloads the whole list.** Discovery re-runs over every applications
  directory when the stamp moves; it is bounded (`MAX_FILES_PER_DIR`, `MAX_APPS`)
  and measured in §52, not incremental.
* **No `inotify`, no threads.** This is a poll by design: the shell crate forbids
  `unsafe` and the workspace takes no dependencies.
## 56. Batch: an install re-reads only the directory that changed (verified)

### 56.1 Requirement and audit

Source: this file's own §55.6 limit, quoted verbatim:

> **A change reloads the whole list.** Discovery re-runs over every applications
> directory when the stamp moves; it is bounded (`MAX_FILES_PER_DIR`, `MAX_APPS`)
> and measured in §52, not incremental.

and the same `G.07,G,Icons,PARTIAL` row: on a machine with several XDG
`applications` directories, installing one package re-parsed every entry in all
of them and re-resolved every icon.

The audit found the shape of the problem in two places:

* `DesktopWatch::poll` answered only *whether* something changed (a `bool`), so
  the caller had no choice but to call `discover_in_with` again over every
  directory.
* `discover_in_with` walked `dirs` from scratch and resolved every icon, with no
  way to say "these entries are already validated" or "this name already
  resolved" — the work was a function of the machine, not of the change.

This batch makes the stamp name the directories that moved (`Change` carries
`app_dirs` and `icons`), and adds `reload_tracked`, which re-reads only those,
reuses the previous entries for the rest, and re-resolves icons only when an icon
directory moved. The two counts it returns (`dirs_read`, `files_read`) are kept on
the shell as diagnostics, so "only one directory was read" is an assertion, not a
claim.

### 56.2 The connection

| File | Line | What it connects |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 339 | `Change`: which application directories moved, and whether the icon tree did |
| `src/litric-shell/src/desktop_entry.rs` | 412 | `poll` fills that answer instead of a boolean; no syscall before the interval |
| `src/litric-shell/src/desktop_entry.rs` | 104 | `Discovery`: the list plus `dirs_read` and `files_read` |
| `src/litric-shell/src/desktop_entry.rs` | 125 | `discover_tracked`: a full discovery that reports its work |
| `src/litric-shell/src/desktop_entry.rs` | 142 | `reload_tracked`: reads `reread`, reuses `keep`, resolves icons only when told to |
| `src/litric-shell/src/desktop_entry.rs` | 972 | unit test: two directories, a new entry in the second, one directory read, the first directory's entry kept with its own source |
| `src/litric-shell/src/desktop_entry.rs` | 1027 | unit test: theme installed after the watch armed, zero directories and zero files read, the icon resolved for the entry already known |
| `src/litric-shell/src/shell.rs` | 3099 | `desktop_dirs_read` / `desktop_files_read`: the last reload's work, for the journal and the tests |
| `src/litric-shell/src/shell.rs` | 7286 | `refresh_desktop`: map `Change` to the reread list, clear the icon cache only when icons moved, reload, journal the counts |
| `src/litric-shell/src/e2e.rs` | 5068 | journey: two real directories, one entry installed while the launcher is open, `dirs_read == 1`, both tiles drawn |
| `ci/lint-file-safety.py` | 142 | the journey's temporary-root cleanup, recorded with its reason |

### 56.3 Cost

Measured on this machine:

* **The check is unchanged:** 0.202 ms per check over 2 directories
  (`one_check_costs_a_bounded_number_of_directory_reads`, 20 polls inside one
  `Instant`). Nothing was added to the poll path; the only new work happens on a
  change.
* **A full discovery reads what it always did:** 2 directories and 2 files in the
  two-directory unit fixture.
* **An incremental reload after one install reads 1 directory and 1 file** in the
  e2e journey, and 1 directory and 2 files in the unit fixture (the changed
  directory holds two entries there). The unchanged directory's entries are
  reused, not re-parsed.
* **An icon-only change reads 0 directories and 0 files.** The decoded icon cache
  is still dropped (a name may resolve elsewhere), but nothing is parsed.
* **No per-frame cost:** `refresh_desktop` is still one `Option` test until the
  interval elapses, and the §55 journey's "30 frames, no `stat`" assertion still
  passes.

### 56.4 Tests and gates

Tests added: 2 unit tests (`an_incremental_reload_reads_only_the_changed_directory`,
`an_icon_only_change_reads_no_desktop_file_at_all`) and 1 e2e journey
(`journey_launcher_reloads_only_the_changed_applications_directory`), 3 in total.

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | 1792 passed, 0 failed |
| `python3 ci/lint-doc-refs.py` | OK, every citation resolves |
| `python3 ci/lint-file-safety.py` | exit 0, with the new journey recorded at line 142 |
| `bash ci/verify.sh` | `summary: 32 pass, 2 fail, 3 skipped` |

The two failures are the unchanged absent local dev-key material; the three skips
are the QEMU-only checks. No gate was weakened, no key was generated, and no
golden frame changed: every fixture list is unwatched, and the golden dump has no
discovered application in it.

### 56.5 Sensitivity

With the incremental path disabled — the reread list forced to every application
directory, which is exactly the behaviour §55 shipped — the journey fails on the
count that matters:

```
panicked at litric-shell/src/e2e.rs:5100:5:
assertion `left == right` failed: a reload read 2 directories; only the changed one should be read
  left: 2
 right: 1
```

Restored, the same journey passes (`test result: ok. 1 passed`).

### 56.6 Limits

* **A change still re-resolves every icon** when an icon directory moved: the
  entries are not re-parsed, but each `Icon=` name is looked up again (one bounded
  `find_in` per entry, `MAX_APPS` at most). A per-name negative cache would be the
  next step; it is not claimed here.
* **Kept entries are trusted.** Their files were validated when the directory was
  last read; a file that becomes invalid *without* changing its directory's
  stamp (a rewrite inside the same second on a coarse-mtime filesystem) is not
  re-validated until that directory is read again.
* **Precedence is preserved by id, not by file mtime.** A newer entry in a later
  directory still loses to an id an earlier directory owns, exactly as a full
  discovery does.
* **One second of latency** and the 1-second mtime granularity caveats from §55
  are unchanged.
## 57. Batch: one icon name is resolved once per state of the icon tree (verified)

### 57.1 Requirement and audit

Source: this file's own §56.6 limit, quoted verbatim:

> **A change still re-resolves every icon** when an icon directory moved: the
> entries are not re-parsed, but each `Icon=` name is looked up again (one bounded
> `find_in` per entry, `MAX_APPS` at most). A per-name negative cache would be the
> next step; it is not claimed here.

The audit found the cost in one place and the missing piece in another:

* `reload_with_cache`'s resolve pass called `themeicons::find_in(icon_dirs, icon,
  48)` once per entry, so a desktop suite whose entries all say `Icon=office`
  walked the theme tree once per entry — the same handful of names, answered
  again and again.
* The shell already knew *whether* the icon tree moved (`Change::icons`, from
  `DesktopWatch`), and already dropped the decoded pixel cache then, but nothing
  remembered the name-to-file answer, so "moved" always meant "ask every name
  again".

This batch adds `IconNames`: one map of `Icon=` name to resolved path (or to a
remembered miss), guarded by the stamps of the icon roots. A name is asked once
per state of the tree; when a root's stamp moves the map is cleared before the
next lookup, so a remembered miss — or another theme's file — is never trusted
across a theme change. Distinct names resolve once no matter how many entries
share them.

### 57.2 The connection

| File | Line | What it connects |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 296 | `MAX_ICON_NAMES = 512`: the map is cleared rather than grown past it |
| `src/litric-shell/src/desktop_entry.rs` | 322 | `IconNames`: dirs, their stamps, the name map, and the counters |
| `src/litric-shell/src/desktop_entry.rs` | 344 | `refresh`: compares the icon-root stamps and clears the map when they moved |
| `src/litric-shell/src/desktop_entry.rs` | 355 | `resolve`: memo hit, else one `themeicons::find_in`, bounded insert |
| `src/litric-shell/src/desktop_entry.rs` | 401 | `bytes`: what the map holds, for the measured bound |
| `src/litric-shell/src/desktop_entry.rs` | 113 | `Discovery::icons_resolved`: distinct names asked in that pass |
| `src/litric-shell/src/desktop_entry.rs` | 173 | `reload_with_cache`: the resolve passes go through the memo |
| `src/litric-shell/src/shell.rs` | 3104 | `desktop_icons_resolved` and `desktop_icon_names`: the counter and the memo, kept across reloads |
| `src/litric-shell/src/shell.rs` | 7297 | `refresh_desktop`: hands the memo to the reload and records the count |
| `src/litric-shell/src/e2e.rs` | 5068 | journey: two entries with one icon name installed while the launcher is open, one resolve, two drawn tiles |
| `src/litric-shell/src/desktop_entry.rs` | 1137 | unit: three entries, one name, one resolve and two memo hits; a second pass resolves nothing |
| `src/litric-shell/src/desktop_entry.rs` | 1193 | unit: a remembered miss is not reused after the theme arrives |
| `ci/lint-file-safety.py` | 142 | the journey's temporary-root cleanup, recorded with its reason |

### 57.3 Cost

Measured on this machine:

* **Three entries sharing one name: 1 resolve, 2 memo hits**; a second full pass
  over the same tree: **0 resolves** (all names already known).
* **An installed pair in the e2e:** 2 entries, 2 files, **1 resolve** — the number
  the assertion reads, and 2 when the memo is bypassed (§57.5).
* **Memory for that fixture: 215 bytes** for one name (`IconNames::bytes`, printed
  by the unit test). The asserted ceiling is `MAX_ICON_NAMES × (255 + 512 + 96)`
  = **441 856 bytes** for a full map of 512 names, a lower bound that excludes
  allocator bookkeeping; the map is cleared rather than grown past 512.
* **No per-frame work:** the memo is only touched during a discovery or a reload.
  The "30 frames, no `stat`" assertion of §55 and the "one check a second" of §56
  still pass unchanged.
* **No new dependency, no unsafe.** One `HashMap<String, Option<PathBuf>>`, one
  stamp per icon root, two counters.

### 57.4 Tests and gates

Tests added: 2 unit tests (`entries_sharing_one_icon_name_resolve_it_once`,
`a_theme_change_does_not_reuse_a_negative_lookup`) and 1 e2e journey
(`journey_launcher_resolves_a_shared_icon_name_once`), 3 in total.

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | 1795 passed, 0 failed |
| `python3 ci/lint-doc-refs.py` | OK, every citation resolves |
| `python3 ci/lint-file-safety.py` | exit 0, with the new journey recorded at line 142 |
| `bash ci/verify.sh` | `summary: 32 pass, 2 fail, 3 skipped` |

The two failures are the unchanged absent local dev-key material; the three skips
are the QEMU-only checks. No gate was weakened, no key was generated, and no
golden frame changed: the golden dump has no discovered application in it.

The frame the journey saved (`artifacts/browser-shots/57-launcher-icon-name-cache.png`)
was opened and checked: both tiles show the document's blue field, amber disc and
white triangle, legible at 82 px — not a placeholder square.

### 57.5 Sensitivity

With the memo bypassed — `resolve` returning straight from `themeicons::find_in`
and counting every call — the same journey fails on the counter:

```
panicked at litric-shell/src/e2e.rs:5105:5:
assertion `left == right` failed: two entries share one icon name, so the theme tree should be asked once, not 2 times
  left: 2
 right: 1
```

Restored, the journey passes (`test result: ok. 1 passed`) and `rg 'if true \{'`
over the module returns nothing.

### 57.6 Limits

* **The whole map is dropped when the icon tree moves**, not the entries that
  changed: one `find_in` per distinct name after a theme install, which is the
  intended semantics rather than a per-name stamp. A page that churns themes
  pays that once per change.
* **A remembered miss lives only until the next stamp change.** Without a theme
  install, a name that resolves to nothing stays remembered as nothing; that is
  correct for a tree that did not move, and §55's watch is what moves it.
* **No sharing between processes.** Each shell keeps its own map; the map is not
  written to disk.
* **Cleared, not evicted, at the cap.** Past 512 names the map empties rather
  than tracking ages, so a session that asked 513 distinct names re-resolves the
  ones it asks next. With `MAX_APPS` at 128 this needs deliberate churn to reach.
* **`bytes` is a lower bound.** It counts names, paths and per-entry headers, not
  the allocator's own overhead, so the real RSS figure is somewhat higher.
* **One second of latency** and the mtime-granularity caveats from §55 are
  unchanged.

## 58. Batch: one installed theme directory re-resolves only the names it supplies (verified)

### 58.1 Requirement

§57 closed with the cost it could not remove, in its own words:

> **The whole map is dropped when the icon tree moves**, not the entries that
> changed: one `find_in` per distinct name after a theme install, which is the
> intended semantics rather than a per-name stamp. A page that churns themes
> pays that once per change.

This batch replaces that whole-map drop with per-name validation, and proves the
difference with a counter taken from a running launcher.

### 58.2 Audit, before the change

* `IconNames` held `order: Vec<(PathBuf, Stamp)>` — every icon root's flat
  directory, then each root's themes, `hicolor` first — plus `names: HashMap<String, Option<PathBuf>>`.
* `IconNames::refresh` rebuilt the order and compared it to the previous one. If
  the *path list* differed at all, it cleared the map. So appending one theme
  directory at the very tail — a directory that cannot shadow anything already
  answered — re-resolved every name.
* A cached entry recorded only its answer, never *which directory* answered, so
  there was nothing to validate a name against on its own.
* `themeicons::find_in` searches: each root's flat `name.png`/`name.svg` first,
  then the roots' themes (`hicolor` first, then by name), then inherited parents
  breadth-first.

### 58.3 What a cached hit now stands on

Each entry keeps `path`, the index of the directory that answered it
(`witness`), and that directory's stamp. `refresh` computes the longest common
prefix between the old and the new order:

* a hit at index `k` survives iff `k < first_changed`, its witness' stamp is
  unchanged, and the file still exists — only an earlier or equal directory can
  shadow it, and a later one loses to it;
* a directory appended at the tail is beyond every existing witness, so it costs
  only the names it actually supplies;
* a removed or renamed directory ends the common prefix there, so every entry
  that pointed into it — or past it — is dropped;
* a miss carries no witness, so any change drops it;
* `Inherits=` in a changed theme is the one case the flat order cannot model (a
  parent is searched after every direct theme, not where its name sorts), so
  that change clears the map. Conservative: more work than needed, never less.

### 58.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 304 | `MAX_ICON_NAMES = 512` |
| `src/litric-shell/src/desktop_entry.rs` | 308 | `struct Cached { path, witness, stamp }` |
| `src/litric-shell/src/desktop_entry.rs` | 353 | `pub struct IconNames` with `resolved`/`hits`/`validated` |
| `src/litric-shell/src/desktop_entry.rs` | 364 | `icon_order` — the conservative search order |
| `src/litric-shell/src/desktop_entry.rs` | 400 | `declares_inherits` — the fallback trigger |
| `src/litric-shell/src/desktop_entry.rs` | 433 | `refresh` — common-prefix invalidation, per-name prune |
| `src/litric-shell/src/desktop_entry.rs` | 500 | `resolve` — witness recorded with the answer |
| `src/litric-shell/src/desktop_entry.rs` | 550 | `validated()` — the new counter |
| `src/litric-shell/src/desktop_entry.rs` | 570 | `bytes()` — the memory figure |
| `src/litric-shell/src/desktop_entry.rs` | 117 | `Discovery::icons_validated`, reported per pass |
| `src/litric-shell/src/shell.rs` | 3107 | `desktop_icons_validated` diagnostic |
| `src/litric-shell/src/shell.rs` | 7285, 7335 | set on load and on each reload |
| `src/litric-shell/src/shell.rs` | 7337 | the journal line now carries both numbers |

### 58.5 Tests that fail without it

| Test | Asserts |
| --- | --- |
| `desktop_entry::tests::a_new_theme_directory_re_resolves_only_the_name_it_supplies` (1206) | three entries on one name: 1 resolve, 0 validated; after a new theme dir with one new name: **1 resolve, 1 validate**, 2 names |
| `desktop_entry::tests::a_name_that_moves_to_an_earlier_directory_is_re_resolved` (1289) | a flat file appearing at the root (index 0) shadows the theme's answer: the new path is returned |
| `desktop_entry::tests::removing_a_theme_directory_drops_the_names_it_answered` (1341) | uninstall: the answer becomes `None`, not a path to a deleted file |
| `desktop_entry::tests::a_change_in_an_inheriting_theme_re_resolves_everything` (1386) | `Inherits=` change falls back: 2 resolves, 0 validated |
| `e2e::journey_launcher_re_resolves_only_the_name_a_new_theme_supplies` (5161) | real temp XDG tree, launcher running: 2 entries share `blue-icon`, a `zzz-extra` theme plus `Late App` arrive, `tick_after(1_010)` → **`desktop_icons_resolved == 1`, `desktop_icons_validated == 1`**; the journal reports `1 directories, 3 entries, 1 names resolved, 1 names validated (... icons true)` |

The §57 tests (`entries_sharing_one_icon_name_resolve_it_once`,
`a_theme_change_does_not_reuse_a_negative_lookup`, and the
`journey_launcher_resolves_a_shared_icon_name_once` journey) pass unchanged.

### 58.6 Sensitivity

With `refresh` forced back to whole-map invalidation (`index < first_changed`
replaced by `false`), both the unit test and the journey fail on the counter:

```
thread 'desktop_entry::tests::a_new_theme_directory_re_resolves_only_the_name_it_supplies' panicked at litric-shell/src/desktop_entry.rs:1266:9:
assertion `left == right` failed: only the new name is new work: 2 names resolved
  left: 2
 right: 1
```

```
thread 'e2e::journey_launcher_re_resolves_only_the_name_a_new_theme_supplies' panicked at litric-shell/src/e2e.rs:5217:5:
assertion `left == right` failed: only the new name is new work; a whole-map drop would report 2, this reports 2
  left: 2
 right: 1
```

Restored, `cargo test -p litric-shell --lib -- desktop_entry::tests::a_new_theme
e2e::journey_launcher_re_resolves` reports `5 passed; 0 failed`.

### 58.7 Cost, measured on this machine

| Figure | Value | How |
| --- | --- | --- |
| bytes for the 4-name / 3-entry fixture | **636 B** (16-entry fixture: 923 B) | `IconNames::bytes()` printed by the test |
| per-entry header | **64 B** | `size_of::<Cached>()` |
| unchanged pass (16 entries, 4 names) | **53.8 µs** each over 2000 passes | `a_pass_over_an_unchanged_tree_is_cheap` |
| names resolved when one theme dir + one name is installed | **1** (was 2 with the whole map dropped) | the e2e counter |
| names validated in that pass | **1** | `desktop_icons_validated` |
| work during frames | **none** | `refresh` is only called from the reload path, which the watch gates at `WATCH_INTERVAL_MS`; §55's "30 frames, no stat" and §56's "poll only at period" tests still pass |
| ceiling | 512 names; by the header and path sizes above ≈ 0.42 MiB worst case | `MAX_ICON_NAMES` and the `bytes()` bound assertion |

### 58.8 Gates

```
$ cargo fmt --all --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.59s
CLIPPY_EXIT=0

$ cargo test --workspace
PASSED=1800 FAILED=0          (55 suites)

$ python3 ci/lint-doc-refs.py
OK: 1031 citation(s) across 155 document(s) resolve

$ python3 ci/lint-file-safety.py
FILESAFETY_EXIT=0
  (the new journey's fixture cleanup is registered with its reason)

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the pre-existing `keys/dev/` key material and the mkosi
image check; the three skips are QEMU-only checks. No gate, golden frame or key
was added, weakened or generated — the golden dump has no discovered
application in it, so it could not move.

The frame the journey saved
(`artifacts/browser-shots/58-launcher-per-name-icon-cache.png`) was opened: three
discovered tiles — `Blue One` and `Blue Two` on the blue field with an amber
disc, `Late App` on a red field with a white triangle — legible at 82 px, none a
placeholder square.

### 58.9 Limits

* **The watch is one directory deep.** A new icon file written *inside* an
  existing theme's `scalable/apps/` changes no watched stamp, so it is only
  noticed when something else moves that theme's own listing. Measured while
  writing the `Inherits=` test: adding a second icon inside the theme's scalable
  directory left the pass at 0 validated, while adding a plain file directly in
  the theme directory moved it. The fix is a deeper stamp (per size directory)
  or an inotify watch; neither is in this batch.
* **`Inherits=` changes clear the whole map.** Correct, but not per-name: a
  theme that inherits cannot be placed in a flat order, so its install costs one
  `find_in` per distinct name.
* **A remembered miss is dropped on any change**, because a change anywhere can
  supply a missing name. Hits are validated per name; misses are not.
* **The common prefix is conservative across roots.** Adding a new icon *root*
  ahead of existing ones shifts every theme's index and therefore re-resolves
  everything; only appends at the tail are cheap.
* **Cleared, not evicted, at the cap.** Past 512 names the map empties.
* **`bytes` is a lower bound** — it counts names, paths and headers, not the
  allocator's own overhead.
* **One second of latency** and the mtime-granularity caveats from §55 stand.

### 58.10 Git

Commit `HEAD` and `origin/main` are the same commit; `git status
--porcelain --untracked-files=no` is empty.

## 59. Batch: an icon written into an installed theme appears without a restart (verified)

### 59.1 Requirement

§58 closed with this limit, in its own words:

> **The watch is one directory deep.** A new icon file written *inside* an
> existing theme's `scalable/apps/` changes no watched stamp, so it is only
> noticed when something else moves that theme's own listing. Measured while
> writing the `Inherits=` test: adding a second icon inside the theme's scalable
> directory left the pass at 0 validated, while adding a plain file directly in
> the theme directory moved it. The fix is a deeper stamp (per size directory)
> or an inotify watch; neither is in this batch.

This batch makes that file visible to a running launcher, and bounds what the
deeper look costs.

### 59.2 Audit, before the change

* `icon_order` stamped each theme directory with `stamp_of(theme)` — that
  directory's modification time plus its entry count, capped at
  `MAX_WATCH_ENTRIES = 512`. `DesktopWatch::poll` watched the application
  directories, the icon roots and the theme directories, the same way.
* A theme directory's children are its *size* directories (`48x48`, `scalable`,
  ...). The icon files live one level further, in `<size>/<category>`. Writing
  `scalable/apps/foo.svg` therefore touches neither the theme's mtime nor its
  entry count.
* Measured on this machine, in Rust, before designing anything: creating a file
  in a directory left that directory's modification time **byte-identical**
  (`mtime unchanged after file create: true`), and creating a subdirectory did
  the same (`true`). The entry count is the only signal this filesystem offers.
  §55 and §56 already relied on the count for application directories; this is
  the same fact applied one level deeper.
* Measured costs, so the budget is a number rather than a guess: `metadata`
  1.83 µs; `read_dir` on a 300-entry directory 137.03 µs; on a 3-entry directory
  3.02 µs. The cost of a leaf stamp is therefore proportional to the files in
  it, which is exactly why the scan is sliced.
* `themeicons::find_in_theme` reads `<theme>/<size>/<category>/name.png`, then
  `<theme>/scalable/<category>/name.svg` and `.png`, then the flat
  `<theme>/name.*`. `scalable` and the numeric sizes are the directories the
  scan has to cover; the flat case is already covered by the theme's own stamp.

### 59.3 What the deep scan is, and what it stands on

Each theme contributes up to `MAX_LEAVES_PER_THEME = 16` leaf directories:
`scalable` first, then the numeric size directories nearest 48 px, five
categories each, so a theme with one size directory and `scalable` contributes
ten. `IconNames` records each leaf with the theme that owns it and a stamp taken
when the order is built — **not** on the first check, because a file written
between arming the cache and the first check would otherwise be recorded as part
of the baseline and never reported.

`scan_leaves` looks at `LEAVES_PER_SCAN = 12` leaves per call, round-robin, and
reports whether any moved. When one did, `forget_themes` drops every name
witnessed in that theme and every remembered miss, and leaves the names another
theme answered alone. That is §58's per-name property, fed by a signal one
directory deeper. A full cycle over the caps takes six checks, and checks happen
at most once per `WATCH_INTERVAL_MS`.

### 59.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 315 | `MAX_LEAVES_PER_THEME = 16` — leaves one theme contributes |
| `src/litric-shell/src/desktop_entry.rs` | 321 | `MAX_THEME_LEAVES = 64` — leaves tracked across every theme |
| `src/litric-shell/src/desktop_entry.rs` | 330 | `LEAVES_PER_SCAN = 12` — the per-check budget |
| `src/litric-shell/src/desktop_entry.rs` | 342 | `theme_leaves` — `scalable` + numeric size dirs, one level down per category |
| `src/litric-shell/src/desktop_entry.rs` | 441 | `IconNames::leaves` — leaf, owning theme, stamp |
| `src/litric-shell/src/desktop_entry.rs` | 595 | `capture_leaves` — stamps the leaves when the order is built |
| `src/litric-shell/src/desktop_entry.rs` | 622 | `scan_leaves` — one bounded slice per check, per-name invalidation |
| `src/litric-shell/src/desktop_entry.rs` | 663 | `forget_themes` — drops hits witnessed there and every miss |
| `src/litric-shell/src/desktop_entry.rs` | 726 | `leaf_scans()` / `leaves_moved()` / `leaf_count()` counters |
| `src/litric-shell/src/desktop_entry.rs` | 995 | `DesktopWatch::due` — the interval gate, no disk touched |
| `src/litric-shell/src/shell.rs` | 7306 | the check runs only when the interval elapsed |
| `src/litric-shell/src/shell.rs` | 7314 | the deep scan, once per check |
| `src/litric-shell/src/shell.rs` | 7327 | a leaf change is an icon change: decoded copies dropped, entries re-resolved |
| `src/litric-shell/src/shell.rs` | 7347 | journal line reports the scans and the tracked leaf count |
| `src/litric-shell/src/themeicons.rs` | 78 | one category list shared by `find_in_theme` and the scan |
| `src/litric-shell/src/e2e.rs` | 5286 | module-level chip sampler (a nested `fn` would own the journey's cleanup) |
| `src/litric-shell/src/e2e.rs` | 5332 | the e2e journey |
| `src/litric-shell/src/desktop_entry.rs` | 1955 | unit: new icon in an existing theme |
| `src/litric-shell/src/desktop_entry.rs` | 2050 | unit: the budget bites, and the measured cost |

### 59.5 Tests that fail without it

| Test | Asserts |
| --- | --- |
| `desktop_entry::tests::a_new_icon_in_an_existing_theme_is_noticed_by_the_deep_scan` | a warm cache with one hit (`zzz-other`) and one miss; a file written into `hicolor/scalable/apps` is reported by `scan_leaves`, `leaves_moved()` is 1, and the reload reports **1** name resolved with the hit kept |
| `desktop_entry::tests::a_deep_scan_inspects_only_its_budget` | a theme offering 55 leaves is capped at 16, one scan inspects exactly 12, and the measured cost is printed |
| `e2e::journey_launcher_picks_up_an_icon_written_into_an_existing_theme` | real temp XDG tree, launcher running: three entries start with **no** icon file, the artwork is written into the theme that already exists, six watched ticks pick it up, `desktop_icons_resolved == 2`, the pixels change, and both artworks are on the tiles |

The §53/§54 SVG and inheritance tests and all four §57/§58 tests pass unchanged.

### 59.6 Sensitivity

With `scan_leaves` forced to return `false` on entry (the deep scan off, the
§58 behaviour), the journey fails on the real assertion:

```
thread 'e2e::journey_launcher_picks_up_an_icon_written_into_an_existing_theme' panicked at litric-shell/src/e2e.rs:5388:5:
an icon written into an existing theme was not picked up in 6 watched ticks (0 leaf scans of 5)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 714 filtered out
```

Restored, the same journey passes and the journal prints
`icons true after 5 leaf scans of 5`.

### 59.7 Cost, measured on this machine

| Figure | Value | How |
| --- | --- | --- |
| one bounded scan, 16 tracked leaves, one of them holding 200 files | **158.4 µs** | printed by `a_deep_scan_inspects_only_its_budget` (debug test build, 200 scans averaged) |
| `metadata` / `read_dir` on a 300-entry dir | 1.83 µs / 137.03 µs | `rustc -O` probe over 2000 / 500 iterations |
| memory held by the leaf list, 16 leaves | **3 519 B** for the whole cache | `IconNames::bytes()` |
| work per frame | **none** | the scan is reached only from `refresh_desktop`, behind `DesktopWatch::due`; §55's "30 frames, no stat" and §56's "poll only at period" tests still pass |
| ceiling | 64 leaves tracked, 12 looked at per check → at most 12 `read_dir` calls per second across the whole desktop | `MAX_THEME_LEAVES`, `LEAVES_PER_SCAN` |

### 59.8 Gates

```
$ cargo fmt --all --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.24s
CLIPPY_EXIT=0

$ cargo test --workspace
PASSED=1804 FAILED=0

$ python3 ci/lint-doc-refs.py
OK: 1063 citation(s) across 155 document(s) resolve

$ python3 ci/lint-file-safety.py
FILESAFETY_EXIT=0
  (the new journey's fixture cleanup is registered with its reason)

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the pre-existing `keys/dev/` key material and the mkosi
image check; the three skips are QEMU-only checks. No gate, golden frame or key
was added, weakened or generated. `verify.sh` run without `cargo` on `PATH`
reports `25 pass, 2 fail, 5 skipped`, which is the same script declining to run
five Rust checks — the numbers above are from the documented environment.

Three tests were added this turn (`#[test]` attributes in the two files went
20 → 22 and 81 → 82); §58's `PASSED=1800` is the figure it recorded, and no test
was removed or renamed.

The two frames the journey saved were opened:

* `artifacts/browser-shots/59-before-the-icon-lands.png` — three tiles on the
  generated glyph, no artwork anywhere in the theme.
* `artifacts/browser-shots/59-launcher-icon-lands-in-existing-theme.png` — the
  same three tiles now carrying the documents that arrived: `Deep One` and
  `Deep Two` on a blue field with an amber disc, `Late Deep` on a red field with
  a white triangle, legible at 82 px, none a placeholder square.

### 59.9 Limits

* **A rename inside one directory is invisible.** The leaf stamp is the entry
  count, because this filesystem moves no directory mtime; renaming `a.svg` to
  `b.svg` keeps the count. A file *added* or *removed* is seen.
* **`symbolic/` is not scanned.** Only `scalable` and numeric `WxH` size
  directories are tracked, which are the directories `find_in_theme` reads.
* **Sixteen leaves per theme.** A theme with more than three size directories on
  top of `scalable` loses its farthest sizes — the ones the launcher's 48 px
  lookup is least likely to answer from.
* **Latency is a cycle, not a tick.** At the caps a full pass takes six checks;
  a change is noticed when the cursor reaches its leaf, so up to ~6 s.
* **The scan is armed only by `load_desktop_apps_from`.** A fixture list placed
  by `load_desktop_apps` is still never watched, by design.
* **`Inherits=` still clears the whole name map** on an order change (§58).
* **Misses are still dropped by any change that touches their tree**, and the
  512-name cap still clears rather than evicts.
* **No inotify.** The crate is `#![forbid(unsafe_code)]` and dependency-free, so
  this remains a polled stamp, with the one-second floor from §55.

### 59.10 Git

Commit `HEAD` and `origin/main` are the same commit; `git status
--porcelain --untracked-files=no` is empty.

## 60. Batch: an icon rewritten in place repaints without a restart (verified)

### 60.1 Requirement

§59 closed with this blind spot, in its own words:

> **A rename inside one directory is invisible.** The leaf stamp is the entry
> count, because this filesystem moves no directory mtime; renaming `a.svg` to
> `b.svg` keeps the count. A file *added* or *removed* is seen.

A package upgrade that rewrites an icon file is the same shape of change: the
path stays, the entry count stays, nothing the leaf scan watches moves. This
batch notices it and repaints the tile with the new pixels.

### 60.2 Audit, before the change

Measured on this machine, on one file in one directory:

| Moment | entries in the directory | directory mtime | file mtime | file length |
| --- | --- | --- | --- | --- |
| after the first write | 1 | 1793273352929382735 | 1793273352929382735 | 6 |
| after rewriting the same path | 1 | 1793273352929382735 | 1793273352929382735 | 54 |

So a rewrite moves **neither** mtime, and the entry count is unchanged: the
§59 leaf stamp is blind to it by construction, exactly as its limits section
said. The only thing that moved is the file's length.

Also audited before touching anything:

* `themeicons::scaled` caches decoded pixels in a thread-local keyed by
  `(PathBuf, usize)`. A rewrite keeps the path, so even a correct `icon_path`
  would be drawn from the stale copy unless the cache is dropped.
* `IconNames::resolve` recorded the path, the witness directory and that
  directory's stamp — nothing about the file itself, so there was nothing to
  compare a rewrite against.
* `shell.rs::refresh_desktop` treated `change.icons || leaves_moved` as "the icon
  tree moved", which is what drops the decoded copies and forces a repaint.

### 60.3 What the file scan stands on

Each cached answer now records `FileStamp { len, mtime }` of the file it
resolved to, taken **at resolve time**, not at the first scan: a file rewritten
between arming the cache and the first check would otherwise become the
baseline and never be reported.

`scan_files` walks `slots` — the names in the order they were resolved, because
a `HashMap` has no order and sorting it on every check would cost more than the
`stat` calls it exists to make — and re-stamps `FILES_PER_SCAN = 32` of them per
check, round-robin. A name whose stamp moved is reported once; the stored stamp
becomes the new baseline. The cached answer is **not** dropped: the path is still
the right answer, so nothing is re-resolved (`icons_resolved == 0` is asserted).
What the shell does is `clear_cache()` and repaint, which is what makes the new
pixels reach the tile.

### 60.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 339 | `FILES_PER_SCAN = 32` — metadata calls per check |
| `src/litric-shell/src/desktop_entry.rs` | 403 | `FileStamp { len, mtime }` — what a resolved file looked like |
| `src/litric-shell/src/desktop_entry.rs` | 409 | `file_stamp` — one `metadata`, never a read |
| `src/litric-shell/src/desktop_entry.rs` | 433 | `Cached::file` — the file's stamp when the name answered |
| `src/litric-shell/src/desktop_entry.rs` | 486 | `IconNames::slots` — resolution order for the round-robin walk |
| `src/litric-shell/src/desktop_entry.rs` | 735 | `remember_slot` — appends, rebuilds sorted at the cap |
| `src/litric-shell/src/desktop_entry.rs` | 755 | `scan_files` — bounded re-stat, reports a rewritten file |
| `src/litric-shell/src/desktop_entry.rs` | 799 | `file_stats()` / `files_changed()` / `slot_count()` counters |
| `src/litric-shell/src/shell.rs` | 7318 | the file scan, once per check |
| `src/litric-shell/src/shell.rs` | 7331 | a rewritten file drops the decoded copies and repaints |
| `src/litric-shell/src/shell.rs` | 7351 | journal line carries both counters |
| `src/litric-shell/src/e2e.rs` | 5436 | the e2e journey |
| `src/litric-shell/src/desktop_entry.rs` | 2248 | unit: rewrite noticed, answer kept |
| `src/litric-shell/src/desktop_entry.rs` | 2320 | unit: the budget bites, and the measured cost |

### 60.5 Tests that fail without it

| Test | Asserts |
| --- | --- |
| `desktop_entry::tests::a_rewritten_icon_file_is_noticed_by_the_file_scan` | a warm name: `scan_files()` is false, then the file is overwritten in place and it is true, `files_changed() == 1`, and the reload reports **0** names resolved with the same path |
| `desktop_entry::tests::a_file_scan_inspects_only_its_budget` | 40 resolved names: one check re-stats exactly 32, and the cost is printed |
| `e2e::journey_launcher_picks_up_a_rewritten_icon_file` | real temp XDG tree, launcher running: the tile carries the first artwork, the same path is rewritten with different artwork, eight watched checks pick it up, `desktop_icons_resolved == 0`, and the tile's pixels become the new artwork |

The §53/§54 SVG and inheritance tests, the four §57/§58 tests and the two §59
tests pass unchanged.

### 60.6 Sensitivity

With `scan_files` forced to return `false` on entry (the §59 behaviour), the new
journey fails on its own assertion:

```
thread 'e2e::journey_launcher_picks_up_a_rewritten_icon_file' panicked at litric-shell/src/e2e.rs:5496:5:
a file rewritten in place was not noticed in 8 watched checks (0 file re-stats of 1 slots)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 717 filtered out
```

Restored, the journey passes and the journal prints
`icons true after 5 leaf scans of 5 and 1 file re-stats of 1`.

### 60.7 Cost, measured on this machine

| Figure | Value | How |
| --- | --- | --- |
| one check over 40 resolved names (32 `metadata` calls) | **113.9 µs** | printed by `a_file_scan_inspects_only_its_budget` (debug test build, 200 checks averaged) |
| memory for those 40 names | **11 369 B** for the whole cache | `IconNames::bytes()` |
| work per frame | **none** | reached only from `refresh_desktop`, behind `DesktopWatch::due`; §55's "30 frames, no stat" and §56's "poll only at period" tests still pass |
| ceiling | 32 files per check, 512 slots → a full cycle in 16 checks | `FILES_PER_SCAN`, `MAX_ICON_NAMES` |
| a `metadata` call, for scale | 1.83 µs | the `rustc -O` probe from §59 |

### 60.8 Gates

```
$ cargo fmt --all --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.31s
CLIPPY_EXIT=0

$ cargo test --workspace
PASSED=1807 FAILED=0

$ python3 ci/lint-doc-refs.py
OK: 1077 citation(s) across 155 document(s) resolve

$ python3 ci/lint-file-safety.py
FILESAFETY_EXIT=0
  (the new journey's fixture cleanup is registered with its reason)

$ bash ci/verify.sh
[1;32m32[0m pass, [1;31m2[0m fail, [1;33m3[0m skipped
```

The two failures are the pre-existing `keys/dev/` key material and the mkosi
image check; the skips are QEMU-only checks. No gate, golden frame or key was
added, weakened or generated. Three tests were added this turn (1804 → 1807).

The two frames the journey saved were opened:

* `artifacts/browser-shots/60-before-the-icon-is-rewritten.png` — the `Swap App`
  tile on a blue field with an amber disc.
* `artifacts/browser-shots/60-launcher-replaced-icon.png` — the same tile, same
  position, now a red field with a white triangle: the replacement artwork, not
  the copy the image cache held, and no restart between the two frames.

### 60.9 Limits

* **A same-length rewrite is invisible.** The fingerprint is length plus mtime,
  and measured on this filesystem a rewrite moves neither mtime, so length is
  the only signal. Rewriting a file with different bytes *of the same size*
  would need a content digest, which costs a read per icon per check and is not
  in this batch.
* **Latency is a cycle.** At 512 names a full pass takes 16 checks, so up to
  ~16 s; the launcher's own list is smaller in practice.
* **A rename is still invisible in the same directory** (§59's limit): the leaf
  count is unchanged and the file behind the old name is gone, so the name keeps
  answering a path that no longer exists until something else moves that theme.
  `refresh` does check `Path::is_file`, so the next validation drops it.
* **The stamp is per name, not per file.** Two names that resolve to the same
  file are re-statted separately.
* **Stale slots are skipped, not compacted**, and `slots` is rebuilt sorted when
  it reaches `MAX_ICON_NAMES`.
* **Still a poll** — no inotify, one second floor from §55, `#![forbid(unsafe_code)]`
  and no new dependency.
* **`Inherits=` still clears the whole name map** on an order change (§58).

### 60.10 Git

Commit `HEAD` and `origin/main` are the same commit; `git status
--porcelain --untracked-files=no` is empty.

## 61. Batch: a same-length, same-mtime icon rewrite repaints without a restart (verified)

### 61.1 Requirement

§60 closed with this blind spot, in its own words:

> **A same-length rewrite is invisible.** The fingerprint is length plus mtime,
> and measured on this filesystem a rewrite moves neither mtime, so length is
> the only signal. Rewriting a file with different bytes *of the same size*
> would need a content digest, which costs a read per icon per check and is not
> in this batch.

This batch adds that digest, bounds what it costs, and proves the tile repaints.

### 61.2 Audit, before the change

Two probes, both run before any code was touched:

| Probe | Result |
| --- | --- |
| write 15 bytes, rewrite the same path with 15 different bytes, back to back | len 15 → 15, file mtime **identical**, directory mtime identical, entry count 1 → 1 |
| the same, with the ~1 s gap a real journey has between the two writes | file mtime `…158.000920068` → `…158.962203231` — different, so a `stat` **can** see it |

That second row corrects a claim §59 and §60 made from the first row. A file's
mtime on this filesystem is **bucketed to about a second**: two writes inside
one bucket produce a byte-identical value, two writes in different buckets do
not. So "the mtime never moves" was too strong — it does not move *inside a
bucket*, and it never moves at all when the writer preserves timestamps, which
is what `cp -p`, `install -p` and `rsync -t` do. Both cases are invisible to
every `stat` the shell makes, and both are what this batch has to catch. The
journey §61 adds therefore restores the timestamps with a real `touch -r` and
asserts that both the length and the mtime are unchanged, so the test cannot
silently degrade into measuring the `stat` path.

Audited before designing:

* `FileStamp { len, mtime }` is compared by `scan_files`; `themeicons::scaled`
  caches decoded pixels by `(PathBuf, usize)`, so a same-path rewrite keeps both
  the answer and the stale pixels unless something compares bytes.
* `scan_files` already visits 32 names per check, round-robin, over
  `IconNames::slots`. A digest pass wants the same walk with a much smaller
  budget, because it opens and reads a file where a re-stat only looks at it.
* Measured for scale, from §59's `rustc -O` probe: `metadata` 1.83 µs.

### 61.3 What the digest pass stands on

`DIGEST_BYTES = 512` of the file, FNV-1a over a stack buffer, folded with the
byte count so a short file cannot collide with a prefix of a longer one. It
allocates nothing, needs no dependency, and never reads the whole file.

Two rules make it honest rather than noisy:

* `FileStamp::head` is `None` until a digest is taken. A `None` head is
  **recorded, never reported**, so a name whose digest has not been taken yet
  cannot invent a change, and a file whose length moved in the same pass
  re-baselines instead of reporting the same rewrite twice.
* `scan_files` (length and mtime) runs first every check; when it sees a move it
  sets `head: None`, which is what hands the bytes to the digest pass next.

`scan_digests` walks the same `slots` vector with its own cursor, takes
`DIGESTS_PER_SCAN = 8` per check, and reports a change the shell treats exactly
like the other two signals: drop the decoded copies, repaint, resolve nothing.

### 61.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 349 | `DIGEST_BYTES = 512` — how much of the file the digest reads |
| `src/litric-shell/src/desktop_entry.rs` | 356 | `DIGESTS_PER_SCAN = 8` — its own, smaller budget |
| `src/litric-shell/src/desktop_entry.rs` | 435 | `head_digest` — FNV-1a over a stack buffer, one read, no allocation |
| `src/litric-shell/src/desktop_entry.rs` | 426 | `FileStamp::head` — `None` until measured, so a first look never reports |
| `src/litric-shell/src/desktop_entry.rs` | 473 | `stamped_file` — metadata plus the digest, taken when a name answers |
| `src/litric-shell/src/desktop_entry.rs` | 883 | `scan_digests` — bounded round-robin digest pass |
| `src/litric-shell/src/desktop_entry.rs` | 938 | `head_digests()` / `digests_changed()` counters |
| `src/litric-shell/src/desktop_entry.rs` | 853 | `scan_files` re-baselines the digest only when length or mtime moved |
| `src/litric-shell/src/shell.rs` | 7321 | the digest pass, once per check |
| `src/litric-shell/src/shell.rs` | 7339 | a moved digest drops the decoded copies and repaints |
| `src/litric-shell/src/shell.rs` | 7359 | journal line carries all three counters |
| `src/litric-shell/src/e2e.rs` | 5532 | the e2e journey |
| `src/litric-shell/src/e2e.rs` | 5601 | the journey asserts the rewrite is stat-blind |
| `src/litric-shell/src/desktop_entry.rs` | 2532 | unit: stat blind, digest sees it |
| `src/litric-shell/src/desktop_entry.rs` | 2606 | unit: the digest budget, measured |

### 61.5 Tests that fail without it

| Test | Asserts |
| --- | --- |
| `desktop_entry::tests::a_same_length_rewrite_is_noticed_by_the_head_digest` | the rewrite keeps the length; `scan_files()` is **false** (§60 blind); `scan_digests()` is **true**; the reload resolves **0** names |
| `desktop_entry::tests::a_head_digest_pass_is_bounded_and_measured` | 40 names: one check reads exactly 8, and both passes are timed and printed |
| `e2e::journey_launcher_picks_up_a_same_length_rewrite` | real XDG tree, launcher running: the replacement has the same byte count **and** the original mtime restored by `touch -r`, the journey asserts both, `digests_changed()` reports it, `files_changed() == 0`, `icons_resolved == 0`, and the tile's pixels become the new artwork |

The §53/§54, §57/§58, §59 and §60 tests pass unchanged.

### 61.6 Sensitivity

With `scan_digests` forced to return `false` on entry (the §60 behaviour), the
new journey fails on its own assertion:

```
thread 'e2e::journey_launcher_picks_up_a_same_length_rewrite' panicked at litric-shell/src/e2e.rs:5621:5:
a same-length, same-mtime rewrite was not noticed in 8 watched checks (0 head digests over 1 slots)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 720 filtered out
```

Restored, the journey passes and the journal prints
`icons true after 5 leaf scans of 5, 1 file re-stats of 1 and 1 head digests`.

### 61.7 Cost, measured on this machine

| Figure | Value | How |
| --- | --- | --- |
| one digest pass, 8 head reads | **74.3 µs** | printed by `a_head_digest_pass_is_bounded_and_measured` (debug test build, 100 passes averaged) |
| one stat pass, 32 `metadata` calls | **94.4 µs** | same test, same run — §60 recorded 113.9 µs for the same pass, so expect ±20 % between runs |
| a whole watched check (both passes) | **≈ 169 µs** | the two rows above |
| memory for 40 names | **11 675 B**, up from 11 369 B in §60 → **+306 B, ≈ 7.6 B per name** | `IconNames::bytes()` |
| work per frame | **none** | reached only from `refresh_desktop`, behind `DesktopWatch::due`; §55's "30 frames, no stat" and §56's "poll only at period" tests still pass |
| ceiling | 8 digests and 32 re-stats per check, 512 slots → a full digest cycle in 64 checks | `DIGESTS_PER_SCAN`, `FILES_PER_SCAN`, `MAX_ICON_NAMES` |

The digest reads 512 bytes; the whole file is never read on a check, and the
decode that follows (when something did change) is the existing bounded one.

### 61.8 Gates

```
$ cargo fmt --all --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.03s
CLIPPY_EXIT=0

$ cargo test --workspace
PASSED=1810 FAILED=0

$ python3 ci/lint-doc-refs.py
OK: 1092 citation(s) across 155 document(s) resolve

$ python3 ci/lint-file-safety.py
FILESAFETY_EXIT=0
  (the new journey's fixture cleanup is registered with its reason)

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the pre-existing `keys/dev/` key material and the mkosi
image check; the skips are QEMU-only checks. No gate, golden frame or key was
added, weakened or generated. Three tests were added this turn (1807 → 1810).

The two frames the journey saved were opened:

* `artifacts/browser-shots/61-before-the-same-length-rewrite.png` — the `Same App`
  tile on a blue field with an amber disc.
* `artifacts/browser-shots/61-launcher-same-length-rewrite.png` — the same tile
  after a rewrite that kept both the byte count and the mtime, now a red field
  with a white disc. Legible at 82 px, and no restart between the frames.

### 61.9 Limits

* **Only the first 512 bytes are digested.** A rewrite that keeps the length and
  the mtime and changes only bytes past that window is still invisible. Closing
  that means hashing the whole file per check, which is the cost this budget
  exists to avoid.
* **A rewrite whose mtime moves is caught earlier, by §60's stat pass**, not by
  the digest — that is the intended order and why the digest re-baselines
  instead of double-reporting.
* **The digest cycle is the slowest of the three signals.** At 512 names a full
  pass takes 64 checks (~64 s), because it is deliberately the smallest budget.
* **Still a poll.** No inotify, the one-second floor from §55,
  `#![forbid(unsafe_code)]`, no new dependency.
* **A rename in the same directory** is still the gap §59 left: `refresh` drops
  the name on its next validation when `Path::is_file` fails.
* **`Inherits=` still clears the whole name map** on an order change (§58).
* **The stamp is per name, not per file**: two names resolving to one file are
  digested twice.

### 61.10 Git

Commit `HEAD` and `origin/main` are the same commit; `git status
--porcelain --untracked-files=no` is empty.

## 62. Batch: a rewrite past the head window is caught by a bounded whole-file digest (verified)

### 62.1 Requirement

§61 closed with this blind spot, in its own words:

> **Only the first 512 bytes are digested.** A rewrite that keeps the length and
> the mtime and changes only bytes past that window is still invisible. Closing
> that means hashing the whole file per check, which is the cost this budget
> exists to avoid.

This batch closes it for the files where hashing the whole file is affordable,
bounds that cost with its own budget and ceiling, and proves the tile repaints.

### 62.2 Audit, before the change

Reproduced on this machine before any code was touched, with the fixture the new
journey uses:

| Property | Value |
| --- | --- |
| document length, before and after | **729 B → 729 B** (identical) |
| first differing byte | **668** — past the 512-byte head window |
| file mtime, before and after | identical (`touch -r` restored it, as `cp -p` does) |
| directory mtime and entry count | unchanged |
| head digest of the first 512 bytes | **identical** — the payload that differs is entirely outside it |
| what the shell reported before this batch | nothing: `scan_files()` false, `scan_digests()` false, no repaint |

Audited before designing:

* `digests_of` did not exist; `head_digest` opened the file, read exactly
  `DIGEST_BYTES` into a stack buffer and stopped, so no code path ever saw byte
  512 of an icon.
* `FileStamp` was `len`, `mtime` and `head`; `scan_files` compares the first two,
  `scan_digests` the third, both round-robin over `IconNames::slots`.
* Measured icon sizes on this host: 12 files under `/usr/share/icons` and
  `/usr/share/pixmaps`, **all 12 at or under 64 KiB** (median 4 642 B, largest
  18 168 B). So the ceiling chosen below covers the entire icon footprint of this
  machine, which is what makes the exact path useful rather than theoretical.

### 62.3 What the whole-file digest stands on

The same FNV-1a, the same stack buffer, the same single open. `digests_of` reads
in 4 KiB chunks and computes two hashes in that one pass:

* the **head** digest over bytes `0..512`, exactly as §61 defined it, so the head
  pass keeps its meaning and its cost;
* the **whole-file** digest over every byte, but only when the length is at or
  under `DIGEST_FULL_MAX = 64 KiB`. Above the ceiling the read stops at the head
  window and `FileStamp::full` stays `None` — the honest "not covered" state.

Two rules keep it from double-reporting, both inherited from §61's shape:

* `full: None` is **recorded, never reported**. A name whose whole-file digest has
  not been taken yet cannot invent a change, and `stamped_file` records the
  baseline at resolve time, not on the first check.
* `scan_files` (length, mtime) runs first; when it sees a move it sets **both**
  digests to `None`, which hands the bytes to the digest passes and guarantees one
  change is reported once.

`scan_full_digests` walks `slots` with its **own cursor** and takes
`FULL_DIGESTS_PER_SCAN = 4` per check — smaller than the head pass's 8, because
each of these reads a whole file where a head digest reads 512 bytes. The shell
treats its report exactly like the other three signals: drop the decoded copies,
repaint, resolve nothing.

### 62.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/desktop_entry.rs` | 369 | `DIGEST_FULL_MAX = 64 KiB` — the size at or under which the whole file is digested |
| `src/litric-shell/src/desktop_entry.rs` | 376 | `FULL_DIGESTS_PER_SCAN = 4` — its own, smaller budget |
| `src/litric-shell/src/desktop_entry.rs` | 450 | `FileStamp::full` — `None` means "not covered", never "changed" |
| `src/litric-shell/src/desktop_entry.rs` | 462 | one open, one bounded read, head and whole-file digests together |
| `src/litric-shell/src/desktop_entry.rs` | 513 | `DIGEST_READ = 4096` — the stack buffer, no allocation |
| `src/litric-shell/src/desktop_entry.rs` | 520 | `head_digest` — the head-only view, reused through `digests_of` |
| `src/litric-shell/src/desktop_entry.rs` | 526 | `file_stamp` — one `metadata`, both digests `None` |
| `src/litric-shell/src/desktop_entry.rs` | 540 | `stamped_file` — the baseline is taken when the name answers |
| `src/litric-shell/src/desktop_entry.rs` | 1029 | `scan_full_digests` — bounded round-robin whole-file pass |
| `src/litric-shell/src/desktop_entry.rs` | 1086 | `full_digests()` — how much the pass actually did |
| `src/litric-shell/src/desktop_entry.rs` | 1092 | `fulls_changed()` — how many checks it reported a covered rewrite |
| `src/litric-shell/src/shell.rs` | 7324 | the whole-file pass, once per check |
| `src/litric-shell/src/shell.rs` | 7364 | journal line carries the head **and** whole-file counts |
| `src/litric-shell/src/e2e.rs` | 5998 | the e2e journey |
| `src/litric-shell/src/e2e.rs` | 5983 | the two documents that differ only past byte 512 |
| `src/litric-shell/src/desktop_entry.rs` | 3049 | unit: tail-only rewrite caught, stat and head blind |
| `src/litric-shell/src/desktop_entry.rs` | 3154 | unit: over the ceiling stays head-only |
| `src/litric-shell/src/desktop_entry.rs` | 3227 | unit: the budget bites, and the cost is measured |
| `ci/lint-file-safety.py` | 125 | the new journey's fixture cleanup, registered with its reason |

### 62.5 Tests that fail without it

| Test | Asserts |
| --- | --- |
| `desktop_entry::tests::a_tail_only_rewrite_is_noticed_by_the_full_digest` | same length, mtime restored, first difference at byte 668 and the first 512 bytes equal; `scan_files()` **false**, `scan_digests()` **false**, `scan_full_digests()` **true**, `fulls_changed() == 1`, then false again |
| `desktop_entry::tests::a_file_over_the_ceiling_stays_on_the_head_digest` | a 65 537-byte file: `head` is present, `full` is **absent**, and the whole-file budget is never spent on it |
| `desktop_entry::tests::a_full_digest_pass_is_bounded_and_measured` | 24 names: one check takes exactly 4 whole-file digests, and both the pass and a 64 KiB read are timed |
| `e2e::journey_launcher_picks_up_a_tail_only_rewrite` | real temp XDG tree, launcher running: `len_before == len_after`, `mtime_before == mtime_after`, `digests_changed() == 0`, `files_changed() == 0`, `fulls_changed() > 0`, `icons_resolved == 0`, and the tile's pixels become the new artwork |

The §53/§54, §57/§58, §59, §60 and §61 tests pass unchanged.

### 62.6 Sensitivity

With the whole-file comparison forced to report "unchanged"
(`if false && fresh != file.full`, the §61 behaviour), the new journey fails on
its own assertion and says which path was blind:

```
thread 'e2e::journey_launcher_picks_up_a_tail_only_rewrite' (80180) panicked at litric-shell/src/e2e.rs:6098:5:
a rewrite past the head window was not noticed in 8 watched checks (8 whole-file digests over 1 slots, first difference at byte 668)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 724 filtered out
```

Restored, the journey passes and the journal prints
`icons true after 5 leaf scans of 5, 1 file re-stats of 1 and 1 head digests and 1 full digests`
— the whole-file pass is what saw it.

### 62.7 Cost, measured on this machine

| Figure | Value | How |
| --- | --- | --- |
| one whole-file pass, 4 exact reads of 729 B | **95.3 µs per check** | printed by `a_full_digest_pass_is_bounded_and_measured` (debug test build, 200 passes averaged) |
| one whole-file read at the 64 KiB ceiling | **930.7 µs** | same test, 50 reads averaged — the worst single read the exact path can take, in a debug build |
| one head pass, 8 × 512 B reads | 74.3 µs | §61, same test shape |
| one stat pass, 32 `metadata` calls | 94.4 µs | §61, same run |
| a whole watched check now (stat + head + full) | **≈ 264 µs** | the three rows above, at those budgets |
| memory | `FileStamp` **40 B**, up from 32 B → **+8 B per name**; 24 names hold **7 689 B**; `Cached` 104 B | `std::mem::size_of`, `IconNames::bytes()` |
| work per frame | **none** | reached only from `refresh_desktop`, behind `DesktopWatch::due`; §55's "30 frames, no stat" and §56's "poll only at period" tests still pass |
| ceilings | 64 KiB per file, 4 whole-file reads per check, 512 slots → a full exact cycle in 128 checks | `DIGEST_FULL_MAX`, `FULL_DIGESTS_PER_SCAN`, `MAX_ICON_NAMES` |

The ceiling is 64 KiB because every icon on this machine is under it (largest
18 168 B, median 4 642 B), so the exact path covers 100 percent of the real icon
set here rather than a fraction of it. A multi-hundred-kilobyte raster would cost
about 1 ms per read in a debug build, which is why it stays out.

### 62.8 Gates

```
$ cargo fmt --all --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.40s
CLIPPY_EXIT=0

$ cargo test --workspace
run 1: 1814 passed, 0 failed
run 2: 1814 passed, 0 failed
run 3: 1814 passed, 0 failed

$ python3 ci/lint-doc-refs.py
OK: 1092 citation(s) across 155 document(s) resolve

$ python3 ci/lint-file-safety.py
FILESAFETY_EXIT=0

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the pre-existing `keys/dev/` key material and the mkosi
image check; the skips are QEMU-only checks. No gate, golden frame or key was
added, weakened or generated. Four tests were added this turn (1810 → 1814), and
the §61 and §62 journeys were each run 5 times plus three full suites without a
failure, because §61 had hit two intermittent failures from mtime-bucket
assumptions.

The two frames the journey saved were opened:

* `artifacts/browser-shots/62-before-the-tail-only-rewrite.png` — the `Tail App`
  tile on a blue field with an amber disc.
* `artifacts/browser-shots/62-launcher-tail-only-rewrite.png` — the same tile
  after a rewrite of bytes past offset 512 with the length **and** mtime
  unchanged, now a red field with a white disc, legible at 82 px, no restart.

### 62.9 Limits

* **Files above 64 KiB still have only the head digest.** A rewrite past byte 512
  of a larger icon with the length and mtime preserved stays invisible; the
  journal makes this visible as a whole-file count that does not grow with the
  name count.
* **The digest is FNV-1a, 32-bit, not cryptographic.** It detects rewrites and
  accidental collisions; a deliberate collision is not modelled.
* **The pass re-reads the same small files every cycle** even when nothing
  changed — bounded to 4 reads per check, so a full cycle at 512 names takes
  about 128 s, but it is repeated I/O rather than a change-triggered read.
* **Still a poll.** No inotify, the one-second floor from §55,
  `#![forbid(unsafe_code)]`, no new dependency.
* **A rename in the same directory** is still the gap §59 left: `refresh` drops
  the name on its next validation when `Path::is_file` fails.
* **`Inherits=` still clears the whole name map** on an order change (§58).
* **The stamp is per name, not per file**: two names resolving to one file are
  digested twice.
* **The measured microseconds are a debug build.** `cargo test` is unoptimized,
  so the 930.7 µs ceiling read is an upper bound, not the shipped cost.

### 62.10 Git

Commit `HEAD` and `origin/main` are the same commit; `git status
--porcelain --untracked-files=no` is empty.

## 63. Batch: capability grants survive a restart, and a corrupt record denies (verified)

### 63.1 Requirement

The gate matrix has carried this limitation since the broker was written, in its
own words:

> grants are in-memory and do not survive reboot (ADR-0016)

The consequence is a permission system that forgets: the user answers a prompt,
the daemon restarts, and every application is denied until it asks again — while
the audit that explains what happened is gone with it. ADR-0016 also places the
broker inside `litric-sysd` and says persistence belongs to the daemon, which is
why the file lives there and `litric-caps` stays I/O-free.

### 63.2 Audit, before the change

Read on this tree before any edit:

* `Broker` held `grants: HashMap<AppId, Vec<Grant>>` and `audit: Vec<AuditEntry>`
  and nothing else; `Sysd::new` built it, called `grant_local_session`, and the
  process owned it until exit.
* The only `grant` call sites in the daemon were inside that constructor. There
  was no wire verb that grants — deliberately: a client that could ask for a
  capability would be the confused deputy the layer exists to prevent.
* Nothing under `src/litric-sysd` read or wrote `/var/lib`; the daemon had no
  state path, no `--state` flag and no start line reporting one.
* A denial was visible only in `CAPS AUDIT`, from memory. `CAPS AUDIT` after a
  restart returned `count=0`.

So the measured “before” is exact: **restart → 0 active grants for the local
client, 0 audit records**, whatever the user had answered.

### 63.3 What the record stands on

The format is line-oriented text with a protocol line, tabs between fields, and
three properties that make it safe to trust:

1. **Corruption is a denial.** A missing protocol line, a newer version, a bad
   app id, an unparseable capability or scope, an unknown issuer or reason, a
   line past 4 KiB, or a file past 256 KiB rejects the **whole** record. The
   daemon then starts with an empty broker — the policy grants included — and
   says `store=corrupt:<why>` on its start line and in `CAPS LIST`. Partial
   acceptance would restore a subset nobody audited.
2. **A write is atomic.** Temporary file in the same directory, `fsync`,
   `rename`, `fsync` the directory — the shape `prefstore` proved. A power cut
   leaves the previous record, never a prefix of the new one.
3. **It is bounded.** `Broker::MAX_GRANTS = 1024` grant records and
   `MAX_AUDIT_RECORDS = 512` decision records, both enforced on write *and* on
   read, against a 256 KiB read ceiling. The audit ring in memory stays at its
   own 4,096.

Two semantics are stated rather than implied:

* **A corrupt file is never overwritten.** A grant attempted while the record is
  unreadable fails with “refusing to overwrite it”, because the damaged file is
  the evidence of what happened.
* **This is a checkpoint, not a write-ahead log.** Decisions are written at
  startup, on every grant or revocation, and when `CAPS AUDIT` is read (that is
  the moment the log has to be worth reading). Decisions made since the last
  checkpoint are lost if the process is `SIGKILL`ed. §63.9 keeps that in the
  limitation list instead of hiding it.

### 63.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-caps/src/lib.rs` | 80 | `Issuer::parse` — a token this build cannot read restores nothing, never `Policy` |
| `src/litric-caps/src/lib.rs` | 168 | `Grant::restore` — the one constructor a store may use, revoked-at included |
| `src/litric-caps/src/lib.rs` | 311 | `Broker::MAX_GRANTS = 1024` — the ceiling both the broker and the file use |
| `src/litric-caps/src/lib.rs` | 348 | `grant_count` — what `CAPS LIST` reports |
| `src/litric-caps/src/lib.rs` | 356 | `all_grants` — the snapshot the store writes |
| `src/litric-caps/src/lib.rs` | 367 | `restore_grants` — bounded restore, returns how many were accepted |
| `src/litric-caps/src/lib.rs` | 381 | `restore_audit` — the audit ring comes back through its existing cap |
| `src/litric-caps/src/lib.rs` | 390 | `record_event` — a revocation is recorded as the decision it creates |
| `src/litric-sysd/src/grants.rs` | 65 | `MAX_FILE_BYTES = 256 KiB` — past this the file is refused before it is buffered |
| `src/litric-sysd/src/grants.rs` | 72 | `MAX_AUDIT_RECORDS = 512` — the newest decisions, measured at 45.6 B/line |
| `src/litric-sysd/src/grants.rs` | 111 | `Store::load` — metadata first, then one bounded read |
| `src/litric-sysd/src/grants.rs` | 141 | `Store::save` — temp file, fsync, rename, fsync the directory |
| `src/litric-sysd/src/grants.rs` | 193 | `encode` — sorted grants, capped audit, tabs as separators |
| `src/litric-sysd/src/grants.rs` | 285 | `decode` — one bad line rejects the whole record |
| `src/litric-sysd/src/grants.rs` | 382 | `parse_app` — accepts the reserved system id the shell’s policy grants live under |
| `src/litric-sysd/src/grants.rs` | 439 | `parse_capability` — the canonical `id@scope` form back to a capability |
| `src/litric-sysd/src/grants.rs` | 455 | `parse_scope` — URI, resource and host, tried in that order so they stay distinct |
| `src/litric-sysd/src/state.rs` | 169 | `with_grant_store` — loaded, seeded, or corrupt-and-empty; no fourth state |
| `src/litric-sysd/src/state.rs` | 223 | `persist_grants` — refuses to overwrite a record it could not read |
| `src/litric-sysd/src/state.rs` | 253 | `grant_and_save` — a grant is not “issued” until it is on disk |
| `src/litric-sysd/src/state.rs` | 264 | `revoke_and_save` — revocation, its audit entry, and the write |
| `src/litric-sysd/src/state.rs` | 214 | `save_grants` — the checkpoint the daemon calls when the audit is read |
| `src/litric-sysd/src/state.rs` | 376 | reading the audit checkpoints it, so the log is worth reading after a crash |
| `src/litric-sysd/src/state.rs` | 747 | `CAPS LIST` — the store note, record counts, and the client’s active capabilities |
| `src/litric-sysd/src/proto.rs` | 85 | `CAPS LIST` on the wire: a question, no capability needed |
| `src/litric-sysd/src/proto.rs` | 341 | its parser arm |
| `src/litric-sysd/src/main.rs` | 52 | `/var/lib/litric-sysd/grants.conf` — under `/var`, so it survives a reboot |
| `src/litric-sysd/src/main.rs` | 170 | the serve mode attaches the store it was given |
| `src/litric-sysd/src/main.rs` | 181 | the start line carries `store=` and `records=`, so a boot log shows the state |
| `src/litric-sysd/src/lib.rs` | 39 | the module is part of the daemon’s library, not a hidden binary detail |
| `src/litric-sysd/src/grants.rs` | 574 | unit: eight damaged records are refused, none half-restored |
| `src/litric-sysd/src/grants.rs` | 629 | unit: full-size record measured — bytes, save µs, load µs |
| `src/litric-sysd/src/state.rs` | 844 | unit: grant and denial reloaded from the same file |
| `src/litric-sysd/src/state.rs` | 888 | unit: corruption empties the broker, policy grants included |
| `src/litric-sysd/src/state.rs` | 940 | unit: revoked-at and its audit entry survive |
| `src/litric-sysd/tests/grants_restart.rs` | 98 | process test: two real daemons, one real socket, a damaged record |
| `ci/lint-file-safety.py` | 160 | the store’s own temporary-file deletion, registered with its reason |
| `ci/lint-file-safety.py` | 169 | the process test’s fixture and socket cleanup, registered with its reason |

### 63.5 Tests, at two levels

Unit tests in `src/litric-sysd/src/grants.rs` and `state.rs`:

* `a_record_round_trips_through_the_file` — grants, an expiring grant, a revoked
  grant and an audit entry survive exactly;
* `a_header_only_record_is_an_empty_broker_not_corruption` — “the user revoked
  everything” is a valid state, not damage;
* `garbage_grants_nothing` — eight shapes of damage, every one refused;
* `a_newer_version_is_refused_instead_of_guessed`;
* `an_oversized_file_is_refused_before_it_is_read`;
* `the_audit_is_written_at_its_cap_not_beyond` and
  `the_grant_table_is_written_at_the_broker_cap` — the ceilings bite, and the
  newest audit records are the ones kept;
* `every_scope_kind_round_trips` and
  `a_denial_reason_survives_with_the_field_it_needs` — the encoders are not
  lossy for the cases the broker can produce;
* `a_grant_and_a_denial_survive_a_restart`,
  `a_revocation_survives_a_restart_and_is_audited`,
  `an_expiring_grant_still_expires_after_a_restart`,
  `a_corrupt_record_grants_nothing_including_the_policy_set`,
  `a_corrupt_record_is_not_overwritten_by_a_new_grant`,
  `caps_list_reports_the_store_and_the_grants`.

The process test is `src/litric-sysd/tests/grants_restart.rs`:
`a_corrupt_record_makes_a_restarted_daemon_grant_nothing` starts the real
`litric-sysd --serve` twice over a real Unix socket, asserts `store=seeded` and
a policy grant on the first run, damages the record with a truncated grant line,
and asserts the second run answers `store=corrupt:…`, `grant_records=0`, and no
`capability=` line at all. It is a restart of the binary, not a re-construction
of a struct.

### 63.6 Sensitivity proof

The property that matters is “corrupt means none”. Forcing the old behaviour —
keeping the policy grants when the record cannot be read — makes the test fail
with the real panic:

```text
thread 'state::tests::a_corrupt_record_grants_nothing_including_the_policy_set' panicked at litric-sysd/src/state.rs:904:9:
not even the policy grants may survive a corrupt record
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 201 filtered out
```

Restored to deny-all, the same test passes and the process test agrees over the
socket.

### 63.7 Measured costs

Measured by `the_record_costs_what_the_budget_says`, which writes the worst case
the format allows and reads it back with the production code path (debug build,
one run, this host):

| Figure | Value |
| --- | ---: |
| record, 1,024 grants + 512 audit records | **69,964 bytes** (68.3 KiB) |
| save (create, write, fsync, rename, dir fsync) | **23,859 µs** |
| load (metadata, read, parse, validate) | **4,792 µs** |
| bytes per record line | ≈ 45.6 B |
| read ceiling | 256 KiB — 3.7× the worst case above |
| per-frame work added | **0** (the store is touched at startup, grant, revoke and audit read only) |
| bytes persisted before this batch | 0 |

The seeded record a normal boot writes is seven policy grants and no audit: a
few hundred bytes. The 1,024-grant case is a ceiling, not an expectation. The
23.9 ms save cost is `fsync` on this host’s filesystem, paid once per grant
change — not on every decision, which is the reason the checkpoint design was
chosen over writing on each `check`.

### 63.8 Gate

```text
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
1,833 passed; 0 failed   (was 1,814 before this batch; +19)

$ python3 ci/lint-doc-refs.py
OK: 1,182 citation(s) across 156 document(s) resolve

$ python3 ci/lint-file-safety.py
OK: 110 deletion site(s), all inside reviewed functions

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures and three skips are the recorded baseline: `keys/dev/` is absent
on this host and mkosi/QEMU are not installed. No gate, golden frame or key file
was edited.

### 63.9 Limits, stated

* **It is a checkpoint, not a WAL.** A `SIGKILL` between checkpoints loses the
  decisions made since the last one. The grants themselves are written on the
  change that creates them, so a permission cannot silently disappear; the
  audit of the last few checks can.
* **The file is protected by ownership, not cryptography.** Mode 0600 in a 0700
  directory under a root-owned path. A host attacker who can write it can grant
  whatever they like, exactly as they could edit `/etc/sudoers`; tamper-evident
  records are a separate decision.
* **There is still no wire verb that grants.** Persistence is for grants issued
  by policy or by a future prompt path, not a way for a client to ask for one.
  `CAPS LIST` is read-only introspection.
* **Nothing has run inside a booted image.** The store path, the unit and the
  mount are host-level facts; the image row stays evidence-level `UNIT` +
  `INTEGRATION`-on-host until a builder boots it (§A1 of the plan).
* **The parked `litric-capd` units remain parked.** ADR-0016 puts the broker
  inside `litric-sysd`; this batch follows that rather than reviving the units.

### 63.10 Git

Base `4c1b7d3`. The batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

## 64. Batch: the Defender crate builds the binary its unit names (verified)

### 64.1 Requirement

The gate matrix carried this claim, written before the crate had a binary:

> no daemon, no on-access hook; the `litric-defender.service` unit points at a
> binary that does not exist

Two things were wrong in one sentence. The unit could not be installed (a
`ExecStart` the image cannot satisfy is the defect `ci/lint-unit-binaries.py`
exists to catch), and the only code that scanned files ran inside the desktop
process — the one place ADR-0022 §2 says untrusted content must not be parsed,
because a parser bug there is a bug in the process that draws the screen.

This batch ships the process ADR-0022 already names: the on-demand scanner
worker. It does **not** ship the on-access hook, and §64.9 says so.

### 64.2 Audit, before the change

* `src/litric-defender` was a library only: `lib.rs`, `engine.rs`, `verdict.rs`,
  `patterns.rs`, `denylist.rs`, `cache.rs`, `sha256.rs` — no `main.rs`, no
  `[[bin]]` in `Cargo.toml`.
* `tools/build.sh` already had `litric-defender` in its staging loop (line
  205), so a binary would have been staged and installed the
  moment one existed; the loop was waiting, not broken.
* `image/units-pending/litric-defender.service` asks for
  `--rules /usr/lib/litric/yara --cache /var/cache/litric/defender`,
  `Type=notify` and `CAP_SYS_ADMIN` — the fanotify transport (ADR-0022 Phase 2),
  none of which exists.
* The Defender application scanned in-process through
  `litric_defender::engine::Engine` (`src/litric-shell/src/shell.rs`), reading a
  file's bytes in the desktop's address space.

So the measured “before” is exact: `src/target/release/litric-defender` did not
exist, and the only scan path was the shell's.

### 64.3 What the worker stands on

* **One bounded read.** `scan_path` stats the file, refuses anything that is not
  a regular file, and reads through `File::take(MAX_SCAN_BYTES + 1)`. A 4 GiB
  file costs 8 MiB and one byte. The `+1` is what catches a file that grew past
  the ceiling between `stat` and `read` instead of silently truncating it.
* **A ceiling that answers honestly.** Past 8 MiB the verdict is
  `unknown reason=too-large` — `AllowUnknown` in permissive mode and a denial in
  `--enforce`, exactly the ladder `verdict::resolve` already defines. A partial
  scan that reported `clean` would be worse than no scan.
* **A framed worker, not an ad-hoc CLI.** `--stdio` speaks
  `PING` / `SCAN <hex-path>` / `QUIT` with `OK <n>` or `ERR <kind>` replies. Hex
  because a desktop file name may contain spaces and the protocol splits on
  whitespace; the same choice the browser broker made for staged downloads.
* **Exit codes a parent can branch on.** `--scan` exits 1 on a detection, 0 on
  allow/unknown, 2 on usage or I/O error.
* **No new dependency, no `unsafe`.** The feed is `Denylist::parse`, the scan is
  the existing engine, the digest is the crate's own SHA-256.

### 64.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-defender/src/lib.rs` | 30 | `MAX_SCAN_BYTES = 8 MiB` — the one ceiling the binary and its test share |
| `src/litric-defender/src/main.rs` | 71 | argument parsing, mode selection, exit codes |
| `src/litric-defender/src/main.rs` | 129 | loads the optional SHA-256 feed; a feed with no usable line is an error, not an empty list |
| `src/litric-defender/src/main.rs` | 175 | `PING` / `SCAN <hex-path>` / `QUIT`: a framed worker a parent can drive |
| `src/litric-defender/src/main.rs` | 246 | metadata first, `take(MAX+1)`, one bounded read |
| `src/litric-defender/src/main.rs` | 275 | past the ceiling the verdict is `unknown reason=too-large`, never a partial `clean` |
| `src/litric-defender/src/main.rs` | 290 | one `key=value` verdict line, whitespace sanitised so one field stays one field |
| `src/litric-defender/src/main.rs` | 332 | hex path decoding — no escape grammar to disagree about |
| `src/litric-defender/tests/scanner.rs` | 118 | process test: real binary, real stdio, deny / allow / too-large / missing |
| `image/units-pending/README.md` | 20 | the parked unit row now says what exists and what still does not |
| `docs/master-plan/25-beta-1.0-gate-matrix.md` | 66 | the gate-matrix claim corrected in the same commit |
| `ci/lint-file-safety.py` | 169 | the test fixture cleanup, registered with its reason |
| `tools/build.sh` | 192 | `build.sh` already staged this binary name; the loop needed no edit |

### 64.5 Tests, at two levels

* Unit tests in the crate (35, unchanged) still cover the engine, the policy
  ladder, the cache, the pattern matcher and SHA-256.
* `src/litric-defender/tests/scanner.rs` spawns the real binary and drives it
  over real stdio: EICAR is **denied by hash** with the feed's label, an ordinary
  file is **allowed** with a non-empty digest, a sparse file one byte past 8 MiB
  is **unknown `reason=too-large`** without being read, a missing path is `ERR
  cannot-stat`, `QUIT` ends the worker, and the one-shot form returns exit 1 for
  the detection and 0 for the clean file.

### 64.6 Sensitivity proof

Forcing the worker to scan no bytes (`engine.scan(..., b"")` instead of the
buffer) makes the process test fail with the real panic:

```text
thread 'the_scanner_denies_eicar_allows_clean_and_refuses_to_read_past_the_ceiling' panicked at litric-defender/tests/scanner.rs:134:5:
EICAR must be denied by hash: verdict=allow tier=- id=- reason=- size=68 digest=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 from_cache=0
test result: FAILED. 0 passed; 1 failed
```

The digest in that message is SHA-256 of the empty string, which is exactly the
bug the probe introduced. Restored, the test passes.

### 64.7 Measured costs

Measured on this host with the debug binary, five runs each, wall clock for one
`--scan` (process start included; `n=5`):

| Input | Median | Mean |
| --- | ---: | ---: |
| 1 MiB | **138.9 ms** | 138.4 ms |
| 8 MiB (the ceiling) | **1075.0 ms** | 1079.8 ms |

| Figure | Value |
| --- | ---: |
| resident buffer, worst case | 8 MiB + 1 byte |
| per-frame work added | **0** (the worker runs only when asked) |
| files read whole by the desktop for a scan | 0 in this path — the worker is the reader |
| new dependencies | 0 |
| binary size / RAM at idle | 0 when not running; it is not a service |

The 1 MiB number is dominated by hashing and the pattern pass, not by I/O: 8 MiB
takes 7.7× as long for 8× the bytes, which is the linear shape a byte scan should
have. No claim is made for a release build — this is the debug number, and the
tracker labels it as such.

### 64.8 Gate

```text
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
1,834 passed; 0 failed   (was 1,833 before this batch; +1 process test)

$ python3 ci/lint-doc-refs.py
OK: 1,201 citation(s) across 156 document(s) resolve

$ python3 ci/lint-file-safety.py
OK: 112 deletion site(s), all inside reviewed functions

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures and three skips are the recorded baseline: `keys/dev/` is absent
and mkosi/QEMU are not installed. No gate, golden frame or key file was edited.

### 64.9 Limits, stated

* **No on-access hook.** The unit's `Type=notify`, `--rules`/`--cache` and
  `CAP_SYS_ADMIN` belong to the fanotify transport (ADR-0022 Phase 2), which does
  not exist. The unit stays in `image/units-pending/`, and the README row now
  says exactly why.
* **No YARA rules.** The worker loads the SHA-256 feed only; the pattern matcher
  is available in the library and is what the engine's unit tests exercise, but
  this binary has no `--rules` flag.
* **The desktop still scans in-process.** Moving `litric-shell` onto this worker
  is a separate change with its own protocol and failure policy; this batch makes
  the worker exist and proves it, it does not rewire the application.
* **No `sd_notify`, no watchdog.** Those belong with the daemon, not the worker.
* **8 MiB is a working ceiling, not a measurement of malware.** Files past it are
  `unknown` by design; the number is stated so the worst case is a figure rather
  than “whatever the user opened”.
* **Nothing in a booted image.** The binary is staged by `tools/build.sh`, but no
  image was built here (mkosi is not installed), so the row stays host-level
  `UNIT` + process evidence.

### 64.10 Git

Base `51cc57e`. The batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

## 65. Batch: a sentinel password never reaches a log rendering (verified)

### 65.1 Requirement

The improvement plan names this batch and what proves it
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 65):

> Provider contract layer with a fake; sentinel password never logged |
> contract tests, redaction test

The execution plan says why it comes before the integration
(`docs/master-plan/26-execution-plan.md`, W7):

> Writing provider code before it is [designed] is how an unredacted passphrase
> ends up in a log.

and the gate matrix asks the same of the network slice
(`docs/master-plan/25-beta-1.0-gate-matrix.md`):

> credential redaction proven with a sentinel password

### 65.2 Audit, before the change

Four types in `src/litric-sysd/src/providers.rs` carried a **derived** `Debug`:

| Type | The field the derive printed |
| --- | --- |
| `NetAction::JoinWifi` | `passphrase` — the Wi-Fi passphrase, in cleartext |
| `AuthAction::Verify` | `password` — the account password, in cleartext |
| `FileChange::Write` | `contents` — which for a join is `[Security]\nPassphrase=…` |
| `Plan` | `stdin` — the account password, and the same file list |

`Request` derives `Debug` too and holds a `SystemAction`, so one `{:?}` of any
request printed all of it. That is one format string away from a journal line, a
panic message, or a bug report somebody attaches.

Measured on this host before the fix — `grep -rn '{:?}' src/litric-sysd/src/*.rs`
matched only assertion messages in `#[cfg(test)]` code, so **no shipping line
rendered an action today**. The defect was latent, not active, and this entry
does not claim a leak that happened. What was true is that the type system
offered the secret to every future log line, and nothing said no.

The reply path was already secret-free and stays so: `describe` builds
`command=` from `Plan::command_line`, the passphrase goes to a credential file,
the password to standard input, and `proto.rs` refuses a field that is not hex
with `<not hex>` instead of echoing it.

### 65.3 What changed

* A hand-written `Debug` for `NetAction`, `AuthAction`, `FileChange` and `Plan`.
  The secret **key** stays in the rendering and the **value** becomes
  `<redacted>`, so a reader can see that a secret existed and was removed rather
  than that the field was forgotten.
* `FileChange::Write` redacts only when the plan itself marked the file
  `private`. A networkd drop-in is not a secret, and hiding it would make the
  rendering less useful than the write it describes.
* `Plan` renders `stdin` as present/absent, never as content.
* No dependency, no `unsafe`, no new field, no allocation at rest.

### 65.4 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-sysd/src/providers.rs` | 515 | `REDACTED` — one spelling, so a test can search for it |
| `src/litric-sysd/src/providers.rs` | 517 | `Debug for NetAction`: `ssid` visible, `passphrase` redacted |
| `src/litric-sysd/src/providers.rs` | 539 | `Debug for AuthAction`: `user` visible, `password` redacted |
| `src/litric-sysd/src/providers.rs` | 551 | `Debug for FileChange`: contents redacted only when `private` |
| `src/litric-sysd/src/providers.rs` | 578 | `Debug for Plan`: `stdin` reported as present/absent |
| `src/litric-sysd/src/providers.rs` | 657 | `command_line` — still the only builder of a reply's `command=` |
| `src/litric-sysd/src/providers.rs` | 1333 | unit test: sentinel in every rendering, credential still written |
| `src/litric-sysd/src/proto.rs` | 279 | a field that is not hex is refused, never echoed |
| `src/litric-sysd/src/proto.rs` | 780 | unit test: `Debug` of a parsed `Request` carries no secret |
| `src/litric-sysd/tests/wifi_secret.rs` | 149 | process test: real daemon, real socket, real credential file |
| `src/litric-sysd/tests/wifi_secret.rs` | 205 | the daemon journal is searched, in text and in hex |
| `src/litric-sysd/tests/wifi_secret.rs` | 217 | `CAPS AUDIT` over the same socket is searched the same way |
| `ci/lint-file-safety.py` | 169 | the fixture's three deletion sites, registered with their reasons |

### 65.5 Tests, at two levels

* **Contract, with the fake.** `RecordingRunner` (the fake provider) plus
  `a_secret_never_appears_in_a_debug_rendering`: a sentinel passphrase and a
  sentinel password are put through `NetAction`, `AuthAction`, `SystemAction`,
  `Plan`, `FileChange` and `command_line`, and every rendering is searched. The
  same test asserts the *opposite* direction — the credential file still
  contains the passphrase and is still marked `private` — because redaction is
  about renderings, and a fix that also stopped storing the secret would be a
  worse bug wearing the same name.
* **Process, over the socket.** `wifi_secret.rs` spawns the real
  `litric-sysd --serve`, sends `NET WIFI JOIN <hex ssid> <hex passphrase>`
  through the real client, and asserts: the reply holds neither the passphrase
  nor its hex; the credential file exists, contains the passphrase, and is mode
  `0600`; the daemon's journal holds neither encoding; and `CAPS AUDIT` over the
  same socket holds neither while still reporting `count=`.
* The host has no `iwctl`, so the action fails after the file is written. That is
  deliberate: the error path is where a secret usually escapes, and it is the
  path a fake would have skipped.

### 65.6 Sensitivity proof

Restoring the old behaviour (`field("passphrase", passphrase)` instead of the
redacted value) makes two tests fail with their real panic messages:

```text
thread 'providers::tests::a_secret_never_appears_in_a_debug_rendering' panicked at litric-sysd/src/providers.rs:1339:9:
network Debug leaked the passphrase: JoinWifi { ssid: "litric-test", passphrase: "S3NT1NEL-passphrase-not-in-logs" }
```

```text
thread 'proto::tests::a_parsed_request_never_renders_the_credential_it_carries' panicked at litric-sysd/src/proto.rs:787:9:
Debug leaked the passphrase: Action { action: Net(JoinWifi { ssid: "Home", passphrase: "correct horse" }), consent: false }
```

The same forced bug left the **process test green** (1 passed), because the reply
never renders a `Debug` — measured, not assumed. That is worth writing down: the
socket test proves the wire and the file, and cannot catch this class on its own.
The unit tests are the guard, and they are the ones that failed.

Restored, all of them pass.

### 65.7 Measured costs

| Figure | Value |
| --- | --- |
| work added per frame | **0** — the paint path is untouched |
| heap added at rest | **0 bytes** — one `&'static str`, no new field, no cache |
| reply bytes changed | **0** — `command=` was already built from `command_line` |
| one socket round trip, join (debug build, process start included) | **2,226 µs** |
| one socket round trip, `CAPS AUDIT` | **3,893 µs** |
| new dependencies | 0 |
| `unsafe` | none; `forbid(unsafe_code)` untouched |

The two numbers are wall clock for a whole `--query` process, printed by the test
itself (`one query took N us`). They measure the path the test drives, not the
redaction, which the shipping path never executes: a `Debug` that is not called
costs nothing, and that is the point of putting the fix there.

### 65.8 Gate

```text
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
1,837 passed; 0 failed   (was 1,834 before this batch; +3 tests)

$ python3 ci/lint-doc-refs.py
OK: 1,218 citation(s) across 156 document(s) resolve

$ python3 ci/lint-file-safety.py
OK: 116 deletion site(s), all inside reviewed functions

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the recorded baseline — `mkosi config uses only real
sections and key placements` and `dev key pairs exist, parse, and match` — and
the three skips are the image build, the boot smoke test and the idle
measurement, all of which need mkosi/QEMU. No gate, golden frame or key file was
edited.

### 65.9 Limits, stated

* **The secret is still stored in the clear.** `var/lib/iwd/<ssid>.psk` holds the
  passphrase in iwd's own format, mode `0600`, because that is the file iwd reads.
  This batch makes the secret harder to *print*, not harder to *read* by root or
  by a same-uid process.
* **The client still passes it in `argv`.** `litric-sysd --query "NET WIFI JOIN
  …"` puts the hex passphrase on the command line of a short-lived process, where
  `/proc` can see it. Moving the request to the socket the way the daemon's
  protocol already allows is a separate change.
* **Nothing zeroises memory.** The passphrase lives in `String`s until they are
  dropped; zeroing would need `unsafe` or a crate, both excluded here.
* **No real iwd/BlueZ integration.** The process test proves the failure path on a
  host without `iwctl`; a join that actually connects is `INTEGRATION` evidence
  and belongs to the builder, not to this entry.
* **Wave 3 is still `PARTIAL`.** The gate-matrix row now says what is proven
  (redaction with a sentinel over the real socket) and what is not (a real
  provider, and the secret-agent design the wave is gated on).
* **The journal check is a guard, not a demonstration.** This path prints nothing
  today, so the assertion passes on an empty file; its value is that the first
  future log line that renders an action fails the test.

### 65.10 Git

Base `369d78a`. The batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

## 66. Batch: the frame budget re-measured, and BUG-0015 priced rather than closed (verified)

### 66.1 Requirement

The improvement plan scopes this batch by its proof
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 66):

> Frame budget: named change, re-measured, BUG-0015 closed or costed |
> `render_bench` before/after, n=15

and `docs/bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md` carries the claim it
has to answer:

> `render_bench`, medians over 15 runs, release, 1280x800: Full frame, two windows
> open (`full_frame`) | **206.359 ms** | 196.926 ms

The batch contract's rule 5 applies to the result
(`docs/master-plan/28-whole-system-improvement-plan.md`):

> **Measurement before optimisation.** No RAM, wakeup or frame claim without a
> tool, a command, an environment and n. A saving must name the capability it
> removed.

### 66.2 Measured baseline, before any edit

Command, run on this commit (`ce08ee8`) before touching a line of code:

```bash
cd src
cargo build --release -p litric-shell --example render_bench
./target/release/examples/render_bench        # RUNS = 15 per scene
```

| Scene | p50 | min | mean | sd |
|---|---:|---:|---:|---:|
| `full_frame` (two windows) | **6.832 ms** | 6.696 | 6.853 | 0.119 |
| `pointer_move_frame` | 7.034 ms | — | — | — |
| `open_window_frame` | 11.980 ms | — | — | — |
| `lock_frame` | 4.423 ms | — | — | — |
| `full_frame_live_blur` (fallback) | 29.936 ms | — | — | — |
| `present_full` | 3.943 ms | — | — | — |
| `present_pointer_damage` | 0.151 ms | — | — | — |

The recorded 206.359 ms and 185.3 ms figures **do not reproduce** on this commit
and host. That is the finding the batch had to start with, and it changes what
"fix BUG-0015" can honestly mean here.

The profile pass attributes the 15.182 ms it can time: `stroke_line` 4.234 ms
(27.9 %, 2,325 calls), `soft_shadow` 3.993 ms (26.3 %, 3 calls),
`stroke_line_attrib` 1.797 ms, `stroke_round_rect` 1.686 ms, `fill_round_rect`
1.227 ms (358,198 px at 3.42 ns/px), `glass_backdrop` 0.534 ms.

### 66.3 The named change: implemented, proved pixel-identical, measured, reverted

The candidate the profile pointed at was the per-row walk in `stroke_line`: a
capsule paints only where its distance is under `half + 0.5`, but the loop walked
its whole inflated box. The clamp was written in the real path
(`src/litric-shell/src/canvas.rs`, `stroke_line`) with the inflated box kept as
the fallback for non-finite input.

Correctness was not assumed. `the_stroke_line_equals_the_per_pixel_distance`
compares the painter against `stroke_line_reference`, the pre-optimisation
implementation verbatim, over 144 combinations — horizontal, vertical, diagonal,
fractional, zero-length, off-canvas, hairline, thick, NaN start, four clips
including an empty one, and opaque / translucent / zero-alpha colours. It passed,
as did all 57 canvas tests, so the change painted the same pixels.

It was then measured with the identical command and n=15:

| Scene | before | after | delta |
|---|---:|---:|---:|
| `full_frame` | 6.832 ms | 6.817 ms | -0.2 % |
| `pointer_move_frame` | 7.034 ms | 7.156 ms | **+1.7 %** |
| `open_window_frame` | 11.980 ms | 12.081 ms | **+0.8 %** |
| `lock_frame` | 4.423 ms | 4.557 ms | **+3.0 %** |
| `full_frame_live_blur` | 29.936 ms | 28.504 ms | -4.8 % |
| `profile.stroke_line.ms` | 4.234 ms | 4.491 ms | +6.1 % |

The baseline's own second run moved between 3 % and 8 % on these rows
(`full_frame` 6.614 ms, `pointer_move_frame` 6.457 ms, `open_window_frame`
11.669 ms, `lock_frame` 4.326 ms), so single-run deltas of 1-3 % cannot be read as
a result on their own. What *is* consistent across the two runs of each side:
the pointer, open-window and lock frames are worse in **both** after-runs
(7.156/7.201, 12.081/12.077, 4.557/4.552) than in **both** before-runs
(7.034/6.457, 11.980/11.669, 4.423/4.326), while `full_frame` lands between the
two baseline runs. Slower in three rows with no row repeatably faster is not a
win. The scene's strokes are short and mostly axis-aligned, so the clamp saves a
few corner pixels per call and charges per-row arithmetic for them. Per the batch
contract's rule 5 — a saving must name what it removed — there was no saving to
name, so the change was **reverted** (`git checkout -- src/litric-shell/src/canvas.rs`)
instead of shipped.

What the measurement did establish: `stroke_line`'s cost is per-call work and the
blend, not the corner pixels of its bounding box; `soft_shadow` is 26 % of a
frame for three calls; and the shipped glass path already avoids the live blur
(`full_frame_live_blur` is the fallback a canvas with no backdrop takes, and it
costs 4.4x the shipped frame).

### 66.4 Outcome: BUG-0015 is priced, not closed

`docs/bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md` now records:

* host, release, n=15: two-window full frame **6.83 ms**, open-window frame
  **11.98 ms** — both inside the 16.67 ms gate (2.44x and 1.39x of headroom);
* the historical 206 ms / 185 ms numbers do not reproduce on this commit, and the
  record says so instead of leaving them as the current state;
* the status changes from `OPEN — measured, not fixed` to **`PRICED (host)`**,
  because the gate is a claim about a booted image on QEMU vCPUs and **no image
  was built here** (mkosi and QEMU are absent). The frame-time gate therefore
  stays unpassed, with the host number known.

No production code ships from this batch. That is the batch's result, not an
omission: the one candidate with a measured baseline behind it made the frame
slower.

### 66.5 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `docs/bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md` | 12 | status: `PRICED (host)`, image number unmeasured |
| `docs/bugs/BUG-0015-a-frame-with-a-window-costs-206ms.md` | 246 | the 2026-10-29 measurement, the reverted change and its before/after table |
| `docs/PROGRESS-TRACKER.md` | 6460 | this entry |
| `src/litric-shell/examples/render_bench.rs` | 53 | `RUNS = 15` — the n this batch used, unchanged |

### 66.6 Tests

No test was added, and none was weakened. The claim this batch makes is a
measurement, and the tests that guard the pixels it measured already exist and
were run: 57 canvas tests including the 144-case stroke matrix and the
bit-identical shadow matrices. A wall-clock assertion was deliberately **not**
added to the suite: this record's own variance section shows the same binary
moving between 212 ms and 692 ms on a loaded host, so a millisecond test would be
a flake, not a gate. The pixels stay guarded by the golden-frame check; the time
claim stays with a booted image.

### 66.7 Measured costs of this batch

| Figure | Value |
| --- | --- |
| production code changed | **0 lines** (the candidate was reverted) |
| per-frame work added | **0** |
| heap added | **0 bytes** |
| tests added / removed | 0 / 0 |
| dependencies / `unsafe` | 0 / none |
| bench runs taken | 2 before + 2 after, n=15 each |

### 66.8 Gate

```text
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
1,837 passed; 0 failed   (unchanged: this batch adds no code)

$ python3 ci/lint-doc-refs.py
OK: 1,228 citation(s) across 156 document(s) resolve

$ python3 ci/lint-file-safety.py
OK: 116 deletion site(s), all inside reviewed functions

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the recorded baseline (`mkosi config…`, `dev key pairs…`)
and the three skips are the image build, boot smoke test and idle measurement
that need mkosi/QEMU. No gate, golden frame or key file was edited.

### 66.9 Limits, stated

* **The gate is still unpassed.** 6.83 ms is a host number on a fast machine. The
  requirement is a booted image over QEMU vCPUs, and that measurement does not
  exist in this environment.
* **The variance is real.** The same record shows the identical binary between
  212 ms and 692 ms on a loaded host. Only medians from one session are compared
  here, and every number is labelled with its host.
* **The 206 ms claim is retired, not disproved.** It was true of the build that
  produced it; what this batch shows is that it is not true of `ce08ee8`, and it
  says which commit the new number belongs to.
* **One candidate was tried, not all of them.** `soft_shadow` (26 % for three
  calls) and the per-pixel blend remain, and the profile in this entry is the
  measured starting point for whichever batch takes them next.
* **No image-level or QEMU evidence** is claimed; `docs/GATES.md` rows for the
  boot and frame gates stay `NOT RUN`.

### 66.10 Git

Base `ce08ee8`. This entry and the bug record are committed on `main` and
pushed; `HEAD == origin/main` and the tracked worktree is empty at the end of the
turn.

## 67. Batch: the wallpaper copy is gone, and the bytes are measured (verified)

### 67.1 Requirement

The improvement plan scopes this batch by its proof
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 67):

> Canvas memory: one buffer + damage tiles, bytes measured | frame oracle, pixel
> diff, bytes

and names the lever it is meant to pull:

> **One canvas + damage tiles** instead of three full-screen buffers | −15.8 MiB
> at 1080p | one full-screen copy; requires damage-region correctness

The batch contract's rule 4 sets the bar for the number
(`docs/master-plan/28-whole-system-improvement-plan.md`):

> **Caps are justified by a measured distribution, not by a round number.**

and rule 2 says a saving has to name what it removed.

### 67.2 Audit, before the change

`Compositor` held three full-screen 32-bit canvases
(`src/litric-shell/src/desktop.rs`): `wallpaper` (painted once, the restore
source for a damaged region), `scene` (the frame), and the blurred glass backdrop
attached to the scene. At 1920×1080 that is 3 × 8,294,400 bytes = 24.9 MB raw,
matching `docs/PERF-budget.md` §3d's arithmetic.

The word "measured" was the part that was missing: §3d multiplied pixels, and
nothing in the tree reported what the process really allocated. So the batch
built the smallest honest harness, `src/litric-shell/examples/canvas_bytes.rs`,
which constructs the shipped `Compositor` and prints `VmRSS` from
`/proc/self/statm` before and after. One mode per process, because a second
compositor in the same process reuses the first one's freed heap and the delta
would then describe the allocator.

Baseline, before any edit:

```bash
cd src
cargo build --release -p litric-shell --example canvas_bytes
MALLOC_MMAP_THRESHOLD_=131072 ./target/release/examples/canvas_bytes 1920x1080
```

| Mode | compositor RSS |
|---|---:|
| 1280×800 | 13,008 KiB |
| 1920×1080 | 25,148 KiB |
| 2560×1600 | 48,824 KiB |

### 67.3 What changed

* `Compositor::wallpaper` is **deleted**. The scene is built from the wallpaper
  and kept; a damaged region is restored by painting the wallpaper into it under
  the region's clip, which is the same deterministic function whose output the
  copy held. The struct keeps `theme` and `choice` instead — 24 bytes rather than
  4 bytes a pixel.
* `paint_cover` walks `Canvas::span` (the canvas intersected with the clip)
  instead of the whole picture. `put` already refused pixels outside the clip, so
  the values written are unchanged; what changes is that restoring a small region
  no longer walks a whole screen to paint a few thousand pixels of it. `span`
  became public for that reason, with its contract in the doc comment.
* `Compositor::new` and `retheme` no longer clone the wallpaper canvas: the scene
  *is* it, and the backdrop is derived from it before it is moved.

### 67.4 Measured bytes, identical command, before and after

The before column is the same binary rebuilt with `desktop.rs` stashed, so only
the change differs.

| Mode | before | after | saved |
|---|---:|---:|---:|
| 1280×800 | 13,008 KiB | **8,984 KiB** | 4,024 KiB |
| 1920×1080 | 25,148 KiB | **17,184 KiB** | 7,964 KiB |
| 2560×1600 | 48,824 KiB | **32,988 KiB** | 15,836 KiB |

The saving is one canvas at every mode (4,096 / 8,100 / 16,000 KiB of raw
payload, plus allocator overhead). Without `MALLOC_MMAP_THRESHOLD_=131072` the
same run reports the saving only at 2560×1600: glibc raises its mmap threshold
after the first large free, so the freed 4 MB block stays in the heap at the
smaller modes and `VmRSS` does not fall. That is an allocator artefact, not a
property of the change, and it is why the table above names the variable.

### 67.5 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/desktop.rs` | 245 | `theme: Theme` — kept instead of the wallpaper buffer |
| `src/litric-shell/src/desktop.rs` | 246 | `choice: Wallpaper` — which wallpaper to repaint |
| `src/litric-shell/src/desktop.rs` | 265 | `new`: the scene *is* the wallpaper; no clone, no second buffer |
| `src/litric-shell/src/desktop.rs` | 287 | `retheme`: rebuilds the scene and records theme/choice |
| `src/litric-shell/src/desktop.rs` | 331 | `repaint`: a region is restored by painting, under the clip |
| `src/litric-shell/src/desktop.rs` | 441 | `paint_cover`: walks `span` (clip) instead of the whole picture |
| `src/litric-shell/src/canvas.rs` | 1336 | `span` is public: the pixels a canvas will accept |
| `src/litric-shell/examples/canvas_bytes.rs` | 25 | `rss_kb` — `/proc/self/statm`, the measurement |
| `src/litric-shell/examples/canvas_bytes.rs` | 49 | one mode per process, so the delta is the desktop |
| `docs/PERF-budget.md` | 247 | §3d corrected: two canvases, with the measured figure |

### 67.6 The test that was written, proved non-sensitive, and removed

A unit test was added: dirty a region, repaint it partially, repaint the same
frame fully, and assert every pixel of the region matches. It passed — and then
it **also passed with the wallpaper restore deleted**, which is the only thing it
was supposed to defend:

```text
=== SENSITIVITY: skip the wallpaper restore ===
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 725 filtered out
```

The reason is real and worth recording: `views::draw` repaints every pixel of the
clip, so a region's final pixels are the desktop's, not the wallpaper's, and the
restore is invisible to that comparison at both positions tried (centre of the
screen and bare wallpaper at the right edge). A test that stays green when the
behaviour it names is removed proves nothing, so it was deleted rather than
shipped. The pixel oracle that does exist and was run is the golden-frame check
inside `ci/verify.sh`, which compares whole frames byte for byte.

### 67.7 Gate

```text
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
1,837 passed; 0 failed   (unchanged: no test ships from this batch)

$ python3 ci/lint-doc-refs.py
OK: 1,246 citation(s) across 156 document(s) resolve

$ python3 ci/lint-file-safety.py
OK: 116 deletion site(s), all inside reviewed functions

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the recorded baseline (`mkosi config…`, `dev key pairs…`);
the three skips are the image build, boot smoke test and idle measurement. The
golden-frame check is among the 32 passes, so whole frames are unchanged. No
gate, golden frame or key file was edited.

### 67.8 Costs and limits, stated

| Figure | Value |
| --- | --- |
| heap at rest | **−4,024 KiB @ 1280×800 · −7,964 KiB @ 1920×1080 · −15,836 KiB @ 2560×1600** |
| production code changed | one field removed, one accessor made public, one loop bounded |
| per-frame work added | not measured — see below |
| dependencies / `unsafe` | 0 / none |

* **The time cost was not measured here.** `render_bench` does not call
  `Compositor::repaint` (it restores with its own blit), and `frame_dump` refuses
  to run on this host: *"no Latin text face is installed, so no frame can be
  drawn"*. So the claim this batch makes is bytes; the cost of painting a
  damaged region instead of blitting it is **unmeasured**, and the loop it uses
  is bounded by the clip rather than the screen, which is the shape that made it
  acceptable to ship without the number.
* **Not one buffer with damage tiles yet.** The plan's lever is "one canvas +
  damage tiles"; this batch removes the third full-screen buffer (the wallpaper
  copy) and leaves two (scene + glass backdrop). The backdrop is a real cache
  that saves the live blur, so removing it is a different trade, not an
  oversight.
* **The lock screen still adds up to two full-screen caches** (`lock::Cache`),
  unchanged here.
* **No image-level evidence.** These are host `VmRSS` figures; a booted image's
  cgroup memory is still unmeasured (`docs/GATES.md` stays `NOT RUN`).

### 67.9 Git

Base `f4b04da`. This batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

## 68. Batch: every shell timer named, and the idle plan held to 1.00/s (verified)

### 68.1 Requirement

The improvement plan names this batch and its proof
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 68):

> Idle wakeup audit: every shell timer named with a reason | timer table, ≤ 2/s
> target

and the budget it is measured against (`docs/PERF-budget.md` §4):

> | Wakeups/second, `litric-shell` alone | **≤ 2 /s** | `powertop` |
>
> A shell that idles at ≤ 2 wakeups/second is only possible if it is **fully event
> [driven]**

The batch contract's rule 5 applies: no claim without a tool, a command, an
environment and `n`.

### 68.2 The inventory (file → line, verified with grep on this commit)

| File:line | Timer | Period | Armed while idle? | Reason |
| --- | --- | ---: | --- | --- |
| `src/litric-shell/src/shell.rs` | 4138 | `IDLE_SLEEP_CAP_MS` = 1,000 ms | **yes** | Floor under the idle plan: the top bar's clock changes on the second, so a longer sleep shows a stale clock. This is the wake the 1.00/s figure comes from. |
| `src/litric-shell/src/shell.rs` | 4176 | the wake plan | — | The plan itself: `min(until next clock change, until the idle state changes, the cap)`. It is the only thing that sets idle pacing. |
| `src/litric-shell/src/main.rs` | 63 | `FACTS_INTERVAL_MS` = 2,000 ms | no (serviced on the next clock wake) | Re-sample facts, drain the notification inbox, re-query environments. Keeps a notification's worst case at 2 s. |
| `src/litric-shell/src/main.rs` | 66 | `STATUS_INTERVAL_MS` = 5,000 ms | no | Rewrite the status file the regression CI reads. |
| `src/litric-shell/src/main.rs` | 76 | `CURSOR_INTERVAL_MS` = 8 ms | **no** | Cursor step, armed only while a pointer step is pending. Not a frame and not idle. |
| `src/litric-shell/src/shell.rs` | 2262 | `MIN_GAP_MS` = 50 ms | **no** | Minimum gap between animation frames, only while a spring is mid-flight. |
| `src/litric-shell/src/shell.rs` | 3244, 3247, 3251 | `BOOT_INTRO_MS` 2,300 · `BOOT_WRITE_MS` 1,600 · `BOOT_COVER_MS` 1,950 | **no** | Boot intro staging, finished 2.3 s after start. |
| `src/litric-shell/src/shell.rs` | 3526 | `REVERT_WINDOW_MS` = 15,000 ms | **no** | Deadline of the "revert" toast, armed only after a settings revert. |
| `src/litric-shell/src/desktop_entry.rs` | 1264 | `WATCH_INTERVAL_MS` = 1,000 ms | only if the launcher is watching | Desktop-entry poll; `next_ms` is advanced per poll (1404, 1421), so it adds no wake the plan did not already ask for. |
| `src/litric-shell/src/filewatch.rs` | 36 | `INTERVAL_MS` = 1,000 ms | only while a Files window watches | Outside-directory change poll for the open table. |

Every entry has a reason, and the three that are armed while idle (the cap, the
clock boundary, and — if a window is open — a 1 s watcher) are the ones the
number below counts. Nothing in the list exists "because it was easy": the facts
timer is the notification latency budget, the status timer is what CI reads, and
the cap is the clock's seconds.

### 68.3 Measured: the idle plan asks for 1.00 wakeups a second

The measurement walks the shell's own answer to "how long may I sleep" for ten
seconds of an idle desktop, waking exactly where the plan says and drawing each
frame the way the loop does:

```bash
cd src
cargo test -p litric-shell --lib the_idle_plan_asks_for_at_most -- --nocapture
```

```text
idle_wakeups_per_s=1.00 over 10000 ms (10 wakes)
test result: ok. 1 passed; 0 failed; ... finished in 0.00s
```

Ten wakes in ten seconds, against a budget of twenty. The first version of this
harness reported **1000.00/s**, which was the harness and not the shell: it never
drew, so `wake_after_ms` correctly answered "0, there is a frame to draw". The
loop's draw step (`take_damage` + `counted_frame`) is part of the measurement, and
the number above is what remains once the desktop is clean.

**What this is and is not.** It is a measurement of the *plan the shipped loop
follows*, not a kernel count. The kernel count was attempted first and is not
available on this host: `litric-shell --presenter` accepts `auto`, `drm` and
`fbdev` only, and there is no framebuffer or DRM node here, so the binary exits
with its usage text instead of idling. The `rusage`-based `ctxt_switches` count
therefore never ran, and no proxy is presented as a kernel figure.

### 68.4 The change: the budget is now enforced, and the rejected candidate

What ships is the audit table above plus one test,
`the_idle_plan_asks_for_at_most_two_wakeups_a_second`
(`src/litric-shell/src/shell.rs`, 11549), which fails the suite if the idle plan
ever asks for more than two wakes a second. That is the difference between a
budget in a document and a budget in the tree.

The candidate that was considered and **rejected with a measurement**: raising
`IDLE_SLEEP_CAP_MS` from 1,000 to 2,000 to halve the wakes. It cannot help,
because the binding term is the clock boundary, which is at most one second away;
the cap is a floor, not the cost. The measured figure would not move, so the
constant keeps its reason and its value. There is no wakeup *reduction* to claim
from this batch, and none is claimed.

### 68.5 Sensitivity proof

Forcing the idle plan to poll every 8 ms (`wake_after_ms` returning `8`) makes the
test fail with its real message:

```text
idle_wakeups_per_s=125.00 over 10000 ms (1250 wakes)
thread 'shell::sidebar_scroll::the_idle_plan_asks_for_at_most_two_wakeups_a_second' panicked at litric-shell/src/shell.rs:11599:9:
an idle shell asks for 125.00 wakeups a second, over the 2/s budget
test result: FAILED. 0 passed; 1 failed
```

Restored, it prints `1.00` and passes. The test is sensitive to exactly the
regression it names (a timer that starts polling faster), which is the lesson
§67 recorded when a test stayed green under its own probe.

### 68.6 Wiring (file → line, measured on this commit)

| File | Line | What it does |
| --- | --- | --- |
| `src/litric-shell/src/shell.rs` | 11549 | the test: ten seconds of idle, wakes counted, `≤ 2/s` asserted and printed |
| `src/litric-shell/src/shell.rs` | 4138 | `IDLE_SLEEP_CAP_MS` — the floor, with its reason in the doc comment |
| `src/litric-shell/src/shell.rs` | 4176 | `wake_after_ms` — the plan the test measures |
| `src/litric-shell/src/main.rs` | 63, 66, 76 | facts · status · cursor timers, each with its reason |
| `src/litric-shell/src/desktop_entry.rs` | 1264 | the desktop-entry watch interval |
| `src/litric-shell/src/filewatch.rs` | 36 | the Files watch interval |
| `docs/PERF-budget.md` | 302 | the budget row now carries the measured 1.00/s and the command |

### 68.7 Gate

```text
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
exit 0

$ cargo test --workspace
1,838 passed; 0 failed   (was 1,837; +1 test)

$ python3 ci/lint-doc-refs.py
OK: 1,267 citation(s) across 156 document(s) resolve

$ python3 ci/lint-file-safety.py
OK: 116 deletion site(s), all inside reviewed functions

$ bash ci/verify.sh
summary: 32 pass, 2 fail, 3 skipped
```

The two failures are the recorded baseline (`mkosi config…`, `dev key pairs…`);
the three skips need mkosi/QEMU. The golden-frame check is among the 32 passes,
so no frame changed. No gate, golden frame or key file was edited.

### 68.8 Costs and limits, stated

| Figure | Value |
| --- | --- |
| wakeups added per second | **0** (no timer changed) |
| wakeups removed per second | **0** (none needed removing; the plan was already inside the budget) |
| work added per frame | **0** |
| heap added | **0 bytes** |
| dependencies / `unsafe` | 0 / none |
| tests added | 1 (sensitive, proved above) |

* **The number is the plan, not the kernel.** No framebuffer or DRM node exists
  here, so `litric-shell` cannot run its loop on this host and
  `/proc/<pid>/status` cannot be sampled. `powertop`, which `PERF-budget` names,
  is not installed either. The proxy is the loop's own sleep plan, and it is
  labelled as such everywhere it appears.
* **Watchers are conditional.** `WATCH_INTERVAL_MS` and `INTERVAL_MS` are armed
  only while a launcher or Files watch exists, which the idle figure does not
  include. A window left open adds one poll a second on top of the 1.00/s — still
  inside 2/s, but it is a different steady state and is stated rather than
  averaged in.
* **The clock is the floor.** Reaching fewer than one wake a second means showing
  the time to the minute instead of the second. That is a user-visible product
  decision, not an optimisation, and it was not taken.
* **No image-level evidence.** The idle figure is host-side; a booted image's
  wakeups are still unmeasured (`docs/GATES.md` stays `NOT RUN`).

### 68.9 Git

Base `eacf5d1`. This batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

---

## 69. Batch: the text cache is bounded in bytes, and the per-glyph cost is printed (verified)

### 69.1 Requirement

The improvement plan names this batch and its proof
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 69):

> Text atlas measured and capped per glyph | bytes/glyph test | here

and the lever it serves (`docs/master-plan/28-whole-system-improvement-plan.md`,
the RAM table):

> | Text atlas cap + per-glyph accounting | bounded, measured | untracked atlas
> growth |

Stage D's outcome row is the same requirement in one line:

> Text atlas and glyph caches measured and capped; bytes per glyph printed by a
> test

The batch contract's rule 5 applies: no claim without a tool, a command, an
environment and `n`.

### 69.2 The inventory (file -> line, verified with grep on this commit)

| File:line | What it allocates | Bytes per glyph for which key | Growth trigger | Never freed? | Named reason |
| --- | --- | --- | --- | --- | --- |
| `src/litric-text/src/lib.rs:20` | `CACHE_LIMIT = 512` shaped runs | none: it counts *runs*, not glyphs | every cache miss | oldest run past 512 | Bounds the lookup work, not the memory. Kept as a second bound after this batch. |
| `src/litric-text/src/lib.rs:30` | `CACHE_BYTE_LIMIT = 8 MiB` of runs | 303.9 bytes/glyph for a 256-byte mask, measured below | every cache miss | **new**: evicted from the oldest end until under the limit | The run's size is text length x font size x masks, so 512 short and 512 long runs differ by orders of magnitude. 8 MiB is a little under one 1280x800 frame. |
| `src/litric-text/src/lib.rs:104` | `run_bytes`: entry struct + text capacity + glyph-vector capacity + every mask's coverage buffer | this is the per-glyph accounting itself | per insert | n/a (accounting) | Without the coverage buffers counted, the cap would be a count in disguise. |
| `src/litric-text/src/lib.rs:121` | `cache_bytes`: the fold over entries | n/a | per insert | n/a (accounting) | One number for the whole cache so the bound is checkable from one place. |
| `src/litric-text/src/lib.rs:133` | `evict_to`: byte bound first, then the count bound | n/a | per insert | pops the oldest | The last entry is never evicted: one run larger than the limit is still the answer to the question asked, and dropping it would miss forever on that string. |
| `src/litric-text/src/lib.rs:492`, `:496` | `remember` after a fresh shape | n/a | cache miss | no | One insertion point for both the blended and the single-face path, so nothing can bypass the bound. |
| `src/litric-text/src/lib.rs:607` | `evict_to(&mut cache, CACHE_BYTE_LIMIT)` | n/a | per insert | pops the oldest | The only call site of the cap in the crate. |
| `src/litric-shell/src/font.rs:1108` | `GLYPHS`: the 105-glyph vector fallback table | 8 bytes/glyph for `BOXES` (see below) | never (static) | process exit | Compiled-in artwork; it is the fallback when no face is installed, not a cache. |
| `src/litric-shell/src/font.rs:1663` | `BOXES`: `OnceLock<Vec<(f32, f32)>>` | 8 bytes/glyph (105 glyphs -> 840 bytes, measured by arithmetic on the static table) | first `ink_box` call (`:1661`) | never, and deliberately so | Inert ink extents for the vector path; a fixed 840 bytes is cheaper than recomputing a bounding box per frame. |
| `src/litric-text/src/lib.rs` `LoadedFace.bytes` | the six installed face files, one `Arc<[u8]>` each | face-sized, not per glyph | load, once | process exit | The faces are the fonts; there is nothing to evict and reloading them per string would be the actual cost. |

The two classes that could grow without bound before this batch were the run
cache's byte content (capped now) and nothing else: the vector table is static,
the face bytes are fixed at load, and the masks live inside the cached runs.

### 69.3 Measured: 303.9 bytes/glyph, and a 96,870-byte cache becomes 7,266

```bash
cd src
cargo test -p litric-text --lib the_run_cache_is_capped -- --nocapture
```

```text
run_bytes=2431 bytes_per_glyph=303.9 coverage_per_glyph=256 count_limit=512 byte_limit=8388608
cache_before_bytes=96870 cache_after_bytes=7266 cache_limit=7294 runs_kept=3
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
```

Read honestly: **this host has no installed font face** (`/usr/share/fonts` is
empty and no `NotoSans-*.ttf` exists anywhere), so a real Noto glyph mask cannot
be measured here -- the same limitation §59, §61 and §68 recorded for text. The
figure above is the accounting itself, driven through the exact `run_bytes` and
`evict_to` the cache calls, over runs of eight 16x16 masks (256 coverage bytes
each): 2,431 bytes per run, 303.9 of them per glyph, of which 256 are the mask.
It is a **fixture measurement of the bound**, not an average Noto glyph. What it
does prove is that the bound holds on real code: forty runs of 2,431 bytes
(96,870 bytes) were evicted down to 7,266 bytes against a 7,294-byte limit, with
the newest kept and the oldest gone.

At the shipped ceiling (8,388,608 bytes) and this fixture's 303.9 bytes/glyph the
cache holds on the order of 27,600 glyph masks before the byte bound bites --
several hundred ordinary UI strings -- and the count bound still stops a cache of
tiny runs at 512 (`src/litric-text/src/lib.rs:20`).

### 69.4 The change, and why it costs nothing per frame

`evict_to` asks the same question it always did, plus one integer comparison per
stored entry: the fold is over entries already in memory, on the insert path
only. There is **no work added to the paint path** -- `layout` hits `find_cached`
(`src/litric-text/src/lib.rs:581`) and returns an `Arc<Run>` clone, exactly as
before -- and no new dependency, no `unsafe` (`#![forbid(unsafe_code)]` still
holds in the crate), and no new per-frame allocation. The bound is checked when
the cache grows, which is when memory is spent.

The over-limit case is honest rather than clever: one run larger than 8 MiB stays
in the cache (the last-entry rule at `src/litric-text/src/lib.rs:133`), because a
run that big is the answer to the call that just happened; the alternative is a
guaranteed miss and a re-rasterisation of the same string on every frame that
asks for it.

### 69.5 Sensitivity: removing the bound must fail, and it does

```text
probe removed the byte comparison from evict_to
cache_before_bytes=96870 cache_after_bytes=96870 cache_limit=7294 runs_kept=40
thread 'tests::the_run_cache_is_capped_in_bytes_as_well_as_in_runs' panicked at
litric-text/src/lib.rs:835:9:
the byte cap did not hold: 96870 bytes against a 7294-byte limit
test result: FAILED. 0 passed; 1 failed; ...
restored -> test result: ok. 1 passed; 0 failed
```

The test is sensitive: with the byte comparison forced false the cache keeps all
forty runs and the assertion fails with the real numbers; restored, it passes.
The run-count half of the same test forces `CACHE_LIMIT + 50` tiny entries and
checks the count bound still stops at 512.

### 69.6 Gates (run on this commit, real output)

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | **1,839 passed, 0 failed** (+1 against 1,838) |
| `python3 ci/lint-doc-refs.py` | `OK: 1273 citation(s) across 156 document(s) resolve` (1,267 before this section was added) |
| `python3 ci/lint-file-safety.py` | `OK: 116 deletion site(s), all inside reviewed functions` |
| `bash ci/verify.sh` | **32 pass, 2 fail, 3 skipped** -- the pre-existing `mkosi config` and `dev key pairs` fails, unchanged |

One mistake was caught by these gates and fixed: the first version of the test
computed `per_run / 8`, which the workspace clippy configuration rejects as
`integer division`; `ci/verify.sh` went to **31 pass, 3 fail** with the clippy
check red. The fix was to compute the per-glyph figure as `f64`, after which the
gate returned to 32/2/3. No assertion was weakened to get there.

### 69.7 Limits, stated plainly

* **No real-glyph measurement on this host.** The 303.9 bytes/glyph figure comes
  from a 16x16 fixture mask, not from Noto Sans. A run of real Noto text at 13 px
  will have smaller masks and a different per-glyph figure; it has not been
  measured because no face exists here. Label it a fixture measurement, not an
  atlas average.
* **The vector path is not capped, it is static.** 105 glyphs x 8 bytes = 840
  bytes, computed from the table's own length; there is nothing to evict.
* **The bound is per `TextSystem`, not per process.** There is one system today;
  a second one would be a second 8 MiB ceiling.
* **No image-level evidence.** All of this is host-side; a booted image's text
  RSS is still unmeasured (`docs/GATES.md` stays `NOT RUN`).
* **The count bound is still the first line of defence for tiny runs.** 512 runs
  of one glyph each is ~20 KiB; the byte cap does not tighten that case.

### 69.8 Git

Base `9bf3099`. This batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

---

## 70. Batch: the Chapter F components exist, are keyboard-reachable, and are shown (verified)

### 70.1 Requirement

The improvement plan names this batch and its proof
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 70):

> Chapter `F`: tooltips + pickers + grids/empty states | keyboard path per
> component, screenshots | here

and Stage E's outcome row (`docs/master-plan/28-whole-system-improvement-plan.md`,
E1):

> Tooltips (absent entirely), pickers/views/trees, grids and empty/loading
> states, progress and validation — one component contract in
> `src/litric-shell/src/ui.rs`, each with a keyboard path

The corpus measured the absences this batch closes: `F14.01` ("the shell has no
tooltip system"), `F07.03` (the accent is a cycling row where the design system
asks for a 7-colour picker), `F21.05` ("no grid") and `F24.03` ("no empty state
offers a way out"). The batch contract's rule 5 applies: no claim without a
tool, a command, an environment and `n`.

### 70.2 What landed, and where (file → line, verified with grep on this commit)

| Component | Contract | Wired to a real surface | Keyboard path |
| --- | --- | --- | --- |
| **Tooltip** | `ui::Tooltip<T>` state machine (`src/litric-shell/src/ui.rs:1659`) with `TOOLTIP_DELAY_MS = 600` (`:1633`), the brush `ui::tooltip` (`:1734`) | the glyph-only status-bar icons: the bell, network, sound, battery, power, metrics, clock and account each carry a label (`status_tooltip`, `src/litric-shell/src/views.rs:3642`), drawn only while that control owns the tooltip (`src/litric-shell/src/views.rs:648`) | hover arms the delay and **focus shows it without waiting**; the state is advanced by `Shell::set_now` (`src/litric-shell/src/shell.rs:4867`), never by the paint path |
| **Picker** | `ui::segmented` (`src/litric-shell/src/ui.rs:1791`) + `ui::pick_step` (`:1771`) | Settings → Appearance: the accent row is now six visible choices (`src/litric-shell/src/views.rs:3065`), each a hotspot (`Hot::SettingsAccentChoice`, `src/litric-shell/src/shell.rs:430`), and `set_accent` (`src/litric-shell/src/shell.rs:3461`) selects **the value chosen**, not the next one | every segment is a registered control, so Tab reaches each and Enter chooses it; the high-contrast path keeps the fixed-label row and says why |
| **Grid** | `ui::grid_cell` (`src/litric-shell/src/ui.rs:1842`) + `ui::grid_step` (`:1867`) | the launcher's tiles: `LAUNCHER_COLUMNS = 4` (`src/litric-shell/src/views.rs:4280`) is read by both the draw and the keys, and the cell math is the shared function | Down/Up move a **row** (4 tiles) instead of one tile; Left/Right move one tile; Home/End jump to the ends. A typed query is a list and steps one result at a time |
| **Empty state** | `ui::EmptyState` + `ui::empty_state` (`src/litric-shell/src/ui.rs:1895`, `:1908`) returning the action's rect for the hotspot | Files, an empty folder: title "Empty folder", the sentence "Nothing in this folder yet." and the action "New folder" (`src/litric-shell/src/views.rs:1403`), whose hotspot (`FilesHot::EmptyNewFolder`, `src/litric-shell/src/apps.rs:1813`) opens the new-folder dialog (`src/litric-shell/src/apps.rs:1821`) | the action is a registered hotspot in the same focus order as every other control; Enter activates it |

The tooltip is deliberately **not** in the paint path: `ui::Tooltip::sync` and
`tick` are two integer comparisons in `set_now`, and the brush only reads
`visible_for`. No allocation, no I/O, no new dependency, and `forbid(unsafe_code)`
still holds. §55's "30 frames, no stat" and §56's "poll only at period"
assertions still pass, and the idle figure from §68 is unchanged because a
tooltip that is waiting is one wake the plan already schedules through the clock
boundary.

### 70.3 The screenshots, and what they show

Produced by the real paint path in the new journey, not composed for the
occasion: `save_shot` writes the canvas `views::draw` produced.

```bash
cd src
LITRIC_E2E_SHOTS=../build/shots70 \
  cargo test -p litric-shell --lib journey_chapter_f
```

| Shot | What is visible |
| --- | --- |
| `70-picker-accent` | Appearance page: "Accent" above six segments — Default, Sky, **Violet**, Teal, Amber, Rose — with Violet outlined because the journey chose it. The value space is on screen; nothing cycles. |
| `70-launcher-grid` | the four-column application grid after Down and Right, with the selection on the next row's first tile. |
| `70-empty-state` | Files at an empty folder: "Empty folder", "Nothing in this folder yet.", and the accent-filled "New folder" action. |
| `70-empty-state-action` | the new-folder dialog opened by that action. |
| `70-tooltip` | hovering the bell for 600 ms: the label "Notifications" above the glyph-only icon, with the rest of the desktop unchanged. |

Both images that mattered most were opened and read: the picker shows the six
named choices with Violet selected, and the tooltip shows the word
"Notifications" in a bordered surface directly above the bell, legible at the
1280×800 frame's real size. The empty-state shot after the launcher spring had
finished shows the title, the sentence and the button without overlap; the first
attempt was taken while the launcher panel was still covering the window, which
is why the journey now waits for the morph before it shoots.

The golden set was re-recorded because two frames legitimately changed:
`05-settings` (the accent picker) and `16-files-empty` (the empty state gained a
title, a sentence and an action). `ci/check-golden-frames.py --frames
build/frames --update` rewrote `docs/evidence/golden/fingerprints.json`, and the
gate now passes.

### 70.4 Sensitivity: the new tests fail when the behaviour is removed

Two probes were forced at once — `pick_step` returning `None` and
`Tooltip::tick` never showing:

```text
probe: picker is inert, tooltip never shows
panicked at litric-shell/src/ui.rs:1976:
assertion failed: tip.tick(1_600)
panicked at litric-shell/src/ui.rs:1989:
assertion `left == right` failed
panicked at litric-shell/src/e2e.rs:724:5
test result: FAILED. 4 passed; 3 failed
restored -> test result: ok. 7 passed; 0 failed
```

The e2e journey failed on the same probe, so the wiring is covered too, not only
the pure functions. Restored, all seven pass.

### 70.5 Gates (run on this commit, real output)

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | **1,846 passed, 0 failed** (+7 against 1,839: six component tests and the journey) |
| `python3 ci/lint-doc-refs.py` | `OK: 1278 citation(s) across 156 document(s) resolve` |
| `python3 ci/lint-file-safety.py` | `OK: 116 deletion site(s), all inside reviewed functions` |
| `bash ci/verify.sh` | **32 pass, 2 fail, 3 skipped** — the pre-existing `mkosi config` and `dev key pairs` fails, unchanged |

Mistakes hit and fixed in this batch, all caught by the gates rather than by
argument: an over-tight grid-overflow assertion (the fourth cell did fit) was
corrected to a genuinely narrow grid; a first tooltip test armed the delay and
advanced the clock in the same tick, so it needed two ticks; clippy rejected
`&t` where `t` was already a reference, an indexed `windows(2)` comparison
(rewritten as a slice pattern) and a `matches!(x, None)` (rewritten as
`is_none()`); and the existing settings journey still expected the cycling row,
so it now asserts the picker's chosen value instead.

### 70.6 Limits, stated plainly

* **Four component families landed; the picker is the accent one.** There is
  still no date, time, colour-spectrum or file picker (`F.07`'s other three
  families), no column resize/reorder (`F21.03`), and no grid view in Files.
  The contract exists; those surfaces are not wired to it.
* **Tooltips cover the status bar, not the window controls.** The unlabelled
  window dots (`F14.02`) still have no tooltip.
* **The launcher's Up/Down changed meaning**: one row (four tiles) instead of one
  tile. That is the grid's keyboard contract, and it is a user-visible behaviour
  change, not a refactor.
* **No screenshot from a booted machine.** These frames come from the same
  `views::draw` the image runs, with fixed facts; `docs/GATES.md` stays
  `NOT RUN` for the image-level rows.
* **No assistive-technology proof.** The accessibility tree is batch 71; a
  tooltip that a screen reader cannot read is still open here.

### 70.7 Git

Base `d96807d`. This batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

---

## 71. Batch: an accessibility tree built from what was drawn, and a keyboard that walks it (verified)

### 71.1 Requirement

The improvement plan names this batch and its proof
(`docs/master-plan/28-whole-system-improvement-plan.md`, batch 71):

> Accessibility tree: roles, names, focus order, keyboard-only proof | tree dump
> test, e2e journey | here

and the corpus item it closes (`Research-Deep-all/LitricOS UI UX Research Part 19 G
Contrast Keyboard and Screen Readers.md`, `G` chapter): an interface that draws
every control by hand must also be able to say what those controls are. §70 left
this open in writing: "a tooltip that a screen reader cannot read is still open
here". The batch contract's rule 5 applies: no claim without a tool, a command,
an environment and `n`.

### 71.2 The contract, and where it lives (file → line, verified with grep on this commit)

| Piece | What it is | Line |
| --- | --- | --- |
| `Role` | the vocabulary: button, text, list, listitem, dialog, statusbar, tab, segment, grid, gridcell, window, progressbar | `src/litric-shell/src/a11y.rs:31` |
| `Node` | one control: role, name, device-pixel rect, `focused`, `disabled`, `tooltip`, grid `row`/`col`, `container` | `src/litric-shell/src/a11y.rs:81` |
| `Tree` | nodes in paint order, the `unnamed` count and the names of what could not be named, plus `dump()` | `src/litric-shell/src/a11y.rs:106`, `:121` |
| `control_tooltip` | the one label a glyph-only control has — the §70 tooltip — used as its name | `src/litric-shell/src/a11y.rs:163` |
| `Shell::accessibility_tree` | the projection: walks the frame's hotspot table (`Hotspots::ids()`) and builds nodes on demand | `src/litric-shell/src/shell.rs:11652` |
| `Shell::a11y_name` | role and name per hotspot, from the text the surface already draws (labels, file names, page titles, accent names) | `src/litric-shell/src/shell.rs:11733` |
| structural nodes | one `statusbar` container, one `grid` container and one `dialog` container, inserted before their first child | `src/litric-shell/src/shell.rs:11717` |

The design is a **projection, not a second model**: the tree is built from the
hotspot table the frame already produced, so a node cannot point at something
that was not painted, and the Tab order *is* the tree order because
`move_keyboard_focus` walks the same `Hotspots::ids()` filtered by
`Hot::keyboard_focusable`. Names are the strings the surfaces already draw — the
status controls' names are exactly the tooltips §70 added. A control whose label
this build cannot know is **reported as unnamed**, never given a generic name:
`Tree::unnamed` and `Tree::unnamed_names` exist so "Button" cannot stand in for a
name and look accessible.

### 71.3 Cost

**0 work and 0 allocation per frame.** Nothing is constructed while painting; the
tree is assembled once per call from the existing hotspot table
(O(hotspots), a few dozen nodes), so an idle desktop with no assistive
technology attached pays nothing. §55's "30 frames, no stat" and §56's "poll only
at period" assertions still pass, and no golden frame changed: this batch draws
no pixels.

### 71.4 Keyboard-only proof, measured

Two journeys, no pointer event in either:

```bash
cd src
cargo test -p litric-shell --lib journey_accessibility_tree -- --nocapture
cargo test -p litric-shell --lib journey_every_surface -- --nocapture
```

* `journey_accessibility_tree_names_controls_and_matches_the_tab_order`
  (`src/litric-shell/src/e2e.rs:754`) draws Settings → Appearance and asserts:
  `unnamed == 0` on that surface (the failure message prints the unnamed names
  and the whole dump); the accent choices are `segment` nodes named Violet,
  Teal, …; the sidebar is `tab` nodes named from `Page::title()`; the bell is a
  `statusbar` node named by its tooltip; the dump contains
  `segment "Violet"`; then it presses **Tab** once per focusable node and checks
  that every step lands on the next node of the tree, wrapping like
  `move_keyboard_focus`, and that exactly one node carries `focused`.
* `journey_every_surface_is_reachable_and_escapable_from_the_keyboard`
  (`src/litric-shell/src/e2e.rs:842`) Tab-reaches the Apps menu, opens the
  launcher with Enter, asserts the tiles are `gridcell` nodes carrying
  `row`/`col`, moves the selection with Right, leaves with Escape, then
  Tab-reaches the Files empty state's "New folder" button, opens the dialog with
  Enter and closes it with Escape — no trap.

Both pass on this commit (`test result: ok`), and the tab-order journey is the
one that proves the claim "focus order = tree order" rather than restating it.

### 71.5 Sensitivity: removing the behaviour makes the tests fail

Two probes, each run alone against the new tests:

```text
probe: every node reports focused=false
panicked at litric-shell/src/e2e.rs:809:9:
a focused node is in the tree
test result: FAILED. 0 passed; 1 failed
restored -> test result: ok. 1 passed

probe: the accent choices stop being named
panicked at litric-shell/src/e2e.rs:763:5:
unnamed: ["Window(1)", "SettingsSidebarScroll", "Workspace(0)", ...]
  left: 6
  right: 0
test result: FAILED. 0 passed; 1 failed
restored -> test result: ok. 1 passed
```

The second probe is also the evidence for the naming rule: the shell reports the
six hotspots it could not name by name, instead of inventing labels. (Those six
were then mapped — window bodies, the Settings sidebar's scroll surface, and the
workspace pips — which is why the committed test asserts zero.)

### 71.6 Gates (run on this commit, real output)

| Gate | Result |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | **1,848 passed, 0 failed** (+2 against 1,846: the two journeys) |
| `python3 ci/lint-doc-refs.py` | `OK: 1279 citation(s) across 156 document(s) resolve` |
| `python3 ci/lint-file-safety.py` | `OK: 116 deletion site(s), all inside reviewed functions` |
| `bash ci/verify.sh` | **32 pass, 2 fail, 3 skipped** — the pre-existing `mkosi config` and `dev key pairs` fails, unchanged |

Mistakes hit and fixed: the first `unnamed == 0` attempt counted six structural
hotspots as missing controls, which is how window bodies, the Settings
sidebar's scroll surface and the workspace pips got names; the Tab-order check
initially treated structural containers as stops, so `Node::container` now marks
them and the check filters exactly the set `move_keyboard_focus` uses; clippy
rejected `pair[0]`/`pair[1]` (rewritten as a slice pattern); and the new journeys
are slow (≈36 s each) because they draw a frame per Tab — a cost in test time,
not in the shell.

### 71.7 Limits, stated plainly

* **No assistive-technology integration.** This is a tree inside the process; no
  AT-SPI/D-Bus endpoint, no screen-reader handshake and no announcement stream
  exists yet. Nothing on a real desktop reads it.
* **Coverage is not total.** Appearance asserts `unnamed == 0`, and the surfaces
  the two journeys walk (status bar, launcher grid, Files empty state, dialog)
  are named; the browser's tab strip, Trash rows, context menus, sort headers and
  several scroll surfaces still contain hotspots with no name, and the tree
  reports them rather than hiding them.
* **Names are static or model strings, not localised at runtime** beyond what the
  surfaces already print (the file rows and page titles are model strings; the
  toolbar labels are the same literals the buttons draw).
* **No accessibility for IME or high-contrast outside the existing theme gate**;
  those remain settings/theme work, not tree work.
* **A container's rectangle is the union of its children's extent** (the status
  bar reports the width of what it draws), not a layout box of its own.

### 71.8 Git

Base `3fb0125`. This batch is committed on `main` and pushed; `HEAD ==
origin/main` and the tracked worktree is empty at the end of the turn.

## 72. Batch: account creation and password change in the privileged daemon (verified)

### 72.1 Requirement

First-run setup must create the account and a later Settings action must change
its password, without the shell spawning `useradd`/`chpasswd` and without a
password ever reaching an argv, an environment block, a log line, a `Debug`
rendering or a reply. Decision A puts the privileged path in `litric-sysd`.

### 72.2 The contract, and where it lives (file → item, verified with grep)

* `src/litric-sysd/src/auth.rs` — `validate_password` (one rule set: name shape,
  ≥12 characters, confirmation equal, password not the name); `create_plans`
  (`useradd` → `chpasswd`); `change_plans` (`unix_chkpwd` → `chpasswd`);
  `AuthProvider` (`create`/`change`/`verify`); `SystemAuthProvider` runs the
  steps through `action::Runner`, stops after a failed first step, maps a wrong
  current password to `VerifyFailed`, scrubs stdin after each step; `FakeAuth`.
* `src/litric-sysd/src/providers.rs` — `AuthAction::{Verify, Create, Change}`,
  `token()` (`verify`/`create`/`change`), `user()`, `capability()` (all three:
  `AuthSession` scoped `lock://<user>`), redacting `Debug`. `auth_plan` keeps the
  one-step verification and refuses Create/Change because they are two plans.
* `src/litric-sysd/src/proto.rs` — `AUTH CREATE <user> <password-hex>
  [CONFIRM]` and `AUTH CREATE <user> <password-hex> <confirm-hex> [CONFIRM]`;
  `AUTH CHANGE <user> <current-hex> <new-hex> [CONFIRM]` and the four-field
  form; `AUTH VERIFY` parser untouched. A field spelled `CONFIRM` at the
  confirmation position is consent, never a hash.
* `src/litric-sysd/src/state.rs` — `Sysd::auth` (`Box<dyn AuthProvider>`),
  `with_auth`, and `Sysd::account`, reached from `Sysd::action` only for Create
  and Change. Verification keeps the old plan path unchanged.

### 72.3 Capability and consent

No new `CapabilityId`: the vocabulary already has `AuthSession`, and the policy
grant table already covers `lock://`. All three verbs therefore require the
`AuthSession` grant plus the live `CONFIRM` word; without consent the broker
refuses before the backend is called.

### 72.4 Tests added

* `proto::tests::account_creation_and_change_parse_without_echoing_a_secret` —
  both verbs parse to the exact action, `Debug` redacts every password field, and
  a non-hex field is refused without echoing it.
* `proto::tests::auth_lines_are_stable_across_parse_and_recomposition` — the
  canonical composer → parser → composer round trip is byte-identical for the
  legacy `AUTH VERIFY` line and for both spellings of create and change.
* `state::tests::account_creation_and_change_cross_the_protocol_with_shared_validation`
  — `Request::parse` → `Sysd::handle` against `FakeAuth`: create succeeds; weak
  password and mismatched confirmation are refused with `weak-password` /
  `password-mismatch`; a wrong current password gives `verify-failed` and writes
  nothing; the right one lands and becomes the current password.
* `auth::tests::{the_system_provider_creates_in_two_steps_without_a_password_in_argv,
  a_failed_first_step_never_runs_the_second,
  a_spawn_failure_and_a_bad_current_password_are_distinct}` — the two-step
  executor, argv/stdin shape, and failure ordering.
* `auth::throttle_tests::{five_failures_lock_one_account_for_a_minute,
  a_correct_password_clears_the_failures,
  guessing_many_names_hits_the_daemon_wide_ceiling,
  the_account_table_stays_bounded,
  four_failures_export_and_the_next_attempt_locks_after_restore,
  restore_clamps_a_deadline_and_drops_expired_entries,
  clock_moving_forward_expires_a_loaded_lockout_and_starts_a_new_window,
  clock_moving_backward_caps_a_loaded_lockout_at_one_lockout_from_load}` — the
  policy with a supplied clock, no sleep, the clamp/restore rules, and both
  clock-change directions.
* `lockout::tests::{a_record_round_trips_through_the_atomic_file,
  a_damaged_file_is_an_error_not_a_panic,
  more_accounts_than_the_meter_holds_is_refused,
  an_oversized_file_is_refused_before_it_is_read}` — the durable file's format,
  atomic write, and bounds.
* `state::tests::{a_lockout_survives_a_restart,
  an_expired_lockout_is_dropped_when_it_is_loaded,
  a_damaged_lockout_file_fails_open_and_leaves_the_daemon_alive,
  a_forward_clock_jump_drops_a_loaded_lockout,
  a_backward_clock_jump_caps_a_loaded_lockout_at_one_minute}` — restart, expiry,
  fail-open, and both clock-change directions through the real file and dispatch
  path.
* `state::tests::failed_verifications_are_locked_out_before_the_helper_runs` —
  five rejections reach the runner, the sixth answers `ERR locked-out` without a
  sixth call, and the attempt after the window reaches the helper again.

### 72.5 Gates (real output)

* `cargo fmt --all --check` → 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → 0.
* `cargo test -p litric-sysd` → 245 passed, 0 failed (238 lib + 5 + 1 + 1).
* `cargo test --workspace` → **1,883 passed, 0 failed**.

### 72.6 Limits, stated plainly

* CI has no root, so the tests never execute `useradd`, `chpasswd` or
  `unix_chkpwd`; construction and ordering are proven, the privileged run is not.
* The confirmation may be omitted (short form, confirmation equals the password)
  or carried as its own hex field (long form, mismatch rejected). The short form
  cannot express a mismatch by construction, which is the point of allowing it.
* `AuthSession` is reused rather than adding `AuthCreate`/`AuthChange`; the id is
  already live-consent and already granted for `lock://`, and a new id would
  churn every exhaustive match for no extra authority.
* Requests have no serializer by design: clients compose the line and the daemon
  parses it, so no `Request::to_line` API was added. Stability is proven with a
  test-local canonical composer (compose → parse → compose) instead.
* Lockout counters survive a restart, but they are wall-clock timestamps: a
  forward clock jump can expire a lockout early and a backward jump can re-arm at
  most one configured lockout from load. Without a monotonic cross-restart
  reference the daemon cannot distinguish that from elapsed time. Up to 2 s of
  ordinary failures can be lost in an abrupt kill; a lockout starting or a
  success clearing one is written immediately. The counters are still not written
  to the audit log.
* The rate limit bounds guesses, not a compromised local account: a caller that
  can already reach the 0600 socket as the daemon's user does not need to guess
  a password to act as that user. Raising the socket's permissions is a separate
  decision.

### 72.7 Verification rate limit

`AUTH VERIFY` is metered after the capability check and before
`providers::perform`, so a refused guess costs no process:

* 5 failed verifications per account in 60 s locks that account for 60 s;
* 20 failures across all accounts in 60 s locks the whole path for 30 s;
* a locked request answers `ERR locked-out` with `retry_after=Ns`, never reaches
  `unix_chkpwd`, and records `rate-limited` in the audit ring (authorisation
  allowed, metering refused); the ring reaches disk on the next store save, not
  inside the refusal path;
* only `reason=exit` counts; spawn failures, timeouts and refusals do not;
* a correct password clears that account's failures, not the global counter;
* the account table is capped at 256 entries and never evicts a live entry to
  make room, so spraying names cannot shed a lockout; the global cap bounds
  guesses when it is full;
* the lock screen parses `retry_after` and closes the field for that long, so a
  one-minute daemon lockout is not cut short by the shell's one-second local
  backoff; a reply without the field falls back to the old behaviour;
* the counters are durable in `<state>.lockout` beside the grant store, written
  atomically (temp + `fsync` + `rename` + directory `fsync`) and at most once
  every 2 s for ordinary failures; a lockout starting or a success clearing one
  is written on that request, so a retry loop cannot become a disk-write loop;
* a missing file starts empty; a damaged, oversized or invalid one is logged and
  ignored (fail-open), never denying grants and never stopping the daemon;
* on load the count is capped at the configured limit, a lockout deadline is
  clamped to 60 s (account) or 30 s (global) from load time, a future window
  start becomes now, and at most 256 accounts are accepted; the file is capped
  at 64 KiB with 512-byte lines;
* a daemon restart no longer resets the meter; clock changes are handled
  explicitly on load: a forward jump consumes the stored lock and drops the
  record once its window is over, so the next failure starts at one; a backward
  jump pulls a future window start to load time and caps the deadline at 60 s
  (account) or 30 s (global) from load, so the skew can extend a wait but never
  become the wait.

### 72.8 Git

Base `79bbd58`; verification pass on `3628937`; rate limit on `acea817`;
integrated lockout on `4796a86`; durable lockout on `9ebedb8`. All are committed
on `main` and pushed; `HEAD == origin/main` and the tracked worktree is empty at
the end of the turn. The durable-lockout batch records 241 package tests and
1,879 workspace tests; the clock-skew tests raise that to 245 and 1,883.

## 74. Batch: update generations, boot counting and a demonstrated rollback (verified)

### 74.1 Requirement

Plan §74 asks for a recovery/update generation engine core with a demonstrated
rollback in a temp root, and for no generation to be advertised before it can
roll back. It was scheduled after §73 (the settings-family wave); this batch
runs it first because the backend is what several `DEFER` rows are waiting for,
and because the ordering rule is backend before surface. §73 is not claimed by
this entry.

### 74.2 The contract, and where it lives (file → item)

* `src/litric-update/src/engine.rs` — `Engine::init` creates one committed A
  generation; `stage` copies a closed source tree into the inactive slot and
  persists it staged; `note_boot` counts attempts and rejects a candidate after
  three failed boots; `commit` activates only after the caller's health checks;
  `discard_pending` is the cancel path; `rollback` marks the left generation bad
  and returns to the other committed one. `copy_tree` refuses symlinks, special
  files, entries past 100,000, bytes past 8 GiB and paths past 4,096 bytes.
* `src/litric-update/src/state.rs` — the bounded `litric-generations` record
  (active, pending, two slot records, eight events), strict parser, `0600` atomic
  write (`fsync` + rename + directory `fsync`), marker read/write, and the
  `MAX_BOOT_ATTEMPTS = 3` / `MAX_EVENTS = 8` / 16 KiB / 512-byte bounds.
* `src/litric-update/src/fsutil.rs` — the single removal choke point for this
  crate's own staging directory, slot directory and state temporary. Every path
  is under `<root>/system`; no user path is accepted.
* `src/litric-update/src/lib.rs` — the public API and structured `UpdateError`;
  no new third-party dependency, only `litric-core` for `Version` ordering.
* `src/Cargo.toml` — the crate is a workspace member; `Cargo.lock` records the
  one workspace edge.
* `ci/lint-file-safety.py` — records the review of the new deletion site and, as
  a gate repair, the two existing lockout-store temporary removals that had no
  review entry.
* `docs/design/2026-10-30-update-generations.md` — the design note, crash
  ordering, exact state machine, tests and limits.

### 74.3 Behavior pinned by the tests

* A candidate is copied and committed without changing the active slot before
  `commit`; after commit the previous generation remains a rollback target.
* Three boots without a commit are allowed; the fourth `note_boot` rejects the
  candidate, keeps the active slot, and marks the candidate bad.
* `rollback` returns to the other committed slot, records slot/reason/user-data,
  and a canary in `<root>/user` is byte-identical afterwards.
* Anti-downgrade is strict (`candidate > active`), before state or slot writes.
* A missing/mismatched marker cannot be committed or rolled back to.
* A symlink in the source, a corrupt state, a state overwrite, an entry/byte
  cap and a second pending candidate are all refused without inventing a slot.
* The state file is `0600`; ten updates leave at most eight switch events.

### 74.4 Tests added

* `tests::a_candidate_is_staged_copied_and_committed_without_touching_the_active_slot`
* `tests::three_failed_boots_discard_the_candidate_and_keep_the_active_generation`
* `tests::rollback_switches_to_the_previous_committed_slot_and_leaves_user_data_alone`
* `tests::a_downgrade_is_refused_before_the_state_or_a_slot_is_written`
* `tests::a_second_stage_requires_the_first_candidate_to_finish`
* `tests::the_boot_budget_is_a_boundary_not_an_accumulator`
* `tests::a_symlink_in_the_source_is_refused_and_the_active_slot_stays`
* `tests::a_missing_marker_is_never_advertised`
* `tests::a_corrupt_state_is_refused_and_left_byte_for_byte`
* `tests::limits_stop_a_candidate_before_it_is_published`
* `tests::init_refuses_to_replace_an_existing_state`
* `tests::the_state_file_is_owner_only`
* `tests::the_switch_history_stays_bounded_across_many_updates`
* `tests::rollback_without_a_second_generation_is_refused`
* `state::tests::{the_record_round_trips_through_the_format,
  a_damaged_record_is_refused_not_guessed, the_switch_history_is_bounded,
  an_oversized_line_is_refused_before_parsing,
  rollback_requires_a_second_committed_generation}`

### 74.5 Gates (real output, 2026-10-30)

* `cargo fmt --all --check` → 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → 0.
* `cargo test -p litric-update` → **19 passed, 0 failed**.
* `cargo test --workspace` → **1,902 passed, 0 failed** (1,883 + 19).
* `python3 ci/count-scope.py` → 1,165 rows: **9 D · 119 P · 291 S · 746 T**.
* `python3 ci/lint-parity.py` → 391 rows, 597 research items.
* `python3 ci/lint-doc-refs.py` → 1,284 citations across 158 documents.
* `python3 ci/lint-line-endings.py` → 380 files, all LF.
* `python3 ci/lint-file-safety.py` → 120 deletion sites, all reviewed.
* `bash ci/verify.sh` → **34 pass, 0 fail, 3 skipped**. The skips are the image
  build (mkosi absent), boot smoke (QEMU absent) and idle RAM/CPU/boot timing
  (same). No failed gate is hidden by the skips.
* Development keys were generated locally with `tools/gen-dev-keys.sh` so the
  mkosi schema linter and key-pair check could run; `keys/dev` is ignored and
  was not committed.

### 74.6 Limits, stated plainly

* This is a core library, not the updater service. No image was built or booted;
  no bootloader reads the state file; no EROFS/verity/ESP write exists.
* `commit` assumes the caller performed the session, storage and critical-service
  health checks. Those checks are not implemented here.
* The engine does not verify signatures, hashes or a signed monotonic counter.
  The caller must do that before `stage`; a valid local upgrade can still be a
  replay of an older signed image.
* No power-cut test is possible in CI. The ordering and fsyncs are designed for
  it, but only the atomic-write and parser behavior are executed.
* One writer is assumed; there is no cross-process lock. The future service unit
  is the serialization point.
* `SYSTEM-SCOPE` rows 760 and 762 moved to `P` because real staged-write,
  marker-verification and boot-count logic exists with passing tests. They do not
  mean the A/B partition or bootloader integration is done.

### 74.7 Git

Base `9d3d22c`; this batch is committed on `main` with the code, design note,
scope update, file-safety review and this entry. The final hash and
`HEAD == origin/main` check are recorded in the delivery report after the push;
the tracked worktree is empty at the end of the turn.

## 75. Batch: the update service — boot counting, status and health-gated activation (verified)

### 75.1 Requirement

§74 left the generation engine as a library. This batch wires it into the
running system: the real `litric-sysd` boot unit counts an attempt before the
socket exists, the daemon exposes the state over its existing IPC, and the
existing capability broker gates staging, commit and rollback. It does not start
§73, build an updater UI, or claim a bootloader/partition integration.

### 75.2 The contract, and where it lives (file → item)

* `src/litric-sysd/src/update.rs` — `UpdateService` owns one configured root;
  `boot` is the `--boot` entrypoint; `status` reports `disabled`,
  `uninitialized`, `corrupt`, `incomplete` or `ready`; `stage` accepts only a
  relative tree under `<root>/incoming`; `commit` consumes the caller's typed
  `HealthSignal`; `rollback` moves the pointer and records `manual`.
* `src/litric-sysd/src/state.rs` — the service is a field of `Sysd`; three
  `update.control` scopes are policy-granted to the local client and checked by
  `check_with_consent` before any engine call. `PROVIDER_GRANTS` now has nine
  entries; the existing dynamic grant-count test covers it.
* `src/litric-sysd/src/proto.rs` — `UPDATE STATUS`, `UPDATE STAGE`,
  `UPDATE COMMIT` and `UPDATE ROLLBACK`, with a required `CONFIRM` word and a
  typed `PASSED|FAILED` health field. `schema=1` is stated in every status.
* `src/litric-sysd/src/main.rs` — `--boot` and `--update-root`; `--serve`
  attaches the same root.
* `src/litric-update/src/engine.rs` — `verify_generation`, a read-only marker
  check used by status; `UpdateError::kind` supplies stable protocol tokens.
* `src/litric-caps/src/capability.rs` — `update.control`, its `update://`
  scheme, high risk, live-consent requirement and three scopes.
* `image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service` — `--boot`
  runs as `ExecStartPre`; `/var/lib/litric/update` is opened for writing.
* `image/mkosi.extra/usr/lib/tmpfiles.d/litric-update.conf` — creates the
  `0700` root and `incoming` directory before the mount namespace is built.
* `src/litric-sysd/tests/update_service.rs` — the real binary and socket.

### 75.3 Behavior pinned by the tests

* A boot with a staged candidate prints `attempt=1`; three attempts are allowed
  and the fourth rejects the candidate. The active slot remains A, the candidate
  is `none`, `last_result=reject:boot-count`, and `<root>/user` is unchanged.
* `UPDATE STATUS` over the socket reports schema 1, active/candidate slots,
  attempt count, rollback availability, last result and last rollback reason.
* `UPDATE COMMIT PASSED CONFIRM` activates B; `UPDATE ROLLBACK CONFIRM` returns
  to A with `reason=manual`; the user-data canary is byte-identical.
* `UPDATE COMMIT FAILED CONFIRM` is refused and leaves the candidate staged.
* A missing grant record produces `ERR refused` with `no-grant` or
  `unknown-application`; the engine is not called.
* A corrupt state answers `state=corrupt` with a reason, and a missing active
  slot answers `state=incomplete`; neither panics.
* Absolute and `..` source paths are refused before staging.

### 75.4 Tests added

* `update::tests::{status_before_any_state_is_uninitialized_and_actions_refuse,
  corrupt_state_is_reported_and_mutations_are_refused,
  a_failed_health_signal_never_commits,
  the_service_rolls_back_and_leaves_user_data_byte_for_byte,
  boot_budget_exhaustion_returns_to_the_active_generation,
  a_missing_slot_directory_is_incomplete_not_a_panic,
  staging_cannot_escape_the_incoming_directory}`
* `state::tests::update_actions_need_consent_and_a_scope_that_survives_revocation`
* `proto::tests::{update_requests_parse_with_an_explicit_health_and_confirmation,
  an_update_without_confirmation_or_with_a_bad_version_is_refused}`
* `capability::tests::update_control_is_scoped_to_one_operation_and_needs_consent`
* `src/litric-sysd/tests/update_service.rs` (4 process/socket tests)

### 75.5 Gates (real output, 2026-10-30)

* `cargo fmt --all --check` → exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
* `cargo test -p litric-update` → **19 passed, 0 failed**.
* `cargo test --workspace` → **1,917 passed, 0 failed** (62 test binaries).
* `python3 ci/count-scope.py` → 1,165 rows: **9 D · 120 P · 290 S · 746 T**.
* `docs/SYSTEM-SCOPE.md` row 764 moves to `P`; row 763 (health checks),
  row 181 (boot-menu counting) and the bootloader/recovery rows stay `S`
  because no probe, no bootloader and no recovery environment exists.
* `python3 ci/lint-doc-refs.py` → 1,330 citations across 159 documents resolve.
* `python3 ci/lint-line-endings.py` → 384 files, all LF.
* `python3 ci/lint-file-safety.py` → all deletion sites reviewed, including the
  four process-test cleanup sites this batch adds.
* `bash ci/verify.sh` → **34 pass, 0 fail, 3 skipped**. The skips are the image
  build (mkosi absent), boot smoke (QEMU absent) and idle RAM/CPU/boot timing
  (same). No failed gate is hidden by the skips.

### 75.6 Limits, stated plainly

* The attempt is counted per `litric-sysd` start, not per kernel boot. No
  bootloader reads the record; no partition is written.
* The budget path is an automatic return to the already-active generation. It
  is not a call to `Engine::rollback`, because the candidate never held the
  active pointer. `UPDATE ROLLBACK` is the real pointer move.
* `COMMIT PASSED` is a caller assertion. Session, storage and critical-service
  probes are still absent; `SYSTEM-SCOPE` row 763 stays `S`.
* No signature/hash/monotonic check, no EROFS/verity/ESP writer, no power-cut
  test and no UI. There is no cross-process lock; systemd orders `--boot`
  before `--serve`, which is the serialisation this batch relies on.

### 75.7 Git

Base `278639b`; this batch is committed on `main` with the service, its tests,
the design note, scope/plan/tracker/GATES updates. The final hash and
`HEAD == origin/main` check are recorded in the delivery report after the push.

## 76. Batch: crash-consistent update state and one writer per root (verified)

### 76.1 Requirement

§75 made the generation engine reachable from `litric-sysd --boot` and the socket, but left two gaps: a transition was not written through a temporary+fsync+rename+directory-fsync sequence, and two processes could interleave a read-modify-write of one root. This batch owns only those gaps. It does not start §73 or any UI, and adds no bootloader, partition writer, image format, signature check or real health probe.

### 76.2 The contract, and where it lives (file → item)

* `src/litric-update/src/state.rs` — sibling temp `.state.tmp-<pid>` → `sync_all` → rename over `<system>/state` → directory fsync. The temp is never parsed; the post-read length check closes the metadata/read race. Zero, truncated and unknown records stay `InvalidState`.
* `src/litric-sysd/src/update_lock.rs` — `update.lock` via `create_new`, mode `0600`, header `litric-update-lock\t1`, then `pid`/`at`/`reason`. Writers wait at most 2 s; status makes one non-blocking attempt. A dead PID is reclaimed; a malformed record younger than 2 s is initialising; a live PID is never evicted. `Drop` removes only the exact record it wrote.
* `src/litric-sysd/src/update.rs` — `UpdateService` no longer caches an engine. Boot, stage, commit and rollback take the root lock and reopen the record under it; status reads under a non-blocking lock snapshot and reports `busy` instead of waiting. `UpdateFailure` adds typed `locked`/`lock-io`.
* `src/litric-sysd/src/main.rs` — `--boot` exits `ExitCode::FAILURE` when the lock is unavailable; a corrupt record still exits 0 and reports it.
* `src/litric-sysd/src/proto.rs` — `UPDATE STATUS [schema]`; schema 1 and 2 are served, 0 and >2 are `ERR schema-unsupported`, and schema 2 adds the busy/lock fields.
* `src/litric-sysd/src/state.rs` — status authorises no mutation and forwards the optional schema; no `update.control` capability is needed to ask.
* `ci/lint-file-safety.py` — records the two reviewed deletion sites: lock release and stale reclaim.
* `src/litric-sysd/tests/update_service.rs` — real binary + socket tests for concurrent and interrupted writes.

### 76.3 Behavior pinned by the tests

* Two `--boot` processes started together both exit 0 and the candidate reaches `candidate_attempts=2`; no increment is lost.
* A live PID's lock makes a second writer exit non-zero with `locked by pid`, while `UPDATE STATUS 2` answers `state=busy` with pid, reason, timestamp and `retry=lock` in under one second.
* A dead-PID lock is reclaimed; the next boot prints `attempt=1` and removes the stale file.
* A leftover `.state.tmp-*` is ignored (`state=ready`). Truncated and zero-length primaries answer `state=corrupt` over the socket; `--boot` reports `litric.update.state=corrupt` and exits 0.
* The §75 stage → boot → commit → rollback path still passes with the `<root>/user` canary byte-identical.
* Bare `UPDATE STATUS` still works; `UPDATE STATUS 2` echoes `requested_schema=2`; `UPDATE STATUS 99` is `ERR schema-unsupported`.
* Read precedence: disabled → root absent → busy (live lock) → unavailable (lock I/O) → state absent → corrupt (symlink/oversize/non-UTF-8/zero/truncated) → incomplete (slot/marker missing) → ready. The design note records the full table.

### 76.4 Tests added

* `update_lock::tests::{a_live_lock_refuses_a_second_holder_and_releases_on_drop, a_dead_pid_is_reclaimed, a_malformed_young_lock_is_not_evicted_during_creation, the_wait_is_bounded}`
* `litric-update::tests::{a_leftover_state_temporary_is_never_parsed_as_state, truncated_and_zero_length_state_is_refused_without_a_panic}`
* `src/litric-sysd/tests/update_service.rs::{two_concurrent_boot_processes_each_count_one_attempt, a_live_writer_makes_status_busy_and_a_second_writer_fail_typed, a_stale_dead_pid_lock_is_reclaimed_by_the_next_boot, interrupted_writes_are_ignored_or_degrade_without_a_panic}`
* Existing §75 tests were kept; only the schema assertion changed.

### 76.5 Gates (real output, 2026-10-30)

* `cargo fmt --all --check` → exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
* `cargo test -p litric-update` → **21 passed, 0 failed**.
* `cargo test --workspace` → **1,927 passed, 0 failed** (62 test binaries).
* `python3 ci/count-scope.py` → 1,165 rows: **9 D · 121 P · 289 S · 746 T**.
* `docs/SYSTEM-SCOPE.md` row 769 moves `S` → `P` for the tested state-record ordering; the image/partition/bootloader stages it also names remain unimplemented, so it is partial, not done.
* `python3 ci/lint-doc-refs.py` → 1,352 citations across 159 documents resolve.
* `python3 ci/lint-line-endings.py` → 385 files, all LF.
* `python3 ci/lint-file-safety.py` → 125 deletion sites, all reviewed.
* `bash ci/verify.sh` → **34 pass, 0 fail, 3 skipped**. The skips are image build (mkosi absent), boot smoke (QEMU absent) and idle RAM/CPU/boot timing (same). No failed gate is hidden by the skips.

### 76.6 Limits, stated plainly

* No bootloader or partition integration: the state file is still not read by systemd-boot, a UKI or the ESP. The attempt is per `litric-sysd` start.
* The atomic ordering covers the generation record, not slot trees, an EROFS image or verity. There is no power-cut matrix or actual power-cycle test; tests simulate a leftover temp and truncated/zero-length records.
* `COMMIT PASSED` remains the caller's typed assertion. No session, storage, critical-service or real health probe exists; row 763 stays `S`.
* No signature, hash, monotonic counter, EROFS/verity or ESP writer; rows 767, 768 and 770 stay `S`.
* No UI consumes the status; §73 was not started. The Research-Deep-all corpus and the whole system are not finished.
* On a platform without `/proc`, a complete lock record is fail-closed as live; Linux, the target, uses `/proc`.

### 76.7 Git

Base `1e9b64d`; this batch is committed on `main` with the lock module, service/protocol/state changes, tests, design note and scope/plan/tracker/GATES updates. The final hash and `HEAD == origin/main` check are recorded in the delivery report after the push.

## 77. Batch: boot-attested health evidence and a commit gate (verified)

### 77.1 Requirement

§76 made the state record crash-consistent and serialised writers, but a commit still accepted the caller's `COMMIT PASSED` word as if it were health evidence. This batch owns one outcome: the running system writes a durable health record bound to the candidate generation and attempt, and `UPDATE COMMIT` refuses to activate without a matching `healthy` record. It does not start §73/UI, and adds no bootloader, partition, EROFS, verity, ESP, signature or anti-downgrade work.

### 77.2 The contract, and where it lives (file → item)

* `src/litric-sysd/src/update_health.rs` — typed `HealthEvidence` (`generation`, `version`, `attempt`, `at`, `verdict`, `checks`), a 1 KiB bounded parser, total `read` (`Missing`/`Valid`/`Invalid`) and an atomic write through `litric_update::write_atomic`. Symlinks, non-regular files, zero/oversized/non-UTF-8/malformed records and trailing lines are invalid, never a panic. A `.health.tmp-*` is never read.
* `src/litric-update/src/fsutil.rs` / `lib.rs` — the §76 sibling-temp + `sync_all` + rename + directory-fsync primitive is now public as `litric_update::write_atomic`; `Store::save` uses the same helper.
* `src/litric-sysd/src/update.rs` — `SCHEMA=3`; `HealthError` kinds `health-missing`, `health-not-booted`, `health-stale`, `health-invalid`, `health-failed`, `health-unattested`, `health-io`; `commit` opens the engine under the root lock, requires `healthy` evidence for the exact pending slot/version/attempt, and calls `Engine::commit` only then; `boot` writes `booting` after `note_boot`; `attest_serving` upgrades only a matching `booting` record; status reports the health summary for schema 3.
* `src/litric-sysd/src/server.rs` — after binding the socket, the serve loop calls attestation once before accepting. A failure is logged, not fatal, and the gate stays closed.
* `src/litric-sysd/src/state.rs` — `attest_update_health`; status passes the optional schema through and still authorises no mutation.
* `src/litric-update/src/engine.rs` — `reject_pending(reason, now)` for the failed-evidence path; it clears the candidate without moving the active pointer.
* `src/litric-sysd/tests/update_service.rs` — real binary/socket tests and a raw health-record helper; no struct-level shortcut for IPC cases.

### 77.3 Behavior pinned by the tests

* Boot attempt 1 writes `booting`; once the real serve loop is up, status says `health_evidence=healthy`, `health_slot=B`, `health_version=1.1.0`, `health_attempt=1`, `health_matches=1`. `COMMIT PASSED CONFIRM` then activates B and the `<root>/user` canary is byte-identical.
* No boot → `ERR health-not-booted`; a removed record with only a temp present → `ERR health-missing`. State bytes are unchanged in both cases.
* Evidence for slot A and evidence for attempt 2 are each `ERR health-stale`; the error names the found and expected attempts. State bytes are unchanged.
* A system-written `failed` record refuses the commit with `ERR health-failed`; the next `--boot` reports `outcome=rejected-health`, `rollback=active-generation`, clears the candidate and records `last_result=reject:health-check`.
* Zero-length, truncated and temp-only evidence degrade to `invalid`/`absent` and typed commit refusals without a panic; state bytes stay unchanged.
* Replaying attempt 1's healthy record after boot attempt 2 is `ERR health-stale`; the attempt counter is part of the evidence identity.
* A boot and a commit started together produce exactly one valid serialised outcome, and status still reports `state=ready`. Boot and commit share the §76 root lock; serve attestation uses one non-blocking attempt so it cannot delay startup.
* Schema 3 adds the health fields; schema 2 answers omit them; schema 0 and >3 remain `ERR schema-unsupported`.

### 77.4 Tests added

* `update_health::tests::{evidence_round_trips_and_is_bound_to_all_three_fields, a_truncated_empty_or_overlong_record_is_invalid_not_a_panic, a_temporary_is_never_read_as_evidence, save_is_atomic_owner_only_and_visible_to_read}`
* `src/litric-sysd/tests/update_service.rs::{commit_without_a_boot_attempt_is_refused_before_any_state_change, missing_evidence_after_a_boot_is_refused_and_the_temporary_is_not_evidence, evidence_for_the_other_generation_is_stale_and_cannot_commit, evidence_for_another_attempt_is_stale_and_cannot_commit, failed_evidence_refuses_commit_and_the_next_boot_rejects_the_candidate, truncated_zero_and_temp_only_health_evidence_degrade_without_a_panic, a_prior_attempts_healthy_evidence_cannot_be_replayed_for_a_new_attempt, a_boot_and_a_commit_cannot_interleave_under_the_root_lock}`
* The existing §75/§76 tests remain; the only assertion changed is the schema number and the added healthy-summary check in the real stage → boot → commit → rollback test.

### 77.5 Gates (real output, 2026-10-30)

* `cargo fmt --all --check` → exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
* `cargo test -p litric-update` → **21 passed, 0 failed**.
* `cargo test -p litric-sysd` → **279 passed, 0 failed** across its unit/integration binaries (16 in `update_service`).
* `cargo test --workspace` → **1,939 passed, 0 failed** (62 test binaries).
* `python3 ci/count-scope.py` → 1,165 rows: **9 D · 122 P · 288 S · 746 T**.
* `docs/SYSTEM-SCOPE.md` row 763 moves `S` → `P`. Row 1147 stays `S`: session, storage and critical-service probes do not exist.
* `python3 ci/lint-doc-refs.py` → 1371 citation(s) across 159 document(s) resolve.
* `python3 ci/lint-line-endings.py` → 386 files, all LF.
* `python3 ci/lint-file-safety.py` → 125 deletion site(s), all inside reviewed functions.
* `bash ci/verify.sh` → **34 pass, 0 fail, 3 skipped**. Skips: image build (mkosi absent), boot smoke (QEMU absent), idle RAM/CPU/boot timing (same). No failure is hidden by a skip.

### 77.6 Limits, stated plainly

* No bootloader/UKI/ESP integration and no partition/EROFS/verity/signature/anti-downgrade writer. The attempt is still per `litric-sysd` start, not a kernel boot.
* `healthy` is not a kernel/network/display/session/storage/critical-service probe. It means the boot entrypoint opened the record and verified the candidate marker, then the daemon bound the real socket and re-verified the same slot/version/attempt in its serve loop.
* Attestation is one non-blocking attempt; if a writer owns the root at that instant, the gate stays closed until a later boot/attestation. It never waits on the lock during startup.
* No power-cut matrix; the health record uses the tested atomic ordering, and corrupt/temp-only inputs are simulated tests.
* No cryptographic authentication of the health record: it lives in the root-owned update root and only `litric-sysd` writes it in normal operation, but a root process can write the file. The gate stops a caller assertion; it is not a defence against a compromised root.
* No UI consumes the status; §73 was not started. The Research-Deep-all corpus and the whole system are not finished.

### 77.7 Git

Base `df3a4e9`; this batch is committed on `main` with the health module, the service/server/engine changes, the atomic-helper reuse, real-socket tests and the design/scope/plan/tracker/GATES updates. The final hash and `HEAD == origin/main` check are recorded in the delivery report after the push.

## A2. Batch: the performance regression harness can fail (verified)

The first Stage A row, **A1**, is still open: it needs a builder with `mkosi`, QEMU and a booted desktop plus a real budget snapshot, none of which exists in this environment. It was not attempted or marked landed. A2 is the next feasible Stage A batch, taken under the plan's feasibility rule.

### A2.1 Requirement

The W9 acceptance sentence said a planted 15 % regression must fail the performance harness, but `render_bench` only printed medians; nothing compared runs. This batch owns that single gap. It does not add a new benchmark, change the renderer, measure a machine, or start zram/MGLRU/oomd work. It is the Stage A2 batch from the master plan, not a UI batch.

### A2.2 The contract, and where it lives (file → item)

* `ci/check-perf-regression.py` — parses the raw `key=value` output of two `render_bench` runs, takes every `*.median_ms` and matching `*.runs`, requires at least 15 runs, and fails at or above the threshold relative to the baseline. Missing scenes, duplicate/non-finite/negative metrics and short samples fail; a new candidate scene is printed as `NEW` without a comparison. The default threshold is the 10 % rule in `docs/PERF-budget.md`; `--threshold 15` is the W9 acceptance boundary.
* `ci/verify.sh` — adds `[3k] Performance regression harness`, running only the self-test. The real comparison needs two controlled runs and is not run on every CI pass.
* `docs/design/2026-10-30-performance-regression-harness.md` — input contract, read-precedence table, exact command, self-test cases and limits.
* `docs/PERF-budget.md` §8 — the command and the 10 %/15 % distinction.
* `docs/master-plan/28-whole-system-improvement-plan.md` — Stage A2 marked landed.

### A2.3 Behavior pinned by the self-test

* An unchanged run passes.
* A planted **+15.0 %** regression fails, with both medians and the exact percentage printed.
* **+14.9 %** passes, so the boundary is not a blanket rejection.
* A scene present in the baseline but absent from the candidate fails.
* A 14-run scene fails; the comparator does not silently compare a short sample.
* A duplicate metric and a negative median fail.
* The CLI was also run against temporary fixture files and exited 1 for the +15.0 % case, proving the file-reading path, not only the in-memory helper.

### A2.4 Tests added

* `ci/check-perf-regression.py --self-test` — seven fixture cases, including the exact 15 % boundary.
* `ci/verify.sh` `[3k]` — the negative control runs in the repository gate.

### A2.5 Gates (real output, 2026-10-30)

* `cargo fmt --all --check` → exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
* `cargo test -p litric-update` → **21 passed, 0 failed**.
* `cargo test --workspace` → **1,939 passed, 0 failed** (62 test binaries; no Rust behavior changed in A2).
* `python3 ci/check-perf-regression.py --self-test` → **PASS**; the planted +15.0 % case exited 1 with `+15.00% >= 15.00%` when run through the CLI.
* `bash ci/verify.sh` → **35 pass, 0 fail, 3 skipped**. The three skips are image build (`mkosi` absent), boot smoke (QEMU absent) and idle RAM/CPU/boot timing (same); no failure is hidden by a skip.
* `python3 ci/count-scope.py` → 1,165 rows: **9 D · 122 P · 288 S · 746 T**; **no row moved** because A2 is a verification harness, not a user capability.
* `python3 ci/lint-doc-refs.py` → 1395 citation(s) across 160 documents resolve.
* `python3 ci/lint-line-endings.py` → 387 files, all LF.
* `python3 ci/lint-file-safety.py` → 125 deletion sites, all inside reviewed functions.

### A2.6 Limits, stated plainly

* This does not close W9: no zram/MGLRU/oomd policy, no boot/idle measurement and no image were added.
* It does not prove a real machine is inside the 16.67 ms frame budget. It compares a candidate run with a recorded baseline; an already-over-budget scene can pass if it did not regress.
* The baseline and candidate must be produced on the same host, release build, canvas and workload. The script cannot verify those facts; that obligation is documented.
* A self-test PASS proves the comparator can fail, not that any candidate is fast. CI runs only the self-test; two real benchmark runs are required for a real verdict.
* No new scene is fail-closed: it is printed `NEW` because there is no baseline to compare. Re-record the baseline deliberately when coverage changes.
* No UI/§73 work and no change to the renderer or paint path.

### A2.7 Git

Base `8d704d0`; this batch is committed on `main` with the comparator, its CI negative control, the design note, PERF-budget and tracker/GATES/plan updates. The final hash and `HEAD == origin/main` check are recorded in the delivery report after the push.

## A3. Batch: the absent-backend claims are retagged (verified)

A1 remains open: it needs a builder with `mkosi`, QEMU, a booted desktop and a
real budget snapshot, none of which exists here. A2 is landed. A3 is the next
feasible Stage A batch. It changes no executable behavior; it removes five
claims whose named execution backend is absent and records the evidence.

### A3.1 Requirement

The master plan's A3 scope is: every `P` row whose backend is absent is
re-tagged, with the reason in the same commit. `P` is not a promise of
completeness; it means partial, real code with passing tests. The audit
therefore asks one question only: does the row name an execution backend that
does not exist at all?

### A3.2 Method and coverage

* Enumerated all **122** `P` rows at base `ca4e7c1`.
* Read each row's claim, then looked for its named binary, crate, unit, socket,
  state transition or test in `src/`, `image/` and the crate test suites.
* Multi-cluster review covered the shell, design, trash, capability, Winbridge,
  environment, update, browser and Defender code.
* A row stayed `P` when its model/library and tests exist and the remaining work
  is a separate integration point. A row moved only when the claimed backend is
  absent, not merely incomplete.
* The full audit note is `docs/audit/2026-10-30-a3-claim-audit.md`.

### A3.3 Rows retagged `P` → `S`

| Row | Claim | What is actually absent | Evidence |
|---:|---|---|---|
| 476 | Capability broker daemon (`litric-capd`) | The daemon binary, crate and socket implementation; only parked unit files exist | `image/units-pending/litric-capd.service` names `/usr/bin/litric-capd`; no such binary or crate exists; ADR-0016 puts the broker in `litric-sysd` |
| 843 | `.exe` double-click → installed, isolated, uninstallable app | The executor for the install plan | `src/litric-winbridge/src/lib.rs` says it does not execute, spawn or write; no `litric-compatd` exists |
| 844 | Per-app disposable Windows environment | Create/start/reset/delete execution | `Bottle` and `Isolation` are typed models with tests; no bottle runtime exists |
| 863 | Host CPU/memory reservation enforced against guests | VM/cgroup/KVM enforcement | `litric-env` checks budgets in its model; `litric-envd` is row 876, still `T` |
| 947 | Demand-based memory sharing between environments | Any page-sharing/ballooning/KSM/zram/rebalancing path | No such code path exists; `litric-env` only tracks committed budgets and hibernation state |

### A3.4 What deliberately did not move

* Rows 760, 762, 763, 764 and 769 have real update staging, marker, boot-count,
  health-evidence and atomic-record code with tests. Their bootloader,
  partition, EROFS, verity and power-cut limits remain recorded gaps, not absent
  code.
* Row 551 has a real `browserd` that spawns Chromium and drives CDP; it reports
  an honest unavailable state when Chromium is absent.
* The capability rows other than 476 have real broker, grant, revocation and
  traversal tests. The separate `capd` daemon is the absent part.
* Rows 860–862 and 864–875 are the environment state model with tests; VM
  lifecycle execution is already row 876 `T`.
* Row 571 has real process/CPU/memory monitor code. Any missing columns are
  incompleteness, not an absent backend.

No row was promoted to `P`; A3 is audit-only.

### A3.5 Verification (real output, 2026-10-30)

* `python3 ci/count-scope.py` → 1,165 rows: **9 D · 117 P · 293 S · 746 T**.
* `python3 ci/lint-doc-refs.py` → **1,408 citations across 161 documents** resolve.
* `python3 ci/lint-line-endings.py` → **387 files, all LF**.
* `python3 ci/lint-file-safety.py` → **125 deletion sites**, all reviewed.
* `cargo fmt --all --check` → exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
* `cargo test -p litric-update` → **21 passed, 0 failed**.
* `cargo test --workspace` → **1,939 passed, 0 failed** (62 test binaries).
* `bash ci/verify.sh` → **35 pass, 0 fail, 3 skipped**. The skips are image
  build (`mkosi` absent), boot smoke (QEMU absent) and idle RAM/CPU/boot timing
  (same).

A3 adds no Rust test and no new gate because it adds no executable behavior.
The negative-control requirement does not apply: there is no new check that
could pass on a planted violation. The existing A2 self-test still runs.

### A3.6 Limits

* This is a source/test audit, not a runtime proof. A module can exist and still
  be wrong; the existing suites and gates remain the evidence for behavior.
* It does not claim the remaining 117 `P` rows are complete. It only establishes
  that their named backend is not wholly absent.
* It does not close A1, W9, bootloader/UKI/partition/EROFS/verity/ESP/signature,
  anti-downgrade, kernel/network/display/session probes or the UI/§73 work.
* It does not complete the Research-Deep-all corpus.

### A3.7 Git

Base `ca4e7c1`. This batch changes the audit note, the ledger and its tally, the
master-plan marker and the tracker/GATES entries. The final hash and
`HEAD == origin/main` check are recorded in the delivery report after the push.
