# A3 — claim audit: rows with an absent backend

Date: 2026-10-30
Base: `ca4e7c1`
Scope: every `P` row in `docs/SYSTEM-SCOPE.md` at that base (122 rows).

## What was audited

`P` means partial, real code with passing tests. It does not mean complete. This
audit asked one narrower question: **does the row name or imply a backend that
does not exist at all?** Incompleteness alone is not a failure of the `P`
status; a model with tests and a documented execution gap is exactly what `P`
is for.

The audit enumerated the 122 rows and reviewed their claimed backends against the
source tree, shipped units and existing tests. The rows below were re-tagged
because their claimed execution backend is absent.
Rows whose model or library exists but whose remaining work is a separate
integration point stayed `P`; examples are the update generation engine rows,
the browser broker, the environment state machine and the Defender engine.

## Re-tagged rows

| Row | Claim | What actually exists | Why it is not `P` | New status |
|---:|---|---|---|---|
| 476 | Capability broker daemon (`litric-capd`) | `litric-caps` is a library; the broker lives inside `litric-sysd`; `image/units-pending/litric-capd.service` and `.socket` are parked and name `/usr/bin/litric-capd`, which no crate builds | The row names a daemon that has no binary, no crate, no socket implementation and no tests. The sysd broker is a different component. | `S` |
| 843 | `.exe` double-click → installed, isolated, uninstallable app | `litric-winbridge` inspects PE files and produces an ordered `InstallPlan`; it explicitly does not execute, spawn, write files or use the network. There is no `litric-compatd`. | The decision/planning half exists, but the claimed double-click-to-installed path has no executor. | `S` |
| 844 | Per-app disposable Windows environment | `Bottle` and `Isolation` are typed models with tests; no environment is created, started, suspended, reset or deleted by a runtime. | The model exists, but no disposable environment backend does. | `S` |
| 863 | Host CPU/memory reservation enforced against guests | `litric-env` computes budgets, refuses a guest that would breach `HostReservation`, and tests those decisions. No VM is started and no cgroup/KVM limit is applied. `litric-envd` is row 876, still `T`. | The policy is tested; the enforcement against guests is absent. | `S` |
| 947 | Demand-based memory sharing between environments | `litric-env` tracks committed guest budgets and hibernation state; there is no page sharing, ballooning, KSM, zram or rebalancing path. | No demand-based sharing code exists. | `S` |

## Not moved

* Rows 760, 762, 763, 764 and 769 have real staged-write, marker,
  boot-count, health-evidence and atomic-record code with tests. Their limits
  (no bootloader, partition, EROFS, verity or power-cut matrix) are recorded;
  the backends that exist are real.
* Row 551 has a real `browserd` that spawns Chromium and drives CDP; the
  "missing engine" null path is honest when Chromium is absent.
* The capability rows other than 476 have the broker, scoped grants, revocation,
  audit and traversal tests inside `litric-caps`/`litric-sysd`; only the separate
  `capd` daemon claim is absent.
* Rows 860–862 and 864–875 are the environment state model, not VM execution;
  they have tests and remain `P`. Row 876 (`litric-envd` execution) is already
  `T`.
* Row 571's monitor has real process/CPU/memory code; any remaining columns are
  incompleteness, not an absent backend.

## Counts

Before: **122 P · 288 S**. After: **117 P · 293 S**. `D`, `T` and the total
(1,165) are unchanged. No row was promoted to `P`.

## Limits

* This is a source and test audit, not a runtime proof. A module can exist and
  still be wrong; that is what the existing unit/integration tests and this
  project's gate log are for.
* The audit does not prove that every remaining `P` row is complete. It only
  removes claims whose named execution backend is absent.
* It does not close A1, W9, or the bootloader/partition/EROFS/verity/signature
  gaps.
