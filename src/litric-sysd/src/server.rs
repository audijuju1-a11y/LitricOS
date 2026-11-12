//! The Unix-socket transport.
//!
//! Everything interesting happens in [`crate::state`]; this file only moves
//! bytes. It is compiled on Unix only, which is why the rest of the crate is
//! carefully free of platform assumptions: the daemon's whole behaviour stays
//! testable on the development host, and only the transport needs the target.
//!
//! Two properties are worth stating because they are the ones that bite:
//!
//! * **A request line is length-limited before it is buffered.** A client that
//!   sends a gigabyte without a newline gets disconnected after 4 KiB rather
//!   than growing the daemon's heap by a gigabyte.
//! * **A connection cannot take the daemon down.** Each is served on its own
//!   thread, and a thread that panics while holding the state lock hands the
//!   state to the next caller rather than poisoning the daemon permanently.
//! * **A second application operation is refused, not queued.** An install can
//!   hold the state lock for as long as its plan allows (900 s for a Flatpak
//!   install), so a second install that simply waited would look like a hung
//!   desktop. It gets `ERR busy` immediately instead.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fs, io, thread};

use crate::proto::{ProtoError, Request, Response, MAX_LINE};
use crate::providers::SystemAction;
use crate::state::{Sysd, Timestamp};

/// Seconds since the Unix epoch, or 0 if the clock is before it.
fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Bind the socket, replacing a stale one from a previous run.
///
/// Refuses to unlink anything that is not a socket. A daemon that removes
/// whatever it finds at its own path is one `--socket` typo away from deleting
/// a user's file. A socket something still answers on belongs to a live daemon
/// and is not replaced either: unlinking it would leave that daemon accepting
/// on an inode no client can reach.
///
/// # Errors
/// Any I/O error from creating the directory, removing the stale socket or
/// binding.
pub fn bind(path: &Path) -> io::Result<UnixListener> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_socket() => {
            if UnixStream::connect(path).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!("another daemon is already listening on {}", path.display()),
                ));
            }
            fs::remove_file(path)?
        }
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} exists and is not a socket", path.display()),
            ))
        }
        Err(_) => {}
    }
    let listener = UnixListener::bind(path)?;
    // Owner only. The shell will need a group here once it exists and runs as
    // the user; widening it before there is anything to widen it for would be
    // permission granted to nobody in particular.
    if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
        // Serving with wider permissions than intended is worse than not
        // starting, and the socket just made is ours to remove.
        let _ = fs::remove_file(path);
        return Err(e);
    }
    Ok(listener)
}

/// The most connections `litric-sysd` serves at once.
///
/// The accept loop used to spawn one thread per connection with no ceiling, so
/// a client that opened connections faster than it closed them multiplied the
/// per-connection line buffer (up to `MAX_LINE + 1` bytes) and its handler
/// stack without limit. This is what the worst case is computed from. A
/// connection over the cap is answered with one `ERR limit` line and closed.
const MAX_CONNECTIONS: usize = 16;

/// Live handler count with a hard ceiling.
#[derive(Debug)]
struct Connections {
    live: Arc<AtomicUsize>,
}

impl Connections {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            live: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Take a slot, or `None` when every slot is already being served.
    fn enter(self: &Arc<Self>) -> Option<ConnectionSlot> {
        let mut live = self.live.load(Ordering::Relaxed);
        loop {
            if live >= MAX_CONNECTIONS {
                return None;
            }
            match self.live.compare_exchange_weak(
                live,
                live.saturating_add(1),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(ConnectionSlot {
                        live: Arc::clone(&self.live),
                    })
                }
                Err(current) => live = current,
            }
        }
    }
}

/// One served connection; dropping it frees the slot.
#[derive(Debug)]
struct ConnectionSlot {
    live: Arc<AtomicUsize>,
}

