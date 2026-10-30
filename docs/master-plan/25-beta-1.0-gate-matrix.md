# 25 — Beta 1.0 gate matrix (whole-system view)

**Created:** 2026-10-11, from the Stage A audit
([docs/audit/2026-10-11-beta-1.0-stage-a-audit.md](../audit/2026-10-11-beta-1.0-stage-a-audit.md))
and the Stage B research ([docs/research/platform-parity.md](../research/platform-parity.md)).

## The single-source-of-truth rule

**[`docs/SYSTEM-SCOPE.md`](../SYSTEM-SCOPE.md) holds the authoritative row per
capability — 1,165 rows**, parsed and enforced by `ci/count-scope.py`. This file
does **not** re-list them. It adds the three things `SYSTEM-SCOPE.md` does not
carry: the **Beta 1.0 release priority** per area, the **blocker** that stops a
row moving up, and the **evidence level** required before a row may be called
verified. A second row-by-row matrix is forbidden here on purpose: two matrices
drift, and a drifted matrix is how a project believes it is further along than
it is (the same reasoning `SYSTEM-SCOPE.md` gives for machine-checking its own
tally).

Status vocabulary is mapped once, in
[docs/research/platform-parity.md](../research/platform-parity.md) §2. The
release-status labels available for a *milestone* claim are only:
`READY_FOR_BETA`, `CONDITIONALLY_READY`, `NOT_READY`, `NOT_VERIFIED`.

## Priority definitions used below

- **P0** — a Beta release blocker. Boot / install / login / desktop / core input
  and output / user data / security / update / recovery that is in the declared
  scope must not fake its state, and must have evidence at the level the claim
  needs.
- **P1** — Beta quality target. Makes daily use smooth (search, workspaces, app
  discovery, diagnostics, multi-monitor) without endangering P0.
- **P2** — post-Beta roadmap. Expensive or ecosystem/driver/licence dependent.

`P1`/`P2` may never be used to postpone a security or data-loss defect that is
P0, and a P0 capability that cannot be built must have its *claim* reduced and
its limitation written down — not replaced by a control that reports success.

---

## 1. Coverage by area

`Rows` are `SYSTEM-SCOPE.md` row numbers/parts (verified counts from the file's
own headings). `In-tree evidence` is what exists **today** in this checkout.

