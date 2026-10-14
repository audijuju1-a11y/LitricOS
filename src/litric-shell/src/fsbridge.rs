//! One bridge between the shell and the filesystem.
//!
//! The shell must work in three places: on a booted LitricOS machine, where
//! `litric_files::Root` resolves logical paths without ever following a
//! symlink; on any Unix host, where the same confinement applies; and on the
//! Windows development machine, where `cargo build` runs in CI and where the
//! Unix-only crate is simply absent.
//!
//! Before this module existed, `views.rs` reached straight into
//! `litric_files::ops` and the workspace stopped compiling on Windows — five
//! errors on the main branch. That was the concrete failure this module
//! exists to prevent: the shell's *behaviour* is tested on every host, so
//! every filesystem call must compile on every host.
//!
//! The Unix side keeps the ADR-0019 guarantees: logical paths only, no
//! symlink following, name validation on creation. The non-Unix side fails
//! every operation with [`FsError::Unsupported`] and hides nothing: the Files
//! window shows the refusal, it does not show an empty folder that looks like
//! a bug.

use crate::apps::FileKind;

/// What one directory entry looks like after the bridge has read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The final path segment.
    pub name: String,
    /// Directory, file, link or other.
    pub kind: FileKind,
    /// Size in bytes.
    pub size: u64,
    /// Modification time, Unix seconds.
    pub mtime: u64,
}

/// Why a filesystem operation failed, in the user's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsError {
    /// The platform has no filesystem bridge (non-Unix hosts).
    Unsupported,
    /// Nothing exists at that path.
    NotFound,
    /// Something is already there.
    Exists,
    /// The OS refused.
    Denied,
    /// The name breaks the naming rules.
    BadName,
    /// The path is not a directory where one was needed.
    NotADirectory,
    /// Anything else.
    Io(String),
}

impl core::fmt::Display for FsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("the file service is not available on this platform"),
            Self::NotFound => f.write_str("no such item"),
            Self::Exists => f.write_str("an item with that name already exists"),
            Self::Denied => f.write_str("permission denied"),
            Self::BadName => f.write_str("that name is not allowed"),
            Self::NotADirectory => f.write_str("that item is not a folder"),
            Self::Io(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for FsError {}

/// The name an operation refused. The caller's dialog says which field it
/// came from; here every bad name is the same refusal, so a hostile name can
/// never branch its way past validation by being differently invalid.
fn reject(name: &str) -> Result<(), FsError> {
    let _ = name;
    Err(FsError::BadName)
}

/// Whether `name` is a legal final segment.
///
/// The rules match `litric_files::path::validate_segment`: non-empty, no
/// path separators, no `.` or `..`, no control characters, under 255 bytes.
/// Enforced here so a host-side test can never write outside the folder it
/// was given, whatever called into the bridge.
#[must_use]
pub fn valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 || name.starts_with('.') {
        return false;
    }
    if name.contains('/') || name.contains('\\') || name == ".." {
        return false;
    }
    if name.contains("://") {
        return false;
    }
    !name.chars().any(char::is_control)
}

/// The logical root the Files application talks about, and the host directory
/// it maps to. On a booted LitricOS machine the host root is `/`.
#[derive(Debug, Clone)]
pub struct FsBridge {
    host_root: std::path::PathBuf,
    #[cfg(unix)]
    zone: litric_locale::TimeZone,
}

#[cfg(unix)]
fn map_err(op: &str, e: &std::io::Error) -> FsError {
    match e.kind() {
        std::io::ErrorKind::NotFound => FsError::NotFound,
        std::io::ErrorKind::AlreadyExists => FsError::Exists,
        std::io::ErrorKind::PermissionDenied => FsError::Denied,
        std::io::ErrorKind::InvalidInput => FsError::BadName,
        _ => FsError::Io(format!("{op}: {e}")),
    }
}

impl FsBridge {
    /// A bridge rooted at `host` (a real directory on disk).
    ///
    /// # Errors
    /// `Unsupported` on non-Unix hosts; `NotFound` / `NotADirectory` when the
    /// root is not a real directory there.
    pub fn open(host: &std::path::Path, zone: litric_locale::TimeZone) -> Result<Self, FsError> {
        #[cfg(not(unix))]
        {
            let _ = (host, zone);
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let meta = std::fs::symlink_metadata(host).map_err(|e| map_err("open", &e))?;
            if !meta.is_dir() {
                return Err(FsError::NotADirectory);
            }
            Ok(Self {
                host_root: host.to_path_buf(),
                zone,
            })
        }
    }

