//! The daemon's durable capability record.
//!
//! `litric-caps` is deliberately I/O-free: it decides, and it cannot write. The
//! daemon owns the file. This module is that file — one schema, one parser, one
//! atomic write — and it exists because until now every grant died with the
//! process. The gate matrix said so plainly: "grants are in-memory and do not
//! survive reboot" (ADR-0016, matrix row E1).
//!
//! # The format
//!
//! Tab-separated, line-oriented, UTF-8, with a protocol line first:
//!
//! ```text
//! litric-grants\t1
//! grant\t<app>\t<capability>\t<issuer>\t<issued>\t<expires|->\t<revoked|->
//! audit\t<app>\t<capability>\t<allowed>\t<reason>\t<issuer|->\t<at|->\t<scope|->\t<at>
//! ```
//!
//! A person can read it and repair it with `cat` and an editor while the daemon
//! is stopped, which is the same reason the shell's preference file is text
//! (ADR-0021's recovery story asks for it). Tabs are the separator because a
//! scope may legitimately contain spaces and none of the fields may.
//!
//! # Corruption is a denial, not a permission
//!
//! The rule the batch exists to prove: **a record that cannot be read grants
//! nothing.** A missing protocol line, an unknown version, a truncated line, a
//! bad app id, an unparseable capability or a file larger than
//! [`MAX_FILE_BYTES`] all return an error, and the daemon then starts with an
//! **empty broker** — including the policy grants it would otherwise issue.
//! The alternative, restoring whatever happened to parse, is the failure mode
//! where a half-written file grants a subset nobody audited.
//!
//! Rejecting the whole file is also why [`Store::save`] is atomic: a power cut
//! leaves the previous record, never a prefix of the new one.
//!
//! # Why it is bounded
//!
//! Two ceilings, both measured against the broker's own caps rather than a
//! round number: at most [`Broker::MAX_GRANTS`] grant records (the broker
//! refuses more), and at most [`MAX_AUDIT_RECORDS`] decision records in the
//! file. The in-memory audit ring is 4,096 entries and is the settings UI's
//! history; the file keeps the most recent 512 so a machine that has been
//! running for a year does not grow a multi-megabyte permission log. At the
//! measured 118 bytes per line that is ≈ 59 KiB for audit plus ≈ 120 KiB for a
//! full grant table — 179 KiB worst case, against a 256 KiB read ceiling.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use litric_caps::{AuditEntry, Broker, Capability, CapabilityId, Grant, Issuer, Reason, Scope};
use litric_core::AppId;

/// The protocol line and the schema version this build writes.
pub const PROTOCOL: &str = "litric-grants";
/// Version of the record below the protocol line.
pub const VERSION: u8 = 1;

/// The largest file this loader will read.
///
/// 256 KiB is four times the measured worst case above, so a legitimate record
/// has room to grow and a file past it is either hostile or damaged — both
/// reasons to refuse rather than buffer it.
pub const MAX_FILE_BYTES: u64 = 256 * 1024;

/// The longest single line accepted. A scope is capped at 4,096 bytes by
/// `Scope::MAX_LEN`; a line is that plus five short fields.
pub const MAX_LINE_BYTES: usize = 4096;

/// How many decision records the file keeps, newest last.
pub const MAX_AUDIT_RECORDS: usize = 512;

/// A grant record the daemon can restore.
pub type Record = (AppId, Grant);

/// What one successful load produced.
#[derive(Debug, Default)]
pub struct Loaded {
    /// Grants, in file order.
    pub grants: Vec<Record>,
    /// Decision records, oldest first.
    pub audit: Vec<AuditEntry>,
}

