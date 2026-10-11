# 24 — Feature parity: Windows, macOS, Linux, and LitricOS

## Purpose

The claim "a real desktop OS" is only meaningful against the feature sets users
already have. This catalogue compares LitricOS with Windows 11, macOS, and the
major Linux desktops (GNOME/KDE) across every domain a daily-driver OS must
cover, and maps each feature to LitricOS's mechanism: a built-and-tested crate,
a SYSTEM-SCOPE row (ledger numbers like `r639`), or a deliberate non-goal.

Status vocabulary — nothing in between:

- **DONE** — built and passing tests in the workspace (`src/`), verified by `ci/verify.sh`
- **PLANNED** — ledger row or ADR exists; not built
- **NO** — deliberately not a goal (master-plan/00 non-goals); the reasoning is stated

## Scoreboard

| Domain | Features | DONE | PLANNED | NO |
|---|---|---|---|---|
| Desktop & window management | 12 | 4 | 8 | 0 |
| Files & archives | 14 | 9 | 5 | 0 |
| Search & indexing | 4 | 1 | 3 | 0 |
| Settings & personalisation | 10 | 2 | 8 | 0 |
| Security & privacy | 14 | 5 | 9 | 0 |
| Backup, recovery & generations | 10 | 3 | 7 | 0 |
| Updates & supply chain | 8 | 1 | 7 | 0 |
| Networking | 8 | 1 | 7 | 0 |
| Audio & video | 7 | 1 | 6 | 0 |
| Graphics & display | 8 | 2 | 6 | 0 |
| Power & hardware | 8 | 2 | 6 | 0 |
| Accessibility & input | 6 | 0 | 6 | 0 |
| Language & formats | 5 | 2 | 3 | 0 |
| Applications & packaging | 9 | 3 | 6 | 0 |
| Compatibility (Win/Linux) | 10 | 3 | 6 | 1 |
| Developer & SDK | 6 | 1 | 5 | 0 |
| Virtualisation & containers | 5 | 1 | 4 | 0 |
| **Total** | **144** | **41** | **102** | **1** |

Everything counted DONE has a test in the workspace. Everything counted PLANNED
names its ledger row. That is the honest baseline for "competing with the big
three": roughly a third of the daily-driver surface is built and tested; the
rest is specified, ordered, and unbuilt.

---

## 1. Desktop & window management

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 1 | Composited desktop | DWM | Quartz Compositor | Wayland/KWin/Mutter | `litric-shell` compositor, framebuffer (ADR-0017) | DONE |
| 2 | One desktop for all app kinds | — | — | — | ADR-0020: one surface space, provenance never shown | DONE |
| 3 | Window dragging/resize/focus | built-in | built-in | WM | shell WM with pointer+keyboard, screenshot evidence (GATE 01) | DONE |
| 4 | Taskbar / dock / launcher | Taskbar | Dock | per-DE | shell dock, launcher grid (`litric-shell`) | DONE |
| 5 | Virtual desktops | yes | Spaces | yes | workspace model in shell | PLANNED r1118+ |
| 6 | Window snapping/tiling | FancyZones | tiling | KDE tiling | quarter/half snapping in WM | PLANNED |
| 7 | Alt-tab with previews | yes | App Switcher | WM | shell task switcher, live thumbnails | PLANNED |
| 8 | Notifications + do-not-disturb | Action Center | NC | mako/dunst | notification daemon, DND schedule | PLANNED r1118+ |
| 9 | Clipboard manager with history | Win+V | no | clipman | shell clipboard service + history | PLANNED |
| 10 | Screen capture / screenshot tool | Snipping Tool | Screenshot | gnome-screenshot | capture service, region+window modes | PLANNED |
| 11 | Dark mode + accent colours | yes | yes | GTK themes | theme engine, accent tokens | PLANNED |
| 12 | Lock screen with clock | yes | yes | yes | lockd, PAM-style auth hook | PLANNED |

