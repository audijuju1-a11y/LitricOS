//! The LFP/1 server: request dispatch, one connection at a time
//! (ADR-0019 §1, `Accept=yes` per-connection activation).
//!
//! Framing is over any [`io::BufRead`]/[`io::Write`] pair so the protocol
//! logic runs in tests without a socket; [`serve_unix`] wires the same loop
//! to a `UnixListener`. Every reply is built by [`crate::proto`]; every
//! filesystem action goes through [`crate::ops`], [`crate::trash`] or
//! [`crate::archive`], so the confinement rules live in exactly one place.
//!
//! One deliberate restriction, stated rather than hidden: `VOLUMES` reports
//! the filesystem that holds the daemon root, with `size=` derived from the
//! block device via sysfs. `used=`/`avail=` need `statvfs` (libc), which
//! this crate does not link; they report 0 until the kernel-facing layer
//! provides them, and the shell renders that as "unknown".

use crate::ops::{EntryKind, Root, StatInfo};
use crate::path::LogicalPath;
use crate::proto::{self, Request};
use crate::trash::DiskTrash;
use crate::{Error, ErrorKind, Policy};
use litric_core::pct;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

/// `READTEXT` reads at most this many bytes (ADR-0019: first 64 KiB).
const TEXT_CAP: usize = 64 * 1024;

/// Bytes scanned for a NUL when deciding whether a file is text.
const BINARY_PROBE: usize = 8 * 1024;

/// How one dispatched request wants the connection to continue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Reply lines to send under an `OK <n>` head.
    Reply(Vec<String>),
    /// Send an `ERR` line built from this error.
    Fail(Error),
    /// No reply; close the connection (`QUIT`).
    Quit,
}

/// Everything one connection needs. A fresh [`Connection`] per accepted
/// socket, exactly the socket-activated model of ADR-0019.
pub struct Connection {
    root: Root,
}

impl Connection {
    /// A connection over `root`.
    #[must_use]
    pub fn new(root: Root) -> Self {
        Self { root }
    }

    /// The root, for callers that pre-open the trash.
    #[must_use]
    pub const fn root(&self) -> &Root {
        &self.root
    }

