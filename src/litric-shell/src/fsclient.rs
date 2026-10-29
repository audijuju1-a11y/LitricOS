//! The shell's side of LFP/1: every file operation the Files application
//! performs, sent to `litric-filesd` (ADR-0019).
//!
//! # Why this is a separate module
//!
//! [`crate::fsbridge::FsBridge`] used to be the whole story: the desktop shell
//! opened the user's documents itself. That put the right to delete a directory
//! tree inside the process that draws pixels, loads fonts and can be made to
//! draw a convincing password prompt. This module moves the I/O out: the bridge
//! keeps its API, and when a daemon is attached it forwards each call here,
//! where it becomes one LFP/1 request and one reply.
//!
//! # Connection policy
//!
//! One connection per process, opened on first use and kept open, because the
//! daemon is socket-activated with `Accept=yes` — a new connection costs a
//! process, so a folder listing must not open one.
//!
//! A request is **never retried**. If the connection breaks after the request
//! line was written, the daemon may already have performed the rename or the
//! delete, and a silent retry would do it twice. The bridge reports the loss
//! and the window re-reads the folder, which is what tells the truth: the state
//! on disk. The connection is reopened by the next call.

#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use litric_core::pct;

use crate::apps::FileKind;
use crate::fsbridge::{Entry, FsError, TrashEntry};

/// How long the shell waits for one reply line before deciding the daemon is
/// wedged.
///
/// The desktop is a single-threaded render loop: a blocking read with no
/// deadline is a frozen screen with no explanation. Thirty seconds is far
/// longer than any local file operation that answers at all, and the daemon
/// sends `PROGRESS` lines during long copies, each of which restarts the clock.
const REPLY_TIMEOUT: Duration = Duration::from_secs(30);

/// One LFP/1 connection: a reader and a writer over the same socket.
struct Link {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl Link {
    fn connect(path: &Path) -> Result<Self, FsError> {
        let stream = UnixStream::connect(path).map_err(|e| {
            FsError::Io(format!(
                "the file service at {} is not reachable ({e})",
                path.display()
            ))
        })?;
        let writer = stream.try_clone().map_err(|e| {
            FsError::Io(format!(
                "the file service at {} could not be used ({e})",
                path.display()
            ))
        })?;
        let _ = stream.set_read_timeout(Some(REPLY_TIMEOUT));
        let _ = stream.set_write_timeout(Some(REPLY_TIMEOUT));
        Ok(Self {
            reader: BufReader::new(stream),
            writer,
        })
    }

    /// Send one request line and read its whole reply.
    fn exchange(&mut self, request: &str) -> Result<Vec<String>, String> {
        let mut line = String::with_capacity(request.len().saturating_add(1));
        line.push_str(request);
        line.push('\n');
        self.writer
            .write_all(line.as_bytes())
            .and_then(|()| self.writer.flush())
            .map_err(|e| format!("could not send the request: {e}"))?;
        loop {
            let head = self.read_line()?;
            // `PROGRESS` is informational and carries no reply of its own; the
            // final `OK`/`ERR` decides the request's outcome.
            if head.starts_with("PROGRESS ") {
                continue;
            }
            if let Some(rest) = head.strip_prefix("OK ") {
                let count: usize = rest
                    .trim()
                    .parse()
                    .map_err(|_| format!("the file service sent a malformed reply: {head:?}"))?;
                let mut lines = Vec::with_capacity(count.min(10_000));
                for _ in 0..count {
                    lines.push(self.read_line()?);
                }
                return Ok(lines);
            }
            if let Some(rest) = head.strip_prefix("ERR ") {
                return Err(format!("ERR {rest}"));
            }
            return Err(format!(
                "the file service sent a reply this shell does not understand: {head:?}"
            ));
        }
    }

    fn read_line(&mut self) -> Result<String, String> {
        let mut buf = Vec::new();
        let n = self
            .reader
            .read_until(b'\n', &mut buf)
            .map_err(|e| format!("the file service stopped answering: {e}"))?;
        if n == 0 {
            return Err("the file service closed the connection".to_owned());
        }
        while matches!(buf.last(), Some(b'\n') | Some(b'\r')) {
            buf.pop();
        }
        String::from_utf8(buf).map_err(|e| format!("the file service sent invalid UTF-8: {e}"))
    }
}

/// A connection to the Files daemon, shared by every surface that opens a
/// bridge.
#[derive(Debug)]
pub struct Daemon {
    socket: PathBuf,
    link: Mutex<Option<Link>>,
}

impl std::fmt::Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Link(open)")
    }
}

