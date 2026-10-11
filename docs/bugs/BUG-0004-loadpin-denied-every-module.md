# BUG-0004 — LoadPin denied every kernel module after switch-root

```
Bug ID        : BUG-0004
Severity      : S1 (CRITICAL) — a security control silently disabled most of the
                running system, including the ESP, BBR and tun
Class         : SEC (PROTOCOL §8)
Subsystem     : kernel/config, image/
First Observed: 2026-08-28, first boot with a working in-guest self test
Introduced by : the fix for BUG-0003
```

## Reproducer

Boot the `nano` image with `loadpin` in the LSM list and read the serial console.

## Expected

LoadPin pins module loading to the dm-verity-protected system partition, so only
modules from the signed image can load.

## Actual

```
LoadPin: ready to pin (currently enforcing)
LoadPin: mnt_sb lacks block device, treating as: writable
LoadPin: kernel-module-compressed pinned obj=".../vsock.ko.zst" pid=1 cmdline="/init"
LoadPin: kernel-module-compressed denied obj=".../vfat.ko.zst"
LoadPin: kernel-module-compressed denied obj=".../tcp_bbr.ko.zst"
LoadPin: kernel-module-compressed denied obj=".../tun.ko.zst"
LoadPin: kernel-module-compressed denied obj=".../pcspkr.ko.zst"
```

**LoadPin pinned to the initrd's tmpfs, not to the verity root.** It pins to the
*first* filesystem a module is loaded from, and the first module (`vsock`) is
loaded by `/init` inside the initrd. After switch-root every module lives on a
different filesystem — the erofs root — and every load is denied.

## Impact

Everything that needed a module after switch-root silently stopped working:

| Module denied | What broke |
|---|---|
| `vfat` | `mount: /efi: unknown filesystem type 'vfat'` → `efi.mount` failed → `dbus-broker` could not set up its mount namespace → **the system bus died**, taking `systemd-logind` and `systemd-homed` with it |
| `tcp_bbr` | BBR congestion control, which `20-litric-net.conf` explicitly configures, was never available. The sysctl silently kept cubic. |
| `tun` | No VPN, no container networking |
| others | 6449 modules shipped; anything not loaded inside the initrd was unreachable |

The visible symptom was `unknown filesystem type 'vfat'` — which points at
storage, filesystems, or a missing package. Nothing about it suggests an LSM.

## Root Cause

**A wrong premise in the fix for BUG-0003.** That fix enabled loadpin with this
reasoning, recorded in the kernel config:

> "loadpin: kernel modules and firmware may only be loaded from the filesystem
> the first one came from. On LitricOS that filesystem is the
> dm-verity-protected read-only system partition, which makes this unusually
> strong here."

The second sentence is false. On a system that boots through an initrd, the
first filesystem is the initrd's tmpfs. The comment described the behaviour we
wanted rather than the behaviour LoadPin has.

## Why Existing Tests Missed It

`ci/lint-kconfig.py` verifies that every name in `CONFIG_LSM` has its symbol
enabled — and loadpin did. The linter checks that an LSM *exists*, which is what
BUG-0003 was about. It cannot check what an LSM *does* at runtime.

This class of defect is only findable by booting. It was found within minutes of
the in-guest self test existing, and not before, across many boots.

## Fix

`loadpin` removed from `CONFIG_LSM` and from the `lsm=` kernel command line, with
the reasoning recorded inline in `kernel/config/litric-base.config` so the same
mistake is not made again from the same false premise.

`CONFIG_SECURITY_LOADPIN` remains enabled so the option is available; only its
activation is removed.

**The security property is not lost.** dm-verity already guarantees that every
module on the system partition hashes back to a root hash signed with our key —
which is the property loadpin was intended to add, obtained by a mechanism that
actually works here. `CONFIG_MODULE_SIG_FORCE` is the orthogonal and correct
mechanism for module-signature enforcement and is a Phase 07 item requiring our
own kernel build.

## Regression Test

`litric-selftest` reports `litric.units.failed=`, which is now `0`. Any future
change that breaks module loading will show up as failed units on the next boot
rather than as a mysterious filesystem error.

A stronger check is warranted and is tracked: the self test should assert that a
representative module can be loaded *after* switch-root, so a module-loading
policy change fails loudly instead of degrading the system quietly.

## Verification

- `litric.units.failed=0` (was 8)
- `LoadPin ... denied` occurrences in the boot log: **0** (was 4+)
- `litric.lsm.active=capability,landlock,lockdown,yama,safesetid,bpf`
- `litric.verity.active=yes`, `litric.root.readonly=yes`
- Boots to a login prompt: yes

## Residual Risk

`integrity` appears in `CONFIG_LSM` and in `lsm=` but does **not** appear in
`/sys/kernel/security/lsm`. `CONFIG_INTEGRITY` provides a substrate rather than
registering an LSM under that name in this kernel. It is inert rather than
harmful, but it is a name in a list that does not correspond to a running
module — the exact condition BUG-0003 was about — and should be either removed or
replaced with a real IMA/EVM configuration once we build our own kernel.
