# 27 — The plan the UI/UX research implies

**Created:** 2026-10-26, on top of `565ea4f`.
**Inputs:** `Research-Deep-all/` — the six research parts, the coverage checklist
and the five registers — read together with the phase pack (`00-MASTER-WORKFLOW.md`
… `15-APPENDIX-*`), [the gate matrix](25-beta-1.0-gate-matrix.md),
[the execution plan](26-execution-plan.md), `docs/DESKTOP-2.0.md` and the
current shell.

**Status:** a plan. Nothing here is evidence, and no row may be quoted as a
result.

---

## 1. What the research actually is

The corpus is an **inventory of other systems' surfaces**, prepared to be
implemented from — not a design document and not a specification.

| File | What it holds | Size |
|---|---|---|
| `Research-Deep-all/LitricOS UI UX Research Part1.md` | baseline versions, master coverage checklist, source plan, validation plan skeleton | 915 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 2 Windows 11 26H2 Settings.md` | Windows Settings catalogue `WB.01`–`WB.12`, persistence map, `ms-settings` URI register | 4,587 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 3  Windows 11 Shell and File Explorer.md` | shell + Explorer catalogue `WA.01`–`WA.16`, shortcut register, Win10/7 deltas | 8,646 lines |
| `Research-Deep-all/LitricOS UI UX Research Part 4  Fluent and WinUI Design System.md` | Fluent token/geometry/material/state rules `WC.01`–`WC.13` | 3,460+ lines |
| `Research-Deep-all/LitricOS UI UX Research Part 5  macOS 27 System Settings.md` | macOS Settings catalogue `MD.01`–`MD.28+` | 2,700+ lines |
| `Research-Deep-all/LitricOS UI UX Research Part 6  macOS 27 Desktop Model.md` | menu bar, Dock, Finder, window model, Spaces, Spotlight, Control Center, dialogs, input, session | 5,726 lines |
| `Research-Deep-all/LitricOS Coverage Checklist.csv` | the ledger behind all of it | 597 items |
| `Research-Deep-all/LitricOS Source Register.csv` | every source URL with tier and status | 3,979 rows |
| `Research-Deep-all/Fluent Token Register.csv` | measured Fluent tokens | 3,553 rows |
| `Research-Deep-all/macOS Design Token Register.csv` | measured macOS tokens | 1,805 rows |
| `Research-Deep-all/Windows Keyboard Shortcut Register.csv` | Windows shortcuts | 365 rows |
| `Research-Deep-all/macOS Keyboard Shortcut Register.csv` | macOS shortcuts | 208 rows |
| `Research-Deep-all/Windows ms-settings URI Register.csv` | Settings deep links | 237 rows |
| `Research-Deep-all/macOS System Settings URL Register.csv` | Settings deep links | 209 rows |

### 1.1 How complete it is — its own numbers

The coverage checklist is the research's own audit, and it is honest about being
unfinished:

| Status | Items | Share |
|---|---:|---:|
| `COMPLETE` | 18 | 3 % |
| `PARTIAL` | 382 | 64 % |
| `PENDING` (not yet researched) | 193 | 32 % |
| `NOT APPLICABLE` | 3 | — |
| `NOT FOUND` | 1 | — |

By section, the shape is: Windows shell (`WA`) 97 partial, Windows Settings
(`WB`) 141 partial, macOS desktop (`MC`) 62 partial, macOS Settings (`MD`) 38
partial, Fluent (`WC`) 12 partial, tokens (`G`) 19 partial — while the whole
Linux-desktop comparison (`LG`–`LR`, eight systems × 15 items = 120), the shared
component catalogue (`F`, 28), workflows (`H`, 21), technical feasibility (`I`,
12) and the final audit (`J`, 9) are **entirely `PENDING`**.

That single table decides the plan's shape: **the research cannot be treated as
a finished specification**, so the build must proceed on the parts that are
settled (Windows shell/settings, macOS desktop/settings, Fluent values) while the
rest is completed in parallel, and no product row may cite a `PENDING` item as
its source.

### 1.2 What the corpus is worth to the build

Three things, and only three:

