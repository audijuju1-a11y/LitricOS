# 17 — Test lab and fuzzing

## Test pyramid

| Ring | Trigger | Purpose |
|---|---|---|
| pre-commit | each change | fast lint, unit, property and contract checks |
| CI image | merge/nightly | reproducible build, boot, integration, fault tests |
| virtual system | nightly | protocol, update, storage, service and UI scenarios |
| debug kernel | scheduled | KUnit/kselftest, sanitizers, concurrency detection |
| hardware lab | nightly/release | HCL boot, sleep, devices, graphics, battery, soak |
| release red team | milestone | adversarial boundary and recovery testing |

## Fuzz inventory

Fuzz package manifests, signatures/metadata parsing, PE inspection, protocol
decoders, Wayland clients, image/media/archive helpers, URI/settings import,
IPC messages, storage metadata and update metadata. Harnesses define seed corpus,
coverage, time/memory cap, crash deduplication, minimisation and security triage.

## Fault injection inventory

Force allocation failure, disk full, read-only filesystem, power loss, failed
signature, revoked permission, expired certificate, device removal, Wi-Fi loss,
clock jump, process/service death, GPU reset, low-memory pressure, bad helper
output, recovery-slot failure and interrupted migration.

## Hardware laboratory

The HCL matrix records exact model, CPU/GPU, storage, Wi-Fi, audio codec,
firmware, kernel, Mesa, dock/display and test date. Every failure is classified
as reproducible, flaky, blocked upstream, unsupported, or test defect. A flaky
test is not ignored: it has an owner, retry policy and removal condition.

## Definition of test completion

A test has preconditions, action, oracle, artifact, cleanup, reproducibility and
failure interpretation. A feature without a negative case is incomplete whenever
it accepts input, controls a resource, persists state, crosses a trust boundary,
or runs concurrently.

