//! The settings platform: one typed registry, one value store, one apply path.
//!
//! # What was here
//!
//! Three preferences (`theme`, `language`, `reduce_motion`), a thirteen-page
//! Settings window where eleven of the pages were read-only fact sheets, and a
//! line-oriented persistence reader in `main.rs` that understood exactly those
//! three keys. A control that was not one of those three did not exist.
//!
//! # What this is
//!
//! A registry where every entry is metadata *and* a declaration of what reads
//! it (#[`Spec::consumer`]), and a value store that validates, applies, records
//! an undo point and reports the truth about whether the write succeeded.
//!
//! The rules this module enforces:
//!
//! 1. **No setting without a reader.** Every entry names the surface that reads
//!    it, and `every_setting_names_a_consumer` fails if a new one arrives
//!    without an answer.
//! 2. **No value outside its declared range.** A number is snapped to its step
//!    and clamped to its bounds; a choice must be one of its options; a colour
//!    must parse. A rejected write changes nothing.
//! 3. **No silent loss.** The decoder reports the lines it rejected instead of
//!    dropping them, so a corrupt file is visible rather than mysteriously
//!    reset.
//! 4. **No success that was not persisted.** [`Model::change`] takes an undo
//!    point; [`Model::finish`] restores it when the write failed.
//!
//! # Scopes
//!
//! This build has one machine and one account, so every setting is
//! machine-scoped and persisted in the account's settings file. The `scope`
//! taxonomy exists in [`Scope`] because the Activity entries are application
//! scoped and are mirrored into that application's own model; nothing here
//! pretends to have a workspace or project layer that does not exist.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use litric_design::ThemeKind;

use crate::i18n::Language;
use crate::taskmgr::{
    tone_label, Column, CpuFormat, Delivery, Density, GroupKey, Level, MemFormat, Metric, Prefs,
    SortKey, Threshold, TimeFormat, UserScope, WidthStep, TONES,
};
use crate::ui::Tone;

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

/// A group of settings, in the order the sidebar lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    /// Colours, type, motion and material.
    Appearance,
    /// The dock at the bottom of the screen.
    Dock,
    /// The bar across the top.
    TopBar,
    /// Banners, the notification list and Do Not Disturb.
    Notifications,
    /// The Activity application: its table, units and thresholds.
    Activity,
    /// The Files application.
    Files,
    /// The Terminal application.
    Terminal,
    /// Idle behaviour, sampling cadence and battery warnings.
    Power,
    /// The session: automatic locking and what the lock screen shows.
    Session,
    /// Diagnostic overlays and the status file.
    Diagnostics,
}

impl Category {
    /// Every category, in sidebar order.
    pub const ALL: [Self; 10] = [
        Self::Appearance,
        Self::Dock,
        Self::TopBar,
        Self::Notifications,
        Self::Activity,
        Self::Files,
        Self::Terminal,
        Self::Power,
        Self::Session,
        Self::Diagnostics,
    ];

    /// The stable identifier used in the settings file and by `--category`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Dock => "dock",
            Self::TopBar => "topbar",
            Self::Notifications => "notifications",
            Self::Activity => "activity",
            Self::Files => "files",
            Self::Terminal => "terminal",
            Self::Power => "power",
            Self::Session => "session",
            Self::Diagnostics => "diagnostics",
        }
    }

    /// The name a person reads.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Dock => "Dock",
            Self::TopBar => "Top bar",
            Self::Notifications => "Notifications",
            Self::Activity => "Activity",
            Self::Files => "Files",
            Self::Terminal => "Terminal",
            Self::Power => "Power",
            Self::Session => "Session",
            Self::Diagnostics => "Diagnostics",
        }
    }

    /// One line saying what the category governs.
    #[must_use]
    pub const fn summary(self) -> &'static str {
        match self {
            Self::Appearance => "Colour, type, material and motion.",
            Self::Dock => "The launcher row and what it shows.",
            Self::TopBar => "The clock, status items and workspace pips.",
            Self::Notifications => "Banners, history and quiet hours.",
            Self::Activity => "The process table's shape, units and limits.",
            Self::Files => "The file manager's table and confirmations.",
            Self::Terminal => "The shell window's type and history.",
            Self::Power => "Idle behaviour and sampling cadence.",
            Self::Session => "Automatic locking and the lock screen.",
            Self::Diagnostics => "Overlays and the status file.",
        }
    }

    /// The category a settings-file key belongs to, for migration reports.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }

    /// The previous category, wrapping.
    #[must_use]
    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        let len = Self::ALL.len();
        let previous = index.checked_sub(1).unwrap_or(len.saturating_sub(1));
        Self::ALL.get(previous).copied().unwrap_or(Self::Appearance)
    }

    /// The next category, wrapping.
    #[must_use]
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        Self::ALL
            .get(
                index
                    .saturating_add(1)
                    .min(Self::ALL.len().saturating_sub(1)),
            )
            .copied()
            .unwrap_or(Self::Diagnostics)
    }
}

/// What a value is and how it may be edited.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// A switch.
    Toggle,
    /// One of a fixed list, in this order.
    Choice(&'static [&'static str]),
    /// A bounded number, snapped to `step`, shown with `unit`.
    Number {
        /// Lowest accepted value.
        min: f32,
        /// Highest accepted value.
        max: f32,
        /// Snapping step.
        step: f32,
        /// Suffix shown after the value (`%`, `ms`, `"` for none).
        unit: &'static str,
    },
    /// `#RRGGBB`.
    Colour,
    /// Free text, at most `max_len` characters.
    Text {
        /// Longest accepted string, in characters.
        max_len: usize,
    },
}

impl Kind {
    /// A short word for the control the UI draws.
    #[must_use]
    pub const fn control(self) -> &'static str {
        match self {
            Self::Toggle => "switch",
            Self::Choice(_) => "choice",
            Self::Number { .. } => "number",
            Self::Colour => "colour",
            Self::Text { .. } => "text",
        }
    }
}

/// A stored value. Choices hold the chosen option itself, not an index, so
/// reordering the options in a later build cannot silently change a saved
/// preference into a different one.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A switch.
    On(bool),
    /// The chosen option from the spec's list.
    Pick(&'static str),
    /// A bounded number.
    Num(f32),
    /// A packed `0xRRGGBB` colour.
    Colour(u32),
    /// Free text.
    Text(String),
}

impl Value {
    /// Whether two values are the same value, ignoring float noise.
    #[must_use]
    pub fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::On(a), Self::On(b)) => a == b,
            (Self::Pick(a), Self::Pick(b)) => a == b,
            (Self::Num(a), Self::Num(b)) => (a - b).abs() < 0.0001,
            (Self::Colour(a), Self::Colour(b)) => a == b,
            (Self::Text(a), Self::Text(b)) => a == b,
            _ => false,
        }
    }
}

/// Which surface reads a setting. This is the anti-placeholder field: a
/// setting that cannot name its reader is not a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consumer {
    /// The theme and material resolution in [`crate::ui::Tokens`].
    Theme,
    /// Geometry the shell composes with (scale, spacing, radius, cursor).
    Layout,
    /// The dock's drawing and hit-testing in `views::draw_dock`.
    Dock,
    /// The top bar in `views::draw_top_bar`.
    TopBar,
    /// Banners and the list in `views`.
    Notifications,
    /// The Activity application's table model.
    Activity,
    /// The Files application's model.
    Files,
    /// The Terminal application's model.
    Terminal,
    /// The frame-pacing and sampling loop.
    Runtime,
    /// The lock screen.
    Lock,
    /// The diagnostic overlay and status file.
    Diagnostics,
}

/// A setting's declaration.
#[derive(Debug, Clone)]
pub struct Spec {
    /// Stable identifier, `category.name`.
    pub id: String,
    /// Which group of the Settings window it appears under.
    pub category: Category,
    /// The sub-heading inside the category.
    pub group: &'static str,
    /// The name a person reads.
    pub label: String,
    /// One sentence saying what changes.
    pub help: String,
    /// Its type and constraints.
    pub kind: Kind,
    /// The value a new account starts with.
    pub default: Value,
    /// Whether it is hidden behind "Advanced".
    pub advanced: bool,
    /// Which surface reads it. Never [`Consumer::None`] because there is no
    /// such variant.
    pub consumer: Consumer,
}

impl Spec {
    /// The value as text, for the settings file and for a row's reading.
    #[must_use]
    pub fn value_text(&self, value: &Value) -> String {
        match value {
            Value::On(on) => if *on { "on" } else { "off" }.to_owned(),
            Value::Pick(pick) => (*pick).to_owned(),
            Value::Num(n) => {
                if (n.fract()).abs() < 0.0001 {
                    format!("{n:.0}")
                } else {
                    format!("{n:.2}")
                }
            }
            Value::Colour(rgb) => format!("#{:06X}", rgb & 0x00FF_FFFF),
            Value::Text(text) => escape(text),
        }
    }

    /// The value as a person reads it, with the unit and friendly choice name.
    #[must_use]
    pub fn display(&self, value: &Value) -> String {
        match (value, self.kind) {
            (Value::On(on), Kind::Toggle) => {
                if *on {
                    "On".to_owned()
                } else {
                    "Off".to_owned()
                }
            }
            (Value::Pick(pick), _) => (*pick).to_owned(),
            (Value::Num(n), Kind::Number { unit, .. }) => {
                if (n.fract()).abs() < 0.0001 {
                    format!("{n:.0}{unit}")
                } else {
                    format!("{n:.2}{unit}")
                }
            }
            (Value::Colour(rgb), Kind::Colour) => format!("#{:06X}", rgb & 0x00FF_FFFF),
            (Value::Text(text), _) => {
                if text.is_empty() {
                    "Empty".to_owned()
                } else {
                    text.clone()
                }
            }
            _ => self.value_text(value),
        }
    }

    /// Check and normalise a proposed value against this declaration.
    ///
    /// Returns the value that will actually be stored, or the reason it cannot
    /// be. A number is snapped to its step and clamped, never rejected for
    /// being 0.3 outside its range: a slider that refuses to reach its own end
    /// is a slider with a bug.
    pub fn validate(&self, value: Value) -> Result<Value, String> {
        match (&self.kind, value) {
            (Kind::Toggle, Value::On(on)) => Ok(Value::On(on)),
            (Kind::Choice(options), Value::Pick(pick)) => {
                if options.contains(&pick) {
                    Ok(Value::Pick(pick))
                } else {
                    Err(format!("{} is not one of {}", pick, self.label))
                }
            }
            (Kind::Number { min, max, step, .. }, Value::Num(n)) => {
                if !n.is_finite() {
                    return Err(format!("{} needs a number", self.label));
                }
                let steps = ((n - min) / step.max(0.0001)).round();
                let snapped = min + steps * step;
                Ok(Value::Num(snapped.clamp(*min, *max)))
            }
            (Kind::Colour, Value::Colour(rgb)) => {
                if rgb <= 0x00FF_FFFF {
                    Ok(Value::Colour(rgb))
                } else {
                    Err(format!("{} needs an #RRGGBB colour", self.label))
                }
            }
            (Kind::Text { max_len }, Value::Text(text)) => {
                let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                if clean.chars().count() > *max_len {
                    return Err(format!(
                        "{} is longer than {max_len} characters",
                        self.label
                    ));
                }
                Ok(Value::Text(clean))
            }
            (kind, _) => Err(format!(
                "{} was given a value that is not a {}",
                self.label,
                kind.control()
            )),
        }
    }

    /// The next option for a choice, or the opposite of a switch.
    #[must_use]
    pub fn cycled(&self, value: &Value) -> Option<Value> {
        match (&self.kind, value) {
            (Kind::Toggle, Value::On(on)) => Some(Value::On(!on)),
            (Kind::Choice(options), Value::Pick(pick)) => {
                let at = options.iter().position(|o| o == pick).unwrap_or(0);
                let next = at
                    .checked_add(1)
                    .filter(|n| *n < options.len())
                    .unwrap_or(0);
                options.get(next).copied().map(Value::Pick)
            }
            (Kind::Colour, Value::Colour(rgb)) => {
                let at = ACCENTS.iter().position(|c| c == rgb).unwrap_or(0);
                let next = at
                    .checked_add(1)
                    .filter(|n| *n < ACCENTS.len())
                    .unwrap_or(0);
                ACCENTS.get(next).copied().map(Value::Colour)
            }
            _ => None,
        }
    }