1. **An inventory of surfaces** — what a person expects to find, page by page
   (`WB.01.01.01` HDR, `MD.10` Accessibility, `WA.07.02` toast lifecycle).
2. **Measured values** — 5,358 design tokens, 573 shortcuts, 446 settings deep
   links. These are numbers, not opinions, and they can be checked.
3. **Behaviour rules** — states, focus visuals, motion timing, empty/error
   states (`WC.10`, `WC.11`, `WC.13`), scroll and grouping behaviour.

What it is **not**: a licence to copy assets, icons, typefaces or trademarks. The
phase pack's hard rule applies unchanged — clean-room behaviour, never
proprietary implementation detail (`00-MASTER-WORKFLOW.md` §8).

---

## 2. The gap this plan closes

| Dimension | Research corpus | LitricOS today | Gap |
|---|---:|---:|---:|
| Settings surfaces | ~237 Windows pages + ~209 macOS panes | 13 pages | ~30× |
| Settings deep links | 446 | 0 | all |
| Shell shortcut actions | 573 across two systems | 36 | ~16× |
| Design tokens | 5,358 | 22 token fields | ~240× |
| Built-in apps | 8 categories in each system | 8 | comparable in count |
| Shell surfaces | ~26 catalogued | desktop, bar, dock, launcher, overview, panels | partial |

The numbers are not a target — a desktop does not need 5,358 tokens or 237 settings
pages. They are a **checklist to decide against**, one row at a time, and the plan
below is built so that every decision is written down rather than implied.

---

## 3. The artefact that makes this tractable: a parity ledger

Building 1,000 controls without a ledger is how a settings application becomes
1,000 special cases. The first deliverable is therefore not a page; it is
`docs/parity/` — one row per research item that LitricOS might implement.

### 3.1 Schema

| Column | Meaning |
|---|---|
| `research_id` | `WB.01.01.03`, `MD.10`, `WA.07.02`, `WC.10`, … the exact checklist id |
| `platform` | `windows-26h2`, `macos-27`, `gnome-51`, … |
| `surface` | the LitricOS surface that would own it (`settings/appearance`, `shell/notifications`, `apps/files`) |
| `decision` | `ADOPT` (same behaviour), `ADAPT` (different because of our constraints), `REJECT` (with a reason), `DEFER` (needs a backend we do not have) |
| `justification` | one sentence. Mandatory for `ADAPT`, `REJECT`, `DEFER` |
| `backend` | the real API or store that makes it functional; `none` is allowed only with `REJECT`/`DEFER` |
| `store_key` | the `settings.conf` / registry key, when it is a setting |
| `evidence` | `STATIC` / `UNIT` / `CONTRACT` / `VM E2E`, with the test name |
| `milestone` | `M1` … `M6` |

Rules:

- A row with `decision = ADOPT` and `backend = none` fails CI. **No inert
  control** — the gate matrix's own condition.
- A `REJECT` without a justification fails CI.
- A `PENDING` research item cannot appear in the ledger at all; it has no
  source. This keeps unfinished research out of the product.
- The ledger is the only place a "we support X" claim may come from.

### 3.2 Where it lives

a machine-checked `ledger.csv` and a human index, both planned under
`docs/parity/`. A new checker, `lint-parity.py` in `ci/` (not written yet),
enforces the three rules above, with a self-test and a negative control, in the
style of the existing `ci/lint-unit-binaries.py`.

---

## 4. Work packages

Each package names the research it draws from, the surface it lands on, and what
proves it. Milestones are in §5.

### WP-01 — Settings foundation (M1)

- **From:** `WB` §11.1 (shell, navigation, search, keyboard), `MD.01` (sidebar),
  `Windows ms-settings URI Register.csv` (deep-link shape).
- **Build:** a typed settings registry (schema version, per-key owner, default,
  validator, store), a searchable index over titles and synonyms, keyboard
  navigation (arrows, Home/End, Enter, Esc), and `litric://settings/<path>` deep
  links that resolve through the registry.
- **Why the registry first:** it is the only way the later page work stays
  affordable; the shell already proved the pattern in
  `src/litric-shell/src/prefstore.rs`.
