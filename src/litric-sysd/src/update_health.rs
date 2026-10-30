//! Boot-attested health evidence for one candidate attempt.
//!
//! The evidence file is written by the running system, never by an IPC caller:
//! `litric-sysd --boot` records `booting` after it has opened the generation
//! record and verified the candidate marker, and the serve loop upgrades the
//! same slot/version/attempt to `healthy` once the socket is ready. A caller can
//! ask for a commit; it cannot manufacture the evidence.
//!
//! The format is deliberately small:
//!
//! ```text
//! litric-update-health\t1
//! generation\tB
//! version\t1.1.0
//! attempt\t1
//! at\t1800000000
//! verdict\thealthy
//! checks\troot,state,marker,serving
//! ```
//!
//! The generation id is the pair `(slot, version)`: the slot alone is reused by
//! a later candidate, and the version alone does not say which boot attempt the
//! evidence belongs to. All three fields are required to match at commit time.
//!
//! The write uses [`litric_update::write_atomic`], the same sibling-temp/
//! fsync/rename/directory-fsync primitive as `<system>/state`. A leftover
//! `.health.tmp-*` is never read. Zero-length, truncated, malformed, symlinked
//! or oversized evidence is reported as invalid; none of those paths panic.

use std::fs;
use std::path::{Path, PathBuf};

use litric_core::Version;
use litric_update::{Slot, UpdateError};

/// The first line of the health evidence record.
pub(crate) const HEALTH_PROTOCOL: &str = "litric-update-health";
/// The on-disk health evidence schema.
pub(crate) const HEALTH_VERSION: u8 = 1;
/// The largest health record accepted.
pub(crate) const MAX_HEALTH_BYTES: u64 = 1024;
/// The longest `checks` value accepted.
const MAX_CHECKS_BYTES: usize = 128;

/// What the running system concluded for one candidate attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HealthVerdict {
    /// The boot entrypoint opened the record and verified the candidate marker,
    /// but the serve loop has not attested yet.
    Booting,
    /// The serve loop is ready. This is the only verdict that can open a commit.
    Healthy,
    /// The system recorded a failed attempt.
    Failed,
}

impl HealthVerdict {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Booting => "booting",
            Self::Healthy => "healthy",
            Self::Failed => "failed",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "booting" => Some(Self::Booting),
            "healthy" => Some(Self::Healthy),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// One system-written health observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HealthEvidence {
    /// The candidate slot the evidence is bound to.
    pub(crate) slot: Slot,
    /// The candidate version, the second half of the generation id.
    pub(crate) version: Version,
    /// The exact boot attempt this evidence observed.
    pub(crate) attempt: u8,
    /// When the running system wrote it, seconds since the Unix epoch.
    pub(crate) at: u64,
    /// What it concluded.
    pub(crate) verdict: HealthVerdict,
    /// Stable check names, comma-separated. Diagnostic only.
    pub(crate) checks: String,
}

impl HealthEvidence {
    pub(crate) fn new(
        slot: Slot,
        version: Version,
        attempt: u8,
        at: u64,
        verdict: HealthVerdict,
        checks: &str,
    ) -> Self {
        Self {
            slot,
            version,
            attempt,
            at,
            verdict,
            checks: checks.to_owned(),
        }
    }

    /// Whether this observation is for exactly this candidate attempt.
    pub(crate) fn matches(&self, slot: Slot, version: &Version, attempt: u8) -> bool {
        self.slot == slot && self.version == *version && self.attempt == attempt
    }

    fn encode(&self) -> String {
        format!(
            "{HEALTH_PROTOCOL}\t{HEALTH_VERSION}\ngeneration\t{}\nversion\t{}\nattempt\t{}\nat\t{}\nverdict\t{}\nchecks\t{}\n",
            self.slot.as_str(),
            self.version,
            self.attempt,
            self.at,
            self.verdict.token(),
            self.checks
        )
    }

    fn decode(text: &str) -> core::result::Result<Self, String> {
        let mut lines = text.lines();
        if lines.next() != Some(&format!("{HEALTH_PROTOCOL}\t{HEALTH_VERSION}")) {
            return Err("unknown health evidence header".to_owned());
        }
        let slot = lines
            .next()
            .and_then(|line| line.strip_prefix("generation\t"))
            .ok_or_else(|| "health evidence has no generation".to_owned())
            .and_then(|raw| Slot::parse(raw).map_err(|error| format!("{error:?}")))?;
        let version = lines
            .next()
            .and_then(|line| line.strip_prefix("version\t"))
            .ok_or_else(|| "health evidence has no version".to_owned())
            .and_then(|raw| Version::parse(raw).map_err(|error| format!("{error}")))?;
        let attempt = lines
            .next()
            .and_then(|line| line.strip_prefix("attempt\t"))
            .ok_or_else(|| "health evidence has no attempt".to_owned())
            .and_then(|raw| {
                raw.parse::<u8>()
                    .map_err(|_| "bad health attempt".to_owned())
            })?;
        if attempt == 0 {
            return Err("health evidence has attempt zero".to_owned());
        }
        let at = lines
            .next()
            .and_then(|line| line.strip_prefix("at\t"))
            .ok_or_else(|| "health evidence has no timestamp".to_owned())
            .and_then(|raw| {
                raw.parse::<u64>()
                    .map_err(|_| "bad health timestamp".to_owned())
            })?;
        let verdict = lines
            .next()
            .and_then(|line| line.strip_prefix("verdict\t"))
            .ok_or_else(|| "health evidence has no verdict".to_owned())
            .and_then(|raw| {
                HealthVerdict::parse(raw).ok_or_else(|| "unknown health verdict".to_owned())
            })?;
        let checks = lines
            .next()
            .and_then(|line| line.strip_prefix("checks\t"))
            .ok_or_else(|| "health evidence has no checks".to_owned())?
            .to_owned();
        if lines.next().is_some() {
            return Err("health evidence has trailing records".to_owned());
        }
        if !valid_checks(&checks) {
            return Err("health evidence has invalid checks".to_owned());
        }
        Ok(Self {
            slot,
            version,
            attempt,
            at,
            verdict,
            checks,
        })
    }
}

