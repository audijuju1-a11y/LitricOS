//! Severity levels and the outcome vocabulary every result reports.
//!
//! MASTER §2: six levels, ten outcomes. The outcomes are the honest part —
//! a test that crashed the harness reports `CRASHED`, not `DETECTED`, and
//! an environment that cannot run a class at all reports
//! [`Outcome::EnvironmentLimitation`] rather than silently skipping. A
//! benchmark that blurs these lies in its headline numbers.

//! Scoreboard counters and rate math saturate by contract; operands are
//! non-negative counters only.
#![allow(clippy::arithmetic_side_effects)]

/// One of the six severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// No hostile property at all; the false-positive baseline.
    L0Benign,
    /// Shapes that look threatening but carry nothing.
    L1Suspicious,
    /// High-risk behaviour simulated inside one process.
    L2HighRisk,
    /// Simulated impact on system resources and services.
    L3SystemImpact,
    /// Simulated kernel-interface and driver-interface impact.
    L4KernelImpact,
    /// Catastrophic-failure simulation; disposable snapshot required.
    L5Catastrophic,
}

impl Level {
    /// The numeric level (0..=5), as used in logs and the CLI `--level`.
    #[must_use]
    pub fn number(self) -> u8 {
        match self {
            Self::L0Benign => 0,
            Self::L1Suspicious => 1,
            Self::L2HighRisk => 2,
            Self::L3SystemImpact => 3,
            Self::L4KernelImpact => 4,
            Self::L5Catastrophic => 5,
        }
    }

    /// Parse a level from its number.
    #[must_use]
    pub fn from_number(n: u8) -> Option<Self> {
        Some(match n {
            0 => Self::L0Benign,
            1 => Self::L1Suspicious,
            2 => Self::L2HighRisk,
            3 => Self::L3SystemImpact,
            4 => Self::L4KernelImpact,
            5 => Self::L5Catastrophic,
            _ => return None,
        })
    }
}

/// What actually happened to a sample. Ten values, no "other".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The engine fired on it.
    Detected,
    /// Detected and the quarantine action succeeded (simulated).
    Blocked,
    /// Detected and the sample moved into quarantine storage.
    Quarantined,
    /// No detection fired (correct for benign, a MISS otherwise).
    Allowed,
    /// The component under test crashed.
    Crashed,
    /// The component did not answer within its budget.
    Timeout,
    /// Failure followed by successful recovery.
    Recovered,
    /// Failure followed by failed recovery — an incident.
    FailedRecovery,
    /// The engine has no rule or parser for this class at all.
    Unsupported,
    /// The environment cannot exercise this class; never counted as pass.
    EnvironmentLimitation,
}

impl Outcome {
    /// Stable machine name used in JSONL/CSV.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Detected => "DETECTED",
            Self::Blocked => "BLOCKED",
            Self::Quarantined => "QUARANTINED",
            Self::Allowed => "ALLOWED",
            Self::Crashed => "CRASHED",
            Self::Timeout => "TIMEOUT",
            Self::Recovered => "RECOVERED",
            Self::FailedRecovery => "FAILED_RECOVERY",
            Self::Unsupported => "UNSUPPORTED",
            Self::EnvironmentLimitation => "ENVIRONMENT_LIMITATION",
        }
    }

    /// Parse back from the machine name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "DETECTED" => Self::Detected,
            "BLOCKED" => Self::Blocked,
            "QUARANTINED" => Self::Quarantined,
            "ALLOWED" => Self::Allowed,
            "CRASHED" => Self::Crashed,
            "TIMEOUT" => Self::Timeout,
            "RECOVERED" => Self::Recovered,
            "FAILED_RECOVERY" => Self::FailedRecovery,
            "UNSUPPORTED" => Self::Unsupported,
            "ENVIRONMENT_LIMITATION" => Self::EnvironmentLimitation,
            _ => return None,
        })
    }

    /// Did the engine raise a hand at all?
    #[must_use]
    pub fn is_detection(self) -> bool {
        matches!(self, Self::Detected | Self::Blocked | Self::Quarantined)
    }
}

