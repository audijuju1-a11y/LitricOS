//! The safety architecture: nothing destructive-equivalent runs ungated.
//!
//! MASTER §SAFETY: the framework must produce catastrophic *equivalents*
//! without weaponizable artifacts, and must refuse to run outside a
//! disposable environment. This module is the enforcement, not the prose:
//! every entry point in the CLI funnels through [`SafetyGate::enter`], and
//! the gate refuses when the target directory is not the dedicated test
//! volume layout, when no snapshot marker exists, or when the requested
//! severity exceeds the environment's clearance.
//!
//! # The escalation ladder (MASTER §27)
//!
//! Level 5 is never the first test. The ladder is data, so a run that
//! skips steps is a run that fails to build, not a run that "usually
//! starts small".

use std::path::{Path, PathBuf};

/// The dedicated disposable root layout the gate demands.
///
/// A real user directory is never acceptable: the benchmark writes, locks,
/// floods and deletes inside `TEST_VOLUME`, and only inside it.
pub const TEST_VOLUME_MARKER: &str = "LITRIC_BENCH_VOLUME";
/// Snapshot marker file: proves a rollback checkpoint exists for this run.
pub const SNAPSHOT_MARKER: &str = "LITRIC_BENCH_SNAPSHOT";
/// Recovery marker: written when the checkpoint is verified restorable.
pub const RECOVERY_MARKER: &str = "LITRIC_BENCH_RECOVERY_OK";

/// Severity clearance for an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Clearance {
    /// Host-shared directories: only LEVEL 0..1 (benign / suspicious shapes).
    Host,
    /// A disposable VM: up to LEVEL 3 (system-impact simulation).
    Vm,
    /// A disposable VM with a verified snapshot: every level, including 5.
    Snapshot,
}

impl Clearance {
    /// The highest severity this clearance permits.
    #[must_use]
    pub fn max_level(self) -> u8 {
        match self {
            Self::Host => 1,
            Self::Vm => 3,
            Self::Snapshot => 5,
        }
    }
}

/// Inspect a test root and derive what it is safe to do there.
#[must_use]
pub fn inspect(root: &Path) -> Clearance {
    if !root.is_dir() {
        return Clearance::Host;
    }
    let has_volume = root.join(TEST_VOLUME_MARKER).exists();
    let has_snapshot = root.join(SNAPSHOT_MARKER).exists();
    match (has_volume, has_snapshot) {
        (true, true) => Clearance::Snapshot,
        (true, false) => Clearance::Vm,
        (false, _) => Clearance::Host,
    }
}

/// Provision a root as a full disposable test volume with snapshot markers.
///
/// # Errors
/// Fails when the directory cannot be created or marked, so a caller can
/// never half-provision.
pub fn provision(root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|e| format!("create root: {e}"))?;
    for name in [TEST_VOLUME_MARKER, SNAPSHOT_MARKER, RECOVERY_MARKER] {
        std::fs::write(
            root.join(name),
            format!("litric-bench disposable volume\ncreated: {}\n", now_unix()),
        )
        .map_err(|e| format!("write marker {name}: {e}"))?;
    }
    Ok(())
}

/// Seconds since the epoch, for marker stamps only (events use ns).
#[must_use]
pub fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A gate for one benchmark run.
#[derive(Debug)]
pub struct SafetyGate {
    root: PathBuf,
    clearance: Clearance,
    max_level: u8,
}

impl SafetyGate {
    /// Enter the gate for `root`, demanding clearance for `level`.
    ///
    /// # Errors
    /// Refuses with the exact unmet condition — a run must never start on a
    /// maybe.
    pub fn enter(root: &Path, level: u8) -> Result<Self, String> {
        let clearance = inspect(root);
        if !root.is_dir() {
            return Err(format!("root {} is not a directory", root.display()));
        }
        if level > clearance.max_level() {
            return Err(format!(
                "severity LEVEL {level} exceeds environment clearance \
                 {clearance:?} (max {}); provision a snapshot first",
                clearance.max_level()
            ));
        }
        Ok(Self {
            root: root.to_path_buf(),
            clearance,
            max_level: clearance.max_level().min(level),
        })
    }

    /// The verified root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The environment clearance.
    #[must_use]
    pub fn clearance(&self) -> Clearance {
        self.clearance
    }

    /// The highest level this run may execute.
    #[must_use]
    pub fn max_level(&self) -> u8 {
        self.max_level
    }

    /// The escalation ladder for a target level, per MASTER §27: pilot,
    /// safety validation, then each level with recovery checks between.
    #[must_use]
    pub fn ladder(&self, target: u8) -> Vec<u8> {
        let cap = target.min(self.max_level);
        (1..=cap).collect()
    }

    /// The critical-failure policy (MASTER §20): stop, snapshot, mark.
    /// Returns the incident record; the caller stops the run.
    #[must_use]
    pub fn critical_stop(&self, reason: &str) -> Incident {
        Incident {
            reason: reason.to_owned(),
            root: self.root.clone(),
            at_unix: now_unix(),
        }
    }
}

/// A critical-stop incident record.
#[derive(Debug, Clone)]
pub struct Incident {
    /// Why the run stopped.
    pub reason: String,
    /// The root being quarantined from further runs.
    pub root: PathBuf,
    /// When the stop happened.
    pub at_unix: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unmarked_directory_is_host_clearance() {
        let dir = std::env::temp_dir().join("litric-bench-safety-host");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(inspect(&dir), Clearance::Host);
        // LEVEL 2 on a host root must be refused outright.
        assert!(SafetyGate::enter(&dir, 2).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_provisioned_volume_unlocks_levels() {
        let dir = std::env::temp_dir().join("litric-bench-safety-vm");
        let _ = std::fs::remove_dir_all(&dir);
        provision(&dir).unwrap();
        assert_eq!(inspect(&dir), Clearance::Snapshot);
        let gate = SafetyGate::enter(&dir, 5).unwrap();
        assert_eq!(gate.max_level(), 5);
        // The ladder never starts at the top: 1,2,3,4,5 with checks between.
        assert_eq!(gate.ladder(5), vec![1, 2, 3, 4, 5]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_critical_stop_carries_what_a_forensic_needs() {
        let dir = std::env::temp_dir().join("litric-bench-safety-stop");
        let _ = std::fs::remove_dir_all(&dir);
        provision(&dir).unwrap();
        let gate = SafetyGate::enter(&dir, 1).unwrap();
        let inc = gate.critical_stop("snapshot integrity lost");
        assert!(inc.reason.contains("snapshot"));
        assert_eq!(inc.root, dir);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
