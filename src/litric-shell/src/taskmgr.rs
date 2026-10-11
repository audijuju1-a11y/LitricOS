//! The Task Manager: the system monitor's process table, as a model.
//!
//! # Why a model and not a drawing function
//!
//! A process table is the one surface where every column, every sort order and
//! every threshold is something a person sets once and then relies on for years,
//! and where getting the setting wrong means diagnosing the wrong process. That
//! makes it exactly the wrong place to bury choices in drawing code: the table
//! is drawn *from* a [`TaskManager`], so a column that is not in the model
//! cannot be drawn, and a preference that is not applied here cannot appear to
//! work in a screenshot.
//!
//! # The two halves
//!
//! * [`Prefs`] -- everything the user can change about the table: which columns,
//!   in what order, at what width, sorted by what, grouped how, formatted as
//!   what, with which thresholds.
//! * [`TaskManager`] -- the same, plus the rows and the interaction state
//!   (selection, filter text, which group is open).
//!
//! The rows come from [`crate::shell::SystemFacts::processes`], which the daemon
//! fills from `/proc`. Nothing here invents a number: a field the machine did
//! not report is `None` all the way to the cell, which prints a dash.
//!
//! # The feature registry
//!
//! [`features`] enumerates every one of those settings as data: an id, a label,
//! a category, and the change it makes. The launcher searches it, the tests
//! count it, and every entry's `apply` writes into a [`Prefs`] field that the
//! table reads -- so an entry that does nothing cannot be added without a test
//! failing. That is the property the previous treatment of this window was
//! missing: it had a list of "features" that no code consulted.

use crate::shell::{ProcRow, SystemFacts};
use crate::ui::Tone;

/// A column of the process table.
///
/// The set is fixed by what a daemon is asked for, not by what looks good in a
/// mock: each variant names a field [`ProcRow`] carries from `/proc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Column {
    /// The program's name.
    Name,
    /// Process id.
    Pid,
    /// Parent process id.
    Parent,
    /// Run state, spelled out.
    State,
    /// Real uid.
    User,
    /// Share of the machine, live.
    Cpu,
    /// Cumulative CPU ticks.
    CpuTime,
    /// Resident set.
    Memory,
    /// Resident set as a share of installed memory.
    MemShare,
    /// Live threads.
    Threads,
    /// The full command line.
    CommandLine,
}

impl Column {
    /// Every column, in the order a new table shows them.
    pub const ALL: [Self; 11] = [
        Self::Name,
        Self::Pid,
        Self::Parent,
        Self::State,
        Self::User,
        Self::Cpu,
        Self::CpuTime,
        Self::Memory,
        Self::MemShare,
        Self::Threads,
        Self::CommandLine,
    ];

    /// The heading, as a person reads it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Pid => "PID",
            Self::Parent => "Parent",
            Self::State => "Status",
            Self::User => "User",
            Self::Cpu => "CPU",
            Self::CpuTime => "CPU time",
            Self::Memory => "Memory",
            Self::MemShare => "% Memory",
            Self::Threads => "Threads",
            Self::CommandLine => "Command line",
        }
    }

    /// What the column means, for the "what is this" affordance.
    #[must_use]
    pub const fn help(self) -> &'static str {
        match self {
            Self::Name => "The program that is running, from its command line.",
            Self::Pid => "The process id the kernel gave it. It is reused after a process ends.",
            Self::Parent => "The process that started it. Ending a tree ends everything under it.",
            Self::State => {
                "Running, sleeping, or waiting on a device. A zombie has ended but has not been \
                 collected by its parent yet."
            }
            Self::User => "The user the process runs as, not the user who started it.",
            Self::Cpu => {
                "Share of the whole machine over the last sample. Eight busy cores read 100%, \
                 not 800%."
            }
            Self::CpuTime => "Processor time used since it started, not a rate.",
            Self::Memory => {
                "Resident set: the memory actually in RAM for this process, not what it has \
                 reserved."
            }
            Self::MemShare => "The resident set as a share of installed memory.",
            Self::Threads => "Live threads in the process.",
            Self::CommandLine => "The full arguments the process was started with.",
        }
    }

    /// Which field this column sorts by.
    #[must_use]
    pub const fn sort_key(self) -> SortKey {
        match self {
            Self::Name => SortKey::Name,
            Self::Pid | Self::Parent => SortKey::Pid,
            Self::State => SortKey::State,
            Self::User => SortKey::User,
            Self::Cpu => SortKey::Cpu,
            Self::CpuTime => SortKey::CpuTime,
            Self::Memory => SortKey::Memory,
            Self::MemShare => SortKey::MemoryShare,
            Self::Threads => SortKey::Threads,
            Self::CommandLine => SortKey::CommandLine,
        }
    }

    /// The natural width of the column, in reference pixels.
    #[must_use]
    pub const fn width(self) -> f32 {
        match self {
            Self::Name => 150.0,
            Self::Pid => 64.0,
            Self::Parent => 64.0,
            Self::State => 90.0,
            Self::User => 72.0,
            Self::Cpu | Self::MemShare => 66.0,
            Self::CpuTime | Self::Memory => 88.0,
            Self::Threads => 66.0,
            Self::CommandLine => 320.0,
        }
    }

    /// What a cell of this column holds for one process.
    #[must_use]
    pub fn cell(self, p: &ProcRow, prefs: &Prefs, total_mb: Option<u64>) -> String {
        format_cell(self, p, prefs, total_mb)
    }
}

/// What a column is sorted by, which is not always the column that was clicked:
/// memory and its share are the same field in two units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortKey {
    /// Name, case-insensitively.
    Name,
    /// Process id.
    Pid,
    /// Run state.
    State,
    /// Real uid.
    User,
    /// Live CPU share.
    Cpu,
    /// Cumulative CPU ticks.
    CpuTime,
    /// Resident set.
    Memory,
    /// Resident set as a share of installed memory.
    MemoryShare,
    /// Live threads.
    Threads,
    /// Full command line.
    CommandLine,
}

impl SortKey {
    /// Every sort key.
    pub const ALL: [Self; 10] = [
        Self::Name,
        Self::Pid,
        Self::State,
        Self::User,
        Self::Cpu,
        Self::CpuTime,
        Self::Memory,
        Self::MemoryShare,
        Self::Threads,
        Self::CommandLine,
    ];

    /// The name this key reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Pid => "pid",
            Self::State => "status",
            Self::User => "user",
            Self::Cpu => "cpu",
            Self::CpuTime => "cpu time",
            Self::Memory => "memory",
            Self::MemoryShare => "memory share",
            Self::Threads => "threads",
            Self::CommandLine => "command line",
        }
    }

    /// The comparable value of one process under this key.
    fn key(self, p: &ProcRow, total_mb: Option<u64>) -> (f64, String) {
        match self {
            Self::Name => (0.0, p.name.to_lowercase()),
            Self::Pid => (p.pid as f64, String::new()),
            Self::State => (0.0, p.state.clone()),
            Self::User => (f64::from(p.uid.unwrap_or(u32::MAX)), String::new()),
            Self::Cpu => (f64::from(p.cpu.unwrap_or(-1.0)), String::new()),
            Self::CpuTime => (p.ticks as f64, String::new()),
            Self::Memory => (p.rss_mb.unwrap_or(0) as f64, String::new()),
            Self::MemoryShare => (share_pct(p, total_mb), String::new()),
            Self::Threads => (f64::from(p.threads), String::new()),
            Self::CommandLine => (0.0, p.cmdline.to_lowercase()),
        }
    }
}

/// Resident set as a percentage of installed memory, or `-1.0` when either half
/// is unknown, so an unknown share sorts below every measured one instead of
/// pretending to be zero.
#[must_use]
pub fn share_pct(p: &ProcRow, total_mb: Option<u64>) -> f64 {
    match (p.rss_mb, total_mb) {
        (Some(rss), Some(total)) if total > 0 => (rss as f64 / total as f64) * 100.0,
        _ => -1.0,
    }
}

/// How the table groups rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupKey {
    /// One flat list, sorted by the sort key.
    None,
    /// One group per program name, the heaviest group first.
    Name,
    /// One group per user.
    User,
    /// One group per run state.
    State,
}

impl GroupKey {
    /// Every grouping.
    pub const ALL: [Self; 4] = [Self::None, Self::Name, Self::User, Self::State];

    /// The name this grouping reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "no grouping",
            Self::Name => "group by name",
            Self::User => "group by user",
            Self::State => "group by status",
        }
    }

    /// The group a row belongs to.
    #[must_use]
    pub fn of(self, p: &ProcRow) -> Option<String> {
        match self {
            Self::None => None,
            Self::Name => Some(p.name.clone()),
            Self::User => Some(
                p.uid
                    .map_or_else(|| "unknown user".to_owned(), |u| format!("uid {u}")),
            ),
            Self::State => Some(p.state.clone()),
        }
    }
}

/// How tall a row is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Density {
    /// Twelve pixels: the most rows on screen.
    Compact,
    /// Sixteen: the default.
    Cozy,
    /// Twenty-two: room for a graph or two lines per row.
    Comfortable,
}

impl Density {
    /// Every density.
    pub const ALL: [Self; 3] = [Self::Compact, Self::Cozy, Self::Comfortable];

    /// The row height in reference pixels.
    #[must_use]
    pub const fn row_h(self) -> f32 {
        match self {
            Self::Compact => 12.0,
            Self::Cozy => 16.0,
            Self::Comfortable => 22.0,
        }
    }

    /// The name this density reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Compact => "compact rows",
            Self::Cozy => "cozy rows",
            Self::Comfortable => "comfortable rows",
        }
    }
}

/// How the CPU column reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuFormat {
    /// Percent of the machine, one decimal.
    Percent,
    /// Cumulative processor ticks.
    Ticks,
    /// A bar and the number, for scanning a column by eye.
    Bar,
}

/// How the memory columns read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemFormat {
    /// Mebibytes.
    Mib,
    /// Gibibytes.
    Gib,
    /// Share of installed memory.
    Share,
}

/// How durations read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeFormat {
    /// Seconds and a unit, e.g. `41s`, `12m`.
    Compact,
    /// `h:mm:ss`.
    Clock,
    /// Raw ticks.
    Ticks,
}

/// Where a numeric column's digits sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// Next to the heading, for text.
    Left,
    /// Against the right edge, for numbers: a column of numbers is read by its
    /// last digit.
    Right,
}

/// How wide a column is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidthStep {
    /// The column's natural width.
    Natural,
    /// Half of it.
    Narrow,
    /// One and a half times it.
    Wide,
}

impl WidthStep {
    /// Every width step.
    pub const ALL: [Self; 3] = [Self::Narrow, Self::Natural, Self::Wide];

    /// The multiplier this step applies.
    #[must_use]
    pub const fn factor(self) -> f32 {
        match self {
            Self::Narrow => 0.5,
            Self::Natural => 1.0,
            Self::Wide => 1.5,
        }
    }

    /// The name this step reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Narrow => "narrow",
            Self::Natural => "natural width",
            Self::Wide => "wide",
        }
    }
}

/// A resource a threshold can watch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    /// Machine CPU busy fraction.
    Cpu,
    /// Memory used.
    Memory,
    /// Swap used.
    Swap,
    /// Load average.
    Load,
    /// A process's share of the CPU.
    ProcessCpu,
    /// A process's resident set.
    ProcessMemory,
    /// The shell's own frame time.
    FrameTime,
    /// The daemon's own footprint.
    DaemonMemory,
}

impl Metric {
    /// Every watched metric.
    pub const ALL: [Self; 8] = [
        Self::Cpu,
        Self::Memory,
        Self::Swap,
        Self::Load,
        Self::ProcessCpu,
        Self::ProcessMemory,
        Self::FrameTime,
        Self::DaemonMemory,
    ];

    /// The name this metric reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "memory",
            Self::Swap => "swap",
            Self::Load => "load average",
            Self::ProcessCpu => "a process's CPU",
            Self::ProcessMemory => "a process's memory",
            Self::FrameTime => "frame time",
            Self::DaemonMemory => "the daemon's memory",
        }
    }

    /// What the metric means, for the "what is this" affordance.
    #[must_use]
    pub const fn help(self) -> &'static str {
        match self {
            Self::Cpu => "The share of all cores that was busy over the last sample.",
            Self::Memory => "Installed memory that is in use, with cache counted as in use.",
            Self::Swap => "Swap in use. Any swap on a healthy desktop is worth looking at.",
            Self::Load => {
                "Processes waiting or running, averaged over a minute. Compare it with \
                           the core count."
            }
            Self::ProcessCpu => "A single process using this much of the machine is worth naming.",
            Self::ProcessMemory => "A single process holding this much memory is worth naming.",
            Self::FrameTime => {
                "How long one frame took to draw. Over 16.67 ms a 60 Hz animation cannot keep up."
            }
            Self::DaemonMemory => {
                "What the system daemon itself costs. It reports everyone \
                                   else's footprint, so it has to report its own."
            }
        }
    }

    /// A band of a metric that is worth noticing.
    #[must_use]
    pub fn band(self, level: Level) -> Threshold {
        use Level::{Critical, Warn};
        match (self, level) {
            (Self::Cpu, Warn) => Threshold::new(70.0, "%"),
            (Self::Cpu, Critical) => Threshold::new(90.0, "%"),
            (Self::Memory, Warn) => Threshold::new(80.0, "%"),
            (Self::Memory, Critical) => Threshold::new(93.0, "%"),
            (Self::Swap, Warn) => Threshold::new(1.0, "MB"),
            (Self::Swap, Critical) => Threshold::new(512.0, "MB"),
            (Self::Load, Warn) => Threshold::new(4.0, ""),
            (Self::Load, Critical) => Threshold::new(8.0, ""),
            (Self::ProcessCpu, Warn) => Threshold::new(25.0, "%"),
            (Self::ProcessCpu, Critical) => Threshold::new(50.0, "%"),
            (Self::ProcessMemory, Warn) => Threshold::new(512.0, "MB"),
            (Self::ProcessMemory, Critical) => Threshold::new(2048.0, "MB"),
            (Self::FrameTime, Warn) => Threshold::new(16.7, "ms"),
            (Self::FrameTime, Critical) => Threshold::new(50.0, "ms"),
            (Self::DaemonMemory, Warn) => Threshold::new(32.0, "MB"),
            (Self::DaemonMemory, Critical) => Threshold::new(128.0, "MB"),
        }
    }
}

