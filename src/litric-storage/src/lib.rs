//! LitricOS storage namespace broker.
//!
//! # Layer position
//!
//! **Layer 3 (System API).** Depends downward on `litric-core` and `litric-caps`.
//! Performs no I/O: it decides *which real path a logical URI means* and
//! *whether the caller may have it*. The daemon that actually opens files calls
//! in here first.
//!
//! # The problem this solves
//!
//! `LITRIC_NATIVE_RUNTIME_ABI_SECURITY_v1.md` section 17 states the rule:
//! *"An application must not assume `/dev/nvme0n1` or `C:\` exists in the native
//! layer."* Applications address storage logically:
//!
//! ```text
//!   user://Documents/report.odt      the user's documents
//!   app://com.example.editor/data    this application's private data
//!   cache://com.example.editor       discardable, may vanish at any time
//!   config://com.example.editor      settings, backed up
//!   temp://com.example.editor        cleared at reboot
//!   secure://com.example.editor      keyring-backed secrets
//!   media://Pictures                 photos and video
//! ```
//!
//! Two things follow, and both are security properties rather than conveniences:
//!
//! 1. **Physical layout is not an API contract.** Moving `/home` to another
//!    device, adding an A/B slot or changing an encryption scheme does not
//!    break a single application.
//! 2. **A path cannot escape its namespace.** `app://` is per-application by
//!    construction, so one application cannot name another's data at all — not
//!    "is denied when it tries", but *cannot express the request*.
//!
//! # What this module refuses to do
//!
//! It never resolves `..`. Resolving means deciding what a path means, and every
//! directory-traversal vulnerability in history is a disagreement between the
//! code that resolved and the code that checked. Rejecting the input outright
//! leaves nothing to disagree about.

#![forbid(unsafe_code)]

use core::fmt;
use litric_caps::{Capability, CapabilityId, Scope};
use litric_core::AppId;

/// A storage namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Namespace {
    /// The user's documents and general files. Shared between applications, and
    /// therefore always capability-gated.
    User,
    /// Photos and video. Separate from `user` so that a photo application can be
    /// given exactly this and nothing else.
    Media,
    /// Private per-application data. Backed up.
    App,
    /// Per-application cache. **May be deleted at any time without warning**;
    /// an application that cannot survive that has a bug.
    Cache,
    /// Per-application configuration. Backed up, and preserved across an
    /// application reinstall unless the user asks otherwise.
    Config,
    /// Per-application scratch space. Cleared at every boot.
    Temp,
    /// Per-application secrets. Backed by the keyring, never by a plain file.
    Secure,
}

impl Namespace {
    /// The URI scheme, including `://`.
    #[must_use]
    pub const fn scheme(self) -> &'static str {
        match self {
            Self::User => "user://",
            Self::Media => "media://",
            Self::App => "app://",
            Self::Cache => "cache://",
            Self::Config => "config://",
            Self::Temp => "temp://",
            Self::Secure => "secure://",
        }
    }

    /// Every namespace, for exhaustiveness tests and for the settings UI.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::User,
            Self::Media,
            Self::App,
            Self::Cache,
            Self::Config,
            Self::Temp,
            Self::Secure,
        ]
    }

    /// Whether paths in this namespace are private to one application.
    ///
    /// Private namespaces embed the application id as their first path segment,
    /// which is what makes cross-application access inexpressible rather than
    /// merely forbidden.
    #[must_use]
    pub const fn is_per_app(self) -> bool {
        matches!(
            self,
            Self::App | Self::Cache | Self::Config | Self::Temp | Self::Secure
        )
    }

    /// Whether an application needs a capability to touch this namespace.
    ///
    /// An application does not need permission to use its own private data —
    /// requiring one would mean prompting the user for something they cannot
    /// meaningfully refuse. Shared namespaces always require a capability.
    #[must_use]
    pub const fn requires_capability(self) -> bool {
        !self.is_per_app()
    }

    /// Whether contents survive a reboot.
    #[must_use]
    pub const fn is_persistent(self) -> bool {
        !matches!(self, Self::Temp)
    }

    /// Whether contents may be discarded by the system under space pressure.
    #[must_use]
    pub const fn is_discardable(self) -> bool {
        matches!(self, Self::Cache | Self::Temp)
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.scheme())
    }
}

