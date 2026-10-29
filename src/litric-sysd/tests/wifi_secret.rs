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

//! A Wi-Fi passphrase over the real socket.
//!
//! The unit test in `providers.rs` proves the `Debug` renderings are redacted.
//! This file proves the path a user actually takes: `NET WIFI JOIN` over the
//! Unix socket, a real credential file under a temporary root, and a reply that
//! carries neither the passphrase nor a rendering that contains it. A secret
//! that only leaks in the error path is still a leak, so the request is sent to
//! a machine with no `iwctl`, where the action fails after the file is written.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

const SENTINEL: &str = "S3NT1NEL-wifi-passphrase";

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "litric-sysd-wifi-secret-{tag}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creates the fixture root");
    dir
}

fn hex(text: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(text.len().saturating_mul(2));
    for byte in text.as_bytes() {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

struct Daemon {
    child: Child,
}

impl Daemon {
    fn start(root: &Path, socket: &Path) -> Self {
        let state = root.join("var/lib/litric-sysd/grants.conf");
        fs::create_dir_all(state.parent().expect("the state file has a parent"))
            .expect("creates the state directory");
        let log = fs::File::create(root.join("daemon.log")).expect("creates the daemon log");
        let child = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
            .arg("--serve")
            .arg("--socket")
            .arg(socket)
            .arg("--home")
            .arg(root)
            .arg("--root")
            .arg(root)
            .arg("--state")
            .arg(&state)
            .env("TZ", "UTC")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::from(log))
            .spawn()
            .expect("the daemon binary starts");
        let mut daemon = Self { child };
        let deadline = Instant::now() + Duration::from_secs(5);
        while !socket.exists() {
            if daemon
                .child
                .try_wait()
                .expect("the daemon's status is readable")
                .is_some()
                || Instant::now() >= deadline
            {
                let log = fs::read_to_string(root.join("daemon.log")).unwrap_or_default();
                panic!("the daemon did not bind {}: {log}", socket.display());
            }
            sleep(Duration::from_millis(5));
        }
        daemon
    }

    /// The daemon's own log, for a failure that needs it.
    fn log(&self, root: &Path) -> String {
        fs::read_to_string(root.join("daemon.log")).unwrap_or_default()
    }

    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// One request through the real client, over the real socket.
///
/// The socket file exists the moment `bind` runs, a fraction before `listen`:
/// a client that connects inside that window is refused, so a refusal that
/// names the connection is retried rather than reported as the daemon's answer.
/// A refusal that comes *from* the daemon is an answer and is returned at once.
fn ask(socket: &Path, request: &str) -> (bool, String) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let (ok, reply) = ask_once(socket, request);
        // The client's own transport failures start with its program name; a
        // daemon reply starts with `OK` or `ERR`, even when the reason it gives
        // is that `iwctl` was not found. Only the former is worth retrying.
        let connecting = reply.starts_with("litric-sysd: ");
        if !connecting || Instant::now() >= deadline {
            return (ok, reply);
        }
        sleep(Duration::from_millis(5));
    }
}

fn ask_once(socket: &Path, request: &str) -> (bool, String) {
    let began = Instant::now();
    let out = Command::new(env!("CARGO_BIN_EXE_litric-sysd"))
        .arg("--query")
        .arg(request)
        .arg("--socket")
        .arg(socket)
        .env("TZ", "UTC")
        .output()
        .expect("the client runs");
    let reply = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    eprintln!("one query took {} us", began.elapsed().as_micros());
    (out.status.success(), reply)
}

fn cleanup(root: &Path, socket: &Path) {
    let _ = fs::remove_file(socket);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_wifi_passphrase_over_the_socket_never_reaches_the_reply_or_the_journal() {
    let root = scratch("join");
    let socket = std::env::temp_dir().join(format!(
        "litric-sysd-wifi-secret-{}.sock",
        std::process::id()
    ));
    let _ = fs::remove_file(&socket);

    let daemon = Daemon::start(&root, &socket);
    let request = format!(
        "NET WIFI JOIN {} {}",
        hex("litric-secret-test"),
        hex(SENTINEL)
    );
    let (ok, reply) = ask(&socket, &request);
    assert!(
        !reply.contains(SENTINEL),
        "the reply carried the passphrase: {reply}"
    );
    assert!(
        !reply.contains(&hex(SENTINEL)),
        "the reply echoed the hex encoding: {reply}"
    );
    // The action is expected to fail on a host without iwctl; what matters is
    // that it answered at all and that the answer is secret-free either way.
    assert!(!reply.trim().is_empty(), "the daemon must answer");
    if !ok {
        assert!(
            reply.contains("ERR"),
            "a failed action must say so: {reply}"
        );
    }

    // The credential is still written, private, with the passphrase in it: the
    // redaction is about renderings, not about refusing to store the secret.
    let path = litric_sysd::providers::NetAction::iwd_dir(&root).join("litric-secret-test.psk");
    let written = fs::read_to_string(&path).expect("the credential file is written");
    assert!(
        written.contains(SENTINEL),
        "the credential file must hold the passphrase: {written}"
    );
    assert!(
        !reply.contains(&written),
        "the reply must not contain the file it wrote"
    );
    let mode = fs::metadata(&path)
        .expect("the credential file exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600, "a credential file must be owner-only");

    // The daemon's own journal is the other half of the test's name. This path
    // prints nothing today, so the check is a guard rather than a demonstration:
    // a future line that renders the action — `{:?}` of the request, an error
    // carrying its argument — fails here instead of in a user's bug report.
    let journal = daemon.log(&root);
    assert!(
        !journal.contains(SENTINEL),
        "the daemon journal carried the passphrase: {journal}"
    );
    assert!(
        !journal.contains(&hex(SENTINEL)),
        "the daemon journal carried the hex encoding: {journal}"
    );

    // And the audit ring the daemon serves over the same socket: a record names
    // the capability, never the credential it was exercised with.
    let (audit_ok, audit) = ask(&socket, "CAPS AUDIT");
    assert!(audit_ok, "CAPS AUDIT must be answered: {audit}");
    assert!(
        !audit.contains(SENTINEL) && !audit.contains(&hex(SENTINEL)),
        "the audit ring carried the passphrase: {audit}"
    );
    assert!(
        audit.contains("count="),
        "the audit ring must report itself: {audit}"
    );

    daemon.stop();
    cleanup(&root, &socket);
}