    /// The host path of a logical path such as `/Documents`, refusing `..`
    /// and separators that would escape the root. On non-Unix this always
    /// fails with [`FsError::Unsupported`]; the caller reports it.
    ///
    /// # Errors
    /// [`FsError::Unsupported`] on non-Unix; [`FsError::BadName`] for a path
    /// that would escape the root.
    pub fn host_path(&self, logical: &str) -> Result<std::path::PathBuf, FsError> {
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let mut out = self.host_root.clone();
            for seg in logical.split('/') {
                if seg.is_empty() {
                    continue;
                }
                if seg == ".." {
                    return Err(FsError::BadName);
                }
                out.push(seg);
            }
            Ok(out)
        }
    }

    /// Read one directory, hidden files excluded, sorted name-first.
    ///
    /// # Errors
    /// As [`Self::host_path`], plus I/O.
    pub fn list_dir(&self, logical: &str) -> Result<Vec<Entry>, FsError> {
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.host_path(logical)?;
            let meta = std::fs::symlink_metadata(&dir).map_err(|e| map_err("list", &e))?;
            if !meta.is_dir() {
                return Err(FsError::NotADirectory);
            }
            let read = std::fs::read_dir(&dir).map_err(|e| map_err("list", &e))?;
            let mut out = Vec::new();
            for entry in read.filter_map(|e| e.ok()) {
                let Ok(name) = entry.file_name().into_string() else {
                    continue;
                };
                if name.starts_with('.') {
                    continue;
                }
                let Ok(info) = entry.metadata() else {
                    continue;
                };
                let kind = if info.is_dir() {
                    FileKind::Dir
                } else if info.file_type().is_symlink() {
                    FileKind::Link
                } else if info.is_file() {
                    FileKind::File
                } else {
                    FileKind::Other
                };
                out.push(Entry {
                    name,
                    kind,
                    size: info.len(),
                    mtime: info
                        .modified()
                        .ok()
                        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                });
            }
            out.sort_by_key(|e| e.name.to_lowercase());
            Ok(out)
        }
    }

    /// Whether an item exists.
    ///
    /// # Errors
    /// [`FsError::Unsupported`] on non-Unix hosts.
    pub fn exists(&self, logical: &str) -> Result<bool, FsError> {
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let p = self.host_path(logical)?;
            Ok(std::fs::symlink_metadata(&p).is_ok())
        }
    }

    /// Create a directory inside `parent`.
    ///
    /// # Errors
    /// [`FsError::BadName`] for an illegal name; then as [`Self::host_path`].
    pub fn mkdir(&self, parent: &str, name: &str) -> Result<(), FsError> {
        if !valid_name(name) {
            return reject(name);
        }
        #[cfg(not(unix))]
        {
            let _ = parent;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.host_path(parent)?;
            std::fs::create_dir(dir.join(name)).map_err(|e| map_err("mkdir", &e))
        }
    }

    /// Create an empty file inside `parent`. Fails when the name is taken.
    ///
    /// # Errors
    /// As [`Self::mkdir`].
    pub fn new_file(&self, parent: &str, name: &str) -> Result<(), FsError> {
        if !valid_name(name) {
            return reject(name);
        }
        #[cfg(not(unix))]
        {
            let _ = parent;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.host_path(parent)?;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join(name))
                .map(|_| ())
                .map_err(|e| map_err("new file", &e))
        }
    }

    /// Rename within `parent`. Never overwrites.
    ///
    /// # Errors
    /// [`FsError::Exists`] when the destination name is taken.
    pub fn rename(&self, parent: &str, old: &str, new: &str) -> Result<(), FsError> {
        if !valid_name(new) {
            return reject(new);
        }
        #[cfg(not(unix))]
        {
            let _ = (parent, old);
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.host_path(parent)?;
            let dest = dir.join(new);
            if std::fs::symlink_metadata(&dest).is_ok() {
                return Err(FsError::Exists);
            }
            std::fs::rename(dir.join(old), &dest).map_err(|e| map_err("rename", &e))
        }
    }

    /// Copy one item into `dest_dir`, keeping both on a name clash as
    /// `name - Copy.ext` (the Explorer rule) and then numbering.
    ///
    /// # Errors
    /// As [`Self::host_path`] plus I/O.
    pub fn copy(&self, from: &str, dest_dir: &str) -> Result<String, FsError> {
        #[cfg(not(unix))]
        {
            let _ = (from, dest_dir);
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let src = self.host_path(from)?;
            let _ = std::fs::symlink_metadata(&src).map_err(|e| map_err("copy", &e))?;
            let name = from.rsplit('/').next().unwrap_or_default().to_owned();
            let dest = self.host_path(dest_dir)?;
            let final_name = if dest.join(&name).exists() {
                crate::fsbridge::copy_of_name(&name, &|n| dest.join(n).exists())
            } else {
                name
            };
            copy_tree(&src, &dest.join(&final_name), 0)?;
            Ok(final_name)
        }
    }

    /// Move one item into `dest_dir` (the paste half of Cut).
    ///
    /// # Errors
    /// As [`Self::copy`].
    pub fn move_item(&self, from: &str, dest_dir: &str) -> Result<String, FsError> {
        #[cfg(not(unix))]
        {
            let _ = (from, dest_dir);
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let src = self.host_path(from)?;
            let _meta = std::fs::symlink_metadata(&src).map_err(|e| map_err("move", &e))?;
            let name = from.rsplit('/').next().unwrap_or_default().to_owned();
            let dest = self.host_path(dest_dir)?;
            let final_name = if dest.join(&name).exists() {
                crate::fsbridge::copy_of_name(&name, &|n| dest.join(n).exists())
            } else {
                name
            };
            let target = dest.join(&final_name);
            match std::fs::rename(&src, &target) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
                    copy_tree(&src, &target, 0)?;
                    let meta = std::fs::symlink_metadata(&src).map_err(|e| map_err("move", &e))?;
                    if meta.is_dir() && !meta.file_type().is_symlink() {
                        std::fs::remove_dir_all(&src).map_err(|e| map_err("move", &e))?;
                    } else {
                        std::fs::remove_file(&src).map_err(|e| map_err("move", &e))?;
                    }
                }
                Err(e) => return Err(map_err("move", &e)),
            }
            Ok(final_name)
        }
    }

    /// Delete permanently. The caller confirms first; this layer only deletes.
    ///
    /// # Errors
    /// As [`Self::host_path`].
    pub fn delete(&self, logical: &str) -> Result<(), FsError> {
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let p = self.host_path(logical)?;
            let meta = std::fs::symlink_metadata(&p).map_err(|e| map_err("delete", &e))?;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                std::fs::remove_dir_all(&p).map_err(|e| map_err("delete", &e))
            } else {
                std::fs::remove_file(&p).map_err(|e| map_err("delete", &e))
            }
        }
    }

    /// Move an item into the freedesktop.org trash under the root, writing
    /// its `.trashinfo` so Restore knows where it came from. The Unix trash
    /// layout lives at `<root>/.local/share/Trash`.
    ///
    /// # Errors
    /// As [`Self::host_path`] plus I/O.
    pub fn trash(&self, logical: &str, now_secs: u64) -> Result<(), FsError> {
        #[cfg(not(unix))]
        {
            let _ = (logical, now_secs);
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let src = self.host_path(logical)?;
            // Existence first: a missing source must fail here, before any
            // record exists, not after it has been reserved.
            let _ = std::fs::symlink_metadata(&src).map_err(|e| map_err("trash", &e))?;
            let base = self.host_root.join(".local/share/Trash");
            let files = base.join("files");
            let info = base.join("info");
            for d in [&files, &info] {
                if let Err(e) = std::fs::create_dir_all(d) {
                    return Err(map_err("trash dirs", &e));
                }
            }
            let name = logical.rsplit('/').next().unwrap_or_default().to_owned();
            let stamp = self.zone.local(i64::try_from(now_secs).unwrap_or(0));
            let original_uri = format!("file://{logical}");
            let record = format!(
                "[Trash Info]\nPath={}\nDeletionDate={:04}-{:02}-{:02}T{:02}:{:02}:{:02}\n",
                crate::fsbridge::pct_path(&original_uri),
                stamp.year,
                stamp.month,
                stamp.day,
                stamp.hour,
                stamp.minute,
                stamp.second,
            );
            // The record is created FIRST, exclusively, which is both what the
            // specification requires and how the name is reserved: two
            // trashings of one filename get two entries instead of one entry
            // overwritten twice, and a failure to record leaves the item where
            // it was. The daemon's writer was fixed for the same defect
            // (BUG-0011); this is the second implementation of one directory.
            let final_name = reserve_trash_name(&files, &info, &name, &record)?;
            if let Err(e) = std::fs::rename(&src, files.join(&final_name)) {
                // Nothing left its place, so the reservation goes too: an info
                // file with no item behind it lists nowhere and restores
                // nothing.
                let _ = std::fs::remove_file(info.join(format!("{final_name}.trashinfo")));
                return Err(map_err("trash", &e));
            }
            Ok(())
        }
    }

    /// Every trash entry, oldest first.
    ///
    /// # Errors
    /// [`FsError::Unsupported`] on non-Unix; I/O when the trash cannot be read.
    pub fn trash_list(&self) -> Result<Vec<TrashEntry>, FsError> {
        #[cfg(not(unix))]
        {
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let base = self.host_root.join(".local/share/Trash");
            let info = base.join("info");
            let files = base.join("files");
            if !info.is_dir() {
                return Ok(Vec::new());
            }
            let read = std::fs::read_dir(&info).map_err(|e| map_err("trash list", &e))?;
            let mut names: Vec<String> = read
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .filter(|n| n.ends_with(".trashinfo"))
                .map(|n| n.trim_end_matches(".trashinfo").to_owned())
                .collect();
            names.sort();
            let mut out = Vec::new();
            for name in names {
                let Some(raw) = std::fs::read_to_string(info.join(format!("{name}.trashinfo")))
                    .ok()
                    .and_then(|t| {
                        let mut original = None;
                        let mut deleted = None;
                        for line in t.lines() {
                            if let Some(v) = line.strip_prefix("Path=") {
                                original = Some(crate::fsbridge::unpct_path(v));
                            } else if let Some(v) = line.strip_prefix("DeletionDate=") {
                                deleted = Some(v.to_owned());
                            }
                        }
                        Some((original?, deleted.unwrap_or_default()))
                    })
                else {
                    continue;
                };
                // A record with no item behind it is a half-written entry:
                // from an interrupted run, or from a build that moved the item
                // before writing the record. Shown as absent, exactly as the
                // daemon's reader treats it, so both implementations of one
                // directory agree about what is in it.
                if std::fs::symlink_metadata(files.join(&name)).is_err() {
                    continue;
                }
                let (original, deleted_iso) = raw;
                // The original URI is `file:///Documents/x`; the restore
                // destination is the directory part.
                let restore_dir = original
                    .strip_prefix("file://")
                    .and_then(|p| p.rsplit_once('/'))
                    // A file that lived in the root has an empty directory part,
                    // and `""` is not a path: the entry would be listed with
                    // nowhere to go back to, and `host_path("")` would resolve it
                    // to the root directory rather than to `/`. Found by
                    // `trash_write_list_restore_and_purge`, the one test that
                    // trashes a file from the top of the share.
                    .map(|(dir, _)| {
                        if dir.is_empty() {
                            "/".to_owned()
                        } else {
                            dir.to_owned()
                        }
                    })
                    .unwrap_or_else(|| "/".to_owned());
                let is_dir = files.join(&name).is_dir();
                out.push(TrashEntry {
                    name,
                    deleted_iso,
                    restore_dir,
                    is_dir,
                });
            }
            Ok(out)
        }
    }

    /// Put a trashed item back where it came from.
    ///
    /// # Errors
    /// [`FsError::NotFound`] when the entry is gone; [`FsError::Exists`] when
    /// something now stands where it would go.
    pub fn trash_restore(&self, name: &str) -> Result<String, FsError> {
        if name.starts_with('.') || name.contains('/') {
            return Err(FsError::BadName);
        }
        let entries = self.trash_list()?;
        let Some(entry) = entries.iter().find(|e| e.name == name) else {
            return Err(FsError::NotFound);
        };
        #[cfg(not(unix))]
        {
            let _ = entry;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let base = self.host_root.join(".local/share/Trash");
            let src = base.join("files").join(name);
            let dest_dir = self.host_path(&entry.restore_dir)?;
            let dest = dest_dir.join(name);
            if std::fs::symlink_metadata(&dest).is_ok() {
                return Err(FsError::Exists);
            }
            std::fs::rename(&src, &dest).map_err(|e| map_err("restore", &e))?;
            let _ = std::fs::remove_file(base.join("info").join(format!("{name}.trashinfo")));
            Ok(entry.restore_dir.clone())
        }
    }

    /// Delete one trash entry for ever.
    ///
    /// # Errors
    /// As [`Self::delete`] on the trash name.
    pub fn trash_purge(&self, name: &str) -> Result<(), FsError> {
        if name.starts_with('.') || name.contains('/') {
            return Err(FsError::BadName);
        }
        let base = format!("/.local/share/Trash/files/{name}");
        self.delete(&base)
    }

    /// Empty the trash. Returns how many entries went.
    ///
    /// # Errors
    /// Per-entry failures skip the entry; only a total failure surfaces.
    pub fn trash_empty(&self) -> Result<usize, FsError> {
        let mut n = 0usize;
        for e in self.trash_list()? {
            if self.trash_purge(&e.name).is_ok() {
                n = n.saturating_add(1);
            }
        }
        Ok(n)
    }

    /// Read a text file (the viewer). Refuses anything over 512 KB so a
    /// binary mislabeled as text cannot freeze the shell.
    ///
    /// # Errors
    /// [`FsError::NotFound`] / [`FsError::Denied`] / [`FsError::Io`].
    pub fn read_text(&self, logical: &str) -> Result<String, FsError> {
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            const MAX: u64 = 512 * 1024;
            let p = self.host_path(logical)?;
            let meta = std::fs::symlink_metadata(&p).map_err(|e| map_err("read", &e))?;
            if meta.len() > MAX {
                return Err(FsError::Io("file is too large to show".to_owned()));
            }
            let bytes = std::fs::read(&p).map_err(|e| map_err("read", &e))?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }

    /// Write text to a file inside `parent` (the New Text File flow), failing
    /// when the name is taken.
    ///
    /// # Errors
    /// As [`Self::new_file`].
    pub fn write_text(&self, parent: &str, name: &str, content: &str) -> Result<(), FsError> {
        if !valid_name(name) {
            return reject(name);
        }
        #[cfg(not(unix))]
        {
            let _ = (parent, content);
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.host_path(parent)?;
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join(name))
                .map_err(|e| map_err("write", &e))?;
            f.write_all(content.as_bytes())
                .map_err(|e| map_err("write", &e))
        }
    }

    /// Walk from `logical` collecting items whose name contains `query`
    /// (case-insensitive), at most 200 results.
    ///
    /// # Errors
    /// As [`Self::host_path`]; unreadable subtrees are skipped.
    pub fn search(&self, from: &str, query: &str) -> Result<Vec<String>, FsError> {
        // `host_path` fails with `Unsupported` on non-Unix, which also keeps
        // the recursive walk below a Unix-only method.
        self.host_path(from)?;
        let mut out = Vec::new();
        let needle = query.to_lowercase();
        if needle.is_empty() {
            return Ok(out);
        }
        self.search_into(from, &needle, &mut out, 0);
        Ok(out)
    }

    #[cfg(unix)]
    fn search_into(&self, dir: &str, needle: &str, out: &mut Vec<String>, depth: usize) {
        if out.len() >= 200 || depth > 16 {
            return;
        }
        let Ok(entries) = self.list_dir(dir) else {
            return;
        };
        for e in entries {
            if out.len() >= 200 {
                return;
            }
            let child = if dir == "/" {
                format!("/{}", e.name)
            } else {
                format!("{dir}/{}", e.name)
            };
            if e.name.to_lowercase().contains(needle) {
                out.push(child.clone());
            }
            if e.kind == FileKind::Dir {
                self.search_into(&child, needle, out, depth.saturating_add(1));
            }
        }
    }

    #[cfg(not(unix))]
    fn search_into(&self, _dir: &str, _needle: &str, _out: &mut Vec<String>, _depth: usize) {}

    /// Compress one or more items into a ZIP at `dest` (name must be free).
    /// Uses `litric-files`' archive writer, which enforces the ADR-0019
    /// limits and needs no third-party crate.
    ///
    /// # Errors
    /// [`FsError::Unsupported`] on non-Unix; then as the archive writer.
    pub fn compress(&self, dest: &str, sources: &[String]) -> Result<(), FsError> {
        let leaf = dest.rsplit('/').next().unwrap_or_default();
        if !valid_name(leaf) {
            return Err(FsError::BadName);
        }
        #[cfg(not(unix))]
        {
            let _ = sources;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            use std::io::Read as _;
            let dest_path = self.host_path(dest)?;
            if std::fs::symlink_metadata(&dest_path).is_ok() {
                return Err(FsError::Exists);
            }
            let mut specs: Vec<(litric_files::zip::ZipSpec, Vec<u8>)> = Vec::new();
            for src in sources {
                let p = self.host_path(src)?;
                let meta = std::fs::symlink_metadata(&p).map_err(|e| map_err("compress", &e))?;
                let base_name = src.rsplit('/').next().unwrap_or_default().to_owned();
                if meta.is_dir() && !meta.file_type().is_symlink() {
                    let mut stack = vec![(p.clone(), base_name.clone())];
                    while let Some((dir, rel)) = stack.pop() {
                        specs.push((
                            litric_files::zip::ZipSpec {
                                name: format!("{rel}/"),
                                store: true,
                            },
                            Vec::new(),
                        ));
                        let read = std::fs::read_dir(&dir).map_err(|e| map_err("compress", &e))?;
                        for entry in read.filter_map(|e| e.ok()) {
                            let Ok(name) = entry.file_name().into_string() else {
                                continue;
                            };
                            let child_rel = format!("{rel}/{name}");
                            let child = dir.join(&name);
                            let m = std::fs::symlink_metadata(&child)
                                .map_err(|e| map_err("compress", &e))?;
                            if m.is_dir() && !m.file_type().is_symlink() {
                                stack.push((child, child_rel));
                            } else if m.is_file() {
                                let bytes =
                                    std::fs::read(&child).map_err(|e| map_err("compress", &e))?;
                                specs.push((
                                    litric_files::zip::ZipSpec {
                                        name: child_rel,
                                        store: false,
                                    },
                                    bytes,
                                ));
                            }
                        }
                    }
                } else if meta.is_file() {
                    let mut f = std::fs::File::open(&p).map_err(|e| map_err("compress", &e))?;
                    let mut bytes = Vec::new();
                    f.read_to_end(&mut bytes)
                        .map_err(|e| map_err("compress", &e))?;
                    specs.push((
                        litric_files::zip::ZipSpec {
                            name: base_name,
                            store: false,
                        },
                        bytes,
                    ));
                } else {
                    return Err(FsError::Io("that item cannot be archived".to_owned()));
                }
            }
            let mut out: Vec<u8> = Vec::new();
            litric_files::zip::write_zip(&mut out, &specs)
                .map_err(|e| FsError::Io(format!("archive: {e}")))?;
            std::fs::write(&dest_path, out).map_err(|e| map_err("compress", &e))
        }
    }

    /// Extract a ZIP or tar archive into a new folder next to it named after
    /// the archive. Validation-first: a hostile archive is refused before any
    /// byte is written.
    ///
    /// # Errors
    /// As the archive reader (unsafe-entry, ratio, depth), plus I/O.
    pub fn extract(&self, archive: &str) -> Result<String, FsError> {
        #[cfg(not(unix))]
        {
            let _ = archive;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            use std::io::Read as _;
            let p = self.host_path(archive)?;
            let mut bytes = Vec::new();
            std::fs::File::open(&p)
                .and_then(|mut f| f.read_to_end(&mut bytes))
                .map_err(|e| map_err("extract", &e))?;
            let stem = archive
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .trim_end_matches(".zip")
                .trim_end_matches(".tar")
                .trim_end_matches(".gz")
                .trim_end_matches(".tgz")
                .to_owned();
            let parent = std::path::PathBuf::from(archive)
                .parent()
                .and_then(|d| d.to_str())
                .unwrap_or("/")
                .to_owned();
            let out_dir = if parent.ends_with('/') {
                format!("{parent}{stem}")
            } else {
                format!("{parent}/{stem}")
            };
            let out_host = self.host_path(&out_dir)?;
            if out_host.exists() {
                return Err(FsError::Exists);
            }
            std::fs::create_dir(&out_host).map_err(|e| map_err("extract", &e))?;
            let limits = litric_files::Limits::ADR_0019;
            let write_entry =
                |root: &Self, name: &str, content: &[u8], is_dir: bool| -> Result<(), FsError> {
                    if name.contains("..") {
                        return Err(FsError::Io("unsafe archive entry".to_owned()));
                    }
                    let logical = format!("{out_dir}/{name}");
                    if is_dir {
                        let host = root.host_path(&logical)?;
                        std::fs::create_dir_all(host).map_err(|e| map_err("extract", &e))
                    } else {
                        let host = root.host_path(&logical)?;
                        if let Some(parent) = host.parent() {
                            std::fs::create_dir_all(parent).map_err(|e| map_err("extract", &e))?;
                        }
                        std::fs::write(host, content).map_err(|e| map_err("extract", &e))
                    }
                };
            if litric_files::zip::read_zip_slice(&bytes, &limits).is_ok() {
                let entries = litric_files::zip::read_zip_slice(&bytes, &limits)
                    .map_err(|e| FsError::Io(format!("archive: {e}")))?;
                for e in &entries {
                    write_entry(self, &e.name, &e.content, e.is_dir)?;
                }
            } else if litric_files::tar::read_archive(&mut &bytes[..], &limits).is_ok() {
                let entries = litric_files::tar::read_archive(&mut &bytes[..], &limits)
                    .map_err(|e| FsError::Io(format!("archive: {e}")))?;
                for (name, is_dir, content) in &entries {
                    write_entry(self, name, content, *is_dir)?;
                }
            } else {
                let _ = std::fs::remove_dir(&out_host);
                return Err(FsError::Io("not an archive this system reads".to_owned()));
            }
            Ok(out_dir)
        }
    }

    /// The bridge's host root, for the Defender's scanner.
    #[must_use]
    pub fn host_root(&self) -> &std::path::Path {
        &self.host_root
    }
}

