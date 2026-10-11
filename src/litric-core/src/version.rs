//! Versioning and ABI compatibility.
//!
//! Two different things live here and they must not be confused:
//!
//! * [`Version`] — an application's version. Ordering decides "is this an
//!   upgrade?", which drives anti-downgrade enforcement in the update path.
//! * [`AbiVersion`] — the platform contract a binary was compiled against.
//!   Ordering is *not* the question; compatibility is. See
//!   `LITRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md` section 14.

use core::fmt;

/// Why a version string was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VersionError {
    /// Not three dot-separated numeric components.
    Malformed,
    /// A component was not a decimal number, or overflowed `u32`.
    BadComponent {
        /// Zero-based component position.
        index: usize,
    },
    /// The pre-release or build suffix contained an illegal byte.
    BadSuffix,
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => write!(f, "version must be major.minor.patch"),
            Self::BadComponent { index } => write!(f, "version component {index} is not a u32"),
            Self::BadSuffix => write!(f, "version suffix contains illegal characters"),
        }
    }
}

impl std::error::Error for VersionError {}

/// A `major.minor.patch` application version with an optional pre-release tag.
///
/// Ordering places any pre-release *below* the corresponding release, so
/// `1.2.0-beta.1 < 1.2.0`. That is what makes "never install something older
/// than what is committed" mean the right thing when betas are in play.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre: Option<String>,
}

impl Version {
    /// Construct directly.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
        }
    }

    /// Parse `1.2.3` or `1.2.3-beta.4`.
    ///
    /// # Errors
    /// [`VersionError`] describing which part failed.
    pub fn parse(raw: &str) -> Result<Self, VersionError> {
        let (core_part, pre) = match raw.split_once('-') {
            Some((c, p)) => {
                if p.is_empty()
                    || !p
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                {
                    return Err(VersionError::BadSuffix);
                }
                (c, Some(p.to_owned()))
            }
            None => (raw, None),
        };

        let mut parts = core_part.split('.');
        let mut nums = [0u32; 3];
        for (index, slot) in nums.iter_mut().enumerate() {
            let Some(text) = parts.next() else {
                return Err(VersionError::Malformed);
            };
            // Reject leading zeros and signs explicitly: "01" and "+1" both
            // parse as 1 under a naive parse, which would make two distinct
            // strings compare equal and defeat anti-downgrade bookkeeping.
            if text.is_empty()
                || (text.len() > 1 && text.starts_with('0'))
                || !text.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(VersionError::BadComponent { index });
            }
            *slot = text
                .parse::<u32>()
                .map_err(|_| VersionError::BadComponent { index })?;
        }
        if parts.next().is_some() {
            return Err(VersionError::Malformed);
        }

        Ok(Self {
            major: nums[0],
            minor: nums[1],
            patch: nums[2],
            pre,
        })
    }

    /// Major component.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }
    /// Minor component.
    #[must_use]
    pub const fn minor(&self) -> u32 {
        self.minor
    }
    /// Patch component.
    #[must_use]
    pub const fn patch(&self) -> u32 {
        self.patch
    }
    /// Pre-release tag, if any.
    #[must_use]
    pub fn pre_release(&self) -> Option<&str> {
        self.pre.as_deref()
    }
    /// Whether this is a pre-release.
    #[must_use]
    pub const fn is_pre_release(&self) -> bool {
        self.pre.is_some()
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
            Ordering::Equal => match (&self.pre, &other.pre) {
                // A release outranks its own pre-releases.
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            },
            other_ord => other_ord,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(p) = &self.pre {
            write!(f, "-{p}")?;
        }
        Ok(())
    }
}

/// The platform ABI a binary targets, e.g. `litric-native-v1`.
///
/// Compatibility rule, per RUNTIME section 14:
///
/// * **Major** differs → incompatible. A shim may exist; the loader must not
///   guess that one does.
/// * **Minor**: the platform may be newer than the binary (additive features),
///   never older. A binary built against minor 5 does not run on minor 3,
///   because it may call something that does not exist.
/// * **Patch**: always compatible in both directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbiVersion {
    major: u16,
    minor: u16,
}

impl AbiVersion {
    /// The ABI this build of the platform implements.
    pub const CURRENT: Self = Self { major: 1, minor: 0 };

