//! Durable lockout counters for `AUTH VERIFY`.
//!
//! # Why its own file
//!
//! The counters live next to the grant store (`<state>.lockout`), in the same
//! tab-separated, line-oriented shape, but not inside it. Two reasons, both
//! about failure: a damaged lockout record must not deny every permission the
//! way a damaged grant record does, and a daemon from before this batch would
//! reject the whole grant file if it met a `lockout` line it did not know.
//!
//! ```text
//! litric-lockout\t1
//! global\t<count>\t<window_start>\t<locked_until>
//! account\t<user>\t<count>\t<window_start>\t<locked_until>
//! ```
//!
//! A missing file is a fresh start. Anything else that cannot be read — wrong
//! protocol or version, a short or oversized line, an invalid account name, a
//! duplicate entry, more than [`MAX_TRACKED_ACCOUNTS`] accounts, a file past
//! [`MAX_FILE_BYTES`] — is an error the caller logs and treats as empty: the
//! meter fails open, never closed. Values inside a readable file are clamped by
//! [`crate::auth::Throttle::restore`], so a hostile deadline cannot outlive the
//! configured lockout.
//!
//! # Bounds and writes
//!
//! At most 257 records (one global plus [`MAX_TRACKED_ACCOUNTS`] accounts), each
//! line under [`MAX_LINE_BYTES`], in a file under [`MAX_FILE_BYTES`]. The state
//! layer writes at most once per [`PERSIST_INTERVAL_SECS`] for ordinary
//! failures, and immediately when a lockout begins or a success clears one, so
//! a caller cannot turn a retry loop into a disk-write loop. The write is the
//! same atomic sequence as the grant store: temporary file in the same
//! directory, `fsync`, `rename`, `fsync` the directory.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::auth::{valid_account_name, LockoutRecord, MAX_TRACKED_ACCOUNTS};

/// The protocol line and schema version this build writes.
pub const PROTOCOL: &str = "litric-lockout";
/// Version of the record below the protocol line.
pub const VERSION: u8 = 1;

/// The largest file this loader will read.
///
/// 64 KiB is far more than 257 short lines need; a larger file is damaged or
/// hostile, and refusing it keeps the read bounded.
pub const MAX_FILE_BYTES: u64 = 64 * 1024;

/// The longest line accepted.
pub const MAX_LINE_BYTES: usize = 512;

/// The shortest gap between two writes caused by ordinary failures.
///
/// A lockout starting or a correct password clears the wait and is written on
/// the same request; a run of failures is coalesced to this interval.
pub const PERSIST_INTERVAL_SECS: u64 = 2;

