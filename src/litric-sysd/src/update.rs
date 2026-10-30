//! The updater half of `litric-sysd`: generation status, staged candidates,
//! health-gated commits and rollback.
//!
//! The engine in `litric-update` owns the state machine; this module owns the
//! service boundary around it. It is deliberately small and has three jobs:
//!
//! * resolve one configured root, never a path from a request;
//! * turn the engine's typed results into the daemon's line protocol;
//! * keep the boot entrypoint (`litric-sysd --boot`) separate from the socket
//!   service, so a boot attempt is counted before any client can ask questions.
//!
//! # What the status is allowed to say
//!
//! A missing state is `state=uninitialized`, not an error and not an invented
//! active slot. A state that cannot be read is `state=corrupt` with its reason;
//! the daemon still starts and still answers, because a machine with a damaged
//! update record has to be diagnosable. Mutating actions are refused in both
//! cases. No status line is a guess: `none` means the engine has no record, and
//! `attempts=0` always means zero recorded attempts rather than "unknown".
//!
//! # Health is a signal, not a probe
//!
//! `commit` accepts [`HealthSignal`] from the caller. This module does not run
//! a session, mount a storage domain or ask a service whether it is healthy;
//! the engine's original contract is that those checks happen before the commit
//! seam. The signal is explicit and typed so a failed check cannot be
//! accidental, and the protocol requires the confirmation word so a single
//! malformed line cannot activate a generation.
//!
//! # The boot budget and "rollback"
//!
//! A staged candidate is never the active pointer. The boot entrypoint counts
//! an attempt for it and, after the fourth request, the engine marks it bad and
//! clears it. The active generation was never moved, so there is no slot switch
//! to perform and calling `Engine::rollback` would be wrong (there is no other
//! committed target). The observable guarantee is the one that matters: the
//! machine remains on the previously committed generation, and the rejected
//! candidate is no longer advertised. Manual `UPDATE ROLLBACK` is the path that
//! actually moves the pointer and records a `rollback` event.

use std::fs;
use std::path::{Component, Path, PathBuf};

use litric_core::Version;
use litric_update::{
    BootOutcome, Engine, Event, EventKind, GenerationState, RollbackReason, UpdateError,
    UpdateState,
};

use crate::state::Timestamp;

/// The update IPC schema reported by `UPDATE STATUS`.
///
/// It is separate from `litric-update`'s on-disk state version: the wire may
/// grow a field without changing the record on disk, and a client must not
/// mistake the two.
pub const SCHEMA: u8 = 1;

/// The root installed by the image. One root, one writer.
pub const DEFAULT_ROOT: &str = "/var/lib/litric/update";

/// What the caller says its health check concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthSignal {
    /// The caller's checks passed. The engine still verifies the marker.
    Passed,
    /// The caller's checks failed. No commit is attempted.
    Failed,
}

impl HealthSignal {
    /// Parse the protocol word.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "passed" => Some(Self::Passed),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }

