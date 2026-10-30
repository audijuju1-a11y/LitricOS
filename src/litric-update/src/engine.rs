//! The staging engine and its state machine.
//!
//! The order of operations in [`Engine::stage`] is the crash story:
//!
//! 1. persist the inactive slot as empty, so no old generation is advertised
//!    while its directory is being replaced;
//! 2. remove the old directory and copy the source into a hidden staging
//!    directory on the same filesystem;
//! 3. write and sync the generation marker, then rename the staging directory
//!    into the slot in one atomic step;
//! 4. persist the candidate and its boot budget.
//!
//! A crash between any two steps leaves either the active generation untouched
//! or an unadvertised staging directory that the next stage removes. The active
//! pointer never moves until [`Engine::commit`] has seen the marker and the
//! caller has completed its health checks.

use crate::fsutil::{ensure_private_dir, remove_any, sync_dir, sync_file};
use crate::state::{
    Event, EventKind, Generation, GenerationState, RollbackReason, Slot, Store, UpdateState,
    UserDataState, MAX_BOOT_ATTEMPTS,
};
use crate::{io_error, Result, Timestamp, UpdateError};
use litric_core::Version;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The default maximum number of files and directories a candidate may contain.
pub const DEFAULT_MAX_ENTRIES: u64 = 100_000;
/// The default maximum number of bytes a candidate may contain.
pub const DEFAULT_MAX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// The default maximum relative path length accepted from a candidate.
pub const DEFAULT_MAX_PATH_BYTES: usize = 4096;

/// Bounds applied while copying a candidate.
///
/// The defaults are deliberately generous for a system image and still finite:
/// an update source is the one input that can fill a disk or exhaust an inode
/// table, so it is measured before the slot is published.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Maximum number of directory entries (files plus directories).
    pub max_entries: u64,
    /// Maximum sum of regular-file sizes, bytes.
    pub max_bytes: u64,
    /// Maximum length of a relative path, in bytes of its display form.
    pub max_path_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_entries: DEFAULT_MAX_ENTRIES,
            max_bytes: DEFAULT_MAX_BYTES,
            max_path_bytes: DEFAULT_MAX_PATH_BYTES,
        }
    }
}

/// What the boot path should do after asking the engine about this boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BootOutcome {
    /// No candidate is staged; boot the committed active slot.
    Active {
        /// The active slot.
        slot: Slot,
    },
    /// A candidate is still inside its boot budget; boot it and ask for a
    /// health commit when the session, storage and critical services are up.
    Booting {
        /// The candidate slot.
        slot: Slot,
        /// The attempt number, starting at one.
        attempt: u8,
    },
    /// The candidate used its budget or its marker is not bootable. It has been
    /// rejected; boot the active slot instead.
    CandidateRejected {
        /// The rejected slot.
        slot: Slot,
        /// The attempts it had accumulated.
        attempts: u8,
    },
}

/// A root with two generation slots and one persisted state record.
#[derive(Debug, Clone)]
pub struct Engine {
    store: Store,
    state: UpdateState,
    limits: Limits,
}

impl Engine {
    /// Initialise a new root with one committed generation in slot A.
    ///
    /// The caller installs the matching system tree before booting; the marker
    /// written here is what later makes that tree admissible. Initialisation
    /// refuses to overwrite an existing state record or a slot directory.
    ///
    /// # Errors
    /// [`UpdateError::StateAlreadyExists`] or [`UpdateError::SlotAlreadyExists`]
    /// when the root is not fresh, and any I/O failure while creating the state.
    pub fn init(root: impl Into<PathBuf>, version: Version) -> Result<Self> {
        let store = Store::new(root);
        let state_path = store.state_path();
        match fs::symlink_metadata(&state_path) {
            Ok(_) => return Err(UpdateError::StateAlreadyExists(state_path)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_error("inspect generation state", &state_path, error)),
        }

        ensure_private_dir(&store.slots_dir())
            .map_err(|error| io_error("create slots directory", &store.slots_dir(), error))?;
        for slot in Slot::ALL {
            let path = store.slot_path(slot);
            match fs::symlink_metadata(&path) {
                Ok(_) => return Err(UpdateError::SlotAlreadyExists(slot)),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(io_error("inspect slot directory", &path, error)),
            }
        }

        let slot_a = store.slot_path(Slot::A);
        fs::create_dir(&slot_a)
            .map_err(|error| io_error("create generation slot", &slot_a, error))?;
        set_dir_mode(&slot_a, 0o755)
            .map_err(|error| io_error("set generation slot mode", &slot_a, error))?;
        store.write_marker(&slot_a, &version)?;
        sync_dir(&slot_a)
            .map_err(|error| io_error("sync initial generation slot", &slot_a, error))?;
        sync_dir(&store.slots_dir())
            .map_err(|error| io_error("sync initial slots directory", &store.slots_dir(), error))?;

        let state = UpdateState::initial(version);
        if let Err(error) = store.save(&state) {
            let _ = remove_any(&slot_a);
            return Err(error);
        }
        Ok(Self {
            store,
            state,
            limits: Limits::default(),
        })
    }

