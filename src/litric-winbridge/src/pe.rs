//! Portable Executable inspection.
//!
//! Parses just enough of the PE/COFF format to answer the questions LitricOS
//! actually needs to answer before it will run a Windows binary:
//!
//! * Is this even a PE, or is it something wearing an `.exe` extension?
//! * Which architecture? (An ARM64 binary on an x86-64 machine is a clear
//!   "no", not a mysterious crash three seconds into a Wine prefix.)
//! * GUI or console? Decides whether the shell opens a terminal surface.
//! * .NET or native? Decides whether the bottle needs a .NET runtime staged.
//! * 16-bit? Decides whether to say "this will never work" up front.
//! * What hardening did the vendor enable? Surfaced to the user, because a
//!   Windows binary without ASLR or DEP is running inside our sandbox and the
//!   user deserves to know what they are trusting.
//!
//! # Threat model
//!
//! **Every byte this module reads is attacker-controlled.** A malicious `.exe`
//! is the entire point of the exercise. So this parser:
//!
//! * never indexes without a bounds check (`clippy::indexing_slicing` is denied
//!   workspace-wide),
//! * never trusts a length field from the file,
//! * never allocates based on a value read from the file,
//! * returns `Result` for everything and panics for nothing.
//!
//! It is a target for `cargo-fuzz` in CI. See `fuzz/fuzz_targets/pe_parse.rs`.

use core::fmt;

/// Why a file could not be inspected as a PE image.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PeError {
    /// Fewer bytes than the smallest possible PE header.
    TooSmall {
        /// Bytes actually available.
        len: usize,
    },
    /// No `MZ` magic at offset 0.
    NotMzImage,
    /// `e_lfanew` pointed outside the file.
    BadPeOffset {
        /// The offset the DOS header claimed.
        offset: u32,
        /// Bytes actually available.
        len: usize,
    },
    /// No `PE\0\0` signature where `e_lfanew` pointed.
    NotPeImage,
    /// The COFF header ran past the end of the file.
    TruncatedCoffHeader,
    /// The optional header ran past the end of the file.
    TruncatedOptionalHeader,
    /// Optional header magic was neither PE32 (`0x10b`) nor PE32+ (`0x20b`).
    UnknownOptionalMagic {
        /// The magic value found.
        magic: u16,
    },
    /// `SizeOfOptionalHeader` was zero -- an object file, not an image.
    NoOptionalHeader,
}

impl fmt::Display for PeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall { len } => write!(f, "file is {len} bytes, too small to be a PE image"),
            Self::NotMzImage => write!(f, "no MZ signature: not a DOS/PE executable"),
            Self::BadPeOffset { offset, len } => {
                write!(f, "PE header offset {offset} is outside a {len}-byte file")
            }
            Self::NotPeImage => write!(f, "no PE signature: DOS executable without a PE image"),
            Self::TruncatedCoffHeader => write!(f, "COFF header is truncated"),
            Self::TruncatedOptionalHeader => write!(f, "optional header is truncated"),
            Self::UnknownOptionalMagic { magic } => {
                write!(f, "unknown optional header magic {magic:#06x}")
            }
            Self::NoOptionalHeader => write!(f, "no optional header: object file, not an image"),
        }
    }
}

impl std::error::Error for PeError {}

/// Target machine architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Machine {
    /// 32-bit x86.
    I386,
    /// 64-bit x86.
    Amd64,
    /// 64-bit ARM.
    Arm64,
    /// 32-bit ARM (Thumb-2).
    ArmNt,
    /// Itanium. Historic; listed so it is reported rather than "unknown".
    Ia64,
    /// Something else, reported verbatim.
    Other(u16),
}

impl Machine {
    const fn from_u16(v: u16) -> Self {
        match v {
            0x014c => Self::I386,
            0x8664 => Self::Amd64,
            0xaa64 => Self::Arm64,
            0x01c4 => Self::ArmNt,
            0x0200 => Self::Ia64,
            other => Self::Other(other),
        }
    }

    /// Whether an x86-64 LitricOS host can execute this under the compatibility
    /// layer.
    ///
    /// Both x86 and x86-64 are runnable: Wine 11 completed WoW64, so 32-bit
    /// titles no longer need a full 32-bit library set on the host. ARM64
    /// Windows binaries are not runnable on an x86-64 host, and saying so
    /// immediately is far better than a confusing failure later.
    #[must_use]
    pub const fn runnable_on_x86_64(self) -> bool {
        matches!(self, Self::I386 | Self::Amd64)
    }

