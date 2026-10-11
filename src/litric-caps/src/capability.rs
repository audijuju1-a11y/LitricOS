//! Capability identifiers and resource scopes.
//!
//! A capability answers *what may be done*. A scope answers *to what*. They are
//! separate on purpose, because the difference between them is the difference
//! between a permission model that limits damage and one that does not:
//!
//! ```text
//!   fs.read                        "read any file"          -- blast radius: everything
//!   fs.read @ user://Documents/tax "read this one folder"   -- blast radius: one folder
//! ```
//!
//! ECOSYSTEM section 6.1 states the rule directly: capabilities should refer to
//! specific resources rather than broad global access, because that is what
//! reduces the blast radius when an application is compromised.

use core::fmt;

/// What kind of action a capability authorises.
///
/// The string forms are the stable contract: they appear in application
/// manifests, in the audit log, and in the permission UI. **Never change one.**
/// Adding a variant is additive and safe; renaming one silently invalidates
/// every manifest in existence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CapabilityId {
    /// Read files within the scope.
    FsRead,
    /// Write files within the scope.
    FsWrite,
    /// Enumerate directory contents within the scope.
    FsList,
    /// Open a camera device.
    Camera,
    /// Open an audio capture device.
    Microphone,
    /// Play audio.
    AudioOutput,
    /// Read the device's geographic location.
    Location,
    /// Make outbound network connections.
    NetClient,
    /// Listen for inbound network connections.
    NetServer,
    /// Discover peers on the local link (mDNS and friends).
    NetLocal,
    /// Use the GPU for rendering.
    Gpu,
    /// Use the GPU for compute.
    GpuCompute,
    /// Post notifications.
    Notify,
    /// Read the clipboard.
    ClipboardRead,
    /// Write the clipboard.
    ClipboardWrite,
    /// Capture the screen or share it.
    ScreenCapture,
    /// Receive input events regardless of focus. Extremely dangerous: this is
    /// what a keylogger needs.
    InputGlobal,
    /// Run local AI inference.
    AiLocal,
    /// Send data to a cloud AI service.
    AiCloud,
    /// Access a USB device.
    DeviceUsb,
    /// Access a Bluetooth device.
    DeviceBluetooth,
    /// Start automatically at login.
    Autostart,
    /// Continue running with no window.
    Background,
    /// Store and retrieve secrets in the keyring.
    SecureStorage,
    /// Change system settings.
    SystemSettings,
    /// Attach a debugger to another process.
    ProcessDebug,
}

