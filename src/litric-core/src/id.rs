//! Application identity.
//!
//! An [`AppId`] is the primary key of the whole platform: the sandbox is keyed
//! on it, capability grants are keyed on it, storage namespaces are derived from
//! it, and the audit log is indexed by it. Everything downstream trusts that an
//! `AppId` value is well-formed, so **all validation happens here, once**.
//!
//! The threat this module defends against is identity confusion: if
//! `com.example.app` and `com.example.App` and `com.example.app.` could all
//! exist, a malicious publisher could register a near-identical id, be granted
//! the user's trust by mistake, or — worse — collide with another app's storage
//! namespace and read its data.

use core::fmt;

/// Why an application identifier was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdError {
    /// The identifier was empty.
    Empty,
    /// The identifier exceeded [`AppId::MAX_LEN`] bytes.
    TooLong {
        /// Actual length in bytes.
        len: usize,
    },
    /// Fewer than three dot-separated labels (`tld.org.app` is the minimum).
    TooFewLabels {
        /// Number of labels found.
        found: usize,
    },
    /// A label was empty, i.e. a leading, trailing or doubled dot.
    EmptyLabel {
        /// Zero-based label position.
        index: usize,
    },
    /// A label exceeded [`AppId::MAX_LABEL_LEN`] bytes.
    LabelTooLong {
        /// Zero-based label position.
        index: usize,
        /// Actual label length in bytes.
        len: usize,
    },
    /// A byte outside `[a-z0-9-]` (plus `.` as separator) appeared.
    IllegalCharacter {
        /// Byte offset within the identifier.
        index: usize,
        /// The offending byte.
        byte: u8,
    },
    /// A label started or ended with `-`.
    LabelHyphenEdge {
        /// Zero-based label position.
        index: usize,
    },
    /// A label started with a digit. Reserved so that ids never look numeric.
    LabelStartsWithDigit {
        /// Zero-based label position.
        index: usize,
    },
    /// The identifier collides with a reserved system namespace.
    Reserved,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "application id is empty"),
            Self::TooLong { len } => {
                write!(
                    f,
                    "application id is {len} bytes, maximum is {}",
                    AppId::MAX_LEN
                )
            }
            Self::TooFewLabels { found } => {
                write!(f, "application id has {found} labels, minimum is 3")
            }
            Self::EmptyLabel { index } => write!(f, "label {index} is empty"),
            Self::LabelTooLong { index, len } => write!(
                f,
                "label {index} is {len} bytes, maximum is {}",
                AppId::MAX_LABEL_LEN
            ),
            Self::IllegalCharacter { index, byte } => {
                write!(f, "illegal byte {byte:#04x} at offset {index}")
            }
            Self::LabelHyphenEdge { index } => {
                write!(f, "label {index} starts or ends with a hyphen")
            }
            Self::LabelStartsWithDigit { index } => {
                write!(f, "label {index} starts with a digit")
            }
            Self::Reserved => write!(f, "application id is in a reserved namespace"),
        }
    }
}

impl std::error::Error for IdError {}

/// A validated, canonical application identifier.
///
/// Canonical form is lowercase reverse-DNS with at least three labels, e.g.
/// `com.litric.files`. Construction is the only way to obtain one, so a value of
/// this type is a proof of validity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppId(String);

impl AppId {
    /// Maximum total length in bytes.
    pub const MAX_LEN: usize = 255;
    /// Maximum length of one dot-separated label.
    pub const MAX_LABEL_LEN: usize = 63;
    /// Minimum number of labels.
    pub const MIN_LABELS: usize = 3;

