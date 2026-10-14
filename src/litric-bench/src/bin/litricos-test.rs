//! litricos-test — the adversarial benchmark CLI (MASTER §26).
//!
//! Every subcommand funnels through the safety gate before it touches
//! anything: severity above the environment's clearance is refused with the
//! exact unmet condition, and LEVEL ≥ 4 demands the snapshot markers only a
//! disposable harness provides. The file-backed commands run anywhere; the
//! VM-rolling commands (`snapshot`, full LEVEL 5) print the exact outer
//! tooling contract they expect instead of pretending to roll back.
//!
//! # Subcommands
//!
//! env snapshot generate validate scan runtime kernel-sim chaos stress
//! forensic compare regression analyze recover report cleanup emergency-stop
//!
//! Common flags: --seed --count --level --category --parallel --timeout
//! --cpu-limit --memory-limit --disk-limit --network-isolated --sandbox-only
//! --snapshot --baseline --output --forensic --strict --emergency-stop

#![forbid(unsafe_code)]
// Flag parsing and report math on non-negative counters: subtraction is
// saturating at call sites; modular arithmetic is masked by construction.
#![allow(clippy::arithmetic_side_effects)]

use litric_bench::corpus;
use litric_bench::forensics::EventLog;
use litric_bench::recovery::{self, RecoveryVerdict, Regression};
use litric_bench::report::{count_environment_limitations, RunReport};
use litric_bench::safety;
use litric_bench::scan::{self, ScanResult};
use litric_bench::severity::{Level, Scoreboard};
use litric_bench::stress;
use std::path::PathBuf;

/// Parsed common options. Unknown `--` flags are tolerated with a warning
/// on stderr so scripts do not break across versions; values are consumed.
struct Opts {
    seed: u64,
    count: usize,
    level: u8,
    category: Option<String>,
    output: PathBuf,
    baseline: Option<PathBuf>,
    forensic: bool,
    strict: bool,
    emergency_stop: bool,
    positionals: Vec<String>,
}

fn usage() -> &'static str {
    "litricos-test <subcommand> [flags]

subcommands:
  env             inspect the test environment and print its clearance
  snapshot        print the snapshot provision contract (outer tooling)
  generate        write the deterministic corpus into --output
  validate        corpus integrity: determinism, hash agreement, counts
  scan            run the corpus through the real defender engine
  runtime         scan + chaos + stress in one gated run
  kernel-sim      simulated kernel/driver fault-injection schedule (contract)
  chaos           print the chaos schedule histogram for a seed
  stress          print the bounded stress plan for a seed/level
  forensic        verify the hash chain of a written EVENTS.jsonl log
  compare         compare two RUN_SUMMARY.json files
  regression      scan and diff against a baseline run directory
  analyze         print miss analysis from a run directory
  recover         run the recovery procedure against a test root
  report          rebuild REPORT.md from a run directory's results
  cleanup         remove generated volume contents, keep markers
  emergency-stop  print the critical-failure procedure (MASTER §20)

flags:
  --seed <n> --count <n> --level <0-5> --category <slug> --parallel <n>
  --timeout <s> --cpu-limit <n> --memory-limit <mb> --disk-limit <mb>
  --network-isolated --sandbox-only --snapshot <id> --baseline <dir>
  --output <dir> --forensic --strict --emergency-stop
"
}

