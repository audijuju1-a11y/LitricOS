//! Litric Files: confined file operations, the freedesktop.org trash on disk,
//! and archives.
//!
//! # Layer position
//!
//! **Layer 4 (Runtime).** It performs the I/O that `litric-storage` (names) and
//! `litric-trash` (what deleting means) deliberately do not, and it is served
//! to the shell by `litric-filesd` over LFP/1 (`docs/adr/0019-litric-files.md`).
//!
//! # Map
//!
//! ```text
//!   path      logical paths ("/Documents/a.txt"), name-collision rules    any host
//!   proto     LFP/1 request parsing and reply encoding                     any host
//!   crc32 inflate deflate gzip zip tar   codecs written from the specs     any host
//!   ops       Root: resolution without following symlinks, file ops       Unix only
//!   trash     the freedesktop.org home trash on disk                      Unix only
//!   archive   COMPRESS / EXTRACT with the ADR-0019 limits                  Unix only
//!   server    LFP/1 over any Read/Write, and a Unix-socket listener        Unix only
//! ```
//!
//! On a host that is not Unix the codecs and the protocol still build and
//! test, and [`Root::open`] returns an `io` error saying the platform is not
//! supported. Nothing pretends to work there: the confinement rules depend on
//! Unix symlink and permission semantics that this crate has only been
//! written against.

#![forbid(unsafe_code)]

use core::fmt;
use std::io;

pub mod crc32;
pub mod deflate;
pub mod gzip;
pub mod inflate;
pub mod path;
pub mod proto;
pub mod tar;
pub mod zip;

#[cfg(unix)]
pub mod archive;
#[cfg(unix)]
pub mod ops;
#[cfg(unix)]
pub mod server;
#[cfg(unix)]
pub mod trash;

#[cfg(unix)]
pub use ops::Root;

/// The error kinds of LFP/1, one per `ERR <kind>` token in ADR-0019.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The request line is malformed.
    BadRequest,
    /// A path or name breaks the naming rules.
    InvalidName,
    /// Nothing exists at that location.
    NotFound,
    /// Something already exists where a new item would go.
    Exists,
    /// A directory was required.
    NotADirectory,
    /// A directory was given where it cannot be used.
    IsADirectory,
    /// A symbolic link was met where following it would be required.
    SymlinkRefused,
    /// The operating system refused access.
    PermissionDenied,
    /// The item is too large for the operation (trash capacity, ZIP without
    /// ZIP64).
    TooLarge,
    /// The archive uses a format or feature that is not implemented.
    UnsupportedArchive,
    /// The archive is damaged or does not follow its specification.
    CorruptArchive,
    /// An archive entry would write outside its destination or create
    /// something other than a regular file or directory.
    UnsafeArchiveEntry,
    /// A resource limit was reached.
    Limit,
    /// Restoring from the trash would replace an existing item.
    RestoreWouldOverwrite,
    /// Any other I/O failure.
    Io,
}

impl ErrorKind {
    /// Every kind, in the order ADR-0019 lists them.
    pub const ALL: [Self; 15] = [
        Self::BadRequest,
        Self::InvalidName,
        Self::NotFound,
        Self::Exists,
        Self::NotADirectory,
        Self::IsADirectory,
        Self::SymlinkRefused,
        Self::PermissionDenied,
        Self::TooLarge,
        Self::UnsupportedArchive,
        Self::CorruptArchive,
        Self::UnsafeArchiveEntry,
        Self::Limit,
        Self::RestoreWouldOverwrite,
        Self::Io,
    ];

    /// The wire token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::BadRequest => "bad-request",
            Self::InvalidName => "invalid-name",
            Self::NotFound => "not-found",
            Self::Exists => "exists",
            Self::NotADirectory => "not-a-directory",
            Self::IsADirectory => "is-a-directory",
            Self::SymlinkRefused => "symlink-refused",
            Self::PermissionDenied => "permission-denied",
            Self::TooLarge => "too-large",
            Self::UnsupportedArchive => "unsupported-archive",
            Self::CorruptArchive => "corrupt-archive",
            Self::UnsafeArchiveEntry => "unsafe-archive-entry",
            Self::Limit => "limit",
            Self::RestoreWouldOverwrite => "restore-would-overwrite",
            Self::Io => "io",
        }
    }
}