| Area | Rows | In-tree evidence today | Priority | Beta status | Blocker |
|---|---|---:|---|---|---|
| Boot chain, UKI, dm-verity, serial console | A1 (1–18) | `image/mkosi.conf`, `image/mkosi.repart/`; GATE 01 boot log | **P0** | `INHERITED_FROM_BASE_OS` + `RUNTIME_VERIFIED` (VM, 2026-08-28) | conditions 2,3,5,6,8 of GATE 01 `NOT RUN`; kernel config fragment never applied (row 19 is `T`) |
| Installer / OOBE / first boot | A1 3–9, D5 444–449, Pack §9.2 | **nothing** | **P0** | `NOT_IMPLEMENTED` | no installer pipeline at all; must not be faked as "coming later" |
| Desktop shell: system bar, dock, launcher, menus | B1 (79–106), B4 173/176 | `src/litric-shell` (547 tests), 14 synthetic-input E2E journeys | **P0** | `PARTIAL` | no client application can draw into the desktop (ADR-0017); `litric-sysd` is the only data source |
| Window management | B2 (107–148) | `src/litric-wm` (45 tests) | **P0** | `PARTIAL` | snapping is half/quarters only; no previews, no overview, no rules engine |
| Workspaces / multi-monitor | B3 (149–172) | `src/litric-wm` | P1 | `PARTIAL` | no output enumeration; single framebuffer |
| Theme, tokens, motion, contrast | B7 (241–264) | `src/litric-design` (40 tests), `src/litric-shell/src/desktop.rs`, `docs/DESIGN-SYSTEM-v3/` | **P0** | `PARTIAL` | accent colour, a three-way wallpaper choice and a light/dark schedule that follows the local clock are persisted settings with contrast gates (2026-10-25, 2026-10-26); there is no image picker, no slideshow, no per-monitor wallpaper and no accent extracted from the wallpaper |
| Settings application and registry | D1–D6 (325–468) | `src/litric-shell/src/prefstore.rs` (versioned record, atomic write); `src/litric-shell/src/registry.rs` (7 rows, deep links, store-key cross-check); `docs/parity/ledger.csv` (41 decisions); no `litric-settingsd` | **P0** | `PARTIAL` | the shell's own preferences have a schema, an atomic persistence path under `/var/lib/litric-shell`, a registry that the search and `litric://settings/...` links resolve through, and a ledger that fails CI on an inert row (2026-10-26); settings the shell does not own have no daemon, and 13 pages exist against the ~446 the research catalogues |
| Notifications, DND, quick settings | B5 (199–218), B1 93–100 | `src/litric-notifyd` (queue, socket, persistence, 53 tests); shell panels; the shell takes the queue at startup and every 2 s | P1 | `PARTIAL` | notifications survive a shell restart and a reboot since 2026-10-26, but there is no live push, no action buttons and no badge model; quick settings still have no provider behind volume, brightness, network or Bluetooth |
| Files: operations, conflicts, trash, archives | C1 (265–312), C2 (313–324) | `src/litric-files` (111 lib tests, 8 bin tests, 3 tests against the built binary over stdio), `src/litric-filesd` (the daemon binary), `src/litric-shell/src/fsclient.rs` (11 tests, one of them a whole Files session over a real socket), `src/litric-trash` (17), `src/litric-storage` (21) | **P0** | `PARTIAL` | since 2026-10-26 the Files window performs every operation over LFP/1 against `litric-filesd`: the daemon creates the storage root, the shell holds no writable path to it, and the socket unit is installed and enabled. Proved on a host over a real socket, **not** in a booted image; the daemon runs as `litric` and the 0600 socket is now owned by `litric`; root can still connect through its DAC override, and there is still no per-connection peer-credential check |
| Search / indexing | B4 (179–198) | nothing | P1 | `NOT_IMPLEMENTED` | no indexer |
| Network: Wi-Fi / Ethernet | D3 (375–400) | **nothing**; no D-Bus dependency in any crate | **P0** | `BLOCKED` | needs NetworkManager integration **plus** a secret-agent design; image profile contents not yet verified |
| Bluetooth | D4 (401–409) | nothing | **P0** | `BLOCKED` | needs BlueZ + an Agent implementation for pairing |
| Audio | D2 (359–374) | nothing | **P0** | `BLOCKED` | needs PipeWire/WirePlumber; ADR-0006 describes the intended stack, no code |
| Display configuration | D1 (325–340) | framebuffer geometry from sysfs (ADR-0017) | P1 | `PARTIAL` (read-only) | mode setting needs ioctl/DRM, which `unsafe_code = "forbid"` rules out today — this is an architecture decision, not a task |
| Power / battery / lifecycle | D1 (343–351), Pack §12 | `src/litric-sysd` reads battery facts | P1 | `PARTIAL` | no suspend/resume, no profiles; hibernation must stay marked unavailable |
| Login, lock, unlock, password change | B6 (226–239) | `src/litric-shell/src/lock.rs` (lock surface); `litric-sysd` `AUTH VERIFY` through the distribution's `unix_chkpwd` | **P0** | `PARTIAL` | the verifier is real and the surface cannot be dismissed without it, but no booted image has been locked and unlocked; password change, user switching and an OOBE account path do not exist (2026-10-26) |
| Users, accounts, sessions | D5 (444–449) | nothing | **P0** | `NOT_IMPLEMENTED` | depends on the auth design above |
| Capability broker, permissions | E1 (469–502) | `src/litric-caps` (43 tests), `litric-sysd` `RESOLVE` | **P0** | `PARTIAL` | grants are in-memory and do not survive reboot (ADR-0016) |
| Code integrity, secure boot, verity | E2 (503–528), A1 11 | dm-verity in the image; `keys/` dev keys | **P0** | `PARTIAL` | Secure Boot boot never exercised (GATE 01 condition 8 `NOT RUN`) |
| Defender | E3 (529–546) | `src/litric-defender` (35 lib tests), `Aho-Corasick` + verdict ladder, and an on-demand scanner binary (§64) | **P0** | `BACKEND_ONLY` | there is no on-access hook; B3 removed the parked `litric-defender.service` and `.slice` because the binary implements neither `--rules` nor `Type=notify` and no fanotify transport exists |
| Updates, rollback, recovery | A1 2–10, E2, Pack §12 | ADR-0021 design; nothing executable | **P0** | `NOT_IMPLEMENTED` | must not be advertised until the generation engine exists |
| Applications: built-ins, platform, packages | F1 (547–590), F2 (591–616) | `src/litric-manifest`, `src/litric-browser` (92 tests with `--features broker`), `src/litric-winbridge` (49) | P1 | `PARTIAL` | no app registry, no store, no installer |
| Windows compatibility | F3 (617–646) | `src/litric-winbridge` planning only | P2 | `PARTIAL` | deliberately not a Beta claim |
| Input, keyboard, IME, Thai | G1 (647–670), 652 | `src/litric-input` (70 tests), `src/litric-locale` (43) | **P0** | `PARTIAL` | Thai word breaking is spec'd, not built; keyboard layouts exist without a settings surface |
| Accessibility | G2 (671–696+) | focus ring, high-contrast tokens, reduced motion | **P0** | `PARTIAL` | no accessibility tree, no screen reader, no on-screen keyboard — must be labelled unsupported, not omitted |
| Diagnostics, logs, crash support | Pack §A17 | `litric-sysd` facts, `/dev/kmsg` spinners | P1 | `PARTIAL` | no journal viewer, no redacted export path |
| Performance budgets | `docs/PERF-budget.md` | budgets defined | **P0** | **`FAIL` as recorded** | boot 9.17 s vs ≤ 1.2 s, image 489 MB vs ≤ 90 MB, 140 procs vs ≤ 32 |

