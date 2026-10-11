# LitricOS Adversarial Benchmark — Run bench-928374-l1

* Seed: 928374 · Corpus: 1000 samples · Recovery: RECOVERED

## Detection (by-construction ground truth)

| metric | value |
|---|---|
| true positives | 122 |
| false negatives (misses) | 858 |
| false positives | 0 |
| true negatives | 20 |
| unsupported | 0 |
| environment limitations | 0 |
| detection rate | 12.45% |
| false-positive rate | 0.00% |

## Performance

| latency | µs |
|---|---|
| mean | 2638623 |
| P95 | 12206375 |
| P99 | 14166048 |
| P99.9 | 16436243 |
| throughput | 0 samples/s |

## Rule gaps (per category)

| category | hostile | detected | coverage | FP |
|---|---|---|---|---|
| 01-benign | 0 | 0 | 0.0% | 0 |
| 02-suspicious-static | 20 | 20 | 100.0% | 0 |
| 03-suspicious-script | 20 | 20 | 100.0% | 0 |
| 04-suspicious-archive | 20 | 20 | 100.0% | 0 |
| 05-malformed | 20 | 0 | 0.0% | 0 |
| 06-corrupted | 20 | 0 | 0.0% | 0 |
| 07-parser-stress | 20 | 0 | 0.0% | 0 |
| 08-memory-stress | 20 | 0 | 0.0% | 0 |
| 09-cpu-stress | 20 | 0 | 0.0% | 0 |
| 10-disk-stress | 20 | 0 | 0.0% | 0 |
| 11-process-tree | 20 | 0 | 0.0% | 0 |
| 12-ipc | 20 | 20 | 100.0% | 0 |
| 13-privilege-boundary | 20 | 0 | 0.0% | 0 |
| 14-service | 20 | 20 | 100.0% | 0 |
| 15-driver-interface | 20 | 0 | 0.0% | 0 |
| 16-kernel-interface | 20 | 0 | 0.0% | 0 |
| 17-fs-filter | 20 | 0 | 0.0% | 0 |
| 18-network-telemetry | 20 | 20 | 100.0% | 0 |
| 19-anti-tamper | 20 | 0 | 0.0% | 0 |
| 20-quarantine | 20 | 0 | 0.0% | 0 |
| 21-rollback | 20 | 0 | 0.0% | 0 |
| 22-recovery | 20 | 0 | 0.0% | 0 |
| 23-high-entropy | 20 | 0 | 0.0% | 0 |
| 24-encoded-data | 20 | 0 | 0.0% | 0 |
| 25-obfuscation-like | 20 | 0 | 0.0% | 0 |
| 26-polymorphic-like | 20 | 0 | 0.0% | 0 |
| 27-packed-like | 20 | 0 | 0.0% | 0 |
| 28-pe-anomaly | 20 | 0 | 0.0% | 0 |
| 29-elf-anomaly | 20 | 0 | 0.0% | 0 |
| 30-exec-metadata | 20 | 0 | 0.0% | 0 |
| 31-archive-bomb | 20 | 0 | 0.0% | 0 |
| 32-extreme-nesting | 20 | 0 | 0.0% | 0 |
| 33-path-stress | 20 | 0 | 0.0% | 0 |
| 34-unicode-path | 20 | 0 | 0.0% | 0 |
| 35-link-like | 20 | 0 | 0.0% | 0 |
| 36-file-lock | 20 | 0 | 0.0% | 0 |
| 37-race | 20 | 0 | 0.0% | 0 |
| 38-toctou | 20 | 0 | 0.0% | 0 |
| 39-process-churn | 20 | 0 | 0.0% | 0 |
| 40-thread-churn | 20 | 0 | 0.0% | 0 |
| 41-handle-churn | 20 | 0 | 0.0% | 0 |
| 42-mem-fragmentation | 20 | 0 | 0.0% | 0 |
| 43-alloc-failure | 20 | 0 | 0.0% | 0 |
| 44-io-failure | 20 | 0 | 0.0% | 0 |
| 45-telemetry-overload | 20 | 0 | 0.0% | 0 |
| 46-log-flood | 20 | 0 | 0.0% | 0 |
| 47-event-flood | 20 | 0 | 0.0% | 0 |
| 48-crash-recovery | 20 | 0 | 0.0% | 0 |
| 49-boot-recovery | 20 | 0 | 0.0% | 0 |
| 50-zero-day-like | 20 | 2 | 10.0% | 0 |

## Recommended defensive changes

- Add hash/pattern coverage for category `05-malformed` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `06-corrupted` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `07-parser-stress` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `08-memory-stress` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `09-cpu-stress` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `10-disk-stress` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `11-process-tree` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `13-privilege-boundary` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `15-driver-interface` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `16-kernel-interface` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `17-fs-filter` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `19-anti-tamper` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `20-quarantine` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `21-rollback` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `22-recovery` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `23-high-entropy` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `24-encoded-data` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `25-obfuscation-like` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `26-polymorphic-like` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `27-packed-like` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `28-pe-anomaly` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `29-elf-anomaly` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `30-exec-metadata` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `31-archive-bomb` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `32-extreme-nesting` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `33-path-stress` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `34-unicode-path` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `35-link-like` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `36-file-lock` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `37-race` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `38-toctou` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `39-process-churn` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `40-thread-churn` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `41-handle-churn` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `42-mem-fragmentation` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `43-alloc-failure` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `44-io-failure` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `45-telemetry-overload` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `46-log-flood` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `47-event-flood` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `48-crash-recovery` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `49-boot-recovery` (20 of 20 hostile samples missed).
- Add hash/pattern coverage for category `50-zero-day-like` (18 of 20 hostile samples missed).