impl CapabilityId {
    /// The stable string form used in manifests and logs.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::FsRead => "fs.read",
            Self::FsWrite => "fs.write",
            Self::FsList => "fs.list",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::AudioOutput => "audio.output",
            Self::Location => "location",
            Self::NetClient => "net.client",
            Self::NetServer => "net.server",
            Self::NetLocal => "net.local",
            Self::Gpu => "gpu",
            Self::GpuCompute => "gpu.compute",
            Self::Notify => "notify",
            Self::ClipboardRead => "clipboard.read",
            Self::ClipboardWrite => "clipboard.write",
            Self::ScreenCapture => "screen.capture",
            Self::InputGlobal => "input.global",
            Self::AiLocal => "ai.local",
            Self::AiCloud => "ai.cloud",
            Self::DeviceUsb => "device.usb",
            Self::DeviceBluetooth => "device.bluetooth",
            Self::Autostart => "autostart",
            Self::Background => "background",
            Self::SecureStorage => "secure.storage",
            Self::SystemSettings => "system.settings",
            Self::ProcessDebug => "process.debug",
        }
    }

    /// Parse the stable string form.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "fs.read" => Self::FsRead,
            "fs.write" => Self::FsWrite,
            "fs.list" => Self::FsList,
            "camera" => Self::Camera,
            "microphone" => Self::Microphone,
            "audio.output" => Self::AudioOutput,
            "location" => Self::Location,
            "net.client" => Self::NetClient,
            "net.server" => Self::NetServer,
            "net.local" => Self::NetLocal,
            "gpu" => Self::Gpu,
            "gpu.compute" => Self::GpuCompute,
            "notify" => Self::Notify,
            "clipboard.read" => Self::ClipboardRead,
            "clipboard.write" => Self::ClipboardWrite,
            "screen.capture" => Self::ScreenCapture,
            "input.global" => Self::InputGlobal,
            "ai.local" => Self::AiLocal,
            "ai.cloud" => Self::AiCloud,
            "device.usb" => Self::DeviceUsb,
            "device.bluetooth" => Self::DeviceBluetooth,
            "autostart" => Self::Autostart,
            "background" => Self::Background,
            "secure.storage" => Self::SecureStorage,
            "system.settings" => Self::SystemSettings,
            "process.debug" => Self::ProcessDebug,
            _ => return None,
        })
    }

    /// Every capability, for the settings UI and for exhaustiveness tests.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::FsRead,
            Self::FsWrite,
            Self::FsList,
            Self::Camera,
            Self::Microphone,
            Self::AudioOutput,
            Self::Location,
            Self::NetClient,
            Self::NetServer,
            Self::NetLocal,
            Self::Gpu,
            Self::GpuCompute,
            Self::Notify,
            Self::ClipboardRead,
            Self::ClipboardWrite,
            Self::ScreenCapture,
            Self::InputGlobal,
            Self::AiLocal,
            Self::AiCloud,
            Self::DeviceUsb,
            Self::DeviceBluetooth,
            Self::Autostart,
            Self::Background,
            Self::SecureStorage,
            Self::SystemSettings,
            Self::ProcessDebug,
        ]
    }

    /// Whether this capability may ever be granted without the user being asked
    /// in that moment.
    ///
    /// `false` means a stored "yes" is not sufficient authority on its own: the
    /// grant must be accompanied by a live user action, such as choosing a file
    /// in the picker or answering a prompt. Camera, microphone, location, screen
    /// capture and global input are the ones people are actually harmed by, and
    /// they are the ones where a silent grant is a betrayal rather than a
    /// convenience.
    #[must_use]
    pub const fn requires_live_consent(self) -> bool {
        matches!(
            self,
            Self::Camera
                | Self::Microphone
                | Self::Location
                | Self::ScreenCapture
                | Self::InputGlobal
                | Self::AiCloud
                | Self::ProcessDebug
                | Self::SystemSettings
        )
    }

    /// Whether an active-use indicator must be shown while this is in use.
    ///
    /// Not negotiable and not configurable. If the microphone is live, the user
    /// sees that the microphone is live.
    #[must_use]
    pub const fn requires_active_indicator(self) -> bool {
        matches!(
            self,
            Self::Camera | Self::Microphone | Self::ScreenCapture | Self::Location
        )
    }

    /// How this capability relates to a resource scope.
    ///
    /// Three classes, not two. The distinction matters and getting it wrong
    /// produces either an unusable permission model or an unsafe one:
    ///
    /// * [`Scoping::Never`] — "the left half of the camera" is not a thing.
    /// * [`Scoping::Required`] — filesystem access must always name a location.
    ///   To express "all of the user's files", grant the root scope `user://`,
    ///   which is explicit and shows up as such in the permission UI. There is
    ///   deliberately no way to write an unbounded filesystem grant that *looks*
    ///   narrow.
    /// * [`Scoping::Optional`] — network and device access. A browser cannot
    ///   enumerate every host it will ever visit, so unscoped must be legal; but
    ///   an app that talks to exactly one API should be granted exactly that
    ///   host, and the model has to be able to say so.
    #[must_use]
    pub const fn scoping(self) -> Scoping {
        match self {
            Self::FsRead | Self::FsWrite | Self::FsList => Scoping::Required,
            Self::NetClient | Self::NetServer | Self::DeviceUsb | Self::DeviceBluetooth => {
                Scoping::Optional
            }
            _ => Scoping::Never,
        }
    }

    /// Plain-language text for the permission prompt.
    ///
    /// Written to be understood by someone who does not know what a capability
    /// is. "Allow access to fs.read" is not a question anyone can answer.
    #[must_use]
    pub const fn prompt(self) -> &'static str {
        match self {
            Self::FsRead => "read your files",
            Self::FsWrite => "change your files",
            Self::FsList => "see what files you have",
            Self::Camera => "use your camera",
            Self::Microphone => "listen through your microphone",
            Self::AudioOutput => "play sound",
            Self::Location => "know where you are",
            Self::NetClient => "connect to the internet",
            Self::NetServer => "accept connections from other devices",
            Self::NetLocal => "find devices on your local network",
            Self::Gpu => "use the graphics card to draw",
            Self::GpuCompute => "run computations on the graphics card",
            Self::Notify => "show you notifications",
            Self::ClipboardRead => "read what you have copied",
            Self::ClipboardWrite => "put things on your clipboard",
            Self::ScreenCapture => "see what is on your screen",
            Self::InputGlobal => "see everything you type, in any application",
            Self::AiLocal => "use AI features on this device",
            Self::AiCloud => "send your data to an AI service on the internet",
            Self::DeviceUsb => "use a connected USB device",
            Self::DeviceBluetooth => "use a Bluetooth device",
            Self::Autostart => "start automatically when you log in",
            Self::Background => "keep running after you close its window",
            Self::SecureStorage => "store passwords in your keyring",
            Self::SystemSettings => "change system settings",
            Self::ProcessDebug => "inspect other running programs",
        }
    }

    /// How dangerous this is, for prompt ordering and for the settings UI.
    #[must_use]
    pub const fn risk(self) -> Risk {
        match self {
            Self::InputGlobal | Self::ProcessDebug | Self::ScreenCapture => Risk::Critical,
            Self::Camera
            | Self::Microphone
            | Self::Location
            | Self::AiCloud
            | Self::SystemSettings
            | Self::FsWrite
            | Self::SecureStorage => Risk::High,
            Self::FsRead
            | Self::FsList
            | Self::NetServer
            | Self::ClipboardRead
            | Self::DeviceUsb
            | Self::DeviceBluetooth
            | Self::Autostart
            | Self::Background => Risk::Medium,
            Self::NetClient
            | Self::NetLocal
            | Self::Gpu
            | Self::GpuCompute
            | Self::Notify
            | Self::ClipboardWrite
            | Self::AudioOutput
            | Self::AiLocal => Risk::Low,
        }
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// How a capability relates to a resource scope. See [`CapabilityId::scoping`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scoping {
    /// A scope is meaningless; only [`Scope::Any`] is legal.
    Never,
    /// A scope is mandatory; [`Scope::Any`] is refused.
    Required,
    /// A scope is allowed but not required.
    Optional,
}