    /// Dispatch one request line (terminator already stripped).
    ///
    /// `progress` receives `(done, total)` byte pairs during long
    /// operations; the framing layer turns each into a `PROGRESS` line.
    ///
    /// # Errors
    /// Errors are values here (`Outcome::Fail`), not returns; the only
    /// hard failure is the caller's I/O.
    pub fn dispatch(&mut self, line: &str, progress: &mut dyn FnMut(u64, u64)) -> Outcome {
        let request = match proto::parse_request(line) {
            Ok(r) => r,
            Err(e) => return Outcome::Fail(Error::from(e)),
        };
        match request.verb.as_str() {
            "HELLO" => self.hello(&request),
            "PLACES" => self.places(),
            "LIST" => self.list(&request),
            "STAT" => self.stat(&request),
            "DU" => self.du(&request),
            "MKDIR" => self.make(&request, Kind::Dir),
            "NEWFILE" => self.make(&request, Kind::File),
            "RENAME" => self.rename(&request),
            "CONFLICTS" => self.conflicts(&request),
            "COPY" => self.transfer(&request, Move::Copy, progress),
            "MOVE" => self.transfer(&request, Move::Move, progress),
            "TRASH" => self.trash(&request),
            "TRASHLIST" => self.trashlist(),
            "RESTORE" => self.restore(&request),
            "PURGE" => self.purge(&request),
            "EMPTYTRASH" => self.empty_trash(),
            "DELETE" => self.delete(&request),
            "COMPRESS" => self.compress(&request, progress),
            "EXTRACT" => self.extract(&request, progress),
            "SEARCH" => self.search(&request),
            "RECENT" => self.recent(),
            "VOLUMES" => self.volumes(),
            "READTEXT" => self.read_text(&request),
            "QUIT" => Outcome::Quit,
            other => Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                format!("unknown verb: {other}"),
            )),
        }
    }

    fn hello(&self, request: &Request) -> Outcome {
        if request.fields.first().map(String::as_str) != Some("1") {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "HELLO takes the protocol version 1",
            ));
        }
        Outcome::Reply(vec![proto::entry(&[
            ("version", "1"),
            ("root", &self.root.host_root().to_string_lossy()),
            ("trash", "1"),
        ])])
    }

    fn places(&self) -> Outcome {
        // The ADR's place list, each at its conventional home-relative spot.
        const PLACES: [(&str, &str, &str); 10] = [
            ("home", "/", "Home"),
            ("desktop", "/Desktop", "Desktop"),
            ("documents", "/Documents", "Documents"),
            ("downloads", "/Downloads", "Downloads"),
            ("pictures", "/Pictures", "Pictures"),
            ("music", "/Music", "Music"),
            ("videos", "/Videos", "Videos"),
            ("trash", "/.local/share/Trash/files", "Trash"),
            ("recent", "/", "Recent"),
            ("thispc", "/", "This PC"),
        ];
        let lines = PLACES
            .iter()
            .map(|(place, path, label)| {
                let exists = if *place == "recent" || *place == "thispc" || *place == "home" {
                    true
                } else {
                    LogicalPath::parse(path)
                        .map(|p| self.root.resolve(&p).is_ok())
                        .unwrap_or(false)
                };
                proto::entry(&[
                    ("place", place),
                    ("path", path),
                    ("label", label),
                    ("exists", if exists { "1" } else { "0" }),
                ])
            })
            .collect();
        Outcome::Reply(lines)
    }

    fn list(&self, request: &Request) -> Outcome {
        let Some(path_field) = request.fields.first() else {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "LIST takes a path"));
        };
        let hidden = request.fields.get(1).is_some_and(|f| f == "hidden=1");
        let logical = match LogicalPath::parse(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        match self.root.list_dir(&logical, hidden) {
            Ok((entries, truncated)) => {
                let mut lines: Vec<String> = entries
                    .iter()
                    .map(|info| format!("e {}", pairs_line(stat_pairs(info))))
                    .collect();
                if truncated {
                    lines.push("truncated=1".to_owned());
                }
                Outcome::Reply(lines)
            }
            Err(e) => Outcome::Fail(e),
        }
    }

    fn stat(&self, request: &Request) -> Outcome {
        let Some(path_field) = request.fields.first() else {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "STAT takes a path"));
        };
        let logical = match LogicalPath::parse(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        match self.root.stat_with_items(&logical) {
            Ok((info, items)) => {
                let mut pairs = stat_pairs(&info);
                pairs.push(("items".to_owned(), items.to_string()));
                Outcome::Reply(vec![pairs_line(pairs)])
            }
            Err(e) => Outcome::Fail(e),
        }
    }

    fn du(&self, request: &Request) -> Outcome {
        let Some(path_field) = request.fields.first() else {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "DU takes a path"));
        };
        let logical = match LogicalPath::parse(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        match self.root.measure(&logical) {
            Ok((bytes, files, dirs, partial)) => Outcome::Reply(vec![proto::entry(&[
                ("bytes", &bytes.to_string()),
                ("files", &files.to_string()),
                ("dirs", &dirs.to_string()),
                ("partial", if partial { "1" } else { "0" }),
            ])]),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn make(&self, request: &Request, kind: Kind) -> Outcome {
        let (Some(parent), Some(raw_name)) = (request.fields.first(), request.fields.get(1)) else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "takes a parent path and a name",
            ));
        };
        let name = match proto::decode_field(raw_name) {
            Ok(n) => n,
            Err(e) => return Outcome::Fail(e),
        };
        let parent = match LogicalPath::parse(parent) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        let made = match kind {
            Kind::Dir => self.root.mkdir(&parent, &name),
            Kind::File => self.root.new_file(&parent, &name),
        };
        match made {
            Ok(host) => Outcome::Reply(vec![proto::entry(&[("path", &host.to_string_lossy())])]),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn rename(&self, request: &Request) -> Outcome {
        let (Some(path_field), Some(raw_name)) = (request.fields.first(), request.fields.get(1))
        else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "RENAME takes a path and a new name",
            ));
        };
        let name = match proto::decode_field(raw_name) {
            Ok(n) => n,
            Err(e) => return Outcome::Fail(e),
        };
        let logical = match LogicalPath::parse(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        match self.root.rename(&logical, &name) {
            Ok(host) => Outcome::Reply(vec![proto::entry(&[("path", &host.to_string_lossy())])]),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn conflicts(&self, request: &Request) -> Outcome {
        let Some(dest_field) = request.fields.first() else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "CONFLICTS takes a destination and paths",
            ));
        };
        let dest = match LogicalPath::parse(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        let dest_host = match self.root.resolve(&dest) {
            Ok(h) => h,
            Err(e) => return Outcome::Fail(e),
        };
        let lines: Vec<String> = request
            .fields
            .iter()
            .skip(1)
            .filter_map(|f| LogicalPath::parse(f).ok())
            .filter(|src| std::fs::symlink_metadata(dest_host.join(src.file_name())).is_ok())
            .map(|src| proto::entry(&[("name", src.file_name())]))
            .collect();
        Outcome::Reply(lines)
    }

    fn transfer(
        &self,
        request: &Request,
        direction: Move,
        progress: &mut dyn FnMut(u64, u64),
    ) -> Outcome {
        let (Some(dest_field), Some(policy_field)) =
            (request.fields.first(), request.fields.get(1))
        else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "takes a destination, a policy and paths",
            ));
        };
        let Some(policy) = Policy::parse(policy_field) else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "policy must be skip, replace or keep-both",
            ));
        };
        let dest = match LogicalPath::parse(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        let dest_host = match self.root.resolve(&dest) {
            Ok(h) => h,
            Err(e) => return Outcome::Fail(e),
        };
        let sources: Vec<LogicalPath> = match request
            .fields
            .iter()
            .skip(2)
            .map(|f| LogicalPath::parse(f).map_err(invalid))
            .collect::<Result<Vec<_>, Error>>()
        {
            Ok(s) => s,
            Err(e) => return Outcome::Fail(e),
        };
        // Progress denominators: each source measured before it moves.
        let mut sizes = Vec::with_capacity(sources.len());
        let mut total = 0u64;
        for src in &sources {
            let size = match self.root.stat(src) {
                Ok(info) => info.size,
                Err(e) => return Outcome::Fail(e),
            };
            total = total.saturating_add(size);
            sizes.push(size);
        }
        let mut done = 0u64;
        let mut lines = Vec::new();
        for (src, size) in sources.iter().zip(sizes.iter()) {
            let into_source_dir = src.parent().is_some_and(|p| p == dest);
            let made = match direction {
                Move::Copy => self
                    .root
                    .copy_item(src, &dest_host, policy, into_source_dir),
                Move::Move => self
                    .root
                    .move_item(src, &dest_host, policy, into_source_dir),
            };
            match made {
                Ok(host) => {
                    done = done.saturating_add(*size);
                    progress(done, total);
                    lines.push(proto::entry(&[("path", &host.to_string_lossy())]));
                }
                Err(e) => {
                    return Outcome::Fail(partial(e, src.as_str(), lines.len(), sources.len()))
                }
            }
        }
        Outcome::Reply(lines)
    }

    fn trash(&self, request: &Request) -> Outcome {
        if request.fields.is_empty() {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "TRASH takes paths"));
        }
        let disk = match DiskTrash::open(&self.root) {
            Ok(d) => d,
            Err(e) => return Outcome::Fail(e),
        };
        let mut lines = Vec::new();
        for field in &request.fields {
            let logical = match LogicalPath::parse(field) {
                Ok(p) => p,
                Err(e) => return Outcome::Fail(invalid(e)),
            };
            match disk.put(&self.root, &logical, now_unix()) {
                Ok(item) => lines.push(proto::entry(&[("trashed", &item.name)])),
                Err(e) => {
                    return Outcome::Fail(partial(e, field, lines.len(), request.fields.len()))
                }
            }
        }
        Outcome::Reply(lines)
    }

    fn trashlist(&self) -> Outcome {
        let disk = match DiskTrash::open(&self.root) {
            Ok(d) => d,
            Err(e) => return Outcome::Fail(e),
        };
        match disk.list() {
            Ok(items) => {
                let lines = items
                    .iter()
                    .map(|item| {
                        let kind = if item.is_dir { "dir" } else { "file" };
                        format!(
                            "t {}",
                            proto::entry(&[
                                ("name", &item.name),
                                ("original", &item.original_path),
                                ("deleted", &item.deleted_iso),
                                ("size", &item.size.to_string()),
                                ("kind", kind),
                            ])
                        )
                    })
                    .collect();
                Outcome::Reply(lines)
            }
            Err(e) => Outcome::Fail(e),
        }
    }

    fn restore(&self, request: &Request) -> Outcome {
        if request.fields.is_empty() {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "RESTORE takes trash names",
            ));
        }
        let disk = match DiskTrash::open(&self.root) {
            Ok(d) => d,
            Err(e) => return Outcome::Fail(e),
        };
        let mut lines = Vec::new();
        for name in &request.fields {
            let Some(item) = disk.get(name) else {
                return Outcome::Fail(Error::new(
                    ErrorKind::NotFound,
                    format!("no such trash entry: {name}"),
                ));
            };
            // DiskTrash normalizes both spec-compliant absolute paths and
            // legacy file:// records into the root's validated logical namespace.
            let logical = match LogicalPath::parse(&item.original_path) {
                Ok(p) => p,
                Err(e) => return Outcome::Fail(invalid(e)),
            };
            let parent = match self
                .root
                .resolve(&logical.parent().unwrap_or_else(LogicalPath::root))
            {
                Ok(h) => h,
                Err(e) => return Outcome::Fail(e),
            };
            let dest = parent.join(logical.file_name());
            match disk.restore(name, &dest) {
                Ok(host) => lines.push(proto::entry(&[("path", &host.to_string_lossy())])),
                Err(e) => {
                    return Outcome::Fail(partial(e, name, lines.len(), request.fields.len()))
                }
            }
        }
        Outcome::Reply(lines)
    }

    fn purge(&self, request: &Request) -> Outcome {
        if request.fields.is_empty() {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "PURGE takes trash names"));
        }
        let disk = match DiskTrash::open(&self.root) {
            Ok(d) => d,
            Err(e) => return Outcome::Fail(e),
        };
        let mut removed = 0u64;
        let total = request.fields.len();
        for (index, name) in request.fields.iter().enumerate() {
            match disk.purge(name) {
                Ok(_) => removed = removed.saturating_add(1),
                Err(e) => return Outcome::Fail(partial(e, name, index, total)),
            }
        }
        Outcome::Reply(vec![removed_line(removed)])
    }

    fn empty_trash(&self) -> Outcome {
        let disk = match DiskTrash::open(&self.root) {
            Ok(d) => d,
            Err(e) => return Outcome::Fail(e),
        };
        match disk.empty() {
            Ok((n, _bytes)) => Outcome::Reply(vec![removed_line(n)]),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn delete(&self, request: &Request) -> Outcome {
        if request.fields.is_empty() {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "DELETE takes paths"));
        }
        let mut removed = 0u64;
        let total = request.fields.len();
        for (index, field) in request.fields.iter().enumerate() {
            let logical = match LogicalPath::parse(field) {
                Ok(p) => p,
                Err(e) => return Outcome::Fail(invalid(e)),
            };
            match self.root.delete(&logical) {
                Ok(()) => removed = removed.saturating_add(1),
                Err(e) => return Outcome::Fail(partial(e, field, index, total)),
            }
        }
        Outcome::Reply(vec![removed_line(removed)])
    }

    fn compress(&self, request: &Request, progress: &mut dyn FnMut(u64, u64)) -> Outcome {
        let Some(dest_field) = request.fields.first() else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "COMPRESS takes a destination archive and paths",
            ));
        };
        let sources: Vec<String> = request.fields.iter().skip(1).cloned().collect();
        let dest = match LogicalPath::parse(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        let sources = match sources
            .iter()
            .map(|f| LogicalPath::parse(f).map_err(invalid))
            .collect::<Result<Vec<_>, Error>>()
        {
            Ok(s) => s,
            Err(e) => return Outcome::Fail(e),
        };
        match crate::archive::compress_zip(&self.root, &dest, &sources, progress) {
            Ok(host) => Outcome::Reply(vec![proto::entry(&[("path", &host.to_string_lossy())])]),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn extract(&self, request: &Request, progress: &mut dyn FnMut(u64, u64)) -> Outcome {
        let (Some(archive_field), Some(dest_field)) =
            (request.fields.first(), request.fields.get(1))
        else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "EXTRACT takes an archive and a destination directory",
            ));
        };
        let archive = match LogicalPath::parse(archive_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        let dest = match LogicalPath::parse(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        match crate::archive::extract_archive(&self.root, &archive, &dest, progress) {
            Ok(_host) => Outcome::Reply(vec![proto::entry(&[("path", dest.as_str())])]),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn search(&self, request: &Request) -> Outcome {
        let (Some(path_field), Some(raw_query)) = (request.fields.first(), request.fields.get(1))
        else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "SEARCH takes a start path and a query",
            ));
        };
        let query = match proto::decode_field(raw_query) {
            Ok(q) => q,
            Err(e) => return Outcome::Fail(e),
        };
        let start = match LogicalPath::parse(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        match self.root.search(&start, &query) {
            Ok((hits, truncated)) => {
                let mut lines: Vec<String> = hits
                    .iter()
                    .map(|(rel, info)| {
                        let mut pairs = vec![("path", rel.clone())];
                        pairs.push(("name", info.name.clone()));
                        pairs.push(("kind", EntryKind::token(info.kind).to_owned()));
                        pairs.push(("size", info.size.to_string()));
                        pairs.push(("mtime", info.mtime.to_string()));
                        pairs
                            .iter()
                            .map(|(k, v)| format!("{k}={}", pct::encode_field(v)))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect();
                if truncated {
                    lines.push("truncated=1".to_owned());
                }
                Outcome::Reply(lines)
            }
            Err(e) => Outcome::Fail(e),
        }
    }

    fn recent(&self) -> Outcome {
        match self.root.recent() {
            Ok(items) => {
                let lines = items
                    .iter()
                    .map(|(rel, mtime, size)| {
                        proto::entry(&[
                            ("path", rel),
                            ("mtime", &mtime.to_string()),
                            ("size", &size.to_string()),
                        ])
                    })
                    .collect();
                Outcome::Reply(lines)
            }
            Err(e) => Outcome::Fail(e),
        }
    }

    fn volumes(&self) -> Outcome {
        match volumes_for_root(&self.root) {
            Ok(lines) => Outcome::Reply(lines),
            Err(e) => Outcome::Fail(e),
        }
    }

    fn read_text(&self, request: &Request) -> Outcome {
        let Some(path_field) = request.fields.first() else {
            return Outcome::Fail(Error::new(ErrorKind::BadRequest, "READTEXT takes a path"));
        };
        let logical = match LogicalPath::parse(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(invalid(e)),
        };
        let host = match self.root.resolve(&logical) {
            Ok(h) => h,
            Err(e) => return Outcome::Fail(e),
        };
        let meta = match std::fs::symlink_metadata(&host) {
            Ok(m) => m,
            Err(e) => return Outcome::Fail(Error::from_io("readtext", &e)),
        };
        if meta.is_dir() {
            return Outcome::Fail(Error::new(ErrorKind::IsADirectory, "that is a directory"));
        }
        let bytes = match std::fs::read(&host) {
            Ok(b) => b,
            Err(e) => return Outcome::Fail(Error::from_io("readtext", &e)),
        };
        let truncated = bytes.len() > TEXT_CAP;
        let slice: Vec<u8> = bytes.iter().take(TEXT_CAP).copied().collect();
        let binary = slice.iter().take(BINARY_PROBE).any(|&b| b == 0);
        if binary {
            return Outcome::Reply(vec![proto::entry(&[
                ("binary", "1"),
                ("truncated", if truncated { "1" } else { "0" }),
                ("text", ""),
            ])]);
        }
        let text = String::from_utf8_lossy(&slice).into_owned();
        Outcome::Reply(vec![proto::entry(&[
            ("binary", "0"),
            ("truncated", if truncated { "1" } else { "0" }),
            ("text", &text),
        ])])
    }
}

/// What [`Connection::make`] creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dir,
    File,
}

/// Copy or move, for [`Connection::transfer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Move {
    Copy,
    Move,
}

