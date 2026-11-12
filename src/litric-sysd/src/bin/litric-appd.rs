//! `litric-appd` — the LitricOS application runtime daemon.
//!
//! # Why this process exists
//!
//! The capability broker, `litric-sysd`, runs with no network
//! (`IPAddressDeny=any`), no home directory (`ProtectHome=yes`), no user
//! namespaces (`RestrictNamespaces=yes`) and a read-only `/var`. Every one of
//! those is correct for the process that decides *whether* something may
//! happen, and every one of them makes it the wrong process to *do* it: a
//! `flatpak install` cannot reach Flathub and `flatpak run` cannot create its
//! sandbox. This daemon is the executor half. It reaches the network, it starts
//! applications, and it decides nothing — the broker forwards only requests
//! that already passed the capability check.
//!
//! # Modes
//!
//! ```text
//!   litric-appd                     bind the socket and answer requests
//!   litric-appd --serve [--socket PATH]
//!   litric-appd --query "STORE LIST"
//! ```
//!
//! The readiness lines are printed on the console as well as the journal, for
//! the same reason `litric-sysd` prints its own: on a headless proof VM a
//! daemon that started silently cannot be told from one that never started.

#![forbid(unsafe_code)]

#[cfg(unix)]
use litric_sysd::server;

use litric_sysd::appd;
#[cfg(unix)]
use litric_sysd::{launch, state};

#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

/// The version reported on the console and over the protocol.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the socket lives by default.
const DEFAULT_SOCKET: &str = appd::DEFAULT_SOCKET;

/// The Flatpak client whose presence is reported at start.
///
/// The store resolves `flatpak` through `PATH`; this is only the readiness
/// measurement, and it names the image path a person can check.
const FLATPAK: &str = "/usr/bin/flatpak";

#[derive(Debug)]
enum Mode {
    Serve,
    Query(String),
    Help,
    Version,
}

#[derive(Debug)]
struct Args {
    mode: Mode,
    socket: PathBuf,
    home: String,
    root: PathBuf,
}

fn parse_args<I: Iterator<Item = String>>(mut it: I) -> Result<Args, String> {
    let mut args = Args {
        mode: Mode::Serve,
        socket: PathBuf::from(DEFAULT_SOCKET),
        // The unit sets HOME to a persistent directory; this fallback keeps a
        // manual start from inheriting a home that does not exist in the image.
        home: std::env::var("HOME").unwrap_or_else(|_| "/var/lib/litric-app".to_owned()),
        root: PathBuf::from("/"),
    };
    let mut saw_mode = false;
    while let Some(a) = it.next() {
        let mut need = |what: &str| -> Result<String, String> {
            it.next().ok_or_else(|| format!("{what} needs a value"))
        };
        match a.as_str() {
            "--serve" => {
                args.mode = Mode::Serve;
                saw_mode = true;
            }
            "--query" => {
                args.mode = Mode::Query(need("--query")?);
                saw_mode = true;
            }
            "--socket" => args.socket = PathBuf::from(need("--socket")?),
            "--home" => args.home = need("--home")?,
            "--root" => args.root = PathBuf::from(need("--root")?),
            "--help" | "-h" => {
                return Ok(Args {
                    mode: Mode::Help,
                    ..args
                })
            }
            "--version" | "-V" => {
                return Ok(Args {
                    mode: Mode::Version,
                    ..args
                })
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if !saw_mode {
        args.mode = Mode::Serve;
    }
    Ok(args)
}

const USAGE: &str = "\
litric-appd — the LitricOS application runtime daemon

USAGE:
    litric-appd [--serve] [--socket PATH] [--home DIR] [--root DIR]
    litric-appd --query REQUEST [--socket PATH]

OPTIONS:
    --serve           Bind the socket and answer requests (the default).
    --query REQUEST   Send one request line to a running daemon.
    --socket PATH     Socket path (default /run/litric/appd.sock).
    --home DIR        The directory the store and launcher treat as home.
    --root DIR        Where /proc and /sys are rooted. For testing.
    -h, --help        This text.
    -V, --version     Version.
";

#[cfg(unix)]
fn run_server(args: &Args) -> ExitCode {
    let state = state::Sysd::from_system(&args.home, &args.root);
    let listener = match server::bind(&args.socket) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!(
                "litric-appd: cannot bind {}: {error}",
                args.socket.display()
            );
            return ExitCode::FAILURE;
        }
    };
    println!("litric.appd.version={VERSION}");
    println!("litric.appd.pid={}", std::process::id());
    println!("litric.appd.socket={}", args.socket.display());
    // The home this daemon hands to `flatpak run`. It is the unit's
    // StateDirectory, and a proof that reads `litric.appd.home=` can tell a
    // persistent application data directory from a path that does not exist.
    println!("litric.appd.home={}", args.home);
    // A measured fact, not an assumption: a store with no client says so at
    // start rather than on the first query.
    println!(
        "litric.appd.flatpak={}",
        if launch::is_executable(Path::new(FLATPAK)) {
            "present"
        } else {
            "missing"
        }
    );
    println!("litric.appd.ready=1");
    if let Err(error) = appd::serve(&listener, appd::Executor::new(state)) {
        eprintln!("litric-appd: serve failed: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(not(unix))]
fn run_server(_args: &Args) -> ExitCode {
    eprintln!("litric-appd: needs a Unix host; this build has no socket transport");
    ExitCode::FAILURE
}

#[cfg(unix)]
fn run_query(args: &Args, request: &str) -> ExitCode {
    match server::query(&args.socket, request) {
        Ok(reply) => {
            print!("{reply}");
            if reply.starts_with("ERR ") {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("litric-appd: {}: {error}", args.socket.display());
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(unix))]
fn run_query(_args: &Args, _request: &str) -> ExitCode {
    eprintln!("litric-appd: --query needs a Unix host");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("litric-appd: {error}");
            eprintln!();
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match &args.mode {
        Mode::Help => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Mode::Version => {
            println!("litric-appd {VERSION}");
            ExitCode::SUCCESS
        }
        Mode::Serve => run_server(&args),
        Mode::Query(request) => run_query(&args, request),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Result<Args, String> {
        parse_args(v.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn no_arguments_means_serve_on_the_shared_runtime_directory() {
        let parsed = args(&[]).expect("parses");
        assert!(matches!(parsed.mode, Mode::Serve));
        assert_eq!(parsed.socket, PathBuf::from("/run/litric/appd.sock"));
    }

    #[test]
    fn options_and_modes_parse() {
        let parsed = args(&["--serve", "--socket", "/tmp/a.sock", "--home", "/var/lib/x"])
            .expect("parses");
        assert_eq!(parsed.socket, PathBuf::from("/tmp/a.sock"));
        assert_eq!(parsed.home, "/var/lib/x");
        assert!(matches!(
            args(&["--query", "STORE LIST"]).expect("parses").mode,
            Mode::Query(ref r) if r == "STORE LIST"
        ));
        assert!(args(&["--socket"]).is_err());
        assert!(args(&["--danger"]).is_err());
    }
}
