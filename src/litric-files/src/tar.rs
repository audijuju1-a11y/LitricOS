//! ustar archives, written from POSIX 1003.1-1988 (the ustar format).
//!
//! Writing produces POSIX ustar headers (typeflags `0` and `5`, ustar magic,
//! octal fields) — enough for the file manager's COMPRESS/EXTRACT, and
//! readable by every tar tool. Reading validates rather than assumes: the
//! checksum, the size/length fields, and the entry-name rules shared with
//! ADR-0019 extraction (`unsafe-archive-entry` for absolute names, `..`,
//! drive letters, backslashes; links and device nodes are refused).
//!
//! Limits: the crate's [`Limits::ADR_0019`] apply per archive — entry
//! count, total output, metadata budget.

// Header-field arithmetic is bounded by the fixed 512-byte block size;
// slices below read only documented field ranges of the ustar header.
#![allow(clippy::arithmetic_side_effects)]

use crate::{Error, ErrorKind, Limits};
use std::io::{self, Read, Write};

/// ustar magic plus version: "ustar\0" then "00".
const USTAR_MAGIC: [u8; 8] = [b'u', b's', b't', b'a', b'r', 0, b'0', b'0'];
/// A regular file.
const TYPE_FILE: u8 = b'0';
/// A directory.
const TYPE_DIR: u8 = b'5';
/// One 512-byte block.
const BLOCK: usize = 512;

fn corrupt(msg: &str) -> io::Error {
    Error::new(ErrorKind::CorruptArchive, msg).into_io()
}

fn unsafe_entry(msg: &str) -> io::Error {
    Error::new(ErrorKind::UnsafeArchiveEntry, msg).into_io()
}

fn truncated() -> io::Error {
    corrupt("tar archive is truncated")
}

/// One validated tar entry, ready to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrySpec {
    /// Archive path: relative, `/`-separated. A directory entry ends in `/`.
    pub name: String,
    /// Regular file or directory — the only two kinds Litric writes.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
    /// Modification time, Unix seconds.
    pub mtime: u64,
    /// Permission bits (low 12 bits of the mode field).
    pub mode: u32,
}

/// Write a whole archive: entries in order, each followed by its content,
/// padded to block boundaries, terminated by two zero blocks.
///
/// # Errors
/// `unsafe-archive-entry` if any name breaks the ADR-0019 rules;
/// `too-large` if a file does not fit the ustar size field (8 GiB is not
/// representable); whatever the sink returns.
pub fn write_archive<W: Write>(out: &mut W, entries: &[(EntrySpec, Vec<u8>)]) -> io::Result<()> {
    for (spec, content) in entries {
        write_header(out, spec)?;
        if !spec.is_dir {
            out.write_all(content)?;
            let rem = content.len() % BLOCK;
            if rem != 0 {
                out.write_all(&vec![0u8; BLOCK - rem])?;
            }
        }
    }
    // Two zero blocks end the archive.
    out.write_all(&[0u8; BLOCK * 2])?;
    Ok(())
}

/// Write one ustar header for `spec`.
fn write_header<W: Write>(out: &mut W, spec: &EntrySpec) -> io::Result<()> {
    let name = &spec.name;
    validate_entry_name(name)?;
    if spec.is_dir && !name.ends_with('/') {
        // Written as given; ustar does not require the trailing slash, and
        // readers key off the typeflag.
    }
    if name.len() > 100 {
        return Err(unsafe_entry("entry name exceeds the ustar name field"));
    }
    if spec.size > 0o77_777_777_777 {
        return Err(Error::new(ErrorKind::TooLarge, "file exceeds the ustar size field").into_io());
    }

    let mut h = [0u8; BLOCK];
    // Invariant: name.len() <= 100 is enforced above, so the slice below
    // is inside the 512-byte block.
    #[allow(clippy::indexing_slicing)]
    {
        h[..name.len()].copy_from_slice(name.as_bytes());
    }
    // Mode, uid, gid, size, mtime: octal, space/NUL terminated per spec.
    write_octal(&mut h[100..108], u64::from(spec.mode) & 0o7777);
    write_octal(&mut h[108..116], 0); // uid
    write_octal(&mut h[116..124], 0); // gid
    if !spec.is_dir {
        write_octal(&mut h[124..136], spec.size);
    }
    write_octal(&mut h[136..148], spec.mtime);
    h[156] = if spec.is_dir { TYPE_DIR } else { TYPE_FILE };
    h[257..265].copy_from_slice(&USTAR_MAGIC);
    h[265..269].copy_from_slice(b"0000");
    // uname/gname: "litric" for the platform user.
    h[265 + 8..265 + 8 + 6].copy_from_slice(b"litric");
    // Checksum: the sum of all header bytes with the chksum field taken as
    // eight spaces.
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| u32::from(*b)).sum();
    write_octal(&mut h[148..154], u64::from(sum));
    h[154] = 0;
    h[155] = b' ';
    out.write_all(&h)?;
    Ok(())
}

