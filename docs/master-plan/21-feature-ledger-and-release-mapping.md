# 21 — Feature ledger and release mapping

## Source of truth

`docs/SYSTEM-SCOPE.md` contains the complete numbered feature inventory. This
plan does not duplicate its 1,117 rows because duplicated feature status will
drift. Instead, every row must receive operational metadata in a machine-checked
ledger or adjacent table before it moves beyond `S`.

## Required metadata per feature

| Field | Meaning |
|---|---|
| ID | existing SYSTEM-SCOPE number |
| release | FND / DP / BETA / V1 / POST |
| owner | Litric, upstream, or integration owner |
| dependency IDs | prerequisite features/ADRs |
| contract | public API/ABI/format impact or `none` |
| trust class | none, local, privileged, external, update/boot |
| budget | RAM/CPU/wakeup/disk/power impact or `n/a` |
| tests | test IDs and hardware matrix requirement |
| evidence | command/artifact IDs when status is P or D |
| risk | issue/ADR/accepted limitation link |

## Release mapping by section

| Scope area | Foundation | DP | Beta | V1 | Post-v1 |
|---|---|---|---|---|---|
| A: boot/kernel/storage | verified boot/build | recovery/encryption | installer/HCL | measured boot/firmware flow | advanced hardware |
| B–D: desktop/settings/files | client compositor | core shell/files/settings | device & accessibility UX | polish/support | niche layouts |
| E: security | config/prototypes | sandbox/capability broker | defender/USB policy | response process | advanced enterprise |
| F: applications | Flatpak integration | SDK preview | store/built-ins | stable SDK | native package redesign |
| G: input/accessibility/i18n | input basics | semantic tree/Thai | assistive workflows | supported baseline | broader locales |
| H: gaming/AI/update/dev | A/B update/dev harness | tools/local AI opt-in | guest preview | validated subsets | anti-cheat/cloud/ARM |
| I–J: environments/resources | budgets/observability | cgroup engine | power/HCL | stable policy | enterprise/advanced VM |

## Status rules

- `T`: no implementation or evidence.
- `S`: design approved but no executable proof.
- `P`: code exists with named, passing tests; operational gaps are explicit.
- `D`: all declared contracts, integration, failure/security/performance tests and
  applicable HCL evidence have passed.

`D` cannot be inferred from a screenshot, compilation, lint pass, or a document.
The existing count script must be extended when the metadata ledger becomes
machine-readable so release/risk totals cannot drift.

