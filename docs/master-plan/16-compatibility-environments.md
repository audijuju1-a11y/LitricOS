# 16 — Compatibility environments

## Scope

Compatibility removes adoption friction; it is not Litric's kernel, app model,
security model, or v1 success criterion. Native SDK applications and sandboxed
Flatpaks always remain supported when compatibility components are absent.

## Tiers

| Tier | v1 status | Boundary |
|---|---|---|
| Linux applications | Flatpak/native apps | normal app sandbox |
| Windows executable assessment | preview | static PE metadata inspection only |
| Windows bottle/runtime | experimental | disposable filesystem/process tree |
| Windows KVM guest | experimental | VM + host cgroup/device policy |
| GPU passthrough/game mode | post-v1 unless HCL proven | IOMMU/HCL/security review |

## Environment model

An environment has immutable base image/runtime, scoped writable data, explicit
mounts, resource budget, network policy, graphics/audio/input policy, lifecycle
state and reset/export/delete action. Native secrets, host admin sockets,
recovery keys and ambient user-home mounts are never shared.

## UX rules

- Show compatibility confidence from a named application/version/device matrix,
  not a generic "supported" label.
- Explain isolation and resource cost before first launch.
- Prefer native application when both native and compatibility entries exist;
  user choice remains explicit.
- Resetting a bottle/guest tells the user exactly which scoped data will be lost
  and offers export/snapshot where supported.

## Test gates

Test guest/bottle launch failure, bad binary parsing, corrupted runtime, resource
exhaustion, suspend/resume, network block, host-file denial, clipboard/capture
consent, guest crash, reset and deletion. Kernel anti-cheat support is excluded
unless a specific title passes legally permitted, reproducible testing.

