//! The Unix-socket transport.
//!
//! Everything interesting happens in [`crate::queue`]; this file moves bytes and
//! decides when the record reaches the disk. It is Unix-only, which is why the
//! queue and the protocol are free of platform assumptions: the whole daemon's
//! behaviour is testable on the development host, and only the socket needs the
//! target.
//!
//! Two properties are load-bearing, and both are copies of decisions
//! `litric-sysd` already made and booted:
//!
//! * **A request line is length-limited before it is buffered.** A client that
//!   sends a gigabyte without a newline is disconnected after 4 KiB.
//! * **A connection cannot take the daemon down.** Each is served on its own
//!   thread, and a thread that panics while holding the lock hands the state to
//!   the next caller instead of poisoning the daemon for good.
//!
//! The socket is owner-only (`0600`). Any program that can open it can put text
//! on the user's screen with an application name attached, which is exactly the
//! authority a notification sender has and exactly as much of it as it should be
//! able to get.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{io, thread};

use crate::proto::{
    encode_error, note_header, note_lines, parse_post, parse_request, Pending, ProtoError, Request,
    Response, MAX_BLOCK_LINES, MAX_LINE,
};
use crate::queue::{Queue, Store};

/// Milliseconds since the Unix epoch, or 0 if the clock is before it.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// The daemon's state: the queue and where it is written.
#[derive(Debug)]
pub struct State {
    queue: Queue,
    store: Option<Store>,
    /// The last persistence failure, kept so `INFO` can report it rather than
    /// leaving it to a journal line the user will never open.
    last_error: Option<String>,
    /// How many records were unreadable when the queue was loaded.
    recovered_skipped: usize,
}

impl State {
    /// A state around `queue`, writing through `store` when there is one.
    #[must_use]
    pub fn new(queue: Queue, store: Option<Store>) -> Self {
        Self {
            queue,
            store,
            last_error: None,
            recovered_skipped: 0,
        }
    }

    /// Load from a store, expiring what is no longer worth showing.
    ///
    /// # Errors
    /// A queue file that exists and cannot be read. An absent one is an empty
    /// queue.
    pub fn load(store: Store, now_ms: u64) -> Result<Self, String> {
        let (mut queue, skipped) = store.load()?;
        let expired = queue.expire(now_ms);
        let mut state = Self::new(queue, Some(store));
        state.recovered_skipped = skipped;
        if expired > 0 {
            // Written back immediately: an expired note that is still on disk
            // would come back if the daemon were killed before the next change.
            state.persist();
        }
        Ok(state)
    }

    /// The queue.
    #[must_use]
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Write the queue, recording whether it worked.
    fn persist(&mut self) -> bool {
        let Some(store) = &self.store else {
            return true;
        };
        match store.save(&self.queue) {
            Ok(()) => {
                self.last_error = None;
                true
            }
            Err(error) => {
                eprintln!("litric-notifyd: the queue is not persisted: {error}");
                self.last_error = Some(error);
                false
            }
        }
    }

    /// Answer one request that is not a post.
    pub fn handle(&mut self, request: Request, now_ms: u64) -> Response {
        self.queue.expire(now_ms);
        match request {
            Request::Hello => Response::ok(vec![format!("version={}", crate::proto::VERSION)]),
            Request::Info => {
                let mut lines = vec![
                    format!("version={}", crate::proto::VERSION),
                    format!("queued={}", self.queue.count()),
                    format!("capacity={}", Queue::CAPACITY),
                    format!("skipped_on_load={}", self.recovered_skipped),
                    format!(
                        "state={}",
                        self.store
                            .as_ref()
                            .map_or("memory", |s| s.path().to_str().unwrap_or("memory"))
                    ),
                    format!("persisting={}", u8::from(self.store.is_some())),
                ];
                if let Some(error) = &self.last_error {
                    lines.push(format!("last_error={error}"));
                }
                Response::ok(lines)
            }
            Request::Count => Response::ok(vec![format!("count={}", self.queue.count())]),
            Request::List => Response::ok(list_lines(self.queue.list())),
            Request::Take => {
                let taken = self.queue.take();
                let persisted = self.persist();
                let mut lines = vec![format!("persisted={}", u8::from(persisted))];
                lines.extend(list_lines(&taken));
                Response::ok(lines)
            }
            Request::Dismiss(id) => {
                let dismissed = self.queue.dismiss(id);
                if dismissed {
                    self.persist();
                }
                Response::ok(vec![format!("dismissed={}", u8::from(dismissed))])
            }
            Request::Clear => {
                let cleared = self.queue.clear();
                if cleared > 0 {
                    self.persist();
                }
                Response::ok(vec![format!("cleared={cleared}")])
            }
            // `Quit` is handled by the connection loop, which closes the socket.
            Request::Quit => Response::ok(Vec::new()),
        }
    }

