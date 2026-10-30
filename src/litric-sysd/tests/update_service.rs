// A test asserts by panicking, and says which step broke with `expect`. The
// workspace denies those lints for the shipping code, where a panic is an
// availability bug; a test that cannot panic cannot fail.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

//! The updater service through the real process and socket.
//!
//! `update.rs` proves the service logic in-process. This file proves the path
//! the system actually takes: `litric-sysd --boot` counts an attempt, the
//! socket answers `UPDATE STATUS`, and `UPDATE COMMIT`/`UPDATE ROLLBACK` reach
//! the same engine root. It also proves that a daemon whose grant record was
//! loaded without update authority cannot mutate anything, and that a corrupt
//! generation record is visible instead of taking the daemon down.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use litric_core::Version;
use litric_update::Engine;

fn scratch(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "litric-sysd-update-e2e-{tag}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn init_and_stage(root: &Path) -> PathBuf {
    Engine::init(root, Version::parse("1.0.0").expect("version")).expect("init generation state");
    let source = root.join("incoming/release");
    fs::create_dir_all(&source).expect("create incoming");
    fs::write(source.join("app"), "candidate tree").expect("write candidate");
    let mut engine = Engine::open(root).expect("open generation state");
    engine
        .stage(&source, Version::parse("1.1.0").expect("version"))
        .expect("stage candidate");
    source
}

fn boot_command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_litric-sysd"));
    command
        .arg("--boot")
        .arg("--update-root")
        .arg(root)
        .env("TZ", "UTC");
    command
}

fn run_boot_output(root: &Path) -> Output {
    boot_command(root)
        .output()
        .expect("the boot entrypoint runs")
}

fn run_boot(root: &Path) -> String {
    let out = run_boot_output(root);
    assert!(
        out.status.success(),
        "boot entrypoint failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn lock_body(pid: u32) -> String {
    format!("litric-update-lock\t1\npid\t{pid}\nat\t1\nreason\tstage\n")
}

fn health_path(root: &Path) -> PathBuf {
    root.join("system/health")
}

fn health_body(slot: &str, version: &str, attempt: u8, verdict: &str) -> String {
    format!(
        "litric-update-health\t1\ngeneration\t{slot}\nversion\t{version}\nattempt\t{attempt}\nat\t1\nverdict\t{verdict}\nchecks\troot,state,marker,serving\n"
    )
}

fn query_command(socket: &Path, request: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_litric-sysd"));
    command
        .arg("--query")
        .arg(request)
        .arg("--socket")
        .arg(socket)
        .env("TZ", "UTC");
    command
}

struct Daemon {
    child: Child,
    socket: PathBuf,
}

impl Daemon {
    fn start(root: &Path, tag: &str) -> Self {
        let socket = std::env::temp_dir().join(format!(
            "litric-sysd-update-e2e-{tag}-{}.sock",
            std::process::id()
        ));
        let _ = fs::remove_file(&socket);
        let log_path = root.join("daemon.log");
        let log = fs::File::create(&log_path).expect("daemon log");
        let child = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
            .arg("--serve")
            .arg("--socket")
            .arg(&socket)
            .arg("--home")
            .arg(root)
            .arg("--root")
            .arg(root)
            .arg("--state")
            .arg(root.join("var/lib/litric-sysd/grants.conf"))
            .arg("--update-root")
            .arg(root)
            .env("TZ", "UTC")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::from(log))
            .spawn()
            .expect("the daemon binary starts");
        let mut daemon = Self { child, socket };
        let deadline = Instant::now() + Duration::from_secs(5);
        while !daemon.socket.exists() {
            if daemon
                .child
                .try_wait()
                .expect("the daemon's status is readable")
                .is_some()
                || Instant::now() >= deadline
            {
                let log = fs::read_to_string(&log_path).unwrap_or_default();
                panic!("the daemon did not bind {}: {log}", daemon.socket.display());
            }
            sleep(Duration::from_millis(5));
        }
        daemon
    }

    fn ask(&self, request: &str) -> (bool, String) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let out = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
                .arg("--query")
                .arg(request)
                .arg("--socket")
                .arg(&self.socket)
                .env("TZ", "UTC")
                .output()
                .expect("the client runs");
            let reply = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            // A transport failure is the normal first few milliseconds after
            // bind; an `ERR` line is the daemon's answer and must be returned.
            if !reply.starts_with("litric-sysd: ") || Instant::now() >= deadline {
                return (out.status.success(), reply);
            }
            sleep(Duration::from_millis(5));
        }
    }

    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

