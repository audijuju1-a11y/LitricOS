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
use litric_shell::fb::{self, PixelFormat};
use litric_shell::shell::{DesktopMode, Effect, EnvRow, Priority, ProcRow, Shell, SystemFacts};
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

struct Args {
    root: PathBuf,
    socket: PathBuf,
    status: PathBuf,
    png: Option<PathBuf>,
    format: Option<PixelFormat>,
    theme: ThemeKind,
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
            socket: PathBuf::from(SYSD_SOCKET),
            status: PathBuf::from(STATUS_PATH),
            png: None,
            format: None,
            theme: ThemeKind::Dark,
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
    --socket PATH     litric-sysd socket (default /run/litric/sysd.sock).
    --status PATH     Where to write the status report.
    --root DIR        Where /proc, /sys and /dev are rooted. For testing.
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
            }
            "--format" => {
                let v = need("--format")?;
                a.format =
                    Some(PixelFormat::parse(&v).ok_or_else(|| format!("no such format: {v}"))?);
            }
            "--socket" => a.socket = PathBuf::from(need("--socket")?),
            "--status" => a.status = PathBuf::from(need("--status")?),
            "--root" => a.root = PathBuf::from(need("--root")?),
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

/// Ask the daemon one question. Unix only, because the socket is.
#[cfg(unix)]
fn ask(socket: &Path, request: &str) -> Option<String> {
    litric_sysd::server::query(socket, request).ok()
}

#[cfg(not(unix))]
fn ask(_socket: &Path, _request: &str) -> Option<String> {
    None
}

