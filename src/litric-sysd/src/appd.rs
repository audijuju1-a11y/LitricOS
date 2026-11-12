//! The application runtime daemon: the half of the application path that runs
//! with the session's network, home and namespaces.
//!
//! `litric-sysd` is the capability broker, and its unit is deliberately unable
//! to reach the network (`IPAddressDeny=any`), read a home directory
//! (`ProtectHome=yes`), create a user namespace (`RestrictNamespaces=yes`) or
//! live in a writable `/var/lib/flatpak`. A process it forks inherits all of
//! that, which is why every application launch and every `flatpak install`
//! failed before this module existed. A broker and an execution runtime cannot
//! be the same process.
//!
//! This module is the second half. `litric-appd` is a separate unit whose whole
//! purpose is to run the two things the broker may not:
//!
//! * the Flatpak catalog and installation (`list`, `search`, `updates`,
//!   `install`, `uninstall`, and the sandbox override writes);
//! * starting an application, through the same [`crate::compat`] resolver the
//!   broker used to call in-process.
//!
//! # It decides nothing
//!
//! `litric-appd` has no capability broker and no client identity. It accepts a
//! closed set of commands — the four catalog reads, the store writes, and one
//! `RUN` line — and it re-validates every argument with the same functions the
//! broker used (`store::valid_app_id`, `compat::resolve`, `launch::validate`).
//! The decision of *whether* a caller may do this was already made by
//! `litric-sysd`, against its durable grant record; a forwarding path that
//! arrived here has passed that gate. Its socket is `0600` inside the `0700`
//! `/run/litric`, so only root — in this image, `litric-sysd` — can speak to it.
//!
//! # Wire format
//!
//! The store commands are the protocol's own (`STORE LIST`, `STORE SEARCH …`,
//! …), parsed by [`crate::proto::Request`]. The launch command is `RUN`, and
//! every field in it is hex, because a target path and its arguments are
//! strings a desktop entry wrote and the wire splits on whitespace:
//!
//! ```text
//! RUN <runtime> <hex-target> <hex-arg>… [ENV <hex-name> <hex-value>…]
//! ```
//!
//! `ENV` cannot be confused with a hex argument: hex is `[0-9a-f]` only.

use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

use crate::compat::{LaunchRequest, Runtime};
use crate::launch::hex_encode;
use crate::proto::{ProtoError, Request, Response, MAX_LINE};
use crate::providers::{unhex, SystemAction};
use crate::sandbox::SandboxAction;
use crate::state::Sysd;
use crate::store::StoreAction;

/// Where the runtime daemon listens by default.
///
/// In the same `0700` directory as the broker's socket: one runtime directory,
/// one owner, and no new path for a permission mistake to appear in.
pub const DEFAULT_SOCKET: &str = "/run/litric/appd.sock";

/// How long a catalog read may take before the broker gives up on the daemon.
///
/// Slightly longer than the store's own 45-second read deadline, so a slow tool
/// is reported by the tool's timeout rather than as an unreachable daemon.
pub const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// How long a store write may take. The install plan allows 900 seconds, and a
/// Forwarded reply that never arrives must not hold the broker's request lock
/// longer than the work itself can legitimately take.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(960);

/// How long starting an application may take. `execve` is immediate; this only
/// bounds a daemon that has stopped answering.
pub const RUN_TIMEOUT: Duration = Duration::from_secs(30);

/// The most payload lines one reply may declare.
///
/// A corrupt or hostile header must not make the reader allocate an arbitrary
/// vector before it has read anything. The store's own cap is 24 rows; 512
/// lines leaves room for the widest legitimate reply and nothing more.
pub const MAX_REPLY_LINES: usize = 512;

/// An idle connection is closed rather than held forever.
#[cfg(unix)]
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

/// Seconds since the Unix epoch, or 0 before it.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// One command the runtime daemon accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Liveness, for a boot probe and for tests.
    Ping,
    /// Close the connection.
    Quit,
    /// One catalog read. Only the four read verbs parse into this variant.
    Store(Request),
    /// One store or sandbox write, already authorised by the broker.
    Write(SystemAction),
    /// Start one application through the compatibility resolver.
    Run(LaunchRequest),
}

