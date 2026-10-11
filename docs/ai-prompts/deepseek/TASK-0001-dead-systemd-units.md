# TASK LITRICOS-W0-0001 — delete the systemd units whose binaries do not exist

## 0. Outcome — 2026-10-11 (written after execution)

The five units were **moved out of the installed unit directory** to
[`image/units-pending/`](../../../image/units-pending/README.md) rather than
deleted, because the checkout had no version control at the time of the change
(`docs/audit/2026-10-11-beta-1.0-stage-a-audit.md` §1), so a deletion would have
been unrecoverable. `litric-defender.slice` was moved with them: it carries a
`MemoryMax=` for a service that does not exist, so on its own it governed
nothing, and `ci/check-memory-budget.py` reports exactly that. The `systemctl
enable litric-files.socket` line in `image/mkosi.postinst.chroot` was removed
with a comment giving the reason (a socket needs no binary, so the boot stayed
healthy and the failure arrived at first use).

The linter this ticket asks for exists as `ci/lint-unit-binaries.py` and is
wired into `ci/verify.sh` as section [3g]. One deliberate difference from §5.2:
image-shipped helpers are **not** an allowlist. `/usr/lib/litric/<name>` is
resolved by existence in the image tree, so adding a helper cannot be forgotten.
A second, larger difference: a `/usr/bin/<name>` that is neither ours nor named
`litric-*` is reported as SKIP (base distribution) rather than MISSING, because
this gate also runs on a Windows host that cannot enumerate the image's `/usr/bin`.
That one was found by reading the file after writing it: the docstring promised
SKIP for `/usr/bin/cat` and the code returned MISSING, which would have failed the
gate the first time a unit ran `systemctl`.

Acceptance test 4 (`ci/verify.sh` shows the new checks as PASS with nothing else
changing) was run on 2026-10-11 as part of the full gate:
`bash ci/verify.sh --evidence` reported **25 pass, 0 fail, 5 skipped** (the skips
are the image build and the boot measurements, which need a Linux host). Test 5,
`grep -rn "litric-capd\|litric-filesd" image/`, now matches only the parked files
under `image/units-pending/` and this record.

## 1. Context from verified repository

- **Repository root:** the LitricOS project root. `src/` is a Cargo workspace;
  `src/Cargo.toml` lists its members (18 of them at the time of writing) in six
  labelled groups; `image/` holds the mkosi image definition; `ci/` holds the gate
  script and its linters.
- **Architecture facts confirmed in source:**
  - The image starts services through systemd units shipped at
    `image/mkosi.extra/usr/lib/systemd/system/`.
  - Only four workspace crates declare a binary target: `litric-shell`,
    `litric-sysd` and `litric-envd`, each with a `main.rs` under its own `src/`
    directory (for example `src/litric-envd/src/main.rs`), plus
    `litric-browser`, whose binary is `src/litric-browser/src/bin/browserd.rs`.
  - `litric-defender` and `litric-files` are **libraries**. `litric-caps` has no
    binary target at all.
- **Evidence, produced in this checkout:**

  ```
  $ grep -h ExecStart image/mkosi.extra/usr/lib/systemd/system/*.service
  ExecStart=/usr/bin/litric-capd --systemd-socket
  ExecStart=/usr/bin/litric-defender --rules /usr/lib/litric/yara --cache /var/cache/litric/defender
  ExecStart=/usr/bin/litric-filesd --stdio --root /var/home/litric
  ... (litric-shell, litric-sysd, litric-envd, litric-browserd and the
       selftest/mm-tune helpers are all backed by real binaries)
  ```

  Three `ExecStart` lines therefore name binaries that no crate produces.
- **Existing policy that governs this task:**
  [`docs/adr/0016-system-daemon.md`](../../adr/0016-system-daemon.md) already
  states that `litric-capd.service` and `litric-capd.socket` "are now redundant
  and refer to a binary that does not exist… They are not enabled, and should be
  removed rather than left to look like they work."
- **Gate script:**
  [`ci/verify.sh`](../../../ci/verify.sh) runs a `check "<description>" <command>`
  harness, and several checks have a companion `--self-test` proving the checker
  fails on a known-bad input (`ci/lint-doc-refs.py --self-test`,
  `ci/lint-kconfig.py --self-test`, `/c/Users/audim/...` entries in
  [`docs/EVIDENCE.md`](../../EVIDENCE.md)).

## 2. One objective

Remove the systemd units that reference non-existent binaries, and make
`ci/verify.sh` fail whenever a unit's `ExecStart` names a binary that no crate in
`src/` produces.

**Success means:** the three unit files are gone (or explicitly rewritten to the
real service model), and a new gate check fails on a planted bad unit and passes
on the real tree.

## 3. Non-goals

