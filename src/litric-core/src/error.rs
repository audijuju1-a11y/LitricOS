//! The LitricOS error taxonomy.
//!
//! Implements the categories required by `LITRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md`
//! section 26. The rule that shapes this module: **do not collapse materially
//! different failures into one generic state when callers need to make different
//! decisions.**
//!
//! A caller asking "should I retry?" or "should I prompt the user for
//! permission?" must be able to answer from the type, never from message text.

use core::fmt;

/// What kind of failure occurred.
///
/// The numeric discriminants are part of the ABI: they cross the IPC boundary
/// and appear in compiled applications. **Never renumber an existing variant.**
/// New variants are appended, and callers must treat an unknown discriminant as
/// [`ErrorKind::Internal`] rather than panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The request was malformed. Retrying it unchanged will fail again.
    InvalidInput = 1,
    /// The caller lacks a required capability. Recoverable by asking the user.
    PermissionDenied = 2,
    /// The named resource does not exist.
    NotFound = 3,
    /// A quota or a system limit was hit. May succeed later.
    ResourceExhausted = 4,
    /// The operation did not complete in its deadline. May succeed on retry.
    Timeout = 5,
    /// The operation was cancelled by the caller or by the system.
    Canceled = 6,
    /// The device exists but is not usable right now (unplugged, asleep, busy).
    DeviceUnavailable = 7,
    /// Client and service could not agree on a protocol or ABI version.
    VersionMismatch = 8,
    /// A hash, signature or verity check failed. **Never retry blindly.**
    IntegrityFailure = 9,
    /// The caller could not be authenticated.
    AuthenticationFailure = 10,
    /// A bug on our side. The caller can do nothing except report it.
    Internal = 11,
    /// Persistent state is inconsistent and needs repair or rollback.
    CorruptState = 12,
    /// The operation is well-formed but this build does not implement it.
    Unsupported = 13,
    /// A precondition on the target object was not met (wrong lifecycle state).
    FailedPrecondition = 14,
    /// The resource already exists and the caller asked for exclusive creation.
    AlreadyExists = 15,
}

impl ErrorKind {
    /// Whether retrying the identical request could plausibly succeed.
    ///
    /// This drives automatic retry in the SDK, so getting it wrong either hangs
    /// clients in a retry loop or gives up on transient failures. When in doubt
    /// the answer is `false`: a missed retry is a slow path, an unwarranted
    /// retry on a non-idempotent operation is data loss.
    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::ResourceExhausted | Self::Timeout | Self::DeviceUnavailable
        )
    }

    /// Whether a user-visible permission prompt could resolve this.
    #[must_use]
    pub const fn is_permission_recoverable(self) -> bool {
        matches!(self, Self::PermissionDenied)
    }

    /// Whether this indicates a security-relevant event that must be audited
    /// even when the caller handles it gracefully.
    ///
    /// Integrity and authentication failures are logged unconditionally: they
    /// are the observable half of an attack, and an attacker who can make the
    /// system fail quietly has already won half the engagement.
    #[must_use]
    pub const fn is_security_event(self) -> bool {
        matches!(
            self,
            Self::IntegrityFailure | Self::AuthenticationFailure | Self::PermissionDenied
        )
    }

    /// Stable machine-readable token. Safe to log, safe to match on, and
    /// deliberately **not** a translated human string.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::NotFound => "NOT_FOUND",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::Timeout => "TIMEOUT",
            Self::Canceled => "CANCELED",
            Self::DeviceUnavailable => "DEVICE_UNAVAILABLE",
            Self::VersionMismatch => "VERSION_MISMATCH",
            Self::IntegrityFailure => "INTEGRITY_FAILURE",
            Self::AuthenticationFailure => "AUTHENTICATION_FAILURE",
            Self::Internal => "INTERNAL_ERROR",
            Self::CorruptState => "CORRUPT_STATE",
            Self::Unsupported => "UNSUPPORTED",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::AlreadyExists => "ALREADY_EXISTS",
        }
    }

    /// Reconstruct from the wire discriminant.
    ///
    /// Unknown values map to [`ErrorKind::Internal`]. A newer service talking to
    /// an older client must degrade, not crash it — that is what makes additive
    /// ABI evolution safe (RUNTIME section 8.3).
    #[must_use]
    pub const fn from_wire(v: u16) -> Self {
        match v {
            1 => Self::InvalidInput,
            2 => Self::PermissionDenied,
            3 => Self::NotFound,
            4 => Self::ResourceExhausted,
            5 => Self::Timeout,
            6 => Self::Canceled,
            7 => Self::DeviceUnavailable,
            8 => Self::VersionMismatch,
            9 => Self::IntegrityFailure,
            10 => Self::AuthenticationFailure,
            12 => Self::CorruptState,
            13 => Self::Unsupported,
            14 => Self::FailedPrecondition,
            15 => Self::AlreadyExists,
            _ => Self::Internal,
        }
    }

    /// The wire discriminant.
    #[must_use]
    pub const fn to_wire(self) -> u16 {
        self as u16
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// A structured LitricOS error.
///
/// `domain` names the subsystem that produced it (`"caps"`, `"storage"`,
/// `"winbridge"`), so a token like `PERMISSION_DENIED` is unambiguous across
/// services. `detail` is a diagnostic aid for humans and logs; **it is never
/// part of the contract** and callers must not branch on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    domain: &'static str,
    detail: Option<String>,
}

