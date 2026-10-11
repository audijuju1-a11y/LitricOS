# 14 — Performance methodology

## Measurement contract

No performance claim is valid unless its artifact records image hash, Git/source
revision when available, kernel/Mesa/firmware versions, device model, AC/battery
state, display configuration, ambient thermal condition, test command, raw
samples, sample count, median, p95/p99 and failure/warm-up policy.

## Workloads

| Domain | Required metric |
|---|---|
| boot | firmware-excluded and end-to-interactive time, critical path trace |
| resume | wake-to-input-ready time and device recovery failures |
| idle | PSS/RSS breakdown, CPU, wakeups, interrupts, writes and power |
| UI | input-to-present p50/p95/p99, frame time, missed flips |
| apps | cold/warm launch, portal latency, memory and cleanup |
| storage | file-operation latency, indexer impact, update and snapshot overhead |
| audio | round-trip latency, xruns, CPU and device switching |
| battery | energy per task and idle drain under controlled display/network state |

## Baseline policy

Compare only identical hardware, firmware, power mode, display refresh,
workload/data set and observation period. Windows/macOS/Linux data is context,
not a Litric result. Publish full methodology beside comparative charts and call
out material configuration differences.

## Budgets and regression

`docs/PERF-budget.md` provides provisional gates: desktop idle RAM ≤320 MB,
aggregate CPU ≤0.5%, system wakeups ≤25/s, shell wakeups ≤2/s and idle writes
≤2 MB/10 min. CI alerts on an unexplained regression; a release blocks when a
hard budget fails or a tail-latency regression harms the foreground journey.

## Experiment discipline

Use at least five independent runs, discard only documented invalid runs, report
variance, profile before optimising, and preserve traces. Optimisations must
state capability cost, security impact, and rollback switch. QEMU is valid for
functional reproducibility but not battery/GPU competitive claims.