/// One entry of the trash, as the Trash window shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashEntry {
    /// The name inside `Trash/files/`.
    pub name: String,
    /// When it was deleted, as recorded.
    pub deleted_iso: String,
    /// The logical directory it came from.
    pub restore_dir: String,
    /// Whether it was a directory.
    pub is_dir: bool,
}

/// `name - Copy.ext`, then `name (2).ext`, ... — the first free name.
#[must_use]
pub fn copy_of_name(base: &str, occupied: &dyn Fn(&str) -> bool) -> String {
    // Split the extension off the *name*, not the stem: for `note.txt` the
    // copy is `note.txt - Copy.txt`, so the stem keeps the extension and the
    // split point is the whole name (matching Explorer's display name).
    let (stem, ext) = match base.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() && !base.starts_with('.') => (base, Some(e)),
        _ => (base, None),
    };
    let alt = match ext {
        Some(e) => format!("{stem} - Copy.{e}"),
        None => format!("{stem} - Copy"),
    };
    if !occupied(&alt) {
        return alt;
    }
    for n in 2..10_000u32 {
        let candidate = match ext {
            Some(e) => format!("{stem} ({n}).{e}"),
            None => format!("{stem} ({n})"),
        };
        if !occupied(&candidate) {
            return candidate;
        }
    }
    base.to_owned()
}