---

## 2. Evidence level required before a row may be called verified

The prompt set insists the level be named. For this project the levels and their
minimum bar are:

| Level | Minimum bar | Applies to |
|---|---|---|
| `STATIC` | a linter in `ci/` **with a `--self-test` or an equivalent negative control** has run and passed | every wave |
| `UNIT` | `cargo test` for the crate, run and reported | every code slice |
| `CONTRACT` | the provider's typed API exercised against a fake that implements the same trait | network, Bluetooth, audio, settings |
| `INTEGRATION` | the real service reachable in a safe environment | network, Bluetooth, audio, power |
| `VM E2E` | an image built and booted, driven through the hypervisor's real input | boot, shell, files, lock |
| `HARDWARE E2E` | a named device, driver and build | Wi-Fi, Bluetooth, audio, display, suspend |

A `UNIT` result may never be reported as `VM E2E`, and a mock provider's success
is never evidence of hardware support.

---

## 3. Wave plan, with the gate that must pass before the next wave starts

| Wave | Work | Exit gate | Status today |
|---|---|---|---|
| **0** | Stage A audit, Stage B research, single-source-of-truth rule, evidence vocabulary | gate 0: architecture facts source-grounded; unknowns explicit; every requested capability has UI/backend/persistence/test status | **partially done** — audit written, but no build or test was run, and version control is missing |
| **1** | Remove the dead artifacts the audit found; correct the documentation contradictions; add the missing dependency-policy check | gate 1: `ci/verify.sh` green **and** a new check with a negative control proves a core crate cannot gain a third-party dependency | **not started**; tickets written, see §4 |
| **2** | Appearance vertical slice: theme/accent/wallpaper tokens → settings store → persistence → shell propagation | gate 2: change a setting, see the shell change, restart the shell, see it survive; contrast measured | blocked behind Wave 1 and behind the settings store decision |
| **3** | Network vertical slice | gate 3: contract tests with a fake provider, then integration; credential redaction proven with a sentinel password | **PARTIAL** — `litric-sysd` answers network actions through the C1 `Provider` contract, whose `FakeProvider` is exercised without tools; credential redaction is proven with a sentinel over the real socket (PROGRESS-TRACKER §65); a real iwd/BlueZ integration and the secret-agent design the wave is gated on are still absent |
| **4** | Bluetooth vertical slice | gate 4: paired ≠ connected proven; pairing cancel and daemon restart handled | **BLOCKED** — same |
| **5** | Boot / login / lock / password | gate 5: security review + real PAM-backed auth integration; lock cannot be bypassed | **BLOCKED** — needs a privileged helper design and a threat-model update first |
| **6** | Display / sound / input / power / time / language | gate 6: one contract row per setting; no inert control exists | not started |
| **7** | Shell completeness (launcher search, control centre, notifications, workspaces) | gate 7: a typed input journey with a real effect, plus keyboard-only path | not started |
| **8** | Security, update, recovery, diagnostics | gate 8: rollback/recovery demonstrated in a disposable VM | not started |
| **9** | Release candidate | gate 9: full approved suite + measurements with methodology | not started |

