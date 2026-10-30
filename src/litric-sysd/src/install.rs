//! Installing a Linux application from a local, validated bundle.
//!
//! The bundle lives under `<application-root>/litric/incoming/<bundle>` and
//! contains exactly two regular files:
//!
//! ```text
//! app           an executable
//! app.desktop   [Desktop Entry] with Type=Application, Name=..., Exec=app
//! ```
//!
//! Installation copies the executable into
//! `<application-root>/litric/apps/<id>/app` and writes
//! `<application-root>/applications/<id>.desktop` with an absolute `Exec=`.
//! Uninstall removes those two paths and nothing else.
//!
//! # Why the bundle name is not a path
//!
//! The request names one directory below the service's own `incoming` root. It
//! may not contain `/`, `.` or `..`, so a request cannot make the daemon read
//! outside the staging tree. Inside that directory every entry is checked with
//! `symlink_metadata`; a link is refused rather than followed.
//!
//! # What this is not
//!
//! There is no package manager, network download, signature verifier or root
//! operation here. The bundle is local, its executable is copied, and the
//! desktop entry is generated from a small validated description. The owner of
//! the application root is the daemon's user; a production image must run this
//! path as the session user, not as root.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use litric_core::AppId;

/// The most bytes copied for one application executable.
pub const MAX_APP_BYTES: u64 = 64 * 1024 * 1024;

/// The most bytes read from a bundle's desktop file.
pub const MAX_DESKTOP_BYTES: u64 = 64 * 1024;

/// The longest display name accepted from a bundle.
pub const MAX_NAME: usize = 128;

/// Why an install or uninstall was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InstallError {
    /// The application id is not a valid reverse-DNS id.
    BadId,
    /// The bundle name is empty, too long, or not one path segment.
    BadBundle,
    /// The bundle directory does not exist.
    MissingBundle,
    /// The bundle contains a link, a device, or an unexpected shape.
    UnsafeBundle,
    /// The bundle's executable is absent, not regular, or not executable.
    BadExecutable,
    /// The desktop file is absent, oversized, or not a usable application.
    BadDesktop,
    /// An application with this id is already installed.
    Exists,
    /// There is no installed application with this id.
    NotInstalled,
    /// The install root could not be prepared or written.
    Io {
        /// What was being attempted.
        action: &'static str,
        /// The OS error kind.
        kind: std::io::ErrorKind,
    },
}

impl InstallError {
    /// The stable protocol token.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::BadId => "bad-id",
            Self::BadBundle => "bad-bundle",
            Self::MissingBundle => "not-found",
            Self::UnsafeBundle | Self::BadExecutable => "unsafe-bundle",
            Self::BadDesktop => "bad-desktop",
            Self::Exists => "exists",
            Self::NotInstalled => "not-installed",
            Self::Io { .. } => "install-io",
        }
    }

    /// A short sentence that never includes a path or request text.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::BadId => "the application id is not a valid reverse-DNS id".to_owned(),
            Self::BadBundle => "the bundle name is not one safe path segment".to_owned(),
            Self::MissingBundle => "the bundle directory does not exist".to_owned(),
            Self::UnsafeBundle => "the bundle contains a link or an unsupported entry".to_owned(),
            Self::BadExecutable => "the bundle executable is missing or not executable".to_owned(),
            Self::BadDesktop => "the bundle desktop file is not a usable application".to_owned(),
            Self::Exists => "an application with this id is already installed".to_owned(),
            Self::NotInstalled => "no installed application has this id".to_owned(),
            Self::Io { action, kind } => format!("{action}: {kind:?}"),
        }
    }
}

/// One validated desktop description from a bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Description {
    name: String,
}

/// A successfully installed application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The canonical application id.
    pub id: String,
    /// The executable's absolute path inside the application root.
    pub executable: PathBuf,
}

