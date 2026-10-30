//! One cross-process lock for one update root.
//!
//! The update state is a read-modify-write record. Atomic replace makes a single
//! write safe, but two processes can still read the same attempt count and both
//! write it back: one increment is lost. The engine's state file has no lock,
//! and the systemd unit only orders `--boot` before `--serve`; it cannot stop
//! two writers that were started by an operator or a retry. This module is that
//! missing serialisation.
//!
//! # Protocol
//!
//! The lock is an owner-only file beside the state root, not a flock: a lock
//! that disappears with the process is not enough after a crash, because the
//! next boot must be able to tell a dead owner from a live one. Acquisition is
//! `create_new`; the file contains:
//!
//! ```text
//! litric-update-lock\t1
//! pid\t1234
//! at\t1800000000
//! reason\tstage
//! ```
//!
//! A writer that finds a valid lock held by a live PID waits for a bounded
//! period ([`WRITER_WAIT`]) and then fails with [`LockError::Locked`]. A status
//! query uses [`try_acquire`] instead and never waits: it reports `busy` while a
//! writer holds the root.
//!
//! # Reclaim
//!
//! A lock is stale only when its owner cannot be alive — the PID has no
//! `/proc/<pid>` entry — or the record is malformed. A live PID is never
//! evicted, however old the timestamp is: staging a large tree may legitimately
//! take minutes. A freshly created but not-yet-written lock is given
//! [`INITIALISING_GRACE`] before it is treated as malformed, so the writer that
//! won `create_new` cannot have its file reclaimed in the gap between create and
//! write.
//!
//! On a platform without `/proc`, liveness cannot be proven, so a syntactically
//! valid lock is treated as live (fail closed) and only malformed records are
//! reclaimed. That is a limitation of this std-only implementation, not a
//! promise that a dead owner is always detected there.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// The lock file's name under the configured update root.
pub const LOCK_NAME: &str = "update.lock";
/// The protocol header's first field.
pub const LOCK_PROTOCOL: &str = "litric-update-lock";
/// The lock record's schema version.
pub const LOCK_VERSION: u8 = 1;
/// The largest lock record accepted. A valid record is under 100 bytes.
pub const MAX_LOCK_BYTES: u64 = 512;
/// How long a writer waits for a live owner before returning `locked`.
pub const WRITER_WAIT: Duration = Duration::from_millis(2_000);
/// How often the bounded wait retries.
const POLL: Duration = Duration::from_millis(5);
/// How long an empty, freshly-created lock is treated as being written.
const INITIALISING_GRACE: Duration = Duration::from_secs(2);
/// The number of create/reclaim attempts before giving up on a churning root.
const ATTEMPTS: u8 = 4;

/// The owner recorded in a lock file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockInfo {
    /// The owner process id.
    pub pid: u32,
    /// When the owner acquired the lock, seconds since the Unix epoch.
    pub at: u64,
    /// Why it acquired the lock.
    pub reason: String,
}

impl LockInfo {
    /// The words a status reply may safely repeat. Unknown reasons are reduced
    /// to `unknown` rather than echoed: a damaged lock file is untrusted input.
    #[must_use]
    pub fn safe_reason(&self) -> &str {
        if self.reason.is_empty() {
            "unknown"
        } else {
            &self.reason
        }
    }
}

/// Why a lock could not be acquired.
#[derive(Debug)]
pub enum LockError {
    /// A live owner holds the root.
    Locked(LockInfo),
    /// The lock file itself could not be inspected, created or removed.
    Io {
        /// What was being attempted.
        action: &'static str,
        /// The path involved, always under the configured update root.
        path: PathBuf,
        /// The OS error kind.
        kind: io::ErrorKind,
        /// The OS message, retained for diagnosis.
        message: String,
    },
    /// A freshly-created lock was still malformed after the grace period.
    Contended,
}