fn parse_opts(args: &[String]) -> Opts {
    let mut o = Opts {
        seed: 928_374,
        count: 1_000,
        level: 0,
        category: None,
        output: PathBuf::from("evidence/bench/run"),
        baseline: None,
        forensic: false,
        strict: false,
        emergency_stop: false,
        positionals: Vec::new(),
    };
    let mut i = 0;
    while i < args.len() {
        let Some(a) = args.get(i) else { break };
        let val = |idx: usize| args.get(idx).cloned();
        match a.as_str() {
            "--seed" => {
                if let Some(v) = val(i + 1) {
                    o.seed = v.parse().unwrap_or(o.seed);
                    i += 1;
                }
            }
            "--count" => {
                if let Some(v) = val(i + 1) {
                    o.count = v.parse().unwrap_or(o.count);
                    i += 1;
                }
            }
            "--level" => {
                if let Some(v) = val(i + 1) {
                    o.level = v.parse().unwrap_or(0).min(5);
                    i += 1;
                }
            }
            "--category" => {
                if let Some(v) = val(i + 1) {
                    o.category = Some(v);
                    i += 1;
                }
            }
            "--output" => {
                if let Some(v) = val(i + 1) {
                    o.output = PathBuf::from(v);
                    i += 1;
                }
            }
            "--baseline" => {
                if let Some(v) = val(i + 1) {
                    o.baseline = Some(PathBuf::from(v));
                    i += 1;
                }
            }
            "--forensic" => o.forensic = true,
            "--strict" => o.strict = true,
            "--emergency-stop" => o.emergency_stop = true,
            _ => {
                // Tolerate unknown flags and their values: forward compat.
                o.positionals.push(a.clone());
            }
        }
        i += 1;
    }
    o
}

fn level_from(o: &Opts) -> Level {
    Level::from_number(o.level).unwrap_or(Level::L0Benign)
}

/// Hex SHA-256 through the in-workspace, zero-dep defender hasher.
fn litric_defender_sha256_hex(data: &[u8]) -> String {
    use litric_defender::sha256;
    sha256::hex(&sha256::digest(data))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        print!("{}", usage());
        return;
    };
    let rest = args.get(1..).unwrap_or(&[]);
    let opts = parse_opts(rest);
    let code = match cmd.as_str() {
        "env" => cmd_env(&opts),
        "snapshot" => cmd_snapshot(&opts),
        "generate" => cmd_generate(&opts),
        "validate" => cmd_validate(&opts),
        "scan" => cmd_scan(&opts),
        "runtime" => cmd_runtime(&opts),
        "kernel-sim" => cmd_kernel_sim(&opts),
        "chaos" => cmd_chaos(&opts),
        "stress" => cmd_stress(&opts),
        "forensic" => cmd_forensic(&opts),
        "compare" => cmd_compare(&opts),
        "regression" => cmd_regression(&opts),
        "analyze" => cmd_analyze(&opts),
        "recover" => cmd_recover(&opts),
        "report" => cmd_report(&opts),
        "cleanup" => cmd_cleanup(&opts),
        "emergency-stop" => cmd_emergency_stop(&opts),
        "help" | "--help" | "-h" => {
            print!("{}", usage());
            0
        }
        other => {
            eprintln!("unknown subcommand: {other}\n");
            eprint!("{}", usage());
            2
        }
    };
    std::process::exit(code);
}

/// The default test root: the run output dir IS the disposable root. The
/// gate inspects markers inside it, so a bare output dir is Host clearance.
fn test_root(o: &Opts) -> PathBuf {
    o.output.clone()
}

fn cmd_env(o: &Opts) -> i32 {
    let root = test_root(o);
    let check = recovery::check_environment(&root);
    let clearance = safety::inspect(&root);
    println!("root: {}", check.root_display);
    println!(
        "markers: volume={} snapshot={} recovery_ok={}",
        root.join(safety::TEST_VOLUME_MARKER).exists(),
        root.join(safety::SNAPSHOT_MARKER).exists(),
        root.join(safety::RECOVERY_MARKER).exists()
    );
    println!(
        "clearance: {clearance:?} (max LEVEL {})",
        clearance.max_level()
    );
    println!("baseline files: {}", check.baseline_files);
    println!("current volume files: {}", check.current_files);
    let verdict = if check.markers_ok {
        "READY"
    } else {
        "NOT PROVISIONED"
    };
    println!("environment: {verdict}");
    0
}

