//! Local, integrity-verified backups of the user's application state.
//!
//! ADR-0021 keeps live data, snapshots, backups and generations separate. This
//! module implements a **local backup**: a copied, immutable-by-convention tree
//! outside the state it copies, plus a manifest binding every copied file to its
//! size, mode and SHA-256. It is not a Btrfs snapshot, a cloud backup or a
//! system generation.
//!
//! The source set is the one this system currently owns:
//!
//! ```text
//! <application-root>/applications/      desktop entries
//! <application-root>/litric/apps/       installed executables
//! ```
//!
//! The backup root is configuration, never a request field. Production uses
//! `<home>/.local/state/litric/backups`, outside the XDG data root, so a backup
//! is not inside the tree it copies. The default retention is seven snapshots.
//!
//! # What can be trusted
//!
//! A manifest is not proof by itself. [`BackupService::verify`] re-opens every
//! file named by it, checks the relative path, size, mode and SHA-256, and then
//! walks the snapshot to refuse an unlisted file. A manifest with an altered
//! digest therefore fails rather than being "verified" from its own claims.
//!
//! The backup is a local copy, not an off-machine backup. A compromised user
//! account can delete both the state and the backup; the docs/design note says
//! this plainly.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

use litric_defender::sha256::{self, Sha256};

/// The on-disk manifest protocol.
pub const MANIFEST_PROTOCOL: &str = "litric-backup";
/// The manifest format version.
pub const MANIFEST_VERSION: u32 = 1;
/// The default automatic backup interval: once per day.
pub const DEFAULT_INTERVAL_SECS: u64 = 24 * 60 * 60;
/// The number of snapshots kept.
pub const MAX_SNAPSHOTS: usize = 7;
/// The maximum number of regular files in one snapshot.
pub const MAX_ENTRIES: usize = 100_000;
/// The maximum total bytes in one snapshot.
pub const MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// The maximum manifest size.
pub const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
/// The maximum relative path length.
pub const MAX_PATH_BYTES: usize = 4096;

const SOURCE_LABEL: &str = "applications,litric/apps";

/// Why a backup operation was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackupError {
    /// No backup root is configured.
    Disabled,
    /// The snapshot id is not a valid `snap-…` id.
    BadId,
    /// The snapshot or source does not exist.
    Missing,
    /// A manifest is absent, oversized, malformed or names another snapshot.
    InvalidManifest,
    /// A file's size, mode or digest no longer matches its manifest.
    Mismatch,
    /// A source or snapshot contains a link or another unsupported entry.
    UnsafeSource,
    /// The snapshot exceeds the entry or byte bound.
    Limit,
    /// The snapshot id is already taken and no suffix was available.
    Exists,
    /// A filesystem operation failed.
    Io {
        /// What was being attempted.
        action: &'static str,
        /// The OS error kind.
        kind: io::ErrorKind,
    },
}

impl BackupError {
    /// The stable protocol token.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Disabled => "backup-disabled",
            Self::BadId => "bad-id",
            Self::Missing => "not-found",
            Self::InvalidManifest => "invalid-manifest",
            Self::Mismatch => "checksum",
            Self::UnsafeSource => "unsafe-source",
            Self::Limit => "limit",
            Self::Exists => "exists",
            Self::Io { .. } => "backup-io",
        }
    }

    /// A short sentence that never contains a filesystem path.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Disabled => "no backup root is configured".to_owned(),
            Self::BadId => "the snapshot id is not valid".to_owned(),
            Self::Missing => "the requested backup or source is absent".to_owned(),
            Self::InvalidManifest => "the backup manifest is invalid".to_owned(),
            Self::Mismatch => "a backup file does not match its manifest".to_owned(),
            Self::UnsafeSource => {
                "the source or snapshot contains a link or unsupported entry".to_owned()
            }
            Self::Limit => "the backup exceeds its entry or byte limit".to_owned(),
            Self::Exists => "the snapshot id is already in use".to_owned(),
            Self::Io { action, kind } => format!("{action}: {kind:?}"),
        }
    }
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.detail())
    }
}

impl std::error::Error for BackupError {}

