//! The client half: what the desktop and a script both use to talk to the queue.
//!
//! It lives in this crate so there is exactly one implementation of the wire
//! format. Two implementations of one protocol is how the browser's frame header
//! came to disagree with itself (`docs/audit/2026-10-26-browser-repair-audit.md`),
//! and a notification that a sender believes it queued and the desktop never sees
//! is the same defect with a friendlier consequence.
//!
//! # Timeouts
//!
//! Every call is bounded. The shell calls [`take`] from its event loop, and a
//! daemon that is wedged, being restarted or not started at all must cost the
//! desktop a bounded pause rather than a frozen screen. The bound is short — a
//! local Unix socket either answers in microseconds or is not there — and a
//! timeout is reported as an error the caller can decide to ignore, which is
//! exactly what the shell does with "no daemon yet".

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::proto::{field, Note, Pending, Priority};
use crate::server::notes_from_lines;

/// How long a call waits for the daemon before giving up.
///
/// A local socket round trip is microseconds; this is the budget for "the daemon
/// is alive but something is wrong", not for a slow network.
pub const TIMEOUT: Duration = Duration::from_millis(250);

/// What a post was answered with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ack {
    /// The queue's id for the notification.
    pub id: u32,
    /// Whether the queue reached the disk. `false` means the notification is
    /// queued but will not survive a reboot, and the caller may say so.
    pub persisted: bool,
}

/// What a take handed over.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Taken {
    /// The notifications, newest first.
    pub notes: Vec<Note>,
    /// Whether the now-shorter queue reached the disk.
    pub persisted: bool,
}

/// Send a request and return the raw reply, with a bounded wait.
///
/// # Errors
/// A connect, write or read failure, including the timeout. The message names
/// the socket so a user can tell "no daemon" from "wrong path".
pub fn request(socket: &Path, request: &str) -> Result<String, String> {
    request_within(socket, request, TIMEOUT)
}

/// [`request`] with an explicit budget, for a caller that knows better.
///
/// # Errors
/// As [`request`].
pub fn request_within(socket: &Path, request: &str, budget: Duration) -> Result<String, String> {
    let mut stream =
        UnixStream::connect(socket).map_err(|error| format!("{}: {error}", socket.display()))?;
    stream
        .set_read_timeout(Some(budget))
        .map_err(|error| format!("cannot set the read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(budget))
        .map_err(|error| format!("cannot set the write timeout: {error}"))?;
    stream
        .write_all(request.trim_end().as_bytes())
        .and_then(|()| stream.write_all(b"\n"))
        .and_then(|()| stream.flush())
        .map_err(|error| format!("cannot send to {}: {error}", socket.display()))?;

    let mut reader = BufReader::new(stream);
    let mut header = String::new();
    reader
        .read_line(&mut header)
        .map_err(|error| format!("cannot read from {}: {error}", socket.display()))?;
    if header.is_empty() {
        return Err(format!("{} closed without an answer", socket.display()));
    }
    let mut out = header.clone();
    if let Some(rest) = header.strip_prefix("OK ") {
        let count: usize = rest.trim().parse().unwrap_or(0);
        for _ in 0..count {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    return Err(format!(
                        "{} sent {count} lines but closed after {}",
                        socket.display(),
                        out.lines().count().saturating_sub(1)
                    ))
                }
                Ok(_) => out.push_str(&line),
                Err(error) => {
                    return Err(format!("cannot read from {}: {error}", socket.display()))
                }
            }
        }
    }
    Ok(out)
}

/// The reply's lines, without the header.
fn body(reply: &str) -> Vec<String> {
    reply.lines().skip(1).map(str::to_owned).collect()
}

/// Post a notification. Returns its id and whether it was persisted.
///
/// # Errors
/// A transport failure, or a refusal. A refusal is returned rather than
/// swallowed: a sender that is told "queued" while the queue rejected the
/// message is a sender that stops trying.
pub fn post(socket: &Path, pending: &Pending) -> Result<Ack, String> {
    let mut request_text = String::from("POST\n");
    request_text.push_str(&format!("app={}\n", pending.app));
    request_text.push_str(&format!("priority={}\n", pending.priority.token()));
    request_text.push_str(&format!("ttl_ms={}\n", pending.ttl_ms.unwrap_or(0)));
    request_text.push_str(&format!("title={}\n", pending.title));
    request_text.push_str(&format!("body={}\n", pending.body));
    request_text.push_str("END\n");

    let reply = request(socket, &request_text)?;
    if let Some(rest) = reply.strip_prefix("ERR ") {
        return Err(format!("refused: {}", rest.trim()));
    }
    let lines = body(&reply);
    let id = field(&lines, "id")
        .and_then(|raw| raw.parse::<u32>().ok())
        .ok_or_else(|| format!("the daemon answered without an id: {reply:?}"))?;
    Ok(Ack {
        id,
        persisted: field(&lines, "persisted") != Some("0"),
    })
}

/// Take the queue: everything queued is returned and removed.
///
/// # Errors
/// A transport failure or a malformed reply.
pub fn take(socket: &Path) -> Result<Taken, String> {
    let reply = request(socket, "TAKE")?;
    if let Some(rest) = reply.strip_prefix("ERR ") {
        return Err(format!("refused: {}", rest.trim()));
    }
    let lines = body(&reply);
    Ok(Taken {
        notes: notes_from_lines(&lines),
        persisted: field(&lines, "persisted") != Some("0"),
    })
}

