# 04 — Boot integrity and recovery

## Boot state machine

```text
firmware → Secure Boot verification → signed UKI → initramfs
→ verify root image → unlock mutable data → health checks → selected slot
→ mark healthy / decrement boot attempt / rollback
```

No stage may silently continue after an integrity failure. Recovery is a separate
signed boot entry with read-only access to system slots and explicit, audited
access to encrypted user data after successful authentication.

## Required implementation milestones

1. Generate development keys only for development; production key ownership,
   offline storage, signing ceremony and rotation are separate release gates.
2. Enrol Secure Boot keys through a documented firmware/Mok flow and test an
   unsigned/tampered UKI rejection on each HCL class.
3. Bind dm-verity root hash to the signed boot artifact; test altered root blocks
   fail closed before normal service startup.
4. Add TPM2 measured-boot evidence and PCR-bound unlock as an optional layer;
   recovery key/passphrase remains mandatory fallback.
5. Implement boot counting and health confirmation only after the graphical
   session, storage mount, and critical service checks succeed.

## Recovery functions

- Select prior OS slot, collect signed diagnostics, verify image hashes, repair
  boot metadata, decrypt/export user data, inspect snapshots, and start a
  restricted network support session only with user confirmation.
- Do not offer "disable verification and continue" as an ordinary recovery path.
- Recovery operations are transactional and show irreversible actions before
  execution.

## Fault tests

Test power loss before/after each update commit, full ESP, corrupt verity block,
wrong signing key, revoked key, failed data unlock, malformed boot count, failed
health check, boot loop, and rollback after kernel regression. Preserve evidence
including slot, generation, reason, and user-data integrity result.