/// A snapshot's manifest summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    /// The snapshot id.
    pub id: String,
    /// Seconds since the Unix epoch when the snapshot was made.
    pub created: u64,
    /// Number of files.
    pub entries: usize,
    /// Total file bytes.
    pub bytes: u64,
}

/// The service's last attempt and scheduling state.
#[derive(Debug)]
pub struct BackupService {
    root: Option<PathBuf>,
    interval_secs: u64,
    next_due: u64,
    last: Option<Summary>,
    last_error: Option<String>,
}

impl BackupService {
    /// A service with no root. Nothing is copied and status is `disabled`.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            root: None,
            interval_secs: 0,
            next_due: 0,
            last: None,
            last_error: None,
        }
    }

    /// A service rooted at `root`, scheduled once per day.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
            interval_secs: DEFAULT_INTERVAL_SECS,
            next_due: 0,
            last: None,
            last_error: None,
        }
    }

    /// The configured backup root.
    #[must_use]
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// The automatic interval, or zero when disabled.
    #[must_use]
    pub const fn interval_secs(&self) -> u64 {
        self.interval_secs
    }

    /// Set the automatic interval. Zero disables the scheduler.
    #[must_use]
    pub const fn with_interval(mut self, secs: u64) -> Self {
        self.interval_secs = secs;
        self
    }

    /// Read the service state as protocol lines.
    #[must_use]
    pub fn status(&self, _now: u64) -> Vec<String> {
        let Some(root) = self.root.as_deref() else {
            return vec!["state=disabled".to_owned(), "interval_secs=0".to_owned()];
        };
        let state = if !root.exists() {
            "uninitialized"
        } else {
            match self.list() {
                Ok(_) => "ready",
                Err(_) => "corrupt",
            }
        };
        let snapshots = self.list().map_or(0, |s| s.len());
        let mut out = vec![
            format!("state={state}"),
            format!("interval_secs={}", self.interval_secs),
            format!("next_due={}", self.next_due),
            format!("snapshots={snapshots}"),
        ];
        if let Some(last) = &self.last {
            out.push(format!("last_id={}", last.id));
            out.push(format!("last_created={}", last.created));
            out.push(format!("last_entries={}", last.entries));
            out.push(format!("last_bytes={}", last.bytes));
        }
        if let Some(error) = &self.last_error {
            out.push(format!("last_error={error}"));
        }
        out
    }

    /// List valid snapshots, newest first.
    ///
    /// # Errors
    /// A root that exists but cannot be read, or a snapshot whose manifest is
    /// invalid. A damaged snapshot is not silently skipped.
    pub fn list(&self) -> Result<Vec<Summary>, BackupError> {
        let Some(root) = self.root.as_deref() else {
            return Ok(Vec::new());
        };
        let snapshots = root.join("snapshots");
        let entries = match fs::read_dir(&snapshots) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(io_error("read snapshot directory", error)),
        };
        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| io_error("read snapshot directory", error))?;
            let meta = fs::symlink_metadata(entry.path())
                .map_err(|error| io_error("inspect snapshot", error))?;
            if meta.file_type().is_symlink() {
                return Err(BackupError::UnsafeSource);
            }
            if !meta.is_dir() {
                continue;
            }
            let id = entry
                .file_name()
                .to_str()
                .map(str::to_owned)
                .ok_or(BackupError::BadId)?;
            let manifest = read_manifest(&entry.path().join("manifest"), &id)?;
            out.push(manifest.summary());
        }
        out.sort_by(|a, b| b.created.cmp(&a.created).then_with(|| b.id.cmp(&a.id)));
        Ok(out)
    }

    /// Verify every file in one snapshot by re-reading and re-hashing it.
    ///
    /// # Errors
    /// A missing/invalid snapshot, an unlisted file, a changed size or mode, or
    /// a digest mismatch.
    pub fn verify(&self, id: &str) -> Result<Summary, BackupError> {
        let root = self.root.as_deref().ok_or(BackupError::Disabled)?;
        validate_id(id)?;
        let dir = root.join("snapshots").join(id);
        let meta = fs::symlink_metadata(&dir).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                BackupError::Missing
            } else {
                io_error("inspect snapshot", error)
            }
        })?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(BackupError::UnsafeSource);
        }
        let manifest = read_manifest(&dir.join("manifest"), id)?;
        let mut seen = BTreeSet::new();
        for entry in &manifest.entries {
            let path = dir.join("files").join(&entry.path);
            let meta = fs::symlink_metadata(&path).map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    BackupError::Mismatch
                } else {
                    io_error("inspect backup file", error)
                }
            })?;
            if meta.file_type().is_symlink() || !meta.is_file() {
                return Err(BackupError::UnsafeSource);
            }
            if meta.len() != entry.size || mode_of(&meta) != entry.mode {
                return Err(BackupError::Mismatch);
            }
            let (size, hash) = hash_file(&path)?;
            if size != entry.size || hash != entry.hash {
                return Err(BackupError::Mismatch);
            }
            seen.insert(entry.path.clone());
        }
        let mut actual = Vec::new();
        walk_files(&dir.join("files"), Path::new(""), &mut actual)?;
        if actual.len() != seen.len() || actual.iter().any(|p| !seen.contains(p)) {
            return Err(BackupError::Mismatch);
        }
        Ok(manifest.summary())
    }

    /// Make one snapshot from `source` and prune beyond the retention bound.
    ///
    /// # Errors
    /// Any source, limit, manifest or filesystem refusal.
    pub fn create(&mut self, source: &Path, now: u64) -> Result<Summary, BackupError> {
        let root = self.root.clone().ok_or(BackupError::Disabled)?;
        let result = create_snapshot(&root, source, now);
        self.next_due = now.saturating_add(self.interval_secs);
        match &result {
            Ok(summary) => {
                self.last = Some(summary.clone());
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.detail());
            }
        }
        result
    }

    /// Run the automatic backup if one is due.
    ///
    /// This is called by the serve loop's scheduler. It does no sleeping and
    /// reads no clock, so a test can drive the exact schedule with a timestamp.
    pub fn tick(&mut self, source: &Path, now: u64) -> Option<Result<Summary, BackupError>> {
        if self.root.is_none() || self.interval_secs == 0 || now < self.next_due {
            return None;
        }
        Some(self.create(source, now))
    }
}

