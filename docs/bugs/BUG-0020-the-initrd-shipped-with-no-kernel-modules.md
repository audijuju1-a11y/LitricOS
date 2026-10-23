# BUG-0020 — The initrd shipped with no kernel modules, so the root never mounted

```
Bug ID        : BUG-0020
Severity      : S1 (CRITICAL) — the image does not boot: it reaches an emergency
                shell instead of a desktop, and nothing above the initrd runs
Class         : BUILD / IMAGE (kernel packaging)
Subsystem     : image/mkosi.conf (`KernelInitrdModules=`),
                image/mkosi.profiles/nano.conf (stale key name),
                surfaced by image/mkosi.repart/10-root.conf (Format=erofs)
First Observed: 2026-10-16, first QEMU boot of the desktop image rebuilt after the
                UI round (tools/build.sh --profile desktop, then QEMU with
                -device VGA -display none)
Status        : FIXED (config corrected; verification below)
```

## Reproducer

```bash
# on the Linux builder
cd ~/litricos && ./tools/build.sh --profile desktop
qemu-system-x86_64 -machine q35 -m 2048 \
  -drive if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
  -drive if=pflash,format=raw,unit=1,file=OVMF_VARS.fd \
  -drive if=virtio,format=raw,file=build/out/litricos.raw \
  -device VGA -display none -serial file:boot.log
tail -40 boot.log
```

## Actual

```
[    4.869260] EXT4-fs (dm-0): VFS: Can't find ext4 filesystem
[    4.871950] EXT4-fs (dm-0): VFS: Can't find ext4 filesystem
[    4.873795] EXT4-fs (dm-0): VFS: Can't find ext4 filesystem
[FAILED] Failed to mount /sysroot.
DEPEND] Dependency failed for Initrd Root File System.
[  OK  ] Started Emergency Shell.
You are in emergency mode.
```

`dm-0` exists, so dm-verity itself worked; the *filesystem* on it could not be
mounted, and the kernel reports the last driver it tried.

## Root Cause

Two config errors in the same setting, one after the other:

1. The setting was spelled `KernelModulesInitrdInclude=` (mkosi 26). mkosi 27
   renamed it `KernelInitrdModules=` and still parses the old name, so this one
   alone would not have been fatal.
2. The list was then **removed entirely** from `image/mkosi.conf`, with a comment
   saying the initrd's modules were "deliberately NOT restricted right now" —
   the assumption being that unrestricted means "every module". In mkosi 26 the
   default was every module; in mkosi 27 `KernelInitrdModules=` defaults to
   **empty**, and that empty list is not "no restriction", it is "no modules".

Measured on the built UKI, not inferred from the config:

```
$ objcopy -O binary --only-section=.initrd litricos.efi /tmp/initrd.bin
$ zstd -dc /tmp/initrd.bin > /tmp/initrd.raw          # 190 182 912 bytes
$ grep -c 'TRAILER' /tmp/initrd.raw                   # one cpio archive
$ cpio -t < /tmp/initrd.raw | wc -l                   # 10 083 entries
$ cpio -t < /tmp/initrd.raw | grep -cE '\.ko(\.zst|\.xz)?$'
0
$ python3 - <<'EOF'
d = open("/tmp/initrd.raw","rb").read()
print("kernel/fs/erofs:", d.find(b"fs/erofs"), " .ko.zst files:", d.count(b"usr/lib/modules/7.2.9-arch1-1"))
EOF
kernel/fs/erofs: -1   .ko.zst files: 0
```

Ten thousand entries, not one module. The system partition is erofs
(`image/mkosi.repart/10-root.conf`, `Format=erofs`) behind dm-verity, and the
kernel the image ships (Arch `linux` 7.2.9-arch1-1) builds **erofs as a module**
(`usr/lib/modules/7.2.9-arch1-1/kernel/fs/erofs/erofs.ko.zst`). A module that is
not in the initrd cannot be loaded before `/sysroot` exists, so the mount was
attempted without erofs registered and the kernel walked its built-in list —
ext4 last — which is what the "Can't find ext4 filesystem" lines are.

The failure therefore looks like a storage or dm-verity fault (the device is
there, the hash verifies, the mount fails) and is actually the initrd missing a
file. The boot log in `docs/evidence/gate01-boot.log` from 2026-08-29 mounts the
same erofs root successfully (`erofs (device dm-0): mounted with root inode`),
which is how the defect was localised to the initrd rather than to repart.

## Fix

`image/mkosi.conf` now names the modules the boot needs, in the mkosi 27
spelling, and says why each is there:

```
KernelInitrdModules=
        default        # mkosi-initrd's core set: block, USB HID, virtio, vfat
        erofs          # the root filesystem's own driver
        dm[-_]*        # device mapper: dm-verity is what the root sits behind
        btrfs          # /var
        ext4
        xfs
        squashfs
```

`image/mkosi.profiles/nano.conf` renames its `KernelModulesInitrdInclude=` list
to `KernelInitrdModules=` for the same reason, so the nano profile's trimming
starts working again instead of naming a key that no longer carries its list.

## What this teaches (LAW 10)

A default that *means* "unrestricted" in one release and "empty" in the next is
the same failure mode this file has now recorded twice: a module list that
silently matches nothing. Both times the symptom was an emergency shell and the
cause was a configuration file. The check that catches the next one is the one
used here — read the built artifact, not the config that was supposed to
produce it — so it is now part of the build: `tools/build.sh` runs
`tools/probe-initrd.py` against the UKI it just produced and refuses to call the
image buildable when the initrd is missing the root filesystem's driver.

## Verification

```bash
# the initrd in the rebuilt UKI carries the drivers now
python3 tools/probe-initrd.py --uki build/out/litricos.efi --require erofs dm-verity

# and the image boots to the desktop
tools/screen-test.sh --profile desktop --at 30 --input 'sendkey meta_l|launcher'
```

Run against the image built *before* the fix, the same command exits 1:

```
  MISSING  erofs
  MISSING  dm-verity
probe-initrd: FAIL -- the initrd does not carry erofs, dm-verity.
```
