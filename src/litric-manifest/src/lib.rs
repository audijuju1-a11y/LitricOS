//! LitricOS application manifests.
//!
//! # Layer position
//!
//! **Layer 4 (Runtime / SDK).** Depends downward on `litric-core` and
//! `litric-caps`. No I/O: it turns bytes into a validated declaration, and the
//! package service decides what to do with it.
//!
//! # Why the format is boring on purpose
//!
//! This parser runs **before any trust decision**. It reads bytes from a package
//! that has not been installed, may not be signed, and may have been written
//! specifically to attack it (RUNTIME section 26, load-time integrity). Every
//! feature in the grammar is attack surface.
//!
//! So the format is a strict INI subset — the same shape as the systemd and
//! mkosi files already used throughout this project. No nesting, no references,
//! no includes, no escapes, no types beyond string and list. A YAML or TOML
//! parser is tens of thousands of lines that must be perfect; this is a few
//! hundred that can be read in one sitting.
//!
//! # The rule that shapes validation
//!
//! **What the manifest declares is exactly what the installer shows the user,
//! and exactly what the application gets.** So:
//!
//! * An unknown capability is a hard error, never ignored. Ignoring it would
//!   mean a manifest could request something the install dialog does not
//!   display.
//! * A duplicate single-valued key is a hard error, not last-wins. Last-wins is
//!   how `CONFIG_LSM=""` sat above the real value in this project's own kernel
//!   config for weeks (BUG-0003).
//! * Anything unparseable fails the whole manifest. A partially understood
//!   declaration of authority is worth less than none.

#![forbid(unsafe_code)]

use core::fmt;
use litric_caps::{Capability, CapabilityId, Risk, Scope};
use litric_core::{AbiVersion, AppId, Version};

/// Resource class, which selects the cgroup limits an application runs under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ResourceClass {
    /// Small utilities. Tight limits.
    Light,
    /// The default.
    #[default]
    Standard,
    /// Games. High CPU and IO weight, generous memory.
    Game,
    /// AI workloads. Large memory, low priority against the shell.
    Ai,
}

impl ResourceClass {
    /// Parse the manifest token.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "light" => Self::Light,
            "standard" => Self::Standard,
            "game" => Self::Game,
            "ai" => Self::Ai,
            _ => return None,
        })
    }

    /// The manifest token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Standard => "standard",
            Self::Game => "game",
            Self::Ai => "ai",
        }
    }
}

impl fmt::Display for ResourceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Latency class, which selects scheduling treatment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum LatencyClass {
    /// Background work. No latency guarantees.
    Normal,
    /// Responds to user input; scheduled ahead of background work.
    #[default]
    Interactive,
    /// Real-time audio callbacks. Subject to the Tier A rules in RUNTIME §16:
    /// no blocking locks, no allocation, no filesystem I/O on the callback.
    RealtimeAudio,
}

impl LatencyClass {
    /// Parse the manifest token.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "normal" => Self::Normal,
            "interactive" => Self::Interactive,
            "realtime-audio" => Self::RealtimeAudio,
            _ => return None,
        })
    }

    /// The manifest token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Interactive => "interactive",
            Self::RealtimeAudio => "realtime-audio",
        }
    }
}

