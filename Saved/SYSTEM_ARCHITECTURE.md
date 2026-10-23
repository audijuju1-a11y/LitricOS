# LitricOS System Architecture (repository-grounded)

## Scope and evidence convention

This document reconstructs the system present in this checkout, primarily from `src/Cargo.toml`, crate source, systemd/image configuration, ADRs, and the master-plan documents. **Implemented** describes code and wiring present in the tree; it does not imply the relevant service was exercised in a running image. **Specified** means documentation/ADR intent without a complete shipping implementation. Historical measurements remain tied to their recorded environments in `docs/EVIDENCE.md` and `docs/PERF-budget.md`.

## System at a glance

```text
UEFI / firmware
  └─ signed UKI / boot entries / dm-verity image (mkosi + ISO/recovery tooling)
      └─ Linux kernel + systemd image (nano, core, desktop, gaming profiles)
          ├─ litric-sysd (system facts, action/protocol service)
          ├─ litric-envd (environment supervisor loop; state recording)
          └─ litric-shell (software framebuffer desktop, input and UI)
              ├─ litric-input      device discovery → normalized input
              ├─ litric-wm         geometry, focus, workspaces, damage
              ├─ litric-design     themes, contrast/material/type/motion tokens
              ├─ litric-text       OpenType shaping/rasterization; vector fallback
              ├─ litric-files      confined file operations and LFP/1 daemon code
              ├─ litric-browser    policy/session/JPEG+JSON; optional browserd
              ├─ litric-defender   pure scan policy engine (no image daemon yet)
              └─ litric-core/caps/storage/trash/manifest/env/locale

Windows software → litric-winbridge inspection + install plan (not executor)
Security test harness → litric-bench → actual litric-defender engine
```

## Layers and ownership

| Layer | Crates / components | Responsibility and boundary |
|---|---|---|
| Boot/image | `image/`, `iso/`, `kernel/config/`, `keys/`, `tools/` | Build and assemble the bootable image, profiles, systemd units, recovery and development/test image flows. Signing material is not part of source inventory beyond key-management policy and ignore rules. |
| Foundation contracts | `litric-core`, `litric-caps`, `litric-design`, `litric-locale` | Validated IDs/versions/handles/errors/protocol encoding; capability vocabulary and broker decision model; pure UI tokens and contrast math; timezone/locale formatting. These avoid rendering or I/O except locale's explicit filesystem detection API. |
| System APIs / models | `litric-storage`, `litric-trash`, `litric-env`, `litric-wm`, `litric-input` | Logical storage URI authorization/path mapping; trash lifecycle policy; environment state/switch/resource policy; pure window management; normalized device-independent input. Most are decision/model code, not operating-system executors. |
| Runtime / daemons | `litric-files`, `litric-sysd`, `litric-envd` | File operation codecs and Unix service; system service protocol/state/providers; supervisor daemon polling guest exits. Wiring is incomplete: sysd explicitly models trash in memory and environment transitions without starting a VM; envd does not yet implement complete control/surface switching. |
| User applications / compatibility | `litric-shell`, `litric-browser`, `litric-winbridge` | Software-drawn desktop; browser UI/policy plus optional Chromium broker; Windows binary/installer inspection and ordered plan. Winbridge does not execute its plan. |
| Security/testing | `litric-defender`, `litric-bench` | Pure SHA-256/denylist/pattern/cache/verdict engine; deterministic adversarial-but-nonweaponizable benchmark and report tooling. Defender transport/on-access daemon remains roadmap. |
| Typography | `litric-text` | Offline OpenType shaping and rasterized glyph masks, using `fontdue` and `harfrust`; system faces with documented fallbacks. |

## Runtime topology actually represented in code

- `litric-shell` is a single application process. It composes UI into a software canvas and presents via framebuffer `write()`; `fb.rs` documents sysfs geometry and no framebuffer `ioctl`/`mmap`. It consumes `litric-design` and shell state, and has links to the sysd/files/browser services.
- The primary system protocol endpoint is `/run/litric/sysd.sock` (`litric-sysd/src/main.rs`); the shell declares the same path. The shell writes status to `/run/litric/shell.status`. `litric-envd` records guest last-exit text below `/run/litric/env/`.
- Browser ADR-0018 describes the intended socket-activated path and Chromium CDP pipe. The binary in the crate is feature-gated (`broker`) and its current source behavior and unit configuration should be checked against the ADR's verification record before treating that full topology as operational. Its code uses `/run/browserd.sock` as the binary's default, while ADR/image configuration names `/run/litric/browser.sock`; this is a concrete configuration seam to reconcile.
- Files LFP/1 service code is in `litric-files`, while the corresponding units are parked in `image/units-pending/`; their presence does not prove a runnable `litric-filesd` binary exists.
- `litric-defender` is a pure library. Its service/slice definitions are parked under `image/units-pending/`, with no built daemon in the current crate.
- The systemd image includes `litric-sysd` and `litric-shell` service configurations. `litric-envd` service references the supervisor executable; whether its complete expected runtime operations are implemented is distinct from the binary's existence.