/// Ground truth of a sample, decided at generation time by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truth {
    /// A well-behaved sample: a detection here is a FALSE POSITIVE.
    Benign,
    /// A hostile-equivalent sample: an ALLOWED here is a MISS (false
    /// negative). No generated sample is an unintended true negative —
    /// that ambiguity is exactly what the ground truth removes.
    Hostile,
}

impl Truth {
    /// Stable machine name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Benign => "BENIGN",
            Self::Hostile => "HOSTILE",
        }
    }
}

/// Scoreboard for a run, split by ground truth.
#[derive(Debug, Default, Clone)]
pub struct Scoreboard {
    /// Hostile samples detected.
    pub true_positives: u64,
    /// Hostile samples allowed through.
    pub false_negatives: u64,
    /// Benign samples flagged.
    pub false_positives: u64,
    /// Benign samples correctly passed.
    pub true_negatives: u64,
    /// Samples the engine could not classify at all.
    pub unsupported: u64,
    /// Outcomes that are neither detection nor clean pass.
    pub operational: u64,
}

impl Scoreboard {
    /// Record one outcome against a truth.
    pub fn record(&mut self, truth: Truth, outcome: Outcome) {
        match (truth, outcome) {
            (Truth::Hostile, o) if o.is_detection() => self.true_positives += 1,
            (Truth::Hostile, Outcome::Allowed) => self.false_negatives += 1,
            (Truth::Benign, Outcome::Allowed) => self.true_negatives += 1,
            (Truth::Benign, o) if o.is_detection() => self.false_positives += 1,
            (_, Outcome::Unsupported) => self.unsupported += 1,
            _ => self.operational += 1,
        }
    }

    /// Detection rate over classified hostile samples (TP / (TP + FN)).
    ///
    /// Both operands go straight from `u64` to `f64`. Narrowing them to
    /// `u32` first, as this did, wraps any counter past 4 294 967 295 into a
    /// smaller number and reports a ratio no sample produced — the silent
    /// wrong-number failure this vocabulary exists to avoid. No corpus run
    /// here is that large, so the old form was not yet wrong in practice;
    /// it was wrong whenever it mattered.
    #[must_use]
    pub fn detection_rate(&self) -> f64 {
        let classified = self.true_positives + self.false_negatives;
        if classified == 0 {
            0.0
        } else {
            self.true_positives as f64 / classified as f64
        }
    }

    /// False-positive rate over classified benign samples (FP / (FP + TN)).
    #[must_use]
    pub fn false_positive_rate(&self) -> f64 {
        let classified = self.true_negatives + self.false_positives;
        if classified == 0 {
            0.0
        } else {
            self.false_positives as f64 / classified as f64
        }
    }

    /// How many samples the scoreboard has counted at all — the denominator
    /// a run's `corpus_size` must equal if every sample reached an outcome.
    #[must_use]
    pub const fn recorded(&self) -> u64 {
        self.true_positives
            + self.false_negatives
            + self.false_positives
            + self.true_negatives
            + self.unsupported
            + self.operational
    }