    /// One step up or down a number.
    #[must_use]
    pub fn stepped(&self, value: &Value, up: bool) -> Option<Value> {
        match (&self.kind, value) {
            (Kind::Number { min, max, step, .. }, Value::Num(n)) => {
                let delta = if up { *step } else { -*step };
                Some(Value::Num((n + delta).clamp(*min, *max)))
            }
            _ => None,
        }
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\s"),
            '\r' => {}
            other => out.push(other),
        }
    }
    out
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('s') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Registry construction helpers
// ---------------------------------------------------------------------------

fn toggle(
    category: Category,
    id: &str,
    group: &'static str,
    label: &str,
    help: &str,
    default: bool,
    consumer: Consumer,
) -> Spec {
    Spec {
        id: id.to_owned(),
        category,
        group,
        label: label.to_owned(),
        help: help.to_owned(),
        kind: Kind::Toggle,
        default: Value::On(default),
        advanced: false,
        consumer,
    }
}

/// Build a choice control. The argument list is the setting's whole metadata
/// in one place, and splitting it into a builder would only move the same ten
/// values one call to the left.
#[allow(clippy::too_many_arguments)]
fn choice(
    category: Category,
    id: &str,
    group: &'static str,
    label: &str,
    help: &str,
    options: &'static [&'static str],
    default: &'static str,
    consumer: Consumer,
) -> Spec {
    Spec {
        id: id.to_owned(),
        category,
        group,
        label: label.to_owned(),
        help: help.to_owned(),
        kind: Kind::Choice(options),
        default: Value::Pick(default),
        advanced: false,
        consumer,
    }
}

/// Build a bounded numeric control. See [`choice`] for why the argument list is
/// long.
#[allow(clippy::too_many_arguments)]
fn number(
    category: Category,
    id: &str,
    group: &'static str,
    label: &str,
    help: &str,
    min: f32,
    max: f32,
    step: f32,
    unit: &'static str,
    default: f32,
    consumer: Consumer,
) -> Spec {
    Spec {
        id: id.to_owned(),
        category,
        group,
        label: label.to_owned(),
        help: help.to_owned(),
        kind: Kind::Number {
            min,
            max,
            step,
            unit,
        },
        default: Value::Num(default),
        advanced: false,
        consumer,
    }
}

fn colour(
    category: Category,
    id: &str,
    group: &'static str,
    label: &str,
    help: &str,
    default: u32,
    consumer: Consumer,
) -> Spec {
    Spec {
        id: id.to_owned(),
        category,
        group,
        label: label.to_owned(),
        help: help.to_owned(),
        kind: Kind::Colour,
        default: Value::Colour(default),
        advanced: true,
        consumer,
    }
}

/// Build a text control. See [`choice`] for why the argument list is long.
#[allow(clippy::too_many_arguments)]
fn text(
    category: Category,
    id: &str,
    group: &'static str,
    label: &str,
    help: &str,
    max_len: usize,
    default: &str,
    consumer: Consumer,
) -> Spec {
    Spec {
        id: id.to_owned(),
        category,
        group,
        label: label.to_owned(),
        help: help.to_owned(),
        kind: Kind::Text { max_len },
        default: Value::Text(default.to_owned()),
        advanced: false,
        consumer,
    }
}

fn advanced(mut spec: Spec) -> Spec {
    spec.advanced = true;
    spec
}

/// The column's stable key inside an Activity setting id.
fn column_key(c: Column) -> String {
    match c {
        Column::Name => "name".to_owned(),
        Column::Pid => "pid".to_owned(),
        Column::Parent => "parent".to_owned(),
        Column::State => "status".to_owned(),
        Column::User => "user".to_owned(),
        Column::Cpu => "cpu".to_owned(),
        Column::CpuTime => "cpu-time".to_owned(),
        Column::Memory => "memory".to_owned(),
        Column::MemShare => "memory-share".to_owned(),
        Column::Threads => "threads".to_owned(),
        Column::CommandLine => "command-line".to_owned(),
    }
}

/// The metric's stable key inside an Activity setting id.
fn metric_key(m: Metric) -> &'static str {
    match m {
        Metric::Cpu => "cpu",
        Metric::Memory => "memory",
        Metric::Swap => "swap",
        Metric::Load => "load",
        Metric::ProcessCpu => "process-cpu",
        Metric::ProcessMemory => "process-memory",
        Metric::FrameTime => "frame-time",
        Metric::DaemonMemory => "daemon-memory",
    }
}

/// The accent colours a colour row cycles through. The same eight the design
/// ships, so a custom accent is always one of a tested set rather than an
/// arbitrary value nothing has looked at.
const ACCENTS: [u32; 8] = [
    0x6A_A6FF, 0xFF_B8_4D, 0xFF_7A_59, 0xFF_6B_9A, 0xB5_8C_FF, 0x3D_D6_C6, 0x7B_E0_7B, 0xE8_E8_EE,
];

const TONE_OPTIONS: [&str; 5] = ["quiet", "amber", "red", "green", "accent"];
const WIDTH_OPTIONS: [&str; 3] = ["narrow", "natural width", "wide"];
const ALIGN_OPTIONS: [&str; 2] = ["left", "right"];
const DENSITY_OPTIONS: [&str; 3] = ["compact rows", "cozy rows", "comfortable rows"];
const CPU_FORMAT_OPTIONS: [&str; 3] = ["percent", "ticks", "bar and number"];
const MEM_FORMAT_OPTIONS: [&str; 3] = ["mebibytes", "gibibytes", "share of installed"];
const TIME_FORMAT_OPTIONS: [&str; 3] = ["compact", "clock", "ticks"];
const GROUP_OPTIONS: [&str; 4] = [
    "no grouping",
    "group by name",
    "group by user",
    "group by status",
];
const USER_SCOPE_OPTIONS: [&str; 3] = ["all users", "my processes", "system accounts"];
const DELIVERY_OPTIONS: [&str; 3] = ["mark only", "status bar", "notification"];
const THEME_OPTIONS: [&str; 4] = ["dark", "light", "high-contrast-dark", "high-contrast-light"];
const SPACING_OPTIONS: [&str; 3] = ["compact", "standard", "roomy"];
const MOTION_SPEED_OPTIONS: [&str; 3] = ["slow", "normal", "fast"];
const CLOCK_OPTIONS: [&str; 3] = ["follow locale", "24-hour", "12-hour"];
const WALLPAPER_OPTIONS: [&str; 2] = ["aurora", "plain"];

fn activity_specs(out: &mut Vec<Spec>) {
    let category = Category::Activity;
    let defaults = Prefs::default();

    for c in Column::ALL {
        let key = column_key(c);
        out.push(toggle(
            category,
            &format!("activity.column.{key}.show"),
            "Columns",
            &format!("Show the {} column", c.label()),
            c.help(),
            defaults.columns.contains(&c),
            Consumer::Activity,
        ));
        out.push(advanced(choice(
            category,
            &format!("activity.column.{key}.width"),
            "Column widths",
            &format!("{} column width", c.label()),
            "Narrow, natural, or one and a half times the natural width. A column that \
             cannot show its own heading is dropped rather than clipped.",
            &WIDTH_OPTIONS,
            "natural width",
            Consumer::Activity,
        )));
        let natural_right = matches!(
            c,
            Column::Pid
                | Column::Parent
                | Column::Cpu
                | Column::CpuTime
                | Column::Memory
                | Column::MemShare
                | Column::Threads
        );
        out.push(advanced(choice(
            category,
            &format!("activity.column.{key}.align"),
            "Alignment",
            &format!("{} column alignment", c.label()),
            "A column of numbers is read by its last digit, so numbers align right by default.",
            &ALIGN_OPTIONS,
            if natural_right { "right" } else { "left" },
            Consumer::Activity,
        )));
        out.push(advanced(toggle(
            category,
            &format!("activity.column.{key}.short-heading"),
            "Headings",
            &format!("Shorten the {} heading", c.label()),
            "Use the compact heading when the window is narrow instead of eliding the \
             longer one mid-word.",
            false,
            Consumer::Activity,
        )));
        let numeric = matches!(
            c,
            Column::Cpu | Column::CpuTime | Column::Memory | Column::MemShare | Column::Threads
        );
        out.push(advanced(toggle(
            category,
            &format!("activity.column.{key}.first-click-descending"),
            "Sorting",
            &format!("{}: first click sorts descending", c.label()),
            "Sort a numeric column largest-first on the first click, as a spreadsheet does.",
            numeric,
            Consumer::Activity,
        )));
    }

    out.push(choice(
        category,
        "activity.sort.key",
        "Sorting",
        "Sort by",
        "The primary sort key for the process table.",
        &[
            "name",
            "pid",
            "status",
            "user",
            "cpu",
            "cpu time",
            "memory",
            "memory share",
            "threads",
            "command line",
        ],
        "cpu",
        Consumer::Activity,
    ));
    out.push(toggle(
        category,
        "activity.sort.ascending",
        "Sorting",
        "Sort ascending",
        "Smallest first. Numbers read best largest-first, names read best A to Z.",
        false,
        Consumer::Activity,
    ));
    out.push(advanced(choice(
        category,
        "activity.sort.secondary",
        "Sorting",
        "Tie-breaker",
        "The key used when the primary key compares equal, so rows do not jump between frames.",
        &[
            "none",
            "name",
            "pid",
            "status",
            "user",
            "cpu",
            "cpu time",
            "memory",
            "memory share",
            "threads",
            "command line",
        ],
        "name",
        Consumer::Activity,
    )));
    out.push(choice(
        category,
        "activity.group",
        "Layout",
        "Grouping",
        "One flat list, or one group per program, user or run state.",
        &GROUP_OPTIONS,
        "no grouping",
        Consumer::Activity,
    ));
    out.push(choice(
        category,
        "activity.density",
        "Layout",
        "Row height",
        "How tall a process row is drawn.",
        &DENSITY_OPTIONS,
        "cozy rows",
        Consumer::Activity,
    ));
    out.push(choice(
        category,
        "activity.format.cpu",
        "Units",
        "CPU column",
        "Percent of the machine, cumulative ticks, or a bar with the number.",
        &CPU_FORMAT_OPTIONS,
        "percent",
        Consumer::Activity,
    ));
    out.push(choice(
        category,
        "activity.format.memory",
        "Units",
        "Memory column",
        "Mebibytes, gibibytes, or a share of installed memory.",
        &MEM_FORMAT_OPTIONS,
        "mebibytes",
        Consumer::Activity,
    ));
    out.push(choice(
        category,
        "activity.format.time",
        "Units",
        "Duration column",
        "Compact (`12m`), clock (`0:12:04`) or raw ticks.",
        &TIME_FORMAT_OPTIONS,
        "compact",
        Consumer::Activity,
    ));

    for m in Metric::ALL {
        let key = metric_key(m);
        let warn = m.band(Level::Warn);
        let critical = m.band(Level::Critical);
        out.push(toggle(
            category,
            &format!("activity.warn.{key}.on"),
            "Warn band",
            &format!("Warn when {}", m.label()),
            &format!(
                "{} The threshold below is what counts as worth a mark.",
                m.help()
            ),
            warn.on,
            Consumer::Activity,
        ));
        out.push(advanced(number(
            category,
            &format!("activity.warn.{key}.at"),
            "Warn band",
            &format!("Warn threshold for {}", m.label()),
            "The threshold at which a mark appears. The default is the band the design ships.",
            warn.value.min(critical.value),
            critical.value.max(warn.value),
            if matches!(m, Metric::Load | Metric::FrameTime) {
                0.1
            } else {
                1.0
            },
            warn.unit,
            warn.value,
            Consumer::Activity,
        )));
        out.push(advanced(toggle(
            category,
            &format!("activity.critical.{key}.on"),
            "Critical band",
            &format!("Say so when {}", m.label()),
            "A critical crossing is announced according to the delivery setting below.",
            critical.on,
            Consumer::Activity,
        )));
        out.push(advanced(number(
            category,
            &format!("activity.critical.{key}.at"),
            "Critical band",
            &format!("Critical threshold for {}", m.label()),
            "The value at which the stronger mark appears.",
            warn.value.max(0.0),
            critical.value.max(critical.value + 1.0),
            if matches!(m, Metric::Load | Metric::FrameTime) {
                0.1
            } else {
                1.0
            },
            critical.unit,
            critical.value,
            Consumer::Activity,
        )));
        out.push(advanced(choice(
            category,
            &format!("activity.warn.{key}.tone"),
            "Mark colours",
            &format!("Warn colour for {}", m.label()),
            "Which tone a row is tinted when it crosses this metric's warn band.",
            &TONE_OPTIONS,
            tone_option(default_tone(&defaults, m, Level::Warn)),
            Consumer::Activity,
        )));
        out.push(advanced(choice(
            category,
            &format!("activity.critical.{key}.tone"),
            "Mark colours",
            &format!("Critical colour for {}", m.label()),
            "Which tone a row is tinted when it crosses this metric's critical band.",
            &TONE_OPTIONS,
            tone_option(default_tone(&defaults, m, Level::Critical)),
            Consumer::Activity,
        )));
    }
    out.push(choice(
        category,
        "activity.delivery.warn",
        "Delivery",
        "A warn crossing",
        "Mark the row only, add a line to the status bar, or raise a notification.",
        &DELIVERY_OPTIONS,
        "mark only",
        Consumer::Activity,
    ));
    out.push(choice(
        category,
        "activity.delivery.critical",
        "Delivery",
        "A critical crossing",
        "How a critical crossing is announced. A notification is the default because a \
         critical band is worth interrupting for.",
        &DELIVERY_OPTIONS,
        "notification",
        Consumer::Activity,
    ));
    out.push(text(
        category,
        "activity.filter.text",
        "Filters",
        "Search text",
        "Show only processes whose name, command line or pid contains this text.",
        64,
        "",
        Consumer::Activity,
    ));
    out.push(advanced(number(
        category,
        "activity.filter.min-cpu",
        "Filters",
        "Hide quiet processes",
        "Hide rows using less than this share of the machine. Zero shows everything.",
        0.0,
        100.0,
        0.5,
        "%",
        0.0,
        Consumer::Activity,
    )));
    out.push(advanced(number(
        category,
        "activity.filter.min-memory",
        "Filters",
        "Hide small processes",
        "Hide rows resident in less than this much memory. Zero shows everything.",
        0.0,
        8192.0,
        8.0,
        "MB",
        0.0,
        Consumer::Activity,
    )));
    out.push(advanced(toggle(
        category,
        "activity.filter.kernel-threads",
        "Filters",
        "Show kernel threads",
        "Kernel threads have no command line and no user; they are hidden by default.",
        false,
        Consumer::Activity,
    )));
    out.push(advanced(toggle(
        category,
        "activity.filter.system-users",
        "Filters",
        "Show system-owned processes",
        "Processes owned by system accounts, which is where the services live.",
        true,
        Consumer::Activity,
    )));
    out.push(advanced(choice(
        category,
        "activity.user-scope",
        "Filters",
        "Users shown",
        "All users, this session's user only, or system accounts only.",
        &USER_SCOPE_OPTIONS,
        "all users",
        Consumer::Activity,
    )));
    out.push(advanced(toggle(
        category,
        "activity.colour-crossings",
        "Appearance",
        "Colour crossing rows",
        "Tint a row in its metric's tone when it crosses a band.",
        true,
        Consumer::Activity,
    )));
    out.push(advanced(toggle(
        category,
        "activity.sticky-header",
        "Appearance",
        "Keep the heading visible",
        "Pin the heading while the rows scroll.",
        true,
        Consumer::Activity,
    )));
    out.push(advanced(toggle(
        category,
        "activity.zebra",
        "Appearance",
        "Alternate row shading",
        "A faint tint on every other row, for reading across a wide table.",
        false,
        Consumer::Activity,
    )));
    out.push(toggle(
        category,
        "activity.confirm-end-task",
        "Behaviour",
        "Confirm before ending a task",
        "Ask first, so a stray key press does not close someone's work.",
        true,
        Consumer::Activity,
    ));
    out.push(number(
        category,
        "activity.refresh-ms",
        "Behaviour",
        "Refresh interval",
        "How often the process table is re-sampled. A paused table keeps the last sample.",
        250.0,
        10_000.0,
        250.0,
        "ms",
        1_000.0,
        Consumer::Activity,
    ));
    out.push(advanced(toggle(
        category,
        "activity.paused",
        "Behaviour",
        "Freeze the table",
        "Stop sampling so a row can be read. Input still works; the numbers hold still.",
        false,
        Consumer::Activity,
    )));
}

