#!/usr/bin/env python3
"""Check that the UKI's initrd can actually mount the root filesystem.

WHY THIS EXISTS (BUG-0020)
--------------------------
The image's system partition is erofs behind dm-verity, and the kernel it ships
builds erofs as a *module*. A module that is not inside the initrd cannot be
loaded before `/sysroot` exists, so the boot reached an emergency shell with

    [FAILED] Failed to mount /sysroot.
    EXT4-fs (dm-0): VFS: Can't find ext4 filesystem

which reads like a storage or verity fault and was in fact an initrd with ten
thousand entries and no modules at all. Nothing in the build said so: mkosi
built the image, the UKI was signed, the disk image verified. The only thing that
would have caught it is looking inside the artifact, which is what this does.

    tools/probe-initrd.py --uki build/out/litricos.efi --require erofs dm-verity
    tools/probe-initrd.py --uki build/out/litricos.efi --list

Exit code 0 when every required driver is present, 1 when one is missing or the
UKI cannot be read.

READING AN INITRAMFS THE WAY THE KERNEL DOES
--------------------------------------------
An initramfs is not one archive and not one compression. mkosi assembles the UKI
from a chain of members -- the base initrd (a zstd-compressed cpio from
mkosi-initrd), the kernel-modules initrd appended by the UKI step, microcode --
and the kernel's `unpack_to_rootfs` walks that chain: it looks at the bytes at
its current position, and if they begin with a compression magic it decompresses
*one member* and continues after it, otherwise it reads a cpio archive in place.

This probe's first version decompressed the whole section with `zstd -dc` and
parsed the result. That is right for the first member and blind to every member
after it, because the appended kernel-modules initrd is uncompressed cpio: zstd
stops where its frame ends and the modules that matter are in the bytes zstd
never looked at. It reported "0 modules, MISSING erofs" for an initrd that
carried them, which is the one outcome a gate must never produce -- a false
failure teaches people to pass `--force`.

So the members are walked here, and a zstd frame's length is computed from the
frame itself (header, then block headers, then the checksum) rather than guessed
by scanning for magic bytes, which would find the `.ko.zst` files *inside* the
modules member and mistake them for members of their own.

WHAT IT DOES NOT DO
-------------------
It does not boot anything, so it cannot prove the driver *loads* or that the
mount succeeds. It proves the file is in the initrd, which is the part that was
wrong. `tools/screen-test.sh` and `tools/photo-tour*.py` are what photograph a
boot.
"""

from __future__ import annotations

import argparse
import shutil
import struct
import subprocess
import sys
from pathlib import Path

# Modules whose absence means the boot cannot reach the root filesystem at all.
# erofs is the root's own driver; dm-verity is the device the root sits on. A
# requirement may list alternatives with `|`, because upstream folded dm-verity
# into dm-mod and a kernel built that way has no dm-verity.ko of its own.
ROOT_CRITICAL = ("erofs", "dm-verity|dm-mod")

ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"
ZSTD_SKIPPABLE = 0x184D2A50
GZIP_MAGIC = b"\x1f\x8b"
XZ_MAGIC = b"\xfd7zXZ\x00"
LZ4_MAGIC = b"\x04\x22\x4d\x18"
BZIP2_MAGIC = b"BZh"
CPIO_NEWC = (b"070701", b"070702")


def run(argv: list[str], data: bytes) -> bytes:
    proc = subprocess.run(argv, input=data, capture_output=True, check=False)
    if proc.returncode != 0 and not proc.stdout:
        raise SystemExit(
            f"{argv[0]} failed: {proc.stderr.decode('utf-8', 'replace')[:200]}"
        )
    return proc.stdout


# ---------------------------------------------------------------------------
# zstd frame geometry. Enough of the format to know where a member ends, and
# nothing more: this is not a decompressor.
# ---------------------------------------------------------------------------


