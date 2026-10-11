//! Logical paths (`/Documents/report.odt`), validated against the ADR-0019
//! segment rules.
//!
//! The shell never sends a host path; every path is relative to the user
//! root. Validation is total: a path is either valid or rejected, never
//! "valid with surprises". The rules exist so that a validated path cannot
//! traverse, cannot smuggle control bytes through the protocol, and cannot
//! exceed filesystem limits.

use crate::{Error, ErrorKind};
use std::fmt;

/// Longest one segment (file or directory name) may be, in bytes.
pub const MAX_SEGMENT: usize = 255;
/// Longest a whole path may be, in bytes.
pub const MAX_PATH: usize = 4096;

/// Why a logical path was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The path was empty, or not rooted at `/`.
    /// `text` echoes the offending input for the error reply.
    NotRooted {
        /// The rejected input.
        text: String,
    },
    /// A segment was empty, `.`, `..`, or contained a separator.
    BadSegment {
        /// The rejected segment.
        segment: String,
    },
    /// A segment contained a NUL or other control byte.
    ControlByte {
        /// The rejected segment.
        segment: String,
    },
    /// A segment exceeded [`MAX_SEGMENT`] bytes.
    SegmentTooLong {
        /// The rejected segment.
        segment: String,
    },
    /// The whole path exceeded [`MAX_PATH`] bytes.
    PathTooLong {
        /// The rejected input's length.
        len: usize,
    },
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRooted { text } => write!(f, "path must start with `/`: {text:?}"),
            Self::BadSegment { segment } => write!(f, "invalid path segment: {segment:?}"),
            Self::ControlByte { segment } => write!(f, "control byte in segment: {segment:?}"),
            Self::SegmentTooLong { segment } => {
                write!(
                    f,
                    "segment exceeds {MAX_SEGMENT} bytes: {}...",
                    &segment[..40]
                )
            }
            Self::PathTooLong { len } => write!(f, "path is {len} bytes, limit is {MAX_PATH}"),
        }
    }
}

impl std::error::Error for PathError {}

/// A validated logical path. Construct only through [`LogicalPath::parse`],
/// so every `LogicalPath` in the system obeys the segment rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalPath {
    /// The normalized path text, always starting with `/`, no trailing
    /// slash (except the root itself).
    text: String,
}

impl LogicalPath {
    /// The user root, `/`.
    #[must_use]
    pub fn root() -> Self {
        Self {
            text: "/".to_string(),
        }
    }

    /// Parse and validate. The result is normalized: repeated slashes
    /// collapse, a trailing slash is dropped.
    ///
    /// # Errors
    /// [`PathError`] with the specific rule that failed.
    pub fn parse(text: &str) -> Result<Self, PathError> {
        if text.len() > MAX_PATH {
            return Err(PathError::PathTooLong { len: text.len() });
        }
        if !text.starts_with('/') {
            return Err(PathError::NotRooted {
                text: text.to_string(),
            });
        }
        let mut out = String::with_capacity(text.len());
        for segment in text.split('/') {
            if segment.is_empty() {
                // Leading `/`, trailing `/` and `//` all land here.
                continue;
            }
            validate_segment(segment)?;
            // A separator before every segment; the leading `/` of the
            // output comes from the first one. Empty branches collapse
            // (clippy: identical blocks).
            out.push('/');
            out.push_str(segment);
        }
        if out.is_empty() {
            out.push('/');
        }
        Ok(Self { text: out })
    }

    /// Join a validated segment onto this path.
    ///
    /// # Errors
    /// [`PathError`] if the joined result breaks any rule.
    pub fn join(&self, segment: &str) -> Result<Self, PathError> {
        validate_segment(segment)?;
        let joined = if self.text == "/" {
            format!("/{segment}")
        } else {
            format!("{}/{segment}", self.text)
        };
        Self::parse(&joined).map_err(|_| PathError::PathTooLong { len: joined.len() })
    }

    /// The validated path text.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The final segment, or `""` for the root.
    /// Slice invariant: `pos + 1 <= text.len()` always holds for a rfind
    /// hit, so the returned slice is inside the string.
    #[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
    pub fn file_name(&self) -> &str {
        match self.text.rfind('/') {
            Some(pos) if pos + 1 < self.text.len() => &self.text[pos + 1..],
            _ => "",
        }
    }

    /// The parent path, or `None` for the root.
    pub fn parent(&self) -> Option<Self> {
        if self.text == "/" {
            return None;
        }
        match self.text.rfind('/') {
            Some(0) => Some(Self::root()),
            Some(pos) => Some(Self {
                text: self.text[..pos].to_string(),
            }),
            None => None,
        }
    }

