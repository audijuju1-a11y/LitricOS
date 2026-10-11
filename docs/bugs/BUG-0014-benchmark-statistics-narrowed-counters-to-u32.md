# BUG-0014 — The benchmark narrowed 64-bit counters and latencies to 32 bits

```
Bug ID        : BUG-0014
Severity      : S3 (MEDIUM) — a plausible-looking number that no sample produced
                is exactly what this harness exists to prevent
Class         : COUNTER / MEASUREMENT INTEGRITY
Subsystem     : src/litric-bench/src/severity.rs, src/litric-bench/src/scan.rs,
                src/litric-bench/src/report.rs
First Observed: 2026-10-09, source inspection of the scoreboard rates and the
                latency profile
```

## Reproducer

```bash
grep -n "as u32" src/litric-bench/src/severity.rs src/litric-bench/src/scan.rs
sed -n '/pub fn percentile/,/^    }/p' src/litric-bench/src/scan.rs
sed -n '/pub fn detection_rate/,/^    }/p' src/litric-bench/src/severity.rs
```

## Actual

**1. The rates narrowed their operands.**

```rust
f64::from(self.true_positives as u32) / f64::from(classified as u32)
```

`true_positives`/`false_negatives`/`false_positives`/`true_negatives` are `u64`,
so any counter above 4 294 967 295 wraps to a smaller number and the ratio is
silently wrong. The same file's own header says the rate math "saturates by
contract"; wrapping is not saturating, and no run is currently that large — the
defect is that the answer would have been wrong precisely in the extreme runs the
harness is built for, without any sign that it was.

**2. The percentile clamped real samples and substituted an invented one.**

```rust
let a = f64::from(u32::try_from(sorted.get(lo).copied().unwrap_or(u64::MAX_u32…)).unwrap_or(u32::MAX));
```

Every sample was routed through `u32`, so a latency above ~71.6 minutes became
4 294 967 295 µs, and the same constant was used as the fallback for an index the
code cannot actually produce. An out-of-range `p` (or `NaN`) made `lo`/`hi` read
past the vector; the caller got 4 294 967 295 back, indistinguishable from a
measurement. A saturation constant standing in for a missing value is the failure
mode this benchmark's own vocabulary (severity.rs) is written to avoid: it reports
`ENVIRONMENT_LIMITATION` rather than a pass, and it must report "no sample" rather
than a latency.

**3. The headline rate did not state its denominator.** `REPORT.md` printed
`detection rate` as `TP/(TP+FN)`, excluding hostile samples that never reached a
classification from both the numerator and the denominator, and the label did not
say so. Hostile samples that were `unsupported` or operational therefore shrank
the population rather than the score. (`REPORT.md` does print the counters, so the
number was reconcilable by hand; the label was still making a claim the arithmetic
did not.)

## Fix

* `detection_rate` / `false_positive_rate` convert the `u64` counters straight to
  `f64` (the same form `report.rs::rate` already used).
* `percentile` rejects `p` outside 0..=100 (including `NaN`) with 0, and its
  fallback for an impossible index is the slowest *real* sample, not a constant.
  No `u32` narrowing anywhere in the path.
* `REPORT.md` labels both rates with the population they are computed over
  ("of N classified hostile samples"), computed with `saturating_add`.
* Two new accessors make the invariant expressible: `Scoreboard::recorded()`
  (every sample counted exactly once) and `classified_hostile()`.

## Regression Test

Added, **not executed** (`src/litric-bench/src/severity.rs`, `src/litric-bench/src/scan.rs`):

* `rates_are_not_narrowed_to_u32` — counters of 5 000 000 000 must give exactly
  0.5. This test **fails against the old code** (the narrowed ratio is ≈0.14), so
  it is a real negative control for the fix.
* `percentile_is_not_clamped_to_the_u32_ceiling` — a 5 000 000 000 µs sample must
  come back as itself. Also fails against the old code.
* `an_out_of_range_percentile_reports_nothing` — `NaN`, `-1.0`, `101.0` → 0, and
  99th percentile of three samples is the maximum.
* `every_truth_and_outcome_lands_in_exactly_one_counter` — all 2 × 10
  (truth, outcome) pairs: `recorded() == 20` with the expected per-quadrant split.
  This is the invariant Workstream C asks for: every sample reaches exactly one
  terminal state, so `attempted = TP + FN + FP + TN + unsupported + operational`.

## Verification

| Check | Result |
|---|---|
| `rustfmt --check --edition 2021` on the three files | **PASS** (exit 0) |
| `bash ci/verify.sh` (cargo hidden) | permitted subset **PASS**, Rust stage **SKIPPED** |
| Rust test execution | **NOT RUN** — prohibited |

## Residual Risk

* The rate counters are still `+` on `u64` inside `Scoreboard::record` and
  `recorded()` (the module allows the lint explicitly). A run of 2^64 samples is
  not reachable; an overflow flag was not added because no reachable path could
  set it, and an unreachable flag is not evidence.
* **No performance claim is made or verified here.** Warm-up, repeated trials,
  worker counts and the mixing of cache-hit and cold scans inside one latency
  profile are unexamined in this session; the profile records `from_cache` per
  sample, so a future run can split the populations, but the printed mean/P95/P99
  today mixes them. Recorded as follow-up, not fixed.
