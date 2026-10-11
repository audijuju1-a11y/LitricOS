# ILTRIC NATIVE ECOSYSTEM — MASTER ARCHITECTURE SPECIFICATION v1.0

Status: Foundational / Normative
Project: Iltric OS
Primary Goal: Define a genuinely native application ecosystem in which Iltric applications execute directly against Iltric's own system contracts, while Windows compatibility remains an optional fallback and is never a dependency of the Iltric core.

---

## 0. EXECUTIVE PRINCIPLE

Iltric is a self-hosted operating system ecosystem. The native platform is not a themed shell, wrapper, or Windows/Linux distribution with a different desktop. The native platform is a coherent contract extending from boot, kernel, system services, runtime, ABI, application model, package format, security model, graphics, audio, networking, storage, developer tooling, and distribution.

The central architectural rule is:

> Native Iltric applications MUST depend on Iltric contracts, not on Windows contracts. Windows compatibility MUST depend downward on Iltric contracts and MUST NOT become a prerequisite for booting, running, updating, securing, or developing the native operating system.

The architecture therefore has a strict direction of dependency:

Hardware
  ↓
Boot / Firmware Interface
  ↓
Iltric Kernel + HAL + Drivers
  ↓
Iltric Core Services
  ↓
Iltric System ABI / API
  ↓
Iltric Runtime + SDK
  ↓
Native Applications

And separately:

Windows Application
  ↓
Windows Compatibility Runtime
  ↓
Windows API Emulation / Translation
  ↓
Iltric System ABI / API
  ↓
Iltric Core / Kernel

The compatibility branch is subordinate to the native branch.

---

# 1. DESIGN GOALS

## 1.1 Primary goals

1. Native applications must have a first-class execution model designed specifically for Iltric.
2. The operating system must have stable system contracts independent of user-interface implementation.
3. Application crashes must be isolated from the kernel and from unrelated applications.
4. Application permissions must be explicit, reviewable, revocable, and preferably capability-based.
5. Native packages must be reproducible, signed, versioned, dependency-aware, and update-safe.
6. ABI stability must be treated as a first-class compatibility problem.
7. Graphics, audio, networking, storage, IPC, input, notifications, and power management must expose coherent native APIs.
8. Native applications must not need a Windows environment merely to access common OS functionality.
9. Windows compatibility must be sandboxable and disposable on a per-application or per-profile basis.
10. The ecosystem must support both consumer applications and low-level professional software.
11. The native stack must permit hardware acceleration without requiring applications to understand hardware-specific details.
12. Iltric must remain operable even if all compatibility components are removed.

## 1.2 Non-goals

1. Reimplementing Windows internally and calling that native.
2. Requiring NTFS, registry internals, Win32, DirectX, or Windows services as fundamental Iltric primitives.
3. Making every subsystem accessible directly from arbitrary application code.
4. Treating package files as trusted merely because they were downloaded from an official server.
5. Assuming software secrecy is a security boundary. Native binaries are assumed observable by the machine owner.

---

# 2. ARCHITECTURAL LAYERS

## 2.1 Layer 0 — Firmware and Boot

Responsibilities:
- Hardware discovery required for boot.
- Secure/verified boot chain where supported.
- Kernel image verification.
- Initramfs / early userspace loading.
- Recovery path.
- Rollback metadata.

The boot chain must never require a Windows installation.

## 2.2 Layer 1 — Iltric Kernel

The kernel owns privileged mechanisms:
- CPU scheduling.
- Address-space management.
- Physical memory management.
- Virtual memory.
- Process and thread primitives.
- Interrupt handling.
- Timers.
- Kernel IPC primitives.
- Synchronization primitives.
- Device model.
- DMA/IOMMU integration.
- Security enforcement primitives.
- Kernel object lifetime.
- Low-level filesystem and storage interfaces.
- Driver isolation hooks.

The kernel should expose the minimum primitive surface required by system services. Rich policy belongs above the kernel whenever possible.

## 2.3 Layer 2 — Hardware Abstraction and Driver Framework

The HAL provides stable abstractions for:
- CPU features.
- Timers.
- Interrupts.
- Memory mapping.
- PCIe / USB / bus discovery.
- Display/GPU.
- Audio.
- Network.
- Storage.
- Input.
- Power.
- Cameras and sensors.

Drivers must be versioned against explicit driver interfaces. The driver ABI must not silently drift with arbitrary kernel changes.