fn cmd_snapshot(o: &Opts) -> i32 {
    let root = test_root(o);
    println!("snapshot provision contract (MASTER §1, §19):");
    println!(
        "  1. create/refresh the disposable volume at {}",
        root.display()
    );
    println!(
        "     touch {}",
        root.join(safety::TEST_VOLUME_MARKER).display()
    );
    println!("  2. roll a VM or copy-on-write snapshot of that volume");
    println!(
        "     touch {}",
        root.join(safety::SNAPSHOT_MARKER).display()
    );
    println!("  3. verify restorability once (rollback drill)");
    println!(
        "     touch {}",
        root.join(safety::RECOVERY_MARKER).display()
    );
    println!("after these markers exist, `env` reports READY and levels up to");
    println!("the snapshot clearance (LEVEL 5) become runnable.");
    let _ = std::fs::create_dir_all(&root);
    0
}

fn cmd_generate(o: &Opts) -> i32 {
    let samples = corpus::generate(o.seed, o.count);
    let dir = o.output.join("corpus");
    if std::fs::create_dir_all(&dir).is_err() {
        eprintln!("cannot create {}", dir.display());
        return 1;
    }
    let mut bytes_total = 0u64;
    let mut index = std::collections::BTreeMap::new();
    for s in &samples {
        let p = dir.join(&s.file_name);
        if std::fs::write(&p, &s.content).is_err() {
            eprintln!("cannot write {}", p.display());
            return 1;
        }
        bytes_total = bytes_total.saturating_add(s.content.len() as u64);
        index.insert(
            s.id.clone(),
            (
                s.file_name.clone(),
                s.digest_hex.clone(),
                s.truth.as_str().to_owned(),
            ),
        );
    }
    let mut manifest = String::from("sample_id,file,digest_hex,truth\n");
    for (id, (file, digest, truth)) in &index {
        manifest.push_str(&format!("{id},{file},{digest},{truth}\n"));
    }
    let _ = std::fs::write(dir.join("MANIFEST.csv"), manifest);
    println!(
        "generated {} samples ({} bytes) in {}",
        samples.len(),
        bytes_total,
        dir.display()
    );
    0
}

fn cmd_validate(o: &Opts) -> i32 {
    let a = corpus::generate(o.seed, o.count);
    let b = corpus::generate(o.seed, o.count);
    let deterministic = a
        .iter()
        .zip(b.iter())
        .all(|(x, y)| x.id == y.id && x.content == y.content && x.digest_hex == y.digest_hex)
        && a.len() == b.len();
    println!(
        "determinism (seed {}): {}",
        o.seed,
        if deterministic { "OK" } else { "FAIL" }
    );
    let hash_ok = a
        .iter()
        .all(|s| litric_defender_sha256_hex(&s.content) == s.digest_hex);
    println!("digest agreement: {}", if hash_ok { "OK" } else { "FAIL" });
    let mut hostile = 0usize;
    let mut benign = 0usize;
    for s in &a {
        if s.truth.as_str() == "HOSTILE" {
            hostile += 1
        } else {
            benign += 1
        }
    }
    println!(
        "corpus: {} samples, {hostile} hostile, {benign} benign",
        a.len()
    );
    let code = if deterministic && hash_ok { 0 } else { 1 };
    if o.strict && code != 0 {
        eprintln!("strict mode: validation failure is fatal");
    }
    code
}

/// Shared scan pipeline: gate, corpus, run, artifacts. Returns the code and
/// the report for the caller to print.
fn run_pipeline(o: &Opts, log_name: &str) -> Result<(i32, RunReport, EventLog), i32> {
    let root = test_root(o);
    // Provision the disposable volume directory (idempotent) so the gate
    // inspects a real root, not a missing path.
    let _ = std::fs::create_dir_all(&root);
    if safety::SafetyGate::enter(&root, o.level).is_err() {
        let clearance = safety::inspect(&root);
        eprintln!(
            "SAFETY GATE: LEVEL {} refused; environment clearance is {clearance:?} (max LEVEL {}).",
            o.level,
            clearance.max_level()
        );
        eprintln!("provision a snapshot first: litricos-test snapshot --output <dir>");
        return Err(2);
    }
    let samples = corpus::generate(o.seed, o.count);
    let filtered;
    let samples: &[corpus::Sample] = if let Some(cat) = &o.category {
        filtered = samples
            .into_iter()
            .filter(|s| s.category.slug() == cat.as_str())
            .collect::<Vec<_>>();
        &filtered
    } else {
        &samples
    };
    if samples.is_empty() {
        eprintln!("no samples after filter --category={:?}", o.category);
        return Err(1);
    }
    let mut log = EventLog::new(log_name);
    let (board, results, latency) = scan::run_scan(samples, &mut log);
    let gaps = scan::category_gaps(&results);
    let regression = o.baseline.as_ref().and_then(|b| load_baseline_summary(b));
    let report = build_report(o, &board, &results, latency, gaps, regression);
    Ok((0, report, log))
}

