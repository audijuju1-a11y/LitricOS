//! `litric-filesd` — the Litric Files daemon (ADR-0019).
//!
//! # What it is for
//!
//! The Files application used to read and write the user's documents from
//! inside the desktop shell: the same process that draws pixels, holds every
//! font, and can lie on the screen also held the right to delete a directory
//! tree. This daemon takes that right away. The shell asks over LFP/1 and the
//! I/O happens here, so the code that can touch a document is a program that
//! draws nothing.
//!
//! # Modes
//!
//! ```text
//!   litric-filesd --stdio                  serve the one connection systemd handed us
//!   litric-filesd --socket /run/litric/files.sock   bind and accept (standalone)
//!   litric-filesd --info                   print the root it would serve and exit
//! ```
//!
//! `--stdio` is what the unit uses: `Accept=yes` plus `StandardInput=socket`
//! means systemd accepts the connection and starts one instance per client,
//! with the socket already on stdin/stdout. Nothing listens on a port, nothing
//! is reachable by another user, and the daemon cannot outlive its client.

#![forbid(unsafe_code)]

use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[cfg(unix)]
use litric_files::server::{serve_stream, serve_unix};
#[cfg(unix)]
use litric_files::Root;

/// The version reported by `--info`.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the socket lives by default. Under `/run` because it must not survive
/// a reboot; the image's `/` is read-only erofs and could not hold it anyway.
const DEFAULT_SOCKET: &str = "/run/litric/files.sock";

/// The root the unit serves: the `litric-files` StateDirectory of the shell
/// unit, so the Files application and the daemon name the same directory.
const DEFAULT_ROOT: &str = "/var/lib/litric-files/root";

/// What the command line asked for.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// Serve the connection already open on stdin/stdout (socket activation).
    Stdio,
    /// Bind a socket and accept connections.
    Socket(PathBuf),
    /// Report the root, its folders and the time zone, then exit.
    Info,
}

/// A parsed command line.
#[derive(Debug, PartialEq, Eq)]
struct Args {
    mode: Mode,
    root: PathBuf,
    /// The one directory `IMPORT` may read a staged file from, and only by bare
    /// name. `None` disables IMPORT with a reason instead of guessing.
    import_dir: Option<PathBuf>,
}

const USAGE: &str = "\
litric-filesd — the Litric Files daemon (LFP/1)

USAGE:
    litric-filesd --stdio [--root <dir>]
    litric-filesd --socket <path> [--root <dir>]
    litric-filesd --info [--root <dir>]

OPTIONS:
    --stdio           serve the connection systemd passed on stdin/stdout
    --socket <path>   bind this socket and accept connections (standalone)
    --info            print what would be served, then exit
    --root <dir>      the user storage root          (default: /var/lib/litric-files/root)
    --import-dir <d>  where IMPORT may read a staged download from, by bare name
    -h, --help        print this text
    -V, --version     print the version";

/// Parse the command line.
///
/// # Errors
/// A sentence naming the argument that was wrong.
fn parse_args(raw: &[String]) -> Result<Option<Args>, String> {
    let mut mode: Option<Mode> = None;
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut import_dir: Option<PathBuf> = None;
    let mut i = 0;
    while let Some(arg) = raw.get(i) {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "-V" | "--version" => {
                println!("litric-filesd {VERSION}");
                return Ok(None);
            }
            "--stdio" => set_mode(&mut mode, Mode::Stdio, "--stdio")?,
            "--info" => set_mode(&mut mode, Mode::Info, "--info")?,
            "--socket" => {
                let Some(path) = raw.get(i.saturating_add(1)) else {
                    return Err("--socket needs a path".to_owned());
                };
                set_mode(&mut mode, Mode::Socket(PathBuf::from(path)), "--socket")?;
                i = i.saturating_add(1);
            }
            "--root" => {
                let Some(dir) = raw.get(i.saturating_add(1)) else {
                    return Err("--root needs a directory".to_owned());
                };
                root = PathBuf::from(dir);
                i = i.saturating_add(1);
            }
            "--import-dir" => {
                let Some(dir) = raw.get(i.saturating_add(1)) else {
                    return Err("--import-dir needs a directory".to_owned());
                };
                import_dir = Some(PathBuf::from(dir));
                i = i.saturating_add(1);
            }
            other => {
                return Err(format!("unrecognised argument: {other}"));
            }
        }
        i = i.saturating_add(1);
    }
    match mode {
        Some(mode) => Ok(Some(Args {
            mode,
            root,
            import_dir,
        })),
        None => Err("one of --stdio, --socket or --info is required".to_owned()),
    }
}

fn set_mode(slot: &mut Option<Mode>, mode: Mode, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("{flag} given more than once, or with another mode"));
    }
    *slot = Some(mode);
    Ok(())
}

