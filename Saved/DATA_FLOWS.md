# Data and Control Flows

The flows below distinguish a code path present in the repository from the larger target described by ADRs and master-plan docs. Protocol names do not prove that a service is installed, running, or activated correctly.

## Boot and desktop startup

```text
firmware/boot entry → UKI + image integrity path → kernel → systemd
                                                    ├─ litric-sysd
                                                    ├─ litric-envd (unit configured)
                                                    └─ litric-shell
                                                         ├─ framebuffer discovery via sysfs
                                                         ├─ input-source discovery
                                                         ├─ facts via sysd or direct fallback
                                                         └─ software Canvas → framebuffer write()
```

- Image/ISO configuration defines boot entries, profile content, services and recovery paths. `docs/master-plan/25-beta-1.0-gate-matrix.md` records evidence-level gaps; use the dated evidence rather than inferring verification from configuration.
- Shell startup is designed to report headless/no-input/no-daemon fallbacks rather than pretending each resource is available.
- The shell's direct framebuffer output is the current display model; the planned Smithay/Wayland client stack is not yet represented by the same code.

## Shell input to UI state to pixels

```text
/dev/input/event* + sysfs → litric-input backend → InputEvent
    → litric-shell::Shell::handle → model/state/effects
    → litric-shell::views + ui + canvas → damage/repaint → framebuffer
```

`litric-wm` owns independent window geometry/focus/z-order/workspace decisions. The shell registers hotspots during drawing; input uses those and shell state. `litric-design` supplies theme, palette, glass, typography, spacing, radii and motion tokens. `litric-text` shapes/rasterizes installed fonts, with vector fallback logic in shell font code. Synthetic input and E2E model journeys test behaviors without a physical display; they are not hardware E2E.

## System facts and actions

```text
shell → Unix socket /run/litric/sysd.sock → litric-sysd protocol parser
     → Request → state + capability checks + provider plan/runner
     → typed response lines → shell model → UI
```

- Sysd facts read `/proc`, `/sys` and configured root paths/fixtures; network/audio/power/Bluetooth/session fact modules are mostly parsers/readers, while provider actions use validated arguments and a runner seam.
- Power/provider actions are capability and (for destructive/sensitive operations) live-consent gated, with audit records in the in-memory service state. Credentials are intended for helper stdin and omitted from response output.
- The shell and daemon both refer to `/run/litric/sysd.sock`; the exact CLI default and units should be compared in `CONFIGURATION_REFERENCE.md`.
- Source explicitly limits claims: sysd trash is in memory; environment switching updates policy state and does not launch a VM; configured local identity is not the same as authenticated per-peer identity.

## Logical storage and file operations

```text
Shell Files model
   → fsbridge (host/test implementation or Unix litric-files ops)
   → StorageUri / LogicalPath validation
   → path confinement + capability authorization
   → filesystem read/write/move/trash/archive
   → LFP/1 typed response → model updates → view redraw
```

- `litric-storage` models URI schemes, owner scoping, traversal rejection, permission requirements, and logical-to-physical resolution. It does not itself open files.
- `litric-files` implements file operation/protocol/archive code; Unix file operations enforce root confinement and avoid following symlinks according to ADR-0019. Trash metadata uses freedesktop layout and collision-safe state.
- The shell's fsbridge also provides development/host paths and caps previews, search depth/entries, and archive work. Do not confuse this adapter with the privileged/service deployment contract.
- The service unit files for `litric-filesd` reside in `image/units-pending`; inspect actual binary target and unit installation before claiming socket activation.

## Browser navigation and rendering

```text
address input → url::interpret → Policy::check_navigation → tab/history state
    → broker LBP/1 → browserd → Chromium via CDP pipe
    ← TAB/NOTICE/FRAME events ← JPEG decode to bounded RGBA frame
```

- Browser library behavior is testable with `NoEngine`, which fails honestly. Policy models HTTPS-only typed input, tracker refusal and per-origin permissions.
- `litric-browserd` owns web-derived CDP JSON/base64/JPEG processing, isolated from the shell. It is optional behind the `broker` feature and the service socket path should match the image config; current code and ADR have differing defaults noted as unresolved.
- A render engine, broker binary, sandbox, and socket unit all need independent runtime verification. URL-model tests are not an interactive browser proof.

## Capability and storage authority

```text
manifest bytes → strict litric-manifest parser → validated AppId + requested caps
                                                ↓ (consent UX / policy, target)
caller identity + requested Capability → litric-caps::Broker → allow/deny + audit
logical URI + caller + grants → litric-storage::StorageBroker → path or typed denial
```

The library model denies by default; scopes are resource-specific; expiry/revocation apply to subsequent decisions; each decision records an audit entry. Actual caller identity authentication, grant persistence, prompting, and daemon IPC are separate concerns. Sysd source documents a local configured identity and lack of per-connection SO_PEERCRED identity integration as an open boundary.

## Environment switching

```text
hotkey → shell → EnvironmentManager::plan_switch → user-visible outcome
                  → (target) envd / KVM lifecycle / surface handoff
```

The currently implemented environment manager is a pure state/resource/switch policy model. `switch_to` changes its active model state; it is not evidence that a guest starts, resumes, allocates a VM, or presents a guest surface. `litric-envd` polls supervised children and records exit information; complete guest lifecycle/control integration remains unresolved.

## Defender scanning and benchmark flow

```text
content bytes → litric-defender::Engine → hash/denylist → pattern matcher
              → verdict policy/cache → ScanReport
litric-bench corpus → real defender engine → false-positive/negative analysis
                   → hash-chained event log / CSV / JSON / Markdown report
```

The defender crate is pure policy/scan code with no file hook or daemon. The benchmark uses deterministic synthetic data and explicit safety gates. The presence of EICAR test input in unit tests is not evidence of on-access antivirus on a running image.

## Locale and time flow

```text
UTC epoch supplied by caller + TZ/system locale config
    → litric-locale::Regional::detect
       ├─ TZif zone offset → local civil datetime
       └─ locale preference → formatted time/date/number/units
    → shell labels / clock
```

Timezone and formatting preferences are separate. Unknown sources report explicit UTC/C fallbacks. Locale crate functions are deterministic for injected inputs, but zone/config detection reads the supplied root.

## Error, limits, and observability

Protocols are line-oriented and bounded; free text uses strict percent encoding in shared core where used. Typed errors carry stable kinds. Diagnostics flow to the shell/sysd status outputs and logs; screenshots, serial logs, and benchmark outputs under `docs/evidence/`, `evidence/`, and `artifacts/` are historical artifacts whose dates and environment must stay attached to claims.