    /// Stable token for logs and UI.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::I386 => "x86",
            Self::Amd64 => "x86_64",
            Self::Arm64 => "arm64",
            Self::ArmNt => "arm32",
            Self::Ia64 => "ia64",
            Self::Other(_) => "unknown",
        }
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(v) => write!(f, "unknown({v:#06x})"),
            other => f.write_str(other.token()),
        }
    }
}

/// The Windows subsystem the image asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Subsystem {
    /// A driver or native NT image. Not runnable in a compatibility layer.
    Native,
    /// A windowed application.
    Gui,
    /// A console application.
    Console,
    /// An EFI image. Not a Windows application at all.
    Efi,
    /// Something else.
    Other(u16),
}

impl Subsystem {
    const fn from_u16(v: u16) -> Self {
        match v {
            1 => Self::Native,
            2 => Self::Gui,
            3 => Self::Console,
            10..=13 => Self::Efi,
            other => Self::Other(other),
        }
    }

    /// Stable token for logs and UI.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Gui => "gui",
            Self::Console => "console",
            Self::Efi => "efi",
            Self::Other(_) => "unknown",
        }
    }
}

impl fmt::Display for Subsystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Security mitigations the vendor compiled in.
///
/// Surfaced to the user before they run an untrusted binary. A 2026 application
/// shipping without ASLR or DEP is a meaningful signal about how much care went
/// into it, and the user is entitled to that signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hardening {
    /// ASLR (`IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE`).
    pub aslr: bool,
    /// DEP / NX (`IMAGE_DLLCHARACTERISTICS_NX_COMPAT`).
    pub nx: bool,
    /// Control Flow Guard (`IMAGE_DLLCHARACTERISTICS_GUARD_CF`).
    pub cfg: bool,
    /// High-entropy 64-bit ASLR.
    pub high_entropy_va: bool,
    /// Structured exception handling present (absence of `NO_SEH`).
    pub seh: bool,
    /// Forces integrity (signature) checking at load.
    pub force_integrity: bool,
}

impl Hardening {
    /// A crude 0-4 score for UI display. Not a security guarantee, and labelled
    /// as such wherever it is shown.
    #[must_use]
    pub fn score(self) -> u8 {
        // Counted rather than summed with casts: the maximum is 4 either way,
        // but this form has no arithmetic for a reviewer (or a lint) to have to
        // reason about.
        [self.aslr, self.nx, self.cfg, self.high_entropy_va]
            .into_iter()
            .filter(|b| *b)
            .count() as u8
    }
}

/// What was learned about a PE image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeInfo {
    /// Target architecture.
    pub machine: Machine,
    /// Requested Windows subsystem.
    pub subsystem: Subsystem,
    /// True for PE32+ (64-bit), false for PE32 (32-bit).
    pub is_64bit: bool,
    /// True if `IMAGE_FILE_DLL` is set: a library, not an application.
    pub is_dll: bool,
    /// True if the image has a CLR header: a .NET assembly.
    pub is_dotnet: bool,
    /// Minimum Windows version the image declares it needs.
    pub min_os: (u16, u16),
    /// Number of sections declared in the COFF header.
    pub section_count: u16,
    /// Compile-time mitigations.
    pub hardening: Hardening,
    /// Whether an Authenticode certificate table is present.
    ///
    /// Presence is **not** validity: this says a signature blob exists, not that
    /// it verifies, not that the signer is trusted, and not that the binary is
    /// safe. Verification is a separate step and this field must never be
    /// rendered to the user as "signed".
    pub has_certificate_table: bool,
}

impl PeInfo {
    /// Whether an x86-64 LitricOS host could plausibly run this image.
    ///
    /// "Plausibly" is doing real work in that sentence: this rules out the
    /// categorical impossibilities (wrong architecture, kernel driver, EFI
    /// binary, DLL with no entry point). It does not predict that any given
    /// application works, and nothing in LitricOS may claim that it does.
    #[must_use]
    pub const fn is_runnable_on_x86_64(&self) -> bool {
        self.machine.runnable_on_x86_64()
            && !self.is_dll
            && matches!(self.subsystem, Subsystem::Gui | Subsystem::Console)
    }