    /// The word used on the wire.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

/// The daemon's updater service.
///
/// `engine` is opened lazily so a daemon with no update root (tests, `--report`)
/// does not touch `/var`, and so a root initialised while the daemon is running
/// becomes visible on the next status query. Once opened it is the writer; the
/// engine's single-writer assumption is the systemd unit's serialisation.
#[derive(Debug)]
pub struct UpdateService {
    root: Option<PathBuf>,
    engine: Option<Engine>,
    last_error: Option<String>,
}

impl UpdateService {
    /// A service with no root. Every mutating request is refused and status is
    /// `state=disabled`.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            root: None,
            engine: None,
            last_error: None,
        }
    }

    /// A service rooted at `root`. Nothing is read until the first request.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
            engine: None,
            last_error: None,
        }
    }

    /// The configured root, for the console line and for tests.
    #[must_use]
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Read the current generation state as protocol lines.
    ///
    /// This never returns an error: a status question must have an answer even
    /// when the answer is "there is no state" or "the state is damaged".
    pub fn status(&mut self) -> Vec<String> {
        let last_error = self.last_error.clone();
        let mut lines = vec![format!("schema={SCHEMA}")];
        let Some(root) = self.root.clone() else {
            lines.push("state=disabled".to_owned());
            return lines;
        };
        lines.push(format!("root={}", root.display()));

        // Read the record rather than reusing the mutating service's cached
        // engine. The boot entrypoint is a separate process that writes this
        // same root before `--serve`; a status query must show what is on disk,
        // not what this process last remembered.
        match Engine::open(&root) {
            Ok(engine) => {
                let state = engine.state();
                let mut problem = None;
                if let Err(error) = engine.verify_generation(state.active()) {
                    problem = Some(error);
                }
                if problem.is_none() {
                    if let Some(pending) = state.pending() {
                        if let Err(error) = engine.verify_generation(pending) {
                            problem = Some(error);
                        }
                    }
                }
                if problem.is_none() && state.can_rollback() {
                    if let Err(error) = engine.verify_generation(state.active().other()) {
                        problem = Some(error);
                    }
                }
                match problem {
                    None => lines.push("state=ready".to_owned()),
                    Some(error) => {
                        lines.push("state=incomplete".to_owned());
                        lines.push(format!("reason={error}"));
                    }
                }
                append_state(&mut lines, state);
            }
            Err(UpdateError::MissingState(_)) => {
                lines.push("state=uninitialized".to_owned());
                append_empty_state(&mut lines);
            }
            Err(error) => {
                lines.push("state=corrupt".to_owned());
                lines.push(format!("reason={error}"));
            }
        }
        if let Some(error) = last_error {
            lines.push(format!("last_error={error}"));
        }
        lines
    }

    /// Stage a candidate from `<root>/incoming/<source>`.
    ///
    /// `source` is a relative path below the service-owned incoming directory.
    /// Absolute paths, `.` and `..` are refused before the engine sees them:
    /// the request names one downloaded tree, not an arbitrary directory the
    /// daemon can read with its privilege.
    ///
    /// # Errors
    /// [`UpdateError::UnsafeSource`] for a path outside `incoming`, and every
    /// engine refusal (no state, downgrade, pending candidate, unsafe tree,
    /// staging bound).
    pub fn stage(&mut self, version: Version, source: &str) -> Result<Vec<String>, UpdateError> {
        let Some(root) = self.root.clone() else {
            let error = UpdateError::MissingState(PathBuf::from("<disabled>"));
            self.last_error = Some(format!("stage:{}", error.kind()));
            return Err(error);
        };
        let source = match source_path(&root, source) {
            Ok(path) => path,
            Err(error) => {
                self.last_error = Some(format!("stage:{}", error.kind()));
                return Err(error);
            }
        };
        let result = (|| {
            let engine = self.engine()?;
            engine.stage(&source, version)
        })();
        match result {
            Ok(slot) => {
                self.last_error = None;
                Ok(vec![
                    format!("slot={}", slot.as_str()),
                    "state=staged".to_owned(),
                    format!("source={}", source.display()),
                ])
            }
            Err(error) => {
                self.last_error = Some(format!("stage:{}", error.kind()));
                Err(error)
            }
        }
    }

    /// Commit the staged candidate only after the caller's health signal.
    ///
    /// The engine checks the marker and the state again; this method checks the
    /// caller's signal first so a failed check cannot accidentally reach it.
    ///
    /// # Errors
    /// [`UpdateError::HealthNotPassed`], [`UpdateError::NoPendingCandidate`],
    /// [`UpdateError::CandidateIncomplete`] or an I/O failure.
    pub fn commit(
        &mut self,
        health: HealthSignal,
        now: Timestamp,
    ) -> Result<Vec<String>, UpdateError> {
        if health != HealthSignal::Passed {
            let error = UpdateError::HealthNotPassed;
            self.last_error = Some(format!("commit:{}", error.kind()));
            return Err(error);
        }
        let result = (|| {
            let engine = self.engine()?;
            engine.commit(now)
        })();
        match result {
            Ok(event) => {
                self.last_error = None;
                Ok(event_lines(&event))
            }
            Err(error) => {
                self.last_error = Some(format!("commit:{}", error.kind()));
                Err(error)
            }
        }
    }

    /// Move the active pointer back to the other committed generation.
    ///
    /// # Errors
    /// [`UpdateError::NoRollbackTarget`], [`UpdateError::PendingCandidate`],
    /// [`UpdateError::CandidateIncomplete`] or an I/O failure.
    pub fn rollback(
        &mut self,
        reason: RollbackReason,
        now: Timestamp,
    ) -> Result<Vec<String>, UpdateError> {
        let result = (|| {
            let engine = self.engine()?;
            engine.rollback(reason, now)
        })();
        match result {
            Ok(event) => {
                self.last_error = None;
                Ok(event_lines(&event))
            }
            Err(error) => {
                self.last_error = Some(format!("rollback:{}", error.kind()));
                Err(error)
            }
        }
    }

    fn engine(&mut self) -> Result<&mut Engine, UpdateError> {
        if self.engine.is_none() {
            let root = self
                .root
                .clone()
                .ok_or_else(|| UpdateError::MissingState(PathBuf::from("<disabled>")))?;
            self.engine = Some(Engine::open(root)?);
        }
        self.engine
            .as_mut()
            .ok_or_else(|| UpdateError::MissingState(PathBuf::from("<disabled>")))
    }
}

