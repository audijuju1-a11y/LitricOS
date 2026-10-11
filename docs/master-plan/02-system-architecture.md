# 02 — System architecture

## Dependency direction

```text
Firmware → signed UKI → Linux LTS / drivers → Litric core services
         → stable public contracts → SDK / native or sandboxed apps

Windows or Linux app → compatibility environment → Litric contracts → kernel
```

The lower line is strictly subordinate. Removing compatibility code must leave
the primary boot, update, security, and native-app paths operational.

## Trust and ownership

| Layer | Owner | Trust | Responsibility |
|---|---|---:|---|
| Firmware/boot | vendor + Litric config | 0 | verified launch and recovery selection |
| Linux kernel/drivers | upstream + Litric config | 0 | hardware mechanism and isolation primitives |
| core services | Litric + upstream | 1 | policy, identity, storage, package/update, resources |
| compositor/shell/SDK | Litric | 2 | interaction, app contracts, trusted mediation |
| apps | publisher | 3–4 | untrusted workload under capabilities |

## Mandatory boundaries

- App ↔ core service: authenticated IPC, application identity, capability check,
  rate/size limits, versioned messages, audit record for sensitive grants.
- App ↔ app: no implicit filesystem, process, clipboard, secret, or shared-memory
  access.
- Guest ↔ host: separate storage, credentials, process tree and cgroup budget.
- Signed system ↔ mutable user data: immutable verified OS must never make user
  data immutable or disposable.

## Architectural invariants

1. No application reaches privileged kernel functionality through an unversioned
   private API.
2. The shell never parses untrusted media, archives, or documents in-process.
3. Every daemon has a lifecycle owner, resource slice, restart policy, and
   diagnostics story.
4. System policy does not rely on desktop UI for enforcement.
5. Unavailable optional services degrade explicitly; security controls do not.

## Required ADR triggers

Create an ADR before changing trust boundary, public ABI, storage format, update
state machine, process model, compositor protocol version, default permission,
or a hard performance budget.

