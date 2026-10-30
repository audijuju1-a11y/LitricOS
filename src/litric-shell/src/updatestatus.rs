//! The updater's status as the shell can safely show it.
//!
//! `litric-sysd` already answers `UPDATE STATUS 3` with the generation engine's
//! record. This module is the shell's half: a typed snapshot of the fields the
//! page understands, parsed from that reply, with unknown fields ignored and
//! unknown states reported as unavailable rather than guessed.
//!
//! # Why the shell does not read the update root
//!
//! The root is owned by the update service. A shell that read it directly would
//! be a second reader with a different lock discipline, and a stale view of a
//! generation is worse than no view. `read` therefore asks the daemon over its
//! socket, the same path `FACTS` uses. The query happens when the page is
//! opened or the user asks again, never in the paint path.
//!
//! # What is deliberately not displayed
//!
//! The daemon's `root=` line is an operator path, not a user-facing fact, and
//! `reason=` may carry I/O detail. The page shows the stable `state` word and
//! the `last_error` code only when it is a short protocol token. A record that
//! cannot be understood is `unavailable`, not a row of invented values.

use std::path::Path;

/// A short update status, as one Settings page needs it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Status {
    /// The IPC schema the daemon reported.
    pub schema: Option<u8>,
    /// The stable state word: `ready`, `uninitialized`, `disabled`, `busy`,
    /// `corrupt`, `incomplete` or `unavailable`.
    pub state: String,
    /// Active slot, `A` or `B`, when the daemon named one.
    pub active_slot: Option<String>,
    /// Version in the active slot.
    pub active_version: Option<String>,
    /// Slot the rollback would activate, when one is available.
    pub rollback_slot: Option<String>,
    /// Version the rollback would activate, when one is available.
    pub rollback_version: Option<String>,
    /// Pending candidate version.
    pub candidate_version: Option<String>,
    /// The candidate's boot attempts.
    pub candidate_attempts: Option<u8>,
    /// `healthy`, `booting` or `failed`, when evidence exists.
    pub health: Option<String>,
    /// Whether the evidence names the candidate the page is showing.
    pub health_matches: Option<bool>,
    /// A short `last_error` code, never a path or a free-form reason.
    pub last_error: Option<String>,
    /// Why the query did not produce a usable status.
    pub error: Option<String>,
}

impl Status {
    /// A status that could not be read, with a stable, user-safe reason.
    #[must_use]
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            state: "unavailable".to_owned(),
            error: Some(reason.into()),
            ..Self::default()
        }
    }

    /// Whether the daemon's record is readable and complete enough to show.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.error.is_none() && self.schema.is_some() && self.state != "unavailable"
    }

    /// The state in words a person reads.
    #[must_use]
    pub fn state_label(&self) -> &'static str {
        match self.state.as_str() {
            "ready" => "Ready",
            "uninitialized" => "No update record",
            "disabled" => "Update service not configured",
            "busy" => "Another update operation is running",
            "corrupt" => "Update record is unreadable",
            "incomplete" => "A generation failed its check",
            _ => "Update service unavailable",
        }
    }

    /// The health row, in plain words.
    #[must_use]
    pub fn health_label(&self) -> &'static str {
        match self.health.as_deref() {
            Some("healthy") if self.health_matches == Some(true) => {
                "Healthy, and attested for the candidate"
            }
            Some("healthy") => "Healthy, but for another attempt",
            Some("booting") => "Booted, waiting for the service to attest it",
            Some("failed") => "The last boot failed its health check",
            Some("absent") | None => "No health evidence",
            Some(_) => "Health evidence is not understood",
        }
    }

    /// The active generation, or `None` when the record has no slot/version.
    #[must_use]
    pub fn active_generation(&self) -> Option<String> {
        match (&self.active_slot, &self.active_version) {
            (Some(slot), Some(version)) => Some(format!("Slot {slot} · {version}")),
            _ => None,
        }
    }

    /// The rollback generation, or `None` when no slot and version are known.
    #[must_use]
    pub fn rollback_generation(&self) -> Option<String> {
        match (&self.rollback_slot, &self.rollback_version) {
            (Some(slot), Some(version)) => Some(format!("Slot {slot} · {version}")),
            _ => None,
        }
    }

    /// The candidate generation, or `None` when nothing is staged.
    #[must_use]
    pub fn candidate_generation(&self) -> Option<String> {
        let version = self.candidate_version.as_ref()?;
        let attempts = self
            .candidate_attempts
            .map_or_else(String::new, |n| format!(" · {n} boot attempt(s)"));
        Some(format!("{version}{attempts}"))
    }
}

/// Ask `litric-sysd` for schema 3 and parse the reply.
#[must_use]
pub fn read(socket: &Path) -> Status {
    let Some(reply) = query(socket) else {
        return Status::unavailable("litric-sysd did not answer");
    };
    parse(&reply)
}

