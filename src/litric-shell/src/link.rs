//! A background connection from the desktop to a socket-activated daemon.
//!
//! The desktop must never block on a daemon. A browser engine that takes a
//! second to start, or a folder copy that takes a minute, would otherwise
//! freeze every window on the screen. So each connection has a reader thread
//! that turns the byte stream into messages, parks them in a queue, and wakes
//! the desktop's loop; the loop drains the queue when it wakes. Commands are
//! short lines and are written directly, with a timeout so a wedged daemon
//! cannot hang the interface either.
//!
//! The link knows nothing about any protocol. A [`Framing`] supplied by the
//! caller says how long a line may be, whether a header line announces a
//! binary payload and how long it is, and which payloads replace each other.
//! That last point is what bounds memory: a browser tab producing frames
//! faster than the desktop draws them keeps one queued frame, not a backlog.
//!
//! Unix only: every daemon it talks to is reached through a Unix socket.

use std::collections::VecDeque;
#[cfg(unix)]
use std::io::Write;
use std::io::{self, BufRead, BufReader, Read};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
#[cfg(unix)]
use std::thread;
use std::thread::JoinHandle;
#[cfg(unix)]
use std::time::Duration;

/// Something that arrived from the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoming {
    /// A text line, without its newline.
    Line(String),
    /// A header line followed by the binary payload it announced.
    Payload {
        /// The header line, without its newline.
        header: String,
        /// The payload.
        bytes: Vec<u8>,
    },
    /// The connection ended; nothing more will arrive. The text says why.
    Closed(String),
}

/// How a byte stream divides into messages.
#[derive(Debug, Clone, Copy)]
pub struct Framing {
    /// The longest header line accepted, excluding the newline. A longer one
    /// closes the connection: a stream that has lost its framing cannot be
    /// resynchronised safely.
    pub max_line: usize,
    /// How many payload bytes follow this header line (0 for none), or an
    /// error that closes the connection. Enforcing a payload size limit is the
    /// caller's job, here.
    pub payload_len: fn(&str) -> Result<usize, String>,
    /// Payloads with the same key replace each other in the queue. `None`
    /// keeps every payload.
    pub coalesce_key: fn(&str) -> Option<String>,
}

impl Framing {
    /// Plain text lines, no payloads.
    #[must_use]
    pub const fn lines(max_line: usize) -> Self {
        Self {
            max_line,
            payload_len: |_| Ok(0),
            coalesce_key: |_| None,
        }
    }
}

/// The queue between the reader thread and the desktop.
#[derive(Debug, Default)]
struct Inbox {
    queue: VecDeque<Incoming>,
    closed: bool,
}

fn lock(inbox: &Mutex<Inbox>) -> MutexGuard<'_, Inbox> {
    // A reader thread that panicked while holding the lock leaves it poisoned;
    // the queue itself is still consistent, so keep using it.
    inbox.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Push a message, replacing a queued payload with the same coalescing key.
/// Used by the reader thread (Unix) and by the queue tests (every host).
#[allow(dead_code)]
fn deliver(inbox: &Mutex<Inbox>, framing: &Framing, msg: Incoming) {
    let mut guard = lock(inbox);
    if let Incoming::Payload { header, .. } = &msg {
        if let Some(key) = (framing.coalesce_key)(header) {
            let same = guard.queue.iter().position(|m| {
                matches!(m, Incoming::Payload { header: h, .. }
                    if (framing.coalesce_key)(h).as_deref() == Some(key.as_str()))
            });
            if let Some(slot) = same.and_then(|i| guard.queue.get_mut(i)) {
                *slot = msg;
                return;
            }
        }
    }
    if matches!(msg, Incoming::Closed(_)) {
        guard.closed = true;
    }
    guard.queue.push_back(msg);
}

