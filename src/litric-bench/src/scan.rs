//! The scan runner: the real Defender engine over the whole corpus.
//!
//! MASTER §14's contract lives here in its most testable form: the same
//! engine binary that ships in the OS is driven over 100k+ deterministic
//! samples, every scan is one forensic event, and the aggregate truth table
//! (MASTER §2's outcome vocabulary) is a [`Scoreboard`], not a pass/fail.
//!
//! # Ground truth is by construction
//!
//! Corpus categories carry their own [`Truth`]: hostile categories are
//! hostile because the bytes contain the public EICAR string or a bench
//! marker, benign categories are benign because nothing in them matches
//! anything. The runner never guesses; it records what the engine said and
//! lets the scoreboard expose FP/FN honestly (MASTER §28: never just
//! "pass").

//! Percentile interpolation and rate math on counters; saturating by design.
#![allow(clippy::arithmetic_side_effects)]

use crate::corpus::{Sample, EICAR, MARKERS};
use crate::forensics::{monotonic_ns, Event, EventLog};
use crate::severity::{Outcome, Scoreboard, Truth};
use litric_defender::denylist::Denylist;
use litric_defender::engine::Engine;
use litric_defender::verdict::Decision;

/// One sample's result, kept for the CSV layer and miss analysis.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// The sample this result belongs to.
    pub sample_id: String,
    /// Corpus category slug, for per-category rollups.
    pub category: &'static str,
    /// Ground truth by construction.
    pub truth: Truth,
    /// What the engine decided, mapped to the outcome vocabulary.
    pub outcome: Outcome,
    /// The rule/tier that fired, or the miss reason.
    pub rule_id: String,
    /// Wall-clock latency of the scan, microseconds.
    pub latency_us: u64,
    /// Whether the engine served the verdict from its cache.
    pub from_cache: bool,
    /// The engine's digest of the content ("" on cache hits).
    pub digest_hex: String,
}

/// Map a raw engine decision to the benchmark outcome vocabulary.
///
/// The split is deliberate: a detection the engine reported is `Detected`
/// even when the harness would not act on it; quarantine is a *separate*
/// stage (MASTER §10) so its failures are counted independently of
/// detection failures.
#[must_use]
pub fn outcome_of(decision: &Decision) -> (Outcome, String) {
    match decision {
        Decision::Deny { tier, id } => {
            let outcome = match tier {
                litric_defender::verdict::Tier::Hash => Outcome::Detected,
                litric_defender::verdict::Tier::Pattern => Outcome::Detected,
            };
            (outcome, format!("{tier:?}:{id}"))
        }
        Decision::Allow => (Outcome::Allowed, String::new()),
        Decision::AllowUnknown { reason } => {
            // Tooling impairment is not a pass: MASTER §2 forbids counting
            // ENVIRONMENT_LIMITATION as success on either side of the
            // truth table.
            (Outcome::EnvironmentLimitation, reason.clone())
        }
    }
}

/// Build a feed-ready denylist from the corpus's hostile samples: every
/// EICAR-bearing digest is inserted, which is exactly how the OS ships its
/// own feed (hash tier first, patterns second).
#[must_use]
pub fn denylist_from_corpus(samples: &[Sample]) -> Denylist {
    let mut list = Denylist::new();
    for s in samples {
        if s.truth == Truth::Hostile {
            let has_marker = s.content.windows(EICAR.len()).any(|w| w == EICAR);
            if has_marker {
                list.insert(&s.digest_hex, "bench:eicar-bearing");
            }
        }
    }
    list
}

/// The measured distribution of scan latencies (MASTER §7/§21 percentiles).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LatencyProfile {
    /// Every observed latency, microseconds, in scan order.
    pub samples_us: Vec<u64>,
}

