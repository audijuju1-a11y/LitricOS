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
    assert_eq!(line(&status, "schema="), Some("schema=2"));
    assert_eq!(line(&status, "state="), Some("state=ready"));
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(line(&status, "candidate_slot="), Some("candidate_slot=B"));
    assert_eq!(
        line(&status, "candidate_attempts="),
        Some("candidate_attempts=1")
    );

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