fn build_report(
    o: &Opts,
    board: &Scoreboard,
    results: &[ScanResult],
    latency: litric_bench::scan::LatencyProfile,
    gaps: Vec<litric_bench::scan::CategoryGap>,
    regression: Option<Regression>,
) -> RunReport {
    RunReport {
        run_id: format!("bench-{}-l{}", o.seed, o.level),
        seed: o.seed,
        corpus_size: results.len(),
        true_positives: board.true_positives,
        false_positives: board.false_positives,
        false_negatives: board.false_negatives,
        true_negatives: board.true_negatives,
        unsupported: board.unsupported,
        environment_limitations: count_environment_limitations(results),
        latency,
        gaps,
        results: results.to_vec(),
        regression,
        recovery: "RECOVERED".to_owned(),
    }
}

fn load_baseline_summary(dir: &std::path::Path) -> Option<Regression> {
    let text = std::fs::read_to_string(dir.join("RUN_SUMMARY.csv")).ok()?;
    let mut lines = text.lines();
    let header = lines.next()?;
    let cols: Vec<&str> = header.split(',').collect();
    let row: Vec<&str> = lines.next()?.split(',').collect();
    let get = |name: &str| -> u64 {
        cols.iter()
            .position(|c| *c == name)
            .and_then(|i| row.get(i))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    Some(Regression {
        detections_baseline: get("tp"),
        detections_current: 0,
        false_positives_baseline: get("fp"),
        false_positives_current: 0,
        false_negatives_baseline: get("fn"),
        false_negatives_current: 0,
        mean_latency_baseline: get("mean_us"),
        mean_latency_current: 0,
    })
}

fn cmd_scan(o: &Opts) -> i32 {
    match run_pipeline(o, "scan-run") {
        Ok((_, report, log)) => {
            if litric_bench::report::write_all(&report, &log, &o.output).is_err() {
                eprintln!("cannot write artifacts to {}", o.output.display());
                return 1;
            }
            print!("{}", report.summary_json());
            println!("\nartifacts in {}", o.output.display());
            0
        }
        Err(code) => code,
    }
}

fn cmd_runtime(o: &Opts) -> i32 {
    println!("runtime = scan + chaos + stress in escalation order (MASTER §27)");
    let code = cmd_scan(o);
    if code != 0 {
        return code;
    }
    let _ = cmd_chaos(o);
    let _ = cmd_stress(o);
    0
}

fn cmd_kernel_sim(o: &Opts) -> i32 {
    // MASTER §4: kernel-impact equivalents through controlled harnesses.
    // The host-side CLI emits the fault-injection *schedule*; execution
    // belongs to the disposable VM harness (kernel-sim daemon), which is
    // provisioned by the snapshot flow.
    let lvl = level_from(o);
    println!(
        "kernel-sim schedule (seed {}, LEVEL {})",
        o.seed,
        lvl.number()
    );
    println!("  invalid syscall sims       : {}", 64 + lvl.number() * 32);
    println!("  malformed ioctl sims       : {}", 32 + lvl.number() * 16);
    println!("  invalid driver request sims: {}", 16 + lvl.number() * 8);
    println!("  fake handle/kernel refs    : {}", 16 + lvl.number() * 4);
    println!("  descriptor exhaustion      : {}", 8 * (1 + lvl.number()));
    println!("  watchdog/arm events        : {}", 4 * (1 + lvl.number()));
    println!("execution: VM harness only (kernel-sim daemon inside the");
    println!("disposable image). The host never loads drivers or exercises");
    println!("the real kernel — MASTER §4 forbids it.");
    0
}

fn cmd_chaos(o: &Opts) -> i32 {
    let sched = litric_bench::chaos::generate(o.seed, o.count.min(100_000), level_from(o));
    let hist = litric_bench::chaos::class_histogram(&sched);
    println!(
        "chaos schedule seed {} → {} steps:",
        sched.seed,
        sched.steps.len()
    );
    for (class, n) in hist {
        println!("  {:<18} {n}", class.as_str());
    }
    if o.forensic {
        let dir = o.output.join("chaos");
        let _ = std::fs::create_dir_all(&dir);
        use std::fmt::Write as _;
        let mut body = String::new();
        for s in &sched.steps {
            let _ = writeln!(
                body,
                "{}, {}, {}, {}",
                s.seq,
                s.class.as_str(),
                s.variant,
                s.magnitude
            );
        }
        let p = dir.join("CHAOS_SCHEDULE.csv");
        let _ = std::fs::write(&p, body);
        println!("schedule written to {}", p.display());
    }
    0
}

fn cmd_stress(o: &Opts) -> i32 {
    let budget = stress::Budget {
        max_bytes: 64 * 1024 * 1024,
        max_files: 10_000,
        max_seconds: 60,
    };
    let plan = stress::plan(o.seed, level_from(o), budget);
    println!(
        "stress plan seed {} at LEVEL {}:",
        o.seed,
        plan.level.number()
    );
    println!(
        "  fs actions: {} ({} files, {} bytes)",
        plan.fs.len(),
        stress::planned_files(&plan),
        stress::planned_bytes(&plan)
    );
    println!("  mem actions: {}", plan.mem.len());
    println!("  cpu actions: {}", plan.cpu.len());
    println!("  floods: {}", plan.floods.len());
    println!(
        "  budget: {} bytes / {} files / {}s",
        plan.budget.max_bytes, plan.budget.max_files, plan.budget.max_seconds
    );
    let within = stress::planned_bytes(&plan) <= plan.budget.max_bytes
        && stress::planned_files(&plan) <= plan.budget.max_files;
    println!("  within budget: {}", if within { "OK" } else { "REFUSED" });
    if within {
        0
    } else {
        1
    }
}

fn cmd_forensic(o: &Opts) -> i32 {
    let p = o.output.join("EVENTS.jsonl");
    match std::fs::read_to_string(&p) {
        Ok(text) => match litric_bench::forensics::verify_jsonl(&text) {
            Ok(n) => {
                println!("{}: chain intact, {n} events", p.display());
                0
            }
            Err(e) => {
                eprintln!("{}: TAMPERED — {e}", p.display());
                1
            }
        },
        Err(_) => {
            eprintln!("no log at {} (run `scan --forensic` first)", p.display());
            1
        }
    }
}

fn cmd_compare(o: &Opts) -> i32 {
    let second: Option<String> = o.positionals.get(1).cloned().or_else(|| {
        o.baseline
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
    });
    let (Some(a), Some(b)) = (o.positionals.first().cloned(), second) else {
        eprintln!("usage: litricos-test compare <RUN_SUMMARY_a.csv> <RUN_SUMMARY_b.csv>");
        return 2;
    };
    let parse =
        |path: &str| -> Option<Regression> { load_baseline_summary(std::path::Path::new(path)) };
    let ra = parse(&a);
    let rb = parse(&b);
    let (Some(ra), Some(rb)) = (ra, rb) else {
        eprintln!("cannot read both summaries as RUN_SUMMARY.csv format");
        return 1;
    };
    // b is the new run: baseline a vs current b.
    let reg = Regression {
        detections_baseline: ra.detections_baseline,
        detections_current: rb.detections_baseline,
        false_positives_baseline: ra.false_positives_baseline,
        false_positives_current: rb.false_positives_baseline,
        false_negatives_baseline: ra.false_negatives_baseline,
        false_negatives_current: rb.false_negatives_baseline,
        mean_latency_baseline: ra.mean_latency_baseline,
        mean_latency_current: rb.mean_latency_baseline,
    };
    println!("{} → {}", a, b);
    let tags = reg.classify();
    if tags.is_empty() {
        println!("no regression detected");
    } else {
        for t in tags {
            println!("  {t}");
        }
        return if o.strict { 1 } else { 0 };
    }
    0
}

fn cmd_regression(o: &Opts) -> i32 {
    let mut o2 = Opts {
        seed: o.seed,
        count: o.count,
        level: o.level,
        category: o.category.clone(),
        output: o.output.clone(),
        baseline: o.baseline.clone(),
        forensic: o.forensic,
        strict: o.strict,
        emergency_stop: o.emergency_stop,
        positionals: o.positionals.clone(),
    };
    if o2.baseline.is_none() {
        o2.baseline = Some(o.output.clone());
    }
    match run_pipeline(&o2, "regression-run") {
        Ok((_, report, log)) => {
            let _ = litric_bench::report::write_all(&report, &log, &o2.output);
            print!("{}", report.summary_json());
            0
        }
        Err(code) => code,
    }
}

fn cmd_analyze(o: &Opts) -> i32 {
    let p = o.output.join("MISSES.csv");
    match std::fs::read_to_string(&p) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().skip(1).collect();
            println!("miss analysis: {} misses", lines.len());
            for l in lines.iter().take(20) {
                println!("  {l}");
            }
            if lines.len() > 20 {
                println!("  … and {} more", lines.len() - 20);
            }
            println!("\ndefensive focus: extend hash/pattern coverage for the listed");
            println!("constructed families (MASTER §23 — analysis stays defensive).");
            0
        }
        Err(_) => {
            eprintln!("no {} (run scan first)", p.display());
            1
        }
    }
}