/// Encode a launch as the one `RUN` line.
///
/// Free of whitespace in every argument by construction: every user-supplied
/// string is hex, so a path containing a space, a newline or `ENV` cannot
/// change how the line parses.
#[must_use]
pub fn launch_line(request: &LaunchRequest) -> String {
    let mut out = String::from("RUN ");
    out.push_str(request.runtime.token());
    out.push(' ');
    out.push_str(&hex_encode(request.target.as_bytes()));
    for arg in &request.args {
        out.push(' ');
        out.push_str(&hex_encode(arg.as_bytes()));
    }
    if !request.env.is_empty() {
        out.push_str(" ENV");
        for (name, value) in &request.env {
            out.push(' ');
            out.push_str(&hex_encode(name.as_bytes()));
            out.push(' ');
            out.push_str(&hex_encode(value.as_bytes()));
        }
    }
    out
}

/// Encode an already-authorised write as the protocol line the daemon parses.
///
/// `None` when the action is not one this daemon executes, or when an argument
/// is not one the protocol could carry as a single field. The caller then takes
/// its unchanged local path, which refuses the same argument with the same
/// words — forwarding must never be a way to smuggle a value past validation.
#[must_use]
pub fn action_line(action: &SystemAction) -> Option<String> {
    match action {
        SystemAction::Store(StoreAction::Install { id, remote })
            if crate::store::valid_app_id(id) && crate::store::valid_remote(remote) =>
        {
            Some(format!("STORE INSTALL {id} {remote} confirm"))
        }
        SystemAction::Store(StoreAction::Uninstall { id }) if crate::store::valid_app_id(id) => {
            Some(format!("STORE UNINSTALL {id} confirm"))
        }
        SystemAction::Sandbox(SandboxAction::Update { id: None }) => {
            Some("STORE UPDATE all confirm".to_owned())
        }
        SystemAction::Sandbox(SandboxAction::Update { id: Some(id) })
            if crate::store::valid_app_id(id) =>
        {
            Some(format!("STORE UPDATE {id} confirm"))
        }
        SystemAction::Sandbox(SandboxAction::Set { id, key, on })
            if crate::store::valid_app_id(id) =>
        {
            Some(format!(
                "STORE SANDBOX SET {id} {} {} confirm",
                key.token(),
                if *on { "on" } else { "off" }
            ))
        }
        SystemAction::Sandbox(SandboxAction::Reset { id }) if crate::store::valid_app_id(id) => {
            Some(format!("STORE SANDBOX RESET {id} confirm"))
        }
        _ => None,
    }
}

/// Parse one command line.
///
/// # Errors
/// [`ProtoError`] naming what is wrong. The variants are the protocol's own so
/// a client cannot tell a refusal here from one at the broker.
pub fn parse(line: &str) -> Result<Command, ProtoError> {
    if line.len() > MAX_LINE {
        return Err(ProtoError::TooLong { len: line.len() });
    }
    let trimmed = line.trim();
    let mut words = trimmed.split_whitespace();
    let Some(verb) = words.next() else {
        return Err(ProtoError::Empty);
    };
    if verb.eq_ignore_ascii_case("run") {
        let args: Vec<&str> = words.collect();
        return parse_run(&args).map(Command::Run);
    }
    match Request::parse(trimmed)? {
        Request::Ping => Ok(Command::Ping),
        Request::Quit => Ok(Command::Quit),
        request @ (Request::StoreList
        | Request::StoreSearch { .. }
        | Request::StoreUpdates
        | Request::StoreSandbox { .. }) => Ok(Command::Store(request)),
        Request::Action { action, .. }
            if matches!(action, SystemAction::Store(_) | SystemAction::Sandbox(_)) =>
        {
            Ok(Command::Write(action))
        }
        _ => Err(ProtoError::UnknownVerb {
            verb: verb.to_owned(),
        }),
    }
}