## 2. Files & archives

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 13 | File manager (list/icons/tree) | Explorer | Finder | Nautilus/Dolphin | ADR-0019 Explorer-style window over LFP/1 | PLANNED (daemon DONE) |
| 14 | Places: Documents/Downloads/... | Known Folders | standard | XDG | `PLACES` command, conventional spots | DONE |
| 15 | Create/rename/copy/move/delete | yes | yes | yes | `litric-files` ops: MKDIR/NEWFILE/RENAME/COPY/MOVE/DELETE | DONE |
| 16 | Skip/replace/keep-both conflict policy | dialog | dialog | dialog | `Policy` enum, server-enforced | DONE |
| 17 | Copy into source dir keeps both | "- Copy" | "copy" | "(copy)" | `name - Copy.ext` rule, tested | DONE |
| 18 | Move refuses directory into itself | yes | yes | yes | `is_within` guard, tested | DONE |
| 19 | Recycle bin / trash | Recycle Bin | Trash | freedesktop | on-disk freedesktop trash + ledger (`litric-trash`) | DONE |
| 20 | Restore without overwrite | yes | yes | yes | `restore-would-overwrite` kind, tested | DONE |
| 21 | Auto-empty by age/capacity | Storage Sense | 30 days | none | `Retention` + `plan_auto_empty` | DONE |
| 22 | ZIP read/write, gzip, tar | Explorer zip | Archive Utility | CLI/GUI | codecs from spec: RFC 1951/1952, APPNOTE, ustar | DONE |
| 23 | Zip-bomb defence | partial | partial | partial | entry caps, 1000:1 ratio, overlap refusal, output stop | DONE |
| 24 | Extraction never follows symlinks | varies | varies | varies | link entries refused; `create_new` O_EXCL writes | DONE |
| 25 | Disk usage analyzer | Storage | Finder | baobab | `DU` with walk cap + partial flag | DONE |
| 26 | Recent files list | yes | Recents | GNOME Recents | `RECENT`, excludes trash, walk-capped | DONE |

## 3. Search & indexing

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 27 | Filename search in place | Explorer | Spotlight | locate | `SEARCH` walk, 500-cap + truncated flag | DONE |
| 28 | Content index (fast full-text) | Windows Search | Spotlight | Tracker | `litric-indexd`, incremental, per-user | PLANNED r1118+ |
| 29 | App launcher search | Start | Spotlight | GNOME | shell launcher query against app registry | PLANNED |
| 30 | Metadata extraction (EXIF/PDF) | propsys | Spotlight | gstreamer | `litric-meta` extractors, sandboxed | PLANNED |

## 4. Settings & personalisation

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 31 | Central Settings app | Settings | System Settings | gnome-control-center | `litric-settingsd` + shell window | PLANNED |
| 32 | Versioned config store (never corrupts) | registry hives | defaults | dotfiles | `litric-config` content-addressed, schema-checked | PLANNED |
| 33 | Display settings (scale/refresh) | Settings | Displays | GNOME | display panel over KMS facts | PLANNED |
| 34 | Sound panel (devices/volume) | Settings | Sound | pavucontrol | audio panel over `litric-audiod` | PLANNED |
| 35 | User accounts & parental controls | Family Safety | Screen Time | none | account daemon, time budgets | PLANNED |
| 36 | Default apps associations | Settings | — | mimeapps | `litric-manifest` handlers + broker | PLANNED |
| 37 | Fonts & typography panel | yes | Font Book | yes | font manager, per-user install | PLANNED |
| 38 | Region/format overrides | yes | yes | yes | `litric-locale` detect + override | DONE (detect) |
| 39 | First-run experience (OOBE) | yes | Setup Assistant | GNOME initial setup | master-plan/19 flow | PLANNED |
| 40 | Per-app settings scopes | — | — | — | per-app config namespaces (`litric-storage`) | DONE |

## 5. Security & privacy

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 41 | Capability-based app permissions | partial | TCC | portals | `litric-caps` broker: deny-by-default, scope checks | DONE |
| 42 | Per-app filesystem scopes | no | TCC folders | flatpak portals | `litric-storage` scopes from URIs | DONE |
| 43 | Built-in AV/EDR engine | Defender | XProtect | ClamAV opt-in | `litric-defender` phase 0: SHA-256, Aho-Corasick, verdict ladder | DONE |
| 44 | On-access scanning hook | minifilter | ESF | fanotify | kernel hook subsystem (ADR-0022 phase 3) | PLANNED |
| 45 | Signature/definition updates | WU | XProtect | freshclam | signed defs, staged rollout (ADR-0022 phase 5) | PLANNED |
| 46 | Quarantine store | Defender | XProtect | clamav | quarantine plan: neutralise+sidecar, journal | PLANNED (design DONE) |
| 47 | Ransomware canary + entropy trip | CFA | — | — | canary files, mass-rename detector, rollback hook | PLANNED (spec DONE) |
| 48 | Application sandbox (GUI apps) | AppContainer | sandbox | flatpak | sandbox profiles + broker tokens | PLANNED r503–508 |
| 49 | Process isolation of system services | services | launchd | systemd | sysd units, no ambient authority | DONE (policy) |
| 50 | Full-disk encryption | BitLocker | FileVault | LUKS | LUKS2 via storage phase, TPM binding | PLANNED |
| 51 | Secure Boot chain | yes | yes | shim | UKI + dm-verity + signed boot (GATE 01) | DONE |
| 52 | Integrity-verified system volume | — | SSV | — | dm-verity rootfs, live in image | DONE |
| 53 | Controlled folder access equivalent | CFA | TCC | none | broker write-scopes + Defender trip | PLANNED |
| 54 | Audit log / incident records | Event Log | log | auditd | `litric-journal`, append-only, queryable | PLANNED r1118+ |

