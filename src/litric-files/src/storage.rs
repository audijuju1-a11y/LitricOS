//! The first-run layout of a user storage root.
//!
//! Two programs need this: `litric-filesd`, which owns the root once the image
//! is booted, and the shell, which opens a root directly on a fixture host
//! where no daemon runs. It used to live only in the shell, so the daemon would
//! have needed a second copy — and two copies of "which folders does a new
//! account start with" is how a user ends up with a `Documents` on one path
//! and not on the other.
//!
//! Nothing here deletes or replaces anything. The initialiser runs on every
//! start, so running it twice must be a no-op, and a name that exists as the
//! wrong kind of thing is an error rather than something to overwrite.

use crate::{Error, ErrorKind};
use std::path::Path;

/// The directories every user storage starts with.
///
/// These are the places the Files window and the desktop icons name, plus
/// `Projects`, which is one of the Files application's default places.
pub const STANDARD_DIRS: [&str; 7] = [
    "Documents",
    "Downloads",
    "Desktop",
    "Pictures",
    "Music",
    "Videos",
    "Projects",
];

/// Create `host` and the standard folders inside it, without disturbing what is
/// already there.
///
/// # Errors
/// `not-a-directory` when the root or a standard name exists as something else
/// (a file, or a symlink: the root is a real directory a systemd sandbox
/// opened, and a link there would move the whole share somewhere else);
/// otherwise the OS error, classified.
#[cfg(unix)]
pub fn ensure_storage(host: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;

    match std::fs::symlink_metadata(host) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(Error::new(
                ErrorKind::NotADirectory,
                "the storage root is a symbolic link, not a directory",
            ))
        }
        Ok(meta) if !meta.is_dir() => {
            return Err(Error::new(
                ErrorKind::NotADirectory,
                "the storage root is not a directory",
            ))
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(host).map_err(|e| Error::from_io("create storage", &e))?;
            // Private by default: this directory holds the user's documents.
            std::fs::set_permissions(host, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| Error::from_io("secure storage", &e))?;
        }
        Err(e) => return Err(Error::from_io("open storage", &e)),
    }
    for name in STANDARD_DIRS {
        let dir = host.join(name);
        match std::fs::symlink_metadata(&dir) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(Error::new(
                    ErrorKind::NotADirectory,
                    format!("{name} in the file storage exists but is not a folder"),
                ))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&dir).map_err(|e| Error::from_io("create folder", &e))?;
            }
            Err(e) => return Err(Error::from_io("create folder", &e)),
        }
    }
    Ok(())
}

/// Create the storage root on a host without Unix directory semantics.
///
/// # Errors
/// Always: there is no implementation to run.
#[cfg(not(unix))]
pub fn ensure_storage(host: &Path) -> Result<(), Error> {
    let _ = host;
    Err(Error::new(
        ErrorKind::Io,
        "litric-files creates file storage on Unix hosts only; \
         this build has no implementation for this platform",
    ))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "litric-storage-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_new_root_gets_every_standard_folder_once() {
        let root = scratch("fresh").join("root");
        ensure_storage(&root).expect("creates");
        for name in STANDARD_DIRS {
            assert!(root.join(name).is_dir(), "{name} missing");
        }
        // Idempotent: the initialiser runs on every start.
        ensure_storage(&root).expect("second run is a no-op");
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn a_standard_name_that_is_a_file_is_refused_not_replaced() {
        let root = scratch("file-in-the-way").join("root");
        std::fs::create_dir_all(&root).expect("mkdir");
        std::fs::write(root.join("Documents"), b"not a folder").expect("write");
        let e = ensure_storage(&root).expect_err("refuses");
        assert_eq!(e.kind(), ErrorKind::NotADirectory);
        // The file is still there: a refusal must not destroy anything.
        assert_eq!(
            std::fs::read(root.join("Documents")).expect("read"),
            b"not a folder"
        );
        let _ = std::fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn a_symlinked_root_is_refused() {
        let base = scratch("symlink");
        std::fs::create_dir_all(base.join("real")).expect("mkdir");
        std::os::unix::fs::symlink(base.join("real"), base.join("link")).expect("symlink");
        let e = ensure_storage(&base.join("link")).expect_err("refuses");
        assert_eq!(e.kind(), ErrorKind::NotADirectory);
        let _ = std::fs::remove_dir_all(&base);
    }
}