- **Acceptance:** every page is reachable from the registry; a search for
  "night light", "wallpaper", "wifi" finds the owning page; a deep link opens
  the exact row; a key with no backend cannot be registered.

### WP-02 — Appearance and personalization (M1)

- **From:** `WB.04`, `MD.11`, `MD.14`, `MD.17`, `WA.01.01` (wallpaper, slideshow,
  accent), `WC.02` (colour tokens).
- **Build:** wallpaper sources (image, folder, solid, accent-derived), slideshow
  interval and order, per-monitor later; accent modes (manual + from-wallpaper);
  theme scheduling (light/dark by time); dock/bar size, position, auto-hide.
- **Acceptance:** each row persists across restart; contrast gate still passes
  for every accent × theme pair; slideshow advances under the shell's real timer.

### WP-03 — Display, sound, power (M2)

- **From:** `WB.01.01` (display, HDR, night light, scale), `WB.01.02` (sound,
  volume mixer), `WB.01.05` (power), `MD.15`, `MD.20`.
- **Dependency:** the DRM/KMS decision in
  [the execution plan](26-execution-plan.md) §4 W8. On framebuffer output, scale
  and night light can be real, HDR and refresh rate cannot — and must say so.
- **Acceptance:** every control either changes a measured fact or is labelled
  unavailable with the reason from the probe.

### WP-04 — Notifications, focus and DND (M2)

- **From:** `WA.07` (toast lifecycle, action buttons, history, priority, DND),
  `WB.01.03`, `WB.01.04`, `MD.19`, `MD.21`.
- **Build:** the notifications daemon planned as W3 in the execution plan, with
  the lifecycle states the research documents (banner timeout, action buttons,
  persistence in the centre, per-app priority), plus focus sessions with an end
  time.
- **Acceptance:** a notification raised while the shell is down arrives on
  start; DND suppresses banners and not the centre; every action is keyboard
  reachable.

### WP-05 — Quick settings / Control Centre (M2)

- **From:** `WA.06` (tiles, edit mode, sliders, sub-panels, media), `MD.13`,
  `MC.07`.
- **Build:** volume and brightness sliders over the real providers, network and
  Bluetooth tiles with honest state, media controls driven by the audio
  provider, and edit mode that persists the tile order.
- **Acceptance:** no tile without a provider; dragging a slider changes the
  measured value; tile order survives restart.

### WP-06 — Shell chrome parity (M3)

- **From:** `WA.02` (Start/launcher), `WA.03` (taskbar: grouping, jump lists,
  previews, overflow, badges), `MC.01`, `MC.02`.
- **Rule:** this is where `ADAPT`/`REJECT` will be most common. No live window
  thumbnails until the canvas has render-to-texture — the shell already refuses
  to fake them (`docs/DESKTOP-2.0.md` §1).
- **Acceptance:** grouping, jump lists, badges and overflow each have a written
  decision; whatever is adopted has a test at the shell level.

### WP-07 — Window management (M3)

- **From:** `WA.09`, `MC.04`, `MC.05`, and `docs/DESKTOP-2.0.md` §5 item 1.
- **Build:** snap layouts picker, per-application window cycling, stage-manager
  style set, rules engine for placement.
- **Acceptance:** the picker produces the geometry the manager already supports;
  keyboard-only path exists for every placement.

### WP-08 — Files and Explorer parity (M3)

- **From:** `WA.13`, `WA.14` (dialogs), `MC.03` (Finder).
- **Build:** tabs, breadcrumb editing, drag and drop, details pane, properties
  sheet, sort/filter, archive operations, conflict dialogs — over the files
  daemon from W1 of the execution plan, never over in-process writes.
- **Acceptance:** every operation crosses the daemon; a failure shows the real
  errno class; conflicts never overwrite silently.

### WP-09 — OOBE, accounts, lock (M4)

- **From:** `WA.10`, `WA.11`, `MD.23`, `MD.25`, `MD.26`.
- **Build:** first-run parity beyond the current wizard (language, account,
  password, privacy choices, network), lock screen options, session switching.
