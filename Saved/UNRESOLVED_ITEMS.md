# Unresolved Items and Verification Questions

This list records uncertainty; it is not a list of inferred bugs unless stated. Resolve items against source, actual image manifests, and fresh runtime evidence. Avoid turning planned behavior into a claim while open.

## Runtime wiring and execution

1. **Browser socket path mismatch:** browserd source defaults to `/run/browserd.sock`; ADR/image configuration describes `/run/litric/browser.sock`. Which path is the canonical endpoint, and does the service override the binary default? Confirm unit `Accept=` stdin/stdout behavior and shell connector.
2. **Files daemon executable/units:** `image/units-pending/` includes `litric-files.socket` and instance unit, but is there a `litric-filesd` binary target or an executable wrapper in the current image build? Identify final target, installation path, service user/home, activation and test it before claiming Files service is live.
3. **Capability daemon vs sysd:** older pending `litric-capd` units point at a separate broker, while ADR-0016 moves decisions into `litric-sysd`. Which units are installed, disabled or obsolete in each profile? Confirm no dead units and ensure protocol caller identity is not trusted from a request field.
4. **Sysd caller identity:** current sysd has a configured local shell identity and its docs call out missing peer-credential per-connection authentication. What is the authority model for desktop actions and `RESOLVE`? Must bind SO_PEERCRED/cgroup/executable identity to app ID before any untrusted client can use privileged service.
5. **Environment execution:** `litric-env` and `litric-envd` represent policy/supervision, while sysd says `ENV SWITCH` is state-only. Which KVM lifecycle, surface handoff, hotkey grab, storage isolation, TPM/OVMF setup, licensing, and rollback behaviors exist in the image today?
6. **Browser broker implementation status:** ADR-0018 documents a fuller HELLO/VIEWPORT/tab/input/frame protocol; source and profile have feature gates and evolving subsets. Identify exact shipped protocol version and verify service with real Chromium (including sandbox and CDP pipe) rather than relying on model tests.
7. **Defender activation:** pure Defender core exists; service definitions are parked and no on-access fanotify/BPF pipeline is established. What UI labels truthfully communicate scan-only vs on-access status? Keep always-on protection claims out until verified.
8. **File operations through shell:** shell has direct/host fsbridge logic and a service protocol crate. Which code path does the real booted shell use, and does it cross a privilege-separated socket? Verify `protectHome`/`ProtectSystem` against the actual path.

## Persistence and lifecycle

9. **Capability durability:** grants and audit are in-memory. Choose schema, ownership, atomic writes, retention/purge policy, backup/recovery and corruption handling before claiming grants survive restart.
10. **Trash persistence:** sysd documents in-memory trash, while `litric-files` implements on-disk XDG trash semantics. Which path is authoritative in deployment? Reconcile duplicate state and ensure file+metadata transaction order survives crash.
11. **Settings persistence:** `litric-shell::settings` has typed preferences and save effects, but no complete settings registry/store is evident. Which daemon/file owns serialization, schema migrations, permissions and recovery?
12. **Browser history/bookmarks persistence:** shell effects request persistence; identify backing service/schema and ensure private browsing/purge behavior if persistence is wired.
13. **User accounts/lock/auth:** verify `unix_chkpwd` helper invocation, allowed account, secret stdin handling, lock lifecycle on failure/timeouts, rate limiting, session identity and screen capture/redaction. The UI's presence is not sufficient to claim account/session security.
14. **Storage sandbox enforcement:** `StorageBroker::resolve` maps logical paths and grants; confirm every file-open path uses it or equivalent root-confinement checks, and that caller identity cannot be forged.
15. **Update/recovery:** signed update activation, anti-rollback counter, A/B slot switch, interrupted update, recovery entry and preservation of user data remain largely specified. What is implemented and tested end-to-end?

## Architecture and contracts

16. **Production compositor:** present shell is a software framebuffer output path, while master-plan/ADR docs discuss eventual Smithay/Wayland client support. Make release and product statements explicit about single-surface limitations and client application absence.
17. **Accessibility:** keyboard journeys/focus visuals exist, but no semantic accessibility tree/screen-reader bridge is established. This is a P0 gate in the release matrix; define support claims and missing acceptance tests.
18. **App packaging:** master plan identifies signed Flatpak as the v1 route; `litric-manifest` and `litric-winbridge` are in-tree. Clarify whether the manifest is metadata for Flatpak integration, a private package format, or a future SDK artifact; do not accidentally promote an undeclared second package manager.
19. **Shell module failure containment:** ADR-0009 mandates catch_unwind isolation for module update/draw and core panic-free lints; confirm actual shell code implements those requirements and that panic profile permits unwind. Confirm untrusted decoders stay out-of-process (some software display path code currently draws local assets).
20. **Input unsafe exception:** `litric-input` docs describe EVIOCGABS as the one sanctioned ioctl; reconcile crate-level deny and explicit allow annotations, platform cfg behavior and actual ioctl ABI tests.
21. **Dependency statement:** `litric-text` declares `fontdue`/`harfrust`, contrary to older blanket “zero third-party crates” descriptions. Keep dependency claims scoped by crate/profile and inspect lockfile/transitive packages and license notices.
22. **Profile package truth:** `desktop.conf` declares Chromium and multiple runtime packages; confirm profile resolution and binary installation paths against a fresh built image. `nano`, `core`, `desktop` and `gaming` package declarations alone do not prove installed closure.
23. **Image/kernel security:** determine whether `kernel/config/litric-base.config` is actually applied by image build; validate module signing/LoadPin/LSM ordering, initrd module inclusion, UKI signature and recovery boot. Bug reports document previous failures.
24. **Memory budget:** `docs/PERF-budget.md` explicitly lists many targets not enforced, sysd/envd without budget rows, and recorded render timings over target. Recompute total with current services; obtain runtime PSS/cgroup/CPU/wakeup samples by profile.
25. **ISO/recovery correctness:** verify `iso/assemble-iso.sh` output and boot entries against actual artifacts; gate matrix records unrun conditions and failures. Ensure live-root and recovery targets are not confused.

## Inventory and audit process

26. **Filesystem enumeration freshness:** inventory currently has 838 entries and all listed paths exist at audit time. Re-enumerate before final release; confirm generated Saved reports are intentionally excluded from the source inventory and avoid recursive self-inventory drift.
27. **Evidence integrity:** check SHA manifests and run metadata for binary/frame tours, logs and benchmark result directories. Preserve before/after distinctions and do not call captured UI proof of backend behavior.
28. **Earlier batch reports:** B01–B04 are retained detailed reports from earlier work. Validate their claims/fingerprints against current files if source has changed since their generation.
29. **B05–B08 summary generation:** reports include path hashes/symbol lists, but some static summaries may generalize. Re-check the exact crate sections before relying on a security statement or API signature.
30. **Repository-specific instructions:** if `AGENTS.md`, `CLAUDE.md`, or project-specific audit instructions exist, check them before future modifications; this continuation has not used any external workflow or provider integrations.

## Blocking claim

Until items 1–25 are settled with runtime evidence, treat the whole product as **partially implemented / not verified for release** consistent with `docs/master-plan/25-beta-1.0-gate-matrix.md` (`NOT_VERIFIED`).