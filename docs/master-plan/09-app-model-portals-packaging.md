# 09 — Application model, portals, and packaging

## v1 model

Applications are distributed as signed Flatpak packages with Litric desktop
integration. Litric adds a stable app identity, capability policy, resource
class, diagnostics and UX; it does not create a second general-purpose package
format in v1.

## Application manifest additions

| Field | Purpose |
|---|---|
| `app_id`, publisher, signature | identity and ownership |
| runtime/architecture/version | launch compatibility |
| requested capabilities | reviewable least privilege |
| resource class | interactive, latency-sensitive, background, batch |
| background declaration | lifecycle and wakeup policy |
| migration/data policy | updates and uninstall behaviour |

## Capability model

- Files are granted by selected handle/directory scope, never broad home access
  by default. Revocation terminates future handle operations and is observable.
- Network, microphone, camera, Bluetooth, USB, clipboard, screen capture and
  background execution each have separate consent and audit records.
- App requests explain purpose in human language supplied by Litric policy, not
  unreviewed arbitrary copy from the application.
- Policy supports user choice, admin restrictions in a later tier, and denial
  that leaves an app functional where feasible.

## Lifecycle

`install → verify → provision storage → launch → suspend/resume → update or
uninstall`. Crashes capture bounded redacted diagnostics. Background services
must be socket/dbus/event activated, cgroup-scoped, rate-limited, and stoppable
from settings. Package updates never execute untrusted maintainer scripts with
ambient host privileges.

## Acceptance tests

Test tampered signature, invalid manifest, forbidden permission, permission
revocation mid-operation, quota exceedance, app/helper crash, offline update,
downgrade rejection, uninstall data choice, and portal spoof/confused-deputy
attempts. Track every tested application/runtimes in a compatibility matrix.

