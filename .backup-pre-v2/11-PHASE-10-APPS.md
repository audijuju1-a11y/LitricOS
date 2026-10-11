# ILTRIC — PHASE 10 PROMPT: APPLICATION PLATFORM, PACKAGES & SDK

**Prerequisite:** GATE 09 passed.
**Deliverable:** apps can be installed, sandboxed, permissioned, updated, rolled back, and built by third parties.

---

## OBJECTIVE
An app ecosystem where installation is not a trust decision, because the sandbox makes it safe by default.

---

## TASKS

1. **App model** — `/apps/`
   - **Flatpak + XDG portals** as the sandboxed app platform, bound to the ILTRIC capability model from Phase 00/07 (bubblewrap sandbox, portal-mediated access, OSTree dedup). Document the disk overhead of runtimes honestly.
   - Rejected: Snap (vendor-centric backend), AppImage (no sandbox, no update story) — may still be *runnable*, but not the platform.
   - A minimal native package layer for the base image only (system components, not user apps).

2. **Manifest format** — `/sdk/manifest/`
   ```
   id, version, signature
   capabilities: [ fs.read.home, net.client, gpu, notify, ... ]
   resource-class: light | standard | game | ai
   latency-class:  normal | interactive | realtime-audio
   ```
   The installer renders capabilities in plain language. Any capability not declared is unavailable at runtime — no exceptions, no escape hatch.

3. **Supply chain** — `/apps/trust/`
   - **cosign / Sigstore** signatures on every artifact
   - **TUF** metadata for the repository (defends rollback, freeze, and mix-and-match attacks even if the server is compromised)
   - reproducible-build verification where the upstream supports it
   - per-app version pinning and rollback (Btrfs snapshot or OSTree deployment)

4. **Compatibility layers** — `/apps/compat/`
   - Linux apps: native + Flatpak
   - Windows apps/games: Wine/Proton stack (owned by Phase 12)
   - Web apps: an installable PWA path with the same capability model
   - **Clean-room / documented APIs only.** No proprietary implementation details copied.

5. **SDK** — `/sdk/`
   Stable APIs so a developer never needs kernel internals: windows, graphics, filesystem, networking, audio, notifications, permissions, identity, storage, AI, device access. Provide bindings for at least: C, Rust, and one scripting language. Ship a "hello ILTRIC" template that builds, installs, runs sandboxed, and requests one capability — in under 5 minutes.

6. **App lifecycle** — install / update / rollback / uninstall must all be atomic and leave no orphaned state. Uninstall removes app data unless the user explicitly keeps it.

7. **Store/manager UI** — permissions are shown *before* install, editable *after* install, and revocable *while running*.

---

## EXIT GATE — GATE 10
- [ ] `hello-iltric` builds from the SDK template and installs on a clean image in < 5 min from scratch.
- [ ] An unsigned or tampered package is refused, logged, and leaves no partial state.
- [ ] TUF metadata check demonstrably blocks a simulated rollback attack from a hostile mirror.
- [ ] App declaring only `notify` cannot read `/home` or reach the network — demonstrated.
- [ ] Install → update → rollback → uninstall cycle leaves the filesystem byte-clean (diff before/after).
- [ ] Runtime capability revocation works on a running app (Gate 07 regression).
- [ ] Idle RAM budget still met with 3 apps installed but not running.

## FAILURE MODES
- Flatpak runtime bloat → measure and publish the real disk cost; dedup is not an excuse to stop measuring.
- Portal UX friction → the fix is better portal dialogs, never widening default permissions.

## NEXT STEP
`12-PHASE-11-HARDWARE.md`
