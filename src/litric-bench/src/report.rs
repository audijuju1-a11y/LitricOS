//! The report layer: every artifact MASTER §25 demands, generated from the
//! forensic event log alone (MASTER §28: if the log cannot answer it, the
//! run did not record it).
//!
//! All CSVs are RFC-4180-escaped; all JSON is hand-built with the same
//! `jstr` discipline as the forensic JSONL — this crate still has zero
//! third-party dependencies.

//! CSV/JSONL row emission and rate math on counters; saturating by design.
#![allow(clippy::arithmetic_side_effects)]

use crate::forensics::EventLog;
use crate::recovery::Regression;
use crate::scan::{CategoryGap, LatencyProfile, ScanResult};
use crate::severity::Truth;
use std::fmt::Write as _;

/// One escaped CSV field.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// A CSV row from fields.
fn csv_row(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| csv_field(f))
        .collect::<Vec<_>>()
        .join(",")
}

/// The complete report bundle for one run.
#[derive(Debug, Clone)]
pub struct RunReport {
    /// Run identity.
    pub run_id: String,
    /// Seed the corpus/chaos derived from.
    pub seed: u64,
    /// Corpus size.
    pub corpus_size: usize,
    /// Final scoreboard numbers (duplicated into the summary for greppers).
    pub true_positives: u64,
    /// Benign samples wrongly flagged.
    pub false_positives: u64,
    /// Hostile samples allowed through.
    pub false_negatives: u64,
    /// Benign samples correctly passed.
    pub true_negatives: u64,
    /// Samples the engine could not classify.
    pub unsupported: u64,
    /// Scans the environment could not exercise (never counted as pass).
    pub environment_limitations: u64,
    /// Latency profile.
    pub latency: LatencyProfile,
    /// Per-category gaps.
    pub gaps: Vec<CategoryGap>,
    /// Per-sample results (the CSV backbone).
    pub results: Vec<ScanResult>,
    /// Regression vs baseline, when a baseline exists.
    pub regression: Option<Regression>,
    /// Recovery verdict machine name.
    pub recovery: String,
}

impl RunReport {
    /// `RUN_SUMMARY.json` — the machine-readable headline.
    #[must_use]
    pub fn summary_json(&self) -> String {
        let mut s = String::with_capacity(1024);
        let _ = writeln!(s, "{{");
        let _ = writeln!(s, "  \"schema\": \"litric-bench.run-summary/1\",");
        let _ = writeln!(s, "  \"run_id\": {},", jstr(&self.run_id));
        let _ = writeln!(s, "  \"seed\": {},", self.seed);
        let _ = writeln!(s, "  \"corpus_size\": {},", self.corpus_size);
        let _ = writeln!(s, "  \"true_positives\": {},", self.true_positives);
        let _ = writeln!(s, "  \"false_positives\": {},", self.false_positives);
        let _ = writeln!(s, "  \"false_negatives\": {},", self.false_negatives);
        let _ = writeln!(s, "  \"true_negatives\": {},", self.true_negatives);
        let _ = writeln!(s, "  \"unsupported\": {},", self.unsupported);
        let _ = writeln!(
            s,
            "  \"environment_limitations\": {},",
            self.environment_limitations
        );
        let _ = writeln!(
            s,
            "  \"detection_rate\": {:.6},",
            rate(
                self.true_positives,
                self.true_positives + self.false_negatives
            )
        );
        let _ = writeln!(
            s,
            "  \"false_positive_rate\": {:.6},",
            rate(
                self.false_positives,
                self.false_positives + self.true_negatives
            )
        );
        let _ = writeln!(s, "  \"latency_mean_us\": {},", self.latency.mean());
        let _ = writeln!(
            s,
            "  \"latency_p50_us\": {},",
            self.latency.percentile(50.0)
        );
        let _ = writeln!(
            s,
            "  \"latency_p95_us\": {},",
            self.latency.percentile(95.0)
        );
        let _ = writeln!(
            s,
            "  \"latency_p99_us\": {},",
            self.latency.percentile(99.0)
        );
        let _ = writeln!(
            s,
            "  \"latency_p999_us\": {},",
            self.latency.percentile(99.9)
        );
        let _ = writeln!(
            s,
            "  \"samples_per_sec\": {},",
            self.latency.samples_per_sec()
        );
        let _ = writeln!(s, "  \"recovery\": {},", jstr(&self.recovery));
        let _ = writeln!(s, "  \"rule_gaps\": {},", self.gaps.len());
        let _ = writeln!(s, "}}");
        s
    }