fn create_snapshot(root: &Path, source: &Path, now: u64) -> Result<Summary, BackupError> {
    if root.starts_with(source) || source.starts_with(root) {
        return Err(BackupError::UnsafeSource);
    }
    let source_meta = fs::symlink_metadata(source).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            BackupError::Missing
        } else {
            io_error("inspect backup source", error)
        }
    })?;
    if source_meta.file_type().is_symlink() || !source_meta.is_dir() {
        return Err(BackupError::UnsafeSource);
    }

    let snapshots = root.join("snapshots");
    create_private_dir(&snapshots)?;
    let id = next_id(&snapshots, now)?;
    let staging = snapshots.join(format!(".{id}.tmp.{}", std::process::id()));
    let final_dir = snapshots.join(&id);
    if fs::symlink_metadata(&staging).is_ok() {
        remove_any(&staging)?;
    }
    create_private_dir(&staging)?;

    let result = (|| -> Result<Summary, BackupError> {
        let files = staging.join("files");
        fs::create_dir(&files).map_err(|error| io_error("create backup tree", error))?;
        let mut entries = Vec::new();
        let mut total = 0u64;
        for relative in ["applications", "litric/apps"] {
            let from = source.join(relative);
            match fs::symlink_metadata(&from) {
                Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
                Ok(_) => return Err(BackupError::UnsafeSource),
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(io_error("inspect backup source", error)),
            }
            copy_tree(
                &from,
                &files.join(relative),
                Path::new(relative),
                &mut entries,
                &mut total,
            )?;
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        let body = encode_manifest(&id, now, &entries);
        write_synced(&staging.join("manifest"), body.as_bytes(), 0o600)?;
        fs::rename(&staging, &final_dir)
            .map_err(|error| io_error("publish backup snapshot", error))?;
        Ok(Summary {
            id,
            created: now,
            entries: entries.len(),
            bytes: total,
        })
    })();

    if result.is_err() {
        let _ = remove_any(&staging);
    }
    let summary = result?;
    prune(&snapshots)?;
    Ok(summary)
}

