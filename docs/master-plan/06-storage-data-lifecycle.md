# 06 — Storage and data lifecycle

## Layout

| Area | Format/policy | Mutability |
|---|---|---|
| ESP + UKI | signed boot artifacts | restricted |
| system slot A/B | EROFS + dm-verity | immutable |
| user and state | LUKS2-protected Btrfs | user/system managed |
| app data/cache | quota-controlled subvolumes | app scoped |
| snapshots | bounded Btrfs snapshots | recovery managed |

## Data invariants

- Operating-system replacement never deletes user data, app data, or recovery
  metadata without separate confirmed action.
- Package installation is staged and atomically activated; uninstall semantics
  distinguish code, cache, settings, and user-created data.
- Filesystem namespaces are stable public concepts (`user://`, `app://`,
  `cache://`, `config://`, `secure://`, `temp://`); physical paths are private.
- Restore cannot silently overwrite newer files. It produces a conflict plan and
  records the source snapshot, destination, and final decision.

## Functional work

1. Implement Btrfs subvolume policy, zstd, scrub, TRIM, SMART reporting and
   pre-update snapshots.
2. Add encrypted home/state provisioning with TPM-unlock opt-in and recovery
   passphrase flow.
3. Implement external-drive mount/eject with device identity, untrusted-media
   scan policy, executable handling and ownership boundaries.
4. Add storage pressure UI: category usage, quota, cleanup candidates, 85% warn,
   95% update block, and safe release of old snapshots only after retention tests.
5. Deliver backup/export/import and disaster-recovery documentation before v1.

## Failure tests

Exercise disk full at every write transaction, Btrfs I/O error, interrupted
snapshot, corrupted metadata, duplicate/missing UUID, device unplug, wrong
decryption key, quota exhaustion, restore collision, and power cut during update.
Verify no silent loss, no cross-app read, and an actionable recovery record.

