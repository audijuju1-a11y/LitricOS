//! Filesystem helpers shared by the state store and the staging engine.
//!
//! They are deliberately small and boring. The one operation worth naming is
//! [`remove_any`]: every removal in this crate goes through it, so the
//! repository's file-safety review has one site to inspect instead of one per
//! state write.

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Remove a file, a directory tree, or nothing at all.
///
/// Used only for paths this crate owns: a stale staging directory, one of the
/// two slot directories under `<root>/system/slots`, or the temporary state
/// file beside the state record. A missing path is success — the caller asked
/// for it to be absent, and it is.
pub(crate) fn remove_any(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Create a directory tree and, on Unix, make its final component owner-only.
pub(crate) fn ensure_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Replace `path` with `bytes` through a synced sibling temporary and a
/// directory fsync.
///
/// The temporary is named `.<file>.tmp-<pid>` in the same directory. A reader
/// only ever opens `path`, so a crash before the rename leaves the previous
/// complete record and a crash after it leaves the new one. A failed write
/// removes the temporary it created; a temporary left by another process is
/// replaced by that process's own name, never parsed.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> crate::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| crate::UpdateError::InvalidState("atomic path has no parent".to_owned()))?;
    ensure_private_dir(parent)
        .map_err(|error| crate::io_error("create atomic record directory", parent, error))?;
    let file_name = path.file_name().ok_or_else(|| {
        crate::UpdateError::InvalidState("atomic path has no file name".to_owned())
    })?;
    let mut temp_name = OsString::from(".");
    temp_name.push(file_name);
    temp_name.push(format!(".tmp-{}", std::process::id()));
    let temp = parent.join(temp_name);
    remove_any(&temp)
        .map_err(|error| crate::io_error("remove stale atomic temporary", &temp, error))?;

    let result = (|| -> crate::Result<()> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temp)
            .map_err(|error| crate::io_error("create atomic temporary", &temp, error))?;
        file.write_all(bytes)
            .map_err(|error| crate::io_error("write atomic record", &temp, error))?;
        file.sync_all()
            .map_err(|error| crate::io_error("sync atomic record", &temp, error))?;
        fs::rename(&temp, path)
            .map_err(|error| crate::io_error("publish atomic record", path, error))?;
        sync_dir(parent)
            .map_err(|error| crate::io_error("sync atomic directory", parent, error))?;
        Ok(())
    })();

    if result.is_err() {
        let _ = remove_any(&temp);
    }
    result
}

/// Flush a directory entry after a rename, so the new name survives a crash.
pub(crate) fn sync_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let dir = fs::File::open(path)?;
        dir.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Flush one regular file's contents before its name is published.
pub(crate) fn sync_file(path: &Path) -> io::Result<()> {
    let file = fs::File::open(path)?;
    file.sync_all()
}
