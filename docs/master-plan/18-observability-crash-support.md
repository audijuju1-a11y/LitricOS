# 18 — Observability, crash, and support

## Principles

Observability must make defects diagnosable without turning the OS into a data
collector. Local diagnostics are default; export is user initiated, minimised,
redacted, encrypted in transit and visible before sending.

## Required records

- Boot/update slot, image/version, verity result and recovery reason.
- Service health, restart count, cgroup pressure/events, resource use and unit
  dependency failure.
- Compositor frame/input statistics, GPU/device reset events and display setup.
- App crash signature, bounded backtrace, SDK/runtime version, consented logs
  and capability decisions—never document contents, passwords or secret tokens.
- Storage SMART/scrub status, snapshot/update outcomes, network/device events.

## Crash workflow

`detect → contain → capture bounded evidence → restart or show recovery action
→ deduplicate → classify severity → reproduce → root-cause → regression test`.
Application crash is isolated; service restart follows unit policy; shell core
crash has a defined session restart path; kernel panic writes configured evidence
and enters safe recovery rather than repeated uncontrolled boot loops.

## Support bundles

Create tiered user-approved bundles: basic system facts; application crash;
network; graphics; storage; and developer diagnostics. Each lists included data,
hashes files before export, supports local save, and is deleted after configured
retention. Remote telemetry is disabled by default in v1.

## Quality metrics

Track crash-free hours on consenting test systems, restart loops, update rollback
rate, boot failure rate, time-to-diagnosis, open defect severity and flaky-test
rate. Metrics guide release decisions but do not identify personal content.