/// Which band of a metric a threshold is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// Worth a quiet mark.
    Warn,
    /// Worth saying out loud.
    Critical,
}

/// A value a metric is compared against.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Threshold {
    /// The value, in `unit`.
    pub value: f32,
    /// The unit it is in.
    pub unit: &'static str,
    /// Whether the mark is on at all.
    pub on: bool,
}

impl Threshold {
    /// A threshold that is off.
    #[must_use]
    pub const fn off() -> Self {
        Self {
            value: 0.0,
            unit: "",
            on: false,
        }
    }

    /// A threshold that is on.
    #[must_use]
    pub const fn new(value: f32, unit: &'static str) -> Self {
        Self {
            value,
            unit,
            on: true,
        }
    }

    /// Whether `v` crosses it.
    #[must_use]
    pub fn crossed(&self, v: f32) -> bool {
        self.on && v >= self.value
    }
}

/// Which users' processes are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserScope {
    /// Everyone's.
    All,
    /// The session's own user only.
    Own,
    /// System accounts only, which is where services live.
    System,
}

impl UserScope {
    /// Every scope.
    pub const ALL: [Self; 3] = [Self::All, Self::Own, Self::System];

    /// The name this scope reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "all users",
            Self::Own => "my processes",
            Self::System => "system accounts",
        }
    }
}

/// How a crossing is announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Delivery {
    /// A mark in the window and nothing else.
    Silent,
    /// A mark and a line in the status bar.
    Status,
    /// A mark and a notification.
    Notify,
}

impl Delivery {
    /// Every delivery.
    pub const ALL: [Self; 3] = [Self::Silent, Self::Status, Self::Notify];

    /// The name this delivery reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "mark only",
            Self::Status => "status bar",
            Self::Notify => "notification",
        }
    }
}

/// What an export produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Export {
    /// Tab-separated, which a spreadsheet opens.
    Tsv,
    /// CSV, with the quoting that a comma in a command line needs.
    Csv,
    /// JSON, for a script.
    Json,
}

impl Export {
    /// Every format.
    pub const ALL: [Self; 3] = [Self::Tsv, Self::Csv, Self::Json];

    /// The name this format reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tsv => "tab-separated",
            Self::Csv => "CSV",
            Self::Json => "JSON",
        }
    }
}

/// The tones a crossing can be drawn in.
///
/// Four, and not the whole palette: a mark means one of four things -- ordinary,
/// worth noticing, wrong, or good -- and a table where every row can be any
/// colour is a table nobody reads.
pub const TONES: [Tone; 4] = [Tone::Neutral, Tone::Warning, Tone::Bad, Tone::Good];

/// The name a tone is chosen by.
#[must_use]
pub const fn tone_label(t: Tone) -> &'static str {
    match t {
        Tone::Neutral => "quiet",
        Tone::Warning => "amber",
        Tone::Bad => "red",
        Tone::Good => "green",
        Tone::Accent => "accent",
    }
}

/// Everything a person can change about the table.
#[derive(Debug, Clone, PartialEq)]
pub struct Prefs {
    /// Which columns are shown, in table order.
    pub columns: Vec<Column>,
    /// Each column's width step.
    pub widths: Vec<(Column, WidthStep)>,
    /// Each column's alignment.
    pub aligns: Vec<(Column, Align)>,
    /// Primary sort.
    pub sort: (SortKey, bool),
    /// Tie-breaker, applied when the primary key compares equal.
    pub secondary: Option<SortKey>,
    /// Grouping.
    pub group: GroupKey,
    /// Row height.
    pub density: Density,
    /// CPU formatting.
    pub cpu_format: CpuFormat,
    /// Memory formatting.
    pub mem_format: MemFormat,
    /// Duration formatting.
    pub time_format: TimeFormat,
    /// Warn band per metric.
    pub warn: Vec<(Metric, Threshold)>,
    /// Critical band per metric.
    pub critical: Vec<(Metric, Threshold)>,
    /// A state filter, empty for all.
    pub filter_state: String,
    /// The search text.
    pub filter_text: String,
    /// Hide rows above this CPU share, in percent (`0.0` shows all).
    pub min_cpu_pct: f32,
    /// Hide rows with less memory than this, in mebibytes (`0` shows all).
    pub min_mem_mb: u64,
    /// Show kernel threads, which have no command line.
    pub show_kernel_threads: bool,
    /// Show processes owned by system users.
    pub show_system_users: bool,
    /// Colour a row whose process crosses a threshold.
    pub colour_crossings: bool,
    /// Keep the heading visible while the rows scroll.
    pub sticky_header: bool,
    /// Alternate the row background.
    pub zebra: bool,
    /// How often the daemon is asked, in milliseconds.
    pub refresh_ms: u64,
    /// Ask before ending a process.
    pub confirm_end_task: bool,
    /// The colour a row is tinted when a process crosses a metric's warn band.
    pub warn_tone: Vec<(Metric, Tone)>,
    /// The colour for the critical band.
    pub critical_tone: Vec<(Metric, Tone)>,
    /// Whether a column's heading is drawn shortened, for a narrow window.
    pub short_headers: Vec<(Column, bool)>,
    /// How each band is announced.
    pub delivery: Vec<(Level, Delivery)>,
    /// Which users' processes are shown.
    pub user_scope: UserScope,
    /// Whether sampling is paused. A paused table shows the last sample it had,
    /// which is what a person wants while they read it.
    pub paused: bool,
    /// Whether a column sorts descending the first time its heading is clicked.
    pub first_click_desc: Vec<(Column, bool)>,
}

impl Default for Prefs {
    fn default() -> Self {
        let visible: Vec<Column> = [
            Column::Name,
            Column::Pid,
            Column::Cpu,
            Column::Memory,
            Column::Threads,
            Column::State,
        ]
        .to_vec();
        Self {
            columns: visible,
            widths: Column::ALL
                .iter()
                .map(|c| (*c, WidthStep::Natural))
                .collect(),
            aligns: Column::ALL
                .iter()
                .map(|c| {
                    let numeric = matches!(
                        c,
                        Column::Pid
                            | Column::Parent
                            | Column::Cpu
                            | Column::CpuTime
                            | Column::Memory
                            | Column::MemShare
                            | Column::Threads
                    );
                    (*c, if numeric { Align::Right } else { Align::Left })
                })
                .collect(),
            sort: (SortKey::Cpu, false),
            secondary: Some(SortKey::Name),
            group: GroupKey::None,
            density: Density::Cozy,
            cpu_format: CpuFormat::Percent,
            mem_format: MemFormat::Mib,
            time_format: TimeFormat::Compact,
            warn: Metric::ALL
                .iter()
                .map(|m| (*m, m.band(Level::Warn)))
                .collect(),
            critical: Metric::ALL
                .iter()
                .map(|m| (*m, m.band(Level::Critical)))
                .collect(),
            filter_state: String::new(),
            filter_text: String::new(),
            min_cpu_pct: 0.0,
            min_mem_mb: 0,
            show_kernel_threads: false,
            show_system_users: true,
            colour_crossings: true,
            sticky_header: true,
            zebra: false,
            refresh_ms: 1_000,
            confirm_end_task: true,
            warn_tone: Metric::ALL.iter().map(|m| (*m, Tone::Warning)).collect(),
            critical_tone: Metric::ALL.iter().map(|m| (*m, Tone::Bad)).collect(),
            short_headers: Column::ALL.iter().map(|c| (*c, false)).collect(),
            delivery: vec![
                (Level::Warn, Delivery::Silent),
                (Level::Critical, Delivery::Notify),
            ],
            user_scope: UserScope::All,
            paused: false,
            first_click_desc: Column::ALL
                .iter()
                .map(|c| {
                    let numeric = matches!(
                        c,
                        Column::Cpu
                            | Column::CpuTime
                            | Column::Memory
                            | Column::MemShare
                            | Column::Threads
                    );
                    (*c, numeric)
                })
                .collect(),
        }
    }
}

impl Prefs {
    /// The width step of a column.
    #[must_use]
    pub fn width_of(&self, c: Column) -> WidthStep {
        self.widths
            .iter()
            .find(|(k, _)| *k == c)
            .map_or(WidthStep::Natural, |(_, v)| *v)
    }

    /// The alignment of a column.
    #[must_use]
    pub fn align_of(&self, c: Column) -> Align {
        self.aligns
            .iter()
            .find(|(k, _)| *k == c)
            .map_or(Align::Left, |(_, v)| *v)
    }

    /// A metric's warn band.
    #[must_use]
    pub fn warn_of(&self, m: Metric) -> Threshold {
        self.warn
            .iter()
            .find(|(k, _)| *k == m)
            .map_or_else(Threshold::off, |(_, t)| *t)
    }

    /// A metric's critical band.
    #[must_use]
    pub fn critical_of(&self, m: Metric) -> Threshold {
        self.critical
            .iter()
            .find(|(k, _)| *k == m)
            .map_or_else(Threshold::off, |(_, t)| *t)
    }

    /// The tone a metric's crossing is drawn in.
    #[must_use]
    pub fn tone_of(&self, m: Metric, level: Level) -> Tone {
        let table = match level {
            Level::Warn => &self.warn_tone,
            Level::Critical => &self.critical_tone,
        };
        table
            .iter()
            .find(|(k, _)| *k == m)
            .map_or(Tone::Neutral, |(_, t)| *t)
    }

    /// How a band is announced.
    #[must_use]
    pub fn delivery_of(&self, level: Level) -> Delivery {
        self.delivery
            .iter()
            .find(|(k, _)| *k == level)
            .map_or(Delivery::Silent, |(_, d)| *d)
    }

    /// Whether a column's heading is shortened.
    #[must_use]
    pub fn short_header(&self, c: Column) -> bool {
        self.short_headers
            .iter()
            .find(|(k, _)| *k == c)
            .is_some_and(|(_, v)| *v)
    }

    /// Whether a column sorts descending on its first click.
    #[must_use]
    pub fn first_click_desc(&self, c: Column) -> bool {
        self.first_click_desc
            .iter()
            .find(|(k, _)| *k == c)
            .is_some_and(|(_, v)| *v)
    }

    /// The total width of the shown columns at this scale, in pixels.
    #[must_use]
    pub fn table_width(&self, scale: f32) -> f32 {
        self.columns
            .iter()
            .map(|c| c.width() * self.width_of(*c).factor() * scale)
            .sum()
    }

    /// Toggle a column's visibility.
    pub fn toggle_column(&mut self, c: Column) {
        if let Some(i) = self.columns.iter().position(|x| *x == c) {
            self.columns.remove(i);
        } else {
            // Inserted at its natural place rather than at the end: a column
            // that reappears at the far right is a column nobody finds again.
            let at = Column::ALL
                .iter()
                .position(|x| *x == c)
                .unwrap_or(self.columns.len());
            let at = at.min(self.columns.len());
            self.columns.insert(at, c);
        }
    }

    /// Sort by a column, flipping the direction when it is already the key.
    pub fn sort_by(&mut self, key: SortKey, ascending: bool) {
        if self.sort.0 == key && self.sort.1 != ascending {
            self.sort.1 = ascending;
        } else {
            self.sort = (key, ascending);
        }
    }
}

/// One entry of the feature registry.
#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    /// A stable id, for the launcher and for tests.
    pub id: String,
    /// What the entry is called.
    pub label: String,
    /// What it does, in one sentence.
    pub help: String,
    /// Which group it belongs to.
    pub category: Category,
    /// The change it makes.
    pub change: Change,
}

/// A group of features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Which columns are shown and how wide they are.
    Columns,
    /// Sort keys and directions.
    Sorting,
    /// Grouping and row shape.
    Layout,
    /// Units and formats.
    Formats,
    /// What counts as worth noticing.
    Thresholds,
    /// Which rows are shown at all.
    Filters,
    /// Actions on the selected process.
    Actions,
    /// How the window behaves.
    Behaviour,
}

impl Category {
    /// Every category, in the order the launcher lists them.
    pub const ALL: [Self; 8] = [
        Self::Columns,
        Self::Sorting,
        Self::Layout,
        Self::Formats,
        Self::Thresholds,
        Self::Filters,
        Self::Actions,
        Self::Behaviour,
    ];