/// Copy a file or a directory tree, refusing to follow symlinks and refusing
/// to recurse deeper than 16 or past 10 000 entries.
#[cfg(unix)]
fn copy_tree(src: &std::path::Path, dest: &std::path::Path, depth: usize) -> Result<(), FsError> {
    const DEPTH_CAP: usize = 16;
    const ENTRIES_CAP: u64 = 10_000;
    struct Budget(u64);
    let mut budget = Budget(ENTRIES_CAP);
    fn go(
        src: &std::path::Path,
        dest: &std::path::Path,
        depth: usize,
        budget: &mut Budget,
    ) -> Result<(), FsError> {
        if depth > DEPTH_CAP {
            return Err(FsError::Io("tree is too deep to copy".to_owned()));
        }
        let meta = std::fs::symlink_metadata(src).map_err(|e| map_err("copy", &e))?;
        if meta.file_type().is_symlink() {
            // A link is copied as a link is displayed: not at all. Refuse it
            // rather than silently copying its target (no-follow guarantee).
            return Err(FsError::Denied);
        }
        if !meta.is_dir() {
            budget.0 = budget.0.saturating_sub(1);
            if budget.0 == 0 {
                return Err(FsError::Io("too much to copy".to_owned()));
            }
            return std::fs::copy(src, dest)
                .map(|_| ())
                .map_err(|e| map_err("copy", &e));
        }
        std::fs::create_dir(dest).map_err(|e| map_err("copy", &e))?;
        let read = std::fs::read_dir(src).map_err(|e| map_err("copy", &e))?;
        for entry in read.filter_map(|e| e.ok()) {
            let child = entry.path();
            let name = entry.file_name();
            go(&child, &dest.join(name), depth.saturating_add(1), budget)?;
        }
        Ok(())
    }
    go(src, dest, depth, &mut budget)
}