fn parse_run(args: &[&str]) -> Result<LaunchRequest, ProtoError> {
    let runtime_token = args.first().copied().unwrap_or_default();
    let Some(runtime) = Runtime::parse(runtime_token) else {
        return Err(ProtoError::BadArgument {
            name: "runtime",
            value: runtime_token.to_owned(),
        });
    };
    let target = hex_text(args.get(1).copied().unwrap_or_default(), "target")?;
    let mut argv = Vec::new();
    let mut index = 2usize;
    while let Some(word) = args.get(index) {
        if *word == "ENV" {
            break;
        }
        argv.push(hex_text(word, "argument")?);
        index = index.saturating_add(1);
    }
    let mut env = Vec::new();
    if args.get(index) == Some(&"ENV") {
        index = index.saturating_add(1);
        while let Some(name) = args.get(index) {
            let value = args.get(index.saturating_add(1)).ok_or(ProtoError::BadArity {
                verb: "RUN ENV",
                expected: 2,
                got: 1,
            })?;
            env.push((
                hex_text(name, "environment name")?,
                hex_text(value, "environment value")?,
            ));
            index = index.saturating_add(2);
        }
    }
    Ok(LaunchRequest {
        runtime,
        target,
        args: argv,
        env,
    })
}

fn hex_text(raw: &str, name: &'static str) -> Result<String, ProtoError> {
    let bytes = unhex(raw).ok_or(ProtoError::BadArgument {
        name,
        value: "<not hex>".to_owned(),
    })?;
    String::from_utf8(bytes).map_err(|_| ProtoError::BadArgument {
        name,
        value: "<not utf-8>".to_owned(),
    })
}

/// The word a command is logged under.
///
/// The broker keeps the audit of *decisions*; this is the other half, the
/// record of what was *executed*. The names are stable tokens so a boot proof
/// can count them, and they are coarse on purpose: which app was installed is
/// the broker's audit line, not this one.
impl Command {
    /// The stable token this command appears as in the journal.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Command::Ping => "ping",
            Command::Quit => "quit",
            Command::Store(_) => "store-read",
            Command::Write(_) => "store-write",
            Command::Run(_) => "run",
        }
    }
}

/// The journal line for one executed command.
///
/// One line per request, whatever its outcome: a daemon that logs only failures
/// cannot be told apart from one that is not being asked. The elapsed time is
/// included because "the install took 400 s" is the measurement a support
/// report needs, and the reply's line count because `OK 0` and `OK 12` are
/// different answers that both look like success in a status line.
#[must_use]
pub fn request_log(command: &Command, reply: &Response, elapsed: Duration) -> String {
    format!(
        "litric.appd.request={} result={} kind={} lines={} ms={}",
        command.name(),
        if reply.is_ok() { "ok" } else { "err" },
        if reply.kind().is_empty() {
            "-"
        } else {
            reply.kind()
        },
        reply.lines().len(),
        elapsed.as_millis()
    )
}

/// The executor half of the daemon.
///
/// It owns a [`Sysd`] only so that the catalog formatting and the compatibility
/// resolver are the same code the broker uses; none of the broker's decision
/// methods are reachable from here.
#[derive(Debug)]
pub struct Executor {
    state: Sysd,
}

impl Executor {
    /// Wrap a state with no transport.
    #[must_use]
    pub fn new(state: Sysd) -> Self {
        Self { state }
    }

    /// Execute one command and return the reply.
    ///
    /// No capability decision happens here, by design: see the module comment.
    pub fn execute(&mut self, command: &Command) -> Response {
        match command {
            Command::Ping => Response::ok(vec!["pong".to_owned()]),
            Command::Quit => Response::ok(Vec::new()),
            // The four read verbs are capability-free in the broker too: they
            // change nothing and `handle` answers them without consulting the
            // grant record.
            Command::Store(request) => self.state.handle(request, now()),
            Command::Write(action) => self.state.apply_authorised(action),
            Command::Run(request) => self.state.execute_launch_local(request),
        }
    }
}