impl fmt::Display for LatencyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Why a manifest was rejected.
///
/// Every variant carries the line number, because a developer whose package was
/// refused needs to know which line to fix.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestError {
    /// The manifest was empty.
    Empty,
    /// Larger than [`Manifest::MAX_LEN`].
    TooLarge {
        /// Actual size in bytes.
        len: usize,
    },
    /// More lines than [`Manifest::MAX_LINES`].
    TooManyLines,
    /// Not valid UTF-8, or containing a control character.
    NotText {
        /// Line number, 1-based.
        line: usize,
    },
    /// A section header was malformed.
    BadSection {
        /// Line number, 1-based.
        line: usize,
    },
    /// A section name the schema does not define.
    UnknownSection {
        /// Line number, 1-based.
        line: usize,
        /// The name found.
        name: String,
    },
    /// A line that is neither a comment, a section, nor `key=value`.
    BadLine {
        /// Line number, 1-based.
        line: usize,
    },
    /// A key the schema does not define in that section.
    UnknownKey {
        /// Line number, 1-based.
        line: usize,
        /// The key found.
        key: String,
    },
    /// A key that may appear once appeared twice.
    DuplicateKey {
        /// Line number of the second occurrence, 1-based.
        line: usize,
        /// The key.
        key: String,
        /// Line number of the first occurrence.
        first: usize,
    },
    /// A key appeared outside any section.
    KeyBeforeSection {
        /// Line number, 1-based.
        line: usize,
    },
    /// A required key was absent.
    MissingKey {
        /// Section it belongs to.
        section: &'static str,
        /// The key.
        key: &'static str,
    },
    /// A value failed its own validation.
    BadValue {
        /// Line number, 1-based.
        line: usize,
        /// The key.
        key: String,
        /// Why.
        reason: String,
    },
    /// A capability string the system does not define.
    UnknownCapability {
        /// Line number, 1-based.
        line: usize,
        /// The token found.
        token: String,
    },
    /// The same capability was requested twice.
    DuplicateCapability {
        /// Line number, 1-based.
        line: usize,
        /// The capability.
        token: String,
    },
    /// More capabilities than [`Manifest::MAX_CAPABILITIES`].
    TooManyCapabilities,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "manifest is empty"),
            Self::TooLarge { len } => {
                write!(f, "manifest is {len} bytes, maximum {}", Manifest::MAX_LEN)
            }
            Self::TooManyLines => write!(f, "manifest has more than {} lines", Manifest::MAX_LINES),
            Self::NotText { line } => write!(f, "line {line}: contains a control character"),
            Self::BadSection { line } => write!(f, "line {line}: malformed section header"),
            Self::UnknownSection { line, name } => {
                write!(f, "line {line}: unknown section [{name}]")
            }
            Self::BadLine { line } => write!(f, "line {line}: expected key=value"),
            Self::UnknownKey { line, key } => write!(f, "line {line}: unknown key {key}"),
            Self::DuplicateKey { line, key, first } => {
                write!(f, "line {line}: {key} already set on line {first}")
            }
            Self::KeyBeforeSection { line } => write!(f, "line {line}: key outside any section"),
            Self::MissingKey { section, key } => write!(f, "[{section}] {key} is required"),
            Self::BadValue { line, key, reason } => write!(f, "line {line}: {key}: {reason}"),
            Self::UnknownCapability { line, token } => {
                write!(f, "line {line}: unknown capability {token:?}")
            }
            Self::DuplicateCapability { line, token } => {
                write!(f, "line {line}: capability {token:?} requested twice")
            }
            Self::TooManyCapabilities => write!(
                f,
                "more than {} capabilities requested",
                Manifest::MAX_CAPABILITIES
            ),
        }
    }
}

impl std::error::Error for ManifestError {}

/// A validated application manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    id: AppId,
    name: String,
    version: Version,
    publisher: String,
    abi: AbiVersion,
    capabilities: Vec<Capability>,
    resource_class: ResourceClass,
    latency_class: LatencyClass,
    main: String,
}

impl Manifest {
    /// Maximum manifest size in bytes.
    pub const MAX_LEN: usize = 64 * 1024;
    /// Maximum number of lines.
    pub const MAX_LINES: usize = 2000;
    /// Maximum number of requested capabilities.
    ///
    /// An application asking for more than this is either confused or hostile,
    /// and either way the install dialog would be unreadable — which is itself
    /// an attack, because an unreadable list is one nobody checks.
    pub const MAX_CAPABILITIES: usize = 64;