/// Run the boot entrypoint. It never returns an error: a machine whose update
/// record is damaged must still boot and must still report the damage.
///
/// The lines are prefixed `litric.update.` so the serial console, the journal
/// and `tools/boot-test.sh` can find them without parsing prose.
#[must_use]
pub fn boot(root: &Path, now: Timestamp) -> Vec<String> {
    let mut lines = vec![
        format!("litric.update.root={}", root.display()),
        format!("litric.update.schema={SCHEMA}"),
    ];
    match Engine::open(root) {
        Ok(mut engine) => {
            lines.push("litric.update.state=ready".to_owned());
            match engine.note_boot(now) {
                Ok(BootOutcome::Active { slot }) => {
                    lines.push("litric.update.outcome=active".to_owned());
                    lines.push(format!("litric.update.slot={}", slot.as_str()));
                }
                Ok(BootOutcome::Booting { slot, attempt }) => {
                    lines.push("litric.update.outcome=booting".to_owned());
                    lines.push(format!("litric.update.slot={}", slot.as_str()));
                    lines.push(format!("litric.update.attempt={attempt}"));
                }
                Ok(BootOutcome::CandidateRejected { slot, attempts }) => {
                    // The active pointer never moved. This is the engine's
                    // automatic return to the committed generation: the failed
                    // candidate is cleared, not activated and then switched.
                    lines.push("litric.update.outcome=rejected-budget".to_owned());
                    lines.push(format!("litric.update.slot={}", slot.as_str()));
                    lines.push(format!("litric.update.attempts={attempts}"));
                    lines.push("litric.update.rollback=active-generation".to_owned());
                }
                Err(error) => {
                    lines.push("litric.update.outcome=error".to_owned());
                    lines.push(format!("litric.update.reason={error}"));
                }
                // BootOutcome is non-exhaustive. A future outcome must be
                // reported as unrecognised rather than folded into a success.
                Ok(_) => lines.push("litric.update.outcome=unrecognised".to_owned()),
            }
            lines.push(format!(
                "litric.update.active={}",
                engine.state().active().as_str()
            ));
            if let Some(pending) = engine.state().pending() {
                lines.push(format!("litric.update.candidate={}", pending.as_str()));
            } else {
                lines.push("litric.update.candidate=none".to_owned());
            }
        }
        Err(UpdateError::MissingState(_)) => {
            // No update record is a normal machine that has never staged an
            // update. It is not an error and this entrypoint writes nothing.
            lines.push("litric.update.state=uninitialized".to_owned());
            lines.push("litric.update.outcome=none".to_owned());
        }
        Err(error) => {
            lines.push("litric.update.state=corrupt".to_owned());
            lines.push(format!("litric.update.reason={error}"));
        }
    }
    lines.push("litric.update.ready=1".to_owned());
    lines
}

