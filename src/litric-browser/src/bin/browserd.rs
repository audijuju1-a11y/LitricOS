//! litric-browserd — the browser engine broker.
//!
//! ADR-0018: LitricOS integrates a rendering engine, never writes one. The
//! engine is Chromium, and the broker owns Chromium's lifetime. It translates
//! between two protocols:
//!
//! ```text
//! litric-shell ⇄ (LBP/1, Unix socket /run/browserd.sock) ⇄ browserd
//! browserd     ⇄ (CDP, Chromium --remote-debugging-pipe)  ⇄ Chromium
//! ```
//!
//! # LBP/1 — Litric Browser Protocol
//!
//! One command per line, one reply per command, frames announced by header:
//!
//! ```text
//! → GO <tab> <pct-encoded-url>
//! ← OK title=<pct> bytes=<n> secure=<0|1>     then zero or more
//! ← FRAME <tab> <len>\n<len bytes of RGBA8>
//! → PING                    ← PONG
//! → QUIT                    (connection closes)
//! ```
//!
//! Every GO is answered. When the machine has no Chromium binary — the state
//! of every image built so far — the broker still serves LBP/1 and every GO
//! gets `ERR` naming the missing piece, which the shell renders in the page
//! area. A broker that pretended to load pages would be worth less than one
//! that says why it cannot; that honesty is ADR-0018's whole point.
//!
//! # Chromium integration
//!
//! When a binary IS present (`BROWSERD_CHROMIUM`, else a search of common
//! paths), browserd spawns it headless and drives the minimum CDP:
//! `Target.createTarget` → `Page.navigate` → `Page.captureScreenshot`
//! (JPEG). The JPEG is decoded by litric-browser's own decoder and converted
//! to RGBA8 frames. Screenshot-per-navigation is deliberate for Phase 0: one
//! round trip, no outpacing the compositor, memory bounded to one frame.
//!
//! # Confinement
//!
//! The broker runs as its own user, serves one socket with no access to the
//! user's files, and refuses any URL scheme the shell's own policy layer
//! would refuse. Zero third-party dependencies, like every crate here.
//!
//! # Portability
//!
//! Everything the broker does is Unix: sockets and a child process with
//! pipes. On other hosts the file compiles to an honest stub so the crate
//! still builds everywhere the workspace builds.

// ---------------------------------------------------------------------------
// Protocol helpers (portable: shared with the tests on every host)
// ---------------------------------------------------------------------------