    /// The name this category reports under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Columns => "Columns",
            Self::Sorting => "Sorting",
            Self::Layout => "Layout",
            Self::Formats => "Formats",
            Self::Thresholds => "Thresholds",
            Self::Filters => "Filters",
            Self::Actions => "Actions",
            Self::Behaviour => "Behaviour",
        }
    }
}

/// What a feature does when it is applied.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// Show or hide a column.
    ToggleColumn(Column),
    /// Set a column's width step.
    ColumnWidth(Column, WidthStep),
    /// Set a column's alignment.
    ColumnAlign(Column, Align),
    /// Sort by a key in a direction.
    Sort(SortKey, bool),
    /// Set the tie-breaker.
    SecondarySort(SortKey),
    /// Set the grouping.
    Group(GroupKey),
    /// Set the row height.
    Density(Density),
    /// Set the CPU format.
    CpuFormat(CpuFormat),
    /// Set the memory format.
    MemFormat(MemFormat),
    /// Set the duration format.
    TimeFormat(TimeFormat),
    /// Turn a metric's band on, at its default value.
    Threshold(Metric, Level, bool),
    /// Show only rows in this state (empty for all).
    FilterState(&'static str),
    /// Hide rows below a share of the CPU.
    MinCpu(f32),
    /// Hide rows below a size in memory.
    MinMemory(u64),
    /// Show or hide kernel threads.
    ShowKernelThreads(bool),
    /// Show or hide system-owned processes.
    ShowSystemUsers(bool),
    /// Colour rows that cross a threshold.
    ColourCrossings(bool),
    /// Keep the heading visible while scrolling.
    StickyHeader(bool),
    /// Alternate row backgrounds.
    Zebra(bool),
    /// How often to ask the daemon.
    Refresh(u64),
    /// Ask before ending a process.
    ConfirmEndTask(bool),
    /// End the selected process.
    EndTask,
    /// End the selected process and everything it started.
    EndTree,
    /// End every process in the selected group.
    EndGroup,
    /// Copy the selected row as text.
    CopyRow,
    /// Copy the whole table as text.
    CopyTable,
    /// Copy one column's values, of the selection or of every shown row.
    CopyColumn(Column, bool),
    /// Copy the table in an exchangeable format.
    Export(Export),
    /// Sample once, now.
    RefreshNow,
    /// Stop sampling.
    Paused(bool),
    /// Draw a column's heading shortened.
    ShortHeader(Column, bool),
    /// Colour of a metric's warn band.
    WarnTone(Metric, Tone),
    /// Colour of a metric's critical band.
    CriticalTone(Metric, Tone),
    /// How a band is announced.
    Delivery(Level, Delivery),
    /// Which users' processes are shown.
    UserScope(UserScope),
    /// Whether a column sorts descending on its first click.
    FirstClickDesc(Column, bool),
    /// Put one category back to its defaults.
    Reset(Category),
}

/// Every feature the Task Manager has, as data.
///
/// Generated from the tables the drawing code reads, so the count cannot drift
/// away from what the table can actually do: adding a column to [`Column`] adds
/// its five entries here, and adding a metric to [`Metric`] adds three. A test
/// asserts this list is larger than the target and that every entry changes
/// something.
#[must_use]
pub fn features() -> Vec<Feature> {
    let mut out: Vec<Feature> = Vec::new();
    let mut push = |category, id: String, label: String, help: String, change| {
        out.push(Feature {
            id,
            label,
            help,
            category,
            change,
        });
    };

    for c in Column::ALL {
        let col_slug = slug(c.label());
        push(
            Category::Columns,
            format!("columns.{col_slug}.show"),
            format!("Show {} column", c.label()),
            c.help().to_owned(),
            Change::ToggleColumn(c),
        );
        for w in WidthStep::ALL {
            push(
                Category::Columns,
                format!("columns.{col_slug}.width.{}", slug(w.label())),
                format!("{} column: {} width", c.label(), w.label()),
                format!(
                    "Draw the {} column at {} of its natural width.",
                    c.label(),
                    w.label()
                ),
                Change::ColumnWidth(c, w),
            );
        }
        for (a, name) in [(Align::Left, "left"), (Align::Right, "right")] {
            push(
                Category::Columns,
                format!("columns.{col_slug}.align.{name}"),
                format!("{} column: align {name}", c.label()),
                if a == Align::Right {
                    "Align the digits against the right edge, so a column of numbers reads by its \
                     last digit."
                        .to_owned()
                } else {
                    "Align the text against the left edge, next to its heading.".to_owned()
                },
                Change::ColumnAlign(c, a),
            );
        }
    }

    for k in SortKey::ALL {
        for (asc, word) in [(true, "ascending"), (false, "descending")] {
            push(
                Category::Sorting,
                format!("sort.{}.{word}", slug(k.label())),
                format!("Sort by {} ({word})", k.label()),
                format!("Order the table by {}, {word}.", k.label()),
                Change::Sort(k, asc),
            );
        }
        push(
            Category::Sorting,
            format!("sort.secondary.{}", slug(k.label())),
            format!("Tie-break by {}", k.label()),
            format!(
                "When two rows compare equal on the sort key, order them by {}.",
                k.label()
            ),
            Change::SecondarySort(k),
        );
    }

    for g in GroupKey::ALL {
        push(
            Category::Layout,
            format!("layout.group.{}", slug(g.label())),
            format!("Group rows: {}", g.label()),
            match g {
                GroupKey::None => "One flat list, ordered by the sort key.".to_owned(),
                GroupKey::Name => {
                    "One block per program, heaviest first, so twenty instances of the same \
                     program read as one row that expands."
                        .to_owned()
                }
                GroupKey::User => "One block per user, so a program running as a service is \
                                   not mistaken for the same program running as you."
                    .to_owned(),
                GroupKey::State => "One block per run state, which is how a pile of sleeping \
                                    processes is told from a pile of running ones."
                    .to_owned(),
            },
            Change::Group(g),
        );
    }

    for d in Density::ALL {
        push(
            Category::Layout,
            format!("layout.density.{}", slug(d.label())),
            d.label().to_owned(),
            format!("Draw rows {} pixels tall.", d.row_h()),
            Change::Density(d),
        );
    }

    for (on, word) in [(true, "on"), (false, "off")] {
        push(
            Category::Layout,
            format!("layout.colour_crossings.{word}"),
            format!("Colour rows over a threshold: {word}"),
            "Tint a row whose process crosses its warn or critical band, so the table answers \
             \"what is heavy\" without reading every number."
                .to_owned(),
            Change::ColourCrossings(on),
        );
        push(
            Category::Layout,
            format!("layout.sticky_header.{word}"),
            format!("Keep the heading visible: {word}"),
            "Keep the column headings on screen while the rows scroll.".to_owned(),
            Change::StickyHeader(on),
        );
        push(
            Category::Layout,
            format!("layout.zebra.{word}"),
            format!("Alternate row shading: {word}"),
            "Tint every second row, which is what makes a long table readable across.".to_owned(),
            Change::Zebra(on),
        );
    }

    for (f, word) in [
        (CpuFormat::Percent, "percent"),
        (CpuFormat::Ticks, "ticks"),
        (CpuFormat::Bar, "bar and number"),
    ] {
        push(
            Category::Formats,
            format!("formats.cpu.{word}"),
            format!("CPU as {word}"),
            match f {
                CpuFormat::Percent => "A share of the whole machine, one decimal place.".to_owned(),
                CpuFormat::Ticks => {
                    "The counter the kernel keeps, which is exact and unreadable.".to_owned()
                }
                CpuFormat::Bar => {
                    "A bar beside the number, for scanning the column by eye.".to_owned()
                }
            },
            Change::CpuFormat(f),
        );
    }
    for (f, word) in [
        (MemFormat::Mib, "mebibytes"),
        (MemFormat::Gib, "gibibytes"),
        (MemFormat::Share, "share of installed"),
    ] {
        push(
            Category::Formats,
            format!("formats.memory.{word}"),
            format!("Memory in {word}"),
            "Resident set is what is in RAM now, not what the process has reserved.".to_owned(),
            Change::MemFormat(f),
        );
    }
    for (f, word) in [
        (TimeFormat::Compact, "compact"),
        (TimeFormat::Clock, "clock"),
        (TimeFormat::Ticks, "ticks"),
    ] {
        push(
            Category::Formats,
            format!("formats.time.{word}"),
            format!("Durations as {word}"),
            "Processor time is cumulative, not a rate: it only ever increases.".to_owned(),
            Change::TimeFormat(f),
        );
    }

    for m in Metric::ALL {
        for (level, word) in [(Level::Warn, "warn"), (Level::Critical, "critical")] {
            for on in [true, false] {
                let band = m.band(level);
                push(
                    Category::Thresholds,
                    format!(
                        "thresholds.{}.{word}.{}",
                        slug(m.label()),
                        if on { "on" } else { "off" }
                    ),
                    format!(
                        "{} {} band: {}",
                        m.label(),
                        word,
                        if on { "on" } else { "off" }
                    ),
                    if on {
                        format!(
                            "{} Mark it when it reaches {} {}.",
                            m.help(),
                            band.value,
                            band.unit
                        )
                    } else {
                        format!("{} Stop marking it at {}.", m.help(), word)
                    },
                    Change::Threshold(m, level, on),
                );
            }
        }
    }

    for state in [
        "",
        "running",
        "sleeping",
        "uninterruptible",
        "zombie",
        "stopped",
        "idle",
    ] {
        let word = if state.is_empty() {
            "all states"
        } else {
            state
        };
        push(
            Category::Filters,
            format!("filters.state.{}", slug(word)),
            format!("Show {word}"),
            "The kernel's run state for a process, spelled out.".to_owned(),
            Change::FilterState(state),
        );
    }
    for (pct, word) in [
        (0.0f32, "any"),
        (1.0, "over 1%"),
        (5.0, "over 5%"),
        (25.0, "over 25%"),
    ] {
        push(
            Category::Filters,
            format!("filters.cpu.{}", slug(word)),
            format!("CPU: {word}"),
            "Hides rows below this share of the machine, so a table of 300 sleeping processes \
             does not bury the one that is running."
                .to_owned(),
            Change::MinCpu(pct),
        );
    }
    for (mb, word) in [
        (0u64, "any"),
        (16, "over 16 MB"),
        (128, "over 128 MB"),
        (512, "over 512 MB"),
    ] {
        push(
            Category::Filters,
            format!("filters.memory.{}", slug(word)),
            format!("Memory: {word}"),
            "Hides rows below this resident set.".to_owned(),
            Change::MinMemory(mb),
        );
    }
    for (on, word) in [(true, "show"), (false, "hide")] {
        push(
            Category::Filters,
            format!("filters.kernel_threads.{word}"),
            format!("{word} kernel threads"),
            "Kernel threads have no command line and cannot be ended. They are real work, but \
             they are not yours."
                .to_owned(),
            Change::ShowKernelThreads(on),
        );
        push(
            Category::Filters,
            format!("filters.system_users.{word}"),
            format!("{word} system processes"),
            "Processes owned by uid 0 and other system users. Hiding them is how you read a \
             table of your own work."
                .to_owned(),
            Change::ShowSystemUsers(on),
        );
    }

    for (ms, word) in [
        (250u64, "4 times a second"),
        (500, "twice a second"),
        (1_000, "once a second"),
        (2_000, "every 2 seconds"),
        (5_000, "every 5 seconds"),
    ] {
        push(
            Category::Behaviour,
            format!("behaviour.refresh.{ms}"),
            format!("Refresh {word}"),
            "How often the daemon is asked for a new sample. A faster table costs processor time \
             to look at."
                .to_owned(),
            Change::Refresh(ms),
        );
    }
    for (on, word) in [(true, "on"), (false, "off")] {
        push(
            Category::Behaviour,
            format!("behaviour.confirm_end_task.{word}"),
            format!("Confirm before ending a task: {word}"),
            "Ending the wrong process loses the work in it, and the table only shows a name."
                .to_owned(),
            Change::ConfirmEndTask(on),
        );
    }

    // A heading has to survive a narrow window, and a shortened heading is how
    // the table keeps its columns instead of hiding them. A column whose short
    // heading is the heading it already draws is left out: offering to shorten
    // "Name" to "Name" is not a choice, it is a promise the window cannot keep.
    for c in Column::ALL {
        let col_slug = slug(c.label());
        for (on, word) in [(true, "short"), (false, "full")] {
            if short_label(c) == c.label() {
                continue;
            }
            push(
                Category::Columns,
                format!("columns.{col_slug}.header.{word}"),
                format!("{} heading: {word}", c.label()),
                format!(
                    "Draw the {} column's heading as \"{}\" when the window is too narrow for \
                     the whole word.",
                    c.label(),
                    if on { short_label(c) } else { c.label() }
                ),
                Change::ShortHeader(c, on),
            );
        }
        for (desc, word) in [(true, "heaviest first"), (false, "lightest first")] {
            push(
                Category::Sorting,
                format!("sort.first_click.{}.{word}", col_slug),
                format!("Clicking {} sorts {word}", c.label()),
                format!(
                    "The direction {} gets on its first click. Numbers are usually read \
                     heaviest-first; names are read A to Z.",
                    c.label()
                ),
                Change::FirstClickDesc(c, desc),
            );
        }
    }

    for m in Metric::ALL {
        let metric_slug = slug(m.label());
        for t in TONES {
            push(
                Category::Thresholds,
                format!("thresholds.{metric_slug}.warn.tone.{}", slug(tone_label(t))),
                format!("{} warn colour: {}", m.label(), tone_label(t)),
                format!(
                    "Tint a row whose process crosses {}'s warn band {}. {}",
                    m.label(),
                    tone_label(t),
                    m.help()
                ),
                Change::WarnTone(m, t),
            );
            push(
                Category::Thresholds,
                format!(
                    "thresholds.{metric_slug}.critical.tone.{}",
                    slug(tone_label(t))
                ),
                format!("{} critical colour: {}", m.label(), tone_label(t)),
                format!(
                    "Tint a row whose process crosses {}'s critical band {}.",
                    m.label(),
                    tone_label(t)
                ),
                Change::CriticalTone(m, t),
            );
        }
    }

    for level in [Level::Warn, Level::Critical] {
        let word = match level {
            Level::Warn => "warn",
            Level::Critical => "critical",
        };
        for d in Delivery::ALL {
            push(
                Category::Behaviour,
                format!("behaviour.delivery.{word}.{}", slug(d.label())),
                format!("Announce {word} bands: {}", d.label()),
                match d {
                    Delivery::Silent => {
                        "Mark the row and say nothing else. A warning that interrupts is a \
                         warning people turn off."
                            .to_owned()
                    }
                    Delivery::Status => "Add a line to the status bar, where the rest of the \
                                          machine's state is."
                        .to_owned(),
                    Delivery::Notify => {
                        "Post a notification, for a crossing worth interrupting for.".to_owned()
                    }
                },
                Change::Delivery(level, d),
            );
        }
    }

    for s in UserScope::ALL {
        push(
            Category::Filters,
            format!("filters.users.{}", slug(s.label())),
            format!("Show {}", s.label()),
            "Processes run as a user, not as the person who started them; a service is not \
             yours to read in a table of your own work."
                .to_owned(),
            Change::UserScope(s),
        );
    }

    for (on, word) in [(true, "pause"), (false, "resume")] {
        push(
            Category::Behaviour,
            format!("behaviour.sampling.{word}"),
            format!("{} sampling", word),
            "A paused table keeps the sample on screen, which is what a person wants while they \
             read a row before ending it."
                .to_owned(),
            Change::Paused(on),
        );
    }
    push(
        Category::Behaviour,
        "behaviour.sampling.now".to_owned(),
        "Sample now".to_owned(),
        "Ask the daemon for one sample immediately rather than waiting for the next tick."
            .to_owned(),
        Change::RefreshNow,
    );

    for e in Export::ALL {
        push(
            Category::Actions,
            format!("actions.export.{}", slug(e.label())),
            format!("Copy the table as {}", e.label()),
            "A table nobody can get out of the window is a table nobody can attach to a bug \
             report."
                .to_owned(),
            Change::Export(e),
        );
    }

    for c in Column::ALL {
        let col_slug = slug(c.label());
        for (all, word) in [(false, "the selected row"), (true, "every shown row")] {
            push(
                Category::Actions,
                format!(
                    "actions.copy_column.{col_slug}.{}",
                    if all { "all" } else { "row" }
                ),
                format!("Copy {} for {word}", c.label()),
                format!("Puts {} for {} on the clipboard as text.", c.label(), word),
                Change::CopyColumn(c, all),
            );
        }
    }

    for cat in Category::ALL {
        push(
            Category::Behaviour,
            format!("behaviour.reset.{}", slug(cat.label())),
            format!("Reset {} to defaults", cat.label()),
            format!(
                "Put every {} setting back to what it was on a new machine. Other categories \
                 are left alone.",
                cat.label().to_lowercase()
            ),
            Change::Reset(cat),
        );
    }

    for (change, id, label, help) in [
        (
            Change::EndTask,
            "actions.end_task",
            "End task",
            "Ask the selected process to stop.",
        ),
        (
            Change::EndTree,
            "actions.end_tree",
            "End process tree",
            "Stop the selected process and everything it started.",
        ),
        (
            Change::EndGroup,
            "actions.end_group",
            "End every process in the group",
            "Stops every row under the selected group heading.",
        ),
        (
            Change::CopyRow,
            "actions.copy_row",
            "Copy the selected row",
            "Puts the row's cells on the clipboard as tab-separated text.",
        ),
        (
            Change::CopyTable,
            "actions.copy_table",
            "Copy the whole table",
            "Puts every shown row and column on the clipboard as tab-separated text.",
        ),
    ] {
        push(
            Category::Actions,
            id.to_owned(),
            label.to_owned(),
            help.to_owned(),
            change,
        );
    }

    out
}