fn source_path(root: &Path, raw: &str) -> Result<PathBuf, UpdateError> {
    let path = Path::new(raw);
    if raw.is_empty() || raw.len() > 4096 || path.is_absolute() {
        return Err(UpdateError::UnsafeSource {
            path: path.to_path_buf(),
            kind: "source path",
        });
    }
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            _ => {
                return Err(UpdateError::UnsafeSource {
                    path: path.to_path_buf(),
                    kind: "source path",
                });
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(UpdateError::UnsafeSource {
            path: path.to_path_buf(),
            kind: "source path",
        });
    }
    let incoming = root.join("incoming");
    let source = incoming.join(relative);
    // `read_link` is not enough here: the engine refuses a symlinked source as
    // well, and this check names the service directory before the copy begins.
    match fs::symlink_metadata(&source) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(UpdateError::UnsafeSource {
            path: source,
            kind: "symlink",
        }),
        _ => Ok(source),
    }
}

fn append_state(lines: &mut Vec<String>, state: &UpdateState) {
    let active = state.active();
    lines.push(format!("active_slot={}", active.as_str()));
    if let Some(generation) = state.generation(active) {
        lines.push(format!("active_version={}", generation.version));
        lines.push(format!(
            "active_state={}",
            generation_state(generation.state)
        ));
    }
    lines.push(format!(
        "committed_slot={}",
        if state
            .generation(active)
            .is_some_and(|generation| generation.state == GenerationState::Committed)
        {
            active.as_str()
        } else {
            "none"
        }
    ));

    if let Some(pending) = state.pending() {
        lines.push(format!("candidate_slot={}", pending.as_str()));
        if let Some(generation) = state.generation(pending) {
            lines.push(format!("candidate_version={}", generation.version));
            lines.push(format!("candidate_attempts={}", generation.attempts));
            lines.push(format!(
                "candidate_state={}",
                generation_state(generation.state)
            ));
        }
    } else {
        append_empty_candidate(lines);
    }

    let other = active.other();
    if let Some(generation) = state
        .generation(other)
        .filter(|generation| generation.state == GenerationState::Committed)
    {
        lines.push(format!("rollback_slot={}", other.as_str()));
        lines.push(format!("rollback_version={}", generation.version));
    } else {
        lines.push("rollback_slot=none".to_owned());
        lines.push("rollback_version=none".to_owned());
    }
    lines.push(format!("can_rollback={}", u8::from(state.can_rollback())));

    match state.events().last() {
        Some(event) => {
            lines.push(format!(
                "last_result={}:{}",
                event_kind(event.kind),
                reason(event.reason)
            ));
            lines.push(format!("last_result_at={}", event.at));
            if event.kind == EventKind::RolledBack {
                lines.push(format!("last_rollback_reason={}", reason(event.reason)));
            } else {
                lines.push("last_rollback_reason=none".to_owned());
            }
        }
        None => {
            lines.push("last_result=none".to_owned());
            lines.push("last_result_at=0".to_owned());
            lines.push("last_rollback_reason=none".to_owned());
        }
    }
}