impl LockError {
    /// A stable token for the protocol's `ERR <kind>` field.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Locked(_) | Self::Contended => "locked",
            Self::Io { .. } => "lock-io",
        }
    }

    fn io(action: &'static str, path: &Path, error: io::Error) -> Self {
        Self::Io {
            action,
            path: path.to_path_buf(),
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}

impl core::fmt::Display for LockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Locked(info) => write!(
                f,
                "update root is locked by pid {} for {}",
                info.pid,
                info.safe_reason()
            ),
            Self::Contended => f.write_str("update root lock is still being written"),
            Self::Io {
                action,
                path,
                message,
                ..
            } => write!(f, "{action} {}: {message}", path.display()),
        }
    }
}

impl std::error::Error for LockError {}

/// A held lock. Dropping it removes only the exact record this process wrote.
#[derive(Debug)]
pub struct UpdateLock {
    path: PathBuf,
    content: String,
}

impl Drop for UpdateLock {
    fn drop(&mut self) {
        // A reclaimer never removes a live owner's lock, so a mismatch here
        // means the file was replaced behind our back. Do not delete somebody
        // else's record.
        if fs::read_to_string(&self.path).is_ok_and(|current| current == self.content) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Try once to take the root lock. Never waits.
///
/// # Errors
/// [`LockError::Locked`] when a live owner holds it, [`LockError::Io`] when the
/// root or lock file cannot be inspected, and [`LockError::Contended`] when the
/// root churns through four create/reclaim attempts.
pub fn try_acquire(root: &Path, reason: &str, now: u64) -> Result<UpdateLock, LockError> {
    if !valid_reason(reason) {
        return Err(LockError::Io {
            action: "validate update lock reason",
            path: root.join(LOCK_NAME),
            kind: io::ErrorKind::InvalidInput,
            message: "reason must be 1..=32 ASCII letters, digits, '-' or '_'".to_owned(),
        });
    }
    let info = LockInfo {
        pid: std::process::id(),
        at: now,
        reason: reason.to_owned(),
    };
    let content = encode(&info);
    let path = root.join(LOCK_NAME);

    for _ in 0..ATTEMPTS {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(mut file) => {
                if let Err(error) = file.write_all(content.as_bytes()) {
                    // We created this name exclusively and have not returned a
                    // handle, so removing it cannot delete another owner's lock.
                    let _ = remove_stale(&path);
                    return Err(LockError::io("write update lock", &path, error));
                }
                if let Err(error) = file.sync_all() {
                    let _ = remove_stale(&path);
                    return Err(LockError::io("sync update lock", &path, error));
                }
                return Ok(UpdateLock { path, content });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => match existing(&path) {
                Ok(Existing::Live(info)) => return Err(LockError::Locked(info)),
                Ok(Existing::Initialising) => {
                    return Err(LockError::Locked(LockInfo {
                        pid: 0,
                        at: 0,
                        reason: "initialising".to_owned(),
                    }))
                }
                Ok(Existing::Stale) => {
                    if let Err(error) = remove_stale(&path) {
                        if error.kind() != io::ErrorKind::NotFound {
                            return Err(LockError::io("reclaim update lock", &path, error));
                        }
                    }
                    continue;
                }
                Ok(Existing::Missing) => continue,
                Err(error) => {
                    return Err(LockError::io("inspect update lock", &path, error));
                }
            },
            Err(error) => return Err(LockError::io("create update lock", &path, error)),
        }
    }
    Err(LockError::Contended)
}

/// Take the root lock, waiting at most [`WRITER_WAIT`] for a live owner.
///
/// # Errors
/// The same forms as [`try_acquire`]. A live owner that outlasts the wait is
/// returned as [`LockError::Locked`] with its pid and reason.
pub fn acquire(root: &Path, reason: &str, now: u64) -> Result<UpdateLock, LockError> {
    let deadline = Instant::now()
        .checked_add(WRITER_WAIT)
        .unwrap_or_else(Instant::now);
    loop {
        match try_acquire(root, reason, now) {
            Ok(lock) => return Ok(lock),
            Err(LockError::Locked(_)) | Err(LockError::Contended) if Instant::now() < deadline => {
                thread::sleep(POLL);
            }
            Err(error) => return Err(error),
        }
    }
}

fn encode(info: &LockInfo) -> String {
    format!(
        "{LOCK_PROTOCOL}\t{LOCK_VERSION}\npid\t{}\nat\t{}\nreason\t{}\n",
        info.pid, info.at, info.reason
    )
}

enum Existing {
    Live(LockInfo),
    Initialising,
    Stale,
    Missing,
}

fn existing(path: &Path) -> io::Result<Existing> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Existing::Missing),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Ok(Existing::Stale);
    }
    if metadata.len() == 0 || metadata.len() > MAX_LOCK_BYTES {
        return Ok(fresh_or_stale(&metadata));
    }
    let bytes = fs::read(path)?;
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(fresh_or_stale(&metadata));
    };
    match parse(text) {
        Some(info) if pid_alive(info.pid) => Ok(Existing::Live(info)),
        Some(_) => Ok(Existing::Stale),
        None => Ok(fresh_or_stale(&metadata)),
    }
}

