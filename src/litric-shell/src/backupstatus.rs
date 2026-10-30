//! The backup service's status as the shell can safely show it.
//!
//! The daemon answers `BACKUP STATUS` with stable lines about its schedule and
//! last snapshot. This module is the shell's read-only half: one query when the
//! Updates page opens, a typed snapshot of the fields the page understands, and
//! no path, free-form I/O detail or unknown state on screen.
//!
//! The page does not poll. A backup is periodic work owned by `litric-sysd`;
//! a shell repainting the status every second would be a second scheduler with
//! no authority. The user can ask again with the page's existing refresh row.

use std::path::Path;

/// A short backup status, as the Updates page needs it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Status {
    /// The stable state word: `ready`, `uninitialized`, `disabled`, `corrupt`
    /// or `unavailable`.
    pub state: String,
    /// Automatic interval in seconds, when the daemon named a valid one.
    pub interval_secs: Option<u64>,
    /// Number of valid snapshots.
    pub snapshots: Option<u64>,
    /// The most recent snapshot id, when one exists.
    pub last_id: Option<String>,
    /// When the most recent snapshot was created.
    pub last_created: Option<u64>,
    /// File count in the most recent snapshot.
    pub last_entries: Option<u64>,
    /// Byte count in the most recent snapshot.
    pub last_bytes: Option<u64>,
    /// A short `last_error` token, never free-form detail.
    pub last_error: Option<String>,
    /// Why the query did not produce a usable status.
    pub error: Option<String>,
}

impl Status {
    /// A status that could not be read, with a user-safe reason.
    #[must_use]
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            state: "unavailable".to_owned(),
            error: Some(reason.into()),
            ..Self::default()
        }
    }

    /// Whether the daemon answered with a state this build understands.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.error.is_none() && !self.state.is_empty() && self.state != "unavailable"
    }

    /// The state in words a person reads.
    #[must_use]
    pub fn state_label(&self) -> &'static str {
        match self.state.as_str() {
            "ready" => "Automatic backup is scheduled",
            "uninitialized" => "No backup has been made yet",
            "disabled" => "Backup service not configured",
            "corrupt" => "A backup snapshot is unreadable",
            _ => "Backup service unavailable",
        }
    }

    /// The schedule in words, or `None` when the daemon did not name one.
    #[must_use]
    pub fn schedule_label(&self) -> Option<String> {
        match self.interval_secs {
            Some(0) => Some("Manual only".to_owned()),
            Some(86_400) => Some("Once a day".to_owned()),
            Some(secs) => Some(format!("Every {secs} seconds")),
            None => None,
        }
    }

    /// The most recent snapshot in words, or `None` when there is none.
    #[must_use]
    pub fn last_snapshot_label(&self) -> Option<String> {
        let id = self.last_id.as_ref()?;
        let entries = self.last_entries.unwrap_or(0);
        let bytes = self.last_bytes.unwrap_or(0);
        Some(format!("{id} · {entries} files · {bytes} bytes"))
    }

    /// The snapshot count in words, or `None` when it was not reported.
    #[must_use]
    pub fn snapshot_count_label(&self) -> Option<String> {
        self.snapshots.map(|count| match count {
            1 => "1 snapshot".to_owned(),
            n => format!("{n} snapshots"),
        })
    }
}

/// Ask `litric-sysd` for `BACKUP STATUS` and parse the reply.
#[must_use]
pub fn read(socket: &Path) -> Status {
    let Some(reply) = query(socket) else {
        return Status::unavailable("litric-sysd did not answer");
    };
    parse(&reply)
}

#[cfg(unix)]
fn query(socket: &Path) -> Option<String> {
    litric_sysd::server::query(socket, "BACKUP STATUS").ok()
}

#[cfg(not(unix))]
fn query(_socket: &Path) -> Option<String> {
    None
}