    /// The segments, root excluded (so the root yields an empty iterator).
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.text.split('/').filter(|s| !s.is_empty())
    }
}

impl fmt::Display for LogicalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Validate one segment against the ADR-0019 rules.
fn validate_segment(segment: &str) -> Result<(), PathError> {
    if segment.is_empty() {
        return Err(PathError::BadSegment {
            segment: segment.to_string(),
        });
    }
    if segment == "." || segment == ".." || segment.contains('/') {
        return Err(PathError::BadSegment {
            segment: segment.to_string(),
        });
    }
    if segment.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err(PathError::ControlByte {
            segment: segment.to_string(),
        });
    }
    if segment.len() > MAX_SEGMENT {
        return Err(PathError::SegmentTooLong {
            segment: segment.to_string(),
        });
    }
    Ok(())
}

impl From<PathError> for Error {
    fn from(e: PathError) -> Self {
        Error::new(ErrorKind::InvalidName, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    // Slices below are bounded by the arrays they read; lengths are
    // asserted by the round-trip equality that follows each use.
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

    use super::*;

    #[test]
    fn parses_and_normalizes() {
        assert_eq!(
            LogicalPath::parse("/Documents/report.odt")
                .expect("valid")
                .as_str(),
            "/Documents/report.odt"
        );
        // Repeated and trailing slashes collapse.
        assert_eq!(
            LogicalPath::parse("//Documents//report.odt/")
                .expect("valid")
                .as_str(),
            "/Documents/report.odt"
        );
        assert_eq!(LogicalPath::parse("/").expect("valid").as_str(), "/");
        assert!(LogicalPath::parse("").is_err(), "not rooted");
        assert!(LogicalPath::parse("Documents").is_err(), "not rooted");
    }

    #[test]
    fn rejects_dot_segments_and_traversal() {
        assert!(matches!(
            LogicalPath::parse("/a/../etc/passwd"),
            Err(PathError::BadSegment { .. })
        ));
        assert!(matches!(
            LogicalPath::parse("/a/./b"),
            Err(PathError::BadSegment { .. })
        ));
    }

    #[test]
    fn rejects_control_bytes_and_long_segments() {
        assert!(matches!(
            LogicalPath::parse("/a\u{0}b"),
            Err(PathError::ControlByte { .. })
        ));
        assert!(matches!(
            LogicalPath::parse("/a\u{1f}b"),
            Err(PathError::ControlByte { .. })
        ));
        let long = "x".repeat(MAX_SEGMENT + 1);
        assert!(matches!(
            LogicalPath::parse(&format!("/{long}")),
            Err(PathError::SegmentTooLong { .. })
        ));
    }

    #[test]
    fn rejects_overlong_paths() {
        let seg = format!("/{}", "x".repeat(255));
        let long = seg.repeat(17);
        assert!(matches!(
            LogicalPath::parse(&long),
            Err(PathError::PathTooLong { .. })
        ));
    }

    #[test]
    fn parent_and_filename() {
        let p = LogicalPath::parse("/Documents/report.odt").expect("valid");
        assert_eq!(p.file_name(), "report.odt");
        assert_eq!(p.parent().expect("has parent").as_str(), "/Documents");
        let root = LogicalPath::root();
        assert_eq!(root.file_name(), "");
        assert_eq!(root.parent(), None);
        let deep = LogicalPath::parse("/a/b").expect("valid");
        assert_eq!(deep.parent().expect("b").as_str(), "/a");
        assert_eq!(
            deep.parent().and_then(|p| p.parent()).expect("a").as_str(),
            "/"
        );
    }

    #[test]
    fn segments_enumerate() {
        let p = LogicalPath::parse("/a/b/c").expect("valid");
        assert_eq!(p.segments().collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert_eq!(LogicalPath::root().segments().count(), 0);
    }

    #[test]
    fn join_validates() {
        let d = LogicalPath::parse("/Documents").expect("valid");
        assert_eq!(
            d.join("new.txt").expect("valid").as_str(),
            "/Documents/new.txt"
        );
        assert!(d.join("..").is_err());
        assert!(d.join("a/b").is_err());
        assert!(d.join("").is_err());
    }

    /// The error converts to the LFP/1 invalid-name kind.
    #[test]
    fn converts_to_error_kind() {
        let e: Error = LogicalPath::parse("nope").expect_err("not rooted").into();
        assert_eq!(e.kind(), ErrorKind::InvalidName);
    }
}
