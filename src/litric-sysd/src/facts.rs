//! Reading what the machine is actually doing.
//!
//! Every claim LitricOS makes about being light has to come from a measurement
//! taken on the running system, not from a design document. This module is
//! where those measurements are read.
//!
//! The parsers take `&str` and the readers take a root directory, so the whole
//! module is testable on a host that has no `/proc` at all. That is deliberate:
//! a parser that can only be exercised on the target is a parser that gets
//! exercised once, by hand, and then trusted forever.

use std::fs;
use std::path::Path;

/// Memory, in kibibytes, as `/proc/meminfo` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemInfo {
    /// Total usable RAM.
    pub total_kb: u64,
    /// What a new allocation could get without swapping. **This**, not
    /// `MemFree`, is the number that means anything: Linux deliberately spends
    /// free memory on cache, so a healthy system always looks nearly full.
    pub available_kb: u64,
    /// Genuinely untouched memory.
    pub free_kb: u64,
    /// Page cache.
    pub cached_kb: u64,
    /// Shared memory, which includes tmpfs and therefore `/run`.
    pub shmem_kb: u64,
    /// Total swap.
    pub swap_total_kb: u64,
    /// Free swap.
    pub swap_free_kb: u64,
}

impl MemInfo {
    /// Parse the contents of `/proc/meminfo`.
    ///
    /// Returns `None` if `MemTotal` is absent, which means this was not a
    /// meminfo file and every derived number would be a fiction.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut m = Self::default();
        let mut saw_total = false;
        for line in text.lines() {
            let Some((key, rest)) = line.split_once(':') else {
                continue;
            };
            let Some(value) = rest.split_whitespace().next() else {
                continue;
            };
            let Ok(kb) = value.parse::<u64>() else {
                continue;
            };
            match key {
                "MemTotal" => {
                    m.total_kb = kb;
                    saw_total = true;
                }
                "MemAvailable" => m.available_kb = kb,
                "MemFree" => m.free_kb = kb,
                "Cached" => m.cached_kb = kb,
                "Shmem" => m.shmem_kb = kb,
                "SwapTotal" => m.swap_total_kb = kb,
                "SwapFree" => m.swap_free_kb = kb,
                _ => {}
            }
        }
        if saw_total {
            Some(m)
        } else {
            None
        }
    }

    /// Memory genuinely committed, in kibibytes.
    ///
    /// `total - available`, not `total - free`. Reporting `total - free` is how
    /// a Linux system with a healthy page cache gets described as running out
    /// of memory.
    #[must_use]
    pub const fn used_kb(&self) -> u64 {
        self.total_kb.saturating_sub(self.available_kb)
    }

    /// Swap in use, in kibibytes.
    #[must_use]
    pub const fn swap_used_kb(&self) -> u64 {
        self.swap_total_kb.saturating_sub(self.swap_free_kb)
    }
}

/// `/proc/uptime`, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Uptime {
    /// Seconds since boot.
    pub up_secs: f64,
    /// Summed idle seconds across all CPUs.
    pub idle_secs: f64,
}

impl Uptime {
    /// Parse the contents of `/proc/uptime`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut it = text.split_whitespace();
        let up = it.next()?.parse::<f64>().ok()?;
        let idle = it.next().and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        Some(Self {
            up_secs: up,
            idle_secs: idle,
        })
    }
}

/// Aggregate CPU time from the first line of `/proc/stat`, in USER_HZ ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuTimes {
    /// Time in user mode.
    pub user: u64,
    /// Time in user mode, niced.
    pub nice: u64,
    /// Time in kernel mode.
    pub system: u64,
    /// Idle.
    pub idle: u64,
    /// Waiting on I/O. Counted as idle: the CPU was available.
    pub iowait: u64,
    /// Hard interrupts.
    pub irq: u64,
    /// Soft interrupts.
    pub softirq: u64,
    /// Time stolen by a hypervisor.
    pub steal: u64,
}

impl CpuTimes {
    /// Read one `cpu`/`cpuN` line into counters.
    fn from_line(line: &str) -> Option<Self> {
        if !line.starts_with("cpu") {
            return None;
        }
        let mut f = line.split_whitespace().skip(1).map(|v| v.parse::<u64>());
        let mut next = || f.next().and_then(std::result::Result::ok).unwrap_or(0);
        Some(Self {
            user: next(),
            nice: next(),
            system: next(),
            idle: next(),
            iowait: next(),
            irq: next(),
            softirq: next(),
            steal: next(),
        })
    }

    /// Parse the aggregate `cpu` line of `/proc/stat`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let line = text.lines().find(|l| l.starts_with("cpu "))?;
        Self::from_line(line)
    }

    /// Parse the per-logical-CPU `cpuN` lines, in the order the kernel lists
    /// them.
    ///
    /// These are cumulative counters, like the aggregate line. A single sample
    /// cannot say what a CPU is doing *now*; the caller has to difference two
    /// samples, which is why the counters are exposed as ticks rather than as a
    /// percentage. A per-core percentage computed from one cumulative counter
    /// would be an average since boot wearing a live badge.
    #[must_use]
    pub fn parse_all(text: &str) -> Vec<Self> {
        text.lines()
            .filter(|l| {
                // `cpu0`, `cpu10`: the character after `cpu` is the digit of the
                // core number. The aggregate line is `cpu`, then a space.
                l.strip_prefix("cpu")
                    .and_then(|r| r.chars().next())
                    .is_some_and(|c| c.is_ascii_digit())
            })
            .filter_map(Self::from_line)
            .collect()
    }

    /// Every tick accounted for.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.user
            .saturating_add(self.nice)
            .saturating_add(self.system)
            .saturating_add(self.idle)
            .saturating_add(self.iowait)
            .saturating_add(self.irq)
            .saturating_add(self.softirq)
            .saturating_add(self.steal)
    }

    /// Ticks that were not idle. `iowait` counts as idle because during it the
    /// CPU was free for anything else that wanted it.
    #[must_use]
    pub const fn busy(&self) -> u64 {
        self.total()
            .saturating_sub(self.idle)
            .saturating_sub(self.iowait)
    }

    /// The fraction of ticks since boot that were not idle, 0.0 to 1.0.
    ///
    /// An average over the whole uptime, not a reading of what the machine is
    /// doing now: one sample of a cumulative counter cannot say that, and
    /// presenting it as though it could is how an idle machine gets reported as
    /// busy for the first minute after boot. The name says which it is.
    #[must_use]
    pub fn busy_fraction_since_boot(&self) -> Option<f64> {
        let total = self.total();
        if total == 0 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        Some((self.busy() as f64) / (total as f64))
    }

    /// Utilisation between two samples, 0.0 to 1.0.
    ///
    /// Returns `None` when no time passed between them, rather than dividing by
    /// zero and reporting a plausible-looking number.
    #[must_use]
    pub fn utilisation(earlier: Self, later: Self) -> Option<f64> {
        let dt = later.total().checked_sub(earlier.total())?;
        if dt == 0 {
            return None;
        }
        let db = later.busy().saturating_sub(earlier.busy());
        // Both fit in f64 exactly at any tick count a real machine reaches.
        #[allow(clippy::cast_precision_loss)]
        Some((db as f64) / (dt as f64))
    }
}