fn copy_tree(
    source: &Path,
    dest: &Path,
    relative: &Path,
    entries: &mut Vec<Entry>,
    total: &mut u64,
) -> Result<(), BackupError> {
    let meta =
        fs::symlink_metadata(source).map_err(|error| io_error("inspect backup source", error))?;
    if meta.file_type().is_symlink() {
        return Err(BackupError::UnsafeSource);
    }
    if meta.is_dir() {
        fs::create_dir_all(dest).map_err(|error| io_error("create backup directory", error))?;
        set_mode(dest, 0o755)?;
        let mut children = fs::read_dir(source)
            .map_err(|error| io_error("read backup source", error))?
            .map(|entry| entry.map_err(|error| io_error("read backup source", error)))
            .collect::<Result<Vec<_>, _>>()?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            let name = child.file_name();
            let name = name.to_str().ok_or(BackupError::UnsafeSource)?;
            if name.contains(['\t', '\n', '\r']) {
                return Err(BackupError::UnsafeSource);
            }
            copy_tree(
                &child.path(),
                &dest.join(name),
                &relative.join(name),
                entries,
                total,
            )?;
        }
        return Ok(());
    }
    if !meta.is_file() {
        return Err(BackupError::UnsafeSource);
    }
    if entries.len() >= MAX_ENTRIES {
        return Err(BackupError::Limit);
    }
    let path = relative.to_str().ok_or(BackupError::UnsafeSource)?;
    if path.is_empty() || path.len() > MAX_PATH_BYTES {
        return Err(BackupError::Limit);
    }
    let mode = mode_of(&meta);
    let (size, hash) = copy_file(source, dest, mode)?;
    *total = total.checked_add(size).ok_or(BackupError::Limit)?;
    if *total > MAX_BYTES {
        return Err(BackupError::Limit);
    }
    entries.push(Entry {
        path: path.to_owned(),
        size,
        hash,
        mode,
    });
    Ok(())
}

fn copy_file(source: &Path, dest: &Path, mode: u32) -> Result<(u64, String), BackupError> {
    let mut input = fs::File::open(source).map_err(|error| io_error("open source file", error))?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let mut output = options
        .open(dest)
        .map_err(|error| io_error("create backup file", error))?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buf = [0u8; 8192];
    loop {
        let n = input
            .read(&mut buf)
            .map_err(|error| io_error("read source file", error))?;
        if n == 0 {
            break;
        }
        size = size.checked_add(n as u64).ok_or(BackupError::Limit)?;
        if size > MAX_BYTES {
            return Err(BackupError::Limit);
        }
        let chunk = buf.get(..n).ok_or(BackupError::Limit)?;
        hasher.update(chunk);
        output
            .write_all(chunk)
            .map_err(|error| io_error("write backup file", error))?;
    }
    output
        .sync_all()
        .map_err(|error| io_error("sync backup file", error))?;
    Ok((size, sha256::hex(&hasher.finish())))
}

fn walk_files(root: &Path, relative: &Path, out: &mut Vec<String>) -> Result<(), BackupError> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error("read backup tree", error)),
    };
    let mut children = entries
        .map(|entry| entry.map_err(|error| io_error("read backup tree", error)))
        .collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let name = child.file_name();
        let name = name.to_str().ok_or(BackupError::UnsafeSource)?;
        let path = relative.join(name);
        let meta = fs::symlink_metadata(child.path())
            .map_err(|error| io_error("inspect backup tree", error))?;
        if meta.file_type().is_symlink() {
            return Err(BackupError::UnsafeSource);
        }
        if meta.is_dir() {
            walk_files(&child.path(), &path, out)?;
        } else if meta.is_file() {
            out.push(path.to_str().ok_or(BackupError::UnsafeSource)?.to_owned());
        } else {
            return Err(BackupError::UnsafeSource);
        }
    }
    Ok(())
}