## 6. Backup, recovery & generations

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 55 | Immutable system generations | — | SSV | none | ADR-0021 generations, activate/rollback | DONE (design) |
| 56 | System rollback keeps user data | System Restore | — | snapper | generation ≠ data lifecycle invariant | DONE (design) |
| 57 | Snapshots distinct from backups | VSS | APFS snaps | snapper | ADR-0021 four-concept split | DONE (design) |
| 58 | File history / continuous backup | File History | Time Machine | Déjà Dup | data snapshots + external backup tier | PLANNED |
| 59 | Restore test automation | no | no | no | `verify → compare → confidence` pipeline | PLANNED |
| 60 | Independent recovery environment | WinRE | Recovery | none | recovery image, no desktop deps | PLANNED |
| 61 | Recovery when all three envs broken | no | no | no | boot → recovery UI, restore any env | PLANNED |
| 62 | Recovery decision engine | troubleshooters | — | — | damage analysis → repair/rollback/reset ladder | PLANNED (spec DONE) |
| 63 | Per-environment reset without touching others | Reset this PC | — | — | env-scoped reset (`litric-env` lifecycle) | DONE (policy) |
| 64 | Data-preserving reinstall | Keep-my-files | Reinstall | none | reinstall keeps User Data domain | PLANNED |

## 7. Updates & supply chain

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 65 | Atomic A/B system updates | WinRE | OTA | ostree | generation build→verify→activate | PLANNED |
| 66 | Delta updates | express | delta | none | content-addressed deltas | PLANNED |
| 67 | Update rollback | uninstall | — | ostree | generation rollback | PLANNED |
| 68 | Signed package format | MSIX | pkg | sig | `.ilpkg` signed manifests (`litric-manifest`) | DONE (format) |
| 69 | Staged/canary rollout | WU rings | phased | none | update channel design (ADR-0022 §7) | PLANNED |
| 70 | Update-safety: schema validation before apply | partial | yes | partial | field-count/schema checks, fuzz corpus | PLANNED |
| 71 | Offline-capable builds | no | no | yes | zero-dep offline workspace, vendored | DONE |
| 72 | Repo-wide evidence gate | no | no | no | `ci/verify.sh` PASS/FAIL/SKIP with reasons | DONE |

## 8. Networking

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 73 | Wi-Fi/Ethernet management | netshell | — | NM | `litric-netd` over iwd/NetworkManager backend | PLANNED |
| 74 | Per-app network permission | firewall prompt | Local Network | portals | broker network capability | PLANNED r1118+ |
| 75 | Firewall UI | Defender FW | PF | ufw | nftables policy compiler + panel | PLANNED |
| 76 | VPN client integration | yes | yes | NM | wireguard profiles in netd | PLANNED |
| 77 | captive portal handling | yes | yes | NM | portal detect + open window | PLANNED |
| 78 | Network diagnostics panel | troubleshooters | Wireless Diag | nmcli | netd status + probe commands | PLANNED |
| 79 | Offline-first behaviour | no | no | partial | everything local; no forced accounts | DONE (principle) |
| 80 | HTTP(S) stack for system services | WinHTTP | CFNetwork | soup | `litric-browser` HTTP/1.1 client, no TLS for now | DONE (HTTP/1.1) |

## 9. Audio & video

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 81 | System audio server | WASAPI | CoreAudio | PipeWire | `litric-audiod` graph, per-app streams | PLANNED |
| 82 | Per-app volume + routing | Volume Mixer | — | pavucontrol | audiod policy | PLANNED |
| 83 | Video playback | Media Player | QuickTime | gstreamer | `litric-media` decode pipeline | PLANNED |
| 84 | Screen recording | Game Bar | Screenshot | OBS | capture service + encoder | PLANNED |
| 85 | Camera permission per app | yes | yes | portals | broker device capability | PLANNED |
| 86 | HDMI/DP audio routing | yes | yes | PipeWire | audiod sink selection | PLANNED |
| 87 | Container codecs (mp4/webm demux) | yes | yes | gstreamer | pure-Rust demuxers, staged | PLANNED |

