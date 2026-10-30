//! `litric-sysd` — the LitricOS system daemon.
//!
//! # Layer position
//!
//! **Layer 4 (Runtime).** The first LitricOS binary that runs *inside* the
//! operating system rather than beside it. Everything below it — the capability
//! engine, the storage broker, the trash policy, the environment manager — was
//! already real code with passing tests and no way to run. This is what runs
//! it.
//!
//! Nothing above Layer 4 can be honest until this exists: a shell that draws a
//! trash icon with no trash behind it, or an environment switcher with no
//! environment manager behind it, is a picture of an operating system.
//!
//! # Modes
//!
//! ```text
//!   litric-sysd --report            measure the machine, print litric.* lines
//!   litric-sysd --boot              count this boot against a staged candidate
//!   litric-sysd --serve             bind the socket and answer requests
//!   litric-sysd --query "ENV LIST"  ask a running daemon one question
//! ```
//!
//! `--report` is what the boot unit runs, because it produces evidence on the
//! serial console that the daemon started, measured something real, and said
//! what it found. `tools/boot-test.sh` greps for those lines; a daemon that
//! started and did nothing observable is indistinguishable from one that never
//! started.

#![forbid(unsafe_code)]

use litric_sysd::state;

#[cfg(unix)]
use litric_sysd::server;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The version reported on the console and over the protocol.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the socket lives by default.
///
/// Under `/run` because it is a runtime object that must not survive a reboot,
/// and because `/` is a read-only dm-verity root that could not hold it anyway.
const DEFAULT_SOCKET: &str = "/run/litric/sysd.sock";

/// Where the durable capability record lives by default.
///
/// Under `/var` because it must survive a reboot — that is the whole point of
/// the file — and because the read-only dm-verity root could not hold it.
const DEFAULT_STATE: &str = "/var/lib/litric-sysd/grants.conf";

/// Where the update generation record and incoming trees live by default.
///
/// Under `/var` because the A/B pointer must survive a reboot. The systemd unit
/// creates this path with tmpfiles and opens exactly this directory for writing;
/// the daemon does not invent a second location.
const DEFAULT_UPDATE_ROOT: &str = litric_sysd::update::DEFAULT_ROOT;

/// What the command line asked for.
#[derive(Debug)]
enum Mode {
    /// Print measured facts and exit.
    Report,
    /// Count one boot against a staged candidate and exit.
    Boot,
    /// Run the socket server.
    Serve,
    /// Send one request to a running daemon.
    Query(String),
    /// Print usage.
    Help,
    /// Print the version.
    Version,
}

#[derive(Debug)]
struct Args {
    mode: Mode,
    socket: PathBuf,
    home: String,
    root: PathBuf,
    state: PathBuf,
    update_root: PathBuf,
}