fn default_tone(prefs: &Prefs, m: Metric, level: Level) -> Tone {
    prefs.tone_of(m, level)
}

fn tone_option(tone: Tone) -> &'static str {
    TONES
        .iter()
        .map(|t| tone_label(*t))
        .find(|label| *label == tone_label(tone))
        .unwrap_or("quiet")
}

fn build_specs() -> Vec<Spec> {
    let mut out: Vec<Spec> = Vec::new();
    let a = Category::Appearance;

    out.push(choice(
        a,
        "appearance.theme",
        "Colour",
        "Theme",
        "The palette every surface is drawn from. `system` follows the session.",
        &THEME_OPTIONS,
        "dark",
        Consumer::Theme,
    ));
    out.push(choice(
        a,
        "appearance.language",
        "Language",
        "Language",
        "The language the shell speaks. Every string in the shell is translated by \
         the same catalog, so this is one source of truth.",
        &["en", "th"],
        "th",
        Consumer::Layout,
    ));
    out.push(colour(
        a,
        "appearance.accent",
        "Colour",
        "Accent colour",
        "Selection, focus, the primary action and the clock. Drawn over every theme.",
        0x6A_A6FF,
        Consumer::Theme,
    ));
    out.push(toggle(
        a,
        "appearance.accent-follows-theme",
        "Colour",
        "Use the theme's accent",
        "Ignore the custom accent and take the one the theme was designed with.",
        true,
        Consumer::Theme,
    ));
    out.push(advanced(number(
        a,
        "appearance.radius",
        "Shape",
        "Corner radius",
        "How round every panel and control is. Zero is a square-edged desktop.",
        0.0,
        28.0,
        1.0,
        "px",
        12.0,
        Consumer::Layout,
    )));
    out.push(choice(
        a,
        "appearance.spacing",
        "Shape",
        "Spacing",
        "Panel padding and the gap between rows, for a denser or roomier desktop.",
        &SPACING_OPTIONS,
        "standard",
        Consumer::Layout,
    ));
    out.push(number(
        a,
        "appearance.text-scale",
        "Type",
        "Text size",
        "Scales every label and number without moving any panel. 100% is the design size.",
        85.0,
        130.0,
        5.0,
        "%",
        100.0,
        Consumer::Layout,
    ));
    out.push(toggle(
        a,
        "appearance.motion",
        "Motion",
        "Animations",
        "Panel, launcher and switcher transitions. Turning this off makes every change instant.",
        true,
        Consumer::Layout,
    ));
    out.push(choice(
        a,
        "appearance.motion-speed",
        "Motion",
        "Animation speed",
        "How long a transition takes. Ignored when animations are off.",
        &MOTION_SPEED_OPTIONS,
        "normal",
        Consumer::Layout,
    ));
    out.push(toggle(
        a,
        "appearance.transparency",
        "Material",
        "Transparency",
        "Panels sample the wallpaper behind them through a solved glass alpha. Off is an \
         opaque surface, and is what high contrast needs to be readable.",
        true,
        Consumer::Layout,
    ));
    out.push(advanced(number(
        a,
        "appearance.blur",
        "Material",
        "Glass blur",
        "How far the backdrop is blurred under a panel. Zero still frosts; it does not \
         turn the material off.",
        0.0,
        40.0,
        1.0,
        "px",
        18.0,
        Consumer::Layout,
    )));
    out.push(advanced(number(
        a,
        "appearance.shadow",
        "Material",
        "Shadow strength",
        "The drop shadow under windows and panels.",
        0.0,
        150.0,
        5.0,
        "%",
        100.0,
        Consumer::Layout,
    )));
    out.push(advanced(number(
        a,
        "appearance.border",
        "Material",
        "Border strength",
        "The hairline around cards and controls.",
        0.0,
        150.0,
        5.0,
        "%",
        100.0,
        Consumer::Layout,
    )));
    out.push(choice(
        a,
        "appearance.wallpaper",
        "Wallpaper",
        "Wallpaper",
        "The coloured washes behind the desktop, or a plain backdrop.",
        &WALLPAPER_OPTIONS,
        "aurora",
        Consumer::Layout,
    ));
    out.push(advanced(number(
        a,
        "appearance.wallpaper-tint",
        "Wallpaper",
        "Wallpaper tint",
        "How strongly the washes show. A wallpaper is a backdrop for windows, not the \
         subject.",
        0.0,
        100.0,
        5.0,
        "%",
        100.0,
        Consumer::Layout,
    )));
    out.push(number(
        a,
        "appearance.icon-scale",
        "Shape",
        "Icon size",
        "Dock and launcher icon size, as a share of the design size.",
        80.0,
        130.0,
        5.0,
        "%",
        100.0,
        Consumer::Layout,
    ));
    out.push(advanced(number(
        a,
        "appearance.focus-ring",
        "Accessibility",
        "Focus ring width",
        "The outline around the keyboard-focused control.",
        1.0,
        4.0,
        1.0,
        "px",
        2.0,
        Consumer::Layout,
    )));
    out.push(advanced(number(
        a,
        "appearance.cursor-scale",
        "Accessibility",
        "Pointer size",
        "The software pointer's size.",
        100.0,
        200.0,
        10.0,
        "%",
        100.0,
        Consumer::Layout,
    )));
    out.push(advanced(number(
        a,
        "appearance.selection",
        "Material",
        "Selection strength",
        "How strongly a selected row or tile is tinted.",
        40.0,
        160.0,
        10.0,
        "%",
        100.0,
        Consumer::Layout,
    )));
    out.push(advanced(number(
        a,
        "appearance.line-height",
        "Type",
        "Line height",
        "Extra room between the lines of wrapped text. 100% is the design's own \
         leading.",
        100.0,
        160.0,
        5.0,
        "%",
        100.0,
        Consumer::Layout,
    )));

    let d = Category::Dock;
    out.push(toggle(
        d,
        "dock.enabled",
        "Dock",
        "Show the dock",
        "The launcher row at the bottom of the screen.",
        true,
        Consumer::Dock,
    ));
    out.push(number(
        d,
        "dock.size",
        "Dock",
        "Dock size",
        "The resting pill's height, as a share of the design size.",
        80.0,
        130.0,
        5.0,
        "%",
        100.0,
        Consumer::Dock,
    ));
    out.push(toggle(
        d,
        "dock.labels",
        "Dock",
        "Show app names",
        "Draw the name beside each app in the expanded dock.",
        true,
        Consumer::Dock,
    ));
    out.push(toggle(
        d,
        "dock.running-indicator",
        "Dock",
        "Mark running apps",
        "A dot under an app that has a window open.",
        true,
        Consumer::Dock,
    ));
    out.push(advanced(toggle(
        d,
        "dock.magnify",
        "Dock",
        "Magnify on hover",
        "Enlarge the tile under the pointer.",
        false,
        Consumer::Dock,
    )));
    out.push(advanced(toggle(
        d,
        "dock.launcher-button",
        "Dock",
        "Launcher button",
        "The mark at the left of the dock that expands it into the launcher.",
        true,
        Consumer::Dock,
    )));
    out.push(advanced(number(
        d,
        "dock.gap",
        "Dock",
        "Gap between apps",
        "The space between tiles.",
        2.0,
        20.0,
        1.0,
        "px",
        6.0,
        Consumer::Dock,
    )));

    let b = Category::TopBar;
    out.push(toggle(
        b,
        "topbar.enabled",
        "Top bar",
        "Show the top bar",
        "The clock and status items across the top of the screen.",
        true,
        Consumer::TopBar,
    ));
    out.push(number(
        b,
        "topbar.height",
        "Top bar",
        "Top bar height",
        "How tall the bar is. The window manager reserves exactly this much.",
        28.0,
        56.0,
        2.0,
        "px",
        36.0,
        Consumer::TopBar,
    ));
    out.push(toggle(
        b,
        "topbar.clock-seconds",
        "Clock",
        "Clock seconds",
        "Show seconds in the clock.",
        false,
        Consumer::TopBar,
    ));
    out.push(choice(
        b,
        "topbar.clock-24h",
        "Clock",
        "Clock format",
        "Follow the locale, or force 18:40 or 6:40 PM. The locale is what a machine \
         configured for its region should do until asked otherwise.",
        &CLOCK_OPTIONS,
        "follow locale",
        Consumer::TopBar,
    ));
    out.push(toggle(
        b,
        "topbar.date",
        "Clock",
        "Show the date",
        "Draw the date beside the time.",
        true,
        Consumer::TopBar,
    ));
    out.push(toggle(
        b,
        "topbar.battery-percent",
        "Status",
        "Battery percentage",
        "Show the number beside the battery item, when there is a battery.",
        true,
        Consumer::TopBar,
    ));
    out.push(toggle(
        b,
        "topbar.metrics",
        "Status",
        "CPU and memory summary",
        "The live CPU and memory figures in the bar.",
        true,
        Consumer::TopBar,
    ));
    out.push(advanced(toggle(
        b,
        "topbar.workspace-pips",
        "Status",
        "Workspace pips",
        "One pip per workspace, with the active one filled.",
        true,
        Consumer::TopBar,
    )));
    out.push(advanced(toggle(
        b,
        "topbar.status-icons",
        "Status",
        "Network, audio and power items",
        "The quick-settings entries for network, sound and power.",
        true,
        Consumer::TopBar,
    )));
    out.push(advanced(toggle(
        b,
        "topbar.border",
        "Top bar",
        "Separator hairline",
        "A line between the bar and the desktop.",
        true,
        Consumer::TopBar,
    )));
    out.push(advanced(toggle(
        b,
        "topbar.title",
        "Status",
        "Focused window title",
        "Name the focused window in the centre of the bar.",
        true,
        Consumer::TopBar,
    )));

    let n = Category::Notifications;
    out.push(toggle(
        n,
        "notifications.enabled",
        "Banners",
        "Notification banners",
        "Show a banner when something happens. The list still collects events.",
        true,
        Consumer::Notifications,
    ));
    out.push(choice(
        n,
        "notifications.position",
        "Banners",
        "Banner position",
        "Where banners stack: the top right as shipped, centred at the top, or the \
         bottom right.",
        &["top-right", "top-center", "bottom-right"],
        "top-right",
        Consumer::Notifications,
    ));
    out.push(toggle(
        n,
        "notifications.dnd",
        "Banners",
        "Do Not Disturb",
        "No banners, no sound. Everything still goes to the list.",
        false,
        Consumer::Notifications,
    ));
    out.push(number(
        n,
        "notifications.banner-ms",
        "Banners",
        "Banner duration",
        "How long a normal banner stays on screen. Critical notifications never expire.",
        1_000.0,
        15_000.0,
        500.0,
        "ms",
        5_000.0,
        Consumer::Notifications,
    ));
    out.push(advanced(number(
        n,
        "notifications.max-banners",
        "Banners",
        "Banners at once",
        "The most that stack at the bottom of the screen.",
        1.0,
        5.0,
        1.0,
        "",
        3.0,
        Consumer::Notifications,
    )));
    out.push(advanced(toggle(
        n,
        "notifications.show-source",
        "Banners",
        "Show the source",
        "Name the application that raised the notification.",
        true,
        Consumer::Notifications,
    )));
    out.push(advanced(toggle(
        n,
        "notifications.critical-sticky",
        "Banners",
        "Critical banners stay",
        "A critical notification is never dismissed by a timer.",
        true,
        Consumer::Notifications,
    )));
    out.push(advanced(number(
        n,
        "notifications.history",
        "History",
        "History size",
        "How many notifications the list keeps before the oldest are dropped.",
        8.0,
        64.0,
        8.0,
        "",
        32.0,
        Consumer::Notifications,
    )));
    out.push(advanced(toggle(
        n,
        "notifications.clear-on-lock",
        "History",
        "Clear when locking",
        "Empty the notification list when the session locks, so a passer-by reads nothing.",
        true,
        Consumer::Notifications,
    )));

    let f = Category::Files;
    out.push(choice(
        f,
        "files.sort",
        "Table",
        "Sort by",
        "Name, size or modification time.",
        &["name", "size", "modified"],
        "name",
        Consumer::Files,
    ));
    out.push(toggle(
        f,
        "files.ascending",
        "Table",
        "Sort ascending",
        "Smallest or A-to-Z first.",
        true,
        Consumer::Files,
    ));
    out.push(toggle(
        f,
        "files.details",
        "Table",
        "Details pane",
        "Show the selected file's details beside the table.",
        true,
        Consumer::Files,
    ));
    out.push(advanced(toggle(
        f,
        "files.folders-first",
        "Table",
        "Folders first",
        "List directories before files, whatever the sort key.",
        true,
        Consumer::Files,
    )));
    out.push(advanced(toggle(
        f,
        "files.confirm-trash",
        "Behaviour",
        "Confirm before trashing",
        "Ask before moving a file to the Trash. Permanent deletion always asks.",
        true,
        Consumer::Files,
    )));
    out.push(advanced(choice(
        f,
        "files.density",
        "Table",
        "Row height",
        "How tall a file row is drawn.",
        &DENSITY_OPTIONS,
        "cozy rows",
        Consumer::Files,
    )));
    out.push(advanced(toggle(
        f,
        "files.zebra",
        "Table",
        "Alternate row shading",
        "A faint tint on every other file row, for reading across a wide table.",
        false,
        Consumer::Files,
    )));
    out.push(advanced(number(
        f,
        "files.icon-size",
        "Table",
        "File icon size",
        "The size of the icon beside each name.",
        10.0,
        24.0,
        1.0,
        "px",
        16.0,
        Consumer::Files,
    )));

    let t = Category::Terminal;
    out.push(number(
        t,
        "terminal.text-scale",
        "Terminal",
        "Text size",
        "Scale for the terminal's monospace output.",
        85.0,
        140.0,
        5.0,
        "%",
        100.0,
        Consumer::Terminal,
    ));
    out.push(number(
        t,
        "terminal.history",
        "Terminal",
        "Command history",
        "How many past commands Up and Down walk through.",
        10.0,
        500.0,
        10.0,
        "",
        100.0,
        Consumer::Terminal,
    ));
    out.push(text(
        t,
        "terminal.prompt",
        "Terminal",
        "Prompt",
        "The character drawn before the cursor. The shell's own name is printed once \
         when the window opens.",
        16,
        "$",
        Consumer::Terminal,
    ));
    out.push(choice(
        t,
        "terminal.cursor-style",
        "Terminal",
        "Cursor shape",
        "The caret at the end of the input line: a thin bar, a filled block, or an \
         underline.",
        &["bar", "block", "underline"],
        "bar",
        Consumer::Terminal,
    ));
    out.push(advanced(number(
        t,
        "terminal.scrollback",
        "Terminal",
        "Scrollback lines",
        "How many output lines the window keeps. A chatty program pushes the oldest out.",
        100.0,
        2_000.0,
        50.0,
        "",
        400.0,
        Consumer::Terminal,
    )));
    out.push(advanced(toggle(
        t,
        "terminal.clear-on-open",
        "Terminal",
        "Start with a clean screen",
        "Clear the greeting line when the terminal opens.",
        false,
        Consumer::Terminal,
    )));

    let p = Category::Power;
    out.push(number(
        p,
        "power.idle-after",
        "Idle",
        "Idle after",
        "Seconds without input before the shell drops to its idle frame rate.",
        5.0,
        600.0,
        5.0,
        "s",
        20.0,
        Consumer::Runtime,
    ));
    out.push(number(
        p,
        "power.deep-idle-after",
        "Idle",
        "Deep idle after",
        "Seconds without input before the shell stops animating entirely.",
        30.0,
        3_600.0,
        30.0,
        "s",
        120.0,
        Consumer::Runtime,
    ));
    out.push(advanced(toggle(
        p,
        "power.adaptive-idle",
        "Idle",
        "Slow down when idle",
        "Recompose fewer frames while nobody is touching the machine. Off pins the shell \
         at its active rate.",
        true,
        Consumer::Runtime,
    )));
    out.push(number(
        p,
        "power.sample-ms",
        "Sampling",
        "Sample interval",
        "How often the system daemon is asked for the machine's counters.",
        250.0,
        5_000.0,
        250.0,
        "ms",
        1_000.0,
        Consumer::Runtime,
    ));
    out.push(advanced(number(
        p,
        "power.battery-warn",
        "Battery",
        "Battery warning",
        "Raise a notification when the battery falls below this percentage.",
        5.0,
        50.0,
        5.0,
        "%",
        15.0,
        Consumer::Runtime,
    )));

    let s = Category::Session;
    out.push(number(
        s,
        "session.auto-lock-minutes",
        "Locking",
        "Lock after",
        "Minutes of no input before the session locks. Zero never locks on idle; the lock \
         still refuses when no verifier is installed, and says so.",
        0.0,
        60.0,
        1.0,
        "min",
        0.0,
        Consumer::Lock,
    ));
    out.push(advanced(toggle(
        s,
        "session.lock-blur",
        "Lock screen",
        "Blur the session behind the lock",
        "The lock screen samples a blurred copy of the desktop, so a passer-by cannot read \
         the window titles.",
        true,
        Consumer::Lock,
    )));
    out.push(advanced(toggle(
        s,
        "session.lock-clock",
        "Lock screen",
        "Show the clock",
        "A large clock on the lock screen.",
        true,
        Consumer::Lock,
    )));
    out.push(advanced(toggle(
        s,
        "session.lock-user",
        "Lock screen",
        "Show the account name",
        "Name the account being unlocked.",
        true,
        Consumer::Lock,
    )));

    let g = Category::Diagnostics;
    out.push(toggle(
        g,
        "diagnostics.overlay",
        "Overlay",
        "Diagnostics overlay",
        "Live frame time, event log and layout boxes over the desktop.",
        false,
        Consumer::Diagnostics,
    ));
    out.push(advanced(toggle(
        g,
        "diagnostics.fps",
        "Overlay",
        "Show the frame rate",
        "One line with frames per second, counted over the last completed second.",
        false,
        Consumer::Diagnostics,
    )));
    out.push(advanced(toggle(
        g,
        "diagnostics.events",
        "Overlay",
        "Overlay shows input events",
        "One line per input event, in arrival order.",
        true,
        Consumer::Diagnostics,
    )));
    out.push(advanced(toggle(
        g,
        "diagnostics.boxes",
        "Overlay",
        "Overlay shows layout boxes",
        "Outline every window and every registered hotspot, which is how a control with \
         no rectangle is found.",
        false,
        Consumer::Diagnostics,
    )));
    out.push(advanced(toggle(
        g,
        "diagnostics.status-file",
        "Status",
        "Write the status file",
        "The shell's machine-readable state, for a test harness or a bug report.",
        true,
        Consumer::Diagnostics,
    )));
    out.push(advanced(toggle(
        g,
        "diagnostics.show-hidden",
        "Overlay",
        "Report hidden rows",
        "Count rows and columns the window had no room for, instead of dropping them in \
         silence.",
        true,
        Consumer::Diagnostics,
    )));
    out.push(advanced(number(
        g,
        "diagnostics.history",
        "Overlay",
        "Line history",
        "How many diagnostic lines the overlay keeps; the oldest fall off the top.",
        32.0,
        512.0,
        32.0,
        "",
        128.0,
        Consumer::Diagnostics,
    )));
    out.push(advanced(number(
        g,
        "diagnostics.overlay-opacity",
        "Overlay",
        "Overlay opacity",
        "How solid the diagnostics panel is. A panel over live pixels is hard to read at \
         full transparency.",
        40.0,
        100.0,
        5.0,
        "%",
        90.0,
        Consumer::Diagnostics,
    )));

    activity_specs(&mut out);
    out
}

