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
//! → HELLO 1                 ← HELLO 1 engine=chromium ... | ERR engine-unavailable <why>
//! → VIEWPORT <w> <h>        ← OK viewport=<w>x<h>
//! → GO <tab> <pct-url>      ← TAB <tab> state=... url=... title=... back= fwd= secure=
//!                           ← FRAME <tab> <w> <h> <len>\n<len bytes of BGRA8>
//! → PING                    ← PONG
//! → QUIT                    (connection closes)
//! ```
//!
//! The default socket is `/run/litric/browser.sock` (`BROWSERD_SOCK` overrides
//! it); under systemd the unit passes `--stdio` and the accepted connection is
//! simply stdin/stdout. The frame header carries the frame's own width and
//! height because the page is captured at the broker's viewport, not at the
//! size of the window the shell happens to draw it in: without them the shell
//! could only guess the scale, and it used to read them from the wrong fields
//! and drop every frame.
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
    while let Some(&byte) = bytes.get(i) {
        if byte == b'%' {
            if let (Some(high), Some(low)) = (bytes.get(i + 1), bytes.get(i + 2)) {
                let hi = (*high as char).to_digit(16);
                let lo = (*low as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push((h * 16 + l) as u8);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(byte);
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
            fn fcntl(fd: i32, cmd: i32, ...) -> i32;
        }

        /// Linux `O_CLOEXEC`.
        const O_CLOEXEC: u32 = 0o2_000_000;
        /// `fcntl` command for the descriptor flag word.
        const F_SETFD: i32 = 2;

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
        ///
        /// `dup2(fd, fd)` is a no-op that does **not** clear close-on-exec, and
        /// the pipe's first descriptor is exactly fd 3 in a freshly started
        /// broker. Chromium then reported "Remote debugging pipe file
        /// descriptors are not open" and exited before the first command: the
        /// engine was dead before it could draw anything. The flags word is
        /// cleared explicitly so the descriptor survives exec whatever number
        /// it arrived on.
        pub fn dup2_or_err(fd: i32, target: i32) -> io::Result<()> {
            #[allow(unsafe_code)]
            let rc = unsafe { dup2(fd, target) };
            if rc != target {
                return Err(io::Error::last_os_error());
            }
            // Best effort: the descriptor is open either way, and the caller
            // distinguishes a broken pipe from a working one.
            #[allow(unsafe_code)]
            let _ = unsafe { fcntl(target, F_SETFD, 0) };
            Ok(())
        }

        /// Best-effort close of a parent-side descriptor.
        pub fn close_fd(fd: i32) {
            #[allow(unsafe_code)]
            unsafe {
                close(fd);
            }
        }
    }

    /// A cached picture of a tab: BGRA8 bytes and the size they describe.
    type CachedFrame = (Vec<u8>, u32, u32);

    /// One pointer event, in page pixels.
    #[derive(Debug, Clone, Copy)]
    pub struct MouseEvent<'a> {
        /// `move`, `down` or `up`.
        kind: &'a str,
        x: f64,
        y: f64,
        /// `left`, `middle` or `right`.
        button: &'a str,
        /// Click count for a press.
        clicks: u32,
        /// CDP modifier bitmask.
        mods: u32,
    }

    /// One key event.
    #[derive(Debug, Clone, Copy)]
    pub struct KeyEvent<'a> {
        down: bool,
        key: &'a str,
        code: &'a str,
        /// Windows virtual key code.
        vk: i64,
        /// CDP modifier bitmask.
        mods: u32,
        /// Text Chromium should treat as part of the key (Enter, Tab).
        text: &'a str,
    }

    /// One tab's target and CDP session.
    #[derive(Debug, Clone)]
    pub struct TabSession {
        target: String,
        session: String,
        url: String,
        title: String,
        state: &'static str,
        secure: bool,
        back: bool,
        fwd: bool,
    }

    /// One headless Chromium driven over `--remote-debugging-pipe`.
    ///
    /// The contract is fixed by Chromium: CDP messages arrive on the child's
    /// fd 3 and leave on fd 4. `Command::stdio` cannot reach fd 3/4, so the
    /// spawn uses the kernel's dup2 between fork and exec.
    #[derive(Debug)]
    pub struct ChromiumProc {
        child: std::process::Child,
        cdp_in: std::fs::File,
        cdp_out: BufReader<std::fs::File>,
        next_id: AtomicU64,
        tabs: HashMap<u32, TabSession>,
        /// Sessions whose `Page.loadEventFired` arrived while another call was
        /// waiting for its response. A response and an event share the pipe.
        loaded: std::collections::HashSet<String>,
        viewport: (u32, u32),
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

    /// The broker-owned download directory, if the service supplied one.
    static DOWNLOAD_DIR: std::sync::OnceLock<Option<std::path::PathBuf>> =
        std::sync::OnceLock::new();

    fn downloads_dir() -> Option<&'static std::path::PathBuf> {
        DOWNLOAD_DIR
            .get_or_init(|| {
                let args = std::env::args().collect::<Vec<_>>();
                args.iter()
                    .position(|arg| arg == "--downloads")
                    .and_then(|index| args.get(index + 1))
                    .map(std::path::PathBuf::from)
            })
            .as_ref()
    }

    /// The tab state that tells the truth after `Page.stopLoading`.
    ///
    /// `document.readyState` is `complete` only when the load had already
    /// finished. Unknown (the evaluation failed) is not a reason to claim
    /// `loaded`: the honest answer is that the page is stopped where it is.
    pub(super) fn stopped_state(ready: Option<&str>) -> &'static str {
        match ready {
            Some("complete") => "loaded",
            _ => "stopped",
        }
    }

    /// Remove one finished download from the staging directory.
    ///
    /// The name must be one plain segment: this is the only place the broker
    /// deletes anything, and `../` in a download's name would be a way out of
    /// its own directory even though it created that directory itself.
    pub(super) fn drop_staged(dir: &std::path::Path, name: &str) -> Result<(), String> {
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return Err("that is not a download name".to_owned());
        }
        let path = dir.join(name);
        let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() || !meta.is_file() {
            return Err("that is not a staged download".to_owned());
        }
        std::fs::remove_file(&path).map_err(|e| e.to_string())
    }

    /// Read one JSON value from the NUL-delimited CDP pipe.
    fn read_json_value(
        reader: &mut BufReader<std::fs::File>,
    ) -> Result<litric_browser::json::Value, String> {
        let mut buf = Vec::new();
        let n = reader
            .read_until(0, &mut buf)
            .map_err(|e| format!("CDP read: {e}"))?;
        if n == 0 {
            return Err("Chromium closed the debugging pipe".to_owned());
        }
        while buf.last() == Some(&0) {
            buf.pop();
        }
        if buf.is_empty() {
            return Err("Chromium sent an empty CDP message".to_owned());
        }
        litric_browser::json::parse(&buf).map_err(|e| format!("CDP JSON: {e}"))
    }

    fn field<'a>(
        v: &'a litric_browser::json::Value,
        key: &str,
    ) -> Option<&'a litric_browser::json::Value> {
        v.get(key)
    }

    fn text_field(v: &litric_browser::json::Value, key: &str) -> Option<String> {
        field(v, key)
            .and_then(litric_browser::json::Value::as_str)
            .map(str::to_owned)
    }

    /// A CDP error object is `{"code":N,"message":"..."}`.
    fn cdp_error(v: &litric_browser::json::Value) -> Option<String> {
        let err = v.get("error")?;
        let message = text_field(err, "message").unwrap_or_else(|| "CDP request failed".to_owned());
        let code = field(err, "code").and_then(litric_browser::json::Value::as_i64);
        Some(code.map_or(message.clone(), |c| format!("{message} (code {c})")))
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
                // Chromium's sandbox is deliberately left ON. The service runs
                // as the unprivileged `litric` user and the unit does not set
                // RestrictNamespaces, so user namespaces are available. A
                // chromium that cannot start its sandbox is reported as an
                // engine failure; it is never silently downgraded to
                // --no-sandbox, which would turn every renderer into a process
                // with the broker's access to the profile and downloads.
                "--disable-gpu".to_owned(),
                "--disable-dev-shm-usage".to_owned(),
                "--hide-scrollbars".to_owned(),
                "--mute-audio".to_owned(),
                "--no-first-run".to_owned(),
                "--no-default-browser-check".to_owned(),
                "--remote-debugging-pipe".to_owned(),
            ];
            // The user-data dir must be inside the unit's ReadWritePaths:
            // with the default (~/.config/chromium) under ProtectHome=yes,
            // Chromium dies on its first mkdir.
            if let Some(profile) = profile_dir() {
                let _ = std::fs::create_dir_all(profile);
                args.push(format!("--user-data-dir={}", profile.display()));
            }
            // Chromium's stderr goes to the broker's, which the unit sends to
            // the journal. Silencing it (the old behaviour) is why a Chromium
            // that died at startup produced nothing but "CDP write: broken
            // pipe" with no cause anywhere on the machine.
            cmd.args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::inherit());
            // Between fork and exec is the only window fd 3/4 can be arranged;
            // dup2 copies lose close-on-exec by kernel rule.
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
            #[allow(unsafe_code)]
            let cdp_in = unsafe { std::fs::File::from_raw_fd(in_w) };
            #[allow(unsafe_code)]
            let cdp_out = unsafe { BufReader::new(std::fs::File::from_raw_fd(out_r)) };
            let mut proc = Self {
                child,
                cdp_in,
                cdp_out,
                next_id: AtomicU64::new(1),
                tabs: HashMap::new(),
                loaded: std::collections::HashSet::new(),
                viewport: (1280, 800),
            };
            if let Some(downloads) = downloads_dir() {
                if let Err(error) = std::fs::create_dir_all(downloads) {
                    eprintln!("browserd: download directory unavailable: {error}");
                } else {
                    let params = format!(
                        r#"{{"behavior":"allow","downloadPath":"{}","eventsEnabled":true}}"#,
                        json_escape(&downloads.display().to_string())
                    );
                    if let Err(error) = proc.call("Browser.setDownloadBehavior", &params) {
                        eprintln!("browserd: download integration unavailable: {error}");
                    }
                }
            }
            Ok(proc)
        }

        /// Send one CDP request and return its `result`.
        fn call_session(
            &mut self,
            method: &str,
            params: &str,
            session: Option<&str>,
        ) -> Result<litric_browser::json::Value, String> {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            let session = session.map_or_else(String::new, |s| {
                format!(r#","sessionId":"{}""#, json_escape(s))
            });
            let msg = format!(
                r#"{{"id":{id},"method":"{}","params":{}{session}}}"#,
                json_escape(method),
                params
            );
            self.cdp_in
                .write_all(msg.as_bytes())
                .and_then(|_| self.cdp_in.write_all(b"\0"))
                .and_then(|_| self.cdp_in.flush())
                .map_err(|e| format!("CDP write: {e}"))?;
            loop {
                let value = read_json_value(&mut self.cdp_out)?;
                if let Some(event_session) = text_field(&value, "sessionId") {
                    if text_field(&value, "method").as_deref() == Some("Page.loadEventFired") {
                        self.loaded.insert(event_session);
                    }
                }
                if field(&value, "id").and_then(litric_browser::json::Value::as_u64) == Some(id) {
                    if let Some(error) = cdp_error(&value) {
                        return Err(format!("{method}: {error}"));
                    }
                    return Ok(field(&value, "result")
                        .cloned()
                        .unwrap_or(litric_browser::json::Value::Null));
                }
                if text_field(&value, "method").as_deref() == Some("Inspector.targetCrashed") {
                    return Err("Chromium target crashed".to_owned());
                }
            }
        }

        /// Send one browser-level request (Target.*, Browser.*).
        pub fn call(
            &mut self,
            method: &str,
            params: &str,
        ) -> Result<litric_browser::json::Value, String> {
            self.call_session(method, params, None)
        }

        fn create_tab(&mut self, tab: u32) -> Result<(), String> {
            if self.tabs.contains_key(&tab) {
                return Ok(());
            }
            let created = self.call("Target.createTarget", r#"{"url":"about:blank"}"#)?;
            let target = text_field(&created, "targetId")
                .ok_or_else(|| "createTarget: no targetId in reply".to_owned())?;
            let params = format!(
                r#"{{"targetId":"{}","flatten":true}}"#,
                json_escape(&target)
            );
            let attached = self.call("Target.attachToTarget", &params)?;
            let session = text_field(&attached, "sessionId")
                .ok_or_else(|| "attachToTarget: no sessionId in reply".to_owned())?;
            self.call_session("Page.enable", "{}", Some(&session))?;
            // A background tab still draws, but it does not take mouse input:
            // Chromium only routes pointer events to the front tab. Without
            // this, a click landed on nothing and a link could not be followed.
            self.call_session("Page.bringToFront", "{}", Some(&session))?;
            let (w, h) = self.viewport;
            let metrics =
                format!(r#"{{"width":{w},"height":{h},"deviceScaleFactor":1,"mobile":false}}"#);
            self.call_session(
                "Emulation.setDeviceMetricsOverride",
                &metrics,
                Some(&session),
            )?;
            self.tabs.insert(
                tab,
                TabSession {
                    target,
                    session,
                    url: "about:blank".to_owned(),
                    title: "New Tab".to_owned(),
                    state: "empty",
                    secure: false,
                    back: false,
                    fwd: false,
                },
            );
            Ok(())
        }

        /// Wait for `Page.loadEventFired` on one session. No fixed sleep is a
        /// guarantee that a page loaded; this is the event Chromium emits when
        /// the document's load event has run, and its absence is reported.
        fn wait_loaded(&mut self, session: &str) -> Result<(), String> {
            if self.loaded.remove(session) {
                return Ok(());
            }
            let deadline = std::time::Instant::now()
                .checked_add(std::time::Duration::from_secs(15))
                .ok_or_else(|| "navigation timeout overflow".to_owned())?;
            loop {
                if std::time::Instant::now() >= deadline {
                    return Err("the page did not finish loading within 15s".to_owned());
                }
                let value = read_json_value(&mut self.cdp_out)?;
                let method = text_field(&value, "method");
                let event_session = text_field(&value, "sessionId");
                if method.as_deref() == Some("Page.loadEventFired")
                    && event_session.as_deref() == Some(session)
                {
                    return Ok(());
                }
                if method.as_deref() == Some("Inspector.targetCrashed")
                    && event_session.as_deref() == Some(session)
                {
                    return Err("the page's renderer crashed".to_owned());
                }
            }
        }

        fn history_flags(&mut self, session: &str) -> (bool, bool) {
            let Ok(history) = self.call_session("Page.getNavigationHistory", "{}", Some(session))
            else {
                return (false, false);
            };
            let current = field(&history, "currentIndex")
                .and_then(litric_browser::json::Value::as_u64)
                .unwrap_or(0);
            let len = field(&history, "entries")
                .and_then(|v| match v {
                    litric_browser::json::Value::Array(a) => Some(a.len() as u64),
                    _ => None,
                })
                .unwrap_or(0);
            (current > 0, current.saturating_add(1) < len)
        }

        fn title(&mut self, session: &str) -> String {
            let params = r#"{"expression":"document.title","returnByValue":true}"#;
            let Ok(reply) = self.call_session("Runtime.evaluate", params, Some(session)) else {
                return String::new();
            };
            field(&reply, "result")
                .and_then(|v| text_field(v, "value"))
                .unwrap_or_default()
        }

        /// Capture the visible page as BGRA8, the byte order the shell's canvas
        /// uses (`0xAARRGGBB` little-endian).
        fn capture(&mut self, session: &str) -> Result<(Vec<u8>, u32, u32), String> {
            let shot = self.call_session(
                "Page.captureScreenshot",
                r#"{"format":"jpeg","quality":80,"fromSurface":true,"captureBeyondViewport":false}"#,
                Some(session),
            )?;
            let b64 = text_field(&shot, "data")
                .ok_or_else(|| "captureScreenshot: no data in reply".to_owned())?;
            let jpeg = litric_browser::base64::decode(b64.as_bytes())
                .map_err(|e| format!("base64: {e}"))?;
            let img = litric_browser::jpeg::decode(&jpeg).map_err(|e| format!("jpeg: {e}"))?;
            let mut frame = Vec::with_capacity(img.pixels.len().saturating_mul(4));
            for px in &img.pixels {
                frame.extend_from_slice(&[
                    *px as u8,        // B
                    (px >> 8) as u8,  // G
                    (px >> 16) as u8, // R
                    0xFF,             // A
                ]);
            }
            Ok((frame, img.width, img.height))
        }

        /// Navigate one tab, wait for its load event, then capture it.
        pub fn navigate(&mut self, tab: u32, url: &str) -> Result<(Vec<u8>, u32, u32), String> {
            self.create_tab(tab)?;
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no tab")?;
            let params = format!(r#"{{"url":"{}"}}"#, json_escape(url));
            let nav = self.call_session("Page.navigate", &params, Some(&session))?;
            if let Some(error) = text_field(&nav, "errorText") {
                if !error.is_empty() {
                    if let Some(t) = self.tabs.get_mut(&tab) {
                        t.state = "failed";
                    }
                    return Err(format!("navigation failed: {error}"));
                }
            }
            self.wait_loaded(&session)?;
            let title = self.title(&session);
            let (back, fwd) = self.history_flags(&session);
            let secure = url.starts_with("https://");
            if let Some(t) = self.tabs.get_mut(&tab) {
                t.url = url.to_owned();
                t.title = if title.is_empty() {
                    url.to_owned()
                } else {
                    title
                };
                t.state = "loaded";
                t.secure = secure;
                t.back = back;
                t.fwd = fwd;
            }
            self.capture(&session)
        }

        /// Bring a tab's recorded address, title and history flags back in line
        /// with what its renderer actually has.
        ///
        /// A page moves on its own: a click follows a link, a form submits, a
        /// redirect fires. Until this existed the tab kept the URL it was
        /// navigated to and the address bar lied about where the user was.
        fn refresh_meta(&mut self, tab: u32) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            let history = self.call_session("Page.getNavigationHistory", "{}", Some(&session))?;
            let index = field(&history, "currentIndex")
                .and_then(litric_browser::json::Value::as_u64)
                .unwrap_or(0);
            let entries = field(&history, "entries").and_then(|v| match v {
                litric_browser::json::Value::Array(a) => Some(a.clone()),
                _ => None,
            });
            let (url, count) = match entries {
                Some(entries) => {
                    let url = entries
                        .get(usize::try_from(index).unwrap_or(usize::MAX))
                        .and_then(|e| text_field(e, "url"))
                        .unwrap_or_default();
                    (url, entries.len() as u64)
                }
                None => (String::new(), 0),
            };
            let title = self.title(&session);
            if let Some(t) = self.tabs.get_mut(&tab) {
                if !url.is_empty() && url != t.url {
                    t.url.clone_from(&url);
                    t.secure = url.starts_with("https://");
                }
                if !title.is_empty() {
                    t.title = title;
                }
                t.back = index > 0;
                t.fwd = index.saturating_add(1) < count;
            }
            Ok(())
        }

        /// Capture the tab as it is, without navigating.
        pub fn refresh_frame(&mut self, tab: u32) -> Result<(Vec<u8>, u32, u32), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            // Read the page's own address and title first: the capture below is
            // of whatever the page has become, and the reply must describe the
            // same page or the picture and the address bar disagree.
            if let Err(e) = self.refresh_meta(tab) {
                eprintln!("browserd: could not read tab {tab} state: {e}");
            }
            self.capture(&session)
        }

        fn history(&mut self, tab: u32, forward: bool) -> Result<(), String> {
            self.create_tab(tab)?;
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no tab")?;
            let history = self.call_session("Page.getNavigationHistory", "{}", Some(&session))?;
            let current = field(&history, "currentIndex")
                .and_then(litric_browser::json::Value::as_u64)
                .unwrap_or(0);
            let target = if forward {
                current.saturating_add(1)
            } else {
                current
                    .checked_sub(1)
                    .ok_or("no earlier page in this tab")?
            };
            let entry = field(&history, "entries")
                .and_then(|v| match v {
                    litric_browser::json::Value::Array(a) => a
                        .get(usize::try_from(target).unwrap_or(usize::MAX))
                        .and_then(|e| field(e, "id"))
                        .and_then(litric_browser::json::Value::as_u64),
                    _ => None,
                })
                .ok_or("no history entry in that direction")?;
            let params = format!(r#"{{"entryId":{entry}}}"#);
            self.call_session("Page.navigateToHistoryEntry", &params, Some(&session))?;
            self.wait_loaded(&session)?;
            let title = self.title(&session);
            let (back, fwd) = self.history_flags(&session);
            if let Some(t) = self.tabs.get_mut(&tab) {
                t.title = title;
                t.state = "loaded";
                t.back = back;
                t.fwd = fwd;
            }
            Ok(())
        }

        fn reload(&mut self, tab: u32) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            self.call_session("Page.reload", r#"{"ignoreCache":false}"#, Some(&session))?;
            self.wait_loaded(&session)?;
            let (back, fwd) = self.history_flags(&session);
            if let Some(t) = self.tabs.get_mut(&tab) {
                t.state = "loaded";
                t.back = back;
                t.fwd = fwd;
            }
            Ok(())
        }

        fn stop(&mut self, tab: u32) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            self.call_session("Page.stopLoading", "{}", Some(&session))?;
            // Stopping does not finish a page. Ask the document what it has:
            // `complete` means the load had already finished and Stop changed
            // nothing, anything else means the user is looking at a partial
            // page and the tab must say so rather than "loaded".
            let ready = self
                .call_session(
                    "Runtime.evaluate",
                    r#"{"expression":"document.readyState","returnByValue":true}"#,
                    Some(&session),
                )
                .ok()
                .and_then(|reply| field(&reply, "result").cloned())
                .and_then(|v| text_field(&v, "value"));
            let state = stopped_state(ready.as_deref());
            if let Some(t) = self.tabs.get_mut(&tab) {
                t.state = state;
            }
            Ok(())
        }

        fn close_tab(&mut self, tab: u32) -> Result<(), String> {
            let target = self
                .tabs
                .remove(&tab)
                .map(|t| t.target)
                .ok_or("no such tab")?;
            let params = format!(r#"{{"targetId":"{}"}}"#, json_escape(&target));
            let _ = self.call("Target.closeTarget", &params);
            Ok(())
        }

        fn mouse(&mut self, tab: u32, event: MouseEvent<'_>) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            let MouseEvent {
                kind,
                x,
                y,
                button,
                clicks,
                mods,
            } = event;
            let kind = match kind {
                "move" => "mouseMoved",
                "down" => "mousePressed",
                "up" => "mouseReleased",
                _ => return Err("mouse event is not move, down or up".to_owned()),
            };
            let (button, buttons) = match button {
                "left" | "middle" | "right" if kind != "mouseMoved" => {
                    (button, if kind == "mousePressed" { 1 } else { 0 })
                }
                _ => ("none", 0),
            };
            let params = format!(
                r#"{{"type":"{kind}","x":{x},"y":{y},"button":"{button}","buttons":{buttons},"clickCount":{clicks},"modifiers":{mods}}}"#
            );
            self.call_session("Input.dispatchMouseEvent", &params, Some(&session))?;
            Ok(())
        }

        fn wheel(&mut self, tab: u32, x: f64, y: f64, dx: f64, dy: f64) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            let params = format!(
                r#"{{"type":"mouseWheel","x":{x},"y":{y},"deltaX":{dx},"deltaY":{dy},"button":"none","modifiers":0}}"#
            );
            self.call_session("Input.dispatchMouseEvent", &params, Some(&session))?;
            Ok(())
        }

        fn key(&mut self, tab: u32, event: KeyEvent<'_>) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            let KeyEvent {
                down,
                key,
                code,
                vk,
                mods,
                text,
            } = event;
            let kind = if down { "keyDown" } else { "keyUp" };
            let text = if down && !text.is_empty() {
                format!(r#","text":"{}""#, json_escape(text))
            } else {
                String::new()
            };
            let params = format!(
                r#"{{"type":"{kind}","key":"{}","code":"{}","windowsVirtualKeyCode":{vk},"nativeVirtualKeyCode":{vk},"modifiers":{mods}{text}}}"#,
                json_escape(key),
                json_escape(code)
            );
            self.call_session("Input.dispatchKeyEvent", &params, Some(&session))?;
            Ok(())
        }

        fn text(&mut self, tab: u32, text: &str) -> Result<(), String> {
            let session = self
                .tabs
                .get(&tab)
                .map(|t| t.session.clone())
                .ok_or("no such tab")?;
            let params = format!(r#"{{"text":"{}"}}"#, json_escape(text));
            self.call_session("Input.insertText", &params, Some(&session))?;
            Ok(())
        }

        /// The `TAB` event for one tab.
        fn status_line(&self, tab: u32) -> Option<String> {
            let t = self.tabs.get(&tab)?;
            Some(format!(
                "TAB {tab} state={} url={} title={} back={} fwd={} secure={}",
                t.state,
                super::pct_encode(&t.url),
                super::pct_encode(&t.title),
                u8::from(t.back),
                u8::from(t.fwd),
                u8::from(t.secure)
            ))
        }
    }

    impl Drop for ChromiumProc {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    /// Find a Chromium binary: the env override, then the usual Linux paths.
    ///
    /// Presence of a path is not enough: the broker must be able to execute it.
    /// A package that installed a directory, a broken symlink or a file with
    /// no execute bit is reported as `None`, which makes the failure say "no
    /// Chromium" instead of "spawn failed: permission denied" later.
    pub fn find_chromium() -> Option<PathBuf> {
        fn usable(p: &Path) -> bool {
            let Ok(meta) = std::fs::metadata(p) else {
                return false;
            };
            if !meta.is_file() {
                return false;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                meta.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                true
            }
        }
        if let Ok(path) = std::env::var("BROWSERD_CHROMIUM") {
            let p = PathBuf::from(path);
            if p.is_absolute() && usable(&p) {
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
            if usable(&p) {
                return Some(p);
            }
        }
        None
    }

    /// One engine action's result: the tab's state plus its new frame.
    #[derive(Debug)]
    pub struct Update {
        pub line: String,
        pub frame: Option<(Vec<u8>, u32, u32)>,
    }

    /// The broker: serve LBP/1 on a Unix socket forever.
    #[derive(Debug)]
    pub struct Broker {
        engine: Mutex<Engine>,
        frames: Mutex<HashMap<u32, CachedFrame>>,
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

        fn hello(&self) -> String {
            let engine = self.engine.lock().unwrap_or_else(|p| p.into_inner());
            match &*engine {
                Engine::Chromium { .. } => {
                    format!(
                        "HELLO 1 engine=chromium version={} sandbox=on",
                        env!("CARGO_PKG_VERSION")
                    )
                }
                Engine::Missing { reason } => {
                    format!("ERR engine-unavailable {}", super::pct_encode(reason))
                }
            }
        }

        /// One GO: navigate, remember the frame, and report the tab.
        pub fn handle_go(&self, tab: u32, url: &str) -> Result<Update, String> {
            self.requests.fetch_add(1, Ordering::Relaxed);
            if url == "litric://about" || url == "about:blank" {
                return Ok(Update {
                    line: format!(
                        "TAB {tab} state=empty url={} title={} back=0 fwd=0 secure=0",
                        super::pct_encode(url),
                        super::pct_encode("New Tab")
                    ),
                    frame: None,
                });
            }
            if url.starts_with("file:") {
                return Err(
                    "file: addresses are refused: the browser has no filesystem grant".to_owned(),
                );
            }
            let parsed = litric_browser::url::Url::parse(url)
                .map_err(|e| format!("that address is not valid ({e:?})"))?;
            if !matches!(
                parsed.scheme,
                litric_browser::url::Scheme::Https | litric_browser::url::Scheme::Http
            ) {
                return Err("only http and https addresses can be loaded".to_owned());
            }
            if parsed.scheme == litric_browser::url::Scheme::Http {
                return Err("http is refused: this browser loads HTTPS only".to_owned());
            }
            match &mut *self.engine.lock().unwrap_or_else(|p| p.into_inner()) {
                Engine::Missing { reason } => Err(reason.clone()),
                Engine::Chromium { proc } => {
                    let frame = proc.navigate(tab, url)?;
                    let line = proc
                        .status_line(tab)
                        .ok_or_else(|| "the tab disappeared during navigation".to_owned())?;
                    self.frames
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .insert(tab, (frame.0.clone(), frame.1, frame.2));
                    Ok(Update {
                        line,
                        frame: Some(frame),
                    })
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

        fn with_chromium<T>(
            &self,
            action: impl FnOnce(&mut ChromiumProc) -> Result<T, String>,
        ) -> Result<T, String> {
            match &mut *self.engine.lock().unwrap_or_else(|p| p.into_inner()) {
                Engine::Missing { reason } => Err(reason.clone()),
                Engine::Chromium { proc } => action(proc),
            }
        }

        fn update_line(&self, tab: u32) -> Option<String> {
            match &*self.engine.lock().unwrap_or_else(|p| p.into_inner()) {
                Engine::Missing { .. } => None,
                Engine::Chromium { proc } => proc.status_line(tab),
            }
        }

        fn cache_frame(&self, tab: u32, frame: CachedFrame) {
            self.frames
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(tab, (frame.0.clone(), frame.1, frame.2));
        }

        /// Handle one LBP/1 line. Returns `false` when the connection should end.
        pub fn handle_line(&self, text: &str, out: &mut impl Write) -> bool {
            let mut parts = text.split_whitespace();
            let Some(cmd) = parts.next() else {
                return true;
            };
            let reply = match cmd {
                "HELLO" => {
                    let version = parts.next();
                    if version != Some("1") {
                        Some("ERR bad-request HELLO%201%20is%20required".to_owned())
                    } else {
                        Some(self.hello())
                    }
                }
                "PING" => Some("PONG".to_owned()),
                "QUIT" => return false,
                "VIEWPORT" => {
                    let w = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let h = parts.next().and_then(|v| v.parse::<u32>().ok());
                    match (w, h) {
                        (Some(w), Some(h))
                            if (64..=4096).contains(&w) && (64..=4096).contains(&h) =>
                        {
                            let result = self.with_chromium(|proc| {
                                proc.viewport = (w, h);
                                let tabs: Vec<(u32, String)> = proc
                                    .tabs
                                    .iter()
                                    .map(|(id, t)| (*id, t.session.clone()))
                                    .collect();
                                let params = format!(
                                    r#"{{"width":{w},"height":{h},"deviceScaleFactor":1,"mobile":false}}"#
                                );
                                for (_, session) in tabs {
                                    proc.call_session(
                                        "Emulation.setDeviceMetricsOverride",
                                        &params,
                                        Some(&session),
                                    )?;
                                }
                                Ok(())
                            });
                            match result {
                                Ok(()) => Some(format!("OK viewport={w}x{h}")),
                                Err(e) => {
                                    Some(format!("ERR bad-request {}", super::pct_encode(&e)))
                                }
                            }
                        }
                        _ => Some("ERR bad-request viewport%20must%20be%2064..4096".to_owned()),
                    }
                }
                "GO" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let url = super::pct_decode(parts.next().unwrap_or("-"));
                    match tab {
                        Some(tab) => match self.handle_go(tab, &url) {
                            Ok(update) => {
                                if let Some(frame) = update.frame {
                                    self.cache_frame(tab, frame);
                                }
                                Some(update.line)
                            }
                            Err(e) => Some(format!("ERR engine-failed {}", super::pct_encode(&e))),
                        },
                        None => Some("ERR bad-request GO%20needs%20a%20tab".to_owned()),
                    }
                }
                "NEW" => {
                    // `NEW <tab> [url]`: the client chooses the id, exactly as
                    // it does for GO, CLOSE and ACTIVATE. A broker-chosen id
                    // made the shell's tab list and the engine's disagree from
                    // the second tab onward, and every command after that went
                    // to the wrong page.
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let url = super::pct_decode(parts.next().unwrap_or("-"));
                    match tab {
                        Some(tab) => {
                            if url == "-" || url.is_empty() {
                                Some(format!(
                                    "TAB {tab} state=empty url=about%3Ablank title=New%20Tab back=0 fwd=0 secure=0"
                                ))
                            } else {
                                match self.handle_go(tab, &url) {
                                    Ok(update) => {
                                        if let Some(frame) = update.frame {
                                            self.cache_frame(tab, frame);
                                        }
                                        Some(update.line)
                                    }
                                    Err(e) => {
                                        Some(format!("ERR engine-failed {}", super::pct_encode(&e)))
                                    }
                                }
                            }
                        }
                        None => Some("ERR bad-request NEW%20needs%20a%20tab".to_owned()),
                    }
                }
                "CLOSE" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    match tab {
                        Some(tab) => {
                            let _ = self.with_chromium(|proc| proc.close_tab(tab));
                            self.frames
                                .lock()
                                .unwrap_or_else(|p| p.into_inner())
                                .remove(&tab);
                            Some(format!("CLOSED {tab}"))
                        }
                        None => Some("ERR bad-request CLOSE%20needs%20a%20tab".to_owned()),
                    }
                }
                "ACTIVATE" | "FRAME" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    match tab {
                        Some(tab) => match self.with_chromium(|proc| proc.refresh_frame(tab)) {
                            Ok(frame) => {
                                self.cache_frame(tab, frame);
                                self.update_line(tab)
                            }
                            Err(e) => Some(format!("ERR engine-failed {}", super::pct_encode(&e))),
                        },
                        None => Some("ERR bad-request ACTIVATE%20needs%20a%20tab".to_owned()),
                    }
                }
                "BACK" | "FORWARD" => {
                    let forward = cmd == "FORWARD";
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    match tab {
                        Some(tab) => {
                            let result = self.with_chromium(|proc| {
                                proc.history(tab, forward)?;
                                proc.refresh_frame(tab)
                            });
                            match result {
                                Ok(frame) => {
                                    self.cache_frame(tab, frame);
                                    self.update_line(tab)
                                }
                                Err(e) => {
                                    Some(format!("ERR engine-failed {}", super::pct_encode(&e)))
                                }
                            }
                        }
                        None => Some("ERR bad-request BACK%2FFORWARD%20need%20a%20tab".to_owned()),
                    }
                }
                "RELOAD" | "STOP" => {
                    let reload = cmd == "RELOAD";
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    match tab {
                        Some(tab) => {
                            let result = self.with_chromium(|proc| {
                                if reload {
                                    proc.reload(tab)?;
                                } else {
                                    proc.stop(tab)?;
                                }
                                proc.refresh_frame(tab)
                            });
                            match result {
                                Ok(frame) => {
                                    self.cache_frame(tab, frame);
                                    self.update_line(tab)
                                }
                                Err(e) => {
                                    Some(format!("ERR engine-failed {}", super::pct_encode(&e)))
                                }
                            }
                        }
                        None => Some("ERR bad-request RELOAD%2FSTOP%20need%20a%20tab".to_owned()),
                    }
                }
                "MOUSE" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let kind = parts.next().unwrap_or("");
                    let x = parts.next().and_then(|v| v.parse::<f64>().ok());
                    let y = parts.next().and_then(|v| v.parse::<f64>().ok());
                    let button = parts.next().unwrap_or("none");
                    let clicks = parts
                        .next()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(1);
                    let mods = parts
                        .next()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    match (tab, x, y) {
                        (Some(tab), Some(x), Some(y)) => {
                            let event = MouseEvent {
                                kind,
                                x,
                                y,
                                button,
                                clicks,
                                mods,
                            };
                            self.with_chromium(|proc| proc.mouse(tab, event))
                                .err()
                                .map(|e| format!("ERR engine-failed {}", super::pct_encode(&e)))
                        }
                        _ => Some(
                            "ERR bad-request MOUSE%20needs%20tab%20and%20coordinates".to_owned(),
                        ),
                    }
                }
                "WHEEL" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let x = parts.next().and_then(|v| v.parse::<f64>().ok());
                    let y = parts.next().and_then(|v| v.parse::<f64>().ok());
                    let dx = parts.next().and_then(|v| v.parse::<f64>().ok());
                    let dy = parts.next().and_then(|v| v.parse::<f64>().ok());
                    match (tab, x, y, dx, dy) {
                        (Some(tab), Some(x), Some(y), Some(dx), Some(dy)) => self
                            .with_chromium(|proc| proc.wheel(tab, x, y, dx, dy))
                            .err()
                            .map(|e| format!("ERR engine-failed {}", super::pct_encode(&e))),
                        _ => Some("ERR bad-request WHEEL%20needs%20five%20numbers".to_owned()),
                    }
                }
                "KEY" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let down = parts.next().unwrap_or("");
                    let key = super::pct_decode(parts.next().unwrap_or("-"));
                    let code = super::pct_decode(parts.next().unwrap_or("-"));
                    let vk = parts
                        .next()
                        .and_then(|v| v.parse::<i64>().ok())
                        .unwrap_or(0);
                    let mods = parts
                        .next()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    let text = super::pct_decode(parts.next().unwrap_or("-"));
                    match (tab, down) {
                        (Some(tab), "down" | "up") => {
                            let event = KeyEvent {
                                down: down == "down",
                                key: &key,
                                code: &code,
                                vk,
                                mods,
                                text: &text,
                            };
                            self.with_chromium(|proc| proc.key(tab, event))
                                .err()
                                .map(|e| format!("ERR engine-failed {}", super::pct_encode(&e)))
                        }
                        _ => Some("ERR bad-request KEY%20needs%20tab%20and%20down%2Fup".to_owned()),
                    }
                }
                "TEXT" => {
                    let tab = parts.next().and_then(|v| v.parse::<u32>().ok());
                    let text = super::pct_decode(parts.next().unwrap_or("-"));
                    match tab {
                        Some(tab) => self
                            .with_chromium(|proc| proc.text(tab, &text))
                            .err()
                            .map(|e| format!("ERR engine-failed {}", super::pct_encode(&e))),
                        None => Some("ERR bad-request TEXT%20needs%20a%20tab".to_owned()),
                    }
                }
                "DOWNLOADS" => {
                    let Some(dir) = downloads_dir() else {
                        return writeln!(out, "ERR downloads%20are%20not%20configured").is_ok();
                    };
                    match std::fs::read_dir(dir) {
                        Ok(entries) => {
                            let mut files = Vec::new();
                            for entry in entries.filter_map(Result::ok) {
                                let Ok(name) = entry.file_name().into_string() else {
                                    continue;
                                };
                                let Ok(metadata) = entry.metadata() else {
                                    continue;
                                };
                                // Chromium writes `<name>.crdownload` until the
                                // download is complete. Listing those would show
                                // the user a file that cannot be opened, and the
                                // importer would move a half-written document
                                // into their Downloads folder.
                                let finished = metadata.is_file()
                                    && !name.starts_with('.')
                                    && !name.ends_with(".crdownload");
                                if finished {
                                    files.push((name, metadata.len()));
                                }
                            }
                            files.sort_by_key(|(name, _)| name.to_lowercase());
                            let mut lines = vec![format!("DOWNLOADS {}", files.len())];
                            for (name, size) in files {
                                lines.push(format!("DOWNLOAD {} {size}", super::pct_encode(&name)));
                            }
                            Some(lines.join("\n"))
                        }
                        Err(error) => {
                            Some(format!("ERR {}", super::pct_encode(&error.to_string())))
                        }
                    }
                }
                "DROP" => {
                    // The shell removed an imported download from the user's
                    // folder; the staged copy has no further purpose. It is
                    // this broker's own directory, so deleting here is its own
                    // authority and nobody else's.
                    let name = super::pct_decode(parts.next().unwrap_or("-"));
                    let Some(dir) = downloads_dir() else {
                        return writeln!(out, "ERR downloads%20are%20not%20configured").is_ok();
                    };
                    // Not `DOWNLOADS 0`: a listing reply would be taken by the
                    // shell as the start of a download report and would clear
                    // the panel's status line for a reason the user never
                    // caused.
                    match drop_staged(dir, &name) {
                        Ok(()) => Some(format!("DROPPED {}", super::pct_encode(&name))),
                        Err(reason) => Some(format!("ERR {}", super::pct_encode(&reason))),
                    }
                }
                _ => Some("ERR bad-request unknown%20command".to_owned()),
            };
            let Some(reply) = reply else {
                return true;
            };
            if writeln!(out, "{reply}").is_err() {
                return false;
            }
            // A command that changed a tab is followed by its current frame.
            if let Some(rest) = reply.strip_prefix("TAB ") {
                let tab = rest
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u32>().ok());
                if let Some(tab) = tab {
                    if let Some((frame, w, h)) = self.take_frame(tab) {
                        if frame.len()
                            != usize::try_from(w)
                                .unwrap_or(0)
                                .saturating_mul(usize::try_from(h).unwrap_or(0))
                                .saturating_mul(4)
                        {
                            let _ = writeln!(out, "ERR engine-failed frame%20length%20mismatch");
                            return true;
                        }
                        let header = format!("FRAME {tab} {w} {h} {}\n", frame.len());
                        if out.write_all(header.as_bytes()).is_err()
                            || out.write_all(&frame).is_err()
                        {
                            return false;
                        }
                    }
                }
            }
            true
        }

        /// Serve one connection: read commands, write replies and frames.
        /// Returns whether the process should keep listening; `false` means the
        /// client said QUIT, which ends the broker just as it ends a
        /// socket-activated instance.
        pub(crate) fn serve(&self, stream: UnixStream) -> bool {
            let mut reader = BufReader::new(match stream.try_clone() {
                Ok(s) => s,
                Err(_) => return true,
            });
            let mut writer = stream;
            loop {
                let mut line = Vec::new();
                let taken = (&mut reader)
                    .take(64 * 1024)
                    .read_until(b'\n', &mut line)
                    .unwrap_or(0);
                if taken == 0 {
                    return true;
                }
                if line.last() != Some(&b'\n') {
                    let _ = writeln!(&mut writer, "ERR too-long line");
                    return true;
                }
                line.pop();
                let text = String::from_utf8_lossy(&line).into_owned();
                if !self.handle_line(&text, &mut writer) {
                    return false;
                }
            }
        }

        /// Bind the socket and serve until the process ends.
        ///
        /// # Errors
        /// Bind or accept errors; the caller logs and exits.
        pub fn run(&self, socket_path: &Path) -> io::Result<()> {
            let listener = bind_listener(socket_path)?;
            eprintln!("browserd: listening on {}", socket_path.display());
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => {
                        if !self.serve(s) {
                            break;
                        }
                    }
                    Err(e) => eprintln!("browserd: accept failed: {e}"),
                }
            }
            Ok(())
        }
    }

    /// Bind `socket_path` without displacing a broker that is still listening.
    ///
    /// The first version removed whatever was at the path before binding. A
    /// process that was killed leaves its socket behind, but a process that is
    /// still running leaves a socket clients are using: unlinking that one
    /// leaves the live broker accepting on an inode nobody can reach, and this
    /// process silently takes over the name. So the path is removed only when
    /// nothing answers on it, a path that is not a socket is left alone and
    /// reported, and the socket is made owner-only -- if that fails the socket
    /// just made is removed and the error returned.
    ///
    /// # Errors
    /// A live listener (`AddrInUse`), a non-socket at the path
    /// (`AlreadyExists`), and whatever bind or `chmod` raise.
    pub(super) fn bind_listener(socket_path: &Path) -> io::Result<UnixListener> {
        use std::os::unix::fs::{FileTypeExt as _, PermissionsExt as _};
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::symlink_metadata(socket_path) {
            Ok(meta) => {
                if !meta.file_type().is_socket() {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "the socket path is occupied by something that is not a socket",
                    ));
                }
                if UnixStream::connect(socket_path).is_ok() {
                    return Err(io::Error::new(
                        io::ErrorKind::AddrInUse,
                        "another broker is already listening on this socket",
                    ));
                }
                std::fs::remove_file(socket_path)?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let listener = UnixListener::bind(socket_path)?;
        if let Err(e) =
            std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))
        {
            let _ = std::fs::remove_file(socket_path);
            return Err(e);
        }
        Ok(listener)
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
            let _ = broker.serve(dup);
            return;
        }
        let broker = broker::Broker::new();
        if let Some(reason) = broker.missing_reason() {
            // In the journal before the first command, so "the browser shows
            // nothing" is answerable from `journalctl -u litric-browser@…`
            // without guessing which piece is absent.
            eprintln!("browserd: engine unavailable: {reason}");
        }
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
    use super::broker::{bind_listener, drop_staged, find_chromium, Broker};
    use super::{pct_decode, pct_encode};
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt as _;
    use std::os::unix::net::UnixStream;

    /// A scratch directory for the bind tests, tagged with this process.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("browserd-bind-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            std::fs::create_dir_all(&dir).is_ok(),
            "mkdir {}",
            dir.display()
        );
        dir
    }

    #[test]
    fn a_live_brokers_socket_is_not_replaced_and_the_socket_is_owner_only() {
        let dir = scratch("live");
        let path = dir.join("browser.sock");
        let first = bind_listener(&path).expect("first bind");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the socket must be owner-only");
        let second = bind_listener(&path).expect_err("a live socket is not replaced");
        assert_eq!(second.kind(), std::io::ErrorKind::AddrInUse);
        UnixStream::connect(&path).expect("the first listener still answers");
        drop(first);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_stale_brokers_socket_is_replaced_and_a_file_is_never_deleted() {
        let dir = scratch("stale");
        let path = dir.join("browser.sock");
        {
            // A listener that was killed: the file stays, nothing answers.
            let dead = std::os::unix::net::UnixListener::bind(&path).expect("bind");
            drop(dead);
        }
        let listener = bind_listener(&path).expect("a stale socket is replaced");
        assert_eq!(
            std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777,
            0o600
        );
        drop(listener);
        let _ = std::fs::remove_file(&path);

        let file = dir.join("important.txt");
        std::fs::write(&file, b"do not delete").expect("write");
        let err = bind_listener(&file).expect_err("a plain file is occupied");
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            std::fs::read(&file).expect("still there"),
            b"do not delete",
            "a broker must not unlink a file it did not create"
        );
    }

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
        // about pages never touch the engine, so they answer on any host.
        let reply = broker.handle_go(1, "about:blank");
        assert!(
            matches!(&reply, Ok(update) if update.line.starts_with("TAB 1 state=empty")),
            "{reply:?}"
        );
    }

    #[test]
    fn the_broker_refuses_addresses_the_policy_cannot_enforce() {
        let broker = Broker::new();
        for url in [
            "file:///etc/shadow",
            "http://example.com/",
            "javascript:alert(1)",
            "https://exa mple.com/",
        ] {
            assert!(
                broker.handle_go(1, url).is_err(),
                "{url} was accepted by the broker"
            );
        }
    }

    #[test]
    fn a_malformed_mouse_command_does_not_crash_or_reach_the_engine() {
        let broker = Broker::new();
        let mut out = Vec::new();
        assert!(broker.handle_line("MOUSE 1 down", &mut out));
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.starts_with("ERR bad-request"), "{text}");
    }

    #[test]
    fn lbp_lines_round_trip_through_the_serve_loop() {
        let (client, server) = UnixStream::pair().expect("pair");
        let broker = Broker::new();
        let worker = std::thread::spawn(move || {
            let _ = broker.serve(server);
        });
        let mut c = client;

        c.write_all(b"PING\n").unwrap();
        c.write_all(b"HELLO 1\n").unwrap();
        c.write_all(b"VIEWPORT 1280 800\n").unwrap();
        c.write_all(b"GO 1 https%3A%2F%2Fexample.com%2F\n").unwrap();
        c.write_all(b"JUNK\n").unwrap();
        c.write_all(b"QUIT\n").unwrap();

        let mut reader = BufReader::new(c);
        let mut pong = String::new();
        reader.read_line(&mut pong).unwrap();
        assert_eq!(pong.trim(), "PONG");

        let mut hello = String::new();
        reader.read_line(&mut hello).unwrap();
        assert!(
            hello.starts_with("HELLO 1 engine=") || hello.starts_with("ERR engine-unavailable"),
            "{hello}"
        );

        let mut viewport = String::new();
        reader.read_line(&mut viewport).unwrap();
        assert!(
            viewport.starts_with("OK viewport=1280x800") || viewport.starts_with("ERR "),
            "{viewport}"
        );

        let mut go = String::new();
        reader.read_line(&mut go).unwrap();
        assert!(go.starts_with("TAB ") || go.starts_with("ERR "), "{go}");

        let mut junk = String::new();
        reader.read_line(&mut junk).unwrap();
        assert_eq!(junk.trim(), "ERR bad-request unknown%20command");

        worker.join().expect("serve thread ends on QUIT");
    }

    #[test]
    fn a_frame_header_carries_the_tab_and_dimensions_the_shell_needs() {
        // The shell reads `FRAME <tab> <w> <h> <n>`. The old broker wrote
        // `FRAME <tab> <n>`, which the shell parsed as width/height and then
        // dropped after a length mismatch. This pins the shape both sides share.
        let header = format!(
            "FRAME {tab} {w} {h} {n}",
            tab = 3,
            w = 1280,
            h = 800,
            n = 1280 * 800 * 4
        );
        let fields: Vec<&str> = header.split(' ').collect();
        assert_eq!(fields, ["FRAME", "3", "1280", "800", "4096000"]);
    }

    #[test]
    fn a_stopped_page_is_not_reported_as_loaded() {
        use super::broker::stopped_state;
        assert_eq!(stopped_state(Some("complete")), "loaded");
        assert_eq!(stopped_state(Some("loading")), "stopped");
        assert_eq!(stopped_state(Some("interactive")), "stopped");
        // The evaluation failed, so nothing is known about the document.
        assert_eq!(stopped_state(None), "stopped");
    }

    fn staging(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-browserd-drop-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            std::fs::create_dir_all(&dir).is_ok(),
            "staging {}",
            dir.display()
        );
        dir
    }

    #[test]
    fn a_staged_download_can_be_dropped_by_name() {
        let dir = staging("ok");
        std::fs::write(dir.join("report.pdf"), b"payload").expect("stage");
        drop_staged(&dir, "report.pdf").expect("drops");
        assert!(!dir.join("report.pdf").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dropping_refuses_a_path_a_link_and_a_directory() {
        let dir = staging("refuse");
        std::fs::create_dir_all(dir.join("folder")).expect("folder");
        std::os::unix::fs::symlink("/etc/hostname", dir.join("link")).expect("symlink");
        assert!(drop_staged(&dir, "../escape").is_err());
        assert!(drop_staged(&dir, "..").is_err());
        assert!(drop_staged(&dir, "folder").is_err());
        assert!(drop_staged(&dir, "link").is_err());
        // Nothing was removed on the way to those refusals.
        assert!(dir.join("folder").is_dir());
        assert!(dir.join("link").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