/// Read the queue without removing anything.
///
/// # Errors
/// A transport failure or a malformed reply.
pub fn list(socket: &Path) -> Result<Vec<Note>, String> {
    let reply = request(socket, "LIST")?;
    if reply.starts_with("ERR ") {
        return Err(format!("refused: {}", reply.trim()));
    }
    Ok(notes_from_lines(&body(&reply)))
}

/// How many are queued.
///
/// # Errors
/// A transport failure or a reply without a count.
pub fn count(socket: &Path) -> Result<usize, String> {
    let reply = request(socket, "COUNT")?;
    field(&body(&reply), "count")
        .and_then(|raw| raw.trim().parse().ok())
        .ok_or_else(|| format!("the daemon answered without a count: {reply:?}"))
}

/// Post a notification with text arguments, for callers that have no [`Pending`]
/// to hand — the `--post` command line, principally.
///
/// # Errors
/// As [`post`].
pub fn post_text(
    socket: &Path,
    app: &str,
    title: &str,
    body: &str,
    priority: Priority,
    ttl_ms: Option<u64>,
) -> Result<Ack, String> {
    post(
        socket,
        &Pending {
            app: app.to_owned(),
            priority,
            title: title.to_owned(),
            body: body.to_owned(),
            ttl_ms,
        },
    )
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Instant;

    fn serve_once(dir: &str, state: crate::server::State) -> std::path::PathBuf {
        // A fresh socket path per test, in the temp directory.
        let path = std::env::temp_dir()
            .join(format!(
                "litric-notifyd-client-{}-{dir}",
                std::process::id()
            ))
            .join("notify.sock");
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
        let listener = crate::server::bind(&path).expect("binds");
        thread::spawn(move || {
            let _ = crate::server::serve(&listener, state);
        });
        path
    }

    fn pending(title: &str) -> Pending {
        Pending {
            app: "com.litric.test".to_owned(),
            priority: Priority::Normal,
            title: title.to_owned(),
            body: "body".to_owned(),
            ttl_ms: Some(60_000),
        }
    }

    #[test]
    fn a_post_and_a_take_round_trip_over_a_real_socket() {
        let path = serve_once(
            "roundtrip",
            crate::server::State::new(Default::default(), None),
        );
        let ack = post(&path, &pending("hello")).expect("posts");
        assert_eq!(ack.id, 1);
        assert!(ack.persisted, "no store means nothing to fail to write");

        let taken = take(&path).expect("takes");
        assert_eq!(taken.notes.len(), 1);
        assert_eq!(taken.notes[0].title, "hello");
        assert_eq!(taken.notes[0].app, "com.litric.test");

        let again = take(&path).expect("takes again");
        assert!(
            again.notes.is_empty(),
            "a delivered note is not redelivered"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn a_take_from_a_desktop_that_was_down_returns_what_was_posted() {
        // The promise, at the level the shell meets it: something posted while
        // nobody was listening is still there when the listener arrives.
        let path = serve_once(
            "offline",
            crate::server::State::new(Default::default(), None),
        );
        post(&path, &pending("posted while down")).expect("posts");
        let taken = take(&path).expect("takes");
        assert_eq!(taken.notes.len(), 1);
        assert_eq!(taken.notes[0].title, "posted while down");
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn a_count_and_a_list_do_not_consume_anything() {
        let path = serve_once("count", crate::server::State::new(Default::default(), None));
        post(&path, &pending("a")).expect("posts");
        post(&path, &pending("b")).expect("posts");
        assert_eq!(count(&path), Ok(2));
        assert_eq!(list(&path).map(|n| n.len()), Ok(2));
        assert_eq!(count(&path), Ok(2), "a list is not a take");
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn a_text_post_is_the_same_path_as_a_pending_one() {
        let path = serve_once("text", crate::server::State::new(Default::default(), None));
        let ack = post_text(
            &path,
            "com.litric.browser",
            "Download finished",
            "report.pdf is in Downloads",
            Priority::High,
            Some(8_000),
        )
        .expect("posts");
        assert_eq!(ack.id, 1);
        let taken = take(&path).expect("takes");
        assert_eq!(taken.notes[0].priority, Priority::High);
        assert!(
            taken.notes[0].expires_ms.is_some(),
            "an 8 s ttl becomes an expiry"
        );
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[test]
    fn a_missing_daemon_is_an_error_and_not_a_hang() {
        let path = std::env::temp_dir().join(format!(
            "litric-notifyd-absent-{}/notify.sock",
            std::process::id()
        ));
        let started = Instant::now();
        let result = take(&path);
        assert!(result.is_err());
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "a missing socket must fail immediately, took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn a_daemon_that_accepts_and_says_nothing_times_out_rather_than_freezing_the_desktop() {
        // The failure this budget exists for: a process that holds the socket
        // open and never writes. Without a timeout the compositor stops
        // drawing, which is the worst outcome a notification can cause.
        let path = std::env::temp_dir()
            .join(format!("litric-notifyd-silent-{}", std::process::id()))
            .join("notify.sock");
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
        let listener = crate::server::bind(&path).expect("binds");
        let held = thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                thread::sleep(Duration::from_millis(900));
                drop(stream);
            }
        });
        let started = Instant::now();
        let result = take(&path);
        let elapsed = started.elapsed();
        assert!(result.is_err(), "silence is not an empty queue");
        assert!(
            elapsed < Duration::from_millis(700),
            "the call took {elapsed:?}, which is longer than its budget"
        );
        held.join().expect("the held connection ended");
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }
}
