# ILTRIC NATIVE RUNTIME, ABI, PROCESS, IPC & SECURITY — DEEP SPECIFICATION v1.0

Status: Foundational / Technical Design
Project: Iltric OS
Scope: Native execution semantics beneath the SDK and above kernel primitives.

This document defines how a native Iltric application should actually execute, communicate, allocate memory, load dependencies, obtain capabilities, interact with graphics/audio/storage/network services, survive upgrades, and remain binary-compatible as the operating system evolves.

---

# 0. CORE THESIS

A native ecosystem succeeds only when its execution contract is explicit.

A compiler is not the native ecosystem.
A UI toolkit is not the native ecosystem.
A package manager is not the native ecosystem.
The native ecosystem is the complete set of contracts that lets a binary built today execute safely and predictably on future Iltric releases.

The key stack is:

Application Source
  ↓
Compiler / Language Runtime
  ↓
Iltric ABI / Calling Convention
  ↓
Iltric Native Runtime
  ↓
System API / IPC
  ↓
Kernel Primitives
  ↓
Hardware

The Windows compatibility stack is intentionally separate:

Windows Binary
  ↓
PE Loader
  ↓
Windows Compatibility Runtime
  ↓
Windows API Emulation / Translation
  ↓
Iltric System API
  ↓
Iltric Kernel

---

# 1. NATIVE EXECUTION CONTRACT

## 1.1 Process creation

A native application launch should conceptually execute these stages:

1. Resolve application identity.
2. Verify package/code signatures.
3. Resolve target architecture.
4. Resolve required ABI/runtime.
5. Resolve declared capabilities.
6. Create process address space.
7. Create initial thread.
8. Map executable segments.
9. Map permitted libraries.
10. Create application IPC channels.
11. Establish environment namespaces.
12. Install capability handles.
13. Initialize runtime.
14. Invoke application entry point.
15. Register lifecycle/health hooks.

The launcher must not grant ambient authority simply because a package is installed.

## 1.2 Entry-point semantics

The native executable must receive a minimal deterministic startup environment:
- Argument vector.
- Environment namespace.
- Application identity.
- Instance ID.
- Capability table.
- IPC bootstrap endpoint.
- Runtime configuration.
- Locale/timezone context where permitted.

Global mutable process state should be minimized.

---

# 2. PROCESS MODEL

Each process should have:
- Process ID.
- Application identity.
- Security identity.
- Capability table.
- Virtual address space.
- Handle table.
- Thread list.
- Resource quotas.
- Lifecycle state.
- Parent/supervisor relationship.
- Optional service role.

Recommended process states:

CREATED
→ STARTING
→ RUNNING
→ SUSPENDED
→ STOPPING
→ EXITED
→ FAILED

The kernel owns enforcement. The session/service manager owns policy and supervision.

---

# 3. THREAD MODEL

A process may contain multiple threads with explicit scheduling class.

Conceptual classes:
- Normal user thread.
- Latency-sensitive thread.
- Background worker.
- Real-time audio callback class, where platform guarantees support it.
- System/driver worker.

Applications should request scheduling properties rather than assume direct control over CPU scheduling.

The scheduler must enforce CPU quotas/fairness and prevent a malicious application from monopolizing resources.

---

# 4. MEMORY MODEL

## 4.1 Virtual memory

Every native process receives a private virtual address space.

Memory regions must have explicit permissions:
- Read.
- Write.
- Execute.

W^X-style policy is recommended where practical: writable pages should not simultaneously be executable unless a narrowly scoped runtime mechanism is used and audited.

## 4.2 Shared memory

Shared memory requires:
- Explicit creation.
- Explicit capability transfer.
- Access permissions.
- Lifetime semantics.
- Synchronization protocol.

Shared memory should never become an ambient global scratch space.

## 4.3 Memory allocation

User-space allocators should be independent of kernel allocation internals.

Runtime allocator responsibilities:
- Small allocations.
- Large mappings.
- Arena management.
- Thread-local caches.
- Alignment.
- Debug instrumentation in development builds.