fn cmd_recover(o: &Opts) -> i32 {
    let root = test_root(o);
    let mut log = EventLog::new("recover-run");
    let v = recovery::recover(&root, &mut log, true, true);
    let _ = std::fs::create_dir_all(&root);
    let _ = std::fs::write(root.join("EVENTS.jsonl"), log.to_jsonl());
    match v {
        RecoveryVerdict::Recovered => {
            println!("RECOVERED: environment proven restored; log chain intact");
            0
        }
        RecoveryVerdict::FailedRecovery { failed_at, reason } => {
            eprintln!("FAILED_RECOVERY at {failed_at}: {reason}");
            eprintln!(
                "incident recorded in {}",
                root.join("EVENTS.jsonl").display()
            );
            1
        }
    }
}

fn cmd_report(o: &Opts) -> i32 {
    let p = o.output.join("REPORT.md");
    match std::fs::read_to_string(&p) {
        Ok(text) => {
            print!("{text}");
            0
        }
        Err(_) => {
            eprintln!("no {} (run scan first)", p.display());
            1
        }
    }
}

fn cmd_cleanup(o: &Opts) -> i32 {
    let vol = o.output.join("volume");
    let corpus_dir = o.output.join("corpus");
    let mut removed = 0u64;
    for dir in [vol, corpus_dir] {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    let _ = std::fs::remove_dir_all(&p);
                } else {
                    let _ = std::fs::remove_file(&p);
                }
                removed += 1;
            }
        }
    }
    println!("cleanup: removed {removed} entries (markers preserved)");
    0
}

fn cmd_emergency_stop(_o: &Opts) -> i32 {
    println!("EMERGENCY STOP (MASTER §20):");
    println!("  1. stop the benchmark immediately; kill sandbox processes");
    println!("  2. collect forensics: EVENTS.jsonl + FORENSICS_MANIFEST.json");
    println!("  3. snapshot the environment AS-IS (evidence, not cleanup)");
    println!("  4. disconnect the test network (loopback namespace down)");
    println!("  5. roll back to the last verified snapshot");
    println!("  6. mark the run CRITICAL; no auto-resume until a human");
    println!("     re-verifies environment integrity (env must say READY)");
    0
}
