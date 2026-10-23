# LITRIC — Capability Vocabulary
**Document:** SEC-01-capabilities.md | **Phase:** 00 | **Status:** DESIGN-ONLY

---

## 1. Capability Specification

Every capability string, its default state, who grants, revocability, and audit status.

| Capability | Default | Grant Authority | Revocable at Runtime | Audit-Logged |
|------------|---------|-----------------|----------------------|--------------|
| `fs.read.home` | DENIED | User consent | YES | YES |
| `fs.write.home` | DENIED | User consent | YES | YES |
| `fs.read.all` | DENIED | User consent | YES | YES |
| `fs.write.all` | DENIED | User consent | YES | YES |
| `camera` | DENIED | User consent | YES | YES |
| `microphone` | DENIED | User consent | YES | YES |
| `net.client` | DENIED | User consent | YES | YES |
| `net.server` | DENIED | User consent | YES | YES |
| `net.local` | DENIED | User consent | YES | YES |
| `gpu` | GRANTED | Manifest | YES | YES |
| `notify` | GRANTED | Manifest | YES | YES |
| `location` | DENIED | User consent | YES | YES |
| `input.global` | DENIED | User consent | YES | YES |
| `ai.local` | DENIED | User consent | YES | YES |
| `ai.cloud` | DENIED | User + system consent | YES | YES |
| `device.usb` | DENIED | User consent | YES | YES |
| `autostart` | DENIED | User consent | YES | YES |
| `bluetooth` | DENIED | User consent | YES | YES |
| `clipboard.read` | DENIED | User consent | YES | YES |
| `clipboard.write` | DENIED | User consent | YES | YES |
| `secure_storage` | DENIED | User consent | YES | YES |
| `system.settings` | DENIED | Admin only | YES | YES |
| `service.background` | DENIED | User consent | YES | YES |
| `net.configure` | DENIED | System (policy grant to the shell) | YES | YES |
| `audio.set` | DENIED | System (policy grant to the shell) | YES | YES |
| `power.control` | DENIED | System + live consent for `power://system/*` | YES | YES |
| `auth.session` | DENIED | System + live consent | YES | YES |
| `device.bluetooth.admin` | DENIED | System (policy grant to the shell) | YES | YES |

**The five rows above were added on 2026-10-12**, when the wave-1 provider actions
became real (`src/litric-caps/src/capability.rs`, enforced by the broker the shell
receives them from in `src/litric-sysd/src/state.rs`). They were in the code and
not in this table, which is the kind of drift this document exists to prevent.
Their scopes are the resource schemes in `Scope::parse_resource`:
`iface://<name>`, `wifi://<ssid>` (both `net.configure`), `audio://default-sink`,
`bt://` and `bt://<bdaddr>`, `lock://<user>`, and `power://system/<verb>`. A
scheme with no path covers every resource of its kind — `wifi://` is every Wi-Fi
network — and a scope this document does not list is not one a grant can be
narrowed to.

## 2. Capability Classes

| Class | Capabilities | Risk Level |
|-------|-------------|------------|
| Storage | `fs.read.home`, `fs.write.home`, `fs.read.all`, `fs.write.all` | MEDIUM-HIGH |
| Media | `camera`, `microphone`, `audio.output`, `audio.input` | HIGH |
| Network | `net.client`, `net.server`, `net.local` | MEDIUM |
| Graphics | `gpu` | LOW |
| AI | `ai.local`, `ai.cloud` | MEDIUM-HIGH |
| Device | `device.usb`, `bluetooth` | MEDIUM |
| System | `system.settings`, `autostart`, `service.background` | HIGH |
| UI | `notify`, `clipboard.read`, `clipboard.write`, `input.global` | LOW-MEDIUM |
| Security | `secure_storage` | MEDIUM |

## 3. Rules

1. **Deny-by-default:** Any capability not in the manifest is unavailable at runtime.
2. **Scope-limited:** `fs.read.home` grants access only to `user://Documents`, not the full filesystem.
3. **Non-transferable:** An app cannot pass its capabilities to another app via IPC.
4. **Revocable at runtime:** `caps_revoke()` takes effect immediately; the app receives a signal.
5. **Audit-logged:** Every grant, deny, and revoke is logged with timestamp + app_id + capability.
6. **User-visible:** The installer shows requested capabilities in plain language before install.
7. **Editable post-install:** Users can modify capabilities in the App Manager at any time.