impl Drop for ConnectionSlot {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Serve until the listener fails.
///
/// # Errors
/// Any I/O error from `accept` that is not recoverable.
pub fn serve(listener: &UnixListener, state: Sysd) -> io::Result<()> {
    // The serve loop is the running-system side of the update health gate. The
    // boot entrypoint recorded `booting` for one candidate attempt; only the
    // loop that actually bound the socket can upgrade it to `healthy`. Failure
    // here is logged, not fatal: a bad update root must not take down sysd.
    if let Err(error) = state.attest_update_health(now()) {
        eprintln!("litric-sysd: update health attestation failed: {error}");
    }
    let interval = state.backup_interval_secs();
    let shared = Arc::new(Mutex::new(state));
    // Held for the duration of one application operation. It exists so that a
    // second one can be told "busy" before it queues on the state lock: by the
    // time it got the lock, the first operation would have finished or the
    // client would have given up, and neither reads as an answer.
    let writer = Arc::new(Mutex::new(()));
    if interval > 0 {
        spawn_backup_scheduler(Arc::clone(&shared), interval);
    }
    let connections = Connections::new();
    for conn in listener.incoming() {
        match conn {
            Ok(mut stream) => {
                let Some(slot) = connections.enter() else {
                    // One reply, then close: the protocol has a kind for a
                    // resource limit, so this is distinguishable from a crash.
                    let _ = stream.write_all(
                        Response::err("limit", "connection limit reached")
                            .encode()
                            .as_bytes(),
                    );
                    continue;
                };
                let shared = Arc::clone(&shared);
                let writer = Arc::clone(&writer);
                // A client that hangs must not stop the next one being served.
                if let Err(e) = thread::Builder::new()
                    .name("litric-sysd-conn".to_owned())
                    .spawn(move || {
                        let _slot = slot;
                        if let Err(e) = handle(&stream, &shared, &writer) {
                            eprintln!("litric-sysd: connection ended: {e}");
                        }
                    })
                {
                    eprintln!("litric-sysd: could not spawn a handler: {e}");
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Run the automatic backup on the daemon's clock.
///
/// The first tick happens immediately and later ticks are checked every minute
/// (or at the interval, when it is shorter). The thread does no request work
/// and shares the same [`Sysd`] lock, so a snapshot cannot observe a half-applied
/// install or update. It is deliberately detached: a backup failure is recorded
/// in `BACKUP STATUS` and must not stop the socket serving the machine.
fn spawn_backup_scheduler(shared: Arc<Mutex<Sysd>>, interval: u64) {
    let wake = interval.clamp(1, 60);
    let spawned = thread::Builder::new()
        .name("litric-sysd-backup".to_owned())
        .spawn(move || loop {
            let now = now();
            let result = {
                let mut guard = match shared.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                guard.backup_tick(now)
            };
            if let Some(Err(error)) = result {
                eprintln!("litric-sysd: automatic backup failed: {error}");
            }
            thread::sleep(Duration::from_secs(wake));
        });
    if let Err(error) = spawned {
        eprintln!("litric-sysd: could not start the backup scheduler: {error}");
    }
}

/// Serve one connection to completion.
fn handle(
    stream: &UnixStream,
    shared: &Arc<Mutex<Sysd>>,
    operation: &Arc<Mutex<()>>,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream.try_clone()?;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        // Read at most one line, and at most MAX_LINE + 1 bytes of it, so an
        // unterminated flood costs a fixed amount of memory rather than an
        // unbounded one.
        let limit = u64::try_from(MAX_LINE)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let read = (&mut reader).take(limit).read_until(b'\n', &mut buf)?;
        if read == 0 {
            return Ok(()); // client closed
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
        let reply = match Request::parse(line.trim_end()) {
            Ok(Request::Quit) => {
                writer.write_all(Response::ok(Vec::new()).encode().as_bytes())?;
                return Ok(());
            }
            Ok(req) => dispatch(shared, operation, &req),
            Err(ProtoError::Empty) => continue,
            Err(e) => Response::err(e.kind(), e.to_string()),
        };
        writer.write_all(reply.encode().as_bytes())?;
        writer.flush()?;
    }
}

/// Whether a request runs an application-side operation that can take minutes.
///
/// These are the requests that reach Flatpak and the launcher: a catalog *read*
/// is a separate variant and is not one of them. The list is deliberately
/// explicit rather than a catch-all, so a new request has to be thought about
/// here before it can be treated as one.
fn is_application_operation(req: &Request) -> bool {
    matches!(
        req,
        Request::Action {
            action: SystemAction::Store(_) | SystemAction::Sandbox(_),
            ..
        } | Request::LaunchApp { .. }
            | Request::LaunchAppEnv { .. }
            | Request::LaunchProfile { .. }
    )
}

/// One application operation at a time, refused rather than queued.
///
/// Returns the guard to hold while the operation runs, `None` for a request
/// that is not one, or the reply to send when one is already running. A
/// poisoned mutex is not a reason to refuse forever — the poison means a
/// handler panicked, and the guard is only ever `()` — so it is taken anyway.
fn writer_guard<'a>(
    writer: &'a Arc<Mutex<()>>,
    req: &Request,
) -> Result<Option<MutexGuard<'a, ()>>, Response> {
    if !is_application_operation(req) {
        return Ok(None);
    }
    match writer.try_lock() {
        Ok(guard) => Ok(Some(guard)),
        Err(TryLockError::WouldBlock) => Err(Response::err(
            "busy",
            "another application operation is already running; retry when it finishes",
        )),
        Err(TryLockError::Poisoned(poisoned)) => Ok(Some(poisoned.into_inner())),
    }
}

/// Take the state lock and answer.
///
/// A previous handler that panicked leaves the mutex poisoned. Refusing every
/// subsequent request because one connection died is a self-inflicted outage,
/// so the state is recovered and service continues.
fn dispatch(shared: &Arc<Mutex<Sysd>>, writer: &Arc<Mutex<()>>, req: &Request) -> Response {
    let _writer = match writer_guard(writer, req) {
        Ok(guard) => guard,
        Err(reply) => return reply,
    };
    let mut guard = match shared.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.handle(req, now())
}

/// Connect, send one request, and return the raw reply.
///
/// This is what makes the daemon testable from a shell script inside the guest
/// without adding `socat` to the image.
///
/// # Errors
/// Any I/O error from connecting, writing or reading.
pub fn query(path: &Path, request: &str) -> io::Result<String> {
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(request.trim_end().as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut header = String::new();
    reader.read_line(&mut header)?;
    let mut out = header.clone();

    // Read exactly the number of lines the header declared. Reading to EOF
    // would work here and would stop working the moment the connection is
    // reused, which is exactly the kind of latent bug a declared count exists
    // to prevent.
    if let Some(rest) = header.strip_prefix("OK ") {
        let count: usize = rest.trim().parse().unwrap_or(0);
        for _ in 0..count {
            let mut l = String::new();
            if reader.read_line(&mut l)? == 0 {
                break;
            }
            out.push_str(&l);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A scratch directory for the bind tests, tagged with this process.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-sysd-bind-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn a_live_daemons_socket_is_not_replaced_and_the_socket_is_owner_only() {
        let dir = scratch("live");
        let path = dir.join("sysd.sock");
        let first = bind(&path).expect("first bind");
        let mode = fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the socket must be owner-only");
        let second = bind(&path).expect_err("a live socket is not replaced");
        assert_eq!(second.kind(), io::ErrorKind::AddrInUse);
        UnixStream::connect(&path).expect("the first listener still answers");
        drop(first);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_stale_socket_is_replaced_and_a_file_is_never_deleted() {
        let dir = scratch("stale");
        let path = dir.join("sysd.sock");
        {
            // A listener that was killed: the file stays, nothing answers.
            let dead = UnixListener::bind(&path).expect("bind");
            drop(dead);
        }
        let listener = bind(&path).expect("a stale socket is replaced");
        assert_eq!(
            fs::metadata(&path).expect("stat").permissions().mode() & 0o777,
            0o600
        );
        drop(listener);
        let _ = fs::remove_file(&path);

        let file = dir.join("important.txt");
        fs::write(&file, b"do not delete").expect("write");
        let err = bind(&file).expect_err("a plain file is occupied");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&file).expect("still there"), b"do not delete");
    }

    #[test]
    fn the_connection_cap_serves_exactly_max_connections() {
        let gate = Connections::new();
        let mut held: Vec<_> = (0..MAX_CONNECTIONS).map(|_| gate.enter()).collect();
        assert!(held.iter().all(Option::is_some), "a slot under the cap");
        assert!(
            gate.enter().is_none(),
            "connection {} must be refused",
            MAX_CONNECTIONS.saturating_add(1)
        );
        held.pop();
        assert!(gate.enter().is_some(), "a closed connection frees its slot");
    }

    /// An install in flight answers a second install with `busy`, immediately,
    /// rather than queueing it behind a plan that may run for 900 seconds.
    #[test]
    fn a_second_application_operation_is_refused_while_one_is_running() {
        use crate::action::testing::RecordingRunner;
        use crate::store::StoreAction;

        let state = Sysd::new(2, 512, "/tmp", Path::new("/"))
            .with_runner(Box::new(RecordingRunner::succeeding(0)));
        let shared = Arc::new(Mutex::new(state));
        let writer = Arc::new(Mutex::new(()));

        let install = Request::Action {
            action: SystemAction::Store(StoreAction::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "flathub".to_owned(),
            }),
            consent: true,
        };

        // Nothing running: the request is executed, not refused.
        let reply = dispatch(&shared, &writer, &install);
        assert_ne!(reply.kind(), "busy", "{reply:?}");

        // One in flight: the next one is told so, and it takes no time.
        let held = writer.lock().expect("the slot");
        let reply = dispatch(&shared, &writer, &install);
        assert_eq!(reply.kind(), "busy", "{reply:?}");
        assert!(
            reply.detail().contains("already running"),
            "the refusal must say what to wait for: {reply:?}"
        );

        // A read is not an application operation and is still served: the UI
        // must be able to ask for facts while an install runs.
        let ping = dispatch(&shared, &writer, &Request::Ping);
        assert!(ping.is_ok(), "{ping:?}");
        drop(held);
    }

    /// The guard is released on drop, including when the held lock was left
    /// poisoned by a handler that panicked.
    #[test]
    fn a_released_operation_slot_leaves_the_next_request_free() {
        use crate::store::StoreAction;

        let writer = Arc::new(Mutex::new(()));
        let uninstall = Request::Action {
            action: SystemAction::Store(StoreAction::Uninstall {
                id: "org.gnome.Builder".to_owned(),
            }),
            consent: true,
        };
        {
            let _held = writer_guard(&writer, &uninstall).expect("free");
        }
        let again = writer_guard(&writer, &uninstall);
        assert!(again.is_ok(), "the slot was released with the guard");
        // The reply carries the guard itself, so it is released by dropping it
        // and not by the assertion above.
        drop(again);

        // Poisoning is not a permanent refusal: the guard carries no state.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = writer.lock().expect("the slot");
            panic!("a handler panicked while holding the slot");
        }));
        let after = writer_guard(&writer, &uninstall);
        assert!(
            after.is_ok(),
            "a poisoned slot must still be usable: {after:?}"
        );
    }
}
