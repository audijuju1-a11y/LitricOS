# ILTRIC — PHASE 08 PROMPT: ATOMIC A/B UPDATE & AUTO-ROLLBACK

**Prerequisite:** GATE 07 passed.
**Deliverable:** an update that can be power-cut at any instant and still leave a bootable machine.

---

## OBJECTIVE
`download → verify signature → write inactive slot → verify integrity → boot new slot → health checks → commit`, with automatic rollback on failure.

---

## TASKS

1. **Mechanism** — `systemd-sysupdate` for full-image A/B slots.
   - Pros: byte-identical slots, instant rollback, trivially matches the erofs+verity design.
   - Known limitation to document: **no delta updates as of systemd 260** — every update ships a full compressed image. State the bandwidth cost honestly.
   - Alternative to evaluate later for delta efficiency: OSTree / `bootc` (file-level, content-addressed, used by Fedora Atomic and Bazzite). Write the ADR; do not switch without a bandwidth measurement that justifies it.

2. **Transfer definitions** — `/update/sysupdate.d/`
   Separate transfers for: system slot image, verity hash, UKI, and (separately versioned) firmware. Each with its own signature.

3. **Signing & anti-downgrade** — `/update/sign/`
   - sign the image + verity root hash + UKI with the ILTRIC release key
   - **cosign/Sigstore** signatures on artifacts; **TUF** metadata for the update repository (protects against rollback/freeze/mix-and-match attacks by a compromised server)
   - monotonic version counter; refuse to install a version older than the committed one unless the user explicitly enters recovery

4. **Boot counting + auto-rollback** — `systemd-boot` boot-count.
   - new slot entry is written with N tries (`+3`)
   - each boot decrements; a boot that reaches `boot-complete.target` marks it good
   - N failures → bootloader automatically selects the previous known-good slot
   Define `boot-complete.target` precisely: compositor up, network manager up, defender up, no failed units.

5. **Health checks** — `/update/health/`
   Post-boot checks that must pass before commit:
   - all critical units active
   - compositor produced at least one frame
   - storage mounts verified, verity intact
   - defender responsive
   - boot time within 150% of the previous known-good (regression guard)
   Any failure → do not commit → rollback on next boot.

6. **Pre-update snapshot** — Btrfs snapshot of `@var` and `@home` before commit, with a user-visible "undo update" for 7 days.

7. **User experience** — updates never interrupt a running game or a full-screen app. Stage in the background, apply on next boot, always show what changed and what it costs (MB).

---

## EXIT GATE — GATE 08
- [ ] **Power-cut test:** kill the VM at 10 random points during an update (download, write, verify, first boot). Every single time the machine boots into a working system. Run it 10×, record results.
- [ ] Tampered update artifact → refused at signature verification, logged, no partition written.
- [ ] Downgrade attempt → refused.
- [ ] Deliberately broken update (e.g. remove the compositor) → boots 3× and auto-rolls back without user action. Demonstrated.
- [ ] **Update apply + commit ≤ 60 s** (excluding download); **rollback = one reboot**. Record both times.
- [ ] Disk overhead of A/B documented and within the partition plan.
- [ ] Regression: Gates 01–07 all still pass on the updated image.

## FAILURE MODES & ROLLBACK
- Both slots bad → recovery environment (Phase 03) is the last line; it must be able to re-flash a slot from removable media.
- ESP full → updater must check free space before staging and fail early with a clear message.
- Clock skew breaks signature validity → use monotonic counters, not wall-clock, for anti-downgrade.

## NEXT STEP
`10-PHASE-09-AI.md`