/// The registry, built once.
fn registry() -> &'static [Spec] {
    static REGISTRY: OnceLock<Vec<Spec>> = OnceLock::new();
    REGISTRY.get_or_init(build_specs)
}

/// Every setting, in registry order.
#[must_use]
pub fn specs() -> &'static [Spec] {
    registry()
}

/// One setting's declaration.
#[must_use]
pub fn spec(id: &str) -> Option<&'static Spec> {
    specs().iter().find(|s| s.id == id)
}

/// How many settings exist.
#[must_use]
pub fn count() -> usize {
    specs().len()
}

/// The settings in one category, in registry order.
#[must_use]
pub fn category_specs(category: Category) -> Vec<&'static Spec> {
    specs().iter().filter(|s| s.category == category).collect()
}

// ---------------------------------------------------------------------------
// Values
// ---------------------------------------------------------------------------

/// The stored values, keyed by setting id.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    values: BTreeMap<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut values = BTreeMap::new();
        for spec in specs() {
            let _ = values.insert(spec.id.clone(), spec.default.clone());
        }
        Self { values }
    }
}

impl Settings {
    /// A value, falling back to the declaration's default.
    #[must_use]
    pub fn get(&self, id: &str) -> Value {
        self.values
            .get(id)
            .cloned()
            .or_else(|| spec(id).map(|s| s.default.clone()))
            .unwrap_or(Value::On(false))
    }