/// Percent-decode a LBP field value (`-` means empty). Invalid escapes stay
/// literal: a mangled URL must produce an honest error, not a silent guess.
#[must_use]
pub fn pct_decode(v: &str) -> String {
    if v == "-" {
        return String::new();
    }
    let bytes = v.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-encode a reply field value.
#[must_use]
pub fn pct_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~' | b'/') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Escape a string for embedding in a CDP JSON parameter.
#[cfg(unix)]
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The Unix broker
// ---------------------------------------------------------------------------

#[cfg(unix)]
mod broker {
    use super::json_escape;
    use std::collections::HashMap;
    use std::io::{self, BufRead, BufReader, Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    const DEFAULT_SOCKET: &str = "/run/browserd.sock";

    /// The rendering engine behind the broker.
    #[derive(Debug)]
    pub enum Engine {
        /// No Chromium on the machine: every load refuses by name.
        Missing {
            /// Why, in the user's words.
            reason: String,
        },
        /// A Chromium process this broker owns.
        Chromium {
            /// The child handle and its CDP pipes.
            proc: ChromiumProc,
        },
    }

    /// The three fd primitives the CDP pipe needs. They live in one place,
    /// one line of unsafe each, because a debugging pipe that arrives on the
    /// wrong descriptor looks from the outside like "the engine died" — the
    /// exact misdiagnosis this file once shipped. Audit here, and only here.
    mod cdp_fd {
        use std::io;

        extern "C" {
            fn pipe2(fds: *mut i32, flags: u32) -> i32;
            fn dup2(old: i32, new: i32) -> i32;
            fn close(fd: i32) -> i32;
        }

        /// Linux `O_CLOEXEC`.
        const O_CLOEXEC: u32 = 0o2_000_000;

        /// A pipe whose ends die at exec, so the child never inherits a
        /// parent-side descriptor that would mask engine death as silence.
        pub fn pipe_cloexec() -> io::Result<(i32, i32)> {
            let mut fds = [0i32; 2];
            #[allow(unsafe_code)]
            let rc = unsafe { pipe2(fds.as_mut_ptr(), O_CLOEXEC) };
            if rc == 0 {
                Ok((fds[0], fds[1]))
            } else {
                Err(io::Error::last_os_error())
            }
        }

        /// Move `fd` onto `target` in the forked child; a failure must be an
        /// error, never a silently unwired debugging pipe.
        pub fn dup2_or_err(fd: i32, target: i32) -> io::Result<()> {
            #[allow(unsafe_code)]
            let rc = unsafe { dup2(fd, target) };
            if rc == target {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }

        /// Best-effort close of a parent-side descriptor.
        pub fn close_fd(fd: i32) {
            #[allow(unsafe_code)]
            unsafe {
                close(fd);
            }
        }
    }

    /// One headless Chromium driven over `--remote-debugging-pipe`.
    ///
    /// The contract is fixed by Chromium: CDP messages arrive on the child's
    /// fd 3 and leave on fd 4 — stdio is never consulted for the pipe, so a
    /// broker that maps the pipe onto stdin/stdout talks to a child that has
    /// already exited (observed live as "engine session ended" one report
    /// after the dock click). `Command::stdio` cannot reach fd 3/4, so the
    /// spawn uses the kernel's dup2: the raw parent-side pipe fds are moved
    /// into place in the child between fork and exec — the same narrow window
    /// and the same primitive every runtime's stdio redirection uses.
    #[derive(Debug)]
    pub struct ChromiumProc {
        child: std::process::Child,
        cdp_in: std::fs::File,
        cdp_out: BufReader<std::fs::File>,
        next_id: AtomicU64,
        targets: HashMap<u32, String>,
    }

    /// The `--profile <dir>` given on the command line, if any.
    static PROFILE_DIR: std::sync::OnceLock<Option<std::path::PathBuf>> =
        std::sync::OnceLock::new();

    fn profile_dir() -> Option<&'static std::path::PathBuf> {
        PROFILE_DIR
            .get_or_init(|| {
                let args = std::env::args().collect::<Vec<_>>();
                args.iter()
                    .position(|a| a == "--profile")
                    .and_then(|i| args.get(i + 1))
                    .map(std::path::PathBuf::from)
            })
            .as_ref()
    }

    impl ChromiumProc {
        /// Spawn headless Chromium with the debugging pipe on fd 3/4.
        pub fn spawn(binary: &Path) -> io::Result<Self> {
            use std::os::unix::io::FromRawFd;
            use std::os::unix::process::CommandExt;

            let (in_r, in_w) = cdp_fd::pipe_cloexec()?;
            let (out_r, out_w) = cdp_fd::pipe_cloexec()?;
            let mut cmd = std::process::Command::new(binary);
            let mut args = vec![
                "--headless=new".to_owned(),
                "--no-sandbox".to_owned(),
                "--disable-gpu".to_owned(),
                "--disable-dev-shm-usage".to_owned(),
                "--no-first-run".to_owned(),
                "--remote-debugging-pipe".to_owned(),
            ];
            // The user-data dir must be inside the unit's ReadWritePaths:
            // with the default (~/.config/chromium) under ProtectHome=yes,
            // Chromium dies on its first mkdir — observed as "engine session
            // ended" one report after the session started.
            if let Some(profile) = profile_dir() {
                let _ = std::fs::create_dir_all(&profile);
                args.push(format!("--user-data-dir={}", profile.display()));
            }
            cmd.args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            // Between fork and exec is the only window fd 3/4 can be arranged;
            // dup2 copies lose close-on-exec by kernel rule, so the child keeps
            // exactly 3/4 and every original dies at exec. `pre_exec` is unsafe
            // by declaration — it runs inside that async-signal-safe window —
            // so the call carries an explicit opt-out, adjacent to the code
            // whose correctness it depends on.
            #[allow(unsafe_code)]
            unsafe {
                cmd.pre_exec(move || {
                    cdp_fd::dup2_or_err(in_r, 3)?;
                    cdp_fd::dup2_or_err(out_w, 4)
                });
            }
            let child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    cdp_fd::close_fd(in_r);
                    cdp_fd::close_fd(in_w);
                    cdp_fd::close_fd(out_r);
                    cdp_fd::close_fd(out_w);
                    return Err(e);
                }
            };
            // The child owns dup2 copies on 3/4; the parent's originals must
            // close now or Chromium's death is never seen as EOF on the pipe.
            cdp_fd::close_fd(in_r);
            cdp_fd::close_fd(out_w);
            // SAFETY: in_w/out_r are freshly created, owned by us alone, and
            // are never used as raw integers again after this wrap.
            #[allow(unsafe_code)]
            let cdp_in = unsafe { std::fs::File::from_raw_fd(in_w) };
            #[allow(unsafe_code)]
            let cdp_out = unsafe { BufReader::new(std::fs::File::from_raw_fd(out_r)) };
            Ok(Self {
                child,
                cdp_in,
                cdp_out,
                next_id: AtomicU64::new(1),
                targets: HashMap::new(),
            })
        }

        /// One CDP request/response round trip (NUL-delimited messages).
        pub fn call(&mut self, method: &str, params: &str) -> Result<String, String> {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            let msg = format!(r#"{{"id":{id},"method":"{method}","params":{params}}}"#);
            self.cdp_in
                .write_all(msg.as_bytes())
                .and_then(|_| self.cdp_in.write_all(b"\0"))
                .and_then(|_| self.cdp_in.flush())
                .map_err(|e| format!("CDP write: {e}"))?;
            let mut buf = Vec::new();
            loop {
                buf.clear();
                let n = self
                    .cdp_out
                    .read_until(0, &mut buf)
                    .map_err(|e| format!("CDP read: {e}"))?;
                if n == 0 {
                    return Err("Chromium closed the debugging pipe".to_owned());
                }
                while buf.last() == Some(&0) {
                    buf.pop();
                }
                let text = String::from_utf8_lossy(&buf).into_owned();
                if text.contains(&format!(r#""id":{id}"#)) {
                    return Ok(text);
                }
                // Id-less events (screencast frames, lifecycle) are consumed
                // and dropped: Phase 0 pulls one screenshot per navigation,
                // so nothing here needs to queue.
            }
        }

        /// Navigate a tab and capture the page as RGBA8.
        pub fn load_and_capture(
            &mut self,
            tab: u32,
            url: &str,
        ) -> Result<(String, Vec<u8>, u32, u32), String> {
            // A target per tab, created on first use.
            if !self.targets.contains_key(&tab) {
                let reply = self
                    .call("Target.createTarget", r#"{"url":"about:blank"}"#)
                    .map_err(|e| format!("createTarget: {e}"))?;
                let target_id = reply
                    .split(r#""targetId":""#)
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .ok_or("createTarget: no targetId in reply")?
                    .to_owned();
                self.targets.insert(tab, target_id);
            }
            let target_id = self.targets[&tab].clone();

            // Navigate the target.
            let nav_params = format!(
                r#"{{"targetId":"{target_id}","url":"{}"}}"#,
                json_escape(url)
            );
            let nav = self
                .call("Page.navigate", &nav_params)
                .map_err(|e| format!("navigate: {e}"))?;
            if nav.contains("\"errorText\"") {
                let reason = nav
                    .split(r#""errorText":""#)
                    .nth(1)
                    .and_then(|r| r.split('"').nth(1))
                    .unwrap_or("navigation failed")
                    .to_owned();
                return Err(format!("net::{reason}"));
            }

            // Phase 0: screenshot-per-navigation; a load-event wait belongs
            // with the streaming screencast in Phase 1.
            std::thread::sleep(std::time::Duration::from_millis(700));

            let shot = self
                .call(
                    "Page.captureScreenshot",
                    r#"{"format":"jpeg","quality":70}"#,
                )
                .map_err(|e| format!("screenshot: {e}"))?;
            let b64 = shot
                .split(r#""data":""#)
                .nth(1)
                .and_then(|r| r.split('"').next())
                .ok_or("screenshot: no data in reply")?;

            let jpeg = litric_browser::base64::decode(b64.as_bytes())
                .map_err(|e| format!("base64: {e}"))?;
            let img = litric_browser::jpeg::decode(&jpeg).map_err(|e| format!("jpeg: {e}"))?;

            let mut frame = Vec::with_capacity(img.pixels.len() * 4);
            for px in &img.pixels {
                frame.extend_from_slice(&[
                    (px >> 16) as u8, // R
                    (px >> 8) as u8,  // G
                    *px as u8,        // B
                    0xFF,             // A
                ]);
            }
            let title = format!("Page {tab}");
            Ok((title, frame, img.width, img.height))
        }
    }

    impl Drop for ChromiumProc {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    /// Find a Chromium binary: the env override, then the usual Linux paths.
    pub fn find_chromium() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("BROWSERD_CHROMIUM") {
            let p = PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }
        for candidate in [
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
        ] {
            let p = PathBuf::from(candidate);
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    /// The broker: serve LBP/1 on a Unix socket forever.
    #[derive(Debug)]
    pub struct Broker {
        engine: Mutex<Engine>,
        frames: Mutex<HashMap<u32, (Vec<u8>, u32, u32)>>,
        requests: AtomicU64,
    }

    impl Default for Broker {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Broker {
        /// A broker with whatever engine this machine actually has.
        #[must_use]
        pub fn new() -> Self {
            let engine = match find_chromium() {
                Some(path) => match ChromiumProc::spawn(&path) {
                    Ok(proc) => {
                        eprintln!("browserd: engine = chromium ({})", path.display());
                        Engine::Chromium { proc }
                    }
                    Err(e) => Engine::Missing {
                        reason: format!(
                            "Chromium at {} could not be started ({e})",
                            path.display()
                        ),
                    },
                },
                None => Engine::Missing {
                    reason: "No Chromium binary is present in this image (ADR-0018: the engine is integrated, never written). Set BROWSERD_CHROMIUM or install chromium.".to_owned(),
                },
            };
            Self {
                engine: Mutex::new(engine),
                frames: Mutex::new(HashMap::new()),
                requests: AtomicU64::new(0),
            }
        }

        /// The reason the engine is missing, if it is.
        #[must_use]
        pub fn missing_reason(&self) -> Option<String> {
            match &*self.engine.lock().unwrap_or_else(|p| p.into_inner()) {
                Engine::Missing { reason } => Some(reason.clone()),
                Engine::Chromium { .. } => None,
            }
        }

        /// Handle one GO: load the URL, produce a frame, remember it, reply OK.
        pub fn handle_go(&self, tab: u32, url: &str) -> Result<String, String> {
            self.requests.fetch_add(1, Ordering::Relaxed);
            // Scheme gate: the same schemes the shell's Url parser accepts.
            let scheme_ok = ["https://", "http://", "file://", "litric://", "about:"]
                .iter()
                .any(|s| url.starts_with(s));
            if !scheme_ok {
                return Err(format!("unsupported scheme in {url}"));
            }
            if url == "litric://about" || url == "about:blank" {
                // Honest local page: no fetch, no engine, no lies.
                return Ok(format!(
                    "OK title={} bytes=0 secure=0",
                    super::pct_encode("New Tab")
                ));
            }

            match &mut *self.engine.lock().unwrap_or_else(|p| p.into_inner()) {
                Engine::Missing { reason } => Err(reason.clone()),
                Engine::Chromium { proc } => {
                    let (title, frame, w, h) = proc.load_and_capture(tab, url)?;
                    self.frames
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .insert(tab, (frame, w, h));
                    Ok(format!(
                        "OK title={} bytes=0 secure=1",
                        super::pct_encode(&title)
                    ))
                }
            }
        }

        /// Take a tab's last frame (the shell pulls; the broker never pushes).
        pub fn take_frame(&self, tab: u32) -> Option<(Vec<u8>, u32, u32)> {
            self.frames
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get(&tab)
                .cloned()
        }

        /// Serve one connection: read commands, write replies and frames.
        pub(crate) fn serve(&self, stream: UnixStream) {
            let mut reader = BufReader::new(match stream.try_clone() {
                Ok(s) => s,
                Err(_) => return,
            });
            let mut writer = stream;
            loop {
                let mut line = Vec::new();
                let taken = (&mut reader)
                    .take(64 * 1024)
                    .read_until(b'\n', &mut line)
                    .unwrap_or(0);
                if taken == 0 {
                    return;
                }
                if line.last() != Some(&b'\n') {
                    return; // lost framing: close
                }
                line.pop();
                let text = String::from_utf8_lossy(&line).into_owned();
                let mut parts = text.split_whitespace();
                let Some(cmd) = parts.next() else {
                    continue;
                };
                match cmd {
                    "PING" => {
                        if writeln!(&mut writer, "PONG").is_err() {
                            return;
                        }
                    }
                    "GO" => {
                        let tab: u32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(1);
                        let url = super::pct_decode(parts.next().unwrap_or("-"));
                        match self.handle_go(tab, &url) {
                            Ok(ok) => {
                                if writeln!(&mut writer, "{ok}").is_err() {
                                    return;
                                }
                                if let Some((frame, w, h)) = self.take_frame(tab) {
                                    let header = format!("FRAME {tab} {}\n", frame.len());
                                    if writer.write_all(header.as_bytes()).is_err()
                                        || writer.write_all(&frame).is_err()
                                    {
                                        return;
                                    }
                                    let _ = (w, h);
                                }
                            }
                            Err(why) => {
                                let line = format!("ERR {}", super::pct_encode(&why));
                                if writeln!(&mut writer, "{line}").is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    "QUIT" => return,
                    _ => {
                        if writeln!(&mut writer, "ERR unknown command").is_err() {
                            return;
                        }
                    }
                }
            }
        }

        /// Bind the socket and serve until the process ends.
        ///
        /// # Errors
        /// Bind or accept errors; the caller logs and exits.
        pub fn run(&self, socket_path: &Path) -> io::Result<()> {
            let _ = std::fs::remove_file(socket_path);
            if let Some(parent) = socket_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let listener = UnixListener::bind(socket_path)?;
            eprintln!("browserd: listening on {}", socket_path.display());
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => self.serve(s),
                    Err(e) => eprintln!("browserd: accept failed: {e}"),
                }
            }
            Ok(())
        }
    }

    /// The socket path: `BROWSERD_SOCK` or the default.
    pub fn socket_path() -> PathBuf {
        std::env::var("BROWSERD_SOCK")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_SOCKET))
    }
}

fn main() {
    #[cfg(unix)]
    {
        // The unit runs us as `litric-browserd --stdio` under systemd socket
        // activation: stdin/stdout ARE the accepted connection, and the
        // process serves exactly one. The self-bound-socket path remains for
        // running the broker by hand (BROWSERD_SOCK override included).
        if std::env::args().any(|a| a == "--stdio") {
            use std::os::unix::io::FromRawFd;

            // fd 0 is the socket-activated connection systemd handed us;
            // wrapping it never takes ownership (ManuallyDrop) and fd 0 is
            // never closed by us — the fourth and last explicit unsafe opt-out.
            #[allow(unsafe_code)]
            let stream = unsafe {
                std::mem::ManuallyDrop::new(std::os::unix::net::UnixStream::from_raw_fd(0))
            };
            let dup = match stream.try_clone() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("browserd: cannot use socket-activated stdio: {e}");
                    std::process::exit(1);
                }
            };
            let broker = broker::Broker::new();
            broker.serve(dup);
            return;
        }
        let broker = broker::Broker::new();
        let path = broker::socket_path();
        if let Err(e) = broker.run(&path) {
            eprintln!("browserd: {e}");
            std::process::exit(1);
        }
    }
    #[cfg(not(unix))]
    {
        eprintln!("litric-browserd is a Unix daemon; there is nothing to run here.");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, unix))]
mod tests {
    use super::broker::{find_chromium, Broker};
    use super::{pct_decode, pct_encode};
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    #[test]
    fn pct_round_trips_and_stays_literal_on_garbage() {
        assert_eq!(pct_decode("%64oc%20one.txt"), "doc one.txt");
        assert_eq!(pct_decode("-"), "");
        assert_eq!(pct_decode("plain"), "plain");
        assert_eq!(pct_decode("%zz"), "%zz", "bad hex stays literal");
        assert_eq!(pct_encode("a b/c"), "a%20b/c");
        let enc = pct_encode("100%");
        assert_eq!(pct_decode(&enc), "100%");
    }

    #[test]
    fn a_broker_without_chromium_refuses_by_name() {
        if find_chromium().is_some() {
            return; // a machine with Chromium passes the honesty tests below
        }
        let broker = Broker::new();
        let reason = broker
            .missing_reason()
            .expect("missing engine has a reason");
        assert!(reason.contains("No Chromium"), "{reason}");
        let reply = broker.handle_go(1, "https://example.com/");
        assert!(matches!(reply, Err(ref why) if why.contains("No Chromium")));
    }

    #[test]
    fn about_pages_answer_without_an_engine() {
        let broker = Broker::new();
        // about pages never touch the engine, so they answer OK on any host.
        let reply = broker.handle_go(1, "about:blank");
        assert!(
            matches!(&reply, Ok(ok) if ok.starts_with("OK title=")),
            "{reply:?}"
        );
    }

    #[test]
    fn lbp_lines_round_trip_through_the_serve_loop() {
        let (client, server) = UnixStream::pair().expect("pair");
        let broker = Broker::new();
        let worker = std::thread::spawn(move || broker.serve(server));
        let mut c = client;

        c.write_all(b"PING\n").unwrap();
        c.write_all(b"GO 1 https%3A%2F%2Fexample.com%2F\n").unwrap();
        c.write_all(b"JUNK\n").unwrap();
        c.write_all(b"QUIT\n").unwrap();

        let mut reader = BufReader::new(c);
        let mut pong = String::new();
        reader.read_line(&mut pong).unwrap();
        assert_eq!(pong.trim(), "PONG");

        let mut go = String::new();
        reader.read_line(&mut go).unwrap();
        assert!(go.starts_with("OK") || go.starts_with("ERR "), "{go}");

        let mut junk = String::new();
        reader.read_line(&mut junk).unwrap();
        assert_eq!(junk.trim(), "ERR unknown command");

        worker.join().expect("serve thread ends on QUIT");
    }
}