- **Acceptance:** a clean image reaches the desktop through the OOBE without a
  terminal; the lock cannot be bypassed; no default credential exists.

### WP-10 — Accessibility and input (M4)

- **From:** `MD.10` (the longest single pane in the corpus), the accessibility
  section of `WA.09`, `G` token/accessibility rows, `H` workflows (needs Track R).
- **Build:** a semantic accessibility tree, a screen-reader bridge decision,
  keyboard-only path per surface, IME/Thai typing, sticky keys, zoom, colour
  filters, reduced motion honouring `prefers-reduced-motion` equivalent.
- **Acceptance:** "every user-facing action is reachable by keyboard alone"
  (GATE 06) becomes a test, not a claim.

### WP-11 — Design system consolidation (M1, then continuous)

- **From:** `WC.01`–`WC.13`, both token registers, `G`.
- **Build:** map the 5,358 measured tokens to the 22 LitricOS token fields with
  a written decision per family (adopt value, adapt to our scale, ignore with
  reason); add the control states, focus visuals and loading/empty/error
  patterns the research found; a golden frame per surface.
- **Acceptance:** no shell colour/space/radius literal outside the token file
  (CI check); every new surface ships a golden frame in the same commit.

### WP-12 — System tools and permission prompts (M5)

- **From:** `WA.12` (UAC analogue), `WA.14` (common dialogs), `WA.15` (system
  tools: Task Manager, Terminal, clipboard history).
- **Build:** a capability consent prompt modelled on UAC's structure but backed
  by `litric-caps`, dialog components, clipboard history over the real store.
- **Acceptance:** a denied capability produces a prompt that names the app, the
  capability and the consequence; allow-once / allow-always / deny are all
  real decisions.

---

## 5. Milestones

| Milestone | Contents | Exit criterion | Needs builder VM |
|---|---|---|---|
| **M1** | WP-01, WP-02, WP-11 foundation | registry + search + deep links + appearance rows, all persisted and contrast-gated | no |
| **M2** | WP-04, WP-05, WP-03 | notifications daemon, control centre over real providers | partly (sliders need a guest) |
| **M3** | WP-06, WP-07, WP-08 | chrome, window management and Files over the files daemon | partly |
| **M4** | WP-09, WP-10 | OOBE + accessibility tree + keyboard-only proof | yes |
| **M5** | WP-12 + remaining settings families | capability prompts and tools | yes |
| **M6** | research close-out + parity audit | every ledger row decided and evidenced; research `PENDING` = 0 | no |

Milestones are ordered by dependency, not by how visible they are. M1 is first
because every later row depends on the registry existing.

---

## 6. Track R — finishing the research

The corpus is 32 % unresearched and 64 % partial. The build may not cite it, so
completion is work, scheduled beside the build:

| Batch | Items | Why it matters to the build | Milestone |
|---|---:|---|---|
| `F` shared component catalogue | 28 | WP-11 needs a decision per component, not per pixel | M2 |
| `G` tokens & accessibility | 19 (partial) | WP-11, WP-10 | M2 |
| `H` workflows & edge cases | 21 | WP-04/07/08 flows; without it, states are invented | M3 |
| `I` technical feasibility | 12 | converts research rows into LitricOS constraints before they reach the ledger | M1 (early) |
| `LG`–`LR` Linux desktops (8 × 15) | 120 | distinguishes "everyone does this" from "Windows does this" — the main defence against copying one vendor | M4 |
| `J` final audit | 9 | closes the checklist and proves the registers | M6 |
| `WB`/`WA`/`MC`/`MD` partial → complete | 338 | the rows the build actually consumes | rolling |

Each batch produces ledger-ready rows, not prose: a research batch is done when
its checklist status is final and its rows carry evidence tiers.

---

## 7. Governance

1. **One source of truth per claim.** A surface's behaviour is described in the
   ledger row; the research file is the evidence, not the requirement.
2. **No inert control.** The planned parity checker fails an `ADOPT` row with
   no backend, and the gate runs it with a self-test once it exists.
3. **No token literal outside the token file.** A new CI check extends the
   existing design lint to the shell.
4. **Every surface ships with a golden frame** in the same commit, so visual
   regressions are caught by pixels and not by memory.
