//! Application compatibility profiles and the path from a profile to a process.
//!
//! LitricOS has three ways to start a graphical application:
//!
//! * a native Linux executable installed below the user's data directory;
//! * a Flatpak application, which the Flatpak runtime starts;
//! * a Steam application, which Steam starts and Proton translates when the
//!   title needs a Windows runtime.
//!
//! This module is the single resolver for those three profiles. It does not
//! pretend a missing runtime is present: a profile whose launcher is absent is
//! refused with [`CompatError::ToolMissing`] before any process is spawned.
//! The executor is [`execute`], and it ultimately uses the same validated
//! `execve` path as every other launch in [`crate::launch`].
//!
//! Nothing here is a claim that a particular game works. Steam chooses the
//! Proton build and game-specific launch options; Flatpak owns the sandbox; and
//! a title with a Windows-only anti-cheat driver can still refuse at runtime.
//! The profile is a route to the vendor's launcher, not a compatibility verdict.

use std::path::{Path, PathBuf};

use crate::launch::{self, LaunchError};

/// The runtime a launch profile uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    /// A native Linux executable.
    LinuxNative,
    /// A Flatpak application, named by its reverse-DNS application id.
    Flatpak,
    /// A Steam application, started with `steam -applaunch`; Steam may use
    /// Proton for a Windows title.
    SteamProton,
}

impl Runtime {
    /// Parse the wire token for a runtime.
    ///
    /// `native` and `linux` both name the native profile. `proton` is accepted
    /// as an alias for the Steam profile because Proton is selected by Steam,
    /// not by a separate command line here.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "linux" | "native" => Some(Self::LinuxNative),
            "flatpak" => Some(Self::Flatpak),
            "steam" | "proton" | "steam-proton" => Some(Self::SteamProton),
            _ => None,
        }
    }

    /// The canonical token for this runtime.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::LinuxNative => "linux",
            Self::Flatpak => "flatpak",
            Self::SteamProton => "steam",
        }
    }
}

/// One requested launch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    /// Which runtime should start it.
    pub runtime: Runtime,
    /// The executable path for Linux, a Flatpak application id, or a Steam app
    /// id.
    pub target: String,
    /// Arguments passed to the application, after the runtime's own arguments.
    pub args: Vec<String>,
    /// The session display variables the daemon is allowed to add.
    pub env: Vec<(String, String)>,
}

impl LaunchRequest {
    /// Build a native request from the existing `LAUNCH`/`LAUNCHENV` vector.
    ///
    /// The first element remains the program path. It is not resolved against
    /// `PATH`, for the same reason [`crate::launch::validate`] does not: the
    /// request names the exact file it wants, and a different resolution would
    /// let the daemon choose a different binary.
    ///
    /// # Errors
    /// [`LaunchError::Empty`] when no program was named.
    pub fn native(argv: &[String], env: &[(String, String)]) -> Result<Self, LaunchError> {
        let Some((target, args)) = argv.split_first() else {
            return Err(LaunchError::Empty);
        };
        Ok(Self {
            runtime: Runtime::LinuxNative,
            target: target.clone(),
            args: args.to_vec(),
            env: env.to_vec(),
        })
    }
}

/// The programs used by the non-native profiles.
///
/// The production values are the ordinary system paths. Tests supply their own
/// tree so they can prove the resolver and executor without installing Steam or
/// Flatpak on the build host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPaths {
    /// The Flatpak command.
    pub flatpak: PathBuf,
    /// The Steam command.
    pub steam: PathBuf,
}

impl Default for ToolPaths {
    fn default() -> Self {
        Self {
            flatpak: PathBuf::from("/usr/bin/flatpak"),
            steam: PathBuf::from("/usr/bin/steam"),
        }
    }
}