#[cfg(unix)]
fn query(socket: &Path) -> Option<String> {
    litric_sysd::server::query(socket, "UPDATE STATUS 3").ok()
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

    let schema = field("schema").and_then(|v| v.parse::<u8>().ok());
    let state = field("state")
        .filter(|v| {
            matches!(
                *v,
                "ready"
                    | "uninitialized"
                    | "disabled"
                    | "busy"
                    | "corrupt"
                    | "incomplete"
                    | "unavailable"
            )
        })
        .unwrap_or("unavailable");
    if state == "unavailable" {
        return Status::unavailable("the daemon did not report a usable state");
    }

    let slot = field("active_slot").filter(|v| *v == "A" || *v == "B");
    let version = field("active_version").and_then(safe_version);
    let rollback_slot = field("rollback_slot").filter(|v| *v == "A" || *v == "B");
    let rollback = field("rollback_version").and_then(safe_version);
    let candidate = field("candidate_version").and_then(safe_version);
    let attempts = field("candidate_attempts").and_then(|v| v.parse::<u8>().ok());
    let health =
        field("health_evidence").filter(|v| matches!(*v, "healthy" | "booting" | "failed"));
    let matches = field("health_matches").and_then(|v| match v {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    });
    let last_error = field("last_error").and_then(safe_error);

    Status {
        schema,
        state: state.to_owned(),
        active_slot: slot.map(str::to_owned),
        active_version: version,
        rollback_slot: rollback_slot.map(str::to_owned),
        rollback_version: rollback,
        candidate_version: candidate,
        candidate_attempts: attempts,
        health: health.map(str::to_owned),
        health_matches: matches,
        last_error,
        error: None,
    }
}

/// A version is display data only here. Accept the same small character set the
/// engine writes, and cap it so a corrupt record cannot make a row enormous.
fn safe_version(raw: &str) -> Option<String> {
    if raw.is_empty() || raw.len() > 64 {
        return None;
    }
    if !raw
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+' | b'_'))
    {
        return None;
    }
    Some(raw.to_owned())
}

/// An error code the page may show: one short token, no path and no spaces.
fn safe_error(raw: &str) -> Option<String> {
    let code = raw.split_whitespace().next()?;
    let code = safe_token(code)?;
    (!code.is_empty()).then_some(code)
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
    fn a_ready_record_keeps_the_generations_it_names() {
        let status = parse(
            "OK 10\nschema=3\nstate=ready\nactive_slot=A\nactive_version=1.4.0\n\
             committed_slot=A\ncandidate_slot=B\ncandidate_version=1.5.0\n\
             candidate_attempts=2\nrollback_slot=B\nrollback_version=1.3.0\n\
             health_evidence=healthy\nhealth_matches=0\n",
        );
        assert!(status.is_available());
        assert_eq!(status.state_label(), "Ready");
        assert_eq!(
            status.active_generation().as_deref(),
            Some("Slot A · 1.4.0")
        );
        assert_eq!(
            status.candidate_generation().as_deref(),
            Some("1.5.0 · 2 boot attempt(s)")
        );
        assert_eq!(status.rollback_version.as_deref(), Some("1.3.0"));
        assert_eq!(
            status.rollback_generation().as_deref(),
            Some("Slot B · 1.3.0")
        );
        assert_eq!(status.health_matches, Some(false));
    }

    #[test]
    fn an_uninitialized_root_is_a_fact_not_an_empty_generation() {
        let status = parse("OK 3\nschema=3\nstate=uninitialized\nactive_slot=none\n");
        assert!(status.is_available());
        assert_eq!(status.state_label(), "No update record");
        assert!(status.active_generation().is_none());
        assert!(status.candidate_generation().is_none());
    }

    #[test]
    fn a_refusal_keeps_only_the_typed_kind() {
        let status = parse("ERR schema-unsupported requested schema 9; this daemon speaks 1..=3");
        assert!(!status.is_available());
        assert_eq!(status.error.as_deref(), Some("schema-unsupported"));
        assert!(!status.error.as_deref().unwrap_or("").contains('/'));
    }

    #[test]
    fn unknown_fields_and_paths_are_ignored() {
        let status = parse(
            "OK 5\nschema=3\nstate=ready\nroot=/var/lib/litric/update\n\
             active_slot=A\nactive_version=1.0.0\nfuture_field=whatever\n\
             last_error=stage:source-unsafe /secret/path\n",
        );
        assert_eq!(status.last_error.as_deref(), Some("stage:source-unsafe"));
        assert_eq!(
            status.active_generation().as_deref(),
            Some("Slot A · 1.0.0")
        );
    }

    #[test]
    fn a_missing_state_is_unavailable_not_ready() {
        let status = parse("OK 1\nschema=3\n");
        assert!(!status.is_available());
        assert_eq!(status.state_label(), "Update service unavailable");
    }

    #[test]
    fn an_unknown_version_is_not_echoed() {
        let status =
            parse("OK 3\nschema=3\nstate=ready\nactive_slot=A\nactive_version=../../etc\n");
        assert!(status.active_generation().is_none());
    }
}
