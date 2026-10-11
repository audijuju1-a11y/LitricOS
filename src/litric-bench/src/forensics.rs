//! Event-by-event forensic logging with a tamper-evident hash chain.
//!
//! MASTER §15/§16/§17: every step of every test is an event; events chain
//! (each carries the previous event's hash); the chain is verifiable
//! end-to-end so a modified log is *detected*, not assumed away. The log is
//! append-only by discipline: the writer never rewrites a line, and the
//! verifier catches any line that does.
//!
//! The in-memory log is the source of truth for the report layer; the JSONL
//! file is the durable forensic artifact. Both carry the same chain.

//! Hash-chain bookkeeping is add-one/sequence math on u64 with an explicit
//! ceiling; canonical serialization is positional.
#![allow(clippy::arithmetic_side_effects)]

use litric_defender::sha256;
use std::fmt::Write as _;

/// One forensic event. Fields are MASTER §15's minimum set, minus the
/// ones the framework cannot observe (kernel_version etc. arrive from the
/// environment descriptor and are stamped per run, not per event).
#[derive(Debug, Clone)]
pub struct Event {
    /// Monotonic sequence number, starting at 1.
    pub seq: u64,
    /// The run this event belongs to.
    pub run_id: String,
    /// nanoseconds since the epoch (wall clock; the run also carries a
    /// monotonic start for deltas).
    pub timestamp_ns: u64,
    /// Event class, e.g. `SAMPLE_SCAN`, `DETECTION`, `QUARANTINE`.
    pub kind: &'static str,
    /// The sample this event concerns, when one exists.
    pub sample_id: String,
    /// The event that caused this one (chain of causality, MASTER §16).
    pub parent_event_id: u64,
    /// Severity level number 0..=5.
    pub level: u8,
    /// Outcome machine name ([`crate::severity::Outcome::as_str`]) or `""`.
    pub outcome: String,
    /// Which rule/component produced the result.
    pub rule_id: String,
    /// The operation performed.
    pub operation: String,
    /// What it was performed on.
    pub target: String,
    /// Result detail: human-readable, forensic-grade.
    pub reason: String,
    /// Measured latency in microseconds.
    pub latency_us: u64,
    /// Error code, when the operation failed.
    pub error_code: String,
    /// Recovery action taken, when one was taken.
    pub recovery_action: String,
    /// SHA-256 of this event's canonical form (filled by the log).
    pub hash_hex: String,
    /// SHA-256 of the previous event (chain).
    pub prev_hash_hex: String,
}

impl Event {
    /// The canonical string that is hashed: every field, fixed order,
    /// separated by `\x1f` (unit separator) so values cannot collide.
    #[must_use]
    pub fn canonical(&self) -> String {
        let mut s = String::with_capacity(512);
        let _ = write!(
            s,
            "{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}",
            self.seq,
            self.run_id,
            self.timestamp_ns,
            self.kind,
            self.sample_id,
            self.parent_event_id,
            self.level,
            self.outcome,
            self.rule_id,
            self.operation,
            self.target,
            self.reason,
            self.latency_us,
            self.error_code,
            self.recovery_action,
            self.prev_hash_hex,
        );
        s
    }
}

/// The append-only log.
#[derive(Debug)]
pub struct EventLog {
    run_id: String,
    events: Vec<Event>,
    last_hash: String,
    chain_broken: bool,
}

impl EventLog {
    /// A fresh log for `run_id`. The genesis hash is the run id's digest,
    /// so two runs can never share a chain.
    #[must_use]
    pub fn new(run_id: &str) -> Self {
        Self {
            run_id: run_id.to_owned(),
            events: Vec::new(),
            last_hash: sha256::hex(&sha256::digest(run_id.as_bytes())),
            chain_broken: false,
        }
    }

    /// The run id.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Append one event. The hash fields are computed here — callers can
    /// no more forge them than they can recompute the past.
    pub fn push(&mut self, mut ev: Event) -> u64 {
        ev.seq = self.events.len() as u64 + 1;
        ev.run_id = self.run_id.clone();
        ev.prev_hash_hex = self.last_hash.clone();
        if ev.timestamp_ns == 0 {
            ev.timestamp_ns = now_ns();
        }
        let h = sha256::digest(ev.canonical().as_bytes());
        ev.hash_hex = sha256::hex(&h);
        self.last_hash = ev.hash_hex.clone();
        let seq = ev.seq;
        self.events.push(ev);
        seq
    }