/// Install `bundle` as `id` below `root`.
///
/// `root` is the XDG data root (`~/.local/share`), not the applications
/// directory: the installer owns both `applications/` and `litric/apps/` there.
///
/// # Errors
/// Every refusal is a typed [`InstallError`]; no partial application is
/// advertised when a step fails.
pub fn install(root: &Path, id: &str, bundle: &str) -> Result<Installed, InstallError> {
    let id = AppId::parse(id).map_err(|_| InstallError::BadId)?;
    let bundle = validate_bundle_name(bundle)?;
    let incoming = root.join("litric/incoming");
    let apps = root.join("litric/apps");
    let applications = root.join("applications");
    let bundle_dir = incoming.join(bundle);
    let bundle_meta = fs::symlink_metadata(&bundle_dir).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            InstallError::MissingBundle
        } else {
            io_error("inspect bundle", error)
        }
    })?;
    if bundle_meta.file_type().is_symlink() || !bundle_meta.is_dir() {
        return Err(InstallError::UnsafeBundle);
    }

    let executable_source = bundle_dir.join("app");
    let executable_meta = fs::symlink_metadata(&executable_source).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            InstallError::BadExecutable
        } else {
            io_error("inspect executable", error)
        }
    })?;
    if executable_meta.file_type().is_symlink()
        || !executable_meta.is_file()
        || executable_meta.len() > MAX_APP_BYTES
        || !is_executable(&executable_meta)
    {
        return Err(InstallError::BadExecutable);
    }

    let desktop_path = bundle_dir.join("app.desktop");
    let desktop = read_regular(&desktop_path, MAX_DESKTOP_BYTES)?;
    let description = parse_desktop(&desktop)?;

    let target = apps.join(id.as_path_component());
    let desktop_target = applications.join(format!("{}.desktop", id.as_path_component()));
    if fs::symlink_metadata(&target).is_ok() || fs::symlink_metadata(&desktop_target).is_ok() {
        return Err(InstallError::Exists);
    }

    fs::create_dir_all(&apps).map_err(|error| io_error("create app directory", error))?;
    fs::create_dir_all(&applications)
        .map_err(|error| io_error("create applications directory", error))?;

    let staging = apps.join(format!(
        ".{}.tmp.{}",
        id.as_path_component(),
        std::process::id()
    ));
    // A process id is reused across boots; a stale partial left by a killed
    // daemon must be removed before this install starts, not renamed into the
    // target. It is inside the installer's own apps directory.
    if fs::symlink_metadata(&staging).is_ok() {
        fs::remove_dir_all(&staging)
            .map_err(|error| io_error("remove stale staging directory", error))?;
    }
    fs::create_dir(&staging).map_err(|error| io_error("create staging directory", error))?;

    let mut committed = false;
    let result = (|| -> Result<(), InstallError> {
        let staged_app = staging.join("app");
        fs::copy(&executable_source, &staged_app)
            .map_err(|error| io_error("copy executable", error))?;
        set_executable(&staged_app)?;
        let desktop_body = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec={}\nX-Litric-AppId={}\n",
            description.name,
            quote_exec(&target.join("app")),
            id
        );
        write_new(&staging.join("app.desktop"), desktop_body.as_bytes(), 0o644)?;
        fs::rename(&staging, &target).map_err(|error| io_error("commit application", error))?;
        committed = true;
        write_atomic(&desktop_target, desktop_body.as_bytes())?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        if committed {
            let _ = fs::remove_dir_all(&target);
        }
        let _ = fs::remove_file(&desktop_target);
        return Err(error);
    }

    Ok(Installed {
        id: id.as_str().to_owned(),
        executable: target.join("app"),
    })
}