/// Why a storage URI was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UriError {
    /// Empty input.
    Empty,
    /// Longer than [`StorageUri::MAX_LEN`].
    TooLong {
        /// Actual length in bytes.
        len: usize,
    },
    /// No recognised namespace scheme.
    UnknownNamespace,
    /// A `.` or `..` path segment. Never resolved, always rejected.
    Traversal,
    /// A NUL, newline or other control byte.
    ControlCharacter {
        /// Byte offset.
        index: usize,
    },
    /// A backslash, which some filesystems treat as a separator.
    Backslash,
    /// An empty path segment, i.e. a doubled separator.
    EmptySegment,
    /// A path segment exceeding [`StorageUri::MAX_SEGMENT`].
    SegmentTooLong,
    /// A private namespace URI whose application id is malformed.
    BadOwner,
}

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "storage URI is empty"),
            Self::TooLong { len } => {
                write!(
                    f,
                    "storage URI is {len} bytes, maximum {}",
                    StorageUri::MAX_LEN
                )
            }
            Self::UnknownNamespace => write!(f, "storage URI has no recognised namespace"),
            Self::Traversal => write!(f, "storage URI contains a '.' or '..' segment"),
            Self::ControlCharacter { index } => {
                write!(f, "storage URI has a control character at offset {index}")
            }
            Self::Backslash => write!(f, "storage URI contains a backslash"),
            Self::EmptySegment => write!(f, "storage URI contains an empty path segment"),
            Self::SegmentTooLong => write!(f, "a path segment is too long"),
            Self::BadOwner => write!(f, "private namespace URI has a malformed application id"),
        }
    }
}

impl std::error::Error for UriError {}

/// A validated logical storage location.
///
/// Holding a value of this type is a proof that the string was checked: it has a
/// known namespace, contains no traversal, no control characters and no
/// backslashes, and — for a private namespace — names a well-formed owner.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorageUri {
    namespace: Namespace,
    /// The owning application, for per-app namespaces.
    owner: Option<AppId>,
    /// Path segments after the namespace and owner. Never empty strings, never
    /// `.` or `..`.
    segments: Vec<String>,
}

impl StorageUri {
    /// Maximum total URI length in bytes.
    pub const MAX_LEN: usize = 4096;
    /// Maximum length of one path segment.
    pub const MAX_SEGMENT: usize = 255;

    /// Parse and validate a logical storage URI.
    ///
    /// # Errors
    /// [`UriError`] naming the first rule violated.
    pub fn parse(raw: &str) -> Result<Self, UriError> {
        if raw.is_empty() {
            return Err(UriError::Empty);
        }
        if raw.len() > Self::MAX_LEN {
            return Err(UriError::TooLong { len: raw.len() });
        }
        for (index, b) in raw.bytes().enumerate() {
            if b.is_ascii_control() {
                return Err(UriError::ControlCharacter { index });
            }
            if b == b'\\' {
                return Err(UriError::Backslash);
            }
        }

        let namespace = Namespace::all()
            .iter()
            .copied()
            .find(|n| raw.starts_with(n.scheme()))
            .ok_or(UriError::UnknownNamespace)?;

        let rest = raw.get(namespace.scheme().len()..).unwrap_or("");
        // A trailing slash is cosmetic; strip it before splitting so that
        // "user://Documents/" and "user://Documents" are the same location.
        let rest = rest.strip_suffix('/').unwrap_or(rest);

        let mut parts: Vec<&str> = if rest.is_empty() {
            Vec::new()
        } else {
            rest.split('/').collect()
        };

        let owner = if namespace.is_per_app() {
            if parts.is_empty() {
                return Err(UriError::BadOwner);
            }
            let head = parts.remove(0);
            Some(AppId::parse(head).map_err(|_| UriError::BadOwner)?)
        } else {
            None
        };

        let mut segments = Vec::with_capacity(parts.len());
        for p in parts {
            if p.is_empty() {
                return Err(UriError::EmptySegment);
            }
            if p == "." || p == ".." {
                return Err(UriError::Traversal);
            }
            if p.len() > Self::MAX_SEGMENT {
                return Err(UriError::SegmentTooLong);
            }
            segments.push(p.to_owned());
        }

        Ok(Self {
            namespace,
            owner,
            segments,
        })
    }