    /// A specific reason the image cannot run, if there is one.
    #[must_use]
    pub fn blocking_reason(&self) -> Option<&'static str> {
        if self.is_dll {
            return Some("this is a DLL, not an application; open the program that uses it");
        }
        match self.subsystem {
            Subsystem::Native => return Some(
                "this is a Windows kernel driver, which cannot run under any compatibility layer",
            ),
            Subsystem::Efi => return Some("this is UEFI firmware, not a Windows application"),
            _ => {}
        }
        if !self.machine.runnable_on_x86_64() {
            return Some("this program was built for a different processor architecture");
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Bounds-checked little-endian readers.
//
// Every read goes through these. They return Option rather than panicking,
// because the input is hostile and a panic in a file-type detector is a denial
// of service on the file manager.
// ---------------------------------------------------------------------------

fn u16_at(buf: &[u8], off: usize) -> Option<u16> {
    let end = off.checked_add(2)?;
    let slice = buf.get(off..end)?;
    let a = *slice.first()?;
    let b = *slice.get(1)?;
    Some(u16::from_le_bytes([a, b]))
}

fn u32_at(buf: &[u8], off: usize) -> Option<u32> {
    let end = off.checked_add(4)?;
    let slice = buf.get(off..end)?;
    let mut arr = [0u8; 4];
    for (dst, src) in arr.iter_mut().zip(slice.iter()) {
        *dst = *src;
    }
    Some(u32::from_le_bytes(arr))
}

/// Smallest input we will even look at.
const MIN_PE_LEN: usize = 0x40;

/// DOS `MZ` magic.
const MZ_MAGIC: u16 = 0x5a4d;
/// Offset of `e_lfanew` in the DOS header.
const E_LFANEW_OFF: usize = 0x3c;
/// `PE\0\0`.
const PE_MAGIC: u32 = 0x0000_4550;

const COFF_MACHINE_OFF: usize = 4;
const COFF_NUMSECTIONS_OFF: usize = 6;
const COFF_SIZEOFOPT_OFF: usize = 20;
const COFF_CHARACTERISTICS_OFF: usize = 22;
const COFF_HEADER_LEN: usize = 24; // signature (4) + COFF fields (20)

const OPT_MAGIC_PE32: u16 = 0x010b;
const OPT_MAGIC_PE32PLUS: u16 = 0x020b;

// Offsets *within* the optional header. Subsystem and the OS-version fields sit
// at the same offset in PE32 and PE32+ because the 4-byte BaseOfData present
// only in PE32 is balanced by ImageBase being 8 bytes instead of 4 in PE32+.
const OPT_MAJOR_OS_OFF: usize = 40;
const OPT_MINOR_OS_OFF: usize = 42;
const OPT_SUBSYSTEM_OFF: usize = 68;
const OPT_DLLCHARACTERISTICS_OFF: usize = 70;

// Where the data directory array starts, which differs between the two forms.
const OPT_DATADIR_OFF_PE32: usize = 96;
const OPT_DATADIR_OFF_PE32PLUS: usize = 112;
const DATADIR_ENTRY_LEN: usize = 8;
const DATADIR_INDEX_CERTIFICATE: usize = 4;
const DATADIR_INDEX_CLR: usize = 14;

const IMAGE_FILE_DLL: u16 = 0x2000;

const DLLCHAR_HIGH_ENTROPY_VA: u16 = 0x0020;
const DLLCHAR_DYNAMIC_BASE: u16 = 0x0040;
const DLLCHAR_FORCE_INTEGRITY: u16 = 0x0080;
const DLLCHAR_NX_COMPAT: u16 = 0x0100;
const DLLCHAR_NO_SEH: u16 = 0x0400;
const DLLCHAR_GUARD_CF: u16 = 0x4000;

/// Inspect a byte slice as a PE image.
///
/// Only the headers are read, so passing the first 4 KiB of a multi-gigabyte
/// installer is both sufficient and the intended usage — the file manager must
/// not read a whole DVD image to decide which icon to draw.
///
/// # Errors
/// A [`PeError`] naming the first structural problem found.
pub fn inspect(buf: &[u8]) -> Result<PeInfo, PeError> {
    if buf.len() < MIN_PE_LEN {
        return Err(PeError::TooSmall { len: buf.len() });
    }
    if u16_at(buf, 0) != Some(MZ_MAGIC) {
        return Err(PeError::NotMzImage);
    }

    let e_lfanew = u32_at(buf, E_LFANEW_OFF).ok_or(PeError::NotMzImage)?;
    let pe_off = usize::try_from(e_lfanew).map_err(|_| PeError::BadPeOffset {
        offset: e_lfanew,
        len: buf.len(),
    })?;

    // The offset must leave room for the whole COFF header, and must not be so
    // large that adding to it overflows.
    let coff_end = pe_off
        .checked_add(COFF_HEADER_LEN)
        .ok_or(PeError::BadPeOffset {
            offset: e_lfanew,
            len: buf.len(),
        })?;
    if coff_end > buf.len() {
        return Err(PeError::BadPeOffset {
            offset: e_lfanew,
            len: buf.len(),
        });
    }

    if u32_at(buf, pe_off) != Some(PE_MAGIC) {
        return Err(PeError::NotPeImage);
    }

    // Slice the COFF header out once, then read within it at small constant
    // offsets. This is deliberate: computing `pe_off + FIELD` at each read makes
    // every read's safety depend on a bounds check written ten lines earlier,
    // which is the pattern that silently breaks the first time someone reorders
    // the function. Bounding the slice makes each read locally safe.
    let coff = buf
        .get(pe_off..coff_end)
        .ok_or(PeError::TruncatedCoffHeader)?;

    let machine_raw = u16_at(coff, COFF_MACHINE_OFF).ok_or(PeError::TruncatedCoffHeader)?;
    let section_count = u16_at(coff, COFF_NUMSECTIONS_OFF).ok_or(PeError::TruncatedCoffHeader)?;
    let size_of_optional = u16_at(coff, COFF_SIZEOFOPT_OFF).ok_or(PeError::TruncatedCoffHeader)?;
    let characteristics =
        u16_at(coff, COFF_CHARACTERISTICS_OFF).ok_or(PeError::TruncatedCoffHeader)?;

    if size_of_optional == 0 {
        return Err(PeError::NoOptionalHeader);
    }

    let opt_off = coff_end;
    let opt_magic = u16_at(buf, opt_off).ok_or(PeError::TruncatedOptionalHeader)?;
    let is_64bit = match opt_magic {
        OPT_MAGIC_PE32 => false,
        OPT_MAGIC_PE32PLUS => true,
        magic => return Err(PeError::UnknownOptionalMagic { magic }),
    };

    let read_opt_u16 = |field_off: usize| -> Option<u16> {
        // Refuse to read past what the header itself claims to be, so a
        // truncated-but-self-consistent file cannot make us read a neighbouring
        // structure and report confident nonsense.
        if field_off.checked_add(2)? > usize::from(size_of_optional) {
            return None;
        }
        u16_at(buf, opt_off.checked_add(field_off)?)
    };

    let subsystem_raw = read_opt_u16(OPT_SUBSYSTEM_OFF).ok_or(PeError::TruncatedOptionalHeader)?;
    let dll_chars = read_opt_u16(OPT_DLLCHARACTERISTICS_OFF).unwrap_or(0);
    let major_os = read_opt_u16(OPT_MAJOR_OS_OFF).unwrap_or(0);
    let minor_os = read_opt_u16(OPT_MINOR_OS_OFF).unwrap_or(0);

    let datadir_base = if is_64bit {
        OPT_DATADIR_OFF_PE32PLUS
    } else {
        OPT_DATADIR_OFF_PE32
    };

    // A data-directory entry counts as "present" only if its RVA is non-zero.
    // A zero RVA with a non-zero size is malformed and treated as absent.
    let dir_present = |index: usize| -> bool {
        let Some(entry_off) = index
            .checked_mul(DATADIR_ENTRY_LEN)
            .and_then(|o| datadir_base.checked_add(o))
        else {
            return false;
        };
        if entry_off.saturating_add(DATADIR_ENTRY_LEN) > usize::from(size_of_optional) {
            return false;
        }
        let Some(abs) = opt_off.checked_add(entry_off) else {
            return false;
        };
        u32_at(buf, abs).is_some_and(|rva| rva != 0)
    };

    Ok(PeInfo {
        machine: Machine::from_u16(machine_raw),
        subsystem: Subsystem::from_u16(subsystem_raw),
        is_64bit,
        is_dll: characteristics & IMAGE_FILE_DLL != 0,
        is_dotnet: dir_present(DATADIR_INDEX_CLR),
        min_os: (major_os, minor_os),
        section_count,
        hardening: Hardening {
            aslr: dll_chars & DLLCHAR_DYNAMIC_BASE != 0,
            nx: dll_chars & DLLCHAR_NX_COMPAT != 0,
            cfg: dll_chars & DLLCHAR_GUARD_CF != 0,
            high_entropy_va: dll_chars & DLLCHAR_HIGH_ENTROPY_VA != 0,
            seh: dll_chars & DLLCHAR_NO_SEH == 0,
            force_integrity: dll_chars & DLLCHAR_FORCE_INTEGRITY != 0,
        },
        has_certificate_table: dir_present(DATADIR_INDEX_CERTIFICATE),
    })
}

/// Whether a buffer looks like a Microsoft Installer database.
///
/// MSI files are OLE2 compound documents, not PE images, so [`inspect`] rejects
/// them and the caller needs this separate check to route them to `msiexec`.
#[must_use]
pub fn is_msi(buf: &[u8]) -> bool {
    const OLE_MAGIC: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];
    buf.get(..8).is_some_and(|head| head == OLE_MAGIC)
}

#[cfg(test)]
// Offset arithmetic in test scaffolding builds synthetic headers from small
// constants. The production rule stays absolute -- this exemption is scoped to
// the test module and visible in review, rather than weakening the workspace
// lint for everyone.
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    /// Build a synthetic but structurally valid PE header.
    ///
    /// Hand-constructing the bytes is deliberate: it means the parser is tested
    /// against the format as specified, not against whatever a particular
    /// compiler happened to emit.
    fn synth_pe(
        machine: u16,
        subsystem: u16,
        pe32_plus: bool,
        characteristics: u16,
        dll_chars: u16,
        clr_rva: u32,
    ) -> Vec<u8> {
        let opt_len: usize = if pe32_plus { 240 } else { 224 };
        let pe_off: usize = 0x80;
        let total = pe_off + COFF_HEADER_LEN + opt_len;
        let mut b = vec![0u8; total];

        let put16 = |b: &mut Vec<u8>, off: usize, v: u16| {
            let bytes = v.to_le_bytes();
            if let Some(s) = b.get_mut(off..off + 2) {
                s.copy_from_slice(&bytes);
            }
        };
        let put32 = |b: &mut Vec<u8>, off: usize, v: u32| {
            let bytes = v.to_le_bytes();
            if let Some(s) = b.get_mut(off..off + 4) {
                s.copy_from_slice(&bytes);
            }
        };

        put16(&mut b, 0, MZ_MAGIC);
        put32(&mut b, E_LFANEW_OFF, pe_off as u32);
        put32(&mut b, pe_off, PE_MAGIC);
        put16(&mut b, pe_off + COFF_MACHINE_OFF, machine);
        put16(&mut b, pe_off + COFF_NUMSECTIONS_OFF, 5);
        put16(&mut b, pe_off + COFF_SIZEOFOPT_OFF, opt_len as u16);
        put16(&mut b, pe_off + COFF_CHARACTERISTICS_OFF, characteristics);

        let opt = pe_off + COFF_HEADER_LEN;
        put16(
            &mut b,
            opt,
            if pe32_plus {
                OPT_MAGIC_PE32PLUS
            } else {
                OPT_MAGIC_PE32
            },
        );
        put16(&mut b, opt + OPT_MAJOR_OS_OFF, 10);
        put16(&mut b, opt + OPT_MINOR_OS_OFF, 0);
        put16(&mut b, opt + OPT_SUBSYSTEM_OFF, subsystem);
        put16(&mut b, opt + OPT_DLLCHARACTERISTICS_OFF, dll_chars);

        let datadir = if pe32_plus {
            OPT_DATADIR_OFF_PE32PLUS
        } else {
            OPT_DATADIR_OFF_PE32
        };
        put32(
            &mut b,
            opt + datadir + DATADIR_INDEX_CLR * DATADIR_ENTRY_LEN,
            clr_rva,
        );
        b
    }