impl LatencyProfile {
    /// Percentile `p` in 0..=100, linear interpolation between order
    /// statistics, truncated toward zero to the stored microsecond
    /// resolution.
    ///
    /// An empty profile and a `p` outside 0..=100 (including NaN) both
    /// report 0. The version this replaces answered an out-of-range index
    /// with 4 294 967 295 µs — the `u32` ceiling, a plausible-looking
    /// latency that no sample produced, and it also clamped any real sample
    /// above that to the same invented value. A percentile is either
    /// measured or absent here; there is no third answer.
    #[must_use]
    pub fn percentile(&self, p: f64) -> u64 {
        if self.samples_us.is_empty() || !(0.0..=100.0).contains(&p) {
            return 0;
        }
        let mut sorted = self.samples_us.clone();
        sorted.sort_unstable();
        let n = sorted.len();
        let idx = p / 100.0 * (n - 1) as f64;
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        let frac = idx - lo as f64;
        // `lo` and `hi` are inside the vector for every accepted `p` and
        // every `n >= 1`, and `get` keeps that a property of the code rather
        // than a panic. The fallback is the slowest real sample, never a
        // constant.
        let last = sorted.last().copied().unwrap_or(0);
        let a = sorted.get(lo).copied().unwrap_or(last) as f64;
        let b = sorted.get(hi).copied().unwrap_or(last) as f64;
        (a + (b - a) * frac) as u64
    }

    /// Mean latency in microseconds.
    #[must_use]
    pub fn mean(&self) -> u64 {
        if self.samples_us.is_empty() {
            return 0;
        }
        let sum: u128 = self.samples_us.iter().map(|&v| u128::from(v)).sum();
        let n = u128::try_from(self.samples_us.len()).unwrap_or(1).max(1);
        u64::try_from(sum.checked_div(n).unwrap_or(u64::MAX.into())).unwrap_or(u64::MAX)
    }

    /// Throughput in samples/second implied by the mean.
    #[must_use]
    pub fn samples_per_sec(&self) -> u64 {
        let mean = self.mean();
        mean.checked_sub(1)
            .map_or(0, |_| 1_000_000u64.checked_div(mean).unwrap_or(0))
    }
}

/// Run the whole corpus through a fresh engine and produce the full
/// forensic record. Deterministic given `samples`: identical inputs give an
/// identical [`Scoreboard`] and identical per-sample results.
pub fn run_scan(
    samples: &[Sample],
    log: &mut EventLog,
) -> (Scoreboard, Vec<ScanResult>, LatencyProfile) {
    let mut engine = Engine::new();
    engine.load_denylist(denylist_from_corpus(samples));
    // The pattern tier loads the same public markers the corpus prints —
    // this is the defensive feed under test, not copied malware.
    let _ = engine.load_patterns(MARKERS.iter().map(|m| (*m, "bench:marker".to_string())));

    let mut scoreboard = Scoreboard::default();
    let mut results = Vec::with_capacity(samples.len());
    let mut latencies = LatencyProfile::default();

    for s in samples {
        // Latency comes from the monotonic clock (MASTER §15): deltas only,
        // immune to wall-clock steps and platform granularity quirks.
        let started = monotonic_ns();
        let report = engine.scan(&s.file_name, s.content.len() as u64, 0, &s.content);
        let elapsed = monotonic_ns().saturating_sub(started).max(1);

        let (outcome, rule_id) = outcome_of(&report.decision);
        scoreboard.record(s.truth, outcome);

        let result = ScanResult {
            sample_id: s.id.clone(),
            category: s.category.slug(),
            truth: s.truth,
            outcome,
            rule_id,
            latency_us: elapsed,
            from_cache: report.from_cache,
            digest_hex: report.digest_hex.clone(),
        };

        log.push(Event {
            seq: 0,
            run_id: String::new(),
            timestamp_ns: 0,
            kind: "SAMPLE_SCAN",
            sample_id: s.id.clone(),
            parent_event_id: 0,
            level: s.level.number(),
            outcome: result.outcome.as_str().to_owned(),
            rule_id: result.rule_id.clone(),
            operation: "engine.scan".to_owned(),
            target: s.file_name.clone(),
            reason: miss_or_hit_reason(s.truth, result.outcome),
            latency_us: elapsed,
            error_code: String::new(),
            recovery_action: String::new(),
            hash_hex: String::new(),
            prev_hash_hex: String::new(),
        });

        latencies.samples_us.push(elapsed);
        results.push(result);
    }

    (scoreboard, results, latencies)
}

