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
use std::io::{self, BufRead, BufWriter, Read as _, Write};
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt as _, PermissionsExt as _};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
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
            "WRITETEXT" => self.write_text(&request),
            "IMPORT" => self.import(&request, progress),
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
        let logical = match logical_path(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
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
        let logical = match logical_path(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
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
        let logical = match logical_path(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
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
        let parent = match logical_path(parent) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        let made = match kind {
            Kind::Dir => self.root.mkdir(&parent, &name),
            Kind::File => self.root.new_file(&parent, &name),
        };
        match made {
            // The logical path, never the host path. The root is `/` in the
            // image, so both spellings were identical there and the difference
            // only appeared on a fixture root — where the shell would have shown
            // `/tmp/x/root/Documents` in a message about `/Documents`.
            Ok(_host) => match parent.join(&name) {
                Ok(created) => Outcome::Reply(vec![proto::entry(&[("path", created.as_str())])]),
                Err(e) => Outcome::Fail(invalid(e)),
            },
            Err(e) => Outcome::Fail(e),
        }
    }

    /// `WRITETEXT <parent> <name> <pct-content>` — create a file with content.
    ///
    /// Exclusive, like `NEWFILE`: a name that is taken is refused rather than
    /// overwritten, so a mistyped name can never destroy an existing document.
    /// The content is one percent-encoded field, which bounds it by
    /// [`proto::MAX_LINE`]; the shell's New Text File flow writes empty or short
    /// files, and anything larger is refused with `limit` instead of being
    /// truncated into a half-written document.
    fn write_text(&self, request: &Request) -> Outcome {
        let (Some(parent), Some(raw_name), Some(raw_content)) = (
            request.fields.first(),
            request.fields.get(1),
            request.fields.get(2),
        ) else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "WRITETEXT takes a parent path, a name and the encoded content",
            ));
        };
        // Exactly three. A fourth field used to be ignored, which is how a
        // request with an extra argument silently wrote the wrong content into
        // the wrong place instead of being refused.
        if request.fields.len() > 3 {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "WRITETEXT takes exactly three fields; quote the content as one",
            ));
        }
        let name = match proto::decode_field(raw_name) {
            Ok(n) => n,
            Err(e) => return Outcome::Fail(e),
        };
        let content = match proto::decode_field(raw_content) {
            Ok(c) => c,
            Err(e) => return Outcome::Fail(e),
        };
        let parent = match logical_path(parent) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        let host = match self.root.new_file(&parent, &name) {
            Ok(h) => h,
            Err(e) => return Outcome::Fail(e),
        };
        // The file was created exclusively a moment ago, so this writes the
        // content the caller asked for into the file it just reserved. A
        // failure here leaves an empty file behind, which is what the user sees
        // as "created but empty" rather than as a success.
        match std::fs::write(&host, content.as_bytes()) {
            Ok(()) => match parent.join(&name) {
                Ok(written) => Outcome::Reply(vec![proto::entry(&[("path", written.as_str())])]),
                Err(e) => Outcome::Fail(invalid(e)),
            },
            Err(e) => Outcome::Fail(Error::from_io("writetext", &e)),
        }
    }

    /// `IMPORT <parent> <name> <staged-name>` — a finished browser download
    /// into the user's tree.
    ///
    /// The staged file is named by one segment and read from the daemon's
    /// `--import-dir`; the destination is a logical path like every other
    /// command. This is the only way bytes enter the tree without a client
    /// writing them, and it is bounded to one directory on one side and one
    /// exclusive create on the other.
    fn import(&self, request: &Request, progress: &mut dyn FnMut(u64, u64)) -> Outcome {
        let (Some(parent), Some(raw_name), Some(raw_staged)) = (
            request.fields.first(),
            request.fields.get(1),
            request.fields.get(2),
        ) else {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "IMPORT takes a parent path, a name and the staged file's name",
            ));
        };
        if request.fields.len() > 3 {
            return Outcome::Fail(Error::new(
                ErrorKind::BadRequest,
                "IMPORT takes exactly three fields",
            ));
        }
        let name = match proto::decode_field(raw_name) {
            Ok(n) => n,
            Err(e) => return Outcome::Fail(e),
        };
        let staged = match proto::decode_field(raw_staged) {
            Ok(n) => n,
            Err(e) => return Outcome::Fail(e),
        };
        let parent = match logical_path(parent) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        match self.root.import_file(&staged, &parent, &name, progress) {
            Ok(()) => match parent.join(&name) {
                Ok(placed) => Outcome::Reply(vec![proto::entry(&[("path", placed.as_str())])]),
                Err(e) => Outcome::Fail(invalid(e)),
            },
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
        let logical = match logical_path(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        match self.root.rename(&logical, &name) {
            Ok(_host) => match logical
                .parent()
                .unwrap_or_else(LogicalPath::root)
                .join(&name)
            {
                Ok(renamed) => Outcome::Reply(vec![proto::entry(&[("path", renamed.as_str())])]),
                Err(e) => Outcome::Fail(invalid(e)),
            },
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
        let dest = match logical_path(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        let dest_host = match self.root.resolve(&dest) {
            Ok(h) => h,
            Err(e) => return Outcome::Fail(e),
        };
        let lines: Vec<String> = request
            .fields
            .iter()
            .skip(1)
            .filter_map(|f| logical_path(f).ok())
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
        let dest = match logical_path(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        let dest_host = match self.root.resolve(&dest) {
            Ok(h) => h,
            Err(e) => return Outcome::Fail(e),
        };
        let sources: Vec<LogicalPath> = match request
            .fields
            .iter()
            .skip(2)
            .map(|f| logical_path(f))
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
                    // The destination's logical path: `dest` joined with the
                    // name the item actually landed under (which a clash may
                    // have changed to `name (2).ext`).
                    let leaf = host
                        .file_name()
                        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                    match dest.join(&leaf) {
                        Ok(placed) => lines.push(proto::entry(&[("path", placed.as_str())])),
                        Err(e) => return Outcome::Fail(invalid(e)),
                    }
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
            let logical = match logical_path(field) {
                Ok(p) => p,
                Err(e) => return Outcome::Fail(e),
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
        for raw in &request.fields {
            // A trash entry is named after the file it holds, so it carries
            // every character a file name can — spaces included. `TRASHLIST`
            // hands the name back percent-encoded, so it is decoded here; a
            // name that needed no encoding decodes to itself.
            let name = match proto::decode_field(raw) {
                Ok(n) => n,
                Err(e) => return Outcome::Fail(e),
            };
            if name.is_empty() {
                return Outcome::Fail(Error::new(ErrorKind::BadRequest, "empty trash name"));
            }
            let name = name.as_str();
            let Some(item) = disk.get(name) else {
                return Outcome::Fail(Error::new(
                    ErrorKind::NotFound,
                    format!("no such trash entry: {name}"),
                ));
            };
            // `original_path` is already a logical path: `DiskTrash`'s reader
            // maps the entry's `Path=` back through
            // `trashinfo::logical_location`, which understands the current
            // absolute-location form and the legacy `file://` one. There is no
            // scheme to strip here any more (BUG-0009).
            //
            // The flag has to be checked BEFORE the parse, not instead of it.
            // An entry recorded against another root (`Path=/somewhere/else/x`,
            // a trash directory shared with a different system) arrives verbatim,
            // and a rooted path parses as a perfectly good logical path — so a
            // restore would resolve it under this root and write the item to a
            // location the entry never named. Syntax cannot tell the two apart;
            // the mapping can. A refused restore is the honest answer.
            if item.outside_root {
                return Outcome::Fail(Error::new(
                    ErrorKind::InvalidName,
                    format!(
                        "{} records a location outside this root ({}); there is \
                         no honest place to restore it to",
                        name, item.original_path
                    ),
                ));
            }
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
                // Where it went back to, as a logical path: the shell shows this
                // in "restored back in <folder>".
                Ok(_host) => match logical
                    .parent()
                    .unwrap_or_else(LogicalPath::root)
                    .join(name)
                {
                    Ok(back) => lines.push(proto::entry(&[("path", back.as_str())])),
                    Err(e) => return Outcome::Fail(invalid(e)),
                },
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
        for (index, raw) in request.fields.iter().enumerate() {
            // Same decoding as RESTORE: the name is a file name, and
            // `TRASHLIST` sends it percent-encoded.
            let name = match proto::decode_field(raw) {
                Ok(n) => n,
                Err(e) => return Outcome::Fail(e),
            };
            if name.is_empty() {
                return Outcome::Fail(Error::new(ErrorKind::BadRequest, "empty trash name"));
            }
            match disk.purge(&name) {
                Ok(_) => removed = removed.saturating_add(1),
                Err(e) => return Outcome::Fail(partial(e, &name, index, total)),
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
            let logical = match logical_path(field) {
                Ok(p) => p,
                Err(e) => return Outcome::Fail(e),
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
        let dest = match logical_path(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        let sources = match sources
            .iter()
            .map(|f| logical_path(f))
            .collect::<Result<Vec<_>, Error>>()
        {
            Ok(s) => s,
            Err(e) => return Outcome::Fail(e),
        };
        match crate::archive::compress_zip(&self.root, &dest, &sources, progress) {
            Ok(_host) => Outcome::Reply(vec![proto::entry(&[("path", dest.as_str())])]),
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
        let archive = match logical_path(archive_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        let dest = match logical_path(dest_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
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
        let start = match logical_path(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
        };
        match self.root.search(&start, &query) {
            Ok((hits, truncated)) => {
                // `e ` in front of every hit, as ADR-0019 documents. Without it
                // a reader that splits `e `-prefixed entry lines saw an empty
                // result set for a search that found files.
                let mut lines: Vec<String> = hits
                    .iter()
                    .map(|(rel, info)| {
                        let pairs = vec![
                            ("path".to_owned(), rel.clone()),
                            ("name".to_owned(), info.name.clone()),
                            ("kind".to_owned(), EntryKind::token(info.kind).to_owned()),
                            ("size".to_owned(), info.size.to_string()),
                            ("mtime".to_owned(), info.mtime.to_string()),
                        ];
                        format!("e {}", pairs_line(pairs))
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
        let logical = match logical_path(path_field) {
            Ok(p) => p,
            Err(e) => return Outcome::Fail(e),
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
        // The cap has to bound the read, not the reply. Reading the whole file
        // and truncating afterwards allocated the file's full size inside the
        // daemon -- the 256 MiB cgroup cap was the only thing between a large
        // log and an OOM kill, while the reply looked capped the whole time.
        let source = match std::fs::File::open(&host) {
            Ok(f) => f,
            Err(e) => return Outcome::Fail(Error::from_io("readtext", &e)),
        };
        let (slice, truncated) = match read_capped(source, TEXT_CAP) {
            Ok(read) => read,
            Err(e) => return Outcome::Fail(Error::from_io("readtext", &e)),
        };
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

/// Read at most `cap` bytes from `source`, and say whether it held more.
///
/// The reader is bounded with [`Read::take`] rather than read to the end and
/// sliced afterwards: a cap that only limits the reply limits nothing about
/// the daemon's peak memory. `cap + 1` is requested so "exactly cap" and "more
/// than cap" stay distinguishable without a second read.
fn read_capped(source: impl io::Read, cap: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::with_capacity(cap.saturating_add(1));
    let mut bounded = source.take((cap as u64).saturating_add(1));
    bounded.read_to_end(&mut bytes)?;
    let truncated = bytes.len() > cap;
    bytes.truncate(cap);
    Ok((bytes, truncated))
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
    // RAW values. `pairs_line` percent-encodes every value exactly once, and it
    // used to receive names that were already encoded here, so a folder called
    // `รายงาน` listed as `%25E0%25B8...` — the escape of the escape. ASCII names
    // hid it, because for them encoding is the identity, which is why it
    // survived until a Thai name went through a socket.
    //
    // `target` is listed once, not twice: a link's target is the same value the
    // line already carries, and a duplicate key is a line a reader has to guess
    // the meaning of.
    vec![
        ("name".to_owned(), info.name.clone()),
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
        ("target".to_owned(), info.target.clone().unwrap_or_default()),
    ]
}

fn invalid(e: crate::path::PathError) -> Error {
    Error::new(ErrorKind::InvalidName, format!("bad path: {e}"))
}

/// Parse a path field, in either of the two forms LFP/1 carries.
///
/// A path is normally sent as it is — `/Documents/report.odt` — because that is
/// what ADR-0019 documents and what every reply contains. But the line grammar
/// separates fields with single spaces, so a folder the user named *My
/// Documents* cannot be written that way at all, and before this existed the
/// daemon had no way to serve one: `MKDIR` could create the name but no
/// subsequent command could address it.
///
/// So a path that cannot be sent as-is is percent-encoded instead, exactly like
/// the free-text fields, and it stops starting with `/` when it is (`/` encodes
/// to `%2F`). That gives one unambiguous rule with no room for guessing:
///
/// * starts with `/`  → the path is literal, used as it is;
/// * anything else    → the path is percent-encoded and must decode.
///
/// # Errors
/// `bad-request` for a malformed escape; `invalid-name` for a path that breaks
/// the naming rules.
fn logical_path(raw: &str) -> Result<LogicalPath, Error> {
    if raw.starts_with('/') {
        return LogicalPath::parse(raw).map_err(invalid);
    }
    let decoded = proto::decode_field(raw)?;
    LogicalPath::parse(&decoded).map_err(invalid)
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
    // One buffer for the life of the connection, cleared between requests. The
    // old loop allocated a fresh `Vec` and a fresh `String` for every line; the
    // shell holds a connection open for a whole session, so that was two
    // allocations per folder listing for no gain.
    let mut line: Vec<u8> = Vec::with_capacity(1024);
    loop {
        let overlong = match read_frame(input, &mut line)? {
            Frame::Eof => return Ok(()), // peer closed
            Frame::Line { overlong } => overlong,
        };
        while matches!(line.last(), Some(b'\n') | Some(b'\r')) {
            line.pop();
        }
        if overlong || line.len() > proto::MAX_LINE {
            writeln!(
                out,
                "{}",
                proto::err(ErrorKind::BadRequest, "request line too long")
            )?;
            out.flush()?;
            continue;
        }
        let mut progress_write = |done: u64, total: u64| {
            let pct_done = percent(done, total);
            let _ = writeln!(out, "{}", proto::progress(done, total, pct_done));
        };
        // A request line is UTF-8 by the protocol (ADR-0019). Borrow it when it
        // is, and only build a replaced copy for bytes that are not: the lossy
        // path must still answer rather than drop the connection.
        let outcome = match std::str::from_utf8(&line) {
            Ok(text) => conn.dispatch(text, &mut progress_write),
            Err(_) => {
                let lossy = String::from_utf8_lossy(&line);
                conn.dispatch(&lossy, &mut progress_write)
            }
        };
        match outcome {
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

/// The most bytes one frame may hold: the protocol's 16,384-byte payload plus
/// its optional `\r` and the `\n` that ends it.
const FRAME_LIMIT: usize = proto::MAX_LINE + 2;

/// What one read of the connection produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frame {
    /// The peer closed before a byte of a new request arrived.
    Eof,
    /// One request line. `overlong` means it exceeded the protocol's cap, so
    /// its tail was consumed and dropped rather than stored.
    Line {
        /// Whether the line was longer than a request may be.
        overlong: bool,
    },
}

/// Read one request line into `line`, never growing it past [`FRAME_LIMIT`].
///
/// `read_until` grew the buffer to whatever the peer sent: a line that never
/// ended cost the daemon one allocation per byte received, for a request the
/// framing rules were going to refuse anyway. Here the payload is stored only
/// up to the cap and the remainder of an over-long line is consumed without
/// being kept, so the *next* request still starts at a line boundary. The
/// reply and the connection's survival are what `read_until` produced; only
/// the memory is bounded.
///
/// A final line without a terminating `\n` is still a line, as before: the old
/// loop processed whatever bytes `read_until` returned at EOF.
///
/// # Errors
/// Whatever the reader raises.
fn read_frame(input: &mut dyn BufRead, line: &mut Vec<u8>) -> io::Result<Frame> {
    line.clear();
    let mut overlong = false;
    loop {
        let buf = input.fill_buf()?;
        if buf.is_empty() {
            if line.is_empty() && !overlong {
                return Ok(Frame::Eof);
            }
            return Ok(Frame::Line { overlong });
        }
        let newline = buf.iter().position(|b| *b == b'\n');
        let take = newline.unwrap_or(buf.len());
        if !overlong {
            let room = FRAME_LIMIT.saturating_sub(line.len());
            let add = take.min(room);
            if let Some(part) = buf.get(..add) {
                line.extend_from_slice(part);
            }
            if add < take {
                // Longer than a valid request can be: stop storing. The rest
                // is dropped as it arrives, below.
                overlong = true;
            }
        }
        match newline {
            Some(at) => {
                input.consume(at.saturating_add(1));
                return Ok(Frame::Line { overlong });
            }
            None => input.consume(take),
        }
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

/// Bind `socket` without displacing a daemon that is still listening.
///
/// A process that was killed leaves its socket file behind, and `bind` then
/// fails with `AddrInUse` forever. The first version of this function removed
/// the file unconditionally, which also unlinked the socket of a *live* daemon:
/// that daemon kept accepting on an inode no client could reach, and the second
/// daemon silently took over the name. Here the path is removed only when
/// nothing answers on it, and a path that is not a socket at all is left alone
/// and reported — a daemon must not delete a file it did not create.
///
/// The socket is then made owner-only. If that cannot be done the socket is
/// removed and the error returned: serving a socket with wider permissions than
/// intended is worse than not starting.
///
/// # Errors
/// A path already held by a live listener (`AddrInUse`), a path that is not a
/// socket (`AlreadyExists`), and whatever bind or `chmod` raise.
#[cfg(unix)]
fn bind_listener(socket: &Path) -> io::Result<UnixListener> {
    match std::fs::symlink_metadata(socket) {
        Ok(meta) => {
            if !meta.file_type().is_socket() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "the socket path is occupied by something that is not a socket",
                ));
            }
            if UnixStream::connect(socket).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "another daemon is already listening on this socket",
                ));
            }
            // Nothing answered: the file is a leftover from a process that is
            // gone, and removing it is the only way to bind this name again.
            std::fs::remove_file(socket)?;
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let listener = UnixListener::bind(socket)?;
    if let Err(e) = std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600)) {
        let _ = std::fs::remove_file(socket);
        return Err(e);
    }
    Ok(listener)
}

/// Bind and serve LFP/1 on a Unix socket. The listener is created by systemd
/// socket activation in production (`Accept=yes` gives one process per
/// connection); this path is for standalone runs and integration tests.
///
/// Each connection is served on its own thread. A client that keeps its
/// connection open — the shell does, so that a folder listing does not pay for
/// a new process — must not stop the next client from being accepted, and with
/// `Accept=yes` the reason is the same: connections are independent.
///
/// # Errors
/// Bind errors surface to the caller; an error accepting or serving one
/// connection does not stop the listener.
pub fn serve_unix(socket: &Path, root: Root) -> io::Result<()> {
    let listener = bind_listener(socket)?;
    // Per-user isolation is the socket's own ownership and mode, enforced by the
    // kernel: `connect` from any uid but the owner's fails before a request byte
    // exists. There is deliberately no userspace credential check here -- std's
    // `UnixStream::peer_cred` is unstable on this toolchain, and reading
    // SO_PEERCRED directly would add the crate's first third-party dependency,
    // which the dependency policy forbids (tracker §27 records the gap).
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let root = root.clone();
                std::thread::spawn(move || {
                    let Ok(reader) = s.try_clone() else {
                        return;
                    };
                    let mut reader = io::BufReader::new(reader);
                    let mut writer = s;
                    let _ = serve_stream(&mut reader, &mut writer, root);
                });
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
    fn writetext_creates_the_file_with_its_content() {
        let (dir, root) = open_root("writetext");
        let mut conn = Connection::new(root);
        let line = one_line(&mut conn, "WRITETEXT /Documents note.txt hello%20there");
        assert!(line.starts_with("path="), "got: {line}");
        assert_eq!(
            fs::read_to_string(dir.join("Documents/note.txt")).expect("written"),
            "hello there"
        );
    }

    #[test]
    fn writetext_never_overwrites_and_decodes_unicode() {
        let (dir, root) = open_root("writetext-exclusive");
        let mut conn = Connection::new(root);
        // Thai text, percent-encoded, round-trips as UTF-8.
        let encoded = litric_core::pct::encode_field("สวัสดี");
        one_line(
            &mut conn,
            &format!("WRITETEXT /Documents thai.txt {encoded}"),
        );
        assert_eq!(
            fs::read_to_string(dir.join("Documents/thai.txt")).expect("written"),
            "สวัสดี"
        );
        // The same name again is refused, and the first content survives.
        match conn.dispatch("WRITETEXT /Documents thai.txt second", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::Exists),
            other => panic!("not a refusal: {other:?}"),
        }
        assert_eq!(
            fs::read_to_string(dir.join("Documents/thai.txt")).expect("still there"),
            "สวัสดี"
        );
    }

    #[test]
    fn writetext_refuses_a_traversal_name() {
        let (_dir, root) = open_root("writetext-traversal");
        let mut conn = Connection::new(root);
        match conn.dispatch("WRITETEXT /Documents ..%2Fescape escape", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::InvalidName),
            other => panic!("not a refusal: {other:?}"),
        }
    }

    #[test]
    fn a_folder_the_user_named_with_a_space_can_be_addressed() {
        let (dir, root) = open_root("spaces");
        let mut conn = Connection::new(root);
        // The name is a free-text field and was always encoded; the path to it
        // is what could not be written down before `path_field` existed.
        one_line(&mut conn, "MKDIR /Documents My%20Documents");
        let encoded = litric_core::pct::encode("/Documents/My Documents");
        let line = one_line(&mut conn, &format!("STAT {encoded}"));
        assert!(line.contains("kind=dir"), "got: {line}");
        one_line(&mut conn, &format!("WRITETEXT {encoded} note.txt hi"));
        assert_eq!(
            fs::read_to_string(dir.join("Documents/My Documents/note.txt")).expect("written"),
            "hi"
        );
        // And the listing of the folder it lives in names it.
        match conn.dispatch(&format!("LIST {encoded}"), &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1, "{lines:?}");
                assert!(lines[0].contains("name=note.txt"), "{lines:?}");
            }
            other => panic!("not a reply: {other:?}"),
        }
    }

    #[test]
    fn a_raw_path_containing_a_percent_sign_is_not_mistaken_for_an_escape() {
        let (dir, root) = open_root("percent-name");
        let mut conn = Connection::new(root);
        // A literal `%` in a name: the client encodes the path, so the wire
        // field does not start with `/` and the escape cannot be confused with
        // a raw path.
        one_line(&mut conn, "MKDIR /Documents 100%25%20done");
        let encoded = litric_core::pct::encode("/Documents/100% done");
        let line = one_line(&mut conn, &format!("STAT {encoded}"));
        assert!(line.contains("kind=dir"), "got: {line}");
        assert!(dir.join("Documents/100% done").is_dir());
    }

    #[test]
    fn a_trashed_name_with_a_space_can_be_listed_and_restored() {
        let (dir, root) = open_root("trash-space");
        let mut conn = Connection::new(root);
        one_line(&mut conn, "WRITETEXT /Documents my%20notes.txt bye");
        one_line(
            &mut conn,
            &format!(
                "TRASH {}",
                litric_core::pct::encode("/Documents/my notes.txt")
            ),
        );
        let listed = match conn.dispatch("TRASHLIST", &mut |_, _| {}) {
            Outcome::Reply(lines) => lines,
            other => panic!("not a reply: {other:?}"),
        };
        assert_eq!(listed.len(), 1, "{listed:?}");
        let name = listed[0]
            .split(' ')
            .find_map(|f| f.strip_prefix("name="))
            .expect("name field")
            .to_owned();
        let decoded = litric_core::pct::decode_field(&name).expect("decodes");
        assert_eq!(decoded, "my notes.txt");
        let restored = one_line(&mut conn, &format!("RESTORE {name}"));
        assert!(restored.starts_with("path="), "got: {restored}");
        assert_eq!(
            fs::read_to_string(dir.join("Documents/my notes.txt")).expect("restored"),
            "bye"
        );
    }

    fn open_root_with_import(tag: &str) -> (PathBuf, PathBuf, Root) {
        let dir = temp_root(tag);
        let staging = dir.join("..staging");
        fs::create_dir_all(&staging).expect("staging");
        let mut root = Root::open(&dir, TimeZone::utc("test")).expect("root opens");
        root.set_import_dir(Some(staging.clone()));
        (dir, staging, root)
    }

    #[test]
    fn import_brings_a_staged_download_into_the_tree() {
        let (dir, staging, root) = open_root_with_import("import");
        fs::write(staging.join("report.pdf"), b"%PDF-1.7 payload").expect("stage");
        let mut conn = Connection::new(root);
        let line = one_line(&mut conn, "IMPORT /Documents report.pdf report.pdf");
        assert_eq!(line, "path=%2FDocuments%2Freport.pdf");
        assert_eq!(
            fs::read(dir.join("Documents/report.pdf")).expect("imported"),
            b"%PDF-1.7 payload"
        );
        // The staged copy is the broker's to remove, not the daemon's.
        assert!(staging.join("report.pdf").is_file());
    }

    #[test]
    fn import_never_overwrites_and_never_leaves_a_partial_file() {
        let (dir, staging, root) = open_root_with_import("import-exists");
        fs::write(dir.join("Documents/report.pdf"), b"the original").expect("original");
        fs::write(staging.join("report.pdf"), b"a newer download").expect("stage");
        let mut conn = Connection::new(root);
        match conn.dispatch("IMPORT /Documents report.pdf report.pdf", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::Exists),
            other => panic!("not a refusal: {other:?}"),
        }
        assert_eq!(
            fs::read(dir.join("Documents/report.pdf")).expect("original kept"),
            b"the original"
        );
        // A missing source leaves nothing behind either.
        match conn.dispatch("IMPORT /Documents gone.zip gone.zip", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::NotFound),
            other => panic!("not a refusal: {other:?}"),
        }
        assert!(!dir.join("Documents/gone.zip").exists());
    }

    #[test]
    fn import_refuses_every_way_out_of_the_staging_directory() {
        let (dir, staging, root) = open_root_with_import("import-escape");
        fs::write(staging.join("real.bin"), b"x").expect("stage");
        // A symlink out of the staging area, a directory, and a traversal name.
        std::os::unix::fs::symlink("/etc/passwd", staging.join("link")).expect("symlink");
        fs::create_dir_all(staging.join("folder")).expect("dir");
        let mut conn = Connection::new(root);
        match conn.dispatch("IMPORT /Documents stolen link", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::SymlinkRefused),
            other => panic!("not a refusal: {other:?}"),
        }
        match conn.dispatch("IMPORT /Documents folder folder", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::IsADirectory),
            other => panic!("not a refusal: {other:?}"),
        }
        match conn.dispatch(
            "IMPORT /Documents escape ..%2F..%2Fetc%2Fpasswd",
            &mut |_, _| {},
        ) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::InvalidName),
            other => panic!("not a refusal: {other:?}"),
        }
        assert!(!dir.join("Documents/stolen").exists());
        assert!(!dir.join("Documents/folder").exists());
    }

    #[test]
    fn import_without_a_configured_directory_says_so() {
        let (_dir, root) = open_root("import-unconfigured");
        let mut conn = Connection::new(root);
        match conn.dispatch("IMPORT /Documents a.txt a.txt", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::BadRequest),
            other => panic!("not a refusal: {other:?}"),
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
    fn every_verb_refuses_a_path_that_leaves_the_root() {
        // One hostile input of each shape a client can put on the wire, against
        // every verb whose argument shape is already pinned by another test in
        // this module. A refusal is the contract; creating something outside the
        // root is the failure this test exists to catch, so the tree is checked
        // afterwards as well as the reply.
        let (dir, root) = open_root("traversal-matrix");
        let mut conn = Connection::new(root);
        // Encoded separators and dot segments: not a name this broker accepts,
        // because a decoded `/` or a `..` segment would address something else.
        let hostile_names = ["..%2Fescape", "%2E%2E%2Fescape", "a%2F..%2Fb"];
        let hostile_paths = ["/Documents/../../escape", "/..", "/Documents/..", "/a/./b"];
        let mut refused = 0_u32;
        for name in hostile_names {
            for line in [
                format!("MKDIR /Documents {name}"),
                format!("NEWFILE /Documents {name}"),
                format!("WRITETEXT /Documents {name} x"),
            ] {
                match conn.dispatch(&line, &mut |_, _| {}) {
                    Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::InvalidName, "{line}"),
                    other => panic!("{line} was not refused: {other:?}"),
                }
                refused = refused.saturating_add(1);
            }
        }
        for path in hostile_paths {
            for line in [
                format!("STAT {path}"),
                format!("LIST {path}"),
                format!("DU {path}"),
                format!("TRASH {path}"),
            ] {
                match conn.dispatch(&line, &mut |_, _| {}) {
                    Outcome::Fail(e) => assert_eq!(e.kind(), ErrorKind::InvalidName, "{line}"),
                    other => panic!("{line} was not refused: {other:?}"),
                }
                refused = refused.saturating_add(1);
            }
        }
        assert_eq!(
            refused, 25,
            "every hostile input was tried against every verb"
        );
        assert!(
            !dir.join("escape").exists(),
            "traversal wrote outside Documents"
        );
        assert!(!dir.join("Documents/escape").exists());
        let left = fs::read_dir(dir.join("Documents")).expect("Documents exists");
        assert_eq!(
            left.count(),
            0,
            "a refused request created something under the root"
        );

        // A backslash is not a path separator on this platform: a client that
        // encodes one gets a file whose name contains it, inside the requested
        // folder. That is the honest Linux answer, and it is safe because the
        // name is one segment -- unlike an archive entry name, which may have
        // been authored on Windows and is therefore refused (see `archive.rs`).
        let line = one_line(&mut conn, "MKDIR /Documents ..%5Cescape");
        assert!(line.starts_with("path="), "got: {line}");
        assert!(
            dir.join("Documents/..\\escape").is_dir(),
            "the backslash is a literal name byte"
        );
        assert!(!dir.join("escape").exists(), "and it is still one segment");
        assert_eq!(
            fs::read_dir(dir.join("Documents"))
                .expect("Documents exists")
                .count(),
            1
        );
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
        let mut conn = Connection::new(root);
        one_line(&mut conn, "NEWFILE /Documents memo.txt");
        let trashed = one_line(&mut conn, "TRASH /Documents/memo.txt");
        assert!(trashed.starts_with("trashed=memo.txt"), "got: {trashed}");
        match conn.dispatch("TRASHLIST", &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1);
                assert!(lines[0].starts_with("t name=memo.txt "));
                // The original location is a logical path, and the wire
                // percent-encodes its separators. It is no longer the
                // `file://` URI that made other tools restore into the wrong
                // place (BUG-0009).
                assert!(lines[0].contains("original=%2FDocuments%2Fmemo.txt"));
                // The URI scheme, not the word: this assertion used to reject
                // any line containing "file", which the same line's own
                // `kind=file` field then tripped. A guard that fires on a field
                // it did not mean to describe is a guard nobody can keep green.
                assert!(!lines[0].contains("file://"), "not a URI: {}", lines[0]);
                assert!(
                    !lines[0].contains("file%3A%2F%2F"),
                    "not an encoded URI: {}",
                    lines[0]
                );
            }
            other => panic!("not a reply: {other:?}"),
        }
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
    fn restoring_an_entry_that_came_from_another_root_is_refused_not_guessed() {
        // The item and its record are both real; the location it names is not
        // ours. `/somewhere/else/stray.txt` is a valid *logical* path, so the
        // only thing standing between it and a restore into this root is the
        // flag `DiskTrash`'s reader sets. This drives the refusal through the
        // wire protocol, which is where a front end would meet it.
        let (dir, root) = open_root("restore-outside");
        let base = dir.join(crate::trash::TRASH_DIR_NAME);
        fs::create_dir_all(base.join("files")).expect("files");
        fs::create_dir_all(base.join("info")).expect("info");
        fs::write(base.join("files/stray.txt"), "x").expect("item");
        fs::write(
            base.join("info/stray.txt.trashinfo"),
            "[Trash Info]\nPath=/somewhere/else/stray.txt\n\
             DeletionDate=2026-01-01T00:00:00\n",
        )
        .expect("record");

        let mut conn = Connection::new(root);
        // It still lists, with the location it actually records.
        let listing = one_line(&mut conn, "TRASHLIST");
        assert!(
            listing.contains("original=%2Fsomewhere%2Felse%2Fstray.txt"),
            "listing must keep the recorded location: {listing}"
        );
        match conn.dispatch("RESTORE stray.txt", &mut |_, _| {}) {
            Outcome::Fail(e) => assert_eq!(e.kind().token(), "invalid-name", "got {e:?}"),
            other => panic!("a restore outside this root was not refused: {other:?}"),
        }
        // Refused, and nothing was created anywhere under the root.
        assert!(!dir.join("somewhere").exists(), "no guessed destination");
        assert!(
            base.join("files/stray.txt").exists(),
            "a refused restore must leave the item in the trash"
        );
    }

    #[test]
    fn trashlist_reports_the_original_logical_location_for_a_legacy_entry() {
        // An entry an older build wrote in the `file://` URI form must still
        // list with its real location, and must still restore (BUG-0009).
        let (dir, root) = open_root("trash-legacy");
        let mut conn = Connection::new(root);
        let base = dir.join(".local/share/Trash");
        fs::create_dir_all(base.join("files")).expect("files");
        fs::create_dir_all(base.join("info")).expect("info");
        fs::write(base.join("files/old.txt"), "old").expect("item");
        // Exactly what the pre-fix daemon wrote: the whole URI, with `/` kept
        // literal and `:` escaped.
        let legacy = crate::trashinfo::pct_encode("file:///Documents/old.txt");
        fs::write(
            base.join("info/old.txt.trashinfo"),
            format!("[Trash Info]\nPath={legacy}\nDeletionDate=2026-01-01T00:00:00\n"),
        )
        .expect("record");

        match conn.dispatch("TRASHLIST", &mut |_, _| {}) {
            Outcome::Reply(lines) => {
                assert_eq!(lines.len(), 1, "{lines:?}");
                assert!(
                    lines[0].contains("original=%2FDocuments%2Fold.txt"),
                    "got: {}",
                    lines[0]
                );
            }
            other => panic!("not a reply: {other:?}"),
        }
        let restored = one_line(&mut conn, "RESTORE old.txt");
        assert!(restored.contains("path="), "got: {restored}");
        assert!(dir.join("Documents/old.txt").exists());
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
                // ADR-0019: every hit is an entry line, `e ` first, values
                // percent-encoded — the path's `/` characters travel as %2F.
                assert!(
                    lines[0].starts_with("e path=%2FDocuments%2Freport-2026.txt "),
                    "got: {}",
                    lines[0]
                );
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
    fn read_capped_never_reads_past_the_cap() {
        use std::cell::Cell;
        use std::rc::Rc;

        /// A source that reports how many bytes were actually taken from it.
        struct Counting {
            left: usize,
            taken: Rc<Cell<usize>>,
        }

        impl io::Read for Counting {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let n = buf.len().min(self.left).min(7_919);
                self.left -= n;
                for slot in buf.iter_mut().take(n) {
                    *slot = b'x';
                }
                self.taken.set(self.taken.get() + n);
                Ok(n)
            }
        }

        let taken = Rc::new(Cell::new(0));
        let (bytes, truncated) = read_capped(
            Counting {
                left: 8 * 1024 * 1024,
                taken: Rc::clone(&taken),
            },
            TEXT_CAP,
        )
        .expect("the read succeeds");
        assert_eq!(bytes.len(), TEXT_CAP, "the reply carries the cap");
        assert!(truncated, "an 8 MiB source is truncated");
        assert_eq!(
            taken.get(),
            TEXT_CAP + 1,
            "the source is asked for one byte past the cap and no more"
        );
    }

    #[test]
    fn read_capped_keeps_short_files_whole() {
        let source = io::Cursor::new(b"short".to_vec());
        let (bytes, truncated) = read_capped(source, TEXT_CAP).expect("read");
        assert_eq!(bytes, b"short");
        assert!(!truncated, "a file under the cap is not truncated");
    }

    #[test]
    fn readtext_caps_a_file_larger_than_the_viewer_limit() {
        let (dir_path, root) = open_root("readtext-cap");
        let mut conn = Connection::new(root);
        let body = vec![b'a'; TEXT_CAP + 4_096];
        fs::write(dir_path.join("Documents").join("big.log"), body).expect("write");
        let line = one_line(&mut conn, "READTEXT /Documents/big.log");
        // `a` is unreserved, so the encoded text is exactly the bytes kept.
        let head = "binary=0 truncated=1 text=";
        assert!(
            line.starts_with(head),
            "got: {}",
            &line[..head.len().min(line.len())]
        );
        assert_eq!(
            line.len() - head.len(),
            TEXT_CAP,
            "a truncated reply carries exactly the cap"
        );
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

    #[cfg(unix)]
    #[test]
    fn the_socket_is_owner_only_and_a_live_daemons_socket_is_not_stolen() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = temp_root("socket-bind");
        let socket = dir.join("files.sock");
        let first = bind_listener(&socket).expect("first bind");
        use std::os::unix::fs::MetadataExt as _;
        let meta = std::fs::metadata(&socket).expect("the socket exists");
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the socket must be owner-only");
        // The owner is this process's user: the temp directory was made by the
        // same process, and on Unix the kernel refuses `connect` from every
        // other uid to a 0600 socket, before any request is read.
        let dir_uid = std::fs::metadata(&dir).expect("the temp dir").uid();
        assert_eq!(
            meta.uid(),
            dir_uid,
            "the socket belongs to the daemon's user"
        );
        // A second daemon must refuse instead of unlinking a socket that is
        // still being served, and the first must still answer on it.
        let second = bind_listener(&socket).expect_err("a live socket is not replaced");
        assert_eq!(second.kind(), io::ErrorKind::AddrInUse);
        UnixStream::connect(&socket).expect("the first listener still answers");
        drop(first);
        let _ = std::fs::remove_file(&socket);
    }

    #[cfg(unix)]
    #[test]
    fn a_stale_socket_file_is_replaced() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = temp_root("socket-stale");
        let socket = dir.join("files.sock");
        {
            // A listener that was killed: the file stays, nothing answers.
            let dead = UnixListener::bind(&socket).expect("bind");
            drop(dead);
        }
        assert!(socket.exists(), "the leftover file is still there");
        let listener = bind_listener(&socket).expect("a stale socket is replaced");
        UnixStream::connect(&socket).expect("the new listener answers");
        let mode = std::fs::metadata(&socket)
            .expect("stat")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        drop(listener);
        let _ = std::fs::remove_file(&socket);
    }

    #[cfg(unix)]
    #[test]
    fn a_path_that_is_not_a_socket_is_never_deleted() {
        let dir = temp_root("socket-not-a-socket");
        let path = dir.join("important.txt");
        std::fs::write(&path, b"do not delete").expect("write");
        let err = bind_listener(&path).expect_err("a plain file is occupied");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read(&path).expect("still there"),
            b"do not delete",
            "a daemon must not unlink a file it did not create"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_connection_from_the_daemons_own_user_is_served_over_a_real_socket() {
        use std::io::{BufRead as _, BufReader, Write as _};
        let dir = temp_root("socket-serve");
        let socket = dir.join("files.sock");
        let (_, root) = open_root("socket-serve-root");
        let path = socket.clone();
        std::thread::spawn(move || {
            let _ = serve_unix(&path, root);
        });
        // `serve_unix` binds synchronously before it accepts, but a spawned
        // thread means the path may not exist for a moment.
        let mut stream = None;
        for _ in 0..200 {
            match UnixStream::connect(&socket) {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
        let stream = stream.expect("the daemon accepted a same-user connection");
        let mut writer = stream.try_clone().expect("clone");
        writer.write_all(b"HELLO 1\n").expect("write");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");
        assert!(line.starts_with("OK "), "got: {line:?}");
        let _ = std::fs::remove_file(&socket);
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

    /// Drive `serve_stream` over a byte slice and return the reply text.
    fn run_stream(root: Root, request: &[u8]) -> String {
        let mut input = io::BufReader::new(request);
        let mut output = Vec::new();
        serve_stream(&mut input, &mut output, root).expect("serve");
        String::from_utf8(output).expect("utf-8")
    }

    #[test]
    fn a_request_at_the_line_limit_is_framed_and_one_byte_over_is_refused() {
        // Exactly `MAX_LINE` payload bytes: framing accepts it, so the answer
        // is about the path, not about the line length.
        let pad = "x".repeat(proto::MAX_LINE - "STAT ".len());
        let (_dir, root) = open_root("line-limit");
        let at_limit = format!("STAT {pad}\n");
        assert_eq!(at_limit.len(), proto::MAX_LINE + 1);
        let too_long = pct::encode_field("request line too long");
        let text = run_stream(root, at_limit.as_bytes());
        assert!(
            !text.contains(&too_long),
            "a line at the limit must be framed, got: {text}"
        );
        // CRLF is the protocol's normal terminator and must be accepted at the
        // limit too.
        let (_dir, root) = open_root("line-limit-crlf");
        let text = run_stream(root, format!("STAT {pad}\r\n").as_bytes());
        assert!(
            !text.contains(&too_long),
            "a CRLF line at the limit must be framed, got: {text}"
        );
        // One byte more is refused, and the connection keeps serving the next
        // request: the tail of the over-long line is drained, not parsed.
        let (_dir, root) = open_root("line-over");
        let over = format!(
            "STAT {}x\nHELLO 1\n",
            "x".repeat(proto::MAX_LINE - "STAT ".len())
        );
        assert_eq!(over.len(), proto::MAX_LINE + 2 + "HELLO 1\n".len());
        let text = run_stream(root, over.as_bytes());
        assert!(text.contains(&too_long), "got: {text}");
        assert!(
            text.contains("OK 1"),
            "the request after an over-long line must be served, got: {text}"
        );
    }

    #[test]
    fn an_endless_line_is_discarded_instead_of_buffered() {
        static CHUNK: [u8; 65_536] = [b'A'; 65_536];

        struct Endless {
            left: usize,
        }

        impl io::Read for Endless {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "the framing reads through BufRead only",
                ))
            }
        }

        impl io::BufRead for Endless {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                Ok(&CHUNK[..self.left.min(CHUNK.len())])
            }

            fn consume(&mut self, amt: usize) {
                self.left = self.left.saturating_sub(amt);
            }
        }

        // Four mebibytes with no newline: `read_until` would have stored every
        // byte of it before refusing.
        let mut input = Endless {
            left: 4 * 1024 * 1024,
        };
        let mut line = Vec::with_capacity(1024);
        let frame = read_frame(&mut input, &mut line).expect("read");
        assert_eq!(frame, Frame::Line { overlong: true });
        assert!(
            line.len() <= proto::MAX_LINE + 2,
            "the line buffer grew to {} bytes",
            line.len()
        );
        assert_eq!(
            input.left, 0,
            "the rest of the line is drained, not re-read"
        );
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
