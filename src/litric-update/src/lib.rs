//! LitricOS system generations: A/B state, staging, boot counting and rollback.
//!
//! # What this is
//!
//! The update half of ADR-0021's four concepts. A **generation** is an
//! immutable, bootable system tree; user data, snapshots and backups are not
//! generations and this crate never writes them. The engine owns exactly two
//! slots, records which one is active, stages a candidate into the inactive
//! slot, counts boots without a health commit, and can move the active pointer
//! back to a committed generation.
//!
//! ```text
//! <root>/system/
//!   state                 active slot, both slot records, bounded switch log
//!   slots/A/generation    the generation marker (protocol + version)
//!   slots/B/generation
//! ```
//!
//! # The rules that matter
//!
//! * **A candidate is never active until a health check commits it.** Staging
//!   only writes the inactive slot.
//! * **Three failed boots discard the candidate.** The active generation stays
//!   active; the engine does not switch to a slot that has not been proven.
//! * **A slot is advertised only when it can be booted.** The state file and
//!   the marker are checked before commit and before rollback.
//! * **Anti-downgrade is a refusal, not a warning.** `stage` rejects a version
//!   that is not newer than the active one.
//! * **The state write is atomic.** Temporary file in the same directory,
//!   `fsync`, `rename`, directory `fsync`; a failed write leaves the previous
//!   record in force.
//! * **User data is never in the transaction.** Only `<root>/system` is touched,
//!   and the lifecycle test proves a canary outside it survives rollback.
//!
//! # Trust boundary
//!
//! This is a mechanism, not a downloader and not a signature verifier. The
//! caller must verify the source tree's signature, hash and target before
//! calling [`Engine::stage`]; the engine proves that what it copied is what it
//! later advertises. The root is assumed to be owned by the update service
//! (`root:root`, not writable by the user); a caller that can replace
//! `<root>/system` can replace the state, exactly as it could replace the
//! bootloader's configuration.
//!
//! One writer is assumed. The state file has no lock: two update services
//! writing the same root would race, and a file lock would be a second
//! mechanism with its own crash semantics. The systemd unit is the serialisation
//! point when this is wired into an image.

#![forbid(unsafe_code)]

mod engine;
mod fsutil;
mod state;
#[cfg(test)]
mod tests;

pub use engine::{BootOutcome, Engine, Limits};
pub use state::{
    Event, EventKind, Generation, GenerationState, RollbackReason, Slot, UpdateState,
    UserDataState, MAX_BOOT_ATTEMPTS, MAX_EVENTS, MAX_LINE_BYTES, MAX_STATE_BYTES, STATE_PROTOCOL,
    STATE_VERSION,
};

use core::fmt;
use litric_core::version::VersionError;
use std::io;
use std::path::PathBuf;

/// Seconds since the Unix epoch, supplied by the caller.
///
/// The engine never reads a clock, so a test can assert exact switch records
/// without sleeping and a replay has stable timestamps.
pub type Timestamp = u64;