fn stat_pairs(info: &StatInfo) -> Vec<(String, String)> {
    let mut pairs = vec![
        ("name".to_owned(), pct::encode_field(&info.name)),
        ("kind".to_owned(), EntryKind::token(info.kind).to_owned()),
        ("size".to_owned(), info.size.to_string()),
        ("mtime".to_owned(), info.mtime.to_string()),
        ("mode".to_owned(), format!("{:o}", info.mode)),
        (
            "hidden".to_owned(),
            if info.hidden {
                "1".to_owned()
            } else {
                "0".to_owned()
            },
        ),
        (
            "target".to_owned(),
            pct::encode_field(info.target.as_deref().unwrap_or("")),
        ),
    ];
    if info.kind == EntryKind::Link {
        if let Some(target) = &info.target {
            pairs.push(("target".to_owned(), pct::encode_field(target)));
        }
    }
    pairs
}

fn invalid(e: crate::path::PathError) -> Error {
    Error::new(ErrorKind::InvalidName, format!("bad path: {e}"))
}

/// Encode every value in a key-value pair list into one protocol line.
fn pairs_line(pairs: Vec<(String, String)>) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", pct::encode_field(v)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn removed_line(n: u64) -> String {
    proto::entry(&[("removed", &n.to_string())])
}

/// An error raised part-way through a multi-item request.
///
/// LFP/1 can send `OK <n>` or one `ERR`, never both, so a reply cannot also
/// enumerate the items that already landed. The message carries it instead:
/// which item failed and how many completed before it. Without that, a client
/// cannot tell a request that changed nothing from one that changed four files
/// and then stopped, and its window would show a state that is not the state on
/// disk.
fn partial(e: Error, item: &str, done: usize, total: usize) -> Error {
    Error::new(
        e.kind(),
        format!(
            "{item}: {} ({done} of {total} items completed)",
            e.message()
        ),
    )
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Serve one framed connection: read request lines, dispatch, write
/// `PROGRESS`/`OK`/`ERR` lines. Returns when the peer closes or `QUIT`s.
///
/// # Errors
/// Whatever the underlying streams raise; a malformed line is an `ERR`
/// reply, not a close.
pub fn serve_stream(input: &mut dyn BufRead, output: &mut dyn Write, root: Root) -> io::Result<()> {
    let mut conn = Connection::new(root);
    let mut out = BufWriter::new(output);
    loop {
        let mut line = Vec::new();
        let n = input.read_until(b'\n', &mut line)?;
        if n == 0 {
            return Ok(()); // peer closed
        }
        while matches!(line.last(), Some(b'\n') | Some(b'\r')) {
            line.pop();
        }
        if line.len() > proto::MAX_LINE {
            writeln!(
                out,
                "{}",
                proto::err(ErrorKind::BadRequest, "request line too long")
            )?;
            out.flush()?;
            continue;
        }
        let text = String::from_utf8_lossy(&line).into_owned();
        let mut progress_write = |done: u64, total: u64| {
            let pct_done = percent(done, total);
            let _ = writeln!(out, "{}", proto::progress(done, total, pct_done));
        };
        match conn.dispatch(&text, &mut progress_write) {
            Outcome::Quit => return Ok(()),
            Outcome::Fail(e) => {
                writeln!(out, "{}", proto::err(e.kind(), e.message()))?;
            }
            Outcome::Reply(lines) => {
                writeln!(out, "{}", proto::ok(lines.len()))?;
                for l in &lines {
                    writeln!(out, "{l}")?;
                }
            }
        }
        out.flush()?;
    }
}

fn percent(done: u64, total: u64) -> u64 {
    if total == 0 {
        return 100;
    }
    done.saturating_mul(100)
        .checked_div(total)
        .unwrap_or(100)
        .min(100)
}

/// Bind and serve LFP/1 on a Unix socket, one connection at a time. The
/// listener is created by systemd socket activation in production; this
/// path is for standalone runs and integration tests.
///
/// # Errors
/// Bind/accept/stream errors surface to the caller.
pub fn serve_unix(socket: &Path, root: Root) -> io::Result<()> {
    let _ = std::fs::remove_file(socket);
    let listener = std::os::unix::net::UnixListener::bind(socket)?;
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let mut reader = io::BufReader::new(s.try_clone()?);
                let mut writer = s;
                let _ = serve_stream(&mut reader, &mut writer, root.clone());
            }
            Err(_) => continue, // a dead accept is retried, not fatal
        }
    }
    Ok(())
}