    /// Build a private-namespace URI for an application.
    ///
    /// # Errors
    /// [`UriError::UnknownNamespace`] if `namespace` is not per-application.
    pub fn for_app(
        namespace: Namespace,
        owner: &AppId,
        segments: &[&str],
    ) -> Result<Self, UriError> {
        if !namespace.is_per_app() {
            return Err(UriError::UnknownNamespace);
        }
        let mut segs = Vec::with_capacity(segments.len());
        for s in segments {
            if s.is_empty() {
                return Err(UriError::EmptySegment);
            }
            if *s == "." || *s == ".." {
                return Err(UriError::Traversal);
            }
            if s.contains('/') || s.contains('\\') {
                return Err(UriError::Backslash);
            }
            segs.push((*s).to_owned());
        }
        Ok(Self {
            namespace,
            owner: Some(owner.clone()),
            segments: segs,
        })
    }

    /// The namespace.
    #[must_use]
    pub const fn namespace(&self) -> Namespace {
        self.namespace
    }

    /// The owning application, for private namespaces.
    #[must_use]
    pub const fn owner(&self) -> Option<&AppId> {
        self.owner.as_ref()
    }

    /// Path segments after the namespace and owner.
    #[must_use]
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// The canonical string form.
    #[must_use]
    pub fn to_uri(&self) -> String {
        let mut s = String::from(self.namespace.scheme());
        if let Some(o) = &self.owner {
            s.push_str(o.as_str());
            if !self.segments.is_empty() {
                s.push('/');
            }
        }
        s.push_str(&self.segments.join("/"));
        s
    }

    /// The capability scope this URI falls under, for a [`litric_caps`] check.
    ///
    /// # Errors
    /// Returns the underlying scope error if the canonical form is somehow not
    /// a valid scope. This should be unreachable — a parsed `StorageUri` is
    /// always a valid scope — and is surfaced rather than unwrapped so that a
    /// future change to either grammar fails loudly instead of panicking in a
    /// system service.
    pub fn as_scope(&self) -> Result<Scope, litric_caps::ScopeError> {
        Scope::parse_uri(&self.to_uri())
    }

    /// Whether `self` is at or beneath `other`.
    #[must_use]
    pub fn is_within(&self, other: &Self) -> bool {
        if self.namespace != other.namespace || self.owner != other.owner {
            return false;
        }
        if other.segments.len() > self.segments.len() {
            return false;
        }
        // Segment-wise comparison, so "Doc" cannot contain "Documents".
        other
            .segments
            .iter()
            .zip(self.segments.iter())
            .all(|(a, b)| a == b)
    }
}

impl fmt::Display for StorageUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_uri())
    }
}

/// Where each namespace lives on the real filesystem.
///
/// Supplied by the daemon at startup, so tests run against a fixture and a
/// future layout change touches exactly one place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// Root for [`Namespace::User`], e.g. `/home/alice`.
    pub user_root: String,
    /// Root for [`Namespace::Media`], e.g. `/home/alice/Media`.
    pub media_root: String,
    /// Root for per-app persistent data, e.g. `/home/alice/.local/share/litric`.
    pub app_root: String,
    /// Root for per-app cache, e.g. `/home/alice/.cache/litric`.
    pub cache_root: String,
    /// Root for per-app config, e.g. `/home/alice/.config/litric`.
    pub config_root: String,
    /// Root for per-app temp, e.g. `/run/user/1000/litric`.
    pub temp_root: String,
}

impl Layout {
    /// The conventional layout for a user's home directory.
    #[must_use]
    pub fn for_home(home: &str) -> Self {
        let home = home.trim_end_matches('/');
        Self {
            user_root: home.to_owned(),
            media_root: format!("{home}/Media"),
            app_root: format!("{home}/.local/share/litric"),
            cache_root: format!("{home}/.cache/litric"),
            config_root: format!("{home}/.config/litric"),
            temp_root: format!("{home}/.litric-run"),
        }
    }

    fn root_for(&self, ns: Namespace) -> Option<&str> {
        Some(match ns {
            Namespace::User => &self.user_root,
            Namespace::Media => &self.media_root,
            Namespace::App => &self.app_root,
            Namespace::Cache => &self.cache_root,
            Namespace::Config => &self.config_root,
            Namespace::Temp => &self.temp_root,
            // Secrets are not files. There is deliberately no filesystem path
            // for this namespace: returning one would invite something to write
            // a secret to disk in plain text.
            Namespace::Secure => return None,
        })
    }
}