/// Why a generation operation was refused.
#[derive(Debug)]
#[non_exhaustive]
pub enum UpdateError {
    /// A filesystem operation failed. The path and the OS error kind are kept
    /// structured; the message is for the operator, never for control flow.
    Io {
        /// What was being attempted, e.g. `read generation state`.
        action: &'static str,
        /// The path involved.
        path: PathBuf,
        /// The OS error kind.
        kind: io::ErrorKind,
        /// The OS message, retained for diagnosis.
        message: String,
    },
    /// A state record is unreadable, internally inconsistent, or past a bound.
    InvalidState(String),
    /// The state file does not exist; the root was never initialised.
    MissingState(PathBuf),
    /// Initialisation would overwrite an existing state record.
    StateAlreadyExists(PathBuf),
    /// A slot directory exists where initialisation expected an empty root.
    SlotAlreadyExists(Slot),
    /// A version string is not a valid LitricOS version.
    BadVersion(VersionError),
    /// A slot name is not `A` or `B`.
    InvalidSlot(String),
    /// The candidate is not newer than the active generation.
    NotAnUpgrade {
        /// The generation currently active.
        active: litric_core::Version,
        /// The refused candidate.
        candidate: litric_core::Version,
    },
    /// A candidate is already staged; commit or discard it first.
    AlreadyPending(Slot),
    /// There is no candidate to commit.
    NoPendingCandidate,
    /// A rollback was requested while a candidate is staged.
    PendingCandidate(Slot),
    /// No other committed generation can be activated.
    NoRollbackTarget,
    /// The slot's generation marker is missing, malformed, or names another
    /// version, so the slot cannot be advertised or activated.
    CandidateIncomplete {
        /// The slot that failed its check.
        slot: Slot,
        /// What was wrong.
        reason: String,
    },
    /// The staging source is not a directory.
    SourceNotDirectory(PathBuf),
    /// The source contains a symlink or another entry that is not a regular
    /// file or a directory. A generation must be a closed tree, so following a
    /// link out of it would copy something the manifest did not describe.
    UnsafeSource {
        /// The offending entry, relative to the source root.
        path: PathBuf,
        /// What kind of entry it was.
        kind: &'static str,
    },
    /// The source has more entries than the configured staging bound.
    TooManyEntries {
        /// The effective bound.
        limit: u64,
    },
    /// The source is larger than the configured staging bound.
    TooManyBytes {
        /// The effective bound.
        limit: u64,
    },
    /// A relative source path is longer than the configured bound.
    PathTooLong {
        /// The bound in bytes.
        limit: usize,
    },
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                action,
                path,
                kind,
                message,
            } => write!(f, "{action} {}: {message} ({kind:?})", path.display()),
            Self::InvalidState(reason) => write!(f, "invalid generation state: {reason}"),
            Self::MissingState(path) => {
                write!(f, "no generation state at {}", path.display())
            }
            Self::StateAlreadyExists(path) => {
                write!(f, "generation state already exists at {}", path.display())
            }
            Self::SlotAlreadyExists(slot) => {
                write!(
                    f,
                    "slot {} already exists; refusing to initialise over it",
                    slot.as_str()
                )
            }
            Self::BadVersion(error) => write!(f, "invalid generation version: {error}"),
            Self::InvalidSlot(raw) => write!(f, "invalid slot {raw:?}; expected A or B"),
            Self::NotAnUpgrade { active, candidate } => write!(
                f,
                "candidate {candidate} is not newer than the active generation {active}"
            ),
            Self::AlreadyPending(slot) => {
                write!(f, "slot {} already holds a staged candidate", slot.as_str())
            }
            Self::NoPendingCandidate => write!(f, "there is no staged candidate to commit"),
            Self::PendingCandidate(slot) => write!(
                f,
                "slot {} is still staged; commit or discard it before rolling back",
                slot.as_str()
            ),
            Self::NoRollbackTarget => {
                write!(
                    f,
                    "the other slot has no committed generation to roll back to"
                )
            }
            Self::CandidateIncomplete { slot, reason } => {
                write!(f, "slot {} cannot be activated: {reason}", slot.as_str())
            }
            Self::SourceNotDirectory(path) => {
                write!(f, "staging source {} is not a directory", path.display())
            }
            Self::UnsafeSource { path, kind } => write!(
                f,
                "staging source contains an unsafe {kind} at {}",
                path.display()
            ),
            Self::TooManyEntries { limit } => {
                write!(f, "staging source has more than {limit} entries")
            }
            Self::TooManyBytes { limit } => {
                write!(f, "staging source is larger than the {limit}-byte bound")
            }
            Self::PathTooLong { limit } => {
                write!(f, "a staging source path is longer than {limit} bytes")
            }
        }
    }
}

impl std::error::Error for UpdateError {}

/// The crate's result alias.
pub type Result<T> = core::result::Result<T, UpdateError>;

pub(crate) fn io_error(
    action: &'static str,
    path: &std::path::Path,
    error: io::Error,
) -> UpdateError {
    UpdateError::Io {
        action,
        path: path.to_path_buf(),
        kind: error.kind(),
        message: error.to_string(),
    }
}
