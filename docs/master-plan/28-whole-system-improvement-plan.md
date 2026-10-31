# 28 — Whole-system improvement plan

**Created:** 2026-10-29, on `6dd1ae9`.
**Inputs:** [the gate matrix](25-beta-1.0-gate-matrix.md),
[the execution plan](26-execution-plan.md),
[the research-driven plan](27-research-driven-plan.md),
`docs/SYSTEM-SCOPE.md`, `docs/PERF-budget.md`, `docs/OPTIMIZATION.md`,
`docs/parity/ledger.csv`, `docs/GATES.md` and the tracker entries §1–§62.

**Status:** a plan. Nothing on this page is evidence, and no figure here may be
quoted as a result. Numbers marked *(measured)* were produced in this
repository; numbers marked *(target)* are budgets.

---

## 0. What this file is, and what it deliberately is not

There are already three plans, and two matrices. A fourth matrix would drift
from the first three within a week — the same reasoning that keeps
`docs/SYSTEM-SCOPE.md` the single row-per-capability source.

So this file is a **sequencer**, not a fourth source of truth:

| Question | Answered by |
|---|---|
| What capability exists, and in what state? | `docs/SYSTEM-SCOPE.md` (1,165 rows, machine-checked) |
| What blocks Beta, at what evidence level? | [25](25-beta-1.0-gate-matrix.md) |
| What are the engineering streams? | [26](26-execution-plan.md) |
| What does the UI/UX corpus imply, and which rows may be built? | [27](27-research-driven-plan.md) + `docs/parity/ledger.csv` |
| **In what order, with what proof per batch?** | **this file** |

It adds exactly three things the others do not: one measured baseline, one
ordering rule applied to every remaining gap, and one batch contract that every
future batch must satisfy before it may be called done.

---

## 1. Baseline measured on `6dd1ae9` *(measured)*

| Fact | Value | How |
|---|---:|---|
| Rust workspace | 23 crates, **134,768** lines | `find src -name '*.rs' \| xargs cat \| wc -l` |
| Tests | **1,814 passing, 0 failed** | `cargo test --workspace`, three consecutive runs (§62) |
| Scope rows | 1,165: **9 `D` · 116 `P` · 294 `S` · 746 `T`** | `docs/SYSTEM-SCOPE.md` tally, `ci/count-scope.py` |
| Research corpus | 597 checklist items: **57 `COMPLETE` · 528 `PARTIAL` · 9 `NOT FOUND` · 3 `N/A`** | parsed from the coverage CSV today |
| Parity decisions | **391** rows | `docs/parity/ledger.csv` |
| Bug records / ADRs | 36 / 21 | `docs/bugs/`, `docs/adr/` |
| Gate run baseline | **32 pass · 2 fail · 3 skipped** | `bash ci/verify.sh` (§60–§62) |
| Idle RAM | **no measurement**; ledger total 320 MB *(target)* | `docs/PERF-budget.md`; snapshot NOT RUN |
| Desktop frame | 185.3 ms median *(measured 2026-10-12)* against 16.67 ms | `render_bench`, BUG-0015, still open |

Three facts drive the whole ordering below:

1. **The corpus is not finished, and the build may not cite unfinished rows.**
   528 items are `PARTIAL` and 9 are `NOT FOUND`; 27 §5 makes `PENDING = 0` the
   exit criterion of milestone M6, not of M1.
2. **Every image-level gate is blocked on a builder VM, not on code.** No
   `mkosi`, no QEMU here; `docs/GATES.md` GATE 02–13 are `NOT REACHED`. A plan
   that schedules VM work first would stall on line one.
3. **The shell shows more than the backends can back.** Network, Bluetooth and
   audio are `BLOCKED` in the matrix. The on-access Defender unit and slice
   were removed in B3; the only Defender executable is the on-demand scanner.
   Adding surfaces before providers is how a product ends up with controls that
   report success.

---

## 2. The ordering rule

Five rules, applied in this order. When two batches compete, the earlier rule
wins.

1. **Truth before claims.** A row that says `P` while its backend is absent is
   a defect, not a feature. Correcting the claim is first-class work (it is how
   §53–§62 were sequenced).
2. **Data safety and privilege before surface.** Capability grants that do not
   survive a restart, a files socket reachable by the wrong user, a deletion
   path without a reviewed choke point — these outrank any control.
3. **Backend before control.** A settings row may be built only when its store
   key and backend exist; otherwise it stays out of the UI and in the ledger as
   `DEFER`/`REJECT` with the reason.