fn prune(snapshots: &Path) -> Result<(), BackupError> {
    let mut dirs = fs::read_dir(snapshots)
        .map_err(|error| io_error("read snapshot directory", error))?
        .map(|entry| entry.map_err(|error| io_error("read snapshot directory", error)))
        .collect::<Result<Vec<_>, _>>()?;
    dirs.sort_by_key(|entry| entry.file_name());
    let mut valid = Vec::new();
    for entry in dirs {
        let name = entry.file_name();
        let Some(id) = name.to_str() else {
            continue;
        };
        if validate_id(id).is_err() {
            continue;
        }
        let meta = fs::symlink_metadata(entry.path())
            .map_err(|error| io_error("inspect snapshot", error))?;
        if meta.is_dir() && !meta.file_type().is_symlink() {
            valid.push((id.to_owned(), entry.path()));
        }
    }
    if valid.len() <= MAX_SNAPSHOTS {
        return Ok(());
    }
    for (_, path) in valid.iter().take(valid.len().saturating_sub(MAX_SNAPSHOTS)) {
        remove_any(path)?;
    }
    Ok(())
}

fn next_id(snapshots: &Path, now: u64) -> Result<String, BackupError> {
    for suffix in 0..1000u32 {
        let id = if suffix == 0 {
            format!("snap-{now:020}")
        } else {
            format!("snap-{now:020}-{}", suffix.saturating_add(1))
        };
        if fs::symlink_metadata(snapshots.join(&id)).is_err() {
            return Ok(id);
        }
    }
    Err(BackupError::Exists)
}

fn validate_id(id: &str) -> Result<(), BackupError> {
    if id.len() < 25 || id.len() > 80 || !id.starts_with("snap-") {
        return Err(BackupError::BadId);
    }
    let rest = &id[5..];
    let mut parts = rest.split('-');
    let time = parts.next().ok_or(BackupError::BadId)?;
    if time.len() != 20 || !time.bytes().all(|b| b.is_ascii_digit()) {
        return Err(BackupError::BadId);
    }
    if let Some(suffix) = parts.next() {
        if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
            return Err(BackupError::BadId);
        }
    }
    if parts.next().is_some() {
        return Err(BackupError::BadId);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    path: String,
    size: u64,
    hash: String,
    mode: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Manifest {
    id: String,
    created: u64,
    entries: Vec<Entry>,
}

impl Manifest {
    fn summary(&self) -> Summary {
        Summary {
            id: self.id.clone(),
            created: self.created,
            entries: self.entries.len(),
            bytes: self.entries.iter().map(|entry| entry.size).sum(),
        }
    }
}

fn encode_manifest(id: &str, created: u64, entries: &[Entry]) -> String {
    let mut out = format!(
        "{MANIFEST_PROTOCOL}\t{MANIFEST_VERSION}\nid\t{id}\ncreated\t{created}\nsource\t{SOURCE_LABEL}\n"
    );
    for entry in entries {
        out.push_str(&format!(
            "entry\t{}\t{}\t{}\t{}\n",
            entry.path, entry.size, entry.hash, entry.mode
        ));
    }
    out
}

fn read_manifest(path: &Path, expected: &str) -> Result<Manifest, BackupError> {
    let meta = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            BackupError::Missing
        } else {
            io_error("inspect manifest", error)
        }
    })?;
    if meta.file_type().is_symlink()
        || !meta.is_file()
        || meta.len() == 0
        || meta.len() > MAX_MANIFEST_BYTES
    {
        return Err(BackupError::InvalidManifest);
    }
    let bytes = fs::read(path).map_err(|error| io_error("read manifest", error))?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(BackupError::InvalidManifest);
    }
    let text = String::from_utf8(bytes).map_err(|_| BackupError::InvalidManifest)?;
    parse_manifest(&text, expected)
}