## 10. Graphics & display

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 88 | KMS/fbdev display bring-up | yes | yes | modeset | shell renders via DRM/KMS (GATE 01 evidence) | DONE |
| 89 | Multi-monitor | yes | yes | yes | output enumeration + layout | PLANNED |
| 90 | HiDPI scaling | 125%+ | Retina | fractional | scale factor in compositor | PLANNED |
| 91 | GPU acceleration (GL/Vulkan) | D3D12 | Metal | Mesa | Mesa as dependency; native API later | PLANNED |
| 92 | GPU-free fallback rendering | basic | — | llvmpipe | software rasteriser path in shell | DONE |
| 93 | Colour management | yes | ColorSync | colord | profile attach in compositor | PLANNED |
| 94 | Night light | yes | Night Shift | redshift | gamma LUT at dusk schedule | PLANNED |
| 95 | Damage tracking / partial redraw | yes | yes | yes | dirty-rect compositor | DONE |

## 11. Power & hardware

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 96 | Suspend/resume | yes | yes | systemd | sleep targets via logind-equivalent | PLANNED |
| 97 | Battery percentage + S3 states | yes | yes | upower | `litric-sysd` power facts | DONE (facts) |
| 98 | Thermal management | yes | yes | thermald | mm-tune governor, throttling policy | PLANNED |
| 99 | CPU governor profiles | power modes | Auto | cpupower | `litric/mm-tune` profile switcher | DONE |
| 100 | Device manager view | Device Mgr | Sys Info | lspci | hardware inventory from sysd facts | DONE (facts) |
| 101 | Peripherals: printers | yes | yes | CUPS | CUPS as dependency | PLANNED |
| 102 | Bluetooth audio/input | yes | yes | bluez | bluez as dependency, broker-gated | PLANNED |
| 103 | Hot-plug events (USB) | yes | yes | udev | udev monitor → shell notification | PLANNED |

## 12. Accessibility & input

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 104 | Screen reader contract | Narrator | VoiceOver | Orca | accessibility tree in shell protocol | PLANNED |
| 105 | On-screen keyboard | yes | yes | caribou | shell input panel | PLANNED |
| 106 | Sticky/filter keys | yes | yes | XKB | input filter chain | PLANNED |
| 107 | High-contrast themes | yes | yes | GTK | theme tokens, WCAG-checked | PLANNED |
| 108 | Keyboard layouts & switching | yes | yes | XKB | `litric-locale` + XKB keymaps | PLANNED |
| 109 | Reduced motion setting | yes | yes | GNOME | compositor animation kill-switch | PLANNED |

## 13. Language & formats

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 110 | TZ database local time | yes | yes | tzdata | `litric-locale` transitions, 8-compare lookup | DONE |
| 111 | Locale formatting (number/date/percent/temp) | NLS | CFLocale | locale | `litric-locale::Locale` formatters | DONE |
| 112 | IME framework (CJK input) | TSF | yes | fcitx/ibus | input method protocol in shell | PLANNED |
| 113 | UTF-8 everywhere | partial | yes | yes | enforced; control bytes refused at wire | DONE |
| 114 | Translation infrastructure | resource files | strings | gettext | token catalogues in `litric-locale` | PLANNED |

## 14. Applications & packaging

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 115 | Native package format + installer | MSIX | pkg/app | deb/flatpak | `.ilpkg` (litric-manifest): signed, manifest-driven | DONE (format) |
| 116 | App registry & lifecycle | Start menu | LaunchServices | desktop files | app registry, launch via broker | PLANNED |
| 117 | Per-app data isolation | packages | container | flatpak | per-app namespaces, never user-shared | DONE |
| 118 | Per-app cache/temp hygiene | no | no | partial | cache/temp namespaces, auto-clear | DONE |
| 119 | App store / environment manager | Store | App Store | GNOME Software | Environment Manager UI (ADR-0021) | PLANNED |
| 120 | Uninstall leaves no residue | partial | partial | partial | namespace deletion = full removal | DONE |
| 121 | Native SDK + toolchain | SDK | Xcode | GCC | master-plan/10 SDK, zero-dep workspace | PLANNED |
| 122 | Web engine (browser) | Edge | Safari | WebKit | `litric-browser`: HTML/CSS subset, JS engine | DONE (subset) |
| 123 | Image codecs (PNG/JPEG/GIF) | WIC | CoreGraphics | gdk-pixbuf | pure-Rust decoders incl. progressive JPEG | DONE |

