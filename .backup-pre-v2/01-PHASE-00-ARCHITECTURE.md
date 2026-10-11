# ILTRIC — PHASE 00 PROMPT: ARCHITECTURE & THREAT MODEL

**Prerequisite:** none. This is the first session.
**Deliverable type:** documents + interface definitions. **No code yet.**

---

## OBJECTIVE
Freeze the ILTRIC architecture so that every later phase has a stable contract to build against.

## WHY
Every later mistake is cheap here and expensive later. Interfaces defined now prevent the "rewrite everything in phase 6" failure that kills hobby OS projects.

---

## TASKS

1. **System boundary diagram** — produce `docs/ARCH-00-boundaries.md`
   - Draw the full stack: firmware → UEFI → bootloader → kernel → HAL → drivers → services → security services → graphics → compositor → shell → runtime → apps.
   - For each layer state: owner, language, upstream-or-ours, trust level (0=kernel .. 4=untrusted app).

2. **Threat model** — `docs/SEC-00-threatmodel.md`
   - Use STRIDE per trust boundary.
   - Adversaries to model: malicious app, malicious USB device, network attacker, malicious update server, physical attacker with disk access, compromised driver, malicious AI prompt/model.
   - For each: attack, current mitigation, phase in which the mitigation lands, residual risk.

3. **Interface contracts** — `docs/API-00-contracts.md`
   Define these NOW (signatures only, no implementation):
   - `iltric.caps`     — capability grant/revoke/query
   - `iltric.fs`       — permissioned filesystem access
   - `iltric.net`      — permissioned network access
   - `iltric.gfx`      — surface/window/output
   - `iltric.audio`    — stream create, latency class
   - `iltric.ai`       — infer(), route hint, consent scope
   - `iltric.update`   — status, stage, commit, rollback
   - `iltric.defender` — scan(path), verdict, quarantine
   - `iltric.power`    — profile get/set
   Rule: **an app must not be able to reach a kernel API except through one of these.**

4. **Capability vocabulary** — `docs/SEC-01-capabilities.md`
   Enumerate the exact capability strings apps may declare:
   `fs.read.home`, `fs.write.home`, `fs.read.all`, `camera`, `microphone`, `net.client`, `net.server`, `gpu`, `notify`, `location`, `input.global`, `ai.local`, `ai.cloud`, `device.usb`, `autostart`.
   For each: default state, who can grant, revocable at runtime yes/no, audit-logged yes/no.

5. **Repository layout** — `docs/REPO-00-layout.md`
   Propose and freeze the tree, e.g.
   ```
   /kernel      config fragments + patches
   /image       mkosi configs, profiles
   /core        init units, sysupdate defs
   /compositor  smithay-based
   /shell       ILTRIC desktop
   /defender    fanotify+yara daemon
   /ai          router + services
   /sdk         headers, bindings
   /tools       qemu launchers, bench harness
   /tests       smoke, regression, bench
   /docs
   ```

6. **Decision records** — create `docs/adr/0001-kernel-foundation.md` … `0008-*.md`
   One ADR per locked baseline decision in MASTER §3. Format: Context / Options / Decision / Consequences / Revisit-if.

7. **Risk register** — `docs/RISK-00.md`
   Must include, with honest severity:
   - kernel-anti-cheat games will never work (accept, document)
   - Windows-host QEMU cannot do virgl/venus GL acceleration → graphics work needs a Linux host or WSL2
   - virtiofs is Linux-host-only → use SSH/SMB or `-hda fat:rw:` for host sharing
   - WHPX SMP boot bug (QEMU #3178) → keep a TCG fallback launcher
   - AMD XDNA NPU stack immature → NPU = experimental only
   - solo-developer effort: ~18–36 person-months to daily-driver prototype

---

## ASSUMPTIONS TO STATE
- Target dev host: Windows 11 + QEMU + WHPX; secondary Linux host or WSL2 for GPU work.
- First-class arch: x86-64. ARM64 is a phase-11+ concern. RISC-V is aspirational.
- Team size: 1–3 people.

---

## EXIT GATE — GATE 00

Pass only when ALL are true:
- [ ] All 7 documents exist and are internally consistent (no contract references an undefined capability).
- [ ] Threat model covers every trust boundary in the boundary diagram — zero boundaries with no entry.
- [ ] Every capability in SEC-01 maps to at least one API in API-00, and vice versa.
- [ ] Every baseline decision in MASTER §3 has an ADR with a "Revisit-if" trigger.
- [ ] Risk register contains the six honest limitations listed above.

**Do not write a single line of kernel config until GATE 00 passes.**

---

## FAILURE MODES
- Architecture astronautics → mitigated by: nothing here may reference a component that isn't in the Phase Map.
- Over-specification → each doc has a hard cap of 3 pages; details belong to the owning phase.

## NEXT STEP
`02-PHASE-01-BOOT.md`