    /// Open an existing root.
    ///
    /// # Errors
    /// [`UpdateError::MissingState`], [`UpdateError::InvalidState`], or an I/O
    /// failure while reading the record.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let store = Store::new(root);
        let state = store.load()?;
        Ok(Self {
            store,
            state,
            limits: Limits::default(),
        })
    }

    /// Override the staging bounds. Intended for tests and for a future
    /// installer that knows the target filesystem's capacity.
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// The bounds in force.
    #[must_use]
    pub const fn limits(&self) -> Limits {
        self.limits
    }

    /// The persisted record.
    #[must_use]
    pub const fn state(&self) -> &UpdateState {
        &self.state
    }

    /// The active generation's version.
    #[must_use]
    pub fn active_version(&self) -> Option<&Version> {
        self.state
            .generation(self.state.active())
            .map(|generation| &generation.version)
    }

    /// Verify that a slot's marker names the version the state expects.
    ///
    /// A read-only probe for the service boundary: status must be able to say
    /// that a slot directory disappeared or was replaced, rather than reporting
    /// a state as ready because the state file alone parses. Activation calls
    /// the same check before it moves the active pointer.
    ///
    /// # Errors
    /// [`UpdateError::CandidateIncomplete`] when the slot directory or marker
    /// is missing, is not a regular file, or names another version.
    pub fn verify_generation(&self, slot: Slot) -> Result<()> {
        self.candidate_ready(slot)
    }

    /// The staged candidate's version.
    #[must_use]
    pub fn pending_version(&self) -> Option<&Version> {
        let pending = self.state.pending()?;
        self.state
            .generation(pending)
            .map(|generation| &generation.version)
    }

    /// Stage `source` into the inactive slot.
    ///
    /// The source is copied, not moved. It must contain only directories and
    /// regular files; a symlink or special file is refused because a generation
    /// must be a closed tree. The active slot and the active pointer do not
    /// change. `version` must be strictly greater than the active version.
    ///
    /// # Errors
    /// Anti-downgrade, a pending candidate, an unsafe source, a bound, or an
    /// I/O failure. A failure leaves the active generation usable; the inactive
    /// slot may be empty, never a half-published candidate.
    pub fn stage(&mut self, source: &Path, version: Version) -> Result<Slot> {
        if let Some(pending) = self.state.pending() {
            return Err(UpdateError::AlreadyPending(pending));
        }
        let active = self.state.active();
        let active_version = self
            .state
            .generation(active)
            .ok_or_else(|| UpdateError::InvalidState("active slot has no generation".to_owned()))?
            .version
            .clone();
        if version <= active_version {
            return Err(UpdateError::NotAnUpgrade {
                active: active_version,
                candidate: version,
            });
        }

        let inactive = active.other();
        let before = self.state.clone();
        self.state.set_generation(inactive, None);
        if let Err(error) = self.store.save(&self.state) {
            self.state = before;
            return Err(error);
        }

        let slot_path = self.store.slot_path(inactive);
        if let Err(error) = remove_any(&slot_path) {
            return Err(io_error("remove old generation slot", &slot_path, error));
        }
        let temp = self.store.stage_temp_path(inactive);
        if let Err(error) = remove_any(&temp) {
            return Err(io_error("remove stale staging directory", &temp, error));
        }

        if let Err(error) = copy_tree(source, &temp, self.limits) {
            let _ = remove_any(&temp);
            return Err(error);
        }
        if let Err(error) = self.store.write_marker(&temp, &version) {
            let _ = remove_any(&temp);
            return Err(error);
        }
        if let Err(error) = sync_dir(&temp) {
            let _ = remove_any(&temp);
            return Err(io_error("sync staging directory", &temp, error));
        }
        if let Err(error) = fs::rename(&temp, &slot_path) {
            let _ = remove_any(&temp);
            return Err(io_error("publish generation slot", &slot_path, error));
        }
        if let Err(error) = sync_dir(&self.store.slots_dir()) {
            let _ = remove_any(&slot_path);
            return Err(io_error(
                "sync generation slots directory",
                &self.store.slots_dir(),
                error,
            ));
        }

        self.state.set_generation(
            inactive,
            Some(Generation {
                version,
                state: GenerationState::Staged,
                attempts: 0,
            }),
        );
        self.state.set_pending(Some(inactive));
        if let Err(error) = self.store.save(&self.state) {
            self.state.set_generation(inactive, None);
            self.state.set_pending(None);
            let _ = remove_any(&slot_path);
            return Err(error);
        }
        Ok(inactive)
    }

    /// Ask what to boot, and count one attempt for a staged candidate.
    ///
    /// The fourth request without a commit discards the candidate, because
    /// [`MAX_BOOT_ATTEMPTS`] is three. A missing or malformed marker discards it
    /// immediately: a slot that cannot prove what it contains is not booted.
    ///
    /// # Errors
    /// An I/O failure while persisting the new count or the rejection.
    pub fn note_boot(&mut self, now: Timestamp) -> Result<BootOutcome> {
        let Some(pending) = self.state.pending() else {
            return Ok(BootOutcome::Active {
                slot: self.state.active(),
            });
        };
        let attempts = self
            .state
            .generation(pending)
            .map_or(0, |generation| generation.attempts);
        let ready = self.candidate_ready(pending).is_ok();
        if attempts >= MAX_BOOT_ATTEMPTS || !ready {
            self.reject_candidate(pending, RollbackReason::BootCount, now)?;
            return Ok(BootOutcome::CandidateRejected {
                slot: pending,
                attempts,
            });
        }

        let before = self.state.clone();
        let next = attempts.saturating_add(1);
        if let Some(generation) = self.state.generation_mut(pending) {
            generation.attempts = next;
        }
        if let Err(error) = self.store.save(&self.state) {
            self.state = before;
            return Err(error);
        }
        Ok(BootOutcome::Booting {
            slot: pending,
            attempt: next,
        })
    }

    /// Commit the staged candidate after the caller's health checks pass.
    ///
    /// The previous active generation stays committed and becomes the rollback
    /// target. The switch record names the slots, the reason and the user-data
    /// result.
    ///
    /// # Errors
    /// [`UpdateError::NoPendingCandidate`], [`UpdateError::CandidateIncomplete`],
    /// or an I/O failure. A failed commit leaves the candidate staged.
    pub fn commit(&mut self, now: Timestamp) -> Result<Event> {
        let pending = self
            .state
            .pending()
            .ok_or(UpdateError::NoPendingCandidate)?;
        self.candidate_ready(pending)?;
        let version = self
            .state
            .generation(pending)
            .ok_or(UpdateError::NoPendingCandidate)?
            .version
            .clone();
        let old = self.state.active();
        let event = Event {
            at: now,
            kind: EventKind::Committed,
            from: old,
            to: pending,
            reason: RollbackReason::HealthCheck,
            user_data: UserDataState::Untouched,
        };

        let before = self.state.clone();
        self.state.set_generation(
            pending,
            Some(Generation {
                version,
                state: GenerationState::Committed,
                attempts: 0,
            }),
        );
        self.state.set_pending(None);
        self.state.set_active(pending);
        self.state.push_event(event.clone());
        if let Err(error) = self.store.save(&self.state) {
            self.state = before;
            return Err(error);
        }
        Ok(event)
    }

    /// Discard the staged candidate without changing the active generation.
    ///
    /// This is the operator's cancel path. The candidate is marked bad, so it
    /// cannot later be mistaken for a rollback target; its directory is left in
    /// place for diagnosis.
    ///
    /// # Errors
    /// [`UpdateError::NoPendingCandidate`] or an I/O failure.
    pub fn discard_pending(&mut self, now: Timestamp) -> Result<Event> {
        self.reject_pending(RollbackReason::UserCancelled, now)
    }

    /// Reject the staged candidate with a named reason.
    ///
    /// The boot service uses this when the running system recorded a failed
    /// health attempt: the active pointer never moves, the candidate is marked
    /// bad and is no longer advertised as pending.
    ///
    /// # Errors
    /// [`UpdateError::NoPendingCandidate`] or an I/O failure.
    pub fn reject_pending(&mut self, reason: RollbackReason, now: Timestamp) -> Result<Event> {
        let pending = self
            .state
            .pending()
            .ok_or(UpdateError::NoPendingCandidate)?;
        self.reject_candidate(pending, reason, now)
    }

    /// Roll the active pointer back to the other committed generation.
    ///
    /// The generation being left is marked bad. That is deliberate: a rollback
    /// is an assertion that the active generation is not trusted, and a tree
    /// that is still advertised as committed would be selected again by the
    /// next rollback. A staged candidate must be committed or discarded first.
    ///
    /// # Errors
    /// [`UpdateError::PendingCandidate`], [`UpdateError::NoRollbackTarget`],
    /// [`UpdateError::CandidateIncomplete`] for a missing target marker, or an
    /// I/O failure.
    pub fn rollback(&mut self, reason: RollbackReason, now: Timestamp) -> Result<Event> {
        if let Some(pending) = self.state.pending() {
            return Err(UpdateError::PendingCandidate(pending));
        }
        let target = self.state.active().other();
        self.state
            .generation(target)
            .filter(|generation| generation.state == GenerationState::Committed)
            .ok_or(UpdateError::NoRollbackTarget)?;
        self.candidate_ready(target)?;

        let old = self.state.active();
        let event = Event {
            at: now,
            kind: EventKind::RolledBack,
            from: old,
            to: target,
            reason,
            user_data: UserDataState::Untouched,
        };
        let before = self.state.clone();
        self.state.set_generation(
            old,
            Some(Generation {
                version: self
                    .state
                    .generation(old)
                    .ok_or_else(|| {
                        UpdateError::InvalidState("active slot has no generation".to_owned())
                    })?
                    .version
                    .clone(),
                state: GenerationState::Bad,
                attempts: 0,
            }),
        );
        self.state.set_active(target);
        self.state.push_event(event.clone());
        if let Err(error) = self.store.save(&self.state) {
            self.state = before;
            return Err(error);
        }
        Ok(event)
    }

    /// Verify that a slot's marker names the version the state expects.
    fn candidate_ready(&self, slot: Slot) -> Result<()> {
        let expected =
            self.state
                .generation(slot)
                .ok_or_else(|| UpdateError::CandidateIncomplete {
                    slot,
                    reason: "the state has no record for this slot".to_owned(),
                })?;
        let actual = self.store.read_marker(slot)?;
        if actual != expected.version {
            return Err(UpdateError::CandidateIncomplete {
                slot,
                reason: format!(
                    "the marker names {actual}, but the state expects {}",
                    expected.version
                ),
            });
        }
        Ok(())
    }

    fn reject_candidate(
        &mut self,
        slot: Slot,
        reason: RollbackReason,
        now: Timestamp,
    ) -> Result<Event> {
        let attempts = self
            .state
            .generation(slot)
            .map_or(0, |generation| generation.attempts);
        let version = self
            .state
            .generation(slot)
            .ok_or(UpdateError::NoPendingCandidate)?
            .version
            .clone();
        let active = self.state.active();
        let event = Event {
            at: now,
            kind: EventKind::Rejected,
            from: active,
            to: slot,
            reason,
            user_data: UserDataState::Untouched,
        };
        let before = self.state.clone();
        self.state.set_generation(
            slot,
            Some(Generation {
                version,
                state: GenerationState::Bad,
                attempts,
            }),
        );
        self.state.set_pending(None);
        self.state.push_event(event.clone());
        if let Err(error) = self.store.save(&self.state) {
            self.state = before;
            return Err(error);
        }
        Ok(event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CopyStats {
    entries: u64,
    bytes: u64,
}

fn copy_tree(source: &Path, target: &Path, limits: Limits) -> Result<CopyStats> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| io_error("inspect staging source", source, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(UpdateError::SourceNotDirectory(source.to_path_buf()));
    }
    fs::create_dir(target).map_err(|error| io_error("create staging directory", target, error))?;
    set_dir_mode(target, 0o755)
        .map_err(|error| io_error("set staging directory mode", target, error))?;

    let mut stats = CopyStats {
        entries: 0,
        bytes: 0,
    };
    let mut stack: Vec<(PathBuf, PathBuf, PathBuf)> =
        vec![(source.to_path_buf(), target.to_path_buf(), PathBuf::new())];
    let mut directories: Vec<PathBuf> = vec![target.to_path_buf()];

    while let Some((source_dir, target_dir, relative_dir)) = stack.pop() {
        let mut entries: Vec<fs::DirEntry> = fs::read_dir(&source_dir)
            .map_err(|error| io_error("read staging source directory", &source_dir, error))?
            .collect::<core::result::Result<Vec<_>, io::Error>>()
            .map_err(|error| io_error("read staging source entry", &source_dir, error))?;
        entries.sort_by_key(fs::DirEntry::file_name);

        for entry in entries {
            let name = entry.file_name();
            let relative = relative_dir.join(&name);
            let source_path = entry.path();
            let target_path = target_dir.join(&name);

            stats.entries = stats
                .entries
                .checked_add(1)
                .ok_or(UpdateError::TooManyEntries {
                    limit: limits.max_entries,
                })?;
            if stats.entries > limits.max_entries {
                return Err(UpdateError::TooManyEntries {
                    limit: limits.max_entries,
                });
            }
            if relative.to_string_lossy().len() > limits.max_path_bytes {
                return Err(UpdateError::PathTooLong {
                    limit: limits.max_path_bytes,
                });
            }

            let metadata = fs::symlink_metadata(&source_path)
                .map_err(|error| io_error("inspect staging source entry", &source_path, error))?;
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                return Err(UpdateError::UnsafeSource {
                    path: relative,
                    kind: "symlink",
                });
            }
            if file_type.is_dir() {
                fs::create_dir(&target_path).map_err(|error| {
                    io_error("create staging subdirectory", &target_path, error)
                })?;
                set_dir_mode(&target_path, 0o755).map_err(|error| {
                    io_error("set staging subdirectory mode", &target_path, error)
                })?;
                directories.push(target_path.clone());
                stack.push((source_path, target_path, relative));
                continue;
            }
            if !file_type.is_file() {
                return Err(UpdateError::UnsafeSource {
                    path: relative,
                    kind: "special file",
                });
            }

            stats.bytes =
                stats
                    .bytes
                    .checked_add(metadata.len())
                    .ok_or(UpdateError::TooManyBytes {
                        limit: limits.max_bytes,
                    })?;
            if stats.bytes > limits.max_bytes {
                return Err(UpdateError::TooManyBytes {
                    limit: limits.max_bytes,
                });
            }
            fs::copy(&source_path, &target_path)
                .map_err(|error| io_error("copy staging source file", &target_path, error))?;
            sync_file(&target_path)
                .map_err(|error| io_error("sync staging source file", &target_path, error))?;
        }
    }

    for directory in directories.iter().rev() {
        sync_dir(directory)
            .map_err(|error| io_error("sync staging source directory", directory, error))?;
    }
    Ok(stats)
}

fn set_dir_mode(path: &Path, mode: u32) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}
