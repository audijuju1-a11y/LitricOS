# TASK LITRICOS-W0-0003 — enforce the dependency policy instead of asserting it

## 0. Outcome — 2026-10-11 (written after execution)

`ci/lint-deps.py` exists, is wired into `ci/verify.sh` as section **[3h]**, and
passes on the real tree: `18 member(s): 16 covered by the no-dependency policy, 2
outside it (applications, native ecosystem)`, 0 violations, exit 0.

**The rule implemented.**

- **Coverage is derived from `src/Cargo.toml`**, from the group comment above each
  member. `core`, `runtime`, `defender` and `benchmark` are covered; `applications`
  and `native ecosystem` are not. Every group name has an explicit decision in the
  linter's `CLASS_POLICY`; a member under a group that table does not name, or
  under no group at all, is a **failure**, which is §4's fail-closed invariant.
- A member whose own manifest declares itself dependency-free is covered whatever
  its group (§5.1, second half). No member needed it — the two manifests that say
  so are already in covered groups — and the check is kept because that claim is
  what it enforces.
- For a covered crate every entry in `dependencies`, `dev-dependencies`,
  `build-dependencies` and their `target.'…'` forms must be a path dependency **on
  a declared member of this workspace**. A path that leaves the workspace, or
  points at a directory that is not a member, is a violation: acceptance test 3
  needs the member list, not merely "has a `path`".
- A `build-dependencies` entry is refused **even inside the workspace**, with the
  reason printed, per §5.
- **Deviations, both deliberate:** the run prints one coverage line per member, so
  the decision is visible on every run rather than only in this file; and a
  violation is reported only for a **covered** crate (the ticket's non-goal is
  respected — `litric-shell` may take a dependency, and self-test case 6 asserts
  that it does not fail the gate).

**Commands run, and their results:**

| Command | Result |
|---|---|
| `python3 ci/lint-deps.py` | coverage table for all 18 members; `0 violation(s) in covered member(s)`; exit **0** |
| `python3 ci/lint-deps.py --self-test` | 9 PASS: registry dependency flagged; path dependency on a member accepted; path outside the workspace flagged; target-specific dependency flagged; in-workspace build-dependency refused; platform crate not covered; unknown manifest group fails closed; a recorded exception clears the violation and is reported; the same tree with no record fails. Exit **0** |
| acceptance test 6, `grep -rn "\[dependencies\]" src/litric-core/Cargo.toml src/litric-caps/Cargo.toml` | both files have the section header and **no entry under it** — `[dependencies]` with nothing following, which is why the linter reports 0 dependency entries for them |
| `bash ci/verify.sh --evidence` | **25 pass, 0 fail, 5 skipped**, exit 0; sections [3h] both PASS |

**Two bugs, found by running the checker rather than reading it** — recorded
because they are the reason §6 asks for a self-test at all:

1. Reading only the last line of a wrapped group comment classified
   `litric-defender` under the group "arrives in Phase 2 and is what makes this a
   platform crate", which silently *uncovered* a crate the manifest says has zero
   dependencies of any kind.
2. Clearing the group at the first member of a group left every later member
   unclassified, so `litric-caps`, `litric-env`, `litric-storage` and five others
   were outside the policy while the summary still read as though coverage were
   complete.

**No disagreement remains between the manifest's classification and the linter:**
all six groups have a decision, and the crate count in `src/Cargo.toml` (18) is
what the linter prints. Three documents said 19; that is fixed in
[TASK-0002](TASK-0002-doc-contradictions.md) §0.

## 1. Context from verified repository