fn line<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.lines().find(|line| line.starts_with(prefix))
}

fn wait_for_healthy(daemon: &Daemon) -> String {
    for _ in 0..100 {
        let (ok, status) = daemon.ask("UPDATE STATUS 3");
        if ok && line(&status, "health_evidence=") == Some("health_evidence=healthy") {
            return status;
        }
        sleep(Duration::from_millis(5));
    }
    panic!("the serve loop did not attest healthy health evidence");
}

#[test]
fn boot_status_commit_and_rollback_use_the_real_socket_and_keep_user_data() {
    let root = scratch("commit");
    init_and_stage(&root);
    let user = root.join("user/notes.txt");
    fs::create_dir_all(user.parent().expect("parent")).expect("user dir");
    fs::write(&user, "the user's work").expect("user data");

    let boot = run_boot(&root);
    assert_eq!(
        line(&boot, "litric.update.outcome="),
        Some("litric.update.outcome=booting")
    );
    assert_eq!(
        line(&boot, "litric.update.attempt="),
        Some("litric.update.attempt=1")
    );

    let daemon = Daemon::start(&root, "commit");
    let (ok, status) = daemon.ask("UPDATE STATUS");
    assert!(ok, "{status}");
    assert_eq!(line(&status, "schema="), Some("schema=3"));
    assert_eq!(line(&status, "state="), Some("state=ready"));
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(line(&status, "candidate_slot="), Some("candidate_slot=B"));
    assert_eq!(
        line(&status, "candidate_attempts="),
        Some("candidate_attempts=1")
    );
    assert_eq!(
        line(&status, "health_evidence="),
        Some("health_evidence=healthy")
    );
    assert_eq!(line(&status, "health_slot="), Some("health_slot=B"));
    assert_eq!(line(&status, "health_attempt="), Some("health_attempt=1"));
    assert_eq!(line(&status, "health_matches="), Some("health_matches=1"));

    // A schema-2 client asks explicitly and gets the lock fields' schema. A
    // schema-1 client (the bare verb above) gets the same additive answer. A
    // newer schema is refused, rather than served fields it did not ask for.
    let (ok, schema_two) = daemon.ask("UPDATE STATUS 2");
    assert!(ok, "{schema_two}");
    assert_eq!(
        line(&schema_two, "requested_schema="),
        Some("requested_schema=2")
    );
    let (ok, newer) = daemon.ask("UPDATE STATUS 99");
    assert!(!ok, "schema 99 must be refused: {newer}");
    assert!(newer.starts_with("ERR schema-unsupported"), "{newer}");

    let (ok, commit) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(ok, "{commit}");
    assert_eq!(line(&commit, "event="), Some("event=commit"));
    let (_, status) = daemon.ask("UPDATE STATUS");
    assert_eq!(line(&status, "active_slot="), Some("active_slot=B"));

    let (ok, rollback) = daemon.ask("UPDATE ROLLBACK CONFIRM");
    assert!(ok, "{rollback}");
    assert_eq!(line(&rollback, "event="), Some("event=rollback"));
    assert_eq!(line(&rollback, "reason="), Some("reason=manual"));
    let (_, status) = daemon.ask("UPDATE STATUS");
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(
        line(&status, "last_rollback_reason="),
        Some("last_rollback_reason=manual")
    );
    assert_eq!(
        fs::read_to_string(&user).expect("user data survives"),
        "the user's work"
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn four_boot_requests_reject_the_candidate_and_leave_the_active_generation() {
    let root = scratch("budget");
    init_and_stage(&root);
    let user = root.join("user/notes.txt");
    fs::create_dir_all(user.parent().expect("parent")).expect("user dir");
    fs::write(&user, "keep me").expect("user data");

    for _ in 0..3 {
        let boot = run_boot(&root);
        assert_eq!(
            line(&boot, "litric.update.outcome="),
            Some("litric.update.outcome=booting")
        );
    }
    let boot = run_boot(&root);
    assert_eq!(
        line(&boot, "litric.update.outcome="),
        Some("litric.update.outcome=rejected-budget")
    );
    assert_eq!(
        line(&boot, "litric.update.rollback="),
        Some("litric.update.rollback=active-generation")
    );

    let daemon = Daemon::start(&root, "budget");
    let (ok, status) = daemon.ask("UPDATE STATUS");
    assert!(ok, "{status}");
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(
        line(&status, "candidate_slot="),
        Some("candidate_slot=none")
    );
    assert_eq!(
        line(&status, "last_result="),
        Some("last_result=reject:boot-count")
    );
    assert_eq!(fs::read_to_string(&user).expect("user data"), "keep me");
    daemon.stop();
    cleanup(&root);
}

#[test]
fn an_empty_grant_record_cannot_mutate_update_state_over_the_socket() {
    let root = scratch("unauthorised");
    let grants = root.join("var/lib/litric-sysd/grants.conf");
    fs::create_dir_all(grants.parent().expect("parent")).expect("grant dir");
    fs::write(&grants, "litric-grants\t1\n").expect("header-only grant record");
    // A valid update root, so the refusal can only be the missing authority.
    init_and_stage(&root);

    let daemon = Daemon::start(&root, "unauthorised");
    let (ok, reply) = daemon.ask("UPDATE ROLLBACK CONFIRM");
    assert!(!ok, "a privileged action must be refused: {reply}");
    assert!(
        reply.starts_with("ERR refused"),
        "the refusal must come from the broker: {reply}"
    );
    assert!(
        reply.contains("no-grant") || reply.contains("unknown-application"),
        "the broker must name the missing authority: {reply}"
    );
    let (_, status) = daemon.ask("UPDATE STATUS");
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    daemon.stop();
    cleanup(&root);
}

#[test]
fn a_corrupt_generation_record_is_reported_and_does_not_stop_the_daemon() {
    let root = scratch("corrupt");
    fs::create_dir_all(root.join("system")).expect("state dir");
    fs::write(root.join("system/state"), "not a generation record\n").expect("damage");

    let daemon = Daemon::start(&root, "corrupt");
    let (ok, status) = daemon.ask("UPDATE STATUS");
    assert!(
        ok,
        "status must answer even when the state is damaged: {status}"
    );
    assert_eq!(line(&status, "state="), Some("state=corrupt"));
    assert!(
        line(&status, "reason=").is_some(),
        "the damage must be named: {status}"
    );
    let (ok, action) = daemon.ask("UPDATE ROLLBACK CONFIRM");
    assert!(!ok, "a mutating action must fail: {action}");
    assert!(action.starts_with("ERR invalid-state"), "{action}");
    daemon.stop();
    cleanup(&root);
}

#[test]
fn two_concurrent_boot_processes_each_count_one_attempt() {
    let root = scratch("concurrent-boot");
    init_and_stage(&root);

    let first = boot_command(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("first boot process");
    let second = boot_command(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("second boot process");
    let first = first.wait_with_output().expect("first boot result");
    let second = second.wait_with_output().expect("second boot result");
    assert!(
        first.status.success(),
        "first boot failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "second boot failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    for out in [&first, &second] {
        let text = String::from_utf8_lossy(&out.stdout);
        assert_eq!(
            line(&text, "litric.update.outcome="),
            Some("litric.update.outcome=booting"),
            "{text}"
        );
    }

    let daemon = Daemon::start(&root, "concurrent-boot");
    let (ok, status) = daemon.ask("UPDATE STATUS 2");
    assert!(ok, "{status}");
    assert_eq!(
        line(&status, "candidate_attempts="),
        Some("candidate_attempts=2"),
        "two serialised boots must both increment: {status}"
    );
    assert_eq!(line(&status, "state="), Some("state=ready"));
    daemon.stop();
    cleanup(&root);
}

#[test]
fn a_live_writer_makes_status_busy_and_a_second_writer_fail_typed() {
    let root = scratch("locked");
    init_and_stage(&root);
    fs::write(root.join("update.lock"), lock_body(std::process::id())).expect("held lock");

    let daemon = Daemon::start(&root, "locked");
    let started = Instant::now();
    let (ok, status) = daemon.ask("UPDATE STATUS 2");
    let elapsed = started.elapsed();
    assert!(ok, "status must answer while locked: {status}");
    assert!(
        elapsed < Duration::from_secs(1),
        "status must not block on the writer: {elapsed:?}"
    );
    assert_eq!(line(&status, "state="), Some("state=busy"), "{status}");
    assert_eq!(
        line(&status, "locked_pid="),
        Some(format!("locked_pid={}", std::process::id()).as_str()),
        "{status}"
    );
    assert_eq!(line(&status, "locked_reason="), Some("locked_reason=stage"));
    assert_eq!(line(&status, "retry="), Some("retry=lock"));

    let out = run_boot_output(&root);
    assert!(
        !out.status.success(),
        "a held lock must fail the boot writer"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("locked by pid"),
        "the failure must be the documented typed error: {stderr}"
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn a_stale_dead_pid_lock_is_reclaimed_by_the_next_boot() {
    let root = scratch("stale-lock");
    init_and_stage(&root);
    fs::write(root.join("update.lock"), lock_body(u32::MAX - 1)).expect("stale lock");

    let boot = run_boot(&root);
    assert_eq!(
        line(&boot, "litric.update.attempt="),
        Some("litric.update.attempt=1"),
        "the boot must run after reclaiming the dead owner: {boot}"
    );
    assert!(
        !root.join("update.lock").exists(),
        "the reclaimed lock must be released"
    );
    cleanup(&root);
}

#[test]
fn interrupted_writes_are_ignored_or_degrade_without_a_panic() {
    let root = scratch("interrupted");
    init_and_stage(&root);
    let state = root.join("system/state");

    // A sibling temporary is not state. The live record still reads as ready.
    fs::write(root.join("system/.state.tmp-999"), "half-written\n").expect("leftover temp");
    let daemon = Daemon::start(&root, "interrupted");
    let (ok, status) = daemon.ask("UPDATE STATUS 2");
    assert!(ok, "{status}");
    assert_eq!(line(&status, "state="), Some("state=ready"), "{status}");

    // An interrupted rename leaves the old file or a truncated one; the latter
    // must be a typed corrupt result, not a panic and not a guessed state.
    fs::write(&state, "litric-generations\t1\n").expect("truncated state");
    let (ok, truncated) = daemon.ask("UPDATE STATUS 2");
    assert!(ok, "a truncated record must still answer: {truncated}");
    assert_eq!(line(&truncated, "state="), Some("state=corrupt"));

    // Zero length is the common result of a create that never wrote.
    fs::write(&state, "").expect("empty state");
    let (ok, empty) = daemon.ask("UPDATE STATUS 2");
    assert!(ok, "an empty record must still answer: {empty}");
    assert_eq!(line(&empty, "state="), Some("state=corrupt"));

    // The boot entrypoint also degrades rather than aborting the machine.
    let boot = run_boot(&root);
    assert_eq!(
        line(&boot, "litric.update.state="),
        Some("litric.update.state=corrupt")
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn commit_without_a_boot_attempt_is_refused_before_any_state_change() {
    let root = scratch("health-not-booted");
    init_and_stage(&root);
    let daemon = Daemon::start(&root, "health-not-booted");
    let (ok, status) = daemon.ask("UPDATE STATUS 3");
    assert!(ok, "{status}");
    assert_eq!(
        line(&status, "health_evidence="),
        Some("health_evidence=absent")
    );
    let before = fs::read(root.join("system/state")).expect("state before");
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "no boot, no commit: {reply}");
    assert!(reply.starts_with("ERR health-not-booted"), "{reply}");
    assert_eq!(
        fs::read(root.join("system/state")).expect("state after"),
        before
    );
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(line(&status, "candidate_slot="), Some("candidate_slot=B"));
    daemon.stop();
    cleanup(&root);
}

#[test]
fn missing_evidence_after_a_boot_is_refused_and_the_temporary_is_not_evidence() {
    let root = scratch("health-missing");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-missing");
    let healthy = wait_for_healthy(&daemon);
    assert_eq!(
        line(&healthy, "health_evidence="),
        Some("health_evidence=healthy")
    );
    // Move the live record aside as if the atomic write had not published it.
    fs::rename(health_path(&root), root.join("system/.health.tmp-999"))
        .expect("make health missing");
    let before = fs::read(root.join("system/state")).expect("state before");
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-missing"), "{reply}");
    assert_eq!(
        fs::read(root.join("system/state")).expect("state after"),
        before
    );
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(
        line(&status, "health_evidence="),
        Some("health_evidence=absent")
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn evidence_for_the_other_generation_is_stale_and_cannot_commit() {
    let root = scratch("health-stale-generation");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-stale-generation");
    wait_for_healthy(&daemon);
    fs::write(health_path(&root), health_body("A", "1.1.0", 1, "healthy"))
        .expect("stale generation evidence");
    let before = fs::read(root.join("system/state")).expect("state before");
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-stale"), "{reply}");
    assert!(reply.contains("slot A"), "{reply}");
    assert_eq!(
        fs::read(root.join("system/state")).expect("state after"),
        before
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn evidence_for_another_attempt_is_stale_and_cannot_commit() {
    let root = scratch("health-stale-attempt");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-stale-attempt");
    wait_for_healthy(&daemon);
    fs::write(health_path(&root), health_body("B", "1.1.0", 2, "healthy"))
        .expect("stale attempt evidence");
    let before = fs::read(root.join("system/state")).expect("state before");
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-stale"), "{reply}");
    assert!(reply.contains("attempt 1"), "{reply}");
    assert_eq!(
        fs::read(root.join("system/state")).expect("state after"),
        before
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn failed_evidence_refuses_commit_and_the_next_boot_rejects_the_candidate() {
    let root = scratch("health-failed");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-failed");
    wait_for_healthy(&daemon);
    fs::write(health_path(&root), health_body("B", "1.1.0", 1, "failed")).expect("failed evidence");
    let before = fs::read(root.join("system/state")).expect("state before");
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-failed"), "{reply}");
    assert_eq!(
        fs::read(root.join("system/state")).expect("state after"),
        before
    );

    let boot = run_boot(&root);
    assert_eq!(
        line(&boot, "litric.update.outcome="),
        Some("litric.update.outcome=rejected-health"),
        "{boot}"
    );
    assert_eq!(
        line(&boot, "litric.update.rollback="),
        Some("litric.update.rollback=active-generation")
    );
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(
        line(&status, "candidate_slot="),
        Some("candidate_slot=none")
    );
    assert_eq!(
        line(&status, "last_result="),
        Some("last_result=reject:health-check")
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn truncated_zero_and_temp_only_health_evidence_degrade_without_a_panic() {
    let root = scratch("health-corrupt");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-corrupt");
    wait_for_healthy(&daemon);
    let state = root.join("system/state");
    let before = fs::read(&state).expect("state before");

    fs::write(health_path(&root), "").expect("zero-length health");
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(
        line(&status, "health_evidence="),
        Some("health_evidence=invalid")
    );
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-invalid"), "{reply}");
    assert_eq!(fs::read(&state).expect("state after zero"), before);

    fs::write(health_path(&root), "litric-update-health\t1\n").expect("truncated health");
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(
        line(&status, "health_evidence="),
        Some("health_evidence=invalid")
    );
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-invalid"), "{reply}");
    assert_eq!(fs::read(&state).expect("state after truncated"), before);

    fs::rename(health_path(&root), root.join("system/.health.tmp-999"))
        .expect("leave only a temporary");
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(
        line(&status, "health_evidence="),
        Some("health_evidence=absent")
    );
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-missing"), "{reply}");
    assert_eq!(fs::read(&state).expect("state after temp-only"), before);
    daemon.stop();
    cleanup(&root);
}

#[test]
fn a_prior_attempts_healthy_evidence_cannot_be_replayed_for_a_new_attempt() {
    let root = scratch("health-replay");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-replay");
    wait_for_healthy(&daemon);
    let healthy = fs::read(health_path(&root)).expect("healthy evidence");
    let boot = run_boot(&root);
    assert_eq!(
        line(&boot, "litric.update.attempt="),
        Some("litric.update.attempt=2")
    );
    fs::write(health_path(&root), healthy).expect("replay attempt 1");
    let before = fs::read(root.join("system/state")).expect("state before");
    let (ok, reply) = daemon.ask("UPDATE COMMIT PASSED CONFIRM");
    assert!(!ok, "{reply}");
    assert!(reply.starts_with("ERR health-stale"), "{reply}");
    assert_eq!(
        fs::read(root.join("system/state")).expect("state after"),
        before
    );
    let (_, status) = daemon.ask("UPDATE STATUS 3");
    assert_eq!(
        line(&status, "candidate_attempts="),
        Some("candidate_attempts=2")
    );
    daemon.stop();
    cleanup(&root);
}

#[test]
fn a_boot_and_a_commit_cannot_interleave_under_the_root_lock() {
    let root = scratch("health-concurrent");
    init_and_stage(&root);
    run_boot(&root);
    let daemon = Daemon::start(&root, "health-concurrent");
    wait_for_healthy(&daemon);

    let boot = boot_command(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("boot process");
    let commit = query_command(&daemon.socket, "UPDATE COMMIT PASSED CONFIRM")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("commit process");
    let boot = boot.wait_with_output().expect("boot result");
    let commit = commit.wait_with_output().expect("commit result");
    assert!(
        boot.status.success(),
        "boot failed: {}",
        String::from_utf8_lossy(&boot.stderr)
    );
    let commit_text = format!(
        "{}{}",
        String::from_utf8_lossy(&commit.stdout),
        String::from_utf8_lossy(&commit.stderr)
    );
    let (ok, status) = daemon.ask("UPDATE STATUS 3");
    assert!(ok, "{status}");
    assert_eq!(line(&status, "state="), Some("state=ready"));
    if commit.status.success() {
        assert_eq!(
            line(&status, "active_slot="),
            Some("active_slot=B"),
            "{status}"
        );
        assert_eq!(
            line(&status, "candidate_slot="),
            Some("candidate_slot=none"),
            "{status}"
        );
    } else {
        assert!(
            commit_text.contains("ERR health-unattested")
                || commit_text.contains("ERR health-stale"),
            "the serialised loser must be a typed health refusal: {commit_text}"
        );
        assert_eq!(
            line(&status, "active_slot="),
            Some("active_slot=A"),
            "{status}"
        );
        assert_eq!(
            line(&status, "candidate_slot="),
            Some("candidate_slot=B"),
            "{status}"
        );
        assert_eq!(
            line(&status, "candidate_attempts="),
            Some("candidate_attempts=2"),
            "{status}"
        );
    }
    daemon.stop();
    cleanup(&root);
}