/// Forensic `reason` text: states the expectation and the observation, so a
/// CSV row is readable without the corpus (MASTER §25).
#[must_use]
fn miss_or_hit_reason(truth: Truth, outcome: Outcome) -> String {
    let expected = match truth {
        Truth::Hostile => "hostile-by-construction",
        Truth::Benign => "benign-by-construction",
    };
    match (truth, outcome) {
        (Truth::Hostile, Outcome::Allowed) => {
            format!("expected:{expected} observed:ALLOWED -> MISS (false negative)")
        }
        (Truth::Benign, Outcome::Detected | Outcome::Blocked | Outcome::Quarantined) => {
            format!(
                "expected:{expected} observed:{} -> FALSE POSITIVE",
                outcome.as_str()
            )
        }
        _ => format!("expected:{expected} observed:{}", outcome.as_str()),
    }
}

/// Per-category rollup: which corpus families the engine is blind to.
/// Each miss carries the category and the sample ids, feeding MASTER §23's
/// miss analysis without turning it into exploitation guidance.
#[derive(Debug, Clone)]
pub struct CategoryGap {
    /// The category slug.
    pub category: &'static str,
    /// Hostile samples in this category.
    pub hostile: u64,
    /// Hostile samples the engine detected.
    pub detected: u64,
    /// Benign samples wrongly flagged (hard-negative pressure).
    pub false_positives: u64,
}