    /// Namespaces reserved for the platform itself.
    ///
    /// A third-party package claiming `os.litric.*` could impersonate a system
    /// component in the installer's permission dialog, which is the entire
    /// attack. The reservation is enforced here rather than in the store,
    /// because the store is not the trust root (ECOSYSTEM section 16).
    const RESERVED_PREFIXES: &'static [&'static str] = &["os.litric.", "sys.litric."];

    /// Validate and canonicalise an application identifier.
    ///
    /// Input is lowercased before validation, so `Com.Example.App` and
    /// `com.example.app` are the same identity and cannot both be registered.
    ///
    /// # Errors
    /// Returns the specific [`IdError`] describing the first rule violated, so
    /// that a developer sees which rule they broke rather than "invalid id".
    pub fn parse(raw: &str) -> Result<Self, IdError> {
        if raw.is_empty() {
            return Err(IdError::Empty);
        }
        if raw.len() > Self::MAX_LEN {
            return Err(IdError::TooLong { len: raw.len() });
        }

        let lowered = raw.to_ascii_lowercase();

        // Reject any non-ASCII byte before doing anything else. Unicode
        // identifiers invite homograph attacks (Cyrillic 'а' in "pаypal"), and
        // there is no legitimate need for them in a machine-readable id.
        for (i, b) in lowered.bytes().enumerate() {
            let ok = b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.';
            if !ok {
                return Err(IdError::IllegalCharacter { index: i, byte: b });
            }
        }

        let labels: Vec<&str> = lowered.split('.').collect();
        if labels.len() < Self::MIN_LABELS {
            return Err(IdError::TooFewLabels {
                found: labels.len(),
            });
        }

        for (index, label) in labels.iter().enumerate() {
            if label.is_empty() {
                return Err(IdError::EmptyLabel { index });
            }
            if label.len() > Self::MAX_LABEL_LEN {
                return Err(IdError::LabelTooLong {
                    index,
                    len: label.len(),
                });
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(IdError::LabelHyphenEdge { index });
            }
            if label.as_bytes().first().is_some_and(u8::is_ascii_digit) {
                return Err(IdError::LabelStartsWithDigit { index });
            }
        }

        if Self::RESERVED_PREFIXES
            .iter()
            .any(|p| lowered.starts_with(p))
        {
            return Err(IdError::Reserved);
        }

        Ok(Self(lowered))
    }

    /// Construct without the reserved-namespace check.
    ///
    /// Only the platform build may use this, to register its own components.
    /// It is deliberately verbose so that a call site is obvious in review.
    ///
    /// # Errors
    /// All checks except [`IdError::Reserved`] still apply.
    pub fn parse_platform_reserved(raw: &str) -> Result<Self, IdError> {
        match Self::parse(raw) {
            Err(IdError::Reserved) => Ok(Self(raw.to_ascii_lowercase())),
            other => other,
        }
    }

    /// The canonical string form.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this identifies a first-party platform component.
    #[must_use]
    pub fn is_platform(&self) -> bool {
        Self::RESERVED_PREFIXES
            .iter()
            .any(|p| self.0.starts_with(p))
    }

    /// A filesystem-safe directory name for this application.
    ///
    /// The canonical id is already restricted to `[a-z0-9.-]`, all of which are
    /// safe on every filesystem we target, so this is the identity function. It
    /// exists as a named method so that if the id grammar ever widens, every
    /// path-construction site changes in one place instead of silently gaining
    /// a directory-traversal bug.
    #[must_use]
    pub fn as_path_component(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_ids() {
        for good in [
            "com.example.app",
            "org.litric.files",
            "io.github.someone.some-tool",
            "net.a.b",
            "com.example.app.helper.background",
        ] {
            assert!(AppId::parse(good).is_ok(), "should accept {good}");
        }
    }

    #[test]
    fn canonicalises_case() {
        // The security property: case variants are the SAME identity, so a
        // lookalike cannot be registered alongside the real one.
        let a = AppId::parse("Com.Example.App").expect("valid");
        let b = AppId::parse("com.example.app").expect("valid");
        assert_eq!(a, b);
        assert_eq!(a.as_str(), "com.example.app");
    }

    #[test]
    fn rejects_structural_nonsense() {
        assert_eq!(AppId::parse(""), Err(IdError::Empty));
        assert!(matches!(
            AppId::parse("com.example"),
            Err(IdError::TooFewLabels { found: 2 })
        ));
        assert!(matches!(
            AppId::parse("app"),
            Err(IdError::TooFewLabels { found: 1 })
        ));
        assert!(matches!(
            AppId::parse(".com.example.app"),
            Err(IdError::EmptyLabel { index: 0 })
        ));
        assert!(matches!(
            AppId::parse("com.example.app."),
            Err(IdError::EmptyLabel { .. })
        ));
        assert!(matches!(
            AppId::parse("com..app.x"),
            Err(IdError::EmptyLabel { index: 1 })
        ));
    }

    #[test]
    fn rejects_homograph_and_traversal_attempts() {
        // These are the inputs an attacker actually sends.
        for hostile in [
            "com.example.app/../../etc/passwd", // path traversal
            "com.example.app\0evil",            // NUL truncation
            "com.exаmple.app",                  // Cyrillic 'а' homograph
            "com.example.app ",                 // trailing space
            "com/example/app",                  // separator confusion
            "com.example.app%2e%2e",            // percent-encoded traversal
            "../../../root",                    // bare traversal
            "com.example.app\n",                // log injection
        ] {
            assert!(
                AppId::parse(hostile).is_err(),
                "must reject hostile input: {hostile:?}"
            );
        }
    }

    #[test]
    fn rejects_hyphen_edges_and_leading_digits() {
        assert!(matches!(
            AppId::parse("com.-bad.app"),
            Err(IdError::LabelHyphenEdge { index: 1 })
        ));
        assert!(matches!(
            AppId::parse("com.bad-.app"),
            Err(IdError::LabelHyphenEdge { index: 1 })
        ));
        assert!(matches!(
            AppId::parse("com.1bad.app"),
            Err(IdError::LabelStartsWithDigit { index: 1 })
        ));
    }

    #[test]
    fn enforces_length_limits() {
        let long_label = "a".repeat(AppId::MAX_LABEL_LEN + 1);
        assert!(matches!(
            AppId::parse(&format!("com.{long_label}.app")),
            Err(IdError::LabelTooLong { index: 1, .. })
        ));

        let huge = format!("com.{}.app", "a".repeat(AppId::MAX_LEN));
        assert!(matches!(AppId::parse(&huge), Err(IdError::TooLong { .. })));
    }

    #[test]
    fn reserves_platform_namespace_from_third_parties() {
        // A third-party package must not be able to look like a system component
        // in the install dialog.
        assert_eq!(AppId::parse("os.litric.shell"), Err(IdError::Reserved));
        assert_eq!(AppId::parse("sys.litric.capd"), Err(IdError::Reserved));
        assert_eq!(AppId::parse("OS.LITRIC.SHELL"), Err(IdError::Reserved));

        // ...but the platform itself can register them, explicitly.
        let sys = AppId::parse_platform_reserved("os.litric.shell").expect("platform id");
        assert!(sys.is_platform());
        assert_eq!(sys.as_str(), "os.litric.shell");
    }

    #[test]
    fn near_miss_namespaces_are_not_reserved() {
        // "os.litricos.x" is not in the reserved prefix, and must not be
        // accidentally captured by a sloppy prefix check.
        assert!(AppId::parse("os.litricos.thing").is_ok());
        assert!(AppId::parse("com.litric.thing").is_ok());
        assert!(!AppId::parse("com.litric.thing")
            .expect("valid")
            .is_platform());
    }

    #[test]
    fn path_component_never_escapes() {
        // Property test: for every id we accept, the derived path component
        // contains no separator and no traversal sequence.
        for good in ["com.example.app", "io.github.a.b-c", "net.x.y.z"] {
            let id = AppId::parse(good).expect("valid");
            let p = id.as_path_component();
            assert!(!p.contains('/'), "{p} contains /");
            assert!(!p.contains('\\'), "{p} contains backslash");
            assert!(!p.contains(".."), "{p} contains ..");
            assert!(!p.is_empty());
        }
    }
}