    /// `RUN_SUMMARY.csv` — the same headline, one row.
    #[must_use]
    pub fn summary_csv(&self) -> String {
        let header = [
            "run_id",
            "seed",
            "corpus",
            "tp",
            "fp",
            "fn",
            "tn",
            "unsupported",
            "env_limit",
            "detect_rate",
            "fp_rate",
            "mean_us",
            "p95_us",
            "p99_us",
            "samples_per_sec",
            "recovery",
        ];
        let row = [
            self.run_id.clone(),
            self.seed.to_string(),
            self.corpus_size.to_string(),
            self.true_positives.to_string(),
            self.false_positives.to_string(),
            self.false_negatives.to_string(),
            self.true_negatives.to_string(),
            self.unsupported.to_string(),
            self.environment_limitations.to_string(),
            format!(
                "{:.6}",
                rate(
                    self.true_positives,
                    self.true_positives + self.false_negatives
                )
            ),
            format!(
                "{:.6}",
                rate(
                    self.false_positives,
                    self.false_positives + self.true_negatives
                )
            ),
            self.latency.mean().to_string(),
            self.latency.percentile(95.0).to_string(),
            self.latency.percentile(99.0).to_string(),
            self.latency.samples_per_sec().to_string(),
            self.recovery.clone(),
        ];
        let mut out = csv_row(&header.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>());
        out.push('\n');
        out.push_str(&csv_row(&row));
        out.push('\n');
        out
    }

    /// `DETECTIONS.csv` — every sample the engine fired on.
    #[must_use]
    pub fn detections_csv(&self) -> String {
        let mut out = String::from("sample_id,category,truth,outcome,rule_id,latency_us\n");
        for r in &self.results {
            if r.outcome.is_detection() {
                let _ = writeln!(
                    out,
                    "{},{},{},{},{},{}",
                    csv_field(&r.sample_id),
                    r.category,
                    r.truth.as_str(),
                    r.outcome.as_str(),
                    csv_field(&r.rule_id),
                    r.latency_us
                );
            }
        }
        out
    }

    /// `MISSES.csv` — every hostile sample the engine allowed (MASTER §23
    /// analysis data; feature profiles stay descriptive, never offensive).
    #[must_use]
    pub fn misses_csv(&self) -> String {
        let mut out = String::from(
            "sample_id,category,level,reason,available_telemetry,possible_defensive_fix\n",
        );
        for r in &self.results {
            if r.truth == Truth::Hostile && !r.outcome.is_detection() {
                // The defensive-fix column is a rule-gap note, never a
                // payload recipe (MASTER §23's boundary).
                let row = [
                    r.sample_id.clone(),
                    r.category.to_owned(),
                    "0".to_owned(),
                    "expected-hostile-observed-allowed".to_owned(),
                    "engine.scan".to_owned(),
                    "add hash or pattern coverage for this constructed family".to_owned(),
                ];
                let _ = writeln!(out, "{}", csv_row(&row));
            }
        }
        out
    }

    /// `RULE_GAPS.csv` — per-category detection coverage.
    #[must_use]
    pub fn rule_gaps_csv(&self) -> String {
        let mut out = String::from("category,hostile,detected,coverage,false_positives\n");
        for g in &self.gaps {
            let _ = writeln!(
                out,
                "{},{},{},{:.6},{}",
                g.category,
                g.hostile,
                g.detected,
                rate(g.detected, g.hostile),
                g.false_positives
            );
        }
        out
    }