fn field<'a>(lines: &'a [&'a str], key: &str) -> Option<&'a str> {
    lines
        .iter()
        .find_map(|l| l.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
}

/// Read the system's state, preferring the daemon and falling back to `/proc`.
fn gather_facts(args: &Args) -> SystemFacts {
    let mut f = SystemFacts {
        source: String::new(),
        ..SystemFacts::default()
    };

    if let Some(reply) = ask(&args.socket, "FACTS") {
        let lines: Vec<&str> = reply.lines().collect();
        let num = |k: &str| -> Option<u64> { field(&lines, k).and_then(|v| v.parse().ok()) };
        f.mem_total_mb = num("litric.mem.total_kb").and_then(|v| v.checked_div(1024));
        f.mem_used_mb = num("litric.mem.used_kb").and_then(|v| v.checked_div(1024));
        f.cpus = num("litric.cpus").and_then(|v| u32::try_from(v).ok());
        f.uptime_secs = field(&lines, "litric.uptime_secs").and_then(|v| v.parse().ok());
        f.cpu_fraction = field(&lines, "litric.cpu.busy_since_boot_pct")
            .and_then(|v| v.parse::<f32>().ok())
            .map(|p| p / 100.0);
        f.verity = field(&lines, "litric.root.verity").map(|v| v == "on");
        // All of the lines read below were already on the wire and unused: the
        // daemon has reported available memory, cache, swap, load, its own
        // footprint, the MGLRU state, the CPU model, tick counters per core and
        // the mount table since it was written. A monitor that shows two numbers
        // while the machine reports a dozen is a monitor nobody can diagnose
        // with.
        f.mem_available_mb = num("litric.mem.available_kb").and_then(|v| v.checked_div(1024));
        f.mem_cached_mb = num("litric.mem.cached_kb").and_then(|v| v.checked_div(1024));
        f.swap_used_mb = num("litric.mem.swap_used_kb").and_then(|v| v.checked_div(1024));
        f.load1 = field(&lines, "litric.load1").and_then(|v| v.parse().ok());
        f.tasks = num("litric.tasks").and_then(|v| u32::try_from(v).ok());
        f.sysd_rss_mb = num("litric.sysd.rss_kb").and_then(|v| v.checked_div(1024));
        f.mglru = field(&lines, "litric.mglru").map(|v| v == "on");
        f.cpu_model = field(&lines, "litric.cpu.model").map(str::to_owned);
        f.cpu_ticks = some_ticks(num("litric.cpu.busy_ticks"), num("litric.cpu.total_ticks"));
        f.core_ticks = parse_core_ticks(&lines);
        f.volumes = parse_volumes(&lines);
        f.processes = parse_processes(&lines);
        f.processes_truncated = field(&lines, "litric.proc.truncated") == Some("1");
        // Authentication, and who is logged in. Three facts from the platform
        // half of the reply that the lock screen cannot do without: what can
        // check a password, whether the daemon refuses the lock, and the account
        // a password would be checked against. `auth.lock=refused` is only
        // present while locking is refused, so its absence is the claim that the
        // lock can be unlocked again -- which is why the boolean is derived from
        // the whole reply rather than from one line.
        f.auth_available = field(&lines, "auth.available").map(str::to_owned);
        // The line is written only while locking is refused, so its presence is
        // the refusal — the shell reads whether the daemon said anything, not
        // the word it said.
        f.auth_lock = Some(field(&lines, "auth.lock").is_none());
        f.session_user = lines
            .iter()
            .find_map(|l| l.strip_prefix("session.id "))
            .and_then(|rest| {
                rest.split_whitespace()
                    .find_map(|kv| kv.strip_prefix("user="))
                    .filter(|u| *u != "-")
                    .map(str::to_owned)
            });
        f.source = "litric-sysd".to_owned();
    }

    if f.source.is_empty() {
        let raw = litric_sysd::facts::read_facts(&args.root);
        if let Some(m) = raw.mem {
            f.mem_total_mb = m.total_kb.checked_div(1024);
            f.mem_used_mb = m.used_kb().checked_div(1024);
        }
        f.cpus = raw.cpus;
        f.uptime_secs = raw.uptime.map(|u| u.up_secs);
        f.cpu_fraction = raw
            .cpu
            .and_then(|c| c.busy_fraction_since_boot())
            .map(|v| v as f32);
        f.verity = raw.root_verity();
        if let Some(m) = raw.mem {
            f.mem_available_mb = m.available_kb.checked_div(1024);
            f.mem_cached_mb = m.cached_kb.checked_div(1024);
            f.swap_used_mb = m.swap_used_kb().checked_div(1024);
        }
        f.load1 = raw.load.map(|l| l.one as f32);
        f.tasks = raw.load.map(|l| l.total);
        f.sysd_rss_mb = raw.self_rss_kb.and_then(|v| v.checked_div(1024));
        f.mglru = raw.mglru;
        f.cpu_model = raw.cpu_model;
        f.cpu_ticks = raw.cpu.map(|c| (c.busy(), c.total()));
        f.core_ticks = raw.per_cpu.iter().map(|c| (c.busy(), c.total())).collect();
        f.volumes = raw
            .mounts
            .iter()
            .map(|m| (m.mount.clone(), m.fs.clone(), m.read_only))
            .collect();
        // The process table is read directly when there is no daemon: the
        // monitor's job is to show what the machine is doing, and a machine
        // whose daemon is not up still has processes. CPU shares stay `None`
        // here until a second sample arrives through `observe`.
        f.processes = raw
            .processes
            .iter()
            .map(|p| ProcRow {
                pid: p.pid,
                name: p.display_name().to_owned(),
                cmdline: p.cmdline.clone(),
                state: p.state_word().to_owned(),
                ppid: p.ppid,
                threads: p.threads,
                ticks: p.ticks(),
                rss_mb: p.rss_kb.and_then(|kb| kb.checked_div(1024)),
                uid: p.uid,
                cpu: None,
            })
            .collect();
        f.processes_truncated = raw.processes.len() >= litric_sysd::facts::MAX_PROCESSES;
        f.source = "direct /proc read".to_owned();
    }

    if let Some(reply) = ask(&args.socket, "TRASH LIST") {
        let lines: Vec<&str> = reply.lines().collect();
        let n = field(&lines, "count").and_then(|v| v.parse().ok());
        let b = field(&lines, "used_bytes").and_then(|v| v.parse().ok());
        if let (Some(n), Some(b)) = (n, b) {
            f.trash = Some((n, b));
        }
    }

    let (gpu_name, gpu_busy) = read_gpu(&args.root);
    f.gpu_name = gpu_name;
    f.gpu_fraction = gpu_busy;
    f.root_fs = read_root_fs(&args.root);
    f.network = read_network(&args.root);
    let (pct, charging) = read_battery(&args.root);
    f.battery_percent = pct;
    f.battery_charging = charging;
    f
}

/// Both counters or neither: a busy count without its total cannot be
/// differenced into a rate, and pairing it with a zero would report a busy CPU.
fn some_ticks(busy: Option<u64>, total: Option<u64>) -> Option<(u64, u64)> {
    Some((busy?, total?))
}

/// Per-core counters from `litric.cpu.coreN.*` lines, in kernel order.
///
/// The daemon emits each core's busy and total on consecutive lines for the same
/// core; the pairing is done on the core number rather than on line order alone,
/// so a reordered stream produces no pairing instead of a wrong one.
fn parse_core_ticks(lines: &[&str]) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = Vec::new();
    let mut pending: Option<(usize, u64)> = None;
    for line in lines {
        let Some(rest) = line.strip_prefix("litric.cpu.core") else {
            continue;
        };
        let Some((idx, kv)) = rest.split_once('.') else {
            continue;
        };
        let Ok(i) = idx.parse::<usize>() else {
            continue;
        };
        let Some((key, value)) = kv.split_once('=') else {
            continue;
        };
        let Ok(v) = value.parse::<u64>() else {
            continue;
        };
        match (key, pending) {
            ("busy_ticks", _) => pending = Some((i, v)),
            ("total_ticks", Some((j, busy))) if j == i => {
                out.push((busy, v));
                pending = None;
            }
            _ => {}
        }
    }
    out
}

/// The process table from `litric.proc=pid|name|state|ppid|threads|ticks|rss|uid|cmdline`.
///
/// A line with a field missing is skipped rather than filled in: a row with an
/// invented pid or a zeroed tick count is a row a person will believe, and the
/// protocol sends `-` for "not measured" so that a real zero and a missing
/// reading stay distinct.
fn parse_processes(lines: &[&str]) -> Vec<ProcRow> {
    let mut out = Vec::new();
    for line in lines {
        let Some(value) = line.strip_prefix("litric.proc=") else {
            continue;
        };
        let fields: Vec<&str> = value.split('|').collect();
        if fields.len() < 9 {
            continue;
        }
        // Read by position through an iterator rather than by index: the count
        // was just checked, and going through the iterator keeps the check and
        // the read from drifting apart. An index here would mean a panic inside
        // a frame if that ever happened, which is the one place it must not.
        let mut fields = fields.into_iter();
        let (Some(pid), Some(name), Some(state), Some(ppid), Some(threads), Some(ticks)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            continue;
        };
        let (Ok(pid), Ok(ppid), Ok(threads), Ok(ticks)) = (
            pid.parse::<u32>(),
            ppid.parse::<u32>(),
            threads.parse::<u32>(),
            ticks.parse::<u64>(),
        ) else {
            continue;
        };
        out.push(ProcRow {
            pid,
            name: unescape_field(name),
            state: match state.chars().next() {
                Some('R') => "running",
                Some('S') => "sleeping",
                Some('D') => "uninterruptible",
                Some('Z') => "zombie",
                Some('T' | 't') => "stopped",
                Some('I') => "idle",
                Some('X' | 'x') => "dead",
                _ => "unknown",
            }
            .to_owned(),
            ppid,
            threads,
            ticks,
            rss_mb: fields
                .next()
                .and_then(|kb| kb.parse::<u64>().ok())
                .and_then(|kb| kb.checked_div(1024)),
            uid: fields.next().and_then(|uid| uid.parse::<u32>().ok()),
            cmdline: unescape_field(fields.next().unwrap_or_default()),
            cpu: None,
        });
    }
    out
}

/// Undo [`litric_sysd::facts::escape_field`]: `%25` is a percent, `%7C` a pipe.
fn unescape_field(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut rest = bytes;
    while let Some((&first, tail)) = rest.split_first() {
        if first == b'%' {
            let hex = tail
                .first()
                .and_then(|b| char::from(*b).to_digit(16))
                .zip(tail.get(1).and_then(|b| char::from(*b).to_digit(16)));
            if let Some((h, l)) = hex {
                let byte = u8::try_from(h.saturating_mul(16).saturating_add(l)).unwrap_or(b'?');
                out.push(byte);
                rest = tail.get(2..).unwrap_or_default();
                continue;
            }
        }
        out.push(first);
        rest = tail;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Mounted volumes from `litric.mount.N=mount|fs|ro|source` lines.
fn parse_volumes(lines: &[&str]) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("litric.mount.") else {
            continue;
        };
        let Some((_, value)) = rest.split_once('=') else {
            continue;
        };
        let mut parts = value.split('|');
        let (Some(mount), Some(fs), Some(mode)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        out.push((mount.to_owned(), fs.to_owned(), mode == "ro"));
    }
    out
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

/// The GPU picture, from sysfs — measured, never guessed.
///
/// A DRM card exists when a kernel driver has bound one: the `driver` symlink
/// under `card*/device` names it. `virtio_gpu` and `bochs-drm` are real GPU
/// devices driving the scanout (the desktop's framebuffer is GPU-backed even
/// though the desktop software-rasterises into it); `simpledrm` is the
/// firmware framebuffer — the honest CPU fallback. Utilisation comes only
/// from `gpu_busy_percent`, which few drivers publish (amdgpu does,
/// virtio-gpu does not); a present but unmeasurable GPU is reported present
/// with no percentage, never 0%.
fn read_gpu(root: &Path) -> (Option<String>, Option<f32>) {
    (0..8u8)
        .find_map(|n| {
            let dev = root.join(format!("sys/class/drm/card{n}/device"));
            let driver = fs::read_link(dev.join("driver")).ok()?;
            let name = driver
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())?;
            let busy = fs::read_to_string(dev.join("gpu_busy_percent"))
                .ok()
                .and_then(|t| t.trim().parse::<f32>().ok())
                .filter(|p| (0.0..=100.0).contains(p))
                .map(|p| p / 100.0);
            Some((Some(name), busy))
        })
        .unwrap_or((None, None))
}

fn read_root_fs(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("proc/self/mounts")).ok()?;
    for line in text.lines() {
        let mut f = line.split_whitespace();
        let _dev = f.next();
        if f.next() == Some("/") {
            return f.next().map(str::to_owned);
        }
    }
    None
}

/// Network state from sysfs. `None` when nothing can be read, never a guess.
fn read_network(root: &Path) -> Option<String> {
    let dir = root.join("sys/class/net");
    let entries = fs::read_dir(&dir).ok()?;
    let mut best: Option<String> = None;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name == "lo" {
            continue;
        }
        let up = fs::read_to_string(e.path().join("operstate"))
            .ok()
            .map(|s| s.trim().to_owned());
        if up.as_deref() == Some("up") {
            // A wireless interface has a `wireless` directory; everything else
            // is treated as wired, which is what the icon has to choose between.
            let wireless = e.path().join("wireless").exists();
            return Some(if wireless { "wifi" } else { "ethernet" }.to_owned());
        }
        best = Some("offline".to_owned());
    }
    best
}