    /// The application identity.
    #[must_use]
    pub const fn id(&self) -> &AppId {
        &self.id
    }
    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Version.
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }
    /// Publisher, as claimed. **Unverified** until the signature is checked.
    #[must_use]
    pub fn publisher(&self) -> &str {
        &self.publisher
    }
    /// Target ABI.
    #[must_use]
    pub const fn abi(&self) -> AbiVersion {
        self.abi
    }
    /// Requested capabilities, in declaration order.
    #[must_use]
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    /// Resource class.
    #[must_use]
    pub const fn resource_class(&self) -> ResourceClass {
        self.resource_class
    }
    /// Latency class.
    #[must_use]
    pub const fn latency_class(&self) -> LatencyClass {
        self.latency_class
    }
    /// Entry point path within the package.
    #[must_use]
    pub fn main(&self) -> &str {
        &self.main
    }

    /// Whether this manifest can run on the given platform ABI.
    #[must_use]
    pub fn is_compatible_with(&self, platform: AbiVersion) -> bool {
        platform.can_host(self.abi)
    }

    /// Capabilities that need a prompt, ordered most dangerous first.
    ///
    /// This is what the install dialog renders. Ordering by risk means the
    /// line a user is most likely to regret is the line they read first, rather
    /// than the twelfth entry after eleven harmless ones.
    #[must_use]
    pub fn prompted_capabilities(&self) -> Vec<&Capability> {
        let mut v: Vec<&Capability> = self
            .capabilities
            .iter()
            .filter(|c| c.id().risk() > Risk::Low)
            .collect();
        v.sort_by(|a, b| b.id().risk().cmp(&a.id().risk()).then(a.id().cmp(&b.id())));
        v
    }

    /// Plain-language lines for the install dialog, most dangerous first.
    #[must_use]
    pub fn consent_summary(&self) -> Vec<String> {
        self.prompted_capabilities()
            .iter()
            .map(|c| match c.scope() {
                Scope::Any => format!("{} — {}", c.id().prompt(), c.id()),
                s => format!("{} ({})", c.id().prompt(), s),
            })
            .collect()
    }

    /// Parse and validate a manifest.
    ///
    /// # Errors
    /// [`ManifestError`] naming the line and the rule violated.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        if text.trim().is_empty() {
            return Err(ManifestError::Empty);
        }
        if text.len() > Self::MAX_LEN {
            return Err(ManifestError::TooLarge { len: text.len() });
        }

        let mut section: Option<&'static str> = None;
        // Single-valued keys: "Section.Key" -> (value, line it was set on).
        let mut scalars: Vec<(String, String, usize)> = Vec::new();
        let mut caps: Vec<(String, usize)> = Vec::new();

        for (idx, raw) in text.lines().enumerate() {
            let line = idx.saturating_add(1);
            if line > Self::MAX_LINES {
                return Err(ManifestError::TooManyLines);
            }

            // Control characters other than the newline that split these lines
            // are rejected: they are how a value smuggles a second line past a
            // reviewer reading the file.
            if raw.bytes().any(|b| b.is_ascii_control() && b != b'\t') {
                return Err(ManifestError::NotText { line });
            }

            let t = raw.trim();
            if t.is_empty() || t.starts_with('#') || t.starts_with(';') {
                continue;
            }

            if t.starts_with('[') {
                let name = t
                    .strip_prefix('[')
                    .and_then(|s| s.strip_suffix(']'))
                    .ok_or(ManifestError::BadSection { line })?
                    .trim();
                section = Some(match name {
                    "Application" => "Application",
                    "Capabilities" => "Capabilities",
                    "Resources" => "Resources",
                    "Entrypoints" => "Entrypoints",
                    other => {
                        return Err(ManifestError::UnknownSection {
                            line,
                            name: other.to_owned(),
                        })
                    }
                });
                continue;
            }

            let Some((key, value)) = t.split_once('=') else {
                return Err(ManifestError::BadLine { line });
            };
            let key = key.trim();
            let value = value.trim();
            let Some(sec) = section else {
                return Err(ManifestError::KeyBeforeSection { line });
            };

            // `Request` is the only repeatable key in the schema.
            if sec == "Capabilities" && key == "Request" {
                caps.push((value.to_owned(), line));
                continue;
            }

            let known = matches!(
                (sec, key),
                (
                    "Application",
                    "Id" | "Name" | "Version" | "Publisher" | "Abi"
                ) | ("Resources", "Class" | "LatencyClass")
                    | ("Entrypoints", "Main")
            );
            if !known {
                return Err(ManifestError::UnknownKey {
                    line,
                    key: key.to_owned(),
                });
            }

            let full = format!("{sec}.{key}");
            if let Some((_, _, first)) = scalars.iter().find(|(k, _, _)| *k == full) {
                return Err(ManifestError::DuplicateKey {
                    line,
                    key: key.to_owned(),
                    first: *first,
                });
            }
            scalars.push((full, value.to_owned(), line));
        }

        let get = |k: &str| -> Option<(&str, usize)> {
            scalars
                .iter()
                .find(|(key, _, _)| key == k)
                .map(|(_, v, l)| (v.as_str(), *l))
        };
        let require = |k: &'static str, section: &'static str, key: &'static str| {
            get(k).ok_or(ManifestError::MissingKey { section, key })
        };

        // --- [Application] --------------------------------------------------
        let (id_raw, id_line) = require("Application.Id", "Application", "Id")?;
        let id = AppId::parse(id_raw).map_err(|e| ManifestError::BadValue {
            line: id_line,
            key: "Id".into(),
            reason: e.to_string(),
        })?;

        let (name, name_line) = require("Application.Name", "Application", "Name")?;
        if name.is_empty() || name.chars().count() > 128 {
            return Err(ManifestError::BadValue {
                line: name_line,
                key: "Name".into(),
                reason: "must be 1 to 128 characters".into(),
            });
        }

        let (ver_raw, ver_line) = require("Application.Version", "Application", "Version")?;
        let version = Version::parse(ver_raw).map_err(|e| ManifestError::BadValue {
            line: ver_line,
            key: "Version".into(),
            reason: e.to_string(),
        })?;

        let (publisher, pub_line) = require("Application.Publisher", "Application", "Publisher")?;
        if publisher.is_empty() || publisher.chars().count() > 128 {
            return Err(ManifestError::BadValue {
                line: pub_line,
                key: "Publisher".into(),
                reason: "must be 1 to 128 characters".into(),
            });
        }

        let (abi_raw, abi_line) = require("Application.Abi", "Application", "Abi")?;
        let abi = AbiVersion::parse(abi_raw).map_err(|e| ManifestError::BadValue {
            line: abi_line,
            key: "Abi".into(),
            reason: e.to_string(),
        })?;

        // --- [Entrypoints] --------------------------------------------------
        let (main, main_line) = require("Entrypoints.Main", "Entrypoints", "Main")?;
        if main.is_empty() || !main.starts_with('/') || main.contains("..") || main.contains('\\') {
            return Err(ManifestError::BadValue {
                line: main_line,
                key: "Main".into(),
                reason: "must be an absolute package-relative path with no '..'".into(),
            });
        }

        // --- [Resources] ----------------------------------------------------
        let resource_class = match get("Resources.Class") {
            Some((v, l)) => ResourceClass::parse(v).ok_or(ManifestError::BadValue {
                line: l,
                key: "Class".into(),
                reason: "expected light, standard, game or ai".into(),
            })?,
            None => ResourceClass::default(),
        };
        let latency_class = match get("Resources.LatencyClass") {
            Some((v, l)) => LatencyClass::parse(v).ok_or(ManifestError::BadValue {
                line: l,
                key: "LatencyClass".into(),
                reason: "expected normal, interactive or realtime-audio".into(),
            })?,
            None => LatencyClass::default(),
        };

        // --- [Capabilities] -------------------------------------------------
        if caps.len() > Self::MAX_CAPABILITIES {
            return Err(ManifestError::TooManyCapabilities);
        }
        let mut capabilities: Vec<Capability> = Vec::with_capacity(caps.len());
        for (token, line) in &caps {
            let cap = parse_capability(token).ok_or_else(|| ManifestError::UnknownCapability {
                line: *line,
                token: token.clone(),
            })?;
            if capabilities.contains(&cap) {
                return Err(ManifestError::DuplicateCapability {
                    line: *line,
                    token: token.clone(),
                });
            }
            capabilities.push(cap);
        }

        Ok(Self {
            id,
            name: name.to_owned(),
            version,
            publisher: publisher.to_owned(),
            abi,
            capabilities,
            resource_class,
            latency_class,
            main: main.to_owned(),
        })
    }
}