## 2.4 Layer 3 — Iltric Core Services

Core services are privileged system components that implement policy and reusable mechanisms:
- Init / service supervisor.
- Device manager.
- Session manager.
- Security broker.
- Identity service.
- VFS service.
- Storage service.
- Package service.
- Update service.
- IPC broker.
- Time service.
- Network management.
- Audio service.
- Graphics service.
- Input service.
- Notification service.
- Power service.
- Crash/report service.
- Compatibility manager.

Core services should communicate over authenticated IPC rather than allowing unrelated applications direct privileged access.

---

# 3. NATIVE SYSTEM CONTRACT

The native system contract consists of four coordinated interfaces:

1. Syscall / Kernel Primitive ABI.
2. System Service IPC ABI.
3. Public Iltric Application ABI.
4. High-level SDK APIs.

Applications should normally use the SDK or stable public API. They should not bind directly to unstable kernel internals.

## 3.1 Contract hierarchy

Kernel ABI
  → used by privileged system components and selected low-level runtimes

System Service ABI
  → used by runtimes and trusted services

Public Application ABI
  → supported compatibility boundary for native applications

SDK
  → ergonomic interface for developers

This separation prevents the GUI toolkit, compiler, or package manager from becoming the definition of the OS itself.

---

# 4. NATIVE APPLICATION MODEL

## 4.1 Application identity

Every native application MUST have a stable identity:

- Application ID: globally unique reverse-domain or registry-based identifier.
- Publisher identity.
- Signing identity.
- Version.
- ABI target.
- Required runtime profile.
- Declared capabilities.
- Entrypoints.
- Optional background services.

Example:

app_id = com.iltric.example.editor
publisher = Example Studio
version = 3.2.0
abi = iltric-native-v1
runtime = iltric-runtime-v1

## 4.2 Application boundaries

An application consists of:
- Immutable code/resources package.
- Writable per-user data.
- Writable temporary data.
- Optional cache.
- Optional secure secret store namespace.
- Optional background component.
- Manifest and policy.

The application must not assume it can arbitrarily write to system directories.

## 4.3 Lifecycle

Canonical lifecycle:

INSTALL → VERIFY → PREPARE → LAUNCH → RUN → SUSPEND/RESUME → EXIT → CLEANUP

Crash lifecycle:

RUN → FAULT → ISOLATE → CAPTURE MINIMAL DIAGNOSTICS → TERMINATE → RECOVER

The lifecycle manager must not need to restart the whole desktop for an ordinary application crash.

---

# 5. NATIVE EXECUTABLE AND PACKAGE MODEL

Iltric should distinguish executable format from distribution package format.

Recommended conceptual split:

.ilx   = executable image / native binary
.ilpkg = distribution package
.ilapp = application bundle / installable app identity

The exact file extensions are implementation choices; the separation of roles is normative.

## 5.1 Executable format requirements

The native executable format should support:
- Architecture identification.
- Endianness.
- Entry point.
- Segments and permissions.
- Relocations.
- Dynamic dependencies.
- Symbol/debug references.
- Build metadata.
- Integrity metadata.
- Optional control-flow and hardening metadata.
- Lazy/eager binding policy.
- Compatibility versioning.

Security properties should be represented explicitly rather than inferred from arbitrary flags.

## 5.2 Package requirements

A package should contain:
- Manifest.
- One or more platform payloads.
- Resources.
- Dependency declarations.
- Capability declarations.
- Signature metadata.
- Build provenance.
- Optional update delta information.

Packages must be verifiable before installation.

---

# 6. CAPABILITY-BASED SECURITY

The default security model should be deny-by-default.

An app receives only the resources/capabilities necessary for its declared function.

Example capability classes:

storage.user_documents
storage.media
network.outbound
network.listen
microphone
camera
location
notifications
clipboard.read
clipboard.write
gpu.compute
gpu.graphics
audio.output
audio.input
bluetooth
usb.device
secure_storage
process.debug
system.settings
service.background

Dangerous capabilities should be mediated by policy and user/system consent.

## 6.1 Object-level authorization

Whenever practical, capabilities should refer to specific resources rather than broad global access.

Bad:

"filesystem.read_all"

Better:

"document://user/Photos/2026"

This reduces blast radius if an app is compromised.

---

# 7. MEMORY AND PROCESS ISOLATION

Every application receives an independent virtual address space by default.

Required protection boundaries:
- User vs kernel.
- App vs app.
- App vs system service.
- Compatibility environment vs native system.

