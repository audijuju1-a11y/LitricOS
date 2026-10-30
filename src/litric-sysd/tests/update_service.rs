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
use std::process::{Child, Command};
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

fn run_boot(root: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
        .arg("--boot")
        .arg("--update-root")
        .arg(root)
        .env("TZ", "UTC")
        .output()
        .expect("the boot entrypoint runs");
    assert!(
        out.status.success(),
        "boot entrypoint failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
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
    assert_eq!(line(&status, "schema="), Some("schema=1"));
    assert_eq!(line(&status, "state="), Some("state=ready"));
    assert_eq!(line(&status, "active_slot="), Some("active_slot=A"));
    assert_eq!(line(&status, "candidate_slot="), Some("candidate_slot=B"));
    assert_eq!(
        line(&status, "candidate_attempts="),
        Some("candidate_attempts=1")
    );

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