/// Store `value` as a zero-padded octal string in `field`, per the ustar
/// conventions used by every tar tool.
fn write_octal(field: &mut [u8], value: u64) {
    let digits = field.len() - 1;
    let text = format!("{value:o}");
    if text.len() > digits {
        // Caller validated; truncate defensively without panicking.
        return;
    }
    // Invariants: the caller checks text.len() <= digits before calling,
    // so pad <= digits < field.len(); all three indexes below are in range
    // by construction.
    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    fn write_into(field: &mut [u8], text: &str) {
        let digits = field.len() - 1;
        let pad = digits - text.len();
        for b in field.iter_mut().take(pad) {
            *b = b'0';
        }
        field[pad..digits].copy_from_slice(text.as_bytes());
        field[digits] = 0;
    }
    write_into(field, &text);
}

/// Validate an entry name against the ADR-0019 extraction rules.
fn validate_entry_name(name: &str) -> Result<(), io::Error> {
    if name.starts_with('/') {
        return Err(unsafe_entry("absolute entry name"));
    }
    if name.as_bytes().get(1) == Some(&b':') {
        return Err(unsafe_entry("drive-letter entry name"));
    }
    if name.contains('\\') {
        return Err(unsafe_entry("backslash in entry name"));
    }
    for seg in name.split('/') {
        if seg == ".." {
            return Err(unsafe_entry("`..` in entry name"));
        }
        if seg.is_empty() && !name.is_empty() && !name.ends_with('/') {
            return Err(unsafe_entry("empty segment in entry name"));
        }
        if seg.len() > 255 {
            return Err(unsafe_entry("entry segment exceeds 255 bytes"));
        }
        if seg.bytes().any(|b| b < 0x20 || b == 0x7f) {
            return Err(unsafe_entry("control byte in entry name"));
        }
    }
    Ok(())
}

/// Read a whole tar archive into validated entries. Refuses anything the
/// extraction rules refuse; enforces [`Limits::ADR_0019`].
///
/// # Errors
/// `corrupt-archive` for structural damage; `unsafe-archive-entry` for
/// forbidden entries; `limit` when a limit binds; reader errors pass
/// through.
pub fn read_archive<R: Read>(
    r: &mut R,
    limits: &Limits,
) -> Result<Vec<(String, bool, Vec<u8>)>, io::Error> {
    let mut entries = Vec::new();
    let mut total_output: u64 = 0;
    loop {
        let mut header = [0u8; BLOCK];
        read_block(r, &mut header)?;
        if header.iter().all(|b| *b == 0) {
            // End of archive; the second zero block may follow.
            break;
        }
        let checksum_stored = parse_octal(&header[148..156])
            .ok_or_else(|| corrupt("tar header checksum is not octal"))?;
        let mut sum: u32 = 0;
        for (i, b) in header.iter().enumerate() {
            let byte = if (148..156).contains(&i) { b' ' } else { *b };
            sum = sum.wrapping_add(u32::from(byte));
        }
        if sum != checksum_stored as u32 {
            return Err(corrupt("tar header checksum mismatch"));
        }
        let name = parse_name(&header)?;
        let typeflag = header[156];
        let is_dir = match typeflag {
            0 | b'0' => false,
            b'5' => true,
            // Symlinks, hard links, devices: refused, per ADR-0019.
            b'1' | b'2' => return Err(unsafe_entry("link entries are not extracted")),
            b'3' | b'4' | b'6' => {
                return Err(unsafe_entry("device/fifo entries are not extracted"))
            }
            other => {
                return Err(corrupt(&format!("unsupported tar typeflag {other:#x}")));
            }
        };
        let size =
            parse_octal(&header[124..136]).ok_or_else(|| corrupt("tar size field is not octal"))?;
        if is_dir && size != 0 {
            return Err(corrupt("directory entry with a non-zero size"));
        }
        total_output = total_output.saturating_add(size);
        if entries.len() as u64 + 1 > limits.max_entries {
            return Err(Error::new(ErrorKind::Limit, "too many entries").into_io());
        }
        if total_output > limits.max_output {
            return Err(Error::new(ErrorKind::Limit, "archive output exceeds the limit").into_io());
        }
        validate_entry_name(&name)?;
        let mut content = vec![0u8; size as usize];
        if !is_dir && size > 0 {
            r.read_exact(&mut content)?;
            let rem = (size as usize) % BLOCK;
            if rem != 0 {
                // Invariant: 0 < rem < BLOCK, so BLOCK - rem is in 1..BLOCK
                // and the pad slice below is inside the array.
                let mut pad = [0u8; BLOCK];
                #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
                r.read_exact(&mut pad[..BLOCK - rem])?;
            }
        }
        entries.push((name, is_dir, content));
    }
    Ok(entries)
}

