//! litric-bench — LitricOS maximum-depth adversarial security benchmark.
//!
//! # What this is
//!
//! A destructive-equivalent test framework: it produces inputs and event
//! sequences whose *impact profile* equals a catastrophic security event —
//! parser pressure, memory and CPU exhaustion, event floods, tampering
//! attempts, quarantine abuse, crash and recovery loops — without ever
//! constructing a weaponizable artifact. Every "malicious" byte sequence in
//! this crate is either the public EICAR test string, random bytes with a
//! threatening shape, or a structural combination the engine has never seen.
//! Nothing here executes, connects, encrypts a real file, or survives a
//! reboot. That is the whole safety architecture ([`safety`]).
//!
//! # Layers
//!
//! ```text
//! safety     hard gates: nothing runs without a disposable root + snapshot
//! severity   LEVEL 0..5 and the outcome vocabulary every result uses
//! corpus     100k+ deterministic synthetic samples in 50 categories
//! forensics  append-only JSONL event log with a SHA-256 hash chain
//! scan       drives the real Defender engine over the corpus, scores FP/FN
//! stress     bounded filesystem / memory / CPU / telemetry pressure
//! chaos      seeded, reproducible event storms across every event class
//! recovery   post-run cleanup, integrity verification, baseline diff
//! report     RUN_SUMMARY / CSVs / REPORT.md from the event log alone
//! ```

#![forbid(unsafe_code)]

pub mod chaos;
pub mod corpus;
pub mod forensics;
pub mod recovery;
pub mod report;
pub mod safety;
pub mod scan;
pub mod severity;
pub mod stress;

/// The framework version, stamped into every event and report.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_is_stamped_from_the_crate() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }
}
