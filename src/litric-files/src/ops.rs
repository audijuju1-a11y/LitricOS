//! [`Root`]: resolution of logical paths to host paths without ever following
//! a symlink, and the file operations the LFP/1 commands are built on.
//!
//! The confinement rule (ADR-0019 §3) is enforced by construction: every
//! path that reaches the filesystem is produced by walking the logical path
//! one segment at a time, checking each intermediate with
//! `symlink_metadata` and requiring a *real* directory. A symlink anywhere
//! in the chain is refused with `symlink-refused` rather than followed. A
//! symlink as the final component is reported as a link and operated on as
//! a link.
//!
//! Residual risk accepted (ADR-0019): the check-then-act window against a
//! process already running as the same user. Closing it needs
//! `openat`/`O_NOFOLLOW`, which needs `unsafe` or libc; both are out of
//! scope for this crate by workspace rule.

use crate::path::LogicalPath;
use crate::{Error, ErrorKind};
use std::fs;
use std::io;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The most entries one directory listing will report before it says
/// `truncated=1`.
pub const LIST_CAP: usize = 10_000;

/// The most files one `DU`/`SEARCH`/`RECENT` walk will visit before it
/// declares itself partial. A walk with no bound is a denial of service
/// waiting for a large directory tree.
pub const WALK_CAP: usize = 50_000;

/// How deep a recursive copy will descend. Symlink cycles cannot happen —
/// links are never followed — but a pathological tree still should not
/// drive the stack.
const COPY_DEPTH_CAP: usize = 64;

/// The kind a filesystem object is, as the protocol spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A directory.
    Dir,
    /// A regular file.
    File,
    /// A symbolic link (reported as the link; never dereferenced).
    Link,
    /// Anything else — sockets, FIFOs, devices. Litric never creates these.
    Other,
}

impl EntryKind {
    /// The `kind=` wire token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Dir => "dir",
            Self::File => "file",
            Self::Link => "link",
            Self::Other => "other",
        }
    }
}

/// One directory entry or one `STAT` result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatInfo {
    /// The final segment, not the whole path.
    pub name: String,
    /// What the object is.
    pub kind: EntryKind,
    /// Size in bytes; for a link, the length of the target path.
    pub size: u64,
    /// Modification time, Unix seconds.
    pub mtime: u64,
    /// Permission bits (the low 12 bits of the st_mode field).
    pub mode: u32,
    /// Whether the name starts with a dot.
    pub hidden: bool,
    /// Where a link points, as recorded (may be relative).
    pub target: Option<String>,
}

/// The user's home as seen by this daemon.
///
/// Constructed only through [`Root::open`], which refuses a host root that
/// is not a real directory.
#[derive(Debug, Clone)]
pub struct Root {
    host_root: PathBuf,
    zone: litric_locale::TimeZone,
}

impl Root {
    /// Open the daemon's root at `host` (e.g. `/var/home/litric`).
    ///
    /// The root itself is checked with `symlink_metadata`: it must exist and
    /// be a real directory, not a link into it.
    ///
    /// # Errors
    /// `not-found` when the root is absent, `not-a-directory` when it is not
    /// a directory, `symlink-refused` when it is a symlink.
    pub fn open(host: &Path, zone: litric_locale::TimeZone) -> Result<Self, Error> {
        let meta = fs::symlink_metadata(host).map_err(|e| Error::from_io("open root", &e))?;
        if meta.file_type().is_symlink() {
            return Err(Error::new(
                ErrorKind::SymlinkRefused,
                "the daemon root itself must not be a symlink",
            ));
        }
        if !meta.is_dir() {
            return Err(Error::new(
                ErrorKind::NotADirectory,
                "the daemon root is not a directory",
            ));
        }
        Ok(Self {
            host_root: host.to_path_buf(),
            zone,
        })
    }

    /// The host root, for the trash layout and the listener path.
    #[must_use]
    pub fn host_root(&self) -> &Path {
        &self.host_root
    }

    /// The time zone used for human-facing dates (trash deletion dates).
    #[must_use]
    pub const fn zone(&self) -> &litric_locale::TimeZone {
        &self.zone
    }

    /// Validate one path segment against the naming rules of
    /// [`crate::path`], so every command shares exactly one definition of a
    /// legal name.
    ///
    /// # Errors
    /// `invalid-name` when the rules refuse it.
    pub fn validate_segment(name: &str) -> Result<(), Error> {
        LogicalPath::root()
            .join(name)
            .map(|_| ())
            .map_err(|e| Error::new(ErrorKind::InvalidName, format!("bad name: {e}")))
    }