impl Daemon {
    /// A client for the daemon listening on `socket`. The connection is opened
    /// lazily, by the first request, so constructing this cannot fail.
    #[must_use]
    pub fn new(socket: impl Into<PathBuf>) -> Arc<Self> {
        Arc::new(Self {
            socket: socket.into(),
            link: Mutex::new(None),
        })
    }

    /// The socket this client talks to.
    #[must_use]
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// Whether the daemon answers `HELLO 1` right now.
    ///
    /// Called once at startup so the boot log and the Files window say which of
    /// the two bridges is in use instead of leaving it to be guessed.
    pub fn probe(&self) -> Result<(), FsError> {
        let lines = self.call("HELLO 1")?;
        if lines
            .iter()
            .any(|l| field(l, "version").as_deref() == Some("1"))
        {
            return Ok(());
        }
        Err(FsError::Io(
            "the file service answered, but not with LFP/1".to_owned(),
        ))
    }

    /// One request, one reply.
    fn call(&self, request: &str) -> Result<Vec<String>, FsError> {
        let mut guard = match self.link.lock() {
            Ok(g) => g,
            // A poisoned mutex means another thread panicked while holding the
            // connection. The bytes on the wire are not ours to trust any more,
            // so the connection is dropped rather than reused.
            Err(poisoned) => {
                let mut inner = poisoned.into_inner();
                *inner = None;
                inner
            }
        };
        let mut link = match guard.take() {
            Some(link) => link,
            None => Link::connect(&self.socket)?,
        };
        match link.exchange(request) {
            Ok(lines) => {
                *guard = Some(link);
                Ok(lines)
            }
            Err(reason) => {
                // Not put back: the reply was not completed, so this connection
                // cannot be trusted to be in step with the next request.
                Err(translate_error(&reason))
            }
        }
    }

    // ---- the operations, one per LFP/1 verb -------------------------------

    /// `LIST <path> [hidden=1]`: the folder's entries. Dot-prefixed entries
    /// are returned only when `show_hidden` asks for them, which is the same
    /// choice the Files window records in `settings.conf`.
    pub fn list_dir(&self, logical: &str, show_hidden: bool) -> Result<Vec<Entry>, FsError> {
        let request = if show_hidden {
            format!("LIST {} hidden=1", wire_path(logical))
        } else {
            format!("LIST {}", wire_path(logical))
        };
        let lines = self.call(&request)?;
        let mut out = Vec::new();
        for line in &lines {
            if !line.starts_with("e ") {
                // `truncated=1` and any future trailer: not an entry.
                continue;
            }
            let Some(name) = field(line, "name") else {
                continue;
            };
            out.push(Entry {
                name,
                kind: match field(line, "kind").as_deref() {
                    Some("dir") => FileKind::Dir,
                    Some("link") => FileKind::Link,
                    Some("file") => FileKind::File,
                    _ => FileKind::Other,
                },
                size: number(line, "size").unwrap_or(0),
                mtime: number(line, "mtime").unwrap_or(0),
            });
        }
        Ok(out)
    }