Native apps must never assume shared writable memory unless a specific shared-memory capability is negotiated.

High-performance shared memory should be supported through explicit handles with permission metadata.

---

# 8. IPC ARCHITECTURE

IPC should be a first-class native facility.

Supported conceptual modes:

1. Request/response RPC.
2. One-way messages.
3. Event streams.
4. Shared-memory channels.
5. Handle passing.
6. Signal/event primitives.

Each IPC endpoint should have:
- Identity.
- Authentication state.
- Capability requirements.
- Message version.
- Quotas.
- Rate policy where needed.

System services should expose versioned IPC contracts so that the SDK can evolve without kernel changes.

---

# 9. GRAPHICS

Graphics should be layered:

Application UI
  ↓
Iltric UI toolkit
  ↓
Windowing / compositor protocol
  ↓
Graphics abstraction
  ↓
GPU runtime
  ↓
GPU driver

The native rendering API should support:
- 2D.
- 3D.
- GPU compute.
- Presentation.
- Synchronization.
- Explicit resource lifetime.
- Multi-display.
- High-DPI scaling.
- HDR/color management where available.

A compatibility backend may translate Windows graphics APIs to the native graphics stack, but native applications should avoid that path entirely.

---

# 10. AUDIO

Native audio should have a deterministic, low-latency engine:

App
 ↓
Native Audio API
 ↓
Mixer / Spatial / DSP graph
 ↓
Audio service
 ↓
Driver
 ↓
Hardware

The API should support:
- Shared and exclusive-style modes where appropriate.
- Low latency.
- Spatial audio.
- Sample-rate conversion.
- Device routing.
- Per-application volume.
- Capture permissions.
- Real-time-safe audio callbacks.

Real-time callbacks must not perform arbitrary blocking IPC, unbounded allocations, or filesystem work.

---

# 11. STORAGE

Native applications must use a logical storage model rather than assuming fixed device paths.

Example namespaces:

user://Documents/
user://Pictures/
app://com.example.app/data/
cache://com.example.app/
config://com.example.app/
temp://com.example.app/
secure://com.example.app/

The storage broker translates namespaces into underlying filesystem locations.

This decouples applications from physical disk layout.

---

# 12. NETWORKING

Native network access should expose:
- Socket abstractions.
- HTTP/TLS client APIs.
- DNS resolution.
- Service discovery.
- Secure credential handling.
- Optional sandbox restrictions.

Applications should not directly modify global network configuration unless explicitly privileged.

---

# 13. DEVELOPER ECOSYSTEM

Native development requires first-party tooling:

Iltric SDK
Iltric compiler/runtime bindings
Iltric linker
Iltric package builder
Iltric debugger
Iltric profiler
Iltric crash analyzer
Iltric UI designer/toolkit
Iltric API documentation generator
Iltric package signer
Iltric emulator/test environment

The SDK must expose stable headers/modules/contracts and version targets explicitly.

---

# 14. ABI GOVERNANCE

The public native ABI is a product, not an implementation detail.

Every public ABI revision needs:
- Version identifier.
- Compatibility matrix.
- Deprecation policy.
- Migration documentation.
- Binary compatibility tests.
- Semantic compatibility tests.

Recommended policy:

Major ABI version:
  breaking changes permitted with migration path.

Minor version:
  additive features where possible.

Patch version:
  bug/security fixes without contract changes.

A compatibility shim can allow old applications to continue running while the system evolves.

---

# 15. PACKAGE AND UPDATE SECURITY

Every installable package should have:
- Cryptographic signature.
- Content hash.
- Publisher identity.
- Version.
- Dependency constraints.
- Target ABI.
- Optional transparency/provenance data.

Updates should use an atomic strategy:

DOWNLOAD → VERIFY → STAGE → ACTIVATE → VERIFY BOOT/RUNTIME → COMMIT

On failure:

ROLLBACK → MARK BAD VERSION → REPORT

The system must avoid half-installed native components.

---

# 16. NATIVE STORE

The Iltric Store should understand the native package model directly.

Store metadata should include:
- App ID.
- Publisher.
- Supported Iltric versions.
- Supported architectures.
- Requested capabilities.
- Signing status.
- Version history.
- Dependency tree.
- Update channels.

The store is not the trust root by itself. Cryptographic package signatures remain authoritative.

---

# 17. WINDOWS COMPATIBILITY BOUNDARY

