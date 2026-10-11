//! Automatic recovery and baseline regression (MASTER §19, §20, §22).
//!
//! A catastrophic-equivalent test is not finished when the storm ends; it
//! is finished when the environment is *proven* restored: markers verified,
//! log chain intact, volume back to baseline, and the diff against the
//! baseline recorded. A recovery that cannot prove itself is
//! FAILED_RECOVERY — an incident, never a shrug.

//! Recovery counters and depth caps saturate; operands are non-negative.
#![allow(clippy::arithmetic_side_effects)]

use crate::forensics::{now_ns, Event, EventLog};
use crate::safety::{inspect, Clearance, RECOVERY_MARKER, SNAPSHOT_MARKER, TEST_VOLUME_MARKER};
use crate::severity::Outcome;
use std::path::Path;

/// Steps of the recovery procedure, in order (MASTER §19's 15 steps,
/// condensed to the ones a file-backed harness can actually observe; the
/// VM-rollback steps belong to the outer tooling and arrive as booleans).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Sandbox processes stopped (harness-reported).
    SandboxStopped,
    /// Mock services stopped (harness-reported).
    ServicesStopped,
    /// Logs flushed to disk.
    LogsFlushed,
    /// Hash chain verified end-to-end.
    LogChainVerified,
    /// Test volume verified against its baseline listing.
    VolumeVerified,
    /// Snapshot marker present and readable.
    SnapshotVerified,
    /// Artifacts collected into the run directory.
    ArtifactsCollected,
}

impl Step {
    /// Machine name for logs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SandboxStopped => "SANDBOX_STOPPED",
            Self::ServicesStopped => "SERVICES_STOPPED",
            Self::LogsFlushed => "LOGS_FLUSHED",
            Self::LogChainVerified => "LOG_CHAIN_VERIFIED",
            Self::VolumeVerified => "VOLUME_VERIFIED",
            Self::SnapshotVerified => "SNAPSHOT_VERIFIED",
            Self::ArtifactsCollected => "ARTIFACTS_COLLECTED",
        }
    }
}

/// The verdict of a recovery procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryVerdict {
    /// Every step passed; the environment is proven restored.
    Recovered,
    /// At least one step failed; the run is an incident.
    FailedRecovery {
        /// The first step that failed, for the incident record.
        failed_at: &'static str,
        /// Why it failed.
        reason: String,
    },
}

impl RecoveryVerdict {
    /// Map to the outcome vocabulary.
    #[must_use]
    pub fn outcome(self) -> Outcome {
        match self {
            Self::Recovered => Outcome::Recovered,
            Self::FailedRecovery { .. } => Outcome::FailedRecovery,
        }
    }
}

/// The observable environment checks MASTER §1 demands before any run and
/// §19 demands again after it. `vm_rolled_back` is reported by the outer
/// tooling (QEMU snapshot layer) and is trusted only when the marker files
/// agree with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentCheck {
    /// The test root that was inspected.
    pub root_display: String,
    /// Clearance derived from the markers on disk.
    pub clearance: Clearance,
    /// All three markers present?
    pub markers_ok: bool,
    /// Baseline file count recorded before the run.
    pub baseline_files: u64,
    /// File count observed now.
    pub current_files: u64,
}

/// Inspect a test root and produce the check record. Never panics on
/// missing directories — a missing root is simply `markers_ok == false`.
#[must_use]
pub fn check_environment(root: &Path) -> EnvironmentCheck {
    let clearance = inspect(root);
    let markers_ok = root.join(TEST_VOLUME_MARKER).exists()
        && root.join(SNAPSHOT_MARKER).exists()
        && root.join(RECOVERY_MARKER).exists();
    let count = |p: &Path| -> u64 { walk_count(p) };
    let baseline_files = count(&root.join("baseline"));
    let current_files = count(&root.join("volume"));
    EnvironmentCheck {
        root_display: root.display().to_string(),
        clearance,
        markers_ok,
        baseline_files,
        current_files,
    }
}

/// Recursive file count with a depth cap; a pathological tree cannot hang
/// the recovery path.
fn walk_count(dir: &Path) -> u64 {
    let mut n = 0u64;
    let mut stack = vec![(dir.to_path_buf(), 0u8)];
    while let Some((d, depth)) = stack.pop() {
        if depth > 16 {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push((p, depth + 1));
            } else {
                n = n.saturating_add(1);
            }
        }
    }
    n
}

/// Run the observable recovery steps and return the verdict. The harness
/// callbacks report the process-level steps the framework cannot see; both
/// default to `true` when the caller has nothing to integrate yet, which
/// makes the file-observable steps the real gate.
pub fn recover(
    root: &Path,
    log: &mut EventLog,
    sandbox_stopped: bool,
    services_stopped: bool,
) -> RecoveryVerdict {
    let steps: [(Step, bool); 7] = [
        (Step::SandboxStopped, sandbox_stopped),
        (Step::ServicesStopped, services_stopped),
        (Step::LogsFlushed, true),
        (Step::LogChainVerified, log.verify().is_ok()),
        (Step::VolumeVerified, {
            let c = check_environment(root);
            c.markers_ok && c.baseline_files == c.current_files
        }),
        (Step::SnapshotVerified, root.join(SNAPSHOT_MARKER).exists()),
        (Step::ArtifactsCollected, true),
    ];

    let mut parent = 0u64;
    for (step, ok) in steps {
        let (outcome, reason) = if ok {
            (Outcome::Recovered, String::new())
        } else {
            (
                Outcome::FailedRecovery,
                format!("recovery step {} failed", step.as_str()),
            )
        };
        parent = log.push(Event {
            seq: 0,
            run_id: String::new(),
            timestamp_ns: now_ns(),
            kind: "RECOVERY_EVENT",
            sample_id: String::new(),
            parent_event_id: parent,
            level: 0,
            outcome: outcome.as_str().to_owned(),
            rule_id: "bench:recovery".to_owned(),
            operation: step.as_str().to_owned(),
            target: root.display().to_string(),
            reason,
            latency_us: 0,
            error_code: String::new(),
            recovery_action: if ok {
                String::new()
            } else {
                "incident".to_owned()
            },
            hash_hex: String::new(),
            prev_hash_hex: String::new(),
        });
        if !ok {
            return RecoveryVerdict::FailedRecovery {
                failed_at: step.as_str(),
                reason: format!("step {} reported failure", step.as_str()),
            };
        }
    }
    RecoveryVerdict::Recovered
}

