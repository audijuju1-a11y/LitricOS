// A test asserts by panicking, and says which step broke with `expect`. The
// workspace denies those lints for the shipping code, where a panic is an
// availability bug; a test that cannot panic cannot fail.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! The daemon as a process: `litric-filesd` spoken to over stdin/stdout, which
//! is exactly how the image runs it (`StandardInput=socket`, `Accept=yes`).
//!
//! Unit tests inside the crate exercise the dispatcher; this file exercises the
//! binary — argument parsing, the root it prepares, the framing loop, and the
//! exit when the client goes away. Those are the parts a socket-activated unit
//! depends on and the parts no in-process test can prove.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litric-filesd-e2e-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

struct Daemon {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Daemon {
    fn start(root: &PathBuf) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_litric-filesd"))
            .arg("--stdio")
            .arg("--root")
            .arg(root)
            .env("TZ", "UTC")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the daemon binary starts");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    /// One request; `Ok(lines)` for `OK n`, `Err(line)` for `ERR ...`.
    fn ask(&mut self, request: &str) -> Result<Vec<String>, String> {
        writeln!(self.stdin, "{request}").expect("write");
        self.stdin.flush().expect("flush");
        loop {
            let mut head = String::new();
            let n = self.stdout.read_line(&mut head).expect("read");
            assert_ne!(n, 0, "the daemon closed its reply stream");
            let head = head.trim_end().to_owned();
            if head.starts_with("PROGRESS ") {
                continue;
            }
            if let Some(count) = head.strip_prefix("OK ") {
                let count: usize = count.parse().expect("a count");
                let mut lines = Vec::with_capacity(count);
                for _ in 0..count {
                    let mut line = String::new();
                    self.stdout.read_line(&mut line).expect("read body");
                    lines.push(line.trim_end().to_owned());
                }
                return Ok(lines);
            }
            return Err(head);
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        // QUIT if the connection is still there, then make sure the process is
        // not left behind if the test failed halfway.
        let _ = writeln!(self.stdin, "QUIT");
        let _ = self.stdin.flush();
        let _ = self.child.wait();
    }
}

#[test]
fn the_binary_serves_a_whole_files_session() {
    let root = scratch("session");
    std::fs::create_dir_all(&root).expect("scratch root");
    let mut daemon = Daemon::start(&root);

    // 1. It announces the root it prepared, and it prepared the first-run
    //    folders itself — the shell never creates them any more.
    let hello = daemon.ask("HELLO 1").expect("hello");
    assert!(hello.iter().any(|l| l.contains("version=1")), "{hello:?}");
    for folder in [
        "Documents",
        "Downloads",
        "Desktop",
        "Pictures",
        "Music",
        "Videos",
        "Projects",
    ] {
        assert!(
            root.join(folder).is_dir(),
            "{folder} was not created in {}",
            root.display()
        );
    }

    // 2. Create, write, list.
    daemon.ask("MKDIR /Documents Album").expect("mkdir");
    daemon
        .ask("WRITETEXT /Documents/Album notes.txt hello%20there")
        .expect("write");
    assert_eq!(
        std::fs::read_to_string(root.join("Documents/Album/notes.txt")).expect("on disk"),
        "hello there"
    );
    let listed = daemon.ask("LIST /Documents/Album").expect("list");
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert!(listed[0].contains("name=notes.txt"), "{listed:?}");

    // 3. A folder whose name has a space: the name was encodable, the path was
    //    not, and the daemon must accept the encoded path.
    daemon
        .ask("MKDIR /Documents My%20Folder")
        .expect("mkdir spaced");
    let encoded = "/Documents/My Folder"
        .replace('/', "%2F")
        .replace(' ', "%20");
    let stat = daemon.ask(&format!("STAT {encoded}")).expect("stat spaced");
    assert!(stat[0].contains("kind=dir"), "{stat:?}");

    // 4. Trash, list it, restore it.
    let trashed = daemon
        .ask("TRASH %2FDocuments%2FAlbum%2Fnotes.txt")
        .expect("trash");
    assert!(trashed[0].contains("trashed="), "{trashed:?}");
    let list = daemon.ask("TRASHLIST").expect("trashlist");
    assert_eq!(list.len(), 1, "{list:?}");
    assert!(
        list[0].contains("original=%2FDocuments%2FAlbum%2Fnotes.txt"),
        "{list:?}"
    );
    let restored = daemon.ask("RESTORE notes.txt").expect("restore");
    assert!(restored[0].contains("path="), "{restored:?}");
    assert!(root.join("Documents/Album/notes.txt").is_file());

    // 5. Read it back, and read a binary file as a refusal rather than as text.
    let read = daemon
        .ask("READTEXT %2FDocuments%2FAlbum%2Fnotes.txt")
        .expect("read");
    assert!(read[0].contains("binary=0"), "{read:?}");
    assert!(read[0].contains("text=hello%20there"), "{read:?}");
    std::fs::write(root.join("Documents/blob.bin"), [0u8, 1, 2]).expect("binary");
    let binary = daemon
        .ask("READTEXT /Documents/blob.bin")
        .expect("read binary");
    assert!(binary[0].contains("binary=1"), "{binary:?}");

    // 6. Compress, extract, delete.
    daemon
        .ask("COMPRESS /Documents/pack.zip %2FDocuments%2FAlbum")
        .expect("compress");
    assert!(root.join("Documents/pack.zip").is_file());
    // The destination must exist: `EXTRACT` writes into it, it does not create
    // it. The shell makes the folder first, also through the daemon.
    daemon
        .ask("MKDIR /Documents unpacked")
        .expect("mkdir unpacked");
    let extracted = daemon
        .ask("EXTRACT /Documents/pack.zip /Documents/unpacked")
        .expect("extract");
    assert!(extracted[0].contains("path="), "{extracted:?}");
    assert!(root.join("Documents/unpacked/Album/notes.txt").is_file());
    daemon.ask("DELETE /Documents/blob.bin").expect("delete");
    assert!(!root.join("Documents/blob.bin").exists());

    // 7. A command that is not in the contract is refused, and a well-formed
    //    one after it still works: one bad request must not wedge the session.
    assert!(daemon.ask("TELEPORT /Documents").is_err());
    assert!(daemon.ask("LIST /Documents").is_ok());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_binary_refuses_a_root_it_cannot_serve() {
    // A file where the root should be: the process must exit with a message
    // rather than accept a connection and fail every request.
    let root = scratch("not-a-root");
    std::fs::create_dir_all(root.parent().expect("parent")).expect("parent");
    std::fs::write(&root, b"this is a file").expect("file");
    let out = Command::new(env!("CARGO_BIN_EXE_litric-filesd"))
        .arg("--stdio")
        .arg("--root")
        .arg(&root)
        .stdin(Stdio::null())
        .output()
        .expect("runs");
    assert!(!out.status.success(), "it claimed success");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("storage root"), "stderr: {stderr}");
    let _ = std::fs::remove_file(&root);
}

#[test]
fn the_binary_reports_what_it_would_serve() {
    let root = scratch("info");
    std::fs::create_dir_all(&root).expect("scratch root");
    let out = Command::new(env!("CARGO_BIN_EXE_litric-filesd"))
        .arg("--info")
        .arg("--root")
        .arg(&root)
        .output()
        .expect("runs");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("protocol=1"), "{text}");
    assert!(text.contains("socket=/run/litric/files.sock"), "{text}");
    // `--info` must not create anything: it is a question, not a first run.
    assert!(!root.join("Documents").exists());
    let _ = std::fs::remove_dir_all(&root);
}
