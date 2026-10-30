//! The two-slot generation record and its atomic store.
//!
//! The record is deliberately line-oriented and small enough to read in one
//! page:
//!
//! ```text
//! litric-generations\t1
//! active\tB
//! pending\tA
//! slot\tA\t1.2.0\tstaged\t2
//! slot\tB\t1.1.0\tcommitted\t0
//! event\t1800000000\tcommit\tB\tA\thealth-check\tuntouched
//! ```
//!
//! The parser refuses anything it does not understand. That is the opposite of
//! the lockout meter's fail-open rule, and it is intentional: a damaged lockout
//! file must not deny every login, while a damaged generation record must not
//! advertise a slot as bootable when its state is unknown. The bootloader and
//! the recovery environment can still choose a known slot; this engine refuses
//! to guess which one that is.

use crate::fsutil::{ensure_private_dir, remove_any, sync_dir, sync_file};
use crate::{io_error, Result, Timestamp, UpdateError};
use litric_core::Version;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The first line of every state record.
pub const STATE_PROTOCOL: &str = "litric-generations";
/// The schema version written below the protocol line.
pub const STATE_VERSION: u8 = 1;
/// The largest state file accepted.
///
/// The record holds two slots and eight events; 16 KiB is far more than that
/// needs and keeps a damaged or hostile file from becoming an unbounded read.
pub const MAX_STATE_BYTES: u64 = 16 * 1024;
/// The longest line accepted in the state file.
pub const MAX_LINE_BYTES: usize = 512;
/// The longest switch history retained.
pub const MAX_EVENTS: usize = 8;
/// Boots a candidate may fail before it is discarded.
///
/// Three failed boots leave the previous active generation untouched and mark
/// the candidate bad on the fourth boot request. Two would be impatient with a
/// slow but healthy machine; ten would turn a boot loop into a support call.
pub const MAX_BOOT_ATTEMPTS: u8 = 3;

/// One of the two system slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    /// The first slot.
    A,
    /// The second slot.
    B,
}

impl Slot {
    /// Both slots, in stable order.
    pub const ALL: [Self; 2] = [Self::A, Self::B];

    /// The other slot.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }

    /// The single-letter name used on the wire and on disk.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
        }
    }

    /// Parse `A` or `B`.
    ///
    /// # Errors
    /// [`UpdateError::InvalidSlot`] for every other spelling.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            _ => Err(UpdateError::InvalidSlot(raw.to_owned())),
        }
    }
}

/// What is known about one slot's generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generation {
    /// The version this slot holds.
    pub version: Version,
    /// Whether it is active material, a candidate, or a rejected tree.
    pub state: GenerationState,
    /// Boot attempts made without a commit. Meaningful for `Staged`; kept for
    /// diagnosis on a rejected slot.
    pub attempts: u8,
}

/// The lifecycle state of a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationState {
    /// A bootable generation that may be active or may be the rollback target.
    Committed,
    /// A complete candidate waiting for a health-check commit.
    Staged,
    /// A candidate that failed its boot budget or was explicitly cancelled.
    /// It is not a rollback target.
    Bad,
}

impl GenerationState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Staged => "staged",
            Self::Bad => "bad",
        }
    }

    fn parse(raw: &str) -> core::result::Result<Self, String> {
        match raw {
            "committed" => Ok(Self::Committed),
            "staged" => Ok(Self::Staged),
            "bad" => Ok(Self::Bad),
            _ => Err(format!("unknown generation state {raw:?}")),
        }
    }
}

/// Why a generation switch happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RollbackReason {
    /// The candidate completed the caller's health checks.
    HealthCheck,
    /// The candidate used its boot budget without a health commit.
    BootCount,
    /// A committed generation failed an integrity or health check.
    Corruption,
    /// An operator asked for the previous generation.
    Manual,
    /// An operator cancelled the candidate before it was activated.
    UserCancelled,
}

