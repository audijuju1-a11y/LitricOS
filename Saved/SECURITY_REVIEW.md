# Security Review (repository-grounded)

## Executive assessment

The repository contains thoughtful, testable security models: deny-by-default capabilities, validated application identity, scoped storage URIs, strict bounded parsers, reversible trash planning, isolation policy, browser privacy decisions, and separation of decision from execution. **Those models do not collectively constitute an end-to-end enforced security system yet.** Several critical controls remain in-memory, incomplete, parked in pending unit files, or planned only. The Beta gate matrix records multiple P0 areas as partial, blocked, not implemented or not verified. Use `docs/master-plan/25-beta-1.0-gate-matrix.md`, `docs/GATES.md` and evidence ledgers rather than inferring deployment from a library test.

## Trust boundaries and assets

Assets include user files and app data, credentials, signing/recovery keys, boot/system integrity, permissions/audit history, microphone/camera and display/session state. Relevant boundaries:

1. firmware/UKI/kernel/rootfs integrity;
2. shell ↔ privileged system daemon;
3. shell ↔ file/browser helper;
4. app identity ↔ capability/storage broker;
5. host ↔ compatibility/guest execution;
6. untrusted bytes ↔ parser/decoder;
7. generated evidence ↔ release claims.

The threat model and master plan enumerate malicious apps/files, network/USB/physical attackers, compromised updates and prompt/model attacks. Controls must be placed at enforcement boundary, not assumed from UI.

## Controls present in source

### Identity and capability decisions

- `AppId` validates a canonical reverse-DNS-like identity once, with bounded labels and reserved platform namespace support.
- Capability vocabulary has typed IDs, risk/scoping, URI/host/resource scopes, and explicit `authorises` matching.
- `Broker` defaults to no grants, scopes authorization, applies expiry and revocation on checks, distinguishes live-consent requirements and records allow and deny decisions in a bounded in-memory audit buffer.
- Critical gap: broker grants/audit are in-memory; daemon caller identity authentication is not established end-to-end. `litric-sysd::Request::Resolve` includes an app field, and current source notes a configured local client identity rather than peer credential validation for all connections. Do not trust arbitrary request identity as authority.

### Storage and file safety

- Logical URI namespaces avoid making physical paths an app contract. Traversal, controls, backslashes, empty/oversized segments and cross-app private ownership are rejected.
- `secure://` does not resolve to an ordinary filesystem path. Shared user/media storage needs capability grants.
- `litric-files` source/ADR describes root-confined Unix operations, no-follow symlink behavior, typed errors and bounded archive extraction. ADR-0019 acknowledges check-then-act TOCTOU residual risk and the need for `openat`/`O_NOFOLLOW`-style hardening.
- Trash state policy avoids silent collisions/overwrite and makes automatic empty plans inspectable. Persistence/daemon integration and crash-atomic transaction semantics require further validation.

### Parsers and protocol limits

- Manifest parser has 64 KiB/2000-line/64-cap bounds, strict sections/keys, duplicate rejection, unknown capability rejection, and entry path checks.
- Sysd and LFP/1 line parsers bound input, enforce arity/UTF-8/control rules and use stable errors. Shared percent decoding is intended to avoid parser disagreement.
- Browser JSON/JPEG decoders have depth, dimension and pixel limits; browserd isolates web-derived parsing from shell. JPEG supports a subset and rejects unsupported modes explicitly.
- ZIP/TAR/GZIP and file formats are attack surface; keep them out of the shell process per ADR-0009/19. Fuzz/negative test evidence should be verified against current CI.

### Process isolation / system configuration

- Systemd units define service users, namespaces, read-only paths, capabilities, cgroups, sockets and restart controls; some are pending, and the budget review lists configured caps that do not bound all expected processes.
- Environment and Winbridge models insist on host ownership and subordinate guest permissions, but do not prove KVM guest isolation/storage separation works.
- Defender policy code is pure and bounded; no running on-access fanotify/BPF enforcement or quarantine executor is implemented. UI scan/pilot evidence is not equivalent to always-on protection.