Kernel responsibilities:
- Physical pages.
- Virtual mappings.
- Page faults.
- Memory protection.
- Accounting.

---

# 5. ABI DESIGN

The ABI is the binary contract between compiled native code and the platform.

It must specify:
- Calling convention.
- Integer and floating-point representation.
- Pointer size/alignment.
- Struct layout rules.
- Enum representation.
- Error representation.
- Name mangling policy if applicable.
- Exception/unwind model.
- TLS semantics.
- Atomic memory model.
- Symbol visibility.
- Dynamic linking semantics.
- Library version rules.

## 5.1 ABI stability layers

Use three layers:

L0 — Internal compiler/runtime ABI
L1 — Stable Iltric Native ABI
L2 — High-level language SDK APIs

L1 is the critical long-term contract.

## 5.2 ABI handles

Opaque handles are strongly recommended for OS-managed objects.

Instead of exposing:

struct InternalWindow {
    implementation details...
};

Use:

IlWindowHandle

The application receives a handle and uses documented calls.

This prevents internal data-structure changes from breaking binary compatibility.

---

# 6. ERROR MODEL

The native API should use typed, machine-readable errors.

An operation should ideally produce:

success OR structured error

Error fields may include:
- Domain.
- Numeric code.
- Retryability.
- Permission status.
- Resource status.
- Human-readable diagnostic token.

Applications must not parse English error strings to determine program behavior.

---

# 7. HANDLE MODEL

Kernel/system resources should be referenced through opaque handles.

Potential handle types:
- Process.
- Thread.
- File.
- Directory.
- Socket.
- Window.
- GPU resource.
- Audio stream.
- Shared memory.
- Event.
- Service endpoint.
- Credential/secure-store object.

The handle table is per-process.

A handle must not become globally valid merely because the numerical value is guessed.

---

# 8. IPC ABI

## 8.1 Goals

IPC must be:
- Fast enough for normal system services.
- Secure by default.
- Versioned.
- Observable for debugging.
- Capable of handle transfer.
- Resistant to confused-deputy problems.

## 8.2 Message structure

Conceptual message:

Header
{
  protocol_id
  version
  message_type
  flags
  request_id
  payload_size
  capability_context
}

Payload

The wire layout must be explicitly specified and not depend on compiler-specific struct packing.

## 8.3 RPC version negotiation

Client and service negotiate a protocol version.

Rules:
- Additive fields may be optional.
- Unknown optional fields must be ignorable.
- Breaking changes require a new major protocol.
- The server should reject unsafe downgrade paths.

---

# 9. CAPABILITY ENGINE

Capabilities are the main application authorization mechanism.

A capability should conceptually contain:

capability_id
resource_type
resource_scope
rights
issuer
expiry(optional)
constraints

Examples:

CAP: audio.output / device=default / write
CAP: file / user://Documents/project / read+write
CAP: network / host=example.com / outbound
CAP: window / own-process / create

Capabilities should be non-forgeable, transferable only through authorized IPC, and revocable where the security architecture supports revocation.

---

# 10. SECURITY BROKER

Applications should not each reinvent permission enforcement.

The security broker evaluates:
- Application identity.
- Publisher identity.
- Requested capability.
- User consent.
- System policy.
- Device state.
- Session state.
- Risk policy.

Important rule:

Authentication answers "who are you?"
Authorization answers "what are you allowed to do?"
Capability possession answers "what resource authority was delegated to you?"

These are related but not interchangeable.

---

# 11. DYNAMIC LINKING

Dynamic libraries should have stable semantic identities and explicit ABI versions.

Recommended model:

lib.il.graphics.v1
lib.il.audio.v2
lib.il.storage.v1

Applications should bind to a supported interface contract, not blindly depend on a physical filename.

The loader should verify:
- Signature where policy requires.
- ABI compatibility.
- Architecture compatibility.
- Dependency closure.
- Load permissions.

---

# 12. RUNTIME SERVICES

The Native Runtime should provide standardized support for:
- Thread creation.
- Synchronization.
- Allocation.
- Timers.
- Async I/O.
- Futures/promises if supported.
- Logging.
- Unicode.
- Localization.
- Error propagation.
- Dynamic library loading.
- Application lifecycle integration.