/// A resolved launch, ready for [`crate::launch::launch_with_env`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    /// The profile that produced this plan.
    pub runtime: Runtime,
    /// The exact argument vector passed to `execve`, including the launcher.
    pub argv: Vec<String>,
    /// The validated display environment.
    pub env: Vec<(String, String)>,
}

/// Why a compatibility launch was refused or failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompatError {
    /// The runtime token is not one this build knows.
    UnknownRuntime(String),
    /// The target is not valid for the selected runtime.
    BadTarget(String),
    /// The runtime launcher is not installed or is not executable.
    ToolMissing {
        /// The runtime whose tool is missing.
        runtime: Runtime,
        /// The path that was checked.
        path: String,
    },
    /// The resolved process could not be started.
    Launch(LaunchError),
}

impl CompatError {
    /// The stable token the protocol reports.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::UnknownRuntime(_) | Self::BadTarget(_) => "bad-argument",
            Self::ToolMissing { .. } => "not-found",
            Self::Launch(error) => error.kind(),
        }
    }

    /// A sentence for the journal and the reply detail.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::UnknownRuntime(runtime) => {
                format!("{runtime} is not a known application runtime")
            }
            Self::BadTarget(reason) => format!("the application target is invalid: {reason}"),
            Self::ToolMissing { runtime, path } => {
                format!("{} is not available at {path}", runtime.token())
            }
            Self::Launch(error) => error.detail(),
        }
    }
}

/// Resolve a profile into the exact process that will be started.
///
/// # Errors
/// A [`CompatError`] if the target is malformed, the runtime launcher is
/// absent, or the resolved vector is outside the launch bounds.
pub fn resolve(request: &LaunchRequest, tools: &ToolPaths) -> Result<LaunchPlan, CompatError> {
    let env = request.env.clone();
    launch::validate_env(&env).map_err(CompatError::Launch)?;
    match request.runtime {
        Runtime::LinuxNative => {
            let mut argv = Vec::with_capacity(1_usize.saturating_add(request.args.len()));
            argv.push(request.target.clone());
            argv.extend(request.args.iter().cloned());
            launch::validate(&argv).map_err(CompatError::Launch)?;
            Ok(LaunchPlan {
                runtime: Runtime::LinuxNative,
                argv,
                env,
            })
        }
        Runtime::Flatpak => {
            if !valid_flatpak_id(&request.target) {
                return Err(CompatError::BadTarget(
                    "a Flatpak application id must be a dotted, slash-free id".to_owned(),
                ));
            }
            require_tool(Runtime::Flatpak, &tools.flatpak)?;
            let mut argv = Vec::with_capacity(3_usize.saturating_add(request.args.len()));
            argv.push(tools.flatpak.display().to_string());
            argv.push("run".to_owned());
            argv.push(request.target.clone());
            argv.extend(request.args.iter().cloned());
            launch::validate(&argv).map_err(CompatError::Launch)?;
            Ok(LaunchPlan {
                runtime: Runtime::Flatpak,
                argv,
                env,
            })
        }
        Runtime::SteamProton => {
            if request.target.is_empty()
                || !request.target.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(CompatError::BadTarget(
                    "a Steam application id must be digits".to_owned(),
                ));
            }
            require_tool(Runtime::SteamProton, &tools.steam)?;
            let mut argv = Vec::with_capacity(3_usize.saturating_add(request.args.len()));
            argv.push(tools.steam.display().to_string());
            argv.push("-applaunch".to_owned());
            argv.push(request.target.clone());
            argv.extend(request.args.iter().cloned());
            launch::validate(&argv).map_err(CompatError::Launch)?;
            Ok(LaunchPlan {
                runtime: Runtime::SteamProton,
                argv,
                env,
            })
        }
    }
}

