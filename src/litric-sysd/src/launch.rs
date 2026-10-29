//! Starting an application on behalf of the session.
//!
//! The shell never forks a program itself. A launch is a request to
//! `litric-sysd`, which validates the argument vector here and then hands it to
//! `execve` through [`std::process::Command`]. There is no shell in the path:
//! the vector is already split, so a value containing `;`, `$(…)`, a pipe or a
//! redirection is one argument and can never become a second command.
//!
//! # What a launch is allowed to be
//!
//! * an absolute path to a regular file with an executable bit;
//! * a bounded vector ([`MAX_ARGV`], [`MAX_ARG_LEN`]);
//! * nothing else. A relative name is not resolved against `PATH` here, because
//!   resolving it would mean this module deciding which binary a name meant —
//!   the desktop entry has already done that against the same `PATH`.
//!
//! The child is detached from the daemon's streams and reaped by a short-lived
//! thread, so a launcher full of applications does not accumulate zombies or
//! block a request until the application exits.

use std::process::{Command, Stdio};

/// The most arguments one launch may carry.
pub const MAX_ARGV: usize = 32;

/// The longest single argument, in bytes.
pub const MAX_ARG_LEN: usize = 512;

/// Why a launch was refused or failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LaunchError {
    /// No program was named.
    Empty,
    /// More arguments than [`MAX_ARGV`].
    TooMany,
    /// One argument longer than [`MAX_ARG_LEN`].
    ArgTooLong,
    /// The program is not an absolute path.
    NotAbsolute,
    /// The path is missing, is not a regular file, or is not executable.
    NotExecutable,
    /// The program exists but could not be started.
    SpawnFailed(String),
}

impl LaunchError {
    /// The stable token the protocol reports.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Empty | Self::TooMany | Self::ArgTooLong | Self::NotAbsolute => "bad-argument",
            Self::NotExecutable => "not-found",
            Self::SpawnFailed(_) => "spawn-failed",
        }
    }

    /// A sentence for the journal and the reply detail.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Empty => "no program was named".to_owned(),
            Self::TooMany => format!("more than {MAX_ARGV} arguments"),
            Self::ArgTooLong => format!("an argument is longer than {MAX_ARG_LEN} bytes"),
            Self::NotAbsolute => "the program is not an absolute path".to_owned(),
            Self::NotExecutable => "the program is not an executable file".to_owned(),
            Self::SpawnFailed(reason) => format!("the program could not be started: {reason}"),
        }
    }
}

/// Whether `path` is a regular file with any execute bit set.
#[must_use]
pub fn is_executable(path: &std::path::Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Check an argument vector without starting anything.
///
/// # Errors
/// The first reason the vector may not be launched.
pub fn validate(argv: &[String]) -> Result<(), LaunchError> {
    let Some((program, rest)) = argv.split_first() else {
        return Err(LaunchError::Empty);
    };
    if argv.len() > MAX_ARGV {
        return Err(LaunchError::TooMany);
    }
    if rest
        .iter()
        .chain(std::iter::once(program))
        .any(|arg| arg.len() > MAX_ARG_LEN)
    {
        return Err(LaunchError::ArgTooLong);
    }
    if !program.starts_with('/') {
        return Err(LaunchError::NotAbsolute);
    }
    if !is_executable(std::path::Path::new(program)) {
        return Err(LaunchError::NotExecutable);
    }
    Ok(())
}

/// Start `argv` detached, returning the child's pid.
///
/// The child is not waited for by the caller. A thread waits for it so the
/// daemon does not keep a zombie per application; the thread exits when the
/// application does.
///
/// # Errors
/// A [`LaunchError`] for a vector that may not be launched, or an I/O failure
/// from the spawn itself.
pub fn launch(argv: &[String]) -> Result<u32, LaunchError> {
    validate(argv)?;
    let (program, rest) = argv.split_first().ok_or(LaunchError::Empty)?;
    let child = Command::new(program)
        .args(rest)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| LaunchError::SpawnFailed(error.kind().to_string()))?;
    let pid = child.id();
    // Reap without blocking the request. The thread owns the child handle.
    std::thread::Builder::new()
        .name("litric-sysd-reap".to_owned())
        .spawn(move || {
            let mut child = child;
            let _ = child.wait();
        })
        .map_err(|error| LaunchError::SpawnFailed(error.kind().to_string()))?;
    Ok(pid)
}