/// An error with an LFP/1 kind and a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    /// An error of `kind`.
    #[must_use]
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Classify an I/O error.
    ///
    /// An `io::Error` that carries an [`Error`] (the codecs report through
    /// `Read` that way) is returned as that error unchanged; otherwise the OS
    /// error kind decides and `context` is prefixed to the message.
    #[must_use]
    pub fn from_io(context: &str, err: &io::Error) -> Self {
        if let Some(inner) = err.get_ref().and_then(|e| e.downcast_ref::<Self>()) {
            return inner.clone();
        }
        let kind = match err.kind() {
            io::ErrorKind::NotFound => ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => ErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => ErrorKind::Exists,
            io::ErrorKind::NotADirectory => ErrorKind::NotADirectory,
            io::ErrorKind::IsADirectory => ErrorKind::IsADirectory,
            io::ErrorKind::FileTooLarge => ErrorKind::TooLarge,
            // `io::ErrorKind::InvalidFilename` is the kind Windows' filesystem
            // returns for a name it rejects, and it is the one kind this table
            // needs that is **not stable on the workspace's declared MSRV of
            // 1.85**: naming it there is `error[E0658]`, which turns
            // `cargo build` red on a 1.85 toolchain even though it compiles on
            // current stable. The mapping is preserved by the raw OS code
            // instead, so the behaviour is identical on the platform that
            // produces it and the crate builds on every toolchain in range.
            _ if is_invalid_filename(err) => ErrorKind::InvalidName,
            _ => ErrorKind::Io,
        };
        Self::new(kind, format!("{context}: {err}"))
    }

    /// Wrap into an `io::Error` so a `Read` implementation can carry it;
    /// [`Error::from_io`] recovers it.
    #[must_use]
    pub fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self)
    }
}

/// Whether an I/O error is Windows' "the name is not valid" (`ERROR_INVALID_NAME`).
///
/// Windows is the only platform that produces `io::ErrorKind::InvalidFilename`,
/// and the raw code is used because naming that variant is a compile error on
/// the workspace's MSRV (see [`Error::from_io`]). On every other platform this
/// is always `false`, which is what the `io::ErrorKind` variant would have
/// meant there too.
#[must_use]
fn is_invalid_filename(err: &io::Error) -> bool {
    // winerror.h: `#define ERROR_INVALID_NAME 123L`.
    cfg!(windows) && err.raw_os_error() == Some(123)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.token(), self.message)
    }
}

impl std::error::Error for Error {}

/// What to do when an item of the same name already exists in the
/// destination of a copy or move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Leave the existing item and do not transfer this one.
    Skip,
    /// Replace the existing item, but only once the new one is complete.
    Replace,
    /// Transfer under a new name, `name (2).ext`, `name (3).ext`, ...
    KeepBoth,
}

impl Policy {
    /// Parse the wire token `skip`, `replace` or `keep-both`.
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "skip" => Some(Self::Skip),
            "replace" => Some(Self::Replace),
            "keep-both" => Some(Self::KeepBoth),
            _ => None,
        }
    }

    /// The wire token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Replace => "replace",
            Self::KeepBoth => "keep-both",
        }
    }
}

/// Resource limits for reading archives (ADR-0019 "Extraction limits").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Most entries (files and directories) one archive may contain.
    pub max_entries: u64,
    /// Most bytes one extraction may write in total.
    pub max_output: u64,
    /// Highest compression ratio accepted for a large entry, as `n` in `n:1`.
    pub max_ratio: u64,
    /// Entries at or below this many uncompressed bytes are exempt from the
    /// ratio rule.
    pub ratio_exempt_size: u64,
    /// Most bytes of entry names and central-directory records held in
    /// memory while validating an archive before anything is written.
    pub max_metadata: u64,
}