/// Resolve and start one profile.
///
/// The resolved vector is handed to [`crate::launch::launch_with_env`], the same
/// executor used by the native `LAUNCHENV` path. A resolver success therefore
/// does not bypass the launch bounds, environment allowlist, or executable-bit
/// check.
///
/// # Errors
/// A [`CompatError`] from resolution, or the spawn failure from the executor.
pub fn execute(request: &LaunchRequest, tools: &ToolPaths) -> Result<u32, CompatError> {
    let plan = resolve(request, tools)?;
    launch::launch_with_env(&plan.argv, &plan.env).map_err(CompatError::Launch)
}

fn require_tool(runtime: Runtime, path: &Path) -> Result<(), CompatError> {
    if launch::is_executable(path) {
        Ok(())
    } else {
        Err(CompatError::ToolMissing {
            runtime,
            path: path.display().to_string(),
        })
    }
}

fn valid_flatpak_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 255 {
        return false;
    }
    let Some(first) = value.as_bytes().first() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    value.split('.').all(|part| !part.is_empty())
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'_' || byte == b'-'
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-sysd-compat-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[cfg(unix)]
    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        let temp = dir.join(format!(".{name}.tmp"));
        std::fs::write(&temp, format!("#!/bin/sh\n{body}\n")).expect("write");
        std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        std::fs::rename(&temp, &path).expect("rename");
        path
    }

    fn wait_for(path: &Path) -> Option<String> {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if let Ok(text) = std::fs::read_to_string(path) {
                return Some(text);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    #[cfg(unix)]
    fn tool_tree(tag: &str, marker: &Path) -> (PathBuf, ToolPaths) {
        let dir = scratch(tag);
        let body = format!("printf '%s\\n' \"$@\" > '{}'", marker.display());
        let flatpak = script(&dir, "flatpak", &body);
        let steam = script(&dir, "steam", &body);
        (dir, ToolPaths { flatpak, steam })
    }

    #[test]
    fn runtime_tokens_are_explicit_and_canonical() {
        assert_eq!(Runtime::parse("linux"), Some(Runtime::LinuxNative));
        assert_eq!(Runtime::parse("NATIVE"), Some(Runtime::LinuxNative));
        assert_eq!(Runtime::parse("Flatpak"), Some(Runtime::Flatpak));
        assert_eq!(Runtime::parse("steam"), Some(Runtime::SteamProton));
        assert_eq!(Runtime::parse("proton"), Some(Runtime::SteamProton));
        assert_eq!(Runtime::parse("wine"), None);
    }

    #[cfg(unix)]
    #[test]
    fn a_native_profile_resolves_and_executes_with_the_session_environment() {
        let dir = scratch("native");
        let marker = dir.join("native-ran");
        let program = script(
            &dir,
            "native-app",
            &format!(
                "printf '%s|%s' \"$WAYLAND_DISPLAY\" \"$1\" > '{}'",
                marker.display()
            ),
        );
        let request = LaunchRequest::native(
            &[program.display().to_string(), "argument".to_owned()],
            &[("WAYLAND_DISPLAY".to_owned(), "wayland-0".to_owned())],
        )
        .expect("native request");
        let plan = resolve(
            &request,
            &ToolPaths {
                flatpak: dir.join("missing-flatpak"),
                steam: dir.join("missing-steam"),
            },
        )
        .expect("native resolves without either external runtime");
        assert_eq!(plan.runtime, Runtime::LinuxNative);
        assert_eq!(
            plan.argv,
            vec![program.display().to_string(), "argument".to_owned()]
        );
        execute(
            &request,
            &ToolPaths {
                flatpak: dir.join("missing-flatpak"),
                steam: dir.join("missing-steam"),
            },
        )
        .expect("native executes");
        assert_eq!(wait_for(&marker).as_deref(), Some("wayland-0|argument"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn flatpak_resolves_to_run_and_the_executor_reaches_the_tool() {
        let marker = std::env::temp_dir().join(format!(
            "litric-sysd-compat-flatpak-args-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&marker);
        let (dir, tools) = tool_tree("flatpak", &marker);
        let request = LaunchRequest {
            runtime: Runtime::Flatpak,
            target: "org.example.Game".to_owned(),
            args: vec!["--fullscreen".to_owned()],
            env: Vec::new(),
        };
        let plan = resolve(&request, &tools).expect("flatpak resolves");
        assert_eq!(
            plan.argv,
            vec![
                tools.flatpak.display().to_string(),
                "run".to_owned(),
                "org.example.Game".to_owned(),
                "--fullscreen".to_owned(),
            ]
        );
        execute(&request, &tools).expect("flatpak executes");
        assert_eq!(
            wait_for(&marker).as_deref(),
            Some("run\norg.example.Game\n--fullscreen\n")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn steam_resolves_to_applaunch_and_the_executor_reaches_the_tool() {
        let marker = std::env::temp_dir().join(format!(
            "litric-sysd-compat-steam-args-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&marker);
        let (dir, tools) = tool_tree("steam", &marker);
        let request = LaunchRequest {
            runtime: Runtime::SteamProton,
            target: "12345".to_owned(),
            args: vec!["-windowed".to_owned()],
            env: Vec::new(),
        };
        let plan = resolve(&request, &tools).expect("steam resolves");
        assert_eq!(
            plan.argv,
            vec![
                tools.steam.display().to_string(),
                "-applaunch".to_owned(),
                "12345".to_owned(),
                "-windowed".to_owned(),
            ]
        );
        execute(&request, &tools).expect("steam executes");
        assert_eq!(
            wait_for(&marker).as_deref(),
            Some("-applaunch\n12345\n-windowed\n")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_profile_never_invents_a_missing_runtime() {
        let dir = scratch("missing-tool");
        let tools = ToolPaths {
            flatpak: dir.join("flatpak"),
            steam: dir.join("steam"),
        };
        for (runtime, target, tool) in [
            (Runtime::Flatpak, "org.example.Game", &tools.flatpak),
            (Runtime::SteamProton, "12345", &tools.steam),
        ] {
            let request = LaunchRequest {
                runtime,
                target: target.to_owned(),
                args: Vec::new(),
                env: Vec::new(),
            };
            assert_eq!(
                resolve(&request, &tools),
                Err(CompatError::ToolMissing {
                    runtime,
                    path: tool.display().to_string(),
                })
            );
            assert_eq!(
                execute(&request, &tools).map_err(|error| error.kind()),
                Err("not-found")
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_targets_are_refused_before_a_tool_is_started() {
        let tools = ToolPaths::default();
        for (runtime, target) in [
            (Runtime::Flatpak, "--socket=x11"),
            (Runtime::Flatpak, "org..Game"),
            (Runtime::Flatpak, "org.example/Game"),
            (Runtime::SteamProton, "12;rm"),
            (Runtime::SteamProton, "-1"),
        ] {
            let request = LaunchRequest {
                runtime,
                target: target.to_owned(),
                args: Vec::new(),
                env: Vec::new(),
            };
            assert!(
                matches!(resolve(&request, &tools), Err(CompatError::BadTarget(_))),
                "{runtime:?} accepted {target}"
            );
        }
        let relative = LaunchRequest {
            runtime: Runtime::LinuxNative,
            target: "relative-program".to_owned(),
            args: Vec::new(),
            env: Vec::new(),
        };
        assert_eq!(
            resolve(&relative, &tools),
            Err(CompatError::Launch(LaunchError::NotAbsolute))
        );
    }

    #[test]
    fn only_the_launch_environment_allowlist_is_accepted() {
        let request = LaunchRequest {
            runtime: Runtime::LinuxNative,
            target: "/bin/true".to_owned(),
            args: Vec::new(),
            env: vec![("LD_PRELOAD".to_owned(), "/tmp/evil.so".to_owned())],
        };
        assert!(matches!(
            resolve(&request, &ToolPaths::default()),
            Err(CompatError::Launch(LaunchError::BadEnv(_)))
        ));
    }
}