fn parse_args<I: Iterator<Item = String>>(mut it: I) -> Result<Args, String> {
    let mut args = Args {
        mode: Mode::Report,
        socket: PathBuf::from(DEFAULT_SOCKET),
        home: std::env::var("HOME").unwrap_or_else(|_| "/root".to_owned()),
        root: PathBuf::from("/"),
        state: PathBuf::from(DEFAULT_STATE),
        update_root: PathBuf::from(DEFAULT_UPDATE_ROOT),
    };
    let mut saw_mode = false;
    while let Some(a) = it.next() {
        let mut need = |what: &str| -> Result<String, String> {
            it.next().ok_or_else(|| format!("{what} needs a value"))
        };
        match a.as_str() {
            "--report" => {
                args.mode = Mode::Report;
                saw_mode = true;
            }
            "--boot" => {
                args.mode = Mode::Boot;
                saw_mode = true;
            }
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
            "--state" => args.state = PathBuf::from(need("--state")?),
            "--update-root" => args.update_root = PathBuf::from(need("--update-root")?),
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
        args.mode = Mode::Report;
    }
    Ok(args)
}

const USAGE: &str = "\
litric-sysd — the LitricOS system daemon

USAGE:
    litric-sysd [--report] [--root DIR]
    litric-sysd --boot [--update-root DIR]
    litric-sysd --serve [--socket PATH] [--home DIR] [--root DIR] [--state PATH]
                       [--update-root DIR]
    litric-sysd --query REQUEST [--socket PATH]

OPTIONS:
    --report          Measure the machine and print litric.* lines (default).
    --boot            Count one boot against a staged update candidate, then
                      exit. Run by the unit's ExecStartPre before --serve.
    --serve           Bind the socket and answer requests.
    --query REQUEST   Send one request line to a running daemon.
    --socket PATH     Socket path (default /run/litric/sysd.sock).
    --home DIR        The home directory the storage layout is derived from.
    --root DIR        Where /proc and /sys are rooted. For testing.
    --state PATH      The durable capability record. Default
                      /var/lib/litric-sysd/grants.conf.
    --update-root DIR The generation record and incoming trees. Default
                      /var/lib/litric/update.
    -h, --help        This text.
    -V, --version     Version.
";

fn report(root: &Path, home: &str) -> ExitCode {
    let s = state::Sysd::from_system(home, root);
    println!("litric.sysd.version={VERSION}");
    let lines = s.facts().to_lines();
    if lines.is_empty() {
        // Never print zeroes that would read as measurements.
        println!("litric.sysd.facts=unavailable");
    } else {
        for l in &lines {
            println!("{l}");
        }
    }
    println!("litric.sysd.environments={}", s.environments().all().len());
    println!("litric.sysd.active={}", s.environments().active().token());
    println!("litric.sysd.ready=1");
    ExitCode::SUCCESS
}

#[cfg(unix)]
fn run_server(args: &Args) -> ExitCode {
    let state = state::Sysd::from_system(&args.home, &args.root)
        .with_grant_store(&args.state)
        .with_update_root(&args.update_root);
    let listener = match server::bind(&args.socket) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("litric-sysd: cannot bind {}: {e}", args.socket.display());
            return ExitCode::FAILURE;
        }
    };
    println!("litric.sysd.version={VERSION}");
    println!("litric.sysd.socket={}", args.socket.display());
    println!(
        "litric.sysd.grants={} store={} records={}",
        state.grants_note(),
        args.state.display(),
        state.grant_count()
    );
    // The measurements go to the console as well as the journal. A daemon that
    // starts silently is indistinguishable, from outside a headless VM, from
    // one that did not start; these lines are what tools/boot-test.sh greps for.
    for l in &state.facts().to_lines() {
        println!("{l}");
    }
    println!("litric.sysd.ready=1");
    if let Err(e) = server::serve(&listener, state) {
        eprintln!("litric-sysd: serve failed: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(not(unix))]
fn run_server(_args: &Args) -> ExitCode {
    eprintln!("litric-sysd: --serve needs a Unix host; this build has no socket transport");
    ExitCode::FAILURE
}

fn run_boot(args: &Args) -> ExitCode {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    for line in litric_sysd::update::boot(&args.update_root, now) {
        println!("{line}");
    }
    ExitCode::SUCCESS
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
        Err(e) => {
            eprintln!("litric-sysd: {}: {e}", args.socket.display());
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(unix))]
fn run_query(_args: &Args, _request: &str) -> ExitCode {
    eprintln!("litric-sysd: --query needs a Unix host; this build has no socket transport");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("litric-sysd: {e}");
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
            println!("litric-sysd {VERSION}");
            ExitCode::SUCCESS
        }
        Mode::Report => report(&args.root, &args.home),
        Mode::Boot => run_boot(&args),
        Mode::Serve => run_server(&args),
        Mode::Query(r) => run_query(&args, r),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Result<Args, String> {
        parse_args(v.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn no_arguments_means_report() {
        let a = args(&[]).expect("parses");
        assert!(matches!(a.mode, Mode::Report));
        assert_eq!(a.socket, PathBuf::from(DEFAULT_SOCKET));
    }

    #[test]
    fn modes_parse() {
        assert!(matches!(
            args(&["--serve"]).expect("parses").mode,
            Mode::Serve
        ));
        assert!(matches!(
            args(&["--boot"]).expect("parses").mode,
            Mode::Boot
        ));
        assert!(matches!(
            args(&["--query", "PING"]).expect("parses").mode,
            Mode::Query(ref r) if r == "PING"
        ));
        assert!(matches!(
            args(&["--help"]).expect("parses").mode,
            Mode::Help
        ));
    }

    #[test]
    fn options_parse() {
        let a =
            args(&["--serve", "--socket", "/tmp/s.sock", "--home", "/home/bob"]).expect("parses");
        assert_eq!(a.socket, PathBuf::from("/tmp/s.sock"));
        assert_eq!(a.home, "/home/bob");

        let b = args(&["--boot", "--update-root", "/tmp/updates"]).expect("parses");
        assert_eq!(b.update_root, PathBuf::from("/tmp/updates"));
    }

    #[test]
    fn an_option_without_its_value_is_an_error_not_a_default() {
        assert!(args(&["--socket"]).is_err());
        assert!(args(&["--query"]).is_err());
    }

    #[test]
    fn an_unknown_argument_is_refused_rather_than_ignored() {
        let e = args(&["--danger"]).expect_err("must fail");
        assert!(e.contains("--danger"), "{e}");
    }
}