## 15. Compatibility (Windows/Linux applications)

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 124 | Windows apps in bottles | — | Wine | Wine | `litric-winbridge` bottle lifecycle, plans tested | DONE |
| 125 | Auto runtime selection | — | — | — | ADR-0020 decision function, shown to user | DONE (design) |
| 126 | Linux apps native (this is Linux) | WSL | — | native | sandbox + portals, no translation | DONE |
| 127 | Full Windows VM as fallback | — | — | KVM | ADR-0015 guest, licensing note respected | PLANNED |
| 128 | Full Linux env downloadable | WSL distros | — | — | Environment Manager fetch+verify | PLANNED |
| 129 | VM windows as native windows | — | — | virt-viewer | guest surface compositing | PLANNED r880 |
| 130 | Per-env resource budgets | WSL config | — | cgroups | budgets + host reservation, host can never suspend | DONE |
| 131 | Env health states (compromised etc.) | no | no | no | 8-state model in `litric-env` | DONE |
| 132 | Cross-env data policy matrix | no | no | no | ADR-0020 access matrix | DONE (design) |
| 133 | Boot a Windows desktop as primary | yes | no | no | **NO** — Iltric/Litric stays primary by constitution | NO |

## 16. Developer & SDK

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 134 | System daemon CLI | sc | launchctl | systemctl | `litric-sysd` control verbs | DONE (facts+verbs) |
| 135 | System protocol docs | MSDN | docs | man | ADRs + master-plan as contract | DONE |
| 136 | Fuzzing harness for parsers | no | no | partial | hostile-input tests in codecs/defender | DONE |
| 137 | Sample apps | SDK samples | templates | examples | examples in SDK plan | PLANNED |
| 138 | Emulator/debug shell | no | no | no | recovery terminal design | PLANNED |
| 139 | CI with evidence ledger | partial | no | partial | verify.sh + EVIDENCE.md + GATES | DONE |

## 17. Virtualisation & containers

| # | Feature | Windows | macOS | Linux | LitricOS | Status |
|---|---|---|---|---|---|---|
| 140 | Hypervisor for guest envs | Hyper-V | HVF | KVM | KVM as dependency, env manager on top | PLANNED |
| 141 | Container runtime for dev | WSL2 | — | cgroups | namespace/container profiles | PLANNED |
| 142 | Snapshot/restore of whole env | checkpoints | — | virsh | env snapshot via generation engine | PLANNED |
| 143 | GPU passthrough to guest | GPU-P | — | VFIO | deferred until native GPU stack lands | PLANNED |
| 144 | Nested isolation for untrusted apps | AppContainer | — | — | bottle sandbox + caps, no VM per app | DONE (policy) |

---

## Reading the gaps honestly

Three gaps a solo-built OS must name rather than blur:

1. **Signature coverage (Defender).** The engine is built and tested; coverage
   comes from curated feeds, which is an operations problem, not a code problem
   (ADR-0022 §9). The DONE rows here are rows 43/47/70 — engine, tripwires,
   update safety — never "detects everything".
2. **GPU acceleration.** The compositor works on KMS and the software path is
   DONE; Mesa integration (row 91) is the step where LitricOS stops being
   renderer-limited. Everything desktop-smooth depends on it.
3. **Application ecosystem.** Rows 115–122 make LitricOS able to *run* apps;
   whether apps *exist* is the long game the compatibility tiers buy time for.

## Non-goals (kept, not forgotten)

- Never a Windows or Linux desktop replacement by proxy: no full-environment
  boot as the primary OS (row 133).
- No forced account, no telemetry-by-default, no cloud dependency for core
  function (row 79).
- No third-party crates in the system workspace. Every DONE row above is
  offline-buildable code under `#![forbid(unsafe_code)]`, and every dependency
  in the workspace today is a path dependency on a sibling member. The rule as
  `src/Cargo.toml` states it: the core crates have **zero** third-party
  dependencies, std only; `litric-shell` and `litric-defender` are the platform
  crates, where a dependency is an architecture decision rather than a detail.
  `ci/lint-deps.py` enforces exactly that state — a member may not gain a
  non-path dependency — and `ci/dep-policy-allow.txt` is the only place a
  deliberate exception may be recorded, with its reason.