/// Risk classification, used for prompt ordering and UI emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Risk {
    /// Routine. Granted with little ceremony.
    Low,
    /// Worth telling the user about.
    Medium,
    /// Prompt, and show it prominently in settings.
    High,
    /// Prompt with an explicit warning. Never pre-selected.
    Critical,
}

/// The resource a capability applies to.
///
/// `Any` is deliberately awkward to construct and is rejected outright for the
/// capabilities where unscoped access is indefensible.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    /// The whole capability class. Only legal for unscoped capabilities.
    Any,
    /// A logical storage URI, e.g. `user://Documents/tax`.
    Uri(String),
    /// A network host, e.g. `api.example.com`.
    Host(String),
    /// A device identifier.
    Device(String),
}

/// Why a scope string was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScopeError {
    /// The scope was empty.
    Empty,
    /// The URI had no recognised scheme.
    UnknownScheme,
    /// A path segment was `.` or `..`, or the path was otherwise not normalised.
    NotNormalised,
    /// A NUL, newline, or other control byte appeared.
    IllegalCharacter,
    /// Longer than [`Scope::MAX_LEN`].
    TooLong,
}

impl fmt::Display for ScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "scope is empty"),
            Self::UnknownScheme => write!(f, "scope has no recognised URI scheme"),
            Self::NotNormalised => write!(f, "scope contains '.' or '..' path segments"),
            Self::IllegalCharacter => write!(f, "scope contains a control character"),
            Self::TooLong => write!(f, "scope exceeds {} bytes", Scope::MAX_LEN),
        }
    }
}