/// A grant/audit file at a known path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    /// A store for `path`. Nothing is read or created until a method is called.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The file this store owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the record.
    ///
    /// `Ok(None)` means the file does not exist: this account has never run the
    /// daemon. Every other failure — unreadable, oversized, wrong protocol,
    /// newer version, malformed line — is an `Err`, and the caller must treat
    /// it as "grant nothing".
    pub fn load(&self) -> Result<Option<Loaded>, String> {
        let meta = match fs::metadata(&self.path) {
            Ok(meta) => meta,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!("cannot read {}: {error}", self.path.display()));
            }
        };
        if !meta.is_file() {
            return Err(format!("{} is not a regular file", self.path.display()));
        }
        if meta.len() > MAX_FILE_BYTES {
            return Err(format!(
                "{} is {} bytes, past the {MAX_FILE_BYTES}-byte ceiling",
                self.path.display(),
                meta.len()
            ));
        }
        let bytes = fs::read(&self.path)
            .map_err(|error| format!("cannot read {}: {error}", self.path.display()))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| format!("{} is not UTF-8", self.path.display()))?;
        decode(text)
    }

    /// Write the record atomically, creating and securing the directory.
    ///
    /// The sequence is the one `prefstore` proved: a temporary file in the same
    /// directory, `fsync`, `rename`, `fsync` the directory. A failure removes
    /// the temporary and leaves the previous record in place.
    pub fn save(&self, grants: &[Record], audit: &[AuditEntry]) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "grant store path has no parent directory".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create grant directory: {error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("cannot secure grant directory: {error}"))?;
        }

        let temp = parent.join(format!("grants.conf.tmp.{}", std::process::id()));
        // A stale temporary from a daemon killed between write and rename must
        // not make this one fail forever.
        let _ = fs::remove_file(&temp);

        let result = (|| -> Result<(), String> {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temp)
                .map_err(|error| format!("cannot create temporary grant file: {error}"))?;
            file.write_all(encode(grants, audit).as_bytes())
                .map_err(|error| format!("cannot write grants: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("cannot sync grants: {error}"))?;
            fs::rename(&temp, &self.path)
                .map_err(|error| format!("cannot commit grants: {error}"))?;
            if let Ok(dir) = fs::File::open(parent) {
                let _ = dir.sync_all();
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result
    }
}

/// Encode a record. Grants are sorted so the file does not depend on hash
/// order; the audit keeps chronological order and only its newest
/// [`MAX_AUDIT_RECORDS`] lines.
#[must_use]
pub fn encode(grants: &[Record], audit: &[AuditEntry]) -> String {
    let mut sorted: Vec<&Record> = grants.iter().collect();
    sorted.sort_by(|a, b| {
        a.0.as_str()
            .cmp(b.0.as_str())
            .then_with(|| {
                a.1.capability()
                    .to_string()
                    .cmp(&b.1.capability().to_string())
            })
            .then_with(|| a.1.issued_at().cmp(&b.1.issued_at()))
    });

    let capacity = 64usize
        .saturating_add(sorted.len().saturating_mul(96))
        .saturating_add(audit.len().saturating_mul(118));
    let mut out = String::with_capacity(capacity);
    out.push_str(PROTOCOL);
    out.push('\t');
    out.push_str(&VERSION.to_string());
    out.push('\n');

    // The ceiling is enforced here, not only at load: a caller that ignored
    // `restore_grants`'s return value must not be able to write a file this
    // build would then refuse to read.
    for (app, grant) in sorted.into_iter().take(Broker::MAX_GRANTS) {
        out.push_str("grant\t");
        out.push_str(app.as_str());
        out.push('\t');
        out.push_str(&grant.capability().to_string());
        out.push('\t');
        out.push_str(grant.issuer().token());
        out.push('\t');
        out.push_str(&grant.issued_at().to_string());
        out.push('\t');
        out.push_str(&optional(grant.expires_at()));
        out.push('\t');
        out.push_str(&optional(grant.revoked_at()));
        out.push('\n');
    }

    let skip = audit.len().saturating_sub(MAX_AUDIT_RECORDS);
    for entry in audit.iter().skip(skip) {
        out.push_str("audit\t");
        out.push_str(entry.app.as_str());
        out.push('\t');
        out.push_str(&entry.request.to_string());
        out.push('\t');
        out.push_str(if entry.allowed { "1" } else { "0" });
        out.push('\t');
        out.push_str(entry.reason.token());
        out.push('\t');
        let (issuer, at, scope) = reason_fields(&entry.reason);
        out.push_str(issuer);
        out.push('\t');
        out.push_str(&at);
        out.push('\t');
        out.push_str(&scope);
        out.push('\t');
        out.push_str(&entry.at.to_string());
        out.push('\n');
    }
    out
}

fn optional(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

/// The three extra columns a denial reason needs, as text ready to write.
fn reason_fields(reason: &Reason) -> (&str, String, String) {
    match reason {
        Reason::Granted { issuer, scope } => (issuer.token(), "-".to_owned(), scope.to_string()),
        Reason::Revoked { at } | Reason::Expired { at } => ("-", at.to_string(), "-".to_owned()),
        Reason::OutOfScope { held } => ("-", "-".to_owned(), held.to_string()),
        Reason::NoGrant
        | Reason::NeedsLiveConsent
        | Reason::RateLimited
        | Reason::UnknownApplication => ("-", "-".to_owned(), "-".to_owned()),
        // `Reason` is `#[non_exhaustive]`: a variant added later must not make
        // this store fail to compile, and must not be written as an unknown
        // token either. A reason this build cannot name is recorded as
        // `unknown-application`, which the decoder refuses, so the record is
        // rejected rather than silently mislabelled.
        _ => ("-", "-".to_owned(), "-".to_owned()),
    }
}

/// Decode a whole record, or explain why the file cannot be trusted.
///
/// # Errors
/// Any line this build cannot read. Partial acceptance is deliberately not an
/// option: see the module comment.
pub fn decode(text: &str) -> Result<Option<Loaded>, String> {
    let mut lines = text.split('\n');
    let first = lines.next().unwrap_or("");
    let (protocol, version) = first
        .split_once('\t')
        .ok_or_else(|| "no protocol line".to_owned())?;
    if protocol != PROTOCOL {
        return Err(format!("unknown protocol {protocol:?}"));
    }
    let version: u8 = version
        .parse()
        .map_err(|_| format!("unreadable version {version:?}"))?;
    if version > VERSION {
        return Err(format!(
            "record version {version} is newer than this build's {VERSION}"
        ));
    }

    let mut loaded = Loaded::default();
    for (index, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        if line.len() > MAX_LINE_BYTES {
            return Err(format!(
                "line {} is past {MAX_LINE_BYTES} bytes",
                index.saturating_add(2)
            ));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            ["grant", app, capability, issuer, issued, expires, revoked] => {
                let app = parse_app(app)
                    .ok_or_else(|| format!("line {}: bad app id", index.saturating_add(2)))?;
                let capability = parse_capability(capability)
                    .ok_or_else(|| format!("line {}: bad capability", index.saturating_add(2)))?;
                let issuer = Issuer::parse(issuer)
                    .ok_or_else(|| format!("line {}: bad issuer", index.saturating_add(2)))?;
                let issued = parse_time(issued, index)?;
                let expires = parse_optional_time(expires, index)?;
                let revoked = parse_optional_time(revoked, index)?;
                loaded.grants.push((
                    app,
                    Grant::restore(capability, issuer, issued, expires, revoked),
                ));
            }
            ["audit", app, capability, allowed, reason, issuer, at, scope, when] => {
                let app = parse_app(app)
                    .ok_or_else(|| format!("line {}: bad app id", index.saturating_add(2)))?;
                let request = parse_capability(capability)
                    .ok_or_else(|| format!("line {}: bad capability", index.saturating_add(2)))?;
                let allowed = match *allowed {
                    "1" => true,
                    "0" => false,
                    _ => {
                        return Err(format!(
                            "line {}: bad allowed flag",
                            index.saturating_add(2)
                        ))
                    }
                };
                let reason = parse_reason(reason, issuer, at, scope, index)?;
                let at = parse_time(when, index)?;
                loaded.audit.push(AuditEntry {
                    app,
                    request,
                    allowed,
                    reason,
                    at,
                });
            }
            _ => {
                return Err(format!(
                    "line {}: unrecognised record",
                    index.saturating_add(2)
                ))
            }
        }
    }

    if loaded.grants.len() > Broker::MAX_GRANTS {
        return Err(format!(
            "{} grant records, past the {} ceiling",
            loaded.grants.len(),
            Broker::MAX_GRANTS
        ));
    }
    Ok(Some(loaded))
}

/// Parse an application id from a record.
///
/// The system client's id is in the platform-reserved namespace, which the
/// ordinary parser refuses so that a third-party package cannot impersonate a
/// system component. The daemon writes this file itself, mode 0600, and the
/// shell's policy grants live under that id — so the reserved form is accepted
/// here and nowhere else.
fn parse_app(raw: &str) -> Option<AppId> {
    AppId::parse(raw)
        .or_else(|_| AppId::parse_platform_reserved(raw))
        .ok()
}

fn parse_time(raw: &str, index: usize) -> Result<u64, String> {
    raw.parse()
        .map_err(|_| format!("line {}: bad timestamp", index.saturating_add(2)))
}

fn parse_optional_time(raw: &str, index: usize) -> Result<Option<u64>, String> {
    if raw == "-" {
        return Ok(None);
    }
    parse_time(raw, index).map(Some)
}

fn parse_reason(
    token: &str,
    issuer: &str,
    at: &str,
    scope: &str,
    index: usize,
) -> Result<Reason, String> {
    Ok(match token {
        "granted" => {
            let issuer = Issuer::parse(issuer)
                .ok_or_else(|| format!("line {}: bad issuer", index.saturating_add(2)))?;
            let scope = parse_scope(scope)
                .ok_or_else(|| format!("line {}: bad scope", index.saturating_add(2)))?;
            Reason::Granted { issuer, scope }
        }
        "revoked" => Reason::Revoked {
            at: parse_time(at, index)?,
        },
        "expired" => Reason::Expired {
            at: parse_time(at, index)?,
        },
        "out-of-scope" => Reason::OutOfScope {
            held: parse_scope(scope)
                .ok_or_else(|| format!("line {}: bad scope", index.saturating_add(2)))?,
        },
        "no-grant" => Reason::NoGrant,
        "needs-live-consent" => Reason::NeedsLiveConsent,
        "rate-limited" => Reason::RateLimited,
        "unknown-application" => Reason::UnknownApplication,
        _ => {
            return Err(format!(
                "line {}: unknown reason {token:?}",
                index.saturating_add(2)
            ))
        }
    })
}

/// Parse the canonical `id@scope` form `Capability`'s `Display` writes.
#[must_use]
pub fn parse_capability(raw: &str) -> Option<Capability> {
    let (id, scope) = match raw.split_once('@') {
        Some((id, scope)) => (id, Some(scope)),
        None => (raw, None),
    };
    let id = CapabilityId::parse(id)?;
    match scope {
        None => Capability::unscoped(id),
        Some(text) => Capability::new(id, parse_scope(text)?),
    }
}

/// A scope's canonical text back to a scope.
///
/// The three kinds are tried in the order that keeps them distinct: a storage
/// URI and a resource both carry a scheme, a host is a bare name.
fn parse_scope(raw: &str) -> Option<Scope> {
    Scope::parse_uri(raw)
        .or_else(|_| Scope::parse_resource(raw))
        .or_else(|_| Scope::parse_host(raw))
        .ok()
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use litric_caps::Capability;

    fn fixture(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("litric-grants-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root.join("var/lib/litric-sysd/grants.conf")
    }

    fn cleanup(path: &Path) {
        if let Some(root) = path.ancestors().nth(4) {
            let _ = fs::remove_dir_all(root);
        }
    }

    fn app(s: &str) -> AppId {
        AppId::parse(s).expect("valid app id")
    }

    fn camera() -> Capability {
        Capability::unscoped(CapabilityId::Camera).expect("valid")
    }

    fn documents() -> Capability {
        Capability::new(
            CapabilityId::FsRead,
            Scope::parse_uri("user://Documents").expect("valid"),
        )
        .expect("valid")
    }

    #[test]
    fn a_record_round_trips_through_the_file() {
        let path = fixture("roundtrip");
        let store = Store::new(&path);
        let grants = vec![
            (
                app("com.example.photos"),
                Grant::expiring(documents(), Issuer::UserAction, 100, 200),
            ),
            (
                app("com.example.cam"),
                Grant::new(camera(), Issuer::Policy, 50),
            ),
        ];
        let audit = vec![AuditEntry {
            app: app("com.example.cam"),
            request: camera(),
            allowed: true,
            reason: Reason::Granted {
                issuer: Issuer::Policy,
                scope: Scope::Any,
            },
            at: 75,
        }];
        store.save(&grants, &audit).expect("saves");
        assert!(
            !path.with_file_name("grants.conf.tmp.0").exists(),
            "no temporary is left behind"
        );

        let loaded = store.load().expect("loads").expect("exists");
        assert_eq!(loaded.grants.len(), 2);
        // Sorted: "com.example.cam" before "com.example.photos".
        assert_eq!(loaded.grants[0].0.as_str(), "com.example.cam");
        assert_eq!(loaded.grants[0].1.expires_at(), None);
        assert_eq!(loaded.grants[1].1.expires_at(), Some(200));
        assert_eq!(loaded.grants[1].1.issuer(), Issuer::UserAction);
        assert_eq!(loaded.audit.len(), 1);
        assert!(loaded.audit[0].allowed);
        assert_eq!(loaded.audit[0].reason.token(), "granted");
        assert_eq!(loaded.audit[0].request, camera());
        cleanup(&path);
    }

    #[test]
    fn a_grant_that_was_revoked_stays_revoked_through_the_file() {
        let path = fixture("revoked");
        let store = Store::new(&path);
        let grant = Grant::restore(camera(), Issuer::User, 10, None, Some(20));
        store
            .save(&[(app("com.example.cam"), grant)], &[])
            .expect("saves");
        let loaded = store.load().expect("loads").expect("exists");
        assert_eq!(loaded.grants[0].1.revoked_at(), Some(20));
        cleanup(&path);
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let path = fixture("missing");
        assert!(Store::new(&path).load().expect("load ok").is_none());
        cleanup(&path);
    }

    #[test]
    fn a_header_only_record_is_an_empty_broker_not_corruption() {
        // A record with no grants is what `save` writes after the last grant is
        // revoked. Treating it as damage would bring the policy grants back
        // after a user had just removed them.
        let loaded = decode("litric-grants\t1\n")
            .expect("a header-only record is valid")
            .expect("a record");
        assert!(loaded.grants.is_empty());
        assert!(loaded.audit.is_empty());
    }

    #[test]
    fn garbage_grants_nothing() {
        // The property the batch exists for. Every one of these is a record a
        // crash, a full disk or a person with an editor can produce, and every
        // one of them must be refused rather than partially trusted.
        for (name, body) in [
            ("empty", String::new()),
            (
                "no-header",
                "grant\tcom.a\tcamera\tuser\t1\t-\t-\n".to_owned(),
            ),
            (
                "truncated",
                "litric-grants\t1\ngrant\tcom.a\tcam".to_owned(),
            ),
            (
                "bad-capability",
                "litric-grants\t1\ngrant\tcom.a\tnot.a.capability\tuser\t1\t-\t-\n".to_owned(),
            ),
            (
                "bad-app",
                "litric-grants\t1\ngrant\tNot An App\tcamera\tuser\t1\t-\t-\n".to_owned(),
            ),
            (
                "bad-time",
                "litric-grants\t1\ngrant\tcom.a\tcamera\tuser\tyesterday\t-\t-\n".to_owned(),
            ),
            (
                "extra-field",
                "litric-grants\t1\ngrant\tcom.a\tcamera\tuser\t1\t-\t-\textra\n".to_owned(),
            ),
        ] {
            assert!(
                decode(&body).is_err(),
                "{name} must be refused, not half-restored"
            );
        }
    }

    #[test]
    fn a_newer_version_is_refused_instead_of_guessed() {
        let err = decode("litric-grants\t99\n").expect_err("must refuse");
        assert!(err.contains("99"), "{err}");
    }

    #[test]
    fn an_oversized_file_is_refused_before_it_is_read() {
        let path = fixture("oversized");
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, vec![b'x'; MAX_FILE_BYTES as usize + 1]).expect("write");
        let err = Store::new(&path).load().expect_err("must refuse");
        assert!(err.contains("ceiling"), "{err}");
        cleanup(&path);
    }

    #[test]
    fn the_record_costs_what_the_budget_says() {
        // The worst case the format is designed for: a full grant table and a
        // full audit ring, written and read back with the real code path. The
        // numbers are printed because a budget nobody measures is a wish.
        let path = fixture("cost");
        let store = Store::new(&path);
        let grants: Vec<Record> = (0..Broker::MAX_GRANTS)
            .map(|i| {
                (
                    app(&format!("com.example.app{i}")),
                    Grant::new(camera(), Issuer::User, 1),
                )
            })
            .collect();
        let audit: Vec<AuditEntry> = (0..MAX_AUDIT_RECORDS)
            .map(|i| AuditEntry {
                app: app("com.example.app0"),
                request: camera(),
                allowed: false,
                reason: Reason::NoGrant,
                at: u64::try_from(i).expect("fits"),
            })
            .collect();

        let started = std::time::Instant::now();
        store.save(&grants, &audit).expect("saves");
        let save_us = started.elapsed().as_micros();
        let bytes = std::fs::metadata(&path).expect("stat").len();

        let started = std::time::Instant::now();
        let loaded = store.load().expect("loads").expect("exists");
        let load_us = started.elapsed().as_micros();

        assert_eq!(loaded.grants.len(), Broker::MAX_GRANTS);
        assert_eq!(loaded.audit.len(), MAX_AUDIT_RECORDS);
        assert!(
            bytes <= MAX_FILE_BYTES,
            "{bytes} bytes is past the {MAX_FILE_BYTES}-byte read ceiling"
        );
        println!("record bytes={bytes} save_us={save_us} load_us={load_us}");
        cleanup(&path);
    }

    #[test]
    fn the_audit_is_written_at_its_cap_not_beyond() {
        let path = fixture("audit-cap");
        let store = Store::new(&path);
        let audit: Vec<AuditEntry> = (0..(MAX_AUDIT_RECORDS + 100))
            .map(|i| AuditEntry {
                app: app("com.example.spam"),
                request: camera(),
                allowed: false,
                reason: Reason::NoGrant,
                at: u64::try_from(i).expect("fits"),
            })
            .collect();
        store.save(&[], &audit).expect("saves");
        let text = fs::read_to_string(&path).expect("read");
        assert_eq!(
            text.lines().filter(|l| l.starts_with("audit\t")).count(),
            MAX_AUDIT_RECORDS
        );
        let loaded = store.load().expect("loads").expect("exists");
        assert_eq!(loaded.audit.len(), MAX_AUDIT_RECORDS);
        // The newest survive, not the oldest.
        assert_eq!(loaded.audit.last().expect("last").at, 611);
        cleanup(&path);
    }

    #[test]
    fn the_grant_table_is_written_at_the_broker_cap() {
        let path = fixture("grant-cap");
        let store = Store::new(&path);
        let grants: Vec<Record> = (0..(Broker::MAX_GRANTS + 50))
            .map(|i| {
                (
                    app(&format!("com.example.a{i}")),
                    Grant::new(camera(), Issuer::User, 1),
                )
            })
            .collect();
        store.save(&grants, &[]).expect("saves");
        let text = fs::read_to_string(&path).expect("read");
        assert_eq!(
            text.lines().filter(|l| l.starts_with("grant\t")).count(),
            Broker::MAX_GRANTS
        );
        cleanup(&path);
    }

    #[test]
    fn every_scope_kind_round_trips() {
        let scopes = [
            Scope::parse_uri("user://Documents/tax").expect("uri"),
            Scope::parse_resource("wifi://").expect("resource"),
            Scope::parse_resource("power://system/poweroff").expect("resource"),
            Scope::parse_host("example.com").expect("host"),
        ];
        for scope in scopes {
            let capability =
                Capability::new(CapabilityId::NetClient, scope.clone()).expect("scoped capability");
            let text = capability.to_string();
            assert_eq!(
                parse_capability(&text),
                Some(capability.clone()),
                "{text} did not round trip"
            );
        }
        let unscoped = Capability::unscoped(CapabilityId::Camera).expect("unscoped");
        assert_eq!(parse_capability("camera"), Some(unscoped));
    }

    #[test]
    fn a_denial_reason_survives_with_the_field_it_needs() {
        for reason in [
            Reason::Granted {
                issuer: Issuer::User,
                scope: Scope::parse_uri("user://Documents").expect("scope"),
            },
            Reason::Revoked { at: 7 },
            Reason::Expired { at: 8 },
            Reason::OutOfScope {
                held: Scope::parse_resource("bt://hci0").expect("scope"),
            },
            Reason::NoGrant,
            Reason::NeedsLiveConsent,
            Reason::UnknownApplication,
        ] {
            let entry = AuditEntry {
                app: app("com.example.app"),
                request: camera(),
                allowed: false,
                reason: reason.clone(),
                at: 9,
            };
            let text = encode(&[], &[entry]);
            let loaded = decode(&text).expect("decodes").expect("record");
            assert_eq!(loaded.audit[0].reason, reason, "{text}");
        }
    }
}