/// Read messages until the stream ends or breaks its framing.
#[allow(dead_code)]
fn pump<R: Read>(
    stream: R,
    framing: Framing,
    inbox: &Mutex<Inbox>,
    wake: &(dyn Fn() + Send + Sync),
) {
    let mut reader = BufReader::with_capacity(64 * 1024, stream);
    let mut line = Vec::new();
    let limit = u64::try_from(framing.max_line)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let why = loop {
        line.clear();
        let n = match (&mut reader).take(limit).read_until(b'\n', &mut line) {
            Ok(n) => n,
            Err(e) => break format!("read failed: {e}"),
        };
        if n == 0 {
            break "the daemon closed the connection".to_owned();
        }
        if line.last() != Some(&b'\n') {
            break if line.len() > framing.max_line {
                format!("a line exceeded {} bytes", framing.max_line)
            } else {
                "the connection ended in the middle of a line".to_owned()
            };
        }
        line.pop();
        let Ok(text) = String::from_utf8(core::mem::take(&mut line)) else {
            break "a header line was not UTF-8".to_owned();
        };
        let len = match (framing.payload_len)(&text) {
            Ok(len) => len,
            Err(e) => break format!("protocol error: {e}"),
        };
        let msg = if len == 0 {
            Incoming::Line(text)
        } else {
            let mut bytes = vec![0u8; len];
            if let Err(e) = reader.read_exact(&mut bytes) {
                break format!("payload truncated: {e}");
            }
            Incoming::Payload {
                header: text,
                bytes,
            }
        };
        deliver(inbox, &framing, msg);
        wake();
    };
    deliver(inbox, &framing, Incoming::Closed(why));
    wake();
}

/// A connection to one daemon. The writer half exists only on Unix, where
/// the daemons live; on any other platform a `Link` can hold an inbox that
/// will only ever end.
#[derive(Debug)]
pub struct Link {
    #[cfg(unix)]
    writer: UnixStream,
    /// The writer half on hosts with no transport: never used, kept so the
    /// struct has the same shape and `Drop` reads the same everywhere.
    #[cfg(not(unix))]
    _writer: (),
    inbox: Arc<Mutex<Inbox>>,
    reader: Option<JoinHandle<()>>,
}

impl Link {
    /// Connect to a daemon's socket. Unix only: every daemon is reached through
    /// a Unix socket, so on any other platform there is nothing to connect to.
    ///
    /// # Errors
    /// Always [`io::ErrorKind::Unsupported`] on non-Unix hosts; any error from
    /// connecting, cloning the stream or starting the reader on Unix.
    #[cfg(unix)]
    pub fn connect(
        path: &Path,
        framing: Framing,
        wake: Box<dyn Fn() + Send + Sync>,
    ) -> io::Result<Self> {
        Self::over(UnixStream::connect(path)?, framing, wake)
    }

