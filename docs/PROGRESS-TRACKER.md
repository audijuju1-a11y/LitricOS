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
`src/litric-shell/src/lib.rs` and `src/main.rs` are `#![forbid(unsafe_code)]`,
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