/// `/proc/loadavg`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LoadAvg {
    /// One-minute load.
    pub one: f64,
    /// Five-minute load.
    pub five: f64,
    /// Fifteen-minute load.
    pub fifteen: f64,
    /// Runnable tasks right now.
    pub running: u32,
    /// Total tasks.
    pub total: u32,
}

impl LoadAvg {
    /// Parse the contents of `/proc/loadavg`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut it = text.split_whitespace();
        let one = it.next()?.parse().ok()?;
        let five = it.next()?.parse().ok()?;
        let fifteen = it.next()?.parse().ok()?;
        let (running, total) = it
            .next()
            .and_then(|f| f.split_once('/'))
            .map_or((0, 0), |(r, t)| {
                (r.parse().unwrap_or(0), t.parse().unwrap_or(0))
            });
        Some(Self {
            one,
            five,
            fifteen,
            running,
            total,
        })
    }
}

/// One process's resident set, in kibibytes, from `/proc/<pid>/status`.
#[must_use]
pub fn parse_vm_rss_kb(status: &str) -> Option<u64> {
    status
        .lines()
        .find(|l| l.starts_with("VmRSS:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// The CPU's model name, from `/proc/cpuinfo`.
///
/// x86 kernels publish `model name`; arm64 kernels publish `Hardware` or
/// `Model` instead. Reading it from the running kernel is the point: a table of
/// model names compiled into the shell is wrong on the next machine, and the
/// value a user sees in a system monitor has to be the CPU that is in the
/// socket, not one the interface guessed.
///
/// A value made only of digits is refused: on x86 the first line of the file is
/// `processor : 0`, and "0" is not a model name.
#[must_use]
pub fn parse_cpu_model(text: &str) -> Option<String> {
    const KEYS: [&str; 3] = ["model name", "Hardware", "Model"];
    let mut best: Option<String> = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !KEYS.contains(&key.trim()) {
            continue;
        }
        let v = value.trim();
        if v.is_empty() || v.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        // `model name` wins wherever it appears; the arm64 keys are the
        // fallback for kernels that do not publish one.
        if key.trim() == "model name" {
            return Some(v.to_owned());
        }
        if best.is_none() {
            best = Some(v.to_owned());
        }
    }
    best
}

/// One mount, from `/proc/mounts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountSpec {
    /// Where it is mounted.
    pub mount: String,
    /// Filesystem type the kernel reports, e.g. `erofs`, `tmpfs`, `ext4`.
    pub fs: String,
    /// Whether the kernel mounted it read-only.
    pub read_only: bool,
    /// The device or pseudo-source the kernel names.
    pub source: String,
}

/// `/proc/mounts` escapes a space, tab and backslash in a path as an octal
/// sequence; a mount point printed with a literal space would be split in the
/// wrong place, so the escape has to be undone to name the real path.
#[must_use]
pub fn unescape_mount(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut chars = path.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let octal: String = chars.clone().take(3).collect();
        if octal.len() == 3 && octal.chars().all(|d| ('0'..='7').contains(&d)) {
            if let Ok(v) = u8::from_str_radix(&octal, 8) {
                out.push(char::from(v));
                for _ in 0..3 {
                    let _ = chars.next();
                }
                continue;
            }
        }
        out.push('\\');
    }
    out
}

/// Parse `/proc/mounts`.
///
/// Every mount is returned, including the kernel's own pseudo-filesystems,
/// because filtering is a presentation decision and the caller knows which
/// ones it means to hide. `source`, `mount` and `fs` are required: a line that
/// cannot supply all three is dropped rather than reported with blanks.
#[must_use]
pub fn parse_mounts(text: &str) -> Vec<MountSpec> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split_whitespace();
            let source = f.next()?;
            let mount = f.next()?;
            let fs = f.next()?;
            let options = f.next().unwrap_or_default();
            Some(MountSpec {
                source: unescape_mount(source),
                mount: unescape_mount(mount),
                fs: fs.to_owned(),
                read_only: options.split(',').any(|o| o == "ro"),
            })
        })
        .collect()
}

/// Whether MGLRU is enabled, from `/sys/kernel/mm/lru_gen/enabled`.
///
/// The file is a hex bitmask, not a boolean: any non-zero value means at least
/// one part of the multi-generational LRU is active.
#[must_use]
pub fn parse_mglru(text: &str) -> Option<bool> {
    let t = text.trim();
    let hex = t.strip_prefix("0x").unwrap_or(t);
    u64::from_str_radix(hex, 16).ok().map(|v| v != 0)
}

