# TASK LITRICOS-W0-0002 — correct the documentation contradictions against the tree

## 0. Outcome — 2026-10-11 (written after execution)

All three contradictions are corrected, and a fourth was found while correcting
them.

| Finding | What changed | Evidence for the replacement statement |
|---|---|---|
| the compositor | `ARCH-00`'s L5 box reads "Rust (software framebuffer, ADR-0017)"; §5's Compositor row is replaced by one "Desktop shell (no separate compositor)" row; a dated correction section names ADR-0017 for the framebuffer decision and ADR-0016 for the process split, and states that L4's service list is a target list | `docs/adr/0017-framebuffer-shell.md`, `docs/adr/0016-system-daemon.md`, `src/litric-shell/Cargo.toml` |
| the directories that do not exist | `REPO-00` gains §0, "What this repository actually is — 2026-10-11": the real top level (`src/`, `image/`, `ci/`, `docs/`, `tests/`, `tools/`, `keys/`, `build/`) and the list of planned-only directories. §1 is retitled "the Phase 00 plan" and §1/§2 keep the plan, as §3 requires | the tree itself; `src/Cargo.toml` |
| RISK-010 | marked **NOT APPLICABLE** with the reason, the correction date, and a pointer to `ci/lint-deps.py` as the check that would make it live again | `docs/adr/0017-framebuffer-shell.md` |
| **not named in the ticket** — the crate count | three documents said the workspace has **19** crates. `src/Cargo.toml` lists **18**. Each place now cites the manifest instead of a number | `src/Cargo.toml`; `ci/lint-deps.py` prints the member list and its groups |

Rather than retarget RISK-010 to the risk the ticket proposed (§5.3), the
version-control gap is recorded as its own entry, **RISK-015**, with the mitigation
that actually happened on the same day: `git init -b main` and a first commit of
637 files. A register that lists only open risks loses the record of how a fixed
one was fixed, and RISK-010 was never the version-control risk.

**Commands run, and their results:**

| Command | Result |
|---|---|
| `python3 ci/lint-doc-refs.py` | `OK: 284 citation(s) across 107 document(s) resolve`, exit 0 |
| `python3 ci/lint-doc-refs.py --self-test` | 3 PASS (fabricated citation flagged, real citation resolves, prose ignored), exit 0 |
| acceptance test 6, run for real: a temporary document under `docs/` citing a markdown path that was never written (named here in prose, not as a path, because a correction that dresses a phantom string up as a citation is the defect it is correcting) | linter exited **1** and named the fabricated path and its line; the plant was then deleted and the linter returned to exit 0 |
| `python3 ci/check-memory-budget.py`, `python3 ci/count-scope.py`, `python3 ci/lint-line-endings.py` | PASS (all three run inside the gate below; count-scope reports the TALLY unchanged) |
| `bash ci/verify.sh --evidence` | **25 pass, 0 fail, 5 skipped**, exit 0; the five skips are the image build and the four boot measurements, which need a Linux host |

**Not verified:** that a reviewer will find the corrected statements convincing;
that is what review is for. Nothing here was checked on a booted image, and none
of these three documents makes a runtime claim after the edits.

## 1. Context from verified repository

Three documents describe an architecture the tree does not have. Each is a
`CONFIRMED_IN_SOURCE` finding from §3.1 of
[the Stage A audit](../../audit/2026-10-11-beta-1.0-stage-a-audit.md).

**Finding 1 — the compositor.**
[`docs/ARCH-00-boundaries.md`](../../ARCH-00-boundaries.md) §5 lists
"Compositor | Litric | Rust | own (Smithay-based)". The tree takes no compositor
dependency: [`src/litric-shell/Cargo.toml`](../../../src/litric-shell/Cargo.toml)
depends only on workspace crates and standard library, the workspace manifest
states that core crates carry zero third-party dependencies, and
[`docs/adr/0017-framebuffer-shell.md`](../../adr/0017-framebuffer-shell.md)
records the accepted alternative: the desktop is drawn in software through
`write()` on the framebuffer, with its own stroke typeface and signed-distance
renderer.

**Finding 2 — directories that do not exist.**
[`docs/REPO-00-layout.md`](../../REPO-00-layout.md) §1/§2 describes
`compositor/`, `shell/panel/`, `defender/onaccess/`, `ai/`, `sdk/`, `gaming/`
and `update/`. None of those paths exist. The real implementation is the Cargo
workspace under `src/`, with the crates listed in `src/Cargo.toml` (18 of them at
the time of writing) also named in
`docs/master-plan/24-feature-parity.md` and
`docs/audit/2026-10-11-beta-1.0-stage-a-audit.md` §2.

**Finding 3 — the risk register describes a risk the project does not carry.**
[`docs/RISK-00.md`](../../RISK-00.md) RISK-010 is "Smithay API Instability",
mitigated by "Pin Smithay version". There is no Smithay version to pin.

