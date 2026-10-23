# LitricOS Repository Audit

This directory contains a source-grounded repository audit and a 838-file inventory. Batch reports describe file contents and key APIs; the reference documents connect those details across architecture, protocols, security, builds and evidence.

## Start here

1. [`SYSTEM_ARCHITECTURE.md`](SYSTEM_ARCHITECTURE.md) — implemented topology vs documented target.
2. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md) and [`DATA_FLOWS.md`](DATA_FLOWS.md) — crate edges and runtime flows.
3. [`API_AND_INTERFACE_REFERENCE.md`](API_AND_INTERFACE_REFERENCE.md), [`CONFIGURATION_REFERENCE.md`](CONFIGURATION_REFERENCE.md) — contracts and configuration entry points.
4. [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md), [`TEST_AND_BUILD_MAP.md`](TEST_AND_BUILD_MAP.md), [`CHANGE_IMPACT_MAP.md`](CHANGE_IMPACT_MAP.md), [`UNRESOLVED_ITEMS.md`](UNRESOLVED_ITEMS.md) — review and follow-up.
5. [`FILE_CATALOG.md`](FILE_CATALOG.md), [`FILE_INVENTORY.json`](FILE_INVENTORY.json), [`FILE_INVENTORY.csv`](FILE_INVENTORY.csv), [`AUDIT_PROGRESS.json`](AUDIT_PROGRESS.json) — item-level records and progress.

## File-level reports

| Report | Scope |
|---|---|
| [`files/01-root-and-workflow-specs.md`](files/01-root-and-workflow-specs.md) | Root metadata, master workflow and phase specs (24 files) |
| [`files/02-kernel-image-iso-keys.md`](files/02-kernel-image-iso-keys.md) | Kernel, mkosi image/profiles, systemd, ISO, keys (65 files) |
| [`files/03-tools-ci-tests.md`](files/03-tools-ci-tests.md) | CI, tests and developer tooling (55 files) |
| [`files/04-workspace-cargo-and-zip.md`](files/04-workspace-cargo-and-zip.md) | Cargo manifests/lock and design archive (4 files) |
| [`files/05-crates-foundation-and-core.md`](files/05-crates-foundation-and-core.md) | Foundation and core crates (32 files) |
| [`files/06-crates-apps-and-services.md`](files/06-crates-apps-and-services.md) | Browser, Defender, Files and Winbridge crates (77 files) |
| [`files/07-crates-runtime-and-daemons.md`](files/07-crates-runtime-and-daemons.md) | Bench, input, sysd and WM crates (41 files) |
| [`files/08-crate-litric-shell.md`](files/08-crate-litric-shell.md) | Shell sources, examples and tests (32 files) |
| [`files/09-docs-architecture-and-design.md`](files/09-docs-architecture-and-design.md) | Architecture, ADR, audit, design and governance docs (109 files) |
| [`files/10-evidence-and-artifacts.md`](files/10-evidence-and-artifacts.md) | Captures, logs, reports and generated artifacts (359 files) |
| [`files/11-backup-pre-v2.md`](files/11-backup-pre-v2.md) | Historical backup snapshot (40 files) |

## Scope and limitations

- The file inventory records 838 project entries discovered for this audit. `Saved/` audit outputs are intentionally not included as source inputs, preventing recursive self-counting.
- `FILE_INVENTORY.json` and CSV status `completed` mean the file is represented in its assigned report/catalog; they do **not** certify that the code builds, a runtime feature works, or a security claim is fully verified.
- This continuation did not run Cargo, CI, image builds, QEMU, or hardware tests. See `TEST_AND_BUILD_MAP.md`; no check is claimed as newly passing.
- Specifications, ADRs, source, historical bugs and evidence can disagree. Reports mark known gaps; use `UNRESOLVED_ITEMS.md` and the current source/evidence before relying on a claim. Historical `.backup-pre-v2/` content is not current authority.
- Overall release state remains **NOT_VERIFIED** as the project gate matrix states. Targets and models are not runtime proof.