    /// `STAT <path>`: whether the item is there.
    pub fn exists(&self, logical: &str) -> Result<bool, FsError> {
        match self.call(&format!("STAT {}", wire_path(logical))) {
            Ok(_) => Ok(true),
            Err(FsError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// `MKDIR <parent> <name>`.
    pub fn mkdir(&self, parent: &str, name: &str) -> Result<(), FsError> {
        self.call(&format!(
            "MKDIR {} {}",
            wire_path(parent),
            pct::encode_field(name)
        ))
        .map(|_| ())
    }

    /// `NEWFILE <parent> <name>`.
    pub fn new_file(&self, parent: &str, name: &str) -> Result<(), FsError> {
        self.call(&format!(
            "NEWFILE {} {}",
            wire_path(parent),
            pct::encode_field(name)
        ))
        .map(|_| ())
    }

    /// `WRITETEXT <parent> <name> <content>`.
    pub fn write_text(&self, parent: &str, name: &str, content: &str) -> Result<(), FsError> {
        self.call(&format!(
            "WRITETEXT {} {} {}",
            wire_path(parent),
            pct::encode_field(name),
            pct::encode_field(content)
        ))
        .map(|_| ())
    }

    /// `IMPORT <parent> <name> <staged-name>`: a finished browser download
    /// into the user's tree.
    ///
    /// The staged file is named by one segment and read from the daemon's own
    /// `--import-dir`; nothing here can name a path outside it.
    pub fn import(&self, parent: &str, name: &str, staged: &str) -> Result<(), FsError> {
        self.call(&format!(
            "IMPORT {} {} {}",
            wire_path(parent),
            pct::encode_field(name),
            pct::encode_field(staged)
        ))
        .map(|_| ())
    }

    /// `RENAME <path> <new-name>`.
    pub fn rename(&self, parent: &str, old: &str, new: &str) -> Result<(), FsError> {
        self.call(&format!(
            "RENAME {} {}",
            wire_path(&join(parent, old)),
            pct::encode_field(new)
        ))
        .map(|_| ())
    }

    /// `COPY <dest-dir> keep-both <path>`: the new item's name.
    pub fn copy(&self, from: &str, dest_dir: &str) -> Result<String, FsError> {
        let lines = self.call(&format!(
            "COPY {} keep-both {}",
            wire_path(dest_dir),
            wire_path(from)
        ))?;
        last_name(&lines).ok_or_else(|| FsError::Io("the file service copied nothing".to_owned()))
    }

    /// `MOVE <dest-dir> keep-both <path>`: the item's name in its new place.
    pub fn move_item(&self, from: &str, dest_dir: &str) -> Result<String, FsError> {
        let lines = self.call(&format!(
            "MOVE {} keep-both {}",
            wire_path(dest_dir),
            wire_path(from)
        ))?;
        last_name(&lines).ok_or_else(|| FsError::Io("the file service moved nothing".to_owned()))
    }

    /// `DELETE <path>`: permanent.
    pub fn delete(&self, logical: &str) -> Result<(), FsError> {
        self.call(&format!("DELETE {}", wire_path(logical)))
            .map(|_| ())
    }

    /// `TRASH <path>`.
    ///
    /// The deletion date is stamped by the daemon from its own clock. The
    /// shell's `now_secs` argument is therefore unused here — one clock writes
    /// the record, and it is the clock of the process that owns the directory.
    pub fn trash(&self, logical: &str) -> Result<(), FsError> {
        self.call(&format!("TRASH {}", wire_path(logical)))
            .map(|_| ())
    }

    /// `TRASHLIST`.
    pub fn trash_list(&self) -> Result<Vec<TrashEntry>, FsError> {
        let lines = self.call("TRASHLIST")?;
        let mut out = Vec::new();
        for line in &lines {
            if !line.starts_with("t ") {
                continue;
            }
            let Some(name) = field(line, "name") else {
                continue;
            };
            let original = field(line, "original").unwrap_or_else(|| "/".to_owned());
            out.push(TrashEntry {
                name,
                deleted_iso: field(line, "deleted").unwrap_or_default(),
                restore_dir: parent_of(&original),
                is_dir: field(line, "kind").as_deref() == Some("dir"),
            });
        }
        Ok(out)
    }

    /// `RESTORE <trash-name>`: the folder the item went back to.
    pub fn trash_restore(&self, name: &str) -> Result<String, FsError> {
        let lines = self.call(&format!("RESTORE {}", pct::encode_field(name)))?;
        lines
            .first()
            .and_then(|l| field(l, "path").map(|p| parent_of(&p)))
            .ok_or_else(|| FsError::Io("the file service restored nothing".to_owned()))
    }

    /// `PURGE <trash-name>`.
    pub fn trash_purge(&self, name: &str) -> Result<(), FsError> {
        self.call(&format!("PURGE {}", pct::encode_field(name)))
            .map(|_| ())
    }

    /// `EMPTYTRASH`: how many entries went.
    pub fn trash_empty(&self) -> Result<usize, FsError> {
        let lines = self.call("EMPTYTRASH")?;
        let removed = lines
            .first()
            .and_then(|l| number(l, "removed"))
            .unwrap_or(0);
        Ok(usize::try_from(removed).unwrap_or(usize::MAX))
    }

    /// `READTEXT <path>`.
    ///
    /// The daemon serves the first 64 KiB of a text file and reports
    /// `binary=1` for content with a NUL in it. A binary file is an error here
    /// rather than a page of replacement characters: the viewer was asked for a
    /// text file and this is not one.
    pub fn read_text(&self, logical: &str) -> Result<String, FsError> {
        let lines = self.call(&format!("READTEXT {}", wire_path(logical)))?;
        let Some(line) = lines.first() else {
            return Err(FsError::Io("the file service read nothing".to_owned()));
        };
        if field(line, "binary").as_deref() == Some("1") {
            return Err(FsError::Io(
                "that file is not text, so there is nothing to show".to_owned(),
            ));
        }
        Ok(field(line, "text").unwrap_or_default())
    }

    /// `SEARCH <path> <query>`: full logical paths, at most 500.
    pub fn search(&self, from: &str, query: &str) -> Result<Vec<String>, FsError> {
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let lines = self.call(&format!(
            "SEARCH {} {}",
            wire_path(from),
            pct::encode_field(query)
        ))?;
        Ok(lines
            .iter()
            .filter(|l| l.starts_with("e "))
            .filter_map(|l| field(l, "path"))
            .collect())
    }

    /// `COMPRESS <dest-zip> <path>...`.
    pub fn compress(&self, dest: &str, sources: &[String]) -> Result<(), FsError> {
        if sources.is_empty() {
            return Err(FsError::BadName);
        }
        let mut request = format!("COMPRESS {}", wire_path(dest));
        for src in sources {
            request.push(' ');
            request.push_str(&wire_path(src));
        }
        self.call(&request).map(|_| ())
    }

    /// `EXTRACT <archive> <dest-dir>`.
    ///
    /// The daemon extracts *into* a directory that must already exist, so the
    /// folder is created first — through the daemon as well, because the shell
    /// does not create folders itself any more. A folder that is already there
    /// is refused by the daemon's `EXTRACT` rule, which is the same refusal the
    /// direct path gave: never merge an archive into an existing folder.
    pub fn extract(&self, archive: &str, dest_dir: &str) -> Result<(), FsError> {
        let parent = parent_of(dest_dir);
        let leaf = dest_dir.rsplit('/').next().unwrap_or_default();
        self.mkdir(&parent, leaf)?;
        self.call(&format!(
            "EXTRACT {} {}",
            wire_path(archive),
            wire_path(dest_dir)
        ))
        .map(|_| ())
    }
}

/// Send a path field in whichever of the two forms LFP/1 carries.
///
/// A path is sent literally when it can be — it starts with `/` and holds no
/// space and no `%` — and percent-encoded otherwise. The rule is the daemon's
/// `logical_path` read backwards, and both ends live in this workspace so a
/// change to one is a change to the other.
fn wire_path(logical: &str) -> String {
    let literal = logical.starts_with('/')
        && logical
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '%' | '='));
    if literal {
        logical.to_owned()
    } else {
        pct::encode_field(logical)
    }
}

/// `parent/name`, with the root's single separator kept single.
fn join(parent: &str, name: &str) -> String {
    if parent.ends_with('/') {
        format!("{parent}{name}")
    } else {
        format!("{parent}/{name}")
    }
}

/// The directory part of a logical path; `/` for an item at the top.
fn parent_of(logical: &str) -> String {
    match logical.rsplit_once('/') {
        Some(("", _)) | None => "/".to_owned(),
        Some((dir, _)) => dir.to_owned(),
    }
}

/// One `key=value` pair of a reply line, percent-decoded.
///
/// Every text value on this protocol is encoded when it needs to be — the
/// daemon writes names, paths and messages through `pct::encode_field` — so the
/// decoding happens here, once, and not at each of the twenty call sites.
fn field(line: &str, key: &str) -> Option<String> {
    let raw = raw_field(line, key)?;
    Some(pct::decode_field(raw).unwrap_or_else(|_| raw.to_owned()))
}

/// A `key=value` pair exactly as it arrived.
fn raw_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split(' ').find_map(|pair| {
        pair.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix('='))
    })
}