/// Parse `camera` or `fs.read@user://Documents`.
fn parse_capability(token: &str) -> Option<Capability> {
    match token.split_once('@') {
        Some((id, scope)) => {
            let id = CapabilityId::parse(id.trim())?;
            let scope = scope.trim();
            // A host scope has no "://"; a storage scope does.
            let s = if scope.contains("://") {
                Scope::parse_uri(scope).ok()?
            } else {
                Scope::parse_host(scope).ok()?
            };
            Capability::new(id, s)
        }
        None => Capability::unscoped(CapabilityId::parse(token.trim())?),
    }
}

impl fmt::Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ({})", self.id, self.version, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "\
# LitricOS application manifest
[Application]
Id=com.example.editor
Name=Example Editor
Version=3.2.0
Publisher=Example Studio
Abi=litric-native-v1

[Capabilities]
Request=fs.read@user://Documents
Request=fs.write@user://Documents
Request=net.client
Request=notify

[Resources]
Class=standard
LatencyClass=interactive

[Entrypoints]
Main=/bin/editor
";

    // ---- happy path --------------------------------------------------------

    #[test]
    fn parses_a_well_formed_manifest() {
        let m = Manifest::parse(GOOD).expect("should parse");
        assert_eq!(m.id().as_str(), "com.example.editor");
        assert_eq!(m.name(), "Example Editor");
        assert_eq!(m.version().to_string(), "3.2.0");
        assert_eq!(m.publisher(), "Example Studio");
        assert_eq!(m.abi(), AbiVersion::new(1, 0));
        assert_eq!(m.resource_class(), ResourceClass::Standard);
        assert_eq!(m.latency_class(), LatencyClass::Interactive);
        assert_eq!(m.main(), "/bin/editor");
        assert_eq!(m.capabilities().len(), 4);
    }

    #[test]
    fn optional_sections_take_documented_defaults() {
        let minimal = "\
[Application]
Id=com.example.app
Name=App
Version=1.0.0
Publisher=Someone
Abi=litric-native-v1
[Entrypoints]
Main=/bin/app
";
        let m = Manifest::parse(minimal).expect("should parse");
        assert_eq!(m.resource_class(), ResourceClass::Standard);
        assert_eq!(m.latency_class(), LatencyClass::Interactive);
        assert!(
            m.capabilities().is_empty(),
            "no declaration means no authority"
        );
    }

    #[test]
    fn comments_blank_lines_and_whitespace_are_tolerated() {
        let messy = "\
   # leading comment
; another style

[Application]
  Id = com.example.app
Name=App
Version=1.0.0
Publisher=X
Abi=litric-native-v1

[Entrypoints]
Main=/bin/app
";
        let m = Manifest::parse(messy).expect("should parse");
        assert_eq!(m.id().as_str(), "com.example.app");
    }

    // ---- the rule: never silently ignore ----------------------------------

    #[test]
    fn an_unknown_capability_is_a_hard_error() {
        // The load-bearing test. If an unknown capability were ignored, a
        // manifest could request something the install dialog never displays.
        let bad = GOOD.replace("Request=notify", "Request=root.everything");
        let e = Manifest::parse(&bad).expect_err("must reject");
        assert!(matches!(e, ManifestError::UnknownCapability { .. }));
        assert!(e.to_string().contains("root.everything"));
    }

    #[test]
    fn an_unknown_key_or_section_is_a_hard_error() {
        let bad = GOOD.replace("[Resources]", "[Secrets]");
        assert!(matches!(
            Manifest::parse(&bad),
            Err(ManifestError::UnknownSection { .. })
        ));

        let bad = GOOD.replace("Class=standard", "Privileged=true");
        assert!(matches!(
            Manifest::parse(&bad),
            Err(ManifestError::UnknownKey { .. })
        ));
    }

    #[test]
    fn a_duplicate_key_is_rejected_not_last_wins() {
        // BUG-0003 in miniature: `CONFIG_LSM=""` sat above the real value and
        // last-wins hid it. A manifest with two Ids must not quietly pick one.
        let bad = GOOD.replace(
            "Id=com.example.editor",
            "Id=com.example.editor\nId=com.evil.app",
        );
        let e = Manifest::parse(&bad).expect_err("must reject");
        assert!(matches!(e, ManifestError::DuplicateKey { .. }), "got {e:?}");
    }

    #[test]
    fn a_duplicate_capability_is_rejected() {
        let bad = GOOD.replace("Request=notify", "Request=notify\nRequest=notify");
        assert!(matches!(
            Manifest::parse(&bad),
            Err(ManifestError::DuplicateCapability { .. })
        ));
    }

    // ---- required fields ---------------------------------------------------

    #[test]
    fn every_required_field_is_actually_required() {
        for (needle, key) in [
            ("Id=com.example.editor\n", "Id"),
            ("Name=Example Editor\n", "Name"),
            ("Version=3.2.0\n", "Version"),
            ("Publisher=Example Studio\n", "Publisher"),
            ("Abi=litric-native-v1\n", "Abi"),
            ("Main=/bin/editor\n", "Main"),
        ] {
            let bad = GOOD.replace(needle, "");
            let e = Manifest::parse(&bad).expect_err("must reject missing {key}");
            assert!(
                matches!(e, ManifestError::MissingKey { key: k, .. } if k == key),
                "removing {key} gave {e:?}"
            );
        }
    }

    // ---- hostile input -----------------------------------------------------

    #[test]
    fn rejects_a_malformed_application_id() {
        for bad_id in [
            "com.example",
            "../../etc/passwd",
            "os.litric.shell",
            "com.exаmple.app",
            "",
        ] {
            let bad = GOOD.replace("Id=com.example.editor", &format!("Id={bad_id}"));
            assert!(Manifest::parse(&bad).is_err(), "must reject id {bad_id:?}");
        }
    }

    #[test]
    fn rejects_an_entrypoint_that_escapes_the_package() {
        for bad_main in [
            "../../../bin/sh",
            "/bin/../../../etc/shadow",
            "bin/editor",
            "C:\\Windows\\system32\\cmd.exe",
            "",
        ] {
            let bad = GOOD.replace("Main=/bin/editor", &format!("Main={bad_main}"));
            assert!(
                Manifest::parse(&bad).is_err(),
                "must reject entrypoint {bad_main:?}"
            );
        }
    }

    #[test]
    fn rejects_a_capability_scope_with_traversal() {
        let bad = GOOD.replace(
            "Request=fs.read@user://Documents",
            "Request=fs.read@user://Documents/../../etc",
        );
        assert!(matches!(
            Manifest::parse(&bad),
            Err(ManifestError::UnknownCapability { .. })
        ));
    }

    #[test]
    fn rejects_control_characters_used_to_smuggle_lines() {
        let bad = "\
[Application]
Id=com.example.app\rId=com.evil.app
Name=App
Version=1.0.0
Publisher=X
Abi=litric-native-v1
[Entrypoints]
Main=/bin/app
";
        assert!(matches!(
            Manifest::parse(bad),
            Err(ManifestError::NotText { .. })
        ));
    }

    #[test]
    fn enforces_size_limits() {
        assert!(matches!(Manifest::parse(""), Err(ManifestError::Empty)));
        assert!(matches!(
            Manifest::parse("   \n\n  "),
            Err(ManifestError::Empty)
        ));

        let huge = "x".repeat(Manifest::MAX_LEN + 1);
        assert!(matches!(
            Manifest::parse(&huge),
            Err(ManifestError::TooLarge { .. })
        ));

        let mut many = String::from("[Capabilities]\n");
        for _ in 0..(Manifest::MAX_CAPABILITIES + 5) {
            many.push_str("Request=notify\n");
        }
        assert!(Manifest::parse(&many).is_err());
    }

    #[test]
    fn a_key_before_any_section_is_rejected() {
        let bad = "Id=com.example.app\n[Application]\nName=X\n";
        assert!(matches!(
            Manifest::parse(bad),
            Err(ManifestError::KeyBeforeSection { line: 1 })
        ));
    }

    #[test]
    fn parsing_never_panics_on_arbitrary_input() {
        for cut in 0..GOOD.len() {
            let _ = Manifest::parse(GOOD.get(..cut).unwrap_or(""));
        }
        for pattern in [
            "[[[[[[",
            "]]]]]]",
            "=======",
            "[Application]\n=\n",
            "[Application]\nId=\n",
            "\u{feff}[Application]\n",
            &"=".repeat(5000),
        ] {
            let _ = Manifest::parse(pattern);
        }
    }

    // ---- consent surface ---------------------------------------------------

    #[test]
    fn the_consent_list_is_ordered_most_dangerous_first() {
        let m = Manifest::parse(
            "\
[Application]
Id=com.example.app
Name=App
Version=1.0.0
Publisher=X
Abi=litric-native-v1
[Capabilities]
Request=notify
Request=net.client
Request=camera
Request=input.global
Request=fs.read@user://Documents
[Entrypoints]
Main=/bin/app
",
        )
        .expect("should parse");

        let prompted = m.prompted_capabilities();
        // Low-risk capabilities are not prompted at all.
        assert!(!prompted.iter().any(|c| c.id() == CapabilityId::Notify));
        assert!(!prompted.iter().any(|c| c.id() == CapabilityId::NetClient));
        // The keylogger capability is first.
        assert_eq!(
            prompted.first().map(|c| c.id()),
            Some(CapabilityId::InputGlobal),
            "the most dangerous request must be read first"
        );

        let summary = m.consent_summary();
        assert!(summary
            .first()
            .is_some_and(|s| s.contains("everything you type")));
        assert!(summary.iter().any(|s| s.contains("user://Documents")));
    }

    #[test]
    fn consent_lines_are_human_language_not_tokens() {
        let m = Manifest::parse(GOOD).expect("should parse");
        for line in m.consent_summary() {
            assert!(
                !line.starts_with("fs."),
                "consent line is a raw token: {line}"
            );
            assert!(!line.is_empty());
        }
    }

    // ---- ABI ---------------------------------------------------------------

    #[test]
    fn abi_compatibility_is_asymmetric() {
        let m = Manifest::parse(GOOD).expect("should parse");
        assert!(m.is_compatible_with(AbiVersion::new(1, 0)));
        assert!(m.is_compatible_with(AbiVersion::new(1, 7)));
        assert!(!m.is_compatible_with(AbiVersion::new(2, 0)));
    }

    #[test]
    fn scoped_and_host_capabilities_both_parse() {
        let m = Manifest::parse(
            "\
[Application]
Id=com.example.app
Name=App
Version=1.0.0
Publisher=X
Abi=litric-native-v1
[Capabilities]
Request=fs.read@user://Documents
Request=net.client@api.example.com
Request=camera
[Entrypoints]
Main=/bin/app
",
        )
        .expect("should parse");
        assert_eq!(m.capabilities().len(), 3);
        assert!(m
            .capabilities()
            .iter()
            .any(|c| matches!(c.scope(), Scope::Host(h) if h == "api.example.com")));
    }
}