/// Serve connections until the listener fails.
///
/// One request at a time, on purpose. The broker serialises its own requests
/// and is the only client; a second thread would add a queue without adding
/// throughput, and a single loop makes "the install that is running" the only
/// thing that can be running.
///
/// # Errors
/// Any `accept` error that is not recoverable.
#[cfg(unix)]
pub fn serve(listener: &UnixListener, mut executor: Executor) -> std::io::Result<()> {
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                if let Err(error) = handle(&stream, &mut executor) {
                    // A client that vanished is not the daemon's failure. A
                    // dropped connection must never stop the next one.
                    eprintln!("litric-appd: connection ended: {error}");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn handle(stream: &UnixStream, executor: &mut Executor) -> std::io::Result<()> {
    stream.set_read_timeout(Some(CONNECTION_TIMEOUT))?;
    stream.set_write_timeout(Some(CONNECTION_TIMEOUT))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream.try_clone()?;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let limit = u64::try_from(MAX_LINE)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let read = (&mut reader).take(limit).read_until(b'\n', &mut buf)?;
        if read == 0 {
            return Ok(());
        }
        if !buf.ends_with(b"\n") {
            writer.write_all(
                Response::err("too-long", format!("over {MAX_LINE} bytes"))
                    .encode()
                    .as_bytes(),
            )?;
            return Ok(());
        }

        let line = String::from_utf8_lossy(&buf);
        let reply = match parse(line.trim_end()) {
            Ok(Command::Quit) => {
                writer.write_all(Response::ok(Vec::new()).encode().as_bytes())?;
                writer.flush()?;
                return Ok(());
            }
            Ok(command) => {
                let started = Instant::now();
                let reply = executor.execute(&command);
                // The executor's own record of what it did. Written before the
                // reply so a client that vanishes cannot lose the audit line.
                eprintln!("{}", request_log(&command, &reply, started.elapsed()));
                reply
            }
            Err(error) => Response::err(error.kind(), error.to_string()),
        };
        writer.write_all(reply.encode().as_bytes())?;
        writer.flush()?;
    }
}

/// Send one request and return the reply.
///
/// # Errors
/// Any I/O error, including a timeout, `UnexpectedEof` for a truncated reply,
/// and `InvalidData` for a reply this build cannot parse.
#[cfg(unix)]
pub fn forward(socket: &Path, request: &str, timeout: Duration) -> std::io::Result<Response> {
    let stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    {
        let mut writer = &stream;
        writer.write_all(request.trim_end().as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }

    let mut reader = BufReader::new(stream);
    let mut header = String::new();
    if reader.read_line(&mut header)? == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "the runtime daemon closed the connection without a reply",
        ));
    }
    if !header.ends_with('\n') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "the runtime daemon closed the connection mid-reply",
        ));
    }
    let mut wire = header.clone();
    if let Some(rest) = header.strip_prefix("OK ") {
        let count: usize = rest.trim().parse().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "no line count in the reply")
        })?;
        if count > MAX_REPLY_LINES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("reply declares {count} lines, over the {MAX_REPLY_LINES} limit"),
            ));
        }
        for _ in 0..count {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "the reply ended before its declared line count",
                ));
            }
            wire.push_str(&line);
        }
    }
    Response::decode(&wire).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "the runtime daemon's reply is not in the protocol",
        )
    })
}