/// Remove the application and its desktop entry.
///
/// Only the exact paths for a validated id are touched. A link at either path
/// is refused rather than followed.
///
/// # Errors
/// [`InstallError::NotInstalled`] when neither path exists, and a typed I/O
/// failure when a removal fails.
pub fn uninstall(root: &Path, id: &str) -> Result<(), InstallError> {
    let id = AppId::parse(id).map_err(|_| InstallError::BadId)?;
    let target = root.join("litric/apps").join(id.as_path_component());
    let desktop = root
        .join("applications")
        .join(format!("{}.desktop", id.as_path_component()));

    let app_meta = fs::symlink_metadata(&target);
    let desktop_meta = fs::symlink_metadata(&desktop);
    if app_meta.is_err() && desktop_meta.is_err() {
        return Err(InstallError::NotInstalled);
    }
    if app_meta
        .as_ref()
        .is_ok_and(|m| m.file_type().is_symlink() || !m.is_dir())
        || desktop_meta
            .as_ref()
            .is_ok_and(|m| m.file_type().is_symlink() || !m.is_file())
    {
        return Err(InstallError::UnsafeBundle);
    }

    if desktop_meta.is_ok() {
        fs::remove_file(&desktop).map_err(|error| io_error("remove desktop entry", error))?;
    }
    if app_meta.is_ok() {
        fs::remove_dir_all(&target).map_err(|error| io_error("remove application", error))?;
    }
    Ok(())
}

fn validate_bundle_name(bundle: &str) -> Result<&str, InstallError> {
    if bundle.is_empty()
        || bundle.len() > 128
        || bundle == "."
        || bundle == ".."
        || bundle.starts_with('.')
    {
        return Err(InstallError::BadBundle);
    }
    if !bundle
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return Err(InstallError::BadBundle);
    }
    Ok(bundle)
}

fn read_regular(path: &Path, max: u64) -> Result<String, InstallError> {
    let meta = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            InstallError::BadDesktop
        } else {
            io_error("inspect desktop file", error)
        }
    })?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > max {
        return Err(InstallError::BadDesktop);
    }
    let mut file = fs::File::open(path).map_err(|error| io_error("open desktop file", error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| io_error("read desktop file", error))?;
    if bytes.len() as u64 > max {
        return Err(InstallError::BadDesktop);
    }
    String::from_utf8(bytes).map_err(|_| InstallError::BadDesktop)
}

fn parse_desktop(text: &str) -> Result<Description, InstallError> {
    let mut in_group = false;
    let mut values: Vec<(String, String)> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_group = line.eq_ignore_ascii_case("[Desktop Entry]");
            continue;
        }
        if !in_group {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_owned();
        if values.iter().any(|(seen, _)| *seen == key) {
            return Err(InstallError::BadDesktop);
        }
        values.push((key, value.trim().to_owned()));
    }
    let get = |key: &str| {
        values
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    };
    if get("Type") != Some("Application") {
        return Err(InstallError::BadDesktop);
    }
    if get("NoDisplay") == Some("true")
        || get("Hidden") == Some("true")
        || get("Terminal") == Some("true")
    {
        return Err(InstallError::BadDesktop);
    }
    if get("Exec") != Some("app") {
        return Err(InstallError::BadDesktop);
    }
    let name = get("Name").ok_or(InstallError::BadDesktop)?.trim();
    if name.is_empty()
        || name.chars().count() > MAX_NAME
        || name
            .chars()
            .any(|c| c.is_control() || c == '\n' || c == '\r')
    {
        return Err(InstallError::BadDesktop);
    }
    Ok(Description {
        name: name.to_owned(),
    })
}

fn is_executable(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        true
    }
}

fn set_executable(path: &Path) -> Result<(), InstallError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .map_err(|error| io_error("set executable permissions", error))?;
    }
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8], mode: u32) -> Result<(), InstallError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let mut file = options
        .open(path)
        .map_err(|error| io_error("create file", error))?;
    file.write_all(bytes)
        .map_err(|error| io_error("write file", error))?;
    file.sync_all()
        .map_err(|error| io_error("sync file", error))
}