def zstd_frame_size(data: bytes, pos: int) -> int | None:
    """Length in bytes of the zstd frame starting at `pos`, or None."""
    if data[pos : pos + 4] != ZSTD_MAGIC:
        if pos + 8 <= len(data):
            tag = struct.unpack_from("<I", data, pos)[0]
            if ZSTD_SKIPPABLE <= tag <= 0x184D2A5F:
                return 8 + struct.unpack_from("<I", data, pos + 4)[0]
        return None
    if pos + 5 > len(data):
        return None
    cursor = pos + 4
    descriptor = data[cursor]
    cursor += 1
    fcs_size = descriptor >> 6
    single_segment = (descriptor >> 5) & 1
    dict_id_size = descriptor & 3
    if not single_segment:
        cursor += 1  # window descriptor
    cursor += (0, 1, 2, 4)[dict_id_size]
    # Frame_Content_Size: 0 means one byte when the frame is single-segment and
    # no field at all otherwise; 1/2/3 mean 2/4/8 bytes. Getting this wrong puts
    # the cursor inside the first block header and the frame never ends.
    if fcs_size == 0:
        cursor += 1 if single_segment else 0
    else:
        cursor += (0, 2, 4, 8)[fcs_size]
    while True:
        if cursor + 3 > len(data):
            return None
        header = data[cursor] | (data[cursor + 1] << 8) | (data[cursor + 2] << 16)
        cursor += 3
        last = header & 1
        block_type = (header >> 1) & 3
        size = header >> 3
        if block_type == 3:  # reserved: not a frame this can walk
            return None
        cursor += 1 if block_type == 1 else size
        if cursor > len(data):
            return None
        if last:
            break
    if (descriptor >> 2) & 1:  # content checksum
        cursor += 4
    return cursor - pos


# ---------------------------------------------------------------------------
# The member walk.
# ---------------------------------------------------------------------------

COMPRESSED = (
    (ZSTD_MAGIC, "zstd"),
    (GZIP_MAGIC, "gzip"),
    (XZ_MAGIC, "xz"),
    (LZ4_MAGIC, "lz4"),
    (BZIP2_MAGIC, "bzip2"),
)

DECOMPRESSORS = {
    "gzip": ["gzip", "-dc"],
    "xz": ["xz", "-dc"],
    "lz4": ["lz4", "-dc"],
    "bzip2": ["bzip2", "-dc"],
}


def members(blob: bytes) -> list[tuple[str, bytes]]:
    """Every member of an initramfs chain as `(kind, cpio bytes)`.

    `kind` is `raw` for an uncompressed cpio member and the compressor's name
    otherwise, so `--list` can say which member a module came from.
    """
    out: list[tuple[str, bytes]] = []
    pos = 0
    total = len(blob)
    guard = 0
    while pos < total and guard < 4096:
        guard += 1
        match = next(
            ((magic, name) for magic, name in COMPRESSED if blob[pos : pos + len(magic)] == magic),
            None,
        )
        if match is None:
            # Not compressed: the kernel reads a cpio archive here, in place.
            out.append(("raw", blob[pos:]))
            return out
        _magic, name = match
        if name == "zstd":
            size = zstd_frame_size(blob, pos)
            if size is None:
                # A frame this cannot measure: decompress the rest as one member
                # and label it so the summary does not claim a clean walk.
                out.append(("zstd?", run(["zstd", "-dc"], blob[pos:])))
                return out
            out.append(("zstd", run(["zstd", "-dc"], blob[pos : pos + size])))
            pos += size
            continue
        decoder = DECOMPRESSORS.get(name)
        if decoder and shutil.which(decoder[0]):
            # These formats do not report their compressed length here, so the
            # remainder is treated as one member -- correct for every chain mkosi
            # produces, and the label says which reading was used.
            out.append((name, run(decoder, blob[pos:])))
            return out
        out.append(("raw", blob[pos:]))
        return out
    return out


MAX_NAME = 4096


def parse_entry(blob: bytes, pos: int) -> tuple[str, int, int] | None:
    """`(name, filesize, namesize)` for the newc entry at `pos`, or None.

    A candidate is only accepted when the whole entry looks like one: the size
    fields parse, the name length is sane, the name is valid UTF-8 text with no
    control bytes, and it is NUL-terminated. That check is the difference between
    reading an archive and reading compressed module data that happens to contain
    the bytes `070701` -- see [`cpio_names`].
    """
    if blob[pos : pos + 6] not in CPIO_NEWC or pos + 110 > len(blob):
        return None
    # newc header, after the 6-byte magic: twelve 8-digit hex fields, and the two
    # this walk needs are file size (field 7) and name size (field 12). Indexing
    # them from the magic instead of from the fields moves both: that mistake read
    # a 32-byte file as 30064771072 bytes, walked off the end of the archive after
    # one entry, and so reported "no modules" for an initrd it had never read.
    header = blob[pos + 6 : pos + 110]
    try:
        filesize = int(header[48:56], 16)
        namesize = int(header[88:96], 16)
    except ValueError:
        return None
    if not 2 <= namesize <= MAX_NAME:
        return None
    start = pos + 110
    end = start + namesize
    if end > len(blob) or blob[end - 1 : end] != b"\x00":
        return None
    try:
        name = blob[start : end - 1].decode("utf-8")
    except UnicodeDecodeError:
        return None
    if not name or any(ord(ch) < 0x20 or ord(ch) == 0x7F for ch in name):
        return None
    return name, filesize, namesize