impl RollbackReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::HealthCheck => "health-check",
            Self::BootCount => "boot-count",
            Self::Corruption => "corruption",
            Self::Manual => "manual",
            Self::UserCancelled => "user-cancelled",
        }
    }

    fn parse(raw: &str) -> core::result::Result<Self, String> {
        match raw {
            "health-check" => Ok(Self::HealthCheck),
            "boot-count" => Ok(Self::BootCount),
            "corruption" => Ok(Self::Corruption),
            "manual" => Ok(Self::Manual),
            "user-cancelled" => Ok(Self::UserCancelled),
            _ => Err(format!("unknown switch reason {raw:?}")),
        }
    }
}

/// What happened to user data during a switch.
///
/// The engine only writes `<root>/system`, so this is always `Untouched` today.
/// The field exists so a future recovery service that also verifies the user
/// data domain has a place to record a stronger result without changing the
/// state format's meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserDataState {
    /// The switch did not write the user data domain.
    Untouched,
}

impl UserDataState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Untouched => "untouched",
        }
    }

    fn parse(raw: &str) -> core::result::Result<Self, String> {
        match raw {
            "untouched" => Ok(Self::Untouched),
            _ => Err(format!("unknown user-data result {raw:?}")),
        }
    }
}

/// One committed, rejected or rolled-back switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// When the caller says the switch happened.
    pub at: Timestamp,
    /// What happened.
    pub kind: EventKind,
    /// The slot that was active before the operation.
    pub from: Slot,
    /// The slot the operation concerned.
    pub to: Slot,
    /// Why it happened.
    pub reason: RollbackReason,
    /// What the operation did to user data.
    pub user_data: UserDataState,
}

/// The kind of switch record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// A staged candidate became active after health checks.
    Committed,
    /// A candidate was discarded before activation.
    Rejected,
    /// A committed generation became active again after another was rejected.
    RolledBack,
}

impl EventKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "commit",
            Self::Rejected => "reject",
            Self::RolledBack => "rollback",
        }
    }

    fn parse(raw: &str) -> core::result::Result<Self, String> {
        match raw {
            "commit" => Ok(Self::Committed),
            "reject" => Ok(Self::Rejected),
            "rollback" => Ok(Self::RolledBack),
            _ => Err(format!("unknown switch kind {raw:?}")),
        }
    }
}

/// The complete persisted generation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateState {
    pub(crate) active: Slot,
    pub(crate) pending: Option<Slot>,
    pub(crate) slots: [Option<Generation>; 2],
    pub(crate) events: Vec<Event>,
}

impl UpdateState {
    /// A fresh record with one committed active generation in slot A.
    pub(crate) fn initial(version: Version) -> Self {
        Self {
            active: Slot::A,
            pending: None,
            slots: [
                Some(Generation {
                    version,
                    state: GenerationState::Committed,
                    attempts: 0,
                }),
                None,
            ],
            events: Vec::new(),
        }
    }

    /// The slot the bootloader should start.
    #[must_use]
    pub const fn active(&self) -> Slot {
        self.active
    }

    /// The candidate slot, when one is staged.
    #[must_use]
    pub const fn pending(&self) -> Option<Slot> {
        self.pending
    }

    /// The record for one slot, if the slot holds anything.
    #[must_use]
    pub fn generation(&self, slot: Slot) -> Option<&Generation> {
        match slot {
            Slot::A => self.slots.first().and_then(Option::as_ref),
            Slot::B => self.slots.get(1).and_then(Option::as_ref),
        }
    }

    /// The bounded switch history, oldest first.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Whether the other slot has a committed generation to return to.
    #[must_use]
    pub fn can_rollback(&self) -> bool {
        self.generation(self.active.other())
            .is_some_and(|generation| generation.state == GenerationState::Committed)
    }

    pub(crate) fn generation_mut(&mut self, slot: Slot) -> Option<&mut Generation> {
        match slot {
            Slot::A => self.slots.first_mut().and_then(Option::as_mut),
            Slot::B => self.slots.get_mut(1).and_then(Option::as_mut),
        }
    }