/// Why a resolution was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolveError {
    /// The caller lacks the capability for this location.
    PermissionDenied {
        /// The capability that would have been required.
        needed: CapabilityId,
        /// The location requested.
        uri: String,
    },
    /// The caller tried to reach another application's private namespace.
    ForeignPrivateNamespace {
        /// The application that actually owns it.
        owner: String,
    },
    /// This namespace has no filesystem path by design.
    NotAFilesystemPath {
        /// Which namespace.
        namespace: Namespace,
    },
    /// The URI itself was invalid.
    BadUri(UriError),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied { needed, uri } => {
                write!(f, "permission denied: {needed} is required for {uri}")
            }
            Self::ForeignPrivateNamespace { owner } => {
                write!(f, "that private storage belongs to {owner}")
            }
            Self::NotAFilesystemPath { namespace } => {
                write!(f, "{namespace} is not backed by the filesystem")
            }
            Self::BadUri(e) => write!(f, "invalid storage URI: {e}"),
        }
    }
}

impl std::error::Error for ResolveError {}

/// What the caller wants to do, which decides the capability required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Read file contents.
    Read,
    /// Create, modify or delete.
    Write,
    /// Enumerate a directory.
    List,
}

impl Access {
    /// The capability this access requires.
    #[must_use]
    pub const fn capability(self) -> CapabilityId {
        match self {
            Self::Read => CapabilityId::FsRead,
            Self::Write => CapabilityId::FsWrite,
            Self::List => CapabilityId::FsList,
        }
    }
}

/// Resolves logical URIs to real paths, under capability control.
#[derive(Debug, Clone)]
pub struct StorageBroker {
    layout: Layout,
}

impl StorageBroker {
    /// Create a broker over a filesystem layout.
    #[must_use]
    pub const fn new(layout: Layout) -> Self {
        Self { layout }
    }

    /// The layout in use.
    #[must_use]
    pub const fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Which capability `caller` needs to perform `access` on `uri`, or `None`
    /// when no capability is required (its own private storage).
    ///
    /// # Errors
    /// [`ResolveError::ForeignPrivateNamespace`] if the URI names another
    /// application's private storage.
    pub fn required_capability(
        &self,
        caller: &AppId,
        uri: &StorageUri,
        access: Access,
    ) -> Result<Option<Capability>, ResolveError> {
        if uri.namespace().is_per_app() {
            let owner = uri
                .owner()
                .ok_or(ResolveError::BadUri(UriError::BadOwner))?;
            if owner != caller {
                return Err(ResolveError::ForeignPrivateNamespace {
                    owner: owner.as_str().to_owned(),
                });
            }
            // Its own private storage. No capability, and no prompt: a prompt
            // the user cannot meaningfully refuse teaches them to click through
            // prompts, which is how the ones that matter stop working.
            return Ok(None);
        }

        let scope = uri
            .as_scope()
            .map_err(|_| ResolveError::BadUri(UriError::UnknownNamespace))?;
        let cap = Capability::new(access.capability(), scope)
            .ok_or(ResolveError::BadUri(UriError::UnknownNamespace))?;
        Ok(Some(cap))
    }