fn fresh_or_stale(metadata: &fs::Metadata) -> Existing {
    let age = metadata
        .modified()
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok());
    match age {
        Some(age) if age < INITIALISING_GRACE => Existing::Initialising,
        _ => Existing::Stale,
    }
}

fn parse(text: &str) -> Option<LockInfo> {
    let mut lines = text.lines();
    if lines.next()? != format!("{LOCK_PROTOCOL}\t{LOCK_VERSION}") {
        return None;
    }
    let pid = lines.next()?.strip_prefix("pid\t")?.parse::<u32>().ok()?;
    let at = lines.next()?.strip_prefix("at\t")?.parse::<u64>().ok()?;
    let reason = lines.next()?.strip_prefix("reason\t")?.to_owned();
    if lines.next().is_some() || pid == 0 || !valid_reason(&reason) {
        return None;
    }
    Some(LockInfo { pid, at, reason })
}

fn valid_reason(reason: &str) -> bool {
    !reason.is_empty()
        && reason.len() <= 32
        && reason
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// A Linux `/proc` entry is the liveness proof. Without `/proc` the answer is
/// conservatively "live"; see the module note.
fn pid_alive(pid: u32) -> bool {
    if cfg!(target_os = "linux") {
        Path::new("/proc").join(pid.to_string()).exists()
    } else {
        true
    }
}

fn remove_stale(path: &Path) -> io::Result<()> {
    fs::remove_file(path)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn root(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("litric-update-lock-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("root");
        path
    }

    #[test]
    fn a_live_lock_refuses_a_second_holder_and_releases_on_drop() {
        let root = root("live");
        let first = try_acquire(&root, "boot", 1).expect("first");
        let error = try_acquire(&root, "commit", 2).expect_err("second");
        assert_eq!(error.kind(), "locked");
        drop(first);
        assert!(!root.join(LOCK_NAME).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_dead_pid_is_reclaimed() {
        let root = root("stale");
        let body = format!(
            "{LOCK_PROTOCOL}\t{LOCK_VERSION}\npid\t{}\nat\t1\nreason\tstage\n",
            u32::MAX - 1
        );
        fs::write(root.join(LOCK_NAME), body).expect("stale lock");
        let _lock = try_acquire(&root, "boot", 3).expect("reclaim");
        let written = fs::read_to_string(root.join(LOCK_NAME)).expect("new lock");
        assert!(written.contains(&format!("pid\t{}", std::process::id())));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_malformed_young_lock_is_not_evicted_during_creation() {
        let root = root("fresh");
        fs::write(root.join(LOCK_NAME), b"").expect("empty lock");
        let error = try_acquire(&root, "boot", 4).expect_err("must wait");
        assert_eq!(error.kind(), "locked");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_wait_is_bounded() {
        let root = root("wait");
        let first = try_acquire(&root, "stage", 5).expect("first");
        let started = Instant::now();
        let error = acquire(&root, "boot", 6).expect_err("live owner");
        assert_eq!(error.kind(), "locked");
        assert!(started.elapsed() >= WRITER_WAIT);
        drop(first);
        let _ = fs::remove_dir_all(root);
    }
}
