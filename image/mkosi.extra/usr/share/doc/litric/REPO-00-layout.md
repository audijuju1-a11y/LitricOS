# LITRIC — Repository Layout
**Document:** REPO-00-layout.md | **Phase:** 00 | **Status:** DESIGN-ONLY — §0 says what the tree actually is

---

## 0. What this repository actually is — 2026-10-11

The tree in §1 and the ownership table in §2 describe the layout planned at
Phase 00. The checkout does not follow it, and §1 has been read as though it did
(`docs/audit/2026-10-11-beta-1.0-stage-a-audit.md` §3.1). This section is what a
reader should use; §1 and §2 are kept as the plan they are.

The real top level:

| Directory | What it holds |
|---|---|
| `src/` | **the implementation**: one Rust workspace, `src/Cargo.toml`, members listed there |
| `image/` | mkosi description (`mkosi.conf`, `mkosi.profiles/`, `mkosi.repart*/`, `mkosi.extra/`); unit files that are installed live under `image/mkosi.extra/usr/lib/systemd/system/`, and ones waiting for a binary live in `image/units-pending/` |
| `ci/` | the gate's own checkers (`ci/verify.sh` and the `ci/lint-*.py` family) |
| `docs/` | evidence (`docs/EVIDENCE.md`), decisions (`docs/adr/`), gates, risk register, the capability ledger `docs/SYSTEM-SCOPE.md` |
| `tests/` | boot smoke tests and the gate scripts that cannot run on a Windows host |
| `tools/` | developer tooling (QEMU launch, frame dumps, playtest drivers) |
| `keys/` | development signing keys; `keys/.gitignore` keeps them out of version control |
| `build/` | generated output — logs, dumped frames, image artifacts; not source |

Not created, and not planned: `compositor/` ([ADR-0017](adr/0017-framebuffer-shell.md)
decided against a compositor), `shell/` (the shell is the `litric-shell` crate),
`defender/`, `ai/`, `sdk/`, `apps/`, `gaming/`, `update/` and `core/` (image-time
configuration lives in `image/mkosi.extra/` instead). Their planned ownership in
§2 is therefore not a description of anyone's directory.

---

## 1. Directory Tree (the Phase 00 plan)

```
litric/
├── boot/                          # UEFI, UKI configs, bootloader
│   ├── uki/                       # UKI build definitions
│   └── keys/                      # Dev signing keys (never production)
│
├── kernel/                        # Kernel config fragments + patches
│   ├── config/                    # .config fragments per profile
│   │   ├── litric-base.config     # Core config (Phase 01)
│   │   ├── litric-gaming.config   # Gaming overlay (Phase 12)
│   │   └── litric-debug.config    # Debug overlay
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
├── shell/                         # LITRIC desktop shell
│   ├── panel/                     # Top/bottom panel
│   ├── launcher/                  # App launcher + unified search
│   ├── files/                     # File manager
│   ├── settings/                  # Settings app (3 tiers)
│   ├── notifications/             # Notification service
│   └── tokens/                    # Design tokens (color, type, spacing)
│
├── defender/                      # Security & defender
│   ├── onaccess/                  # reserved for the future fanotify hook (not implemented)
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
│   └── templates/                 # "hello-litric" and templates
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

## 3. Naming Conventions (in force)

- Config files: `kebab-case` (e.g., `litric-base.config`)
- Source files: `snake_case` (Rust), `kebab-case` (shell scripts)
- Documentation: `UPPER-kebab-case.md` for phase docs, `lowercase.md` for guides
- ADRs: `NNNN-kebab-case.md`
