# BUG-0021 — The image booted by firmware landed in emergency mode

```
Bug ID        : BUG-0021
Severity      : S1 (CRITICAL) — the shipped disk image does not boot on real
                hardware or in any VM that boots through its own bootloader
Class         : BOOT / IMAGE CONFIGURATION
Subsystem     : image/mkosi.conf (KernelCommandLine=), the UKI's .cmdline
                section, systemd-fstab-generator in the initrd
First Observed: 2026-10-17, the first time this image was booted through UEFI
                with its own `EFI/Linux/*.efi` UKI instead of with
                `qemu -kernel vmlinuz -initrd initrd.img -append "..."`
Related       : BUG-0020 (the initrd shipped with no kernel modules) — same
                class: a boot input that every automated run supplied by hand,
                and the image therefore never carried
Status        : **FIXED 2026-10-17** — `rootfstype=erofs` is now part of
                KernelCommandLine=; verified by booting the same disk through
                UEFI + the UKI and reaching `litric-shell`
```

## Symptom

Booting the built disk with firmware:

```
[  OK  ] Reached target Local Verity Protected Volumes.
         Mounting /sysroot...
[    6.994406] EXT4-fs (dm-0): VFS: Can't find ext4 filesystem
[    6.999764] EXT4-fs (dm-0): VFS: Can't find ext4 filesystem
[    7.020771] EXT4-fs (dm-0): VFS: Can't find ext4 filesystem
[FAILED] Failed to mount /sysroot.
[DEPEND] Dependency failed for Mountpoints Configured in the Real Root.
[  OK  ] Started Emergency Shell.
You are in emergency mode.
```

dm-verity came up cleanly — the root hash matched, `/dev/mapper/root` existed —
and then the mount failed because it was attempted as ext4.

## Root cause

The root filesystem is erofs. The image's command line did not say so:

```
roothash=c834b53c... loglevel=7 video=Virtual-1:1920x1080 console=tty0 \
console=ttyS0,115200 ro systemd.machine_id=firstboot lsm=... 
```

The initrd's `systemd-fstab-generator` reads `roothash=` and builds a
`/sysroot` mount for `/dev/mapper/root`, and with no `rootfstype=` it falls back
to its default, ext4. Nothing else in the image pins the type: mkosi writes the
root into the UKI as a verity device, and the generated fstab names the *device*,
not the filesystem.

## Why every earlier boot missed it

Every automated boot in the repository launched the kernel directly:

```bash
qemu -kernel build/out/vmlinuz -initrd build/out/initrd.cpio.zst \
     -append "roothash=... rootfstype=erofs ..."
```

`rootfstype=erofs` was typed on the command line by the harness, so the image
never had to carry it. `build/boot-direct-qmp.sh`, `tests/smoke_boot.py` and
every evidence log in `evidence/` boot that way. The firmware path — the one a
real machine takes — had never been executed.

That is the pattern worth remembering: a build input supplied by the test
harness is not verified as part of the artifact. The artifact has to boot with
nothing appended.

## Fix

`image/mkosi.conf`:

```
        ro
        rootfstype=erofs
```

with the reason written next to it, because `ro` and `rootfstype` look like
duplicates of what mkosi already knows and will be "tidied away" otherwise.

## Verification

Booted the same disk image through OVMF, letting systemd-boot load the UKI, with
no `-kernel`, no `-initrd` and no `-append`:

```
[   46.196783] litric-shell[487]: litric.shell.locale.measurement=metric
[   46.196991] litric-shell[487]: litric.shell.locale.keyboard=us
[   46.197560] litric-shell[487]: litric.shell.rows_presented=12960
[   46.198683] litric-shell[487]: litric.shell.frames=13
[   46.199984] litric-shell[487]: litric.shell.frames_over_budget=0
```

The desktop is up 46 s after power-on from a cold UEFI start, and input was
verified through the emulated USB tablet and keyboard.

Before the rebuild was available, the fix was proven on the existing disk by
adding a systemd-boot entry whose `options` carried the UKI's own command line
plus `rootfstype=erofs` — note that an entry pointing `linux=` at a UKI does
*not* inherit the UKI's `.cmdline`, which is why the first attempt at this
(passing only `rootfstype=erofs`) failed with `verity: Root hash verification
failed (-ENOKEY)`.