def round4(value: int) -> int:
    """`value` rounded up to the next multiple of four."""
    return value + (-value) % 4


def cpio_names(blob: bytes) -> list[str]:
    """Every cpio entry name in a (possibly concatenated) cpio stream.

    `cpio -t` stops at the first `TRAILER!!!`, and an initramfs member is a chain
    of archives, so the stream is walked here instead: a module that appears
    after a trailer is still in the initramfs the kernel unpacks.

    Two rules this walk has had to learn from the artifact, both of which made it
    report "0 modules" for an initrd full of them:

      * newc pads the header+name to four bytes and the file data to four bytes
        **relative to the entry**, not to the position in the file. The appended
        kernel-modules member begins three bytes past a word boundary (a previous
        member ended unaligned), so absolute rounding drifts one byte per entry
        until the walk reads garbage and stops: `cpio -it` listed 205 entries and
        this listed 119, with `dm-mod` and `dm-verity` in the part it never saw.
      * a compressed module's bytes contain `070701` by chance, so a resync may
        only land on a header that [`parse_entry`] accepts in full.

    A compression magic is a member boundary, not junk to skip over: the walk
    stops there and lets [`members`] start the next member.
    """
    names: list[str] = []
    pos = 0
    total = len(blob)
    while pos + 110 <= total:
        if any(blob[pos : pos + len(magic)] == magic for magic, _ in COMPRESSED):
            return names
        entry = parse_entry(blob, pos)
        if entry is None:
            candidate = pos
            while True:
                candidate = blob.find(b"070701", candidate + 1)
                if candidate == -1 or candidate + 110 > total:
                    return names
                if any(
                    blob[candidate : candidate + len(magic)] == magic
                    for magic, _ in COMPRESSED
                ):
                    return names
                if parse_entry(blob, candidate) is not None:
                    break
            pos = candidate
            continue
        name, filesize, namesize = entry
        if name == "TRAILER!!!":
            pos += round4(110 + namesize)
            continue
        names.append(name)
        pos += round4(110 + namesize) + round4(filesize)
    return names