## Main data/control boundaries

1. **Boot → root filesystem:** signed boot configuration and verity metadata establish the image integrity path. Recovery/live entries are separate boot targets; the gate matrix records several boot conditions as NOT RUN or failed at its recorded state.
2. **Shell → system daemon:** a Unix socket line protocol carries requests/replies; daemon state uses the capability broker and logical storage model. The daemon must not trust client-supplied authority. Current sysd source documents that peer identity authentication is not yet a complete per-connection design and uses a configured local shell identity for actions.
3. **Shell → files service:** LFP/1 uses bounded, percent-encoded lines and typed error tokens; `litric-files` performs actual Unix file operations. Logical path, symlink, archive and trash rules are covered in ADR-0019 and crate tests; deployment/activation remains separate.
4. **Shell → browser broker → Chromium:** policy/session model and LBP/1 isolate browser-derived JSON/JPEG/CDP handling from the shell. The browser library's `NoEngine` is an explicit honest fallback; a rendering engine is not implied by the model crate.
5. **Shell → input sources:** `InputSource`/normalized `InputEvent` separate raw device events from desktop policy. Scripted sources support tests without hardware.
6. **Environment manager → supervisor/VM:** `litric-env` decides state transitions, active environment, budgets and switch outcome. It is pure and does not launch VMs; `litric-envd` is the process boundary intended to perform supervision, but full KVM lifecycle/display switching is not established by the current model.
7. **App → capability/storage:** the source models identity-anchored grants, scope matching, expiry/revocation and audit decisions, then logical URI namespace and path mapping. Persistence/transport enforcement and actual untrusted application sandbox lifecycle are not implied by these libraries.

## Architectural decisions reflected in the tree

- **ADR-0009:** shell/compositor in one process to constrain idle memory and share scene/text/render state; applications remain separate in the intended future architecture. It also explicitly makes module panic containment and out-of-process parsing requirements.
- **ADR-0015:** LitricOS remains host/control plane; Windows/Linux guests are subordinate; compatibility runtime is preferred for ordinary Windows applications. This is a target and model, not evidence of functioning guest surface switching.
- **ADR-0016/0017:** the current path is a system daemon plus software framebuffer desktop, not the previously envisioned full Smithay/Wayland client desktop. The screen output path is not proof of a production client compositor.
- **ADR-0018/0019/0022/0023:** Chromium is integrated rather than authored; file parsing/extraction stays in the confined service; Defender separates policy from enforcement; providers are being integrated without introducing D-Bus dependencies in core crates.

## Important implementation/specification gaps

- The master-plan architecture describes a fuller production stack (Flatpak, Wayland/Smithay, portals, accessibility tree, broad sandbox mediation) than the implemented framebuffer shell and runtime currently provide.
- Several core models are not yet persistence/execution services: grants/audit are in-memory in `litric-caps`; sysd state says trash is in-memory; env switching changes policy state only; defender has no on-access integration.
- The file/browser service code and the pending/active systemd units must be reconciled with executable names, socket paths, service user, privilege model and protocol versions.
- Image profiles and performance budgets have explicit unmeasured/unimplemented cells. `docs/PERF-budget.md` identifies missing enforcement/budget accounting and recorded renderer results far over frame targets; do not report target numbers as achieved measurements.
- Accessibility remains an explicit release blocker/partial area in the gate matrix; focus and keyboard journeys do not amount to a semantic accessibility tree or assistive-technology support.

## Source map

See `DEPENDENCY_GRAPH.md`, `DATA_FLOWS.md`, `API_AND_INTERFACE_REFERENCE.md`, `CONFIGURATION_REFERENCE.md`, `SECURITY_REVIEW.md`, `TEST_AND_BUILD_MAP.md`, `CHANGE_IMPACT_MAP.md`, and `UNRESOLVED_ITEMS.md` for crate-level edges, protocols, configuration points, risk review, verification and open decisions.