/// The fields of `/proc/<pid>/stat` a monitoring surface needs.
///
/// The file's second field is the process name in parentheses and it may contain
/// spaces *and* parentheses, so it cannot be split on whitespace -- a process
/// called `my (odd) name` is legal and a naive split reports its CPU time as its
/// thread count. The kernel's own rule is that the name is the last `(` to the
/// last `)`, which is what [`parse_proc_stat`] uses.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcStat {
    /// `R`, `S`, `D`, `Z`, `T`, `t`, `I` or `X`.
    pub state: char,
    /// Parent pid.
    pub ppid: u32,
    /// Ticks of user time, from the same clock as `/proc/stat`.
    pub utime: u64,
    /// Ticks of system time.
    pub stime: u64,
    /// Live threads.
    pub threads: u32,
    /// Virtual size in bytes.
    pub vsize: u64,
}

/// Parse the numeric fields of `/proc/<pid>/stat` after the process name.
#[must_use]
pub fn parse_proc_stat(text: &str) -> Option<ProcStat> {
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    if close < open {
        return None;
    }
    // Fields after `comm`: state, ppid, pgrp, session, tty, tpgid, flags,
    // minflt, cminflt, majflt, cmajflt, utime, stime, cutime, cstime, priority,
    // nice, num_threads, itrealvalue, starttime, vsize, rss ...
    let mut it = text.get(close.saturating_add(2)..)?.split_whitespace();
    let state = it.next()?.chars().next()?;
    let ppid = it.next()?.parse().ok()?;
    // `ppid` is field 4, so the next token to arrive is field 5.
    let mut nth = 4usize;
    let mut utime = None;
    let mut stime = None;
    let mut threads = None;
    let mut vsize = None;
    for field in it {
        nth = nth.saturating_add(1);
        match nth {
            14 => utime = field.parse().ok(),
            15 => stime = field.parse().ok(),
            20 => threads = field.parse().ok(),
            23 => vsize = field.parse().ok(),
            _ => {}
        }
    }
    Some(ProcStat {
        state,
        ppid,
        utime: utime?,
        stime: stime?,
        threads: threads?,
        vsize: vsize.unwrap_or(0),
    })
}

/// The real uid from `/proc/<pid>/status`, which is the first of the four
/// numbers on the `Uid:` line (real, effective, saved, filesystem).
#[must_use]
pub fn parse_uid(status: &str) -> Option<u32> {
    status
        .lines()
        .find(|l| l.starts_with("Uid:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// One process, as `/proc/<pid>/` reports it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessFacts {
    /// Process id.
    pub pid: u32,
    /// The name the kernel holds, from `/proc/<pid>/comm`.
    pub comm: String,
    /// The full command line, NUL bytes turned into spaces. Empty for a kernel
    /// thread, which has none -- an empty command line is a fact here, not a
    /// failed read.
    pub cmdline: String,
    /// The parent's pid.
    pub ppid: u32,
    /// Run state, as a character.
    pub state: char,
    /// Live threads.
    pub threads: u32,
    /// User ticks.
    pub utime: u64,
    /// System ticks.
    pub stime: u64,
    /// Resident set in kibibytes, when `status` was readable.
    pub rss_kb: Option<u64>,
    /// Virtual size in bytes.
    pub vsize: u64,
    /// Real uid, when `status` was readable. `None` is "not measured", which is
    /// not the same as uid 0.
    pub uid: Option<u32>,
}

impl ProcessFacts {
    /// Total CPU ticks this process has been given.
    #[must_use]
    pub const fn ticks(&self) -> u64 {
        self.utime.saturating_add(self.stime)
    }

    /// A one-character state a person can read, spelled out.
    #[must_use]
    pub const fn state_word(&self) -> &'static str {
        match self.state {
            'R' => "running",
            'S' => "sleeping",
            'D' => "uninterruptible",
            'Z' => "zombie",
            'T' | 't' => "stopped",
            'I' => "idle",
            'X' | 'x' => "dead",
            _ => "unknown",
        }
    }

    /// What to show as the process's name: the command line's first word when
    /// there is one, otherwise the kernel's `comm`, which is all a kernel thread
    /// has.
    #[must_use]
    pub fn display_name(&self) -> &str {
        let first = self.cmdline.split(' ').next().unwrap_or("");
        let base = first.rsplit('/').next().unwrap_or(first);
        if base.is_empty() {
            &self.comm
        } else {
            base
        }
    }
}

/// Whether a directory entry is a process id.
#[must_use]
fn is_pid(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit()) && !name.starts_with('0')
}

/// Read one process, or `None` if it went away between the listing and the read.
///
/// A process that exits mid-walk is normal, not an error: the whole point of a
/// process table is that it is a snapshot of something that keeps moving.
#[must_use]
pub fn read_process(root: &Path, pid: u32) -> Option<ProcessFacts> {
    let dir = format!("proc/{pid}");
    let stat = read(root, &format!("{dir}/stat"))?;
    let parsed = parse_proc_stat(&stat)?;
    let status = read(root, &format!("{dir}/status"));
    let cmdline = read(root, &format!("{dir}/cmdline")).map(|s| {
        s.trim_end_matches('\0')
            .split('\0')
            .collect::<Vec<_>>()
            .join(" ")
    });
    let comm = read(root, &format!("{dir}/comm"))
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();
    Some(ProcessFacts {
        pid,
        comm,
        cmdline: cmdline.unwrap_or_default(),
        ppid: parsed.ppid,
        state: parsed.state,
        threads: parsed.threads,
        utime: parsed.utime,
        stime: parsed.stime,
        rss_kb: status.as_deref().and_then(parse_vm_rss_kb),
        vsize: parsed.vsize,
        uid: status.as_deref().and_then(parse_uid),
    })
}