/// The time zone used for human-facing dates (the trash deletion date).
///
/// The daemon serves one machine, so it reads the machine's zone: `TZ` when the
/// unit sets it, otherwise `/etc/localtime`. A missing zone is UTC with a
/// reason, never a guess — the deletion date is a record the user reads back
/// when they decide what to restore.
fn zone() -> litric_locale::TimeZone {
    let tz = std::env::var("TZ").ok();
    litric_locale::detect(Path::new("/"), tz.as_deref())
}
fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&raw) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(why) => {
            eprintln!("litric-filesd: {why}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            // stderr is the journal for the unit. A daemon that cannot open its
            // root must not accept a connection and then fail every request:
            // that looks like the Files application being broken.
            eprintln!("litric-filesd: {why}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    if args.mode == Mode::Info {
        return info(&args.root, args.import_dir.as_deref());
    }
    serve(args)
}

/// Serve LFP/1, in the mode the command line asked for.
///
/// # Errors
/// A root that cannot be prepared, and whatever the transport raises.
#[cfg(unix)]
fn serve(args: &Args) -> Result<(), String> {
    // The storage root is created here, not by the shell: the daemon owns the
    // directory that holds the user's documents, so the first-run layout is
    // laid down by the program that will serve it. `ensure_storage` is a no-op
    // when everything already exists.
    litric_files::ensure_storage(&args.root).map_err(|e| {
        format!(
            "cannot prepare the storage root {}: {e}",
            args.root.display()
        )
    })?;
    let mut root = Root::open(&args.root, zone())
        .map_err(|e| format!("cannot serve {}: {e}", args.root.display()))?;
    // `--import-dir` is the only directory outside the root this daemon will
    // read from, and only one bare file name at a time (IMPORT). Not required:
    // a machine whose browser never downloads nothing into it needs no staging.
    root.set_import_dir(args.import_dir.clone());
    match &args.mode {
        Mode::Stdio => {
            // The file descriptor systemd handed over IS the connection; there
            // is nobody to accept from and nothing to listen on.
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin.lock());
            let mut writer = io::stdout();
            serve_stream(&mut reader, &mut writer, root).map_err(|e| format!("connection: {e}"))
        }
        Mode::Socket(path) => {
            eprintln!(
                "litric-filesd {VERSION}: serving {} on {}",
                args.root.display(),
                path.display()
            );
            serve_unix(path, root).map_err(|e| format!("socket {}: {e}", path.display()))
        }
        Mode::Info => info(&args.root, args.import_dir.as_deref()),
    }
}

/// There is no file service on a host without Unix semantics: the confinement
/// rules depend on symlink and permission behaviour this daemon has only been
/// written against. The workspace builds there so it can be tested elsewhere;
/// the daemon says so instead of pretending.
///
/// # Errors
/// Always.
#[cfg(not(unix))]
fn serve(args: &Args) -> Result<(), String> {
    Err(format!(
        "litric-filesd serves file operations on Unix hosts only; \
         this build cannot serve {}",
        args.root.display()
    ))
}

/// Print what this daemon would serve.
fn info(root: &Path, import_dir: Option<&Path>) -> Result<(), String> {
    println!("version={VERSION}");
    println!("root={}", root.display());
    println!("socket={DEFAULT_SOCKET}");
    println!("protocol=1");
    match import_dir {
        Some(dir) => println!("import={}", dir.display()),
        None => println!("import=none"),
    }
    println!("zone={}", zone().name());
    for name in litric_files::STANDARD_DIRS {
        let present = root.join(name).is_dir();
        println!("dir={name} present={}", u8::from(present));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn the_unit_invocation_parses() {
        let parsed = parse_args(&args(&["--stdio"]))
            .expect("valid")
            .expect("a mode");
        assert_eq!(parsed.mode, Mode::Stdio);
        assert_eq!(parsed.root, PathBuf::from(DEFAULT_ROOT));
    }

    #[test]
    fn a_root_can_be_named_for_a_fixture() {
        let parsed = parse_args(&args(&["--stdio", "--root", "/tmp/x"]))
            .expect("valid")
            .expect("a mode");
        assert_eq!(parsed.root, PathBuf::from("/tmp/x"));
    }

    #[test]
    fn a_socket_run_takes_its_path() {
        let parsed = parse_args(&args(&["--socket", "/run/litric/files.sock"]))
            .expect("valid")
            .expect("a mode");
        assert_eq!(
            parsed.mode,
            Mode::Socket(PathBuf::from("/run/litric/files.sock"))
        );
    }

    #[test]
    fn a_command_line_with_no_mode_is_refused() {
        let why = parse_args(&args(&["--root", "/tmp/x"])).expect_err("refused");
        assert!(why.contains("--stdio"), "got: {why}");
    }

    #[test]
    fn two_modes_at_once_are_refused() {
        let why = parse_args(&args(&["--stdio", "--info"])).expect_err("refused");
        assert!(why.contains("more than once"), "got: {why}");
    }

    #[test]
    fn a_flag_missing_its_value_is_refused() {
        assert!(parse_args(&args(&["--socket"])).is_err());
        assert!(parse_args(&args(&["--root"])).is_err());
    }

    #[test]
    fn help_and_version_exit_cleanly() {
        assert_eq!(parse_args(&args(&["--help"])), Ok(None));
        assert_eq!(parse_args(&args(&["-V"])), Ok(None));
    }

    #[test]
    fn an_unknown_argument_is_refused_rather_than_ignored() {
        let why = parse_args(&args(&["--stdio", "--fast"])).expect_err("refused");
        assert!(why.contains("--fast"), "got: {why}");
    }
}
