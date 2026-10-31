//! `litric-shell` — the LitricOS desktop.
//!
//! # Startup sequence
//!
//! ```text
//!   kernel → systemd → litric-sysd → input discovery → display discovery
//!          → renderer init → desktop state → shell ready → interactive
//! ```
//!
//! Every step is optional except the last two. No display means headless mode;
//! no input means a desktop you can look at but not use; no daemon means facts
//! read straight from `/proc`. **Each fallback is reported, never silent** —
//! the status file and the console say which path was taken and why, because
//! "the desktop came up" and "the desktop came up without a keyboard" are
//! different outcomes and must not look alike.
//!
//! # The loop
//!
//! ```text
//!   poll(frame budget) ──► handle ──► damage ──► draw ──► present(damage rows)
//!         ▲                                                      │
//!         └────────── blocks; input wakes it instantly ──────────┘
//! ```
//!
//! With nothing happening the loop draws nothing and presents nothing: it sits
//! in `recv_timeout`. That is what Adaptive Idle actually is — the budget only
//! bounds how long the compositor sleeps, and a keypress arrives on its own
//! thread and wakes it at once, so input latency never changes with it.

#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use litric_design::{Theme, ThemeKind};
use litric_input::backend::InputSource;
use litric_input::keys::layout_by_name;
use litric_locale::Regional;
use litric_shell::canvas::{Canvas, Rect, Repaint};
use litric_shell::desktop;
use litric_shell::facts::{ask, field};
use litric_shell::fb::PixelFormat;
use litric_shell::present::{self, Choice};
use litric_shell::shell::{DesktopMode, Effect, EnvRow, Priority, Shell};
use litric_shell::shortcuts::App;
use litric_shell::ui::Tokens;

/// Version, reported on the console and in the status file.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where `litric-sysd` listens.
const SYSD_SOCKET: &str = "/run/litric/sysd.sock";

/// Where the shell reports what it drew.
const STATUS_PATH: &str = "/run/litric/shell.status";

/// How often the system facts are re-read, in milliseconds. Two seconds is
/// often enough for a memory readout to feel live and rare enough that it is
/// not what keeps the machine awake.
const FACTS_INTERVAL_MS: u64 = 2_000;

/// How often the status file is rewritten.
const STATUS_INTERVAL_MS: u64 = 5_000;

/// How often a pointer move that changes nothing but the arrow is presented,
/// in milliseconds.
///
/// A cursor-only step is two small rectangles, but each one is still a blit and
/// still dirties the framebuffer, and a tablet device under QEMU reports far
/// more moves than any display shows. Eight milliseconds is 125 steps a second:
/// the arrow never trails the hand by a frame the eye can see, and the work per
/// second has a ceiling instead of following the event rate.
const CURSOR_INTERVAL_MS: u64 = 8;

struct Args {
    root: PathBuf,
    /// Where the Files application's persistent storage lives. `None` means
    /// `<root>/var/lib/litric-files/root`, so a fixture root keeps its files in
    /// the fixture and the real system uses the unit's StateDirectory.
    files_root: Option<PathBuf>,
    /// The Files daemon's socket. `None` means "use the system one when it is
    /// there": a booted image always has it, a fixture host does not.
    files_socket: Option<PathBuf>,
    socket: PathBuf,
    status: PathBuf,
    png: Option<PathBuf>,
    format: Option<PixelFormat>,
    /// Which output path to take. The default prefers DRM/KMS and accepts
    /// the framebuffer, stating which one it got.
    presenter: Choice,
    theme: ThemeKind,
    theme_explicit: bool,
    size: Option<(u32, u32)>,
    scale: Option<f32>,
    once: bool,
    seconds: Option<u64>,
    help: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/"),
            files_root: None,
            files_socket: None,
            socket: PathBuf::from(SYSD_SOCKET),
            status: PathBuf::from(STATUS_PATH),
            png: None,
            format: None,
            presenter: Choice::Auto,
            theme: ThemeKind::Dark,
            theme_explicit: false,
            size: None,
            scale: None,
            once: false,
            seconds: None,
            help: false,
        }
    }
}

const USAGE: &str = "\
litric-shell — the LitricOS desktop

USAGE:
    litric-shell                      run the desktop
    litric-shell --once               draw one frame and exit
    litric-shell --png FILE           render to a PNG instead of a display
    litric-shell --seconds N          run for N seconds, then exit

OPTIONS:
    --png FILE        Render to a PNG instead of the display.
    --size WxH        Canvas size for --png (default 1280x800).
    --scale N         Logical-to-physical scale factor (default from the display).
    --theme NAME      dark, light, high-contrast-dark, high-contrast-light.
    --format NAME     Framebuffer byte order: xrgb8888, xbgr8888, rgb565.
    --presenter NAME  Output path: auto (default), drm, fbdev. drm needs a
                      KMS card and the native helper; fbdev never starts one.
    --socket PATH     litric-sysd socket (default /run/litric/sysd.sock).
    --status PATH     Where to write the status report.
    --root DIR        Where /proc, /sys and /dev are rooted. For testing.
    --files-root DIR  Where the Files application keeps user files. Defaults to
                      <root>/var/lib/litric-files/root.
    --files-socket P  The Files daemon's socket (default /run/litric/files.sock).
                      Given explicitly, the shell insists on the daemon; left
                      out, it uses the daemon when the socket exists and opens
                      the files itself when it does not.
    --once            Draw a single frame and exit.
    --seconds N       Exit after N seconds. For automated boot tests.
    -h, --help        This text.
";