Waves 3, 4 and 5 are deliberately gated on **design**, not on code. The prompt
set's own rule applies: a feature that needs a new privileged service, an
auth-stack change or a boot-chain change gets a design note, threat model and
rollback plan first — and until then the capability stays out of the UI rather
than appearing as a control that reports success.

---

## 4. Delegation status: what can actually be dispatched to DeepSeek

The workflow the prompt set describes (Claude orchestrates, DeepSeek
implements) assumes a route to invoke the model. **This environment has no such
route**: there is no DeepSeek model-invocation tool, no API key configured for
one, and no network path exposed to the agent for that purpose. Saying otherwise
would be exactly the kind of false claim the prompt set forbids.

Therefore the delegation step is **manual**: ready-to-copy, fully-scoped tickets
are written to `docs/ai-prompts/deepseek/`, and the human carries them to
DeepSeek. When a ticket's answer comes back it is treated as a **proposal**:
symbols are checked against the tree, the diff is inspected, tests are re-run
here, and the evidence ledger is updated — none of which is waived because a
model wrote it.

Three tickets are ready:

| Ticket | Objective | Why it is first |
|---|---|---|
| `TASK-0001` | Delete the three systemd units whose binaries do not exist, and make `ci/verify.sh` fail if that recurs | smallest possible change with a permanent regression guard; zero dependencies |
| `TASK-0002` | Correct the three documentation contradictions found in Stage A | reviewers currently describe a compositor that does not exist |
| `TASK-0003` | Add a dependency-policy check: a core crate acquiring a third-party dependency must fail the gate, with a negative control | the workspace's headline constraint is enforced by convention only |

Every later ticket depends on work that does not exist yet (a settings store, a
provider contract, an auth helper). Writing those tickets now would produce
prompts full of invented symbol names, which the prompt set explicitly forbids.

---

## 5. Current release status

**`NOT_VERIFIED`.**

Reasons, stated rather than summarised:

- No build and no test ran in this pass; the last recorded result (941 tests) is
  `HISTORICAL`.
- GATE 01 carries five `NOT RUN` conditions and one `FAIL`; GATES 02–13 are
  `NOT REACHED`.
- Bare-metal E2E has never been performed; every boot is QEMU.
- The P0 areas of login/lock/authentication, installer, updates and recovery do
  not exist, and network/Bluetooth/audio are blocked on design.
- Performance claims currently fail their own budgets, so a measurement-backed
  Beta claim is not available.