/// The heading a column uses when it has to be short.
#[must_use]
pub const fn short_label(c: Column) -> &'static str {
    match c {
        Column::Name => "Name",
        Column::Pid => "PID",
        Column::Parent => "PPID",
        Column::State => "State",
        Column::User => "User",
        Column::Cpu => "CPU",
        Column::CpuTime => "CPU t",
        Column::Memory => "Mem",
        Column::MemShare => "%Mem",
        Column::Threads => "Thr",
        Column::CommandLine => "Command",
    }
}

/// The heading a column draws under these preferences.
#[must_use]
pub fn heading(c: Column, prefs: &Prefs) -> &'static str {
    if prefs.short_header(c) {
        short_label(c)
    } else {
        c.label()
    }
}

/// A label turned into an id fragment.
#[must_use]
fn slug(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Apply one feature to the preferences. Returns whether it changed anything.
///
/// Actions return `false`: they act on a selection, not on the table's shape,
/// and claiming they changed a preference would be a lie the caller then reports.
pub fn apply(prefs: &mut Prefs, change: &Change) -> bool {
    let before = prefs.clone();
    match change {
        Change::ToggleColumn(c) => prefs.toggle_column(*c),
        Change::ColumnWidth(c, w) => {
            set(&mut prefs.widths, *c, *w);
        }
        Change::ColumnAlign(c, a) => {
            set(&mut prefs.aligns, *c, *a);
        }
        Change::Sort(k, asc) => prefs.sort_by(*k, *asc),
        Change::SecondarySort(k) => prefs.secondary = Some(*k),
        Change::Group(g) => prefs.group = *g,
        Change::Density(d) => prefs.density = *d,
        Change::CpuFormat(f) => prefs.cpu_format = *f,
        Change::MemFormat(f) => prefs.mem_format = *f,
        Change::TimeFormat(f) => prefs.time_format = *f,
        Change::Threshold(m, level, on) => {
            let band = if *on {
                m.band(*level)
            } else {
                Threshold::off()
            };
            match level {
                Level::Warn => {
                    set(&mut prefs.warn, *m, band);
                }
                Level::Critical => {
                    set(&mut prefs.critical, *m, band);
                }
            }
        }
        Change::FilterState(s) => prefs.filter_state = (*s).to_owned(),
        Change::MinCpu(pct) => prefs.min_cpu_pct = *pct,
        Change::MinMemory(mb) => prefs.min_mem_mb = *mb,
        Change::ShowKernelThreads(on) => prefs.show_kernel_threads = *on,
        Change::ShowSystemUsers(on) => prefs.show_system_users = *on,
        Change::ColourCrossings(on) => prefs.colour_crossings = *on,
        Change::StickyHeader(on) => prefs.sticky_header = *on,
        Change::Zebra(on) => prefs.zebra = *on,
        Change::Refresh(ms) => prefs.refresh_ms = *ms,
        Change::ConfirmEndTask(on) => prefs.confirm_end_task = *on,
        Change::Paused(on) => prefs.paused = *on,
        Change::ShortHeader(c, on) => {
            set(&mut prefs.short_headers, *c, *on);
        }
        Change::WarnTone(m, t) => {
            set(&mut prefs.warn_tone, *m, *t);
        }
        Change::CriticalTone(m, t) => {
            set(&mut prefs.critical_tone, *m, *t);
        }
        Change::Delivery(level, d) => {
            set(&mut prefs.delivery, *level, *d);
        }
        Change::UserScope(s) => prefs.user_scope = *s,
        Change::FirstClickDesc(c, on) => {
            set(&mut prefs.first_click_desc, *c, *on);
        }
        Change::Reset(category) => {
            reset_category(prefs, *category);
        }
        // Actions need a selection; they are not settings, and reporting that
        // they "changed" a preference would be a lie the caller then shows.
        Change::EndTask
        | Change::EndTree
        | Change::EndGroup
        | Change::CopyRow
        | Change::CopyTable
        | Change::CopyColumn(..)
        | Change::Export(..)
        | Change::RefreshNow => return false,
    }
    *prefs != before
}

/// Put one category back to its defaults. Returns whether anything moved.
///
/// A category reset is not "all settings": reverting the columns must not also
/// revert the thresholds, because those were set for a different reason and a
/// reset that undoes unrelated work is a reset nobody uses.
pub fn reset_category(prefs: &mut Prefs, category: Category) -> bool {
    let defaults = Prefs::default();
    let before = prefs.clone();
    match category {
        Category::Columns => {
            prefs.columns.clone_from(&defaults.columns);
            prefs.widths.clone_from(&defaults.widths);
            prefs.aligns.clone_from(&defaults.aligns);
            prefs.short_headers.clone_from(&defaults.short_headers);
            prefs
                .first_click_desc
                .clone_from(&defaults.first_click_desc);
        }
        Category::Sorting => prefs.sort = defaults.sort,
        Category::Layout => {
            prefs.group = defaults.group;
            prefs.density = defaults.density;
            prefs.zebra = defaults.zebra;
            prefs.sticky_header = defaults.sticky_header;
            prefs.colour_crossings = defaults.colour_crossings;
        }
        Category::Formats => {
            prefs.cpu_format = defaults.cpu_format;
            prefs.mem_format = defaults.mem_format;
            prefs.time_format = defaults.time_format;
        }
        Category::Thresholds => {
            prefs.warn.clone_from(&defaults.warn);
            prefs.critical.clone_from(&defaults.critical);
            prefs.warn_tone.clone_from(&defaults.warn_tone);
            prefs.critical_tone.clone_from(&defaults.critical_tone);
        }
        Category::Filters => {
            prefs.filter_state = defaults.filter_state;
            prefs.filter_text = defaults.filter_text;
            prefs.min_cpu_pct = defaults.min_cpu_pct;
            prefs.min_mem_mb = defaults.min_mem_mb;
            prefs.show_kernel_threads = defaults.show_kernel_threads;
            prefs.show_system_users = defaults.show_system_users;
            prefs.user_scope = defaults.user_scope;
        }
        // Actions have nothing to put back: they are verbs, not settings.
        Category::Actions => return false,
        Category::Behaviour => {
            prefs.refresh_ms = defaults.refresh_ms;
            prefs.confirm_end_task = defaults.confirm_end_task;
            prefs.paused = defaults.paused;
            prefs.delivery.clone_from(&defaults.delivery);
        }
    }
    *prefs != before
}

/// Set a value in a `(key, value)` table, adding the key if it is missing.
fn set<K: PartialEq, V: PartialEq>(table: &mut Vec<(K, V)>, key: K, value: V) -> bool {
    match table.iter_mut().find(|(k, _)| *k == key) {
        Some(slot) => {
            if slot.1 == value {
                return false;
            }
            slot.1 = value;
            true
        }
        None => {
            table.push((key, value));
            false
        }
    }
}

/// One drawn row: a group heading, or a process.
#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    /// A group's heading, with how many rows are under it.
    Group {
        /// The group's name.
        name: String,
        /// How many rows it holds.
        count: usize,
        /// Whether it is drawn open.
        open: bool,
    },
    /// A process.
    Process {
        /// Its position in [`TaskManager::filtered`].
        index: usize,
    },
}

/// The machine-wide readings the Activity window draws above the process table.
///
/// Each one is a field the daemon reported or the shell measured: a reading that
/// is `None` is a reading nothing produced, and it is drawn as a dash rather
/// than as a zero, because a machine that did not say how much swap it uses is
/// not a machine using no swap.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Machine {
    /// Busy share of all logical CPUs over the last sample, 0.0 to 1.0.
    pub cpu: Option<f32>,
    /// Whether `cpu` is a live rate between two samples rather than the average
    /// since boot. The meter says which it is: a monitor that draws an average
    /// and calls it "now" teaches its user to distrust the number.
    pub cpu_live: bool,
    /// RAM in use, MiB.
    pub mem_used_mb: Option<u64>,
    /// Installed RAM, MiB.
    pub mem_total_mb: Option<u64>,
    /// RAM a new allocation could get, MiB.
    pub mem_available_mb: Option<u64>,
    /// Page cache and buffers, MiB.
    pub mem_cached_mb: Option<u64>,
    /// Swap in use, MiB.
    pub swap_used_mb: Option<u64>,
    /// One-minute load average.
    pub load1: Option<f32>,
    /// Logical CPUs online, so load can be read against it.
    pub cpus: Option<u32>,
    /// The system daemon's own resident set, MiB.
    pub sysd_rss_mb: Option<u64>,
    /// The last frame's draw time in milliseconds. The shell measures this; no
    /// part of the machine reports it.
    pub frame_ms: Option<f32>,
    /// Seconds since boot.
    pub uptime_secs: Option<f64>,
    /// Processes and threads the kernel is tracking.
    pub tasks: Option<u32>,
}

