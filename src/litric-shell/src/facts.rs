//! Reading one sample of the machine for the desktop.
//!
//! This is the shell's single facts reader. The periodic loop and a confirmed
//! Activity action both go through [`gather`], so the table and the meters
//! always describe the sample the machine just gave.

use std::fs;
use std::path::Path;

use crate::shell::{InputDevice, NetIface, ProcRow, SystemFacts};

/// Where `litric-sysd` listens when the runner does not say otherwise.
pub(crate) const DEFAULT_SOCKET: &str = "/run/litric/sysd.sock";

/// Undo [`litric_sysd::facts::escape_field`]: `%25` is a percent, `%7C` a pipe.
pub fn unescape_field(s: &str) -> String {
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

/// Ask the daemon one question. Unix only, because the socket is.
#[cfg(unix)]
pub fn ask(socket: &Path, request: &str) -> Option<String> {
    litric_sysd::server::query(socket, request).ok()
}

/// Parsed from a daemon reply for the desktop.
/// On a host without Unix sockets there is no daemon to ask.
#[cfg(not(unix))]
pub fn ask(_socket: &Path, _request: &str) -> Option<String> {
    None
}

/// The value after `key=` on a daemon reply line.
pub fn field<'a>(lines: &'a [&'a str], key: &str) -> Option<&'a str> {
    lines
        .iter()
        .find_map(|l| l.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
}

/// Read the system's state, preferring the daemon and falling back to `/proc`.
pub(crate) fn gather(socket: &Path, root: &Path) -> SystemFacts {
    let mut f = SystemFacts {
        source: String::new(),
        ..SystemFacts::default()
    };

    if let Some(reply) = ask(socket, "FACTS") {
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
        // Interface counters, which have been on the wire since the daemon was
        // written and had no reader: `net.iface eth0 state=up ... rx= tx=`. They
        // are cumulative, so a rate is the difference between two samples, and
        // the shell is the one holding the clock that can divide them.
        let (rx, tx) = parse_net_bytes(&lines);
        f.net_rx_bytes = rx;
        f.net_tx_bytes = tx;
        // The devices themselves. The shell draws input and network panels
        // from these, and the difference between "the machine has none" and
        // "the daemon did not say" is carried by an empty vector against a
        // missing reply, which the panels distinguish.
        f.input_devices = parse_input_devices(&lines);
        f.net_ifaces = parse_net_ifaces(&lines, root);
        f.bt_state = field(&lines, "bt.state").map(str::to_owned);
        f.bt_daemon = field(&lines, "bt.daemon").map(|v| v == "present");
        // The radio's power, which is not in sysfs: it appears in a reply only
        // after the daemon has asked BlueZ, and its absence means nobody has.
        f.bt_powered = field(&lines, "bt.powered").and_then(|v| match v {
            "1" => Some(true),
            "0" => Some(false),
            // A third value is not a power state, so nothing is kept.
            _ => None,
        });
        f.bt_adapters = parse_bt_adapters(&lines);
        f.audio_cards = parse_audio_cards(&lines);
        f.audio_server = lines.iter().find_map(|l| {
            l.strip_prefix("audio.state server=")
                .and_then(|rest| rest.split_whitespace().next())
                .map(str::to_owned)
        });
        // The level the sound server reported. Absent means nobody has measured
        // it — the panel says so rather than showing a zero it made up.
        if let Some(rest) = lines.iter().find_map(|l| l.strip_prefix("audio.level ")) {
            for kv in rest.split_whitespace() {
                if let Some(v) = kv.strip_prefix("volume=") {
                    f.audio_volume = v.parse::<u8>().ok().filter(|v| *v <= 100);
                } else if let Some(v) = kv.strip_prefix("muted=") {
                    f.audio_muted = match v {
                        "1" => Some(true),
                        "0" => Some(false),
                        // A third value is not a mute state, so nothing is kept.
                        _ => None,
                    };
                }
            }
        }
        // Authentication, and who is logged in. Three facts from the platform
        // half of the reply that the lock screen cannot do without: what can
        // check a password, whether the daemon refuses the lock, and the account
        // a password would be checked against. `auth.lock=refused` is only
        // present while locking is refused, so its presence must map to true.
        // Derive this from the whole reply rather than interpreting the literal
        // word `refused` as a successful lock.
        f.auth_available = field(&lines, "auth.available").map(str::to_owned);
        // The line is written only while locking is refused, so its presence is
        // the refusal — the shell reads whether the daemon said anything.
        f.auth_lock = Some(field(&lines, "auth.lock").is_some());
        // The account a lock would check, and whether that account has a usable
        // password. Without the second fact a machine whose `litric` account is
        // locked (a skipped first run) would still offer a lock screen that no
        // password could open.
        f.auth_account = field(&lines, "auth.account")
            .filter(|account| *account != "-" && !account.is_empty())
            .map(str::to_owned);
        f.auth_password = field(&lines, "auth.password").map(str::to_owned);
        f.auth_reason = field(&lines, "auth.reason").map(str::to_owned);
        // Display only. The lock does *not* use this: an active serial console
        // session as root is not the desktop account. Prefer an active record,
        // and fall back to the first so a machine with only an online session
        // still shows who it belongs to.
        let session_user = |want_active: bool| {
            lines
                .iter()
                .filter(|l| l.strip_prefix("session.id ").is_some())
                .find(|l| !want_active || l.split_whitespace().any(|kv| kv == "state=active"))
                .and_then(|l| l.strip_prefix("session.id "))
                .and_then(|rest| {
                    rest.split_whitespace()
                        .find_map(|kv| kv.strip_prefix("user="))
                        .filter(|u| *u != "-")
                        .map(str::to_owned)
                })
        };
        f.session_user = session_user(true).or_else(|| session_user(false));
        f.source = "litric-sysd".to_owned();
    }

    if f.source.is_empty() {
        let raw = litric_sysd::facts::read_facts(root);
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
        // The network counters come from the daemon's own reader. The direct
        // path has no reader for them, so they stay `None` and the graph says
        // so, rather than reporting a machine that transfers nothing.
        let net_lines = raw.platform.net.to_lines();
        let net_refs: Vec<&str> = net_lines.iter().map(String::as_str).collect();
        let (rx, tx) = parse_net_bytes(&net_refs);
        f.net_rx_bytes = rx;
        f.net_tx_bytes = tx;
        // The same facts, taken from the same reader the daemon uses. Without
        // this branch a machine whose daemon is down would show an empty
        // Network page and claim it has no interfaces, which is exactly the
        // lie the platform module's `unreadable` flag exists to prevent.
        f.input_devices = raw
            .platform
            .input
            .devices
            .iter()
            .map(|d| InputDevice {
                node: d.node.clone(),
                name: d.name.clone().unwrap_or_else(|| "-".to_owned()),
                kind: d.kind.as_str().to_owned(),
            })
            .collect();
        f.net_ifaces = raw
            .platform
            .net
            .ifaces
            .iter()
            .map(|i| NetIface {
                wireless: is_wireless(root, &i.name),
                name: i.name.clone(),
                state: i.operstate.clone(),
                mac: i.mac.clone().unwrap_or_else(|| "-".to_owned()),
            })
            .collect();
        f.bt_state = Some(raw.platform.bluetooth.state().to_owned());
        f.bt_daemon = Some(raw.platform.bluetooth.daemon);
        f.bt_adapters = raw
            .platform
            .bluetooth
            .adapters
            .iter()
            .map(|a| format!("{} {}", a.name, a.address.as_deref().unwrap_or("-")))
            .collect();
        f.audio_cards = raw
            .platform
            .audio
            .cards
            .iter()
            .map(|c| format!("card{} {}", c.index, c.name))
            .collect();
        f.audio_server = raw
            .platform
            .audio
            .server
            .as_ref()
            .map(|s| s.state.to_owned());
    }

    if let Some(reply) = ask(socket, "TRASH LIST") {
        let lines: Vec<&str> = reply.lines().collect();
        let n = field(&lines, "count").and_then(|v| v.parse().ok());
        let b = field(&lines, "used_bytes").and_then(|v| v.parse().ok());
        if let (Some(n), Some(b)) = (n, b) {
            f.trash = Some((n, b));
        }
    }

    let (gpu_name, gpu_busy) = read_gpu(root);
    f.gpu_name = gpu_name;
    f.gpu_fraction = gpu_busy;
    f.root_fs = read_root_fs(root);
    f.network = read_network(root);
    let (pct, charging) = read_battery(root);
    f.battery_percent = pct;
    f.battery_charging = charging;
    f
}

/// Both counters or neither: a busy count without its total cannot be
/// differenced into a rate, and pairing it with a zero would report a busy CPU.
fn some_ticks(busy: Option<u64>, total: Option<u64>) -> Option<(u64, u64)> {
    Some((busy?, total?))
}

/// Total bytes received and sent, from the daemon's `net.iface` lines.
///
/// Summed across the machine's interfaces, loopback excepted: `lo` carries the
/// shell's own conversation with the daemon, which is not what a person
/// watching a network rate is watching for. A line whose counters did not parse
/// is skipped rather than counted as zero, and the total is `None` when no line
/// carried a counter at all -- a machine with no measurable interface is not a
/// machine with no traffic.
fn parse_net_bytes(lines: &[&str]) -> (Option<u64>, Option<u64>) {
    let mut rx: Option<u64> = None;
    let mut tx: Option<u64> = None;
    for line in lines {
        let Some(rest) = line.strip_prefix("net.iface ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        if name == "lo" {
            continue;
        }
        for kv in parts {
            if let Some(v) = kv.strip_prefix("rx=") {
                if let Ok(n) = v.parse::<u64>() {
                    rx = Some(rx.unwrap_or(0).saturating_add(n));
                }
            } else if let Some(v) = kv.strip_prefix("tx=") {
                if let Ok(n) = v.parse::<u64>() {
                    tx = Some(tx.unwrap_or(0).saturating_add(n));
                }
            }
        }
    }
    (rx, tx)
}

/// Whether the driver for `name` published a wireless directory.
fn is_wireless(root: &Path, name: &str) -> bool {
    root.join(format!("sys/class/net/{name}/wireless")).is_dir()
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

/// Input devices, from the platform's `input.device` lines.
///
/// One line is `input.device event3 kind=keyboard name=QEMU USB Keyboard`. The
/// name is last and may contain spaces, so it is taken as the remainder rather
/// than as another token. A line whose name is missing still yields a device:
/// an event node the platform could not name is a node that exists.
pub fn parse_input_devices(lines: &[&str]) -> Vec<InputDevice> {
    let mut out = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("input.device ") else {
            continue;
        };
        let (head, name) = match rest.split_once(" name=") {
            Some((h, n)) => (h, n),
            None => (rest, "-"),
        };
        let mut parts = head.split_whitespace();
        let Some(node) = parts.next() else {
            continue;
        };
        let kind = parts
            .find_map(|p| p.strip_prefix("kind="))
            .unwrap_or("other");
        out.push(InputDevice {
            node: node.to_owned(),
            name: name.trim().to_owned(),
            kind: kind.to_owned(),
        });
    }
    out
}

/// Network interfaces, from the platform's `net.iface` lines.
///
/// The platform reports state, carrier, MAC, MTU and byte counters in one line.
/// The shell only needs the identity and the state here -- the counters are
/// summed elsewhere -- but it asks sysfs whether the driver is wireless,
/// because the platform reader does not publish that and guessing from the
/// interface name (`wlan*`) would call a USB dongle named `eth1` a cable.
pub fn parse_net_ifaces(lines: &[&str], root: &Path) -> Vec<NetIface> {
    let mut out = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("net.iface ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let mut state = "unknown".to_owned();
        let mut mac = "-".to_owned();
        for kv in parts {
            if let Some(v) = kv.strip_prefix("state=") {
                state = v.to_owned();
            } else if let Some(v) = kv.strip_prefix("mac=") {
                mac = v.to_owned();
            }
        }
        out.push(NetIface {
            wireless: is_wireless(root, name),
            name: name.to_owned(),
            state,
            mac,
        });
    }
    out
}

/// Bluetooth adapters, as `hci0 00:1a:7d:da:71:13`.
pub fn parse_bt_adapters(lines: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("bt.adapter ") else {
            continue;
        };
        let mut name = "-";
        let mut address = "-";
        for kv in rest.split_whitespace() {
            if let Some(v) = kv.strip_prefix("name=") {
                name = v;
            } else if let Some(v) = kv.strip_prefix("address=") {
                address = v;
            }
        }
        out.push(format!("{name} {address}"));
    }
    out
}

/// Sound cards, as `card0 HDA Intel PCH`.
pub fn parse_audio_cards(lines: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("audio.card ") else {
            continue;
        };
        // The card name contains spaces (`HDA Intel PCH`), so it runs to the
        // next key rather than to the next space: the daemon writes
        // `name=<card name> driver=<driver>`, and the driver is not part of the
        // name.
        let (head, name) = match rest.split_once(" name=") {
            Some((h, n)) => (h, n.split_once(" driver=").map_or(n, |(name, _)| name)),
            None => (rest, "-"),
        };
        let mut index = "-";
        let mut id = "-";
        for kv in head.split_whitespace() {
            if let Some(v) = kv.strip_prefix("index=") {
                index = v;
            } else if let Some(v) = kv.strip_prefix("id=") {
                id = v;
            }
        }
        out.push(format!("card{index} {id} {name}"));
    }
    out
}

/// The process table from `litric.proc=pid|name|state|ppid|threads|ticks|rss|uid|cmdline`.
///
/// A line with a field missing is skipped rather than filled in: a row with an
/// invented pid or a zeroed tick count is a row a person will believe, and the
/// protocol sends `-` for "not measured" so that a real zero and a missing
/// reading stay distinct.
pub fn parse_processes(lines: &[&str]) -> Vec<ProcRow> {
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

/// Parsed from a daemon reply for the desktop.
pub fn read_root_fs(root: &Path) -> Option<String> {
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
pub fn read_network(root: &Path) -> Option<String> {
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
pub fn read_battery(root: &Path) -> (Option<u8>, Option<bool>) {
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
