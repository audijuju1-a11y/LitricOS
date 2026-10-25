//! End-to-end: a real, foreign Wayland client against this server.
//!
//! The client is not written by us. It is `weston-simple-shm`, the reference
//! `wl_shm` + `xdg-shell` client from the Weston tree: C, libwayland-client,
//! about thirty lines of protocol. If it can map a window here, the wire
//! format, the file-descriptor passing and the configure handshake are real,
//! not a round trip between two halves of the same Rust code.
//!
//! The test is skipped — loudly — when no client binary is available, because a
//! test that pretends to pass without its subject is worse than no test:
//!
//! ```text
//! LITRIC_WAYLAND_CLIENT=/path/to/weston-simple-shm cargo test -p litric-compositor
//! ```
//!
//! The binary needs `libwayland-client.so.0`; `LD_LIBRARY_PATH` is inherited.

// A test's job is to stop at the first surprise and say what it saw. That is
// a panic, on purpose, so the workspace's never-unwrap and no-indexing rules
// are waived here the same way the other integration tests waive them.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use litric_compositor::{Event, Server};

fn scratch(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir =
        std::env::temp_dir().join(format!("litric-wl-{}-{}-{nanos}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    }
    dir
}

fn client_path() -> Option<PathBuf> {
    let path = std::env::var_os("LITRIC_WAYLAND_CLIENT")?;
    let path = PathBuf::from(path);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_client(binary: &Path, runtime: &Path) -> KillOnDrop {
    let log = runtime.join("client.log");
    let out = std::fs::File::create(&log).expect("client log");
    let err = out.try_clone().expect("client log clone");
    let child = Command::new(binary)
        .env("XDG_RUNTIME_DIR", runtime)
        .env("WAYLAND_DISPLAY", "wayland-0")
        .env_remove("DISPLAY")
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .spawn()
        .expect("the Wayland client starts");
    KillOnDrop(child)
}

/// Events already drained but not yet matched.
///
/// A single `drain` returns every event that happened in that instant, and the
/// first matching one is rarely the only one that matters: `Mapped` and the
/// first `Frame` arrive together. Dropping the rest is how a test ends up
/// waiting five seconds for a frame that is already in hand.
#[derive(Default)]
struct Inbox {
    pending: std::collections::VecDeque<Event>,
}

impl Inbox {
    fn wait(
        &mut self,
        server: &Server,
        deadline: Duration,
        mut want: impl FnMut(&Event) -> bool,
    ) -> Option<Event> {
        let start = Instant::now();
        loop {
            if let Some(pos) = self.pending.iter().position(&mut want) {
                return self.pending.remove(pos);
            }
            if start.elapsed() >= deadline {
                return None;
            }
            for ev in server.drain() {
                self.pending.push_back(ev);
            }
            std::thread::sleep(Duration::from_millis(4));
        }
    }
}

#[test]
fn a_foreign_client_maps_a_window_and_goes_away_cleanly() {
    let Some(binary) = client_path() else {
        eprintln!(
            "SKIP: set LITRIC_WAYLAND_CLIENT to a wl_shm client (e.g. weston-simple-shm) \
             to run the foreign-client test"
        );
        return;
    };
    let runtime = scratch("shm");
    let server = Server::start(&runtime, None).expect("server starts");
    assert_eq!(server.display_name(), "wayland-0");
    assert!(server.socket_path().exists(), "the socket exists");

    let mut child = spawn_client(&binary, &runtime);

    let mut inbox = Inbox::default();
    let mapped = inbox
        .wait(&server, Duration::from_secs(10), |ev| {
            matches!(ev, Event::Mapped { .. })
        })
        .unwrap_or_else(|| {
            let log = std::fs::read_to_string(runtime.join("client.log")).unwrap_or_default();
            panic!("the client never mapped a window. client log:\n{log}");
        });
    let surface = match &mapped {
        Event::Mapped {
            surface,
            width,
            height,
            ..
        } => {
            assert!(*width > 0 && *height > 0, "a mapped surface has a size");
            *surface
        }
        _ => unreachable!(),
    };

    // The client paints a coloured gradient with a moving shape; a frame of
    // solid zeroes would mean we read the wrong bytes (or never read them).
    let got_frame = inbox.wait(
        &server,
        Duration::from_secs(5),
        |ev| matches!(ev, Event::Frame { surface: s } if *s == surface),
    );
    assert!(
        got_frame.is_some(),
        "the client commits a frame after mapping"
    );
    let frame = server
        .take_frame(surface)
        .expect("a committed frame is readable");
    assert_eq!(
        frame.pixels.len(),
        usize::try_from(
            u64::from(frame.width)
                .saturating_mul(u64::from(frame.height))
                .saturating_mul(4)
        )
        .expect("size")
    );
    // Sample a grid across the whole frame rather than its top-left corner:
    // the reference client fills the background with one colour and draws a
    // shape over part of it, so a corner sample can honestly be uniform.
    let colours = {
        let mut seen: Vec<u32> = Vec::new();
        let step_y = frame.height.checked_div(32).unwrap_or(1).max(1);
        let step_x = frame.width.checked_div(32).unwrap_or(1).max(1);
        let mut y = 0;
        while y < frame.height {
            let mut x = 0;
            while x < frame.width {
                let at = usize::try_from(
                    u64::from(y)
                        .saturating_mul(u64::from(frame.width))
                        .saturating_add(u64::from(x))
                        .saturating_mul(4),
                )
                .expect("pixel offset");
                let Some(px) = frame
                    .pixels
                    .get(at..at.saturating_add(4))
                    .and_then(|quad| quad.first_chunk::<4>())
                else {
                    break;
                };
                let word = u32::from_le_bytes(*px);
                if !seen.contains(&word) {
                    seen.push(word);
                }
                x += step_x;
            }
            y += step_y;
        }
        seen.len()
    };
    assert!(
        colours > 2,
        "the client's pixels arrived and contain a drawing (found {colours} distinct colours)"
    );
    assert!(
        frame
            .pixels
            .chunks_exact(4)
            .any(|px| px.first_chunk::<4>().is_some_and(|p| p[..3] != [0, 0, 0])),
        "the frame is not a block of zeroes"
    );

    // A frame callback is what lets the client animate. It is answered when the
    // desktop presents, so the client should keep committing after we do.
    server.presented(surface, 16);
    let next = inbox.wait(
        &server,
        Duration::from_secs(5),
        |ev| matches!(ev, Event::Frame { surface: s } if *s == surface),
    );
    assert!(
        next.is_some(),
        "the client draws again after its frame callback is answered"
    );

    // Resize: xdg_toplevel.configure must reach the client, and it must come
    // back with a buffer of the size we asked for.
    server.configure(surface, 320, 200);
    server.presented(surface, 32);
    let resized = inbox.wait(
        &server,
        Duration::from_secs(5),
        |ev| matches!(ev, Event::Frame { surface: s } if *s == surface),
    );
    assert!(resized.is_some(), "the client answers a configure");
    let frame = server.take_frame(surface).expect("the resized frame");
    assert_eq!(
        (frame.logical_width(), frame.logical_height()),
        (320, 200),
        "weston-simple-shm honours the size it was configured with"
    );

    // Input: the pointer enters, moves and clicks. The client prints the
    // events to its log, which is the only observable a reference client has.
    server.keyboard_focus(Some(surface));
    server.pointer_enter(surface, 10.0, 10.0);
    server.pointer_motion(surface, 40.0, 30.0);
    server.pointer_button(272, true);
    server.pointer_button(272, false);
    server.key(30, true, 1); // KEY_A
    server.key(30, false, 2);

    // A strict C client aborts the connection on a malformed event, so the
    // fact that it is still running and still drawing after the input is the
    // reference client's own statement that what we sent was valid.
    server.presented(surface, 48);
    let after_input = inbox.wait(
        &server,
        Duration::from_secs(5),
        |ev| matches!(ev, Event::Frame { surface: s } if *s == surface),
    );
    assert!(
        after_input.is_some() && child.0.try_wait().ok().flatten().is_none(),
        "the client survives the pointer and key events and keeps drawing"
    );

    // Close: the client must exit on its own when the window manager asks.
    server.close(surface);
    let start = Instant::now();
    let mut exited = false;
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(Some(_)) = child.0.try_wait() {
            exited = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let log = std::fs::read_to_string(runtime.join("client.log")).unwrap_or_default();
    assert!(
        exited,
        "the client exits when its toplevel is closed. client log:\n{log}"
    );

    drop(server);
    let _ = std::fs::remove_dir_all(&runtime);
}

#[test]
fn a_client_that_vanishes_does_not_take_the_server_with_it() {
    let Some(binary) = client_path() else {
        eprintln!("SKIP: no LITRIC_WAYLAND_CLIENT");
        return;
    };
    let runtime = scratch("vanish");
    let server = Server::start(&runtime, None).expect("server starts");
    let child = spawn_client(&binary, &runtime);
    let mut inbox = Inbox::default();
    let mapped = inbox.wait(&server, Duration::from_secs(10), |ev| {
        matches!(ev, Event::Mapped { .. })
    });
    assert!(mapped.is_some());
    let surface = match mapped {
        Some(Event::Mapped { surface, .. }) => surface,
        _ => 0,
    };
    drop(child); // SIGKILL
    let gone = inbox.wait(
        &server,
        Duration::from_secs(5),
        |ev| matches!(ev, Event::Gone { surface: s } if *s == surface),
    );
    assert!(gone.is_some(), "a killed client reports its surface gone");
    // The server keeps serving: a second client can still connect and map.
    let _second = spawn_client(&binary, &runtime);
    let again = inbox.wait(&server, Duration::from_secs(10), |ev| {
        matches!(ev, Event::Mapped { .. })
    });
    assert!(again.is_some(), "the compositor survives a client crash");
    drop(server);
    let _ = std::fs::remove_dir_all(&runtime);
}

#[test]
fn two_servers_refuse_the_same_socket() {
    let runtime = scratch("bind");
    let first = Server::start(&runtime, None).expect("first server");
    match Server::start(&runtime, None) {
        Ok(_) => panic!("a second compositor bound the same socket"),
        Err(e) => {
            let text = e.to_string();
            assert!(
                text.contains("wayland-0"),
                "the error names the socket: {text}"
            );
        }
    }
    drop(first);
    let _ = std::fs::remove_dir_all(&runtime);
}

#[test]
fn a_relative_runtime_directory_is_refused() {
    let err = Server::start(Path::new("relative/run"), None).expect_err("must refuse");
    let text = err.to_string();
    assert!(text.contains("XDG_RUNTIME_DIR"), "{text}");
}