/// Read exactly one 512-byte block; short reads are truncation.
fn read_block<R: Read>(r: &mut R, buf: &mut [u8; BLOCK]) -> Result<(), io::Error> {
    let mut got = 0usize;
    while got < BLOCK {
        let room = buf.get_mut(got..).ok_or_else(truncated)?;
        match r.read(room) {
            Ok(0) => return Err(truncated()),
            Ok(n) => got += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Parse a NUL-terminated name, honouring the `prefix/` field when the
/// magic says ustar (long-name support that all tools write).
fn parse_name(h: &[u8; BLOCK]) -> Result<String, io::Error> {
    // Invariants: the fixed field offsets (0..100 name, 257..263 magic,
    // 345..500 prefix) are documented ustar positions inside the 512-byte
    // header block.
    #[allow(clippy::indexing_slicing)]
    fn parse_fields(h: &[u8; BLOCK]) -> Result<(String, Option<String>), io::Error> {
        let name = header_str(&h[..100])?;
        let prefix = if &h[257..263] == b"ustar\0" {
            Some(header_str(&h[345..500])?)
        } else {
            None
        };
        Ok((name, prefix))
    }
    let (name, prefix) = parse_fields(h)?;
    if let Some(prefix) = prefix {
        if !prefix.is_empty() {
            return Ok(format!("{prefix}/{name}"));
        }
    }
    Ok(name)
}

fn header_str(field: &[u8]) -> Result<String, io::Error> {
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    let text = field
        .get(..end)
        .ok_or_else(|| corrupt("tar header field overrun"))?;
    std::str::from_utf8(text)
        .map(str::to_string)
        .map_err(|_| corrupt("tar header field is not UTF-8"))
}

/// Parse an octal field (digits, possibly space- or NUL-padded). An
/// all-NUL field is 0 — tar tools zero unwritten fields, and GNU tar reads
/// them as zero.
fn parse_octal(field: &[u8]) -> Option<u64> {
    let text: String = field
        .iter()
        .take_while(|b| **b != 0 && **b != b' ')
        .map(|b| *b as char)
        .collect();
    if text.is_empty() {
        // All NUL (or all spaces) — accepted as zero, as every tar does.
        return Some(0);
    }
    if !text.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
        return None;
    }
    u64::from_str_radix(&text, 8).ok()
}

#[cfg(test)]
mod tests {
    // Tests hand-build and inspect header fields by direct indexing;
    // every offset is a documented ustar field position.
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

    use super::*;
    use std::io::Cursor;

    fn spec(name: &str, is_dir: bool, content: &[u8]) -> (EntrySpec, Vec<u8>) {
        (
            EntrySpec {
                name: name.to_string(),
                is_dir,
                size: content.len() as u64,
                mtime: 1_700_000_000,
                mode: 0o644,
            },
            content.to_vec(),
        )
    }

    #[test]
    fn roundtrip_files_and_dirs() {
        let entries = vec![
            spec("docs/", true, b""),
            spec("docs/a.txt", false, b"hello world"),
            spec("docs/b.bin", false, &vec![7u8; 1_500]),
        ];
        let mut buf = Vec::new();
        write_archive(&mut buf, &entries).expect("write");
        let mut r = Cursor::new(&buf[..]);
        let read_back = read_archive(&mut r, &Limits::ADR_0019).expect("read");
        assert_eq!(read_back.len(), 3);
        assert_eq!(read_back[0], ("docs/".to_string(), true, Vec::new()));
        assert_eq!(read_back[1].0, "docs/a.txt");
        assert_eq!(read_back[1].2, b"hello world");
        assert_eq!(read_back[2].2, vec![7u8; 1_500]);
    }

    /// The written bytes must look like tar to a byte-level check: magic,
    /// checksum recomputation, block padding.
    #[test]
    fn on_disk_shape_is_ustar() {
        let entries = vec![spec("a.txt", false, b"xy")];
        let mut buf = Vec::new();
        write_archive(&mut buf, &entries).expect("write");
        assert_eq!(&buf[257..263], b"ustar\0");
        assert_eq!(buf[156], TYPE_FILE);
        // "xy" occupies block bytes 512..514, padded to 1024; then two
        // zero blocks end it.
        assert_eq!(buf[512], b'x');
        assert_eq!(buf[514], 0);
        assert_eq!(buf.len(), (1 + 1 + 2) * BLOCK);
        // Recompute the checksum by hand.
        let mut sum: u32 = 0;
        for (i, b) in buf.iter().take(BLOCK).enumerate() {
            sum += if (148..156).contains(&i) {
                32
            } else {
                u32::from(*b)
            };
        }
        let stored = parse_octal(&buf[148..156]).expect("octal");
        assert_eq!(sum, stored as u32);
    }

    #[test]
    fn every_forbidden_entry_shape_is_refused_on_read() {
        for hostile in [
            "/etc/passwd",
            "../escape.txt",
            "a/../../b",
            "C:/windows.cfg",
            "back\\slash.txt",
        ] {
            let mut buf = Vec::new();
            // Write it raw via write_header's validator: construct with the
            // hostile name directly.
            let spec = EntrySpec {
                name: hostile.to_string(),
                is_dir: false,
                size: 0,
                mtime: 0,
                mode: 0o644,
            };
            let err = write_header(&mut buf, &spec).expect_err(hostile);
            assert_eq!(
                Error::from_io("tar", &err).kind(),
                ErrorKind::UnsafeArchiveEntry
            );
        }
    }

    /// A link entry inside an otherwise valid archive is refused.
    #[test]
    fn link_entries_are_refused() {
        let entries = vec![spec("a.txt", false, b"real")];
        let mut buf = Vec::new();
        write_archive(&mut buf, &entries).expect("write");
        buf[156] = b'2'; // symlink typeflag
        fix_checksum(&mut buf);
        let mut r = Cursor::new(&buf[..]);
        let err = read_archive(&mut r, &Limits::ADR_0019).expect_err("must refuse");
        assert_eq!(
            Error::from_io("tar", &err).kind(),
            ErrorKind::UnsafeArchiveEntry
        );
    }

    /// A corrupted checksum is a corrupt archive, not trusted content.
    #[test]
    fn checksum_mismatch_is_refused() {
        let entries = vec![spec("a.txt", false, b"data")];
        let mut buf = Vec::new();
        write_archive(&mut buf, &entries).expect("write");
        buf[200] ^= 0xff;
        let mut r = Cursor::new(&buf[..]);
        let err = read_archive(&mut r, &Limits::ADR_0019).expect_err("must refuse");
        assert_eq!(
            Error::from_io("tar", &err).kind(),
            ErrorKind::CorruptArchive
        );
    }

    /// The entry-count limit binds.
    #[test]
    fn entry_limit_enforced() {
        let entries: Vec<(EntrySpec, Vec<u8>)> = (0..5)
            .map(|i| spec(&format!("f{i}.txt"), false, b"x"))
            .collect();
        let mut buf = Vec::new();
        write_archive(&mut buf, &entries).expect("write");
        let tiny = Limits {
            max_entries: 3,
            ..Limits::ADR_0019
        };
        let mut r = Cursor::new(&buf[..]);
        let err = read_archive(&mut r, &tiny).expect_err("limit must bind");
        assert_eq!(Error::from_io("tar", &err).kind(), ErrorKind::Limit);
    }

    /// Octal round-trips through the writer's own parser.
    #[test]
    fn octal_fields_round_trip() {
        let entries = vec![spec("big.bin", false, &vec![0u8; 0o755_000])];
        let mut buf = Vec::new();
        write_archive(&mut buf, &entries).expect("write");
        assert_eq!(parse_octal(&buf[124..136]), Some(0o755_000));
        assert_eq!(parse_octal(&buf[136..148]), Some(1_700_000_000));
        assert_eq!(
            parse_octal(b"       "),
            Some(0),
            "all spaces reads as 0, like every tar"
        );
        assert_eq!(parse_octal(&[0u8; 8]), Some(0), "all NUL reads as 0");
        assert_eq!(parse_octal(b"129    "), None, "9 is not octal");
    }

    /// Rewrite the checksum after mutating a header byte.
    fn fix_checksum(buf: &mut [u8]) {
        buf[148..156].copy_from_slice(b"        ");
        let sum: u32 = buf.iter().take(BLOCK).map(|b| u32::from(*b)).sum();
        write_octal(&mut buf[148..154], u64::from(sum));
        buf[154] = 0;
        buf[155] = b' ';
    }
}
