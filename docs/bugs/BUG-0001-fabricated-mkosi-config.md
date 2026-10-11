# BUG-0001 — Fabricated mkosi configuration schema

```
Bug ID        : BUG-0001
Severity      : S2 (HIGH) — build pipeline cannot execute; false GATE 01 evidence
Class         : TOOLCHAIN + DOC (see PROTOCOL §8 bug-class matrix)
Subsystem     : image/ (build pipeline)
First Observed: 2026-08-26, audit of the pre-v2 tree
```

## Reproducer

```bash
mkosi --directory image/ build
```

## Expected

mkosi parses `image/mkosi.conf` and produces a disk image.

## Actual

`image/mkosi.conf` (pre-v2) declared configuration sections that **do not exist in
mkosi**:

| Section in the file | Exists in mkosi? |
|---|---|
| `[EFIBootPartition]` | NO |
| `[RootPartition]` | NO |
| `[RootVerity]` | NO |
| `[Tree]` | NO |
| `[Initrd]` | NO |
| `[SecureBoot]` | NO |
| `[Qemu]` | NO |
| `[Build]` with `PackageManager=` | NO |

mkosi's real top-level sections are `[Config]`, `[Distribution]`, `[Output]`,
`[Content]`, `[Validation]`, `[Host]`, `[Build]`. **Partition geometry is not
expressed in `mkosi.conf` at all** — it comes from `systemd-repart` drop-in files
under `mkosi.repart/*.conf`. Package lists live under `[Content] Packages=`, not
`[Tree] Packages=`. Verity is produced by repart `Verity=` / `VerityMatchKey=`
directives, not by a `[RootVerity]` section.

The file therefore fails at parse time. Nothing it describes was ever built.

## Impact

1. **Build blocked.** The advertised pipeline could not have produced an image.
2. **False evidence.** `docs/GATES.md` recorded GATE 01 as `PASS_WITH_LIMITATIONS`
   citing this file as a deliverable. GATE 01's actual conditions require a
   *booting image*, a *reproducible hash*, a *measured boot time* and a *measured
   idle RAM*. None were produced. This violates PROTOCOL LAW 1 (no evidence, no
   claim), LAW 8 (never hide failure), and MASTER §2 G3 (no fabricated results).

## Root Cause

Configuration was written from recollection of what an image builder's schema
"should" look like, rather than from the tool's actual documented schema, and was
never executed — so the error could not surface. The gate was then recorded as
passed on the basis of *files existing* rather than *behaviour observed*.

## Why Existing Tests Missed It

There were no tests. `tools/build-image.sh` was itself never run (its own preflight
requires Linux; the host is Windows without WSL). Nothing in the repo could have
detected the defect.

## Fix

1. `image/mkosi.conf` rewritten against the real schema.
2. Partition geometry moved to `image/mkosi.repart/*.conf` (systemd-repart syntax).
3. `ci/lint-mkosi.py` added — statically rejects unknown mkosi section names, so
   this defect class cannot silently return.
   This report originally cited the linter as ci/lint-config.sh, a filename that
   has never existed in this repository; the path above was corrected on
   2026-10-09 by `ci/lint-doc-refs.py`, which is now the check that stops a
   citation like that from surviving again.
4. GATE 01 revoked in `docs/GATES.md` and reclassified `BLOCKED`.

## Regression Test

`ci/lint-mkosi.py` — fails if `image/mkosi.conf` contains a section outside the
known-good allowlist. Verified to fail against the pre-v2 file preserved at
`.backup-pre-v2/image/mkosi.conf`.

## Verification

- `ci/lint-mkosi.py` run against pre-v2 file: **must fail** (this is the
  fail-without-the-fix property required by PROTOCOL §32).
- `ci/lint-mkosi.py` run against v2 file: **must pass**.
- Full verification (mkosi actually building) remains **NOT RUN** — requires a
  Linux build host. See `docs/EVIDENCE.md`.

## Residual Risk

Static linting proves the section names are legal; it does not prove the values
are semantically correct or that the build succeeds. Only an executed build on a
Linux host closes this. Tracked as the GATE 01 blocker.