/// Battery percentage and charging state, or `None` when there is no battery.
fn read_battery(root: &Path) -> (Option<u8>, Option<bool>) {
    let dir = root.join("sys/class/power_supply");
    let Ok(entries) = fs::read_dir(&dir) else {
        return (None, None);
    };
    for e in entries.flatten() {
        let kind = fs::read_to_string(e.path().join("type"))
            .ok()
            .map(|s| s.trim().to_owned());
        if kind.as_deref() != Some("Battery") {
            continue;
        }
        let pct = fs::read_to_string(e.path().join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());
        let charging = fs::read_to_string(e.path().join("status"))
            .ok()
            .map(|s| s.trim() == "Charging");
        return (pct, charging);
    }
    (None, None)
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
fn apply(args: &Args, shell: &mut Shell, effects: &[Effect]) {
    for e in effects {
        match e {
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
                let request = format!("AUTH VERIFY {user} {} CONFIRM", secret.hex());
                match ask(&args.socket, &request) {
                    Some(reply) if reply.starts_with("ERR ") => {
                        // The reply says which of "not authorised", "the helper
                        // refused" and "the helper is missing" happened, in its
                        // own words. It never contains the password, and neither
                        // does this: what is shown is the daemon's line, trimmed.
                        shell.lock_failed(reply.trim());
                    }
                    Some(_) => shell.unlock(),
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
            Effect::Volume(_) => {
                shell.notify(
                    "Audio",
                    "No audio device",
                    "Not implemented: the audio stack is a later layer.",
                    Priority::Low,
                    Some(4_000),
                );
            }
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

fn start(args: &Args, width: u32, height: u32, scale: f32) -> Session {
    let theme = Theme::of(args.theme);
    let regional = Regional::detect(&args.root, |k| std::env::var(k).ok());
    let tokens = Tokens::new(theme, scale);
    let mut shell = Shell::new(tokens, width as f32, height as f32, regional);
    // The same root the facts, the framebuffer and the input devices are read
    // through, so a shell started against a fixture tree reads its files
    // there too instead of reaching into the machine's real root.
    shell.files_root = args.root.clone();

    shell.sample(gather_facts(args));
    shell.environments = gather_environments(args);

    // A first run opens the two panels that say what this machine is. An empty
    // desktop is not a neutral starting point: it gives a user nothing to act
    // on and hides the fact that anything is running at all.
    {
        use litric_shell::shortcuts::App;
        let work = shell.wm.work_area();
        let pad = 64.0 * scale;
        let panel_w = (work.w * 0.34).clamp(320.0 * scale, 520.0 * scale);
        // Placed rather than cascaded: the cascade offset is smaller than the
        // narrower window, so two cascaded panels leave the second entirely
        // hidden behind the first.
        let activity = litric_wm::Rect::new(
            work.x + pad,
            work.y + pad,
            panel_w,
            (work.h * 0.46).max(260.0 * scale),
        );
        let environments = litric_wm::Rect::new(
            work.right() - pad - panel_w,
            work.y + pad,
            panel_w,
            (work.h * 0.34).max(220.0 * scale),
        );
        let env_id = shell.wm.open_at(
            App::Environments.app_id(),
            App::Environments.title(),
            environments,
        );
        let act_id = shell
            .wm
            .open_at(App::Activity.app_id(), App::Activity.title(), activity);
        let _ = env_id;
        shell.wm.focus_window(act_id);
    }

    let compositor = desktop::Compositor::new(width, height, &theme);

    let mut lines = vec![
        format!("version={VERSION}"),
        format!("canvas={width}x{height}"),
        format!("scale={scale:.2}"),
        format!("theme={}", args.theme.token()),
        format!("facts.source={}", shell.facts.source),
        format!("environments={}", shell.environments.len()),
    ];
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

fn run_display(args: &Args) -> Result<Vec<String>, String> {
    let console = take_console(&args.root);
    let Some((sys, dev, mut info)) = fb::discover(&args.root) else {
        // Headless is not a failure. The machine boots, the daemon runs, and
        // the shell says why there is nothing to look at.
        return Ok(vec![
            "target=none".to_owned(),
            "mode=headless(no framebuffer found)".to_owned(),
            "drawn=0".to_owned(),
        ]);
    };
    if let Some(f) = args.format {
        info.format = f;
    }
    let (w, h) = (info.width, info.height);
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

    let mut display = fb::Framebuffer::open(&dev, info)
        .map_err(|e| format!("cannot open {}: {e}", dev.display()))?;

    session.lines.push("target=framebuffer".to_owned());
    session.lines.push(format!("fbcon={console}"));
    session
        .lines
        .push(format!("fb={}", display.info().summary()));
    session.lines.push(format!("fb.sysfs={}", sys.display()));
    session.lines.extend(report_in.lines());
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

    // The first frame, immediately: a machine that boots to a black screen for
    // a second looks broken even when it is not.
    session.shell.tick(now_ms());
    render(&mut session, true, &[]);
    let _ = session.shell.take_damage();
    display
        .present(&session.compositor.scene)
        .map_err(|e| format!("cannot write {}: {e}", dev.display()))?;
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

    loop {
        if let Some(limit) = args.seconds {
            if started.elapsed() >= Duration::from_secs(limit) {
                break;
            }
        }

        let budget = Duration::from_millis(session.shell.frame_interval_ms());
        let events = input.poll(budget);
        let t = now_ms();

        for ev in &events {
            let effects = session.shell.handle(ev, t);
            if !effects.is_empty() {
                apply(args, &mut session.shell, &effects);
            }
        }
        session.shell.tick(t);

        if t.saturating_sub(last_facts) >= FACTS_INTERVAL_MS {
            last_facts = t;
            let before = session.shell.facts.clone();
            session.shell.sample(gather_facts(args));
            if session.shell.facts != before {
                session.shell.request_full_redraw();
            }
            // Re-query environments periodically in case daemon was not ready at startup.
            // Always re-query and update, even if empty, to detect when daemon
            // becomes available.
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

        if session.shell.is_dirty() {
            let full = session.shell.needs_full_redraw();
            let damage = session.shell.take_damage();
            let plan = render(&mut session, full, &damage);
            let scene = &session.compositor.scene;
            let rows = match &plan {
                Repaint::Full => display.present(scene).map(|()| display.info().height),
                Repaint::Regions(regions) => display.present_damage(scene, regions),
            }
            .map_err(|e| format!("present failed: {e}"))?;
            rows_written = rows_written.saturating_add(u64::from(rows));
        }

        if t.saturating_sub(last_status) >= STATUS_INTERVAL_MS {
            last_status = t;
            let mut lines = session.lines.clone();
            lines.extend(session.shell.diagnostic_lines());
            lines.push(format!("rows_presented={rows_written}"));
            lines.push(format!("frames={}", session.shell.frames()));
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
    use litric_shell::shell::Hot;

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
}