fn parse_args<I: Iterator<Item = String>>(mut it: I) -> Result<Args, String> {
    let mut a = Args::default();
    while let Some(arg) = it.next() {
        let mut need = |what: &str| -> Result<String, String> {
            it.next().ok_or_else(|| format!("{what} needs a value"))
        };
        match arg.as_str() {
            "--png" => a.png = Some(PathBuf::from(need("--png")?)),
            "--size" => {
                let v = need("--size")?;
                let (w, h) = v
                    .split_once('x')
                    .ok_or_else(|| format!("--size {v} is not WxH"))?;
                let w: u32 = w.parse().map_err(|_| format!("bad width in {v}"))?;
                let h: u32 = h.parse().map_err(|_| format!("bad height in {v}"))?;
                if w == 0 || h == 0 {
                    return Err(format!("--size {v} has a zero dimension"));
                }
                a.size = Some((w, h));
            }
            "--scale" => {
                let v = need("--scale")?;
                let s: f32 = v
                    .parse()
                    .map_err(|_| format!("--scale {v} is not a number"))?;
                if !(0.25..=8.0).contains(&s) {
                    return Err(format!("--scale {v} is out of range"));
                }
                a.scale = Some(s);
            }
            "--theme" => {
                let v = need("--theme")?;
                a.theme = ThemeKind::all()
                    .iter()
                    .copied()
                    .find(|t| t.token() == v)
                    .ok_or_else(|| format!("no such theme: {v}"))?;
                a.theme_explicit = true;
            }
            "--format" => {
                let v = need("--format")?;
                a.format =
                    Some(PixelFormat::parse(&v).ok_or_else(|| format!("no such format: {v}"))?);
            }
            "--presenter" => {
                let v = need("--presenter")?;
                a.presenter = Choice::parse(&v).ok_or_else(|| format!("no such presenter: {v}"))?;
            }
            "--socket" => a.socket = PathBuf::from(need("--socket")?),
            "--status" => a.status = PathBuf::from(need("--status")?),
            "--root" => a.root = PathBuf::from(need("--root")?),
            "--files-root" => a.files_root = Some(PathBuf::from(need("--files-root")?)),
            "--files-socket" => a.files_socket = Some(PathBuf::from(need("--files-socket")?)),
            "--once" => a.once = true,
            "--seconds" => {
                let v = need("--seconds")?;
                a.seconds = Some(
                    v.parse()
                        .map_err(|_| format!("--seconds {v} is not a number"))?,
                );
            }
            "-h" | "--help" => a.help = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(a)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Whether a daemon reply actually confirms a password verification.
///
/// `OK n` is a transport frame, not an answer: without `applied=true` for
/// `auth.verify` there is no evidence that the helper ran and accepted the
/// password. Anything else — an empty body, a changed action name, a truncated
/// reply — is a refusal, because fail-closed is the only useful direction for a
/// lock screen.
fn reply_confirms_verification(reply: &str) -> bool {
    let lines: Vec<&str> = reply.lines().collect();
    field(&lines, "applied") == Some("true") && field(&lines, "action") == Some("auth.verify")
}

/// The `retry_after=<seconds>` field of a lockout reply, if it carries one.
///
/// The daemon's detail is a sentence plus that one machine-readable field. A
/// missing, empty or non-numeric field is `None`, and the caller falls back to
/// showing the sentence and its own backoff rather than inventing a wait.
fn lockout_seconds(reply: &str) -> Option<u64> {
    reply
        .split("retry_after=")
        .nth(1)?
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

/// The readable part of a lockout reply, without the `retry_after` field.
fn lockout_message(reply: &str) -> &str {
    reply.split("; retry_after=").next().unwrap_or(reply)
}

/// Whether an account name can be placed in a request line unmodified.
fn usable_lock_account(user: &str) -> bool {
    !user.is_empty()
        && !user.chars().any(char::is_whitespace)
        && !user.chars().any(char::is_control)
}

/// Take whatever the notification queue is holding into the centre.
///
/// `0` covers both "nothing was queued" and "no queue answered". The second case
/// is normal — before this daemon exists, and for the second it takes to restart
/// — and a desktop that put an error banner on screen every time a helper
/// restarted would teach its user to dismiss banners without reading them.
///
/// The count goes into the status file, because "four notifications arrived while
/// the desktop was down" is a fact that can be checked from outside the shell and
/// a claim that cannot is one nobody should have to take on trust.
fn drain_inbox(session: &mut Session) -> usize {
    let Ok(notes) = litric_shell::inbox::take_default() else {
        return 0;
    };
    let adopted = session.shell.adopt_inbox(notes);
    let line = format!("inbox={adopted}");
    match session
        .lines
        .iter_mut()
        .find(|existing| existing.starts_with("inbox="))
    {
        Some(existing) => *existing = line,
        None => session.lines.push(line),
    }
    adopted
}

/// The environments, from the daemon. An empty list means the daemon did not
/// answer, and the interface says so rather than showing fabricated rows.
fn gather_environments(args: &Args) -> Vec<EnvRow> {
    // Asked every two seconds. It used to print the whole reply to the console
    // each time, which put ten lines a second into the journal and onto the
    // serial console of an idle machine; an unanswered query is already visible
    // as the "Unavailable" row in the interface.
    let Some(reply) = ask(&args.socket, "ENV LIST") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in reply.lines() {
        let Some(rest) = line.strip_prefix("env=") else {
            continue;
        };
        let mut token = "";
        let mut state = "";
        for (i, part) in rest.split_whitespace().enumerate() {
            if i == 0 {
                token = part;
            } else if let Some(v) = part.strip_prefix("state=") {
                state = v;
            }
        }
        let (title, hotkey) = match token {
            "windows" => ("Windows 11", "Win+1"),
            "litric" => ("LitricOS", "Win+2"),
            "linux" => ("Linux", "Win+3"),
            other => (other, ""),
        };
        out.push(EnvRow {
            token: token.to_owned(),
            title: title.to_owned(),
            state: state.to_owned(),
            active: state == "running",
            hotkey: hotkey.to_owned(),
        });
    }
    out
}

/// Stop the kernel's framebuffer console from drawing over the desktop.
fn take_console(root: &Path) -> String {
    let dir = root.join("sys/class/vtconsole");
    let Ok(entries) = fs::read_dir(&dir) else {
        return "absent".to_owned();
    };
    let mut released = 0u32;
    let mut tried = 0u32;
    for e in entries.flatten() {
        let Ok(name) = fs::read_to_string(e.path().join("name")) else {
            continue;
        };
        if !name.contains("frame buffer") {
            continue;
        }
        tried = tried.saturating_add(1);
        if fs::write(e.path().join("bind"), b"0\n").is_ok() {
            released = released.saturating_add(1);
        }
    }
    if tried == 0 {
        "no-fbcon".to_owned()
    } else if released == tried {
        format!("released={released}")
    } else {
        format!("released={released}/{tried} (permission?)")
    }
}

/// Where the Wayland socket lives.
///
/// `XDG_RUNTIME_DIR` when the session set one (systemd always does). The
/// fallback exists so a developer running the shell from a terminal still gets
/// a socket, and it is made 0700 explicitly: a Wayland socket is an
/// unauthenticated door to the compositor, and the directory is the only lock
/// on it.
fn shell_runtime_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(dir);
        if path.is_absolute() && path.is_dir() {
            return path;
        }
    }
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
    let dir = home.join(".litric").join("run");
    let _ = fs::create_dir_all(&dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }
    dir
}

fn report(path: &Path, lines: &[String]) {
    for l in lines {
        println!("litric.shell.{l}");
    }
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut f) = fs::File::create(path) {
        for l in lines {
            let _ = writeln!(f, "{l}");
        }
    }
}

/// Apply the effects the shell asked for.
///
/// Anything with no subsystem behind it becomes a notification saying so. That
/// is the difference between a shortcut that appears to work and one that is
/// honest: pressing the key does something visible either way, and only one of
/// them is a lie.
/// How much one volume step changes, in percent.
///
/// Five, the step a keyboard key and a panel button both take. It is applied to
/// the *measured* level, so repeated presses track the machine rather than a
/// number this process accumulated on its own.
const VOLUME_STEP: i16 = 5;

/// The level one step from `current` lands on, clamped to what a sink accepts.
fn stepped_level(current: u8, step: i8) -> u8 {
    let delta = VOLUME_STEP.saturating_mul(i16::from(step));
    let next = i16::from(current).saturating_add(delta).clamp(0, 100);
    u8::try_from(next).unwrap_or(current)
}

/// Ask the system daemon to change the volume, and say what it answered.
///
/// Three outcomes, three sentences: the daemon applied it (the panel shows the
/// new level, and the next facts poll replaces it with the level the sound
/// server read back), the daemon refused it (its own reason is shown), or the
/// daemon could not be reached at all. There is no fourth case where the
/// interface claims a change it did not make.
fn apply_volume(args: &Args, shell: &mut Shell, step: i8) {
    let known = shell.quick.volume;
    let request = if step == 0 {
        format!(
            "AUDIO MUTE {}",
            if shell.quick.muted { "off" } else { "on" }
        )
    } else {
        // No measurement yet: the buttons are disabled in the panel, and a
        // keyboard shortcut that arrives anyway starts from half, which is
        // stated in the note rather than silently assumed.
        format!("AUDIO VOLUME {}", stepped_level(known.unwrap_or(50), step))
    };
    let Some(reply) = ask(&args.socket, &request) else {
        shell.quick.audio_note = "litric-sysd did not answer: nothing was changed.".to_owned();
        shell.notify(
            "Audio",
            "The system service did not answer",
            "The volume was not changed.",
            Priority::High,
            Some(6_000),
        );
        return;
    };
    let lines: Vec<&str> = reply.lines().collect();
    if field(&lines, "applied") == Some("true") {
        if step == 0 {
            shell.quick.muted = !shell.quick.muted;
            shell.quick.audio_note = if shell.quick.muted {
                "Muted, as the sound server confirmed.".to_owned()
            } else {
                "Unmuted, as the sound server confirmed.".to_owned()
            };
        } else if let Some(next) = field(&lines, "percent") {
            // "Asked", not "set": what the panel shows is the level the sound
            // server reads back, so the note must not claim the number in it as
            // the machine's own state.
            shell.quick.audio_note =
                format!("Asked the sound server for {next}%; the level it reads back is what the panel shows.");
        } else {
            shell.quick.audio_note = "Changed, and the level is being read back.".to_owned();
        }
        // Show the requested level at once and let the next sample correct it.
        // A panel that kept the old number until the poll would look like the
        // button had done nothing.
        if step != 0 {
            if let Some(current) = known {
                shell.quick.volume = Some(stepped_level(current, step));
            }
        }
    } else {
        let detail = field(&lines, "detail").unwrap_or("the action did not happen");
        shell.quick.audio_note = format!("Refused: {detail}");
        shell.notify(
            "Audio",
            "The volume was not changed",
            detail,
            Priority::High,
            Some(6_000),
        );
    }
}

/// Ask the daemon where the sink's level and the radio's power stand.
///
/// Two questions, two answers, and no third case where the interface keeps
/// showing a number nobody measured: a question that cannot be answered leaves
/// the panel saying so, and the note under it names the tool that was missing.
fn read_panel_state(args: &Args, shell: &mut Shell) {
    match ask(&args.socket, "AUDIO LEVEL") {
        Some(reply) => {
            let lines: Vec<&str> = reply.lines().collect();
            apply_measured_volume(shell, &lines);
        }
        None => {
            shell.quick.audio_note = "litric-sysd did not answer: the level is unknown.".to_owned();
        }
    }
    match ask(&args.socket, "BT STATUS") {
        Some(reply) => {
            let lines: Vec<&str> = reply.lines().collect();
            match field(&lines, "bt.powered") {
                Some("1") => shell.quick.bt_powered = Some(true),
                Some("0") => shell.quick.bt_powered = Some(false),
                _ => {
                    let detail = field(&lines, "detail").unwrap_or("no power state was reported");
                    shell.quick.bt_note = format!("the radio could not be read: {detail}");
                }
            }
        }
        None => {
            shell.quick.bt_note =
                "litric-sysd did not answer: the radio state is unknown.".to_owned();
        }
    }
}

/// Take the level and mute flag out of a reply, if it carries them.
///
/// The line is `audio.level volume=60 muted=0` -- a name followed by
/// `key=value` pairs, not a single `key=value` -- so it is split on its own
/// prefix rather than read with [`field`], which would look for
/// `audio.level=...` and find nothing.
fn apply_measured_volume(shell: &mut Shell, lines: &[&str]) {
    let Some(rest) = lines.iter().find_map(|l| l.strip_prefix("audio.level ")) else {
        let detail = field(lines, "detail").unwrap_or("no level was reported");
        shell.quick.audio_note = format!("the level could not be read: {detail}");
        return;
    };
    for kv in rest.split_whitespace() {
        if let Some(v) = kv.strip_prefix("volume=") {
            if let Some(level) = v.parse::<u8>().ok().filter(|v| *v <= 100) {
                shell.quick.volume = Some(level);
            }
        } else if let Some(v) = kv.strip_prefix("muted=") {
            shell.quick.muted = v == "1";
        }
    }
}

/// Bring one interface up or down, and report what the machine said.
///
/// A change with three outcomes and three sentences, the same shape as the
/// volume's: applied (the next facts sample shows the new link state, because
/// the state is read from sysfs and not from this process), refused with the
/// daemon's own reason, or unreachable.
fn apply_link(args: &Args, shell: &mut Shell, iface: &str, up: bool) {
    let request = format!("NET LINK {iface} {}", if up { "up" } else { "down" });
    match ask(&args.socket, &request) {
        Some(reply) => {
            let lines: Vec<&str> = reply.lines().collect();
            if field(&lines, "applied") == Some("true") {
                shell.quick.net_note = format!(
                    "{} was asked to come {}; the state below is read from the kernel.",
                    iface,
                    if up { "up" } else { "down" }
                );
            } else {
                let detail = field(&lines, "detail").unwrap_or("the link was not changed");
                shell.quick.net_note = format!("Refused: {detail}");
                shell.notify(
                    "Network",
                    "The link was not changed",
                    detail,
                    Priority::High,
                    Some(6_000),
                );
            }
        }
        None => {
            shell.quick.net_note = "litric-sysd did not answer: nothing was changed.".to_owned();
            shell.notify(
                "Network",
                "The system service did not answer",
                "The link was not changed.",
                Priority::High,
                Some(6_000),
            );
        }
    }
}

/// Ask `systemd-networkd` to re-read one interface's configuration.
fn apply_reconfigure(args: &Args, shell: &mut Shell, iface: &str) {
    let request = format!("NET RECONFIGURE {iface}");
    match ask(&args.socket, &request) {
        Some(reply) => {
            let lines: Vec<&str> = reply.lines().collect();
            if field(&lines, "applied") == Some("true") {
                shell.quick.net_note = format!("{iface} was asked to re-read its configuration.");
            } else {
                let detail =
                    field(&lines, "detail").unwrap_or("the interface was not reconfigured");
                shell.quick.net_note = format!("Refused: {detail}");
                shell.notify(
                    "Network",
                    "The interface was not reconfigured",
                    detail,
                    Priority::High,
                    Some(6_000),
                );
            }
        }
        None => {
            shell.quick.net_note = "litric-sysd did not answer: nothing was changed.".to_owned();
        }
    }
}

/// Turn the radio on or off, and read the machine's answer back.
///
/// The switch does not move the panel on its own: what the panel shows is what
/// BlueZ says after the change, so a radio that refuses to come up leaves the
/// switch where the machine is rather than where the click was.
fn apply_bluetooth_power(args: &Args, shell: &mut Shell, on: bool) {
    let request = format!("BT POWER {}", if on { "on" } else { "off" });
    match ask(&args.socket, &request) {
        Some(reply) => {
            let lines: Vec<&str> = reply.lines().collect();
            if field(&lines, "applied") == Some("true") {
                shell.quick.bt_note = format!(
                    "the radio was asked to turn {}; the state below is what BlueZ reports.",
                    if on { "on" } else { "off" }
                );
                // Read it back at once, so the switch answers the click in the
                // same frame instead of waiting for the next facts poll -- and
                // shows the radio's answer rather than the request.
                if let Some(confirm) = ask(&args.socket, "BT STATUS") {
                    let confirm = confirm.lines().collect::<Vec<&str>>();
                    match field(&confirm, "bt.powered") {
                        Some("1") => shell.quick.bt_powered = Some(true),
                        Some("0") => shell.quick.bt_powered = Some(false),
                        _ => {}
                    }
                }
            } else {
                let detail = field(&lines, "detail").unwrap_or("the radio was not changed");
                shell.quick.bt_note = format!("Refused: {detail}");
                shell.notify(
                    "Bluetooth",
                    "The radio was not changed",
                    detail,
                    Priority::High,
                    Some(6_000),
                );
            }
        }
        None => {
            shell.quick.bt_note = "litric-sysd did not answer: nothing was changed.".to_owned();
            shell.notify(
                "Bluetooth",
                "The system service did not answer",
                "The radio was not changed.",
                Priority::High,
                Some(6_000),
            );
        }
    }
}

fn apply(args: &Args, shell: &mut Shell, effects: &[Effect]) {
    for e in effects {
        match e {
            Effect::PersistPreferences(preferences) => {
                let result = store_preferences(args, *preferences);
                if let Err(error) = &result {
                    shell.notify(
                        "Settings",
                        "Could not save preferences",
                        error,
                        Priority::High,
                        Some(8_000),
                    );
                }
                shell.finish_preferences_save(result);
            }
            Effect::PersistBrowserData => {
                let history = shell.browser.history.iter().cloned().collect::<Vec<_>>();
                if let Err(error) = store_browser_data(args, &history, &shell.browser.bookmarks) {
                    // The in-memory list is kept: a failed write must not erase
                    // what the user just did. The browser's own status line says
                    // the save failed too, because "bookmark added" next to a
                    // save error is the interface contradicting itself.
                    shell.browser.status = format!("not saved: {error}");
                    shell.notify(
                        "Browser",
                        "Could not save history or bookmarks",
                        &error,
                        Priority::High,
                        Some(8_000),
                    );
                } else if shell.browser.status.starts_with("not saved:") {
                    shell.browser.status = "saved".to_owned();
                }
            }
            Effect::SwitchEnvironment(n) => {
                let token = match n {
                    1 => "windows",
                    3 => "linux",
                    _ => "litric",
                };
                match ask(&args.socket, &format!("ENV SWITCH {token}")) {
                    Some(reply) => {
                        let lines: Vec<&str> = reply.lines().collect();
                        let outcome = field(&lines, "outcome").unwrap_or("unknown");
                        let applied = field(&lines, "applied").unwrap_or("unknown");
                        if reply.starts_with("ERR ") {
                            shell.notify(
                                "Environments",
                                "Switch refused",
                                reply.trim(),
                                Priority::High,
                                Some(6_000),
                            );
                        } else {
                            // The daemon says `applied=state-only` because
                            // litric-envd does not exist. Repeating that
                            // verbatim is the point.
                            shell.notify(
                                "Environments",
                                &format!("{token}: {outcome}"),
                                &format!("applied={applied}"),
                                Priority::Normal,
                                Some(6_000),
                            );
                        }
                        shell.environments = gather_environments(args);
                    }
                    None => {
                        shell.notify(
                            "Environments",
                            "litric-sysd did not answer",
                            "The environment manager is unreachable.",
                            Priority::High,
                            Some(6_000),
                        );
                    }
                }
            }
            Effect::SetPowerProfile(p) => {
                shell.notify(
                    "Power",
                    &format!("{} requested", p.title()),
                    "Not applied: no power-management subsystem exists yet.",
                    Priority::Normal,
                    Some(5_000),
                );
            }
            // Locking is taken only when the machine can unlock it again: a
            // verifier exists and the daemon did not refuse. When it cannot, the
            // reason shown is the daemon's own sentence -- verbatim, the same
            // rule the power path follows.
            Effect::RequestLock => {
                if !shell.lock_now() {
                    let reason = shell.lock_refusal();
                    shell.notify(
                        "Session",
                        "Locking is refused",
                        &reason,
                        Priority::High,
                        Some(8_000),
                    );
                }
            }
            // A password was typed on the lock screen. `AUTH VERIFY` is the one
            // request that carries a secret, it is sent from here and nowhere
            // else, and it is sent with `CONFIRM` because `AuthSession` is a
            // live-consent capability and the password the user just typed is
            // the answer to that prompt.
            Effect::VerifyPassword { user, secret } => {
                // The account comes from the daemon's own facts. A name with a
                // space or a newline in it would turn the request line into a
                // different request, so it is refused here as well as by the
                // daemon's parser.
                if !usable_lock_account(user) {
                    shell.lock_failed(
                        "The account to check could not be determined, so the password \
                         was not sent. The screen stays locked.",
                    );
                    continue;
                }
                let request = format!("AUTH VERIFY {user} {} CONFIRM", secret.hex());
                match ask(&args.socket, &request) {
                    // The daemon's meter has an answer with a duration in it.
                    // Its wait is what the field is closed for: the local
                    // backoff starts at one second and would send the next
                    // attempt long before a one-minute lockout is over.
                    Some(reply) if reply.starts_with("ERR locked-out") => {
                        match lockout_seconds(&reply) {
                            Some(seconds) => {
                                shell.lock_locked_out(lockout_message(&reply).trim(), seconds);
                            }
                            None => shell.lock_failed(reply.trim()),
                        }
                    }
                    Some(reply) if reply.starts_with("ERR ") => {
                        // The reply says which of "not authorised", "the helper
                        // refused" and "the helper is missing" happened, in its
                        // own words. It never contains the password, and neither
                        // does this: what is shown is the daemon's line, trimmed.
                        shell.lock_failed(reply.trim());
                    }
                    // Only an explicit `applied=true` for `auth.verify` unlocks.
                    // A truncated, empty or unrecognised reply is not a "yes":
                    // the previous code unlocked on *any* non-error line, which
                    // made a protocol change or a garbled answer a bypass.
                    Some(reply) => {
                        if reply_confirms_verification(&reply) {
                            shell.unlock();
                        } else {
                            shell.lock_failed(
                                "The system daemon's answer did not confirm the password, \
                                 so the screen stays locked.",
                            );
                        }
                    }
                    None => shell.lock_failed(
                        "The system daemon did not answer, so this password could \
                         not be checked. The screen stays locked.",
                    ),
                }
            }
            // The user answered a prompt naming this action, so the daemon is
            // asked for real: `POWER <verb> CONFIRM` is the only form that
            // carries the confirmation the broker's live-consent rule needs,
            // and it is sent from here and nowhere else in the shell.
            Effect::Power(command) => {
                let request = command.request_line();
                match ask(&args.socket, &request) {
                    // The daemon refused, or it ran the tool and the tool
                    // refused. Either way the machine is still up, and the
                    // reason it gives is the answer to show — verbatim, because
                    // a paraphrase of `needs-live-consent` helps nobody.
                    Some(reply) if reply.starts_with("ERR ") => {
                        shell.notify(
                            "Power",
                            "The machine did not act",
                            reply.trim(),
                            Priority::High,
                            Some(10_000),
                        );
                    }
                    Some(reply) => {
                        let lines: Vec<&str> = reply.lines().collect();
                        shell.notify(
                            "Power",
                            &format!(
                                "{} requested",
                                field(&lines, "action").unwrap_or(command.title())
                            ),
                            &format!("applied={}", field(&lines, "applied").unwrap_or("unknown")),
                            Priority::Normal,
                            Some(10_000),
                        );
                    }
                    None => {
                        shell.notify(
                            "Power",
                            "litric-sysd did not answer",
                            "Nothing was asked of the machine: the system service is unreachable.",
                            Priority::High,
                            Some(10_000),
                        );
                    }
                }
            }
            Effect::Volume(step) => apply_volume(args, shell, *step),
            // The panel's two questions, asked together and only when somebody
            // is looking: both cost a process and both answers are measurements.
            Effect::ReadPanelState => {
                read_panel_state(args, shell);
            }
            Effect::SetLink { iface, up } => apply_link(args, shell, iface, *up),
            Effect::ReconfigureLink(iface) => apply_reconfigure(args, shell, iface),
            Effect::SetBluetoothPower(on) => apply_bluetooth_power(args, shell, *on),
            Effect::Brightness(_) => {
                shell.notify(
                    "Display",
                    "Brightness is not controllable",
                    "This display exposes no backlight.",
                    Priority::Low,
                    Some(4_000),
                );
            }
            // Opening the browser starts the engine session (ADR-0018): the
            // dock click and the launcher both land here, and a browser that
            // needed a second click on a menu item to reach the web was a
            // browser that looked broken. The status line records the real
            // answer when the broker socket is not there.
            Effect::Launched(app) if *app == App::Browser => {
                let _ = shell.browser_start_session();
            }
            _ => {}
        }
    }
}

/// Minimal PNG writer, so `--png` needs no image library.
fn write_png(path: &Path, canvas: &Canvas) -> std::io::Result<()> {
    fn crc(bytes: &[u8]) -> u32 {
        let mut table = [0u32; 256];
        let mut i = 0usize;
        while i < 256 {
            let mut c = i as u32;
            let mut k = 0u32;
            while k < 8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
                k = k.saturating_add(1);
            }
            if let Some(slot) = table.get_mut(i) {
                *slot = c;
            }
            i = i.saturating_add(1);
        }
        let mut c = 0xFFFF_FFFFu32;
        for b in bytes {
            let idx = ((c ^ u32::from(*b)) & 0xFF) as usize;
            c = table.get(idx).copied().unwrap_or(0) ^ (c >> 8);
        }
        c ^ 0xFFFF_FFFF
    }

    fn zlib_stored(raw: &[u8]) -> Vec<u8> {
        let mut out = vec![0x78, 0x01];
        let mut pos = 0usize;
        while pos < raw.len() {
            let n = (raw.len().saturating_sub(pos)).min(65_535);
            let last = u8::from(pos.saturating_add(n) >= raw.len());
            out.push(last);
            let n16 = n as u16;
            out.extend_from_slice(&n16.to_le_bytes());
            out.extend_from_slice(&(!n16).to_le_bytes());
            if let Some(chunk) = raw.get(pos..pos.saturating_add(n)) {
                out.extend_from_slice(chunk);
            }
            pos = pos.saturating_add(n);
        }
        let (mut a, mut b) = (1u32, 0u32);
        for byte in raw {
            a = (a.saturating_add(u32::from(*byte))) % 65_521;
            b = (b.saturating_add(a)) % 65_521;
        }
        out.extend_from_slice(&((b << 16) | a).to_be_bytes());
        out
    }

    let w = canvas.width();
    let h = canvas.height();
    let mut raw = Vec::with_capacity(
        ((w as usize).saturating_mul(3).saturating_add(1)).saturating_mul(h as usize),
    );
    for y in 0..h {
        raw.push(0u8);
        for x in 0..w {
            let p = canvas.get(i64::from(x), i64::from(y)).unwrap_or(0);
            raw.push(((p >> 16) & 0xFF) as u8);
            raw.push(((p >> 8) & 0xFF) as u8);
            raw.push((p & 0xFF) as u8);
        }
    }

    let mut png: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let chunk = |tag: &[u8; 4], payload: &[u8], out: &mut Vec<u8>| {
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(tag);
        out.extend_from_slice(payload);
        let mut c = tag.to_vec();
        c.extend_from_slice(payload);
        out.extend_from_slice(&crc(&c).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &ihdr, &mut png);
    chunk(b"IDAT", &zlib_stored(&raw), &mut png);
    chunk(b"IEND", &[], &mut png);
    fs::write(path, &png)
}

/// The shell and the compositor that draws it.
struct Session {
    shell: Shell,
    compositor: desktop::Compositor,
    lines: Vec<String>,
}

fn preference_path(args: &Args) -> PathBuf {
    args.root.join("var/lib/litric-shell/settings.conf")
}

/// Where the Files application's user files live.
///
/// `--files-root` names it outright; otherwise it is the `litric-files` state
/// directory the shell unit declares, under whatever root this process was
/// started against. A fixture root therefore keeps its files inside the fixture
/// instead of writing to the real `/var`.
fn files_storage_root(args: &Args) -> PathBuf {
    args.files_root
        .clone()
        .unwrap_or_else(|| args.root.join("var/lib/litric-files/root"))
}

fn browser_data_path(args: &Args) -> PathBuf {
    // The same directory the unit's `StateDirectory=litric-browser` opens for
    // writing. It used to be a path no unit declared, so under
    // `ProtectSystem=strict` every save was EROFS and the history was lost on
    // the next restart while the interface said nothing about it.
    args.root.join("var/lib/litric-browser/browser-data.conf")
}

/// Read the history and bookmarks, and say when the file could not be read.
///
/// The old version returned an empty list for *every* error, so a permission
/// denial, a corrupt file and a first run were indistinguishable. They are
/// three different states: a missing file is a normal first run; a read error
/// must be reported; a malformed line must not throw away the lines that did
/// parse, and must not delete the file.
fn load_browser_data(args: &Args) -> (Vec<String>, Vec<String>, Option<String>) {
    let path = browser_data_path(args);
    let mut history = Vec::new();
    let mut bookmarks = Vec::new();
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return (history, bookmarks, None);
        }
        Err(e) => {
            return (
                history,
                bookmarks,
                Some(format!(
                    "Browser data at {} could not be read ({e}); the file was left in \
                     place and this session starts with empty history and bookmarks.",
                    path.display()
                )),
            );
        }
    };
    let mut unreadable = 0usize;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            unreadable = unreadable.saturating_add(1);
            continue;
        };
        let Ok(value) = litric_core::pct::decode(value) else {
            unreadable = unreadable.saturating_add(1);
            continue;
        };
        // A store written before BUG-0036 holds whatever was typed. Adopting
        // the parser's canonical text on load means a file carrying
        // `https://EXAMPLE.com` and `https://example.com:443` becomes one page,
        // instead of two rows that only collapse once each has been visited.
        let value = litric_shell::apps::BrowserApp::canonical_address(&value);
        if value.len() > 4096 || value.is_empty() || value == "litric://about" {
            continue;
        }
        let values = match key {
            "history" => &mut history,
            "bookmark" => &mut bookmarks,
            _ => continue,
        };
        if !values.contains(&value) {
            values.push(value);
        }
    }
    history.truncate(200);
    bookmarks.truncate(200);
    let warning = (unreadable > 0).then(|| {
        format!(
            "{unreadable} line(s) in {} could not be understood. The readable entries \
             were kept and the file was left as it was.",
            path.display()
        )
    });
    (history, bookmarks, warning)
}

