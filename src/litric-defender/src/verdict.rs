//! Verdict policy: the fail-open ladder that turns detection results into
//! a single decision (ADR-0022 §6).
//!
//! The rule, and why: Defender runs inside the OS it protects. A bug that
//! makes the scanner unable to scan must degrade to "unknown", never to
//! "the machine cannot open files". But an *actual detection* is the
//! opposite kind of event: it is evidence, not malfunction, and must never
//! be waved through by a policy bug. So the ladder is asymmetric:
//!
//! - unknown/broken tooling  -> ALLOW (fail-open), report as `AllowedUnknown`
//! - definite detection      -> DENY, always, regardless of policy mode
//!
//! `Permissive`/`Enforce` therefore differ only in what happens to
//! *unknowns*. This kills the classic fail-open/fail-closed trap where a
//! policy bug silently overrides a positive detection.

use crate::cache::Verdict;

/// Operating mode for unknown (unscanned / tooling-impaired) files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Unknown files are allowed (Phase 0 default: the engine is not yet on
    /// the critical path of the OS, so failing closed would buy nothing and
    /// cost availability).
    #[default]
    Permissive,
    /// Unknown files are denied. Phase 2+, when the on-access transport is
    /// proven and the verdict path is measured.
    Enforce,
}

/// What the scanner observed, before policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    /// Hash and patterns were checked; nothing matched.
    Clean,
    /// A definite detection: denylist label or pattern id, plus the tier.
    Detection {
        /// Which detection tier fired.
        tier: Tier,
        /// The denylist label or pattern identifier that matched.
        id: String,
    },
    /// The scan could not be completed (I/O error, unreadable file,
    /// unreadable feed state). This is tooling impairment, not evidence.
    Unscannable {
        /// Why the scan could not complete, for the audit record.
        reason: String,
    },
}

/// Which detection tier fired. Derived ordering is weakest-to-strongest, so
/// `Tier::Hash > Tier::Pattern`: when both tiers fire on one file, the hash
/// (no false positives by construction) is the report that counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// Byte-pattern match — strong but not infallible.
    Pattern,
    /// Exact SHA-256 match — no false positives by construction.
    Hash,
}

/// The decision, with an explanation attached — every decision the engine
/// emits must be auditable after the fact (ADR-0022 §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// No detection; file may be opened.
    Allow,
    /// No detection fired but the file was not actually scanned.
    AllowUnknown {
        /// Why the file could not be scanned, for the audit record.
        reason: String,
    },
    /// A detection fired; the file may not be opened.
    Deny {
        /// Which detection tier fired.
        tier: Tier,
        /// The denylist label or pattern identifier that matched.
        id: String,
    },
}

/// Resolve an observation under a mode. Pure function — everything the
/// policy needs is in its arguments, so the whole ladder is unit-testable
/// without any filesystem.
#[must_use]
pub fn resolve(observation: &Observation, mode: Mode) -> Decision {
    match observation {
        Observation::Clean => Decision::Allow,
        Observation::Detection { tier, id } => {
            // The asymmetric core: a real detection denies in every mode.
            Decision::Deny {
                tier: *tier,
                id: id.clone(),
            }
        }
        Observation::Unscannable { reason } => match mode {
            Mode::Permissive => Decision::AllowUnknown {
                reason: reason.clone(),
            },
            Mode::Enforce => Decision::Deny {
                tier: Tier::Pattern,
                id: format!("unscannable:{reason}"),
            },
        },
    }
}

/// True when the decision corresponds to a cacheable [`Verdict`]. `Allow`
/// and `AllowUnknown` cache as clean — caching an unknown as clean is the
/// deliberate Phase 0 trade-off (re-scanning unscannable files on every
/// open taxes the OS without adding evidence), and the cache key's
/// (size, mtime) binding means a file that *changes* re-scans anyway.
#[must_use]
pub fn cacheable_verdict(decision: &Decision) -> Option<Verdict> {
    match decision {
        Decision::Allow | Decision::AllowUnknown { .. } => Some(Verdict::Clean),
        Decision::Deny { .. } => Some(Verdict::Detected),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_allows_in_both_modes() {
        for mode in [Mode::Permissive, Mode::Enforce] {
            assert_eq!(resolve(&Observation::Clean, mode), Decision::Allow);
        }
    }

    /// The critical property: detections deny even in Permissive mode.
    #[test]
    fn detection_denies_even_when_permissive() {
        for mode in [Mode::Permissive, Mode::Enforce] {
            let obs = Observation::Detection {
                tier: Tier::Hash,
                id: "e3b0c442..empty-file".into(),
            };
            assert_eq!(
                resolve(&obs, mode),
                Decision::Deny {
                    tier: Tier::Hash,
                    id: "e3b0c442..empty-file".into()
                },
                "a detection must never be waved through by policy mode"
            );
        }
    }

    #[test]
    fn unscannable_follows_the_mode() {
        let obs = Observation::Unscannable {
            reason: "EIO".into(),
        };
        assert_eq!(
            resolve(&obs, Mode::Permissive),
            Decision::AllowUnknown {
                reason: "EIO".into()
            }
        );
        assert!(matches!(
            resolve(&obs, Mode::Enforce),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn every_decision_is_cacheable() {
        assert_eq!(cacheable_verdict(&Decision::Allow), Some(Verdict::Clean));
        assert_eq!(
            cacheable_verdict(&Decision::AllowUnknown { reason: "x".into() }),
            Some(Verdict::Clean)
        );
        assert_eq!(
            cacheable_verdict(&Decision::Deny {
                tier: Tier::Hash,
                id: "h".into()
            }),
            Some(Verdict::Detected)
        );
    }

    /// Tier ordering: hash evidence outranks pattern evidence when both
    /// exist (the engine reports the strongest tier).
    #[test]
    fn tier_ordering() {
        assert!(Tier::Hash > Tier::Pattern);
    }
}
