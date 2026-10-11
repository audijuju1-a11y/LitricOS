# 03 — Native contract governance

## Contract layers

| Layer | Audience | Stability |
|---|---|---|
| Kernel primitives | privileged services | Linux/upstream controlled |
| Litric service IPC | services and SDK runtimes | versioned, compatibility tested |
| Public application API | application developers | stable v1 contract |
| SDK | application developers | additive minor revisions |

Existing conceptual contracts in `docs/API-00-contracts.md` are the starting
point; they are not implementation proof. Each accepted API must acquire a
machine-readable schema, error taxonomy, ownership rules, cancellation semantics,
threading model, permission requirement, resource limit, and contract test.

## v1 public service families

- `caps`: grant, revoke, inspect and audit scoped capabilities.
- `storage`: portal-issued file handles and logical app/user/cache/secure storage.
- `network`: outbound/listen policy, DNS, VPN exposure and cancellation.
- `surface`: window lifecycle, input, clipboard, drag/drop and presentation.
- `audio`: stream lifecycle, capture consent, device routing and latency class.
- `notifications`, `power`, `update`, `diagnostics`, and `accessibility`.

## Compatibility policy

- Major ABI: breaking changes only with a supported migration path and parallel
  runtime availability.
- Minor ABI: additive, feature-discoverable, and safe for older clients.
- Patch ABI: no semantic or binary break.
- Each app declares a minimum and tested maximum contract version. Unsupported
  versions fail before launch with a diagnosable reason.

## Rules for handles and IPC

- Opaque handles have explicit owner, scope, lifetime, close/cancel operation,
  duplicate policy, and behaviour after permission revocation.
- IPC decoders are bounded, schema-validated and fuzzed; untrusted length,
  encoding, enum, path and duplicated-field input is rejected.
- Sensitive operations bind caller identity to the request server-side; never
  trust a client-supplied app identifier or ambient D-Bus name alone.

## Acceptance tests

Contract tests cover old/new version negotiation, malformed messages, resource
exhaustion, cancellation races, permission revocation during I/O, process death,
and deterministic errors. ABI snapshots run in CI before every stable release.