fn store_browser_data(args: &Args, history: &[String], bookmarks: &[String]) -> Result<(), String> {
    use std::fs::OpenOptions;

    let path = browser_data_path(args);
    let parent = path
        .parent()
        .ok_or_else(|| "browser data path has no parent directory".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "cannot create the browser data directory {}: {error}",
            parent.display()
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|error| {
            format!(
                "cannot secure the browser data directory {}: {error}",
                parent.display()
            )
        })?;
    }

    // A unique suffix, not just the pid. A process killed between creating its
    // temporary file and renaming it leaves the name behind; with only the pid,
    // a recycled pid would collide with that leftover and every later save would
    // fail with `AlreadyExists` -- a permanent loss caused by one crash. The
    // timestamp makes the chance of collision negligible without deleting any
    // file this run did not create.
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let temp = parent.join(format!(
        "browser-data.conf.tmp.{}.{stamp}",
        std::process::id()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<(), String> {
        let mut file = options.open(&temp).map_err(|error| {
            format!(
                "cannot create temporary browser data {}: {error}",
                temp.display()
            )
        })?;
        for url in history.iter().take(200) {
            writeln!(file, "history={}", litric_core::pct::encode(url))
                .map_err(|error| format!("cannot write browser history: {error}"))?;
        }
        for url in bookmarks.iter().take(200) {
            writeln!(file, "bookmark={}", litric_core::pct::encode(url))
                .map_err(|error| format!("cannot write browser bookmarks: {error}"))?;
        }
        file.sync_all()
            .map_err(|error| format!("cannot sync browser data: {error}"))?;
        fs::rename(&temp, &path).map_err(|error| {
            format!("cannot commit browser data to {}: {error}", path.display())
        })?;
        // The rename is what makes the new file visible; the directory sync is
        // what makes it survive a power cut. A filesystem that refuses to sync a
        // directory has still committed the rename, so its complaint is not
        // turned into a failed save.
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn load_preferences(args: &Args, regional: &Regional) -> litric_shell::settings::Preferences {
    use litric_shell::i18n::Language;
    use litric_shell::prefstore::Store;
    use litric_shell::settings::Preferences;

    let detected_language = if regional.locale.tag.starts_with("en") {
        Language::English
    } else {
        // The product default is Thai when no supported UI locale was detected.
        Language::Thai
    };
    let defaults = Preferences::new(args.theme, detected_language);
    let store = Store::new(preference_path(args));
    let mut preferences = match store.load() {
        Ok(Some(preferences)) => preferences,
        // No file means this account has never finished setup.
        Ok(None) => Preferences {
            onboarded: false,
            ..defaults
        },
        // A record that exists but cannot be read is reported, not treated as a
        // fresh account: silently resetting preferences is how a user loses
        // their desktop to a permissions mistake. Setup stays finished, the
        // defaults are used, and the reason is on stderr.
        Err(error) => {
            eprintln!("litric-shell: {error}; using defaults");
            Preferences {
                onboarded: true,
                ..defaults
            }
        }
    };
    if args.theme_explicit {
        preferences.theme = args.theme;
    }
    preferences
}

fn store_preferences(
    args: &Args,
    preferences: litric_shell::settings::Preferences,
) -> Result<(), String> {
    litric_shell::prefstore::Store::new(preference_path(args)).save(preferences)
}

fn start(args: &Args, width: u32, height: u32, scale: f32) -> Session {
    let _ = litric_shell::font::initialize();
    let regional = Regional::detect(&args.root, |k| std::env::var(k).ok());
    let preferences = load_preferences(args, &regional);
    let theme = Theme::of(preferences.theme).with_accent(preferences.accent);
    let tokens = Tokens::new(theme, scale);
    let mut shell = Shell::new(tokens, width as f32, height as f32, regional);
    shell.sysd_socket = args.socket.clone();
    shell.facts_root = args.root.clone();
    shell.load_preferences(preferences);
    // The programs this machine actually has, validated before they are
    // offered. Discovery is bounded; the directories it read are watched from
    // here on, so an application or theme installed while the session runs
    // reaches the launcher without a restart.
    shell.load_desktop_apps_from(
        &litric_shell::desktop_entry::data_dirs(),
        &litric_shell::themeicons::theme_dirs(),
        0,
    );
    let (history, bookmarks, browser_warning) = load_browser_data(args);
    shell.browser.history = history.into();
    shell.browser.bookmarks = bookmarks;
    if let Some(warning) = browser_warning {
        // Reported once, at startup, rather than swallowed: a broken store read
        // as a first run is how history disappears without anyone noticing.
        shell.notify(
            "Browser",
            "Saved browser data could not be read completely",
            &warning,
            Priority::High,
            None,
        );
    }
    // The user's files live in a dedicated persistent directory. The old code
    // used the boot root itself, so `/Documents` meant the real root filesystem:
    // read-only erofs on the image, and outside the shell unit's writable paths
    // even on a test host.
    let files_root = files_storage_root(args);
    shell.files_root = files_root.clone();
    // Who holds the user's documents (ADR-0019). On a booted machine the daemon
    // does: the shell asks it over LFP/1 and never opens a document for writing
    // itself. On a fixture host there is no daemon, and the shell opens the
    // directory directly, which is what the test suite exercises.
    //
    // An explicit --files-socket is an instruction, not a hint: if the daemon
    // is not answering, the Files application is reported as unavailable rather
    // than quietly reopening the storage in this process. Falling back would be
    // the one thing this change exists to stop.
    // Who holds the user's documents (ADR-0019). On a booted machine the daemon
    // does: the shell asks over LFP/1 and never opens a document for writing
    // itself. On a fixture host there is no daemon, and the shell opens the
    // directory directly, which is what the test suite exercises.
    //
    // An explicit --files-socket is an instruction, not a hint: when the daemon
    // is named and does not answer, the Files application is reported as
    // unavailable rather than quietly reopening the storage in this process.
    // Falling back would undo the only thing this arrangement exists for.
    #[cfg(unix)]
    let files_socket = args.files_socket.clone().unwrap_or_else(|| {
        std::env::var_os("LITRIC_FILES_SOCKET")
            .map_or_else(|| PathBuf::from("/run/litric/files.sock"), PathBuf::from)
    });
    #[cfg(unix)]
    let want_daemon = args.files_socket.is_some() || files_socket.exists();
    let mut files_status: Option<String> = None;
    if !args.root.is_dir() {
        // A fixture root that was never created is a root with nothing in it,
        // not an invitation to create `no-such-root/var/...` in the caller's
        // working directory.
        shell.files.error = Some(format!(
            "the configured root {} does not exist, so no file storage was opened",
            args.root.display()
        ));
    } else {
        #[cfg(unix)]
        {
            let mut via_daemon = false;
            if want_daemon {
                // One connection for the whole session: the daemon is socket
                // activated per connection, so a folder listing must not cost
                // a process.
                let daemon = litric_shell::fsclient::Daemon::new(files_socket.clone());
                match daemon.probe() {
                    Ok(()) => {
                        shell.files_daemon = Some(daemon);
                        via_daemon = true;
                        files_status =
                            Some(format!("files=daemon socket={}", files_socket.display()));
                    }
                    Err(error) => {
                        let message = format!(
                            "the file service at {} is not answering: {error}. The Files \
                             application is unavailable rather than falling back to opening \
                             the storage from this process.",
                            files_socket.display()
                        );
                        shell.files.error = Some(message.clone());
                        shell.notify(
                            "Files",
                            "File service unavailable",
                            &message,
                            Priority::High,
                            None,
                        );
                        files_status = Some(format!(
                            "files=daemon-unavailable socket={}",
                            files_socket.display()
                        ));
                    }
                }
            }
            if !via_daemon && shell.files.error.is_none() {
                files_status = Some(format!(
                    "files=direct (no daemon socket at {})",
                    files_socket.display()
                ));
                if let Err(error) = litric_shell::fsbridge::ensure_storage(&files_root) {
                    let message = format!(
                        "the file storage at {} is not ready: {error}",
                        files_root.display()
                    );
                    shell.files.error = Some(message.clone());
                    shell.notify(
                        "Files",
                        "Files storage unavailable",
                        &message,
                        Priority::High,
                        None,
                    );
                }
            }
        }
        #[cfg(not(unix))]
        if let Err(error) = litric_shell::fsbridge::ensure_storage(&files_root) {
            let message = format!(
                "the file storage at {} is not ready: {error}",
                files_root.display()
            );
            shell.files.error = Some(message);
        }
    }

    shell.refresh_facts();
    shell.environments = gather_environments(args);

    let compositor = desktop::Compositor::new(width, height, &theme, preferences.wallpaper);

    let mut lines = vec![
        format!("version={VERSION}"),
        format!("canvas={width}x{height}"),
        format!("scale={scale:.2}"),
        format!("theme={}", preferences.theme.token()),
        format!("facts.source={}", shell.facts.source),
        format!("environments={}", shell.environments.len()),
    ];
    if let Some(status) = &files_status {
        lines.push(status.clone());
    }
    lines.extend(
        shell
            .regional
            .lines(shell.clock_seconds())
            .into_iter()
            .map(|l| format!("regional.{l}")),
    );

    Session {
        shell,
        compositor,
        lines,
    }
}

/// Render one frame into the scene: everything when `full`, otherwise the
/// regions made from `damage`, each restored from the wallpaper and drawn with
/// the clip set to it. Returns what was repainted, so exactly that is
/// presented.
fn render(session: &mut Session, full: bool, damage: &[Rect]) -> Repaint {
    session.compositor.repaint(&mut session.shell, full, damage)
}

/// Whether the `n`-th over-budget frame is worth a log line.
///
/// On a machine that is over budget, *every* frame is over budget -- the
/// renderer is over budget today (BUG-0015, BUG-0018) -- so one line per frame
/// would make the journal the largest cost in the loop and bury the first
/// measurement, which is the only one that describes the onset. The first ten
/// lines say what is happening; after that only every hundredth frame reports,
/// so a long run still states that it is continuing without doing so a
/// thousand times a minute.
#[must_use]
fn should_log_over_budget(n: u64) -> bool {
    // `checked_rem` rather than `%`: the workspace denies integer arithmetic
    // side effects, and a remainder is the one place this number is divided.
    n <= 10 || n.checked_rem(100) == Some(0)
}

fn run_png(args: &Args) -> Result<Vec<String>, String> {
    let (w, h) = args.size.unwrap_or((1280, 800));
    let scale = args
        .scale
        .unwrap_or_else(|| ((w as f32) / 1280.0).clamp(0.6, 3.0));
    let mut session = start(args, w, h, scale);
    session.shell.mode = DesktopMode::Ready;
    session.shell.tick(now_ms());
    render(&mut session, true, &[]);
    let path = args.png.clone().unwrap_or_else(|| PathBuf::from("out.png"));
    write_png(&path, &session.compositor.scene)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    let mut lines = session.lines;
    lines.push(format!("png={}", path.display()));
    lines.push("target=file".to_owned());
    lines.push("drawn=1".to_owned());
    Ok(lines)
}

/// The status of a shell that found no surface it can present to.
///
/// Headless is a valid state, not a reason to omit the output line: ADR-0026
/// promises that a reader can always tell which presenter was chosen, and an
/// absent line is indistinguishable from an old or truncated status file.
fn headless_lines(why: &str) -> Vec<String> {
    vec![
        "target=none".to_owned(),
        format!("present=none({why})"),
        format!("mode=headless({why})"),
        "drawn=0".to_owned(),
    ]
}

fn run_display(args: &Args) -> Result<Vec<String>, String> {
    let console = take_console(&args.root);
    // The surface is chosen before anything is composed, because its geometry
    // decides the canvas: a mode set by the card and a framebuffer's sysfs
    // size are different sources of truth and the desktop must be built for
    // the one that is actually being scanned out.
    let opened = match present::open(&args.root, args.presenter, args.format) {
        Ok(opened) => opened,
        Err(why) => {
            // Headless is not a failure. The machine boots, the daemon runs,
            // and the shell says why there is nothing to look at.
            return Ok(headless_lines(&why));
        }
    };
    let present::Opened {
        output: mut display,
        lines: present_lines,
    } = opened;
    let (w, h) = (display.width(), display.height());
    let scale = args
        .scale
        .unwrap_or_else(|| ((w as f32) / 1280.0).clamp(0.75, 3.0));

    let mut session = start(args, w, h, scale);
    let mut input = litric_input::EvdevSource::open(&args.root, w as f32, h as f32);
    let report_in = input.report().clone();

    // The keyboard layout follows the locale, so a machine set to en-GB types
    // a pound sign where the key says pound.
    if let Some(layout) = layout_by_name(&session.shell.regional.locale.keyboard_layout) {
        input.set_layout(layout);
    }

    session.shell.mode = if report_in.has_keyboard() {
        DesktopMode::Ready
    } else if report_in.has_pointer() {
        DesktopMode::Degraded {
            reason: "no keyboard".to_owned(),
        }
    } else {
        DesktopMode::Degraded {
            reason: "no input devices".to_owned(),
        }
    };

    session.lines.extend(present_lines);
    session.lines.push(format!("fbcon={console}"));
    session.lines.extend(report_in.lines());
    // External applications need a Wayland socket in the session's runtime
    // directory. The desktop is complete without one — its own windows are
    // drawn by this process — so a failure is a status line, not an exit.
    let runtime = shell_runtime_dir();
    match session.shell.attach_clients(&runtime) {
        Ok(()) => {
            let name = session
                .shell
                .clients
                .as_ref()
                .map(|c| c.display_name().to_owned())
                .unwrap_or_default();
            session.lines.push(format!("wayland.display={name}"));
            session.lines.push(format!(
                "wayland.socket={}",
                runtime.join("wayland-0").display()
            ));
        }
        Err(why) => session.lines.push(format!("wayland=unavailable ({why})")),
    }
    session
        .lines
        .push(format!("mode={}", session.shell.mode.token()));

    if !report_in.has_keyboard() {
        session.shell.notify(
            "Input",
            "No keyboard found",
            "The desktop is running but cannot be typed on.",
            Priority::High,
            None,
        );
    }

    // What was raised while the desktop was not running. Taken before the first
    // frame, so the centre is already right when the user first opens it, and
    // deliberately not banner-ed: six interruptions at boot is not a welcome.
    let recovered = drain_inbox(&mut session);
    if recovered > 0 {
        session.shell.notify(
            "Notifications",
            &format!("{recovered} while you were away"),
            "They are in the notification centre.",
            Priority::Low,
            Some(9_000),
        );
    }

    // The boot intro starts with the first frame, so the machine never shows
    // a black screen while the desktop is being made ready.
    session.shell.start_boot_intro(now_ms());
    // The first frame, immediately: a machine that boots to a black screen for
    // a second looks broken even when it is not.
    session.shell.tick(now_ms());
    render(&mut session, true, &[]);
    let _ = session.shell.take_damage();
    display
        .present(&session.compositor.scene)
        .map_err(|e| format!("cannot present the first frame: {e}"))?;
    session.lines.push("drawn=1".to_owned());
    report(&args.status, &session.lines);

    if args.once {
        return Ok(session.lines);
    }

    // ---- the loop ---------------------------------------------------------
    let started = Instant::now();
    let mut last_facts = now_ms();
    let mut last_status = now_ms();
    let mut rows_written: u64 = 0;
    // Frame-budget accounting. `docs/PERF-budget.md` §4 gates the frame time at
    // the life state's own interval and says the shell logs a warning on every
    // over-budget frame; that warning did not exist, so the gate it names was
    // observable only by hand-timing the renderer out of tree. These two
    // counters make the claim true and put the number in the status file,
    // which is where the regression CI reads it.
    let mut frames_drawn: u64 = 0;
    let mut over_budget: u64 = 0;
    // Frame clock. Input is not a frame: a 1 kHz pointer delivers a hundred
    // damage rectangles between two presents, and painting each one composed
    // the whole desktop once per event batch. That is what made the frame rate
    // fall as the mouse moved -- the work grew with the *event* rate while the
    // display still only showed 60 frames a second. Now an event is merged into
    // the damage and the next frame is drawn on the frame clock.
    let mut last_frame = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .unwrap_or_else(Instant::now);
    // The pointer's own clock, separate from the frame clock: the arrow is not
    // a frame of the desktop, and holding it to 16 ms (or to 33 when idle) is
    // what made it feel stuck to the hand.
    let cursor_gap = Duration::from_millis(CURSOR_INTERVAL_MS);
    let mut last_cursor = Instant::now()
        .checked_sub(cursor_gap)
        .unwrap_or_else(Instant::now);

    loop {
        if let Some(limit) = args.seconds {
            if started.elapsed() >= Duration::from_secs(limit) {
                break;
            }
        }

        // How long the loop may sleep. The shell answers the question (zero
        // when it has a frame to draw, the life state's pace while something is
        // moving, and the next moment the screen can change by itself when
        // nothing is happening); the loop's own periodic duties cap that, since
        // the facts are re-sampled every FACTS_INTERVAL_MS whatever the shell
        // is doing. Input does not wait on this: `poll` returns the instant a
        // key or pointer event arrives.
        let frame_gap = Duration::from_millis(session.shell.frame_interval_ms());
        let before = now_ms();
        let mut budget =
            Duration::from_millis(session.shell.wake_after_ms(before).min(FACTS_INTERVAL_MS));
        if session.shell.is_dirty() && last_frame.elapsed() < frame_gap {
            // Wait out the rest of the frame rather than drawing again. What
            // arrives while waiting is damage, and damage is merged.
            budget = budget.max(frame_gap.saturating_sub(last_frame.elapsed()));
        }
        let events = input.poll(budget);
        let t = now_ms();

        for ev in &events {
            let effects = session.shell.handle(ev, t);
            if !effects.is_empty() {
                apply(args, &mut session.shell, &effects);
                if effects
                    .iter()
                    .any(|effect| matches!(effect, Effect::PersistPreferences(_)))
                {
                    session.compositor.retheme(
                        &session.shell.tokens.theme,
                        session.shell.settings.preferences.wallpaper,
                    );
                }
            }
        }
        session.shell.tick(t);

        if t.saturating_sub(last_facts) >= FACTS_INTERVAL_MS {
            last_facts = t;
            let before = session.shell.facts.clone();
            session.shell.refresh_facts();
            if session.shell.facts != before {
                session.shell.request_full_redraw();
            }
            // Re-query environments periodically in case daemon was not ready at startup.
            // Always re-query and update, even if empty, to detect when daemon
            // becomes available.
            // Third-party senders post while the desktop is running too. The
            // same cadence as the facts keeps the cost to one local socket
            // round trip every two seconds, bounded by the client's timeout, and
            // keeps a notification's worst-case delay at that interval rather
            // than at the next shell restart.
            drain_inbox(&mut session);
            let envs = gather_environments(args);
            if session.shell.environments != envs {
                session.shell.environments = envs;
                session.shell.request_full_redraw();
                let count = format!("environments={}", session.shell.environments.len());
                if let Some(line) = session
                    .lines
                    .iter_mut()
                    .find(|l| l.starts_with("environments="))
                {
                    *line = count;
                }
            }
        }

        // A move that changed nothing but the arrow: restore the pixels under
        // the old one, draw the new one and present two small rectangles. No
        // composition, and no frame clock -- the pointer follows the mouse,
        // which is the one thing a desktop must never make a person wait for.
        if session.shell.cursor_step_pending() {
            let waited = last_cursor.elapsed();
            if waited < cursor_gap {
                std::thread::sleep(cursor_gap.saturating_sub(waited));
            }
            last_cursor = Instant::now();
        }
        if let Some((from, to)) = session.shell.take_cursor_step() {
            if let Some(plan) = session.compositor.move_cursor(&session.shell, from, to) {
                let scene = &session.compositor.scene;
                let rows = match &plan {
                    Repaint::Full => display.present(scene),
                    Repaint::Regions(regions) => display.present_damage(scene, regions),
                }
                .map_err(|e| format!("present failed: {e}"))?;
                rows_written = rows_written.saturating_add(u64::from(rows));
                frames_drawn = frames_drawn.saturating_add(1);
                last_frame = Instant::now();
            } else {
                // The frame was not where the arrow was left -- a retheme, a
                // resize -- so compose instead of guessing, or the old arrow
                // would be left behind.
                session.shell.request_full_redraw();
            }
        }

        if session.shell.is_dirty() && last_frame.elapsed() >= frame_gap {
            last_frame = Instant::now();
            let full = session.shell.needs_full_redraw();
            let damage = session.shell.take_damage();
            // The budget is the life state's own interval -- 8 ms under a
            // dragged window, 16 ms active, 33 ms idle or locked -- measured
            // from the start of composition to the end of presentation, which
            // is the input-to-present path the gate names.
            let budget_ms = session.shell.frame_interval_ms();
            let compose_started = Instant::now();
            let plan = render(&mut session, full, &damage);
            let scene = &session.compositor.scene;
            let rows = match &plan {
                Repaint::Full => display.present(scene),
                Repaint::Regions(regions) => display.present_damage(scene, regions),
            }
            .map_err(|e| format!("present failed: {e}"))?;
            rows_written = rows_written.saturating_add(u64::from(rows));
            frames_drawn = frames_drawn.saturating_add(1);
            let took_ms = u64::try_from(compose_started.elapsed().as_millis()).unwrap_or(u64::MAX);
            if took_ms > budget_ms {
                over_budget = over_budget.saturating_add(1);
                if should_log_over_budget(over_budget) {
                    eprintln!(
                        "litric-shell: frame {frames_drawn} took {took_ms} ms, over the \
                         {budget_ms} ms budget for {} (over budget {over_budget} time(s))",
                        session.shell.life().token()
                    );
                }
            }
        }

        if t.saturating_sub(last_status) >= STATUS_INTERVAL_MS {
            last_status = t;
            let mut lines = session.lines.clone();
            lines.extend(session.shell.diagnostic_lines());
            lines.push(format!("rows_presented={rows_written}"));
            lines.push(format!("frames={}", session.shell.frames()));
            lines.push(format!("frames_drawn={frames_drawn}"));
            lines.push(format!("frames_over_budget={over_budget}"));
            report(&args.status, &lines);
        }
    }

    let mut lines = session.lines;
    lines.extend(session.shell.diagnostic_lines());
    lines.push(format!("rows_presented={rows_written}"));
    Ok(lines)
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("litric-shell: {e}");
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    if args.help {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let result = if args.png.is_some() {
        run_png(&args)
    } else {
        run_display(&args)
    };

    match result {
        Ok(lines) => {
            report(&args.status, &lines);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("litric-shell: {e}");
            report(&args.status, &[format!("error={e}"), "drawn=0".to_owned()]);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use litric_input::event::{InputEvent, Point, PointerButton};
    use litric_input::keys::{Key, Modifiers, Side};
    use litric_shell::canvas::rgb;
    use litric_shell::facts::{
        parse_audio_cards, parse_bt_adapters, parse_input_devices, parse_net_ifaces,
        parse_processes, read_battery, read_network, read_root_fs, unescape_field,
    };
    use litric_shell::shell::Hot;

    #[test]
    fn first_run_stays_pending_until_setup_is_recorded() {
        let root = std::env::temp_dir().join(format!("litric-onboarding-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("fixture root");
        let args = Args {
            root: root.clone(),
            ..Args::default()
        };
        let regional = Regional::fallback("test");

        // No settings file: the account has never finished setup.
        assert!(!load_preferences(&args, &regional).onboarded);

        // A setup choice made before finishing keeps the setup screen up.
        let mut pending = load_preferences(&args, &regional);
        pending.onboarded = false;
        pending.language = litric_shell::i18n::Language::English;
        store_preferences(&args, pending).expect("pending choice is stored");
        assert!(!load_preferences(&args, &regional).onboarded);

        // Finishing setup is recorded and kept across restarts.
        let mut done = load_preferences(&args, &regional);
        done.onboarded = true;
        store_preferences(&args, done).expect("finished setup is stored");
        assert!(load_preferences(&args, &regional).onboarded);

        // A file written before the flag existed came from a finished setup.
        fs::write(preference_path(&args), "theme=dark\nlanguage=en\n").expect("legacy file");
        assert!(load_preferences(&args, &regional).onboarded);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_accent_choice_survives_a_restart_and_unknown_tokens_fall_back() {
        use litric_design::Accent;

        let root = std::env::temp_dir().join(format!("litric-accent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("fixture root");
        let args = Args {
            root: root.clone(),
            ..Args::default()
        };
        let regional = Regional::fallback("test");

        let mut chosen = load_preferences(&args, &regional);
        chosen.accent = Accent::Violet;
        chosen.wallpaper = litric_shell::desktop::Wallpaper::Glow;
        store_preferences(&args, chosen).expect("appearance is stored");
        let stored = load_preferences(&args, &regional);
        assert_eq!(stored.accent, Accent::Violet);
        assert_eq!(stored.wallpaper, litric_shell::desktop::Wallpaper::Glow);

        // A token written by a newer build must not make the whole record
        // unreadable: the rest of the file still loads and the accent falls
        // back to the theme's own.
        fs::write(
            preference_path(&args),
            "theme=light\nlanguage=th\naccent=chartreuse\nwallpaper=neon\nonboarded=1\n",
        )
        .expect("future file");
        let loaded = load_preferences(&args, &regional);
        assert_eq!(loaded.accent, Accent::Default);
        assert_eq!(loaded.wallpaper, litric_shell::desktop::Wallpaper::System);
        assert_eq!(loaded.theme, ThemeKind::Light);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn input_devices_are_parsed_with_their_names_intact() {
        let lines = [
            "input.device.count=2",
            "input.kbd.count=1",
            "input.device event0 kind=keyboard name=QEMU USB Keyboard",
            "input.device event2 kind=tablet name=QEMU USB Tablet",
        ];
        let devices = parse_input_devices(&lines);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].node, "event0");
        assert_eq!(devices[0].kind, "keyboard");
        // A name with spaces is the rest of the line, not one token.
        assert_eq!(devices[0].name, "QEMU USB Keyboard");
        assert_eq!(devices[1].kind, "tablet");
        // The counts are not devices.
        assert!(devices.iter().all(|d| d.node.starts_with("event")));
    }

    #[test]
    fn a_network_line_keeps_its_mac_and_its_state() {
        let lines = [
            "net.iface eth0 state=up carrier=1 mac=52:54:00:12:34:56 mtu=1500 rx=1 tx=2",
            "net.iface lo state=unknown carrier=- mac=00:00:00:00:00:00 mtu=65536 rx=0 tx=0",
        ];
        let ifaces = parse_net_ifaces(&lines, Path::new("no-such-root"));
        assert_eq!(ifaces.len(), 2);
        assert_eq!(ifaces[0].name, "eth0");
        assert_eq!(ifaces[0].state, "up");
        assert_eq!(ifaces[0].mac, "52:54:00:12:34:56");
        // No sysfs behind the fixture root, so neither claims to be wireless.
        assert!(!ifaces[0].wireless);
    }

    #[test]
    fn a_sound_card_name_with_spaces_is_not_cut_at_the_first_word() {
        let lines = ["audio.card index=0 id=PCH name=HDA Intel PCH driver=HDA-Intel"];
        assert_eq!(parse_audio_cards(&lines), vec!["card0 PCH HDA Intel PCH"]);
    }

    #[test]
    fn a_bluetooth_adapter_line_carries_its_address() {
        let lines = ["bt.adapter name=hci0 address=00:1a:7d:da:71:13 device_name=-"];
        assert_eq!(parse_bt_adapters(&lines), vec!["hci0 00:1a:7d:da:71:13"]);
    }

    #[test]
    fn browser_history_and_bookmarks_round_trip_as_encoded_private_state() {
        let root = std::env::temp_dir().join(format!("litric-browser-data-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let args = Args {
            root: root.clone(),
            socket: root.join("missing.sock"),
            ..Args::default()
        };
        let history = vec![
            "https://example.test/search?q=one two&lang=th".to_owned(),
            "https://litricos.example/".to_owned(),
        ];
        let bookmarks = vec!["https://docs.example/ไทย?a=1&b=2".to_owned()];
        store_browser_data(&args, &history, &bookmarks).expect("browser data saves");
        let (loaded_history, loaded_bookmarks, warning) = load_browser_data(&args);
        assert_eq!((loaded_history, loaded_bookmarks), (history, bookmarks));
        assert!(warning.is_none(), "a healthy file reports nothing");
        let contents = fs::read_to_string(browser_data_path(&args)).expect("saved data");
        assert!(
            !contents.contains("ไทย"),
            "non-ASCII text is encoded on disk"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(browser_data_path(&args))
                .expect("saved file metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
        let _ = fs::remove_dir_all(root);
    }

    // ---- partial-redraw idempotence oracle -------------------------------

    /// The first-run desktop `start()` builds, with fixed facts, drawn once in
    /// full. A fractional scale puts window edges and damage between pixels,
    /// which is where a clip and a restore can disagree.
    fn oracle_session() -> (Args, Session) {
        let a = Args {
            root: PathBuf::from("no-such-root-oracle"),
            socket: PathBuf::from("no-such-socket-oracle"),
            ..Args::default()
        };
        let mut s = start(&a, 960, 600, 0.75);
        s.shell.mode = DesktopMode::Ready;
        // The oracle compares the desktop, so it starts with setup already done.
        s.shell.settings.preferences.onboarded = true;
        s.shell.facts.mem_total_mb = Some(7867);
        s.shell.facts.mem_used_mb = Some(527);
        s.shell.facts.cpus = Some(8);
        s.shell.facts.uptime_secs = Some(4_000.0);
        s.shell.facts.cpu_fraction = Some(0.07);
        s.shell.facts.source = "oracle fixture".to_owned();
        s.shell.environments = ["windows", "litric", "linux"]
            .iter()
            .map(|t| EnvRow {
                token: (*t).to_owned(),
                title: (*t).to_owned(),
                state: if *t == "litric" {
                    "running".to_owned()
                } else {
                    "not-installed".to_owned()
                },
                active: *t == "litric",
                hotkey: "Win+2".to_owned(),
            })
            .collect();
        s.shell.tick(1_000);
        render(&mut s, true, &[]);
        let _ = s.shell.take_damage();
        (a, s)
    }

    /// One pass of the display loop's drawing half, as `run_display` does it.
    fn loop_frame(s: &mut Session) {
        if s.shell.is_dirty() {
            let full = s.shell.needs_full_redraw();
            let damage = s.shell.take_damage();
            render(s, full, &damage);
        }
    }

    /// Compare the scene the loop produced with a full redraw of the same
    /// state, then carry on from the full redraw.
    fn assert_matches_full_redraw(s: &mut Session, step: &str) {
        let partial = s.compositor.scene.pixels().to_vec();
        s.compositor.scene.clear(rgb(0xFF00FF));
        render(s, true, &[]);
        let _ = s.shell.take_damage();
        let w = s.compositor.scene.width() as usize;
        let mut n = 0usize;
        let mut worst = 0u32;
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        for (i, (a, b)) in partial.iter().zip(s.compositor.scene.pixels()).enumerate() {
            if a == b {
                continue;
            }
            n += 1;
            let x = i.checked_rem(w).unwrap_or(0);
            let y = i.checked_div(w).unwrap_or(0);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
            for shift in [0u32, 8, 16] {
                worst = worst.max(((a >> shift) & 0xFF).abs_diff((b >> shift) & 0xFF));
            }
        }
        assert!(
            n == 0,
            "{step}: the partial redraw differs from a full redraw in {n} pixels, \
             within ({x0},{y0})-({x1},{y1}), worst channel difference {worst}"
        );
    }

    #[test]
    fn a_process_line_is_read_field_by_field_and_a_short_one_is_skipped() {
        let lines = [
            "litric.proc=42|litric-shell|S|1|9|5200|421888|1000|/usr/bin/litric-shell%20--daemon",
            // Eight fields: a line from a daemon that sends a different shape.
            // A row with an invented command line is worse than no row.
            "litric.proc=43|short|S|1|2|10|8|0",
            // A pid that is not a number, likewise.
            "litric.proc=notanumber|weird|S|1|2|10|8|0|/bin/weird",
            // The protocol sends `-` for a reading it does not have.
            "litric.proc=44|unmeasured|R|1|1|7|-|-|/bin/unmeasured",
        ];
        let rows = parse_processes(&lines);
        assert_eq!(rows.len(), 2, "two of the four lines are whole rows");
        let first = rows.first().expect("the first whole row");
        assert_eq!(first.pid, 42);
        assert_eq!(first.name, "litric-shell");
        assert_eq!(first.state, "sleeping", "S is spelled out");
        assert_eq!(first.ppid, 1);
        assert_eq!(first.threads, 9);
        assert_eq!(first.ticks, 5_200);
        assert_eq!(first.rss_mb, Some(412), "kilobytes become mebibytes");
        assert_eq!(first.uid, Some(1000));
        assert_eq!(
            first.cmdline, "/usr/bin/litric-shell --daemon",
            "an escaped space is a space"
        );
        assert_eq!(first.cpu, None, "a cumulative counter is not a rate");
        let unmeasured = rows.get(1).expect("the second whole row");
        assert_eq!(unmeasured.rss_mb, None, "a dash is not a zero");
        assert_eq!(unmeasured.uid, None);
        assert_eq!(unmeasured.state, "running");
    }

    #[test]
    fn an_escape_that_is_not_hex_is_left_as_it_was_written() {
        assert_eq!(unescape_field("a%41b"), "aAb");
        assert_eq!(unescape_field("%7C"), "|");
        assert_eq!(unescape_field("100%25"), "100%");
        // Not an escape: the text stays readable rather than losing characters.
        assert_eq!(unescape_field("%zz"), "%zz");
        assert_eq!(unescape_field("50%"), "50%");
        assert_eq!(unescape_field("plain"), "plain");
    }

    fn frame_of(s: &Session, app: &str) -> litric_wm::Rect {
        s.shell
            .wm
            .all()
            .iter()
            .find(|w| w.app_id == app)
            .map(|w| w.frame)
            .expect("window is open")
    }

    #[test]
    fn over_budget_logging_is_rate_limited_but_never_silent() {
        // The onset is always reported: a regression that appears on frame 1
        // must reach the log, not be rounded away by the rate limit.
        for n in 1..=10 {
            assert!(
                should_log_over_budget(n),
                "frame {n} of the onset is reported"
            );
        }
        assert!(
            !should_log_over_budget(11),
            "the eleventh over-budget frame is where the limit starts"
        );
        assert!(should_log_over_budget(100), "every hundredth still reports");
        assert!(should_log_over_budget(1_000));
        assert!(!should_log_over_budget(1_001));
    }

    #[test]
    fn a_partial_redraw_is_pixel_identical_to_a_full_redraw() {
        let (args, mut s) = oracle_session();
        let mut t = 2_000u64;
        let mv = |x: f32, y: f32| InputEvent::PointerMove {
            at: Point::new(x, y),
            delta: Point::new(1.0, 1.0),
        };
        let button = |x: f32, y: f32, pressed: bool| InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed,
            at: Point::new(x, y),
            clicks: 1,
            mods: Modifiers::NONE,
        };
        let meta = Modifiers {
            meta: true,
            ..Modifiers::NONE
        };
        let mut run = |s: &mut Session, step: &str, events: Vec<InputEvent>| {
            for ev in &events {
                t += 16;
                let effects = s.shell.handle(ev, t);
                apply(&args, &mut s.shell, &effects);
            }
            s.shell.tick(t);
            loop_frame(s);
            assert_matches_full_redraw(s, step);
        };

        run(
            &mut s,
            "pointer over the empty desktop",
            vec![mv(480.0, 520.0)],
        );

        let apps = s.shell.hotspots().rect_of(Hot::MenuApps).expect("menu");
        let logo = s.shell.hotspots().rect_of(Hot::Logo).expect("logo");
        run(
            &mut s,
            "pointer across the top bar",
            vec![
                mv(apps.center().0, apps.center().1),
                mv(logo.center().0, logo.center().1),
            ],
        );

        // A calm first desktop deliberately has no windows.  Open the two
        // windows exercised below as a person would, then use their damaged
        // regions as the partial-redraw oracle.
        let effects = s.shell.open_app(App::Environments);
        apply(&args, &mut s.shell, &effects);
        let effects = s.shell.open_app(App::Activity);
        apply(&args, &mut s.shell, &effects);
        loop_frame(&mut s);
        assert_matches_full_redraw(&mut s, "explicitly opened application windows");

        let env = frame_of(&s, "com.litric.environments");
        let (bx, by) = s.shell.wm.chrome().button_rect(&env, 0).center();
        run(&mut s, "pointer onto a window button", vec![mv(bx, by)]);
        run(&mut s, "pointer off the button", vec![mv(480.0, 520.0)]);

        let (tx, ty) = s.shell.wm.chrome().title_rect(&env).center();
        run(
            &mut s,
            "press on a title bar focuses and raises",
            vec![mv(tx, ty), button(tx, ty, true)],
        );
        run(
            &mut s,
            "drag the window",
            vec![
                mv(tx - 60.0, ty + 40.0),
                button(tx - 60.0, ty + 40.0, false),
            ],
        );

        run(
            &mut s,
            "open Files with Win+E",
            vec![
                InputEvent::KeyDown {
                    key: Key::Meta(Side::Left),
                    code: 0,
                    mods: Modifiers::NONE,
                    repeat: false,
                },
                InputEvent::KeyDown {
                    key: Key::Char('e'),
                    code: 0,
                    mods: meta,
                    repeat: false,
                },
                InputEvent::KeyUp {
                    key: Key::Meta(Side::Left),
                    code: 0,
                    mods: Modifiers::NONE,
                },
            ],
        );

        if let Some(row) = s.shell.hotspots().rect_of(Hot::QuickEnvironment(1)) {
            run(
                &mut s,
                "hover an environment row",
                vec![mv(row.center().0, row.center().1)],
            );
        }

        let act = frame_of(&s, "com.litric.activity");
        let (ax, ay) = (act.x + 30.0, act.bottom() - 30.0);
        run(
            &mut s,
            "click another window to focus it",
            vec![mv(ax, ay), button(ax, ay, true), button(ax, ay, false)],
        );

        let files = frame_of(&s, "com.litric.files");
        let (cx, cy) = s.shell.wm.chrome().button_rect(&files, 0).center();
        run(
            &mut s,
            "close a window with its close button",
            vec![mv(cx, cy), button(cx, cy, true), button(cx, cy, false)],
        );

        run(
            &mut s,
            "a notification banner appears",
            vec![mv(470.0, 510.0)],
        );
        apply(&args, &mut s.shell, &[Effect::Volume(1)]);
        loop_frame(&mut s);
        assert_matches_full_redraw(&mut s, "a notification banner is drawn");
        s.shell.tick(t + 10_000);
        loop_frame(&mut s);
        assert_matches_full_redraw(&mut s, "the banner expires");
    }

    fn args(v: &[&str]) -> Result<Args, String> {
        parse_args(v.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn defaults_are_the_production_paths() {
        let a = args(&[]).expect("parses");
        assert_eq!(a.socket, PathBuf::from(SYSD_SOCKET));
        assert_eq!(a.status, PathBuf::from(STATUS_PATH));
        assert_eq!(a.theme, ThemeKind::Dark);
        assert!(a.png.is_none() && !a.once);
    }

    #[test]
    fn options_parse() {
        let a = args(&[
            "--png",
            "/tmp/x.png",
            "--size",
            "800x600",
            "--theme",
            "light",
            "--scale",
            "1.5",
            "--seconds",
            "30",
        ])
        .expect("parses");
        assert_eq!(a.png, Some(PathBuf::from("/tmp/x.png")));
        assert_eq!(a.size, Some((800, 600)));
        assert_eq!(a.theme, ThemeKind::Light);
        assert_eq!(a.scale, Some(1.5));
        assert_eq!(a.seconds, Some(30));
    }

    #[test]
    fn bad_values_are_refused_rather_than_defaulted() {
        assert!(args(&["--size", "800"]).is_err());
        assert!(args(&["--size", "0x600"]).is_err());
        assert!(args(&["--theme", "neon"]).is_err());
        assert!(args(&["--scale", "0.01"]).is_err(), "an absurd scale");
        assert!(args(&["--scale", "huge"]).is_err());
        assert!(args(&["--seconds", "soon"]).is_err());
        assert!(args(&["--nope"]).is_err());
    }

    #[test]
    fn a_facts_field_is_read_by_exact_key() {
        let lines = vec!["litric.mem.total_kb=946100", "litric.cpus=2"];
        assert_eq!(field(&lines, "litric.cpus"), Some("2"));
        assert_eq!(field(&lines, "litric.mem"), None, "a prefix must not match");
    }

    #[test]
    fn the_root_filesystem_is_read_from_the_mount_table() {
        let root = std::env::temp_dir().join("litric-shell-mounts2");
        let proc = root.join("proc/self");
        fs::create_dir_all(&proc).expect("fixture");
        fs::write(
            proc.join("mounts"),
            "devtmpfs /dev devtmpfs rw 0 0\n/dev/mapper/root / erofs ro 0 0\n",
        )
        .expect("write");
        assert_eq!(read_root_fs(&root).as_deref(), Some("erofs"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn an_unreadable_mount_table_is_none_rather_than_a_guess() {
        assert_eq!(read_root_fs(Path::new("no-such-root-33aa")), None);
    }

    #[test]
    fn network_state_is_read_from_sysfs_and_absent_when_there_is_none() {
        let root = std::env::temp_dir().join("litric-shell-net");
        let _ = fs::remove_dir_all(&root);
        let eth = root.join("sys/class/net/enp0s3");
        fs::create_dir_all(&eth).expect("dirs");
        fs::write(eth.join("operstate"), "up\n").expect("operstate");
        assert_eq!(read_network(&root).as_deref(), Some("ethernet"));

        fs::write(eth.join("operstate"), "down\n").expect("operstate");
        assert_eq!(read_network(&root).as_deref(), Some("offline"));

        assert_eq!(read_network(Path::new("no-such-root-44bb")), None);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_machine_with_no_battery_reports_none_not_zero_percent() {
        // Showing 0% on a desktop with no battery is the classic version of
        // this bug, and it looks like the machine is about to die.
        assert_eq!(read_battery(Path::new("no-such-root-55cc")), (None, None));
    }

    #[test]
    fn a_battery_is_read_when_there_is_one() {
        let root = std::env::temp_dir().join("litric-shell-batt");
        let _ = fs::remove_dir_all(&root);
        let bat = root.join("sys/class/power_supply/BAT0");
        fs::create_dir_all(&bat).expect("dirs");
        fs::write(bat.join("type"), "Battery\n").expect("type");
        fs::write(bat.join("capacity"), "73\n").expect("capacity");
        fs::write(bat.join("status"), "Discharging\n").expect("status");
        assert_eq!(read_battery(&root), (Some(73), Some(false)));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn headless_status_names_the_presenter_and_the_reason() {
        let lines = headless_lines("no card node under /dev/dri");
        assert_eq!(
            lines,
            vec![
                "target=none",
                "present=none(no card node under /dev/dri)",
                "mode=headless(no card node under /dev/dri)",
                "drawn=0",
            ]
        );
    }

    #[test]
    fn a_png_is_written_and_is_a_png() {
        let dir = std::env::temp_dir().join("litric-shell-png2");
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("frame.png");
        let mut c = Canvas::new(8, 4, rgb(0x112233));
        c.put(0, 0, rgb(0xFF0000));
        write_png(&path, &c).expect("writes");
        let bytes = fs::read(&path).expect("read back");
        assert_eq!(
            &bytes[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        assert!(bytes.windows(4).any(|w| w == b"IDAT"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rendering_on_a_machine_with_nothing_still_produces_a_desktop() {
        // The whole fallback chain in one test: no daemon, no /proc, no zone,
        // no locale. It must still draw, and it must say where the numbers
        // came from rather than inventing any.
        let dir = std::env::temp_dir().join("litric-shell-fallback");
        fs::create_dir_all(&dir).expect("dir");
        let png = dir.join("out.png");
        let a = Args {
            root: PathBuf::from("no-such-root-66dd"),
            socket: PathBuf::from("no-such-socket"),
            status: dir.join("status"),
            png: Some(png.clone()),
            size: Some((640, 400)),
            ..Args::default()
        };
        let lines = run_png(&a).expect("renders");
        assert!(lines.iter().any(|l| l == "drawn=1"), "{lines:?}");
        assert!(
            lines
                .iter()
                .any(|l| l.contains("facts.source=direct /proc read")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("regional.tz.source=fallback-utc")),
            "{lines:?}"
        );
        assert!(png.is_file());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_new_desktop_starts_without_open_application_windows() {
        let dir = std::env::temp_dir().join("litric-shell-first-desktop");
        fs::create_dir_all(&dir).expect("temporary root");
        let args = Args {
            root: dir.join("empty-root"),
            socket: dir.join("missing-socket"),
            status: dir.join("status"),
            ..Args::default()
        };
        let session = start(&args, 1280, 800, 1.0);
        assert!(
            session.shell.visible_windows().is_empty(),
            "a new user should see the desktop and choose what to open"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn files_live_in_their_own_state_directory_not_at_the_boot_root() {
        let production = Args::default();
        assert_eq!(
            files_storage_root(&production),
            PathBuf::from("/var/lib/litric-files/root"),
            "the unit's StateDirectory, not /"
        );

        let fixture = Args {
            root: PathBuf::from("/tmp/litric-fixture"),
            ..Args::default()
        };
        assert_eq!(
            files_storage_root(&fixture),
            PathBuf::from("/tmp/litric-fixture/var/lib/litric-files/root"),
            "a fixture root keeps its files inside the fixture"
        );

        let explicit = Args {
            files_root: Some(PathBuf::from("/srv/litric-files")),
            ..Args::default()
        };
        assert_eq!(
            files_storage_root(&explicit),
            PathBuf::from("/srv/litric-files"),
            "an explicit --files-root wins"
        );
        // The option exists so a test or an installer can name the store without
        // also moving /proc and /sys.
        let parsed = args(&["--files-root", "/srv/files"]).expect("parses");
        assert_eq!(parsed.files_root, Some(PathBuf::from("/srv/files")));
    }

    #[cfg(unix)]
    #[test]
    fn start_prepares_the_file_share_before_the_files_app_opens() {
        let dir = std::env::temp_dir().join(format!("litric-shell-files-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("fixture root");
        let args = Args {
            root: dir.clone(),
            socket: dir.join("missing.sock"),
            status: dir.join("status"),
            ..Args::default()
        };
        let session = start(&args, 1280, 800, 1.0);
        let storage = dir.join("var/lib/litric-files/root");
        assert_eq!(session.shell.files_root, storage);
        for name in litric_shell::fsbridge::DEFAULT_STORAGE_DIRS {
            assert!(
                storage.join(name).is_dir(),
                "{name} is ready before the Files window opens"
            );
        }
        assert!(
            session.shell.files.error.is_none(),
            "{:?}",
            session.shell.files.error
        );
        // The store is not the root the /proc and /sys fixtures are read from.
        assert_ne!(session.shell.files_root, dir);
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn start_reports_file_storage_that_cannot_be_opened_instead_of_an_empty_folder() {
        let dir =
            std::env::temp_dir().join(format!("litric-shell-files-broken-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("var/lib")).expect("fixture root");
        // The state path exists, but as a file: systemd cannot create a state
        // directory there, and neither can the shell. The point of the test is
        // that startup says so rather than opening a window on an empty folder.
        fs::write(dir.join("var/lib/litric-files"), "not a folder").expect("fixture file");
        let args = Args {
            root: dir.clone(),
            socket: dir.join("missing.sock"),
            status: dir.join("status"),
            ..Args::default()
        };
        let session = start(&args, 1280, 800, 1.0);
        let error = session
            .shell
            .files
            .error
            .as_deref()
            .expect("the Files window must say why it has no storage");
        assert!(error.contains("not ready"), "{error}");
        assert!(error.contains("not a folder"), "{error}");
        assert!(
            session
                .shell
                .notifications
                .items()
                .iter()
                .any(|n| n.title.contains("Files storage unavailable")),
            "a startup storage failure is also a notification"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn browser_data_lives_in_the_directory_the_unit_opens_for_writing() {
        let args = Args {
            root: PathBuf::from("/tmp/litric-fixture"),
            ..Args::default()
        };
        assert_eq!(
            browser_data_path(&args),
            PathBuf::from("/tmp/litric-fixture/var/lib/litric-browser/browser-data.conf")
        );
    }

    #[test]
    fn a_corrupt_browser_file_keeps_its_readable_entries_and_is_reported() {
        let root =
            std::env::temp_dir().join(format!("litric-browser-corrupt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let args = Args {
            root: root.clone(),
            socket: root.join("missing.sock"),
            ..Args::default()
        };
        let path = browser_data_path(&args);
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        let original = "not-a-key-value\nhistory=https%3A%2F%2Fkept.example%2F\nbookmark=\n";
        fs::write(&path, original).expect("fixture");
        let (history, bookmarks, warning) = load_browser_data(&args);
        assert_eq!(history, vec!["https://kept.example/".to_owned()]);
        assert!(bookmarks.is_empty());
        assert!(
            warning.as_deref().is_some_and(|w| w.contains("1 line(s)")),
            "{warning:?}"
        );
        // Loading never edits the file: the malformed bytes are still there for a
        // person (or a later recovery) to look at.
        assert_eq!(fs::read_to_string(&path).expect("still readable"), original);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn saved_addresses_load_as_one_canonical_spelling() {
        let root =
            std::env::temp_dir().join(format!("litric-browser-canonical-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let args = Args {
            root: root.clone(),
            socket: root.join("missing.sock"),
            ..Args::default()
        };
        let path = browser_data_path(&args);
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        // Two spellings of one page, plus a bookmark typed without a root
        // slash: exactly what a build before BUG-0036 wrote down.
        let stored = "history=https%3A%2F%2FEXAMPLE.com%2F\n\
                      history=https%3A%2F%2Fexample.com%3A443%2F\n\
                      bookmark=https%3A%2F%2Fexample.com\n";
        fs::write(&path, stored).expect("fixture");
        let (history, bookmarks, warning) = load_browser_data(&args);
        assert_eq!(
            history,
            vec!["https://example.com/".to_owned()],
            "one row for one page"
        );
        assert_eq!(
            bookmarks,
            vec!["https://example.com/".to_owned()],
            "the saved bookmark is canonical too"
        );
        assert!(warning.is_none(), "{warning:?}");
        fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_browser_file_is_reported_rather_than_called_a_first_run() {
        let root =
            std::env::temp_dir().join(format!("litric-browser-unreadable-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let args = Args {
            root: root.clone(),
            socket: root.join("missing.sock"),
            ..Args::default()
        };
        let path = browser_data_path(&args);
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        // Invalid UTF-8 is an error on every host, including one where the test
        // happens to run as root and can read through a mode change.
        fs::write(&path, [0xFF, 0xFE, b'\n']).expect("fixture");
        let (history, bookmarks, warning) = load_browser_data(&args);
        assert!(history.is_empty() && bookmarks.is_empty());
        assert!(
            warning
                .as_deref()
                .is_some_and(|w| w.contains("could not be read")),
            "{warning:?}"
        );
        assert!(path.is_file(), "the unreadable file is not deleted");
        fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_temporary_file_left_by_a_crash_does_not_block_the_next_save() {
        let root = std::env::temp_dir().join(format!("litric-browser-temp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let args = Args {
            root: root.clone(),
            socket: root.join("missing.sock"),
            ..Args::default()
        };
        let path = browser_data_path(&args);
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        // The old temporary name was pid-only; a recycled pid would collide with
        // this leftover and every save would fail.
        let stale = path
            .parent()
            .expect("parent")
            .join(format!("browser-data.conf.tmp.{}", std::process::id()));
        fs::write(&stale, "crashed").expect("stale temp");
        store_browser_data(&args, &["https://one.example/".to_owned()], &[])
            .expect("a stale temp is not a reason to lose the save");
        assert!(stale.is_file(), "the other process's file is left alone");
        assert_eq!(
            load_browser_data(&args).0,
            vec!["https://one.example/".to_owned()]
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_lockout_reply_carries_the_wait_the_screen_uses() {
        let reply = "ERR locked-out too many failed attempts; retry_after=60";
        assert_eq!(lockout_seconds(reply), Some(60));
        assert_eq!(
            lockout_message(reply),
            "ERR locked-out too many failed attempts"
        );
        assert_eq!(
            lockout_seconds("ERR locked-out too many failed attempts"),
            None
        );
        assert_eq!(
            lockout_seconds("ERR action-failed exit retry_after=oops"),
            None
        );
    }

    #[test]
    fn only_an_explicit_verified_reply_unlocks() {
        assert!(reply_confirms_verification(
            "OK 5\naction=auth.verify\ncommand=/usr/bin/unix_chkpwd alice nonull\n\
             user=alice\ninput=stdin\napplied=true\n"
        ));
        for reply in [
            "",
            "OK 0\n",
            "OK 2\naction=auth.verify\napplied=false\n",
            "OK 1\napplied=true\n",
            "OK 2\naction=auth.remote\napplied=true\n",
            "ERR action-failed unix_chkpwd exited 7\n",
            "garbage\n",
        ] {
            assert!(
                !reply_confirms_verification(reply),
                "a reply that is not an explicit verification must not unlock: {reply:?}"
            );
        }
    }

    #[test]
    fn an_account_name_that_would_change_the_request_line_is_refused() {
        assert!(usable_lock_account("litric"));
        assert!(!usable_lock_account(""));
        assert!(!usable_lock_account("litric CONFIRM"));
        assert!(!usable_lock_account("litric\nAUTH VERIFY root"));
        assert!(!usable_lock_account("litric\u{0}"));
    }
}