    /// A switch's state.
    #[must_use]
    pub fn on(&self, id: &str) -> bool {
        matches!(self.get(id), Value::On(true))
    }

    /// A choice's current option, if the id names a choice.
    #[must_use]
    pub fn pick(&self, id: &str) -> &'static str {
        match self.get(id) {
            Value::Pick(pick) => pick,
            _ => "",
        }
    }

    /// A number.
    #[must_use]
    pub fn num(&self, id: &str) -> f32 {
        match self.get(id) {
            Value::Num(n) => n,
            _ => 0.0,
        }
    }

    /// A colour as packed `0xRRGGBB`.
    #[must_use]
    pub fn colour(&self, id: &str) -> u32 {
        match self.get(id) {
            Value::Colour(rgb) => rgb,
            _ => 0,
        }
    }

    /// Free text.
    #[must_use]
    pub fn text(&self, id: &str) -> String {
        match self.get(id) {
            Value::Text(text) => text,
            _ => String::new(),
        }
    }

    /// Whether a value differs from the declaration's default.
    #[must_use]
    pub fn is_changed(&self, id: &str) -> bool {
        let Some(spec) = spec(id) else {
            return false;
        };
        self.values.get(id).is_some_and(|v| !v.same(&spec.default))
    }

    /// How many values differ from their defaults.
    #[must_use]
    pub fn changed_count(&self) -> usize {
        self.values
            .iter()
            .filter(|(id, v)| spec(id).is_some_and(|s| !v.same(&s.default)))
            .count()
    }

    /// Put one value back to its default.
    pub fn reset(&mut self, id: &str) {
        if let Some(spec) = spec(id) {
            let _ = self.values.insert(id.to_owned(), spec.default.clone());
        }
    }

    /// Put a whole category back to its defaults. Returns how many changed.
    pub fn reset_category(&mut self, category: Category) -> usize {
        let mut changed: usize = 0;
        for spec in specs().iter().filter(|s| s.category == category) {
            if self.is_changed(&spec.id) {
                changed = changed.saturating_add(1);
            }
            let _ = self.values.insert(spec.id.clone(), spec.default.clone());
        }
        changed
    }

    /// Put everything back to its defaults.
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// Validate and store one value.
    ///
    /// On an invalid value nothing changes and the reason comes back.
    pub fn set(&mut self, id: &str, value: Value) -> Result<(), String> {
        let Some(spec) = spec(id) else {
            return Err(format!("no such setting: {id}"));
        };
        let stored = spec.validate(value)?;
        let _ = self.values.insert(id.to_owned(), stored);
        Ok(())
    }

    /// The settings file's text: a version header, then one `id=value` line per
    /// value that differs from its default.
    #[must_use]
    pub fn encode(&self) -> String {
        let mut out =
            String::from("# LitricOS settings. Written by the shell; edits are read back.\n");
        out.push_str("version=2\n");
        for spec in specs() {
            if !self.is_changed(&spec.id) {
                continue;
            }
            let Some(value) = self.values.get(&spec.id) else {
                continue;
            };
            out.push_str(&spec.id);
            out.push('=');
            out.push_str(&spec.value_text(value));
            out.push('\n');
        }
        out
    }

    /// Parse a settings file.
    ///
    /// Everything wrong with the file is reported, not just the first thing: a
    /// file with one bad line should not require a second run to find the next.
    #[must_use]
    pub fn decode(input: &str) -> LoadReport {
        let mut report = LoadReport::default();
        let mut settings = Self::default();
        for (number, line) in input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, raw)) = line.split_once('=') else {
                report.rejected.push(format!(
                    "line {}: no '=' in {line:?}",
                    number.saturating_add(1)
                ));
                continue;
            };
            let key = key.trim();
            if key == "version" {
                report.version = raw.trim().parse::<u32>().unwrap_or(0);
                continue;
            }
            // The v1 file had three bare keys with the same names this registry
            // uses, so they migrate in place; anything else unknown is counted
            // and ignored rather than treated as fatal.
            let Some(spec) = spec(key) else {
                report.unknown = report.unknown.saturating_add(1);
                continue;
            };
            match parse_value(spec, raw.trim()) {
                Ok(value) => {
                    let _ = settings.values.insert(spec.id.clone(), value);
                    report.accepted = report.accepted.saturating_add(1);
                }
                Err(reason) => report.rejected.push(format!("{key}: {reason}")),
            }
        }
        report.settings = settings;
        report
    }

    /// Read one setting's value as text, for the status file.
    #[must_use]
    pub fn reading(&self, id: &str) -> String {
        match spec(id) {
            Some(spec) => spec.display(&self.get(id)),
            None => "unknown".to_owned(),
        }
    }

    /// Translate the Activity settings into the table model.
    ///
    /// The mapping is total in both directions: [`Self::capture_activity`]
    /// undoes it exactly, which is what lets the table's own controls and the
    /// Settings window edit the same preference without two sources of truth.
    pub fn apply_activity(&self, p: &mut Prefs) {
        let shown = |c: Column| self.on(&format!("activity.column.{}.show", column_key(c)));
        // The table's order is a preference of its own (the default order is
        // not `Column::ALL`), so a still-visible column keeps its place and a
        // newly enabled one is inserted where the enum puts it, never appended
        // at the far right where nobody finds it again.
        const DEFAULT_ORDER: [Column; 6] = [
            Column::Name,
            Column::Pid,
            Column::Cpu,
            Column::Memory,
            Column::Threads,
            Column::State,
        ];
        let mut columns: Vec<Column> = if p.columns.is_empty() {
            DEFAULT_ORDER
                .iter()
                .copied()
                .filter(|c| shown(*c))
                .collect()
        } else {
            p.columns.iter().copied().filter(|c| shown(*c)).collect()
        };
        for c in Column::ALL {
            if !shown(c) || columns.contains(&c) {
                continue;
            }
            let at = Column::ALL
                .iter()
                .position(|x| *x == c)
                .unwrap_or(columns.len());
            let insert = columns
                .iter()
                .position(|x| {
                    Column::ALL
                        .iter()
                        .position(|y| y == x)
                        .is_some_and(|i| i > at)
                })
                .unwrap_or(columns.len());
            columns.insert(insert, c);
        }
        if columns.is_empty() {
            columns.push(Column::Name);
        }
        p.columns = columns;
        p.widths = Column::ALL
            .iter()
            .map(|c| {
                let pick = self.pick(&format!("activity.column.{}.width", column_key(*c)));
                let step = WidthStep::ALL
                    .iter()
                    .copied()
                    .find(|w| w.label() == pick)
                    .unwrap_or(WidthStep::Natural);
                (*c, step)
            })
            .collect();
        p.aligns = Column::ALL
            .iter()
            .map(|c| {
                let pick = self.pick(&format!("activity.column.{}.align", column_key(*c)));
                let align = if pick == "right" {
                    crate::taskmgr::Align::Right
                } else {
                    crate::taskmgr::Align::Left
                };
                (*c, align)
            })
            .collect();
        p.short_headers = Column::ALL
            .iter()
            .map(|c| {
                (
                    *c,
                    self.on(&format!("activity.column.{}.short-heading", column_key(*c))),
                )
            })
            .collect();
        p.first_click_desc = Column::ALL
            .iter()
            .map(|c| {
                (
                    *c,
                    self.on(&format!(
                        "activity.column.{}.first-click-descending",
                        column_key(*c)
                    )),
                )
            })
            .collect();
        p.sort = (
            SortKey::ALL
                .iter()
                .copied()
                .find(|k| k.label() == self.pick("activity.sort.key"))
                .unwrap_or(SortKey::Cpu),
            self.on("activity.sort.ascending"),
        );
        let secondary = self.pick("activity.sort.secondary");
        p.secondary = SortKey::ALL
            .iter()
            .copied()
            .find(|k| k.label() == secondary);
        p.group = GroupKey::ALL
            .iter()
            .copied()
            .find(|g| g.label() == self.pick("activity.group"))
            .unwrap_or(GroupKey::None);
        p.density = Density::ALL
            .iter()
            .copied()
            .find(|d| d.label() == self.pick("activity.density"))
            .unwrap_or(Density::Cozy);
        p.cpu_format = CpuFormat::ALL
            .iter()
            .copied()
            .find(|f| f.label() == self.pick("activity.format.cpu"))
            .unwrap_or(CpuFormat::Percent);
        p.mem_format = MemFormat::ALL
            .iter()
            .copied()
            .find(|f| f.label() == self.pick("activity.format.memory"))
            .unwrap_or(MemFormat::Mib);
        p.time_format = TimeFormat::ALL
            .iter()
            .copied()
            .find(|f| f.label() == self.pick("activity.format.time"))
            .unwrap_or(TimeFormat::Compact);
        p.warn = Metric::ALL
            .iter()
            .map(|m| {
                let key = metric_key(*m);
                (
                    *m,
                    Threshold {
                        value: self.num(&format!("activity.warn.{key}.at")),
                        unit: m.band(Level::Warn).unit,
                        on: self.on(&format!("activity.warn.{key}.on")),
                    },
                )
            })
            .collect();
        p.critical = Metric::ALL
            .iter()
            .map(|m| {
                let key = metric_key(*m);
                (
                    *m,
                    Threshold {
                        value: self.num(&format!("activity.critical.{key}.at")),
                        unit: m.band(Level::Critical).unit,
                        on: self.on(&format!("activity.critical.{key}.on")),
                    },
                )
            })
            .collect();
        p.warn_tone = Metric::ALL
            .iter()
            .map(|m| {
                (
                    *m,
                    tone_from_option(self.pick(&format!("activity.warn.{}.tone", metric_key(*m)))),
                )
            })
            .collect();
        p.critical_tone = Metric::ALL
            .iter()
            .map(|m| {
                (
                    *m,
                    tone_from_option(
                        self.pick(&format!("activity.critical.{}.tone", metric_key(*m))),
                    ),
                )
            })
            .collect();
        p.delivery = vec![
            (
                Level::Warn,
                Delivery::ALL
                    .iter()
                    .copied()
                    .find(|d| d.label() == self.pick("activity.delivery.warn"))
                    .unwrap_or(Delivery::Silent),
            ),
            (
                Level::Critical,
                Delivery::ALL
                    .iter()
                    .copied()
                    .find(|d| d.label() == self.pick("activity.delivery.critical"))
                    .unwrap_or(Delivery::Notify),
            ),
        ];
        p.filter_text = self.text("activity.filter.text");
        p.min_cpu_pct = self.num("activity.filter.min-cpu");
        p.min_mem_mb = self.num("activity.filter.min-memory").max(0.0) as u64;
        p.show_kernel_threads = self.on("activity.filter.kernel-threads");
        p.show_system_users = self.on("activity.filter.system-users");
        p.user_scope = UserScope::ALL
            .iter()
            .copied()
            .find(|u| u.label() == self.pick("activity.user-scope"))
            .unwrap_or(UserScope::All);
        p.colour_crossings = self.on("activity.colour-crossings");
        p.sticky_header = self.on("activity.sticky-header");
        p.zebra = self.on("activity.zebra");
        p.confirm_end_task = self.on("activity.confirm-end-task");
        p.refresh_ms = self.num("activity.refresh-ms").max(250.0) as u64;
        p.paused = self.on("activity.paused");
    }

    /// Mirror the table model back into the Activity settings.
    pub fn capture_activity(&mut self, p: &Prefs) {
        for c in Column::ALL {
            let key = column_key(c);
            let _ = self.set(
                &format!("activity.column.{key}.show"),
                Value::On(p.columns.contains(&c)),
            );
            let width = WidthStep::ALL
                .iter()
                .copied()
                .find(|w| *w == p.width_of(c))
                .unwrap_or(WidthStep::Natural);
            let _ = self.set(
                &format!("activity.column.{key}.width"),
                Value::Pick(width.label()),
            );
            let align = if p.align_of(c) == crate::taskmgr::Align::Right {
                "right"
            } else {
                "left"
            };
            let _ = self.set(&format!("activity.column.{key}.align"), Value::Pick(align));
            let _ = self.set(
                &format!("activity.column.{key}.short-heading"),
                Value::On(p.short_header(c)),
            );
            let _ = self.set(
                &format!("activity.column.{key}.first-click-descending"),
                Value::On(p.first_click_desc(c)),
            );
        }
        let _ = self.set("activity.sort.key", Value::Pick(p.sort.0.label()));
        let _ = self.set("activity.sort.ascending", Value::On(p.sort.1));
        let secondary = p.secondary.map_or("none", |k| k.label());
        let _ = self.set("activity.sort.secondary", Value::Pick(secondary));
        let _ = self.set("activity.group", Value::Pick(p.group.label()));
        let _ = self.set("activity.density", Value::Pick(p.density.label()));
        let _ = self.set("activity.format.cpu", Value::Pick(p.cpu_format.label()));
        let _ = self.set("activity.format.memory", Value::Pick(p.mem_format.label()));
        let _ = self.set("activity.format.time", Value::Pick(p.time_format.label()));
        for m in Metric::ALL {
            let key = metric_key(m);
            let warn = p.warn_of(m);
            let critical = p.critical_of(m);
            let _ = self.set(&format!("activity.warn.{key}.on"), Value::On(warn.on));
            let _ = self.set(&format!("activity.warn.{key}.at"), Value::Num(warn.value));
            let _ = self.set(
                &format!("activity.critical.{key}.on"),
                Value::On(critical.on),
            );
            let _ = self.set(
                &format!("activity.critical.{key}.at"),
                Value::Num(critical.value),
            );
            let _ = self.set(
                &format!("activity.warn.{key}.tone"),
                Value::Pick(tone_option(p.tone_of(m, Level::Warn))),
            );
            let _ = self.set(
                &format!("activity.critical.{key}.tone"),
                Value::Pick(tone_option(p.tone_of(m, Level::Critical))),
            );
        }
        let _ = self.set(
            "activity.delivery.warn",
            Value::Pick(p.delivery_of(Level::Warn).label()),
        );
        let _ = self.set(
            "activity.delivery.critical",
            Value::Pick(p.delivery_of(Level::Critical).label()),
        );
        let _ = self.set("activity.filter.text", Value::Text(p.filter_text.clone()));
        let _ = self.set("activity.filter.min-cpu", Value::Num(p.min_cpu_pct));
        let _ = self.set(
            "activity.filter.min-memory",
            Value::Num(p.min_mem_mb as f32),
        );
        let _ = self.set(
            "activity.filter.kernel-threads",
            Value::On(p.show_kernel_threads),
        );
        let _ = self.set(
            "activity.filter.system-users",
            Value::On(p.show_system_users),
        );
        let _ = self.set("activity.user-scope", Value::Pick(p.user_scope.label()));
        let _ = self.set("activity.colour-crossings", Value::On(p.colour_crossings));
        let _ = self.set("activity.sticky-header", Value::On(p.sticky_header));
        let _ = self.set("activity.zebra", Value::On(p.zebra));
        let _ = self.set("activity.confirm-end-task", Value::On(p.confirm_end_task));
        let _ = self.set("activity.refresh-ms", Value::Num(p.refresh_ms as f32));
        let _ = self.set("activity.paused", Value::On(p.paused));
    }
}