    /// Resolve a URI to a real filesystem path.
    ///
    /// `granted` is the set of capabilities the caller actually holds, obtained
    /// from the capability broker. It is never taken from the caller.
    ///
    /// # Errors
    /// [`ResolveError`] describing why the request was refused.
    pub fn resolve(
        &self,
        caller: &AppId,
        uri: &StorageUri,
        access: Access,
        granted: &[Capability],
    ) -> Result<String, ResolveError> {
        if let Some(needed) = self.required_capability(caller, uri, access)? {
            let ok = granted.iter().any(|g| g.authorises(&needed));
            if !ok {
                return Err(ResolveError::PermissionDenied {
                    needed: access.capability(),
                    uri: uri.to_uri(),
                });
            }
        }

        let root =
            self.layout
                .root_for(uri.namespace())
                .ok_or(ResolveError::NotAFilesystemPath {
                    namespace: uri.namespace(),
                })?;

        let mut path = String::from(root);
        if let Some(owner) = uri.owner() {
            path.push('/');
            path.push_str(owner.as_path_component());
        }
        for seg in uri.segments() {
            path.push('/');
            path.push_str(seg);
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use litric_caps::Scope;

    fn app(s: &str) -> AppId {
        AppId::parse(s).expect("valid app id")
    }
    fn layout() -> Layout {
        Layout::for_home("/home/alice")
    }
    fn broker() -> StorageBroker {
        StorageBroker::new(layout())
    }
    fn cap(id: CapabilityId, uri: &str) -> Capability {
        Capability::new(id, Scope::parse_uri(uri).expect("valid scope")).expect("valid capability")
    }

    // ---- parsing -----------------------------------------------------------

    #[test]
    fn parses_every_namespace() {
        assert_eq!(
            StorageUri::parse("user://Documents")
                .expect("v")
                .namespace(),
            Namespace::User
        );
        assert_eq!(
            StorageUri::parse("media://Pictures")
                .expect("v")
                .namespace(),
            Namespace::Media
        );
        for scheme in ["app", "cache", "config", "temp", "secure"] {
            let u = StorageUri::parse(&format!("{scheme}://com.example.app/data")).expect("v");
            assert!(u.namespace().is_per_app());
            assert_eq!(u.owner().map(AppId::as_str), Some("com.example.app"));
        }
    }

    #[test]
    fn round_trips_through_the_canonical_form() {
        for s in [
            "user://Documents",
            "user://Documents/tax/2026.pdf",
            "media://Pictures/2026",
            "app://com.example.app/data",
            "cache://com.example.app",
            "config://com.example.app/settings.conf",
        ] {
            let u = StorageUri::parse(s).expect("v");
            assert_eq!(u.to_uri(), s, "round trip failed for {s}");
        }
    }

    #[test]
    fn trailing_slash_is_normalised() {
        assert_eq!(
            StorageUri::parse("user://Documents/").expect("v"),
            StorageUri::parse("user://Documents").expect("v")
        );
        assert_eq!(StorageUri::parse("user://").expect("v").segments().len(), 0);
    }

    // ---- hostile input -----------------------------------------------------

    #[test]
    fn rejects_traversal_rather_than_resolving_it() {
        for hostile in [
            "user://../etc/shadow",
            "user://Documents/../../root",
            "user://.",
            "user://Documents/./x",
            "app://com.example.app/../com.other.app/data",
            "cache://com.example.app/..",
        ] {
            let r = StorageUri::parse(hostile);
            assert!(r.is_err(), "must reject {hostile}, got {r:?}");
        }
    }

    #[test]
    fn rejects_control_characters_backslashes_and_empty_segments() {
        assert!(matches!(
            StorageUri::parse("user://a\0b"),
            Err(UriError::ControlCharacter { .. })
        ));
        assert!(matches!(
            StorageUri::parse("user://a\nb"),
            Err(UriError::ControlCharacter { .. })
        ));
        assert_eq!(
            StorageUri::parse("user://Windows\\System32"),
            Err(UriError::Backslash)
        );
        assert_eq!(
            StorageUri::parse("user://a//b"),
            Err(UriError::EmptySegment)
        );
    }

    #[test]
    fn rejects_unknown_and_absolute_paths() {
        for bad in [
            "/etc/passwd",
            "C:\\Windows",
            "file:///etc/passwd",
            "http://example.com",
            "",
            "users://Documents",
        ] {
            assert!(StorageUri::parse(bad).is_err(), "must reject {bad}");
        }
    }

    #[test]
    fn private_namespace_requires_a_well_formed_owner() {
        assert_eq!(StorageUri::parse("app://"), Err(UriError::BadOwner));
        assert_eq!(
            StorageUri::parse("app://notanappid"),
            Err(UriError::BadOwner)
        );
        assert_eq!(
            StorageUri::parse("cache://../escape"),
            Err(UriError::BadOwner)
        );
        assert!(StorageUri::parse("app://com.example.app").is_ok());
    }

    #[test]
    fn parsing_never_panics_on_arbitrary_input() {
        let base = "app://com.example.app/data/file.txt";
        for cut in 0..=base.len() {
            let _ = StorageUri::parse(base.get(..cut).unwrap_or(""));
        }
        for pattern in [
            "\u{feff}user://x",
            "user://\u{202e}gnp.exe",
            &"a".repeat(9000),
        ] {
            let _ = StorageUri::parse(pattern);
        }
    }

    // ---- containment -------------------------------------------------------

    #[test]
    fn containment_is_segment_wise() {
        let docs = StorageUri::parse("user://Documents").expect("v");
        let inside = StorageUri::parse("user://Documents/tax/2026.pdf").expect("v");
        let lookalike = StorageUri::parse("user://DocumentsSecret").expect("v");

        assert!(inside.is_within(&docs));
        assert!(docs.is_within(&docs));
        assert!(
            !lookalike.is_within(&docs),
            "prefix match must not cross a segment"
        );
        assert!(!docs.is_within(&inside));
    }

    #[test]
    fn containment_never_crosses_namespaces_or_owners() {
        let a = StorageUri::parse("app://com.example.one/data").expect("v");
        let b = StorageUri::parse("app://com.example.two/data").expect("v");
        assert!(!a.is_within(&b));
        assert!(!b.is_within(&a));

        let user = StorageUri::parse("user://Documents").expect("v");
        let media = StorageUri::parse("media://Documents").expect("v");
        assert!(!user.is_within(&media));
        assert!(!media.is_within(&user));
    }

    // ---- the security property: private namespaces are inexpressible -------

    #[test]
    fn one_app_cannot_reach_another_apps_private_storage() {
        // Not "is denied when it tries" -- the request is refused at the broker
        // before any capability is even consulted, because private storage is
        // owned, not permissioned.
        let b = broker();
        let me = app("com.example.mine");
        let theirs = StorageUri::parse("app://com.example.theirs/secrets").expect("v");

        let r = b.required_capability(&me, &theirs, Access::Read);
        assert!(matches!(
            r,
            Err(ResolveError::ForeignPrivateNamespace { .. })
        ));

        // And no capability, however broad, changes that.
        let everything = cap(CapabilityId::FsRead, "user://");
        let r = b.resolve(&me, &theirs, Access::Read, &[everything]);
        assert!(matches!(
            r,
            Err(ResolveError::ForeignPrivateNamespace { .. })
        ));
    }

    #[test]
    fn an_app_reaches_its_own_private_storage_with_no_capability_at_all() {
        let b = broker();
        let me = app("com.example.mine");
        let mine = StorageUri::parse("app://com.example.mine/data").expect("v");

        assert_eq!(
            b.required_capability(&me, &mine, Access::Write)
                .expect("ok"),
            None,
            "an app must not be prompted for its own data"
        );

        let path = b.resolve(&me, &mine, Access::Write, &[]).expect("resolves");
        assert_eq!(
            path,
            "/home/alice/.local/share/litric/com.example.mine/data"
        );
    }

    // ---- capability enforcement -------------------------------------------

    #[test]
    fn shared_namespaces_require_a_capability() {
        let b = broker();
        let me = app("com.example.editor");
        let docs = StorageUri::parse("user://Documents/report.odt").expect("v");

        // No grants at all.
        let r = b.resolve(&me, &docs, Access::Read, &[]);
        assert!(matches!(r, Err(ResolveError::PermissionDenied { .. })));

        // A grant on the containing folder works.
        let granted = cap(CapabilityId::FsRead, "user://Documents");
        let path = b
            .resolve(&me, &docs, Access::Read, &[granted])
            .expect("should resolve");
        assert_eq!(path, "/home/alice/Documents/report.odt");
    }

    #[test]
    fn a_read_grant_does_not_permit_a_write() {
        let b = broker();
        let me = app("com.example.editor");
        let docs = StorageUri::parse("user://Documents/report.odt").expect("v");
        let read_only = cap(CapabilityId::FsRead, "user://Documents");

        assert!(b
            .resolve(&me, &docs, Access::Read, std::slice::from_ref(&read_only))
            .is_ok());
        let r = b.resolve(&me, &docs, Access::Write, &[read_only]);
        assert!(matches!(
            r,
            Err(ResolveError::PermissionDenied {
                needed: CapabilityId::FsWrite,
                ..
            })
        ));
    }

    #[test]
    fn a_grant_on_one_folder_does_not_reach_a_sibling() {
        let b = broker();
        let me = app("com.example.editor");
        let granted = cap(CapabilityId::FsRead, "user://Documents");

        for outside in [
            "user://Pictures/holiday.jpg",
            "user://DocumentsSecret/x",
            "user://.ssh/id_ed25519",
            "media://Pictures",
        ] {
            let uri = StorageUri::parse(outside).expect("v");
            let r = b.resolve(&me, &uri, Access::Read, std::slice::from_ref(&granted));
            assert!(
                matches!(r, Err(ResolveError::PermissionDenied { .. })),
                "{outside} must be denied"
            );
        }
    }

    // ---- resolution --------------------------------------------------------

    #[test]
    fn resolution_places_each_namespace_under_its_own_root() {
        let b = broker();
        let me = app("com.example.app");
        let all = cap(CapabilityId::FsRead, "user://");
        let media_all = cap(CapabilityId::FsRead, "media://");

        let cases = [
            (
                "user://Documents/x",
                vec![all.clone()],
                "/home/alice/Documents/x",
            ),
            (
                "media://Pictures",
                vec![media_all],
                "/home/alice/Media/Pictures",
            ),
            (
                "app://com.example.app/db",
                vec![],
                "/home/alice/.local/share/litric/com.example.app/db",
            ),
            (
                "cache://com.example.app/thumbs",
                vec![],
                "/home/alice/.cache/litric/com.example.app/thumbs",
            ),
            (
                "config://com.example.app/settings",
                vec![],
                "/home/alice/.config/litric/com.example.app/settings",
            ),
            (
                "temp://com.example.app/scratch",
                vec![],
                "/home/alice/.litric-run/com.example.app/scratch",
            ),
        ];

        for (uri, grants, expected) in cases {
            let u = StorageUri::parse(uri).expect("v");
            let got = b
                .resolve(&me, &u, Access::Read, &grants)
                .unwrap_or_else(|e| panic!("{uri} should resolve: {e}"));
            assert_eq!(got, expected, "wrong path for {uri}");
        }
    }

    #[test]
    fn resolved_paths_never_escape_their_root() {
        // Property test. For every URI we accept, the resolved path must start
        // with the namespace root and contain no traversal sequence. This is the
        // end-to-end statement of the whole module.
        let b = broker();
        let me = app("com.example.app");
        let all = cap(CapabilityId::FsRead, "user://");

        for candidate in [
            "user://Documents",
            "user://a/b/c/d/e",
            "user://..hidden",
            "user://...",
            "user://-",
            "user://a b c",
            "app://com.example.app/x/y",
            "cache://com.example.app",
        ] {
            let Ok(u) = StorageUri::parse(candidate) else {
                continue;
            };
            let Ok(path) = b.resolve(&me, &u, Access::Read, std::slice::from_ref(&all)) else {
                continue;
            };
            assert!(!path.contains("/../"), "{candidate} -> {path} contains ..");
            assert!(!path.ends_with("/.."), "{candidate} -> {path} ends with ..");
            assert!(
                path.starts_with("/home/alice"),
                "{candidate} -> {path} escaped the home root"
            );
        }
    }

    #[test]
    fn secure_storage_has_no_filesystem_path() {
        // Deliberate: returning a path would invite something to write a secret
        // to disk in plain text.
        let b = broker();
        let me = app("com.example.app");
        let u = StorageUri::parse("secure://com.example.app/token").expect("v");
        let r = b.resolve(&me, &u, Access::Read, &[]);
        assert!(matches!(
            r,
            Err(ResolveError::NotAFilesystemPath {
                namespace: Namespace::Secure
            })
        ));
    }

    // ---- namespace semantics ----------------------------------------------

    #[test]
    fn namespace_properties_are_coherent() {
        for ns in Namespace::all() {
            // Per-app namespaces never require a capability; shared ones always do.
            assert_eq!(ns.requires_capability(), !ns.is_per_app(), "{ns}");
            // Every scheme is distinct and well formed.
            assert!(ns.scheme().ends_with("://"), "{ns}");
        }
        assert!(!Namespace::Temp.is_persistent());
        assert!(Namespace::Config.is_persistent());
        assert!(Namespace::Cache.is_discardable());
        assert!(!Namespace::App.is_discardable());
    }

    #[test]
    fn schemes_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for ns in Namespace::all() {
            assert!(seen.insert(ns.scheme()), "duplicate scheme {}", ns.scheme());
        }
    }

    #[test]
    fn every_uri_produces_a_valid_capability_scope() {
        // The two grammars must agree, or a location could be resolvable and
        // yet impossible to write a grant for.
        for s in [
            "user://Documents",
            "media://Pictures/2026",
            "app://com.example.app/data",
            "user://",
        ] {
            let u = StorageUri::parse(s).expect("v");
            assert!(u.as_scope().is_ok(), "{s} has no valid capability scope");
        }
    }
}