5. **No copy of assets.** Behaviour may be adopted; icons, typefaces, sounds,
   names and layouts that are trade dress are not.
6. **The matrix row is corrected in the commit that changes the fact.** The
   gate matrix already carries two stale rows (lock/auth, browser); drift is a
   defect.

---

## 8. Risks

| Risk | Why it bites | Mitigation |
|---|---|---|
| 1,000-control scope | every row looks small; together they are years | the ledger's `DEFER`/`REJECT` decisions are first-class, and M1 ships the registry before any family |
| copying one vendor's model | the corpus is 60 % Windows by volume | Track R's Linux batch, and an explicit rule that a behaviour needs a reason beyond "Windows does it" |
| research mistaken for spec | `WB`/`WA` are `PARTIAL` by their own audit | `PENDING` rows are barred from the ledger; `PARTIAL` rows need a second source before `ADOPT` |
| no builder VM | no gate can move | M1 is builder-free by design; M2+ items are split into "code here / verify there" |
| Thai typography and IME | the research corpus is English-centric | WP-10 treats Thai as a first-class case; the existing text audit stays a gate |
| design tokens as decoration | 5,358 values with no mapping is noise | WP-11 requires a decision per token family, not per token |

---

## 9. First three sessions

1. **Session 1 — the ledger and its checker.** The schema document, a seeded
   ledger covering the 13 existing settings pages and the shell surfaces that
   already exist, the `lint-parity.py` checker with its self-test, and the two
   stale gate-matrix rows corrected. None of these exists yet; this session
   creates them.
2. **Session 2 — settings registry + search + deep links (WP-01).** Typed
   registry over `prefstore`, search index with synonyms, keyboard navigation,
   `litric://settings/...` resolution, tests for each.
3. **Session 3 — appearance family (WP-02).** Wallpaper sources and slideshow,
   accent from wallpaper, theme schedule, dock size/position — every row recorded
   in the ledger with its store key and test.

### 9.1 What has landed since this plan was written

| Session | State | Where |
|---|---|---|
| 1 — ledger and checker | **done** (2026-10-26): 41 seeded rows, `ci/lint-parity.py` with a 16-case self-test, wired into `ci/verify.sh`; the three stale gate-matrix rows corrected | `docs/parity/ledger.csv`, `ci/lint-parity.py` |
| 2 — registry, search, deep links | **done**: 7 rows, a two-way test against the store, `litric://settings/<page>/<key>`, and the launcher search reaches rows as well as pages | `src/litric-shell/src/registry.rs` |
| 3 — appearance family | **partial**: the automatic light/dark schedule is in (`theme_schedule`, resolved on every tick); wallpaper sources, slideshow, accent-from-wallpaper and dock size remain `DEFER` in the ledger | `src/litric-shell/src/settings.rs` |
| 4 — browser controls | **done**: Stop and Reload are in the toolbar over the engine's own `STOP`/`RELOAD`, each disabled when it would act on nothing; the broker reports `stopped` instead of `loaded` when a stop interrupted a page | `src/litric-shell/src/views.rs`, `src/litric-browser/src/bin/browserd.rs` |
| 5 — quick settings volume | **done** (WP-05, first control): the panel reads the sink's level through `litric-sysd` (`AUDIO LEVEL`, read back after every change) and offers quieter/louder/mute; the two `DEFER` rows it answers (`WB.01.02`, `WA.06`) are `ADAPT` with the constraint written down | `src/litric-sysd/src/providers.rs`, `src/litric-shell/src/main.rs` |

Session 3 is partial on purpose: the three rows it deferred are deferred for a
reason the ledger records, not because there was no time. Slideshow and
per-monitor wallpaper need an image decoder and a monitor model; dock geometry is
one measured strip inside the layout pass.

## 10. What this plan refuses to do

- It does not declare parity with Windows or macOS. It uses them as evidence of
  what people expect.
- It does not implement a control whose backend does not exist, and does not
  hide a missing backend behind a disabled-looking row.
- It does not treat the research as complete. 193 items are unread, and the
  build may not use them until they are not.