fn parse_manifest(text: &str, expected: &str) -> Result<Manifest, BackupError> {
    let mut lines = text.lines();
    let header = format!("{MANIFEST_PROTOCOL}\t{MANIFEST_VERSION}");
    if lines.next() != Some(header.as_str()) {
        return Err(BackupError::InvalidManifest);
    }
    let mut id = None;
    let mut created = None;
    let mut source = None;
    let mut entries = Vec::new();
    let mut total = 0u64;
    let mut last_path: Option<String> = None;
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            ["id", value] if id.is_none() => id = Some((*value).to_owned()),
            ["created", value] if created.is_none() => {
                created = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| BackupError::InvalidManifest)?,
                );
            }
            ["source", value] if source.is_none() => source = Some((*value).to_owned()),
            ["entry", path, size, hash, mode] => {
                if path.is_empty() || path.len() > MAX_PATH_BYTES {
                    return Err(BackupError::InvalidManifest);
                }
                validate_relative(path)?;
                if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(BackupError::InvalidManifest);
                }
                let size = size
                    .parse::<u64>()
                    .map_err(|_| BackupError::InvalidManifest)?;
                let mode = mode
                    .parse::<u32>()
                    .map_err(|_| BackupError::InvalidManifest)?;
                if mode > 0o777 {
                    return Err(BackupError::InvalidManifest);
                }
                total = total
                    .checked_add(size)
                    .ok_or(BackupError::InvalidManifest)?;
                if entries.len() >= MAX_ENTRIES || total > MAX_BYTES {
                    return Err(BackupError::Limit);
                }
                if last_path
                    .as_ref()
                    .is_some_and(|last| *path <= last.as_str())
                {
                    return Err(BackupError::InvalidManifest);
                }
                last_path = Some((*path).to_owned());
                entries.push(Entry {
                    path: (*path).to_owned(),
                    size,
                    hash: (*hash).to_ascii_lowercase(),
                    mode,
                });
            }
            _ => return Err(BackupError::InvalidManifest),
        }
    }
    if id.as_deref() != Some(expected) || source.as_deref() != Some(SOURCE_LABEL) {
        return Err(BackupError::InvalidManifest);
    }
    Ok(Manifest {
        id: id.ok_or(BackupError::InvalidManifest)?,
        created: created.ok_or(BackupError::InvalidManifest)?,
        entries,
    })
}

fn validate_relative(path: &str) -> Result<(), BackupError> {
    if path.contains(['\0', '\t', '\n', '\r']) {
        return Err(BackupError::UnsafeSource);
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(BackupError::UnsafeSource);
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<(u64, String), BackupError> {
    let mut file = fs::File::open(path).map_err(|error| io_error("open backup file", error))?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|error| io_error("read backup file", error))?;
        if n == 0 {
            break;
        }
        size = size.checked_add(n as u64).ok_or(BackupError::Limit)?;
        hasher.update(buf.get(..n).ok_or(BackupError::Limit)?);
    }
    Ok((size, sha256::hex(&hasher.finish())))
}

fn mode_of(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o777
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        0o644
    }
}

fn set_mode(path: &Path, mode: u32) -> Result<(), BackupError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(|error| io_error("set backup permissions", error))?;
    }
    let _ = mode;
    Ok(())
}

fn create_private_dir(path: &Path) -> Result<(), BackupError> {
    fs::create_dir_all(path).map_err(|error| io_error("create backup directory", error))?;
    set_mode(path, 0o700)
}

fn write_synced(path: &Path, bytes: &[u8], mode: u32) -> Result<(), BackupError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let mut file = options
        .open(path)
        .map_err(|error| io_error("create manifest", error))?;
    file.write_all(bytes)
        .map_err(|error| io_error("write manifest", error))?;
    file.sync_all()
        .map_err(|error| io_error("sync manifest", error))
}

fn remove_any(path: &Path) -> Result<(), BackupError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => {
            fs::remove_dir_all(path).map_err(|error| io_error("remove backup entry", error))
        }
        Ok(_) => fs::remove_file(path).map_err(|error| io_error("remove backup entry", error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("inspect backup entry", error)),
    }
}