    pub(crate) fn set_generation(&mut self, slot: Slot, generation: Option<Generation>) {
        match slot {
            Slot::A => {
                if let Some(slot) = self.slots.first_mut() {
                    *slot = generation;
                }
            }
            Slot::B => {
                if let Some(slot) = self.slots.get_mut(1) {
                    *slot = generation;
                }
            }
        }
    }

    pub(crate) fn set_active(&mut self, slot: Slot) {
        self.active = slot;
    }

    pub(crate) fn set_pending(&mut self, slot: Option<Slot>) {
        self.pending = slot;
    }

    pub(crate) fn push_event(&mut self, event: Event) {
        if self.events.len() >= MAX_EVENTS {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    fn validate(&self) -> core::result::Result<(), String> {
        let active = self
            .generation(self.active)
            .ok_or_else(|| format!("active slot {} has no generation", self.active.as_str()))?;
        if active.state != GenerationState::Committed {
            return Err(format!(
                "active slot {} is {}, not committed",
                self.active.as_str(),
                active.state.as_str()
            ));
        }
        if let Some(pending) = self.pending {
            if pending == self.active {
                return Err("the pending slot cannot be the active slot".to_owned());
            }
            let candidate = self
                .generation(pending)
                .ok_or_else(|| format!("pending slot {} has no generation", pending.as_str()))?;
            if candidate.state != GenerationState::Staged {
                return Err(format!(
                    "pending slot {} is {}, not staged",
                    pending.as_str(),
                    candidate.state.as_str()
                ));
            }
        }
        for slot in Slot::ALL {
            let Some(generation) = self.generation(slot) else {
                continue;
            };
            if generation.attempts > MAX_BOOT_ATTEMPTS {
                return Err(format!(
                    "slot {} has {} boot attempts, past the {MAX_BOOT_ATTEMPTS} bound",
                    slot.as_str(),
                    generation.attempts
                ));
            }
            if generation.state == GenerationState::Staged && self.pending != Some(slot) {
                return Err(format!(
                    "slot {} is staged but is not the pending slot",
                    slot.as_str()
                ));
            }
        }
        if self.events.len() > MAX_EVENTS {
            return Err(format!(
                "{} switch events, past the {MAX_EVENTS} bound",
                self.events.len()
            ));
        }
        Ok(())
    }

    pub(crate) fn encode(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str(STATE_PROTOCOL);
        out.push('\t');
        out.push_str(&STATE_VERSION.to_string());
        out.push('\n');
        out.push_str("active\t");
        out.push_str(self.active.as_str());
        out.push('\n');
        if let Some(pending) = self.pending {
            out.push_str("pending\t");
            out.push_str(pending.as_str());
            out.push('\n');
        }
        for slot in Slot::ALL {
            let Some(generation) = self.generation(slot) else {
                continue;
            };
            out.push_str("slot\t");
            out.push_str(slot.as_str());
            out.push('\t');
            out.push_str(&generation.version.to_string());
            out.push('\t');
            out.push_str(generation.state.as_str());
            out.push('\t');
            out.push_str(&generation.attempts.to_string());
            out.push('\n');
        }
        for event in &self.events {
            out.push_str("event\t");
            out.push_str(&event.at.to_string());
            out.push('\t');
            out.push_str(event.kind.as_str());
            out.push('\t');
            out.push_str(event.from.as_str());
            out.push('\t');
            out.push_str(event.to.as_str());
            out.push('\t');
            out.push_str(event.reason.as_str());
            out.push('\t');
            out.push_str(event.user_data.as_str());
            out.push('\n');
        }
        out
    }

    pub(crate) fn decode(text: &str) -> core::result::Result<Self, String> {
        let mut lines = text.lines();
        let header = lines
            .next()
            .ok_or_else(|| "empty generation record".to_owned())?;
        let expected = format!("{STATE_PROTOCOL}\t{STATE_VERSION}");
        if header != expected {
            return Err(format!("unknown generation record header {header:?}"));
        }

        let mut active: Option<Slot> = None;
        let mut pending: Option<Slot> = None;
        let mut slot_a: Option<Generation> = None;
        let mut slot_b: Option<Generation> = None;
        let mut events: Vec<Event> = Vec::new();

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
                ["active", raw] => {
                    if active.is_some() {
                        return Err(format!("line {number}: duplicate active record"));
                    }
                    active =
                        Some(Slot::parse(raw).map_err(|error| format!("line {number}: {error}"))?);
                }
                ["pending", raw] => {
                    if pending.is_some() {
                        return Err(format!("line {number}: duplicate pending record"));
                    }
                    pending =
                        Some(Slot::parse(raw).map_err(|error| format!("line {number}: {error}"))?);
                }
                ["slot", raw_slot, raw_version, raw_state, raw_attempts] => {
                    let slot =
                        Slot::parse(raw_slot).map_err(|error| format!("line {number}: {error}"))?;
                    let version = Version::parse(raw_version)
                        .map_err(|error| format!("line {number}: bad version: {error}"))?;
                    let state = GenerationState::parse(raw_state)
                        .map_err(|error| format!("line {number}: {error}"))?;
                    let attempts = raw_attempts
                        .parse::<u8>()
                        .map_err(|_| format!("line {number}: bad attempt count"))?;
                    let generation = Generation {
                        version,
                        state,
                        attempts,
                    };
                    let target = match slot {
                        Slot::A => &mut slot_a,
                        Slot::B => &mut slot_b,
                    };
                    if target.is_some() {
                        return Err(format!(
                            "line {number}: duplicate slot {} record",
                            slot.as_str()
                        ));
                    }
                    *target = Some(generation);
                }
                ["event", raw_at, raw_kind, raw_from, raw_to, raw_reason, raw_user] => {
                    if events.len() >= MAX_EVENTS {
                        return Err(format!(
                            "line {number}: more than {MAX_EVENTS} switch events"
                        ));
                    }
                    let at = raw_at
                        .parse::<Timestamp>()
                        .map_err(|_| format!("line {number}: bad switch timestamp"))?;
                    let kind = EventKind::parse(raw_kind)
                        .map_err(|error| format!("line {number}: {error}"))?;
                    let from =
                        Slot::parse(raw_from).map_err(|error| format!("line {number}: {error}"))?;
                    let to =
                        Slot::parse(raw_to).map_err(|error| format!("line {number}: {error}"))?;
                    let reason = RollbackReason::parse(raw_reason)
                        .map_err(|error| format!("line {number}: {error}"))?;
                    let user_data = UserDataState::parse(raw_user)
                        .map_err(|error| format!("line {number}: {error}"))?;
                    events.push(Event {
                        at,
                        kind,
                        from,
                        to,
                        reason,
                        user_data,
                    });
                }
                _ => return Err(format!("line {number}: unrecognised record")),
            }
        }

        let active = active.ok_or_else(|| "generation record has no active slot".to_owned())?;
        let state = Self {
            active,
            pending,
            slots: [slot_a, slot_b],
            events,
        };
        state.validate()?;
        Ok(state)
    }
}