/// Every process the kernel lists, sorted by pid.
///
/// Bounded by `limit`: a machine with tens of thousands of processes must not be
/// able to make the daemon allocate without end, and the surface showing them
/// shows a visible list, not a count of everything that ever existed.
#[must_use]
pub fn read_processes(root: &Path, limit: usize) -> Vec<ProcessFacts> {
    let Ok(entries) = fs::read_dir(root.join("proc")) else {
        return Vec::new();
    };
    let mut pids: Vec<u32> = entries
        .flatten()
        .filter_map(|e| e.file_name().to_string_lossy().into_owned().into())
        .filter(|n: &String| is_pid(n))
        .filter_map(|n| n.parse::<u32>().ok())
        .collect();
    pids.sort_unstable();
    pids.truncate(limit);
    pids.into_iter()
        .filter_map(|pid| read_process(root, pid))
        .collect()
}

/// Per-process CPU share between two samples, as a fraction of the whole machine.
///
/// The denominator is the machine's tick delta, not one core's, so a process
/// using one core fully on an eight-core machine reads 0.125 and a process using
/// all eight reads 1.0. That is the honest number for "how much of this machine
/// is this program using"; a per-core share that can read 800% is a different
/// question, and the monitor answers it by showing the core count next to it.
///
/// A process that is new in `later` has no delta and reports 0, and one that has
/// vanished is absent: neither is an error, and inventing a value for either is
/// how a monitor starts lying.
#[must_use]
pub fn cpu_shares(
    earlier: &[ProcessFacts],
    later: &[ProcessFacts],
    machine_ticks: u64,
) -> Vec<(u32, f64)> {
    if machine_ticks == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(later.len());
    for now in later {
        let before = earlier.iter().find(|p| p.pid == now.pid);
        let delta = match before {
            Some(b) if now.ticks() >= b.ticks() => now.ticks().saturating_sub(b.ticks()),
            // A pid that was reused: the difference is meaningless, so it is not
            // reported as a spike.
            _ => 0,
        };
        let share = (delta as f64) / (machine_ticks as f64);
        out.push((now.pid, share.clamp(0.0, 1.0)));
    }
    out
}

/// Everything worth reporting about the running system.
#[derive(Debug, Clone, Default)]
pub struct Facts {
    /// Memory, if `/proc/meminfo` was readable.
    pub mem: Option<MemInfo>,
    /// Uptime, if `/proc/uptime` was readable.
    pub uptime: Option<Uptime>,
    /// CPU ticks, if `/proc/stat` was readable.
    pub cpu: Option<CpuTimes>,
    /// Load, if `/proc/loadavg` was readable.
    pub load: Option<LoadAvg>,
    /// This process's own resident set. A system daemon that cannot say what it
    /// costs has no business reporting anyone else's footprint.
    pub self_rss_kb: Option<u64>,
    /// Whether MGLRU is active.
    pub mglru: Option<bool>,
    /// The kernel command line.
    pub cmdline: Option<String>,
    /// Online CPU count.
    pub cpus: Option<u32>,
    /// The CPU's model name, from `/proc/cpuinfo`.
    pub cpu_model: Option<String>,
    /// Cumulative tick counters per logical CPU. Empty when `/proc/stat`
    /// published no per-CPU lines.
    pub per_cpu: Vec<CpuTimes>,
    /// Mounted filesystems, from `/proc/mounts`.
    pub mounts: Vec<MountSpec>,
    /// The process table, from `/proc/<pid>/`. Cumulative tick counters, so the
    /// reader of two samples can difference them; empty when `/proc` is not
    /// mounted, which a container without `--pid=host` can do.
    pub processes: Vec<ProcessFacts>,
}

fn read(root: &Path, rel: &str) -> Option<String> {
    fs::read_to_string(root.join(rel)).ok()
}

/// Percent-escape what a protocol field cannot carry literally.
///
/// Only `%` and `|` need it: spaces are fine inside a field because the client
/// splits on the separator, not on whitespace. Escaping fewer characters keeps
/// a command line readable in the reply, which is the reason the protocol is
/// text at all.
#[must_use]
pub fn escape_field(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '|' => out.push_str("%7C"),
            c => out.push(c),
        }
    }
    out
}

/// Count CPUs from the `/sys/devices/system/cpu/online` range list, e.g.
/// `0-3`, `0`, or `0-1,4-7`.
#[must_use]
pub fn parse_cpu_online(text: &str) -> Option<u32> {
    let mut n: u32 = 0;
    for part in text.trim().split(',') {
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((a, b)) => {
                let lo: u32 = a.parse().ok()?;
                let hi: u32 = b.parse().ok()?;
                n = n.saturating_add(hi.saturating_sub(lo).saturating_add(1));
            }
            None => {
                part.parse::<u32>().ok()?;
                n = n.saturating_add(1);
            }
        }
    }
    if n == 0 {
        None
    } else {
        Some(n)
    }
}

/// Read every fact from a filesystem root.
///
/// `root` is `/` in production and a fixture directory in the tests. Anything
/// unreadable comes back as `None` rather than as a default, because a zero
/// that is indistinguishable from a real zero is how a monitoring system starts
/// lying.
#[must_use]
pub fn read_facts(root: &Path) -> Facts {
    Facts {
        mem: read(root, "proc/meminfo")
            .as_deref()
            .and_then(MemInfo::parse),
        uptime: read(root, "proc/uptime").as_deref().and_then(Uptime::parse),
        cpu: read(root, "proc/stat").as_deref().and_then(CpuTimes::parse),
        load: read(root, "proc/loadavg")
            .as_deref()
            .and_then(LoadAvg::parse),
        self_rss_kb: read(root, "proc/self/status")
            .as_deref()
            .and_then(parse_vm_rss_kb),
        mglru: read(root, "sys/kernel/mm/lru_gen/enabled")
            .as_deref()
            .and_then(parse_mglru),
        cmdline: read(root, "proc/cmdline").map(|s| s.trim().to_owned()),
        cpus: read(root, "sys/devices/system/cpu/online")
            .as_deref()
            .and_then(parse_cpu_online),
        cpu_model: read(root, "proc/cpuinfo")
            .as_deref()
            .and_then(parse_cpu_model),
        per_cpu: read(root, "proc/stat")
            .as_deref()
            .map_or_else(Vec::new, CpuTimes::parse_all),
        mounts: read(root, "proc/mounts")
            .as_deref()
            .map_or_else(Vec::new, parse_mounts),
        processes: read_processes(root, MAX_PROCESSES),
    }
}