impl Limits {
    /// The limits ADR-0019 sets: 100 000 entries, 4 GiB of output, 1000:1 on
    /// entries larger than 1 MiB, and 64 MiB of entry metadata.
    pub const ADR_0019: Self = Self {
        max_entries: 100_000,
        max_output: 4 * 1024 * 1024 * 1024,
        max_ratio: 1000,
        ratio_exempt_size: 1024 * 1024,
        max_metadata: 64 * 1024 * 1024,
    };

    /// Whether `uncompressed` bytes from `compressed` bytes breaks the ratio
    /// rule.
    #[must_use]
    pub const fn ratio_exceeded(&self, uncompressed: u64, compressed: u64) -> bool {
        uncompressed > self.ratio_exempt_size
            && uncompressed > compressed.saturating_mul(self.max_ratio)
    }
}

#[cfg(not(unix))]
mod unsupported {
    /// File operations are not implemented on this host. The type has no
    /// values; [`Root::open`] always fails.
    #[derive(Debug)]
    pub enum Root {}

    impl Root {
        /// Always fails with an `io` error naming the missing platform
        /// support.
        ///
        /// # Errors
        /// Always.
        pub fn open(
            host: &std::path::Path,
            zone: litric_locale::TimeZone,
        ) -> Result<Self, crate::Error> {
            let _ = (host, zone);
            Err(crate::Error::new(
                crate::ErrorKind::Io,
                "litric-files performs file operations on Unix hosts only; \
                 this build has no implementation for this platform",
            ))
        }
    }
}

#[cfg(not(unix))]
pub use unsupported::Root;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_kind_has_the_adr_token() {
        let tokens: Vec<&str> = ErrorKind::ALL.iter().map(|k| k.token()).collect();
        assert_eq!(
            tokens,
            vec![
                "bad-request",
                "invalid-name",
                "not-found",
                "exists",
                "not-a-directory",
                "is-a-directory",
                "symlink-refused",
                "permission-denied",
                "too-large",
                "unsupported-archive",
                "corrupt-archive",
                "unsafe-archive-entry",
                "limit",
                "restore-would-overwrite",
                "io",
            ]
        );
    }

    #[test]
    fn an_error_survives_a_trip_through_io_error() {
        let e = Error::new(ErrorKind::CorruptArchive, "bad block type");
        let io_err = e.clone().into_io();
        assert_eq!(Error::from_io("ctx", &io_err), e);
    }

    #[test]
    fn os_errors_map_to_their_kinds() {
        let nf = io::Error::from(io::ErrorKind::NotFound);
        assert_eq!(Error::from_io("x", &nf).kind(), ErrorKind::NotFound);
        let ex = io::Error::from(io::ErrorKind::AlreadyExists);
        assert_eq!(Error::from_io("x", &ex).kind(), ErrorKind::Exists);
        let other = io::Error::other("boom");
        assert_eq!(Error::from_io("x", &other).kind(), ErrorKind::Io);
    }

    #[test]
    fn the_invalid_name_mapping_keys_off_the_raw_os_code() {
        // `io::ErrorKind::InvalidFilename` is unstable at the declared MSRV, so
        // the arm cannot name it and matches the raw code instead. Windows is
        // the only platform where that code means a filename error.
        let name = io::Error::from_raw_os_error(123);
        assert_eq!(is_invalid_filename(&name), cfg!(windows));
        assert!(!is_invalid_filename(&io::Error::from_raw_os_error(2)));
    }

    #[test]
    fn policies_round_trip() {
        for p in [Policy::Skip, Policy::Replace, Policy::KeepBoth] {
            assert_eq!(Policy::parse(p.token()), Some(p));
        }
        assert_eq!(Policy::parse("Replace"), None);
    }

    #[test]
    fn the_ratio_rule_exempts_small_entries_only() {
        let l = Limits::ADR_0019;
        assert!(!l.ratio_exceeded(1024 * 1024, 1), "1 MiB is exempt");
        assert!(l.ratio_exceeded(1024 * 1024 + 1, 1));
        assert!(
            !l.ratio_exceeded(2_000_000, 2000),
            "exactly 1000:1 is allowed"
        );
        assert!(l.ratio_exceeded(2_000_001, 2000));
        assert!(l.ratio_exceeded(2_000_000, 0));
    }
}