The runtime must not become a hidden second kernel. Privileged operations remain system-service/kernel responsibilities.

---

# 13. ASYNCHRONOUS I/O

A modern runtime should avoid forcing applications into one blocking model.

Recommended primitives:
- Readiness events.
- Completion queues.
- Async file I/O.
- Async network I/O.
- Timers.
- Structured cancellation.

All asynchronous operations need explicit lifetime semantics so callbacks cannot reference destroyed application objects.

---

# 14. UI AND WINDOWING ABI

A native application should request a window through a stable object API:

CreateWindow
SetTitle
SetBounds
SetInputRegion
PresentFrame
SetState
CloseWindow

The implementation remains in the compositor/window manager.

Applications should not draw directly into global desktop memory.

## 14.1 Composition model

Recommended:

App renders → compositor consumes surface → compositor composites → display backend presents.

This allows:
- Isolation.
- Scaling.
- Effects.
- Multiple windows.
- Display topology management.
- Crash-safe cleanup.

---

# 15. GRAPHICS RESOURCE MODEL

Graphics resources must have lifetime-managed handles:
- Device.
- Context/queue.
- Buffer.
- Texture.
- Shader.
- Pipeline.
- Surface.
- Fence/event.

The native API should avoid exposing raw driver pointers.

A GPU backend may internally target Vulkan or another lower-level graphics technology, but that is an implementation detail. The native API remains stable above it.

---

# 16. AUDIO RUNTIME

The audio architecture must separate normal app threads from hard real-time callback code.

Recommended two-tier model:

Tier A — Real-time audio callback
- No blocking locks.
- No filesystem I/O.
- No unbounded allocation.
- No arbitrary system calls.
- Only bounded operations and preallocated data.

Tier B — Audio worker/service
- Device management.
- Routing.
- DSP configuration.
- Buffer preparation.
- Non-real-time communication.

The application uses a thread-safe control plane and a real-time data plane.

This is important for low-latency spatial audio and professional audio workloads.

---

# 17. STORAGE ABI

Applications should operate on logical URIs/namespaces.

Examples:

user://Documents/file.txt
app://com.example.editor/config.json
cache://com.example.editor/thumbnails
secure://com.example.editor/token

The storage subsystem owns:
- Path translation.
- Access controls.
- Encryption policy.
- Quotas.
- Atomic writes.
- Journaling/safe commit.

An application must not assume /dev/nvme0n1 or C:\ exists in the native layer.

---

# 18. SECURE STORAGE

Secrets must not be stored in application binary or package resources.

Secure storage should provide:
- Per-application namespace.
- Access-controlled retrieval.
- Optional hardware-backed key integration.
- Rotation.
- Revocation.
- Audit events for privileged access.

Rule:

"A secret embedded in a native binary is not a secret from the machine owner."

---

# 19. NATIVE NETWORK ABI

Provide structured interfaces for:
- DNS.
- TCP/UDP or modern transport abstractions.
- TLS.
- HTTP.
- WebSocket/streaming where needed.
- Proxy policy.
- Firewall integration.

The runtime should support cancellation and timeouts as first-class fields.

---

# 20. APPLICATION SANDBOX

A native application sandbox should define:
- Filesystem namespace.
- Device visibility.
- Network policy.
- IPC policy.
- Process creation policy.
- Debugging policy.
- Hardware access policy.
- Resource limits.

A sandbox profile should be declarative and signed with the application metadata.

The platform may grant a narrower policy than requested.

---

# 21. RESOURCE GOVERNANCE

Each process should be subject to resource accounting.

Possible limits:
- CPU time.
- Memory.
- Threads.
- Open handles.
- GPU memory.
- Audio streams.
- Network sockets.
- Storage quota.
- Background execution.

These prevent a badly behaved app from dominating the platform.

---

# 22. DEBUGGING AND DEVELOPMENT MODE

Developer mode should intentionally relax selected constraints under explicit system control.