    /// Walk `logical` from the root, requiring every intermediate to be a
    /// real directory. The final component may be anything that exists.
    ///
    /// # Errors
    /// `not-found` for a missing component, `not-a-directory` for an
    /// intermediate that is not a directory, `symlink-refused` for a
    /// symlink in the chain.
    pub fn resolve(&self, logical: &LogicalPath) -> Result<PathBuf, Error> {
        let mut current = self.host_root.clone();
        if !logical.as_str().is_empty() {
            Self::check_root_dir(&current)?;
        }
        // Peekable, not collected: every operation resolves paths, so the
        // walk allocates nothing. A component with another one behind it is
        // intermediate and must be a real directory (ADR-0019 §3); the last
        // component may be anything, which is how STAT and DELETE accept a
        // file. Without this check the mistake surfaced later as a bare
        // `not-found` from the join below, which named the wrong problem.
        let mut segments = logical.segments().peekable();
        while let Some(segment) = segments.next() {
            current = current.join(segment);
            let meta = fs::symlink_metadata(&current).map_err(|_| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("no such item: {}", logical.as_str()),
                )
            })?;
            if meta.file_type().is_symlink() {
                return Err(Error::new(
                    ErrorKind::SymlinkRefused,
                    format!("a symlink stands in the way: {}", logical.as_str()),
                ));
            }
            if segments.peek().is_some() && !meta.is_dir() {
                return Err(Error::new(
                    ErrorKind::NotADirectory,
                    format!("a file stands in the way: {}", logical.as_str()),
                ));
            }
        }
        Ok(current)
    }

    /// Walk to the parent of `logical` and return the host directory plus
    /// the final segment, without requiring the final segment to exist.
    /// An existing final component that is a symlink is *reported*, not
    /// followed, so create operations fail on it instead of writing
    /// through it.
    ///
    /// # Errors
    /// As [`Root::resolve`] for the parent chain; the final component is
    /// not checked.
    pub fn resolve_parent(&self, logical: &LogicalPath) -> Result<(PathBuf, String), Error> {
        let parent = logical
            .parent()
            .ok_or_else(|| Error::new(ErrorKind::NotADirectory, "the root has no parent"))?;
        let dir = self.resolve(&parent)?;
        Ok((dir, logical.file_name().to_owned()))
    }

    /// The host directory for a logical directory, creating it when missing.
    ///
    /// [`Root::resolve`] refuses a path whose components do not exist, which is
    /// right for every operation on an item that is supposed to be there. It is
    /// wrong for extraction: an archive is allowed to store `docs/report.txt`
    /// without also storing `docs/`, and the parent then has to be created as
    /// part of writing the entry. Before this existed, such an archive failed
    /// with `not-found`, naming a folder the archive had every right not to
    /// list.
    ///
    /// The components are still walked one at a time and each existing one is
    /// checked with `symlink_metadata`, so a link standing where a folder should
    /// be is refused rather than written through — the confinement is the same,
    /// only the create is new.
    ///
    /// # Errors
    /// `symlink-refused` / `not-a-directory` for a component in the way; OS
    /// errors map through [`Error::from_io`].
    pub fn resolve_dir_creating(&self, logical: &LogicalPath) -> Result<PathBuf, Error> {
        let mut host = self.host_root.clone();
        for segment in logical.as_str().split('/').filter(|s| !s.is_empty()) {
            host.push(segment);
        }
        self.ensure_dir_chain(&host)?;
        Ok(host)
    }

    /// Create every missing directory between the root and `host_dir`,
    /// verifying each existing component with `symlink_metadata` so a link
    /// in the way is refused rather than followed. Used by restore, which
    /// must be able to bring back a file whose parent folders were also
    /// deleted.
    ///
    /// # Errors
    /// `symlink-refused` when a component exists but is a link,
    /// `not-a-directory` when it exists but is not a directory; OS errors
    /// map through [`Error::from_io`].
    pub fn ensure_dir_chain(&self, host_dir: &Path) -> Result<(), Error> {
        let rel = host_dir
            .strip_prefix(&self.host_root)
            .map_err(|_| Error::new(ErrorKind::PermissionDenied, "outside the daemon root"))?;
        let mut current = self.host_root.clone();
        Self::check_root_dir(&current)?;
        for segment in rel.components().filter_map(|c| c.as_os_str().to_str()) {
            current = current.join(segment);
            match fs::symlink_metadata(&current) {
                Ok(meta) => {
                    if meta.file_type().is_symlink() {
                        return Err(Error::new(
                            ErrorKind::SymlinkRefused,
                            format!("a symlink stands in the way: {}", segment),
                        ));
                    }
                    if !meta.is_dir() {
                        return Err(Error::new(
                            ErrorKind::NotADirectory,
                            format!("a non-directory stands in the way: {}", segment),
                        ));
                    }
                }
                Err(_) => {
                    fs::create_dir(&current).map_err(|e| Error::from_io("create parent", &e))?;
                }
            }
        }
        Ok(())
    }

    fn check_root_dir(root: &Path) -> Result<(), Error> {
        let meta = fs::symlink_metadata(root).map_err(|e| Error::from_io("root", &e))?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(Error::new(
                ErrorKind::NotADirectory,
                "the daemon root vanished or was replaced",
            ));
        }
        Ok(())
    }

    /// One entry description from a host path, using `symlink_metadata` so
    /// a link is described as a link.
    ///
    /// # Errors
    /// [`Error::from_io`] when the metadata read fails.
    pub fn stat_host(&self, host: &Path, name: &str) -> Result<StatInfo, Error> {
        let meta = fs::symlink_metadata(host).map_err(|e| Error::from_io("stat", &e))?;
        let file_type = meta.file_type();
        let kind = if file_type.is_symlink() {
            EntryKind::Link
        } else if file_type.is_dir() {
            EntryKind::Dir
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };
        let target = if file_type.is_symlink() {
            Some(
                fs::read_link(host)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            )
        } else {
            None
        };
        Ok(StatInfo {
            name: name.to_owned(),
            kind,
            size: meta.len(),
            mtime: unix_time_of(&meta),
            mode: meta.permissions().mode() & 0o7777,
            hidden: name.starts_with('.'),
            target,
        })
    }

    /// `STAT` a logical path.
    ///
    /// # Errors
    /// As [`Root::resolve`], then [`Root::stat_host`].
    pub fn stat(&self, logical: &LogicalPath) -> Result<StatInfo, Error> {
        let host = self.resolve(logical)?;
        self.stat_host(&host, logical.file_name())
    }

    /// List a directory, sorted by name, capped at [`LIST_CAP`]. The bool
    /// is the `truncated` flag.
    ///
    /// # Errors
    /// As [`Root::resolve`]; `not-a-directory` when the final component is
    /// not a directory.
    pub fn list_dir(
        &self,
        logical: &LogicalPath,
        hidden: bool,
    ) -> Result<(Vec<StatInfo>, bool), Error> {
        let host = self.resolve(logical)?;
        let meta = fs::symlink_metadata(&host).map_err(|e| Error::from_io("list", &e))?;
        if meta.file_type().is_symlink() {
            return Err(Error::new(
                ErrorKind::SymlinkRefused,
                "listing follows no links",
            ));
        }
        if !meta.is_dir() {
            return Err(Error::new(
                ErrorKind::NotADirectory,
                "that item is not a directory",
            ));
        }
        let mut names: Vec<String> = fs::read_dir(&host)
            .map_err(|e| Error::from_io("read dir", &e))?
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(String::from))
            })
            .collect();
        names.sort();
        let truncated = names.len() > LIST_CAP;
        let out = names
            .into_iter()
            .take(LIST_CAP)
            .filter(|n| hidden || !n.starts_with('.'))
            .map(|n| {
                self.stat_host(&host.join(&n), &n)
                    .unwrap_or_else(|_| hidden_placeholder(&n))
            })
            .collect();
        Ok((out, truncated))
    }

    /// `DU`: bytes, files, directories, and whether the walk hit
    /// [`WALK_CAP`]. Links contribute their own length, not their target's.
    ///
    /// # Errors
    /// As [`Root::resolve`].
    pub fn measure(&self, logical: &LogicalPath) -> Result<(u64, u64, u64, bool), Error> {
        let host = self.resolve(logical)?;
        let mut acc = Measure {
            bytes: 0,
            files: 0,
            dirs: 0,
            visited: 0,
            partial: false,
        };
        Self::measure_into(&host, &mut acc, 0);
        Ok((acc.bytes, acc.files, acc.dirs, acc.partial))
    }

    fn measure_into(host: &Path, acc: &mut Measure, depth: usize) {
        if acc.visited >= WALK_CAP || depth > COPY_DEPTH_CAP {
            acc.partial = true;
            return;
        }
        let Ok(meta) = fs::symlink_metadata(host) else {
            return; // raced with a delete; count what is still there
        };
        acc.visited = acc.visited.saturating_add(1);
        if meta.file_type().is_symlink() || !meta.is_dir() {
            acc.bytes = acc.bytes.saturating_add(meta.len());
            if meta.file_type().is_symlink() {
                // counted as one file-sized object, no target walk
            } else {
                acc.files = acc.files.saturating_add(1);
            }
            return;
        }
        acc.dirs = acc.dirs.saturating_add(1);
        let Ok(read) = fs::read_dir(host) else {
            acc.partial = true;
            return;
        };
        for entry in read.filter_map(|e| e.ok()) {
            if acc.visited >= WALK_CAP {
                acc.partial = true;
                return;
            }
            Self::measure_into(&entry.path(), acc, depth.saturating_add(1));
        }
    }

    /// `MKDIR`: create `name` inside `parent`. Never overwrites, never
    /// writes through an existing symlink at that name (`create_dir` fails
    /// on any existing entry).
    ///
    /// # Errors
    /// `invalid-name` for a bad segment, `exists` when the name is taken,
    /// chain errors from [`Root::resolve_parent`].
    pub fn mkdir(&self, parent: &LogicalPath, name: &str) -> Result<PathBuf, Error> {
        Self::validate_segment(name)?;
        let dir = self.resolve(parent)?;
        let target = dir.join(name);
        fs::create_dir(&target).map_err(|e| Error::from_io("mkdir", &e))?;
        Ok(target)
    }

    /// `NEWFILE`: create an empty file. `create_new` is O_EXCL, which
    /// fails on an existing entry *including a symlink* — the no-follow
    /// guarantee for creation comes from the syscall, not a check.
    ///
    /// # Errors
    /// As [`Root::mkdir`].
    pub fn new_file(&self, parent: &LogicalPath, name: &str) -> Result<PathBuf, Error> {
        Self::validate_segment(name)?;
        let dir = self.resolve(parent)?;
        let target = dir.join(name);
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|e| Error::from_io("new file", &e))?;
        Ok(target)
    }

    /// `RENAME` within one directory. `rename(2)` never follows symlinks,
    /// so a link is renamed as a link. The destination is checked first
    /// because ADR-0019 renames never overwrite; the check-then-act window
    /// on that check is the accepted residual risk.
    ///
    /// # Errors
    /// `invalid-name` for a bad new name, `exists` when the destination is
    /// occupied, chain errors for the source.
    pub fn rename(&self, logical: &LogicalPath, new_name: &str) -> Result<PathBuf, Error> {
        Self::validate_segment(new_name)?;
        let (parent_host, _) = self.resolve_parent(logical)?;
        let dest = parent_host.join(new_name);
        if fs::symlink_metadata(&dest).is_ok() {
            return Err(Error::new(
                ErrorKind::Exists,
                "an item with that name already exists",
            ));
        }
        let src = self.resolve(logical)?;
        fs::rename(&src, &dest).map_err(|e| Error::from_io("rename", &e))?;
        Ok(dest)
    }

    /// Whether `candidate` is `dir` itself or lives inside it. Used by the
    /// "a directory can never be moved into itself" rule.
    #[must_use]
    pub fn is_within(host: &Path, dir: &Path) -> bool {
        host.starts_with(dir)
    }

    /// `COPY` one item into `dest_dir` (a resolved host directory).
    /// Returns the host path created. `into_source_dir` selects the
    /// ADR-0019 rule that copying into the source's own directory always
    /// keeps both under the `name - Copy.ext` name.
    ///
    /// # Errors
    /// The ADR-0019 kinds; `policy` decides what an occupied name means.
    pub fn copy_item(
        &self,
        logical: &LogicalPath,
        dest_dir: &Path,
        policy: crate::Policy,
        into_source_dir: bool,
    ) -> Result<PathBuf, Error> {
        let src = self.resolve(logical)?;
        let meta = fs::symlink_metadata(&src).map_err(|e| Error::from_io("copy", &e))?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            // A file or a link: one name decision, one copy.
        } else if Self::is_within(dest_dir, &src) && !into_source_dir {
            return Err(Error::new(
                ErrorKind::IsADirectory,
                "a directory cannot be copied into itself",
            ));
        }
        let base = logical.file_name().to_owned();
        let (name, action) = self.pick_name(dest_dir, &base, policy, into_source_dir)?;
        if action == NameAction::Skip {
            return Err(Error::new(ErrorKind::Exists, "destination name is taken"));
        }
        let dest = dest_dir.join(&name);
        if action == NameAction::Replace {
            // `Policy::Replace` promises the existing item is gone only once
            // the new one is complete, so the copy is staged next to the
            // destination and swapped in afterwards. Deleting first would
            // lose the old item whenever the copy fails (a special file in
            // the tree, a full disk, a permission), which is exactly the
            // failure the promise exists to prevent.
            let staged = self.stage_copy(&src, dest_dir)?;
            self.install_staged(&staged, &dest)?;
            return Ok(dest);
        }
        Self::copy_into(&src, &dest, 0)?;
        Ok(dest)
    }

    /// Copy `src` to a hidden staging name inside `dir` and return it.
    /// Nothing else in `dir` is touched, so a failure here leaves the
    /// directory exactly as it was.
    ///
    /// # Errors
    /// As [`Root::copy_into`]; a failed copy removes its own staging entry.
    fn stage_copy(&self, src: &Path, dir: &Path) -> Result<PathBuf, Error> {
        let staged = self.staging_name(dir)?;
        match Self::copy_into(src, &staged, 0) {
            Ok(()) => Ok(staged),
            Err(e) => {
                // The staging entry is this operation's own garbage. If even
                // the cleanup fails, the destination is still untouched; the
                // leftover is hidden and named as Litric's own.
                let _ = self.remove_host(&staged);
                Err(e)
            }
        }
    }

    /// A free `.litric-staged-N` name inside `dir`. Short on purpose: an
    /// original name up to 255 bytes must not push the staging name past
    /// the segment limit. Choosing the name is a check-then-act step like
    /// every other name in this module (ADR-0019 residual risk); what the
    /// staging buys is that the *destination* is untouched until the copy
    /// is complete, which holds whether or not this check raced.
    fn staging_name(&self, dir: &Path) -> Result<PathBuf, Error> {
        for n in 0..64u32 {
            let candidate = dir.join(format!(".litric-staged-{n}"));
            if fs::symlink_metadata(&candidate).is_err() {
                return Ok(candidate);
            }
        }
        Err(Error::new(
            ErrorKind::Exists,
            "no free staging name in the destination directory",
        ))
    }

    /// Put a *complete* `staged` item at `dest`, replacing whatever stands
    /// there. One `rename(2)` is atomic for a file or a link; an occupied
    /// directory cannot be replaced in one step, so it is moved aside, the
    /// staged item renamed in, and the old tree removed only after that.
    /// Until the last step every move is reversible, so no failure here can
    /// lose the item the user asked to replace.
    ///
    /// # Errors
    /// [`Error::from_io`] when the destination cannot be vacated; the
    /// message says where the previous item is preserved if it could not be
    /// put back.
    fn install_staged(&self, staged: &Path, dest: &Path) -> Result<(), Error> {
        if fs::rename(staged, dest).is_ok() {
            return Ok(());
        }
        let Some(parent) = dest.parent() else {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "the destination has no parent directory",
            ));
        };
        let backup = self.staging_name(parent)?;
        fs::rename(dest, &backup).map_err(|e| Error::from_io("replace", &e))?;
        match fs::rename(staged, dest) {
            Ok(()) => {
                // The new item is in place; the old one is now only garbage.
                let _ = self.remove_host(&backup);
                Ok(())
            }
            Err(e) => match fs::rename(&backup, dest) {
                Ok(()) => Err(Error::from_io("replace", &e)),
                Err(_) => Err(Error::new(
                    ErrorKind::Io,
                    format!(
                        "replace failed and the previous item is preserved at {}",
                        backup.display()
                    ),
                )),
            },
        }
    }

    /// `MOVE` one item into `dest_dir`. Rename when possible; copy-then-
    /// delete across filesystems (`EXDEV`). The same-into-itself rule is
    /// checked before anything happens.
    ///
    /// # Errors
    /// As [`Root::copy_item`].
    pub fn move_item(
        &self,
        logical: &LogicalPath,
        dest_dir: &Path,
        policy: crate::Policy,
        into_source_dir: bool,
    ) -> Result<PathBuf, Error> {
        let src = self.resolve(logical)?;
        let meta = fs::symlink_metadata(&src).map_err(|e| Error::from_io("move", &e))?;
        let is_dir = !meta.file_type().is_symlink() && meta.is_dir();
        if is_dir && Self::is_within(dest_dir, &src) && !into_source_dir {
            return Err(Error::new(
                ErrorKind::IsADirectory,
                "a directory cannot be moved into itself",
            ));
        }
        if is_dir && dest_dir == src {
            return Err(Error::new(
                ErrorKind::IsADirectory,
                "that is the directory itself",
            ));
        }
        let base = logical.file_name().to_owned();
        let (name, action) = self.pick_name(dest_dir, &base, policy, into_source_dir)?;
        if action == NameAction::Skip {
            return Err(Error::new(ErrorKind::Exists, "destination name is taken"));
        }
        let dest = dest_dir.join(&name);
        if action == NameAction::Replace {
            // As in `copy_item`: the occupied destination is only removed
            // once the new item is complete and in place.
            self.move_staged(&src, dest_dir, &dest)?;
            return Ok(dest);
        }
        match fs::rename(&src, &dest) {
            Ok(()) => Ok(dest),
            Err(e) if is_exdev(&e) => {
                // Cross-filesystem: copy, then delete the source. A crash
                // between the two leaves a copy, not a loss — the safe
                // ordering.
                Self::copy_into(&src, &dest, 0)?;
                self.remove_host(&src)?;
                Ok(dest)
            }
            Err(e) => Err(Error::from_io("move", &e)),
        }
    }

    /// `MOVE` onto an occupied name without destroying the occupant before
    /// the source is safe. A rename replaces a file or link atomically; an
    /// occupied directory is moved aside and restored if the swap fails.
    /// Across filesystems the source is copied to a staging name first and
    /// the source is deleted only once that copy is in place.
    ///
    /// # Errors
    /// The rename error joined with whatever the replacement attempt said;
    /// the error kind is the rename's.
    fn move_staged(&self, src: &Path, dest_dir: &Path, dest: &Path) -> Result<(), Error> {
        match fs::rename(src, dest) {
            Ok(()) => Ok(()),
            Err(e) if is_exdev(&e) => {
                let staged = self.stage_copy(src, dest_dir)?;
                self.install_staged(&staged, dest)?;
                self.remove_host(src)
            }
            Err(e) => {
                let first = Error::from_io("move", &e);
                match self.install_staged(src, dest) {
                    Ok(()) => Ok(()),
                    Err(second) => Err(Error::new(
                        first.kind(),
                        format!("{}. {}", first.message(), second.message()),
                    )),
                }
            }
        }
    }

    fn pick_name(
        &self,
        dest_dir: &Path,
        base: &str,
        policy: crate::Policy,
        into_source_dir: bool,
    ) -> Result<(String, NameAction), Error> {
        let occupied = |n: &str| fs::symlink_metadata(dest_dir.join(n)).is_ok();
        if !occupied(base) {
            return Ok((base.to_owned(), NameAction::Plain));
        }
        if into_source_dir {
            let alt = copy_of_name(base);
            if !occupied(&alt) {
                return Ok((alt, NameAction::Plain));
            }
            return numbered_name(base, &occupied)
                .map(|n| (n, NameAction::Plain))
                .ok_or_else(|| Error::new(ErrorKind::Exists, "no free numbered name"));
        }
        match policy {
            crate::Policy::Skip => Ok((String::new(), NameAction::Skip)),
            crate::Policy::Replace => Ok((base.to_owned(), NameAction::Replace)),
            crate::Policy::KeepBoth => numbered_name(base, &occupied)
                .map(|n| (n, NameAction::Plain))
                .ok_or_else(|| Error::new(ErrorKind::Exists, "no free numbered name")),
        }
    }

    fn copy_into(src: &Path, dest: &Path, depth: usize) -> Result<(), Error> {
        if depth > COPY_DEPTH_CAP {
            return Err(Error::new(ErrorKind::Limit, "tree is too deep to copy"));
        }
        let meta = fs::symlink_metadata(src).map_err(|e| Error::from_io("copy", &e))?;
        if meta.file_type().is_symlink() {
            let target = fs::read_link(src).map_err(|e| Error::from_io("read link", &e))?;
            symlink(target, dest).map_err(|e| Error::from_io("recreate link", &e))?;
            return Ok(());
        }
        if meta.is_dir() {
            fs::create_dir(dest).map_err(|e| Error::from_io("create dir", &e))?;
            let read = fs::read_dir(src).map_err(|e| Error::from_io("read dir", &e))?;
            for entry in read.filter_map(|e| e.ok()) {
                Self::copy_into(
                    &entry.path(),
                    &dest.join(entry.file_name()),
                    depth.saturating_add(1),
                )?;
            }
            return Ok(());
        }
        if !meta.is_file() {
            return Err(Error::new(
                ErrorKind::UnsupportedArchive,
                "only files, directories and links are copyable",
            ));
        }
        fs::copy(src, dest).map_err(|e| Error::from_io("copy file", &e))?;
        Ok(())
    }

    /// Remove whatever stands at `host` — file, link, or directory tree.
    /// `remove_dir_all` on Unix refuses to follow symlinks inside the tree.
    ///
    /// # Errors
    /// [`Error::from_io`].
    pub fn remove_host(&self, host: &Path) -> Result<(), Error> {
        let meta = fs::symlink_metadata(host).map_err(|e| Error::from_io("remove", &e))?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            fs::remove_file(host).map_err(|e| Error::from_io("remove file", &e))
        } else {
            fs::remove_dir_all(host).map_err(|e| Error::from_io("remove tree", &e))
        }
    }

    /// `DELETE` a logical path permanently. The caller confirms with the
    /// user; this layer only deletes.
    ///
    /// # Errors
    /// As [`Root::resolve`], then [`Root::remove_host`].
    pub fn delete(&self, logical: &LogicalPath) -> Result<(), Error> {
        let host = self.resolve(logical)?;
        self.remove_host(&host)
    }

    /// Resolve a logical path and describe the entry found there; also
    /// count the items inside when it is a directory (the `STAT` reply's
    /// `items=` field).
    ///
    /// # Errors
    /// As [`Root::resolve`].
    pub fn stat_with_items(&self, logical: &LogicalPath) -> Result<(StatInfo, u64), Error> {
        let host = self.resolve(logical)?;
        let info = self.stat_host(&host, logical.file_name())?;
        let items = if info.kind == EntryKind::Dir {
            u64::try_from(
                fs::read_dir(&host)
                    .map(|rd| rd.filter_map(|e| e.ok()).count())
                    .unwrap_or(0),
            )
            .unwrap_or(0)
        } else {
            0
        };
        Ok((info, items))
    }

    /// Walk from `logical` collecting entries whose name contains `query`
    /// (case-insensitively), at most 500 results. The bool is `truncated`.
    ///
    /// # Errors
    /// As [`Root::resolve`] for the start point; an unreadable subtree is
    /// skipped rather than failing the whole search.
    pub fn search(
        &self,
        logical: &LogicalPath,
        query: &str,
    ) -> Result<(Vec<(String, StatInfo)>, bool), Error> {
        let start = self.resolve(logical)?;
        let needle = query.to_lowercase();
        let mut out = Vec::new();
        let mut truncated = false;
        self.search_into(
            &start,
            logical.as_str(),
            &needle,
            &mut out,
            &mut truncated,
            0,
        );
        Ok((out, truncated))
    }

    fn search_into(
        &self,
        host: &Path,
        rel: &str,
        needle: &str,
        out: &mut Vec<(String, StatInfo)>,
        truncated: &mut bool,
        depth: usize,
    ) {
        if *truncated || depth > COPY_DEPTH_CAP || out.len() >= 500 {
            *truncated = *truncated || out.len() >= 500;
            return;
        }
        let Ok(read) = fs::read_dir(host) else {
            return;
        };
        for entry in read.filter_map(|e| e.ok()) {
            if out.len() >= 500 {
                *truncated = true;
                return;
            }
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let child_rel = join_rel(rel, &name);
            match self.stat_host(&entry.path(), &name) {
                Ok(info) => {
                    if name.to_lowercase().contains(needle) && !name.starts_with('.') {
                        if out.len() < 500 {
                            out.push((child_rel.clone(), info.clone()));
                        } else {
                            *truncated = true;
                        }
                    }
                    if info.kind == EntryKind::Dir {
                        self.search_into(
                            &entry.path(),
                            &child_rel,
                            needle,
                            out,
                            truncated,
                            depth.saturating_add(1),
                        );
                    }
                }
                Err(_) => continue, // a vanished or unreadable entry is not a search failure
            }
        }
    }

    /// The 100 most recently modified files under the root, excluding the
    /// trash subtree. Returns `(relative path, mtime, size)`.
    ///
    /// # Errors
    /// Only when the root itself is unreadable; subtrees that fail are
    /// skipped.
    pub fn recent(&self) -> Result<Vec<(String, u64, u64)>, Error> {
        let mut all = Vec::new();
        let mut visited = 0usize;
        self.recent_into(&self.host_root.clone(), "", &mut all, &mut visited);
        all.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        Ok(all.into_iter().take(100).collect())
    }

    fn recent_into(
        &self,
        host: &Path,
        rel: &str,
        out: &mut Vec<(String, u64, u64)>,
        visited: &mut usize,
    ) {
        if *visited >= WALK_CAP || out.len() >= 5_000 {
            return;
        }
        let Ok(read) = fs::read_dir(host) else {
            return;
        };
        for entry in read.filter_map(|e| e.ok()) {
            *visited = visited.saturating_add(1);
            if *visited >= WALK_CAP {
                return;
            }
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let child_rel = join_rel(rel, &name);
            // The trash never appears in RECENT: deleted files are not files.
            if rel.is_empty() && name == crate::trash::TRASH_DIR_NAME {
                continue;
            }
            if let Ok(info) = self.stat_host(&entry.path(), &name) {
                if info.kind == EntryKind::Dir {
                    self.recent_into(&entry.path(), &child_rel, out, visited);
                } else if info.kind == EntryKind::File {
                    out.push((child_rel, info.mtime, info.size));
                }
            }
        }
    }
}

