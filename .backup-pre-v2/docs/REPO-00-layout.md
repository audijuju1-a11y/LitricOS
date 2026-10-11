# ILTRIC — Repository Layout
**Document:** REPO-00-layout.md | **Phase:** 00 | **Status:** DESIGN-ONLY

---

## 1. Directory Tree

```
iltric/
├── boot/                          # UEFI, UKI configs, bootloader
│   ├── uki/                       # UKI build definitions
│   └── keys/                      # Dev signing keys (never production)
│
├── kernel/                        # Kernel config fragments + patches
│   ├── config/                    # .config fragments per profile
│   │   ├── iltric-base.config     # Core config (Phase 01)
│   │   ├── iltric-gaming.config   # Gaming overlay (Phase 12)
│   │   └── iltric-debug.config    # Debug overlay
│   └── patches/                   # Any required patches (minimal)
│
├── image/                         # mkosi image pipeline
│   ├── mkosi.conf                 # Base image definition
│   └── profiles/                  # Profile overlays (desktop, headless)
│
├── core/                          # Core system services & configs
│   ├── units/                     # systemd service overrides
│   ├── journald.conf.d/           # journald memory policy
│   ├── zram-generator.conf        # zram config
│   ├── sysctl.d/                  # kernel tuning (mm, network)
│   ├── btrfs/                     # Btrfs mount + subvolume policy
│   ├── crypt/                     # LUKS2 + TPM2 config
│   ├── network/                   # systemd-networkd + iwd configs
│   ├── firewall/                  # nftables ruleset
│   └── recovery/                  # Recovery target + tools
│
├── compositor/                    # Smithay-based compositor (Rust)
│   ├── src/                       # Compositor source
│   ├── protocols/                 # Wayland protocol implementations
│   └── benches/                   # Frame-time + latency benchmarks
│
├── shell/                         # ILTRIC desktop shell
│   ├── panel/                     # Top/bottom panel
│   ├── launcher/                  # App launcher + unified search
│   ├── files/                     # File manager
│   ├── settings/                  # Settings app (3 tiers)
│   ├── notifications/             # Notification service
│   └── tokens/                    # Design tokens (color, type, spacing)
│
├── defender/                      # Security & defender
│   ├── onaccess/                  # fanotify on-access engine (Rust/C)
│   ├── engine/                    # YARA + hash denylist + classifier
│   ├── behavior/                  # eBPF behavioral hooks
│   └── quarantine/                # Quarantine management
│
├── ai/                            # AI subsystem
│   ├── router/                    # Capability/model router
│   ├── models/                    # Model policy + storage
│   ├── npu/                       # NPU integration (OpenVINO)
│   ├── speech/                    # STT (whisper.cpp) + TTS (piper)
│   └── audit/                     # AI usage audit log
│
├── sdk/                           # Native SDK
│   ├── headers/                   # Stable C/Rust API headers
│   ├── bindings/                  # Language bindings
│   ├── manifest/                  # App manifest format spec
│   └── templates/                 # "hello-iltric" and templates
│
├── apps/                          # Application platform
│   ├── trust/                     # cosign + TUF metadata
│   ├── compat/                    # Compatibility layers
│   └── flatpak/                   # Flatpak integration
│
├── tools/                         # Developer tooling
│   ├── run-qemu.ps1              # Windows QEMU launcher
│   ├── run-qemu.sh               # Linux/Mac QEMU launcher
│   ├── logs/                      # Boot logs
│   └── bench/                     # Benchmark harness
│       ├── mem/                   # Memory measurement
│       └── latency/               # Frame-time + audio latency
│
├── tests/                         # Test suite
│   ├── smoke/                     # Boot smoke tests
│   ├── regression/                # Regression tests
│   └── security/                  # Security tests
│
├── docs/                          # Documentation
│   ├── ARCH-00-boundaries.md      # System boundary diagram
│   ├── SEC-00-threatmodel.md      # Threat model
│   ├── API-00-contracts.md        # Interface contracts
│   ├── SEC-01-capabilities.md     # Capability vocabulary
│   ├── REPO-00-layout.md          # This file
│   ├── RISK-00.md                 # Risk register
│   ├── GATES.md                   # Gate results log
│   ├── HCL.md                     # Hardware compatibility list
│   ├── PERF-budget.md             # RAM budget ledger
│   ├── PERF-claims.md             # Publication rules
│   └── adr/                       # Architecture decision records
│       ├── 0001-kernel-foundation.md
│       ├── 0002-init-system.md
│       ├── 0003-image-builder.md
│       ├── 0004-filesystem-layout.md
│       ├── 0005-compositor.md
│       ├── 0006-audio-stack.md
│       ├── 0007-package-format.md
│       └── 0008-security-model.md
│
├── gaming/                        # Gaming mode (cross-cutting)
│   ├── gamescope/
│   ├── gamemode/
│   ├── proton/
│   ├── ntsync/
│   └── sched/
│
├── update/                        # Update system
│   ├── sysupdate.d/               # systemd-sysupdate transfer defs
│   └── sign/                      # Signing + TUF metadata
│
└── 00-MASTER-WORKFLOW.md          # Master workflow (always at root)
```

## 2. Ownership by Directory

| Directory | Primary Owner | Language | Phase |
|-----------|--------------|----------|-------|
| `boot/` | Systems engineer | Shell/config | P01 |
| `kernel/` | Kernel engineer | Config/patches | P01 |
| `image/` | Release engineer | Config | P01 |
| `core/` | Systems engineer | Config/scripts | P02-P04 |
| `compositor/` | Graphics engineer | Rust | P05 |
| `shell/` | Desktop engineer | Rust | P06 |
| `defender/` | Security engineer | Rust/C | P07 |
| `ai/` | AI engineer | Rust/C | P09 |
| `sdk/` | SDK engineer | Rust/C | P10 |
| `apps/` | Platform engineer | Config | P10 |
| `tools/` | Release/QA | Shell/Python | P01+ |
| `tests/` | QA engineer | Python/Rust | P01+ |
| `docs/` | All | Markdown | P00+ |
| `gaming/` | Gaming engineer | Config/scripts | P12 |
| `update/` | Release engineer | Config | P08 |

## 3. Naming Conventions

- Config files: `kebab-case` (e.g., `iltric-base.config`)
- Source files: `snake_case` (Rust), `kebab-case` (shell scripts)
- Documentation: `UPPER-kebab-case.md` for phase docs, `lowercase.md` for guides
- ADRs: `NNNN-kebab-case.md`
