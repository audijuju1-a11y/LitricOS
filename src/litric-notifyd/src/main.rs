//! `litric-notifyd` — the LitricOS notification queue.
//!
//! # What it is for
//!
//! A notification raised while the desktop is not running is not lost. A program
//! posts here, the queue is written to disk, and the shell takes it when it
//! starts. That is the whole promise, and it is the difference between a
//! notification centre and a list in the compositor's memory.
//!
//! # Modes
//!
//! ```text
//!   litric-notifyd --report                      what the queue holds, for a boot log
//!   litric-notifyd --serve                       bind the socket and answer requests
//!   litric-notifyd --post --app A --title T      queue a notification from a script
//!   litric-notifyd --take                        take the queue (what the shell does)
//!   litric-notifyd --list                        read it without removing anything
//!   litric-notifyd --count                       how many are queued
//!   litric-notifyd --query "INFO"                send one request verbatim
//! ```
//!
//! `--post` exists because a queue nothing can write to is a queue nothing can
//! test. Without it the only way to see a notification survive a shell restart
//! would be to write a C program against the socket, which is why the boot test
//! could not have exercised this at all.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use litric_notifyd::proto::Priority;
use litric_notifyd::queue::{Queue, Store};

#[cfg(unix)]
use litric_notifyd::{client, server};

/// The version reported on the console and over the protocol.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the socket lives by default.
///
/// Under `/run` because it is a runtime object that must not survive a reboot,
/// and because `/` is a read-only dm-verity root that could not hold it anyway.
const DEFAULT_SOCKET: &str = "/run/litric/notify.sock";

/// Where the queue is written by default.
///
/// Under `/var` because that is the part of this system whose whole purpose is to
/// outlive the process, and in the unit's own `StateDirectory` so the sandbox
/// opens exactly one directory for writing.
const DEFAULT_STATE: &str = "/var/lib/litric-notifyd/queue.conf";

/// What the command line asked for.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// Print what is queued and exit.
    Report,
    /// Run the socket server.
    Serve,
    /// Queue a notification.
    Post,
    /// Take the queue and print it.
    Take,
    /// Read the queue and print it.
    List,
    /// Print the number queued.
    Count,
    /// Send one request verbatim.
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
    state: PathBuf,
    app: String,
    title: Option<String>,
    body: String,
    priority: Priority,
    ttl_ms: Option<u64>,
}

