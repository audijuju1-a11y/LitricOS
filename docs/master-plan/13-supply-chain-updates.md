# 13 — Supply chain and updates

## Trust chain

`source revision → hermetic/reproducible build → SBOM/provenance → signed image
and metadata → verified download → staged A/B target → health-confirmed boot`.

Each link records the build inputs, toolchain, dependency lock state, artifact
hash, signer identity, timestamp and verification result. Development keys must
never be promoted to release keys.

## Required controls

- Pin and review dependencies for privileged Rust/platform components; preserve
  the existing zero-third-party-dependency rule for core crates.
- Generate SPDX or CycloneDX SBOM per release; scan advisories; track exceptions
  with expiry/owner; retain source/build provenance.
- Rebuild release candidates independently and compare artifacts or explain
  intentional nondeterminism.
- Use threshold signing and an offline-root/key-rotation procedure for release
  metadata. TUF roles/expiry protect metadata compromise and rollback/freeze
  attacks; A/B slots protect interrupted deployment.
- Verify signatures, freshness, version floor, hashes and available disk space
  before activation. Do not disable verification outside test images.

## Update lifecycle

`discover → select → download → verify → stage → reboot into candidate → health
attest → commit` with explicit `cancel`, `fail`, `rollback`, and `quarantine`
states. Apps and OS update independently; an app update cannot mutate the base
OS. User data migrations are idempotent, versioned and have pre-migration
snapshots.

## Test and response

Test expired/revoked keys, compromised mirror, malformed metadata, replay,
rollback, freeze, wrong target, download interruption, power loss, full disk,
bad candidate boot and failed migration. Rehearse signing-key compromise,
emergency revocation and critical-update rollout before v1.