    /// Accept a posted notification.
    pub fn handle_post(&mut self, pending: &Pending, now_ms: u64) -> Response {
        let id = self.queue.post(pending, now_ms);
        let persisted = self.persist();
        Response::ok(vec![
            format!("id={id}"),
            format!("persisted={}", u8::from(persisted)),
        ])
    }
}

/// The lines a list of notes is written as: `NOTE <id>`, its fields, `END`.
fn list_lines(notes: &[crate::proto::Note]) -> Vec<String> {
    let mut out = Vec::new();
    for note in notes {
        out.push(format!("NOTE {}", note.id));
        out.extend(note_lines(note));
        out.push("END".to_owned());
    }
    out
}

/// Parse notes out of the lines between a reply header and its end.
#[must_use]
pub fn notes_from_lines(lines: &[String]) -> Vec<crate::proto::Note> {
    let mut notes = Vec::new();
    let mut id: Option<u32> = None;
    let mut block: Vec<String> = Vec::new();
    for line in lines {
        if let Some(note_id) = note_header(line) {
            // A second header without an END means the previous one was
            // truncated; drop it rather than merging two notifications.
            id = Some(note_id);
            block.clear();
        } else if line == "END" {
            if let Some(note_id) = id.take() {
                if let Some(note) = crate::proto::note_from_lines(note_id, &block) {
                    notes.push(note);
                }
            }
            block.clear();
        } else if id.is_some() {
            block.push(line.clone());
        }
    }
    notes
}

/// Bind the socket, replacing a stale one from a previous run.
///
/// Refuses to unlink anything that is not a socket: a daemon that removes
/// whatever it finds at its own path is one typo away from deleting a user's
/// file. A socket something still answers on belongs to a live daemon and is
/// not replaced either: unlinking it would leave that daemon accepting on an
/// inode no client can reach.
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
    if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
        // Serving with wider permissions than intended is worse than not
        // starting, and the socket just made is ours to remove.
        let _ = fs::remove_file(path);
        return Err(e);
    }
    Ok(listener)
}

/// The most connections `litric-notifyd` serves at once.
///
/// The accept loop used to spawn one thread per connection with no ceiling, so
/// a client that opened connections faster than it closed them multiplied the
/// per-connection line buffer (up to `MAX_LINE + 1` bytes) and its handler
/// stack without limit. This is what the worst case is computed from.
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
/// Any `accept` error that is not recoverable.
pub fn serve(listener: &UnixListener, state: State) -> io::Result<()> {
    let shared = Arc::new(Mutex::new(state));
    let connections = Connections::new();
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                let Some(slot) = connections.enter() else {
                    // This protocol answers only OK frames, so there is no error
                    // frame to send. A closed connection is unambiguous and the
                    // client retries; nothing was allocated for it.
                    continue;
                };
                let shared = Arc::clone(&shared);
                // A client that hangs must not stop the next one being served.
                if let Err(e) = thread::Builder::new()
                    .name("litric-notifyd-conn".to_owned())
                    .spawn(move || {
                        let _slot = slot;
                        if let Err(e) = handle(&stream, &shared) {
                            eprintln!("litric-notifyd: connection ended: {e}");
                        }
                    })
                {
                    eprintln!("litric-notifyd: could not spawn a handler: {e}");
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Read one line, bounded.
fn read_line(reader: &mut impl BufRead) -> io::Result<Option<String>> {
    let limit = u64::try_from(MAX_LINE)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut buf = Vec::new();
    let read = reader.take(limit).read_until(b'\n', &mut buf)?;
    if read == 0 {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&buf).trim_end().to_owned()))
}

