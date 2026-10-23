# Test and Build Map

## Build layout

- Rust workspace root: `src/Cargo.toml`; 19 workspace packages and `src/Cargo.lock`.
- First-party application/system executables include `litric-shell`, `litric-sysd`, `litric-envd`, feature-gated `litric-browserd`, and `litricos-test` from `litric-bench`.
- `tools/build.sh`, `tools/build-image.sh`, `tools/bootstrap-windows.ps1`, `tools/run-qemu.*`, `tools/boot-test.sh`, and `iso/assemble-iso.sh` implement host/build/image/VM flows. Inspect environment preconditions and side effects before invoking: some create image/ISO/UKI/VM artifacts or development keys.
- `ci/verify.sh` is the top-level static/build/test gate orchestration. `ci/` has dependency, docs, mkosi, Kconfig, line-ending, unit-binary, file-safety, memory-budget, golden-frame and Python syntax checks.
- `tests/` contains gate 01 verification, smoke boot, harness integrity, and root-verity udev rule tests. Tools also provide boot, QEMU, framebuffer, photo-tour, UI/performance and browser probes.

## Test layers

| Layer | Representative suites | What a pass establishes | Does not establish |
|---|---|---|---|
| Crate unit tests | Inline `#[cfg(test)]` in Rust crates; `cargo test -p <crate>` | Pure model/parser/codec behavior for exercised cases | Daemon activation, kernel integration, hardware operation, image content |
| Shell integration tests | `src/litric-shell/tests/{design_lint,font_standards,overflow_audit,text_layout_audit}.rs` | Design-token use, type metrics, text overflow/collision audits for declared render fixtures | User acceptance, hardware input, semantic accessibility tree |
| Synthetic E2E | `litric-shell::e2e` journeys | Shell behavior under synthetic event input and model effects | Real device input/service/desktop image behavior |
| Python/script harness tests | `tests/test_harness_integrity.py`, syntax and CI self-tests | Harness invariants and static checks | Actual target image boots or the feature is useful |
| Image/VM gates | `tests/gate01-verify.sh`, `tests/smoke_boot.py`, `tools/boot-test.sh`, QEMU helpers | Boot and serial/system integration if actually executed with fresh artifacts | Bare-metal/hardware compatibility |
| Evidence tours | `tools/photo-tour*.py`, `tools/verify-tour.py`, stored screenshots/logs | Specific recorded interaction/state for the dated artifact | Current code state unless regenerated; backend persistence or security absent from pixels |
| Benchmarks | shell `render_bench`, `measure-idle.py`, `measure-ui.py`, `litricos-test` | Measured behavior only for recorded environment/input/tool | Different host or profile; no extrapolation to hardware without evidence |

## High-value source test areas

- `litric-core`: ID/version/ABI, handles, structured errors, percent encoding.
- `litric-caps`: default denial, scope containment, expiry/revocation, live consent, audit ring, no cross-app grant bleed.
- `litric-storage` / `litric-trash`: malformed URI/traversal/owner/capability checks, collision handling, restore conflicts, retention plans.
- `litric-manifest`: malformed/unknown/duplicate keys/capabilities and bounded lengths.
- `litric-files`: LFP/1 codec, path confinement, archive codecs and extraction safety; review Unix-only test cfgs.
- `litric-browser`: URL/policy/tab model, strict JSON/base64/JPEG fixture decoder, bounds/malformed input; broker tests may need Unix/feature activation.
- `litric-defender` / `litric-bench`: known hash/pattern detection, bounded cache, deterministic corpus, report/forensic/safety behavior.
- `litric-input`, `litric-wm`, `litric-locale`, `litric-env`: hardware-independent model tests; they do not replace device/VM tests.
- `litric-shell`: render/interaction and text audits; test sizes/fixtures may not cover every scale, locale, theme or real screen.

## Evidence interpretation

- `docs/GATES.md`, `docs/EVIDENCE.md`, `docs/master-plan/25-beta-1.0-gate-matrix.md`, bug reports, logs, image tours and `.ppm`/`.png` artifacts include dated results and gaps. Read associated source run metadata for commit, target profile, host/VM, command, sample count and status.
- `docs/PERF-budget.md` explicitly differentiates TARGET, MEASURED and NOT RUN. Several entries are not enforced, and recorded software-renderer timings exceed frame budgets. A static checker or cargo test does not prove runtime resource compliance.
- A historical log is not a current test run. Screenshots are not proof of capability enforcement, data persistence, or non-destructive error behavior.

## Current execution status in this audit

No build, `cargo test`, image build, CI script, QEMU boot or hardware test was run as part of the documentation work. This map reports test locations and expected scope only. Run checks from the actual project/tooling environment and retain exit status, command, environment and logs before marking them passed.