    /// Construct.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Parse `litric-native-v1` or `litric-native-v1.3`.
    ///
    /// # Errors
    /// [`VersionError::Malformed`] if the prefix or the numbers are wrong.
    pub fn parse(raw: &str) -> Result<Self, VersionError> {
        const PREFIX: &str = "litric-native-v";
        let rest = raw.strip_prefix(PREFIX).ok_or(VersionError::Malformed)?;
        let (maj, min) = match rest.split_once('.') {
            Some((a, b)) => (a, b),
            None => (rest, "0"),
        };
        let major = maj
            .parse::<u16>()
            .map_err(|_| VersionError::BadComponent { index: 0 })?;
        let minor = min
            .parse::<u16>()
            .map_err(|_| VersionError::BadComponent { index: 1 })?;
        Ok(Self { major, minor })
    }

    /// Whether `self` (the running platform) can host a binary built for
    /// `target`.
    ///
    /// This is deliberately not `>=`. Asymmetry is the whole point: newer
    /// platform hosts older binary, never the reverse.
    #[must_use]
    pub const fn can_host(self, target: Self) -> bool {
        self.major == target.major && self.minor >= target.minor
    }

    /// Major component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }
    /// Minor component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

impl fmt::Display for AbiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "litric-native-v{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_orders_versions() {
        let a = Version::parse("1.2.3").expect("valid");
        assert_eq!((a.major(), a.minor(), a.patch()), (1, 2, 3));
        assert!(Version::parse("1.2.4").expect("v") > a);
        assert!(Version::parse("1.3.0").expect("v") > a);
        assert!(Version::parse("2.0.0").expect("v") > a);
        assert!(Version::parse("1.2.2").expect("v") < a);
    }

    #[test]
    fn pre_release_sorts_below_its_release() {
        // Anti-downgrade depends on this: committing 1.2.0 must make
        // 1.2.0-beta.9 an older version, not a newer one.
        let rel = Version::parse("1.2.0").expect("v");
        let beta = Version::parse("1.2.0-beta.9").expect("v");
        assert!(beta < rel);
        assert!(rel > beta);
        assert!(
            Version::parse("1.2.0-beta.2").expect("v") < Version::parse("1.2.0-beta.3").expect("v")
        );
    }

    #[test]
    fn rejects_ambiguous_numeric_forms() {
        // "01" and "1" must not both parse, or two distinct recorded versions
        // would compare equal and the monotonic counter would stop being
        // monotonic.
        assert!(Version::parse("01.2.3").is_err());
        assert!(Version::parse("1.02.3").is_err());
        assert!(Version::parse("+1.2.3").is_err());
        assert!(Version::parse("-1.2.3").is_err());
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("1.2.x").is_err());
        assert!(Version::parse("").is_err());
        assert!(Version::parse("1.2.3-").is_err());
        assert!(Version::parse("1.2.3-bad suffix").is_err());
    }

    #[test]
    fn version_round_trips_through_display() {
        for s in ["0.0.0", "1.2.3", "10.20.30", "1.0.0-rc.1", "4294967295.0.0"] {
            let v = Version::parse(s).expect("valid");
            assert_eq!(v.to_string(), s);
        }
    }

    #[test]
    fn abi_hosting_is_asymmetric() {
        let platform = AbiVersion::new(1, 5);
        // Newer platform hosts an older binary.
        assert!(platform.can_host(AbiVersion::new(1, 0)));
        assert!(platform.can_host(AbiVersion::new(1, 5)));
        // Older platform must refuse a newer binary -- it may call something
        // that does not exist yet.
        assert!(!platform.can_host(AbiVersion::new(1, 6)));
        // Major mismatch is never compatible, in either direction.
        assert!(!platform.can_host(AbiVersion::new(2, 0)));
        assert!(!AbiVersion::new(2, 0).can_host(AbiVersion::new(1, 9)));
    }

    #[test]
    fn abi_parses_both_spellings() {
        assert_eq!(
            AbiVersion::parse("litric-native-v1").expect("v"),
            AbiVersion::new(1, 0)
        );
        assert_eq!(
            AbiVersion::parse("litric-native-v2.7").expect("v"),
            AbiVersion::new(2, 7)
        );
        assert!(AbiVersion::parse("iltric-native-v1").is_err());
        assert!(AbiVersion::parse("v1").is_err());
        assert!(AbiVersion::parse("litric-native-vX").is_err());
    }
}