/// A lockout file at a known path.
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

    /// Read the counters.
    ///
    /// `Ok(None)` means the file does not exist and the meter starts empty.
    /// Every other failure is an `Err` the caller must log and treat as empty:
    /// count nothing rather than refuse every login.
    pub fn load(&self) -> Result<Option<Vec<LockoutRecord>>, String> {
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
        decode(text).map(Some)
    }

    /// Write the counters atomically, creating and securing the directory.
    ///
    /// # Errors
    /// Any filesystem error. The previous file is left in place when the write
    /// fails.
    pub fn save(&self, records: &[LockoutRecord]) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "lockout store path has no parent directory".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create lockout directory: {error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("cannot secure lockout directory: {error}"))?;
        }

        let temp = parent.join(format!("lockout.tmp.{}", std::process::id()));
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
                .map_err(|error| format!("cannot create temporary lockout file: {error}"))?;
            file.write_all(encode(records).as_bytes())
                .map_err(|error| format!("cannot write lockout state: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("cannot sync lockout state: {error}"))?;
            fs::rename(&temp, &self.path)
                .map_err(|error| format!("cannot commit lockout state: {error}"))?;
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

/// Encode counters. A `None` user is the global entry, written first; accounts
/// follow in name order so the file does not depend on hash order.
#[must_use]
pub fn encode(records: &[LockoutRecord]) -> String {
    let mut out = String::with_capacity(64usize.saturating_add(records.len().saturating_mul(64)));
    out.push_str(PROTOCOL);
    out.push('\t');
    out.push_str(&VERSION.to_string());
    out.push('\n');

    if let Some(global) = records.iter().find(|record| record.user.is_none()) {
        write_record(&mut out, "global", None, global);
    }
    let mut accounts: Vec<&LockoutRecord> = records
        .iter()
        .filter(|record| record.user.is_some())
        .collect();
    accounts.sort_by(|a, b| a.user.cmp(&b.user));
    for record in accounts.into_iter().take(MAX_TRACKED_ACCOUNTS) {
        write_record(&mut out, "account", record.user.as_deref(), record);
    }
    out
}

fn write_record(out: &mut String, kind: &str, user: Option<&str>, record: &LockoutRecord) {
    out.push_str(kind);
    if let Some(user) = user {
        out.push('\t');
        out.push_str(user);
    }
    out.push('\t');
    out.push_str(&record.count.to_string());
    out.push('\t');
    out.push_str(&record.window_start.to_string());
    out.push('\t');
    out.push_str(&record.locked_until.to_string());
    out.push('\n');
}

/// Parse counters. See the module note for the failure rule.
///
/// # Errors
/// A missing header, an unknown version, a malformed line, a duplicate or an
/// invalid account name, or more entries than the meter can hold.
pub fn decode(text: &str) -> Result<Vec<LockoutRecord>, String> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| "empty lockout record".to_owned())?;
    let expected = format!("{PROTOCOL}\t{VERSION}");
    if header != expected {
        return Err(format!("unknown lockout record header {header:?}"));
    }

    let mut records = Vec::new();
    let mut global_seen = false;
    let mut accounts: BTreeSet<String> = BTreeSet::new();
    for (index, line) in lines.enumerate() {
        let number = index.saturating_add(2);
        if line.is_empty() {
            return Err(format!("line {number}: empty record"));
        }
        if line.len() > MAX_LINE_BYTES {
            return Err(format!("line {number}: past {MAX_LINE_BYTES} bytes"));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        match fields.as_slice() {
            ["global", count, window, locked] => {
                if global_seen {
                    return Err(format!("line {number}: duplicate global record"));
                }
                global_seen = true;
                records.push(LockoutRecord {
                    user: None,
                    count: parse_count(count, number)?,
                    window_start: parse_time(window, number)?,
                    locked_until: parse_time(locked, number)?,
                });
            }
            ["account", user, count, window, locked] => {
                if !valid_account_name(user) {
                    return Err(format!("line {number}: invalid account name"));
                }
                if accounts.len() >= MAX_TRACKED_ACCOUNTS {
                    return Err(format!(
                        "line {number}: more than {MAX_TRACKED_ACCOUNTS} accounts"
                    ));
                }
                if !accounts.insert((*user).to_owned()) {
                    return Err(format!("line {number}: duplicate account"));
                }
                records.push(LockoutRecord {
                    user: Some((*user).to_owned()),
                    count: parse_count(count, number)?,
                    window_start: parse_time(window, number)?,
                    locked_until: parse_time(locked, number)?,
                });
            }
            _ => return Err(format!("line {number}: unrecognised record")),
        }
    }
    Ok(records)
}

fn parse_count(raw: &str, line: usize) -> Result<u32, String> {
    raw.parse()
        .map_err(|_| format!("line {line}: bad count {raw:?}"))
}

fn parse_time(raw: &str, line: usize) -> Result<u64, String> {
    raw.parse()
        .map_err(|_| format!("line {line}: bad timestamp {raw:?}"))
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn root(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("litric-lockout-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        path
    }

    fn sample() -> Vec<LockoutRecord> {
        vec![
            LockoutRecord {
                user: None,
                count: 2,
                window_start: 1_000,
                locked_until: 0,
            },
            LockoutRecord {
                user: Some("alice".to_owned()),
                count: 5,
                window_start: 1_000,
                locked_until: 1_060,
            },
        ]
    }

    #[test]
    fn a_record_round_trips_through_the_atomic_file() {
        let dir = root("roundtrip");
        let store = Store::new(dir.join("grants.lockout"));
        store.save(&sample()).expect("saves");
        assert_eq!(store.load().expect("loads"), Some(sample()));
        assert!(
            !dir.join(format!("lockout.tmp.{}", std::process::id()))
                .exists(),
            "the temporary is not left behind"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_damaged_file_is_an_error_not_a_panic() {
        let dir = root("damaged");
        fs::create_dir_all(&dir).expect("dir");
        let store = Store::new(dir.join("grants.lockout"));
        for text in [
            "",
            "garbage\n",
            "litric-lockout\t2\n",
            "litric-lockout\t1\naccount\talice\tx\t0\t0\n",
            "litric-lockout\t1\naccount\tAlice\t1\t0\t0\n",
            "litric-lockout\t1\nglobal\t1\t0\t0\nglobal\t1\t0\t0\n",
        ] {
            fs::write(store.path(), text).expect("writes");
            assert!(store.load().is_err(), "{text:?} must not load");
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn more_accounts_than_the_meter_holds_is_refused() {
        let mut text = String::from("litric-lockout\t1\n");
        for i in 0..=MAX_TRACKED_ACCOUNTS {
            text.push_str(&format!("account\tuser{i}\t1\t0\t0\n"));
        }
        assert!(decode(&text).is_err());
    }

    #[test]
    fn an_oversized_file_is_refused_before_it_is_read() {
        let dir = root("oversized");
        fs::create_dir_all(&dir).expect("dir");
        let store = Store::new(dir.join("grants.lockout"));
        fs::write(store.path(), vec![b'x'; MAX_FILE_BYTES as usize + 1]).expect("writes");
        assert!(store.load().is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