    /// `RESOURCE_USAGE.csv` — one row per run (canvas metrics; the sandbox
    /// harness appends RSS/commit rows in VM runs).
    #[must_use]
    pub fn resource_usage_csv(&self) -> String {
        let mut out = String::from("phase,mean_latency_us,p95_us,p99_us,p999_us\n");
        let _ = writeln!(
            out,
            "scan,{},{},{},{},{}",
            self.latency.mean(),
            self.latency.percentile(50.0),
            self.latency.percentile(95.0),
            self.latency.percentile(99.0),
            self.latency.percentile(99.9),
        );
        out
    }

    /// `REGRESSIONS.csv` — empty when no baseline was supplied.
    #[must_use]
    pub fn regressions_csv(&self) -> String {
        let mut out = String::from("tag\n");
        if let Some(r) = &self.regression {
            for tag in r.classify() {
                let _ = writeln!(out, "{tag}");
            }
        }
        out
    }

    /// `CRASHES.csv` — placeholder schema; chaos runs append rows.
    #[must_use]
    pub fn crashes_csv(&self) -> String {
        String::from("event_seq,class,outcome,component\n")
    }

    /// `RECOVERY.csv` — the recovery verdict with the run id.
    #[must_use]
    pub fn recovery_csv(&self) -> String {
        format!(
            "run_id,verdict\n{},{}\n",
            csv_field(&self.run_id),
            self.recovery
        )
    }

    /// `REPORT.md` — the human-readable forensic report (MASTER §25).
    #[must_use]
    pub fn report_md(&self) -> String {
        let mut s = String::with_capacity(4096);
        let _ = writeln!(s, "# LitricOS Adversarial Benchmark — Run {}", self.run_id);
        let _ = writeln!(s);
        let _ = writeln!(
            s,
            "* Seed: {} · Corpus: {} samples · Recovery: {}",
            self.seed, self.corpus_size, self.recovery
        );
        let _ = writeln!(s);
        let _ = writeln!(s, "## Detection (by-construction ground truth)");
        let _ = writeln!(s);
        let _ = writeln!(s, "| metric | value |");
        let _ = writeln!(s, "|---|---|");
        let _ = writeln!(s, "| true positives | {} |", self.true_positives);
        let _ = writeln!(s, "| false negatives (misses) | {} |", self.false_negatives);
        let _ = writeln!(s, "| false positives | {} |", self.false_positives);
        let _ = writeln!(s, "| true negatives | {} |", self.true_negatives);
        let _ = writeln!(s, "| unsupported | {} |", self.unsupported);
        let _ = writeln!(
            s,
            "| environment limitations | {} |",
            self.environment_limitations
        );
        // The denominators are printed, not implied. Hostile samples that
        // never reached a classification (`unsupported` and the operational
        // outcomes) are left out of the numerator *and* the denominator, so
        // the headline is a rate over classified samples — printing the
        // population keeps a run that could classify very little from reading
        // as a clean sweep.
        let classified_hostile = self.true_positives.saturating_add(self.false_negatives);
        let _ = writeln!(
            s,
            "| detection rate (of {} classified hostile samples) | {:.2}% |",
            classified_hostile,
            100.0 * rate(self.true_positives, classified_hostile)
        );
        let classified_benign = self.false_positives.saturating_add(self.true_negatives);
        let _ = writeln!(
            s,
            "| false-positive rate (of {} classified benign samples) | {:.2}% |",
            classified_benign,
            100.0 * rate(self.false_positives, classified_benign)
        );
        let _ = writeln!(s);
        let _ = writeln!(s, "## Performance");
        let _ = writeln!(s);
        let _ = writeln!(s, "| latency | µs |");
        let _ = writeln!(s, "|---|---|");
        let _ = writeln!(s, "| mean | {} |", self.latency.mean());
        let _ = writeln!(s, "| P95 | {} |", self.latency.percentile(95.0));
        let _ = writeln!(s, "| P99 | {} |", self.latency.percentile(99.0));
        let _ = writeln!(s, "| P99.9 | {} |", self.latency.percentile(99.9));
        let _ = writeln!(
            s,
            "| throughput | {} samples/s |",
            self.latency.samples_per_sec()
        );
        let _ = writeln!(s);
        let _ = writeln!(s, "## Rule gaps (per category)");
        let _ = writeln!(s);
        let _ = writeln!(s, "| category | hostile | detected | coverage | FP |");
        let _ = writeln!(s, "|---|---|---|---|---|");
        for g in &self.gaps {
            let _ = writeln!(
                s,
                "| {} | {} | {} | {:.1}% | {} |",
                g.category,
                g.hostile,
                g.detected,
                100.0 * rate(g.detected, g.hostile),
                g.false_positives
            );
        }
        if let Some(r) = &self.regression {
            let _ = writeln!(s);
            let _ = writeln!(s, "## Regression vs baseline");
            let _ = writeln!(s);
            let tags = r.classify();
            if tags.is_empty() {
                let _ = writeln!(s, "No regression detected.");
            } else {
                for t in tags {
                    let _ = writeln!(s, "- {t}");
                }
            }
        }
        let _ = writeln!(s);
        let _ = writeln!(s, "## Recommended defensive changes");
        let _ = writeln!(s);
        for g in self.gaps.iter().filter(|g| g.detected < g.hostile) {
            let _ = writeln!(
                s,
                "- Add hash/pattern coverage for category `{}` ({} of {} hostile samples missed).",
                g.category,
                g.hostile - g.detected,
                g.hostile
            );
        }
        if self.false_positives > 0 {
            let _ = writeln!(s, "- Review hard-negative categories that produced false positives; benign shape families must not trip pattern tiers.");
        }
        s
    }