Capabilities may include:
- Debug own process.
- Inspect own memory.
- Load unsigned development modules.
- Attach profiler.
- Capture graphics trace.
- Capture audio trace.

Production mode should not silently inherit developer privileges.

---

# 23. CRASH DUMPS

Native crash diagnostics should capture enough information for debugging without automatically dumping secrets.

Possible payload:
- Process metadata.
- Thread list.
- Fault address.
- Register snapshot.
- Stack traces.
- Loaded module hashes.
- Selected logs.
- Build ID.

Memory capture must be policy-controlled because raw memory can contain credentials and user data.

---

# 24. JIT / SELF-MODIFYING CODE

If Iltric supports JIT runtimes, the system should enforce an explicit memory-transition policy.

Safer conceptual flow:

Writable code generation buffer
  ↓
Runtime validation
  ↓
Transition to executable state
  ↓
Execution

Do not allow arbitrary user apps to make uncontrolled RWX memory the default.

---

# 25. BINARY HARDENING

Native binaries should support platform hardening metadata where technically available:
- ASLR/PIE.
- Stack protection.
- Control-flow protection.
- Relocation hardening.
- Read-only relocations.
- Non-executable data.
- Shadow-stack or equivalent features where supported.
- Signed modules where policy requires.

These mechanisms increase resistance but are not substitutes for sandboxing and authorization.

---

# 26. LOAD-TIME INTEGRITY

Before mapping executable content, the loader should evaluate applicable policies:

1. Package integrity.
2. Publisher signature.
3. Revocation status.
4. ABI compatibility.
5. Architecture compatibility.
6. Dependency integrity.
7. Required capabilities.
8. System policy.

The loader should fail closed for security-critical integrity checks.

---

# 27. LIBRARY AND PLUGIN SECURITY

A plugin is not automatically trustworthy merely because the host application requested it.

Recommended plugin model:
- Separate identity.
- Separate signature.
- Explicit ABI version.
- Capability subset.
- Optional isolated process.
- Host-controlled lifecycle.

For high-risk plugins, out-of-process execution is preferable.

---

# 28. BACKGROUND SERVICES

Native applications may request background execution, but this must be managed.

Service metadata should specify:
- Trigger.
- Required capabilities.
- Resource class.
- Restart policy.
- User visibility.
- Persistence requirement.

Arbitrary applications should not create permanent privileged daemons without platform approval.

---

# 29. POWER MANAGEMENT

Applications should express intent:
- latency-sensitive
- interactive
- background
- batch
- wake-lock required

The OS decides actual CPU/device power behavior according to system policy.

---

# 30. COMPATIBILITY WITH OLD NATIVE APPS

The runtime should provide compatibility shims where feasible.

Example:

App built for il-native-v1
       ↓
compat shim
       ↓
current Iltric v3 system services

The goal is to let the platform evolve without forcing every developer to rebuild immediately.

---

# 31. VERSION NEGOTIATION

Every major service and runtime contract should advertise:

minimum_supported
maximum_supported
preferred_version
features_bitset

Feature negotiation is preferable to fragile operating-system-version checks.

Bad:

if IltricVersion >= 3.4 then use_feature_X

Better:

if ServiceCapabilities includes FEATURE_X then use_feature_X

---

# 32. PACKAGE INSTALLATION TRANSACTION

A robust installation transaction should be:

1. Download or receive package.
2. Verify hash.
3. Verify signature.
4. Parse manifest.
5. Validate compatibility.
6. Resolve dependencies.
7. Stage files.
8. Register metadata.
9. Prepare policy/capabilities.
10. Commit atomically.
11. Activate.
12. Validate launch.
13. Record successful activation.

On any critical failure:

Abort and remove staged state without corrupting the active install.

---

# 33. ATOMIC UPDATE MODEL

Use immutable versioned application payloads where practical.

Conceptual layout:

apps/com.example.app/versions/1.0.0/
apps/com.example.app/versions/1.1.0/
apps/com.example.app/current -> 1.1.0

Activation changes the pointer/state atomically.

Rollback:

current -> 1.0.0

This model greatly simplifies recovery from bad updates.