fn io_error(action: &'static str, error: io::Error) -> BackupError {
    BackupError::Io {
        action,
        kind: error.kind(),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn scratch(tag: &str) -> io::Result<PathBuf> {
        let root = std::env::temp_dir().join(format!("litric-backup-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    fn source(root: &Path) -> io::Result<PathBuf> {
        let source = root.join("data");
        fs::create_dir_all(source.join("applications"))?;
        fs::create_dir_all(source.join("litric/apps/com.example.fixture"))?;
        fs::write(
            source.join("applications/com.example.fixture.desktop"),
            "[Desktop Entry]\nType=Application\nName=Fixture\nExec=app\n",
        )?;
        fs::write(
            source.join("litric/apps/com.example.fixture/app"),
            "#!/bin/sh\nprintf fixture\n",
        )?;
        fs::set_permissions(
            source.join("litric/apps/com.example.fixture/app"),
            fs::Permissions::from_mode(0o755),
        )?;
        Ok(source)
    }

    #[test]
    fn create_and_verify_rehash_every_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("verify")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups);
        let summary = service.create(&source, 1_800_000_000)?;
        assert_eq!(summary.entries, 2);
        assert_eq!(service.verify(&summary.id)?, summary);
        let snapshot = backups
            .join("snapshots")
            .join(&summary.id)
            .join("files/applications/com.example.fixture.desktop");
        assert!(fs::read_to_string(&snapshot)?.contains("Name=Fixture"));
        fs::write(
            source.join("applications/com.example.fixture.desktop"),
            "changed after backup",
        )?;
        assert_eq!(service.verify(&summary.id)?, summary);
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn a_manifest_digest_mismatch_is_refused_not_verified() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = scratch("mismatch")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups);
        let summary = service.create(&source, 1_800_000_001)?;
        let snapshot = backups
            .join("snapshots")
            .join(&summary.id)
            .join("files/applications/com.example.fixture.desktop");
        fs::write(&snapshot, "tampered")?;
        assert_eq!(service.verify(&summary.id), Err(BackupError::Mismatch));
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn retention_prunes_the_oldest_and_keeps_seven() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("retention")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups);
        for n in 0..(MAX_SNAPSHOTS + 2) {
            service.create(&source, 1_800_000_000 + n as u64)?;
        }
        let list = service.list()?;
        assert_eq!(list.len(), MAX_SNAPSHOTS);
        assert_eq!(list.first().map(|s| s.created), Some(1_800_000_008));
        assert!(list.iter().all(|s| s.id.starts_with("snap-")));
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn links_and_bad_ids_are_refused() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("refusals")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups);
        assert_eq!(service.verify("../escape"), Err(BackupError::BadId));
        std::os::unix::fs::symlink("/bin/sh", source.join("applications/linked"))?;
        assert_eq!(
            service.create(&source, 1_800_000_010),
            Err(BackupError::UnsafeSource)
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn a_corrupt_manifest_is_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("manifest")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups);
        let summary = service.create(&source, 1_800_000_020)?;
        let manifest = backups.join("snapshots").join(&summary.id).join("manifest");
        fs::write(&manifest, "not a manifest")?;
        assert_eq!(
            service.verify(&summary.id),
            Err(BackupError::InvalidManifest)
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn a_missing_snapshot_is_refused_as_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("missing")?;
        let backups = root.join("backups");
        let service = BackupService::new(&backups);
        assert_eq!(
            service.verify("snap-00000000001800000000"),
            Err(BackupError::Missing)
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn a_manifest_naming_another_snapshot_is_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("manifest-id")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups);
        let summary = service.create(&source, 1_800_000_030)?;
        let manifest = backups.join("snapshots").join(&summary.id).join("manifest");
        let text = fs::read_to_string(&manifest)?;
        let text = text.replacen(
            &format!("id\t{}", summary.id),
            "id\tsnap-00000000001800000000",
            1,
        );
        fs::write(&manifest, text)?;
        assert_eq!(
            service.verify(&summary.id),
            Err(BackupError::InvalidManifest)
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn scheduler_runs_once_per_interval_and_not_before_due(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("schedule")?;
        let source = source(&root)?;
        let backups = root.join("backups");
        let mut service = BackupService::new(&backups).with_interval(2);
        assert!(service.tick(&source, 10).is_some());
        assert!(service.tick(&source, 11).is_none());
        assert_eq!(service.list()?.len(), 1);
        assert!(service.tick(&source, 12).is_some());
        cleanup(&root);
        Ok(())
    }
}