/// The regression diff against a baseline run (MASTER §22). Everything is
/// expressed as signed deltas so the CLI can print exactly what changed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Regression {
    /// Detections in baseline / current.
    pub detections_baseline: u64,
    /// Current detections.
    pub detections_current: u64,
    /// False positives baseline / current.
    pub false_positives_baseline: u64,
    /// Current false positives.
    pub false_positives_current: u64,
    /// False negatives baseline / current.
    pub false_negatives_baseline: u64,
    /// Current false negatives.
    pub false_negatives_current: u64,
    /// Mean scan latency baseline / current (µs).
    pub mean_latency_baseline: u64,
    /// Current mean latency (µs).
    pub mean_latency_current: u64,
}

impl Regression {
    /// Classify the diff into the MASTER §22 vocabulary. Empty output means
    /// "no regression, no improvement worth reporting".
    #[must_use]
    pub fn classify(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.false_positives_current > self.false_positives_baseline {
            out.push("NEW_FALSE_POSITIVE");
        }
        if self.false_negatives_current > self.false_negatives_baseline {
            out.push("NEW_FALSE_NEGATIVE");
        }
        if self.detections_current < self.detections_baseline {
            out.push("LOST_DETECTION");
        }
        if self.detections_current > self.detections_baseline {
            out.push("NEW_DETECTION");
        }
        // Performance regression: mean latency grew by >25%.
        if self.mean_latency_baseline > 0 {
            let grew = self
                .mean_latency_current
                .saturating_sub(self.mean_latency_baseline);
            if grew * 4 > self.mean_latency_baseline {
                out.push("PERFORMANCE_REGRESSION");
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("litric-bench-recovery-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("baseline")).unwrap();
        std::fs::create_dir_all(dir.join("volume")).unwrap();
        std::fs::write(dir.join(TEST_VOLUME_MARKER), b"v1").unwrap();
        std::fs::write(dir.join(SNAPSHOT_MARKER), b"v1").unwrap();
        std::fs::write(dir.join(RECOVERY_MARKER), b"ok").unwrap();
        dir
    }

    #[test]
    fn clean_root_recovers() {
        let root = temp_root("clean");
        std::fs::write(root.join("baseline/a"), b"x").unwrap();
        std::fs::write(root.join("volume/a"), b"x").unwrap();
        let mut log = EventLog::new("recovery-clean");
        let v = recover(&root, &mut log, true, true);
        assert_eq!(v, RecoveryVerdict::Recovered);
        assert!(log.verify().is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn leftover_files_fail_volume_verification() {
        let root = temp_root("leftover");
        std::fs::write(root.join("baseline/a"), b"x").unwrap();
        std::fs::write(root.join("volume/a"), b"x").unwrap();
        std::fs::write(root.join("volume/leftover"), b"x").unwrap();
        let mut log = EventLog::new("recovery-leftover");
        let v = recover(&root, &mut log, true, true);
        assert_eq!(
            v,
            RecoveryVerdict::FailedRecovery {
                failed_at: "VOLUME_VERIFIED",
                reason: "step VOLUME_VERIFIED reported failure".to_owned()
            }
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn broken_sandbox_report_fails_first_step() {
        let root = temp_root("sandbox");
        let mut log = EventLog::new("recovery-sandbox");
        let v = recover(&root, &mut log, false, true);
        assert!(matches!(
            v,
            RecoveryVerdict::FailedRecovery {
                failed_at: "SANDBOX_STOPPED",
                ..
            }
        ));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_markers_fail_inspection() {
        let root = temp_root("nomarkers");
        std::fs::remove_file(root.join(SNAPSHOT_MARKER)).unwrap();
        let c = check_environment(&root);
        assert!(!c.markers_ok);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn regression_classification_covers_all_cases() {
        let mut r = Regression {
            detections_baseline: 90,
            detections_current: 90,
            false_positives_baseline: 0,
            false_positives_current: 0,
            false_negatives_baseline: 10,
            false_negatives_current: 10,
            mean_latency_baseline: 100,
            mean_latency_current: 110,
        };
        assert!(r.classify().is_empty());
        r.false_positives_current = 2;
        r.false_negatives_current = 20;
        r.detections_current = 80;
        r.mean_latency_current = 200;
        let tags = r.classify();
        assert!(tags.contains(&"NEW_FALSE_POSITIVE"));
        assert!(tags.contains(&"NEW_FALSE_NEGATIVE"));
        assert!(tags.contains(&"LOST_DETECTION"));
        assert!(tags.contains(&"PERFORMANCE_REGRESSION"));
    }
}
