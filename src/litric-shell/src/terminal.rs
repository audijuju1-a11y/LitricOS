//! The process half of the Terminal: a persistent shell attached to a real
//! pseudo-terminal, with a live window size.
//!
//! Linux gives a PTY its window size through `TIOCSWINSZ` and tells the
//! foreground process with `SIGWINCH`. Both need `unsafe` or a small C helper,
//! and this crate forbids `unsafe`. The sibling `litric-pty.c` is that helper:
//! it allocates the PTY, starts `/bin/sh -i`, copies bytes in both directions,
//! and polls a 0600 size file written by Rust. When the numbers change it
//! applies the new size and signals the foreground process group.
//!
//! The size file is deliberately a file rather than a socket: it is easy to
//! inspect while debugging, the writer has no lifetime coupling to the helper,
//! and a malformed line simply leaves the previous size in place. The file is
//! removed when the session is dropped.
//!
//! # Why a prompt marker
//!
//! A PTY has no "command finished" event. The shell prints `PS1` when it is
//! ready for the next command, so `PS1` is set to a marker that cannot occur in
//! ordinary output. The reader turns that marker into [`Event::Ready`] and never
//! forwards it to the display. That is what lets the UI distinguish a running
//! command from an idle shell without guessing from output timing.
//!
//! # Threads
//!
//! One reader thread drains the PTY and one writer thread feeds it. Neither can
//! block the frame loop: the shell only sends bytes and drains an `mpsc`
//! channel. Dropping the session kills the helper and removes the size file, so
//! a window or a `New session` action cannot leak a shell.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// The prompt the shell prints when it is ready for another command.
pub const READY_MARKER: &str = "__LITRIC_READY__";

/// The initial PTY window; the UI may change it at any time with
/// [`Session::resize`].
pub const DEFAULT_COLS: u16 = 100;
/// The initial PTY height.
pub const DEFAULT_ROWS: u16 = 30;

/// One thing that happened to the session.
#[derive(Debug)]
pub enum Event {
    /// Bytes the shell printed. They may split a UTF-8 sequence or a control
    /// sequence; the display layer buffers both.
    Output(Vec<u8>),
    /// The shell is ready for another command.
    Ready,
    /// The shell exited, or its PTY closed.
    Closed,
}

/// A running shell under a pseudo-terminal.
#[derive(Debug)]
pub struct Session {
    child: Child,
    input: Sender<Vec<u8>>,
    output: Receiver<Event>,
    size_path: PathBuf,
    closed: bool,
}

impl Session {
    /// Start `/bin/sh -i` under a PTY and return a handle to it.
    ///
    /// `envs` is the desktop environment an application needs
    /// (`WAYLAND_DISPLAY`, `XDG_RUNTIME_DIR`); the shell inherits it so a GUI
    /// program started from the Terminal can reach the compositor.
    pub fn spawn(envs: Vec<(String, String)>) -> Result<Self, String> {
        let size_path = make_size_path()?;
        if let Err(error) = write_size(&size_path, DEFAULT_COLS, DEFAULT_ROWS) {
            remove_size_file(&size_path);
            return Err(format!("cannot write the PTY size file: {error}"));
        }

        let helper = helper_path();
        let mut command = Command::new(&helper);
        command
            .arg(&size_path)
            .env("TERM", "xterm-256color")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for (key, value) in envs {
            command.env(key, value);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                remove_size_file(&size_path);
                return Err(format!(
                    "cannot start the PTY helper {}: {error}",
                    helper.display()
                ));
            }
        };
        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                remove_size_file(&size_path);
                return Err("the pseudo-terminal has no input pipe".to_owned());
            }
        };
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                remove_size_file(&size_path);
                return Err("the pseudo-terminal has no output pipe".to_owned());
            }
        };

        let (input_tx, input_rx) = mpsc::channel::<Vec<u8>>();
        let (output_tx, output_rx) = mpsc::channel::<Event>();

        let writer = thread::Builder::new()
            .name("litric-terminal-writer".to_owned())
            .spawn(move || writer(stdin, input_rx));
        if writer.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            remove_size_file(&size_path);
            return Err("cannot start the terminal writer thread".to_owned());
        }
        let reader = thread::Builder::new()
            .name("litric-terminal-reader".to_owned())
            .spawn(move || reader(stdout, output_tx));
        if reader.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            remove_size_file(&size_path);
            return Err("cannot start the terminal reader thread".to_owned());
        }

        Ok(Self {
            child,
            input: input_tx,
            output: output_rx,
            size_path,
            closed: false,
        })
    }

    /// Send bytes to the shell's stdin. Returns false once the session is gone.
    #[must_use]
    pub fn send(&self, bytes: &[u8]) -> bool {
        !self.closed && self.input.send(bytes.to_vec()).is_ok()
    }

    /// Set the visible PTY size. The helper polls this file every ~100 ms,
    /// applies `TIOCSWINSZ`, and signals the foreground process group.
    ///
    /// Returns false if the session is closed or the file can no longer be
    /// written. Sizes outside the supported range are clamped rather than
    /// rejected, matching what a window manager does with a tiny window.
    #[must_use]
    pub fn resize(&mut self, cols: u16, rows: u16) -> bool {
        if self.closed {
            return false;
        }
        let cols = cols.clamp(20, 500);
        let rows = rows.clamp(5, 300);
        write_size(&self.size_path, cols, rows).is_ok()
    }

    /// The size file, for diagnostics.
    #[must_use]
    pub fn size_path(&self) -> &Path {
        &self.size_path
    }

    /// Everything the session produced since the last call. Never blocks.
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        loop {
            match self.output.try_recv() {
                Ok(Event::Closed) => {
                    if !self.closed {
                        self.closed = true;
                        events.push(Event::Closed);
                    }
                    break;
                }
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if !self.closed {
                        self.closed = true;
                        events.push(Event::Closed);
                    }
                    break;
                }
            }
        }
        events
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Dropping the sender also gives the writer an EOF, but the helper is
        // what has to stop: a shell waiting for input would otherwise outlive
        // the window that owned it.
        let _ = self.child.kill();
        let _ = self.child.wait();
        remove_size_file(&self.size_path);
    }
}

