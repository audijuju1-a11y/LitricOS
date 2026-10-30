//! Filesystem helpers shared by the state store and the staging engine.
//!
//! They are deliberately small and boring. The one operation worth naming is
//! [`remove_any`]: every removal in this crate goes through it, so the
//! repository's file-safety review has one site to inspect instead of one per
//! state write.

use std::fs;
use std::io;
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