/// Quote an absolute program path for a desktop entry's `Exec` value.
///
/// The launcher's tokenizer understands double quotes and backslash escapes.
/// Quoting keeps a data root containing a space one argument instead of two; a
/// quote or backslash is escaped so it cannot terminate the quoted value.
fn quote_exec(path: &Path) -> String {
    let mut out = String::from("\"");
    for c in path.to_string_lossy().chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    let parent = path.parent().ok_or(InstallError::Io {
        action: "prepare desktop entry",
        kind: std::io::ErrorKind::InvalidInput,
    })?;
    let temp = parent.join(format!(
        ".{}.tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("desktop"),
        std::process::id()
    ));
    let _ = fs::remove_file(&temp);
    write_new(&temp, bytes, 0o644)?;
    fs::rename(&temp, path).map_err(|error| io_error("commit desktop entry", error))?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn io_error(action: &'static str, error: std::io::Error) -> InstallError {
    InstallError::Io {
        action,
        kind: error.kind(),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn scratch(tag: &str) -> std::io::Result<PathBuf> {
        let root =
            std::env::temp_dir().join(format!("litric-sysd-install-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    fn bundle(root: &Path, name: &str, body: &str) -> std::io::Result<PathBuf> {
        let incoming = root.join("litric/incoming");
        fs::create_dir_all(&incoming)?;
        let dir = incoming.join(name);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("app"), format!("#!/bin/sh\n{body}\n"))?;
        fs::set_permissions(dir.join("app"), fs::Permissions::from_mode(0o755))?;
        fs::write(
            dir.join("app.desktop"),
            "[Desktop Entry]\nType=Application\nName=Fixture App\nExec=app\n",
        )?;
        Ok(dir)
    }

    #[test]
    fn a_bundle_installs_the_executable_and_a_runnable_desktop_entry(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("install")?;
        bundle(&root, "fixture", "printf installed")?;
        let installed = install(&root, "com.example.fixture", "fixture").expect("install");
        assert_eq!(installed.id, "com.example.fixture");
        assert!(is_executable(
            &fs::metadata(&installed.executable).expect("stat")
        ));
        let desktop = fs::read_to_string(root.join("applications/com.example.fixture.desktop"))
            .expect("desktop");
        assert!(desktop.contains("Exec=\""), "{desktop}");
        assert!(desktop.contains("Name=Fixture App"), "{desktop}");
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn traversal_a_link_and_a_missing_executable_are_refused(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("refusals")?;
        assert_eq!(
            install(&root, "com.example.fixture", ".hidden"),
            Err(InstallError::BadBundle)
        );
        assert_eq!(
            install(&root, "com.example.fixture", "../escape"),
            Err(InstallError::BadBundle)
        );
        let dir = bundle(&root, "fixture", "exit 0")?;
        std::fs::rename(dir.join("app"), dir.join("real-app")).expect("move app");
        std::os::unix::fs::symlink("/bin/sh", dir.join("app")).expect("symlink");
        assert_eq!(
            install(&root, "com.example.fixture", "fixture"),
            Err(InstallError::BadExecutable)
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn duplicate_install_and_unknown_uninstall_are_typed_refusals(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("duplicate")?;
        bundle(&root, "fixture", "exit 0")?;
        install(&root, "com.example.fixture", "fixture").expect("first install");
        assert_eq!(
            install(&root, "com.example.fixture", "fixture"),
            Err(InstallError::Exists)
        );
        uninstall(&root, "com.example.fixture").expect("uninstall");
        assert_eq!(
            uninstall(&root, "com.example.fixture"),
            Err(InstallError::NotInstalled)
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn uninstall_removes_only_the_installed_paths() -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("uninstall")?;
        bundle(&root, "fixture", "exit 0")?;
        let canary = root.join("keep.txt");
        fs::write(&canary, b"keep").expect("canary");
        install(&root, "com.example.fixture", "fixture").expect("install");
        uninstall(&root, "com.example.fixture").expect("uninstall");
        assert!(canary.exists());
        assert!(!root
            .join("applications/com.example.fixture.desktop")
            .exists());
        assert!(!root.join("litric/apps/com.example.fixture").exists());
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn a_desktop_file_with_the_wrong_exec_or_name_is_refused(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = scratch("desktop")?;
        let dir = bundle(&root, "fixture", "exit 0")?;
        fs::write(
            dir.join("app.desktop"),
            "[Desktop Entry]\nType=Application\nName=\nExec=/bin/sh\n",
        )
        .expect("desktop");
        assert_eq!(
            install(&root, "com.example.fixture", "fixture"),
            Err(InstallError::BadDesktop)
        );
        cleanup(&root);
        Ok(())
    }
}