/// This build has no Unix sockets, so there is nothing to forward to.
///
/// The broker still compiles and its tests still run: the method exists so the
/// store and launch paths do not need a platform `cfg` at every call site.
#[cfg(not(unix))]
pub fn forward(_socket: &Path, _request: &str, _timeout: Duration) -> std::io::Result<Response> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "this build has no Unix socket transport",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        runtime: Runtime,
        target: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> LaunchRequest {
        LaunchRequest {
            runtime,
            target: target.to_owned(),
            args: args.iter().map(|a| (*a).to_owned()).collect(),
            env: env
                .iter()
                .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
                .collect(),
        }
    }

    #[test]
    fn a_run_line_round_trips_paths_arguments_and_the_display_environment() {
        let original = request(
            Runtime::Flatpak,
            "org.gnome.Builder",
            &["--new-window", "a b\tc"],
            &[("WAYLAND_DISPLAY", "wayland-0"), ("XDG_RUNTIME_DIR", "/run/litric")],
        );
        let line = launch_line(&original);
        assert!(line.starts_with("RUN flatpak "));
        // One separator between fields and nothing else: a value containing a
        // space would have produced an extra field and could not round-trip.
        assert!(line.split(' ').all(|field| !field.is_empty()));
        let parsed = parse(&line).expect("the line parses");
        assert_eq!(parsed, Command::Run(original));
    }

    #[test]
    fn a_target_that_looks_like_the_env_word_cannot_change_the_shape() {
        let original = request(Runtime::LinuxNative, "/tmp/ENV", &["ENV"], &[]);
        let line = launch_line(&original);
        assert_eq!(line.matches(" ENV").count(), 0);
        let parsed = parse(&line).expect("the line parses");
        assert_eq!(parsed, Command::Run(original));
    }

    #[test]
    fn store_lines_are_the_protocols_own_and_name_exactly_one_app() {
        assert_eq!(
            action_line(&SystemAction::Store(StoreAction::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "flathub".to_owned(),
            }))
            .as_deref(),
            Some("STORE INSTALL org.gnome.Builder flathub confirm")
        );
        assert_eq!(
            action_line(&SystemAction::Store(StoreAction::Uninstall {
                id: "org.gnome.Builder".to_owned(),
            }))
            .as_deref(),
            Some("STORE UNINSTALL org.gnome.Builder confirm")
        );
        assert_eq!(
            action_line(&SystemAction::Sandbox(SandboxAction::Update { id: None })).as_deref(),
            Some("STORE UPDATE all confirm")
        );
    }

    #[test]
    fn an_argument_that_the_protocol_could_not_carry_is_not_forwarded() {
        // A value the store would refuse must not become two fields on the way
        // to the daemon. `None` sends the caller down its local refusal path.
        for id in ["org.gnome.Builder flathub", "x confirm STORE UNINSTALL y", ""] {
            assert_eq!(
                action_line(&SystemAction::Store(StoreAction::Install {
                    id: id.to_owned(),
                    remote: "flathub".to_owned(),
                })),
                None,
                "{id:?}"
            );
        }
        assert_eq!(
            action_line(&SystemAction::Store(StoreAction::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "flat hub".to_owned(),
            })),
            None
        );
    }

    #[test]
    fn only_the_closed_set_of_commands_parses() {
        assert_eq!(parse("PING"), Ok(Command::Ping));
        assert_eq!(parse("QUIT"), Ok(Command::Quit));
        assert_eq!(parse("STORE LIST"), Ok(Command::Store(Request::StoreList)));
        assert!(matches!(
            parse("STORE SEARCH photo editor"),
            Ok(Command::Store(Request::StoreSearch { query })) if query == "photo editor"
        ));
        assert!(matches!(
            parse("STORE INSTALL org.gnome.Builder flathub confirm"),
            Ok(Command::Write(SystemAction::Store(StoreAction::Install { .. })))
        ));
        // Everything the broker owns stays with the broker.
        for line in [
            "POWER REBOOT CONFIRM",
            "AUTH SET PASSWORD alice",
            "LAUNCH /usr/bin/thing",
            "NET LINK lo up",
            "RUN",
            "RUN flatpak zzz",
        ] {
            assert!(parse(line).is_err(), "{line} must not be a runtime command");
        }
    }

    #[test]
    fn every_request_leaves_one_journal_line_naming_what_it_did() {
        let install = Command::Write(SystemAction::Store(StoreAction::Install {
            id: "org.gnome.Builder".to_owned(),
            remote: "flathub".to_owned(),
        }));
        let line = request_log(
            &install,
            &Response::ok(vec!["state=installed".to_owned()]),
            Duration::from_millis(1234),
        );
        assert_eq!(
            line,
            "litric.appd.request=store-write result=ok kind=- lines=1 ms=1234"
        );

        let refused = request_log(
            &Command::Ping,
            &Response::err("busy", "another operation runs"),
            Duration::from_millis(0),
        );
        assert_eq!(
            refused,
            "litric.appd.request=ping result=err kind=busy lines=0 ms=0"
        );

        // The four command kinds are distinguishable from the log alone.
        let names = [
            Command::Ping.name(),
            Command::Quit.name(),
            Command::Store(Request::StoreList).name(),
            install.name(),
            Command::Run(LaunchRequest {
                runtime: Runtime::Flatpak,
                target: "org.gnome.Builder".to_owned(),
                args: Vec::new(),
                env: Vec::new(),
            })
            .name(),
        ];
        let unique: std::collections::BTreeSet<&&str> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "names: {names:?}");
    }

    #[test]
    fn a_reply_that_declares_more_lines_than_it_carries_is_not_a_reply() {
        assert!(Response::decode("OK 2\none\n").is_none());
        assert_eq!(
            Response::decode("OK 1\none\n"),
            Some(Response::ok(vec!["one".to_owned()]))
        );
        assert_eq!(
            Response::decode("ERR refused no grant\n"),
            Some(Response::err("refused", "no grant"))
        );
    }

    #[cfg(unix)]
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "litric-appd-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[cfg(unix)]
    fn script(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        let temp = dir.join(format!(".{name}.tmp"));
        std::fs::write(&temp, format!("#!/bin/sh\n{body}\n")).expect("write");
        std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        std::fs::rename(&temp, &path).expect("rename");
        path
    }

    #[cfg(unix)]
    fn wait_for(path: &Path) -> Option<String> {
        use std::time::Instant;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if let Ok(text) = std::fs::read_to_string(path) {
                return Some(text);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    /// The whole path, over a real socket: a broker forwards a catalog read, a
    /// store install and a launch to a real runtime daemon, and gets back what
    /// the executor actually did.
    #[cfg(unix)]
    #[test]
    fn a_read_an_install_and_a_launch_cross_the_real_socket() {
        use crate::action::testing::RecordingRunner;

        let dir = scratch("cross");
        let socket = dir.join("appd.sock");
        let marker = dir.join("launched");

        let listing = "Application\tName\tVersion\tOrigin\tSize\n\
            org.gnome.Builder\tBuilder\t46.1\tflathub\t1.2 GB\n";
        let runner = RecordingRunner::succeeding(0).answering(0, listing);

        let runtime_state = Sysd::new(4, 1024, "/var/lib/litric-app", Path::new("no-such-root"))
            .with_runner(Box::new(runner.clone()));
        let listener = crate::server::bind(&socket).expect("bind the runtime daemon socket");
        std::thread::spawn(move || {
            let _ = serve(&listener, Executor::new(runtime_state));
        });

        let mut broker = Sysd::new(4, 1024, "/root", Path::new("no-such-root"))
            .with_appd_socket(&socket);

        // A catalog read: the reply is the runtime daemon's, and it parsed the
        // real `flatpak list --system` output the runner answered with.
        let reply = broker.handle(&Request::StoreList, 0);
        assert!(reply.is_ok(), "{reply:?}");
        assert!(reply
            .lines()
            .iter()
            .any(|line| line == "store.state=ready"));
        assert!(reply
            .lines()
            .iter()
            .any(|line| line.contains("org.gnome.Builder")));

        // An install: the broker decides, the runtime daemon runs the plan.
        let reply = broker.handle(
            &Request::Action {
                action: SystemAction::Store(StoreAction::Install {
                    id: "org.gnome.Builder".to_owned(),
                    remote: "flathub".to_owned(),
                }),
                consent: true,
            },
            0,
        );
        assert!(reply.is_ok(), "{reply:?}");
        let calls = runner.calls();
        let (program, args) = calls.last().expect("the install was run");
        assert_eq!(program, "flatpak");
        assert_eq!(
            args,
            &vec![
                "install".to_owned(),
                "--system".to_owned(),
                "--noninteractive".to_owned(),
                "--assumeyes".to_owned(),
                "flathub".to_owned(),
                "org.gnome.Builder".to_owned(),
            ]
        );

        // A launch: the reply names a pid the runtime daemon started, and the
        // program really ran in that process.
        let program = script(&dir, "app", &format!("printf ok > '{}'", marker.display()));
        let reply = broker.handle(
            &Request::LaunchApp {
                argv: vec![program.display().to_string()],
            },
            0,
        );
        assert!(reply.is_ok(), "{reply:?}");
        assert!(reply.lines().iter().any(|line| line.starts_with("pid=")));
        assert_eq!(wait_for(&marker).as_deref(), Some("ok"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A broker configured with a daemon that is not running says so, and does
    /// not silently execute the work in its own namespace instead.
    #[cfg(unix)]
    #[test]
    fn a_configured_but_absent_runtime_daemon_is_a_typed_refusal() {
        let dir = scratch("absent");
        let mut broker = Sysd::new(4, 1024, "/root", Path::new("no-such-root"))
            .with_appd_socket(dir.join("nothing-here.sock"));
        let reply = broker.handle(&Request::StoreList, 0);
        assert!(!reply.is_ok());
        assert_eq!(reply.kind(), "appd-unavailable");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A launcher request carries a target the resolver would refuse; the
    /// runtime daemon refuses it with the resolver's own words.
    #[cfg(unix)]
    #[test]
    fn the_runtime_daemon_refuses_a_target_it_cannot_execute() {
        let dir = scratch("bad-target");
        let socket = dir.join("appd.sock");
        let listener = crate::server::bind(&socket).expect("bind");
        let state = Sysd::new(4, 1024, "/var/lib/litric-app", Path::new("no-such-root"));
        std::thread::spawn(move || {
            let _ = serve(&listener, Executor::new(state));
        });

        let reply = crate::appd::forward(&socket, "RUN linux 72656c6174697665", RUN_TIMEOUT)
            .expect("a reply");
        assert!(!reply.is_ok());
        assert_eq!(reply.kind(), "bad-argument");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The whole chain over both sockets: a client speaks to the broker's
    /// socket, the broker authorises and forwards, and the runtime daemon
    /// executes. This is the path the shell takes — `shell.sysd_socket` — with
    /// the executor on the far side rather than in the broker's process.
    #[cfg(unix)]
    #[test]
    fn the_broker_socket_carries_a_store_install_to_the_runtime_daemon() {
        use crate::action::testing::RecordingRunner;

        let dir = scratch("chain");
        let appd_socket = dir.join("appd.sock");
        let sysd_socket = dir.join("sysd.sock");

        let listing = "Application\tName\tVersion\tOrigin\tSize\n\
            org.gnome.Builder\tBuilder\t46.1\tflathub\t1.2 GB\n";
        let runner = RecordingRunner::succeeding(0).answering(0, listing);
        let runtime_state = Sysd::new(4, 1024, "/var/lib/litric-app", Path::new("no-such-root"))
            .with_runner(Box::new(runner.clone()));
        let runtime = crate::server::bind(&appd_socket).expect("bind the runtime daemon");
        std::thread::spawn(move || {
            let _ = serve(&runtime, Executor::new(runtime_state));
        });

        let broker_state = Sysd::new(4, 1024, "/root", Path::new("no-such-root"))
            .with_appd_socket(&appd_socket);
        let broker = crate::server::bind(&sysd_socket).expect("bind the broker");
        std::thread::spawn(move || {
            let _ = crate::server::serve(&broker, broker_state);
        });

        // A read crosses both sockets and comes back as the protocol's own.
        let read =
            crate::server::query(&sysd_socket, "STORE LIST").expect("a read over both sockets");
        assert!(read.starts_with("OK "), "{read}");
        assert!(read.contains("org.gnome.Builder"), "{read}");

        // A write does too: authorised by the broker, executed by the daemon.
        let install = crate::server::query(
            &sysd_socket,
            "STORE INSTALL org.gnome.Builder flathub confirm",
        )
        .expect("an install over both sockets");
        assert!(install.starts_with("OK "), "{install}");
        let calls = runner.calls();
        let (program, args) = calls.last().expect("the install reached the runner");
        assert_eq!(program, "flatpak");
        assert!(
            args.iter().any(|arg| arg == "org.gnome.Builder"),
            "{args:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