    #[test]
    fn parses_a_64bit_gui_application() {
        let bytes = synth_pe(
            0x8664,
            2,
            true,
            0x0002,
            DLLCHAR_DYNAMIC_BASE | DLLCHAR_NX_COMPAT,
            0,
        );
        let info = inspect(&bytes).expect("valid PE");
        assert_eq!(info.machine, Machine::Amd64);
        assert_eq!(info.subsystem, Subsystem::Gui);
        assert!(info.is_64bit);
        assert!(!info.is_dll);
        assert!(!info.is_dotnet);
        assert_eq!(info.min_os, (10, 0));
        assert_eq!(info.section_count, 5);
        assert!(info.hardening.aslr);
        assert!(info.hardening.nx);
        assert!(!info.hardening.cfg);
        assert!(info.is_runnable_on_x86_64());
        assert_eq!(info.blocking_reason(), None);
    }

    #[test]
    fn parses_a_32bit_console_application() {
        let bytes = synth_pe(0x014c, 3, false, 0x0002, 0, 0);
        let info = inspect(&bytes).expect("valid PE");
        assert_eq!(info.machine, Machine::I386);
        assert_eq!(info.subsystem, Subsystem::Console);
        assert!(!info.is_64bit);
        // WoW64 in Wine 11 means 32-bit is still runnable on an x86-64 host.
        assert!(info.is_runnable_on_x86_64());
    }

