# The performance regression harness (A2)

Status: implemented. This is a comparator for `render_bench` runs, not a new
benchmark and not a measurement of a machine. It makes the W9 acceptance
sentence executable: a planted 15 % regression must fail.

## Why it exists

`src/litric-shell/examples/render_bench.rs` already prints a median and a sample
count for every scene it times. `docs/PERF-budget.md` already says CI fails on a
regression over its limit. Nothing joined the two, so a candidate could be 30 %
slower and every gate would still print PASS. A number with no comparator is a
recording, not a gate.

The real drawing path stays in `render_bench`. This script reads that output;
it does not compose a scene, start a framebuffer or duplicate a timer. If the
benchmark and the gate were two implementations, they would drift.

## Input and command

A baseline is the raw output of a known-good `render_bench` run on the same
host, release profile, canvas and workload as the candidate:

```text
cargo run --release -p litric-shell --example render_bench -- /tmp/perf/base > baseline.txt
cargo run --release -p litric-shell --example render_bench -- /tmp/perf/candidate > candidate.txt
python3 ci/check-perf-regression.py --baseline baseline.txt --actual candidate.txt
```

The comparator reads every `*.median_ms` line and its matching `*.runs`. A
median without 15 runs is not a result. The default threshold is 10 %, matching
`docs/PERF-budget.md` rule 4; `--threshold 15` is the W9 acceptance boundary.
The failure test is `>=`, so an exact 15.0 % regression fails.

Only medians are compared. The benchmark also prints profile, bucket and
pixel counts; those are diagnostic lines and do not have a stable per-run
threshold. A new scene in the candidate is printed as `NEW` and does not fail
the comparison, because there is no baseline for it; the baseline must be
re-recorded deliberately to make it a gate.

## Read precedence

The first matching row wins. Bad input is a failure, never a silent pass.

| # | Observation | Result |
|---:|---|---|
| 1 | baseline or candidate cannot be opened | `perf-regression: FAIL` + reason |
| 2 | no numeric `key=value` metrics | fail |
| 3 | a numeric metric is duplicated, non-finite or negative | fail |
| 4 | no `*.median_ms` metric | fail |
| 5 | a median has no `*.runs`, a non-integer run count, or fewer than 15 runs | fail |
| 6 | a baseline scene is absent from the candidate | fail, naming the scene |
| 7 | the candidate median is at or above the threshold above baseline | fail, with both medians and the percentage |
| 8 | the candidate has a scene the baseline does not | print `NEW`; no comparison |
| 9 | every baseline scene is below the threshold | PASS |

## Self-test, and what it proves

`ci/verify.sh` runs:

```text
python3 ci/check-perf-regression.py --self-test
```

The self-test uses a small baseline with 15 runs and checks the exact boundary:
an unchanged run passes, a planted **+15.0 %** regression fails with the
percentage named, **+14.9 %** passes, and a missing scene, a 14-run sample, a
duplicate metric and a negative median each fail. This proves the comparator can
fail. It does not prove any real candidate is fast.

## What this does not establish

* It does not measure a booted image, kernel, display, GPU or compositor; it
  consumes the host-side `render_bench` output.
* It does not validate that both runs used the same host, build, canvas or
  workload. That is the operator's documented obligation; the script cannot see
  it.
* It does not enforce the absolute 16.67 ms frame budget. It detects a
  regression against a recorded baseline; an already-over-budget scene can pass
  this comparator because it did not get worse.
* It does not close W9. The zram/MGLRU/oomd policy and measured boot/idle
  numbers still require a builder or a booted machine.
* `ci/verify.sh` runs only the self-test. A real comparison needs two controlled
  benchmark runs and is not part of the default gate.

## References

* [Whole-system plan A2](../master-plan/28-whole-system-improvement-plan.md)
* [Resource budget: frame budget and rules of engagement](../PERF-budget.md)
* [The benchmark itself](../../src/litric-shell/examples/render_bench.rs)