/// One machine-wide meter, as the Activity window draws it.
#[derive(Debug, Clone, PartialEq)]
pub struct Meter {
    /// Which resource it reads.
    pub metric: Metric,
    /// What it is called on screen.
    pub label: &'static str,
    /// The reading, already in its own unit. A dash when nothing measured it.
    pub text: String,
    /// How full the bar is, 0.0 to 1.0 of the meter's own scale.
    pub share: f32,
    /// The band the reading crossed, if it crossed one.
    pub level: Option<Level>,
    /// The colour it is drawn in: the crossed band's colour, or neutral when it
    /// crossed nothing (or when crossing colours are turned off).
    pub tone: Tone,
    /// The marks that are on, in words ("warn 70 %, critical 90 %"), so the
    /// window can say what a reading would have to reach to be marked.
    pub marks: String,
    /// Whether the machine measured this at all. A meter that was never measured
    /// is drawn as unavailable rather than as an idle machine.
    pub measured: bool,
}

/// The Task Manager's model: preferences, rows, and what is selected.
#[derive(Debug, Clone, Default)]
pub struct TaskManager {
    /// Everything the user set.
    pub prefs: Prefs,
    /// The rows the daemon reported, already rate-converted.
    pub rows: Vec<ProcRow>,
    /// Installed memory, for the share column.
    pub total_mb: Option<u64>,
    /// The machine-wide readings the meters are drawn from.
    pub machine: Machine,
    /// The selected row index into [`TaskManager::filtered`].
    pub selected: Option<usize>,
    /// The first row drawn.
    pub scroll: usize,
    /// Groups the user closed.
    pub collapsed: Vec<String>,
    /// The last message the window shows (a copy, a refusal, an end).
    pub message: Option<String>,
}

impl TaskManager {
    /// A manager with the default preferences.
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefs: Prefs::default(),
            ..Self::default()
        }
    }

    /// Take the machine's process table, keeping the selection by pid.
    pub fn observe(&mut self, facts: &SystemFacts) {
        let selected_pid = self.selected_pid();
        self.rows.clone_from(&facts.processes);
        self.total_mb = facts.mem_total_mb;
        // The machine meters come from the same facts, so a meter and the table
        // under it can never disagree about the machine they describe. The frame
        // time is the shell's own and is carried across a sample: it is measured
        // once a frame, and a sample is not a frame.
        self.machine = Machine {
            cpu: facts.cpu_now.or(facts.cpu_fraction),
            cpu_live: facts.cpu_now.is_some(),
            mem_used_mb: facts.mem_used_mb,
            mem_total_mb: facts.mem_total_mb,
            mem_available_mb: facts.mem_available_mb,
            mem_cached_mb: facts.mem_cached_mb,
            swap_used_mb: facts.swap_used_mb,
            load1: facts.load1,
            cpus: facts.cpus,
            sysd_rss_mb: facts.sysd_rss_mb,
            frame_ms: self.machine.frame_ms,
            uptime_secs: facts.uptime_secs,
            tasks: facts.tasks,
        };
        if let Some(pid) = selected_pid {
            let vis = self.filtered();
            if let Some(i) = vis.iter().position(|r| r.pid == pid) {
                self.selected = Some(i);
            }
        }
    }

    /// Record how long the last frame took to draw.
    ///
    /// This one reading does not come from the daemon: it is measured by the
    /// shell, which is the only component that knows how long its own frame took.
    pub fn set_frame_ms(&mut self, ms: f32) {
        self.machine.frame_ms = Some(ms);
    }

    /// The machine-wide meters, each compared against the bands it was given.
    ///
    /// The bands are read here rather than in the drawing code. A threshold a
    /// person set that no reading is ever compared against is a setting that
    /// does nothing, which is the defect this model exists to make impossible.
    #[must_use]
    pub fn meters(&self) -> Vec<Meter> {
        let m = &self.machine;
        let cpu_pct = m.cpu.map(|f| f * 100.0);
        let mem_pct = match (m.mem_used_mb, m.mem_total_mb) {
            (Some(used), Some(total)) if total > 0 => Some(used as f32 / total as f32 * 100.0),
            _ => None,
        };
        let swap_mb = m.swap_used_mb.map(|v| v as f32);
        let load = m.load1;
        let frame = m.frame_ms;
        let daemon_mb = m.sysd_rss_mb.map(|v| v as f32);
        // Load is read against the core count, because a load of 4 is idle on a
        // 16-core machine and saturated on a two-core one. The band itself is a
        // raw load average, as `/proc/loadavg` reports it.
        let cores = m.cpus.unwrap_or(1).max(1) as f32;
        let cpu_label = if m.cpu_live {
            "CPU"
        } else {
            "CPU · since boot"
        };
        vec![
            self.meter_of(Metric::Cpu, cpu_label, cpu_pct, cpu_pct, |v| {
                format!("{v:.0}%")
            }),
            self.meter_of(Metric::Memory, "Memory", mem_pct, mem_pct, |v| {
                format!("{v:.0}%")
            }),
            self.meter_of(Metric::Swap, "Swap", swap_mb, swap_mb, |v| {
                format!("{v:.0} MB")
            }),
            self.meter_of(
                Metric::Load,
                "Load average",
                load,
                load.map(|v| v / cores),
                |v| format!("{v:.2}"),
            ),
            self.meter_of(
                Metric::FrameTime,
                "Frame time",
                frame,
                frame.map(|v| v / 100.0),
                |v| format!("{v:.1} ms"),
            ),
            self.meter_of(Metric::DaemonMemory, "Daemon", daemon_mb, daemon_mb, |v| {
                format!("{v:.0} MB")
            }),
        ]
    }

    /// Build one meter: the reading, the bar, and the band it crossed.
    fn meter_of(
        &self,
        metric: Metric,
        label: &'static str,
        value: Option<f32>,
        share: Option<f32>,
        text: impl Fn(f32) -> String,
    ) -> Meter {
        let warn = self.prefs.warn_of(metric);
        let critical = self.prefs.critical_of(metric);
        let level = value.and_then(|v| {
            if critical.crossed(v) {
                Some(Level::Critical)
            } else if warn.crossed(v) {
                Some(Level::Warn)
            } else {
                None
            }
        });
        let tone = match level {
            Some(l) if self.prefs.colour_crossings => self.prefs.tone_of(metric, l),
            _ => Tone::Neutral,
        };
        let mut marks = String::new();
        for (tag, t) in [("warn", warn), ("critical", critical)] {
            if !t.on {
                continue;
            }
            if !marks.is_empty() {
                marks.push_str(", ");
            }
            if t.unit.is_empty() {
                marks.push_str(&format!("{tag} {}", t.value));
            } else {
                marks.push_str(&format!("{tag} {} {}", t.value, t.unit));
            }
        }
        Meter {
            metric,
            label,
            text: value.map_or_else(|| "\u{2014}".to_owned(), &text),
            share: share.unwrap_or(0.0).clamp(0.0, 1.0),
            level,
            tone,
            marks,
            measured: value.is_some(),
        }
    }

    /// The pid under the selection, if there is one.
    #[must_use]
    pub fn selected_pid(&self) -> Option<u32> {
        self.filtered().get(self.selected?).map(|p| p.pid)
    }

    /// The rows that pass the filters, in sort order.
    #[must_use]
    pub fn filtered(&self) -> Vec<&ProcRow> {
        let needle = self.prefs.filter_text.to_lowercase();
        let mut out: Vec<&ProcRow> = self
            .rows
            .iter()
            .filter(|p| {
                if !self.scope_admits(p) {
                    return false;
                }
                if !self.prefs.show_kernel_threads && p.cmdline.is_empty() {
                    return false;
                }
                if !self.prefs.show_system_users && p.uid == Some(0) {
                    return false;
                }
                if !self.prefs.filter_state.is_empty() && p.state != self.prefs.filter_state {
                    return false;
                }
                if self.prefs.min_cpu_pct > 0.0
                    && p.cpu.unwrap_or(0.0) * 100.0 < self.prefs.min_cpu_pct
                {
                    return false;
                }
                if self.prefs.min_mem_mb > 0 && p.rss_mb.unwrap_or(0) < self.prefs.min_mem_mb {
                    return false;
                }
                if !needle.is_empty()
                    && !p.name.to_lowercase().contains(&needle)
                    && !p.cmdline.to_lowercase().contains(&needle)
                {
                    return false;
                }
                true
            })
            .collect();
        let (key, asc) = self.prefs.sort;
        let total = self.total_mb;
        let secondary = self.prefs.secondary;
        out.sort_by(|a, b| {
            let ka = key.key(a, total);
            let kb = key.key(b, total);
            let ord = if key == SortKey::Name || key == SortKey::CommandLine {
                ka.1.cmp(&kb.1)
            } else {
                ka.0.total_cmp(&kb.0)
            };
            let ord = if asc { ord } else { ord.reverse() };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
            match secondary {
                Some(s) => {
                    let sa = s.key(a, total);
                    let sb = s.key(b, total);
                    if s == SortKey::Name || s == SortKey::CommandLine {
                        sa.1.cmp(&sb.1)
                    } else {
                        sa.0.total_cmp(&sb.0)
                    }
                }
                None => std::cmp::Ordering::Equal,
            }
        });
        out
    }

    /// The drawn rows: group headings and processes, in order.
    #[must_use]
    pub fn rows_to_draw(&self) -> Vec<Row> {
        let vis = self.filtered();
        if self.prefs.group == GroupKey::None {
            return (0..vis.len()).map(|index| Row::Process { index }).collect();
        }
        // Groups are ordered by their heaviest member, so the group that is
        // using the machine is the first thing on screen.
        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        for (i, p) in vis.iter().enumerate() {
            let Some(name) = self.prefs.group.of(p) else {
                continue;
            };
            match groups.iter_mut().find(|(g, _)| *g == name) {
                Some((_, rows)) => rows.push(i),
                None => groups.push((name, vec![i])),
            }
        }
        let weight = |rows: &[usize]| -> f64 {
            rows.iter()
                .map(|i| vis.get(*i).and_then(|p| p.cpu).map_or(0.0, f64::from))
                .sum::<f64>()
        };
        groups.sort_by(|a, b| weight(&b.1).total_cmp(&weight(&a.1)));
        let mut out = Vec::new();
        for (name, rows) in groups {
            let open = !self.collapsed.contains(&name);
            out.push(Row::Group {
                name,
                count: rows.len(),
                open,
            });
            if open {
                out.extend(rows.into_iter().map(|index| Row::Process { index }));
            }
        }
        out
    }

    /// Which band this process crosses, and on which metric.
    ///
    /// The metric matters because the colour is set per metric: a table where
    /// every warning is the same red is a table where a 50 MB process and a
    /// 2 GB process look alike.
    #[must_use]
    pub fn crossing(&self, p: &ProcRow) -> Option<(Level, Metric)> {
        let cpu = p.cpu.unwrap_or(0.0) * 100.0;
        for (level, metric, value, measured) in [
            (
                Level::Critical,
                Metric::ProcessCpu,
                self.prefs.critical_of(Metric::ProcessCpu).value,
                p.cpu.is_some(),
            ),
            (
                Level::Critical,
                Metric::ProcessMemory,
                self.prefs.critical_of(Metric::ProcessMemory).value,
                p.rss_mb.is_some(),
            ),
            (
                Level::Warn,
                Metric::ProcessCpu,
                self.prefs.warn_of(Metric::ProcessCpu).value,
                p.cpu.is_some(),
            ),
            (
                Level::Warn,
                Metric::ProcessMemory,
                self.prefs.warn_of(Metric::ProcessMemory).value,
                p.rss_mb.is_some(),
            ),
        ] {
            let band = match level {
                Level::Warn => self.prefs.warn_of(metric),
                Level::Critical => self.prefs.critical_of(metric),
            };
            let current = if metric == Metric::ProcessCpu {
                cpu
            } else {
                p.rss_mb.map_or(0.0, |mb| mb as f32)
            };
            if band.on && measured && current >= value {
                return Some((level, metric));
            }
        }
        None
    }

    /// The tone a row is tinted with, from the band it crosses.
    #[must_use]
    pub fn row_tone(&self, p: &ProcRow) -> Option<Tone> {
        if !self.prefs.colour_crossings {
            return None;
        }
        let (level, metric) = self.crossing(p)?;
        Some(self.prefs.tone_of(metric, level))
    }

    /// The rows the current user scope admits.
    #[must_use]
    pub fn scope_admits(&self, p: &ProcRow) -> bool {
        match self.prefs.user_scope {
            UserScope::All => true,
            UserScope::Own => p.uid == Some(1000) || (p.uid != Some(0) && p.uid.is_some()),
            UserScope::System => p.uid == Some(0),
        }
    }

    /// One column's values as text: the selected row, or every shown row.
    #[must_use]
    pub fn column_text(&self, c: Column, all_rows: bool) -> Option<String> {
        let vis = self.filtered();
        if all_rows {
            if vis.is_empty() {
                return None;
            }
            let mut out: String = vis
                .iter()
                .map(|p| c.cell(p, &self.prefs, self.total_mb))
                .collect::<Vec<_>>()
                .join("\n");
            out.push('\n');
            return Some(out);
        }
        let p = vis.get(self.selected?)?;
        Some(c.cell(p, &self.prefs, self.total_mb))
    }

    /// The table in an exchangeable format.
    #[must_use]
    pub fn export(&self, format: Export) -> String {
        match format {
            Export::Tsv => self.table_text(),
            Export::Csv => {
                let quote = |s: &str| -> String {
                    if s.contains([',', '"', '\n']) {
                        format!("\"{}\"", s.replace('"', "\"\""))
                    } else {
                        s.to_owned()
                    }
                };
                let mut out = String::new();
                out.push_str(
                    &self
                        .prefs
                        .columns
                        .iter()
                        .map(|c| quote(c.label()))
                        .collect::<Vec<_>>()
                        .join(","),
                );
                out.push('\n');
                for p in self.filtered() {
                    out.push_str(
                        &self
                            .prefs
                            .columns
                            .iter()
                            .map(|c| quote(&c.cell(p, &self.prefs, self.total_mb)))
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                    out.push('\n');
                }
                out
            }
            Export::Json => {
                let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
                let rows: Vec<String> = self
                    .filtered()
                    .iter()
                    .map(|p| {
                        let fields: Vec<String> = self
                            .prefs
                            .columns
                            .iter()
                            .map(|c| {
                                format!(
                                    "\"{}\":\"{}\"",
                                    c.label(),
                                    esc(&c.cell(p, &self.prefs, self.total_mb))
                                )
                            })
                            .collect();
                        format!("{{{}}}", fields.join(","))
                    })
                    .collect();
                format!("[{}]\n", rows.join(","))
            }
        }
    }

    /// The selected row as tab-separated text, for `CopyRow`.
    #[must_use]
    pub fn row_text(&self) -> Option<String> {
        let vis = self.filtered();
        let p = vis.get(self.selected?)?;
        Some(
            self.prefs
                .columns
                .iter()
                .map(|c| c.cell(p, &self.prefs, self.total_mb))
                .collect::<Vec<_>>()
                .join("\t"),
        )
    }

    /// The whole table as tab-separated text, for `CopyTable`.
    #[must_use]
    pub fn table_text(&self) -> String {
        let mut out = String::new();
        out.push_str(
            &self
                .prefs
                .columns
                .iter()
                .map(|c| c.label())
                .collect::<Vec<_>>()
                .join("\t"),
        );
        out.push('\n');
        for p in self.filtered() {
            out.push_str(
                &self
                    .prefs
                    .columns
                    .iter()
                    .map(|c| c.cell(p, &self.prefs, self.total_mb))
                    .collect::<Vec<_>>()
                    .join("\t"),
            );
            out.push('\n');
        }
        out
    }

    /// Run a feature's action, for the entries that are actions rather than
    /// settings. Returns what to tell the user.
    #[must_use]
    pub fn act(&mut self, change: &Change) -> Option<String> {
        match change {
            Change::CopyRow => {
                let text = self.row_text();
                if text.is_some() {
                    self.message = Some("Row copied".to_owned());
                }
                text
            }
            Change::CopyTable => {
                self.message = Some("Table copied".to_owned());
                Some(self.table_text())
            }
            Change::EndTask | Change::EndTree | Change::EndGroup => {
                let pid = self.selected_pid()?;
                // The shell performs the end; the model says what was asked for,
                // so a model that has nothing selected cannot end anything.
                self.message = Some(format!("Ending process {pid}"));
                None
            }
            Change::CopyColumn(c, all) => {
                let text = self.column_text(*c, *all)?;
                self.message = Some(format!("{} copied", c.label()));
                Some(text)
            }
            Change::Export(format) => {
                self.message = Some(format!("Table copied as {}", format.label()));
                Some(self.export(*format))
            }
            Change::RefreshNow => {
                self.message = Some("Sampling now".to_owned());
                None
            }
            _ => None,
        }
    }
}