    /// `FORENSICS_MANIFEST.json` — artifact list with sizes filled by the
    /// CLI after writing.
    #[must_use]
    pub fn manifest_json(artifacts: &[(&str, u64)]) -> String {
        let mut s = String::from("{\n  \"artifacts\": [\n");
        for (i, (name, size)) in artifacts.iter().enumerate() {
            let comma = if i + 1 == artifacts.len() { "" } else { "," };
            let _ = writeln!(
                s,
                "    {{\"name\": {}, \"bytes\": {}}}{}",
                jstr(name),
                size,
                comma
            );
        }
        s.push_str("  ]\n}\n");
        s
    }
}

/// A minimal JSON string escape (ASCII control bytes + quote + backslash).
fn jstr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
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
    out.push('"');
    out
}

/// Safe ratio: 0.0 when the denominator is 0.
fn rate(num: u64, den: u64) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}

/// Count outcomes from results for direct scoreboard mirroring.
#[must_use]
pub fn count_environment_limitations(results: &[ScanResult]) -> u64 {
    results
        .iter()
        .filter(|r| r.outcome.as_str() == "ENVIRONMENT_LIMITATION")
        .count() as u64
}

/// Write every artifact of a run into `dir`. Returns the manifest.
/// The event log JSONL is written by the CLI (it owns the log).
pub fn write_all(run: &RunReport, log: &EventLog, dir: &std::path::Path) -> Result<u64, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut artifacts: Vec<(String, u64)> = Vec::new();
    let mut put = |name: &str, body: String| -> Result<(), String> {
        let p = dir.join(name);
        let n = u64::try_from(body.len()).unwrap_or(u64::MAX);
        std::fs::write(&p, body).map_err(|e| e.to_string())?;
        artifacts.push((name.to_owned(), n));
        Ok(())
    };
    put("RUN_SUMMARY.json", run.summary_json())?;
    put("RUN_SUMMARY.csv", run.summary_csv())?;
    put("DETECTIONS.csv", run.detections_csv())?;
    put("MISSES.csv", run.misses_csv())?;
    put("REGRESSIONS.csv", run.regressions_csv())?;
    put("RESOURCE_USAGE.csv", run.resource_usage_csv())?;
    put("CRASHES.csv", run.crashes_csv())?;
    put("RECOVERY.csv", run.recovery_csv())?;
    put("RULE_GAPS.csv", run.rule_gaps_csv())?;
    put("REPORT.md", run.report_md())?;
    put("EVENTS.jsonl", log.to_jsonl())?;
    let manifest = RunReport::manifest_json(
        &artifacts
            .iter()
            .map(|(n, b)| (n.as_str(), *b))
            .collect::<Vec<_>>(),
    );
    let manifest_len = u64::try_from(manifest.len()).unwrap_or(u64::MAX);
    std::fs::write(dir.join("FORENSICS_MANIFEST.json"), manifest).map_err(|e| e.to_string())?;
    Ok(manifest_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escaping_handles_commas_quotes_newlines() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn summary_json_is_parseable_shape() {
        let r = RunReport {
            run_id: "test-run".into(),
            seed: 42,
            corpus_size: 100,
            true_positives: 30,
            false_positives: 0,
            false_negatives: 5,
            true_negatives: 65,
            unsupported: 0,
            environment_limitations: 0,
            latency: LatencyProfile::default(),
            gaps: vec![],
            results: vec![],
            regression: None,
            recovery: "RECOVERED".into(),
        };
        let j = r.summary_json();
        assert!(j.starts_with('{') && j.trim_end().ends_with('}'));
        assert!(j.contains("\"detection_rate\": 0."));
        assert!(j.contains("\"run_id\": \"test-run\""));
    }

    #[test]
    fn report_md_contains_required_sections() {
        let r = RunReport {
            run_id: "md-run".into(),
            seed: 1,
            corpus_size: 10,
            true_positives: 8,
            false_positives: 1,
            false_negatives: 2,
            true_negatives: 7,
            unsupported: 0,
            environment_limitations: 0,
            latency: LatencyProfile::default(),
            gaps: vec![CategoryGap {
                category: "benign",
                hostile: 5,
                detected: 3,
                false_positives: 1,
            }],
            results: vec![],
            regression: None,
            recovery: "RECOVERED".into(),
        };
        let md = r.report_md();
        for section in [
            "Detection",
            "Performance",
            "Rule gaps",
            "Recommended defensive changes",
        ] {
            assert!(md.contains(section), "missing section {section}");
        }
        assert!(md.contains("benign"));
    }

    #[test]
    fn manifest_lists_all_artifacts() {
        let m = RunReport::manifest_json(&[("RUN_SUMMARY.json", 10), ("REPORT.md", 999)]);
        assert!(m.contains("\"RUN_SUMMARY.json\""));
        assert!(m.contains("999"));
        assert!(!m.trim_end().ends_with(','));
    }

    #[test]
    fn write_all_produces_every_file() {
        let tmp = std::env::temp_dir().join("litric-bench-report-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let log = EventLog::new("report-test");
        let r = RunReport {
            run_id: "write-test".into(),
            seed: 9,
            corpus_size: 3,
            true_positives: 2,
            false_positives: 0,
            false_negatives: 1,
            true_negatives: 0,
            unsupported: 0,
            environment_limitations: 0,
            latency: LatencyProfile::default(),
            gaps: vec![],
            results: vec![],
            regression: None,
            recovery: "RECOVERED".into(),
        };
        let manifest_len = write_all(&r, &log, &tmp).unwrap();
        assert!(manifest_len > 0);
        for f in [
            "RUN_SUMMARY.json",
            "RUN_SUMMARY.csv",
            "EVENTS.jsonl",
            "DETECTIONS.csv",
            "MISSES.csv",
            "REGRESSIONS.csv",
            "RESOURCE_USAGE.csv",
            "CRASHES.csv",
            "RECOVERY.csv",
            "RULE_GAPS.csv",
            "REPORT.md",
            "FORENSICS_MANIFEST.json",
        ] {
            assert!(tmp.join(f).exists(), "missing artifact {f}");
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