**Why this is not cosmetic:** a reviewer reading `ARCH-00` or `REPO-00` will
believe the shell hosts client surfaces. It does not — ADR-0017 states there is
no input protocol, no client application and no window hosting, and
`docs/SYSTEM-SCOPE.md` keeps row 1004 at `T` for that reason. Documentation that
overstates progress is the defect class this repository has already recorded
three times (BUG-0001, BUG-0002, and the GATE 01 entry-1 revocation).

## 2. One objective

Make `ARCH-00`, `REPO-00` and `RISK-00` describe the architecture the tree
actually has, by pointing at the ADRs and `SYSTEM-SCOPE.md` rather than by
restating a plan that was superseded.

**Success means:** each of the three contradictions is gone, every replacement
statement is sourced from a file or ADR that exists, and
`ci/lint-doc-refs.py` still passes.

## 3. Non-goals

- Do not rewrite these documents wholesale. Their value is as design history;
  the smallest honest correction is a corrected row plus a short note saying what
  changed and which ADR superseded it.
- Do not delete the Smithay plan from history. ADR-0005 is the compositor
  decision and ADR-0017 extends it; the layout document may keep a clearly
  marked "planned, not built" section **only if** it says so in those words and
  links ADR-0017.
- Do not touch `docs/SYSTEM-SCOPE.md` — its tally is machine-checked by
  `ci/count-scope.py` and any edit must go through that gate separately.
- No code changes, no unit changes (that is TASK-0001).

## 4. Required invariants

- Every path written in backticks must exist — `ci/lint-doc-refs.py` enforces
  this and will fail the gate otherwise. Verify each path before you cite it.
- LF line endings only.
- No statement may claim runtime behaviour. These are design documents; the only
  verified facts available to them are source facts and booted-VM records, and
  each such claim must name its evidence file.
- Do not invent a new status vocabulary: `SYSTEM-SCOPE.md` uses `D`/`P`/`S`/`T`,
  and `docs/master-plan/24-feature-parity.md` uses `DONE`/`PLANNED`/`NO`.

## 5. Expected implementation

1. `docs/ARCH-00-boundaries.md`: correct the compositor/shell ownership rows,
   name the real mechanism (software framebuffer renderer, ADR-0017) and the real
   process split (shell separate from the `litric-sysd` daemon, ADR-0016), and add
   a dated note stating the Smithay plan was superseded and by which ADR.
2. `docs/REPO-00-layout.md`: replace the directory tree with the real top-level
   layout (`src/`, `image/`, `ci/`, `tools/`, `tests/`, `docs/`, `kernel/`,
   `keys/`, `evidence/`), and either delete the planned-only directories or move
   them into a section explicitly headed as unbuilt, each with the ADR or
   `SYSTEM-SCOPE.md` row that specifies it.
3. `docs/RISK-00.md`: retarget RISK-010 to a risk the project actually carries.
   The candidate, evidenced by the Stage A audit §9, is **"no version control in
   the working copy"** — that is what currently prevents per-patch review and
   rollback. Keep the register's existing format (severity, impact, details,
   mitigation, status) and state honestly that the compromise is unmitigated
   until `.git` is restored.
4. Add one line to each corrected section giving the date and the reason, in the
   style the repository already uses for corrections.

## 6. Acceptance tests

| # | Action | Expected |
|---|---|---|
| 1 | `python3 ci/lint-doc-refs.py` | PASS — every cited path resolves |
| 2 | `python3 ci/lint-doc-refs.py --self-test` | still PASS (the linter's negative control is unchanged and still detects a fabricated citation) |
| 3 | `python3 ci/count-scope.py` | PASS and the TALLY table is unchanged |
| 4 | `python3 ci/lint-line-endings.py` | PASS |
| 5 | grep the three documents for `Smithay` | only in a statement that explicitly marks it as superseded or unbuilt |
| 6 | **negative control** | plant a citation to a fabricated markdown file under docs (a path that does not exist) in one edited file and confirm `ci/lint-doc-refs.py` fails on it; then remove the plant |

Level: `STATIC`. No build and no VM are needed.

## 7. Files allowed to change

- **Allowed:** `docs/ARCH-00-boundaries.md`, `docs/REPO-00-layout.md`,
  `docs/RISK-00.md`.
- **Forbidden:** `docs/SYSTEM-SCOPE.md`, `docs/GATES.md`, `docs/EVIDENCE.md`,
  `ci/**`, `src/**`, `image/**`. `docs/GATES.md` and `docs/EVIDENCE.md` are
  append-only records; corrections to them are a separate, deliberate act.

## 8. Output format

Return, in order: (1) which contradiction each edit resolves and the exact
evidence for it; (2) the minimal edits; (3) the actual diff; (4) the commands you
ran and their results, **or `NOT RUN` if you ran none**; (5) anything you could
not verify; (6) reviewer notes.

Do not state that the linters passed unless you executed them.