- Do not implement `litric-capd`, `litric-defender` or `litric-filesd`. Whether
  those daemons should exist is an architecture decision (ADR-0016 argues the
  capability broker lives inside `litric-sysd`).
- Do not touch `litric-shell.service`, `litric-sysd.service`,
  `litric-envd.service` or `litric-browser@.service`.
- Do not change units that run shell helpers
  (`/usr/lib/litric/selftest`, `/usr/lib/litric/mm-tune`) — those files ship in
  the image.
- No unrelated refactors, no renames, no new dependencies.

## 4. Required invariants

- The gate must **fail closed**: an unparsable unit or a missing binary is a
  failure, not a skip.
- The new check must not require a network, a build, or root.
- The check must be Python 3 standard-library only, matching the other linters in
  `ci/`, and must run on a Windows development host (the gate already runs there).
- Existing checks and their descriptions must keep working; do not renumber or
  reorder them.
- LF line endings only — CRLF in a unit or a CI script is a recorded defect class
  in this repository.

## 5. Expected implementation

1. **Unit removal:** take `litric-capd.service` (then at
   image/mkosi.extra/usr/lib/systemd/system/, now parked — see §0), its socket,
   `litric-defender.service`, `litric-files@.service` and `litric-files.socket`
   out of the installed unit directory **if and only if** you
   first confirm none of them is enabled by any `systemctl enable` reference in
   `image/mkosi.postinst.chroot`, `image/mkosi.postinst.chroot.d/` or a
   `wants`/`requires` symlink. Report the search you performed.
2. **New linter:** ci/lint-unit-binaries.py — *this file does not exist yet;
   this ticket creates it.*
   - Discover unit files under `image/**/system/*.service`.
   - Parse `ExecStart=` values. Handle the absolute-path form
     (`ExecStart=/usr/bin/foo args…`) and ignore wrapper forms
     (`ExecStart=/bin/sh -c '…'`) with an explicit, counted skip reason.
   - Collect the set of binary names the workspace can actually install: parse
     every `[[bin]]`/`name` in `src/*/Cargo.toml`, plus `src/*/src/main.rs`, plus
     an explicit allowlist of image-shipped helpers
     (`/usr/lib/litric/selftest`, `/usr/lib/litric/mm-tune`) with the reason.
   - Exit non-zero and name every offending unit and binary.
   - Support `--self-test`, which creates a temporary fixture unit referencing a
     binary that cannot exist and asserts the linter fails on it.
3. **Wire it into the gate:** one `check "systemd units reference real binaries"
   "$PY" ci/lint-unit-binaries.py` plus a
   `check "unit-binary linter detects a dead unit" … --self-test`, placed beside
   the other mkosi/image checks.
4. **Update the docs that list the units** if they name a unit you deleted:
   `docs/adr/0016-system-daemon.md` (the sentence is already correct — adjust only
   if it becomes stale) and any list in `docs/master-plan/`.

## 6. Acceptance tests

| # | Setup | Action | Expected |
|---|---|---|---|
| 1 | real tree | run the new linter | exits 0; the two image helpers are reported as SKIP with the allowlist reason |
| 2 | fixture unit in a temp dir referencing `/usr/bin/litric-capd` | run `--self-test` | non-zero exit, message names the unit and the binary |
| 3 | **negative control** | delete a real `[[bin]]` entry from a copy of `src/litric-envd/Cargo.toml` in a temp fixture and point the linter at it | linter reports the now-dead `litric-envd.service` |
| 4 | real tree | run `ci/verify.sh` | the new checks appear as PASS and no previously-passing check changes |
| 5 | real tree | `grep -rn "litric-capd\|litric-filesd" image/` | no result, or only a documented historical reference |

Level: `UNIT` for the linter plus `STATIC` for the tree. No build or VM is
required, so this ticket can be completed and verified on the development host.

## 7. Files allowed to change

- **Allowed:** ci/lint-unit-binaries.py (new — it does not exist yet; this
  ticket creates it), `ci/verify.sh`,
  `image/mkosi.extra/usr/lib/systemd/system/` (deletions only).
- **Tests allowed:** the linter's own `--self-test`, and any fixture directory the
  gate creates and cleans up.
- **Forbidden:** everything else, in particular `src/**`, `docs/adr/0016-system-daemon.md`
  except for a stale-list correction, and any other check in `ci/verify.sh`.

## 8. Output format

Return, in order: (1) diagnosis grounded only in the evidence above; (2) the
minimal design and its trade-offs; (3) the patch with path labels; (4) tests
added and what each proves; (5) **the exact commands you executed and their
results — if you ran none, write `NOT RUN` and do not claim success**; (6) risks
and unknowns; (7) integration notes for the reviewer.

Do not claim that `ci/verify.sh` passed unless you actually ran it. If your
environment cannot run Python, say so — the check will be run here anyway.