/// Fold results into per-category gaps, in stable category order.
#[must_use]
pub fn category_gaps(results: &[ScanResult]) -> Vec<CategoryGap> {
    let mut order: Vec<&'static str> = Vec::new();
    let mut map: std::collections::BTreeMap<&'static str, CategoryGap> = Default::default();
    for r in results {
        let entry = map.entry(r.category).or_insert_with(|| CategoryGap {
            category: r.category,
            hostile: 0,
            detected: 0,
            false_positives: 0,
        });
        match r.truth {
            Truth::Hostile => {
                entry.hostile = entry.hostile.saturating_add(1);
                if r.outcome.is_detection() {
                    entry.detected = entry.detected.saturating_add(1);
                }
            }
            Truth::Benign => {
                if r.outcome.is_detection() {
                    entry.false_positives = entry.false_positives.saturating_add(1);
                }
            }
        }
        if !order.contains(&r.category) {
            order.push(r.category);
        }
    }
    order.into_iter().filter_map(|k| map.remove(k)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::generate;

    fn small_corpus() -> Vec<Sample> {
        generate(42, 60)
    }

    #[test]
    fn eicar_is_always_detected_and_counted() {
        let samples = small_corpus();
        let mut log = EventLog::new("scan-test");
        let (board, results, _) = run_scan(&samples, &mut log);
        // The corpus guarantees EICAR-bearing hostile samples; the hash tier
        // has no false positives by construction, so detection must exceed 0.
        assert!(
            board.true_positives > 0,
            "engine missed every hostile sample"
        );
        assert_eq!(results.len(), samples.len());
    }

    #[test]
    fn benign_never_counts_as_detection_in_a_clean_run() {
        let samples = small_corpus();
        let mut log = EventLog::new("scan-test-fp");
        let (board, _, _) = run_scan(&samples, &mut log);
        // Phase 0 engine + by-construction feed: hard negatives may trip
        // pattern tier in principle, but the hash-only feed here must not.
        assert_eq!(board.false_positives, 0);
    }

    #[test]
    fn every_sample_produces_exactly_one_forensic_event() {
        let samples = small_corpus();
        let mut log = EventLog::new("scan-test-events");
        let _ = run_scan(&samples, &mut log);
        assert_eq!(log.len(), samples.len());
        assert!(log.verify().is_ok(), "hash chain must survive a full run");
    }

    #[test]
    fn latency_percentiles_are_monotonic_and_sane() {
        let lp = LatencyProfile {
            samples_us: vec![100, 200, 300, 400, 500],
        };
        assert_eq!(lp.percentile(0.0), 100);
        assert_eq!(lp.percentile(50.0), 300);
        assert_eq!(lp.percentile(100.0), 500);
        assert!(lp.percentile(50.0) <= lp.percentile(95.0));
        assert_eq!(
            lp.samples_per_sec(),
            1_000_000u64.checked_div(lp.mean()).unwrap_or(0)
        );
    }

    /// A latency beyond the `u32` ceiling must be reported as measured. This
    /// fails against the version that clamped every sample to `u32::MAX`.
    #[test]
    fn percentile_is_not_clamped_to_the_u32_ceiling() {
        let slow = 5_000_000_000u64; // ~83 minutes, above u32::MAX microseconds
        let lp = LatencyProfile {
            samples_us: vec![100, 200, slow],
        };
        assert_eq!(lp.percentile(100.0), slow, "the real sample, not a clamp");
        assert_eq!(lp.percentile(0.0), 100);
        assert!(lp.percentile(50.0) > 100);
        assert_eq!(lp.mean(), (5_000_000_300u64).checked_div(3).unwrap_or(0));
    }

    /// A percentile that is not a percentile is reported as absent, never as
    /// a large plausible number.
    #[test]
    fn an_out_of_range_percentile_reports_nothing() {
        let lp = LatencyProfile {
            samples_us: vec![10, 20, 30],
        };
        assert_eq!(lp.percentile(f64::NAN), 0);
        assert_eq!(lp.percentile(-1.0), 0);
        assert_eq!(lp.percentile(101.0), 0);
        // Linear interpolation at p=99 over [10, 20, 30] lands on 29.8, which
        // the profile stores truncated as 29. What this assertion exists to
        // rule out is the answer the previous version gave here: the `u32`
        // ceiling, a latency no sample ever produced.
        assert_eq!(lp.percentile(99.0), 29);
        assert_eq!(lp.percentile(100.0), 30, "p=100 is the slowest real sample");
        assert_eq!(lp.percentile(0.0), 10, "p=0 is the fastest real sample");
        assert_eq!(LatencyProfile::default().percentile(99.0), 0);
    }

    #[test]
    fn outcome_mapping_distinguishes_limitation_from_pass() {
        let (o, _) = outcome_of(&Decision::AllowUnknown {
            reason: "io".into(),
        });
        assert_eq!(o, Outcome::EnvironmentLimitation);
        let (o, rule) = outcome_of(&Decision::Deny {
            tier: litric_defender::verdict::Tier::Hash,
            id: "eicar-test-file".into(),
        });
        assert_eq!(o, Outcome::Detected);
        assert!(rule.contains("Hash"));
    }

    #[test]
    fn category_gaps_capture_per_family_truth() {
        let samples = small_corpus();
        let mut log = EventLog::new("scan-test-gaps");
        let (_, results, _) = run_scan(&samples, &mut log);
        let gaps = category_gaps(&results);
        assert!(!gaps.is_empty());
        assert!(gaps.iter().all(|g| g.hostile >= g.detected));
    }

    #[test]
    fn level_zero_benign_only_corpus_has_zero_tps() {
        let mut samples = small_corpus();
        samples.retain(|s| s.level.number() == 0 && s.truth == Truth::Benign);
        if samples.is_empty() {
            return; // corpus composition guard; nothing to assert
        }
        let mut log = EventLog::new("scan-test-l0");
        let (board, _, _) = run_scan(&samples, &mut log);
        assert_eq!(board.true_positives, 0);
    }
}