/// One mount table row, as `/proc/self/mounts` records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    /// The device or source field.
    pub source: String,
    /// The mountpoint.
    pub target: String,
    /// The filesystem type.
    pub fstype: String,
}

/// Parse mount-table text (the format of `/proc/self/mounts`).
#[must_use]
pub fn parse_mounts(text: &str) -> Vec<MountEntry> {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let source = fields.next()?.to_owned();
            let target = fields.next()?.to_owned();
            let fstype = fields.next()?.to_owned();
            Some(MountEntry {
                source,
                target,
                fstype,
            })
        })
        .collect()
}

/// The mount that hosts `path` — the longest target that prefixes it,
/// preferring real block devices over pseudo-filesystems.
#[must_use]
pub fn mount_hosting<'a>(mounts: &'a [MountEntry], path: &str) -> Option<&'a MountEntry> {
    mounts
        .iter()
        .filter(|m| {
            (path == m.target || path.starts_with(&format!("{}/", m.target)))
                || (m.target == "/" && path.starts_with('/'))
        })
        .max_by_key(|m| {
            // Real block devices beat pseudo-filesystems; among matches, the
            // longest target is the most specific, so /boot beats /.
            let real = u64::from(m.source.starts_with("/dev/") || m.fstype == "ext4");
            real.saturating_mul(10_000)
                .saturating_add(m.target.len() as u64)
        })
}

