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
    /// The filesystem does not allow writes at all.
    ReadOnly,
    /// The filesystem has no room left.
    Full,
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
            Self::ReadOnly => {
                f.write_str("the filesystem is read-only, so nothing can be written there")
            }
            Self::Full => f.write_str("the storage is full, so nothing more can be written"),
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
    /// The Files daemon, when the machine has one. `None` means this bridge
    /// performs the I/O itself, which is what a fixture host and a unit test
    /// need and what a booted image must not: there, the daemon owns the
    /// directory and this process does not open it for writing at all.
    #[cfg(unix)]
    daemon: Option<std::sync::Arc<crate::fsclient::Daemon>>,
}

#[cfg(unix)]
fn map_err(op: &str, e: &std::io::Error) -> FsError {
    match e.kind() {
        std::io::ErrorKind::NotFound => FsError::NotFound,
        std::io::ErrorKind::AlreadyExists => FsError::Exists,
        std::io::ErrorKind::PermissionDenied => FsError::Denied,
        // A component of the path is a file. The raw sentence for this is "Not
        // a directory (os error 20)", which says nothing a person can act on;
        // the storage path is included by the caller.
        std::io::ErrorKind::NotADirectory => FsError::NotADirectory,
        std::io::ErrorKind::ReadOnlyFilesystem => FsError::ReadOnly,
        std::io::ErrorKind::StorageFull => FsError::Full,
        std::io::ErrorKind::InvalidInput => FsError::BadName,
        // The raw operating-system sentence is the only diagnosis left (torn
        // filesystem, I/O error, quota, an unreadable parent), and dropping it
        // would leave "could not create the folder" with no cause at all.
        _ => FsError::Io(format!("{op}: {e}")),
    }
}

/// The directories every user storage starts with.
///
/// Re-exported from `litric-files`, which is where the daemon gets the same
/// list. Two copies of "which folders does a new account start with" is how a
/// user ends up with a `Documents` on one path and not on the other.
pub const DEFAULT_STORAGE_DIRS: [&str; 7] = litric_files::STANDARD_DIRS;

/// Create `host` and the standard folders inside it, without disturbing what is
/// already there.
///
/// This is the direct path: it exists for a fixture host and for the test
/// suite. On a booted machine `litric-filesd` performs it, because the daemon
/// owns the directory (ADR-0019).
///
/// # Errors
/// `NotADirectory` when the root or a standard name exists as something else;
/// otherwise the OS error, mapped to a sentence a person can act on.
pub fn ensure_storage(host: &std::path::Path) -> Result<(), FsError> {
    litric_files::ensure_storage(host).map_err(|e| FsError::from_files(&e))
}