---

# 34. NATIVE DEVELOPMENT TOOLCHAIN

Recommended tooling pipeline:

Source
  ↓
Compiler
  ↓
Static analysis
  ↓
Linker
  ↓
Native ABI validation
  ↓
Hardening verification
  ↓
Reproducible package build
  ↓
Signing
  ↓
Test environment
  ↓
Repository submission

A package should be rejected if it fails declared ABI, capability, or integrity requirements.

---

# 35. REPRODUCIBLE BUILDS

The native ecosystem benefits greatly from reproducibility.

Build metadata should identify:
- Source revision.
- Compiler version.
- SDK version.
- Dependency hashes.
- Build flags.
- Target architecture.
- Reproducibility status.

The system should be able to compare independently built artifacts.

---

# 36. SYSTEM CALL POLICY

The public app should not directly depend on undocumented syscalls.

Preferred path:

Application → Stable API/IPC → System service → Stable syscall primitive

This allows kernel internals to evolve without breaking applications.

Low-level native runtimes may use specialized syscalls only through an explicit supported ABI.

---

# 37. CONFUSED-DEPUTY DEFENSE

A privileged service must never assume:

"The caller requested it, therefore the caller is entitled to the resource."

Instead it must validate the capability and resource scope attached to the request.

Example:

App A asks storage service to delete file F.

Storage service verifies:
- Caller identity.
- Capability.
- Resource scope.
- Operation rights.

Without this, a compromised app could use a legitimate privileged service as a deputy to attack another user's data.

---

# 38. IPC RESOURCE OWNERSHIP

Every transferred resource must have explicit ownership/lifetime semantics.

Possible semantics:
- Borrowed.
- Shared.
- Transferred.
- Duplicated.
- Revoked.

This avoids use-after-close and hidden lifetime bugs in system services.

---

# 39. NATIVE PERFORMANCE CONTRACT

The native path must be measurable.

Important metrics:
- Process startup time.
- Time-to-first-window.
- IPC latency.
- Syscall overhead.
- Frame latency.
- GPU submission latency.
- Audio callback deadline misses.
- Storage I/O latency.
- Network round-trip overhead.
- Memory overhead.

The platform should provide profiling interfaces that distinguish native costs from compatibility-layer costs.

---

# 40. NATIVE VS WINDOWS COMPATIBILITY METRICS

Native:

Application
→ Iltric API
→ Iltric runtime
→ native service
→ driver

Compatibility:

Windows application
→ Windows API
→ translation
→ Iltric API
→ Iltric runtime/service
→ driver

The diagnostic system should be able to display this path so developers can identify where overhead originates.

---

# 41. COMPATIBILITY ENVIRONMENT SECURITY

The Windows compatibility environment must not be trusted as a privileged subsystem.

Recommended isolation:
- Separate process tree.
- Separate filesystem namespace.
- Separate registry namespace.
- Restricted device visibility.
- Controlled network policy.
- Controlled IPC.
- No ambient access to native application memory.

A Windows app should be considered untrusted code even if its publisher is trusted, because its dependencies and execution behavior may differ from the user's expectations.

---

# 42. NATIVE API DESIGN RULES

Rule 1: Prefer opaque handles over exposed internal structs.
Rule 2: Prefer explicit lifetimes over implicit global state.
Rule 3: Prefer feature negotiation over version sniffing.
Rule 4: Prefer capability checks over permission names alone.
Rule 5: Prefer structured errors over strings.
Rule 6: Prefer immutable data for cross-thread ownership transfer.
Rule 7: Prefer bounded real-time operations in audio/media paths.
Rule 8: Prefer asynchronous APIs for long operations.
Rule 9: Prefer stable contracts over implementation leakage.
Rule 10: Never make the Windows API the definition of Iltric.

---

# 43. RECOMMENDED NATIVE API DOMAINS

Il.Process
Il.Thread
Il.Memory
Il.Handle
Il.Ipc
Il.Security
Il.Storage
Il.File
Il.Network
Il.Window
Il.Graphics
Il.Gpu
Il.Audio
Il.Input
Il.Notification
Il.Device
Il.Power
Il.Time
Il.Crypto
Il.Package
Il.Diagnostics
Il.Compatibility