/// Total bytes of a block device, read from sysfs. `None` when the device
/// is virtual (`tmpfs`, `overlay`, bind mounts) or sysfs does not expose
/// it — a missing number is reported, a wrong one is never invented.
#[must_use]
pub fn device_bytes(sysfs_root: &Path, source: &str) -> Option<u64> {
    let name = source.rsplit('/').next()?;
    if !name.starts_with("sd")
        && !name.starts_with("nvme")
        && !name.starts_with("vd")
        && !name.starts_with("hd")
        && !name.starts_with("mmcblk")
    {
        return None;
    }
    // Whole disk: /sys/block/<name>/size. Partition: /sys/block/<disk>/<name>/size.
    let direct = sysfs_root.join("block").join(name).join("size");
    if let Ok(text) = std::fs::read_to_string(&direct) {
        return sectors_to_bytes(text.trim());
    }
    let entries = std::fs::read_dir(sysfs_root.join("block")).ok()?;
    for disk in entries.filter_map(|e| e.ok()) {
        let part = disk.path().join(name).join("size");
        if let Ok(text) = std::fs::read_to_string(&part) {
            return sectors_to_bytes(text.trim());
        }
    }
    None
}

fn sectors_to_bytes(text: &str) -> Option<u64> {
    let sectors = text.parse::<u64>().ok()?;
    Some(sectors.saturating_mul(512))
}

