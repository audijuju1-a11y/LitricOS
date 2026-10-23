# Configuration Reference

## Configuration surfaces

| Area | Principal files | Purpose / caution |
|---|---|---|
| Rust workspace | `src/Cargo.toml`, `src/Cargo.lock`, `src/clippy.toml`, crate `Cargo.toml` files | Workspace members, compiler/lint settings, build targets, feature gates and resolved dependency graph. `DEPENDENCY_GRAPH.md` is derived from these. |
| Image definition | `image/mkosi.conf`, `image/mkosi.profiles/{nano,core,desktop,gaming}.conf`, `image/mkosi.initrd.conf` | Base package/image settings and profile deltas. A declared package list is not the same as an installed image manifest. |
| Image file staging | `image/mkosi.extra/**`, `image/mkosi.initrd.extra/**`, `image/mkosi.postinst.chroot` | System services, sysctl, sysusers, tmpfiles, udev/network/journald configs, first-run/selftest helpers and post-install changes. Review against unit enablement and package presence. |
| Partitions/repart | `image/mkosi.repart/**`, `image/mkosi.repart-nano/**`, `image/mkosi.profiles/nano/mkosi.repart/**` | ESP, root, verity, signature, var and profile-specific partition layout. |
| Live ISO/boot entries | `iso/assemble-iso.sh`, `iso/hooks/**`, `iso/loader/**` | ISO assembly, live-root service/hook, loader defaults and boot modes (live, safe, noscan, recovery). |
| Kernel | `kernel/config/litric-base.config` | Kconfig fragment; a source config is not proof it was applied to the built kernel. Gate records this as a limitation. |
| Keys | `keys/.gitignore`, `tools/gen-dev-keys.sh` | Development key handling and ignore policy. Never commit generated private signing material; inspect exact script effects before running. |
| CI/dependency rules | `ci/verify.sh`, `ci/lint-*.py`, `ci/dep-policy-allow.txt` | Static validation, dependency policy, line endings, docs refs, mkosi, kernel config, unit binaries, file safety, budgets/golden frames. Tests and self-tests must actually run to establish results. |
| Developer commands | `tools/build.sh`, `tools/build-image.sh`, `tools/run-qemu.*`, `tools/boot-test.sh`, `LitricOS.cmd` | Build/boot/test/image helpers. Scripts can invoke host tools, WSL/QEMU, modify generated artifacts or create key material; inspect before execution. |
| Release/evidence | `docs/GATES.md`, `docs/PERF-budget.md`, `docs/EVIDENCE.md`, `docs/master-plan/25-beta-1.0-gate-matrix.md` | Gate conditions, budget status, measurements and evidence ledger. Targets are not results; retain run environment/date and whether checks are NOT RUN. |

## Image profile summary

- `nano`: serial/CI/embedded floor; comments state no `Packages=` line in the profile and aggressively remove runtime/developer files. It should not be assumed to include desktop, network, or archive tooling.
- `core`: headless base additions include iwd, wireless/regdb, resolvconf, nftables, storage/measurement tools and related packages.
- `desktop`: graphics stack (Mesa/libdrm/libinput/seatd), PipeWire/WirePlumber, BlueZ, fonts, Flatpak/bubblewrap/portals, YARA/security tools, firmware and Chromium are declared; first-party binaries are built/staged separately. Profile comments explicitly acknowledge that package closure/toolkit checks are not fully enforced.
- `gaming`: adds gamescope/GameMode/MangoHud/scheduler tools and states gaming behavior is runtime-only, not a boot-command-line change.

Use exact manifests from a built image for shipped package truth. The profile text alone is a declaration.

## systemd configuration and deployment state

- Active candidates in `image/mkosi.extra/usr/lib/systemd/system/` include `litric-sysd.service`, `litric-shell.service`, `litric-envd.service`, browser socket/service, slices and tuning/first-run units.
- `image/units-pending/` explicitly contains parked capability and Defender units and files service/socket definitions. Do not infer they are installed/enabled. Their targets may not exist yet (see gate matrix, ADR-0016/19/22 and unresolved list).
- Service `MemoryMax`/`MemoryHigh`, user, filesystem protections, socket `Accept=`, credentials, capability sets and restart policy are security/configuration properties. Compare each unit with actual executable and parent slice. `docs/PERF-budget.md` flags several claims/targets not configured and missing resident budget rows.

## Application and UI settings

- `litric-shell::settings::Preferences` models theme/language and selected shell preferences; `Page::ALL` lists 13 Settings pages, which does not prove a persistent settings registry or all controls are live.
- `litric-browser::Policy` holds privacy/navigation settings; `litric-shell::apps` maintains browser/file app models.
- `litric-caps::Broker` grants/audit are in-memory in current implementation. Persistence and authenticated caller identity remain separate unresolved work.
- Regional settings are detected from environment, `/etc/locale.conf`, timezone files and user choice structures; locale is independent from timezone.

## Configuration precedence / trust notes

1. Source code defaults and constants establish model defaults (e.g. sysd socket path).
2. CLI flags and environment variables may override selected developer/runtime paths; consult the target binary's parser before relying on a flag.
3. Image profile, staged files, systemd enablement and postinst actions determine what is actually installed/started.
4. Runtime evidence and service status establish behavior. A config file, service unit, or UI row alone is not proof of functional support.
5. User data/secrets and signing keys require separate persistence and permission design; do not put secret values into tracked config or evidence logs.

See `UNRESOLVED_ITEMS.md` for contradictory/mismatched configuration and claims requiring confirmation.