/// Take a trash name by creating its `.trashinfo` with `O_EXCL`.
///
/// `create_new` is `O_EXCL`, so a name another process (or an earlier item)
/// already holds fails the create and the next candidate is tried instead of
/// that entry being overwritten. A name counts as taken when **either** half
/// of the entry is present — the record in `info/` or the item in `files/` —
/// which is what stops a half-written entry from being overwritten by the
/// next trashing of the same name.
#[cfg(unix)]
fn reserve_trash_name(
    files: &std::path::Path,
    info: &std::path::Path,
    base: &str,
    record: &str,
) -> Result<String, FsError> {
    let taken = |n: &str| info.join(format!("{n}.trashinfo")).exists() || files.join(n).exists();
    let mut tried: Vec<String> = Vec::new();
    let mut candidate = base.to_owned();
    for _ in 0..10_000u32 {
        if !taken(&candidate) {
            let path = info.join(format!("{candidate}.trashinfo"));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    // `write_all` is where a full disk or a permission error
                    // surfaces; a `File` drop cannot report a close error.
                    if let Err(e) = std::io::Write::write_all(&mut file, record.as_bytes()) {
                        let _ = std::fs::remove_file(&path);
                        return Err(map_err("trashinfo", &e));
                    }
                    return Ok(candidate);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(map_err("trashinfo", &e)),
            }
        }
        tried.push(candidate);
        candidate =
            crate::fsbridge::copy_of_name(base, &|n| taken(n) || tried.iter().any(|t| t == n));
    }
    Err(FsError::Io("no free trash name for that item".to_owned()))
}