fn writer(mut stdin: impl Write, input: Receiver<Vec<u8>>) {
    for bytes in input {
        if stdin.write_all(&bytes).is_err() || stdin.flush().is_err() {
            break;
        }
    }
}

/// A unique 0600 file in the temporary directory.
fn make_size_path() -> Result<PathBuf, String> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    for attempt in 0..100 {
        let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            "litric-pty-{}-{nonce}-{sequence}-{attempt}.size",
            std::process::id()
        );
        let path = std::env::temp_dir().join(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
        {
            Ok(mut file) => {
                if let Err(error) =
                    file.write_all(format!("{DEFAULT_COLS} {DEFAULT_ROWS}\n").as_bytes())
                {
                    remove_size_file(&path);
                    return Err(error.to_string());
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("cannot create a unique PTY size file".to_owned())
}

/// Remove the session's own size file.
///
/// The only deletion this module performs. `path` is produced by
/// [`make_size_path`] from the process id, a nonce and a counter, is created
/// with mode 0600, and is never supplied by a user or a remote caller.
fn remove_size_file(path: &Path) {
    let _ = fs::remove_file(path);
}

fn write_size(path: &Path, cols: u16, rows: u16) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(format!("{cols} {rows}\n").as_bytes())?;
    file.flush()
}

/// The helper to execute.
///
/// Tests compile the sibling C source and use that binary, so `cargo test`
/// proves the resize path instead of testing a stale installed helper. The
/// image build installs `/usr/bin/litric-pty`; a source checkout can put it on
/// `PATH`.
fn helper_path() -> PathBuf {
    #[cfg(test)]
    {
        test_helper_path()
    }
    #[cfg(not(test))]
    {
        if let Some(path) = std::env::var_os("LITRIC_PTY_HELPER") {
            return PathBuf::from(path);
        }
        for candidate in ["/usr/bin/litric-pty", "/usr/local/bin/litric-pty"] {
            if Path::new(candidate).is_file() {
                return PathBuf::from(candidate);
            }
        }
        PathBuf::from("litric-pty")
    }
}

#[cfg(test)]
fn test_helper_path() -> PathBuf {
    use std::sync::OnceLock;
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let target = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
                || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target"),
                PathBuf::from,
            );
            let output = target.join("debug").join("litric-pty");
            let source = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("litric-pty.c");
            let compiled = Command::new("cc")
                .args(["-std=c17", "-O2", "-Wall", "-Wextra", "-Werror"])
                .arg(&source)
                .args(["-lutil", "-o"])
                .arg(&output)
                .output()
                .unwrap_or_else(|error| panic!("cannot run cc for {}: {error}", source.display()));
            assert!(
                compiled.status.success(),
                "cc failed for {}: {}",
                source.display(),
                String::from_utf8_lossy(&compiled.stderr)
            );
            output
        })
        .clone()
}