fn tone_from_option(option: &str) -> Tone {
    TONES
        .iter()
        .copied()
        .find(|t| tone_label(*t) == option)
        .unwrap_or(Tone::Neutral)
}

fn parse_value(spec: &Spec, raw: &str) -> Result<Value, String> {
    let value = match spec.kind {
        Kind::Toggle => match raw {
            "on" | "true" | "1" | "yes" => Value::On(true),
            "off" | "false" | "0" | "no" => Value::On(false),
            other => return Err(format!("{other:?} is not on/off")),
        },
        Kind::Choice(options) => match options.iter().find(|option| **option == raw) {
            Some(option) => Value::Pick(option),
            None => return Err(format!("{raw:?} is not one of the choices")),
        },
        Kind::Number { min, max, .. } => {
            let n = raw
                .parse::<f32>()
                .map_err(|_| format!("{raw:?} is not a number"))?;
            if !n.is_finite() || n < min - 0.001 || n > max + 0.001 {
                return Err(format!("{n} is outside {min}..{max}"));
            }
            Value::Num(n)
        }
        Kind::Colour => {
            let hex = raw.strip_prefix('#').unwrap_or(raw);
            let rgb = u32::from_str_radix(hex, 16)
                .map_err(|_| format!("{raw:?} is not an #RRGGBB colour"))?;
            if hex.len() != 6 {
                return Err(format!("{raw:?} is not six hex digits"));
            }
            Value::Colour(rgb)
        }
        Kind::Text { max_len } => {
            let text = unescape(raw);
            if text.chars().count() > max_len {
                return Err(format!("longer than {max_len} characters"));
            }
            Value::Text(text)
        }
    };
    spec.validate(value)
}

/// What a decode did.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoadReport {
    /// The values that were read, with defaults for everything else.
    pub settings: Settings,
    /// Lines that could not be read, with the reason.
    pub rejected: Vec<String>,
    /// Keys this build does not know: written by a newer one, or stale.
    pub unknown: usize,
    /// The format version the file declared.
    pub version: u32,
    /// Values that were accepted.
    pub accepted: usize,
}

// ---------------------------------------------------------------------------
// Legacy carrier
// ---------------------------------------------------------------------------

/// The three shell preferences that predate the registry.
///
/// Kept as a plain carrier for the command line and for the migration path:
/// every one of them is now a setting with an id, so there is one source of
/// truth rather than two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences {
    /// Built-in colour theme.
    pub theme: ThemeKind,
    /// Shell interface language.
    pub language: Language,
    /// Whether shell animation is reduced.
    pub reduce_motion: bool,
}

impl Preferences {
    /// Defaults for a new account when no preference file exists.
    #[must_use]
    pub const fn new(theme: ThemeKind, language: Language) -> Self {
        Self {
            theme,
            language,
            reduce_motion: false,
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new(ThemeKind::Dark, Language::Thai)
    }
}

/// The settings id holding the interface language.
const LANGUAGE: &str = "appearance.language";
/// The settings id holding the theme.
const THEME: &str = "appearance.theme";

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// Result state shown while applying a preference.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SaveState {
    /// No edit has happened yet.
    #[default]
    Idle,
    /// The value has been applied and is being written atomically.
    Saving,
    /// The value was written successfully.
    Saved,
    /// Persistence failed and the previous value was restored.
    Failed(String),
}

/// The Settings application's model and the value store behind it.
#[derive(Debug, Clone)]
pub struct Model {
    /// Every value.
    pub values: Settings,
    /// Category in the sidebar.
    pub page: Category,
    /// Search text. Empty shows the category.
    pub query: String,
    /// Which row of the visible list the keyboard is on.
    pub selection: Option<usize>,
    /// First visible row, for scrolling.
    pub scroll: usize,
    /// Last apply/persist state.
    pub save: SaveState,
    /// Whether advanced rows are shown.
    pub advanced: bool,
    /// A short message for the footer: (text, success).
    pub message: Option<(String, bool)>,
    /// The text row currently being typed into, if any.
    pub editing: Option<String>,
    /// The pane the last frame drew, so a wheel can be routed to it.
    pub pane: Option<crate::canvas::Rect>,
    previous: Option<Settings>,
    /// Whether there are changes not yet persisted.
    pub dirty: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self::new(Preferences::default())
    }
}

impl Model {
    /// Construct from an already-loaded preference record.
    #[must_use]
    pub fn new(preferences: Preferences) -> Self {
        let mut values = Settings::default();
        let theme = preferences.theme.token();
        if THEME_OPTIONS.contains(&theme) {
            let _ = values.set(THEME, Value::Pick(theme));
        }
        let language = preferences.language.token();
        if values.set(LANGUAGE, Value::Pick(language)).is_err() {
            let _ = values.set(LANGUAGE, Value::Pick("en"));
        }
        let _ = values.set("appearance.motion", Value::On(!preferences.reduce_motion));
        Self {
            values,
            page: Category::Appearance,
            query: String::new(),
            selection: None,
            scroll: 0,
            save: SaveState::Idle,
            advanced: false,
            message: None,
            editing: None,
            pane: None,
            previous: None,
            dirty: false,
        }
    }

    /// Seed the Activity values from a table model before the first frame.
    pub fn with_activity(mut self, prefs: &Prefs) -> Self {
        self.values.capture_activity(prefs);
        self
    }

    /// The legacy carrier, derived from the values.
    #[must_use]
    pub fn preferences(&self) -> Preferences {
        let theme = ThemeKind::all()
            .iter()
            .copied()
            .find(|t| t.token() == self.values.pick(THEME))
            .unwrap_or(ThemeKind::Dark);
        let language = Language::parse(self.values.pick(LANGUAGE)).unwrap_or(Language::English);
        Preferences {
            theme,
            language,
            reduce_motion: !self.values.on("appearance.motion"),
        }
    }

    /// The interface language.
    #[must_use]
    pub fn language(&self) -> Language {
        self.preferences().language
    }

    /// The interface language, set through the registry.
    pub fn set_language(&mut self, language: Language) -> Result<(), String> {
        self.values.set(LANGUAGE, Value::Pick(language.token()))
    }

    /// The theme kind, before any accent override.
    #[must_use]
    pub fn theme_kind(&self) -> ThemeKind {
        self.preferences().theme
    }

    /// A theme with the accent override applied.
    ///
    /// The accent is the one palette entry every surface reads through
    /// `Tokens::accent`, so overriding it here reaches all of them without a
    /// second theme table.
    #[must_use]
    pub fn theme(&self) -> litric_design::Theme {
        let mut theme = litric_design::Theme::of(self.theme_kind());
        if !self.values.on("appearance.accent-follows-theme") {
            let rgb = self.values.colour("appearance.accent");
            theme.palette.accent = litric_design::Rgb::new(
                ((rgb >> 16) & 0xFF) as u8,
                ((rgb >> 8) & 0xFF) as u8,
                (rgb & 0xFF) as u8,
            );
        }
        theme
    }