Namespaces are conceptual and may be renamed during implementation.

---

# 44. EXAMPLE NATIVE APPLICATION FLOW

Launch:

User selects app
  ↓
Shell requests launch
  ↓
Application manager resolves App ID
  ↓
Package manager verifies active version
  ↓
Security broker evaluates capabilities
  ↓
Runtime selects ABI
  ↓
Loader maps executable and dependencies
  ↓
Process created
  ↓
Capabilities installed
  ↓
IPC bootstrap established
  ↓
Runtime starts
  ↓
App entry point executes
  ↓
Window created
  ↓
Graphics surfaces allocated
  ↓
Audio/network/storage handles opened as needed

Exit:

App exits
  ↓
Runtime closes resources
  ↓
System reclaims handles/capabilities
  ↓
Session manager records result

---

# 45. REFERENCE NATIVE PROJECT STRUCTURE

Example developer project:

MyApp/
├── manifest.il
├── src/
│   ├── main.il
│   ├── ui.il
│   └── model.il
├── assets/
├── permissions/
│   └── capabilities.il
├── tests/
├── build.il
└── package.il

Build:

ilbuild
  ↓
compile
  ↓
link
  ↓
ABI validate
  ↓
harden
  ↓
package
  ↓
sign

---

# 46. SECURITY BOUNDARY MAP

Trusted:
- Boot verification chain.
- Kernel.
- Verified core services.
- Security policy database.
- Package verification logic.

Partially trusted:
- Signed native apps.
- Signed plugins.
- Compatibility runtimes.

Untrusted by default:
- User documents.
- Network input.
- Arbitrary native application data.
- Windows binaries.
- Plugin content.
- Downloaded assets.

The system should not collapse these trust levels merely because components share the same publisher.

---

# 47. DEFENSE-IN-DEPTH PRINCIPLE

No single mechanism should be considered sufficient.

Example defense chain for a native app:

Package signature
  ↓
Integrity verification
  ↓
ABI validation
  ↓
Capability policy
  ↓
Sandbox
  ↓
Process isolation
  ↓
Memory protection
  ↓
Resource quotas
  ↓
Runtime checks
  ↓
Crash isolation
  ↓
Update rollback

If one layer fails, the next should still reduce impact.

---

# 48. FUNDAMENTAL LIMIT: CLIENT SOFTWARE IS OBSERVABLE

Iltric must assume that anyone who owns a machine can inspect local application binaries, memory at runtime, and local resources subject to their control.

Therefore:
- Do not embed master secrets in apps.
- Do not depend on hidden API keys.
- Do not treat obfuscation as authorization.
- Keep authority on security-sensitive services.
- Use cryptographic verification for authenticity.
- Use capabilities and policy for authorization.

This is an architectural law, not an optional security feature.

---

# 49. NATIVE ECOSYSTEM SUCCESS CRITERIA

The native platform is considered architecturally successful when:

1. A developer can build an app without a Windows SDK.
2. A native app can boot/run without Windows compatibility components installed.
3. A native app has stable binary/API contracts.
4. Apps can be sandboxed with capabilities.
5. Applications can be installed/updated atomically.
6. The desktop and system services are themselves native applications/services.
7. Graphics and audio have native low-latency paths.
8. Windows apps are optional and isolated.
9. Old native applications can continue through ABI/runtime compatibility mechanisms.
10. Iltric can evolve without redefining every application contract at every release.

---

# 50. FINAL DESIGN LAW

Iltric should not try to become a better Windows compatibility layer.

Iltric should become a complete operating-system platform first, then make Windows software work inside it.

The ordering matters:

FIRST:

Kernel → Core Services → Native ABI → Runtime → SDK → Packages → Native Apps

SECOND:

Windows Compatibility → Translation → Per-App Environment → Windows Apps

That order preserves the identity of Iltric as a Self OS and prevents the compatibility subsystem from becoming the hidden foundation of the ecosystem.
