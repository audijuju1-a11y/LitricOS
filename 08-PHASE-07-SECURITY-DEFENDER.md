# LITRIC — PHASE 07 PROMPT: SECURITY, SANDBOX & LITRIC DEFENDER

**Prerequisite:** GATE 06 passed.
**Deliverable:** capability sandbox enforced for real + a lightweight, always-on defender that does NOT eat the RAM budget.

> **Design constraint that drives everything here:** a resident ClamAV daemon costs ~500 MB–1 GB RAM because it loads the whole signature DB. That is incompatible with a ≤600 MB idle target. **ClamAV is on-demand only** (start → scan → stop). Always-on protection comes from fanotify + YARA + behavior, not from a big signature daemon.

---

## OBJECTIVE
Make "an app cannot do what it was not granted" a property of the system, and add malware detection that is honest about its limits.

---

## PART A — SANDBOX & CAPABILITIES

1. **Default sandbox for every app** — `/security/sandbox/`
   Layers, all of them:
   - **Landlock** — unprivileged filesystem/network restriction LSM; primary per-app confinement, low overhead
   - **seccomp-bpf** — syscall filter per app profile
   - **bubblewrap** — namespace isolation (mount/pid/net/user)
   - **cgroups v2** — memory/CPU/IO limits per app
   - systemd sandboxing directives for system services (from Phase 02)
   Rejected as default: SELinux (powerful but heavy/complex — provide as an optional "high security mode" policy); AppArmor as a middle option.

2. **Capability enforcement** — wire `litric.caps` (Phase 00) to real checks:
   - manifest declares capabilities; installer shows them; user grants
   - runtime revocation must actually take effect (test it)
   - every grant/deny/revoke is audit-logged with app id, capability, timestamp, and result
   - **no ambient authority**: an app with no capabilities can read nothing but its own sandbox

3. **Portals** — route camera, microphone, screen capture, file picker, location through XDG portals so the *user's action* is the grant, not a checkbox.

4. **Boot & runtime integrity**
   - Secure Boot with LITRIC keys (dev keys in VM; document the enrollment flow via MokManager / KEK+db)
   - signed UKI, dm-verity rootfs (Phase 03), `fs-verity` on app content
   - TPM 2.0 measured boot; PCR policy documented
   - IMA/EVM evaluated — enable if the measured overhead is acceptable, record the number either way

---

## PART B — LITRIC DEFENDER

5. **On-access engine** — `/defender/onaccess/`
   - `fanotify` with `FAN_OPEN_PERM` / `FAN_ACCESS_PERM` (needs `CONFIG_FANOTIFY_ACCESS_PERMISSIONS` + `CAP_SYS_ADMIN`); use `FAN_PRE_ACCESS` pre-content hooks where the kernel supports them
   - flow: intercept open → cache lookup (hash + fs-verity state) → verdict → `FAN_ALLOW` / `FAN_DENY`
   - **cache aggressively**: an unchanged, already-scanned, verity-backed file must never be re-scanned. This is the whole performance story.
   - hard timeout: if the scanner does not answer in N ms, fail **open** with a logged warning (a hung scanner must never freeze the desktop) — and treat repeated timeouts as a P1 bug

6. **Detection layers** — `/defender/engine/`
   - hash denylist (fast, exact)
   - **YARA** rules engine (BSD-3, lightweight) for heuristics/families
   - optional static ELF/PE classifier — if attempted, train on an open dataset (e.g. EMBER) and report real precision/recall, not vibes
   - **ClamAV on-demand only**: scheduled/manual full scans spawn `clamd`, scan, then stop it

7. **Behavioral layer (eBPF)** — `/defender/behavior/`
   BPF-LSM hooks for exec, privilege change, unusual file mass-rename (ransomware signal), unexpected outbound connections. Concepts modeled on Falco / Tetragon / Tracee. Scoring, not instant killing — surface to the user with evidence.

8. **Quarantine & response** — move to an encrypted quarantine with restore. Never delete silently. Every action reversible and logged.

9. **Honesty requirements — put these in the UI and the docs**
   - "LITRIC Defender is sandbox-first. Detection is advisory and will not match commercial AV coverage."
   - false-positive reporting path exists
   - cloud lookup, if ever added, is **off by default** and explicitly consented

---

## EXIT GATE — GATE 07
- [ ] **Defender resident RAM ≤ 100 MB** (`defender.slice` `MemoryMax=100M` holds under a full-tree scan).
- [ ] On-access latency penalty on **cached** files ≤ single-digit ms — measured with `fio`/`hyperfine` on a real directory tree, before/after.
- [ ] EICAR test file detected and blocked on open.
- [ ] A YARA test-corpus run reports real detection counts (not "it works").
- [ ] Scanner hang injection → desktop stays responsive, fail-open path logged.
- [ ] App with no capabilities cannot read `/home`, open the camera, or reach the network. Demonstrated with a purpose-built test app.
- [ ] Runtime revocation of `microphone` takes effect on a *running* app.
- [ ] Tampered rootfs still fails closed (Gate 03 regression).
- [ ] Idle RAM total still ≤ 600 MB **with defender running**.

## FAILURE MODES & ROLLBACK
- False positive blocks a system binary → allowlist by verity hash + immediate restore from quarantine; ship a "Defender safe mode" boot option that disables on-access entirely.
- fanotify permission events deadlock (scanner reads the file it is gating) → scanner must use `FAN_MARK` exclusion for its own I/O; test explicitly.
- BPF program rejected on kernel upgrade → defender degrades to hash+YARA and warns; it must never block boot.

## NEXT STEP
`09-PHASE-08-UPDATE.md`
