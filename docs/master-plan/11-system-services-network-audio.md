# 11 — System services, network, and audio

## Service model

System services run under systemd with an explicit unit owner, privilege set,
capability boundary, restart policy, cgroup slice, readiness condition, log
budget, health probe and shutdown ordering. Socket/dbus activation is preferred
for non-essential services; permanently resident daemons require a measured
justification.

## Core service catalogue

| Service | Responsibility | Boundary |
|---|---|---|
| identity/session | login, lock, user session | credentials and policy |
| capability broker | grants/revocation/audit | app to privileged resource |
| storage/update | logical paths, packages, slots | persistent system state |
| network | Wi-Fi, Ethernet, DNS, VPN policy | external network |
| audio | PipeWire routing, capture, latency | mic/audio device |
| device/power | removable media, battery, thermal | hardware state |
| diagnostics | bounded crash and health data | privacy and support |

## Networking

- Use iwd/networkd/resolved according to the image policy; expose one Litric UX
  state model instead of leaking backend differences.
- Default inbound deny, explicit hotspot/server permissions, DNS privacy policy,
  captive-portal detection, VPN kill-switch semantics, metered-network controls
  and network-change events for applications.
- Wi-Fi credentials are secret-store objects with per-user ownership and no
  access by guest/compatibility environments.

## Audio

- PipeWire/WirePlumber are activated on demand, not assumed idle residents.
- Support output selection, per-app volume, device hotplug, microphone consent,
  visible mic indicator, communication/game/music policy and normal/low latency
  classes.
- Realtime path must avoid blocking I/O, allocation and user interaction;
  permission rejection and device loss have a defined non-crashing result.

## Tests

Run service restart/dependency loss tests, network roaming/captive/VPN tests,
audio device hotplug and capture revocation tests, priority inversion checks,
latency loopback measurements, and idle resource audits for every service set.