/// Serve one connection to completion.
fn handle(stream: &UnixStream, shared: &Arc<Mutex<State>>) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream.try_clone()?;

    loop {
        let Some(line) = read_line(&mut reader)? else {
            return Ok(()); // client closed
        };
        if line.is_empty() {
            continue;
        }
        if line.len() > MAX_LINE {
            writer.write_all(
                encode_error(ProtoError::TooLong.kind(), "a line exceeded the limit").as_bytes(),
            )?;
            return Ok(());
        }

        // A post is a verb plus a block. Collected here, where the connection
        // is, so `parse_post` stays a pure function of lines.
        if line.trim().eq_ignore_ascii_case("POST") {
            let mut block: Vec<String> = Vec::new();
            let mut truncated = false;
            loop {
                let Some(next) = read_line(&mut reader)? else {
                    return Ok(());
                };
                if next.eq_ignore_ascii_case("END") {
                    break;
                }
                if block.len() >= MAX_BLOCK_LINES {
                    truncated = true;
                    break;
                }
                block.push(next);
            }
            let reply = if truncated {
                encode_error("malformed", "a post may carry at most 8 fields")
            } else {
                match parse_post(&block) {
                    Ok(pending) => lock(shared).handle_post(&pending, now_ms()).encode(),
                    Err(error) => encode_error(error.kind(), &error.to_string()),
                }
            };
            writer.write_all(reply.as_bytes())?;
            writer.flush()?;
            if truncated {
                return Ok(());
            }
            continue;
        }

        let reply = match parse_request(&line) {
            Ok(Request::Quit) => {
                writer.write_all(Response::ok(Vec::new()).encode().as_bytes())?;
                writer.flush()?;
                return Ok(());
            }
            Ok(request) => lock(shared).handle(request, now_ms()).encode(),
            Err(error) => encode_error(error.kind(), &error.to_string()),
        };
        writer.write_all(reply.as_bytes())?;
        writer.flush()?;
    }
}

/// Take the state lock and answer.
///
/// A previous handler that panicked leaves the mutex poisoned. Refusing every
/// later request because one connection died would turn a bad client into an
/// outage, so the state is recovered and service continues.
fn lock(shared: &Arc<Mutex<State>>) -> std::sync::MutexGuard<'_, State> {
    match shared.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Connect, send one request, and read the declared reply.