    #[test]
    fn detects_dotnet_assemblies() {
        let native = synth_pe(0x8664, 2, true, 0x0002, 0, 0);
        assert!(!inspect(&native).expect("valid").is_dotnet);

        let managed = synth_pe(0x8664, 2, true, 0x0002, 0, 0x2000);
        assert!(inspect(&managed).expect("valid").is_dotnet);
    }

    #[test]
    fn subsystem_offset_is_correct_for_both_pe_forms() {
        // Regression guard. Subsystem lives at optional-header offset 68 in BOTH
        // PE32 and PE32+; getting this wrong misreads a GUI app as a driver and
        // refuses to run perfectly good software.
        for plus in [false, true] {
            let b = synth_pe(0x8664, 2, plus, 0x0002, 0, 0);
            assert_eq!(
                inspect(&b).expect("valid").subsystem,
                Subsystem::Gui,
                "pe32+={plus}"
            );
            let b = synth_pe(0x8664, 1, plus, 0x0002, 0, 0);
            assert_eq!(
                inspect(&b).expect("valid").subsystem,
                Subsystem::Native,
                "pe32+={plus}"
            );
        }
    }

    #[test]
    fn refuses_things_that_cannot_run_with_a_specific_reason() {
        let driver = inspect(&synth_pe(0x8664, 1, true, 0x0002, 0, 0)).expect("valid");
        assert!(!driver.is_runnable_on_x86_64());
        assert!(driver
            .blocking_reason()
            .is_some_and(|r| r.contains("driver")));

        let dll = inspect(&synth_pe(0x8664, 2, true, IMAGE_FILE_DLL, 0, 0)).expect("valid");
        assert!(!dll.is_runnable_on_x86_64());
        assert!(dll.blocking_reason().is_some_and(|r| r.contains("DLL")));

        let arm = inspect(&synth_pe(0xaa64, 2, true, 0x0002, 0, 0)).expect("valid");
        assert!(!arm.is_runnable_on_x86_64());
        assert!(arm
            .blocking_reason()
            .is_some_and(|r| r.contains("architecture")));

        let efi = inspect(&synth_pe(0x8664, 11, true, 0x0002, 0, 0)).expect("valid");
        assert!(efi.blocking_reason().is_some_and(|r| r.contains("UEFI")));
    }

