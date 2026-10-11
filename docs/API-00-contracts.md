# LITRIC — Interface Contracts (Signatures Only)
**Document:** API-00-contracts.md | **Phase:** 00 | **Status:** DESIGN-ONLY
**Rule:** An app must not reach a kernel API except through one of these interfaces.

---

## 1. litric.caps — Capability Management

```
// Grant a capability to an app
fn caps_grant(app_id: AppId, cap: Capability, scope: Scope) -> Result<(), CapsError>

// Revoke a capability from an app (effective immediately)
fn caps_revoke(app_id: AppId, cap: Capability) -> Result<(), CapsError>

// Query whether an app holds a capability
fn caps_query(app_id: AppId, cap: Capability) -> CapStatus  // GRANTED | DENIED | REVOKED

// List all capabilities for an app
fn caps_list(app_id: AppId) -> Vec<CapabilityEntry>

// Audit log: last N capability events for an app
fn caps_audit(app_id: AppId, limit: u32) -> Vec<CapsAuditEvent>
```

**Trust boundary:** Core service (L1) → App (L3-4)
**Capability:** `system.settings` (admin only)

---

## 2. litric.fs — Permissioned Filesystem Access

```
// Open a file through the storage broker (never raw path)
fn fs_open(namespace: Namespace, path: &str, mode: OpenMode) -> Result<FileHandle, FsError>

// Read from an opened handle
fn fs_read(handle: FileHandle, buf: &mut [u8]) -> Result<usize, FsError>

// Write to an opened handle
fn fs_write(handle: FileHandle, buf: &[u8]) -> Result<usize, FsError>

// List directory contents (only within capability scope)
fn fs_list(namespace: Namespace, path: &str) -> Result<Vec<DirEntry>, FsError>

// Close handle, release resources
fn fs_close(handle: FileHandle) -> Result<(), FsError>

// Namespaces: user:// app:// cache:// config:// secure:// temp://
```

**Trust boundary:** Core service (L1) → App (L3-4)
**Capability:** `fs.read.*`, `fs.write.*` (scope-limited per capability)

---

## 3. litric.net — Permissioned Network Access

```
// Create a TCP connection (outbound only)
fn net_connect(host: &str, port: u16, timeout_ms: u32) -> Result<NetHandle, NetError>

// Create a listening socket (server mode)
fn net_listen(addr: &str, port: u16) -> Result<NetHandle, NetError>

// Send data
fn net_send(handle: NetHandle, data: &[u8]) -> Result<usize, NetError>

// Receive data
fn net_recv(handle: NetHandle, buf: &mut [u8], timeout_ms: u32) -> Result<usize, NetError>

// Resolve DNS
fn net_resolve(host: &str) -> Result<Vec<IpAddr>, NetError>

// Close connection
fn net_close(handle: NetHandle) -> Result<(), NetError>
```

**Trust boundary:** Core service (L1) → App (L3-4)
**Capability:** `net.client` (outbound), `net.server` (listen)
**Default:** NO network for sandboxed apps

---

## 4. litric.gfx — Surface/Window/Output

```
// Create a window/surface
fn gfx_create_surface(width: u32, height: u32, title: &str) -> Result<SurfaceHandle, GfxError>

// Present a rendered frame
fn gfx_present(surface: SurfaceHandle, buffer: BufferHandle) -> Result<(), GfxError>

// Set window properties
fn gfx_set_title(surface: SurfaceHandle, title: &str) -> Result<(), GfxError>
fn gfx_set_bounds(surface: SurfaceHandle, x: i32, y: i32, w: u32, h: u32) -> Result<(), GfxError>
fn gfx_set_state(surface: SurfaceHandle, state: WindowState) -> Result<(), GfxError>  // Normal/Maximized/Minimized

// Handle input events (returned via poll)
fn gfx_poll_events(surface: SurfaceHandle) -> Option<InputEvent>

// Destroy surface
fn gfx_destroy(surface: SurfaceHandle) -> Result<(), GfxError>
```

**Trust boundary:** Compositor (L2) → App (L3-4)
**Capability:** `gpu.graphics` (window), `gpu.compute` (compute only)
**Composition model:** App renders → compositor consumes → compositor composites → display

---

## 5. litric.audio — Audio Stream