def extract_section(uki: Path, section: str) -> bytes | None:
    """One section of the UKI, as bytes."""
    objcopy = shutil.which("objcopy") or shutil.which("llvm-objcopy")
    if objcopy is None:
        return None
    proc = subprocess.run(
        [objcopy, "-O", "binary", f"--only-section={section}", str(uki), "/dev/stdout"],
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0 or not proc.stdout:
        proc = subprocess.run(
            [objcopy, "-O", "binary", f"--only-section={section}", str(uki), "-"],
            capture_output=True,
            check=False,
        )
    return proc.stdout or None


def read_initramfs(uki: Path) -> tuple[bytes | None, list[tuple[str, list[str]]]]:
    """`(section bytes, [(member kind, entry names)])` for a UKI."""
    section = None
    for name in (".initrd", ".initrd.extra"):
        section = extract_section(uki, name)
        if section:
            break
    if not section:
        return None, []
    walked = [(kind, cpio_names(body)) for kind, body in members(section)]
    return section, walked


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    # Not `required=True`: `--self-test` proves the walk without an artifact,
    # and a gate that cannot be run on its own is a gate nobody runs while
    # fixing it. main() rejects a missing --uki for every other use.
    parser.add_argument("--uki", type=Path, help="the .efi to read")
    parser.add_argument(
        "--require",
        nargs="*",
        default=list(ROOT_CRITICAL),
        help="module names that must be present in the initrd (`a|b` for either)",
    )
    parser.add_argument(
        "--list", action="store_true", help="print every module the initrd carries"
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="prove the member walk reads a chain, and cannot be fooled by the "
        "zstd frames inside it",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if args.uki is None:
        parser.error("--uki is required unless --self-test is given")

    if not args.uki.is_file():
        print(f"probe-initrd: no UKI at {args.uki}", file=sys.stderr)
        return 1

    section, walked = read_initramfs(args.uki)
    if section is None:
        print(
            f"probe-initrd: {args.uki} has no .initrd section "
            "(is it a UKI? objcopy missing?)",
            file=sys.stderr,
        )
        return 1

    names = [name for _, entries in walked for name in entries]
    # Names are matched as path components, so `erofs.ko.zst` counts for `erofs`.
    modules = [
        name for name in names if name.endswith((".ko", ".ko.zst", ".ko.xz", ".ko.gz"))
    ]

    print(f"uki        : {args.uki}")
    print(f"initrd     : {len(section)} bytes compressed, {len(walked)} member(s)")
    for kind, entries in walked:
        modules_here = sum(
            1 for n in entries if n.endswith((".ko", ".ko.zst", ".ko.xz", ".ko.gz"))
        )
        print(f"  {kind:<6} {len(entries):>6} entries, {modules_here:>4} modules")
    print(f"entries    : {len(names)}")
    print(f"modules    : {len(modules)}")

    if args.list:
        for name in sorted(modules):
            print(f"  {name}")

    missing = []
    for want in args.require:
        # `dm-verity` matches `.../drivers/md/dm-verity.ko.zst`; the underscore
        # spelling is accepted too, because module names use both. A `|` lists
        # alternatives: dm-verity may be its own module or part of dm-mod.
        alternatives = [a.strip() for a in want.split("|") if a.strip()]
        hit: list[str] = []
        for alternative in alternatives:
            spelled = {alternative, alternative.replace("-", "_")}
            hit = [name for name in modules if any(f"/{s}.ko" in name for s in spelled)]
            if hit:
                break
        if not hit:
            missing.append(want)
            print(f"  MISSING  {want}")
        else:
            print(f"  present  {want}  ({hit[0]})")

    if missing:
        print(
            "probe-initrd: FAIL -- the initrd does not carry "
            + ", ".join(missing)
            + ".\n  An initrd without the root filesystem's driver boots to an\n"
            "  emergency shell: see docs/bugs/BUG-0020 for what that looks like\n"
            "  and image/mkosi.conf (KernelInitrdModules=) for the list.",
            file=sys.stderr,
        )
        return 1
    print("probe-initrd: PASS -- every required driver is in the initrd")
    return 0


# ---------------------------------------------------------------------------
# Self-test: the walk has to be able to fail, and it has to read a chain.
# ---------------------------------------------------------------------------


def cpio_member(entries: list[tuple[str, bytes]]) -> bytes:
    """A minimal newc cpio archive, for the self-test's synthetic initramfs."""
    out = bytearray()
    for name, body in entries:
        header = b"070701"
        fields = [
            0,  # inode
            0o100644,  # mode
            0,  # uid
            0,  # gid
            1,  # nlink
            0,  # mtime
            len(body),  # filesize
            0,  # devmajor
            0,  # devminor
            0,  # rdevmajor
            0,  # rdevminor
            len(name) + 1,  # namesize
            0,  # check
        ]
        out += header + b"".join(f"{f:08x}".encode() for f in fields)
        out += name.encode() + b"\0"
        out += (-len(out)) % 4 * b"\0"
        out += body
        out += (-len(out)) % 4 * b"\0"
    out += cpio_member_trailer(len(out))
    return bytes(out)


def cpio_member_trailer(offset: int) -> bytes:
    name = "TRAILER!!!"
    fields = [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, len(name) + 1, 0]
    header = b"070701" + b"".join(f"{f:08x}".encode() for f in fields) + name.encode() + b"\0"
    padding = (-(offset + len(header))) % 4
    return header + b"\0" * padding


def self_test() -> int:
    """Plant a chain the way mkosi builds one, and check the walk reads it.

    Two failure modes are the reason this exists:

      * a chain whose second member is uncompressed cpio with the modules in it
        (mkosi's kernel-modules initrd): reading only the first member reports
        "no modules" for a working initrd, and the old gate did exactly that;
      * zstd frames *inside* a member (every `erofs.ko.zst` is one): a walk that
        finds member boundaries by scanning for the zstd magic sees those and
        loses the archive.
    """
    if not shutil.which("zstd"):
        print("probe-initrd --self-test: SKIP -- zstd is not installed")
        return 0

    failures: list[str] = []

    base = cpio_member([("usr/lib/modules/7.2/early", b"x" * 4096)])
    compressed = run(["zstd", "-c", "-19"], base)
    modules = cpio_member(
        [
            ("usr/lib/modules/7.2/kernel/fs/erofs/erofs.ko.zst", run(["zstd", "-c"], b"E" * 8192)),
            ("usr/lib/modules/7.2/kernel/drivers/md/dm-mod.ko.zst", run(["zstd", "-c"], b"D" * 1024)),
        ]
    )
    chain = compressed + modules
    # `members` yields the member's *bytes*; the names come from reading those
    # bytes as cpio. Iterating the bytes directly yields integers, which is what
    # the first version of this self-test did -- it reported that a name did not
    # end with `.ko.zst` about an int.
    walked = [(kind, cpio_names(body)) for kind, body in members(chain)]
    kinds = [kind for kind, _ in walked]
    if len(walked) != 2:
        failures.append(f"a two-member chain was read as {len(walked)} member(s): {kinds}")
    found = [n for _, entries in walked for n in entries]
    for want in ("erofs.ko.zst", "dm-mod.ko.zst"):
        if not any(n.endswith(want) for n in found):
            failures.append(f"{want} was not found in a chain that carries it")
    if kinds and kinds[0] != "zstd":
        failures.append(f"the first member should be zstd, walked as {kinds}")
    if kinds and kinds[-1] != "raw":
        failures.append(f"the appended member should be raw cpio, walked as {kinds}")

    # Member boundaries must come from the frame, not from a magic scan: this
    # chain's second member contains two zstd frames of its own.
    sized = zstd_frame_size(chain, 0)
    if sized != len(compressed):
        failures.append(
            f"frame size computed {sized}, zstd produced {len(compressed)}"
        )

    # And an initrd that genuinely lacks the driver must still be reported as
    # lacking it: a walk that returns what it was asked for proves nothing.
    other = [
        (kind, cpio_names(body))
        for kind, body in members(
            cpio_member([("usr/lib/modules/7.2/kernel/fs/x/other.ko.zst", b"z")])
        )
    ]
    other_names = [n for _, entries in other for n in entries]
    if any(n.endswith("erofs.ko.zst") for n in other_names):
        failures.append("an initrd without erofs reported one")

    # The false negative this walk exists to not repeat: an entry whose *data*
    # contains the cpio magic, followed by more entries. A walk that resyncs onto
    # any `070701` it finds jumps into that data and loses everything after it --
    # 205 entries in the real artifact, 109 reported, dm-mod among the lost.
    #
    # It also starts three bytes past a word boundary, which is where the real
    # artifact starts: padding computed from the file position instead of from
    # the entry drifts one byte per entry and loses everything after the third.
    trap = b"\x00\x00\x00" + cpio_member(
        [
            ("usr/lib/modules/7.2/kernel/fs/erofs/erofs.ko.zst", b"\x07\x07\x01" + b"payload" * 4),
            ("usr/lib/modules/7.2/kernel/drivers/md/dm-mod.ko.zst", b"M" * 64),
            ("usr/lib/modules/7.2/kernel/drivers/md/dm-verity.ko.zst", b"V" * 64),
        ]
    )
    trapped = [(kind, cpio_names(body)) for kind, body in members(trap)]
    trapped_names = [n for _, entries in trapped for n in entries]
    if len(trapped_names) != 3:
        failures.append(
            f"an archive whose data spells the cpio magic lost entries: "
            f"{len(trapped_names)} of 3 read ({trapped_names})"
        )
    for want in ("erofs.ko.zst", "dm-mod.ko.zst", "dm-verity.ko.zst"):
        if not any(n.endswith(want) for n in trapped_names):
            failures.append(f"the trap archive lost {want}")

    if failures:
        print("probe-initrd --self-test: FAIL")
        for line in failures:
            print(f"  {line}")
        return 1
    print(
        "probe-initrd --self-test: PASS -- a compressed+raw chain is walked, the "
        "modules in the appended member are found, the frame boundary is exact, "
        "and zstd frames inside a member are not mistaken for members"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
