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
//! * optionally, the two variables that describe the session's Wayland socket;
//! * nothing else. A relative name is not resolved against `PATH` here, because
//!   resolving it would mean this module deciding which binary a name meant —
//!   the desktop entry has already done that against the same `PATH`.
//!
//! A launched application does not inherit the shell's display: it is started by
//! the daemon, which has no `WAYLAND_DISPLAY` of its own. The launch can carry an
//! allowlisted environment ([`ALLOWED_ENV`]) so the shell can name the socket it
//! is serving. Arbitrary environment variables are refused: `LD_PRELOAD` and its
//! relatives are code injection, not display configuration.
//!
//! The child is detached from the daemon's streams and reaped by a short-lived
//! thread, so a launcher full of applications does not accumulate zombies or
//! block a request until the application exits.

use std::process::{Command, Stdio};

/// The most arguments one launch may carry.
pub const MAX_ARGV: usize = 32;

/// The longest single argument, in bytes.
pub const MAX_ARG_LEN: usize = 512;

/// The most environment entries one launch may carry.
pub const MAX_ENV: usize = 8;

/// The longest environment value, in bytes.
pub const MAX_ENV_VALUE_LEN: usize = 512;

/// The only environment names a launch may set.
///
/// Both are facts about where the session's compositor listens, and neither is
/// meaningful to the dynamic loader. A launch that could set `LD_PRELOAD` could
/// run a different program than the path it named, which is a second launch
/// authority hidden inside the first.
pub const ALLOWED_ENV: [&str; 2] = ["WAYLAND_DISPLAY", "XDG_RUNTIME_DIR"];

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
    /// An environment entry is not one of the allowed session variables.
    BadEnv(String),
    /// The program exists but could not be started.
    SpawnFailed(String),
}

impl LaunchError {
    /// The stable token the protocol reports.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Empty
            | Self::TooMany
            | Self::ArgTooLong
            | Self::NotAbsolute
            | Self::BadEnv(_) => "bad-argument",
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
            Self::BadEnv(reason) => format!("the launch environment is invalid: {reason}"),
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

/// Check the environment a launch may add without starting anything.
///
/// # Errors
/// The first entry that is not one of [`ALLOWED_ENV`] or has an unusable value.
pub fn validate_env(env: &[(String, String)]) -> Result<(), LaunchError> {
    if env.len() > MAX_ENV {
        return Err(LaunchError::BadEnv(format!("more than {MAX_ENV} entries")));
    }
    for (index, (name, value)) in env.iter().enumerate() {
        if !ALLOWED_ENV.contains(&name.as_str()) {
            return Err(LaunchError::BadEnv(format!(
                "{name} is not a session display variable"
            )));
        }
        if env.iter().take(index).any(|(earlier, _)| earlier == name) {
            return Err(LaunchError::BadEnv(format!("duplicate {name}")));
        }
        if value.len() > MAX_ENV_VALUE_LEN || value.contains('\0') {
            return Err(LaunchError::BadEnv(format!("{name} has an unusable value")));
        }
        match name.as_str() {
            "WAYLAND_DISPLAY"
                if value.is_empty() || value == "." || value == ".." || value.contains('/') =>
            {
                return Err(LaunchError::BadEnv(
                    "WAYLAND_DISPLAY is not a display name".to_owned(),
                ));
            }
            "XDG_RUNTIME_DIR" if !value.starts_with('/') => {
                return Err(LaunchError::BadEnv(
                    "XDG_RUNTIME_DIR is not an absolute path".to_owned(),
                ));
            }
            _ => {}
        }
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
    launch_with_env(argv, &[])
}

/// Start `argv` detached with the session's display environment, returning the
/// child's pid.
///
/// The entries are appended to the daemon's own environment, which still
/// carries locale and `PATH`. Nothing about the daemon is modified.
///
/// # Errors
/// A [`LaunchError`] for a vector or environment that may not be launched, or an
/// I/O failure from the spawn itself.
pub fn launch_with_env(argv: &[String], env: &[(String, String)]) -> Result<u32, LaunchError> {
    validate(argv)?;
    validate_env(env)?;
    let (program, rest) = argv.split_first().ok_or(LaunchError::Empty)?;
    let mut command = Command::new(program);
    command
        .args(rest)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, value) in env {
        command.env(name, value);
    }
    let child = command
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
    fn the_display_environment_reaches_the_program() {
        let dir = scratch("env");
        let marker = dir.join("display");
        let program = script(
            &dir,
            "display-writer",
            &format!("printf '%s' \"$WAYLAND_DISPLAY\" > '{}'", marker.display()),
        );
        launch_with_env(
            &[program.display().to_string()],
            &[("WAYLAND_DISPLAY".to_owned(), "wayland-0".to_owned())],
        )
        .expect("launch with a display");
        assert_eq!(wait_for(&marker).as_deref(), Some("wayland-0"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_the_session_display_variables_may_be_added() {
        let preload = [("LD_PRELOAD".to_owned(), "/tmp/evil.so".to_owned())];
        assert!(matches!(
            validate_env(&preload),
            Err(LaunchError::BadEnv(_))
        ));
        assert!(
            validate_env(&[("WAYLAND_DISPLAY".to_owned(), "../not-a-display".to_owned())]).is_err()
        );
        assert!(validate_env(&[("XDG_RUNTIME_DIR".to_owned(), "relative".to_owned())]).is_err());
        assert!(validate_env(&[
            ("WAYLAND_DISPLAY".to_owned(), "wayland-0".to_owned()),
            ("WAYLAND_DISPLAY".to_owned(), "wayland-1".to_owned()),
        ])
        .is_err());
        assert!(validate_env(&[
            ("WAYLAND_DISPLAY".to_owned(), "wayland-0".to_owned()),
            ("XDG_RUNTIME_DIR".to_owned(), "/run/user/1000".to_owned()),
        ])
        .is_ok());
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