fn parse_args<I: Iterator<Item = String>>(mut it: I) -> Result<Args, String> {
    let mut args = Args {
        mode: Mode::Report,
        socket: PathBuf::from(DEFAULT_SOCKET),
        state: PathBuf::from(DEFAULT_STATE),
        app: String::new(),
        title: None,
        body: String::new(),
        priority: Priority::Normal,
        ttl_ms: None,
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
            "--serve" => {
                args.mode = Mode::Serve;
                saw_mode = true;
            }
            "--post" => {
                args.mode = Mode::Post;
                saw_mode = true;
            }
            "--take" => {
                args.mode = Mode::Take;
                saw_mode = true;
            }
            "--list" => {
                args.mode = Mode::List;
                saw_mode = true;
            }
            "--count" => {
                args.mode = Mode::Count;
                saw_mode = true;
            }
            "--query" => {
                args.mode = Mode::Query(need("--query")?);
                saw_mode = true;
            }
            "--socket" => args.socket = PathBuf::from(need("--socket")?),
            "--state" => args.state = PathBuf::from(need("--state")?),
            "--app" => args.app = need("--app")?,
            "--title" => args.title = Some(need("--title")?),
            "--body" => args.body = need("--body")?,
            "--priority" => {
                let raw = need("--priority")?;
                args.priority = Priority::parse(&raw)
                    .ok_or_else(|| format!("--priority: {raw:?} is not a level"))?;
            }
            "--ttl-ms" => {
                let raw = need("--ttl-ms")?;
                args.ttl_ms = Some(
                    raw.parse::<u64>()
                        .map_err(|_| format!("--ttl-ms: {raw:?} is not a number"))?,
                );
            }
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
    // A post with no title is refused here rather than over the wire, so the
    // message names the option the caller forgot instead of the protocol field.
    if args.mode == Mode::Post {
        if args.title.as_deref().unwrap_or("").is_empty() {
            return Err("--post needs --title".to_owned());
        }
        if args.app.is_empty() {
            return Err("--post needs --app".to_owned());
        }
    }
    Ok(args)
}

const USAGE: &str = "\
litric-notifyd — the LitricOS notification queue

USAGE:
    litric-notifyd [--report] [--state PATH]
    litric-notifyd --serve [--socket PATH] [--state PATH]
    litric-notifyd --post --app APP --title TITLE [--body TEXT] [--priority P] [--ttl-ms N]
    litric-notifyd --take | --list | --count
    litric-notifyd --query REQUEST [--socket PATH]

OPTIONS:
    --report          Print what is queued and exit (default).
    --serve           Bind the socket and answer requests.
    --post            Queue a notification; needs --app and --title.
    --take            Take the queue (what the desktop does at startup).
    --list            Read the queue without removing anything.
    --count           Print how many are queued.
    --query REQUEST   Send one request line to a running daemon.
    --socket PATH     Socket path (default /run/litric/notify.sock).
    --state PATH      Queue file (default /var/lib/litric-notifyd/queue.conf).
    --priority P      low | normal | high | critical (default normal).
    --ttl-ms N        How long the notification is worth showing, in ms.
    --body TEXT       The detail line.
    -h, --help        This text.
    -V, --version     Version.
";

fn report(state: &Path) -> ExitCode {
    let store = Store::new(state);
    println!("litric.notifyd.version={VERSION}");
    println!("litric.notifyd.state={}", state.display());
    match store.load() {
        Ok((queue, skipped)) => {
            println!("litric.notifyd.queued={}", queue.count());
            println!("litric.notifyd.capacity={}", Queue::CAPACITY);
            if skipped > 0 {
                // Never silently: a record that could not be read is a
                // notification somebody sent and this system lost.
                println!("litric.notifyd.skipped={skipped}");
            }
        }
        Err(error) => {
            println!("litric.notifyd.queued=unreadable");
            eprintln!("litric-notifyd: {error}");
        }
    }
    println!("litric.notifyd.ready=1");
    ExitCode::SUCCESS
}

#[cfg(unix)]
fn run_server(args: &Args) -> ExitCode {
    let store = Store::new(&args.state);
    let state = match server::State::load(store, now_ms_fallback()) {
        Ok(state) => state,
        Err(error) => {
            // Refusing to start is the honest answer: starting with an empty
            // queue would look healthy while every queued notification is
            // unreachable on disk.
            eprintln!("litric-notifyd: {error}");
            return ExitCode::FAILURE;
        }
    };
    let listener = match server::bind(&args.socket) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!(
                "litric-notifyd: cannot bind {}: {error}",
                args.socket.display()
            );
            return ExitCode::FAILURE;
        }
    };
    println!("litric.notifyd.version={VERSION}");
    println!("litric.notifyd.socket={}", args.socket.display());
    println!("litric.notifyd.queued={}", state.queue().count());
    // On the console as well as the journal, so a headless boot shows that the
    // queue started and what it recovered.
    println!("litric.notifyd.ready=1");
    if let Err(error) = server::serve(&listener, state) {
        eprintln!("litric-notifyd: serve failed: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(not(unix))]
fn run_server(_args: &Args) -> ExitCode {
    eprintln!("litric-notifyd: --serve needs a Unix host; this build has no socket transport");
    ExitCode::FAILURE
}

/// The clock, for the one place the binary needs it before a request exists.
fn now_ms_fallback() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(unix)]
fn run_client(args: &Args) -> ExitCode {
    match &args.mode {
        Mode::Post => {
            let title = args.title.clone().unwrap_or_default();
            match client::post_text(
                &args.socket,
                &args.app,
                &title,
                &args.body,
                args.priority,
                args.ttl_ms,
            ) {
                Ok(ack) => {
                    println!("id={}", ack.id);
                    if !ack.persisted {
                        // The notification is queued and will be shown; what
                        // failed is the promise that it survives a reboot.
                        eprintln!(
                            "litric-notifyd: queued but NOT persisted; it will not survive a restart"
                        );
                        return ExitCode::FAILURE;
                    }
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("litric-notifyd: {}", args.socket.display());
                    eprintln!("litric-notifyd: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Mode::Take | Mode::List => {
            let result = if args.mode == Mode::Take {
                client::take(&args.socket).map(|taken| (taken.notes, taken.persisted))
            } else {
                client::list(&args.socket).map(|notes| (notes, true))
            };
            match result {
                Ok((notes, persisted)) => {
                    for note in &notes {
                        println!("NOTE {}", note.id);
                        for line in litric_notifyd::proto::note_lines(note) {
                            println!("{line}");
                        }
                        println!("END");
                    }
                    println!("count={}", notes.len());
                    if !persisted {
                        eprintln!("litric-notifyd: the emptied queue did not reach the disk");
                        return ExitCode::FAILURE;
                    }
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("litric-notifyd: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Mode::Count => match client::count(&args.socket) {
            Ok(count) => {
                println!("{count}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("litric-notifyd: {error}");
                ExitCode::FAILURE
            }
        },
        Mode::Query(request) => match client::request(&args.socket, request) {
            Ok(reply) => {
                print!("{reply}");
                if reply.starts_with("ERR ") {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(error) => {
                eprintln!("litric-notifyd: {error}");
                ExitCode::FAILURE
            }
        },
        _ => ExitCode::SUCCESS,
    }
}

#[cfg(not(unix))]
fn run_client(_args: &Args) -> ExitCode {
    eprintln!("litric-notifyd: this build has no socket transport, so it cannot queue anything");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("litric-notifyd: {error}");
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
            println!("litric-notifyd {VERSION}");
            ExitCode::SUCCESS
        }
        Mode::Report => report(&args.state),
        Mode::Serve => run_server(&args),
        _ => run_client(&args),
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
        assert_eq!(a.mode, Mode::Report);
        assert_eq!(a.socket, PathBuf::from(DEFAULT_SOCKET));
        assert_eq!(a.state, PathBuf::from(DEFAULT_STATE));
    }

    #[test]
    fn every_mode_parses() {
        assert_eq!(args(&["--serve"]).expect("parses").mode, Mode::Serve);
        assert_eq!(args(&["--count"]).expect("parses").mode, Mode::Count);
        assert_eq!(args(&["--take"]).expect("parses").mode, Mode::Take);
        assert_eq!(args(&["--list"]).expect("parses").mode, Mode::List);
        assert_eq!(args(&["--help"]).expect("parses").mode, Mode::Help);
        assert_eq!(args(&["--version"]).expect("parses").mode, Mode::Version);
        assert_eq!(
            args(&["--query", "INFO"]).expect("parses").mode,
            Mode::Query("INFO".to_owned())
        );
    }

    #[test]
    fn a_post_that_is_missing_its_title_is_refused_before_it_reaches_the_wire() {
        // The error names the option the caller forgot.
        let error = args(&["--post", "--app", "com.litric.test"]).expect_err("must fail");
        assert!(error.contains("--title"), "{error}");
        let error = args(&["--post", "--title", "x"]).expect_err("must fail");
        assert!(error.contains("--app"), "{error}");
    }

    #[test]
    fn a_post_parses_its_optional_fields() {
        let a = args(&[
            "--post",
            "--app",
            "com.litric.browser",
            "--title",
            "Download finished",
            "--body",
            "report.pdf",
            "--priority",
            "high",
            "--ttl-ms",
            "8000",
        ])
        .expect("parses");
        assert_eq!(a.mode, Mode::Post);
        assert_eq!(a.app, "com.litric.browser");
        assert_eq!(a.priority, Priority::High);
        assert_eq!(a.ttl_ms, Some(8000));
        assert_eq!(a.body, "report.pdf");
    }

    #[test]
    fn an_unreadable_priority_or_ttl_is_refused_rather_than_defaulted() {
        let error = args(&["--priority", "urgent"]).expect_err("must fail");
        assert!(error.contains("urgent"), "{error}");
        let error = args(&["--ttl-ms", "5s"]).expect_err("must fail");
        assert!(error.contains("5s"), "{error}");
    }

    #[test]
    fn an_option_without_its_value_is_an_error_not_a_default() {
        for flag in ["--socket", "--state", "--query", "--title", "--app"] {
            assert!(args(&[flag]).is_err(), "{flag}");
        }
    }

    #[test]
    fn an_unknown_argument_is_refused_rather_than_ignored() {
        let error = args(&["--danger"]).expect_err("must fail");
        assert!(error.contains("--danger"), "{error}");
    }
}