///
/// # Errors
/// Any I/O error from connecting, writing or reading.
pub fn query(path: &Path, request: &str) -> io::Result<String> {
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(request.trim_end().as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    read_reply(BufReader::new(stream))
}

/// Send a one-line request and read the whole reply.
fn read_reply(mut reader: impl BufRead) -> io::Result<String> {
    let mut header = String::new();
    reader.read_line(&mut header)?;
    let mut out = header.clone();

    // Read exactly the number of lines the header declared. Reading to EOF would
    // work here and would stop working the moment the connection is reused,
    // which is the latent bug a declared count exists to prevent.
    if let Some(rest) = header.strip_prefix("OK ") {
        let count: usize = rest.trim().parse().unwrap_or(0);
        for _ in 0..count {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            out.push_str(&line);
        }
    }
    Ok(out)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::proto::{Note, Priority};

    fn state() -> State {
        State::new(Queue::default(), None)
    }

    /// A notification that does not expire, so a test can advance the clock
    /// without the queue silently emptying underneath it.
    fn pending(title: &str) -> Pending {
        Pending {
            app: "com.litric.test".to_owned(),
            priority: Priority::High,
            title: title.to_owned(),
            body: "body".to_owned(),
            ttl_ms: None,
        }
    }

    fn expiring(title: &str) -> Pending {
        Pending {
            ttl_ms: Some(1_000),
            ..pending(title)
        }
    }

    /// A scratch directory for the bind tests, tagged with this process.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-notifyd-bind-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("mkdir");
        dir
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

    #[test]
    fn a_live_daemons_socket_is_not_replaced_and_the_socket_is_owner_only() {
        let dir = scratch("live");
        let path = dir.join("notify.sock");
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
        let path = dir.join("notify.sock");
        {
            // A listener that was killed: the file stays, nothing answers.
            let dead = UnixListener::bind(&path).expect("bind");
            drop(dead);
        }
        let listener = bind(&path).expect("a stale socket is replaced");
        let mode = fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        drop(listener);
        let _ = fs::remove_file(&path);

        let file = dir.join("important.txt");
        fs::write(&file, b"do not delete").expect("write");
        let err = bind(&file).expect_err("a plain file is occupied");
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read(&file).expect("still there"),
            b"do not delete",
            "a daemon must not unlink a file it did not create"
        );
    }

    #[test]
    fn a_post_is_answered_with_its_id_and_whether_it_reached_the_disk() {
        let mut s = state();
        let reply = s.handle_post(&pending("first"), 1_000).encode();
        assert!(reply.starts_with("OK 2\n"), "{reply}");
        assert!(reply.contains("id=1\n"), "{reply}");
        assert!(
            reply.contains("persisted=1\n"),
            "a daemon with no store has nothing to persist, and says so truthfully: {reply}"
        );
        assert_eq!(s.queue().count(), 1);
    }

    #[test]
    fn take_hands_over_the_queue_and_empties_it_in_one_request() {
        let mut s = state();
        s.handle_post(&pending("a"), 1_000);
        s.handle_post(&pending("b"), 2_000);
        let reply = s.handle(Request::Take, 3_000).encode();
        // Two notes, eight lines each (`NOTE`, six fields, `END`), plus the
        // persisted flag.
        assert!(reply.starts_with("OK 17\n"), "{reply}");
        assert_eq!(s.queue().count(), 0);
        let lines: Vec<String> = reply.lines().skip(1).map(str::to_owned).collect();
        let notes = notes_from_lines(&lines);
        assert_eq!(notes.len(), 2, "{lines:?}");
        assert_eq!(notes[0].title, "b", "newest first");
    }

    #[test]
    fn a_second_take_is_empty() {
        // The property the shell's startup drain depends on: a restart does not
        // re-show what the last run already showed.
        let mut s = state();
        s.handle_post(&pending("a"), 1_000);
        s.handle(Request::Take, 2_000);
        let reply = s.handle(Request::Take, 3_000).encode();
        assert_eq!(reply, "OK 1\npersisted=1\n");
    }

    #[test]
    fn a_list_does_not_remove_anything() {
        let mut s = state();
        s.handle_post(&pending("a"), 1_000);
        let reply = s.handle(Request::List, 2_000).encode();
        assert_eq!(
            notes_from_lines(&reply.lines().skip(1).map(str::to_owned).collect::<Vec<_>>()).len(),
            1
        );
        assert_eq!(s.queue().count(), 1);
    }

    #[test]
    fn dismiss_and_clear_report_what_they_did() {
        let mut s = state();
        let id = 1;
        s.handle_post(&pending("a"), 1_000);
        let dismissed = s.handle(Request::Dismiss(id), 2_000).encode();
        assert!(dismissed.contains("dismissed=1"), "{dismissed}");
        let again = s.handle(Request::Dismiss(id), 2_000).encode();
        assert!(again.contains("dismissed=0"), "{again}");
        s.handle_post(&pending("b"), 3_000);
        let cleared = s.handle(Request::Clear, 4_000).encode();
        assert!(cleared.contains("cleared=1"), "{cleared}");
    }

    #[test]
    fn an_expired_note_is_not_returned_even_if_it_is_still_queued() {
        let mut s = state();
        s.handle_post(&expiring("short"), 1_000); // expires at 2000
        let reply = s.handle(Request::Take, 2_500).encode();
        assert_eq!(reply, "OK 1\npersisted=1\n");
        assert_eq!(s.queue().count(), 0);
    }

    #[test]
    fn info_says_where_the_queue_lives_and_what_it_lost() {
        let mut s = state();
        let reply = s.handle(Request::Info, 1_000).encode();
        assert!(reply.contains("queued=0"), "{reply}");
        assert!(reply.contains("state=memory"), "{reply}");
        assert!(
            reply.contains(&format!("capacity={}", Queue::CAPACITY)),
            "{reply}"
        );
        assert!(reply.contains("persisting=0"), "{reply}");
    }

    #[test]
    fn a_take_reply_parses_back_into_the_notes_that_were_queued() {
        let notes = vec![Note {
            id: 3,
            app: "com.litric.files".to_owned(),
            priority: Priority::Critical,
            title: "Disk is full · ดิสก์เต็ม".to_owned(),
            body: "Free space in /var".to_owned(),
            created_ms: 1_700_000_000_000,
            expires_ms: None,
        }];
        let lines = list_lines(&notes);
        assert_eq!(notes_from_lines(&lines), notes);
        // Every line a block carries is inside it: `NOTE`, six fields, `END`.
        assert_eq!(lines.len(), 8);
    }

    #[test]
    fn a_truncated_note_block_is_dropped_rather_than_merged() {
        let lines: Vec<String> = vec![
            "NOTE 1".to_owned(),
            "app=a".to_owned(),
            "priority=low".to_owned(),
            "title=one".to_owned(),
            "NOTE 2".to_owned(),
            "app=b".to_owned(),
            "priority=low".to_owned(),
            "title=two".to_owned(),
            "created_ms=5".to_owned(),
            "expires_ms=0".to_owned(),
            "END".to_owned(),
        ];
        let notes = notes_from_lines(&lines);
        assert_eq!(notes.len(), 1, "the unterminated block is dropped");
        assert_eq!(notes[0].title, "two");
    }

    #[test]
    fn a_socket_query_round_trips_a_post_and_a_take() {
        // The transport, exercised for real: bind in a temporary directory,
        // serve on a thread, and talk to it the way the shell will.
        let dir = std::env::temp_dir().join(format!("litric-notifyd-sock-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("notify.sock");
        let listener = bind(&path).expect("binds");
        let state = State::new(Queue::default(), None);
        let worker = thread::spawn(move || {
            // One connection, then done: `query` opens one each time.
            if let Ok((stream, _)) = listener.accept() {
                let shared = Arc::new(Mutex::new(state));
                let _ = handle(&stream, &shared);
            }
        });

        let mut stream = UnixStream::connect(&path).expect("connects");
        stream
            .write_all(b"POST\napp=com.litric.test\npriority=high\nttl_ms=0\ntitle=hello\nEND\n")
            .expect("writes");
        stream.flush().expect("flushes");
        let reply = read_reply(BufReader::new(stream)).expect("reads");
        assert!(reply.contains("id=1"), "{reply}");
        worker.join().expect("the handler finished");

        let listed = {
            let listener = bind(&path).expect("rebinds");
            let state = State::new(Queue::default(), None);
            let t = thread::spawn(move || {
                if let Ok((stream, _)) = listener.accept() {
                    let shared = Arc::new(Mutex::new(state));
                    let _ = handle(&stream, &shared);
                }
            });
            let reply = query(&path, "TAKE").expect("the daemon answers");
            t.join().expect("the handler finished");
            reply
        };
        // A fresh state has nothing: the point of the test is that the
        // transport carries a post and a take, not that memory is durable.
        assert!(listed.starts_with("OK "), "{listed}");
        let _ = fs::remove_dir_all(&dir);
    }
}