/// Length of the longest suffix of `haystack` that is a prefix of `needle`.
///
/// This is the only part of a read that might still turn into a prompt marker
/// after the next read. Everything before it is safe to show now.
fn marker_prefix_suffix(haystack: &[u8], needle: &[u8]) -> usize {
    let max = haystack.len().min(needle.len().saturating_sub(1));
    for length in (1..=max).rev() {
        if let Some(prefix) = needle.get(..length) {
            if haystack.ends_with(prefix) {
                return length;
            }
        }
    }
    0
}

/// Read the PTY, split out the prompt marker, and forward everything else.
///
/// The marker can straddle a read boundary, so the last `len - 1` bytes are
/// kept until the next read can decide whether they are a prefix of it.
fn reader(mut stdout: impl Read, output: Sender<Event>) {
    let marker = READY_MARKER.as_bytes();
    let mut pending = Vec::new();
    let mut buf = [0u8; 4096];

    loop {
        let count = match stdout.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        pending.extend_from_slice(buf.get(..count).unwrap_or(&[]));

        loop {
            match find(&pending, marker) {
                Some(position) => {
                    if position > 0
                        && output
                            .send(Event::Output(
                                pending.get(..position).unwrap_or(&[]).to_vec(),
                            ))
                            .is_err()
                    {
                        return;
                    }
                    if output.send(Event::Ready).is_err() {
                        return;
                    }
                    pending.drain(..position.saturating_add(marker.len()));
                }
                None => {
                    // Hold only a suffix that could still be the beginning of
                    // the marker. Holding a fixed 16 bytes delayed a short
                    // line by a whole second when the next prompt was a second
                    // away, which made streaming output look buffered.
                    let keep = marker_prefix_suffix(&pending, marker);
                    let cut = pending.len().saturating_sub(keep);
                    if cut > 0 {
                        if output
                            .send(Event::Output(pending.get(..cut).unwrap_or(&[]).to_vec()))
                            .is_err()
                        {
                            return;
                        }
                        pending.drain(..cut);
                    }
                    break;
                }
            }
        }
    }

    if !pending.is_empty() {
        let _ = output.send(Event::Output(pending));
    }
    let _ = output.send(Event::Closed);
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::TerminalApp;
    use std::time::{Duration, Instant};

    fn collect_until(session: &mut Session, needle: &str, timeout: Duration) -> String {
        let mut text = String::new();
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            for event in session.drain() {
                if let Event::Output(bytes) = event {
                    text.push_str(&String::from_utf8_lossy(&bytes));
                }
            }
            if text.contains(needle) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        text
    }

    #[test]
    fn a_pty_session_keeps_shell_state_between_commands() {
        let mut session = Session::spawn(Vec::new()).expect("start a PTY shell");
        wait_for_ready(&mut session);
        assert!(session.send(b"cd /tmp\n"));
        assert!(session.send(b"pwd\n"));

        let text = collect_until(&mut session, "/tmp", Duration::from_secs(10));
        assert!(text.contains("/tmp"), "output was {text:?}");
        assert!(session.send(b"exit\n"));
    }

    #[test]
    fn a_pty_session_stops_a_command_with_ctrl_c() {
        let mut session = Session::spawn(Vec::new()).expect("start a PTY shell");
        wait_for_ready(&mut session);
        // Do not guess how long the shell needs to start a child. Print a
        // marker first and wait for it; otherwise a loaded test host can send
        // Ctrl+C while the shell is still setting up the command, which tests
        // scheduling rather than the interrupt path.
        assert!(session.send(b"printf 'CTRL-C-READY\\n'; sleep 10\n"));
        let started = collect_until(&mut session, "CTRL-C-READY", Duration::from_secs(5));
        assert!(
            started.contains("CTRL-C-READY"),
            "command did not start: {started:?}"
        );
        assert!(session.send(b"\x03"));

        let deadline = Instant::now()
            .checked_add(Duration::from_secs(10))
            .unwrap_or_else(Instant::now);
        let mut ready = false;
        while Instant::now() < deadline {
            for event in session.drain() {
                if matches!(event, Event::Ready) {
                    ready = true;
                }
            }
            if ready {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(ready, "Ctrl+C did not return the shell to its prompt");
        assert!(session.send(b"exit\n"));
    }

    #[test]
    fn resizing_the_window_changes_what_stty_size_reports() {
        let mut session = Session::spawn(Vec::new()).expect("start a PTY shell");
        wait_for_ready(&mut session);
        assert!(session.send(b"stty size\n"));
        let before = collect_until(&mut session, "30 100", Duration::from_secs(5));
        assert!(before.contains("30 100"), "before resize: {before:?}");

        assert!(session.resize(120, 40));
        // The helper polls the size file every 100 ms and then signals the
        // foreground process group. Give it one poll interval before measuring.
        thread::sleep(Duration::from_millis(300));
        assert!(session.send(b"stty size\n"));
        let after = collect_until(&mut session, "40 120", Duration::from_secs(5));
        assert!(after.contains("40 120"), "after resize: {after:?}");
        assert!(
            !after.contains("30 100"),
            "the old size survived: {after:?}"
        );
        assert!(session.send(b"exit\n"));
    }

    #[test]
    fn a_real_vim_session_styles_cells_and_restores_the_primary_screen() {
        if Command::new("vim").arg("--version").output().is_err() {
            eprintln!("vim is absent; the real alternate-screen test cannot run");
            return;
        }
        let mut session = Session::spawn(Vec::new()).expect("start a PTY shell");
        wait_for_ready(&mut session);
        // A source file gives vim syntax highlighting, so the parser sees real
        // SGR sequences rather than only the default status-line attributes.
        let source = format!("{}/src/terminal_screen.rs", env!("CARGO_MANIFEST_DIR"));
        let command = format!("vim -u NONE -N -n -c 'set t_Co=256' -c 'syntax on' {source}\n");
        assert!(session.send(command.as_bytes()));
        let entered = collect_until(&mut session, "\x1b[?1049h", Duration::from_secs(10));
        assert!(
            entered.contains("\x1b[?1049h"),
            "vim did not enter the alternate screen: {entered:?}"
        );

        let mut term = TerminalApp::new();
        term.set_viewport(100, 30);
        term.push_output(entered.as_bytes());
        assert!(
            wait_for_coloured_screen(&mut session, &mut term, Duration::from_secs(10)),
            "vim produced no coloured cell in the alternate grid"
        );

        // Normal-mode `:q!` reaches vim as raw bytes, then it restores the
        // primary screen. This is the real program, not a string fixture.
        assert!(session.send(b":q!\r"));
        let left = collect_until(&mut session, "\x1b[?1049l", Duration::from_secs(10));
        assert!(
            left.contains("\x1b[?1049l"),
            "vim did not restore the primary screen: {left:?}"
        );
        term.push_output(left.as_bytes());
        assert!(
            !term.alt_active(),
            "the parser did not leave vim's alternate screen"
        );
        assert!(session.send(b"echo restored-after-vim\n"));
        let restored = collect_until(&mut session, "restored-after-vim", Duration::from_secs(5));
        assert!(restored.contains("restored-after-vim"));
        assert!(session.send(b"exit\n"));
    }

    #[test]
    fn a_raw_fixture_receives_arrow_bytes_and_restores_the_screen() {
        let mut session = Session::spawn(Vec::new()).expect("start a PTY shell");
        wait_for_ready(&mut session);
        // The fixture is a shell command, not an installed editor: it proves
        // bytes reach a program in raw mode and that both sides of the
        // alternate screen survive a real PTY.
        assert!(session.send(
            b"printf '\\033[?1049h\\033[2J\\033[HRAW-SCREEN\\033[?25l'; stty raw -echo; dd bs=1 count=3 2>/dev/null | od -An -tx1; stty sane; printf '\\033[?25h\\033[?1049lFIXTURE-DONE\\n'\n"
        ));
        let raw = collect_until(&mut session, "RAW-SCREEN", Duration::from_secs(5));
        assert!(
            raw.contains("\x1b[?1049h"),
            "fixture did not enter alt: {raw:?}"
        );
        assert!(session.send(b"\x1b[A"));
        let done = collect_until(&mut session, "FIXTURE-DONE", Duration::from_secs(5));
        assert!(
            done.contains("1b 5b 41"),
            "arrow bytes did not reach dd: {done:?}"
        );
        assert!(
            done.contains("\x1b[?1049l"),
            "fixture did not leave alt: {done:?}"
        );
        // `collect_until` may consume the prompt marker together with the
        // fixture's last line; prove the shell is still usable with a real
        // command instead of waiting for a marker already drained.
        assert!(session.send(b"echo after-fixture\n"));
        let after = collect_until(&mut session, "after-fixture", Duration::from_secs(5));
        assert!(
            after.contains("after-fixture"),
            "session did not recover: {after:?}"
        );
        assert!(session.send(b"exit\n"));
    }

    fn screen_has_colour(term: &TerminalApp) -> bool {
        let Some(screen) = term.screen() else {
            return false;
        };
        for row in 0..screen.rows() {
            for col in 0..screen.cols() {
                if screen.cell_at(row, col).is_some_and(|cell| {
                    cell.style.fg != crate::terminal_screen::Color::Default
                        || cell.style.bg != crate::terminal_screen::Color::Default
                }) {
                    return true;
                }
            }
        }
        false
    }

    fn wait_for_coloured_screen(
        session: &mut Session,
        term: &mut TerminalApp,
        timeout: Duration,
    ) -> bool {
        let deadline = Instant::now()
            .checked_add(timeout)
            .unwrap_or_else(Instant::now);
        while Instant::now() < deadline {
            for event in session.drain() {
                if let Event::Output(bytes) = event {
                    term.push_output(&bytes);
                }
            }
            if screen_has_colour(term) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn a_real_pty_fixture_delivers_sgr_and_wide_cells_to_the_grid() {
        use crate::terminal_screen::Color;

        let mut session = Session::spawn(Vec::new()).expect("start a PTY shell");
        wait_for_ready(&mut session);
        let command = b"printf '\\033[?1049h\\033[2J\\033[H\\033[31;1;4mR\\033[0m\\033[38;5;196mG\\033[38;2;1;2;3mT\\033[0m\\347\\225\\214A\\340\\270\\201\\340\\270\\261READY\\033[?25l'; read x; printf '\\033[?25h\\033[?1049lFIXTURE-STYLED-DONE\\n'\n";
        assert!(session.send(command));
        let ready = collect_until(&mut session, "READY", Duration::from_secs(5));
        assert!(
            ready.contains("READY"),
            "fixture did not draw its screen: {ready:?}"
        );

        let mut term = TerminalApp::new();
        term.set_viewport(30, 8);
        term.push_output(ready.as_bytes());
        let screen = term
            .screen()
            .expect("fixture did not enter alternate screen");
        let cell = |col: usize| screen.cell_at(0, col).expect("fixture cell");
        assert_eq!(cell(0).text.as_str(), "R");
        assert_eq!(cell(0).style.fg, Color::Indexed(1));
        assert!(cell(0).style.attrs.bold && cell(0).style.attrs.underline);
        assert_eq!(cell(1).text.as_str(), "G");
        assert_eq!(cell(1).style.fg, Color::Indexed(196));
        assert_eq!(cell(2).text.as_str(), "T");
        assert_eq!(cell(2).style.fg, Color::Rgb(1, 2, 3));
        assert_eq!(cell(3).text.as_str(), "界");
        assert_eq!(cell(3).width, 2);
        assert_eq!(cell(4).width, 0);
        assert_eq!(cell(5).text.as_str(), "A");
        assert_eq!(cell(6).text.as_str(), "กั");
        assert_eq!(cell(6).width, 1);
        assert_eq!(screen.cursor_col, 12);

        assert!(session.send(b"\n"));
        let done = collect_until(&mut session, "FIXTURE-STYLED-DONE", Duration::from_secs(5));
        assert!(
            done.contains("FIXTURE-STYLED-DONE"),
            "fixture did not finish: {done:?}"
        );
        term.push_output(done.as_bytes());
        assert!(
            !term.alt_active(),
            "fixture did not restore the primary screen"
        );
        assert!(session.send(b"exit\n"));
    }

    fn wait_for_ready(session: &mut Session) {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(5))
            .unwrap_or_else(Instant::now);
        let mut text = String::new();
        while Instant::now() < deadline {
            let mut ready = false;
            for event in session.drain() {
                match event {
                    Event::Output(bytes) => text.push_str(&String::from_utf8_lossy(&bytes)),
                    Event::Ready => ready = true,
                    Event::Closed => {
                        panic!("the PTY closed before its readiness marker; output {text:?}")
                    }
                }
            }
            if ready {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("the shell never printed its readiness marker; output {text:?}");
    }
}
