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

//! The capability record across a real daemon restart.
//!
//! The unit tests in `state.rs` prove the state reloads. This file proves the
//! **process** does: it starts `litric-sysd --serve`, asks it over the real
//! Unix socket what it holds, stops it, damages the record, and starts it
//! again. That is the sequence a power cut produces, and no in-process test can
//! show it — argument parsing, the socket, the seeded-then-loaded transition
//! and the refusal all have to survive a second `main`.
//!
//! The damage is deliberately the shape a crash leaves: a truncated grant line.
//! The daemon must come back with **no** grants, not with the policy set it
//! would otherwise issue, because a record that cannot be read is a denial.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litric-sysd-e2e-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creates the fixture root");
    dir
}

struct Daemon {
    child: Child,
}

impl Daemon {
    fn start(root: &Path, socket: &Path, state: &Path) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
            .arg("--serve")
            .arg("--socket")
            .arg(socket)
            .arg("--home")
            .arg(root)
            .arg("--root")
            .arg(root)
            .arg("--state")
            .arg(state)
            .env("TZ", "UTC")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("the daemon binary starts");
        Self { child }
    }

    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// One request through the real `--query` client, retried until the daemon is
/// serving. A failed connect is the normal state for the first few milliseconds
/// after spawn, so it is not an error until the deadline.
fn ask(socket: &Path, request: &str) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let out = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
            .arg("--query")
            .arg(request)
            .arg("--socket")
            .arg(socket)
            .env("TZ", "UTC")
            .output()
            .expect("the client runs");
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
        }
        if Instant::now() >= deadline {
            return Err(String::from_utf8_lossy(&out.stderr).into_owned());
        }
        sleep(Duration::from_millis(10));
    }
}

fn cleanup(root: &Path, socket: &Path) {
    let _ = fs::remove_file(socket);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_corrupt_record_makes_a_restarted_daemon_grant_nothing() {
    let root = scratch("restart");
    let socket = std::env::temp_dir().join(format!("litric-sysd-e2e-{}.sock", std::process::id()));
    let _ = fs::remove_file(&socket);
    let state = root.join("var/lib/litric-sysd/grants.conf");

    let first = Daemon::start(&root, &socket, &state);
    let first_reply = ask(&socket, "CAPS LIST").expect("the first daemon answers");
    assert!(
        first_reply.contains("store=seeded"),
        "a missing record is seeded, not loaded: {first_reply}"
    );
    assert!(
        first_reply.contains("capability=power.control"),
        "the policy grants are present on a fresh start: {first_reply}"
    );
    first.stop();
    assert!(state.exists(), "the first start wrote the record");
    let written = fs::read_to_string(&state).expect("the record is readable");
    assert!(
        written.starts_with("litric-grants\t1\n"),
        "the protocol line is on disk: {written}"
    );

    // The damage a crash between write and rename would leave.
    fs::write(
        &state,
        "litric-grants\t1\ngrant\tos.litric.shell\tpower.control@power://system/poweroff\tp\n",
    )
    .expect("damage the record");

    let second = Daemon::start(&root, &socket, &state);
    let second_reply = ask(&socket, "CAPS LIST").expect("the second daemon answers");
    assert!(
        second_reply.contains("store=corrupt:"),
        "the refusal is visible over the socket: {second_reply}"
    );
    assert!(
        second_reply.contains("grant_records=0"),
        "a corrupt record grants nothing: {second_reply}"
    );
    assert!(
        !second_reply.contains("capability="),
        "not even the policy grants survive a corrupt record: {second_reply}"
    );
    second.stop();

    // The damaged file is evidence and must not have been rewritten.
    assert_eq!(
        fs::read_to_string(&state).expect("the record is still there"),
        "litric-grants\t1\ngrant\tos.litric.shell\tpower.control@power://system/poweroff\tp\n"
    );

    cleanup(&root, &socket);
}