4. **Research part order governs UI order.** Chapter `F` (shared components)
   before the families that compose them; `G` (accessibility, tokens) before
   breadth; `H` (first boot, defaults) before `WB`/`MD` page families; `L`
   (Linux comparison) after `WB`/`MD` so it corrects rather than seeds; `WB`
   (policy/MDM-only) last because it constrains the others.
5. **Measurement before optimisation.** No RAM, wakeup or frame claim without
   a tool, a command, an environment and n. A saving must name the capability
   it removed (`docs/PERF-budget.md` §7 rule 1).

The consequence is a program that starts with a measurement and claim pass, not
with a feature.

---

## 3. The program

Stages are ordered; batches inside a stage are ordered too. "Here" means this
host (no `mkosi`, no QEMU, no GPU); "builder" means a Linux host with
`tools/provision-builder.sh` applied.

### Stage A — Truth and measurement (blocks every claim)

| Batch | Outcome | Where |
|---|---|---|
| **A1** | One booted desktop measured: `budget-snapshot` taken, `ci/check-memory-snapshot.py` verdict recorded, GATE 02 rows re-tagged from `TARGET` to `MEASURED` or left failing | builder |
| **A2** | The perf harness fails on a planted 15 % regression (its self-test proves it), so W9's acceptance is executable (**landed 2026-10-30**, see tracker A2) | here |
| **A3** | Claim audit: every `P` row whose backend is absent is re-tagged, with the reason in the same commit (**landed 2026-10-30**: five rows retagged `P`→`S`, 117 `P` / 293 `S`; see `docs/audit/2026-10-30-a3-claim-audit.md`) | here |

A1 is the only batch that unblocks the image gates. A2 and A3 are landed; A1
remains open because this host has no builder, QEMU or booted desktop.

### Stage B — Data safety and privilege (P0)

| Batch | Outcome | Where |
|---|---|---|
| **B1** | Capability grants, denials and the audit survive a daemon restart; a corrupt store degrades to *no grants*, never *all grants* | here |
| **B2** | The Files daemon is reached by a per-user socket or a documented, tested equivalent; the 0600 root-owned compromise is retired or its threat-model entry is made explicit (**landed 2026-10-30**: socket owner aligned to `litric`, threat-model entry and `[3i]` negative control added; still no per-peer check, root shell accepted) | here |
| **B3** | Defender either ships the daemon its unit names, or the unit and every claim about on-access scanning are removed until it does (**landed 2026-10-30**: removed the parked on-access service + slice and corrected current claims; `[3l]` has a planted-unit negative control; on-demand scanner only) | here |
| **B4** | The recovery/update generation engine exists for a temp root, with a rollback demonstrated and an ADR-0021 check: no generation advertised before it can roll back | here |

### Stage C — Providers behind the tiles (P0 / `BLOCKED`)

| Batch | Outcome | Where |
|---|---|---|
| **C1** | A provider contract layer (trait + fake) for network, Bluetooth and audio; the shell holds no D-Bus client; a sentinel password never reaches a log (**landed 2026-10-30**: `Provider`/`ToolProvider`/`FakeProvider`, `Sysd::with_provider`, `[3m]` boundary gate with planted `zbus` negative controls; real iwd/BlueZ integration remains absent) | here (contract) |
| **C2** | The KMS output path from ADR-0026 runs: mode enumerated and set, `present=drm` in the status file (**host provider lifecycle, status and refusal path landed 2026-10-31**; a real card is still required for the modeset/flip hop) | here (host) / builder (card) |

### Stage D — Shell correctness, RAM and idle discipline

| Batch | Outcome | Where |
|---|---|---|
| **D1** | Desktop frame measured again and reduced with a named change; BUG-0015 closed or its cost written down per region | here |
| **D2** | Idle wakeups audited against the ≤ 2/s shell target; every timer named, with a reason, in one table | here |
| **D3** | Text atlas and glyph caches measured and capped; bytes per glyph printed by a test | here |
| **D4** | Canvas memory: two of the three full-screen buffers removed or made damage-region caches — a stated capability removal, with the pixel cost measured | here |

Stage D exists because the shell is the largest allocation and the only frame
producer. Its budget is already written: `litric-shell.slice` at
`MemoryHigh=110M` / `MemoryMax=140M`, and `MAX_SURFACE_PIXELS` = 2560×1600 in
`src/litric-shell/src/present.rs` line 60.

### Stage E — Research parity, in corpus order

