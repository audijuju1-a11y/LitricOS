//! The chaos engine: seeded, reproducible event storms (MASTER §14).
//!
//! Chaos here is a *sequence* of synthetic events drawn from every class
//! the OS must survive — file, process, thread, memory, I/O, network,
//! security, telemetry, crash, resource. The sequence is a pure function of
//! the seed: seed 928374 replays byte-identically on any machine, which is
//! what turns a chaos run from an anecdote into a regression test.
//!
//! Nothing in this module performs the events against a live system; it
//! produces the schedule the sandbox harness executes inside the
//! TEST_VOLUME, plus the replayable record for the forensic log.

//! The weighted class draw and magnitude bands are modular arithmetic on
//! u64 by design (SplitMix64 output); all operands are unsigned and masked.
#![allow(clippy::arithmetic_side_effects)]

use crate::severity::Level;

/// The event classes chaos draws from, in stable order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventClass {
    /// File create/write/delete/rename.
    FileEvent,
    /// Process start/exit churn.
    ProcessEvent,
    /// Thread spawn/join churn.
    ThreadEvent,
    /// Allocation pressure.
    MemoryEvent,
    /// Read/write bursts.
    IoEvent,
    /// Loopback-only connection storms.
    NetworkEvent,
    /// Detection/allow/deny feed pressure.
    SecurityEvent,
    /// Telemetry batch pressure.
    TelemetryEvent,
    /// Crash injection points.
    CrashEvent,
    /// Resource ceiling probes.
    ResourceEvent,
}

impl EventClass {
    /// All classes in stable order.
    #[must_use]
    pub fn all() -> &'static [EventClass] {
        &[
            Self::FileEvent,
            Self::ProcessEvent,
            Self::ThreadEvent,
            Self::MemoryEvent,
            Self::IoEvent,
            Self::NetworkEvent,
            Self::SecurityEvent,
            Self::TelemetryEvent,
            Self::CrashEvent,
            Self::ResourceEvent,
        ]
    }

    /// Stable machine name for logs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileEvent => "FILE_EVENT",
            Self::ProcessEvent => "PROCESS_EVENT",
            Self::ThreadEvent => "THREAD_EVENT",
            Self::MemoryEvent => "MEMORY_EVENT",
            Self::IoEvent => "IO_EVENT",
            Self::NetworkEvent => "NETWORK_EVENT",
            Self::SecurityEvent => "SECURITY_EVENT",
            Self::TelemetryEvent => "TELEMETRY_EVENT",
            Self::CrashEvent => "CRASH_EVENT",
            Self::ResourceEvent => "RESOURCE_EVENT",
        }
    }
}

/// One scheduled chaos step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChaosStep {
    /// 1-based position in the sequence.
    pub seq: u64,
    /// The class the step exercises.
    pub class: EventClass,
    /// Deterministic payload selector within the class (0..=255).
    pub variant: u8,
    /// Magnitude for the harness (bytes, counts, iterations).
    pub magnitude: u64,
}

/// A full chaos schedule: replay it anywhere, get the same storm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChaosSchedule {
    /// The seed that generated (and regenerates) this schedule.
    pub seed: u64,
    /// The steps, in execution order.
    pub steps: Vec<ChaosStep>,
    /// Level the schedule targets.
    pub level: Level,
}