### Shell and secret handling

- Shell model separates input decisions from view drawing and uses a `Secret` wrapper/effect for password handoff, avoiding diagnostic formatting of password contents.
- The shell is a highly trusted display process: framebuffer write access can present a false password prompt. Its service confinement/network/file restrictions need runtime validation.
- Auth/lock implementation is a significant sensitive boundary; verify correct Unix helper behavior, failure limits, timing/rate controls, lock/unlock atomicity, and that credentials never enter logs or argv.

## Critical gaps / risks

1. **Privilege identity:** per-connection authentication and binding of AppId to executable/cgroup/peer credentials must precede production authorization. Client-supplied IDs are not authentication.
2. **Persistence/atomicity:** capability grants/audit and service trash state do not have durable, crash-safe storage; settings and user-data lifecycle remain incomplete.
3. **Pending services:** capability daemon, Defender and file daemon units are parked; corresponding binaries/activation paths may not exist. Image claims must match executable inventory.
4. **Incomplete enforcement:** pure model support for storage, environment, Defender, app manifest or browser does not establish sandbox/setup/exec hooks.
5. **TOCTOU and same-UID adversary:** path metadata checks can race. Service privilege separation, `openat`/no-follow operations and hardlink/device handling merit review.
6. **Archive/decoder attack surface:** verify fuzz coverage, resource ceilings, integer overflow handling, duplicate/overlapping ZIP entries and cleanup on partial writes.
7. **Dependency/supply-chain:** only `litric-text` declares third-party crates directly, but resolved transitive dependencies and image package provenance still require lockfile/build attestation. Verify CI dependency policy and lock drift.
8. **Secrets/signing keys:** `keys/.gitignore` is not a key-management ceremony. Dev keys must not be production keys; verify no secret-bearing files or dumps entered evidence/archives.
9. **Update and recovery:** signed update, rollback resistance, atomic switch and recovery are primarily specified; do not claim them as functioning until fault-tested.
10. **Availability/resource budgets:** unbounded messages, request loops, scans, output queues and audit/state growth can become DoS. Several perf budget rows are targets or acknowledged gaps; hard systemd enforcement and runtime measurements are needed.
11. **Accessibility and user consent:** visible dialog code does not prove prompt authenticity, keyboard accessibility or system-side consent binding. Consent must name exact action/scope, be fresh and broker-enforced.
12. **Documentation drift:** ADRs and older master docs may describe superseded architectures. `docs/bugs/` catalogues prior fabricated/incorrect claims; preserve verification caveats and reconcile source paths.

## Recommended verification evidence (not executed by this audit)

- Run unit/property/fuzz suites for URI, manifest, protocols, JPEG/JSON, archive, browser policy, capability scope and secret redaction.
- Boot a fresh image and check actual service users, socket modes, `NoNewPrivileges`, capabilities, cgroups, mount restrictions and executable paths.
- Probe each Unix socket as authorized and unauthorized peer; attempt forged AppId, cross-app paths, stale/revoked grants, expired credentials, overlong/control inputs and newline framing attacks.
- Exercise file operations with symlink swaps, mount boundaries, hard links, collisions, malformed archives, disk-full and termination mid-operation; verify no overwrite/loss.
- Verify password/helper flow and session locking end-to-end without secrets in argv, logs, test reports or screenshot evidence.
- Test image verification, signing/rollback/recovery from a disposable VM, then specific hardware only where claims require it.
- Measure resource caps and idle behavior against current budgets. Mark any NOT RUN explicitly.

## Residual status

The source supports a strong policy-by-construction direction, but current end-to-end security must be described as **partial / not fully verified**, especially for authenticated broker identity, persistent grants, sandbox execution, on-access Defender, update/recovery and image deployment. No claim of a clean production security review is justified by model tests alone.