/// The on-disk store for one root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Store {
    root: PathBuf,
}

impl Store {
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) fn state_path(&self) -> PathBuf {
        self.root.join("system").join("state")
    }

    pub(crate) fn slots_dir(&self) -> PathBuf {
        self.root.join("system").join("slots")
    }

    pub(crate) fn slot_path(&self, slot: Slot) -> PathBuf {
        self.slots_dir().join(slot.as_str())
    }

    pub(crate) fn stage_temp_path(&self, slot: Slot) -> PathBuf {
        self.slots_dir()
            .join(format!(".{}.staging-{}", slot.as_str(), std::process::id()))
    }

    pub(crate) fn load(&self) -> Result<UpdateState> {
        let path = self.state_path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(UpdateError::MissingState(path));
            }
            Err(error) => return Err(io_error("read generation state", &path, error)),
        };
        if metadata.file_type().is_symlink() {
            return Err(UpdateError::InvalidState(format!(
                "{} is a symlink",
                path.display()
            )));
        }
        if !metadata.is_file() {
            return Err(UpdateError::InvalidState(format!(
                "{} is not a regular file",
                path.display()
            )));
        }
        if metadata.len() > MAX_STATE_BYTES {
            return Err(UpdateError::InvalidState(format!(
                "{} is {} bytes, past the {MAX_STATE_BYTES}-byte ceiling",
                path.display(),
                metadata.len()
            )));
        }
        let mut file = fs::File::open(&path)
            .map_err(|error| io_error("open generation state", &path, error))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| io_error("read generation state", &path, error))?;
        // The metadata check above is the fast refusal; this one closes the
        // race where the file grows between `metadata` and the read.
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(UpdateError::InvalidState(format!(
                "{} grew past the {MAX_STATE_BYTES}-byte ceiling while being read",
                path.display()
            )));
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| UpdateError::InvalidState(format!("{} is not UTF-8", path.display())))?;
        UpdateState::decode(text).map_err(UpdateError::InvalidState)
    }

    pub(crate) fn save(&self, state: &UpdateState) -> Result<()> {
        state.validate().map_err(UpdateError::InvalidState)?;
        let path = self.state_path();
        let parent = path
            .parent()
            .ok_or_else(|| UpdateError::InvalidState("state path has no parent".to_owned()))?;
        ensure_private_dir(parent)
            .map_err(|error| io_error("create generation directory", parent, error))?;

        let temp = parent.join(format!(".state.tmp-{}", std::process::id()));
        remove_any(&temp)
            .map_err(|error| io_error("remove stale state temporary", &temp, error))?;

        let result = (|| -> Result<()> {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temp)
                .map_err(|error| io_error("create state temporary", &temp, error))?;
            file.write_all(state.encode().as_bytes())
                .map_err(|error| io_error("write generation state", &temp, error))?;
            file.sync_all()
                .map_err(|error| io_error("sync generation state", &temp, error))?;
            fs::rename(&temp, &path)
                .map_err(|error| io_error("commit generation state", &path, error))?;
            sync_dir(parent)
                .map_err(|error| io_error("sync generation directory", parent, error))?;
            Ok(())
        })();

        if result.is_err() {
            let _ = remove_any(&temp);
        }
        result
    }

    pub(crate) fn write_marker(&self, slot_path: &Path, version: &Version) -> Result<()> {
        let path = slot_path.join("generation");
        let body = format!("{STATE_PROTOCOL}-generation\t{STATE_VERSION}\nversion\t{version}\n");
        fs::write(&path, body)
            .map_err(|error| io_error("write generation marker", &path, error))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644))
                .map_err(|error| io_error("set generation marker mode", &path, error))?;
        }
        sync_file(&path).map_err(|error| io_error("sync generation marker", &path, error))
    }

    pub(crate) fn read_marker(&self, slot: Slot) -> Result<Version> {
        let slot_path = self.slot_path(slot);
        let metadata =
            fs::symlink_metadata(&slot_path).map_err(|error| UpdateError::CandidateIncomplete {
                slot,
                reason: format!("slot directory is missing: {error}"),
            })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(UpdateError::CandidateIncomplete {
                slot,
                reason: "slot directory is not a real directory".to_owned(),
            });
        }
        let path = slot_path.join("generation");
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| UpdateError::CandidateIncomplete {
                slot,
                reason: format!("generation marker is missing: {error}"),
            })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker is not a regular file".to_owned(),
            });
        }
        if metadata.len() > 256 {
            return Err(UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker is oversized".to_owned(),
            });
        }
        let text = fs::read_to_string(&path).map_err(|error| UpdateError::CandidateIncomplete {
            slot,
            reason: format!("cannot read generation marker: {error}"),
        })?;
        let mut lines = text.lines();
        let header = lines
            .next()
            .ok_or_else(|| UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker is empty".to_owned(),
            })?;
        let expected = format!("{STATE_PROTOCOL}-generation\t{STATE_VERSION}");
        if header != expected {
            return Err(UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker has an unknown header".to_owned(),
            });
        }
        let version_line = lines
            .next()
            .ok_or_else(|| UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker has no version".to_owned(),
            })?;
        let raw = version_line.strip_prefix("version\t").ok_or_else(|| {
            UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker has no version field".to_owned(),
            }
        })?;
        let version = Version::parse(raw).map_err(|error| UpdateError::CandidateIncomplete {
            slot,
            reason: format!("generation marker version is invalid: {error}"),
        })?;
        if lines.next().is_some() {
            return Err(UpdateError::CandidateIncomplete {
                slot,
                reason: "generation marker has trailing records".to_owned(),
            });
        }
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> UpdateState {
        let mut state = UpdateState::initial(Version::parse("1.0.0").expect("version"));
        let candidate = Generation {
            version: Version::parse("1.1.0").expect("version"),
            state: GenerationState::Staged,
            attempts: 2,
        };
        state.set_generation(Slot::B, Some(candidate));
        state.set_pending(Some(Slot::B));
        state.push_event(Event {
            at: 1_800_000_000,
            kind: EventKind::Committed,
            from: Slot::A,
            to: Slot::B,
            reason: RollbackReason::HealthCheck,
            user_data: UserDataState::Untouched,
        });
        state
    }

    #[test]
    fn the_record_round_trips_through_the_format() {
        let original = state();
        let encoded = original.encode();
        let decoded = UpdateState::decode(&encoded).expect("decodes");
        assert_eq!(decoded, original);
        assert!(encoded.contains("slot\tB\t1.1.0\tstaged\t2"));
    }

    #[test]
    fn a_damaged_record_is_refused_not_guessed() {
        let encoded = state().encode();
        for broken in [
            String::new(),
            "litric-generations\t2\n".to_owned(),
            encoded.replace("active\tA", "active\tC"),
            encoded.replace("active\tA", "active\tB\nactive\tB"),
            format!("{encoded}pending\tA\n"),
            encoded.replace("staged", "committed"),
            encoded.replace("staged\t2", "staged\t9"),
            encoded.replace("event\t", "event\tx"),
            format!("{encoded}\n"),
        ] {
            assert!(
                UpdateState::decode(&broken).is_err(),
                "{broken:?} was accepted"
            );
        }
    }

    #[test]
    fn the_switch_history_is_bounded() {
        let mut state = state();
        for at in 0..(MAX_EVENTS as u64 + 4) {
            state.push_event(Event {
                at,
                kind: EventKind::Rejected,
                from: Slot::A,
                to: Slot::B,
                reason: RollbackReason::Manual,
                user_data: UserDataState::Untouched,
            });
        }
        assert_eq!(state.events().len(), MAX_EVENTS);
        assert_eq!(state.events().first().map(|event| event.at), Some(4));
        let decoded = UpdateState::decode(&state.encode()).expect("decodes");
        assert_eq!(decoded.events().len(), MAX_EVENTS);
    }

    #[test]
    fn an_oversized_line_is_refused_before_parsing() {
        let line = format!("event\t{}\n", "x".repeat(MAX_LINE_BYTES));
        let text = format!(
            "{STATE_PROTOCOL}\t{STATE_VERSION}\nactive\tA\nslot\tA\t1.0.0\tcommitted\t0\n{line}"
        );
        assert!(UpdateState::decode(&text).is_err());
    }

    #[test]
    fn rollback_requires_a_second_committed_generation() {
        let only = UpdateState::initial(Version::parse("1.0.0").expect("version"));
        assert!(!only.can_rollback());
        let both = state();
        assert!(!both.can_rollback(), "a staged candidate is not a target");
    }
}