    // ---- hostile input -----------------------------------------------------

    #[test]
    fn rejects_non_pe_input_without_panicking() {
        assert!(matches!(inspect(&[]), Err(PeError::TooSmall { len: 0 })));
        assert!(matches!(inspect(&[0u8; 10]), Err(PeError::TooSmall { .. })));
        assert!(matches!(inspect(&[0u8; 128]), Err(PeError::NotMzImage)));

        // ELF, not PE.
        let mut elf = vec![0u8; 128];
        if let Some(s) = elf.get_mut(..4) {
            s.copy_from_slice(&[0x7f, b'E', b'L', b'F']);
        }
        assert!(matches!(inspect(&elf), Err(PeError::NotMzImage)));
    }

    #[test]
    fn rejects_out_of_range_pe_offset() {
        // The classic: MZ present, e_lfanew points to the moon.
        let mut b = vec![0u8; 128];
        if let Some(s) = b.get_mut(..2) {
            s.copy_from_slice(&MZ_MAGIC.to_le_bytes());
        }
        if let Some(s) = b.get_mut(E_LFANEW_OFF..E_LFANEW_OFF + 4) {
            s.copy_from_slice(&u32::MAX.to_le_bytes());
        }
        assert!(matches!(inspect(&b), Err(PeError::BadPeOffset { .. })));

        // And a value that is in range but leaves no room for the COFF header.
        if let Some(s) = b.get_mut(E_LFANEW_OFF..E_LFANEW_OFF + 4) {
            s.copy_from_slice(&120u32.to_le_bytes());
        }
        assert!(matches!(inspect(&b), Err(PeError::BadPeOffset { .. })));
    }