Windows compatibility is a separate subsystem.

It may include:
- PE loader.
- Windows API compatibility runtime.
- DLL environment.
- Registry virtualization.
- Windows path virtualization.
- Graphics translation.
- Audio translation.
- Input mapping.
- COM compatibility.
- Windows runtime components where legally and technically appropriate.

The compatibility subsystem must consume Iltric resources through documented interfaces.

It must never directly redefine Iltric kernel semantics.

## 17.1 Isolation principle

A Windows application should run inside a profile/environment that can be:
- Created.
- Suspended.
- Reset.
- Repaired.
- Migrated.
- Deleted.

The compatibility environment should be disposable.

## 17.2 Native priority

When both a native app and a Windows app exist for the same product, the system should prefer native by policy unless the user explicitly selects another version.

---

# 18. PERFORMANCE OBJECTIVE

Native execution should minimize translation boundaries.

Native:

App → SDK → Native Runtime/System API → Kernel/Services → Driver

Compatibility:

Windows App → Windows API layer → Translation → Native Runtime/System API → Kernel/Services → Driver

Therefore native should have lower latency, lower overhead, clearer profiling, and fewer compatibility failure modes.

---

# 19. OBSERVABILITY

The system should expose native observability without compromising privacy.

Required tooling:
- Process inspection.
- Thread inspection.
- Memory maps.
- CPU profiling.
- GPU profiling.
- Audio latency diagnostics.
- IPC tracing.
- Filesystem access tracing.
- Crash dumps.
- Service health.
- Package provenance.

Access to sensitive diagnostics should require elevated capabilities.

---

# 20. CRASH AND RECOVERY MODEL

A user-space crash should normally affect only:
- The crashed app.
- Its owned helper processes.
- Optional dependent user-space services.

The system should restart user services selectively.

Kernel crashes are a separate failure class and require crash capture + recovery/rollback strategy.

---

# 21. TESTING REQUIREMENTS

Native ecosystem releases must pass:
- ABI compatibility tests.
- Syscall contract tests.
- IPC compatibility tests.
- Package verification tests.
- Permission enforcement tests.
- Sandbox escape regression tests.
- Resource quota tests.
- Upgrade/rollback tests.
- Crash isolation tests.
- Graphics backend tests.
- Audio latency tests.
- Network isolation tests.
- Storage corruption/recovery tests.

A native app should ideally be tested on a matrix of:
- Iltric release.
- CPU architecture.
- GPU backend.
- Driver revision.
- Feature flags.

---

# 22. ARCHITECTURAL INVARIANTS

The following rules must be treated as hard constraints unless this specification is deliberately revised:

1. Iltric must boot without Windows.
2. Iltric core services must operate without Windows.
3. Native applications must not require Win32.
4. The native package format must not require PE as its execution format.
5. Native security must not depend on obscurity of binaries.
6. Every privileged operation must have an explicit authorization path.
7. Compatibility environments must be lower-level dependencies than neither the kernel nor native apps.
8. Public APIs must be versioned.
9. Updates must be verifiable and recoverable.
10. Native app crashes must not normally crash the OS.
11. Application data must be separated from application code.
12. Physical filesystem layout must not be an API contract.
13. Graphics/audio/network/storage capabilities must be exposed through native interfaces.
14. Hardware-specific behavior belongs below the public app API.
15. Native is the primary product; compatibility is the bridge.

---

# 23. REFERENCE REPOSITORY STRUCTURE

Suggested source tree:

iltric/
├── boot/
├── kernel/
│   ├── arch/
│   ├── mm/
│   ├── sched/
│   ├── ipc/
│   ├── security/
│   ├── drivers/
│   └── fs/
├── hal/
├── core/
│   ├── init/
│   ├── devmgr/
│   ├── securityd/
│   ├── storaged/
│   ├── packaged/
│   ├── updated/
│   ├── netd/
│   ├── audiod/
│   ├── gfxd/
│   └── inputd/
├── runtime/
├── sdk/
├── abi/
├── ui/
├── tools/
├── debugger/
├── profiler/
├── package-format/
├── store/
├── compat/
│   └── windows/
└── tests/

The repository layout is intentionally aligned to the architectural dependency graph.

---

# 24. END STATE

A successful Iltric installation should feel like:

"This is an operating system with its own software world. Windows programs are supported when needed, but they are guests inside Iltric rather than the foundation of Iltric."

That distinction is the defining property of the Iltric Native Ecosystem.