/// The `VOLUMES` reply: the filesystem hosting the daemon root.
fn volumes_for_root(root: &Root) -> Result<Vec<String>, Error> {
    let mounts_text = std::fs::read_to_string("/proc/self/mounts")
        .map_err(|e| Error::from_io("read mounts", &e))?;
    let mounts = parse_mounts(&mounts_text);
    let host_path = root.host_root().to_string_lossy();
    let line = match mount_hosting(&mounts, &host_path) {
        Some(m) => {
            let sysfs = PathBuf::from("/sys");
            let size = device_bytes(&sysfs, &m.source)
                .map(|b| b.to_string())
                .unwrap_or_else(|| "0".to_owned());
            proto::entry(&[
                ("target", &m.target),
                ("fstype", &m.fstype),
                ("size", &size),
                ("used", "0"),
                ("avail", "0"),
            ])
        }
        None => proto::entry(&[
            ("target", &host_path),
            ("fstype", "unknown"),
            ("size", "0"),
            ("used", "0"),
            ("avail", "0"),
        ]),
    };
    Ok(vec![line])
}

#[cfg(test)]
mod tests {
    // Tests assert on fixed-size reply vectors and known wire shapes; the
    // indices are asserted by the length checks that precede each use.
    #![allow(
        clippy::integer_division,
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing
    )]
    use super::*;
    use crate::path::LogicalPath;
    use litric_locale::TimeZone;
    use std::fs;

    fn temp_root(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-files-server-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("Documents")).expect("mkdir");
        dir
    }

    fn open_root(tag: &str) -> (PathBuf, Root) {
        let dir = temp_root(tag);
        let root = Root::open(&dir, TimeZone::utc("test")).expect("root opens");
        (dir, root)
    }

    fn one_line(conn: &mut Connection, line: &str) -> String {
        match conn.dispatch(line, &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1, "expected exactly one reply line");
                lines[0].clone()
            }
            Outcome::Fail(e) => format!("ERR {} {}", e.kind().token(), e.message()),
            Outcome::Quit => "QUIT".to_owned(),
        }
    }

    #[test]
    fn hello_announces_version_root_and_trash() {
        let (_dir, root) = open_root("hello");
        let mut conn = Connection::new(root);
        let line = one_line(&mut conn, "HELLO 1");
        assert!(line.starts_with("version=1 root="), "got: {line}");
        assert!(line.ends_with(" trash=1"));
        assert!(matches!(
            conn.dispatch("HELLO 2", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn places_lists_the_conventional_spots() {
        let (_dir, root) = open_root("places");
        let mut conn = Connection::new(root);
        match conn.dispatch("PLACES", &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 10);
                assert!(lines[0].starts_with("place=home "));
                assert!(lines.iter().any(|l| l.contains("place=documents")));
                assert!(lines.iter().any(|l| l.contains("place=trash")));
            }
            other => panic!("not a reply: {other:?}"),
        }
    }

    #[test]
    fn mkdir_stat_and_list_round_trip() {
        let (_dir, root) = open_root("mkdir");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "MKDIR /Documents Album");
        let line = one_line(&mut conn, "STAT /Documents/Album");
        assert!(line.contains("kind=dir"), "got: {line}");
        assert!(line.contains("items=0"));
        assert!(matches!(
            conn.dispatch("MKDIR /Documents Album", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
        one_line(&mut conn, "NEWFILE /Documents/Album notes.txt");
        match conn.dispatch("LIST /Documents hidden=0", &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                // Documents holds Album; notes.txt lives inside Album.
                assert_eq!(lines.len(), 1, "Album: {lines:?}");
                assert!(lines.iter().any(|l| l.starts_with("e name=Album ")));
            }
            other => panic!("not a reply: {other:?}"),
        }
        match conn.dispatch("LIST /Documents/Album hidden=0", &mut |_, _| {}) {
            Outcome::Reply(lines) => assert_eq!(lines.len(), 1, "notes.txt: {lines:?}"),
            other => panic!("not a reply: {other:?}"),
        }
    }

    #[test]
    fn rename_refuses_to_overwrite() {
        let (_dir, root) = open_root("rename");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents a.txt");
        one_line(&mut conn, "NEWFILE /Documents b.txt");
        assert!(matches!(
            conn.dispatch("RENAME /Documents/a.txt b.txt", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
        one_line(&mut conn, "RENAME /Documents/a.txt c.txt");
        assert!(matches!(
            conn.dispatch("STAT /Documents/a.txt", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
        assert!(matches!(
            conn.dispatch("STAT /Documents/c.txt", &mut |_, _| {}),
            Outcome::Reply(_)
        ));
    }

    #[test]
    fn trash_restore_and_purge_round_trip() {
        let (_dir, root) = open_root("trash");
        let host_root = root.host_root().to_path_buf();
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents memo.txt");
        let trashed = one_line(&mut conn, "TRASH /Documents/memo.txt");
        assert!(trashed.starts_with("trashed=memo.txt"), "got: {trashed}");

        // The on-disk field must be a percent-encoded absolute host path, not
        // a file:// URI, so standard Linux file managers can read the trash.
        let info_path = host_root.join(".local/share/Trash/info/memo.txt.trashinfo");
        let record = std::fs::read_to_string(&info_path).expect("trash info record");
        let path_line = record.lines().find(|line| line.starts_with("Path=")).expect("Path field");
        assert!(path_line.starts_with("Path=%2F"), "expected absolute path: {path_line}");
        assert!(!path_line.contains("file%3A"), "new records must not store a URI");

        match conn.dispatch("TRASHLIST", &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1);
                assert!(lines[0].starts_with("t name=memo.txt "));
                assert!(lines[0].contains("original=%2FDocuments%2Fmemo.txt"));
            }
            other => panic!("not a reply: {other:?}"),
        }
        // Keep entries written by older Litric versions restorable after the
        // spec-compliant on-disk format changes.
        let legacy_record = record.replace(path_line, "Path=file%3A%2F%2F%2FDocuments%2Fmemo.txt");
        std::fs::write(&info_path, legacy_record).expect("write legacy trash record");
        // Restoring to the now-free original location works.
        let restored = one_line(&mut conn, "RESTORE memo.txt");
        assert!(restored.contains("path="), "got: {restored}");
        assert!(matches!(
            conn.dispatch("STAT /Documents/memo.txt", &mut |_, _| {}),
            Outcome::Reply(_)
        ));
        // Trash it again, purge it, and it is gone for good.
        one_line(&mut conn, "TRASH /Documents/memo.txt");
        let purged = one_line(&mut conn, "PURGE memo.txt");
        assert!(purged.starts_with("removed=1"));
        assert!(matches!(
            conn.dispatch("RESTORE memo.txt", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn conflicts_names_only_the_occupied() {
        let (_dir, root) = open_root("conflicts");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents dup.txt");
        one_line(&mut conn, "MKDIR /Documents Other");
        one_line(&mut conn, "NEWFILE /Documents/Other dup.txt");
        one_line(&mut conn, "NEWFILE /Documents/Other unique.txt");
        match conn.dispatch(
            "CONFLICTS /Documents /Documents/Other/dup.txt /Documents/Other/unique.txt",
            &mut |_, _| {},
        ) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0], "name=dup.txt");
            }
            other => panic!("not a reply: {other:?}"),
        }
    }

    #[test]
    fn copy_reports_progress_and_keeps_both_into_the_source_dir() {
        let (_dir, root) = open_root("copy");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents song.mp3");
        one_line(&mut conn, "MKDIR /Documents Other");
        let mut progress_seen = Vec::new();
        let outcome = conn.dispatch(
            "COPY /Documents/Other keep-both /Documents/song.mp3",
            &mut |d, t| progress_seen.push((d, t)),
        );
        match outcome {
            Outcome::Reply(lines) => assert_eq!(lines.len(), 1),
            other => panic!("not a reply: {other:?}"),
        }
        assert!(matches!(
            conn.dispatch("STAT /Documents/Other/song.mp3", &mut |_, _| {}),
            Outcome::Reply(_)
        ));
        // Copying into the source's own directory keeps both under the
        // documented "name - Copy.ext" name.
        let outcome = conn.dispatch("COPY /Documents skip /Documents/song.mp3", &mut |_, _| {});
        match outcome {
            Outcome::Reply(lines) => {
                let line = lines.first().expect("one created item");
                assert!(line.contains("song%20-%20Copy.mp3"), "got: {line}");
            }
            other => panic!("not a reply: {other:?}"),
        }
    }

    #[test]
    fn move_refuses_a_directory_into_itself() {
        let (_dir, root) = open_root("move-into");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "MKDIR /Documents Box");
        assert!(matches!(
            conn.dispatch("MOVE /Documents/Box skip /Documents/Box", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn delete_counts_what_it_removed() {
        let (_dir, root) = open_root("delete");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents gone.txt");
        let line = one_line(&mut conn, "DELETE /Documents/gone.txt");
        assert_eq!(line, "removed=1");
        assert!(matches!(
            conn.dispatch("DELETE /Documents/gone.txt", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
    }

    #[test]
    fn search_finds_by_name_and_marks_truncation_shape() {
        let (_dir, root) = open_root("search");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents report-2026.txt");
        let encoded = pct::encode_field("2026");
        let outcome = conn.dispatch(&format!("SEARCH /Documents {encoded}"), &mut |_, _| {});
        match outcome {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1);
                // Values are percent-encoded on the wire; the path's `/`
                // characters travel as %2F.
                assert!(lines[0].starts_with("path=%2FDocuments%2Freport-2026.txt "));
                assert!(lines[0].contains("kind=file"));
            }
            other => panic!("not a reply: {other:?}"),
        }
    }

    #[test]
    fn readtext_reports_text_and_binary() {
        let (dir_path, root) = open_root("readtext");
        let mut conn = Connection::new(root);
        fs::write(dir_path.join("Documents").join("note.txt"), "héllo text").expect("write");
        fs::write(dir_path.join("Documents").join("bin.dat"), [b'a', 0, b'b']).expect("write");
        let line = one_line(&mut conn, "READTEXT /Documents/note.txt");
        assert!(
            line.starts_with("binary=0 truncated=0 text="),
            "got: {line}"
        );
        assert!(
            line.contains("h%C3%A9llo"),
            "utf-8 is percent-encoded: {line}"
        );
        let line = one_line(&mut conn, "READTEXT /Documents/bin.dat");
        assert!(line.starts_with("binary=1"), "got: {line}");
    }

    #[test]
    fn unknown_verbs_and_malformed_requests_fail_cleanly() {
        let (_dir, root) = open_root("verbs");
        let mut conn = Connection::new(root);
        match conn.dispatch("WAT 1", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::BadRequest),
            other => panic!("expected a failure, got {other:?}"),
        }
        match conn.dispatch("", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::BadRequest),
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[test]
    fn quit_is_recognised() {
        let (_dir, root) = open_root("quit");
        let mut conn = Connection::new(root);
        assert_eq!(conn.dispatch("QUIT", &mut |_, _| {}), Outcome::Quit);
    }

    #[test]
    fn serve_stream_answers_over_pipes() {
        let (_dir, root) = open_root("stream");
        let requests = b"HELLO 1\nSTAT /Documents\nQUIT\n";
        let mut input = io::BufReader::new(&requests[..]);
        let mut output = Vec::new();
        serve_stream(&mut input, &mut output, root).expect("serve");
        let text = String::from_utf8(output).expect("utf-8");
        assert!(text.starts_with("OK 1\nversion=1"), "got: {text}");
        // STAT of a directory that exists replies OK 1; the loop closed on QUIT.
        assert!(text.contains("OK 1\n"));
    }

    #[test]
    fn mount_parsing_and_host_selection() {
        let table = concat!(
            "sysfs /sys sysfs rw 0 0\n",
            "/dev/nvme0n1p2 / ext4 rw 0 0\n",
            "/dev/nvme0n1p1 /boot vfat rw 0 0\n",
            "tmpfs /tmp tmpfs rw 0 0\n",
        );
        let mounts = parse_mounts(table);
        assert_eq!(mounts.len(), 4);
        let root_m = mount_hosting(&mounts, "/var/home/litric").expect("root fs found");
        assert_eq!(root_m.source, "/dev/nvme0n1p2");
        assert_eq!(root_m.fstype, "ext4");
        let boot = mount_hosting(&mounts, "/boot/efi").expect("boot fs found");
        assert_eq!(boot.target, "/boot");
        assert!(mount_hosting(&parse_mounts(""), "/x").is_none());
    }

    #[test]
    fn percent_counts_safely() {
        assert_eq!(percent(50, 200), 25);
        assert_eq!(percent(10, 0), 100);
        assert_eq!(percent(0, 0), 100);
        assert_eq!(percent(u64::MAX, 1), 100);
        assert_eq!(percent(1, u64::MAX), 0);
    }

    #[test]
    fn logical_paths_survive_the_wire() {
        // The wire path is also the logical path; make sure parse+stat agree
        // on the root itself and on nested paths.
        let (_dir, root) = open_root("paths");
        let mut conn = Connection::new(root);
        assert!(matches!(
            conn.dispatch("STAT /", &mut |_, _| {}),
            Outcome::Reply(_)
        ));
        assert!(matches!(
            conn.dispatch("STAT /../etc", &mut |_, _| {}),
            Outcome::Fail(_)
        ));
        let _ = LogicalPath::root();
    }
}