fn valid_checks(checks: &str) -> bool {
    !checks.is_empty()
        && checks.len() <= MAX_CHECKS_BYTES
        && checks.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b',' || byte == b'-'
        })
}

/// The result of reading the evidence path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HealthRead {
    /// No record exists at all.
    Missing,
    /// A complete record exists.
    Valid(HealthEvidence),
    /// A record exists but cannot be trusted.
    Invalid(String),
}

pub(crate) fn path(root: &Path) -> PathBuf {
    root.join("system").join("health")
}

/// Read the evidence without following a symlink or reading a temporary.
///
/// This is deliberately total: a permission error, a truncated record or an
/// oversized file is `Invalid`, not an error the caller has to remember to
/// handle. A missing path is `Missing`.
pub(crate) fn read(root: &Path) -> HealthRead {
    let path = path(root);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return HealthRead::Missing,
        Err(error) => {
            return HealthRead::Invalid(format!("cannot inspect health evidence: {error}"))
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return HealthRead::Invalid("health evidence is not a regular file".to_owned());
    }
    if metadata.len() == 0 {
        return HealthRead::Invalid("health evidence is empty".to_owned());
    }
    if metadata.len() > MAX_HEALTH_BYTES {
        return HealthRead::Invalid("health evidence is oversized".to_owned());
    }
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => return HealthRead::Invalid(format!("cannot read health evidence: {error}")),
    };
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_HEALTH_BYTES {
        return HealthRead::Invalid("health evidence grew while being read".to_owned());
    }
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(_) => return HealthRead::Invalid("health evidence is not UTF-8".to_owned()),
    };
    match HealthEvidence::decode(text) {
        Ok(evidence) => HealthRead::Valid(evidence),
        Err(reason) => HealthRead::Invalid(reason),
    }
}

/// Write one complete evidence record through the shared atomic primitive.
pub(crate) fn save(root: &Path, evidence: &HealthEvidence) -> Result<(), UpdateError> {
    litric_update::write_atomic(&path(root), evidence.encode().as_bytes())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "litric-update-health-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("system")).expect("system dir");
        root
    }

    fn evidence() -> HealthEvidence {
        HealthEvidence::new(
            Slot::B,
            Version::parse("1.1.0").expect("version"),
            2,
            1_800_000_000,
            HealthVerdict::Healthy,
            "root,state,marker,serving",
        )
    }

    #[test]
    fn evidence_round_trips_and_is_bound_to_all_three_fields() {
        let evidence = evidence();
        let decoded = HealthEvidence::decode(&evidence.encode()).expect("decode");
        assert_eq!(decoded, evidence);
        assert!(decoded.matches(Slot::B, &Version::parse("1.1.0").expect("version"), 2));
        assert!(!decoded.matches(Slot::A, &Version::parse("1.1.0").expect("version"), 2));
        assert!(!decoded.matches(Slot::B, &Version::parse("1.2.0").expect("version"), 2));
        assert!(!decoded.matches(Slot::B, &Version::parse("1.1.0").expect("version"), 1));
    }

    #[test]
    fn a_truncated_empty_or_overlong_record_is_invalid_not_a_panic() {
        for text in [
            "",
            "litric-update-health\t1\n",
            "litric-update-health\t1\nversion\t1.1.0\n",
            "litric-update-health\t2\n",
        ] {
            assert!(HealthEvidence::decode(text).is_err(), "{text:?}");
        }
        let root = root("invalid");
        fs::write(path(&root), "litric-update-health\t1\n").expect("truncated");
        assert!(matches!(read(&root), HealthRead::Invalid(_)));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_temporary_is_never_read_as_evidence() {
        let root = root("temporary");
        fs::write(
            root.join("system/.health.tmp-999"),
            evidence().encode().as_bytes(),
        )
        .expect("temporary");
        assert_eq!(read(&root), HealthRead::Missing);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_is_atomic_owner_only_and_visible_to_read() {
        let root = root("save");
        let evidence = evidence();
        save(&root, &evidence).expect("save");
        assert_eq!(read(&root), HealthRead::Valid(evidence));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(path(&root))
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
        let _ = fs::remove_dir_all(root);
    }
}