- The workspace manifest [`src/Cargo.toml`](../../../src/Cargo.toml) opens with a
  **HARD RULE** stated in a comment: the crates in `core-crates` below have
  **zero third-party dependencies, std only**, for two stated reasons — supply
  chain (a system component's dependencies run with its authority) and
  verifiability (they must compile and test on a host with no network and no
  Linux).
- `docs/master-plan/24-feature-parity.md` repeats the claim in its closing
  section: *"No third-party crates in the system workspace: every DONE row above
  is zero-dependency, offline-buildable code under `#![forbid(unsafe_code)]`."*
- Several crates are declared zero-dependency in their own manifests:
  [`src/litric-defender/Cargo.toml`](../../../src/litric-defender/Cargo.toml)
  says *"Deliberately empty… it does not get to pull crates from the internet"*,
  and `src/litric-bench` is described the same way in the workspace comment.
- **The rule is enforced by convention only.** `ci/verify.sh` contains checks for
  shell syntax, line endings, Python syntax, mkosi config, kconfig, the
  `SYSTEM-SCOPE.md` tally, documentation citations, the memory ledger, deletion
  sites, signing keys, and the Cargo build/test/fmt/clippy set — **no check
  inspects dependencies at all**, and neither `ci/` nor `tests/` contains a
  dependency-policy file (`grep -rln -iE "third-party|dependency policy|no_deps"
  ci/ tests/` returns nothing).
- Consequence, and the reason this ticket exists: nothing stops a patch from
  adding a crate to a core crate and silently converting an offline-verifiable
  system component into something that must be fetched from the network. That
  change would also make the workspace's own headline claim false.

## 2. One objective

Add ci/lint-deps.py (which does not exist yet; this ticket creates it), wired
into `ci/verify.sh`, which fails when a crate that
the workspace declares dependency-free acquires any external dependency, and
which provides a `--self-test` negative control.

**Success means:** the real tree passes; a planted external dependency in a core
crate fails; and the set of crates the policy covers is derived from the manifest
rather than hard-coded in the linter.

## 3. Non-goals

- Do not add, remove or upgrade any dependency.
- Do not extend the policy to `litric-shell` in this ticket. `src/Cargo.toml`
  already names `litric-shell` and `litric-defender` as platform crates that are
  *allowed* to take dependencies later; deciding the exact boundary for the shell
  is an architecture decision. Cover the crates the manifest calls core, plus any
  crate whose own manifest declares itself dependency-free.
- Do not attempt a vulnerability audit, licence inventory or lockfile diffing —
  those are different objectives.
- Do not change `src/Cargo.toml`'s crate classification. If the classification
  and the linter disagree, report the disagreement instead of editing it away.

## 4. Required invariants

- Python 3 standard library only; must run on the Windows development host, where
  the gate currently runs, and must not need a network or a Cargo build.
- Fail closed: an unparsable manifest or a crate whose classification cannot be
  determined is a failure, not a skip. A silent skip is exactly how a policy
  check becomes decorative.
- Path handling must be relative to the repository root and independent of the
  caller's working directory, matching the other linters in `ci/`.
- The linter must not print the contents of a manifest; it reports crate names
  and dependency names only.
- LF line endings.
- Existing checks keep their wording and order.

## 5. Expected implementation

1. **Classification source of truth:** read the `members` list in
   `src/Cargo.toml` and the manifest's own core/platform comment blocks. Preferred
   approach: parse the *section the manifest labels as core* rather than
   hand-copying names, and treat an unlisted member as a failure with a clear
   message. If the manifest's structure makes that impossible without ambiguity,
   say so in the output section and propose the smallest manifest change that
   would make it unambiguous — do **not** make that change.
2. **The linter, ci/lint-deps.py:**
   - For each covered crate, read `src/<crate>/Cargo.toml`.
   - Collect dependency tables: `[dependencies]`, `[dev-dependencies]`,
     `[build-dependencies]`, and any target-specific table
     (`[target.'…'.dependencies]`).
   - A dependency is acceptable only if it is **path-based and inside the
     workspace** (a `path = "../<member>"` pointing at another declared member).
     Everything else — a version-only dependency, a git dependency, a
     registry dependency — is a violation.
   - Additionally, refuse a `[build-dependencies]` entry in a covered crate even
     if path-based, because a build script runs at build time on the host. State
     this rule explicitly in the output.
   - Print one line per covered crate with its verdict, then a summary, then exit
     non-zero if any violation was found.
   - `--self-test`: create a temporary fixture workspace in a temp directory with
     a core crate that depends on `serde = "1"` (and, separately, a path
     dependency to a non-member), and assert the linter fails on both. Clean up
     only what it created.
3. **Wire into the gate:** one check line plus its `--self-test` companion,
   placed next to the other source-level policy checks, phrased in the existing
   style: `check "core crates take no third-party dependencies" "$PY" ci/lint-deps.py`.
4. **Report, do not fix:** if the real tree already contains a violation, list it
   in the ticket answer and stop. That finding is more valuable than a green run.

## 6. Acceptance tests

| # | Setup | Action | Expected |
|---|---|---|---|
| 1 | real tree | `python3 ci/lint-deps.py` | exits 0, lists every covered crate |
| 2 | fixture core crate with `serde = "1"` | `--self-test` | non-zero exit; message names the crate and the dependency |
| 3 | fixture core crate with a `path` dependency to a directory that is not a workspace member | `--self-test` | non-zero exit |
| 4 | fixture crate with a path dependency to another member | `--self-test` | passes that case (this is the positive control that the linter is not merely rejecting all dependencies) |
| 5 | real tree | `ci/verify.sh` | the new checks PASS; no previously passing check changes state |
| 6 | real tree, after the ticket | `grep -rn "\[dependencies\]" src/litric-core/Cargo.toml src/litric-caps/Cargo.toml` | no third-party entries |

Level: `STATIC` for the linter plus `UNIT` for the self-test. No build, no VM.

## 7. Files allowed to change

- **Allowed:** ci/lint-deps.py (new — it does not exist yet; this ticket
  creates it), `ci/verify.sh`.
- **Tests allowed:** the linter's `--self-test` fixtures under a temporary
  directory.
- **Forbidden:** `src/**` including `src/Cargo.toml` and every crate manifest,
  `docs/SYSTEM-SCOPE.md`, `docs/GATES.md`, `docs/EVIDENCE.md`, `image/**`.

## 8. Output format

Return, in order: (1) the exact rule you implemented and any rule you had to
interpret; (2) which crates the policy covers and how that set was derived;
(3) the patch; (4) the tests and what each proves; (5) **the exact commands you
ran with their results — `NOT RUN` if you ran none**; (6) the disagreement
between classification and linter, if any; (7) reviewer notes.

A response that claims `ci/verify.sh` passed without a command and output is not
acceptable, and one that silently edits a manifest to make the check pass will be
rejected.