/// Format one cell of the table.
fn format_cell(c: Column, p: &ProcRow, prefs: &Prefs, total_mb: Option<u64>) -> String {
    match c {
        Column::Name => p.name.clone(),
        Column::Pid => p.pid.to_string(),
        Column::Parent => p.ppid.to_string(),
        Column::State => p.state.clone(),
        Column::User => p
            .uid
            .map_or_else(|| "\u{2014}".to_owned(), |u| u.to_string()),
        Column::Cpu => match prefs.cpu_format {
            CpuFormat::Percent | CpuFormat::Bar => p.cpu_pct(),
            CpuFormat::Ticks => p.ticks.to_string(),
        },
        Column::CpuTime => format_time(p.ticks, prefs.time_format),
        Column::Memory => match prefs.mem_format {
            MemFormat::Mib => p.rss_text(),
            MemFormat::Gib => p.rss_mb.map_or_else(
                || "\u{2014}".to_owned(),
                |mb| format!("{:.2} GB", mb as f64 / 1024.0),
            ),
            MemFormat::Share => {
                let s = share_pct(p, total_mb);
                if s < 0.0 {
                    "\u{2014}".to_owned()
                } else {
                    format!("{s:.1}%")
                }
            }
        },
        Column::MemShare => {
            let s = share_pct(p, total_mb);
            if s < 0.0 {
                "\u{2014}".to_owned()
            } else {
                format!("{s:.1}%")
            }
        }
        Column::Threads => p.threads.to_string(),
        Column::CommandLine => {
            if p.cmdline.is_empty() {
                "kernel thread".to_owned()
            } else {
                p.cmdline.clone()
            }
        }
    }
}