    /// How many hostile samples the detection rate is calculated over: the
    /// ones that reached a classification at all.
    #[must_use]
    pub const fn classified_hostile(&self) -> u64 {
        self.true_positives + self.false_negatives
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_round_trip_through_the_machine_names() {
        for o in [
            Outcome::Detected,
            Outcome::Blocked,
            Outcome::Quarantined,
            Outcome::Allowed,
            Outcome::Crashed,
            Outcome::Timeout,
            Outcome::Recovered,
            Outcome::FailedRecovery,
            Outcome::Unsupported,
            Outcome::EnvironmentLimitation,
        ] {
            assert_eq!(Outcome::parse(o.as_str()), Some(o), "{}", o.as_str());
        }
        assert_eq!(Outcome::parse("SOMETHING_ELSE"), None);
    }

    #[test]
    fn the_scoreboard_counts_each_quadrant_once() {
        let mut s = Scoreboard::default();
        s.record(Truth::Hostile, Outcome::Detected);
        s.record(Truth::Hostile, Outcome::Allowed);
        s.record(Truth::Benign, Outcome::Detected);
        s.record(Truth::Benign, Outcome::Allowed);
        assert_eq!(s.true_positives, 1);
        assert_eq!(s.false_negatives, 1);
        assert_eq!(s.false_positives, 1);
        assert_eq!(s.true_negatives, 1);
        assert!((s.detection_rate() - 0.5).abs() < 1e-9);
        assert!((s.false_positive_rate() - 0.5).abs() < 1e-9);
    }

    /// Every (truth, outcome) pair must land in exactly one counter, so no
    /// sample can be counted twice and none can vanish. This is the
    /// `recorded == number of samples` invariant the report is built on.
    #[test]
    fn every_truth_and_outcome_lands_in_exactly_one_counter() {
        let outcomes = [
            Outcome::Detected,
            Outcome::Blocked,
            Outcome::Quarantined,
            Outcome::Allowed,
            Outcome::Crashed,
            Outcome::Timeout,
            Outcome::Recovered,
            Outcome::FailedRecovery,
            Outcome::Unsupported,
            Outcome::EnvironmentLimitation,
        ];
        let mut s = Scoreboard::default();
        let mut n = 0u64;
        for truth in [Truth::Hostile, Truth::Benign] {
            for outcome in outcomes {
                s.record(truth, outcome);
                n += 1;
            }
        }
        assert_eq!(s.recorded(), n, "every sample is counted exactly once");
        // The quadrants only ever hold reachable combinations.
        assert_eq!(s.true_positives, 3, "detections on hostile samples");
        assert_eq!(s.false_positives, 3, "detections on benign samples");
        assert_eq!(s.false_negatives, 1, "hostile samples allowed");
        assert_eq!(s.true_negatives, 1, "benign samples allowed");
        assert_eq!(s.unsupported, 2, "`unsupported` is truth-independent");
        assert_eq!(s.operational, 10, "everything else is operational");
    }

    /// The rate must survive counters larger than the `u32` ceiling: this
    /// test fails against a version that narrows the operands first.
    #[test]
    fn rates_are_not_narrowed_to_u32() {
        let s = Scoreboard {
            true_positives: 5_000_000_000,
            false_negatives: 5_000_000_000,
            ..Scoreboard::default()
        };
        assert!(
            (s.detection_rate() - 0.5).abs() < 1e-9,
            "counter narrowing must not change the ratio"
        );
        assert_eq!(s.classified_hostile(), 10_000_000_000);
        // Both counters sit above the `u32` ceiling and their ratio is exactly
        // 1/5. The first draft of this test used 700_000_000 and
        // 3_500_000_000 against an expectation of 0.2 -- 0.7/4.2 is 1/6, so it
        // asserted a ratio no pair of counters produces, which is the same
        // class of invented number the narrowing bug produced.
        let fp = Scoreboard {
            false_positives: 5_000_000_000,
            true_negatives: 20_000_000_000,
            ..Scoreboard::default()
        };
        assert_eq!(fp.recorded(), 25_000_000_000, "counters survive as `u64`");
        assert!((fp.false_positive_rate() - 0.2).abs() < 1e-9);
    }

    #[test]
    fn crashed_is_never_a_detection_nor_a_clean_pass() {
        let mut s = Scoreboard::default();
        s.record(Truth::Hostile, Outcome::Crashed);
        s.record(Truth::Benign, Outcome::EnvironmentLimitation);
        assert_eq!(s.true_positives, 0);
        assert_eq!(s.false_negatives, 0);
        assert_eq!(s.operational, 2);
    }
}