/// `file:///Documents/a%20b.txt` — percent-encode for a `.trashinfo` Path=.
#[cfg(unix)]
#[must_use]
fn pct_path(uri: &str) -> String {
    let mut out = String::with_capacity(uri.len());
    for b in uri.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The inverse of [`pct_path`].
#[cfg(unix)]
#[must_use]
fn unpct_path(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes.get(i) == Some(&b'%') {
            if let Some(hex) = s.get(i.saturating_add(1)..i.saturating_add(3)) {
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    out.push(b);
                    i = i.saturating_add(3);
                    continue;
                }
            }
        }
        if let Some(b) = bytes.get(i) {
            out.push(*b);
        }
        i = i.saturating_add(1);
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[cfg_attr(not(unix), allow(dead_code))]
    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("litric-shell-fsbridge-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("fixture root");
        dir
    }

    #[test]
    fn names_are_validated_before_any_touch_of_the_disk() {
        assert!(valid_name("Work"));
        assert!(valid_name("2026 Q3 notes.txt"));
        assert!(!valid_name(""));
        assert!(!valid_name(".hidden"));
        assert!(!valid_name(".."));
        assert!(!valid_name("a/b"));
        assert!(!valid_name("a\\b"));
        assert!(!valid_name("a\u{7}b"));
    }

    #[cfg(unix)]
    #[test]
    fn list_read_create_rename_and_delete_round_trip() {
        let root = temp_root("rt");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).expect("bridge");
        assert_eq!(b.list_dir("/").expect("empty"), Vec::new());
        b.mkdir("/", "Work").expect("mkdir");
        assert_eq!(
            b.list_dir("/").unwrap().first().map(|e| e.kind),
            Some(FileKind::Dir)
        );
        b.new_file("/Work", "a.txt").expect("new file");
        b.write_text("/Work", "b.txt", "hello สวัสดี").expect("write");
        // `read_text` answers a `Result`, so `as_deref` yields `Result<&str, &FsError>`
        // and the expectation is `Ok`, not `Some`.
        assert_eq!(b.read_text("/Work/b.txt").as_deref(), Ok("hello สวัสดี"));
        assert!(matches!(b.new_file("/Work", "a.txt"), Err(FsError::Exists)));
        b.rename("/Work", "a.txt", "c.txt").expect("rename");
        assert!(!b.exists("/Work/a.txt").unwrap());
        assert!(b.exists("/Work/c.txt").unwrap());
        assert!(matches!(
            b.rename("/Work", "c.txt", "d/e"),
            Err(FsError::BadName)
        ));
        b.delete("/Work").expect("delete tree");
        assert!(!b.exists("/Work").unwrap());
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn copy_keeps_both_and_numbers_the_next_clash() {
        let root = temp_root("cp");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.mkdir("/", "D").unwrap();
        b.write_text("/", "note.txt", "x").unwrap();
        let n1 = b.copy("/note.txt", "/D").expect("first copy");
        let n2 = b.copy("/note.txt", "/D").expect("second copy");
        assert_eq!(n1, "note.txt");
        assert_eq!(n2, "note.txt - Copy.txt");
        let n3 = b.copy("/note.txt", "/D").expect("third copy");
        assert_eq!(n3, "note.txt (2).txt");
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn trash_write_list_restore_and_purge() {
        let root = temp_root("tr");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.write_text("/", "gone.txt", "bye").unwrap();
        b.trash("/gone.txt", 1_900_000_000).expect("trash");
        let list = b.trash_list().expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list.first().map(|i| i.name.as_str()), Some("gone.txt"));
        assert_eq!(list.first().map(|i| i.restore_dir.as_str()), Some("/"));
        assert!(!b.exists("/gone.txt").unwrap());
        b.trash_restore("gone.txt").expect("restore");
        assert!(b.exists("/gone.txt").unwrap());
        assert_eq!(b.trash_list().unwrap().len(), 0);
        b.trash("/gone.txt", 1_900_000_100).unwrap();
        b.trash_purge("gone.txt").expect("purge");
        assert_eq!(b.trash_list().unwrap().len(), 0);
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn search_finds_nested_names_case_insensitively() {
        let root = temp_root("se");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.mkdir("/", "Projects").unwrap();
        b.mkdir("/Projects", "Alpha").unwrap();
        b.write_text("/Projects/Alpha", "README.md", "# hi")
            .unwrap();
        let hits = b.search("/", "readme").expect("search");
        assert_eq!(hits, vec!["/Projects/Alpha/README.md".to_owned()]);
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn compress_then_extract_round_trips_a_tree() {
        let root = temp_root("ar");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.mkdir("/", "Site").unwrap();
        b.mkdir("/Site", "img").unwrap();
        b.write_text("/Site", "index.html", "<h1>litric</h1>")
            .unwrap();
        b.write_text("/Site/img", "logo.txt", "O").unwrap();
        b.compress("/site.zip", &["/Site".to_owned()]).expect("zip");
        b.extract("/site.zip").expect("unzip");
        assert_eq!(
            b.read_text("/site/Site/index.html").as_deref(),
            Ok("<h1>litric</h1>")
        );
        assert_eq!(b.read_text("/site/Site/img/logo.txt").as_deref(), Ok("O"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn copy_of_name_follows_the_explorer_rule() {
        // The closure answers "is this name already taken?". The first call
        // finds every plain "- Copy" name free; the second finds the first
        // two candidates taken.
        let free = |n: &str| n != "note.txt - Copy.txt";
        assert_eq!(copy_of_name("note.txt", &free), "note.txt - Copy.txt");
        let taken =
            |n: &str| n == "note.txt" || n == "note.txt - Copy.txt" || n == "note.txt (2).txt";
        assert_eq!(copy_of_name("note.txt", &taken), "note.txt (3).txt");
        assert_eq!(
            copy_of_name("Makefile", &|n| n.ends_with("Copy")),
            "Makefile (2)"
        );
        assert_eq!(copy_of_name(".hidden", &|_| false), ".hidden - Copy");
    }

    #[cfg(not(unix))]
    #[test]
    fn a_non_unix_bridge_refuses_everything_by_name() {
        let dir = std::env::temp_dir();
        let b = FsBridge::open(&dir, litric_locale::TimeZone::utc("test"))
            .expect_err("non-unix must refuse to open");
        assert!(matches!(b, FsError::Unsupported));
    }
}