/// Processor ticks as seconds, or as a duration a person reads.
///
/// `USER_HZ` is 100 on every Linux architecture this system builds for, which is
/// why ticks divide by a hundred here; the value comes from the kernel's own
/// counter either way.
#[must_use]
pub fn format_time(ticks: u64, format: TimeFormat) -> String {
    match format {
        TimeFormat::Ticks => ticks.to_string(),
        TimeFormat::Compact => {
            let secs = ticks.div_euclid(100);
            if secs < 60 {
                format!("{secs}s")
            } else if secs < 3_600 {
                format!("{}m{}s", secs.div_euclid(60), secs.rem_euclid(60))
            } else {
                format!(
                    "{}h{}m",
                    secs.div_euclid(3_600),
                    secs.rem_euclid(3_600).div_euclid(60)
                )
            }
        }
        TimeFormat::Clock => {
            let secs = ticks.div_euclid(100);
            format!(
                "{}:{:02}:{:02}",
                secs.div_euclid(3_600),
                secs.rem_euclid(3_600).div_euclid(60),
                secs.rem_euclid(60)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(pid: u32, name: &str, cpu: Option<f32>, rss: Option<u64>) -> ProcRow {
        ProcRow {
            pid,
            name: name.to_owned(),
            cmdline: format!("/usr/bin/{name}"),
            state: "sleeping".to_owned(),
            ppid: 1,
            threads: 2,
            ticks: 100,
            rss_mb: rss,
            uid: Some(1000),
            cpu,
        }
    }

    fn manager() -> TaskManager {
        let mut m = TaskManager::new();
        m.rows = vec![
            row(1, "systemd", Some(0.001), None),
            row(2, "heavy", Some(0.50), Some(512)),
            row(3, "light", Some(0.02), Some(8)),
        ];
        m.total_mb = Some(8192);
        m
    }

    #[test]
    fn the_table_sorts_by_the_key_it_was_given_in_both_directions() {
        let mut m = manager();
        m.prefs.sort_by(SortKey::Cpu, false);
        let heavy_first: Vec<u32> = m.filtered().iter().map(|p| p.pid).collect();
        assert_eq!(heavy_first, [2, 3, 1]);
        m.prefs.sort_by(SortKey::Cpu, true);
        let light_first: Vec<u32> = m.filtered().iter().map(|p| p.pid).collect();
        assert_eq!(light_first, [1, 3, 2]);
        m.prefs.sort_by(SortKey::Name, true);
        let by_name: Vec<&str> = m.filtered().iter().map(|p| p.name.as_str()).collect();
        assert_eq!(by_name, ["heavy", "light", "systemd"]);
    }

    #[test]
    fn a_process_with_no_rate_sorts_below_every_measured_one() {
        // A first sample has no rate. Sorting it as 0.0% would put a process
        // nobody has measured among the idle ones, which reads as "idle".
        let mut m = manager();
        m.rows.push(row(4, "unmeasured", None, None));
        m.prefs.sort_by(SortKey::Cpu, false);
        let order: Vec<u32> = m.filtered().iter().map(|p| p.pid).collect();
        assert_eq!(
            order,
            [2, 3, 1, 4],
            "the unmeasured one is last, not fourth of four by 0"
        );
    }

    #[test]
    fn every_filter_hides_what_it_says_it_hides() {
        let mut m = manager();
        m.prefs.min_cpu_pct = 5.0;
        assert_eq!(m.filtered().len(), 1, "only the 50% process is over 5%");
        m.prefs.min_cpu_pct = 0.0;
        m.prefs.min_mem_mb = 100;
        assert_eq!(m.filtered().len(), 1, "only the 512 MB process has 100 MB");
        m.prefs.min_mem_mb = 0;
        m.prefs.filter_state = "running".to_owned();
        assert!(m.filtered().is_empty(), "every fixture row is sleeping");
        m.prefs.filter_state.clear();
        m.prefs.filter_text = "HEAV".to_owned();
        assert_eq!(m.filtered().len(), 1, "the search is case-insensitive");
        m.prefs.filter_text.clear();
        m.prefs.show_system_users = false;
        m.rows.first_mut().expect("the fixture has a first row").uid = Some(0);
        assert_eq!(m.filtered().len(), 2, "the root-owned row is hidden");
        m.prefs.show_system_users = true;
        assert_eq!(
            m.filtered().len(),
            3,
            "and shown again -- note that the scope below sees the same row"
        );
        m.prefs.user_scope = UserScope::System;
        assert_eq!(
            m.filtered().len(),
            1,
            "the system scope is uid 0, which is the one row that was changed"
        );
        m.prefs.user_scope = UserScope::All;
        m.rows.push(ProcRow {
            cmdline: String::new(),
            name: "kworker".to_owned(),
            ..row(9, "kworker", Some(0.0), None)
        });
        assert_eq!(
            m.filtered().len(),
            3,
            "a kernel thread is hidden by default: it has no command line and cannot be ended"
        );
        m.prefs.show_kernel_threads = true;
        assert_eq!(m.filtered().len(), 4, "and shown when the setting says so");
    }

    #[test]
    fn grouping_puts_the_heaviest_group_first_and_respects_a_closed_one() {
        let mut m = manager();
        m.rows.push(row(4, "light", Some(0.30), Some(4)));
        m.prefs.group = GroupKey::Name;
        let rows = m.rows_to_draw();
        match rows.first() {
            Some(Row::Group { name, count, open }) => {
                assert_eq!(name, "heavy", "the 50% group leads");
                assert_eq!(*count, 1);
                assert!(*open);
            }
            other => panic!("expected a heading, got {other:?}"),
        }
        assert_eq!(
            rows.len(),
            7,
            "three headings and four rows (two of them named light)"
        );
        m.collapsed.push("light".to_owned());
        let closed = m.rows_to_draw();
        assert!(
            closed.iter().any(|r| matches!(
                r,
                Row::Group { name, open, count } if name == "light" && !*open && *count == 2
            )),
            "a closed group still shows its heading and its count: {closed:?}"
        );
        assert_eq!(closed.len(), 5, "and hides its two rows");
    }

    #[test]
    fn a_closed_group_still_counts_the_rows_it_hides() {
        let mut m = manager();
        m.prefs.group = GroupKey::Name;
        let open = m.rows_to_draw().len();
        m.collapsed.push("light".to_owned());
        assert!(
            m.rows_to_draw().len() < open,
            "closing a group must draw fewer rows"
        );
    }

    #[test]
    fn the_selection_follows_its_process_across_a_resort() {
        // The table re-sorts every second. A selection held by row number would
        // move to a different program under the user's hand.
        let mut m = manager();
        m.prefs.sort_by(SortKey::Cpu, false);
        m.selected = Some(0);
        assert_eq!(m.selected_pid(), Some(2));
        // The process that was heaviest is now the lightest.
        let facts = SystemFacts {
            processes: vec![
                ProcRow {
                    cpu: Some(0.01),
                    ..row(2, "heavy", Some(0.01), Some(512))
                },
                ProcRow {
                    cpu: Some(0.9),
                    ..row(3, "light", None, Some(8))
                },
            ],
            ..SystemFacts::default()
        };
        m.observe(&facts);
        assert_eq!(m.selected_pid(), Some(2), "the selection follows the pid");
        assert_eq!(m.selected, Some(1), "which is now row one of the new order");
    }

    #[test]
    fn the_same_machine_data_renders_differently_under_different_preferences() {
        // The wiring the whole module exists for: a preference has to reach the
        // text a person reads, not just a struct field.
        let m = manager();
        let p = &m.rows[1];
        let mut prefs = Prefs::default();
        assert_eq!(Column::Cpu.cell(p, &prefs, Some(8192)), "50.0");
        assert_eq!(Column::Memory.cell(p, &prefs, Some(8192)), "512 MB");
        assert_eq!(Column::CpuTime.cell(p, &prefs, Some(8192)), "1s");
        prefs.cpu_format = CpuFormat::Ticks;
        prefs.mem_format = MemFormat::Gib;
        prefs.time_format = TimeFormat::Clock;
        assert_eq!(Column::Cpu.cell(p, &prefs, Some(8192)), "100");
        assert_eq!(Column::Memory.cell(p, &prefs, Some(8192)), "0.50 GB");
        assert_eq!(Column::CpuTime.cell(p, &prefs, Some(8192)), "0:00:01");
        prefs.mem_format = MemFormat::Share;
        assert_eq!(Column::Memory.cell(p, &prefs, Some(8192)), "6.2%");
    }

    #[test]
    fn an_unmeasured_field_prints_a_dash_in_every_format() {
        // The rule the whole system is built on: a fact nobody measured is not
        // zero, and the table must not make it look like one.
        let p = row(5, "no-numbers", None, None);
        let mut prefs = Prefs::default();
        assert_eq!(Column::Cpu.cell(&p, &prefs, Some(8192)), "\u{2014}");
        assert_eq!(Column::Memory.cell(&p, &prefs, Some(8192)), "\u{2014}");
        prefs.mem_format = MemFormat::Gib;
        assert_eq!(Column::Memory.cell(&p, &prefs, Some(8192)), "\u{2014}");
        prefs.mem_format = MemFormat::Share;
        assert_eq!(Column::Memory.cell(&p, &prefs, Some(8192)), "\u{2014}");
        assert_eq!(Column::MemShare.cell(&p, &prefs, None), "\u{2014}");
        assert_eq!(
            share_pct(&p, None),
            -1.0,
            "an unknown share sorts below every measured one"
        );
    }

    #[test]
    fn a_threshold_marks_a_process_by_the_setting_not_by_a_constant() {
        let mut m = manager();
        m.rows.push(row(4, "middling", Some(0.30), Some(4)));
        let heavy = m.rows.get(1).expect("the fixture's heaviest row").clone();
        let middling = m.rows.get(3).expect("the fixture's middling row").clone();
        let light = m.rows.get(2).expect("the fixture's lightest row").clone();
        assert_eq!(
            m.crossing(&heavy),
            Some((Level::Critical, Metric::ProcessCpu)),
            "50% crosses the 50% critical band"
        );
        assert_eq!(
            m.crossing(&middling),
            Some((Level::Warn, Metric::ProcessCpu)),
            "30% is over the 25% warn band and under the critical one"
        );
        assert_eq!(m.crossing(&light), None, "2% is not worth marking");
        assert_eq!(
            m.row_tone(&heavy),
            Some(Tone::Bad),
            "the critical tone is what the row is tinted with"
        );
        assert_eq!(m.row_tone(&middling), Some(Tone::Warning));
        let mut prefs = m.prefs.clone();
        assert!(apply(
            &mut prefs,
            &Change::Threshold(Metric::ProcessCpu, Level::Critical, false)
        ));
        m.prefs = prefs;
        assert_eq!(
            m.crossing(&heavy),
            Some((Level::Warn, Metric::ProcessCpu)),
            "with the critical band off, 50% falls back to the warn band"
        );
        assert!(apply(
            &mut m.prefs.clone(),
            &Change::CriticalTone(Metric::ProcessCpu, Tone::Good)
        ));
        m.prefs.critical_tone = vec![(Metric::ProcessCpu, Tone::Good)];
        m.prefs.colour_crossings = false;
        assert_eq!(
            m.row_tone(&heavy),
            None,
            "with colouring off nothing is tinted, whatever the bands say"
        );
    }

    /// A sample of the machine: the aggregate tick counters, and one row per
    /// process carrying the cumulative ticks that process had used.
    fn sample(total_ticks: u64, busy_ticks: u64, procs: &[(u32, u64)]) -> SystemFacts {
        let mut f = SystemFacts {
            cpu_ticks: Some((busy_ticks, total_ticks)),
            cpus: Some(8),
            mem_total_mb: Some(8192),
            mem_used_mb: Some(1024),
            ..SystemFacts::default()
        };
        f.processes = procs
            .iter()
            .map(|(pid, ticks)| {
                let mut p = row(*pid, &format!("process-{pid}"), None, Some(8));
                p.ticks = *ticks;
                p
            })
            .collect();
        f
    }

    #[test]
    fn a_process_burning_a_quarter_of_the_machine_reads_twenty_five_percent() {
        // The end-to-end property the window promises: the percentage in the
        // table is the share of *this machine* between two samples, so a process
        // that used 200 of the 800 ticks the machine spent reads 25%, and the
        // machine's own meter reads 50%. The counters below are what a `/proc`
        // would report, not a model of them.
        let mut facts = sample(4_000, 1_000, &[(1, 100), (2, 500)]);
        facts.observe(sample(4_800, 1_400, &[(1, 300), (2, 500), (3, 640)]));
        assert_eq!(facts.cpu_now, Some(0.5), "400 of the machine's 800 ticks");
        let rate = |pid: u32| {
            facts
                .processes
                .iter()
                .find(|p| p.pid == pid)
                .and_then(|p| p.cpu)
        };
        assert_eq!(rate(1), Some(0.25), "200 of the machine's 800 ticks");
        assert_eq!(
            rate(2),
            Some(0.0),
            "a process that used nothing is 0, not a dash"
        );
        assert_eq!(
            rate(3),
            None,
            "a process that was not in the first sample has no rate to report"
        );
        // What the table draws from that same sample.
        let mut m = TaskManager::new();
        m.observe(&facts);
        m.prefs.sort_by(SortKey::Cpu, false);
        let cells: Vec<String> = m
            .filtered()
            .iter()
            .map(|p| Column::Cpu.cell(p, &m.prefs, m.total_mb))
            .collect();
        assert_eq!(cells.first().map(String::as_str), Some("25.0"));
        assert!(
            cells.iter().any(|c| c == "\u{2014}"),
            "the process nobody has measured draws a dash: {cells:?}"
        );
    }

    #[test]
    fn a_share_of_the_machine_is_never_more_than_the_machine() {
        // A counter that jumps -- a pid reused, a restart -- would otherwise
        // divide into a number over 100%, which reads as a hotter process than
        // the machine can run.
        let mut facts = sample(4_000, 1_000, &[(1, 0)]);
        facts.observe(sample(4_100, 1_050, &[(1, 5_000)]));
        assert_eq!(
            facts.processes.first().and_then(|p| p.cpu),
            Some(1.0),
            "a share is clamped to the whole machine"
        );
    }

    #[test]
    fn every_feature_changes_something_the_table_reads() {
        // The gate the module's doc comment promises. An entry that applies to a
        // field nothing reads is a feature that does not exist, and this is what
        // stops one being added.
        let feats = features();
        assert!(
            feats.len() >= 300,
            "the registry is {} entries, against the 300 it promises",
            feats.len()
        );
        let mut seen = std::collections::HashSet::new();
        for f in &feats {
            assert!(seen.insert(f.id.clone()), "duplicate feature id {}", f.id);
            assert!(!f.label.is_empty(), "{} has no label", f.id);
            assert!(
                f.help.len() > 20,
                "{} has no help text, so the window cannot explain it",
                f.id
            );
        }
        // Actions act on a selection rather than on the table's shape, so they
        // are counted separately -- and every one of them has to be able to
        // produce something, which `an_action_with_nothing_selected_does_nothing`
        // checks from the other side.
        let is_action = |c: &Change| {
            matches!(
                c,
                Change::EndTask
                    | Change::EndTree
                    | Change::EndGroup
                    | Change::CopyRow
                    | Change::CopyTable
                    | Change::CopyColumn(..)
                    | Change::Export(..)
                    | Change::RefreshNow
            )
        };
        let settings = feats.iter().filter(|f| !is_action(&f.change)).count();
        let actions = feats.len().saturating_sub(settings);
        assert!(
            settings > feats.len().div_euclid(2),
            "{settings} settings and {actions} actions, out of {}",
            feats.len()
        );
        // Every setting must move at least one preference away from the default
        // it was handed. An entry whose `apply` returns false is an entry whose
        // target nothing reads, which is the defect this registry exists to
        // make impossible.
        // A setting is a choice, so every dimension it belongs to must offer at
        // least two values. A dimension with one value is a switch that can only
        // be in the position it is already in, which is not a feature.
        let mut dims: std::collections::BTreeMap<&'static str, std::collections::BTreeSet<String>> =
            std::collections::BTreeMap::new();
        for f in &feats {
            // Actions act on a selection rather than on the table's shape, and a
            // reset puts a category back rather than choosing anything, so both
            // are covered by their own tests below.
            if is_action(&f.change) || matches!(f.change, Change::Reset(_)) {
                continue;
            }
            dims.entry(dimension(&f.change))
                .or_default()
                .insert(format!("{:?}", f.change));
        }
        let single: Vec<&&str> = dims
            .iter()
            .filter(|(_, v)| v.len() < 2)
            .map(|(k, _)| k)
            .collect();
        assert!(
            single.is_empty(),
            "{single:?} offer only one value, so they are not choices"
        );
        assert!(
            dims.len() >= 20,
            "only {} dimensions in {} settings",
            dims.len(),
            settings
        );
    }

    #[test]
    fn every_dimension_is_read_by_the_table_or_its_behaviour() {
        // The stricter half of the promise: not just "a field moved", but "the
        // window reads it". For every dimension, two settings that choose
        // different values must produce two different tables (or two different
        // window behaviours) -- if they do not, the preference is one only the
        // tests can see, which is exactly the defect this registry exists to
        // make impossible.
        let all = features();
        let mut by_dim: std::collections::BTreeMap<&'static str, Vec<&Feature>> =
            std::collections::BTreeMap::new();
        for f in &all {
            if matches!(f.change, Change::Reset(_)) {
                continue;
            }
            by_dim.entry(dimension(&f.change)).or_default().push(f);
        }
        let mut unread: Vec<String> = Vec::new();
        for (dim, feats) in &by_dim {
            // Actions `act` on a selection and are covered from the other side,
            // by `an_action_with_nothing_selected_does_nothing`.
            if *dim == "action" {
                continue;
            }
            let Some((a, b)) = feats.iter().find_map(|a| {
                feats
                    .iter()
                    .find(|b| format!("{:?}", b.change) != format!("{:?}", a.change))
                    .map(|b| (a, b))
            }) else {
                unread.push(format!("{dim}: one value only"));
                continue;
            };
            if projection(&a.change) == projection(&b.change) {
                unread.push(format!("{dim}: {a:?} and {b:?} look the same"));
            }
        }
        assert!(
            unread.is_empty(),
            "{} dimension(s) are not read by the window: {unread:#?}",
            unread.len()
        );
    }

    /// The table a setting produces, plus the window behaviour it sets, as one
    /// string. `Reset` is excluded from the test above, because resetting a
    /// category to the value it is already at is a no-op by design and the
    /// category is covered by `a_category_reset_leaves_the_other_categories_alone`.
    fn projection(change: &Change) -> String {
        // Rendered under three machine states, so a band that is only crossed on a
        // busy machine, or only on a struggling one, is still visible here: a
        // threshold nobody can ever see is a threshold that does nothing.
        let mut out = String::new();
        for (state, machine) in machine_states() {
            let mut m = projection_manager();
            m.machine = machine;
            apply(&mut m.prefs, change);
            out.push_str(&format!("on a {state} machine: {}\n", render(&m)));
        }
        // The settings that change how the window behaves rather than what the
        // table says: a preference here is read by a timer, a confirmation or a
        // keystroke, so it has no table row of its own to appear in.
        let mut m = projection_manager();
        apply(&mut m.prefs, change);
        out.push_str(&format!(
            "behaviour: paused={} refresh={} confirm={} delivery={:?} reset={:?}\n",
            m.prefs.paused,
            m.prefs.refresh_ms,
            m.prefs.confirm_end_task,
            m.prefs.delivery,
            matches!(change, Change::Reset(_)),
        ));
        for c in Column::ALL {
            out.push_str(&format!(
                "click {} desc-first={}\n",
                c.label(),
                m.prefs.first_click_desc(c)
            ));
        }
        out
    }

    /// The fixture the projection renders: one row for everything a filter can
    /// hide, a row over each process threshold, and every column shown, so that
    /// any setting has somewhere to show itself.
    fn projection_manager() -> TaskManager {
        let mut m = manager();
        m.rows.push(row(4, "middling", Some(0.30), Some(4)));
        // A kernel thread: the one row the kernel-thread filter hides.
        let mut thread = row(5, "kworker/0:1", Some(0.01), Some(0));
        thread.cmdline = String::new();
        thread.uid = Some(0);
        m.rows.push(thread);
        // A system process: owned by uid 0, with a command line, so only the
        // system-process filter and the user scope have anything to hide.
        let mut service = row(6, "systemd-udevd", Some(0.01), Some(12));
        service.uid = Some(0);
        m.rows.push(service);
        // A row over the process-memory critical band, which the 512 MB fixture
        // row does not reach -- and one over the warn band alone, because a row
        // that is over the CPU band is marked for its CPU, not for its memory.
        m.rows.push(row(7, "bigmem", Some(0.0), Some(4096)));
        m.rows.push(row(8, "memholder", Some(0.0), Some(1024)));
        // Every column, because a column that is not shown cannot show that its
        // width, alignment or heading setting works.
        m.prefs.columns = Column::ALL.to_vec();
        // A tie on the sort key, so a tie-break setting is visible: every fixture
        // row is sleeping and the table is ordered by status.
        m.prefs.sort_by(SortKey::State, true);
        m
    }

    /// Three machine states: idle, over every warn band, and over every critical
    /// band. Together they make every band, and every band colour, observable.
    fn machine_states() -> [(&'static str, Machine); 3] {
        let machine =
            |cpu: f32, used: u64, swap: u64, load: f32, daemon: u64, frame: f32| Machine {
                cpu: Some(cpu),
                cpu_live: true,
                mem_used_mb: Some(used),
                mem_total_mb: Some(8192),
                mem_available_mb: Some(8192u64.saturating_sub(used)),
                mem_cached_mb: Some(used.div_euclid(4)),
                swap_used_mb: Some(swap),
                load1: Some(load),
                cpus: Some(8),
                sysd_rss_mb: Some(daemon),
                frame_ms: Some(frame),
                uptime_secs: Some(3600.0),
                tasks: Some(120),
            };
        [
            ("idle", machine(0.02, 900, 0, 0.1, 4, 8.0)),
            ("busy", machine(0.75, 7_000, 2, 5.0, 40, 20.0)),
            ("struggling", machine(0.95, 7_800, 600, 9.0, 140, 60.0)),
        ]
    }

    /// The dimension a setting belongs to: two settings in the same dimension
    /// are two values of the same choice, and must be distinguishable on screen.
    fn dimension(c: &Change) -> &'static str {
        match c {
            Change::ToggleColumn(_) => "column visibility",
            Change::ColumnWidth(..) => "column width",
            Change::ColumnAlign(..) => "column alignment",
            Change::ShortHeader(..) => "column heading length",
            Change::FirstClickDesc(..) => "click sort direction",
            Change::Sort(..) => "sort key and direction",
            Change::SecondarySort(..) => "tie-break key",
            Change::Group(..) => "grouping",
            Change::Density(..) => "row height",
            Change::CpuFormat(..) => "cpu format",
            Change::MemFormat(..) => "memory format",
            Change::TimeFormat(..) => "duration format",
            Change::Threshold(m, l, _) => match l {
                Level::Warn => match m {
                    Metric::Cpu => "cpu warn band",
                    Metric::Memory => "memory warn band",
                    Metric::Swap => "swap warn band",
                    Metric::Load => "load warn band",
                    Metric::ProcessCpu => "process cpu warn band",
                    Metric::ProcessMemory => "process memory warn band",
                    Metric::FrameTime => "frame time warn band",
                    Metric::DaemonMemory => "daemon memory warn band",
                },
                Level::Critical => match m {
                    Metric::Cpu => "cpu critical band",
                    Metric::Memory => "memory critical band",
                    Metric::Swap => "swap critical band",
                    Metric::Load => "load critical band",
                    Metric::ProcessCpu => "process cpu critical band",
                    Metric::ProcessMemory => "process memory critical band",
                    Metric::FrameTime => "frame time critical band",
                    Metric::DaemonMemory => "daemon memory critical band",
                },
            },
            Change::WarnTone(m, _) => match m {
                Metric::ProcessCpu => "process cpu warn colour",
                Metric::ProcessMemory => "process memory warn colour",
                _ => "machine warn colour",
            },
            Change::CriticalTone(m, _) => match m {
                Metric::ProcessCpu => "process cpu critical colour",
                Metric::ProcessMemory => "process memory critical colour",
                _ => "machine critical colour",
            },
            Change::Delivery(l, _) => match l {
                Level::Warn => "warn delivery",
                Level::Critical => "critical delivery",
            },
            Change::FilterState(_) => "state filter",
            Change::MinCpu(_) => "cpu filter",
            Change::MinMemory(_) => "memory filter",
            Change::ShowKernelThreads(_) => "kernel threads",
            Change::ShowSystemUsers(_) => "system processes",
            Change::UserScope(_) => "user scope",
            Change::ColourCrossings(_) => "crossing colouring",
            Change::StickyHeader(_) => "heading stays visible",
            Change::Zebra(_) => "row shading",
            Change::Paused(_) => "sampling",
            Change::Refresh(_) => "refresh interval",
            Change::ConfirmEndTask(_) => "end-task confirmation",
            Change::Reset(c) => match c {
                Category::Columns => "reset columns",
                Category::Sorting => "reset sorting",
                Category::Layout => "reset layout",
                Category::Formats => "reset formats",
                Category::Thresholds => "reset thresholds",
                Category::Filters => "reset filters",
                Category::Actions => "reset actions",
                Category::Behaviour => "reset behaviour",
            },
            Change::EndTask
            | Change::EndTree
            | Change::EndGroup
            | Change::CopyRow
            | Change::CopyTable
            | Change::CopyColumn(..)
            | Change::Export(..)
            | Change::RefreshNow => "action",
        }
    }

    /// Everything a frame of the table depends on, as one string: the headings,
    /// the rows, the tints, the grouping and the row height. Two managers that
    /// render the same string are indistinguishable to a person looking at them.
    fn render(m: &TaskManager) -> String {
        let mut out = String::new();
        let columns: Vec<String> = m
            .prefs
            .columns
            .iter()
            .map(|c| {
                format!(
                    "{:?} w={:?} a={:?} h={}",
                    c,
                    m.prefs.width_of(*c),
                    m.prefs.align_of(*c),
                    heading(*c, &m.prefs)
                )
            })
            .collect();
        out.push_str(&format!(
            "scale={:?} h={:?} zebra={} sticky={} columns=[{}]\n",
            m.prefs.table_width(1.0),
            m.prefs.density.row_h(),
            m.prefs.zebra,
            m.prefs.sticky_header,
            columns.join("; ")
        ));
        // The meters, because they are part of the window a person looks at: a
        // band that marks one of them is a setting that changes this frame.
        for meter in m.meters() {
            out.push_str(&format!(
                "meter {} {} share={:.3} level={:?} tone={:?} marks={} measured={}\n",
                meter.label,
                meter.text,
                meter.share,
                meter.level,
                meter.tone,
                meter.marks,
                meter.measured
            ));
        }
        for r in m.rows_to_draw() {
            match r {
                Row::Group { name, count, open } => {
                    out.push_str(&format!("[{name} ({count}) open={open}]\n"));
                }
                Row::Process { index } => {
                    let vis = m.filtered();
                    if let Some(p) = vis.get(index) {
                        let cells: Vec<String> = m
                            .prefs
                            .columns
                            .iter()
                            .map(|c| c.cell(p, &m.prefs, m.total_mb))
                            .collect();
                        out.push_str(&format!(
                            "{}|{:?}|{:?}|{:?}\n",
                            cells.join("\t"),
                            m.row_tone(p),
                            m.prefs.columns.first().map_or_else(
                                || m.prefs.align_of(Column::Name),
                                |c| m.prefs.align_of(*c)
                            ),
                            m.prefs.columns.first().map_or_else(
                                || m.prefs.width_of(Column::Name),
                                |c| m.prefs.width_of(*c)
                            )
                        ));
                    }
                }
            }
        }
        let headers: Vec<&str> = m
            .prefs
            .columns
            .iter()
            .map(|c| heading(*c, &m.prefs))
            .collect();
        out.push_str(&headers.join("\t"));
        // The drawn rows, with the tone each is tinted, so a colour setting that
        // nothing reads cannot pass this test.
        out.push('\n');
        out.push_str(&format!("filtered={}\n", m.filtered().len()));
        out
    }

    #[test]
    fn an_action_with_nothing_selected_does_nothing() {
        let mut m = manager();
        m.selected = None;
        // `CopyTable` is not in this list on purpose: it copies the table, and a
        // table exists whether or not a row is selected. Every other action
        // names a process, so with none selected they have nothing to name.
        for change in [
            Change::EndTask,
            Change::EndTree,
            Change::EndGroup,
            Change::CopyRow,
            Change::CopyColumn(Column::Pid, false),
        ] {
            assert_eq!(
                m.act(&change),
                None,
                "{change:?} produced something with no row selected"
            );
        }
        m.selected = Some(0);
        assert!(m.act(&Change::CopyRow).is_some());
        assert!(m.act(&Change::CopyColumn(Column::Pid, true)).is_some());
        assert!(m.act(&Change::Export(Export::Json)).is_some());
    }

    #[test]
    fn an_export_is_something_another_program_can_read() {
        let mut m = manager();
        m.prefs.sort_by(SortKey::Pid, true);
        let csv = m.export(Export::Csv);
        assert_eq!(csv.lines().count(), 4, "a heading and three rows");
        assert!(csv.starts_with("Name,PID,CPU"), "{csv:?}");
        let json = m.export(Export::Json);
        assert!(json.starts_with('[') && json.trim_end().ends_with(']'));
        assert!(json.contains("\"PID\":\"1\""), "{json}");
        // A command line with a comma would break a naive CSV; this one quotes.
        m.rows
            .first_mut()
            .expect("the fixture has a first row")
            .cmdline = "/bin/sh -c echo,a".to_owned();
        m.prefs.columns.push(Column::CommandLine);
        let csv = m.export(Export::Csv);
        assert!(
            csv.contains("\"/bin/sh -c echo,a\""),
            "a comma in a field has to be quoted: {csv}"
        );
    }

    #[test]
    fn a_category_reset_leaves_the_other_categories_alone() {
        let mut p = Prefs::default();
        apply(&mut p, &Change::Density(Density::Comfortable));
        apply(&mut p, &Change::Group(GroupKey::Name));
        apply(&mut p, &Change::MemFormat(MemFormat::Gib));
        assert!(reset_category(&mut p, Category::Formats));
        assert_eq!(p.mem_format, MemFormat::Mib, "formats went back to default");
        assert_eq!(p.density, Density::Comfortable, "layout was not reset");
        assert_eq!(p.group, GroupKey::Name, "nor was grouping");
        assert!(
            !reset_category(&mut p, Category::Formats),
            "already default"
        );
    }

    #[test]
    fn the_registry_covers_every_column_key_metric_and_group() {
        let feats = features();
        let has = |id: &str| feats.iter().any(|f| f.id == id);
        for c in Column::ALL {
            assert!(
                has(&format!("columns.{}.show", slug(c.label()))),
                "{} has no show/hide entry",
                c.label()
            );
        }
        for k in SortKey::ALL {
            assert!(
                has(&format!("sort.{}.ascending", slug(k.label()))),
                "{} cannot be sorted by",
                k.label()
            );
        }
        for m in Metric::ALL {
            assert!(
                has(&format!("thresholds.{}.warn.on", slug(m.label()))),
                "{} has no warn band to set",
                m.label()
            );
        }
        for g in GroupKey::ALL {
            assert!(
                has(&format!("layout.group.{}", slug(g.label()))),
                "{} cannot be chosen",
                g.label()
            );
        }
    }

    #[test]
    fn copying_a_row_and_the_table_produces_text_a_person_can_paste() {
        let mut m = manager();
        m.prefs.sort_by(SortKey::Cpu, false);
        m.selected = Some(0);
        let row_text = m.row_text().expect("a row is selected");
        assert!(row_text.starts_with("heavy"), "{row_text:?}");
        assert_eq!(
            row_text.split('\t').count(),
            m.prefs.columns.len(),
            "one cell per shown column"
        );
        let table = m.table_text();
        assert_eq!(table.lines().count(), 4, "a heading and three rows");
        assert!(table.starts_with("Name\tPID\tCPU"));
    }

    #[test]
    fn a_column_toggled_off_comes_back_where_it_belongs() {
        let mut p = Prefs::default();
        p.toggle_column(Column::Pid);
        assert!(!p.columns.contains(&Column::Pid));
        p.toggle_column(Column::Pid);
        assert_eq!(
            p.columns.get(1),
            Some(&Column::Pid),
            "a column returns to its own place, not to the end: {:?}",
            p.columns
        );
    }

    #[test]
    fn a_short_header_is_shorter_but_still_names_the_column() {
        let mut p = Prefs::default();
        assert_eq!(heading(Column::MemShare, &p), "% Memory");
        apply(&mut p, &Change::ShortHeader(Column::MemShare, true));
        assert_eq!(heading(Column::MemShare, &p), "%Mem");
        for c in Column::ALL {
            assert!(
                !short_label(c).is_empty() && short_label(c).len() <= c.label().len() + 1,
                "{}'s short heading is not shorter",
                c.label()
            );
        }
    }

    #[test]
    fn the_table_width_is_the_sum_of_the_columns_it_shows() {
        let mut p = Prefs::default();
        let natural = p.table_width(1.0);
        p.toggle_column(Column::CommandLine);
        assert!(p.table_width(1.0) > natural);
        p.toggle_column(Column::CommandLine);
        p.widths
            .iter_mut()
            .for_each(|(_, w)| *w = WidthStep::Narrow);
        assert!((p.table_width(1.0) - natural * 0.5).abs() < 0.01);
    }

    #[test]
    fn time_reads_as_a_duration_at_every_scale_a_process_can_reach() {
        assert_eq!(format_time(0, TimeFormat::Compact), "0s");
        assert_eq!(format_time(4_100, TimeFormat::Compact), "41s");
        assert_eq!(format_time(61_000, TimeFormat::Compact), "10m10s");
        assert_eq!(format_time(7_320_000, TimeFormat::Compact), "20h20m");
        assert_eq!(format_time(61_000, TimeFormat::Clock), "0:10:10");
        assert_eq!(format_time(4_100, TimeFormat::Ticks), "4100");
    }
}