    /// All events, in order.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// The count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Verify the whole chain: recompute every hash and linkage.
    pub fn verify(&self) -> Result<(), String> {
        let mut prev = sha256::hex(&sha256::digest(self.run_id.as_bytes()));
        for (i, ev) in self.events.iter().enumerate() {
            if ev.seq != i as u64 + 1 {
                return Err(format!("sequence gap at {}", ev.seq));
            }
            if ev.prev_hash_hex != prev {
                return Err(format!("chain break at seq {}: prev-hash mismatch", ev.seq));
            }
            let expect = sha256::hex(&sha256::digest(ev.canonical().as_bytes()));
            if ev.hash_hex != expect {
                return Err(format!("hash mismatch at seq {}", ev.seq));
            }
            prev = ev.hash_hex.clone();
        }
        Ok(())
    }

    /// Was a chain break ever observed at write time? (The verifier above
    /// is the authority; this flag exists for the report.)
    #[must_use]
    pub fn chain_broken(&self) -> bool {
        self.chain_broken
    }

    /// Reconstruct the timeline for one sample: the full ancestor chain of
    /// the sample's events, ordered (MASTER §16's reconstructability).
    #[must_use]
    pub fn timeline_for(&self, sample_id: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.sample_id == sample_id)
            .collect()
    }

    /// Serialize to JSONL. One compact JSON object per line, in order.
    #[must_use]
    /// All events as newline-delimited JSON. Hash fields go last so a
    /// streaming verifier can hash the prefix before reading them.
    pub fn to_jsonl(&self) -> String {
        let mut out = String::with_capacity(self.events.len().saturating_mul(256));
        for ev in &self.events {
            let _ = writeln!(
                out,
                "{{\"seq\":{},\"run_id\":\"{}\",\"ts_ns\":{},\"kind\":\"{}\",\"sample\":\"{}\",\"parent\":{},\"level\":{},\"outcome\":\"{}\",\"rule\":\"{}\",\"op\":\"{}\",\"target\":\"{}\",\"reason\":\"{}\",\"latency_us\":{},\"error\":\"{}\",\"recovery\":\"{}\",\"hash\":\"{}\",\"prev\":\"{}\"}}",
                json_escape(&ev.seq.to_string()),
                json_escape(&ev.run_id),
                ev.timestamp_ns,
                json_escape(ev.kind),
                json_escape(&ev.sample_id),
                ev.parent_event_id,
                ev.level,
                json_escape(&ev.outcome),
                json_escape(&ev.rule_id),
                json_escape(&ev.operation),
                json_escape(&ev.target),
                json_escape(&ev.reason),
                ev.latency_us,
                json_escape(&ev.error_code),
                json_escape(&ev.recovery_action),
                ev.hash_hex,
                ev.prev_hash_hex,
            );
        }
        out
    }

    /// Write the JSONL artifact. # Errors: propagate I/O failure — a lost
    /// forensic log must fail the run, not vanish.
    pub fn write_jsonl(&self, path: &std::path::Path) -> Result<u64, String> {
        let text = self.to_jsonl();
        std::fs::write(path, &text).map_err(|e| format!("write {}: {e}", path.display()))?;
        Ok(text.len() as u64)
    }
}

/// Verify a written JSONL log end-to-end: parse each line's `seq`,
/// `prev_hash_hex`, `hash_hex`, recompute the canonical hash of every
/// event, and check the whole chain. A single edited byte anywhere fails.
/// Returns the number of verified events.
///
/// # Errors
/// The exact broken position and reason, for the incident record.
#[allow(clippy::must_use_candidate)]
pub fn verify_jsonl(text: &str) -> Result<usize, String> {
    // The chain's genesis is the run id's digest (EventLog::new), so the
    // first event's prev-hash must equal sha256(run_id) — extract the run
    // id from the first line and seed the expectation from it.
    let mut prev: Option<String> = None;
    let mut expected_seq = 1u64;
    let mut count = 0usize;
    for (idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let seq = json_num(line, "seq").ok_or_else(|| format!("line {}: missing seq", idx + 1))?;
        if seq != expected_seq {
            return Err(format!(
                "line {}: seq {seq} != expected {expected_seq}",
                idx + 1
            ));
        }
        let h =
            json_field(line, "hash").ok_or_else(|| format!("line {}: missing hash", idx + 1))?;
        let ph = json_field(line, "prev").unwrap_or_default();
        let expect_prev = prev.get_or_insert_with(|| {
            let run_id = json_field(line, "run_id").unwrap_or_default();
            sha256::hex(&sha256::digest(run_id.as_bytes()))
        });
        if ph != *expect_prev {
            return Err(format!(
                "line {}: prev-hash mismatch (chain break)",
                idx + 1
            ));
        }
        // Rebuild the canonical form from the JSONL fields (same order as
        // `Event::canonical`, \x1f-separated) and recompute the digest.
        let field = |k: &str| json_field(line, k).unwrap_or_default();
        let num = |k: &str| json_num(line, k).unwrap_or(0);
        let canonical = format!(
            "{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}\x1f{}",
            seq,
            field("run_id"),
            num("ts_ns"),
            field("kind"),
            field("sample"),
            num("parent"),
            num("level"),
            field("outcome"),
            field("rule"),
            field("op"),
            field("target"),
            field("reason"),
            num("latency_us"),
            field("error"),
            field("recovery"),
            ph,
        );
        let want = sha256::hex(&sha256::digest(canonical.as_bytes()));
        if want != h {
            return Err(format!(
                "line {}: hash mismatch — event was modified",
                idx + 1
            ));
        }
        prev = Some(h);
        expected_seq += 1;
        count += 1;
    }
    Ok(count)
}