fn append_empty_state(lines: &mut Vec<String>) {
    lines.push("active_slot=none".to_owned());
    lines.push("active_version=none".to_owned());
    lines.push("active_state=none".to_owned());
    lines.push("committed_slot=none".to_owned());
    append_empty_candidate(lines);
    lines.push("rollback_slot=none".to_owned());
    lines.push("rollback_version=none".to_owned());
    lines.push("can_rollback=0".to_owned());
    lines.push("last_result=none".to_owned());
    lines.push("last_result_at=0".to_owned());
    lines.push("last_rollback_reason=none".to_owned());
}

fn append_empty_candidate(lines: &mut Vec<String>) {
    lines.push("candidate_slot=none".to_owned());
    lines.push("candidate_version=none".to_owned());
    lines.push("candidate_attempts=0".to_owned());
    lines.push("candidate_state=none".to_owned());
}

fn event_lines(event: &Event) -> Vec<String> {
    vec![
        format!("event={}", event_kind(event.kind)),
        format!("from={}", event.from.as_str()),
        format!("to={}", event.to.as_str()),
        format!("reason={}", reason(event.reason)),
        format!("user_data={}", user_data(event)),
        format!("at={}", event.at),
    ]
}

fn generation_state(state: GenerationState) -> &'static str {
    match state {
        GenerationState::Committed => "committed",
        GenerationState::Staged => "staged",
        GenerationState::Bad => "bad",
    }
}

fn event_kind(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Committed => "commit",
        EventKind::Rejected => "reject",
        EventKind::RolledBack => "rollback",
    }
}

fn reason(reason: RollbackReason) -> &'static str {
    match reason {
        RollbackReason::HealthCheck => "health-check",
        RollbackReason::BootCount => "boot-count",
        RollbackReason::Corruption => "corruption",
        RollbackReason::Manual => "manual",
        RollbackReason::UserCancelled => "user-cancelled",
    }
}

