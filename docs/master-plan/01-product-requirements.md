# 01 — Product requirements

## Supported baseline

| Dimension | v1 commitment | Not a v1 commitment |
|---|---|---|
| CPU | x86-64 Intel and AMD systems in the HCL | every CPU generation |
| Memory | 8 GB minimum | 4 GB daily-driver guarantee |
| Display | one internal/one external display; tested hotplug | arbitrary multi-GPU arrays |
| Storage | UEFI, GPT, NVMe/SATA where tested | unsafe in-place installation layouts |
| Applications | native SDK and Flatpak | all Win32/Linux binaries |

## Release outcomes

### FND — system proof

- A signed image builds reproducibly, boots, reaches login, starts a native
  Wayland client, and recovers from a failed staged update.
- Kernel config is built into the running kernel, not merely linted.
- The evidence pipeline records boot, image hash, kernel config, and test host.

### DP — platform proof

- Native SDK preview, portals, sandboxed app launch, capability UI, files,
  network, audio, settings, diagnostics, and an initial HCL work end-to-end.
- A developer can build, package, profile, crash, and debug a sample app.

### BETA — daily-use validation

- Installer, session/lock screen, recovery UI, backups, Wi-Fi, Bluetooth,
  removable storage, printing, suspend/resume, accessibility baseline, and
  first-party file/settings/search applications are usable on the HCL.

### V1 — supported promise

- Published HCL, release channel, security response process, stable SDK/ABI,
  update/rollback record, measured performance report, and documented limits.

## User journeys that cannot regress

1. Install → encrypt → first boot → account/session → desktop.
2. Select a document in a sandboxed app → revoke access → verify immediate deny.
3. Lose power during update → boot prior healthy slot → preserve user data.
4. Dock, undock, suspend, resume, reconnect Wi-Fi/audio/display without session
   loss or uncontrolled wakeups.
5. Keyboard-only user completes setup, launches an app, changes settings, and
   reaches recovery tools.

## Product metrics

The performance numbers in `docs/PERF-budget.md` are gates, not results. The
public scorecard will report controlled measurements for boot, resume, idle PSS,
idle CPU, wakeups, idle writes, battery drain, p99 UI latency, crash-free time,
successful update rate, and recovery success rate.