/// How many processes one `FACTS` reply carries.
///
/// A process table is a window onto a list that can be thousands long, and the
/// daemon's reply is one text line per process. 512 is more rows than any
/// window shows and small enough that the reply stays inside the protocol's own
/// line budget; a machine with more gets the lowest 512 pids, which is stated in
/// the reply (`litric.proc.truncated=1`) rather than silently implied.
pub const MAX_PROCESSES: usize = 512;

impl Facts {
    /// The value of one `key=value` parameter on the kernel command line.
    ///
    /// The last occurrence wins, which is how the kernel itself resolves a
    /// repeated parameter. Reading the first would report a setting the running
    /// kernel is not using.
    #[must_use]
    pub fn cmdline_param(&self, key: &str) -> Option<&str> {
        self.cmdline
            .as_deref()?
            .split_whitespace()
            .rev()
            .find_map(|w| {
                w.split_once('=')
                    .and_then(|(k, v)| if k == key { Some(v) } else { None })
            })
    }

    /// Whether the root filesystem is under dm-verity, according to the command
    /// line the kernel was actually given.
    #[must_use]
    pub fn root_verity(&self) -> Option<bool> {
        self.cmdline.as_ref().map(|c| c.contains("roothash="))
    }

    /// The facts as `key=value` lines, in a stable order.
    ///
    /// Stable because `tools/boot-test.sh` and the in-guest selftest both grep
    /// this output; a reordering would silently break a check that reports
    /// success by absence.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(m) = self.mem {
            out.push(format!("litric.mem.total_kb={}", m.total_kb));
            out.push(format!("litric.mem.available_kb={}", m.available_kb));
            out.push(format!("litric.mem.used_kb={}", m.used_kb()));
            out.push(format!("litric.mem.cached_kb={}", m.cached_kb));
            out.push(format!("litric.mem.swap_used_kb={}", m.swap_used_kb()));
        }
        if let Some(u) = self.uptime {
            out.push(format!("litric.uptime_secs={:.2}", u.up_secs));
        }
        if let Some(l) = self.load {
            out.push(format!("litric.load1={:.2}", l.one));
            out.push(format!("litric.tasks={}", l.total));
        }
        if let Some(c) = self.cpus {
            out.push(format!("litric.cpus={c}"));
        }
        if let Some(r) = self.self_rss_kb {
            out.push(format!("litric.sysd.rss_kb={r}"));
        }
        if let Some(g) = self.mglru {
            out.push(format!("litric.mglru={}", if g { "on" } else { "off" }));
        }
        if let Some(f) = self.cpu.and_then(|c| c.busy_fraction_since_boot()) {
            out.push(format!("litric.cpu.busy_since_boot_pct={:.1}", f * 100.0));
        }
        // Cumulative ticks, so the shell can difference two of its own samples
        // and show what the CPU is doing now rather than an average since boot.
        if let Some(c) = self.cpu {
            out.push(format!("litric.cpu.busy_ticks={}", c.busy()));
            out.push(format!("litric.cpu.total_ticks={}", c.total()));
        }
        for (i, c) in self.per_cpu.iter().enumerate() {
            out.push(format!("litric.cpu.core{i}.busy_ticks={}", c.busy()));
            out.push(format!("litric.cpu.core{i}.total_ticks={}", c.total()));
        }
        if let Some(model) = &self.cpu_model {
            out.push(format!("litric.cpu.model={model}"));
        }
        for (i, m) in self.mounts.iter().enumerate() {
            out.push(format!(
                "litric.mount.{i}={}|{}|{}|{}",
                m.mount,
                m.fs,
                if m.read_only { "ro" } else { "rw" },
                m.source
            ));
        }
        if let Some(v) = self.root_verity() {
            out.push(format!(
                "litric.root.verity={}",
                if v { "on" } else { "off" }
            ));
        }
        if let Some(l) = self.cmdline_param("lsm") {
            out.push(format!("litric.lsm={l}"));
        }
        // One line per process. Fields are `|`-separated and the name is
        // percent-escaped, because a command line contains spaces, `|` and
        // anything else a shell allows, and a reply the client has to guess how
        // to split is a reply that breaks on the first process called `a|b`.
        for p in &self.processes {
            out.push(format!(
                "litric.proc={}|{}|{}|{}|{}|{}|{}|{}|{}",
                p.pid,
                escape_field(p.display_name()),
                p.state,
                p.ppid,
                p.threads,
                p.ticks(),
                p.rss_kb.map_or_else(|| "-".to_owned(), |v| v.to_string()),
                p.uid.map_or_else(|| "-".to_owned(), |v| v.to_string()),
                escape_field(&p.cmdline)
            ));
        }
        if !self.processes.is_empty() {
            out.push(format!("litric.proc.count={}", self.processes.len()));
            if self.processes.len() >= MAX_PROCESSES {
                out.push("litric.proc.truncated=1".to_owned());
            }
        }
        out
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    const MEMINFO: &str = "\
MemTotal:         946100 kB
MemFree:          612344 kB
MemAvailable:     801220 kB
Buffers:            3120 kB
Cached:           182904 kB
SwapCached:            0 kB
Shmem:             41220 kB
SwapTotal:        262140 kB
SwapFree:         262140 kB
";

    #[test]
    fn meminfo_parses_the_fields_that_matter() {
        let m = MemInfo::parse(MEMINFO).expect("parses");
        assert_eq!(m.total_kb, 946_100);
        assert_eq!(m.available_kb, 801_220);
        assert_eq!(m.cached_kb, 182_904);
        assert_eq!(m.shmem_kb, 41_220);
        assert_eq!(m.swap_used_kb(), 0);
    }

    #[test]
    fn used_memory_is_measured_against_available_not_free() {
        let m = MemInfo::parse(MEMINFO).expect("parses");
        // total - available = 144_880. total - free would be 333_756, which
        // would describe a mostly-idle machine as two-thirds consumed.
        assert_eq!(m.used_kb(), 144_880);
        assert_ne!(m.used_kb(), m.total_kb - m.free_kb);
    }

    #[test]
    fn meminfo_without_memtotal_is_rejected_rather_than_defaulted() {
        assert!(MemInfo::parse("Cached: 100 kB\n").is_none());
        assert!(MemInfo::parse("").is_none());
        assert!(MemInfo::parse("this is not meminfo").is_none());
    }

    #[test]
    fn meminfo_survives_junk_lines() {
        let m = MemInfo::parse("MemTotal: 100 kB\nBroken\nCached: notanumber kB\n")
            .expect("MemTotal present");
        assert_eq!(m.total_kb, 100);
        assert_eq!(m.cached_kb, 0);
    }

    #[test]
    fn uptime_parses() {
        let u = Uptime::parse("9.17 34.55\n").expect("parses");
        assert!((u.up_secs - 9.17).abs() < 1e-9);
        assert!((u.idle_secs - 34.55).abs() < 1e-9);
        assert!(Uptime::parse("").is_none());
    }

    #[test]
    fn cpu_times_split_busy_from_idle_with_iowait_on_the_idle_side() {
        let c = CpuTimes::parse("cpu  100 20 50 800 30 5 5 0\ncpu0 1 2 3 4\n").expect("parses");
        assert_eq!(c.user, 100);
        assert_eq!(c.idle, 800);
        assert_eq!(c.iowait, 30);
        assert_eq!(c.total(), 1010);
        // busy excludes idle AND iowait: 100+20+50+5+5 = 180.
        assert_eq!(c.busy(), 180);
    }

    #[test]
    fn cpu_line_must_be_the_aggregate_not_a_per_core_line() {
        // "cpu0" must not be mistaken for "cpu ": the prefix has a space.
        let c = CpuTimes::parse("cpu0 1 2 3 4\n");
        assert!(c.is_none(), "per-core line was accepted as the aggregate");
    }

    #[test]
    fn busy_since_boot_is_an_average_and_says_so_when_it_cannot_be_computed() {
        let c = CpuTimes::parse("cpu  100 20 50 800 30 5 5 0").expect("parses");
        let f = c.busy_fraction_since_boot().expect("ticks elapsed");
        assert!((f - 180.0 / 1010.0).abs() < 1e-9, "got {f}");
        assert!(
            CpuTimes::default().busy_fraction_since_boot().is_none(),
            "no ticks means no answer, not 0%"
        );
    }

    #[test]
    fn utilisation_needs_two_distinct_samples() {
        let a = CpuTimes::parse("cpu  100 0 0 900 0 0 0 0").expect("a");
        let b = CpuTimes::parse("cpu  200 0 0 1800 0 0 0 0").expect("b");
        let u = CpuTimes::utilisation(a, b).expect("time passed");
        assert!((u - 0.1).abs() < 1e-9, "got {u}");
        assert!(
            CpuTimes::utilisation(a, a).is_none(),
            "no elapsed time must be None, not 0% or NaN"
        );
    }

    #[test]
    fn utilisation_rejects_a_counter_that_went_backwards() {
        let a = CpuTimes::parse("cpu  200 0 0 1800 0 0 0 0").expect("a");
        let b = CpuTimes::parse("cpu  100 0 0 900 0 0 0 0").expect("b");
        assert!(CpuTimes::utilisation(a, b).is_none());
    }

    #[test]
    fn loadavg_parses_including_the_task_fraction() {
        let l = LoadAvg::parse("0.52 0.31 0.18 2/143 918\n").expect("parses");
        assert!((l.one - 0.52).abs() < 1e-9);
        assert_eq!(l.running, 2);
        assert_eq!(l.total, 143);
    }

    #[test]
    fn vm_rss_parses() {
        let s = "Name:\tlitric-sysd\nVmPeak:\t   12000 kB\nVmRSS:\t    2148 kB\n";
        assert_eq!(parse_vm_rss_kb(s), Some(2148));
        assert_eq!(parse_vm_rss_kb("Name: x\n"), None);
    }

    #[test]
    fn mglru_is_a_bitmask_not_a_boolean() {
        assert_eq!(parse_mglru("0x0007\n"), Some(true));
        assert_eq!(parse_mglru("0007"), Some(true));
        assert_eq!(parse_mglru("0x0000"), Some(false));
        assert_eq!(parse_mglru("y"), None);
    }

    #[test]
    fn cpu_online_handles_ranges_and_lists() {
        assert_eq!(parse_cpu_online("0-3\n"), Some(4));
        assert_eq!(parse_cpu_online("0"), Some(1));
        assert_eq!(parse_cpu_online("0-1,4-7"), Some(6));
        assert_eq!(parse_cpu_online(""), None);
    }

    #[test]
    fn reading_a_root_that_has_no_proc_yields_nothing_rather_than_zeroes() {
        let f = read_facts(Path::new("definitely-not-a-real-root-9f3a"));
        assert!(f.mem.is_none());
        assert!(f.uptime.is_none());
        assert!(f.to_lines().is_empty(), "absent facts must not be reported");
    }

    #[test]
    fn cmdline_parameters_are_read_with_the_last_occurrence_winning() {
        let f = Facts {
            cmdline: Some(
                "ro lsm=landlock,yama roothash=abc123 lsm=landlock,lockdown,yama".to_owned(),
            ),
            ..Facts::default()
        };
        // The kernel takes the last one; reporting the first would describe a
        // security posture the machine is not actually running.
        assert_eq!(f.cmdline_param("lsm"), Some("landlock,lockdown,yama"));
        assert_eq!(f.root_verity(), Some(true));
        assert_eq!(f.cmdline_param("quiet"), None);
    }

    #[test]
    fn a_root_without_verity_reports_off_rather_than_nothing() {
        let f = Facts {
            cmdline: Some("ro console=ttyS0".to_owned()),
            ..Facts::default()
        };
        assert_eq!(f.root_verity(), Some(false));
        assert!(f.to_lines().contains(&"litric.root.verity=off".to_owned()));
    }

    const CPUINFO_X86: &str = "\
processor : 0
vendor_id : AuthenticAMD
model : 33
model name : AMD Ryzen 7 5800X 8-Core Processor
stepping : 0
";

    const CPUINFO_ARM: &str = "\
Processor : AArch64 Processor rev 1 (aarch64)
Hardware : Qualcomm Technologies, Inc SM8350
";

    #[test]
    fn the_cpu_model_comes_from_the_kernel_not_from_a_table() {
        assert_eq!(
            parse_cpu_model(CPUINFO_X86).as_deref(),
            Some("AMD Ryzen 7 5800X 8-Core Processor")
        );
        assert_eq!(
            parse_cpu_model(CPUINFO_ARM).as_deref(),
            Some("Qualcomm Technologies, Inc SM8350")
        );
        // No model published: nothing is reported, rather than a guess.
        assert_eq!(parse_cpu_model("processor : 0\n"), None);
        assert_eq!(parse_cpu_model(""), None);
    }

    const STAT_PER_CORE: &str = "\
cpu  100 0 50 850 0 0 0 0
cpu0 60 0 30 400 0 0 0 0
cpu1 40 0 20 450 0 0 0 0
intr 12345
";

    #[test]
    fn per_cpu_counters_are_read_in_order_and_never_from_the_aggregate() {
        let all = CpuTimes::parse_all(STAT_PER_CORE);
        assert_eq!(all.len(), 2, "two logical CPUs, not the aggregate line");
        assert_eq!(all.first().map(|c| c.user), Some(60));
        assert_eq!(all.get(1).map(|c| c.user), Some(40));
        // The aggregate line stays available for the whole-machine figure.
        assert_eq!(CpuTimes::parse(STAT_PER_CORE).map(|c| c.user), Some(100));
        // A file with no per-CPU lines yields nothing, never one invented core.
        assert!(CpuTimes::parse_all("intr 1\n").is_empty());
    }

    #[test]
    fn utilisation_is_the_ratio_of_the_difference_between_two_samples() {
        let earlier = CpuTimes {
            user: 100,
            idle: 900,
            ..CpuTimes::default()
        };
        let later = CpuTimes {
            user: 150,
            idle: 950,
            ..CpuTimes::default()
        };
        let u = CpuTimes::utilisation(earlier, later).unwrap_or(f64::NAN);
        assert!((u - 0.5).abs() < 1e-9, "50 busy ticks of 100 elapsed: {u}");
        // Two samples of the same instant cannot produce a rate.
        assert_eq!(CpuTimes::utilisation(earlier, earlier), None);
    }

    const MOUNTS: &str = "\
/dev/vda / erofs ro,relatime,seclabel 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=102400k 0 0
/dev/vdb /persist ext4 rw,relatime 0 0
/dev/vdc /media/My\\040Disk vfat ro,relatime 0 0
";

    #[test]
    fn mounts_report_read_only_and_unescape_the_mount_point() {
        let m = parse_mounts(MOUNTS);
        assert_eq!(m.len(), 5);
        let root = m.first();
        assert_eq!(root.map(|x| x.fs.as_str()), Some("erofs"));
        assert_eq!(root.map(|x| x.read_only), Some(true));
        let weird = m.iter().find(|x| x.source == "/dev/vdc");
        assert_eq!(weird.map(|x| x.mount.as_str()), Some("/media/My Disk"));
        // A line that cannot name source, mount and type is dropped rather than
        // reported with blanks.
        assert!(parse_mounts("/dev/vda /only-two-fields\n").is_empty());
    }

    #[test]
    fn facts_read_from_a_fixture_root_and_report_stable_lines() {
        // Proves the reader as well as the parsers, on a host with no /proc.
        let root = std::env::temp_dir().join("litric-sysd-facts-fixture");
        let proc = root.join("proc");
        fs::create_dir_all(&proc).expect("fixture dirs");
        fs::write(proc.join("meminfo"), MEMINFO).expect("meminfo");
        fs::write(proc.join("uptime"), "9.17 34.55\n").expect("uptime");
        fs::write(proc.join("loadavg"), "0.52 0.31 0.18 2/143 918\n").expect("loadavg");

        let f = read_facts(&root);
        let lines = f.to_lines();
        assert!(
            lines.contains(&"litric.mem.used_kb=144880".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"litric.uptime_secs=9.17".to_owned()),
            "{lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.starts_with("litric.mglru")),
            "a fact that was not readable must not appear at all"
        );

        fs::remove_dir_all(&root).ok();
    }

    /// A `/proc/<pid>/stat` line, with a name that contains a space and two
    /// parentheses. This is the case a whitespace split gets wrong: the fields
    /// after the name shift by three and every number read is the wrong one.
    const ODD_STAT: &str = "4242 (my (odd) name) S 1 4242 4242 0 -1 4194560 100 0 0 0 \
                           12 34 0 0 20 0 3 0 12345 67890123 456 \
                           18446744073709551615 1 1 0 0 0 0 0 0 0 0 0 0 17 2 0 0 0 0 0 0 0 0 0 0 0 0 0";

    #[test]
    fn a_process_name_with_spaces_and_parens_does_not_shift_the_fields() {
        let s = parse_proc_stat(ODD_STAT).expect("parses");
        assert_eq!(s.state, 'S');
        assert_eq!(s.ppid, 1);
        assert_eq!(s.utime, 12, "utime is field 14, not the 12th token");
        assert_eq!(s.stime, 34);
        assert_eq!(s.threads, 3);
        assert_eq!(s.vsize, 67_890_123);
    }

    #[test]
    fn a_process_with_no_name_in_parens_is_refused_rather_than_guessed() {
        assert!(parse_proc_stat("1234 S 1 2 3").is_none());
        assert!(parse_proc_stat("(no pid) S 1").is_none());
    }

    #[test]
    fn the_real_uid_is_the_first_of_the_four_on_the_uid_line() {
        let status = "Name:\tsysd\nUid:\t0\t1000\t1000\t1000\nVmRSS:\t\t2048 kB\n";
        assert_eq!(parse_uid(status), Some(0));
        assert_eq!(parse_vm_rss_kb(status), Some(2048));
        assert_eq!(parse_uid("Name:\tx\n"), None);
    }

    #[test]
    fn a_process_walks_the_real_layout_of_a_proc_directory() {
        // The reader, not just the parsers: a process directory with a space in
        // its command line, a kernel thread with an empty one, and a directory
        // whose name is not a pid.
        let root = std::env::temp_dir().join("litric-sysd-proc-fixture");
        let proc = root.join("proc");
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(proc.join("4242")).expect("4242");
        fs::create_dir_all(proc.join("7")).expect("7");
        fs::create_dir_all(proc.join("acpi")).expect("not a pid");
        fs::write(proc.join("4242/stat"), ODD_STAT).expect("stat");
        fs::write(
            proc.join("4242/status"),
            "Uid:\t1000\t1000\t1000\t1000\nVmRSS:\t\t4096 kB\n",
        )
        .expect("status");
        fs::write(
            proc.join("4242/cmdline"),
            "/usr/bin/litric-files\0--open\0/Documents\0",
        )
        .expect("cmdline");
        fs::write(proc.join("4242/comm"), "litric-files\n").expect("comm");
        fs::write(
            proc.join("7/stat"),
            "7 (kworker/0:1) I 2 0 0 0 -1 0 0 0 0 0 3 1 0 0 20 0 1 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0 0 0 0 0 0 0 0",
        )
        .expect("stat");
        fs::write(proc.join("7/cmdline"), "\0\0").expect("empty cmdline");
        fs::write(proc.join("7/comm"), "kworker/0:1\n").expect("comm");

        let ps = read_processes(&root, 32);
        assert_eq!(ps.len(), 2, "only the two pid directories: {ps:?}");
        assert_eq!(ps.first().map(|p| p.pid), Some(7));
        let odd = ps.iter().find(|p| p.pid == 4242).expect("4242");
        assert_eq!(odd.display_name(), "litric-files", "the command line wins");
        assert_eq!(odd.cmdline, "/usr/bin/litric-files --open /Documents");
        assert_eq!(odd.rss_kb, Some(4096));
        assert_eq!(odd.uid, Some(1000));
        assert_eq!(odd.ticks(), 46);
        let kernel = ps.iter().find(|p| p.pid == 7).expect("7");
        assert_eq!(
            kernel.display_name(),
            "kworker/0:1",
            "a kernel thread has no command line, so the kernel's name is what it is called"
        );
        assert_eq!(kernel.state_word(), "idle");

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cpu_share_is_a_fraction_of_the_whole_machine_and_fabricates_nothing() {
        let before = vec![
            ProcessFacts {
                pid: 10,
                utime: 100,
                stime: 0,
                ..ProcessFacts::default()
            },
            ProcessFacts {
                pid: 11,
                utime: 5,
                stime: 0,
                ..ProcessFacts::default()
            },
        ];
        let after = vec![
            ProcessFacts {
                pid: 10,
                utime: 300,
                stime: 0,
                ..ProcessFacts::default()
            },
            // 11 is gone; 12 is new and has no earlier sample.
            ProcessFacts {
                pid: 12,
                utime: 9_999,
                stime: 0,
                ..ProcessFacts::default()
            },
        ];
        let shares = cpu_shares(&before, &after, 800);
        assert_eq!(shares.len(), 2, "the vanished process is absent, not zero");
        let of = |pid: u32| shares.iter().find(|(p, _)| *p == pid).map(|(_, s)| *s);
        assert_eq!(
            of(10),
            Some(0.25),
            "200 of 800 ticks is a quarter of the box"
        );
        assert_eq!(of(12), Some(0.0), "a new process has no rate yet");
        assert!(cpu_shares(&before, &after, 0).is_empty());
    }

    #[test]
    fn a_counter_that_went_backwards_is_not_a_spike() {
        // Pids are reused. When one is, the delta is negative and reporting it
        // as a large positive number is the classic top-level false alarm.
        let before = vec![ProcessFacts {
            pid: 99,
            utime: 5_000,
            ..ProcessFacts::default()
        }];
        let after = vec![ProcessFacts {
            pid: 99,
            utime: 3,
            ..ProcessFacts::default()
        }];
        assert_eq!(cpu_shares(&before, &after, 100), vec![(99, 0.0)]);
    }

    #[test]
    fn the_process_reply_escapes_the_characters_that_would_break_it() {
        let f = Facts {
            processes: vec![ProcessFacts {
                pid: 3,
                comm: "a|b".to_owned(),
                cmdline: "/bin/sh -c 'echo 100%|done'".to_owned(),
                state: 'R',
                ppid: 1,
                threads: 1,
                utime: 2,
                stime: 1,
                rss_kb: Some(12),
                uid: Some(1000),
                ..ProcessFacts::default()
            }],
            ..Facts::default()
        };
        let lines = f.to_lines();
        let proc = lines
            .iter()
            .find(|l| l.starts_with("litric.proc="))
            .expect("a process line");
        assert_eq!(
            proc.matches('|').count(),
            8,
            "nine fields, eight separators"
        );
        assert!(!proc.contains("a|b"), "the name's pipe is escaped");
        assert!(lines.contains(&"litric.proc.count=1".to_owned()));
    }
}