/// A numeric reply value. Numbers carry no escapes, so the raw text is read.
fn number(line: &str, key: &str) -> Option<u64> {
    raw_field(line, key).and_then(|v| v.parse().ok())
}

/// The name of the item a `COPY`/`MOVE` created.
fn last_name(lines: &[String]) -> Option<String> {
    let path = lines.iter().find_map(|l| field(l, "path"))?;
    Some(path.rsplit('/').next().unwrap_or(&path).to_owned())
}

/// Turn a transport failure into what the user should be told.
fn translate_error(reason: &str) -> FsError {
    let Some(rest) = reason.strip_prefix("ERR ") else {
        return FsError::Io(reason.to_owned());
    };
    let (kind, message) = rest.split_once(' ').unwrap_or((rest, ""));
    let message = pct::decode_field(message).unwrap_or_else(|_| message.to_owned());
    match kind {
        "not-found" => FsError::NotFound,
        "exists" => FsError::Exists,
        "invalid-name" => FsError::BadName,
        "permission-denied" | "symlink-refused" => FsError::Denied,
        "not-a-directory" | "is-a-directory" => FsError::NotADirectory,
        _ => FsError::Io(message),
    }
}

#[cfg(test)]
mod tests {
    // Tests assert by panicking and index the replies they just asserted the
    // length of; the workspace denies both in shipping code, where a panic is
    // an availability bug.
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]

    use super::*;

    // ---------------------------------------------------------------------
    // End to end: a real daemon on a real socket, and the bridge the Files
    // window would use. These tests are the reason the client is written
    // against the protocol rather than against a trait: the wire format is the
    // thing that can be wrong, and only a socket shows it.
    // ---------------------------------------------------------------------

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-fsclient-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// Start a daemon on a socket under `root`, and wait until it accepts.
    fn serve_staged(
        root: &Path,
        socket: &Path,
        import_dir: Option<&Path>,
    ) -> std::thread::JoinHandle<()> {
        litric_files::ensure_storage(root).expect("storage root");
        let mut opened = litric_files::Root::open(root, litric_locale::TimeZone::utc("test"))
            .expect("root opens");
        opened.set_import_dir(import_dir.map(Path::to_path_buf));
        let path = socket.to_path_buf();
        let handle = std::thread::spawn(move || {
            let _ = litric_files::server::serve_unix(&path, opened);
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if std::os::unix::net::UnixStream::connect(socket).is_ok() {
                return handle;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!(
            "the test daemon never started listening on {}",
            socket.display()
        );
    }

    fn serve(root: &Path, socket: &Path) -> std::thread::JoinHandle<()> {
        serve_staged(root, socket, None)
    }

    fn bridge(root: &Path, socket: &Path) -> crate::fsbridge::FsBridge {
        let daemon = Daemon::new(socket);
        daemon.probe().expect("the daemon answers LFP/1");
        crate::fsbridge::FsBridge::open_via_daemon(
            root,
            litric_locale::TimeZone::utc("test"),
            daemon,
        )
        .expect("bridge")
    }

    #[test]
    fn the_files_window_operations_all_go_through_the_daemon() {
        let base = scratch("session");
        let root = base.join("root");
        let socket = base.join("files.sock");
        std::fs::create_dir_all(&base).expect("scratch");
        let _server = serve(&root, &socket);
        let b = bridge(&root, &socket);

        // Create a folder and a file in it.
        b.mkdir("/", "Docs").expect("mkdir");
        assert!(root.join("Docs").is_dir(), "on disk");
        b.write_text("/Docs", "note.txt", "hello").expect("write");
        assert_eq!(
            std::fs::read_to_string(root.join("Docs/note.txt")).expect("file"),
            "hello"
        );
        assert!(b.exists("/Docs/note.txt").expect("exists"));
        assert!(!b.exists("/Docs/missing.txt").expect("absent"));

        // Listing comes back as entries the table can draw.
        let listed = b.list_dir("/Docs").expect("list");
        assert_eq!(listed.len(), 1, "{listed:?}");
        assert_eq!(listed[0].name, "note.txt");
        assert_eq!(listed[0].kind, FileKind::File);

        // Rename, copy, and the copy keeps both under the Explorer name.
        b.rename("/Docs", "note.txt", "renamed.txt")
            .expect("rename");
        assert!(root.join("Docs/renamed.txt").is_file());
        let copied = b.copy("/Docs/renamed.txt", "/Docs").expect("copy");
        assert_eq!(copied, "renamed - Copy.txt", "the clash rule");
        assert!(root.join("Docs/renamed - Copy.txt").is_file());

        // Search, which walks inside the daemon.
        let hits = b.search("/", "renamed").expect("search");
        assert!(
            hits.iter().any(|p| p.ends_with("/Docs/renamed.txt")),
            "{hits:?}"
        );

        // Trash, list, restore.
        b.trash("/Docs/renamed - Copy.txt", 0).expect("trash");
        assert!(!root.join("Docs/renamed - Copy.txt").exists());
        let trash = b.trash_list().expect("trash list");
        assert_eq!(trash.len(), 1, "{trash:?}");
        assert_eq!(trash[0].name, "renamed - Copy.txt");
        assert_eq!(trash[0].restore_dir, "/Docs");
        assert_eq!(
            b.trash_restore("renamed - Copy.txt").expect("restore"),
            "/Docs"
        );
        assert!(root.join("Docs/renamed - Copy.txt").is_file());

        // Reading text, and a binary file refused as text.
        assert_eq!(b.read_text("/Docs/renamed.txt").expect("read"), "hello");
        std::fs::write(root.join("Docs/blob.bin"), [0u8, 9]).expect("binary");
        assert!(b.read_text("/Docs/blob.bin").is_err());

        // Archive out and back in, then delete for good.
        b.compress("/Docs/pack.zip", &["/Docs/renamed.txt".to_owned()])
            .expect("compress");
        assert!(root.join("Docs/pack.zip").is_file());
        assert_eq!(b.extract("/Docs/pack.zip").expect("extract"), "/Docs/pack");
        assert!(root.join("Docs/pack/renamed.txt").is_file());
        b.delete("/Docs/blob.bin").expect("delete");
        assert!(!root.join("Docs/blob.bin").exists());

        // Trash again, then empty it: the count is the daemon's.
        b.trash("/Docs/renamed - Copy.txt", 0).expect("trash");
        assert_eq!(b.trash_empty().expect("empty"), 1);
        assert!(b.trash_list().expect("list").is_empty());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_staged_download_reaches_downloads_through_the_daemon() {
        let base = scratch("import-download");
        let root = base.join("root");
        let staging = base.join("staging");
        let socket = base.join("files.sock");
        std::fs::create_dir_all(&staging).expect("staging");
        std::fs::write(staging.join("report.pdf"), b"%PDF-1.7 body").expect("stage");
        // A link in the staging area is not a download: the import must refuse
        // it rather than follow it out of the directory.
        std::os::unix::fs::symlink("/etc/hostname", staging.join("link")).expect("symlink");
        let _server = serve_staged(&root, &socket, Some(&staging));
        let b = bridge(&root, &socket);

        b.import("/Downloads", "report.pdf", "report.pdf")
            .expect("import");
        assert_eq!(
            std::fs::read(root.join("Downloads/report.pdf")).expect("imported"),
            b"%PDF-1.7 body"
        );
        // The staged copy is still there: the broker owns it, and it is dropped
        // only after the import is confirmed.
        assert!(staging.join("report.pdf").is_file());
        // The panel's list is the same call the Files window makes.
        let listed = b.list_dir("/Downloads").expect("list");
        assert_eq!(listed.len(), 1, "{listed:?}");
        assert_eq!(listed[0].name, "report.pdf");

        // The same name again is refused: a download never replaces a document.
        assert_eq!(
            b.import("/Downloads", "report.pdf", "report.pdf")
                .expect_err("refused"),
            FsError::Exists
        );
        // A link is not a download, and a path is not a name.
        assert_eq!(
            b.import("/Downloads", "stolen", "link")
                .expect_err("refused"),
            FsError::Denied
        );
        assert_eq!(
            b.import("/Downloads", "escape", "../escape")
                .expect_err("refused"),
            FsError::BadName
        );
        assert!(!root.join("Downloads/stolen").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_missing_daemon_is_an_error_not_a_quiet_fallback() {
        // The directory is perfectly writable and the shell could have opened it
        // itself. With a daemon attached to a socket nobody is listening on, the
        // operation must fail: falling back here is the exact behaviour this
        // arrangement exists to prevent.
        let base = scratch("no-daemon");
        let root = base.join("root");
        std::fs::create_dir_all(&root).expect("root");
        let socket = base.join("absent.sock");
        let daemon = Daemon::new(&socket);
        assert!(daemon.probe().is_err(), "probe must fail");
        let b = crate::fsbridge::FsBridge::open_via_daemon(
            &root,
            litric_locale::TimeZone::utc("test"),
            daemon,
        )
        .expect("bridge");
        match b.mkdir("/", "Docs") {
            Err(FsError::Io(message)) => {
                assert!(message.contains("not reachable"), "got: {message}");
            }
            other => panic!("expected a transport error, got {other:?}"),
        }
        assert!(!root.join("Docs").exists(), "nothing may have been created");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_folder_named_with_a_space_and_a_percent_sign_survives_the_round_trip() {
        let base = scratch("awkward-names");
        let root = base.join("root");
        let socket = base.join("files.sock");
        std::fs::create_dir_all(&base).expect("scratch");
        let _server = serve(&root, &socket);
        let b = bridge(&root, &socket);
        b.mkdir("/Documents", "My 100% Folder").expect("mkdir");
        b.write_text("/Documents/My 100% Folder", "รายงาน.txt", "สวัสดี")
            .expect("write");
        let listed = b.list_dir("/Documents/My 100% Folder").expect("list");
        assert_eq!(listed.len(), 1, "{listed:?}");
        assert_eq!(listed[0].name, "รายงาน.txt");
        assert_eq!(
            b.read_text("/Documents/My 100% Folder/รายงาน.txt")
                .expect("read"),
            "สวัสดี"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_plain_path_goes_on_the_wire_as_it_is() {
        // ADR-0019's examples stay valid: no escapes for an ordinary path.
        assert_eq!(wire_path("/Documents/report.odt"), "/Documents/report.odt");
    }

    #[test]
    fn a_path_the_line_grammar_cannot_carry_is_encoded() {
        assert_eq!(
            wire_path("/Documents/My Folder"),
            "%2FDocuments%2FMy%20Folder"
        );
        // A literal `%` would otherwise be read as the start of an escape.
        assert_eq!(
            wire_path("/Documents/100% done"),
            "%2FDocuments%2F100%25%20done"
        );
        // Non-ASCII: Thai file names exist and must survive the wire.
        let thai = wire_path("/Documents/รายงาน.txt");
        assert!(thai.starts_with('%'), "got: {thai}");
        assert_eq!(
            pct::decode_field(&thai).as_deref(),
            Ok("/Documents/รายงาน.txt")
        );
    }

    #[test]
    fn an_encoded_path_no_longer_starts_with_a_slash() {
        // This is the property the daemon's `logical_path` keys off, so it is
        // asserted rather than assumed.
        assert!(!wire_path("/a b").starts_with('/'));
    }

    #[test]
    fn join_keeps_one_separator_at_the_root() {
        assert_eq!(join("/", "a.txt"), "/a.txt");
        assert_eq!(join("/Documents", "a.txt"), "/Documents/a.txt");
    }

    #[test]
    fn the_parent_of_a_top_level_item_is_the_root() {
        assert_eq!(parent_of("/a.txt"), "/");
        assert_eq!(parent_of("/Documents/a.txt"), "/Documents");
        assert_eq!(parent_of("/"), "/");
    }

    #[test]
    fn reply_fields_decode() {
        let line = "e name=my%20file.txt kind=file size=12 mtime=99 mode=0644 hidden=0";
        assert_eq!(field(line, "kind").as_deref(), Some("file"));
        assert_eq!(field(line, "name").as_deref(), Some("my file.txt"));
        assert_eq!(number(line, "size"), Some(12));
        assert_eq!(number(line, "missing"), None);
    }

    #[test]
    fn every_error_kind_maps_to_a_sentence_the_window_can_show() {
        assert_eq!(translate_error("ERR not-found nope"), FsError::NotFound);
        assert_eq!(translate_error("ERR exists taken"), FsError::Exists);
        assert_eq!(translate_error("ERR invalid-name bad"), FsError::BadName);
        assert_eq!(translate_error("ERR permission-denied no"), FsError::Denied);
        assert_eq!(
            translate_error("ERR not-a-directory file"),
            FsError::NotADirectory
        );
        match translate_error("ERR io disk%20on%20fire") {
            FsError::Io(m) => assert_eq!(m, "disk on fire"),
            other => panic!("wrong mapping: {other:?}"),
        }
        // A transport failure keeps its own sentence.
        match translate_error("the file service closed the connection") {
            FsError::Io(m) => assert!(m.contains("closed the connection")),
            other => panic!("wrong mapping: {other:?}"),
        }
    }

    #[test]
    fn the_created_name_is_taken_from_the_full_path() {
        // The daemon percent-encodes the path it reports, so a folder with a
        // space in its name must come back readable.
        let lines = vec!["path=%2FDocuments%2FMy%20Folder".to_owned()];
        assert_eq!(last_name(&lines).as_deref(), Some("My Folder"));
    }
}
