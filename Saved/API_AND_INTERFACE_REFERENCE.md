# API and Interface Reference

## Reading this reference

This is a source-oriented map, not a promise that every API is wired into a running image. Public Rust APIs are intra-workspace contracts; protocols are Unix/local service contracts. Verify protocol details in implementation/ADR and unit tests before changing either. ABI-facing types in `litric-core` are called out as change-sensitive.

## Shared Rust contracts (`litric-core`)

- `AppId::parse` / `parse_platform_reserved`: validated reverse-DNS application identity; 255-byte maximum, 63-byte labels, at least three labels, lowercase ASCII and reserved namespace handling. `as_str`, `is_platform`, `as_path_component` expose canonical identity safely.
- `Version::parse` represents major/minor/patch plus optional prerelease; prerelease sorts before release. `AbiVersion::CURRENT` and `can_host` express runtime compatibility; do not use application semver ordering as ABI compatibility.
- `Handle` is an opaque `u64` with `HandleKind`, index and generation fields. Do not expose its internal layout as an app API.
- `Error`, `ErrorKind`, `Result<T>` are structured error contracts with stable wire tokens and `to_wire`/`from_wire` mapping.
- `litric_core::pct`: strict percent-encoding for the line protocols; protocol field convention includes `-` as empty and `%2D` for literal dash.

## Capability API (`litric-caps`)

- `CapabilityId`, `Risk`, `Scoping`, `Scope`, `Capability` form the policy vocabulary. Parse scope via `Scope::parse_uri`, `parse_host`, or `parse_resource` depending on intended resource class.
- `Capability::new(id, scope)` validates that the scope matches capability scoping; `unscoped` is available only for unscoped capabilities.
- `authorises` uses resource scope containment; `requires_live_consent`, `is_destructive`, active-indicator and risk/prompt metadata support policy/UI.
- `Broker::register`, `grant`, `revoke`, `revoke_all`, `check`, `check_with_consent`, `would_allow`, `active_capabilities`, `grants_for`, `audit_log`, `audit_for`, `purge_audit` comprise the in-memory decision interface in `litric-caps/src/lib.rs`.
- The crate docs say identity must be authenticated by the daemon (SO_PEERCRED + cgroup in the intended broker architecture), never accepted from an untrusted caller. Current sysd code's local identity shortcut/per-connection gap is documented separately; do not assume the library enforces IPC authentication.

## Storage, manifest and environment APIs

- `StorageUri::parse` accepts `user://`, `media://`, `app://`, `cache://`, `config://`, `temp://`, and `secure://` URI forms. `StorageBroker::required_capability` maps shared access to `FsRead`/`FsWrite`/`FsList`; `resolve` checks granted capabilities and application ownership before producing a filesystem path. `secure://` is not an ordinary filesystem path.
- `Manifest::parse` validates a bounded INI subset (`MAX_LEN=64 KiB`, `MAX_LINES=2000`, `MAX_CAPABILITIES=64`) and produces typed identity, versions, ABI, resource/latency class, main path and capabilities. `prompted_capabilities`/`consent_summary` are installer-facing summaries.
- `EnvironmentManager` methods `plan_switch`, `switch_to`, `transition`, `set_health`, `set_budget`, `rebind`, and environment getters model policy only. The pure model has no VM or surface executor.
- `WindowManager` exposes window/workspace/focus/hit-test/drag/snap/damage APIs; `InputSource` exposes backend-independent events.

## LSP/sysd protocol (`litric-sysd/src/proto.rs`)

- One bounded text request per line (`MAX_LINE=4096`); stable tokens/arity are parsed strictly. Request families include `PING`, facts, environment list/plan/switch, trash list/empty planning, capability audit, power, provider actions, storage `RESOLVE`, and quit.
- `POWER` and provider `ACTION` carry explicit consent state. `RESOLVE` has app identity, URI, and access word; production security requires that identity be authenticated by the daemon rather than trusted from the request.
- Responses use `OK <count>` plus exactly that many payload lines or `ERR <kind> <detail>`; newline/control handling protects framing. Refer to `Request`, `Response`, `ProtoError`, and their tests for exact grammar.
- Daemon path in code: `/run/litric/sysd.sock`; shell uses the same path. CLI supports report/serve/query modes. Inspect image unit for user, socket and hardening details.

## LFP/1 file protocol (`litric-files/src/proto.rs`, ADR-0019)

- Versioned `LFP/1`; maximum request line 16,384 bytes; one UTF-8 line; single-space-separated fields; free text percent-encoded via `litric_core::pct`.
- Reply framing: `OK <n>` followed by `n` lines; `ERR <kind> <pct-msg>`; long operations may emit `PROGRESS <done> <total> <pct>` before final reply. Entry values are percent-encoded `key=value` fields.
- Command set in ADR-0019 includes HELLO, PLACES, LIST, STAT, DU, MKDIR, NEWFILE, RENAME, CONFLICTS, COPY, MOVE, TRASH/TRASHLIST/RESTORE/PURGE/EMPTYTRASH/DELETE, COMPRESS/EXTRACT, SEARCH, RECENT, VOLUMES, READTEXT, QUIT. Exact implemented handlers should be checked in `litric-files/src/server.rs`.
- Stable errors include `bad-request`, `invalid-name`, `not-found`, `exists`, `not-a-directory`, `is-a-directory`, `symlink-refused`, `permission-denied`, `too-large`, `unsupported-archive`, `corrupt-archive`, `unsafe-archive-entry`, `limit`, `restore-would-overwrite`, `io`.

## LBP/1 browser protocol (ADR-0018 and browserd source)

- Browser library surface: `Url::parse`, `interpret`, `Policy::check_navigation`, `Session`, tab/history and engine trait.
- Broker commands documented by ADR: HELLO, VIEWPORT, NEW, CLOSE, ACTIVATE, GO, BACK/FORWARD/RELOAD/STOP, MOUSE, WHEEL, KEY, TEXT, PING, QUIT. Events include TAB, CLOSED, FRAME, NOTICE, ERR, PONG; frame body is raw bounded RGBA pixels.
- Free text fields are percent-encoded. The broker drives Chromium over CDP pipes, not a TCP debugging port, per ADR. The source has an optional `broker` feature and an in-code socket default; compare its `/run/browserd.sock` with image/ADR `/run/litric/browser.sock` before relying on deployment.
- LBP details are in transition: ADR text is the intended fuller protocol while current source evolution includes a narrower implemented subset. Keep that distinction explicit.

## Shell effects boundary

`Shell::handle` returns model effects for `main` to carry out (e.g. switch environment, lock request, preference persistence, password verification, volume/brightness, power, launched app). This keeps drawing/input policy testable. `Secret` wraps password material to avoid accidental diagnostic formatting. Actual effect execution and backend completion are separate from UI state changes.

## Configuration compatibility

Public contract changes need source, test, docs, protocol/ABI version, and compatibility review. `litric-core` explicitly marks its public types as ABI-sensitive. Protocol implementations must preserve bounds, exact framing, strict field decoding, stable error tokens and no secret echo. Record ADRs before changes that alter trust boundaries, identity, storage format, process topology, compositor protocol, defaults or hard budgets.