    #[test]
    fn rejects_dos_stub_without_pe_image() {
        let mut b = vec![0u8; 256];
        if let Some(s) = b.get_mut(..2) {
            s.copy_from_slice(&MZ_MAGIC.to_le_bytes());
        }
        if let Some(s) = b.get_mut(E_LFANEW_OFF..E_LFANEW_OFF + 4) {
            s.copy_from_slice(&0x80u32.to_le_bytes());
        }
        // No "PE\0\0" written at 0x80 -- a genuine 16-bit DOS/Win3.x executable.
        assert!(matches!(inspect(&b), Err(PeError::NotPeImage)));
    }

    #[test]
    fn rejects_bad_optional_header() {
        let mut b = synth_pe(0x8664, 2, true, 0x0002, 0, 0);
        let opt = 0x80 + COFF_HEADER_LEN;
        // Nonsense magic.
        if let Some(s) = b.get_mut(opt..opt + 2) {
            s.copy_from_slice(&0xdeadu16.to_le_bytes());
        }
        assert!(matches!(
            inspect(&b),
            Err(PeError::UnknownOptionalMagic { magic: 0xdead })
        ));

        // SizeOfOptionalHeader = 0 means an object file.
        let mut b = synth_pe(0x8664, 2, true, 0x0002, 0, 0);
        if let Some(s) = b.get_mut(0x80 + COFF_SIZEOFOPT_OFF..0x80 + COFF_SIZEOFOPT_OFF + 2) {
            s.copy_from_slice(&0u16.to_le_bytes());
        }
        assert!(matches!(inspect(&b), Err(PeError::NoOptionalHeader)));
    }

    #[test]
    fn truncation_at_every_offset_never_panics() {
        // Property test standing in for the fuzzer: a valid image truncated at
        // every possible length must always produce Ok or Err, never a panic.
        // File managers call this on whatever the user clicked.
        let full = synth_pe(0x8664, 2, true, 0x0002, DLLCHAR_DYNAMIC_BASE, 0x2000);
        for cut in 0..full.len() {
            let slice = full.get(..cut).unwrap_or(&[]);
            let _ = inspect(slice);
        }
    }

    #[test]
    fn lying_size_of_optional_header_is_contained() {
        // SizeOfOptionalHeader claims a huge value the file does not contain.
        // We must not read past the buffer, and must not report confident
        // garbage read from adjacent memory.
        let mut b = synth_pe(0x8664, 2, true, 0x0002, 0, 0);
        if let Some(s) = b.get_mut(0x80 + COFF_SIZEOFOPT_OFF..0x80 + COFF_SIZEOFOPT_OFF + 2) {
            s.copy_from_slice(&u16::MAX.to_le_bytes());
        }
        let _ = inspect(&b); // must not panic
    }

    #[test]
    fn identifies_msi_containers() {
        let ole = [0xd0u8, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1, 0x00, 0x00];
        assert!(is_msi(&ole));
        assert!(!is_msi(b"MZ\x90\x00"));
        assert!(!is_msi(&[]));
        assert!(!is_msi(&[0xd0, 0xcf]));
    }
}
