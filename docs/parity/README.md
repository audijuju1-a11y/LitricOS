# The parity ledger

This directory holds the one record of **what LitricOS decided to do about each
surface the UI/UX research catalogued**. It exists because the research corpus in
`Research-Deep-all/` is an inventory of other systems, not a specification, and
an inventory turned straight into code becomes a thousand special cases.

- `ledger.csv` — one row per research item, machine-checked.
- The rules below — enforced by `ci/lint-parity.py`, which runs in `ci/verify.sh`.

The plan this ledger serves is [27 — the plan the UI/UX research
implies](../master-plan/27-research-driven-plan.md) §3.

## The columns

| Column | Meaning |
|---|---|
| `research_id` | The exact id from the coverage checklist, e.g. `WB.04.01`, `MD.11`, `WA.06`. |
| `platform` | `windows-26h2`, `macos-27`, `gnome-51`, … the system the item came from. |
| `surface` | The LitricOS surface that owns the behaviour: `settings/appearance`, `shell/search`, `apps/files`. |
| `decision` | `ADOPT`, `ADAPT`, `REJECT`, `DEFER` — see below. |
| `justification` | One sentence. Mandatory for `ADAPT`, `REJECT` and `DEFER`. |
| `backend` | The API, crate or store that makes it functional. `none` only with `REJECT`/`DEFER`. |
| `store_key` | The `settings.conf` key, when the row is a setting. Empty otherwise. |
| `evidence` | `STATIC: <path>`, `UNIT: <module::test>`, `CONTRACT: <name>`, `VM E2E: <name>`. |
| `milestone` | `M1` … `M6` from the plan. |

## The four decisions

| Decision | Means | Requires |
|---|---|---|
| `ADOPT` | Same behaviour, because nothing about this machine prevents it. | a backend, and evidence |
| `ADAPT` | The same user-visible outcome by different means, because of a real constraint (no DRM, no daemon, no network stack). | a backend **and** a justification that names the constraint |
| `REJECT` | Deliberately not doing it. | a justification |
| `DEFER` | Wanted, blocked on a backend this release does not have. | a justification |

`DEFER` is not a soft `REJECT`: it is a promise with a milestone attached. `M6`
closes the ledger, and a `DEFER` that is still `DEFER` then is a `REJECT` that
has not been honest yet.

## What is not a row

The checklist's 597 items include things that are not surfaces. `CH00` is the
version baseline the research was taken against -- which build of Windows 11,
which release of macOS, which desktop -- and it is verified by the research
itself, not by this system: there is nothing here to adopt, adapt, reject or
defer. `CH00`'s twelve citable items are therefore deliberately absent, and the
linter does not ask for them. The `PENDING` chapters (`F`, `H`, `I`, `J`, and the
eight Linux desktops) are absent for the other reason: research that is not
finished may not be cited, so they become rows when their status does.

## The five rules the checker enforces

1. **The columns are exactly the ten above**, in that order. A ledger that grows
   a column the checker does not know is a ledger the checker is not reading.
2. **The research item must exist and must be finished enough to cite.** The id
   is looked up in `Research-Deep-all/LitricOS Coverage Checklist.csv`, and its
   status must be `COMPLETE` or `PARTIAL`. A `PENDING` item has not been
   researched yet, so a row citing it has no source — it is an idea wearing a
   citation. 184 of the 597 checklist items are `PENDING` today.
3. **No inert control.** `ADOPT` and `ADAPT` must name a `backend` other than
   `none`. This is the gate matrix's own condition, and the reason the shell has
   no Back/Forward buttons that do nothing.
4. **A reason, or it did not happen.** `ADAPT`, `REJECT` and `DEFER` need a
   justification of at least 40 characters: enough to say *why*, not enough to
   write an essay.
5. **The evidence must be findable.** A `STATIC` path must exist in this
   repository; a `UNIT` test name must appear as a `fn` in the tree. This is the
   same defect `ci/lint-doc-refs.py` was written for — a claim whose evidence
   cannot be opened — applied to the parity record.

Two more, because they are cheap and catch real mistakes: `research_id` is
unique (one row per item), and a non-empty `store_key` must be a key the shell's
preference store actually writes. A store key in the ledger that
`src/litric-shell/src/prefstore.rs` never writes is a setting that exists only in
a document.

## What this ledger is not

- **Not a claim of parity with Windows or macOS.** It is evidence of what people
  expect, decided row by row against what this machine can actually do.
- **Not a licence to copy.** Behaviour may be adopted; icons, typefaces, sounds,
  names and layouts that are trade dress may not.
- **Not a plan.** The plan is `docs/master-plan/`; this file records decisions
  the plan has already been through.

## Adding a row

1. Find the id in `Research-Deep-all/LitricOS Coverage Checklist.csv`. If its
   status is `PENDING`, the row cannot be written yet — that is Track R's work.
2. Decide. `ADAPT` is the honest answer far more often than `ADOPT`: a
   framebuffer is not a GPU, and a shell with no daemons is not a shell with
   them.
3. Name the backend that makes it real. If there is none, the decision is
   `DEFER`, not `ADOPT` with a disabled-looking row.
4. Point the evidence at something that exists today.
5. Run `python3 ci/lint-parity.py`.