struct Measure {
    bytes: u64,
    files: u64,
    dirs: u64,
    visited: usize,
    partial: bool,
}

/// Why [`Root::pick_name`] will act on an occupied name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameAction {
    /// The name was free; use it.
    Plain,
    /// Policy says skip.
    Skip,
    /// Policy says replace.
    Replace,
}

fn is_exdev(e: &io::Error) -> bool {
    // 18 is EXDEV on Linux: the rename crossed a filesystem boundary.
    e.raw_os_error() == Some(18)
}

/// `name.ext` becomes `name - Copy.ext` — the ADR-0019 name for a copy
/// made into the item's own directory.
#[must_use]
pub fn copy_of_name(base: &str) -> String {
    match base.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem} - Copy.{ext}"),
        _ => format!("{base} - Copy"),
    }
}

/// `name.ext` becomes `name.2.ext`, `name.3.ext`, ... until a free name is
/// found. Returns `None` past 10_000, which callers report as `exists`.
fn numbered_name(base: &str, occupied: &dyn Fn(&str) -> bool) -> Option<String> {
    let (stem, ext) = match base.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s, Some(e)),
        _ => (base, None),
    };
    for n in 2..10_000u32 {
        let candidate = match ext {
            Some(e) => format!("{stem}.{n}.{e}"),
            None => format!("{stem}.{n}"),
        };
        if !occupied(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Relative path join used by the search and recent walks. The root itself
/// is the empty string; every child is `parent/name`.
fn join_rel(parent: &str, name: &str) -> String {
    // A walk that starts at the root passes "/" as its parent, and joining it
    // naively produced "//Documents/a.txt" — a path that is accepted everywhere
    // and matches nothing that was printed from a LIST.
    if parent.is_empty() || parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}

fn hidden_placeholder(name: &str) -> StatInfo {
    StatInfo {
        name: name.to_owned(),
        kind: EntryKind::Other,
        size: 0,
        mtime: 0,
        mode: 0,
        hidden: name.starts_with('.'),
        target: None,
    }
}

/// Unix seconds of a metadata timestamp, saturating at the epoch for
/// pre-1970 stamps (which do not occur on files Litric manages).
#[must_use]
pub fn unix_time_of(meta: &fs::Metadata) -> u64 {
    match meta.modified() {
        Ok(t) => t
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        Err(_) => {
            let _ = SystemTime::now(); // no clock; zero is the honest answer
            0
        }
    }
}

#[cfg(test)]
mod tests {
    // Fixtures are small fixed trees built under the process's temporary
    // directory, never the daemon root.
    #![allow(clippy::indexing_slicing)]

    use super::*;
    use crate::Policy;
    use litric_locale::TimeZone;
    use std::fs;
    use std::os::unix::net::UnixListener;

    fn temp_root(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-files-ops-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("Documents")).expect("mkdir");
        dir
    }

    fn open_root(tag: &str) -> (PathBuf, Root) {
        let dir = temp_root(tag);
        let root = Root::open(&dir, TimeZone::utc("test")).expect("root opens");
        (dir, root)
    }

    fn logical(text: &str) -> LogicalPath {
        LogicalPath::parse(text).expect("valid logical path")
    }

    /// Any staging name left behind in `dir` after an operation finished.
    fn staging_leftovers(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.starts_with(".litric-staged-"))
            .collect()
    }

    /// The regression `Policy::Replace` promises against: a replacement that
    /// cannot complete must leave the item it was replacing untouched.
    #[test]
    fn replace_keeps_the_destination_when_the_copy_fails() {
        let (dir, root) = open_root("replace-fails");
        fs::write(dir.join("Documents/keep.txt"), b"old").expect("destination");
        fs::create_dir(dir.join("Documents/src")).expect("source dir");
        // The incoming item is a *tree* named `keep.txt`, because only a tree
        // makes `copy_into` walk into something it cannot copy. Writing a
        // plain file here would have made the whole test pass for the wrong
        // reason: a single regular file always copies.
        fs::create_dir(dir.join("Documents/src/keep.txt")).expect("source tree");
        // A unix socket is neither a file, a directory nor a link, so
        // `copy_into` refuses the tree with `unsupported-archive`. It also
        // needs no unsafe code or libc to create, unlike a FIFO.
        let _socket =
            UnixListener::bind(dir.join("Documents/src/keep.txt/pipe.sock")).expect("socket");
        let dest_dir = root.resolve(&logical("/Documents")).expect("dest dir");
        let err = root
            .copy_item(
                &logical("/Documents/src/keep.txt"),
                &dest_dir,
                Policy::Replace,
                false,
            )
            .expect_err("copying a socket must fail");
        assert_eq!(err.kind(), ErrorKind::UnsupportedArchive);
        assert_eq!(
            fs::read_to_string(dir.join("Documents/keep.txt")).expect("read back"),
            "old",
            "the item being replaced must survive a failed replacement"
        );
        assert!(
            staging_leftovers(&dir.join("Documents")).is_empty(),
            "a failed replacement must not leave a staging entry behind"
        );
    }

    /// The same path, succeeding: the new item is in place and the old one
    /// is gone, with nothing staged left over.
    #[test]
    fn replace_installs_the_new_item_and_removes_the_old() {
        let (dir, root) = open_root("replace-ok");
        fs::write(dir.join("Documents/report.txt"), b"old copy").expect("destination");
        fs::create_dir(dir.join("src")).expect("source dir");
        fs::write(dir.join("src/report.txt"), b"new copy").expect("source");
        let dest_dir = root.resolve(&logical("/Documents")).expect("dest dir");
        let made = root
            .copy_item(
                &logical("/src/report.txt"),
                &dest_dir,
                Policy::Replace,
                false,
            )
            .expect("replace");
        assert_eq!(made, dir.join("Documents/report.txt"));
        assert_eq!(fs::read_to_string(&made).expect("read back"), "new copy");
        assert!(staging_leftovers(&dir.join("Documents")).is_empty());
    }

    /// A directory cannot be replaced by one rename, so the old tree is
    /// moved aside and only dropped once the new one is in place. Before
    /// the fix this path deleted the occupant first, so a failure anywhere
    /// in the swap lost it for nothing.
    #[test]
    fn move_onto_an_occupied_directory_swaps_instead_of_deleting_first() {
        let (dir, root) = open_root("move-onto-dir");
        fs::create_dir_all(dir.join("Documents/backup/tree")).expect("occupied tree");
        fs::write(dir.join("Documents/backup/tree/old.txt"), b"old").expect("old file");
        fs::create_dir_all(dir.join("Documents/tree")).expect("source tree");
        fs::write(dir.join("Documents/tree/new.txt"), b"new").expect("new file");
        let dest_dir = root
            .resolve(&logical("/Documents/backup"))
            .expect("dest dir");
        root.move_item(
            &logical("/Documents/tree"),
            &dest_dir,
            Policy::Replace,
            false,
        )
        .expect("move");
        assert!(
            !dir.join("Documents/tree").exists(),
            "the source moved, so it is gone from its old place"
        );
        assert_eq!(
            fs::read_to_string(dir.join("Documents/backup/tree/new.txt")).expect("read back"),
            "new"
        );
        assert!(
            !dir.join("Documents/backup/tree/old.txt").exists(),
            "the replaced tree is gone once the new one is in place"
        );
        assert!(staging_leftovers(&dir.join("Documents/backup")).is_empty());
    }

    /// ADR-0019 §3 says every intermediate component must be a real
    /// directory. Before, that check was a no-op and the walk failed later
    /// with `not-found`, which named the wrong problem.
    #[test]
    fn resolve_reports_a_file_standing_in_the_middle() {
        let (dir, root) = open_root("resolve-middle");
        fs::write(dir.join("Documents/note.txt"), b"x").expect("file");
        let err = root
            .resolve(&logical("/Documents/note.txt/inner"))
            .expect_err("a file is not a directory");
        assert_eq!(err.kind(), ErrorKind::NotADirectory);
        // The final component is still allowed to be a file: STAT needs it.
        assert!(root.resolve(&logical("/Documents/note.txt")).is_ok());
    }
}