impl std::error::Error for ScopeError {}

/// Storage URI schemes, from RUNTIME section 17.
const URI_SCHEMES: &[&str] = &[
    "user://",
    "app://",
    "cache://",
    "config://",
    "temp://",
    "secure://",
    "media://",
];

impl Scope {
    /// Maximum scope length in bytes.
    pub const MAX_LEN: usize = 4096;

    /// Parse and normalise a storage-URI scope.
    ///
    /// # Errors
    /// [`ScopeError`] naming the rule that was violated.
    ///
    /// # Security
    /// This is the function that stops `user://Documents/../../etc/shadow`. It
    /// rejects `..` outright rather than resolving it: resolving means deciding
    /// what the path means, and every directory-traversal vulnerability in
    /// history is a disagreement between the code that resolved and the code
    /// that checked. Refusing to accept the input at all leaves nothing to
    /// disagree about.
    pub fn parse_uri(raw: &str) -> Result<Self, ScopeError> {
        if raw.is_empty() {
            return Err(ScopeError::Empty);
        }
        if raw.len() > Self::MAX_LEN {
            return Err(ScopeError::TooLong);
        }
        if raw.bytes().any(|b| b.is_ascii_control()) {
            return Err(ScopeError::IllegalCharacter);
        }

        let scheme = URI_SCHEMES
            .iter()
            .find(|s| raw.starts_with(**s))
            .ok_or(ScopeError::UnknownScheme)?;

        let path = raw.get(scheme.len()..).unwrap_or("");
        for segment in path.split('/') {
            if segment == "." || segment == ".." {
                return Err(ScopeError::NotNormalised);
            }
        }
        // A trailing slash would make containment comparisons ambiguous
        // ("user://Docs" vs "user://Docs/"), so normalise it away once here
        // rather than handling both forms at every comparison site.
        //
        // The trim is applied to the PATH ONLY, never to the whole string:
        // `"user://".trim_end_matches('/')` yields `"user:"`, destroying the
        // scheme and turning the root scope into an unparseable fragment that
        // then fails to contain anything. The root scope is the one a
        // "read all my files" grant uses, so that bug would have silently
        // produced a grant authorising nothing.
        let path = raw.get(scheme.len()..).unwrap_or("");
        let trimmed_path = path.trim_end_matches('/');
        Ok(Self::Uri(format!("{scheme}{trimmed_path}")))
    }

    /// Parse a network host scope.
    ///
    /// # Errors
    /// [`ScopeError`] if empty, over-long, or containing control characters.
    pub fn parse_host(raw: &str) -> Result<Self, ScopeError> {
        if raw.is_empty() {
            return Err(ScopeError::Empty);
        }
        if raw.len() > 253 {
            return Err(ScopeError::TooLong);
        }
        if raw
            .bytes()
            .any(|b| b.is_ascii_control() || b == b'/' || b == b' ')
        {
            return Err(ScopeError::IllegalCharacter);
        }
        Ok(Self::Host(raw.to_ascii_lowercase()))
    }

    /// Whether `self` (a grant's scope) authorises `other` (a request's scope).
    ///
    /// Containment is **path-segment aware**. This is the subtle part and it is
    /// where naive implementations break:
    ///
    /// ```text
    ///   user://Doc        does NOT contain  user://Documents
    ///   user://Documents  does     contain  user://Documents/tax/2026.pdf
    /// ```
    ///
    /// A plain `starts_with` says the first line is contained, which hands an
    /// application every folder whose name merely begins with the one it was
    /// granted.
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            // A grant of Any covers everything of its kind.
            (Self::Any, _) => true,
            // A request for Any is only satisfied by a grant of Any, handled above.
            (_, Self::Any) => false,

