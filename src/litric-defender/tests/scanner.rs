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

//! The scanner as a process: `litric-defender` spoken to over stdin/stdout.
//!
//! The unit tests in the crate exercise the engine. This file exercises the
//! binary that ADR-0022 says must be the only process touching untrusted bytes:
//! argument parsing, the framed worker, the bounded read, and the exit code a
//! parent would branch on. It is the test that would have failed before §64,
//! because there was no binary to spawn.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn scratch(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("litric-defender-e2e-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("creates the fixture root");
    dir
}

fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

const EICAR: &[u8] = br"X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";

struct Worker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Worker {
    fn start(feed: &PathBuf) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_litric-defender"))
            .arg("--stdio")
            .arg("--denylist")
            .arg(feed)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the scanner binary starts");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    /// One `SCAN`, returning the verdict line or the `ERR` line.
    fn scan(&mut self, path: &Path) -> Result<String, String> {
        let encoded = hex(path.to_string_lossy().as_bytes());
        writeln!(self.stdin, "SCAN {encoded}").expect("write");
        self.stdin.flush().expect("flush");
        self.read_reply()
    }

    fn read_reply(&mut self) -> Result<String, String> {
        let mut head = String::new();
        let n = self.stdout.read_line(&mut head).expect("read");
        assert_ne!(n, 0, "the scanner closed its reply stream");
        let head = head.trim_end().to_owned();
        if let Some(count) = head.strip_prefix("OK ") {
            let count: usize = count.parse().expect("a count");
            let mut lines = Vec::with_capacity(count);
            for _ in 0..count {
                let mut line = String::new();
                self.stdout.read_line(&mut line).expect("read body");
                lines.push(line.trim_end().to_owned());
            }
            return Ok(lines.join("\n"));
        }
        Err(head)
    }

    fn stop(mut self) {
        let _ = writeln!(self.stdin, "QUIT");
        let _ = self.stdin.flush();
        let _ = self.child.wait();
    }
}

fn one_shot(feed: &Path, path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_litric-defender"))
        .arg("--scan")
        .arg(path)
        .arg("--denylist")
        .arg(feed)
        .output()
        .expect("the one-shot scanner runs")
}

#[test]
fn the_scanner_denies_eicar_allows_clean_and_refuses_to_read_past_the_ceiling() {
    let root = scratch("scan");
    let feed = root.join("feed.ndb");
    let eicar = root.join("eicar.com");
    let clean = root.join("clean.txt");
    let huge = root.join("huge.bin");

    fs::write(&clean, b"an ordinary text file\n").expect("clean fixture");
    let digest = litric_defender::sha256::digest(EICAR);
    fs::write(&feed, format!("{} eicar-test\n", hex(&digest))).expect("feed");
    fs::write(&eicar, EICAR).expect("eicar fixture");

    let mut worker = Worker::start(&feed);

    // A detection is denied by exact hash, and the label is the feed's.
    let verdict = worker.scan(&eicar).expect("a verdict");
    assert!(
        verdict.contains("verdict=deny") && verdict.contains("tier=hash"),
        "EICAR must be denied by hash: {verdict}"
    );
    assert!(verdict.contains("id=eicar-test"), "{verdict}");

    // A clean file is allowed, and the digest of what was scanned is reported.
    let verdict = worker.scan(&clean).expect("a verdict");
    assert!(
        verdict.contains("verdict=allow"),
        "an ordinary file must be allowed: {verdict}"
    );
    assert!(verdict.contains("digest="), "{verdict}");
    assert!(!verdict.contains("digest=-"), "{verdict}");

    // The ceiling: a sparse file past 8 MiB is refused without reading it. The
    // expected answer is `unknown`, not `clean` — a partial scan that said clean
    // would be the failure this ceiling exists to avoid.
    let file = fs::File::create(&huge).expect("huge fixture");
    file.set_len(litric_defender::MAX_SCAN_BYTES.saturating_add(1))
        .expect("sparse length");
    let verdict = worker.scan(&huge).expect("a verdict");
    assert!(
        verdict.contains("verdict=unknown") && verdict.contains("reason=too-large"),
        "a file past the ceiling must be unknown, not clean: {verdict}"
    );

    // A missing path is an error the parent can branch on, not a verdict.
    let missing = root.join("does-not-exist");
    let err = worker.scan(&missing).expect_err("missing files are ERR");
    assert!(err.starts_with("ERR cannot-stat"), "{err}");

    worker.stop();

    // The one-shot form returns the exit code a script would act on.
    let out = one_shot(&feed, &eicar);
    assert_eq!(out.status.code(), Some(1), "a detection exits 1");
    assert!(String::from_utf8_lossy(&out.stdout).contains("verdict=deny"));
    let out = one_shot(&feed, &clean);
    assert_eq!(out.status.code(), Some(0), "a clean file exits 0");

    cleanup(&root);
}