/// Parse a complete reply, including its `OK n` / `ERR kind` header.
///
/// `pub` for the parser's own tests; callers normally use [`read`].
#[must_use]
pub fn parse(reply: &str) -> Status {
    let mut lines = reply.lines();
    let Some(header) = lines.next() else {
        return Status::unavailable("empty reply");
    };
    if let Some(rest) = header.strip_prefix("ERR ") {
        let kind = rest.split_whitespace().next().unwrap_or("refused");
        return Status::unavailable(safe_token(kind).unwrap_or_else(|| "refused".to_owned()));
    }
    if !header.starts_with("OK ") {
        return Status::unavailable("reply has no status header");
    }

    let body: Vec<&str> = lines.collect();
    let field = |key: &str| -> Option<&str> {
        body.iter().find_map(|line| {
            line.strip_prefix(key)
                .and_then(|rest| rest.strip_prefix('='))
        })
    };

    let state = field("state")
        .filter(|value| {
            matches!(
                *value,
                "ready" | "uninitialized" | "disabled" | "corrupt" | "unavailable"
            )
        })
        .unwrap_or("unavailable");
    if state == "unavailable" {
        return Status::unavailable("the daemon did not report a usable state");
    }

    Status {
        state: state.to_owned(),
        interval_secs: field("interval_secs").and_then(parse_u64),
        snapshots: field("snapshots").and_then(parse_u64),
        last_id: field("last_id").and_then(safe_id),
        last_created: field("last_created").and_then(parse_u64),
        last_entries: field("last_entries").and_then(parse_u64),
        last_bytes: field("last_bytes").and_then(parse_u64),
        last_error: field("last_error").and_then(safe_error),
        error: None,
    }
}

fn parse_u64(raw: &str) -> Option<u64> {
    if raw.is_empty() || raw.len() > 20 {
        return None;
    }
    raw.parse::<u64>().ok()
}

/// Snapshot ids are `snap-` plus twenty digits and an optional numeric suffix.
fn safe_id(raw: &str) -> Option<String> {
    if raw.len() < 25 || raw.len() > 80 || !raw.starts_with("snap-") {
        return None;
    }
    if !raw[5..].bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return None;
    }
    Some(raw.to_owned())
}

/// An error code the page may show: one short token, no path and no spaces.
fn safe_error(raw: &str) -> Option<String> {
    let code = raw.split_whitespace().next()?;
    safe_token(code)
}

fn safe_token(raw: &str) -> Option<String> {
    if raw.is_empty() || raw.len() > 96 {
        return None;
    }
    raw.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b':' | b'-' | b'_' | b'.'))
        .then(|| raw.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ready_schedule_keeps_its_counts() {
        let status = parse(
            "OK 8\nstate=ready\ninterval_secs=86400\nnext_due=1800086400\nsnapshots=2\n\
             last_id=snap-00000000001800000000\nlast_created=1800000000\n\
             last_entries=7\nlast_bytes=4096\n",
        );
        assert!(status.is_available());
        assert_eq!(status.state_label(), "Automatic backup is scheduled");
        assert_eq!(status.schedule_label().as_deref(), Some("Once a day"));
        assert_eq!(
            status.snapshot_count_label().as_deref(),
            Some("2 snapshots")
        );
        assert_eq!(
            status.last_snapshot_label().as_deref(),
            Some("snap-00000000001800000000 · 7 files · 4096 bytes")
        );
    }

    #[test]
    fn a_disabled_service_is_a_named_fact_not_an_error() {
        let status = parse("OK 2\nstate=disabled\ninterval_secs=0\n");
        assert!(status.is_available());
        assert_eq!(status.state_label(), "Backup service not configured");
        assert_eq!(status.schedule_label().as_deref(), Some("Manual only"));
        assert!(status.last_snapshot_label().is_none());
    }

    #[test]
    fn a_refusal_keeps_only_the_typed_kind() {
        let status = parse("ERR backup-disabled no backup root is configured");
        assert!(!status.is_available());
        assert_eq!(status.error.as_deref(), Some("backup-disabled"));
        assert!(!status.error.as_deref().unwrap_or("").contains('/'));
    }

    #[test]
    fn unknown_fields_paths_and_bad_ids_are_not_displayed() {
        let status = parse(
            "OK 5\nstate=ready\nroot=/home/alice/.local/state/litric/backups\n\
             interval_secs=10\nlast_id=../../etc/passwd\nlast_error=backup-io /secret/path\n",
        );
        assert!(status.is_available());
        assert!(status.last_id.is_none());
        assert_eq!(status.last_error.as_deref(), Some("backup-io"));
    }

    #[test]
    fn a_missing_state_is_unavailable_not_ready() {
        let status = parse("OK 1\ninterval_secs=10\n");
        assert!(!status.is_available());
        assert_eq!(status.state_label(), "Backup service unavailable");
    }
}