| Batch | Corpus source | Outcome |
|---|---|---|
| **E1** | Parts 10–18 (`F`) | Tooltips (absent entirely), pickers/views/trees, grids and empty/loading states, progress and validation — one component contract in `src/litric-shell/src/ui.rs`, each with a keyboard path |
| **E2** | Parts 19–20 (`G`) | An accessibility tree with roles, names and focus order; keyboard-only proof for every surface; the unsupported assistive-technology list stated, not omitted |
| **E3** | Parts 21–22 (`H`) | First-boot defaults, permissions, updates, recovery and power surfaces — each only where Stage B/C produced a backend |
| **E4** | Part 23 (`I`) | Technical-feasibility rows either implemented or recorded as `REJECT` with the mechanism that rules them out |
| **E5** | Parts 25–28 (`L`) | Eight Linux desktops compared: adopt at most a named set of behaviours, each with a reason that is not "Windows does it" |
| **E6** | Part 29 (`WB`) | Group-policy/MDM-only rows decided in the ledger before any control is drawn |
| **E7** | `WB`/`MD` families | Settings breadth: registry row + store key + backend + test per control; no inert control ships |
| **E8** | Part 3/6 (`WA`/`MC`) | Shell and desktop-parity items not yet built: search over files, window rules, per-monitor behaviour |

### Stage F — Applications and release