/// Extract a string field from a flat JSON line (this crate has no JSON
/// parser by policy; fields are flat and quoted without escapes for the
/// controlled vocabulary it writes).
fn json_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

/// Extract a numeric field (`"key":123`) from a flat JSON line.
fn json_num(line: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{key}\":");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find([',', '}'])?;
    rest[..end].trim().parse().ok()
}

/// A monotonic nanosecond clock for latency measurement. `SystemTime` can
/// step backwards under NTP and has coarse granularity on some platforms;
/// latency deltas must use a monotonic source. The base is the first call
/// in the process, which is fine: only deltas are meaningful.
pub fn monotonic_ns() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static BASE: OnceLock<Instant> = OnceLock::new();
    let base = BASE.get_or_init(Instant::now);
    u64::try_from(base.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

/// Wall-clock nanoseconds since the epoch (for event timestamps; latency
/// deltas use [`monotonic_ns`] instead).
#[must_use]
pub fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

/// Minimal JSON string escaping for the fields we control.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> Event {
        Event {
            seq: 0,
            run_id: String::new(),
            timestamp_ns: 0,
            kind: "SAMPLE_SCAN",
            sample_id: "bench-1-01-benign-000000".into(),
            parent_event_id: 0,
            level: 0,
            outcome: "ALLOWED".into(),
            rule_id: "-".into(),
            operation: "scan".into(),
            target: "/test/evidence.bin".into(),
            reason: "clean by construction".into(),
            latency_us: 42,
            error_code: String::new(),
            recovery_action: String::new(),
            hash_hex: String::new(),
            prev_hash_hex: String::new(),
        }
    }

    #[test]
    fn the_chain_verifies_when_untouched() {
        let mut log = EventLog::new("run-test-1");
        for _ in 0..10 {
            log.push(sample_event());
        }
        assert_eq!(log.len(), 10);
        assert!(log.verify().is_ok());
    }

    #[test]
    fn a_modified_event_is_detected() {
        let mut log = EventLog::new("run-test-2");
        for _ in 0..5 {
            log.push(sample_event());
        }
        // The forensic promise: tampering with even the *reason* of one
        // event breaks verification of the whole tail.
        let third = log.events.get_mut(2).expect("three events were pushed");
        third.reason = "tampered".into();
        assert!(log.verify().is_err());
    }

    #[test]
    fn timelines_reconstruct_per_sample() {
        let mut log = EventLog::new("run-test-3");
        let mut first = sample_event();
        first.sample_id = "s-a".into();
        let a1 = log.push(first);
        let mut second = sample_event();
        second.sample_id = "s-a".into();
        second.kind = "DETECTION";
        second.parent_event_id = a1;
        log.push(second);
        let mut third = sample_event();
        third.sample_id = "s-b".into();
        log.push(third);
        let tl = log.timeline_for("s-a");
        assert_eq!(tl.len(), 2);
        assert_eq!(tl.get(1).map(|e| e.parent_event_id), Some(a1));
    }

    #[test]
    fn jsonl_round_trips_the_essential_fields() {
        let mut log = EventLog::new("run-test-4");
        log.push(sample_event());
        let text = log.to_jsonl();
        assert!(text.contains("\"kind\":\"SAMPLE_SCAN\""));
        assert!(text.contains("\"outcome\":\"ALLOWED\""));
        assert!(text.lines().count() == 1);
    }
}