    /// The non-Unix form: there is no daemon transport, so every connect fails
    /// by name and nothing pretends otherwise.
    #[cfg(not(unix))]
    #[allow(missing_docs)]
    pub fn connect(
        _path: &Path,
        _framing: Framing,
        _wake: Box<dyn Fn() + Send + Sync>,
    ) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "daemons are reached through Unix sockets",
        ))
    }

    /// Start a link over a connected stream. Unix only.
    ///
    /// # Errors
    /// Any error from cloning the stream or starting the reader.
    #[cfg(unix)]
    #[allow(clippy::too_many_lines)]
    pub fn over(
        stream: UnixStream,
        framing: Framing,
        wake: Box<dyn Fn() + Send + Sync>,
    ) -> io::Result<Self> {
        // Commands are a few hundred bytes. If the daemon has stopped reading
        // for this long it is wedged, and the interface must not wedge with it.
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let reader = stream.try_clone()?;
        let inbox = Arc::new(Mutex::new(Inbox::default()));
        let shared = Arc::clone(&inbox);
        let handle = thread::Builder::new()
            .name("litric-shell-link".to_owned())
            .stack_size(256 * 1024)
            .spawn(move || pump(reader, framing, &shared, &*wake))?;
        Ok(Self {
            writer: stream,
            inbox,
            reader: Some(handle),
        })
    }

    /// Send one command line. A newline inside `line` is refused: it would
    /// split one command into two.
    ///
    /// # Errors
    /// `InvalidInput` for an embedded newline, or the write error.
    pub fn send_line(&mut self, line: &str) -> io::Result<()> {
        if line.contains('\n') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a command line may not contain a newline",
            ));
        }
        let mut buf = Vec::with_capacity(line.len().saturating_add(1));
        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
        self.write_line(&buf)
    }

    #[cfg(unix)]
    fn write_line(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)?;
        self.writer.flush()
    }

    #[cfg(not(unix))]
    fn write_line(&mut self, _buf: &[u8]) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no daemon transport on this platform",
        ))
    }

    /// Everything that has arrived since the last call, oldest first.
    #[must_use]
    pub fn drain(&self) -> Vec<Incoming> {
        lock(&self.inbox).queue.drain(..).collect()
    }

    /// Whether the connection has ended. Messages that arrived before the end
    /// are still returned by [`Self::drain`].
    #[must_use]
    pub fn is_closed(&self) -> bool {
        lock(&self.inbox).closed
    }

    /// The writer half, for callers that need to select on it. Unix only.
    #[cfg(unix)]
    #[must_use]
    pub fn writer(&self) -> &UnixStream {
        &self.writer
    }

    /// The writer half's placeholder on hosts with no daemon transport.
    #[cfg(not(unix))]
    #[must_use]
    pub const fn writer(&self) -> &() {
        &()
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        // Shutting the socket down makes the reader's blocking read return, so
        // the thread ends instead of outliving the window that owned it.
        #[cfg(unix)]
        {
            let _ = self.writer.shutdown(std::net::Shutdown::Both);
            if let Some(handle) = self.reader.take() {
                let _ = handle.join();
            }
        }
        #[cfg(not(unix))]
        {
            let _ = self.reader.take();
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    fn frame_len(header: &str) -> Result<usize, String> {
        match header.strip_prefix("FRAME ") {
            Some(rest) => {
                let n: usize = rest
                    .split(' ')
                    .nth(1)
                    .ok_or("no length")?
                    .parse()
                    .map_err(|_| "bad length".to_owned())?;
                if n > 1024 {
                    return Err("too large".to_owned());
                }
                Ok(n)
            }
            None => Ok(0),
        }
    }

    fn frame_key(header: &str) -> Option<String> {
        header
            .strip_prefix("FRAME ")
            .and_then(|r| r.split(' ').next())
            .map(str::to_owned)
    }

    const FRAMED: Framing = Framing {
        max_line: 64,
        payload_len: frame_len,
        coalesce_key: frame_key,
    };

    fn quiet() -> Box<dyn Fn() + Send + Sync> {
        Box::new(|| {})
    }

    fn wait_for(link: &Link, pred: impl Fn(&[Incoming]) -> bool) -> Vec<Incoming> {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(5))
            .expect("a deadline five seconds out");
        let mut seen = Vec::new();
        while Instant::now() < deadline {
            seen.extend(link.drain());
            if pred(&seen) {
                return seen;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out; saw {seen:?}");
    }

    fn closed(s: &[Incoming]) -> bool {
        s.iter().any(|m| matches!(m, Incoming::Closed(_)))
    }

    #[test]
    fn lines_and_payloads_arrive_in_order_and_each_wakes_the_loop() {
        let (ours, mut theirs) = UnixStream::pair().expect("pair");
        let wakes = Arc::new(AtomicUsize::new(0));
        let w = Arc::clone(&wakes);
        let link = Link::over(
            ours,
            FRAMED,
            Box::new(move || {
                w.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .expect("link");
        theirs
            .write_all(b"HELLO 1\nFRAME 7 3\nabcTAB 7 x\n")
            .expect("write");
        let got = wait_for(&link, |s| s.len() >= 3);
        assert_eq!(
            got,
            vec![
                Incoming::Line("HELLO 1".to_owned()),
                Incoming::Payload {
                    header: "FRAME 7 3".to_owned(),
                    bytes: b"abc".to_vec()
                },
                Incoming::Line("TAB 7 x".to_owned()),
            ]
        );
        assert!(wakes.load(Ordering::SeqCst) >= 3, "every message must wake");
        assert!(!link.is_closed());
    }

    #[test]
    fn the_last_frame_for_a_key_is_the_one_that_survives() {
        let (ours, mut theirs) = UnixStream::pair().expect("pair");
        let link = Link::over(ours, FRAMED, quiet()).expect("link");
        theirs
            .write_all(b"FRAME 1 2\naaFRAME 2 2\nbbFRAME 1 2\nccEND\n")
            .expect("write");
        let got = wait_for(&link, |s| s.contains(&Incoming::Line("END".to_owned())));
        let last_for_1 = got.iter().rev().find_map(|m| match m {
            Incoming::Payload { header, bytes } if header.starts_with("FRAME 1 ") => {
                Some(bytes.clone())
            }
            _ => None,
        });
        assert_eq!(last_for_1, Some(b"cc".to_vec()));
        assert!(got.iter().any(
            |m| matches!(m, Incoming::Payload { header, .. } if header.starts_with("FRAME 2 "))
        ));
    }

    #[test]
    fn coalescing_is_exact_when_nothing_is_drained_in_between() {
        // Bounded memory: frames produced faster than they are drawn must not
        // pile up, and different keys (tabs) are kept separately.
        let inbox = Mutex::new(Inbox::default());
        for (h, b) in [
            ("FRAME 1 1", b"a"),
            ("FRAME 2 1", b"b"),
            ("FRAME 1 1", b"c"),
        ] {
            deliver(
                &inbox,
                &FRAMED,
                Incoming::Payload {
                    header: h.to_owned(),
                    bytes: b.to_vec(),
                },
            );
        }
        deliver(&inbox, &FRAMED, Incoming::Line("TAB".to_owned()));
        let q: Vec<Incoming> = lock(&inbox).queue.drain(..).collect();
        assert_eq!(q.len(), 3, "{q:?}");
        assert_eq!(
            q.first(),
            Some(&Incoming::Payload {
                header: "FRAME 1 1".to_owned(),
                bytes: b"c".to_vec()
            }),
            "the newer frame takes the older one's place"
        );
    }

    #[test]
    fn eof_closes_the_link_after_delivering_what_came_before() {
        let (ours, mut theirs) = UnixStream::pair().expect("pair");
        let link = Link::over(ours, FRAMED, quiet()).expect("link");
        theirs.write_all(b"LAST\n").expect("write");
        drop(theirs);
        let got = wait_for(&link, closed);
        assert_eq!(got.first(), Some(&Incoming::Line("LAST".to_owned())));
        assert!(link.is_closed());
    }

    #[test]
    fn an_overlong_line_closes_the_link_instead_of_growing_memory() {
        let (ours, mut theirs) = UnixStream::pair().expect("pair");
        let link = Link::over(ours, FRAMED, quiet()).expect("link");
        theirs.write_all(&[b'x'; 200]).expect("write");
        let got = wait_for(&link, closed);
        assert!(
            matches!(got.last(), Some(Incoming::Closed(why)) if why.contains("exceeded")),
            "{got:?}"
        );
    }

    #[test]
    fn a_rejected_payload_header_closes_the_link() {
        let (ours, mut theirs) = UnixStream::pair().expect("pair");
        let link = Link::over(ours, FRAMED, quiet()).expect("link");
        theirs.write_all(b"FRAME 1 99999\n").expect("write");
        let got = wait_for(&link, closed);
        assert!(
            matches!(got.last(), Some(Incoming::Closed(why)) if why.contains("protocol error")),
            "{got:?}"
        );
    }

    #[test]
    fn a_truncated_payload_closes_the_link() {
        let (ours, mut theirs) = UnixStream::pair().expect("pair");
        let link = Link::over(ours, FRAMED, quiet()).expect("link");
        theirs.write_all(b"FRAME 1 10\nabc").expect("write");
        drop(theirs);
        let got = wait_for(&link, closed);
        assert!(
            matches!(got.last(), Some(Incoming::Closed(why)) if why.contains("truncated")),
            "{got:?}"
        );
    }

    #[test]
    fn commands_are_written_as_lines_and_embedded_newlines_are_refused() {
        let (ours, theirs) = UnixStream::pair().expect("pair");
        let mut link = Link::over(ours, FRAMED, quiet()).expect("link");
        link.send_line("GO 1 https%3A%2F%2Fexample.com")
            .expect("send");
        assert!(link.send_line("GO 1\nQUIT").is_err());
        let mut reader = BufReader::new(theirs);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read");
        assert_eq!(line, "GO 1 https%3A%2F%2Fexample.com\n");
    }

    #[test]
    fn dropping_the_link_ends_its_reader_thread() {
        let (ours, theirs) = UnixStream::pair().expect("pair");
        let link = Link::over(ours, FRAMED, quiet()).expect("link");
        let started = Instant::now();
        drop(link); // joins the reader; must not hang although `theirs` is open
        assert!(started.elapsed() < Duration::from_secs(5));
        drop(theirs);
    }
}