/// SplitMix64 — identical constants to the corpus's generator so both
/// layers derive from the same PRNG family without sharing state.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Generate the schedule for `seed` with `count` steps at `level`.
///
/// Determinism contract: `generate(seed, n)` twice gives two equal
/// schedules. Level weights the mix — higher levels draw crash and
/// resource events more often (MASTER §27's escalation ladder in data).
#[must_use]
pub fn generate(seed: u64, count: usize, level: Level) -> ChaosSchedule {
    let mut rng = Rng(seed | 1);
    let lvl = u32::from(level.number());
    let mut steps = Vec::with_capacity(count);

    for i in 0..count {
        // Weighted class draw: crash/resource weight grows with level but
        // never dominates; file/io remain the substrate at every level.
        let roll = rng.next() % 100;
        let crash_hi = 92 + u64::from(lvl.min(6));
        let class = if roll <= 39 {
            EventClass::FileEvent
        } else if roll <= 54 {
            EventClass::IoEvent
        } else if roll <= 64 {
            EventClass::MemoryEvent
        } else if roll <= 71 {
            EventClass::ProcessEvent
        } else if roll <= 78 {
            EventClass::ThreadEvent
        } else if roll <= 83 {
            EventClass::NetworkEvent
        } else if roll <= 88 {
            EventClass::SecurityEvent
        } else if roll <= 92 {
            EventClass::TelemetryEvent
        } else if roll <= crash_hi {
            EventClass::CrashEvent
        } else {
            EventClass::ResourceEvent
        };
        let magnitude = match class {
            EventClass::FileEvent => rng.next() % 4096 + 1,
            EventClass::IoEvent => rng.next() % 65536 + 1,
            EventClass::MemoryEvent => rng.next() % (1024 * 1024) + 1,
            EventClass::ProcessEvent | EventClass::ThreadEvent => rng.next() % 64 + 1,
            EventClass::NetworkEvent => rng.next() % 1500 + 1,
            EventClass::SecurityEvent => rng.next() % 500 + 1,
            EventClass::TelemetryEvent => rng.next() % 8192 + 1,
            EventClass::CrashEvent => 1,
            EventClass::ResourceEvent => rng.next() % 32 + 1,
        };
        steps.push(ChaosStep {
            seq: i as u64 + 1,
            class,
            variant: (rng.next() % 256) as u8,
            magnitude,
        });
    }

    ChaosSchedule { seed, steps, level }
}

/// Count steps per class — the mix report the CLI prints before running.
#[must_use]
pub fn class_histogram(schedule: &ChaosSchedule) -> Vec<(EventClass, usize)> {
    EventClass::all()
        .iter()
        .map(|c| {
            let n = schedule.steps.iter().filter(|s| s.class == *c).count();
            (*c, n)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::severity::Level;

    #[test]
    fn same_seed_same_storm() {
        let a = generate(928_374, 10_000, Level::L3SystemImpact);
        let b = generate(928_374, 10_000, Level::L3SystemImpact);
        assert_eq!(a, b, "MASTER §14: seed 928374 -> identical sequence");
    }

    #[test]
    fn different_seed_different_storm() {
        let a = generate(1, 100, Level::L2HighRisk);
        let b = generate(2, 100, Level::L2HighRisk);
        assert_ne!(a, b);
    }

    #[test]
    fn histogram_sums_to_total_and_covers_classes() {
        let s = generate(77, 2_000, Level::L4KernelImpact);
        let hist = class_histogram(&s);
        let total: usize = hist.iter().map(|(_, n)| *n).sum();
        assert_eq!(total, s.steps.len());
        // With 2000 draws across 10 classes at any level, every class must
        // appear at least once (the weakest weight is 2%).
        for (c, n) in &hist {
            assert!(*n > 0, "class {c:?} never drawn");
        }
    }

    #[test]
    fn higher_level_draws_more_crash_events() {
        let low = class_histogram(&generate(9, 5_000, Level::L1Suspicious));
        let high = class_histogram(&generate(9, 5_000, Level::L5Catastrophic));
        let crashes = |h: &[(EventClass, usize)]| {
            h.iter()
                .find(|(c, _)| *c == EventClass::CrashEvent)
                .map_or(0, |(_, n)| *n)
        };
        assert!(
            crashes(&high) > crashes(&low),
            "escalation ladder must be visible in the mix"
        );
    }

    #[test]
    fn seq_is_dense_from_one() {
        let s = generate(3, 50, Level::L2HighRisk);
        for (i, step) in s.steps.iter().enumerate() {
            assert_eq!(step.seq, i as u64 + 1);
        }
    }
}