fn user_data(event: &Event) -> &'static str {
    match event.user_data {
        litric_update::UserDataState::Untouched => "untouched",
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    const NOW: Timestamp = 1_800_000_000;

    fn version(raw: &str) -> Version {
        Version::parse(raw).expect("test version")
    }

    fn scratch(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("litric-sysd-update-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create scratch");
        path
    }

    fn init(root: &Path) -> Engine {
        Engine::init(root, version("1.0.0")).expect("init")
    }

    fn source(root: &Path) -> PathBuf {
        let incoming = root.join("incoming/release");
        fs::create_dir_all(&incoming).expect("create incoming");
        fs::write(incoming.join("app"), "new tree").expect("write tree");
        incoming
    }

    fn line<'a>(lines: &'a [String], prefix: &str) -> Option<&'a String> {
        lines.iter().find(|line| line.starts_with(prefix))
    }

    fn status(s: &mut UpdateService) -> Vec<String> {
        s.status()
    }

    #[test]
    fn status_before_any_state_is_uninitialized_and_actions_refuse() {
        let root = scratch("empty");
        let mut service = UpdateService::new(&root);
        let lines = status(&mut service);
        assert_eq!(
            line(&lines, "state="),
            Some(&"state=uninitialized".to_owned())
        );
        assert_eq!(
            line(&lines, "active_slot="),
            Some(&"active_slot=none".to_owned())
        );
        let error = service
            .commit(HealthSignal::Passed, NOW)
            .expect_err("no state");
        assert_eq!(error.kind(), "missing-state");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn corrupt_state_is_reported_and_mutations_are_refused() {
        let root = scratch("corrupt");
        fs::create_dir_all(root.join("system")).expect("state dir");
        fs::write(root.join("system/state"), "not a generation record\n").expect("damage");
        let mut service = UpdateService::new(&root);
        let lines = status(&mut service);
        assert_eq!(line(&lines, "state="), Some(&"state=corrupt".to_owned()));
        let error = service
            .rollback(RollbackReason::Manual, NOW)
            .expect_err("corrupt state");
        assert_eq!(error.kind(), "invalid-state");
        assert!(line(&status(&mut service), "last_error=").is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_failed_health_signal_never_commits() {
        let root = scratch("health");
        init(&root);
        source(&root);
        let mut service = UpdateService::new(&root);
        service.stage(version("1.1.0"), "release").expect("stage");
        let error = service
            .commit(HealthSignal::Failed, NOW + 1)
            .expect_err("failed health");
        assert_eq!(error.kind(), "health-not-passed");
        let lines = status(&mut service);
        assert_eq!(
            line(&lines, "active_slot="),
            Some(&"active_slot=A".to_owned())
        );
        assert_eq!(
            line(&lines, "candidate_slot="),
            Some(&"candidate_slot=B".to_owned())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn the_service_rolls_back_and_leaves_user_data_byte_for_byte() {
        let root = scratch("rollback");
        init(&root);
        source(&root);
        let user = root.join("user/notes.txt");
        fs::create_dir_all(user.parent().expect("parent")).expect("user dir");
        fs::write(&user, "the user's work").expect("user data");

        let mut service = UpdateService::new(&root);
        service.stage(version("1.1.0"), "release").expect("stage");
        service
            .commit(HealthSignal::Passed, NOW + 1)
            .expect("commit");
        let lines = service
            .rollback(RollbackReason::Manual, NOW + 2)
            .expect("rollback");
        assert_eq!(line(&lines, "event="), Some(&"event=rollback".to_owned()));
        assert_eq!(
            line(&status(&mut service), "active_slot="),
            Some(&"active_slot=A".to_owned())
        );
        assert_eq!(
            fs::read_to_string(&user).expect("user data survives"),
            "the user's work"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn boot_budget_exhaustion_returns_to_the_active_generation() {
        let root = scratch("budget");
        init(&root);
        source(&root);
        let user = root.join("user/notes.txt");
        fs::create_dir_all(user.parent().expect("parent")).expect("user dir");
        fs::write(&user, "keep me").expect("user data");
        let mut service = UpdateService::new(&root);
        service.stage(version("1.1.0"), "release").expect("stage");

        for _ in 0..3 {
            let lines = boot(&root, NOW);
            assert!(line(&lines, "litric.update.outcome=")
                .is_some_and(|line| line == "litric.update.outcome=booting"));
        }
        let lines = boot(&root, NOW);
        assert_eq!(
            line(&lines, "litric.update.outcome="),
            Some(&"litric.update.outcome=rejected-budget".to_owned())
        );
        assert_eq!(
            line(&lines, "litric.update.active="),
            Some(&"litric.update.active=A".to_owned())
        );

        let state = status(&mut service);
        assert_eq!(
            line(&state, "active_slot="),
            Some(&"active_slot=A".to_owned())
        );
        assert_eq!(
            line(&state, "candidate_slot="),
            Some(&"candidate_slot=none".to_owned())
        );
        assert_eq!(
            line(&state, "last_result="),
            Some(&"last_result=reject:boot-count".to_owned())
        );
        assert_eq!(fs::read_to_string(&user).expect("user data"), "keep me");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_missing_slot_directory_is_incomplete_not_a_panic() {
        let root = scratch("missing-slot");
        init(&root);
        fs::remove_dir_all(root.join("system/slots/A")).expect("remove active slot");
        let mut service = UpdateService::new(&root);
        let lines = status(&mut service);
        assert_eq!(line(&lines, "state="), Some(&"state=incomplete".to_owned()));
        assert!(
            line(&lines, "reason=").is_some(),
            "the missing slot must be named: {lines:?}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn staging_cannot_escape_the_incoming_directory() {
        let root = scratch("escape");
        init(&root);
        let mut service = UpdateService::new(&root);
        for source in ["../release", "/etc", ".", "a/../b"] {
            let error = service
                .stage(version("1.1.0"), source)
                .expect_err("escape must be refused");
            assert_eq!(error.kind(), "unsafe-source");
        }
        let _ = fs::remove_dir_all(root);
    }
}