            (Self::Uri(grant), Self::Uri(req)) => path_contains(grant, req),

            // Host matching allows a leading-dot wildcard for subdomains:
            // ".example.com" covers "api.example.com" but not "example.com" or
            // "notexample.com".
            (Self::Host(grant), Self::Host(req)) => {
                if let Some(suffix) = grant.strip_prefix('.') {
                    req == suffix || req.ends_with(grant.as_str())
                } else {
                    grant == req
                }
            }

            (Self::Device(grant), Self::Device(req)) => grant == req,

            // Different scope kinds never match. A filesystem grant does not
            // authorise a network request, however similar the strings look.
            _ => false,
        }
    }

    /// The string form, for logs and the permission UI.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Any => "*",
            Self::Uri(s) | Self::Host(s) | Self::Device(s) => s,
        }
    }
}

/// Segment-aware prefix containment for normalised URI scopes.
fn path_contains(grant: &str, req: &str) -> bool {
    if grant == req {
        return true;
    }
    match req.strip_prefix(grant) {
        Some(rest) => {
            // The request must start with the grant AND there must be a segment
            // boundary between them, so that "user://Doc" cannot swallow
            // "user://Documents".
            //
            // The boundary can come from either side. A normalised path scope
            // never ends in '/', so the boundary is the first byte of `rest`.
            // The ROOT scope is the exception: "user://" already ends with the
            // separator, so requiring another one would make the root contain
            // nothing at all -- which would turn a "read all my files" grant
            // into a grant authorising precisely nothing.
            grant.ends_with('/') || rest.starts_with('/')
        }
        None => false,
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capability applied to a resource: the unit of authorisation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capability {
    id: CapabilityId,
    scope: Scope,
}

impl Capability {
    /// Construct.
    ///
    /// # Errors
    /// Returns `None` if the capability is scoped but was given [`Scope::Any`],
    /// or is unscoped but was given a specific scope. Both are programming
    /// errors that would otherwise produce a grant meaning something other than
    /// what the caller intended.
    #[must_use]
    pub fn new(id: CapabilityId, scope: Scope) -> Option<Self> {
        let concrete = !matches!(scope, Scope::Any);
        match (id.scoping(), concrete) {
            // "camera, but only for user://Documents" authorises nothing
            // coherent, so it is refused rather than silently ignored.
            (Scoping::Never, false) => Some(Self { id, scope }),
            (Scoping::Never, true) => None,
            // An unbounded filesystem grant must be written as the explicit root
            // scope, never as Any, so that it is visible as what it is.
            (Scoping::Required, true) => Some(Self { id, scope }),
            (Scoping::Required, false) => None,
            (Scoping::Optional, _) => Some(Self { id, scope }),
        }
    }

    /// Construct an unscoped capability.
    #[must_use]
    pub fn unscoped(id: CapabilityId) -> Option<Self> {
        Self::new(id, Scope::Any)
    }

    /// The capability kind.
    #[must_use]
    pub const fn id(&self) -> CapabilityId {
        self.id
    }

    /// The resource scope.
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }

    /// Whether holding `self` authorises `request`.
    #[must_use]
    pub fn authorises(&self, request: &Self) -> bool {
        self.id == request.id && self.scope.contains(&request.scope)
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scope {
            Scope::Any => f.write_str(self.id.token()),
            s => write!(f, "{}@{}", self.id.token(), s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- capability identity ----------------------------------------------

    #[test]
    fn every_capability_round_trips_through_its_token() {
        for id in CapabilityId::all() {
            assert_eq!(
                CapabilityId::parse(id.token()),
                Some(*id),
                "token {:?} does not round trip",
                id.token()
            );
        }
    }

    #[test]
    fn tokens_are_unique() {
        // A duplicated token would make two different capabilities
        // indistinguishable in a manifest -- one would silently grant the other.
        let mut seen = std::collections::BTreeSet::new();
        for id in CapabilityId::all() {
            assert!(seen.insert(id.token()), "duplicate token: {}", id.token());
        }
        assert_eq!(seen.len(), CapabilityId::all().len());
    }

    #[test]
    fn every_capability_has_a_human_prompt() {
        for id in CapabilityId::all() {
            let p = id.prompt();
            assert!(!p.is_empty(), "{id} has no prompt");
            // The prompt is shown to a person. It must not be the token.
            assert_ne!(p, id.token(), "{id} prompt is just the token");
        }
    }

    #[test]
    fn unknown_tokens_are_rejected_not_guessed() {
        assert_eq!(CapabilityId::parse("fs.read.everything"), None);
        assert_eq!(CapabilityId::parse("FS.READ"), None);
        assert_eq!(CapabilityId::parse(""), None);
        assert_eq!(CapabilityId::parse("camera "), None);
        assert_eq!(CapabilityId::parse("../camera"), None);
    }

    #[test]
    fn dangerous_capabilities_require_live_consent_and_an_indicator() {
        // The security-critical classification in this file. If a future edit
        // quietly moves camera out of this set, this test fails.
        for id in [
            CapabilityId::Camera,
            CapabilityId::Microphone,
            CapabilityId::Location,
            CapabilityId::ScreenCapture,
        ] {
            assert!(id.requires_live_consent(), "{id} must require live consent");
            assert!(
                id.requires_active_indicator(),
                "{id} must show an indicator"
            );
        }
        assert!(CapabilityId::InputGlobal.requires_live_consent());
        assert!(CapabilityId::AiCloud.requires_live_consent());

        // ...and the harmless ones must not, or every prompt becomes noise and
        // users click through all of them.
        assert!(!CapabilityId::Notify.requires_live_consent());
        assert!(!CapabilityId::Gpu.requires_live_consent());
        assert!(!CapabilityId::AudioOutput.requires_live_consent());
    }

    #[test]
    fn keylogging_and_screen_reading_are_classified_critical() {
        assert_eq!(CapabilityId::InputGlobal.risk(), Risk::Critical);
        assert_eq!(CapabilityId::ScreenCapture.risk(), Risk::Critical);
        assert_eq!(CapabilityId::ProcessDebug.risk(), Risk::Critical);
        assert_eq!(CapabilityId::Camera.risk(), Risk::High);
        assert_eq!(CapabilityId::Notify.risk(), Risk::Low);
    }

    // ---- scope parsing -----------------------------------------------------

    #[test]
    fn accepts_well_formed_storage_uris() {
        for good in [
            "user://Documents",
            "user://Documents/tax/2026",
            "app://com.example.app/data",
            "cache://com.example.app",
            "secure://com.example.app/token",
            "media://Pictures",
        ] {
            assert!(Scope::parse_uri(good).is_ok(), "should accept {good}");
        }
    }

    #[test]
    fn rejects_traversal_rather_than_resolving_it() {
        // The single most important test in this file.
        for hostile in [
            "user://Documents/../../etc/shadow",
            "user://..",
            "user://./Documents",
            "user://Documents/./secret",
            "user://a/../../..",
        ] {
            assert_eq!(
                Scope::parse_uri(hostile),
                Err(ScopeError::NotNormalised),
                "must reject traversal: {hostile}"
            );
        }
    }

    #[test]
    fn rejects_control_characters_and_unknown_schemes() {
        assert_eq!(
            Scope::parse_uri("user://a\0b"),
            Err(ScopeError::IllegalCharacter)
        );
        assert_eq!(
            Scope::parse_uri("user://a\nb"),
            Err(ScopeError::IllegalCharacter)
        );
        assert_eq!(
            Scope::parse_uri("/etc/passwd"),
            Err(ScopeError::UnknownScheme)
        );
        assert_eq!(
            Scope::parse_uri("file:///etc/passwd"),
            Err(ScopeError::UnknownScheme)
        );
        assert_eq!(
            Scope::parse_uri("C:\\Windows"),
            Err(ScopeError::UnknownScheme)
        );
        assert_eq!(Scope::parse_uri(""), Err(ScopeError::Empty));
    }

    #[test]
    fn trailing_slashes_are_normalised_away() {
        assert_eq!(
            Scope::parse_uri("user://Documents/"),
            Scope::parse_uri("user://Documents")
        );
    }

    // ---- containment: the blast-radius property ---------------------------

    #[test]
    fn prefix_containment_is_segment_aware() {
        let grant = Scope::parse_uri("user://Documents").expect("valid");

        // Contained.
        for req in [
            "user://Documents",
            "user://Documents/tax",
            "user://Documents/a/b/c.txt",
        ] {
            let r = Scope::parse_uri(req).expect("valid");
            assert!(grant.contains(&r), "{grant} should contain {r}");
        }

        // NOT contained. A naive starts_with() would wrongly allow the first
        // two, handing an app every folder whose name begins with "Documents".
        for req in [
            "user://DocumentsSecret",
            "user://Documents2",
            "user://Doc",
            "user://Pictures",
            "user://",
        ] {
            let r = Scope::parse_uri(req).expect("valid");
            assert!(!grant.contains(&r), "{grant} must NOT contain {r}");
        }
    }

    #[test]
    fn the_root_scope_contains_everything_beneath_it_but_nothing_beside_it() {
        // Regression: normalising "user://" by trimming trailing slashes yielded
        // "user:", and the segment-boundary rule then required a separator that
        // the root already supplies. Both bugs turned a "read all my files"
        // grant into one that authorised nothing.
        let root = Scope::parse_uri("user://").expect("valid");
        assert_eq!(
            root.as_str(),
            "user://",
            "the scheme must survive normalisation"
        );

        for req in [
            "user://Documents",
            "user://Documents/tax/2026.pdf",
            "user://a",
        ] {
            let r = Scope::parse_uri(req).expect("valid");
            assert!(root.contains(&r), "root should contain {r}");
        }

        // A different namespace is NOT beneath the user root.
        let other = Scope::parse_uri("app://com.example.app/data").expect("valid");
        assert!(!root.contains(&other));
        let media = Scope::parse_uri("media://Pictures").expect("valid");
        assert!(!root.contains(&media));

        // And nothing beneath the root contains the root.
        let docs = Scope::parse_uri("user://Documents").expect("valid");
        assert!(!docs.contains(&root));
    }

    #[test]
    fn narrow_grant_does_not_authorise_its_parent() {
        // Escalation check: a grant to a subfolder must not reach upward.
        let grant = Scope::parse_uri("user://Documents/tax").expect("valid");
        let parent = Scope::parse_uri("user://Documents").expect("valid");
        assert!(!grant.contains(&parent));
        assert!(parent.contains(&grant));
    }

    #[test]
    fn any_grant_covers_everything_but_a_specific_grant_never_covers_any() {
        let specific = Scope::parse_uri("user://Documents").expect("valid");
        assert!(Scope::Any.contains(&specific));
        assert!(!specific.contains(&Scope::Any));
        assert!(Scope::Any.contains(&Scope::Any));
    }

    #[test]
    fn scope_kinds_never_cross() {
        let uri = Scope::parse_uri("user://Documents").expect("valid");
        let host = Scope::parse_host("Documents").expect("valid");
        assert!(!uri.contains(&host));
        assert!(!host.contains(&uri));
    }

    #[test]
    fn host_wildcards_match_subdomains_only() {
        let wild = Scope::parse_host(".example.com").expect("valid");
        for req in ["api.example.com", "a.b.example.com", "example.com"] {
            let r = Scope::parse_host(req).expect("valid");
            assert!(wild.contains(&r), "{wild} should contain {r}");
        }
        // The classic bypass: a domain that merely ends with the same letters.
        for req in ["notexample.com", "evil-example.com", "example.com.evil.net"] {
            let r = Scope::parse_host(req).expect("valid");
            assert!(!wild.contains(&r), "{wild} must NOT contain {r}");
        }

        let exact = Scope::parse_host("example.com").expect("valid");
        assert!(!exact.contains(&Scope::parse_host("api.example.com").expect("valid")));
    }

    #[test]
    fn host_matching_is_case_insensitive() {
        let a = Scope::parse_host("Example.COM").expect("valid");
        let b = Scope::parse_host("example.com").expect("valid");
        assert!(a.contains(&b));
    }

    // ---- capability construction ------------------------------------------

    #[test]
    fn never_scoped_capabilities_refuse_a_scope() {
        // "camera, but only for user://Documents" is not a coherent grant.
        // Refusing the combination stops a grant that READS as narrow from
        // actually being unbounded.
        assert_eq!(CapabilityId::Camera.scoping(), Scoping::Never);
        assert!(Capability::unscoped(CapabilityId::Camera).is_some());
        assert!(Capability::new(
            CapabilityId::Camera,
            Scope::parse_uri("user://Documents").expect("valid")
        )
        .is_none());
    }

    #[test]
    fn filesystem_capabilities_always_require_an_explicit_scope() {
        assert_eq!(CapabilityId::FsRead.scoping(), Scoping::Required);
        // There is no way to write an unbounded filesystem grant as `Any`.
        assert!(Capability::unscoped(CapabilityId::FsRead).is_none());
        assert!(Capability::unscoped(CapabilityId::FsWrite).is_none());

        // "All the user's files" must be spelled out as the root scope, so the
        // permission UI can show it for what it is.
        let root = Capability::new(
            CapabilityId::FsRead,
            Scope::parse_uri("user://").expect("valid"),
        )
        .expect("root scope is legal");
        let anything = Capability::new(
            CapabilityId::FsRead,
            Scope::parse_uri("user://Documents/tax").expect("valid"),
        )
        .expect("v");
        assert!(
            root.authorises(&anything),
            "user:// is the root of user storage"
        );
        assert_eq!(root.to_string(), "fs.read@user://");
    }

    #[test]
    fn network_capabilities_may_be_scoped_or_not() {
        // A browser cannot enumerate every host it will visit, so unscoped must
        // be legal. An app that talks to one API should still be pinned to it.
        assert_eq!(CapabilityId::NetClient.scoping(), Scoping::Optional);

        let any = Capability::unscoped(CapabilityId::NetClient).expect("unscoped is legal");
        let pinned = Capability::new(
            CapabilityId::NetClient,
            Scope::parse_host("api.example.com").expect("valid"),
        )
        .expect("scoped is legal");

        assert!(
            any.authorises(&pinned),
            "an unscoped grant covers a specific host"
        );
        assert!(
            !pinned.authorises(&any),
            "a pinned grant must not become unbounded"
        );

        let elsewhere = Capability::new(
            CapabilityId::NetClient,
            Scope::parse_host("evil.example.net").expect("valid"),
        )
        .expect("v");
        assert!(!pinned.authorises(&elsewhere));
    }

    #[test]
    fn authorisation_requires_both_id_and_scope_to_match() {
        let uri = |s: &str| Scope::parse_uri(s).expect("valid");
        let grant = Capability::new(CapabilityId::FsRead, uri("user://Documents")).expect("v");

        let ok = Capability::new(CapabilityId::FsRead, uri("user://Documents/a")).expect("v");
        assert!(grant.authorises(&ok));

        // Same scope, different capability: read does not imply write.
        let write = Capability::new(CapabilityId::FsWrite, uri("user://Documents/a")).expect("v");
        assert!(!grant.authorises(&write));

        // Same capability, outside scope.
        let outside = Capability::new(CapabilityId::FsRead, uri("user://Pictures")).expect("v");
        assert!(!grant.authorises(&outside));
    }

    #[test]
    fn display_form_is_stable() {
        let uri = Scope::parse_uri("user://Documents").expect("valid");
        let c = Capability::new(CapabilityId::FsRead, uri).expect("v");
        assert_eq!(c.to_string(), "fs.read@user://Documents");
        assert_eq!(
            Capability::unscoped(CapabilityId::Camera)
                .expect("v")
                .to_string(),
            "camera"
        );
    }
}