/// Hex encoding for one protocol argument: the wire splits on whitespace, and
/// an argument is a string a user's desktop entry wrote.
#[must_use]
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        out.push(char::from(
            HEX.get(usize::from(b >> 4)).copied().unwrap_or(b'0'),
        ));
        out.push(char::from(
            HEX.get(usize::from(b & 0x0f)).copied().unwrap_or(b'0'),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-sysd-launch-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[cfg(unix)]
    fn script(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        path
    }

    fn wait_for(path: &std::path::Path) -> Option<String> {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if let Ok(text) = std::fs::read_to_string(path) {
                return Some(text);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    #[test]
    fn an_empty_or_relative_program_is_refused_before_anything_runs() {
        assert_eq!(validate(&[]), Err(LaunchError::Empty));
        assert_eq!(validate(&["sh".to_owned()]), Err(LaunchError::NotAbsolute));
        assert_eq!(
            validate(&["/nonexistent/program".to_owned()]),
            Err(LaunchError::NotExecutable)
        );
    }

    #[test]
    fn a_vector_over_the_bounds_is_refused() {
        let too_many: Vec<String> = std::iter::once("/bin/sh".to_owned())
            .chain((0..MAX_ARGV).map(|i| format!("{i}")))
            .collect();
        assert_eq!(validate(&too_many), Err(LaunchError::TooMany));
        let long = vec!["/bin/sh".to_owned(), "x".repeat(MAX_ARG_LEN + 1)];
        assert_eq!(validate(&long), Err(LaunchError::ArgTooLong));
    }

    #[test]
    fn a_directory_and_a_non_executable_file_are_not_programs() {
        let dir = scratch("validate");
        assert_eq!(
            validate(&[dir.display().to_string()]),
            Err(LaunchError::NotExecutable)
        );
        let plain = dir.join("plain.sh");
        std::fs::write(&plain, b"#!/bin/sh\n").expect("write");
        assert_eq!(
            validate(&[plain.display().to_string()]),
            Err(LaunchError::NotExecutable)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_real_program_runs_and_its_marker_appears() {
        let dir = scratch("marker");
        let marker = dir.join("ran");
        let program = script(
            &dir,
            "writer",
            &format!("printf ok > '{}'", marker.display()),
        );
        let pid = launch(&[program.display().to_string()]).expect("launch");
        assert!(pid > 0);
        assert_eq!(wait_for(&marker).as_deref(), Some("ok"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn the_argument_vector_reaches_the_program_without_a_shell() {
        // A value that a shell would have interpreted is one argument here.
        let dir = scratch("argv");
        let out = dir.join("args");
        let program = script(
            &dir,
            "echo-args",
            &format!("printf '%s\\n' \"$@\" > '{}'", out.display()),
        );
        let value = "a;b $(touch should-not-exist) | c".to_owned();
        launch(&[program.display().to_string(), value.clone()]).expect("launch");
        let text = wait_for(&out).expect("the program wrote its arguments");
        assert_eq!(text.trim_end(), value);
        assert!(
            !dir.join("should-not-exist").exists(),
            "the value was interpreted as a command"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_program_that_disappears_after_validation_fails_honestly() {
        let dir = scratch("missing");
        let program = script(&dir, "gone", "exit 0");
        let argv = vec![program.display().to_string()];
        validate(&argv).expect("valid before deletion");
        std::fs::remove_file(&program).expect("remove");
        assert_eq!(launch(&argv), Err(LaunchError::NotExecutable));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hex_round_trips_through_the_protocol_encoding() {
        let encoded = hex_encode("a b\tc".as_bytes());
        assert_eq!(encoded, "6120620963");
        assert_eq!(crate::providers::unhex(&encoded), Some(b"a b\tc".to_vec()));
    }
}