impl FsError {
    /// The sentence for an error the `litric-files` library raised.
    fn from_files(e: &litric_files::Error) -> Self {
        use litric_files::ErrorKind;
        match e.kind() {
            ErrorKind::NotFound => Self::NotFound,
            ErrorKind::Exists => Self::Exists,
            ErrorKind::PermissionDenied | ErrorKind::SymlinkRefused => Self::Denied,
            ErrorKind::InvalidName | ErrorKind::BadRequest => Self::BadName,
            ErrorKind::NotADirectory | ErrorKind::IsADirectory => Self::NotADirectory,
            ErrorKind::TooLarge | ErrorKind::Limit => {
                Self::Io(format!("that is too large: {}", e.message()))
            }
            // Every remaining kind is a real refusal with a real reason, and
            // the reason is the message the library wrote.
            _ => Self::Io(e.message().to_owned()),
        }
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

            // Direct mode: this process opens the documents itself. Only a
            // fixture host reaches this; the image attaches the daemon with
            // `open_via_daemon` instead.
            Ok(Self {
                host_root: host.to_path_buf(),
                zone,
                daemon: None,
            })
        }
    }

    /// A bridge that sends every operation to `daemon` instead of performing it
    /// here (ADR-0019).
    ///
    /// The root is still checked as a directory on this host: the shell shows
    /// paths, and a bridge whose root does not exist would report "not found"
    /// for a storage the daemon is serving. The daemon creates the root, so on
    /// a booted machine this check passes because the daemon already ran.
    ///
    /// # Errors
    /// `Unsupported` on non-Unix hosts; `NotFound` / `NotADirectory` when the
    /// root is not a real directory there.
    #[cfg(unix)]
    pub fn open_via_daemon(
        host: &std::path::Path,
        zone: litric_locale::TimeZone,
        daemon: std::sync::Arc<crate::fsclient::Daemon>,
    ) -> Result<Self, FsError> {
        let mut bridge = Self::open(host, zone)?;
        bridge.daemon = Some(daemon);
        Ok(bridge)
    }

    /// The daemon this bridge forwards to, if any.
    #[cfg(unix)]
    #[must_use]
    pub fn daemon(&self) -> Option<&std::sync::Arc<crate::fsclient::Daemon>> {
        self.daemon.as_ref()
    }

    /// The host path of a logical path such as `/Documents`, refusing `..`,
    /// `.` and any path whose **parent** is a symbolic link. On non-Unix this
    /// always fails with [`FsError::Unsupported`]; the caller reports it.
    ///
    /// Rejecting `..` alone was not enough: `/Documents/link/file` is lexically
    /// inside the storage root while `link` may point at `/etc`, and the old
    /// implementation happily returned that path. Every existing component is
    /// checked with `symlink_metadata`, so a link in the middle of a path is a
    /// refusal rather than a route out of the share (ADR-0019).
    ///
    /// # Errors
    /// [`FsError::Unsupported`] on non-Unix; [`FsError::BadName`] for a path
    /// that would escape the root; [`FsError::Denied`] when a parent is a link.
    pub fn host_path(&self, logical: &str) -> Result<std::path::PathBuf, FsError> {
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            self.checked_path(logical, false)
        }
    }

    /// The host path with the final component checked too: a path that names a
    /// symbolic link is refused. Used by the operations that *read through* a
    /// path, where the kernel would follow it (`read_dir`, `read`).
    #[cfg(unix)]
    fn checked_path(
        &self,
        logical: &str,
        include_last: bool,
    ) -> Result<std::path::PathBuf, FsError> {
        let mut out = self.host_root.clone();
        let segments: Vec<&str> = logical.split('/').filter(|s| !s.is_empty()).collect();
        let mut missing = false;
        for (index, seg) in segments.iter().enumerate() {
            // `.` addresses the same directory and `..` names a parent. Neither
            // is a name this bridge accepts, even when the result would still
            // be inside the root: a logical path is a sequence of names.
            if *seg == ".." || *seg == "." || seg.contains('\0') {
                return Err(FsError::BadName);
            }
            out.push(seg);
            if missing {
                continue;
            }
            let last = index.saturating_add(1) == segments.len();
            if last && !include_last {
                continue;
            }
            match std::fs::symlink_metadata(&out) {
                Ok(meta) => {
                    if meta.file_type().is_symlink() {
                        return Err(FsError::Denied);
                    }
                }
                // A component that does not exist cannot be a link, and the
                // operation itself will report `NotFound` where that matters.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => missing = true,
                Err(e) => return Err(map_err("resolve", &e)),
            }
        }
        Ok(out)
    }

    /// A path whose final component must not be a symbolic link either.
    #[cfg(unix)]
    fn read_path(&self, logical: &str) -> Result<std::path::PathBuf, FsError> {
        self.checked_path(logical, true)
    }

    /// An internal storage directory (the trash), created if absent and refused
    /// if a link stands where one of its components should be.
    #[cfg(unix)]
    fn storage_dir(&self, logical: &str) -> Result<std::path::PathBuf, FsError> {
        let path = self.checked_path(logical, true)?;
        std::fs::create_dir_all(&path).map_err(|e| map_err("create storage folder", &e))?;
        Ok(path)
    }

    /// Read one directory, hidden files excluded, sorted name-first.
    ///
    /// # Errors
    /// As [`Self::host_path`], plus I/O.
    pub fn list_dir(&self, logical: &str) -> Result<Vec<Entry>, FsError> {
        self.list_dir_with_hidden(logical, false)
    }

    /// Read one directory, asking the service for dot-prefixed entries only
    /// when `show_hidden` is set. The filter is applied by the file service on
    /// the daemon path and by the local fallback here, so the Files choice
    /// changes one real directory read rather than hiding rows after the fact.
    ///
    /// # Errors
    /// As [`Self::host_path`], plus I/O.
    pub fn list_dir_with_hidden(
        &self,
        logical: &str,
        show_hidden: bool,
    ) -> Result<Vec<Entry>, FsError> {
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.list_dir(logical, show_hidden);
        }
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.read_path(logical)?;
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
                if !show_hidden && name.starts_with('.') {
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.exists(logical);
        }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.mkdir(parent, name);
        }
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
            // The parent is a *directory the operation acts through*, so its
            // own final component has to be a real directory too. Checking only
            // the components above it would let `/Documents/link` (a link to
            // anywhere) receive the new folder.
            let dir = self.checked_path(parent, true)?;
            let target = dir.join(name);
            std::fs::create_dir(&target).map_err(|e| map_err("mkdir", &e))
        }
    }

    /// Create an empty file inside `parent`. Fails when the name is taken.
    ///
    /// # Errors
    /// As [`Self::mkdir`].
    pub fn new_file(&self, parent: &str, name: &str) -> Result<(), FsError> {
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.new_file(parent, name);
        }
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
            let dir = self.checked_path(parent, true)?;
            let target = dir.join(name);
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .map(|_| ())
                .map_err(|e| map_err("new file", &e))
        }
    }

    /// Bring a staged browser download into `parent` under `name`.
    ///
    /// Only the daemon can do this: it owns the destination, and the staging
    /// directory is named on its command line. A bridge without a daemon says
    /// so instead of copying the file itself, because copying it here is the
    /// in-process write this arrangement exists to remove.
    ///
    /// # Errors
    /// [`FsError::Io`] when there is no daemon; otherwise as the daemon replies
    /// (`Exists` when the name is taken, `NotFound` when the staged file is
    /// gone, `BadName` for a name that is not one segment).
    pub fn import(&self, parent: &str, name: &str, staged: &str) -> Result<(), FsError> {
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.import(parent, name, staged);
        }
        let _ = (parent, name, staged);
        Err(FsError::Io(
            "a staged download can only be imported by the file service, and this \
             host has none"
                .to_owned(),
        ))
    }

    /// Rename within `parent`. Never overwrites.
    ///
    /// # Errors
    /// [`FsError::Exists`] when the destination name is taken.
    pub fn rename(&self, parent: &str, old: &str, new: &str) -> Result<(), FsError> {
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.rename(parent, old, new);
        }
        // Both names are final segments. `old` was only checked by the caller, and
        // a bridge method that trusts its caller with a path is a bridge whose
        // confinement is a promise: `rename(dir, "/etc/passwd", ...)` used to
        // join an absolute path over the directory and leave the storage root.
        if !valid_name(old) {
            return reject(old);
        }
        if !valid_name(new) {
            return reject(new);
        }
        #[cfg(not(unix))]
        {
            let _ = parent;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let dir = self.checked_path(parent, true)?;
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.copy(from, dest_dir);
        }
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
            if !valid_name(&name) {
                return Err(FsError::BadName);
            }
            let dest = self.checked_path(dest_dir, true)?;
            if !dest.is_dir() {
                return Err(FsError::NotADirectory);
            }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.move_item(from, dest_dir);
        }
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
            if !valid_name(&name) {
                return Err(FsError::BadName);
            }
            let dest = self.checked_path(dest_dir, true)?;
            if !dest.is_dir() {
                return Err(FsError::NotADirectory);
            }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.delete(logical);
        }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.trash(logical);
        }
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
            let files = self.storage_dir("/.local/share/Trash/files")?;
            let info = self.storage_dir("/.local/share/Trash/info")?;
            let name = logical.rsplit('/').next().unwrap_or_default().to_owned();
            let stamp = self.zone.local(i64::try_from(now_secs).unwrap_or(0));
            let deleted_iso = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                stamp.year, stamp.month, stamp.day, stamp.hour, stamp.minute, stamp.second,
            );
            // `Path=` is the original *location*: the absolute host pathname,
            // which is what the Freedesktop specification requires. It used to
            // be `file://<logical>`, which a conforming tool reads as a
            // *relative* pathname and restores to the wrong place (BUG-0009).
            //
            // The format is not spelled out here any more. The daemon writes the
            // same file, so it lives in one place —
            // `litric_files::trashinfo` — and the two writers cannot drift. Two
            // copies is how one defect came to be fixed twice (BUG-0011) with a
            // third waiting quietly in the copy (BUG-0009).
            let record = litric_files::trashinfo::record(&src, &deleted_iso);
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.trash_list();
        }
        #[cfg(not(unix))]
        {
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            let info = self.checked_path("/.local/share/Trash/info", true)?;
            let files = self.checked_path("/.local/share/Trash/files", true)?;
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
                                // The stored value is an absolute host path — or,
                                // for an entry an older build wrote, a `file://`
                                // URI. `logical_location` maps both back to a
                                // logical path; anything else it hands back
                                // verbatim, so it cannot be mistaken for one.
                                original = Some(litric_files::trashinfo::logical_location(
                                    &self.host_root,
                                    &litric_files::trashinfo::pct_decode(v),
                                ));
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
                // The logical original is `/Documents/x`; the restore
                // destination is its directory part.
                let restore_dir = original
                    .rsplit_once('/')
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.trash_restore(name);
        }
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
            let src = self
                .checked_path("/.local/share/Trash/files", true)?
                .join(name);
            let dest_dir = self.checked_path(&entry.restore_dir, true)?;
            if !dest_dir.is_dir() {
                return Err(FsError::NotADirectory);
            }
            let dest = dest_dir.join(name);
            if std::fs::symlink_metadata(&dest).is_ok() {
                return Err(FsError::Exists);
            }
            std::fs::rename(&src, &dest).map_err(|e| map_err("restore", &e))?;
            let info = self.checked_path("/.local/share/Trash/info", true)?;
            let _ = std::fs::remove_file(info.join(format!("{name}.trashinfo")));
            Ok(entry.restore_dir.clone())
        }
    }

    /// Delete one trash entry for ever.
    ///
    /// # Errors
    /// As [`Self::delete`] on the trash name.
    pub fn trash_purge(&self, name: &str) -> Result<(), FsError> {
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.trash_purge(name);
        }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.trash_empty();
        }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.read_text(logical);
        }
        #[cfg(not(unix))]
        {
            let _ = logical;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            const MAX: u64 = 512 * 1024;
            let p = self.read_path(logical)?;
            let meta = std::fs::symlink_metadata(&p).map_err(|e| map_err("read", &e))?;
            if !meta.is_file() {
                return Err(FsError::NotADirectory);
            }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.write_text(parent, name, content);
        }
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
            let dir = self.checked_path(parent, true)?;
            let target = dir.join(name);
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.search(from, query);
        }
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
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            return daemon.compress(dest, sources);
        }
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
            let dest_path = self.checked_path(dest, true)?;
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
        // The daemon is told the destination explicitly rather than deriving
        // it, so both ends agree on where an archive lands.
        // The daemon owns the user's documents (ADR-0019), so the operation
        // does not happen in this process at all.
        #[cfg(unix)]
        if let Some(daemon) = &self.daemon {
            let out_dir = extract_dir(archive);
            daemon.extract(archive, &out_dir)?;
            return Ok(out_dir);
        }
        #[cfg(not(unix))]
        {
            let _ = archive;
            Err(FsError::Unsupported)
        }
        #[cfg(unix)]
        {
            use std::io::Read as _;
            let p = self.read_path(archive)?;
            let mut bytes = Vec::new();
            std::fs::File::open(&p)
                .and_then(|mut f| f.read_to_end(&mut bytes))
                .map_err(|e| map_err("extract", &e))?;
            let out_dir = extract_dir(archive);
            let out_host = self.checked_path(&out_dir, true)?;
            if std::fs::symlink_metadata(&out_host).is_ok() {
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

/// Where an archive's contents are written: a folder beside it, named after the
/// archive with its extension removed.
///
/// One implementation, used by the direct path and by the daemon path. A second
/// copy is how the two ends would come to disagree about where an extraction
/// lands — the class of defect this module has already paid for once
/// (BUG-0009/BUG-0011).
fn extract_dir(archive: &str) -> String {
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
    if parent.ends_with('/') {
        format!("{parent}{stem}")
    } else {
        format!("{parent}/{stem}")
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

// The `Path=` field's encoding and decoding, and the whole record's shape, used
// to be spelled out here as well as in the daemon. Both writers now call
// `litric_files::trashinfo`, so "the two agree" is a property of the code rather
// than a promise in a comment — and, because that module is not `#[cfg(unix)]`,
// the format is testable on a host that has no Unix at all.

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
    fn hidden_entries_are_listed_only_when_the_caller_asks() {
        let root = temp_root("hidden");
        std::fs::create_dir_all(root.join("Documents")).expect("fixture folder");
        std::fs::write(root.join("Documents/visible.txt"), b"visible").expect("visible file");
        std::fs::write(root.join("Documents/.secret"), b"hidden").expect("hidden file");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).expect("bridge");

        let hidden_excluded = b.list_dir("/Documents").expect("listing");
        assert!(
            hidden_excluded.iter().all(|entry| entry.name != ".secret"),
            "a normal listing hides dot entries: {hidden_excluded:?}"
        );
        assert!(
            hidden_excluded
                .iter()
                .any(|entry| entry.name == "visible.txt"),
            "a normal listing still shows visible entries"
        );

        let hidden_included = b
            .list_dir_with_hidden("/Documents", true)
            .expect("listing with hidden entries");
        assert!(
            hidden_included.iter().any(|entry| entry.name == ".secret"),
            "the explicit request returns the dot entry: {hidden_included:?}"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn rename_refuses_an_existing_destination_and_keeps_both_files() {
        let root = temp_root("rename-exists");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.write_text("/", "old.txt", "old").unwrap();
        b.write_text("/", "new.txt", "new").unwrap();
        assert!(matches!(
            b.rename("/", "old.txt", "new.txt"),
            Err(FsError::Exists)
        ));
        assert_eq!(b.read_text("/old.txt").as_deref(), Ok("old"));
        assert_eq!(b.read_text("/new.txt").as_deref(), Ok("new"));
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
    fn the_trash_record_holds_the_absolute_location_not_a_uri() {
        // BUG-0009, shell half. The shell writes the same directory the daemon
        // does, so what it records has to be the item's real host location and
        // has to be readable by the daemon's reader — otherwise the two writers
        // share a directory they disagree about.
        let root = temp_root("tr-abs");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.write_text("/", "gone.txt", "bye").unwrap();
        b.trash("/gone.txt", 1_900_000_000).expect("trash");

        let raw = std::fs::read_to_string(root.join(".local/share/Trash/info/gone.txt.trashinfo"))
            .expect("the record was written");
        let value = raw
            .lines()
            .find_map(|l| l.strip_prefix("Path="))
            .expect("a Path= line");
        let decoded = litric_files::trashinfo::pct_decode(value);
        assert_eq!(decoded, root.join("gone.txt").to_string_lossy());
        assert!(
            litric_files::trashinfo::is_not_a_uri(&decoded),
            "Path= must be a location, not a URI: {decoded:?}"
        );
        // The shared reader — the one the daemon's `DiskTrash` uses — maps the
        // same bytes back to the logical path, so the entry restores in place.
        assert_eq!(
            litric_files::trashinfo::logical_location(&root, &decoded),
            "/gone.txt"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_record_written_in_the_legacy_uri_form_still_lists_and_restores() {
        // Changing the format must not strand a trash an older build wrote.
        let root = temp_root("tr-legacy");
        let base = root.join(".local/share/Trash");
        std::fs::create_dir_all(base.join("files")).unwrap();
        std::fs::create_dir_all(base.join("info")).unwrap();
        std::fs::create_dir_all(root.join("Documents")).unwrap();
        std::fs::write(base.join("files/old.txt"), "old").unwrap();
        std::fs::write(
            base.join("info/old.txt.trashinfo"),
            "[Trash Info]\nPath=file%3A///Documents/old.txt\nDeletionDate=2026-01-01T00:00:00\n",
        )
        .unwrap();

        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        let list = b.trash_list().expect("list");
        assert_eq!(list.len(), 1, "the legacy entry is still listed");
        assert_eq!(
            list.first().map(|e| e.restore_dir.as_str()),
            Some("/Documents")
        );
        b.trash_restore("old.txt").expect("restores");
        assert!(root.join("Documents/old.txt").exists());
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

    #[cfg(unix)]
    #[test]
    fn storage_initialisation_is_idempotent_and_never_removes_anything() {
        let root = temp_root("init");
        let storage = root.join("var/lib/litric-files/root");
        ensure_storage(&storage).expect("first boot creates the share");
        for name in DEFAULT_STORAGE_DIRS {
            assert!(
                storage.join(name).is_dir(),
                "{name} is created for the Files application"
            );
        }
        // The first boot must leave a usable file alone, and a restart must not
        // recreate an empty share over it.
        std::fs::write(storage.join("Documents/notes.txt"), "keep me").expect("user file");
        ensure_storage(&storage).expect("second boot is a no-op");
        assert_eq!(
            std::fs::read_to_string(storage.join("Documents/notes.txt")).expect("readable"),
            "keep me"
        );
        // A root that exists as a file is reported, not replaced.
        let not_a_dir = root.join("not-a-dir");
        std::fs::write(&not_a_dir, "x").expect("fixture");
        assert!(matches!(
            ensure_storage(&not_a_dir),
            Err(FsError::NotADirectory)
        ));
        // A quick check of the mode: the share itself is private.
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&storage)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o700, "the file share is not readable by others");
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_in_a_parent_is_refused_and_nothing_is_written_through_it() {
        let root = temp_root("symlink-parent");
        let outside = temp_root("symlink-outside");
        ensure_storage(&root).expect("storage");
        std::os::unix::fs::symlink(&outside, root.join("Documents/link")).expect("fixture link");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();

        assert!(
            matches!(b.mkdir("/Documents/link", "escape"), Err(FsError::Denied)),
            "a folder cannot be created through a link"
        );
        assert!(
            matches!(
                b.write_text("/Documents/link", "escape.txt", "x"),
                Err(FsError::Denied)
            ),
            "a file cannot be written through a link"
        );
        assert!(
            matches!(b.list_dir("/Documents/link"), Err(FsError::Denied)),
            "a link is not listed through"
        );
        assert!(!outside.join("escape").exists());
        assert!(!outside.join("escape.txt").exists());
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_link_to_a_file_cannot_be_read_through() {
        let root = temp_root("symlink-read");
        let outside = temp_root("symlink-secret");
        std::fs::write(outside.join("secret.txt"), "not yours").expect("fixture");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        std::os::unix::fs::symlink(outside.join("secret.txt"), root.join("passwd.txt"))
            .expect("fixture link");
        assert!(matches!(b.read_text("/passwd.txt"), Err(FsError::Denied)));
        // Deleting the link itself is still possible: that changes no file the
        // link names, and a user must be able to remove a link they can see.
        b.delete("/passwd.txt")
            .expect("the link itself is removable");
        assert!(outside.join("secret.txt").exists());
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_storage_root_that_is_a_link_is_refused() {
        let parent = temp_root("symlink-root");
        let real = temp_root("symlink-root-real");
        let linked = parent.join("root");
        std::os::unix::fs::symlink(&real, &linked).expect("fixture link");
        assert!(matches!(
            ensure_storage(&linked),
            Err(FsError::NotADirectory)
        ));
        assert!(matches!(
            FsBridge::open(&linked, litric_locale::TimeZone::utc("test")),
            Err(FsError::NotADirectory)
        ));
        std::fs::remove_dir_all(&parent).ok();
        std::fs::remove_dir_all(&real).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_trash_directory_replaced_by_a_link_is_not_written_through() {
        let root = temp_root("trash-link");
        let outside = temp_root("trash-outside");
        ensure_storage(&root).expect("storage");
        std::fs::create_dir_all(root.join(".local/share")).expect("fixture");
        std::os::unix::fs::symlink(&outside, root.join(".local/share/Trash"))
            .expect("fixture link");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        b.write_text("/Documents", "gone.txt", "bye").expect("file");
        assert!(
            matches!(
                b.trash("/Documents/gone.txt", 1_900_000_000),
                Err(FsError::Denied)
            ),
            "a trashed item is not moved through a link"
        );
        assert!(
            root.join("Documents/gone.txt").exists(),
            "the item stayed put"
        );
        assert!(std::fs::read_dir(&outside)
            .expect("outside")
            .next()
            .is_none());
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_traversing_or_absolute_old_name_is_refused_by_rename() {
        let root = temp_root("rename-escape");
        let outside = temp_root("rename-outside");
        std::fs::write(outside.join("secret.txt"), "x").expect("fixture");
        std::fs::write(root.join("ok.txt"), "ok").expect("fixture");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test")).unwrap();
        assert!(matches!(
            b.rename(
                "/",
                outside.join("secret.txt").to_str().unwrap_or(""),
                "stolen.txt"
            ),
            Err(FsError::BadName)
        ));
        assert!(outside.join("secret.txt").exists());
        assert!(matches!(
            b.rename("/", "ok.txt", "../moved.txt"),
            Err(FsError::BadName)
        ));
        assert!(root.join("ok.txt").exists());
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[test]
    fn dot_segments_and_traversal_are_refused() {
        let root = std::env::temp_dir().join("litric-fsbridge-dots");
        std::fs::create_dir_all(&root).expect("fixture");
        let b = FsBridge::open(&root, litric_locale::TimeZone::utc("test"));
        #[cfg(unix)]
        {
            let b = b.expect("bridge");
            assert!(matches!(b.host_path("/a/../b"), Err(FsError::BadName)));
            assert!(matches!(b.host_path("/./a"), Err(FsError::BadName)));
            assert!(matches!(b.host_path(".."), Err(FsError::BadName)));
        }
        #[cfg(not(unix))]
        assert!(b.is_err());
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn the_error_for_a_full_or_read_only_disk_is_a_sentence() {
        // Constructing the operating-system error directly is how both cases can
        // be tested without mounting a read-only filesystem or filling a disk.
        assert_eq!(
            map_err("write", &std::io::Error::from_raw_os_error(30)),
            FsError::ReadOnly
        );
        assert_eq!(
            map_err("write", &std::io::Error::from_raw_os_error(28)),
            FsError::Full
        );
        assert_eq!(
            FsError::ReadOnly.to_string(),
            "the filesystem is read-only, so nothing can be written there"
        );
        assert!(
            FsError::Full.to_string().contains("storage is full"),
            "a full disk says so"
        );
    }
}