| Batch | Outcome |
|---|---|
| **F1** | App registry and installer over `src/litric-manifest`, with a real install and uninstall in a temp root (**partial 2026-10-30**: §79 adds local bundle install/uninstall; the manifest registry remains open) |
| **F2** | OOBE account creation and password change (the matrix's named `NOT_IMPLEMENTED` gap), with the lock surface unable to be dismissed around it |
| **F3** | M6 close-out: every ledger row decided and evidenced, research `PENDING` = 0, gate-matrix truth pass, release label still `NOT_VERIFIED` unless gates 01–13 each carry the required evidence level |

### Parallel track R — finishing the corpus

Track R (27 §6) continues in every stage: chapters `F`, `G`, `H`, `I`, `J` and
the eight `L` systems, plus the 9 `NOT FOUND` items. The rule is unchanged:
a build row may not cite a `PARTIAL` item as a specification, and may not cite a
`NOT FOUND` item at all.

---

## 4. The RAM and quality program

The user-facing goal is "lightest and most stable", so the budget is treated as
a gate, not a wish. Today's known costs:

| Item | Cost | Status |
|---|---:|---|
| Two full-screen canvases, 1920×1080 | 15.8 MiB raw; 16.8 MiB measured (§67) | `docs/PERF-budget.md` §3d |
| Five with lock caches | 39.6 MiB raw | same |
| Same, at 2560×1600 (the ceiling) | 46.9 / 78.1 MiB | same |
| `litric-shell` cgroup | 110M high / 140M max | configured, not measured |
| Icon name cache | **7,689 B for 24 names** *(measured, §62)* | capped at `MAX_ICON_NAMES` = 512 |
| Icon digest buffer | 4 KiB stack, no heap *(measured, §62)* | 4 files per check |
| Per-frame desktop discovery work | **0** *(measured, §55–§62)* | asserted by tests |

Rules for every batch that touches memory or time:

1. Print the number in the test that measures it; a number in prose alone is not
   a measurement.
2. State what was removed. "Faster" with no named removal is a regression in
   disguise.
3. No new work in the paint path. The §55 "30 frames, no stat" and §56
   "poll only at period" assertions must keep passing.
4. Caps are justified by a measured distribution, not by a round number — the
   way `DIGEST_FULL_MAX` = 64 KiB was justified by 12 real icon files whose
   largest is 18,168 B.
5. A new cap gets a test at the cap, and a test one byte over it.

Levers, in order of expected value (hypotheses, per `docs/PERF-budget.md` §6):

| Lever | Expected effect | What it removes |
|---|---|---|
| One canvas + damage tiles instead of two full-screen buffers | −7.9 MiB at 1080p | one full-screen copy (§67 removed the other); requires damage-region correctness |
| Region blur cache instead of two full-screen passes | the renderer's largest cost (91.2 ms measured) | full-screen blur on unchanged regions |
| Event-driven idle, every timer named | toward ≤ 2 wakeups/s | polling that exists only because it was easy |
| Text atlas cap + per-glyph accounting | bounded, measured | untracked atlas growth |
| Socket-activated daemons stay asleep | 0 MB until first use | always-resident services |
| Kernel module/initrd trim (builder) | boot and image size | unused drivers |

---

## 5. The batch contract

Every batch from §63 onward — and retroactively §53–§62 — must satisfy all nine
lines before it may be called done. This is the pattern that produced §57–§62,
written down so it stops being folklore.

| # | Requirement |
|---|---|
| 1 | **Audit first, with numbers.** The real code path, file→line, and a measured reproduction of the defect before any edit. |
| 2 | **Implement in the real path**, not behind a flag, not in a test-only shim. |
| 3 | **Unit tests that fail without the change**, plus a test at the new cap and one past it. |
| 4 | **A real e2e journey** in `src/litric-shell/src/e2e.rs` over real temporary directories, driving the real loop. |
| 5 | **Sensitivity proof**: force the old behaviour back, paste the real panic with file:line, restore, show green. |
| 6 | **Real visual proof** for anything visible: a PNG under `artifacts/browser-shots/`, opened and inspected, not just counted. |
| 7 | **Measured costs**: µs, bytes, per-frame work (must be 0), and the caps chosen. |
| 8 | **The full gate**: fmt, clippy `-D warnings`, workspace tests, both lints, `ci/verify.sh` at 32/2/3 with no new failures. |
| 9 | **Tracker entry** in the established format, doc-refs green, coherent commit, pushed to `main` with `HEAD == origin/main`. |

---

## 6. The next twelve batches

Ordered by §2. Each is scoped to one working session so it can be proved
end to end rather than half-built.

| § | Batch | Proves | Runs |
|---|---|---|---|
| 63 | Capability grants survive a restart; corrupt store means no grants | restart test, corruption test, audit entry | here |
| 64 | Defender's unit and its claims match reality (daemon or removal) | `lint-unit-binaries.py`, unit starts or claim removed | here |
| 65 | Provider contract layer with a fake; sentinel password never logged | contract tests, redaction test | here |
| 66 | Frame budget: named change, re-measured, BUG-0015 closed or costed | `render_bench` before/after, n=15 | here |
| 67 | Canvas memory: one buffer + damage tiles, bytes measured | frame oracle, pixel diff, bytes | here |
| 68 | Idle wakeup audit: every shell timer named with a reason | timer table, ≤ 2/s target | here |
| 69 | Text atlas measured and capped per glyph | bytes/glyph test | here |
| 70 | Chapter `F`: tooltips + pickers + grids/empty states | keyboard path per component, screenshots | here |
| 71 | Accessibility tree: roles, names, focus order, keyboard-only proof | tree dump test, e2e journey | here |
| 72 | OOBE account + password change, with no dismiss path around the lock | e2e journey over a temp root | here |
| 73 | Settings family wave (research order `WB` → `MD`), ledger-first | **closed at the page/row surface 2026-10-30**: all 15 `settings::Page::ALL` pages render through `views::draw`; every shell-owned registry row is on its registered page; daemon-backed Network/Bluetooth/Sound actions are exposed; the 175 research rows remain honestly split **49 ADAPT · 4 ADOPT · 61 DEFER · 61 REJECT** — closure does not invent the deferred backends (see tracker §81) | here |
| 74 | Recovery/update generation engine core with a demonstrated rollback | temp-root generation swap + revert (**landed 2026-10-30**, 19 tests; see tracker §74) | here |
| 75 | Update service: boot-attempt entrypoint and status/actions over the real socket | `--boot` + socket integration tests on a temp root (**landed 2026-10-30**, 1,917 workspace tests; see tracker §75) | here |
| 76 | Update state crash consistency and one writer per root | atomic record ordering + simulated interruption + concurrent boot/live-lock/reclaim tests (**landed 2026-10-30**, 1,927 workspace tests; see tracker §76) | here |
| 77 | Boot-attested update health evidence and a real commit gate | system-written `booting`/`healthy` record bound to slot/version/attempt + real-socket missing/stale/malformed/failed/replay/concurrency tests (**landed 2026-10-30**, 1,939 workspace tests; see tracker §77) | here |
| 79 | Linux application bundle install, launch and uninstall over the real socket | validated local bundle, generated `.desktop`, real installed program writes a marker, uninstall removes files and discovery entry (**landed 2026-10-30**, 1,958 workspace tests; see tracker §79) | here |
| 80 | Automatic local backup with a checksummed manifest and verification | real timer-created snapshot, independent hashes, retention, source mutation, tamper refusal, `BACKUP LIST` and shell status over a real socket (**landed 2026-10-30**, 1,974 workspace tests; see tracker §80) | here |
| 82 | A real Linux application installs, launches and reaches `views::draw` through the real daemon | genuine Weston `weston-simple-shm` installed via `APP INSTALL`, launched by the shell with `LAUNCHENV`, mapped as a `litric.client` window, pixels asserted through `views::draw`, closed and uninstalled over a real Unix socket (**landed 2026-10-31**, 1,982 workspace tests; see tracker §82) | here |

| 83 | C2 provider lifecycle, card discovery and status contract | actual C helper compiles and refuses a non-card; start/present/state/failure/shutdown protocol tests; card2/3 startup conditions; explicit `drm.mode`/`drm.connector` and `present=none` status (**landed 2026-10-31**, 1,984 workspace tests; real card remains NOT RUN; see tracker §83) | here (host only) |

| 84 | An installed application that exits before mapping is visible | bounded pending-launch check, window cancellation, real-socket unsafe-install/early-exit/stale-launch journey (**landed 2026-10-31**, 1,985 workspace tests; see tracker §84) | here |
| 85 | User-facing Applications install UI | launcher entry, existing-dialog install form, worker with a 15 s UI deadline, distinct empty/missing/unsafe/duplicate outcomes, success notification and immediate discovery, real-socket UI install-and-launch journey (**landed 2026-10-31**, 1,987 workspace tests; see tracker §85) | here |
| 86 | User-facing Applications uninstall UI | installed-tile context menu, existing-dialog confirmation, worker with the same 15 s deadline, distinct not-installed/unsafe/IO/refused outcomes, success notification and immediate discovery, real-socket UI uninstall journey (**landed 2026-10-31**, 1,988 workspace tests; see tracker §86) | here |
| 87 | Uninstall lifecycle converges across shell surfaces | discovered-app dock pins, pin removal on uninstall and discovery, distinct stale-launch sentence, real-socket UI pin/uninstall/stale-launch journey (**landed 2026-10-31**, 1,988 workspace tests; see tracker §87) | here |

Image-level rows (A1, the real-card half of C2, and every `VM E2E` claim) stay
in `docs/GATES.md` as `NOT RUN` until a builder exists. That is a scheduling fact, not an excuse: the
plan is ordered so that nine of the next twelve batches are provable here.

---

## 7. Constraints that do not bend

- No new dependency. No `unsafe` (`forbid(unsafe_code)` in the shell). No new
  work in the paint path.
- No gate, golden frame or key file is edited to make a run pass.
- No control ships without a backend; no backend is faked to justify a control.
- No `VM E2E` or `HARDWARE E2E` claim from a unit test.
- No second source of truth: a matrix row is corrected in the commit that
  changes the fact.
- Caps are measured, printed by a test, and enforced by a test one byte over.

---

## 8. Risks

| Risk | Why it bites | Mitigation |
|---|---|---|
| No builder VM | every image gate stays `NOT RUN` | Stage A1 first; batches chosen so most proof is host-side |
| Surface outruns backend | controls that report success | rule 3 of §2; ledger `DEFER` is a first-class decision |
| Research mistaken for specification | 528 items are `PARTIAL` | Track R; no build row cites an unfinished item |
| RAM work that only moves page cache | `docs/OPTIMIZATION.md` already documents that mistake | measure `MemAvailable`, PSS and cgroup current; never `MemFree` alone |
| Frame work that trades correctness for speed | the damage oracle exists for this | partial-redraw oracle in CI; pixel diff per batch |
| Silent regression in the icon/desktop pipeline | §57–§62 are recent, subtle | keep their tests as the regression net; no weakening |
| Plan drift | four documents, one truth | this file cites, never restates; counts are machine-checked |

---

## 9. How progress is enforced

`bash ci/verify.sh` already runs: fmt, clippy, the workspace suite, the scope
tally (`ci/count-scope.py`), the citation linter (`ci/lint-doc-refs.py`), the
deletion-site review (`ci/lint-file-safety.py`), the parity ledger
(`ci/lint-parity.py`), the memory-ledger checker
(`ci/check-memory-budget.py`), the snapshot checker's self-test, the unit-binary
checker and the golden-frame check. A batch that cannot pass that gate is not
done.

The tracker (`docs/PROGRESS-TRACKER.md`) is the evidence log: one `## N. Batch`
section per completed batch, with the audit, the wiring table, the screenshot,
the gate output and the cost. The plan above never appears as done there; only
the batch that was actually run does.

---

## 10. What this plan refuses to do

- It does not promise a Beta date. The label stays `NOT_VERIFIED` until gates
  01–13 carry the evidence level each row requires.
- It does not schedule work that cannot be proved in the environment that runs
  it, and it does not re-label a blocked row as done because it is visible.
- It does not treat the research corpus as a specification, or copy one
  vendor's model because it is the largest section of the corpus.
- It does not spend effort on sysctls before the shell's buffers, timers and
  caches are measured — `docs/PERF-budget.md` §6 already says where the value
  is.