impl Error {
    /// Construct an error with no free-text detail.
    #[must_use]
    pub const fn new(domain: &'static str, kind: ErrorKind) -> Self {
        Self {
            kind,
            domain,
            detail: None,
        }
    }

    /// Construct an error carrying a diagnostic string.
    ///
    /// The detail must never contain a secret, a full filesystem path outside
    /// the caller's own namespace, or the contents of user data — errors are
    /// logged, and logs are read by people who are not the data's owner.
    #[must_use]
    pub fn with_detail(domain: &'static str, kind: ErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            domain,
            detail: Some(detail.into()),
        }
    }

    /// The failure category.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The subsystem that produced this error.
    #[must_use]
    pub const fn domain(&self) -> &'static str {
        self.domain
    }

    /// The diagnostic detail, if any. Not part of the contract.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// See [`ErrorKind::is_retryable`].
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        self.kind.is_retryable()
    }

    /// See [`ErrorKind::is_security_event`].
    #[must_use]
    pub const fn is_security_event(&self) -> bool {
        self.kind.is_security_event()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(d) => write!(f, "{}: {}: {}", self.domain, self.kind.token(), d),
            None => write!(f, "{}: {}", self.domain, self.kind.token()),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_discriminants_round_trip() {
        // Every variant must survive a round trip through the wire form.
        // This is the ABI test that catches an accidental renumbering.
        let all = [
            ErrorKind::InvalidInput,
            ErrorKind::PermissionDenied,
            ErrorKind::NotFound,
            ErrorKind::ResourceExhausted,
            ErrorKind::Timeout,
            ErrorKind::Canceled,
            ErrorKind::DeviceUnavailable,
            ErrorKind::VersionMismatch,
            ErrorKind::IntegrityFailure,
            ErrorKind::AuthenticationFailure,
            ErrorKind::Internal,
            ErrorKind::CorruptState,
            ErrorKind::Unsupported,
            ErrorKind::FailedPrecondition,
            ErrorKind::AlreadyExists,
        ];
        for k in all {
            assert_eq!(
                ErrorKind::from_wire(k.to_wire()),
                k,
                "round trip failed for {k}"
            );
        }
    }

    #[test]
    fn abi_discriminants_are_frozen() {
        // If this test fails, an existing variant was renumbered, which silently
        // changes the meaning of errors returned to already-compiled binaries.
        // Fix the code, never this test.
        assert_eq!(ErrorKind::InvalidInput.to_wire(), 1);
        assert_eq!(ErrorKind::PermissionDenied.to_wire(), 2);
        assert_eq!(ErrorKind::NotFound.to_wire(), 3);
        assert_eq!(ErrorKind::IntegrityFailure.to_wire(), 9);
        assert_eq!(ErrorKind::Unsupported.to_wire(), 13);
        assert_eq!(ErrorKind::AlreadyExists.to_wire(), 15);
    }

    #[test]
    fn unknown_wire_value_degrades_instead_of_panicking() {
        // A future service returning error 9999 must not take down an old client.
        assert_eq!(ErrorKind::from_wire(9999), ErrorKind::Internal);
        assert_eq!(ErrorKind::from_wire(0), ErrorKind::Internal);
        assert_eq!(ErrorKind::from_wire(u16::MAX), ErrorKind::Internal);
    }

    #[test]
    fn integrity_failure_is_never_retryable() {
        // Retrying a failed signature check is how you turn a detection into a
        // brute-force oracle. It must always be a hard stop.
        assert!(!ErrorKind::IntegrityFailure.is_retryable());
        assert!(!ErrorKind::AuthenticationFailure.is_retryable());
        assert!(!ErrorKind::PermissionDenied.is_retryable());
        assert!(!ErrorKind::InvalidInput.is_retryable());
    }

    #[test]
    fn transient_failures_are_retryable() {
        assert!(ErrorKind::Timeout.is_retryable());
        assert!(ErrorKind::ResourceExhausted.is_retryable());
        assert!(ErrorKind::DeviceUnavailable.is_retryable());
    }

    #[test]
    fn security_events_are_flagged_for_audit() {
        assert!(ErrorKind::PermissionDenied.is_security_event());
        assert!(ErrorKind::IntegrityFailure.is_security_event());
        assert!(ErrorKind::AuthenticationFailure.is_security_event());
        assert!(!ErrorKind::NotFound.is_security_event());
        assert!(!ErrorKind::Timeout.is_security_event());
    }

    #[test]
    fn display_is_stable_and_machine_readable() {
        let e = Error::new("caps", ErrorKind::PermissionDenied);
        assert_eq!(e.to_string(), "caps: PERMISSION_DENIED");
        let e = Error::with_detail("storage", ErrorKind::NotFound, "user://Documents");
        assert_eq!(e.to_string(), "storage: NOT_FOUND: user://Documents");
    }
}