```
// Create an audio output stream
fn audio_output_stream(config: AudioConfig) -> Result<AudioHandle, AudioError>

// Create an audio input stream (microphone)
fn audio_input_stream(config: AudioConfig) -> Result<AudioHandle, AudioError>

// Write audio data (real-time safe, Tier A)
fn audio_write(handle: AudioHandle, samples: &[f32]) -> Result<(), AudioError>

// Read audio data (real-time safe, Tier A)
fn audio_read(handle: AudioHandle, samples: &mut [f32]) -> Result<usize, AudioError>

// Set volume (per-stream)
fn audio_set_volume(handle: AudioHandle, volume: f32) -> Result<(), AudioError>

// Set latency class
fn audio_set_latency_class(handle: AudioHandle, cls: LatencyClass) -> Result<(), AudioError>
// LatencyClass: Normal | Low | UltraLow

// Close stream
fn audio_close(handle: AudioHandle) -> Result<(), AudioError>
```

**Trust boundary:** Audio service (L2) → App (L3-4)
**Capability:** `audio.output`, `audio.input`
**Real-time rule:** Tier A callbacks: no blocking, no alloc, no I/O

---

## 6. litric.ai — AI Inference & Routing

```
// Run inference
fn ai_infer(model_id: ModelId, prompt: &str, opts: InferOptions) -> Result<InferResult, AiError>

// Route hint: prefer CPU/GPU/NPU/cloud
fn ai_route_hint(hint: RouteHint) -> Result<(), AiError>

// Get consent scope for cloud inference
fn ai_consent_status() -> ConsentScope  // Off | LocalOnly | LocalAndCloud

// Get audit log for AI usage
fn ai_audit(limit: u32) -> Vec<AiAuditEvent>

// Unload model from memory (free RAM)
fn ai_unload(model_id: ModelId) -> Result<(), AiError>
```

**Trust boundary:** AI service (L1) → App (L3-4)
**Capability:** `ai.local` (on-device), `ai.cloud` (remote, off by default)
**Privacy rule:** No silent screen capture. No data exfil without explicit consent.

---

## 7. litric.update — System Update Management

```
// Check for available updates
fn update_check() -> Result<Vec<UpdateInfo>, UpdateError>

// Stage an update (download + verify)
fn update_stage(update_id: UpdateId) -> Result<(), UpdateError>

// Commit the staged update (applies on next boot)
fn update_commit() -> Result<(), UpdateError>

// Rollback to previous slot
fn update_rollback() -> Result<(), UpdateError>

// Get current update status
fn update_status() -> UpdateStatus  // Idle | Downloading | Staged | Committing | Failed
```

**Trust boundary:** Update service (L1) → System
**Capability:** `system.settings` (admin only)
**Security:** cosign + TUF + monotonic version counter

---

## 8. litric.defender — Security Scanning

```
// Scan a file or directory
fn defender_scan(path: &str, opts: ScanOptions) -> Result<ScanResult, DefenderError>

// Get verdict for a file (cached or new scan)
fn defender_verdict(path: &str) -> Verdict  // Clean | Suspicious | Malicious | Unknown

// Quarantine a file (move to encrypted quarantine)
fn defender_quarantine(path: &str) -> Result<QuarantineId, DefenderError>

// Restore from quarantine
fn defender_restore(quarantine_id: QuarantineId) -> Result<(), DefenderError>

// Get defender status
fn defender_status() -> DefenderStatus  // Active | Scanning | Paused | Error
```

**Trust boundary:** Defender service (L1) → App (L3-4)
**Capability:** `system.settings` (admin only for manual scans)
**Honesty:** Advisory only; will not match commercial AV coverage.

---

## 9. litric.power — Power Profile Management

```
// Get current power profile
fn power_get_profile() -> PowerProfile  // PowerSaver | Balanced | Performance | Custom

// Set power profile
fn power_set_profile(profile: PowerProfile) -> Result<(), PowerError>

// Get battery status (laptop only)
fn power_battery() -> Option<BatteryInfo>  // {level, charging, time_remaining}

// Express application intent
fn power_set_intent(intent: AppIntent) -> Result<(), PowerError>
// AppIntent: LatencySensitive | Interactive | Background | Batch
```

**Trust boundary:** Power service (L2) → App (L3-4)
**Capability:** `system.settings` (admin only for expert knobs)