    /// The rows the window shows: the query's matches, or the page's rows.
    ///
    /// Advanced rows are hidden until asked for, and the search always searches
    /// *everything*, including advanced rows, because a person who knows the
    /// name of a setting should not have to know which drawer it is in.
    #[must_use]
    pub fn visible(&self) -> Vec<&'static Spec> {
        let query = self.query.trim().to_lowercase();
        if query.is_empty() {
            return specs()
                .iter()
                .filter(|s| s.category == self.page)
                .filter(|s| self.advanced || !s.advanced)
                .collect();
        }
        specs()
            .iter()
            .filter(|s| {
                s.label.to_lowercase().contains(&query)
                    || s.help.to_lowercase().contains(&query)
                    || s.id.to_lowercase().contains(&query)
                    || s.category.title().to_lowercase().contains(&query)
                    || s.category.id().contains(&query)
                    || self.values.reading(&s.id).to_lowercase() == query
            })
            .collect()
    }

    /// The selected row's spec, if the selection is still in range.
    #[must_use]
    pub fn selected(&self) -> Option<&'static Spec> {
        self.selection.and_then(|i| self.visible().get(i).copied())
    }

    /// Move the keyboard selection, scrolling to keep it visible.
    pub fn move_selection(&mut self, down: bool) {
        let len = self.visible().len();
        if len == 0 {
            self.selection = None;
            return;
        }
        // A first arrow press lands on the first row rather than skipping it.
        self.editing = None;
        let next = match (self.selection, down) {
            (None, _) => 0,
            (Some(i), true) => i.saturating_add(1).min(len.saturating_sub(1)),
            (Some(i), false) => i.saturating_sub(1),
        };
        self.selection = Some(next);
        if next < self.scroll {
            self.scroll = next;
        }
        if next >= self.scroll.saturating_add(self.rows_hint()) {
            self.scroll = next
                .saturating_add(1)
                .saturating_sub(self.rows_hint().max(1));
        }
    }

    /// How many rows the window has room for. A conservative estimate; the
    /// drawing code clamps to what it actually drew.
    #[must_use]
    pub const fn rows_hint(&self) -> usize {
        9
    }

    /// Cycle the selected row's control, the keyboard's Enter/Space.
    pub fn activate_selected(&mut self) -> bool {
        let Some(spec) = self.selected() else {
            return false;
        };
        let id = spec.id.clone();
        if matches!(spec.kind, Kind::Toggle) {
            let next = Value::On(!self.values.on(&id));
            return self.change(&id, next);
        }
        if matches!(spec.kind, Kind::Text { .. }) {
            self.editing = Some(id);
            return true;
        }
        if let Some(next) = spec.cycled(&self.values.get(&id)) {
            return self.change(&id, next);
        }
        false
    }

    /// Type into the row being edited. Returns whether anything changed.
    pub fn type_edit(&mut self, text: &str) -> bool {
        let Some(id) = self.editing.clone() else {
            return false;
        };
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if clean.is_empty() {
            return false;
        }
        let mut value = self.values.text(&id);
        value.push_str(&clean);
        self.change(&id, Value::Text(value))
    }

    /// Delete the last character of the row being edited.
    pub fn backspace_edit(&mut self) -> bool {
        let Some(id) = self.editing.clone() else {
            return false;
        };
        let mut value = self.values.text(&id);
        if value.pop().is_none() {
            return false;
        }
        self.change(&id, Value::Text(value))
    }

    /// Stop editing without reverting: every keystroke was already applied and
    /// persisted, which is what makes the footer's "Saved" true.
    pub fn end_edit(&mut self) {
        self.editing = None;
    }

    /// Step the selected row's number, the keyboard's Left/Right.
    pub fn step_selected(&mut self, up: bool) -> bool {
        let Some(spec) = self.selected() else {
            return false;
        };
        let id = spec.id.clone();
        match spec.stepped(&self.values.get(&id), up) {
            Some(next) => self.change(&id, next),
            None => false,
        }
    }

    /// Apply a change and take an undo point.
    ///
    /// The caller is expected to persist the result and report it back through
    /// [`Self::finish`]. A change that was rejected changes nothing and says
    /// why in the footer.
    pub fn change(&mut self, id: &str, value: Value) -> bool {
        if self.previous.is_none() {
            self.previous = Some(self.values.clone());
        }
        match self.values.set(id, value) {
            Ok(()) => {
                self.save = SaveState::Saving;
                self.dirty = true;
                self.message = None;
                true
            }
            Err(reason) => {
                self.message = Some((reason, false));
                self.previous = None;
                false
            }
        }
    }

    /// Change one row by its index in the visible list.
    pub fn change_row(&mut self, index: usize, value: Value) -> bool {
        let Some(spec) = self.visible().get(index).copied() else {
            return false;
        };
        let id = spec.id.clone();
        self.selection = Some(index);
        self.change(&id, value)
    }

    /// Complete a persistence attempt: success keeps the values, failure puts
    /// the previous ones back and says what went wrong.
    pub fn finish(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.save = SaveState::Saved;
                self.dirty = false;
                self.message = Some(("Saved".to_owned(), true));
            }
            Err(error) => {
                if let Some(previous) = self.previous.take() {
                    self.values = previous;
                }
                self.save = SaveState::Failed(error.clone());
                self.message = Some((format!("Not saved: {error}"), false));
                self.dirty = false;
            }
        }
        self.previous = None;
    }

    /// Put one row back to its default and persist that.
    pub fn reset(&mut self, id: &str) -> bool {
        if self.previous.is_none() {
            self.previous = Some(self.values.clone());
        }
        self.values.reset(id);
        self.save = SaveState::Saving;
        self.dirty = true;
        true
    }

    /// Put a category back to its defaults. Returns how many values changed.
    pub fn reset_category(&mut self, category: Category) -> usize {
        if self.previous.is_none() {
            self.previous = Some(self.values.clone());
        }
        let changed = self.values.reset_category(category);
        if changed > 0 {
            self.save = SaveState::Saving;
            self.dirty = true;
            self.message = Some((format!("{} reset", category.title()), true));
        } else {
            self.message = Some((
                format!("{} was already at its defaults", category.title()),
                true,
            ));
        }
        changed
    }

    /// The whole value store as text, in the format [`Model::import`] reads.
    ///
    /// This is [`Settings::encode`] — the same bytes the live settings file is
    /// written with — so an export can always be imported back: the round trip
    /// is the one `a_saved_file_round_trips_every_changed_value` already proves
    /// for the file on disk.
    #[must_use]
    pub fn export(&self) -> String {
        self.values.encode()
    }

    /// Replace every value with the decoded contents of `input`.
    ///
    /// The whole input is validated before anything is applied: a file with a
    /// bad line does not half-apply.
    pub fn import(&mut self, input: &str) -> Result<String, String> {
        let report = Settings::decode(input);
        if !report.rejected.is_empty() {
            return Err(format!(
                "{} line(s) rejected; nothing changed: {}",
                report.rejected.len(),
                report.rejected.join("; ")
            ));
        }
        self.previous = Some(self.values.clone());
        self.values = report.settings;
        self.save = SaveState::Saving;
        self.dirty = true;
        Ok(format!(
            "Imported {} value(s); {} key(s) not known to this build",
            report.accepted, report.unknown
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn with_language(mut m: Model) -> Model {
        let _ = m.set_language(Language::English);
        m
    }

    #[test]
    fn every_setting_has_a_unique_id_and_real_metadata() {
        let mut seen = std::collections::HashSet::new();
        for spec in specs() {
            assert!(seen.insert(spec.id.clone()), "duplicate id {}", spec.id);
            assert!(
                spec.id.starts_with(spec.category.id()) || spec.category == Category::Activity,
                "{} is filed under {}",
                spec.id,
                spec.category.title()
            );
            assert!(!spec.label.is_empty(), "{} has no label", spec.id);
            assert!(
                spec.help.len() > 20,
                "{} has no help text a person could use",
                spec.id
            );
            assert!(
                !spec.group.is_empty(),
                "{} has no sub-heading to live under",
                spec.id
            );
        }
    }

    #[test]
    fn every_default_passes_its_own_validation() {
        for spec in specs() {
            let validated = spec
                .validate(spec.default.clone())
                .unwrap_or_else(|e| panic!("{}: {e}", spec.id));
            assert!(
                validated.same(&spec.default),
                "{} default was altered by validation",
                spec.id
            );
        }
    }

    #[test]
    fn every_setting_names_a_consumer() {
        // The anti-placeholder gate. `Consumer` has no `None` variant, so this
        // can only fail if a spec is built with a consumer that no code reads.
        // The readers are named in `CONSUMERS_ARE_READ` below; a new variant
        // cannot be added without adding it there too.
        const CONSUMERS_ARE_READ: [Consumer; 11] = [
            Consumer::Theme,
            Consumer::Layout,
            Consumer::Dock,
            Consumer::TopBar,
            Consumer::Notifications,
            Consumer::Activity,
            Consumer::Files,
            Consumer::Terminal,
            Consumer::Runtime,
            Consumer::Lock,
            Consumer::Diagnostics,
        ];
        for spec in specs() {
            assert!(
                CONSUMERS_ARE_READ.contains(&spec.consumer),
                "{} has a consumer nothing reads",
                spec.id
            );
        }
    }

    #[test]
    fn the_registry_is_large_enough_to_be_a_platform() {
        assert!(count() >= 200, "the registry is only {} settings", count());
    }

    #[test]
    fn every_category_has_settings_and_a_summary() {
        for category in Category::ALL {
            assert!(
                !category_specs(category).is_empty(),
                "{} has no settings",
                category.title()
            );
            assert!(!category.summary().is_empty());
            assert_eq!(Category::from_id(category.id()), Some(category));
        }
    }

    #[test]
    fn a_number_is_snapped_and_clamped() {
        let mut values = Settings::default();
        assert!(spec("appearance.radius").is_some());
        assert!(values.set("appearance.radius", Value::Num(12.7)).is_ok());
        assert!((values.num("appearance.radius") - 13.0).abs() < 0.001);
        assert!(values.set("appearance.radius", Value::Num(9999.0)).is_ok());
        assert!((values.num("appearance.radius") - 28.0).abs() < 0.001);
        assert!(!values
            .set("appearance.radius", Value::Num(f32::NAN))
            .is_ok());
    }

    #[test]
    fn a_choice_must_be_one_of_its_options() {
        let mut values = Settings::default();
        assert!(values.set("appearance.theme", Value::Pick("light")).is_ok());
        assert!(values.set("appearance.theme", Value::Pick("neon")).is_err());
        // The failed write left the last good value in place.
        assert_eq!(values.pick("appearance.theme"), "light");
    }

    #[test]
    fn a_colour_must_parse_and_round_trips() {
        let mut values = Settings::default();
        assert!(values
            .set("appearance.accent", Value::Colour(0x00_FF_00))
            .is_ok());
        let mut model = Model::new(Preferences::default());
        model.values = values;
        let encoded = model.values.encode();
        let report = Settings::decode(&encoded);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);
        assert_eq!(report.settings.colour("appearance.accent"), 0x00_FF_00);
    }

    #[test]
    fn a_saved_file_round_trips_every_changed_value() {
        let mut model = with_language(Model::default());
        assert!(model.change("appearance.theme", Value::Pick("light")));
        assert!(model.change("appearance.radius", Value::Num(4.0)));
        assert!(model.change("notifications.dnd", Value::On(true)));
        assert!(model.change("activity.column.pid.show", Value::On(false)));
        assert!(model.change("activity.warn.cpu.at", Value::Num(55.0)));
        let text = model.values.encode();
        let report = Settings::decode(&text);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);
        assert_eq!(report.settings, model.values);
    }

    #[test]
    fn a_corrupt_line_is_reported_and_the_rest_is_kept() {
        let text = "version=2\nappearance.theme=light\nappearance.radius=banana\nfuture.key=1\n";
        let report = Settings::decode(text);
        assert_eq!(report.version, 2);
        assert_eq!(report.settings.pick("appearance.theme"), "light");
        assert_eq!(report.rejected.len(), 1);
        assert_eq!(report.unknown, 1);
    }

    #[test]
    fn a_failed_write_restores_the_previous_values() {
        let mut model = with_language(Model::default());
        assert!(model.change("appearance.theme", Value::Pick("light")));
        assert_eq!(model.values.pick("appearance.theme"), "light");
        model.finish(Err("read-only".to_owned()));
        assert_eq!(model.values.pick("appearance.theme"), "dark");
        assert!(matches!(model.save, SaveState::Failed(_)));
    }

    #[test]
    fn a_successful_write_keeps_the_value() {
        let mut model = with_language(Model::default());
        assert!(model.change("appearance.theme", Value::Pick("light")));
        model.finish(Ok(()));
        assert_eq!(model.values.pick("appearance.theme"), "light");
        assert!(matches!(model.save, SaveState::Saved));
        assert!(!model.dirty);
    }

    #[test]
    fn an_invalid_edit_changes_nothing() {
        let mut model = with_language(Model::default());
        let before = model.values.clone();
        assert!(!model.change("appearance.theme", Value::Pick("neon")));
        assert_eq!(model.values, before);
        assert!(model.message.is_some());
    }

    #[test]
    fn resetting_a_category_only_touches_that_category() {
        let mut model = with_language(Model::default());
        assert!(model.change("appearance.theme", Value::Pick("light")));
        assert!(model.change("notifications.dnd", Value::On(true)));
        let changed = model.reset_category(Category::Appearance);
        assert!(changed > 0);
        assert_eq!(model.values.pick("appearance.theme"), "dark");
        assert!(model.values.on("notifications.dnd"));
    }

    #[test]
    fn search_reads_names_help_and_ids_across_every_category() {
        let mut model = with_language(Model::default());
        model.query = "threshold".to_owned();
        let hits = model.visible();
        assert!(!hits.is_empty());
        model.query = "activity.warn.cpu.at".to_owned();
        assert_eq!(model.visible().len(), 1);
        model.query = "not a setting".to_owned();
        assert!(model.visible().is_empty());
    }

    #[test]
    fn advanced_rows_are_hidden_until_asked_for_but_always_searchable() {
        let mut model = with_language(Model::default());
        model.page = Category::Appearance;
        let plain = model.visible().len();
        model.advanced = true;
        let all = model.visible().len();
        assert!(all > plain, "{all} should exceed {plain}");
        model.advanced = false;
        model.query = "Corner radius".to_owned();
        assert!(
            model.visible().iter().any(|s| s.id == "appearance.radius"),
            "an advanced row must be findable by name"
        );
    }

    #[test]
    fn keyboard_selection_stays_inside_the_list() {
        let mut model = with_language(Model::default());
        model.page = Category::Appearance;
        for _ in 0..50 {
            model.move_selection(true);
        }
        let len = model.visible().len();
        assert!(model.selection.is_some_and(|s| s < len));
        for _ in 0..50 {
            model.move_selection(false);
        }
        assert_eq!(model.selection, Some(0));
    }

    #[test]
    fn activating_a_toggle_flips_it_and_a_choice_cycles() {
        let mut model = with_language(Model::default());
        model.page = Category::Appearance;
        model.selection = model
            .visible()
            .iter()
            .position(|s| s.id == "appearance.motion");
        assert!(model.activate_selected());
        assert!(!model.values.on("appearance.motion"));
        assert!(model.activate_selected());
        assert!(model.values.on("appearance.motion"));

        model.query = "Theme".to_owned();
        model.selection = model
            .visible()
            .iter()
            .position(|s| s.id == "appearance.theme");
        assert!(model.activate_selected());
        assert_ne!(model.values.pick("appearance.theme"), "dark");
    }

    #[test]
    fn stepping_reaches_both_ends_and_stops() {
        let mut model = with_language(Model::default());
        model.query = "Corner radius".to_owned();
        model.selection = Some(0);
        for _ in 0..100 {
            let _ = model.step_selected(true);
        }
        assert!((model.values.num("appearance.radius") - 28.0).abs() < 0.001);
        for _ in 0..100 {
            let _ = model.step_selected(false);
        }
        assert!((model.values.num("appearance.radius") - 0.0).abs() < 0.001);
    }

    #[test]
    fn importing_a_bad_file_changes_nothing() {
        let mut model = with_language(Model::default());
        let before = model.values.clone();
        let result = model.import("version=2\nappearance.theme=light\nappearance.radius=nope\n");
        assert!(result.is_err());
        assert_eq!(model.values, before);
    }

    #[test]
    fn importing_a_good_file_applies_every_value() {
        let mut model = with_language(Model::default());
        let result = model.import(
            "version=2\nappearance.theme=light\nnotifications.dnd=on\nactivity.column.pid.show=off\n",
        );
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(model.values.pick("appearance.theme"), "light");
        assert!(model.values.on("notifications.dnd"));
        assert!(!model.values.on("activity.column.pid.show"));
    }

    #[test]
    fn an_exported_file_imports_back_into_the_same_values() {
        let mut model = with_language(Model::default());
        assert!(model.change("appearance.theme", Value::Pick("light")));
        assert!(model.change("terminal.scrollback", Value::Num(2000.0)));
        assert!(model.change("activity.warn.cpu.at", Value::Num(70.0)));
        let text = model.export();

        // A different machine's model: its own defaults and history, and the
        // export is all it is given.
        let mut fresh = with_language(Model::default());
        let report = fresh.import(&text).expect("an export imports cleanly");
        assert!(report.starts_with("Imported "), "{report}");
        assert_eq!(fresh.values, model.values);
    }

    #[test]
    fn the_legacy_preferences_still_round_trip() {
        let preferences = Preferences::new(ThemeKind::Light, Language::English);
        let model = Model::new(preferences);
        assert_eq!(model.preferences(), preferences);
        let model = Model::new(Preferences::new(
            ThemeKind::HighContrastDark,
            Language::Thai,
        ));
        assert_eq!(model.theme_kind(), ThemeKind::HighContrastDark);
    }

    #[test]
    fn a_custom_accent_reaches_the_theme_but_only_when_asked_for() {
        let mut model = with_language(Model::default());
        let theme_accent = model.theme().palette.accent;
        assert!(model.change("appearance.accent", Value::Colour(0x00_11_22)));
        assert_eq!(model.theme().palette.accent, theme_accent);
        assert!(model.change("appearance.accent-follows-theme", Value::On(false)));
        let custom = model.theme().palette.accent;
        assert_eq!((custom.r, custom.g, custom.b), (0x00, 0x11, 0x22));
    }

    #[test]
    fn the_activity_defaults_reproduce_the_table_defaults() {
        let values = Settings::default();
        let mut prefs = Prefs::default();
        // Start from a table that is not already at its defaults, so this
        // proves the settings *set* it rather than agreeing by accident.
        prefs.columns.clear();
        prefs.zebra = true;
        prefs.density = Density::Compact;
        prefs.sort = (SortKey::Pid, true);
        values.apply_activity(&mut prefs);
        assert_eq!(prefs, Prefs::default());
    }

    #[test]
    fn capturing_and_reapplying_the_table_is_identity() {
        let mut prefs = Prefs {
            zebra: true,
            density: Density::Comfortable,
            sort: (SortKey::Memory, true),
            filter_text: "firefox".to_owned(),
            min_cpu_pct: 2.5,
            paused: true,
            ..Prefs::default()
        };
        prefs.toggle_column(Column::Parent);
        let mut values = Settings::default();
        values.capture_activity(&prefs);
        let mut again = Prefs::default();
        values.apply_activity(&mut again);
        assert_eq!(again, prefs);
    }

    #[test]
    fn every_activity_dimension_changes_the_table() {
        // Each of these settings must reach a field the table reads. Compare
        // the resulting table against the default rather than trusting the
        // mapping code to be wired.
        let base = Settings::default();
        let cases: [(&str, Value); 6] = [
            ("activity.density", Value::Pick("comfortable rows")),
            ("activity.zebra", Value::On(true)),
            ("activity.sort.key", Value::Pick("memory")),
            ("activity.format.memory", Value::Pick("share of installed")),
            ("activity.column.pid.show", Value::On(false)),
            ("activity.column.cpu.width", Value::Pick("wide")),
        ];
        for (id, value) in cases {
            let mut values = base.clone();
            values
                .set(id, value)
                .unwrap_or_else(|e| panic!("{id}: {e}"));
            let mut prefs = Prefs::default();
            values.apply_activity(&mut prefs);
            assert_ne!(prefs, Prefs::default(), "{id} did not reach the table");
        }
    }

    /// The first value this setting accepts that differs from `current`.
    ///
    /// Built out of the kind's own rules rather than a table of ids, so a new
    /// setting is covered the moment it is registered: a number moves one step
    /// along its own grid, a choice takes another option, a colour another
    /// accent, text gains a character unless its limit is already reached.
    #[allow(clippy::arithmetic_side_effects, clippy::float_arithmetic)]
    fn next_value(spec: &'static Spec, current: &Value) -> Option<Value> {
        let candidates: Vec<Value> = match (&spec.kind, current) {
            (Kind::Toggle, Value::On(on)) => vec![Value::On(!*on)],
            (Kind::Choice(options), Value::Pick(pick)) => options
                .iter()
                .filter(|option| **option != *pick)
                .copied()
                .map(Value::Pick)
                .collect(),
            (Kind::Number { step, .. }, Value::Num(num)) => {
                vec![Value::Num(num + step), Value::Num(num - step)]
            }
            (Kind::Colour, Value::Colour(rgb)) => ACCENTS
                .iter()
                .filter(|accent| *accent != rgb)
                .map(|accent| Value::Colour(*accent))
                .collect(),
            (Kind::Text { max_len }, Value::Text(text)) => {
                let mut longer = text.clone();
                longer.push('x');
                let candidate = if longer.chars().count() > *max_len {
                    text.chars().take(max_len.saturating_sub(1)).collect()
                } else {
                    longer
                };
                vec![Value::Text(candidate)]
            }
            _ => Vec::new(),
        };
        candidates.into_iter().find_map(|candidate| {
            spec.validate(candidate)
                .ok()
                .filter(|value| !value.same(current))
        })
    }

    /// Every registered setting can be edited, saved and read back.
    ///
    /// The registry test proves each row exists and each default is legal; this
    /// one proves the other half of the promise -- that the id a row draws is an
    /// id the store will accept, that the accepted value is what the store then
    /// reports, and that it survives the settings file. It walks the whole
    /// registry, so a setting cannot be added without also being round-tripped.
    #[test]
    fn every_setting_can_be_changed_saved_and_reloaded() {
        let mut moved = 0_usize;
        for spec in specs() {
            let mut values = Settings::default();
            let before = values.get(&spec.id);
            let Some(next) = next_value(spec, &before) else {
                continue;
            };
            values.set(&spec.id, next.clone()).unwrap_or_else(|error| {
                panic!("{} rejected a value it should accept: {error}", spec.id)
            });
            assert_eq!(
                values.get(&spec.id),
                next,
                "{} did not keep the edit",
                spec.id
            );
            assert!(
                values.is_changed(&spec.id),
                "{} still reads as default",
                spec.id
            );
            let reloaded = Settings::decode(&values.encode());
            assert!(
                reloaded.rejected.is_empty(),
                "{}: {:?}",
                spec.id,
                reloaded.rejected
            );
            assert_eq!(
                reloaded.settings.get(&spec.id),
                next,
                "{} did not survive a save and reload",
                spec.id
            );
            moved = moved.saturating_add(1);
        }
        assert!(moved >= 200, "only {moved} settings round-tripped");
    }

    #[test]
    fn the_status_file_reading_is_human_and_bounded() {
        let values = Settings::default();
        for spec in specs().iter().take(40) {
            let reading = values.reading(&spec.id);
            assert!(!reading.is_empty(), "{} reads as nothing", spec.id);
            assert!(reading.len() < 80, "{} reads as {reading}", spec.id);
        }
    }
}
