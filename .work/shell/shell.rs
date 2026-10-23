//! The desktop's state and what input does to it.
//!
//! Deliberately free of drawing. Everything here is a decision — which window
//! has focus, which overlay is open, whether the machine is idle, what a click
//! at (412, 88) means — and [`crate::views`] turns those decisions into pixels.
//! Splitting them is what makes a desktop testable: every behaviour below is
//! exercised by feeding synthetic input to a shell with no display attached.
//!
//! ```text
//!   InputEvent ──► Shell::handle ──► Effect        (things main must do)
//!                       │
//!                       ├──► WindowManager         (focus, move, snap)
//!                       ├──► Overlay               (launcher, switcher, ...)
//!                       └──► damage()              (what to repaint)
//! ```

use std::collections::VecDeque;
#[cfg(unix)]
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::AtomicBool;
#[cfg(unix)]
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
#[cfg(unix)]
use std::sync::Once;
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use std::time::Duration;

use litric_input::event::{InputEvent, Point, PointerButton};
use litric_input::keys::{Key, Modifiers};
use litric_locale::Regional;
use litric_wm::{HitZone, Rect as WmRect, WindowId, WindowManager, WindowState};

use crate::canvas::Rect;
use crate::lock::Secret;
use crate::shortcuts::{Action, App, MetaTap, Shortcuts};
use crate::ui::{Hotspots, Tokens};

/// How much of the desktop is working.
///
/// Not success-or-failure: a machine with no input devices still has a usable
/// display and should say so, and a machine with no display should still boot.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DesktopMode {
    /// Starting up; nothing has been drawn yet.
    Loading,
    /// Everything the desktop needs is present.
    Ready,
    /// Running with something missing, named here.
    Degraded {
        /// What is missing, in the user's words.
        reason: String,
    },
    /// No display at all. The daemon runs; nothing is drawn.
    Headless {
        /// Why no display was found.
        reason: String,
    },
    /// Something failed badly enough that only the recovery surface is shown.
    Recovery {
        /// What failed.
        reason: String,
    },
}

impl DesktopMode {
    /// Whether the desktop should accept input.
    #[must_use]
    pub const fn is_interactive(&self) -> bool {
        matches!(self, Self::Ready | Self::Degraded { .. })
    }

    /// A short token for the status file.
    #[must_use]
    pub fn token(&self) -> String {
        match self {
            Self::Loading => "loading".to_owned(),
            Self::Ready => "ready".to_owned(),
            Self::Degraded { reason } => format!("degraded({reason})"),
            Self::Headless { reason } => format!("headless({reason})"),
            Self::Recovery { reason } => format!("recovery({reason})"),
        }
    }
}

/// What the desktop is doing, which decides how hard the compositor works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LifeState {
    /// Somebody is using it.
    Active,
    /// Nobody has touched it for a while.
    Idle,
    /// Nobody has touched it for a long while.
    DeepIdle,
    /// A window is being dragged or resized: the one case where frames matter
    /// most and dropping any is immediately visible.
    WindowInteraction,
    /// The session is locked.
    Locked,
}

impl LifeState {
    /// A short token for the status file.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::DeepIdle => "deep-idle",
            Self::WindowInteraction => "window-interaction",
            Self::Locked => "locked",
        }
    }

    /// The compositor's target frame interval, in milliseconds.
    ///
    /// **This is a render budget and nothing else.** Input is delivered by a
    /// blocking read on its own thread and reaches the shell the instant it
    /// happens, whatever this returns; audio and system timers are not on this
    /// path at all. Idle reduces how often pixels are recomputed, which is
    /// where the power goes, and nothing else.
    #[must_use]
    pub const fn frame_interval_ms(self) -> u64 {
        match self {
            // A drag has to track the pointer; anything slower is visible as
            // the window lagging behind the cursor.
            Self::WindowInteraction => 8,
            Self::Active => 16,
            // 30 FPS, and only while something is still being drawn. This is
            // a ceiling on the work one frame may take, not a wake-up timer:
            // an idle desktop with no damage asks for no frame at all (see
            // [`Shell::wake_after_ms`]), which is where the power goes.
            Self::Idle => 33,
            // Nothing is moving. Wake often enough for the clock, no more.
            Self::DeepIdle => 500,
            // A locked screen is not an idle one any more: its colour fields
            // drift and the clock breathes, which is the whole design of the
            // surface. 30 FPS, the same budget as an idle desktop, because
            // what is on screen is a slow drift rather than a drag.
            Self::Locked => 33,
        }
    }

    /// Whether animation should run at all in this state.
    #[must_use]
    pub const fn animates(self) -> bool {
        matches!(self, Self::Active | Self::WindowInteraction | Self::Locked)
    }
}

/// When inactivity becomes idle, and idle becomes deep idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdlePolicy {
    /// Milliseconds of no input before [`LifeState::Idle`].
    pub idle_after_ms: u64,
    /// Milliseconds of no input before [`LifeState::DeepIdle`].
    pub deep_after_ms: u64,
}

impl Default for IdlePolicy {
    /// Twenty seconds, then two minutes.
    ///
    /// Twenty is long enough that reading a paragraph does not drop the frame
    /// rate under you, and short enough that a machine left alone stops burning
    /// power almost immediately.
    fn default() -> Self {
        Self {
            idle_after_ms: 20_000,
            deep_after_ms: 120_000,
        }
    }
}

impl IdlePolicy {
    /// Which state applies after this long without input.
    #[must_use]
    pub const fn state_after(self, since_input_ms: u64) -> LifeState {
        if since_input_ms >= self.deep_after_ms {
            LifeState::DeepIdle
        } else if since_input_ms >= self.idle_after_ms {
            LifeState::Idle
        } else {
            LifeState::Active
        }
    }
}

/// Which overlay is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Overlay {
    /// Nothing.
    None,
    /// The application launcher.
    Launcher {
        /// What has been typed.
        query: String,
        /// Which result is highlighted.
        selected: usize,
    },
    /// The window switcher, held open while Alt is down.
    Switcher {
        /// Which window is highlighted.
        selected: usize,
    },
    /// Quick settings: power profiles, style toggles and environments.
    QuickSettings,
    /// The notification list.
    Notifications,
    /// The power menu: lock, sleep, shut down, restart.
    Power,
    /// The LitricOS menu at the top-left: the session's own settings.
    Litro,
}

impl Overlay {
    /// Whether it takes the keyboard.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// A short token for the status file.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Launcher { .. } => "launcher",
            Self::Switcher { .. } => "switcher",
            Self::QuickSettings => "quick-settings",
            Self::Notifications => "notifications",
            Self::Power => "power",
            Self::Litro => "litro",
        }
    }
}

/// Something in the interface that can be clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Hot {
    /// The LitricOS mark: opens the launcher.
    Logo,
    /// A window's body: clicks inside a window belong to it and must never
    /// fall through to the desktop icon underneath.
    Window(u32),
    /// A pinned app in the dock.
    DockItem(u8),
    /// A search-engine chip in the browser's start page.
    EngineChip(u8),
    /// A quick-link tile in the browser's start page.
    QuickLink(u8),
    /// The terminal's input field.
    TerminalInput,
    /// Clear the visible Terminal scrollback.
    TerminalClear,
    /// Start a fresh Terminal command session.
    TerminalNewSession,
    /// The Apps menu.
    MenuApps,
    /// The Workspace menu.
    MenuWorkspace,
    /// The Environment menu.
    MenuEnvironment,
    /// A workspace pip in the top bar.
    Workspace(u8),
    /// The network status item.
    StatusNetwork,
    /// The audio status item.
    StatusAudio,
    /// The battery status item.
    StatusBattery,
    /// The power-profile status item.
    StatusPower,
    /// The CPU and memory summary.
    StatusMetrics,
    /// The notification bell.
    StatusNotifications,
    /// The clock.
    StatusClock,
    /// The user menu: the power menu lives here.
    StatusUser,
    /// A row in the power menu, in this order: lock, sleep, shut down, restart.
    PowerItem(u8),
    /// The LitricOS menu's session-settings row.
    LitroSession,
    /// The LitricOS menu's lock row.
    LitroLock,
    /// A row in the launcher.
    LauncherItem(usize),
    /// The launcher's search field.
    LauncherSearch,
    /// A row in the window switcher.
    SwitcherItem(usize),
    /// A toggle in quick settings.
    QuickToggle(u8),
    /// The power-profile selector in quick settings.
    QuickPower(u8),
    /// An environment row in quick settings.
    QuickEnvironment(u8),
    /// The browser's new-tab button.
    BrowserNewTab,
    /// Navigate the active browser tab backward.
    BrowserBack,
    /// Navigate the active browser tab forward.
    BrowserForward,
    /// Add or remove the active address from bookmarks.
    BrowserBookmark,
    /// Show the recent history panel.
    BrowserHistory,
    /// Show saved addresses.
    BrowserBookmarks,
    /// Show downloaded files reported by browserd.
    BrowserDownloads,
    /// Show the current page's site permission state.
    BrowserSitePermissions,
    /// Open a history entry by its current visible index.
    BrowserHistoryItem(usize),
    /// Open a saved address by its index.
    BrowserBookmarkItem(usize),
    /// A tab in the browser's tab strip.
    BrowserTab(usize),
    /// A tab's close button.
    BrowserCloseTab(usize),
    /// The browser's omnibox go/commit area.
    BrowserGo,
    /// The omnibox text field (focus target).
    BrowserOmnibox,
    /// A Files hotspot (places, crumbs, headers, rows, nav buttons).
    Files(crate::apps::FilesHot),
    /// An icon on the desktop grid.
    DesktopIcon(usize),
    /// A context-menu row.
    MenuRow(crate::apps::MenuAction),
    /// One of the session rows in quick settings (suspend, restart, power off).
    QuickPowerAction(u8),
    /// A row in the Trash window.
    TrashRow(usize),
    /// The Trash window's Empty button.
    TrashEmpty,
    /// The Trash window's Restore button.
    TrashRestore,
    /// The Trash window's permanently delete selected entry action.
    TrashPurge,
    /// Confirm a pending permanent Trash deletion.
    TrashConfirm,
    /// Cancel a pending permanent Trash deletion.
    TrashCancel,
    /// The Trash search field.
    TrashFilter,
    /// The Activity process search field.
    ActivityFilter,
    /// Sort the Activity table by a visible column.
    ActivitySort(crate::taskmgr::Column),
    /// The lock screen's full-surface catcher: a click wakes the password
    /// field.
    LockWake,
    /// The lock screen's password field.
    LockEntry,
    /// The lock screen's submit button.
    LockSubmit,
    /// The lock screen's customize button.
    LockCustomize,
    /// A tab in the lock screen's customize sheet.
    LockTab(u8),
    /// A preset button in the customize sheet: `(group, option)`.
    LockOption(u8, u8),
    /// A switch in the customize sheet.
    LockSwitch(u8),
    /// Reset the customize sheet to the shipped look.
    LockReset,
    /// Close the customize sheet.
    LockDone,
    /// A category row in Settings.
    SettingsCategory(u8),
    /// The Settings search field.
    SettingsSearch,
    /// Show or hide the rows only an advanced reader wants.
    SettingsAdvanced,
    /// Put the shown category (or the whole registry, while searching) back to
    /// its defaults.
    SettingsReset,
    /// Write the whole settings store to the export file as text.
    SettingsExport,
    /// Read the export file back into the settings store, validated first.
    SettingsImport,
    /// One setting row, by its index in the visible list.
    SettingsRow(u16),
    /// A number row's decrease (`false`) or increase (`true`) button.
    SettingsStep(u16, bool),
    /// Change the shell interface language.
    SettingsLanguage,
    /// Cycle the supported colour themes.
    SettingsTheme,
    /// Toggle reduced motion in the shell.
    SettingsMotion,
    /// The dock's launcher button: expands the dock into the launcher.
    DockToggle,
    /// The dock's search bubble.
    DockSearch,
    /// Collapse the expanded dock back to its resting pill.
    DockCollapse,
    /// The lock button at the top of the expanded launcher.
    DockLock,
    /// A dialog's confirm button.
    DialogConfirm,
    /// A dialog's cancel button.
    DialogCancel,
    /// A notification.
    NotificationItem(usize),
    /// Dismiss-all in the notification list.
    NotificationsClear,
    /// A control in the boot flow: the first-run wizard's buttons and
    /// language cards, hit-tested like any other hotspot.
    FirstRun(crate::firstrun::Action),
    /// The wallpaper.
    Desktop,
}

impl Hot {
    /// Whether this hotspot is a useful stop in the shared keyboard focus
    /// order. Full-surface catchers and window bodies are deliberately skipped.
    #[must_use]
    pub const fn keyboard_focusable(self) -> bool {
        !matches!(self, Self::Window(_) | Self::Desktop | Self::LockWake)
    }
}

/// How important a notification is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Background information.
    Low,
    /// The default.
    Normal,
    /// Needs attention soon.
    High,
    /// Something is wrong now. Never auto-dismissed.
    Critical,
}

/// One notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// Identity.
    pub id: u32,
    /// Who sent it.
    pub source: String,
    /// One line.
    pub title: String,
    /// More detail.
    pub body: String,
    /// How important.
    pub priority: Priority,
    /// When it arrived, in milliseconds since the epoch.
    pub at_ms: u64,
    /// When it should disappear on its own, or `None` for persistent.
    pub expires_ms: Option<u64>,
}

/// The notification queue.
#[derive(Debug, Clone, Default)]
pub struct Notifications {
    items: VecDeque<Notification>,
    next_id: u32,
    /// Whether Do Not Disturb is on.
    pub do_not_disturb: bool,
}

impl Notifications {
    /// The most a queue holds before the oldest are dropped.
    ///
    /// A queue that grows without bound is a memory leak with a user interface.
    pub const CAPACITY: usize = 64;

    /// Post a notification. Returns its id.
    pub fn post(
        &mut self,
        source: &str,
        title: &str,
        body: &str,
        priority: Priority,
        now_ms: u64,
        ttl_ms: Option<u64>,
    ) -> u32 {
        let id = self.next_id.saturating_add(1);
        self.next_id = id;
        // Critical notifications are never given an expiry, whatever the
        // caller asked for: something that is wrong now does not stop being
        // wrong because five seconds passed.
        let expires_ms = if priority == Priority::Critical {
            None
        } else {
            ttl_ms.map(|t| now_ms.saturating_add(t))
        };
        self.items.push_front(Notification {
            id,
            source: source.to_owned(),
            title: title.to_owned(),
            body: body.to_owned(),
            priority,
            at_ms: now_ms,
            expires_ms,
        });
        while self.items.len() > Self::CAPACITY {
            // Drop the oldest non-critical one rather than simply the oldest.
            let victim = self
                .items
                .iter()
                .rposition(|n| n.priority != Priority::Critical)
                .unwrap_or(self.items.len().saturating_sub(1));
            self.items.remove(victim);
        }
        id
    }

    /// Remove anything whose time is up. Returns how many went.
    pub fn expire(&mut self, now_ms: u64) -> usize {
        let before = self.items.len();
        self.items
            .retain(|n| n.expires_ms.is_none_or(|t| t > now_ms));
        before.saturating_sub(self.items.len())
    }

    /// When the next notification will expire on its own, if any will.
    #[must_use]
    pub fn next_expiry(&self) -> Option<u64> {
        self.items.iter().filter_map(|n| n.expires_ms).min()
    }

    /// Dismiss one.
    pub fn dismiss(&mut self, id: u32) -> bool {
        let before = self.items.len();
        self.items.retain(|n| n.id != id);
        before != self.items.len()
    }

    /// Dismiss everything that is allowed to be dismissed in bulk.
    pub fn clear(&mut self) -> usize {
        let before = self.items.len();
        self.items.retain(|n| n.priority == Priority::Critical);
        before.saturating_sub(self.items.len())
    }

    /// Newest first.
    #[must_use]
    pub fn items(&self) -> Vec<&Notification> {
        self.items.iter().collect()
    }

    /// How many are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The most urgent priority currently waiting.
    #[must_use]
    pub fn peak_priority(&self) -> Option<Priority> {
        self.items.iter().map(|n| n.priority).max()
    }

    /// Whether a banner should be shown for a new notification.
    ///
    /// Do Not Disturb silences everything except critical, which is the only
    /// setting a person can safely be given: a switch that also hides "your
    /// disk is full" is a switch that loses data.
    #[must_use]
    pub const fn should_banner(&self, priority: Priority) -> bool {
        !self.do_not_disturb || matches!(priority, Priority::Critical)
    }
}

/// A power profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PowerProfile {
    /// Longest battery life.
    Eco,
    /// The default.
    Balanced,
    /// Favour speed.
    Performance,
    /// Everything available.
    Maximum,
}

impl PowerProfile {
    /// Every profile, in order.
    pub const ALL: [Self; 4] = [Self::Eco, Self::Balanced, Self::Performance, Self::Maximum];

    /// The word the daemon's protocol uses.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Eco => "eco",
            Self::Balanced => "balanced",
            Self::Performance => "performance",
            Self::Maximum => "maximum",
        }
    }

    /// The request line that asks the machine for this profile.
    #[must_use]
    pub fn request_line(self) -> String {
        format!("POWER PROFILE {}", self.token())
    }

    /// The name shown to the user.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Eco => "Eco",
            Self::Balanced => "Balanced",
            Self::Performance => "Performance",
            Self::Maximum => "Maximum",
        }
    }
}

/// A machine transition the session can ask for.
///
/// Distinct from [`PowerProfile`], and deliberately not folded into it: a
/// profile is a *policy* the machine may or may not honour, and this is an
/// instruction to PID 1 that either happens or does not. They occupy different
/// places on screen for the same reason — a profile is a segmented control
/// that takes effect as you click it, and these are three rows behind a
/// confirmation that names the action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PowerCommand {
    /// Suspend to RAM. Resumable, and the only one of the three that is.
    Suspend,
    /// Restart.
    Restart,
    /// Power off.
    PowerOff,
}

impl PowerCommand {
    /// Every command, in menu order: the reversible one first, the one that
    /// leaves the machine off last.
    pub const ALL: [Self; 3] = [Self::Suspend, Self::Restart, Self::PowerOff];

    /// The name shown to the user.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Suspend => "Suspend",
            Self::Restart => "Restart",
            Self::PowerOff => "Power off",
        }
    }

    /// The word the daemon's protocol uses.
    ///
    /// `Restart` is `REBOOT` there because that is the name of the thing being
    /// asked for at that layer. The two vocabularies are translated here, once,
    /// rather than the shell building a request line out of two constants that
    /// happen to look alike today.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Suspend => "SUSPEND",
            Self::Restart => "REBOOT",
            Self::PowerOff => "POWEROFF",
        }
    }

    /// The question the confirm step asks.
    #[must_use]
    pub const fn question(self) -> &'static str {
        match self {
            Self::Suspend => "Suspend the machine?",
            Self::Restart => "Restart the machine?",
            Self::PowerOff => "Power the machine off?",
        }
    }

    /// What the user is about to lose, said plainly.
    #[must_use]
    pub const fn consequence(self) -> &'static str {
        match self {
            Self::Suspend => "This session stays open and the machine can be woken again.",
            Self::Restart => "Everything running stops, and the machine comes back up.",
            Self::PowerOff => "Everything running stops, and the machine stays off.",
        }
    }

    /// The request line the daemon answers.
    ///
    /// Built here and nowhere else, so the confirmation word exists in exactly
    /// one place in this process. A shell that assembled the string at each
    /// call site could send a request without it and believe it had confirmed
    /// something.
    #[must_use]
    pub fn request_line(self) -> String {
        format!("POWER {} CONFIRM", self.token())
    }
}

/// The state quick settings shows and changes.
///
/// Every field here is a **request**. Whether the system honoured it is a
/// separate question, and one this build cannot yet answer for most of them —
/// see [`QuickSettings::applied`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickSettings {
    /// Requested power profile.
    pub power: PowerProfile,
    /// Do Not Disturb.
    pub do_not_disturb: bool,
    /// Adaptive Idle: whether the compositor may reduce its frame rate.
    pub adaptive_idle: bool,
    /// Reduce transparency, for readability.
    pub reduce_transparency: bool,
    /// Reduce motion.
    pub reduce_motion: bool,
    /// Which of these have actually been applied to the system, rather than
    /// only recorded here.
    pub applied: Vec<String>,
}

impl Default for QuickSettings {
    fn default() -> Self {
        Self {
            power: PowerProfile::Balanced,
            do_not_disturb: false,
            adaptive_idle: true,
            reduce_transparency: false,
            reduce_motion: false,
            applied: vec![
                "adaptive_idle".to_owned(),
                "reduce_motion".to_owned(),
                "reduce_transparency".to_owned(),
                "do_not_disturb".to_owned(),
            ],
        }
    }
}

impl QuickSettings {
    /// Whether a control changes anything outside the shell.
    ///
    /// Used to mark the ones that do not, so a toggle that only records a
    /// preference is not presented as though it reconfigured the machine.
    #[must_use]
    pub fn is_applied(&self, key: &str) -> bool {
        self.applied.iter().any(|k| k == key)
    }
}

/// One environment as the desktop knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvRow {
    /// `windows`, `litric`, `linux`.
    pub token: String,
    /// The name shown.
    pub title: String,
    /// `running`, `suspended`, `hibernated`, `stopped`, `not-installed`.
    pub state: String,
    /// Whether it is the active one.
    pub active: bool,
    /// The shortcut that switches to it.
    pub hotkey: String,
}

/// The numbers the desktop shows, and where they came from.
///
/// Every field is an `Option`. A machine that cannot report its GPU load shows
/// "Unavailable", not zero.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SystemFacts {
    /// Total RAM, MiB.
    pub mem_total_mb: Option<u64>,
    /// RAM in use, MiB.
    pub mem_used_mb: Option<u64>,
    /// CPU utilisation between the last two `/proc/stat` samples, 0.0 to 1.0.
    /// `None` until there have been two samples.
    pub cpu_fraction: Option<f32>,
    /// Online CPUs.
    pub cpus: Option<u32>,
    /// Seconds since boot.
    pub uptime_secs: Option<f64>,
    /// Root filesystem type.
    pub root_fs: Option<String>,
    /// Whether dm-verity is protecting the root.
    pub verity: Option<bool>,
    /// Items in the trash and their size.
    pub trash: Option<(usize, u64)>,
    /// GPU utilisation from the DRM driver's `gpu_busy_percent`, 0.0 to 1.0.
    /// `None` when the driver does not publish one, which most do not.
    pub gpu_fraction: Option<f32>,
    /// The kernel driver bound to the DRM card, e.g. `bochs-drm`, so a GPU
    /// that cannot be measured is still named.
    pub gpu_name: Option<String>,
    /// Power draw in watts. Never populated yet.
    pub power_watts: Option<f32>,
    /// Package temperature in Celsius. Never populated yet.
    pub temperature_c: Option<f32>,
    /// Battery percentage, when there is a battery.
    pub battery_percent: Option<u8>,
    /// Whether the battery is charging.
    pub battery_charging: Option<bool>,
    /// Network state: `wifi`, `ethernet`, `offline`.
    pub network: Option<String>,
    /// Bytes received on the machine's interfaces, cumulative, as the daemon
    /// reports them (`net.iface ... rx=`). Loopback is left out: traffic that
    /// never left the machine is not what a person watching a rate graph is
    /// watching.
    pub net_rx_bytes: Option<u64>,
    /// Bytes transmitted, cumulative, on the same terms as
    /// [`SystemFacts::net_rx_bytes`].
    pub net_tx_bytes: Option<u64>,
    /// Receive rate between the last two samples, bytes per second. `None`
    /// until two samples a known interval apart exist, which is the same rule
    /// the CPU's live figure follows.
    pub net_rx_per_s: Option<f64>,
    /// Transmit rate, bytes per second, on the same terms.
    pub net_tx_per_s: Option<f64>,
    /// When this sample was taken, on the shell's own clock, in milliseconds.
    ///
    /// Nothing else in the facts needs it: every other rate is a fraction of
    /// the machine's own counters, which need no wall clock. A byte rate does,
    /// so the time is carried from the one place that has a clock -- the
    /// sample's entry to the shell -- rather than guessed from the frame
    /// interval or assumed to be one second.
    pub sampled_ms: Option<u64>,
    /// The CPU's model name, as the running kernel reports it. `None` when the
    /// kernel published none: the interface then shows no model rather than a
    /// name from a table compiled into the shell.
    pub cpu_model: Option<String>,
    /// Cumulative CPU tick counters (busy, total) from the last sample. Kept
    /// only so the next sample can be differenced: a single cumulative counter
    /// cannot say what the machine is doing now.
    pub cpu_ticks: Option<(u64, u64)>,
    /// Cumulative per-logical-CPU tick counters (busy, total) from the last
    /// sample, in kernel order.
    pub core_ticks: Vec<(u64, u64)>,
    /// CPU utilisation between the last two samples, 0.0 to 1.0. Empty until a
    /// second sample exists.
    pub cpu_cores: Vec<f32>,
    /// Whole-machine utilisation between the last two samples, 0.0 to 1.0.
    /// `None` until a second sample exists, in which case `cpu_fraction` (the
    /// average since boot) is what the interface falls back to, and says so.
    pub cpu_now: Option<f32>,
    /// RAM a new allocation could get, MiB. This, not free memory, is the
    /// number that means anything on a machine with a page cache.
    pub mem_available_mb: Option<u64>,
    /// Page cache and buffers, MiB.
    pub mem_cached_mb: Option<u64>,
    /// Swap in use, MiB.
    pub swap_used_mb: Option<u64>,
    /// One-minute load average.
    pub load1: Option<f32>,
    /// Processes and threads the kernel is tracking.
    pub tasks: Option<u32>,
    /// Resident set of the system daemon, MiB -- the daemon's own footprint,
    /// which a monitor that reports everyone else's should report.
    pub sysd_rss_mb: Option<u64>,
    /// Whether the multi-generational LRU is active.
    pub mglru: Option<bool>,
    /// Mounted volumes as (mount point, filesystem, read-only).
    pub volumes: Vec<(String, String, bool)>,
    /// The process table. Empty when nothing reported one, and the monitor says
    /// so rather than drawing an empty table that reads as "no processes".
    pub processes: Vec<ProcRow>,
    /// Whether the daemon's process list was cut short by its own bound.
    pub processes_truncated: bool,
    /// Where these numbers came from.
    pub source: String,
    /// What can verify a password on this machine: `unix-chkpwd`, `pam-helper`
    /// or `none` (or absent, when the daemon did not answer at all).
    ///
    /// The token is carried as the daemon spelled it rather than as a booleans,
    /// so that the shell and the daemon cannot disagree about *why* a lock is
    /// refused: [`Shell::can_lock`] asks the same parser the daemon builds the
    /// fact with.
    pub auth_available: Option<String>,
    /// Whether the daemon says this boot still needs the setup wizard
    /// (`first.run=yes`): the account has no password and no language has been
    /// chosen. Absent means no wizard — the splash alone is the safe default
    /// for every machine that has already been set up.
    pub first_run: Option<bool>,
    /// Whether the daemon says locking is refused (`auth.lock=refused`).
    ///
    /// `None` when the line was not in the reply at all, which is what a
    /// daemon that did not answer looks like. Absent and "not refused" are the
    /// same thing to a lock screen: without a verifier there is nothing to
    /// unlock with, so [`Shell::can_lock`] refuses in both cases.
    pub auth_lock: Option<bool>,
    /// The user logged in on this machine, from logind's session record.
    pub session_user: Option<String>,
    /// Keyboards, pointers and tablets the kernel is actually driving.
    ///
    /// Empty means the platform reported none, which on a desktop is a finding
    /// worth showing: a machine with no keyboard attached and one whose
    /// keyboard failed to probe both deliver no keystrokes, and Settings has to
    /// be able to say which of the two it is looking at.
    pub input_devices: Vec<InputDevice>,
    /// Network interfaces, loopback included: a panel that hides `lo` is a
    /// panel that cannot explain why the daemon is reachable on a machine with
    /// no cable in it.
    pub net_ifaces: Vec<NetIface>,
    /// The Bluetooth verdict from the platform: `no-adapter`, `blocked`,
    /// `no-daemon`, `ready` or `unknown`.
    pub bt_state: Option<String>,
    /// Bluetooth adapters, as `hci0 00:1a:7d:da:71:13`.
    pub bt_adapters: Vec<String>,
    /// Whether a Bluetooth daemon is installed.
    pub bt_daemon: Option<bool>,
    /// Sound cards, as `card0 HDA Intel PCH`.
    pub audio_cards: Vec<String>,
    /// Whether a sound server is running, as the platform reports it.
    pub audio_server: Option<String>,
}

/// One input node, as Settings shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDevice {
    /// `/dev/input` node name, e.g. `event3`.
    pub node: String,
    /// The kernel's name for the device.
    pub name: String,
    /// `keyboard`, `mouse`, `tablet` or `other`.
    pub kind: String,
}

/// One network interface, as Settings shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetIface {
    /// Kernel name, e.g. `eth0`.
    pub name: String,
    /// `up`, `down` or `unknown`.
    pub state: String,
    /// The address, or `-` when the driver published none.
    pub mac: String,
    /// Whether the driver has a wireless directory.
    pub wireless: bool,
}

/// One process as the system monitor shows it.
///
/// The counters are cumulative, as `/proc` reports them, so two samples can be
/// differenced by [`SystemFacts::observe`]; `cpu` is the result of that
/// subtraction and stays `None` until there are two samples to subtract. A
/// monitor whose first frame shows every process at 0% has invented a number;
/// "not measured yet" is the honest alternative.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcRow {
    /// Process id.
    pub pid: u32,
    /// What to call it: the command line's program name, or the kernel's name
    /// for a thread that has no command line.
    pub name: String,
    /// The full command line, for the details pane.
    pub cmdline: String,
    /// Run state, spelled out.
    pub state: String,
    /// Parent pid.
    pub ppid: u32,
    /// Live threads.
    pub threads: u32,
    /// Cumulative CPU ticks (user + system).
    pub ticks: u64,
    /// Resident set in mebibytes, when it was readable.
    pub rss_mb: Option<u64>,
    /// Real uid, when it was readable.
    pub uid: Option<u32>,
    /// Share of the whole machine over the last sample, 0.0 to 1.0.
    pub cpu: Option<f32>,
}

impl ProcRow {
    /// The share as a percentage string, or the dash for "not measured yet".
    #[must_use]
    pub fn cpu_pct(&self) -> String {
        self.cpu
            .map_or_else(|| "\u{2014}".to_owned(), |f| format!("{:.1}", f * 100.0))
    }

    /// Memory as a string, with the dash for "not measured".
    #[must_use]
    pub fn rss_text(&self) -> String {
        self.rss_mb
            .map_or_else(|| "\u{2014}".to_owned(), |mb| format!("{mb} MB"))
    }
}

impl SystemFacts {
    /// Take a new sample, turning cumulative counters into live rates.
    ///
    /// The counters in a `/proc`-style interface only ever increase, so the
    /// only honest way to say what a CPU is doing *now* is to subtract the
    /// previous sample. That subtraction lives here rather than in the reader
    /// because it is the one part of fact-gathering that has state, and a
    /// version of it that can be called twice with two synthetic samples is a
    /// version that can be tested on a host with no `/proc` at all.
    ///
    /// A counter that went backwards (a machine that rebooted between samples)
    /// produces nothing rather than a negative or an enormous rate.
    pub fn observe(&mut self, mut fresh: Self) {
        fresh.cpu_now = tick_fraction(self.cpu_ticks, fresh.cpu_ticks);
        fresh.cpu_cores = fresh
            .core_ticks
            .iter()
            .zip(self.core_ticks.iter())
            .filter_map(|(now, then)| tick_fraction(Some(*then), Some(*now)))
            .collect();
        // Byte rates need three things: two counters and the time between them.
        // A counter that went backwards (an interface reset, a machine that
        // rebooted) produces nothing rather than a negative rate.
        let seconds = match (self.sampled_ms, fresh.sampled_ms) {
            (Some(then), Some(now)) => now
                .checked_sub(then)
                // Zero elapsed time is not a rate of zero, it is no measurement:
                // dividing by it is how a shell reports an interface carrying
                // infinity bytes a second.
                .filter(|ms| *ms > 0)
                .map(|ms| ms as f64 / 1000.0),
            _ => None,
        };
        fresh.net_rx_per_s =
            seconds.and_then(|s| byte_rate(self.net_rx_bytes, fresh.net_rx_bytes, s));
        fresh.net_tx_per_s =
            seconds.and_then(|s| byte_rate(self.net_tx_bytes, fresh.net_tx_bytes, s));
        // The process table's rates come from the same subtraction, against the
        // machine's own tick delta so a share reads as "of this machine" rather
        // than "of one core": eight cores fully busy is 1.0, not 8.0.
        let machine_ticks = self
            .cpu_ticks
            .zip(fresh.cpu_ticks)
            .and_then(|((_, then_total), (_, now_total))| now_total.checked_sub(then_total));
        if let Some(delta) = machine_ticks.filter(|d| *d > 0) {
            for now in &mut fresh.processes {
                let before = self
                    .processes
                    .iter()
                    .find(|p| p.pid == now.pid)
                    .map(|p| p.ticks);
                now.cpu = before
                    .and_then(|then| now.ticks.checked_sub(then))
                    .map(|d| (d as f64 / delta as f64).min(1.0) as f32);
            }
        }
        *self = fresh;
    }
}

/// Busy-vs-total ratio between two counter samples, 0.0 to 1.0.
///
/// `None` when either sample is missing, when no time passed between them, or
/// when a counter moved backwards -- each of which would otherwise produce a
/// plausible-looking number that the machine never reported.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn tick_fraction(then: Option<(u64, u64)>, now: Option<(u64, u64)>) -> Option<f32> {
    let (then_busy, then_total) = then?;
    let (now_busy, now_total) = now?;
    let elapsed = now_total.checked_sub(then_total)?;
    if elapsed == 0 {
        return None;
    }
    let busy = now_busy.saturating_sub(then_busy).min(elapsed);
    Some((busy as f64 / elapsed as f64) as f32)
}

/// `(now - then) / seconds`, or `None` when the counters cannot support a
/// rate. The `None` cases are the point: a first sample has nothing to subtract
/// from, and a counter that went backwards is an interface that reset, not a
/// machine receiving negative bytes.
#[must_use]
#[allow(clippy::cast_precision_loss)]
fn byte_rate(then: Option<u64>, now: Option<u64>, seconds: f64) -> Option<f64> {
    let (then, now) = (then?, now?);
    if seconds <= 0.0 {
        return None;
    }
    let delta = now.checked_sub(then)?;
    Some(delta as f64 / seconds)
}

impl SystemFacts {
    /// A value, or the word for "we do not know".
    #[must_use]
    pub fn or_unknown<T: std::fmt::Display>(v: Option<T>) -> String {
        v.map_or_else(|| "Unavailable".to_owned(), |x| x.to_string())
    }
}

impl Shell {
    /// What [`SystemFacts::source`] says when the daemon itself answered.
    ///
    /// The boot splash reads this as its readiness: a reply on the fact bus
    /// means the services a desktop talks to are up far enough to be shown.
    pub const SOURCE_DAEMON: &str = "litric-sysd";

    /// Take one sample of the machine.
    ///
    /// The process table's rates are differenced against the previous sample, so
    /// the monitor has to see every sample the shell does; doing it in one place
    /// is what makes that true by construction rather than by remembering.
    pub fn sample(&mut self, fresh: SystemFacts) {
        // The shell owns the clock, so the sample carries its time: a byte rate
        // cannot be computed from counters alone. A caller that already stamped
        // the facts -- a replay, a test -- keeps its own stamp.
        let daemon_answered = fresh.source == Self::SOURCE_DAEMON;
        let first_run_answer = fresh.first_run;
        let mut fresh = fresh;
        if fresh.sampled_ms.is_none() {
            fresh.sampled_ms = Some(self.now_ms);
        }
        self.facts.observe(fresh);
        if daemon_answered {
            if let Some(fr) = &mut self.first_run {
                fr.system_ready = true;
            }
        }
        // Whether this machine needs setup is a question the boot flow is
        // still allowed to be waiting on; a sample that answers it ends the
        // wait, whatever kind of sample it was.
        if let (Some(fr), Some(answer)) = (&mut self.first_run, first_run_answer) {
            if fr.wizard.is_none() {
                fr.wizard = Some(answer);
            }
        }
        self.activity.observe(&self.facts);

        // A battery warning that fired once stays fired until the battery has
        // actually recovered, so a machine sitting at 14% does not post the
        // same notification every second.
        if let Some(percent) = self.facts.battery_percent {
            let warn_at = self.settings.values.num("power.battery-warn") as u8;
            let charging = self.facts.battery_charging == Some(true);
            if percent <= warn_at && !charging && !self.battery_warned {
                self.battery_warned = true;
                let title = format!("Battery at {percent}%");
                self.notify(
                    "Power",
                    &title,
                    "Plug in, or the machine will shut down without warning.",
                    Priority::High,
                    Some(8_000),
                );
            } else if percent > warn_at.saturating_add(4) {
                self.battery_warned = false;
            }
        }
    }
}

/// Something `main` has to do that the shell cannot do itself.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Effect {
    /// Ask the environment manager to switch.
    SwitchEnvironment(u8),
    /// Lock the session.
    ///
    /// The caller locks the shell only when [`Shell::can_lock`] is true — a
    /// machine that can check a password *and* a daemon that did not refuse —
    /// and otherwise shows [`Shell::lock_refusal`] as it stands.
    RequestLock,
    /// Persist a shell-owned preference after applying it optimistically.
    PersistSettings(crate::settings::Settings),
    /// Write the whole settings store to the export file, as the text
    /// [`crate::settings::Model::import`] reads back.
    ExportSettings(crate::settings::Settings),
    /// Read the export file and apply it. The reader is in `main` because the
    /// file is on disk and the shell's own reach ends at its state directory.
    ImportSettings,
    /// Persist Browser history and bookmarks after a user action.
    PersistBrowserData,
    /// Check this password, and unlock if it verifies.
    ///
    /// The secret leaves the shell here and nowhere else. It is a [`Secret`] so
    /// that a diagnostic that prints an effect cannot print the password, and
    /// the caller that receives it builds the daemon's request from it directly
    /// rather than storing it.
    VerifyPassword {
        /// The account, from the session logind reports.
        user: String,
        /// What the user typed.
        secret: Secret,
    },
    /// Set this account's password without knowing the old one — the first-run
    /// wizard's only way to leave a mark on the machine. It is a [`Secret`] for
    /// the same reason [`Effect::VerifyPassword`] is: an effect that can be
    /// printed must never print a password.
    SetPassword {
        /// The account, named by the wizard.
        user: String,
        /// What the user chose.
        secret: Secret,
    },
    /// Adjust the volume by one notch (five percent). Zero toggles mute: the
    /// shell holds no mixer state, so it means "flip it" rather than guessing.
    Volume(i8),
    /// Adjust the screen brightness by one notch (five percent of the
    /// backlight's range).
    Brightness(i8),
    /// A power profile was requested.
    SetPowerProfile(PowerProfile),
    /// A power transition was confirmed by the user and must be carried out.
    ///
    /// This effect only exists after the confirm step has been answered, so the
    /// caller that receives it is the one that asks the daemon — nothing else
    /// in the shell can produce it.
    Power(PowerCommand),
    /// A window was opened for an application the shell does not implement.
    Launched(App),
}

/// The Defender's runtime state: the model the window shows plus the
/// background worker that feeds it. The scan runs on a thread because a scan
/// of the home folders reads every byte under them; blocking the compositor
/// for that would freeze the desktop exactly when the user is watching it.
#[derive(Debug, Default)]
pub struct DefenderState {
    /// What the window shows.
    pub app: crate::apps::DefenderApp,
    /// Queue of logical paths still to scan in this run.
    pub queue: Vec<String>,
    /// Path being scanned right now, for the status line.
    pub current: Option<String>,
    /// Whether a scan is running (worker alive).
    pub running: bool,
}

/// One result the scan worker produced.
#[derive(Debug)]
pub struct ScanResult {
    /// The logical path.
    pub path: String,
    /// The engine's report.
    pub report: litric_defender::engine::ScanReport,
}

/// Shared between the shell and the scan worker thread.
#[derive(Debug, Default)]
pub struct ScanShared {
    /// Results waiting for the shell to collect.
    pub results: Mutex<Vec<ScanResult>>,
    /// The worker sets this when the queue is exhausted.
    pub done: AtomicBool,
}

impl Shell {
    /// Queue the standard scan: the home folders a user actually keeps
    /// documents in. Depth and count caps live in the walker, not here.
    pub fn start_scan(&mut self) -> Vec<Effect> {
        if self.defender.running {
            return Vec::new();
        }
        let zone = self.regional.zone.clone();
        let mut queue = Vec::new();
        {
            let Ok(bridge) = self.try_files_bridge(zone) else {
                self.defender.app.message =
                    "the file service is not available on this platform".to_owned();
                return Vec::new();
            };
            for dir in [
                "/Documents",
                "/Downloads",
                "/Pictures",
                "/Music",
                "/Videos",
                "/Projects",
            ] {
                if let Ok(entries) = bridge.list_dir(dir) {
                    for e in entries {
                        let p = format!("{}/{}", dir.trim_end_matches('/'), e.name);
                        if e.kind == crate::apps::FileKind::Dir {
                            collect_files(&bridge, &p, &mut queue, 0);
                        } else if e.kind == crate::apps::FileKind::File {
                            queue.push(p);
                        }
                    }
                }
            }
        }
        let total = queue.len();
        self.defender.app.start(total);
        self.defender.queue = queue;
        self.defender.running = true;
        self.defender.current = None;
        self.request_full_redraw();
        Vec::new()
    }

    /// The bridge the shell's file surfaces share, rooted at [`Self::files_root`].
    /// On a host with no root filesystem (Windows CI) there is no bridge;
    /// callers show the refusal in place rather than panicking in the render
    /// loop.
    fn try_files_bridge(
        &self,
        zone: litric_locale::TimeZone,
    ) -> Result<crate::fsbridge::FsBridge, crate::fsbridge::FsError> {
        crate::fsbridge::FsBridge::open(&self.files_root, zone)
    }

    /// Scan a bounded slice of the queue synchronously. Called between
    /// frames, so a scan never costs the desktop more than a slice of one
    /// frame per tick; the budget keeps 60 fps honest on a big folder.
    pub fn scan_step(&mut self, budget: usize) {
        if !self.defender.running {
            return;
        }
        let zone = self.regional.zone.clone();
        let Ok(bridge) = self.try_files_bridge(zone) else {
            self.defender.running = false;
            self.defender.app.message = "the file service is not available".to_owned();
            return;
        };
        let mut engine = litric_defender::engine::Engine::new();
        engine.load_denylist(eicar_denylist());
        let mut done = 0usize;
        while done < budget {
            if self.defender.current.is_none() {
                self.defender.current = self.defender.queue.pop();
                if self.defender.current.is_none() {
                    self.defender.app.finish();
                    self.defender.running = false;
                    self.notify_defender();
                    self.request_full_redraw();
                    return;
                }
            }
            let Some(path) = self.defender.current.take() else {
                return;
            };
            let host = bridge.host_path(&path).unwrap_or_default();
            let report = match std::fs::read(&host) {
                Ok(bytes) => {
                    let mtime = std::fs::metadata(&host)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    engine.scan(&path, bytes.len() as u64, mtime, &bytes)
                }
                Err(e) => litric_defender::engine::ScanReport {
                    decision: litric_defender::verdict::Decision::AllowUnknown {
                        reason: format!("read: {e}"),
                    },
                    digest_hex: String::new(),
                    size: 0,
                    from_cache: false,
                },
            };
            self.defender.app.record(&path, &report.decision);
            done = done.saturating_add(1);
        }
        self.request_full_redraw();
    }

    /// Post the end-of-scan notification, with the banner the user asked for.
    fn notify_defender(&mut self) {
        let msg = self.defender.app.message.clone();
        let threats = self.defender.app.rows.iter().filter(|r| r.detected).count();
        let (title, body, priority) = if threats > 0 {
            (
                "Litric Defender".to_owned(),
                msg.clone(),
                Priority::Critical,
            )
        } else {
            ("Litric Defender".to_owned(), msg.clone(), Priority::Normal)
        };
        self.notify("Defender", &title, &body, priority, Some(8_000));
    }

    /// Drain the browser link's inbox: frames become the pending framebuffer,
    /// closed links are torn down and the page area explains why.
    pub fn pump_browser(&mut self) {
        use crate::link::Incoming;
        let Some(link) = &self.browser_session else {
            return;
        };
        let messages = link.drain();
        if messages.is_empty() {
            return;
        }
        let mut closed = false;
        for m in messages {
            match m {
                Incoming::Payload { header, bytes } => {
                    if header.starts_with("FRAME ") {
                        if let Some(rest) = header.strip_prefix("FRAME ") {
                            if let Some((w, h)) = parse_frame_dims(rest) {
                                let expected = u64::from(w)
                                    .checked_mul(u64::from(h))
                                    .and_then(|n| n.checked_mul(4))
                                    .and_then(|n| usize::try_from(n).ok());
                                if expected.is_some_and(|n| n == bytes.len()) {
                                    self.browser_frame = Some(Arc::from(bytes.into_boxed_slice()));
                                    self.browser.last_frame = Some((w, h));
                                    self.request_full_redraw();
                                }
                            }
                        }
                    }
                }
                Incoming::Line(line) => {
                    if let Some(rest) = line.strip_prefix("TITLE ") {
                        if let Some(t) = self.browser.tabs.get_mut(self.browser.active) {
                            t.title = rest.to_owned();
                        }
                    } else if line.starts_with("ERR ") {
                        if let Some(t) = self.browser.tabs.get_mut(self.browser.active) {
                            t.error = true;
                        }
                        self.browser.status = line;
                    } else if line.starts_with("DOWNLOADS ") {
                        let total = line
                            .split_whitespace()
                            .nth(1)
                            .and_then(|count| count.parse::<usize>().ok())
                            .unwrap_or(0);
                        self.browser.downloads_status = Some(format!("{total} file(s)"));
                    } else if let Some(rest) = line.strip_prefix("DOWNLOAD ") {
                        let mut parts = rest.split_whitespace();
                        let name = parts
                            .next()
                            .and_then(|field| litric_core::pct::decode(field).ok())
                            .unwrap_or_default();
                        let size = parts
                            .next()
                            .and_then(|value| value.parse().ok())
                            .unwrap_or(0);
                        if !name.is_empty() {
                            self.browser
                                .downloads
                                .push(crate::apps::BrowserDownload { name, size });
                        }
                    }
                }
                Incoming::Closed(_) => closed = true,
            }
        }
        if closed {
            self.browser_session = None;
            self.browser.status = "engine session ended".to_owned();
        }
    }

    /// Start (or reuse) the browser engine session over its socket.
    pub fn browser_start_session(&mut self) -> Vec<Effect> {
        if self.browser_session.is_some() {
            return Vec::new();
        }
        let framing = crate::link::Framing {
            max_line: 256,
            payload_len: browser_frame_len,
            coalesce_key: |h| {
                h.strip_prefix("FRAME ")
                    .and_then(|r| r.split(' ').next())
                    .map(str::to_owned)
            },
        };
        match crate::link::Link::connect(
            std::path::Path::new("/run/litric/browser.sock"),
            framing,
            Box::new(|| {}),
        ) {
            Ok(link) => {
                self.browser_session = Some(link);
                self.browser.status = "engine session started".to_owned();
                // The session exists to show something: navigate the active
                // tab right away, the way a browser loads its start page.
                if let Some(url) = self.browser.navigate_active() {
                    self.send_go(&url);
                }
                Vec::new()
            }
            Err(e) => {
                self.browser.status = format!(
                    "engine not reachable ({e}); start litric-browserd, or use the menu's Start"
                );
                self.request_full_redraw();
                Vec::new()
            }
        }
    }

    /// Send one `GO <tab> <url>` to the engine session.
    ///
    /// This is the line a search or an address commit has to produce; a
    /// navigation that stops at the model never lights a pixel of the web.
    fn send_go(&mut self, url: &str) {
        let tab = self.browser.active.saturating_add(1);
        let pct = litric_core::pct::encode(url);
        if let Some(link) = self.browser_session.as_mut() {
            match link.send_line(&format!("GO {tab} {pct}")) {
                Ok(()) => self.browser.status = format!("loading {url}"),
                Err(e) => self.browser.status = format!("send failed: {e}"),
            }
        }
    }

    /// Navigate to a URL: start the engine session if needed, then GO.
    pub fn browser_navigate(&mut self, url: &str) {
        if self.browser_session.is_none() {
            self.browser_start_session();
            // A successful start already navigated the active tab; a failed
            // one left its reason in the status line.
            return;
        }
        self.send_go(url);
        self.request_full_redraw();
    }

    /// A dock click: open the app, focus it if backgrounded, minimize it if
    /// it is already the focused one — the macOS dock's open/focus/hide
    /// rhythm on one button.
    fn dock_click(&mut self, app: App) -> Vec<Effect> {
        let open = self
            .wm
            .all()
            .iter()
            .find(|w| w.app_id == app.app_id())
            .map(|w| (w.id, Some(w.id) == self.wm.focused()));
        match open {
            Some((id, true)) => {
                self.wm.minimize(id);
                Vec::new()
            }
            _ => self.open_app(app),
        }
    }

    /// Run one terminal command for real on the guest: `/bin/sh -c` under
    /// `timeout`, output streamed back through a channel that `tick`
    /// drains, so a slow or runaway command can never stall a frame.
    pub fn terminal_run(&mut self) {
        let cmd = self.terminal.input.trim().to_owned();
        self.terminal.input.clear();
        if cmd.is_empty() {
            return;
        }
        #[cfg(unix)]
        {
            if self.terminal_job.is_some() {
                self.terminal
                    .push_line("litric: a command is still running");
                self.terminal.input = cmd;
                self.request_full_redraw();
                return;
            }
            self.terminal.remember_command(&cmd);
            self.terminal.echo_command(&cmd);
            let (tx, rx) = std::sync::mpsc::channel::<String>();
            self.terminal.busy = true;
            self.terminal_job = Some(rx);
            let spawn = std::thread::Builder::new()
                .name("litric-terminal".to_owned())
                .spawn(move || {
                    let out = std::process::Command::new("timeout")
                        .args(["30", "/bin/sh", "-c", &cmd])
                        .output();
                    match out {
                        Ok(o) => {
                            let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
                            text.push_str(&String::from_utf8_lossy(&o.stderr));
                            for line in text.lines().take(300) {
                                if tx.send(line.to_owned()).is_err() {
                                    break;
                                }
                            }
                            if !o.status.success() {
                                let _ =
                                    tx.send(format!("[exit {:?}]", o.status.code().unwrap_or(-1)));
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(format!("litric: cannot run: {e}"));
                        }
                    }
                    let _ = tx.send("\u{0}DONE".to_owned());
                });
            if spawn.is_err() {
                self.terminal.busy = false;
                self.terminal_job = None;
                self.terminal
                    .push_line("litric: cannot start a worker thread");
            }
        }
        #[cfg(not(unix))]
        {
            self.terminal.remember_command(&cmd);
            self.terminal.echo_command(&cmd);
            self.terminal.note =
                Some("การรันคำสั่ง Linux ต้องอยู่บน LitricOS เครื่องจริง — บิลด์นี้รันบนโฮสต์ Windows".to_owned());
        }
        self.request_full_redraw();
    }

    /// Drain the terminal job's output into the scrollback. Called from
    /// `tick`; returns whether anything new arrived (a repaint is owed).
    fn terminal_pump(&mut self) -> bool {
        #[cfg(unix)]
        {
            let Some(rx) = self.terminal_job.as_mut() else {
                return false;
            };
            let mut got = false;
            loop {
                match rx.try_recv() {
                    Ok(line) if line == "\u{0}DONE" => {
                        self.terminal.busy = false;
                        self.terminal_job = None;
                        got = true;
                        break;
                    }
                    Ok(line) => {
                        self.terminal.push_line(line);
                        got = true;
                    }
                    Err(_) => break,
                }
            }
            got
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    /// Execute one filesystem operation from a confirmed dialog.
    fn perform_file_op(&mut self, op: crate::apps::FilesOp) {
        use crate::apps::FilesOp;
        let zone = self.regional.zone.clone();
        let Ok(bridge) = self.try_files_bridge(zone) else {
            self.files.dialog = Some(crate::apps::DialogKind::Error(
                "the file service is not available on this platform".to_owned(),
            ));
            return;
        };
        let out: Result<String, crate::fsbridge::FsError> = match op {
            FilesOp::Trash(p) => {
                let now = u64::try_from(self.clock_seconds().max(0)).unwrap_or(0);
                bridge
                    .trash(&p, now)
                    .map(|_| format!("Moved to trash: {p}"))
            }
            FilesOp::TrashMany(paths) => {
                let now = u64::try_from(self.clock_seconds().max(0)).unwrap_or(0);
                paths
                    .iter()
                    .try_for_each(|path| bridge.trash(path, now))
                    .map(|()| format!("Moved {} selected items to trash", paths.len()))
            }
            FilesOp::Delete(p) => bridge.delete(&p).map(|_| format!("Deleted: {p}")),
            FilesOp::DeleteMany(paths) => paths
                .iter()
                .try_for_each(|path| bridge.delete(path))
                .map(|()| format!("Permanently deleted {} selected items", paths.len())),
            FilesOp::PasteCopy => match self.files.clipboard.clone() {
                Some((false, p)) => bridge
                    .copy(&p, &self.files.path)
                    .map(|n| format!("Copied to {} as {n}", self.files.path)),
                _ => Err(crate::fsbridge::FsError::NotFound),
            },
            FilesOp::PasteCut => match self.files.clipboard.clone() {
                Some((true, p)) => bridge
                    .move_item(&p, &self.files.path)
                    .map(|n| format!("Moved to {} as {n}", self.files.path)),
                _ => Err(crate::fsbridge::FsError::NotFound),
            },
            FilesOp::Compress(name) => {
                let src = self.files.selected_path();
                match src {
                    Some(p) => {
                        let dest = format!("{}/{}", self.files.path.trim_end_matches('/'), name);
                        bridge
                            .compress(&dest, &[p.as_str().to_owned()])
                            .map(|_| format!("Created {dest}"))
                    }
                    None => Err(crate::fsbridge::FsError::NotFound),
                }
            }
            FilesOp::CompressMany { name, sources } => {
                let dest = format!("{}/{}", self.files.path.trim_end_matches('/'), name);
                bridge
                    .compress(&dest, &sources)
                    .map(|_| format!("Created {dest} from {} items", sources.len()))
            }
            FilesOp::Extract(p) => bridge.extract(&p).map(|d| format!("Extracted to {d}")),
        };
        match out {
            Ok(msg) => {
                self.files.listed = None;
                self.notify("Files", "Done", &msg, Priority::Low, Some(4_000));
            }
            Err(e) => {
                self.files.dialog = Some(crate::apps::DialogKind::Error(e.to_string()));
            }
        }
        self.request_full_redraw();
    }

    /// Move to the Trash, asking first when the setting says to.
    ///
    /// The setting covers the recoverable action only. Permanent deletion
    /// always asks, because a preference about the Trash that silently grew
    /// to cover the irreversible one would be a different setting wearing the
    /// same name.
    fn trash_with_confirm(&mut self, title: &str, detail: &str, action: crate::apps::FilesOp) {
        self.files.dialog = None;
        if self.files.confirm_trash {
            self.files.ask_confirm(title, detail, action);
            self.request_full_redraw();
        } else {
            self.perform_file_op(action);
        }
    }

    /// Confirm-and-run a Files dialog submission.
    fn submit_files_dialog(&mut self) {
        use crate::apps::DialogKind;
        let zone = self.regional.zone.clone();
        let Ok(bridge) = self.try_files_bridge(zone) else {
            self.files.dialog = Some(crate::apps::DialogKind::Error(
                "the file service is not available on this platform".to_owned(),
            ));
            return;
        };
        let Some(dialog) = self.files.dialog.clone() else {
            return;
        };
        let result: Result<String, crate::fsbridge::FsError> = match dialog {
            DialogKind::NewFolder => {
                let name = self.files.dialog_field.clone();
                bridge
                    .mkdir(&self.files.path, &name)
                    .map(|_| format!("/{}", name))
            }
            DialogKind::NewFile => {
                let name = self.files.dialog_field.clone();
                bridge
                    .write_text(&self.files.path, &name, "")
                    .map(|_| format!("/{}", name))
            }
            DialogKind::Rename { path, .. } => {
                let parent = path
                    .rsplit_once('/')
                    .map(|(d, _)| d.to_owned())
                    .unwrap_or_else(|| "/".to_owned());
                let parent = if parent.is_empty() {
                    "/".to_owned()
                } else {
                    parent
                };
                let old = path.rsplit('/').next().unwrap_or_default().to_owned();
                let new = self.files.dialog_field.clone();
                bridge.rename(&parent, &old, &new).map(|_| new.clone())
            }
            DialogKind::Confirm { action, .. } => {
                self.files.dialog = None;
                self.perform_file_op(action);
                return;
            }
            DialogKind::Error(_) => {
                self.files.dialog = None;
                return;
            }
        };
        match result {
            Ok(what) => {
                self.files.dialog = None;
                self.files.listed = None;
                self.notify("Files", "Done", &what, Priority::Low, Some(3_000));
            }
            Err(e) => {
                self.files.dialog = Some(DialogKind::Error(e.to_string()));
            }
        }
        self.request_full_redraw();
    }

    /// Execute a context-menu action.
    pub fn perform_menu(&mut self, action: crate::apps::MenuAction) -> Vec<Effect> {
        use crate::apps::MenuAction as MA;
        match action {
            MA::NewFolder => {
                let in_files = self.context_menu.as_ref().is_some_and(|m| {
                    matches!(
                        m.surface,
                        crate::apps::MenuSurface::Files | crate::apps::MenuSurface::FilesRow
                    )
                });
                if in_files {
                    self.files.ask_new_folder();
                } else {
                    self.desktop_dialog = Some(crate::apps::DesktopDialog::NewFolder);
                    self.desktop_dialog_field.clear();
                    self.desktop_dialog_selection = None;
                }
            }
            MA::NewFile => self.files.ask_new_file(),
            MA::Paste => {
                let op = crate::apps::FilesOp::PasteCopy;
                self.files
                    .ask_confirm("Paste", "Paste the clipboard here?", op);
            }
            MA::Rename => self.files.ask_rename(),
            MA::Copy => {
                self.files.copy_selection();
            }
            MA::Cut => {
                self.files.cut_selection();
            }
            MA::Trash => {
                if let Some(p) = self.files.selected_path() {
                    self.trash_with_confirm(
                        "Move to trash",
                        &format!("Move {p} to the trash?"),
                        crate::apps::FilesOp::Trash(p),
                    );
                }
            }
            MA::Compress => {
                let _ = crate::apps::files_action(&mut self.files, crate::apps::FilesHot::Compress);
            }
            MA::Extract => {
                let _ = crate::apps::files_action(&mut self.files, crate::apps::FilesHot::Extract);
            }
            MA::SortName => self.files.toggle_sort(crate::apps::SortKey::Name),
            MA::SortSize => self.files.toggle_sort(crate::apps::SortKey::Size),
            MA::SortModified => self.files.toggle_sort(crate::apps::SortKey::Modified),
            MA::ToggleDetails => self.files.details = !self.files.details,
            MA::ScanNow => return self.start_scan(),
            MA::BrowserStart => return self.browser_start_session(),
            MA::BrowserReload => {
                if let Some(t) = self.browser.tabs.get(self.browser.active) {
                    let url = t.url.clone();
                    self.browser.status = format!("reloading {url}");
                    let _ = self.browser.navigate_active();
                }
            }
            MA::BrowserCloseTab => {
                let i = self.browser.active;
                self.browser.close(i);
            }
            MA::DisplaySettings => {
                return self.perform(crate::shortcuts::Action::Open(App::Settings))
            }
            MA::OpenTrash => return self.perform(Action::Open(App::Trash)),
            MA::EmptyTrash => {
                if self.trash.confirm != Some(crate::apps::TrashConfirmation::Empty) {
                    self.trash.confirm = Some(crate::apps::TrashConfirmation::Empty);
                    return Vec::new();
                }
                self.trash.confirm = None;
                let zone = self.regional.zone.clone();
                let Ok(bridge) = self.try_files_bridge(zone) else {
                    self.trash.error = Some("the file service is not available".to_owned());
                    return Vec::new();
                };
                match bridge.trash_empty() {
                    Ok(n) => {
                        self.trash.listed = false;
                        self.notify(
                            "Trash",
                            "Emptied",
                            &format!("{n} item(s) deleted for ever"),
                            Priority::Normal,
                            Some(4_000),
                        );
                    }
                    Err(e) => {
                        self.trash.error = Some(e.to_string());
                    }
                }
            }
            MA::Restore => {
                if self.trash.has_visible_selection() {
                    if let Some(name) = self.trash.selected_name() {
                        let zone = self.regional.zone.clone();
                        let Ok(bridge) = self.try_files_bridge(zone) else {
                            self.trash.error = Some("the file service is not available".to_owned());
                            return Vec::new();
                        };
                        match bridge.trash_restore(&name) {
                            Ok(dir) => {
                                self.trash.listed = false;
                                self.notify(
                                    "Trash",
                                    "Restored",
                                    &format!("{name} back in {dir}"),
                                    Priority::Low,
                                    Some(4_000),
                                );
                            }
                            Err(e) => self.trash.error = Some(e.to_string()),
                        }
                    }
                }
            }
            MA::PurgeTrashItem => {
                if !self.trash.has_visible_selection() {
                    return Vec::new();
                }
                let Some(name) = self.trash.selected_name() else {
                    return Vec::new();
                };
                let pending = crate::apps::TrashConfirmation::Purge(name.clone());
                if self.trash.confirm.as_ref() != Some(&pending) {
                    self.trash.confirm = Some(pending);
                    return Vec::new();
                }
                self.trash.confirm = None;
                let zone = self.regional.zone.clone();
                let Ok(bridge) = self.try_files_bridge(zone) else {
                    self.trash.error = Some("the file service is not available".to_owned());
                    return Vec::new();
                };
                match bridge.trash_purge(&name) {
                    Ok(()) => {
                        self.trash.listed = false;
                        self.trash.selected = None;
                        self.notify(
                            "Trash",
                            "Deleted permanently",
                            &name,
                            Priority::Normal,
                            Some(4_000),
                        );
                    }
                    Err(e) => self.trash.error = Some(e.to_string()),
                }
            }
            MA::OpenSelection => {
                if let Some(i) = self.desktop_icons.selected {
                    return self.open_desktop_icon(i);
                }
                if let Some(p) = self.files.selected_path() {
                    return self.open_files_entry(&p);
                }
            }
            MA::NextWallpaper => {
                return self.cycle_theme();
            }
            MA::EngineGoogle => {
                self.browser.engine = crate::apps::SearchEngine::Google;
            }
            MA::EngineDuckDuckGo => {
                self.browser.engine = crate::apps::SearchEngine::DuckDuckGo;
            }
            MA::EngineBing => {
                self.browser.engine = crate::apps::SearchEngine::Bing;
            }
        }
        Vec::new()
    }

    /// Open a desktop icon: an app, or Files at a folder.
    pub fn open_desktop_icon(&mut self, i: usize) -> Vec<Effect> {
        let Some(icon) = self.desktop_icons.items.get(i) else {
            return Vec::new();
        };
        match icon.target.clone() {
            crate::apps::IconTarget::App(app) => self.perform(crate::shortcuts::Action::Open(app)),
            crate::apps::IconTarget::Folder(path) => {
                self.files.navigate(&path);
                self.perform(crate::shortcuts::Action::Open(App::Files))
            }
        }
    }

    /// Open a Files entry: folders navigate, text files open the viewer,
    /// archives offer extraction, anything else is named honestly.
    pub fn open_files_entry(&mut self, path: &str) -> Vec<Effect> {
        let zone = self.regional.zone.clone();
        let Ok(bridge) = self.try_files_bridge(zone) else {
            self.files.dialog = Some(crate::apps::DialogKind::Error(
                "the file service is not available on this platform".to_owned(),
            ));
            return Vec::new();
        };
        let name = path.rsplit('/').next().unwrap_or_default().to_owned();
        if path.ends_with(".zip") || path.ends_with(".tar") || path.ends_with(".tgz") {
            self.files.ask_confirm(
                "Extract",
                &format!("Extract {path} into a folder beside it?"),
                crate::apps::FilesOp::Extract(path.to_owned()),
            );
        } else if name.ends_with(".txt")
            || name.ends_with(".md")
            || name.ends_with(".log")
            || name.ends_with(".json")
        {
            match bridge.read_text(path) {
                Ok(text) => {
                    self.files.viewer = Some(crate::apps::FileViewer {
                        path: path.to_owned(),
                        text,
                        scroll: 0,
                    });
                }
                Err(e) => {
                    self.files.dialog = Some(crate::apps::DialogKind::Error(e.to_string()));
                }
            }
        } else {
            self.notify(
                "Files",
                "No application for this",
                &format!("{name}: LitricOS has no app registered for this type yet."),
                Priority::Low,
                Some(5_000),
            );
        }
        Vec::new()
    }

    /// Refresh the Trash listing when the window is open and stale.
    pub fn refresh_trash(&mut self) {
        if self.trash.listed {
            return;
        }
        let zone = self.regional.zone.clone();
        let Ok(bridge) = self.try_files_bridge(zone) else {
            self.trash.error = Some("the file service is not available".to_owned());
            self.trash.listed = true;
            return;
        };
        match bridge.trash_list() {
            Ok(entries) => {
                self.trash.entries = entries
                    .into_iter()
                    .map(|e| (e.name, e.deleted_iso, e.restore_dir, e.is_dir))
                    .collect();
                self.trash.error = None;
            }
            Err(e) => self.trash.error = Some(e.to_string()),
        }
        self.trash.selected = None;
        self.trash.listed = true;
    }

    /// Submit the desktop's own dialog.
    ///
    /// Returns the effects the submission produced, which is what makes the
    /// power confirmation a decision rather than a drawing: the effect leaves
    /// this function only after the user answered the prompt that named the
    /// action.
    fn submit_desktop_dialog(&mut self) -> Vec<Effect> {
        use crate::apps::DesktopDialog;
        if let Some(DesktopDialog::NewFolder) = self.desktop_dialog {
            let name = self.desktop_dialog_field.clone();
            let zone = self.regional.zone.clone();
            let Ok(bridge) = self.try_files_bridge(zone) else {
                self.desktop_dialog = Some(DesktopDialog::Error(
                    "the file service is not available on this platform".to_owned(),
                ));
                return Vec::new();
            };
            match bridge.mkdir("/Documents", &name) {
                Ok(()) => {
                    self.desktop_dialog = None;
                    self.files.listed = None;
                    self.notify(
                        "Files",
                        "Folder created",
                        &format!("/Documents/{name}"),
                        Priority::Low,
                        Some(3_000),
                    );
                }
                Err(e) => {
                    self.desktop_dialog = Some(DesktopDialog::Error(e.to_string()));
                }
            }
            self.request_full_redraw();
            return Vec::new();
        }
        let confirmed = match self.desktop_dialog {
            Some(DesktopDialog::PowerConfirm { command }) => Some(command),
            _ => None,
        };
        self.desktop_dialog = None;
        self.request_full_redraw();
        confirmed.map_or_else(Vec::new, |command| vec![Effect::Power(command)])
    }
}

/// The desktop.
#[derive(Debug)]
pub struct Shell {
    /// Design tokens for this display.
    pub tokens: Tokens,
    /// Windows, focus, workspaces.
    pub wm: WindowManager,
    /// The shortcut table.
    pub shortcuts: Shortcuts,
    /// Where the machine is, and how its user writes things.
    pub regional: Regional,
    /// How much of the desktop is working.
    pub mode: DesktopMode,
    /// Which overlay is showing.
    pub overlay: Overlay,
    /// Waiting notifications.
    pub notifications: Notifications,
    /// Quick settings.
    pub quick: QuickSettings,
    /// The environments.
    pub environments: Vec<EnvRow>,
    /// Shell-owned language, theme and accessibility settings.
    pub settings: crate::settings::Model,
    /// The numbers on screen.
    pub facts: SystemFacts,
    /// The boot flow — the splash every boot shows, and the first-run wizard
    /// a first boot needs. It owns the screen and every input event while it
    /// is `Some`, and it is `None` on a machine that is simply running.
    pub first_run: Option<crate::firstrun::FirstRun>,
    /// The Activity window's model: machine meters, the process table, and what
    /// a person has changed about both.
    pub activity: crate::taskmgr::TaskManager,
    /// The Files application model (Explorer-style places, history, table).
    pub files: crate::apps::FilesApp,
    /// The host root every file surface reads and writes through.
    ///
    /// `/` on a booted machine, and what `--root` moves: the shell already
    /// reads `/proc`, `/sys` and `/dev` through that root, but its file
    /// surfaces used to hardcode `/`, so a shell pointed at a fixture tree
    /// still wrote to the machine it was running on. It is also the seam a
    /// test needs: a journey that creates a folder can aim at a temporary
    /// directory instead of the real root.
    pub files_root: std::path::PathBuf,
    /// The Litric Browser window model (Chrome-style tab strip + omnibox).
    pub browser: crate::apps::BrowserApp,
    /// The Defender application model and its scan state.
    pub defender: DefenderState,
    /// The Trash application model.
    pub trash: crate::apps::TrashApp,
    /// The Terminal application model.
    pub terminal: crate::apps::TerminalApp,
    /// The running terminal command's output channel (unix only).
    #[cfg(unix)]
    terminal_job: Option<std::sync::mpsc::Receiver<String>>,
    /// An open context menu, if any.
    pub context_menu: Option<crate::apps::ContextMenu>,
    /// The desktop's icon grid.
    pub desktop_icons: crate::apps::DesktopIcons,
    /// An open dialog on the desktop itself (new folder on the wallpaper).
    pub desktop_dialog: Option<crate::apps::DesktopDialog>,
    /// The desktop dialog's text field.
    pub desktop_dialog_field: String,
    /// The desktop dialog field's selection.
    pub desktop_dialog_selection: Option<(usize, usize)>,
    /// The browser engine session, when one has been started.
    pub browser_session: Option<crate::link::Link>,
    /// The last frame the engine produced: RGBA8 pixels plus dimensions.
    /// One frame only — a newer frame replaces an undrawn older one, which is
    /// what bounds the memory a fast page can make the compositor hold.
    pub browser_frame: Option<Arc<[u8]>>,
    /// Whether the diagnostic overlay is showing.
    pub diagnostics: bool,
    /// When to go idle.
    pub idle_policy: IdlePolicy,
    /// The lock screen: whether the session is locked, what has been typed, and
    /// the look. Every field it holds while locked is invisible to every other
    /// surface, because while it is locked nothing else is drawn at all.
    pub lock: crate::lock::LockState,
    /// The lock surface's memory between frames: the picture behind it and the
    /// blurred copy its glass panels sample.
    ///
    /// Kept here rather than in [`crate::lock::LockState`] because it is a
    /// picture, not a state: the same surface draws whatever is in the cache,
    /// and dropping the cache changes nothing a person can see.
    pub lock_cache: crate::lock::Cache,

    hot: Hotspots<Hot>,
    pointer: Point,
    hovered: Option<Hot>,
    /// Current target in the shell-wide Tab/Shift+Tab focus order.
    keyboard_focus: Option<Hot>,
    pressed: Option<Hot>,
    meta_tap: MetaTap,
    last_input_ms: u64,
    now_ms: u64,
    /// The time [`Shell::tick`] last stepped the animation springs from, or
    /// `None` before the first tick.
    ///
    /// Springs are advanced by the wall time that actually passed, never by a
    /// constant. The loop's frame pace is 8 ms under a dragged window, 16 ms
    /// active, 33 ms idle and more when nothing is happening at all, so a
    /// constant step makes every animation in the shell run at a different
    /// speed on every one of those — and at the intended speed on only the one
    /// the constant was chosen for.
    last_step_ms: Option<u64>,
    damage: Vec<Rect>,
    full_redraw: bool,
    frames: u64,
    /// The frame-rate probe behind `diagnostics.fps`: when the open one-second
    /// window started. Zero is "no frame counted yet".
    fps_window_start_ms: u64,
    /// Frames presented inside the open window.
    fps_window_frames: u32,
    /// Frames per second over the last completed window. `None` until a window
    /// has closed, so the overlay says it is warming rather than reporting a
    /// rate from two frames.
    fps_last: Option<u32>,
    /// The shell's overlay springs, stepped once per frame in `tick`.
    anims: crate::anim::ShellAnims,
    /// Whether Alt is still held while the switcher is open.
    switcher_holding: bool,
    /// The focused window when damage was last taken. A focus change repaints
    /// both windows and the top bar's title.
    drawn_focus: Option<WindowId>,
    /// The clock text as of the last time the shell's time moved. The top bar
    /// is damaged when it changes and not otherwise.
    drawn_clock: String,
    /// Whether the low-battery notification has already been raised this
    /// discharge.
    battery_warned: bool,
}

impl Shell {
    /// Replace startup defaults with a validated preference record.
    pub fn load_preferences(&mut self, preferences: crate::settings::Preferences) {
        self.settings =
            crate::settings::Model::new(preferences).with_activity(&self.activity.prefs);
        self.apply_settings();
    }

    /// Replace every value with a settings file's contents.
    pub fn load_settings(&mut self, values: crate::settings::Settings) {
        let mut model = crate::settings::Model::new(crate::settings::Preferences::default());
        model.values = values;
        self.settings = model.with_activity(&self.activity.prefs);
        self.apply_settings();
    }

    /// The effect that asks `main` to write the current values atomically.
    fn persist_settings(&self) -> Vec<Effect> {
        vec![Effect::PersistSettings(self.settings.values.clone())]
    }

    /// Apply every value to the surface that reads it.
    ///
    /// This is the one place the settings become behaviour. It is called after
    /// startup and after every accepted change, and it is idempotent: calling it
    /// twice with the same values produces the same desktop.
    pub fn apply_settings(&mut self) {
        use crate::ui::Look;
        let values = self.settings.values.clone();
        let mut theme = self.settings.theme();

        // Material: blur, border and the transparency switch belong to the
        // theme's glass description, which `desktop::glass` reads for every
        // panel, so overriding them here reaches all of them at once.
        let blur = values.num("appearance.blur").clamp(0.0, 40.0);
        let border = values.num("appearance.border") / 100.0;
        theme.glass.blur_px = if values.on("appearance.transparency") {
            u16::try_from(blur.round() as i64).unwrap_or(0)
        } else {
            0
        };
        theme.glass.border_alpha_pct = u8::try_from(
            (f32::from(theme.glass.border_alpha_pct) * border)
                .round()
                .clamp(0.0, 255.0) as i64,
        )
        .unwrap_or(14);
        if !values.on("appearance.transparency") {
            theme.glass.tint_alpha_pct = 100;
        }

        let spacing = match values.pick("appearance.spacing") {
            "compact" => 0.85,
            "roomy" => 1.2,
            _ => 1.0,
        };
        self.tokens = crate::ui::Tokens::with_look(
            theme,
            self.tokens.scale,
            Look {
                radius: (values.num("appearance.radius") / 12.0).max(0.0),
                spacing,
                type_scale: (values.num("appearance.text-scale") / 100.0).clamp(0.5, 2.0),
                focus_width: values.num("appearance.focus-ring").clamp(1.0, 6.0),
                cursor: (values.num("appearance.cursor-scale") / 100.0).clamp(0.5, 3.0),
                shadow: (values.num("appearance.shadow") / 100.0).clamp(0.0, 2.0),
                selection: (values.num("appearance.selection") / 100.0).clamp(0.0, 2.0),
                icons: (values.num("appearance.icon-scale") / 100.0).clamp(0.5, 2.0),
                line_height: (values.num("appearance.line-height") / 100.0).clamp(0.8, 2.0),
            },
        );

        self.quick.reduce_motion = !values.on("appearance.motion");
        // Speed is a multiplier on the design's own springs, so "fast" keeps
        // each panel's character (the launcher still settles later than the
        // context menu) while every one of them finishes sooner.
        let speed = match values.pick("appearance.motion-speed") {
            "slow" => 0.6,
            "fast" => 1.6,
            _ => 1.0,
        };
        self.anims.set_speed(speed);
        self.quick.adaptive_idle = values.on("power.adaptive-idle");
        self.idle_policy = IdlePolicy {
            idle_after_ms: (values.num("power.idle-after") * 1000.0).max(0.0) as u64,
            deep_after_ms: (values.num("power.deep-idle-after") * 1000.0).max(1.0) as u64,
        };
        self.notifications.do_not_disturb = values.on("notifications.dnd");
        self.diagnostics = values.on("diagnostics.overlay");
        let file_sort = match values.pick("files.sort") {
            "size" => crate::apps::SortKey::Size,
            "modified" => crate::apps::SortKey::Modified,
            _ => crate::apps::SortKey::Name,
        };
        self.files.sort = (file_sort, values.on("files.ascending"));
        self.files.details = values.on("files.details");
        self.files.folders_first = values.on("files.folders-first");
        self.files.confirm_trash = values.on("files.confirm-trash");
        self.terminal.max_lines = values.num("terminal.scrollback").max(32.0) as usize;
        self.terminal.max_history = values.num("terminal.history").max(4.0) as usize;
        self.settings
            .values
            .apply_activity(&mut self.activity.prefs);

        // The top bar's height is geometry, not just paint: the window manager
        // reserves exactly this much, and a bar drawn taller than the reserved
        // band would sit over the first row of pixels a window owns.
        let top = self.top_bar_height();
        let screen = self.wm.screen();
        self.wm.set_display(screen.w, screen.h, top);
        self.request_full_redraw();
    }

    /// Complete a settings write, rolling back to the previous values if the
    /// write failed.
    pub fn finish_preferences_save(&mut self, result: Result<(), String>) {
        self.settings.finish(result);
        self.apply_settings();
    }

    /// Replace every value with the decoded contents of an export, then apply
    /// them to the desktop.
    ///
    /// Validation happens before anything is applied ([`crate::settings::Model::import`]
    /// is all-or-nothing), and persistence is left to the caller, which is
    /// where the file lives. `Ok` carries the report the settings footer shows.
    pub fn import_settings(&mut self, text: &str) -> Result<String, String> {
        let report = self.settings.import(text)?;
        self.apply_settings();
        Ok(report)
    }

    /// Change one setting, apply it and ask `main` to persist it.
    pub fn change_setting(&mut self, id: &str, value: crate::settings::Value) -> Vec<Effect> {
        if !self.settings.change(id, value) {
            self.request_full_redraw();
            return Vec::new();
        }
        self.apply_settings();
        self.persist_settings()
    }

    /// Mirror the Activity table back into the settings store after the table's
    /// own controls changed it. Returns the persistence effect when anything
    /// actually moved.
    pub fn capture_activity_settings(&mut self) -> Vec<Effect> {
        let before = self.settings.values.clone();
        self.settings.values.capture_activity(&self.activity.prefs);
        if self.settings.values == before {
            return Vec::new();
        }
        self.settings.save = crate::settings::SaveState::Saving;
        self.settings.dirty = true;
        self.persist_settings()
    }

    fn toggle_language(&mut self) -> Vec<Effect> {
        let next = match self.settings.language() {
            crate::i18n::Language::Thai => crate::i18n::Language::English,
            crate::i18n::Language::English => crate::i18n::Language::Thai,
        };
        if self.settings.set_language(next).is_err() {
            return Vec::new();
        }
        self.apply_settings();
        self.persist_settings()
    }

    fn cycle_theme(&mut self) -> Vec<Effect> {
        let themes = litric_design::ThemeKind::all();
        let current = self.settings.theme_kind();
        let index = themes.iter().position(|kind| *kind == current).unwrap_or(0);
        let Some(theme) = themes
            .get(index.saturating_add(1))
            .or_else(|| themes.first())
            .copied()
        else {
            return Vec::new();
        };
        self.change_setting(
            "appearance.theme",
            crate::settings::Value::Pick(theme.token()),
        )
    }

    fn toggle_reduced_motion(&mut self) -> Vec<Effect> {
        let on = !self.settings.values.on("appearance.motion");
        self.change_setting("appearance.motion", crate::settings::Value::On(on))
    }

    /// The wallpaper the compositor should paint: whether the coloured washes
    /// are on, and how strongly they show (0..1).
    ///
    /// The compositor owns the paper, so it asks the shell for this rather than
    /// reading the settings store itself: one place decides what the settings
    /// mean, and the painting code only paints.
    #[must_use]
    pub fn wallpaper_spec(&self) -> (bool, f32) {
        (
            self.settings.values.pick("appearance.wallpaper") != "plain",
            (self.settings.values.num("appearance.wallpaper-tint") / 100.0).clamp(0.0, 1.0),
        )
    }

    /// How often the machine's counters are re-read, from the setting.
    #[must_use]
    pub fn sample_interval_ms(&self) -> u64 {
        (self.settings.values.num("power.sample-ms") as u64).clamp(250, 5_000)
    }

    /// The height of the top bar in pixels, from its setting.
    #[must_use]
    pub fn top_bar_height(&self) -> f32 {
        if self.settings.values.on("topbar.enabled") {
            self.tokens
                .px(self.settings.values.num("topbar.height").clamp(20.0, 80.0))
        } else {
            0.0
        }
    }

    /// A desktop for a display of this size.
    #[must_use]
    pub fn new(tokens: Tokens, width: f32, height: f32, regional: Regional) -> Self {
        let top = tokens.px(Self::TOP_BAR_LOGICAL);
        let drawn_clock = regional.clock(0);
        let theme = tokens.theme.kind;
        let language = if regional.locale.tag.starts_with("en") {
            crate::i18n::Language::English
        } else {
            crate::i18n::Language::Thai
        };
        Self {
            wm: WindowManager::new(width, height, top),
            tokens,
            shortcuts: Shortcuts::default(),
            regional,
            mode: DesktopMode::Loading,
            overlay: Overlay::None,
            notifications: Notifications::default(),
            quick: QuickSettings::default(),
            environments: Vec::new(),
            settings: crate::settings::Model::new(crate::settings::Preferences::new(
                theme, language,
            )),
            facts: SystemFacts::default(),
            first_run: None,
            activity: crate::taskmgr::TaskManager::new(),
            files: crate::apps::FilesApp::default(),
            files_root: std::path::PathBuf::from("/"),
            browser: crate::apps::BrowserApp::default(),
            defender: DefenderState::default(),
            trash: crate::apps::TrashApp::default(),
            terminal: crate::apps::TerminalApp::new(),
            #[cfg(unix)]
            terminal_job: None,
            context_menu: None,
            desktop_icons: crate::apps::DesktopIcons::minimal(),
            desktop_dialog: None,
            desktop_dialog_field: String::new(),
            desktop_dialog_selection: None,
            browser_session: None,
            browser_frame: None,
            diagnostics: false,
            idle_policy: IdlePolicy::default(),
            lock: crate::lock::LockState::default(),
            lock_cache: crate::lock::Cache::default(),
            hot: Hotspots::default(),
            pointer: Point::new(width / 2.0, height / 2.0),
            hovered: None,
            keyboard_focus: None,
            pressed: None,
            meta_tap: MetaTap::default(),
            last_input_ms: 0,
            now_ms: 0,
            last_step_ms: None,
            damage: Vec::new(),
            full_redraw: true,
            frames: 0,
            fps_window_start_ms: 0,
            fps_window_frames: 0,
            fps_last: None,
            anims: crate::anim::ShellAnims::new(),
            switcher_holding: false,
            drawn_focus: None,
            drawn_clock,
            battery_warned: false,
        }
    }

    /// The top bar's height in logical units. There is no bottom bar: one
    /// system bar carries everything, which is the whole layout decision.
    pub const TOP_BAR_LOGICAL: f32 = 28.0;

    /// The top bar's rectangle.
    #[must_use]
    pub fn top_bar(&self) -> Rect {
        let s = self.wm.screen();
        Rect::new(0.0, 0.0, s.w, self.tokens.px(Self::TOP_BAR_LOGICAL))
    }

    /// Where the pointer is.
    #[must_use]
    pub const fn pointer(&self) -> Point {
        self.pointer
    }

    /// What the pointer is over.
    #[must_use]
    pub const fn hovered(&self) -> Option<Hot> {
        self.hovered
    }

    /// Whether a control is under the pointer or has keyboard focus.
    #[must_use]
    pub fn is_highlighted(&self, hot: Hot) -> bool {
        matches!(self.hovered, Some(h) if h == hot)
            || matches!(self.keyboard_focus, Some(h) if h == hot)
    }

    /// The current keyboard target, primarily for diagnostics and tests.
    #[must_use]
    pub const fn keyboard_focus(&self) -> Option<Hot> {
        self.keyboard_focus
    }

    fn move_keyboard_focus(&mut self, backwards: bool) {
        let candidates: Vec<Hot> = self
            .hot
            .ids()
            .filter(|hot| hot.keyboard_focusable())
            .collect();
        if candidates.is_empty() {
            return;
        }
        let current = self
            .keyboard_focus
            .and_then(|focused| candidates.iter().position(|hot| *hot == focused));
        let next = match (current, backwards) {
            (Some(0), true) => candidates.len().saturating_sub(1),
            (Some(index), true) => index.saturating_sub(1),
            // Wrap at the end of the list. The guard above keeps the list
            // non-empty; `checked_rem` keeps that a fact of the arithmetic
            // rather than an assumption about the guard.
            (Some(index), false) => index
                .saturating_add(1)
                .checked_rem(candidates.len())
                .unwrap_or(0),
            (None, true) => candidates.len().saturating_sub(1),
            (None, false) => 0,
        };
        self.keyboard_focus = candidates.get(next).copied();
        self.request_full_redraw();
    }

    fn keyboard_focus_is_text_field(&self) -> bool {
        matches!(
            self.keyboard_focus,
            Some(
                Hot::TerminalInput
                    | Hot::BrowserOmnibox
                    | Hot::TrashFilter
                    | Hot::ActivityFilter
                    | Hot::SettingsSearch
            ) | Some(Hot::Files(crate::apps::FilesHot::Filter))
                | Some(Hot::LauncherSearch)
        ) || self.settings.editing.is_some()
    }

    /// What the pointer is pressed on.
    #[must_use]
    pub const fn pressed(&self) -> Option<Hot> {
        self.pressed
    }

    /// The hotspot table, for the renderer to fill in during layout.
    #[must_use]
    pub fn hotspots_mut(&mut self) -> &mut Hotspots<Hot> {
        &mut self.hot
    }

    /// The hotspot table.
    #[must_use]
    pub const fn hotspots(&self) -> &Hotspots<Hot> {
        &self.hot
    }

    /// How many frames have been drawn.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.frames
    }

    /// The overlay springs, for the views to read reveal progress from.
    #[must_use]
    pub const fn anims(&self) -> &crate::anim::ShellAnims {
        &self.anims
    }

    /// Mutable access for the draw side, which is the only place that knows
    /// the window list's final order; elevation is sampled there.
    pub fn anims_mut(&mut self) -> &mut crate::anim::ShellAnims {
        &mut self.anims
    }

    /// Count a frame.
    ///
    /// This is also the frame-rate probe behind `diagnostics.fps`: presented
    /// frames are counted into one-second windows, and a closed window reports
    /// its rate. The window's **real** length divides, so a second in which the
    /// shell stalled reports the low number it owes — a rate computed against
    /// the nominal second would print 60 while the desktop crawled.
    pub fn counted_frame(&mut self) {
        self.frames = self.frames.saturating_add(1);
        if self.fps_window_start_ms == 0 {
            self.fps_window_start_ms = self.now_ms;
        }
        self.fps_window_frames = self.fps_window_frames.saturating_add(1);
        let Some(span) = self
            .now_ms
            .checked_sub(self.fps_window_start_ms)
            .filter(|span| *span >= 1_000)
        else {
            return;
        };
        let fps = u32::try_from(span)
            .ok()
            .and_then(|span| {
                self.fps_window_frames
                    .saturating_mul(1_000)
                    .checked_div(span)
            })
            .unwrap_or(0);
        self.fps_last = Some(fps);
        self.fps_window_start_ms = self.now_ms;
        self.fps_window_frames = 0;
    }

    /// The current life state.
    #[must_use]
    pub fn life(&self) -> LifeState {
        // Locked outranks everything: nothing else is drawn, so nothing else
        // decides how often a frame is worth composing.
        if self.lock.locked {
            return LifeState::Locked;
        }
        if self.wm.is_dragging() {
            return LifeState::WindowInteraction;
        }
        if !self.quick.adaptive_idle {
            return LifeState::Active;
        }
        self.idle_policy
            .state_after(self.now_ms.saturating_sub(self.last_input_ms))
    }

    /// The pace the current life state asks for, in milliseconds: how long one
    /// frame may take while there is something to draw.
    ///
    /// This is a **ceiling on work, not the length of the next sleep**.
    /// [`Shell::wake_after_ms`] answers the sleep question, and it is the one
    /// the frame loop asks, because a desktop that wakes 30 times a second to
    /// discover it has nothing to draw is a desktop that spends its battery
    /// proving it is awake.
    #[must_use]
    pub fn frame_interval_ms(&self) -> u64 {
        self.life().frame_interval_ms()
    }

    /// Whether a spring is mid-flight, so the next frame would draw something
    /// new even with no input and no damage of its own.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.anims.animating()
            // The boot flow animates outside the overlay springs: the splash
            // reveal is drawn from the clock and the wizard's pages slide in.
            // While either is on screen the frame loop must hold the active
            // pace, or the wordmark would write itself one frame a second.
            || self.first_run.as_ref().is_some_and(|fr| {
                !fr.arrival.settled()
                    || matches!(
                        fr.page,
                        crate::firstrun::Page::Splash | crate::firstrun::Page::Preparing
                    )
            })
    }

    /// The longest a frame loop may sleep when nothing needs drawing.
    ///
    /// One second. The top bar's clock is the slowest thing on screen that
    /// changes by itself and it changes on the second, so a longer sleep would
    /// show a stale clock; and it puts a floor of one wake-up a second under
    /// this process, against the two `docs/PERF-budget.md` §4 allows.
    pub const IDLE_SLEEP_CAP_MS: u64 = 1_000;

    /// How long the frame loop may sleep before the shell must be looked at
    /// again, in milliseconds. Zero means "there is a frame to draw": do not
    /// sleep at all.
    ///
    /// Three answers, in the order the question is asked:
    ///
    ///   * **Damage waiting** — draw it. A dragged window outranks everything
    ///     (dropping one of those frames is visible as the window lagging the
    ///     pointer); a locked session comes next, and it is the case that needs
    ///     throttling most, because a locked frame is always damage and would
    ///     otherwise be asked for as fast as the CPU can produce it (BUG-0018
    ///     measures one at 570 ms); a spring mid-flight gets the active pace.
    ///   * **Nothing moving and nothing damaged** — sleep until the next moment
    ///     the screen can change on its own: the clock's next tick, or the
    ///     idle timer that would lower the pace. This is the branch that
    ///     matters on a laptop.
    ///
    /// Input is not part of this calculation and does not need to be: the loop
    /// blocks on the input channel, so a key or a pointer wakes it the instant
    /// it happens whatever this returns. See `litric_input::Waker`.
    #[must_use]
    pub fn wake_after_ms(&self, now_ms: u64) -> u64 {
        if self.is_dirty() {
            return match self.life() {
                LifeState::WindowInteraction => LifeState::WindowInteraction.frame_interval_ms(),
                LifeState::Locked => LifeState::Locked.frame_interval_ms(),
                // Motion that outlives the idle timer still runs at the active
                // pace: a panel opening twenty-five seconds after the last
                // keypress must not open at half speed.
                _ if self.animating() => LifeState::Active.frame_interval_ms(),
                _ => 0,
            };
        }
        let until_clock = self.next_clock_change_ms().saturating_sub(now_ms);
        until_clock
            .min(self.ms_until_idle_change(now_ms))
            .min(Self::IDLE_SLEEP_CAP_MS)
    }

    /// Milliseconds until the life state would change with no further input,
    /// or `u64::MAX` when nothing would change it on its own.
    ///
    /// Zero is never returned for a state that is still current: [`Self::life`]
    /// and [`IdlePolicy::state_after`] compare the same elapsed time against the
    /// same thresholds, so a deadline that has already passed is a state that
    /// has already moved on.
    fn ms_until_idle_change(&self, now_ms: u64) -> u64 {
        if !self.quick.adaptive_idle {
            // Nothing lowers the pace, so there is no moment to wake for.
            return u64::MAX;
        }
        let quiet = now_ms.saturating_sub(self.last_input_ms);
        match self.life() {
            LifeState::Active => self.idle_policy.idle_after_ms.saturating_sub(quiet),
            LifeState::Idle => self.idle_policy.deep_after_ms.saturating_sub(quiet),
            // A dragged window and a locked session outrank the idle timers,
            // and deep idle is already the slowest state there is.
            _ => u64::MAX,
        }
    }

    /// How far the animation clock moved since the last tick, in milliseconds.
    ///
    /// Before the first tick there is nothing to measure from: `Shell::new`
    /// cannot know the time, and treating "unset" as "milliseconds since the
    /// epoch" would put every spring past its settle budget and snap it to its
    /// target on the frame it was started. So the first tick establishes the
    /// clock and steps nothing; [`Shell::tick_after`] is the exception, because
    /// a caller that names the elapsed time is not guessing at it.
    fn animation_step_ms(&mut self, now_ms: u64) -> u32 {
        let dt = self
            .last_step_ms
            .map_or(0, |last| now_ms.saturating_sub(last));
        self.last_step_ms = Some(now_ms);
        u32::try_from(dt).unwrap_or(u32::MAX)
    }

    // ---- the lock ---------------------------------------------------------

    /// What can verify a password on this machine, as the daemon reports it.
    ///
    /// `None` when nothing said — no daemon, or a daemon too old to have the
    /// fact. Parsed with the daemon's own parser, so the shell cannot hold an
    /// opinion the daemon would contradict.
    #[must_use]
    pub fn verifier(&self) -> Option<litric_sysd::platform::session::Verifier> {
        self.facts
            .auth_available
            .as_deref()
            .map(litric_sysd::platform::session::Verifier::parse)
    }

    /// Whether the lock may be offered: a password can be checked **and** the
    /// daemon did not refuse the lock.
    ///
    /// Two facts, both required, and the pairing is the point. A lock surface on
    /// a machine that cannot verify a credential is a screen with no way out —
    /// the failure `docs/design/wave1/04-authentication.md` §5 U1 names — and a
    /// daemon that was never reached reports neither fact, so it refuses too.
    #[must_use]
    pub fn can_lock(&self) -> bool {
        self.verifier()
            .is_some_and(litric_sysd::platform::session::Verifier::can_lock)
            && self.facts.auth_lock == Some(false)
    }

    /// Why locking is refused, in the daemon's own words.
    #[must_use]
    pub fn lock_refusal(&self) -> String {
        self.verifier().map_or_else(
            || {
                "The system daemon did not answer, so this session cannot check a \
                 password and will not pretend to lock the screen."
                    .to_owned()
            },
            |v| v.refusal().to_owned(),
        )
    }

    /// Lock the session. Returns whether the lock was taken.
    ///
    /// While locked the shell draws one surface and refuses input to everything
    /// else; the content that was on the screen is gone from the frame buffer on
    /// the next frame, because the lock surface starts by clearing it. The
    /// account the password is checked against is the one logind says is logged
    /// in. The single-user image has no PAM/logind desktop session yet, so its
    /// configured desktop account (`litric`) is the fallback.
    pub fn lock_now(&mut self) -> bool {
        if !self.can_lock() {
            return false;
        }
        self.lock.locked = true;
        if self.settings.values.on("notifications.clear-on-lock") {
            self.notifications.clear();
        }
        self.lock.verdict = crate::lock::Verdict::Idle;
        self.lock.user = self
            .facts
            .session_user
            .clone()
            .unwrap_or_else(|| "litric".to_owned());
        // The password field is immediately visible. Requiring an extra
        // Space/Enter press hid the only useful lock action behind a hint.
        self.lock.wake();
        self.lock.flow_ms = 0;
        self.pointer = Point::new(self.wm.screen().w / 2.0, self.wm.screen().h / 2.0);
        // Every overlay is a surface the lock does not draw, so an open menu
        // must not still be on the shell's mind when the session comes back.
        self.overlay = Overlay::None;
        self.context_menu = None;
        self.request_full_redraw();
        true
    }

    /// Leave the lock, because a password was verified.
    pub fn unlock(&mut self) {
        self.lock.locked = false;
        self.lock.back();
        self.lock.verdict = crate::lock::Verdict::Idle;
        self.request_full_redraw();
    }

    /// The daemon refused an attempt. The lock stays on, and the reason is the
    /// daemon's own sentence rather than a paraphrase.
    pub fn lock_failed(&mut self, reason: &str) {
        self.lock.verdict = crate::lock::Verdict::Failed(reason.to_owned());
        self.request_full_redraw();
    }

    /// The attempt to check: the account and the secret, taken out of the shell
    /// so the plain text exists in exactly one place for as long as it takes to
    /// hand it over.
    pub fn take_unlock(&mut self) -> Option<(String, crate::lock::Secret)> {
        if !self.lock.locked {
            return None;
        }
        let secret = self.lock.take_secret();
        if secret.is_empty() {
            return None;
        }
        self.lock.verdict = crate::lock::Verdict::Checking;
        self.request_full_redraw();
        Some((self.lock.user.clone(), secret))
    }

    /// Say that this machine cannot check a password, which the lock screen
    /// shows instead of collecting one it could never use.
    pub fn lock_no_verifier(&mut self, reason: &str) {
        self.lock.verdict = crate::lock::Verdict::NoVerifier(reason.to_owned());
        self.request_full_redraw();
    }

    /// Handle one event while the session is locked.
    ///
    /// The lock surface owns the keyboard and the pointer: nothing else in the
    /// shell sees an event while this returns `true`, which is what "locked"
    /// means on a framebuffer desktop with no client surfaces to hide.
    /// Input while the boot flow is on screen. The wizard's keys go to the
    /// wizard; the pointer works its hotspots exactly as the lock screen does,
    /// because a button here is a hotspot like any other.
    fn first_run_input(&mut self, ev: &InputEvent) -> Vec<Effect> {
        match ev {
            InputEvent::PointerMove { at, .. } => {
                self.pointer = *at;
                let now = self.hot.hit(at.x, at.y);
                if now != self.hovered {
                    for id in [self.hovered, now].into_iter().flatten() {
                        if let Some(r) = self.hot.rect_of(id) {
                            self.damage.push(r.inflate(2.0));
                        }
                    }
                    self.hovered = now;
                }
                let s = self.tokens.scale;
                self.damage.push(crate::ui::cursor_bounds(at.x, at.y, s));
                Vec::new()
            }
            InputEvent::PointerButton {
                button,
                pressed,
                at,
                ..
            } => {
                self.pointer = *at;
                if *button != PointerButton::Primary {
                    return Vec::new();
                }
                if !*pressed {
                    let released_on = self.hot.hit(at.x, at.y);
                    let was = self.pressed.take();
                    if let (Some(a), Some(b)) = (was, released_on) {
                        if a == b {
                            return self.activate(a);
                        }
                    }
                    return Vec::new();
                }
                if let Some(hit) = self.hot.hit(at.x, at.y) {
                    self.pressed = Some(hit);
                    if let Some(r) = self.hot.rect_of(hit) {
                        self.damage.push(r.inflate(2.0));
                    }
                }
                Vec::new()
            }
            other => {
                let mut effects = self
                    .first_run
                    .as_mut()
                    .map_or_else(Vec::new, |fr| fr.input(other));
                self.commit_first_run_language(&mut effects);
                self.request_full_redraw();
                effects
            }
        }
    }

    fn locked_input(&mut self, ev: &InputEvent) -> Vec<Effect> {
        match ev {
            InputEvent::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return Vec::new();
                }
                match key {
                    Key::Char(_) | Key::Backspace | Key::Enter | Key::Escape => self.lock_key(*key),
                    // Tab cycles the sheet's tabs, which is the one piece of
                    // navigation a mouse-only user cannot discover.
                    Key::Tab => {
                        if self.lock.edit {
                            let next = self.lock.tab.index().wrapping_add(1) % 4;
                            self.lock.tab = crate::lock::Tab::from_index(next);
                            self.request_full_redraw();
                        }
                        Vec::new()
                    }
                    _ => Vec::new(),
                }
            }
            InputEvent::TextInput { text } => {
                self.lock.push(text);
                self.request_full_redraw();
                Vec::new()
            }
            InputEvent::PointerMove { at, .. } => {
                self.pointer = *at;
                let now = self.hot.hit(at.x, at.y);
                if now != self.hovered {
                    for id in [self.hovered, now].into_iter().flatten() {
                        if let Some(r) = self.hot.rect_of(id) {
                            self.damage.push(r.inflate(2.0));
                        }
                    }
                    self.hovered = now;
                }
                let s = self.tokens.scale;
                self.damage.push(crate::ui::cursor_bounds(at.x, at.y, s));
                Vec::new()
            }
            InputEvent::PointerButton {
                button,
                pressed,
                at,
                ..
            } => {
                self.pointer = *at;
                if *button != PointerButton::Primary {
                    return Vec::new();
                }
                if !*pressed {
                    let released_on = self.hot.hit(at.x, at.y);
                    let was = self.pressed.take();
                    if let (Some(a), Some(b)) = (was, released_on) {
                        if a == b {
                            return self.activate(a);
                        }
                    }
                    return Vec::new();
                }
                if let Some(hit) = self.hot.hit(at.x, at.y) {
                    self.pressed = Some(hit);
                    if let Some(r) = self.hot.rect_of(hit) {
                        self.damage.push(r.inflate(2.0));
                    }
                }
                Vec::new()
            }
            // Scrolling and every event this build does not know are nothing
            // while locked: there is no other surface for them to mean.
            _ => Vec::new(),
        }
    }

    /// One key press on the lock screen.
    fn lock_key(&mut self, key: Key) -> Vec<Effect> {
        match key {
            Key::Escape => {
                if self.lock.edit {
                    self.lock.edit = false;
                } else {
                    self.lock.back();
                }
                self.request_full_redraw();
                Vec::new()
            }
            // Space and Enter both bring the field out, which is the design's
            // "press Space or Enter" hint. Space is a character, not a key of
            // its own, in this input layer.
            Key::Char(' ') | Key::Enter if self.lock.stage == crate::lock::Stage::Idle => {
                self.lock.wake();
                self.request_full_redraw();
                Vec::new()
            }
            Key::Enter => self.submit_unlock(),
            Key::Backspace => {
                self.lock.backspace();
                self.request_full_redraw();
                Vec::new()
            }
            Key::Char(_) => Vec::new(),
            _ => Vec::new(),
        }
    }

    /// Ask the caller to check the typed password.
    fn submit_unlock(&mut self) -> Vec<Effect> {
        if !self.lock.locked || self.lock.stage != crate::lock::Stage::Entry {
            return Vec::new();
        }
        if !self.can_lock() {
            let reason = self.lock_refusal();
            self.lock_no_verifier(&reason);
            return Vec::new();
        }
        self.take_unlock().map_or_else(Vec::new, |(user, secret)| {
            vec![Effect::VerifyPassword { user, secret }]
        })
    }

    /// Move time forward without any input, for the idle timers and the clock.
    pub fn tick(&mut self, now_ms: u64) {
        self.set_now(now_ms);
        // Automatic locking. It is checked here rather than in the input path
        // because the condition is the *absence* of input, which no input event
        // can report. A lock with no verifier refuses and says so, which is why
        // the return value is deliberately discarded: the user is told by the
        // screen, not by a swallowed error.
        let minutes = self.settings.values.num("session.auto-lock-minutes");
        if minutes > 0.0
            && self.mode == DesktopMode::Ready
            && !self.lock.locked
            && now_ms.saturating_sub(self.last_input_ms) >= (minutes as u64).saturating_mul(60_000)
        {
            let _ = self.lock_now();
        }
        // A still lock has no reason to repaint until its clock changes. When
        // the user enables motion, redraw at the animation's bounded 30 Hz.
        if self.lock.locked && self.lock.advance(now_ms) {
            self.request_full_redraw();
        }
        if self.terminal_pump() {
            // Output arrived in the terminal's scrollback: only the terminal's
            // own frame shows it. A command printing at 30 Hz used to repaint
            // the whole desktop at 30 Hz along with it.
            let frames: Vec<Rect> = self
                .wm
                .all()
                .iter()
                .filter(|w| w.app_id == App::Terminal.app_id())
                .map(|w| to_rect(w.frame))
                .collect();
            let reach = self.shadow_reach();
            for r in frames {
                self.damage.push(r.inflate(reach));
            }
        }
        // The overlay springs follow the overlay state and advance one step.
        // sync() is idempotent, so doing it here means no call site that
        // opens or closes an overlay has to remember the animation exists.
        self.anims.sync(
            self.overlay.is_open(),
            self.context_menu.is_some(),
            matches!(self.overlay, Overlay::Launcher { .. }),
        );
        // Focus elevation belongs to tick for the same reason: the window
        // list is state, the draw side only samples values.
        {
            let focused = self.wm.focused();
            let windows: Vec<(u32, bool)> = self
                .wm
                .visible()
                .iter()
                .map(|w| (w.id, Some(w.id) == focused))
                .collect();
            for (id, is_focused) in windows {
                self.anims.focus.elevation(id, is_focused);
            }
        }
        // The springs advance by the wall time that actually passed. The frame
        // loop decides how much that is — 8 ms under a dragged window, 16 ms
        // active, 33 ms idle, more when nothing is happening — so the motion
        // tokens' settle budgets measure milliseconds rather than frames, and
        // an animation looks the same at every pace.
        let dt = self.animation_step_ms(now_ms);
        let moved = self.anims.step(dt);
        // A spring that moved is damage (BUG-0019), and damage at the
        // granularity the movement actually has. `step` reports the value
        // changing, not the spring still going, so a spring that reaches its
        // settle budget and snaps the rest of the way in a stalled frame still
        // lands here with pixels that must be drawn.
        if moved.overlays {
            self.request_full_redraw();
        }
        // A focus spring raises or lowers one window: its shadow and glow
        // change inside its own frame plus the shadow's reach, and nothing
        // else on screen moves with it. Switching windows used to repaint all
        // 1.02M pixels of a 1280x800 desktop to move one shadow.
        let reach = self.shadow_reach();
        for id in moved.focus {
            if let Some(w) = self.wm.window(id) {
                let r = to_rect(w.frame).inflate(reach);
                self.damage.push(r);
            }
        }
        // The boot flow runs its own spring and hands the screen over when it
        // is done: the splash leaves once the system answered and the reveal
        // finished, the wizard leaves once its password is set. Both are
        // driven by the same `dt` the overlay springs get, so the wordmark
        // takes the same milliseconds at every frame pace.
        if self.first_run.is_some() {
            let mut finished = false;
            if let Some(fr) = &mut self.first_run {
                let live = matches!(
                    fr.page,
                    crate::firstrun::Page::Splash | crate::firstrun::Page::Preparing
                );
                let stepped = fr.tick(dt);
                if fr.page == crate::firstrun::Page::Splash && fr.splash_done(now_ms) {
                    if fr.wizard == Some(true) {
                        fr.go(crate::firstrun::Page::Welcome);
                    } else {
                        finished = true;
                    }
                }
                if stepped || live {
                    self.request_full_redraw();
                }
            }
            if finished {
                self.first_run = None;
                self.request_full_redraw();
            }
        }
        // The Defender's scan and the browser's engine pump run between
        // frames, in bounded slices, so neither can starve the compositor.
        // Both are gated on the platform having a filesystem/socket: on a
        // Windows CI host the bridge refuses to open and the scan never runs,
        // by name, rather than panicking in the render loop.
        if self.defender.running && cfg!(unix) {
            self.scan_step(8);
        }
        self.pump_browser();
        if self.notifications.expire(now_ms) > 0 {
            self.request_full_redraw();
        }
    }

    /// Tick the shell `dt_ms` after the time of its own last tick.
    ///
    /// The frame loop has a wall clock and calls [`Self::tick`] with a real
    /// timestamp. A caller that has none — a test, a replay, a benchmark, the
    /// e2e journeys — has to say how much time passed, and this is that: the
    /// same tick expressed as a delta.
    ///
    /// The delta is what the springs see, **including on the first call**,
    /// because a caller that names the elapsed time is not guessing at it. The
    /// base is the last *tick* rather than the current time, so a harness that
    /// hands `handle` its own fixed timestamps cannot drag the animation clock
    /// backwards and freeze every spring at zero.
    pub fn tick_after(&mut self, dt_ms: u32) {
        let base = self.last_step_ms.unwrap_or(self.now_ms);
        self.last_step_ms = Some(base);
        self.tick(base.saturating_add(u64::from(dt_ms)));
    }

    /// Move the shell's clock, damaging the top bar when the clock text it
    /// shows changes.
    /// Start the boot flow: the splash every boot shows, and — when the
    /// machine still needs its language and password — the first-run wizard
    /// after it. The caller says which it is, from the daemon's own answer;
    /// the shell only draws the walk it is told to give.
    pub fn begin_boot_flow(&mut self, now_ms: u64, wizard: Option<bool>) {
        let mut fr = crate::firstrun::FirstRun::new(now_ms, wizard);
        // The wizard opens on the language the machine already speaks; its
        // cards only move when they are chosen.
        fr.language = self.settings.language();
        self.first_run = Some(fr);
        self.request_full_redraw();
    }

    /// The wizard carries its own language choice, and this commits it the way
    /// the Settings row does — apply, then persist — so the choice survives the
    /// boot that asked for it. Idempotent: when the settings already say the
    /// same language there is nothing to write.
    fn commit_first_run_language(&mut self, effects: &mut Vec<Effect>) {
        let Some(fr) = self.first_run.as_ref() else {
            return;
        };
        let language = fr.language;
        if self.settings.language() == language {
            return;
        }
        if self.settings.set_language(language).is_ok() {
            self.apply_settings();
            effects.extend(self.persist_settings());
        }
    }

    /// The shell's clock in milliseconds — the same time [`Self::handle`] and
    /// [`Self::tick`] are given — so a draw that animates from the clock (the
    /// boot splash) moves in step with the springs.
    #[must_use]
    pub const fn now_ms(&self) -> u64 {
        self.now_ms
    }

    /// The daemon answered the wizard's password request. On success the boot
    /// flow is over and the desktop takes the screen; on refusal the wizard
    /// shows the daemon's own words and keeps asking.
    pub fn password_set_result(&mut self, ok: bool, reason: Option<String>) {
        let done = self
            .first_run
            .as_mut()
            .is_some_and(|fr| fr.password_result(ok, reason));
        if done {
            self.first_run = None;
        }
        self.request_full_redraw();
    }

    fn set_now(&mut self, now_ms: u64) {
        self.now_ms = now_ms;
        // A caller whose clock is not the one the boot flow started with — a
        // test or replay with its own timestamps — would freeze the reveal at
        // zero forever, because the flow measures itself from its start. Rebase
        // to the clock actually in use instead of trusting a clock that jumped.
        if let Some(fr) = &mut self.first_run {
            if fr.started_ms > now_ms {
                fr.started_ms = now_ms;
            }
        }
        let clock = self.regional.clock(self.clock_seconds());
        if clock != self.drawn_clock {
            self.drawn_clock = clock;
            if self.lock.locked {
                self.request_full_redraw();
            } else {
                let bar = self.top_bar();
                self.damage.push(bar);
            }
        }
    }

    /// When the clock in the top bar will next show different text, in
    /// milliseconds since the epoch.
    #[must_use]
    pub fn next_clock_change_ms(&self) -> u64 {
        let now = self.clock_seconds();
        let at = now.saturating_add(self.regional.seconds_until_clock_changes(now));
        u64::try_from(at).unwrap_or(0).saturating_mul(1000)
    }

    /// Resize the display.
    pub fn set_display(&mut self, width: f32, height: f32, scale: f32) {
        self.tokens.scale = scale;
        self.wm
            .set_display(width, height, self.tokens.px(Self::TOP_BAR_LOGICAL));
        self.pointer = Point::new(
            self.pointer.x.min(width - 1.0),
            self.pointer.y.min(height - 1.0),
        );
        self.request_full_redraw();
    }

    /// Mark the whole screen for repainting.
    pub fn request_full_redraw(&mut self) {
        self.full_redraw = true;
    }

    /// Mark a region for repainting.
    pub fn damage(&mut self, r: Rect) {
        self.damage.push(r);
    }

    /// Whether the next frame must repaint everything.
    ///
    /// Also true for any repaint while the diagnostic overlay is showing: its
    /// lines change with every frame, and repainting only part of it would
    /// leave old and new numbers side by side.
    #[must_use]
    pub fn needs_full_redraw(&self) -> bool {
        self.full_redraw || (self.diagnostics && self.is_dirty())
    }

    /// Whether anything at all needs repainting.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.full_redraw
            || !self.damage.is_empty()
            || self.wm.is_dirty()
            || self.wm.focused() != self.drawn_focus
    }

    /// How far a window's drop shadow reaches past its frame, in pixels.
    ///
    /// `views::draw_windows` gives a focused window a shadow of spread 30 and
    /// offset 12 (logical); [`crate::canvas::Canvas::soft_shadow`] grows its
    /// outermost ring by 11/12 of the spread, and the ring's edge is
    /// anti-aliased by half a pixel. The window manager damages a frame grown
    /// by 8 or 24, which stops short of that: closing or moving a window left
    /// the bottom of its shadow on the desktop.
    fn shadow_reach(&self) -> f32 {
        self.tokens.px(30.0 + 12.0) + 1.0
    }

    /// Take everything that needs repainting.
    ///
    /// Window-manager damage is grown to cover the window's shadow. When focus
    /// has moved since the last call, the previously focused window, the newly
    /// focused one (shadows included) and the top bar, which names the focused
    /// window, are added.
    pub fn take_damage(&mut self) -> Vec<Rect> {
        let mut out = core::mem::take(&mut self.damage);
        let reach = self.shadow_reach();
        for r in self.wm.take_damage() {
            out.push(to_rect(r).inflate(reach));
        }
        let focus = self.wm.focused();
        if focus != self.drawn_focus {
            out.push(self.top_bar());
            for id in [self.drawn_focus, focus].into_iter().flatten() {
                if let Some(w) = self.wm.window(id) {
                    out.push(to_rect(w.frame).inflate(reach));
                }
            }
            self.drawn_focus = focus;
        }
        self.full_redraw = false;
        out
    }

    /// Replace the facts, damaging only what shows a value that changed.
    ///
    /// The top bar shows CPU and memory as rounded text, the battery level and
    /// the network state; it is damaged when one of those, as displayed,
    /// changes. An Activity window on the active workspace shows the meters
    /// with their bars, uptime as text, and the root, verity and source lines;
    /// each such window is damaged when one of those changes. Anything else —
    /// the CPU count, the trash, uptime with no Activity window showing —
    /// damages nothing.
    pub fn set_facts(&mut self, facts: SystemFacts) {
        if facts == self.facts {
            return;
        }
        let old = core::mem::replace(&mut self.facts, facts);
        if top_bar_view(&old) != top_bar_view(&self.facts) {
            let bar = self.top_bar();
            self.damage.push(bar);
        }
        if activity_view(&old) != activity_view(&self.facts) {
            let frames: Vec<Rect> = self
                .wm
                .visible()
                .iter()
                .filter(|w| w.app_id == App::Activity.app_id())
                .map(|w| to_rect(w.frame))
                .collect();
            self.damage.extend(frames);
        }
    }

    /// The overlay's region, for damage when it opens or closes.
    fn overlay_damage(&mut self) {
        // Overlays are large and can move; repainting the whole screen when one
        // opens or closes is cheaper than tracking their exact extent and
        // getting it subtly wrong.
        self.request_full_redraw();
    }

    // ---- input ------------------------------------------------------------

    /// Handle one input event. Returns anything `main` must act on.
    pub fn handle(&mut self, ev: &InputEvent, now_ms: u64) -> Vec<Effect> {
        self.set_now(now_ms);
        if ev.is_user_activity() {
            let was = self.life();
            self.last_input_ms = now_ms;
            // Coming back from idle repaints: the frame on screen may be up to
            // half a second old, and the first thing the user does is look at
            // the clock.
            if was != LifeState::Active {
                self.request_full_redraw();
            }
        }
        // The boot flow owns the screen and every event while it is up: the
        // desktop behind it does not exist yet, and a first-run wizard that
        // leaked a Meta press through to the launcher would be half a desktop.
        if self.first_run.is_some() {
            return self.first_run_input(ev);
        }
        if !self.mode.is_interactive() {
            return Vec::new();
        }
        // The lock owns every event while it is on. Nothing below this line --
        // an overlay, a focused window, a global shortcut -- can see one, which
        // is what hiding the session means on a framebuffer desktop with no
        // client surfaces to take away.
        if self.lock.locked {
            return self.locked_input(ev);
        }

        match ev {
            InputEvent::KeyDown {
                key, mods, repeat, ..
            } => self.key_down(*key, *mods, *repeat),
            InputEvent::KeyUp { key, mods, .. } => self.key_up(*key, *mods),
            InputEvent::TextInput { text } => {
                self.text_input(text);
                Vec::new()
            }
            InputEvent::PointerMove { at, .. } => {
                self.pointer_move(*at);
                Vec::new()
            }
            InputEvent::PointerButton {
                button,
                pressed,
                at,
                clicks,
                mods,
            } => self.pointer_button(*button, *pressed, *at, *clicks, *mods),
            InputEvent::Scroll { dy, at, .. } => {
                self.scroll(*dy, *at);
                Vec::new()
            }
            // InputEvent is non-exhaustive: a backend that gains touch or a
            // controller must not be able to make this fail to compile, and an
            // unhandled event is correctly nothing rather than a guess.
            _ => Vec::new(),
        }
    }

    fn key_down(&mut self, key: Key, mods: Modifiers, repeat: bool) -> Vec<Effect> {
        if key.is_modifier() {
            if matches!(key, Key::Meta(_)) && !repeat {
                self.meta_tap.press();
            }
            return Vec::new();
        }
        self.meta_tap.interfere();

        // One focus order for the controls each frame actually drew. Tab and
        // Shift+Tab work across app chrome and panel controls; Enter activates
        // the focused control. Text fields keep Enter's app-specific meaning.
        if key == Key::Tab
            && !repeat
            && !mods.alt
            && !matches!(&self.overlay, Overlay::Switcher { .. })
        {
            self.move_keyboard_focus(mods.shift);
            return Vec::new();
        }
        if key == Key::Enter
            && !repeat
            && !self.overlay.is_open()
            && !self.keyboard_focus_is_text_field()
            && self
                .keyboard_focus
                .is_some_and(|hot| self.hot.rect_of(hot).is_some())
        {
            if let Some(hot) = self.keyboard_focus {
                return self.activate(hot);
            }
        }

        // The launcher and the switcher eat navigation keys before the global
        // table sees them, so Escape closes the launcher rather than doing
        // whatever Escape does on the desktop.
        if self.overlay.is_open() {
            if let Some(handled) = self.overlay_key(key, mods) {
                return handled;
            }
            // The overlay did not want the key: fall through to the global
            // table, which is how Alt+Tab still works over a launcher search
            // that contains no letters the launcher cares about.
            let Some(action) = self.shortcuts.resolve(key, mods) else {
                return Vec::new();
            };
            return self.perform(action);
        }

        // A focused application window gets the key first. Its unhandled
        // keys fall through to the global table below, which is what makes
        // Win+E open Files even while Files has focus.
        if let Some(handled) = self.app_key(key, mods) {
            return handled;
        }

        let Some(action) = self.shortcuts.resolve(key, mods) else {
            return Vec::new();
        };
        self.perform(action)
    }

    fn key_up(&mut self, key: Key, _mods: Modifiers) -> Vec<Effect> {
        if matches!(key, Key::Meta(_)) {
            if self.meta_tap.release() {
                return self.perform(Action::ToggleLauncher);
            }
            return Vec::new();
        }
        // Releasing Alt commits the window switcher, which is what makes
        // Alt+Tab+Tab+release land on the third window rather than the second.
        if matches!(key, Key::Alt(_)) && self.switcher_holding {
            self.switcher_holding = false;
            if let Overlay::Switcher { selected } = self.overlay {
                let order: Vec<WindowId> = self.wm.switcher_order().iter().map(|w| w.id).collect();
                if let Some(id) = order.get(selected).copied() {
                    self.wm.focus_window(id);
                    self.wm.unminimize(id);
                }
            }
            self.overlay = Overlay::None;
            self.overlay_damage();
        }
        Vec::new()
    }

    /// Keys the open overlay consumes. `None` means it did not want the key.
    fn overlay_key(&mut self, key: Key, mods: Modifiers) -> Option<Vec<Effect>> {
        match &mut self.overlay {
            Overlay::Launcher { query, selected } => match key {
                Key::Escape => {
                    self.overlay = Overlay::None;
                    self.overlay_damage();
                    Some(Vec::new())
                }
                Key::Down => {
                    let n = App::search(query).len();
                    if n > 0 {
                        *selected = selected.saturating_add(1).checked_rem(n).unwrap_or(0);
                    }
                    self.overlay_damage();
                    Some(Vec::new())
                }
                Key::Up => {
                    let n = App::search(query).len();
                    if n > 0 {
                        *selected = selected.checked_sub(1).unwrap_or(n.saturating_sub(1));
                    }
                    self.overlay_damage();
                    Some(Vec::new())
                }
                Key::Enter => {
                    let q = query.clone();
                    let idx = *selected;
                    let app = App::search(&q).get(idx).copied();
                    self.overlay = Overlay::None;
                    self.overlay_damage();
                    Some(app.map_or_else(Vec::new, |a| self.open_app(a)))
                }
                Key::Backspace => {
                    query.pop();
                    *selected = 0;
                    self.overlay_damage();
                    Some(Vec::new())
                }
                _ => None,
            },
            Overlay::Switcher { selected } => match key {
                Key::Tab => {
                    let n = self.wm.switcher_order().len();
                    if n > 0 {
                        *selected = selected.saturating_add(1).checked_rem(n).unwrap_or(0);
                    }
                    self.overlay_damage();
                    Some(Vec::new())
                }
                Key::Escape => {
                    self.overlay = Overlay::None;
                    self.switcher_holding = false;
                    self.overlay_damage();
                    Some(Vec::new())
                }
                _ => None,
            },
            Overlay::QuickSettings
            | Overlay::Notifications
            | Overlay::Power
            | Overlay::Litro => {
                if key == Key::Escape {
                    self.overlay = Overlay::None;
                    self.overlay_damage();
                    Some(Vec::new())
                } else {
                    None
                }
            }
            Overlay::None => {
                if let Some(handled) = self.app_key(key, mods) {
                    Some(handled)
                } else {
                    // An unhandled key in an app window falls through to the
                    // global table, which is what makes Win+E work while Files
                    // has focus.
                    let Some(action) = self.shortcuts.resolve(key, mods) else {
                        return Some(Vec::new());
                    };
                    Some(self.perform(action))
                }
            }
        }
    }

    /// Keys routed to the focused application window (no overlay open).
    fn app_key(&mut self, key: Key, mods: Modifiers) -> Option<Vec<Effect>> {
        let focused = self.wm.focused();
        let app_id = focused
            .and_then(|id| self.wm.window(id))
            .map(|w| w.app_id.clone())?;
        match app_id.as_str() {
            "com.litric.settings" => match key {
                Key::Char('l') if mods.ctrl => Some(self.toggle_language()),
                Key::Char('t') if mods.ctrl => Some(self.cycle_theme()),
                Key::Char('m') if mods.ctrl => Some(self.toggle_reduced_motion()),
                Key::Down => {
                    self.settings.move_selection(true);
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Up => {
                    self.settings.move_selection(false);
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::PageDown => {
                    for _ in 0..5 {
                        self.settings.move_selection(true);
                    }
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::PageUp => {
                    for _ in 0..5 {
                        self.settings.move_selection(false);
                    }
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Enter => {
                    if self.settings.editing.is_some() {
                        self.settings.end_edit();
                        self.request_full_redraw();
                        return Some(Vec::new());
                    }
                    if self.settings.activate_selected() {
                        self.apply_settings();
                        return Some(self.persist_settings());
                    }
                    Some(Vec::new())
                }
                Key::Right => {
                    if self.settings.step_selected(true) {
                        self.apply_settings();
                        return Some(self.persist_settings());
                    }
                    let page = self.settings.page.next();
                    self.settings.page = page;
                    self.settings.scroll = 0;
                    self.settings.selection = None;
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Left => {
                    if self.settings.editing.is_some() {
                        return Some(Vec::new());
                    }
                    let page = self.settings.page.previous();
                    self.settings.page = page;
                    self.settings.scroll = 0;
                    self.settings.selection = None;
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Backspace => {
                    if self.settings.editing.is_some() {
                        if self.settings.backspace_edit() {
                            self.apply_settings();
                            return Some(self.persist_settings());
                        }
                    } else if self.settings.selection.is_none() {
                        self.settings.query.pop();
                    }
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Escape => {
                    if self.settings.editing.take().is_some() {
                        self.request_full_redraw();
                    } else if !self.settings.query.is_empty() {
                        self.settings.query.clear();
                        self.settings.scroll = 0;
                        self.request_full_redraw();
                    }
                    Some(Vec::new())
                }
                _ => None,
            },
            "com.litric.browser" => match key {
                Key::Escape => {
                    self.browser.omnibox_focus = false;
                    Some(Vec::new())
                }
                Key::Enter => {
                    if let Some(url) = self.browser.commit_omnibox() {
                        self.browser_navigate(&url);
                        self.request_full_redraw();
                        Some(vec![Effect::PersistBrowserData])
                    } else {
                        self.request_full_redraw();
                        Some(Vec::new())
                    }
                }
                _ => None,
            },
            "com.litric.terminal" => match key {
                Key::Char('l') if mods.ctrl => {
                    self.terminal.clear_screen();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Enter => {
                    self.terminal_run();
                    Some(Vec::new())
                }
                Key::Up => {
                    self.terminal.history_up();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Down => {
                    self.terminal.history_down();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Backspace => {
                    self.terminal.input.pop();
                    self.terminal.history_cursor = None;
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Escape => {
                    self.terminal.focused = false;
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                _ => None,
            },
            "com.litric.activity" => match key {
                Key::Backspace => {
                    self.activity.prefs.filter_text.pop();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Escape => {
                    self.activity.prefs.filter_text.clear();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                _ => None,
            },
            "com.litric.files" => match key {
                Key::Backspace => {
                    // Backspace in a dialog edits the field, not the filter.
                    if self.files.dialog.is_some() {
                        let mut field = self.files.dialog_field.clone();
                        let mut sel = self.files.dialog_selection;
                        let _outcome = crate::apps::dialog_field_key(
                            &mut field,
                            &mut sel,
                            Key::Backspace,
                            Modifiers::NONE,
                        );
                        self.files.dialog_field = field;
                        self.files.dialog_selection = sel;
                    } else {
                        self.files.filter.pop();
                    }
                    Some(Vec::new())
                }
                Key::Down => {
                    let n = self.files.visible().len();
                    if n > 0 {
                        let next = self
                            .files
                            .selected
                            .map_or(0, |i| i.saturating_add(1).min(n.saturating_sub(1)));
                        self.files.select_row(next);
                    }
                    Some(Vec::new())
                }
                Key::Up => {
                    if let Some(current) = self.files.selected {
                        self.files.select_row(current.saturating_sub(1));
                    }
                    Some(Vec::new())
                }
                Key::Escape => {
                    // Escape in a dialog cancels the dialog; otherwise it
                    // clears the filter, the Files convention.
                    if self.files.dialog.is_some() {
                        self.files.close_dialog();
                    } else {
                        self.files.filter.clear();
                    }
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                // Files keyboard: the Explorer set. F2 renames, Delete trashes
                // (with a confirm), Ctrl+C/X/V drive the clipboard, Ctrl+F
                // focuses the filter.
                Key::Function(2) => {
                    self.files.ask_rename();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Delete if self.files.dialog.is_none() => {
                    let paths = self.files.selected_paths();
                    if mods.ctrl {
                        // Shift+Delete would purge; plain Ctrl+Delete is
                        // nothing. A permanent delete needs a confirm first.
                        if let Some(action) = selection_action(
                            &paths,
                            crate::apps::FilesOp::Delete,
                            crate::apps::FilesOp::DeleteMany,
                        ) {
                            self.files.ask_confirm(
                                "Delete permanently",
                                &format!("Delete {} selected item(s) permanently?", paths.len()),
                                action,
                            );
                            self.request_full_redraw();
                        }
                        Some(Vec::new())
                    } else if mods.shift {
                        if let Some(action) = selection_action(
                            &paths,
                            crate::apps::FilesOp::Delete,
                            crate::apps::FilesOp::DeleteMany,
                        ) {
                            self.files.ask_confirm(
                                "Delete permanently",
                                &format!("Delete {} selected item(s) permanently?", paths.len()),
                                action,
                            );
                            self.request_full_redraw();
                        }
                        Some(Vec::new())
                    } else {
                        if let Some(action) = selection_action(
                            &paths,
                            crate::apps::FilesOp::Trash,
                            crate::apps::FilesOp::TrashMany,
                        ) {
                            self.trash_with_confirm(
                                "Move to trash",
                                &format!("Move {} selected item(s) to the trash?", paths.len()),
                                action,
                            );
                        }
                        Some(Vec::new())
                    }
                }
                Key::Char('c') if mods.ctrl => {
                    self.files.copy_selection();
                    Some(Vec::new())
                }
                Key::Char('x') if mods.ctrl => {
                    self.files.cut_selection();
                    Some(Vec::new())
                }
                Key::Char('v') if mods.ctrl => {
                    if let Some((cut, p)) = self.files.clipboard.clone() {
                        let op = if cut {
                            crate::apps::FilesOp::PasteCut
                        } else {
                            crate::apps::FilesOp::PasteCopy
                        };
                        let what = if cut { "Move" } else { "Copy" };
                        self.files.ask_confirm(
                            what,
                            &format!("{what} {p} into {}?", self.files.path),
                            op,
                        );
                        self.request_full_redraw();
                    }
                    Some(Vec::new())
                }
                Key::Char('f') if mods.ctrl => {
                    self.files.focus = crate::apps::FilesFocus::Table;
                    Some(Vec::new())
                }
                Key::Char('a') if mods.ctrl => {
                    self.files.select_all_visible();
                    Some(Vec::new())
                }
                Key::Enter => {
                    // Open: folders navigate, files go through open_files_entry.
                    if let Some(p) = self.files.selected_path() {
                        let is_dir = self
                            .files
                            .selection()
                            .is_some_and(|r| r.kind == crate::apps::FileKind::Dir);
                        if is_dir {
                            self.files.navigate(&p);
                        } else {
                            return Some(self.open_files_entry(&p));
                        }
                    }
                    Some(Vec::new())
                }
                _ => None,
            },
            "com.litric.trash" => match key {
                Key::Backspace => {
                    self.trash.filter.pop();
                    if !self.trash.has_visible_selection() {
                        self.trash.selected = None;
                    }
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Escape => {
                    if self.trash.confirm.take().is_none() {
                        self.trash.filter.clear();
                    }
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Down => {
                    let visible = self.trash.visible_indices();
                    let next = self
                        .trash
                        .selected
                        .and_then(|selected| visible.iter().position(|index| *index == selected))
                        .map_or(0, |position| {
                            position
                                .saturating_add(1)
                                .min(visible.len().saturating_sub(1))
                        });
                    self.trash.selected = visible.get(next).copied();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Up => {
                    let visible = self.trash.visible_indices();
                    let previous = self
                        .trash
                        .selected
                        .and_then(|selected| visible.iter().position(|index| *index == selected))
                        .unwrap_or(0)
                        .saturating_sub(1);
                    self.trash.selected = visible.get(previous).copied();
                    self.request_full_redraw();
                    Some(Vec::new())
                }
                Key::Enter => Some(self.perform_menu(crate::apps::MenuAction::Restore)),
                Key::Delete => Some(self.perform_menu(crate::apps::MenuAction::PurgeTrashItem)),
                _ => None,
            },
            _ => None,
        }
    }

    fn text_input(&mut self, text: &str) {
        // A modal dialog on the desktop owns the keyboard, and it is asked
        // **first** — before any window, focused or not. It used to be reached
        // only through `text_into_app`, which returns early when no window has
        // focus, so a name typed into the new-folder dialog of an empty desktop
        // went nowhere and the dialog then refused an empty name. Found by
        // `journey_right_click_desktop_menu_creates_a_folder_via_dialog`, which
        // is the only test that opens a desktop dialog without a window first.
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if self.desktop_dialog.is_some() && !clean.is_empty() {
            self.desktop_dialog_field.push_str(&clean);
            return;
        }
        // Otherwise a focused app text field eats characters before overlays do —
        // the omnibox, the Files filter and the Files dialogs.
        if self.text_into_app(text) {
            return;
        }
        if let Overlay::Launcher { query, selected } = &mut self.overlay {
            // Control characters never reach a text field: a Tab or a newline
            // in a search box is a rendering problem waiting to happen.
            let clean: String = text.chars().filter(|c| !c.is_control()).collect();
            if !clean.is_empty() {
                query.push_str(&clean);
                *selected = 0;
                self.overlay_damage();
            }
        }
    }

    /// Feed typed characters into the focused app window's field. Returns
    /// whether the text was consumed.
    fn text_into_app(&mut self, text: &str) -> bool {
        let focused = self.wm.focused();
        let Some(app_id) = focused
            .and_then(|id| self.wm.window(id))
            .map(|w| w.app_id.clone())
        else {
            return false;
        };
        if app_id == "com.litric.settings" {
            // A text row being edited takes characters first; otherwise the
            // search field does. Typing anywhere in the window is a search,
            // which is the behaviour every settings window has taught people.
            if self.settings.editing.is_some() {
                if self.settings.type_edit(text) {
                    self.apply_settings();
                }
                self.request_full_redraw();
                return true;
            }
            let clean: String = text.chars().filter(|c| !c.is_control()).collect();
            if !clean.is_empty() {
                self.settings.query.push_str(&clean);
                self.settings.scroll = 0;
                self.settings.selection = None;
                self.request_full_redraw();
            }
            return true;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if clean.is_empty() {
            return false;
        }
        // A modal dialog on the focused window eats text first.
        if self.files.dialog.is_some() && self.files.dialog_focus {
            self.files.dialog_field.push_str(&clean);
            true
        } else if self.desktop_dialog.is_some() {
            self.desktop_dialog_field.push_str(&clean);
            true
        } else {
            match app_id.as_str() {
                "com.litric.browser" if self.browser.omnibox_focus => {
                    // A fresh focus has the old address "selected": the
                    // first character replaces it.
                    if self.browser.omnibox_fresh {
                        self.browser.omnibox.clear();
                        self.browser.omnibox_fresh = false;
                    }
                    self.browser.omnibox.push_str(&clean);
                    true
                }
                "com.litric.terminal" if self.terminal.focused => {
                    self.terminal.input.push_str(&clean);
                    self.terminal.history_cursor = None;
                    true
                }
                "com.litric.activity" => {
                    self.activity.prefs.filter_text.push_str(&clean);
                    self.request_full_redraw();
                    true
                }
                "com.litric.trash" => {
                    self.trash.filter.push_str(&clean);
                    if !self.trash.has_visible_selection() {
                        self.trash.selected = None;
                    }
                    self.request_full_redraw();
                    true
                }
                "com.litric.files" => {
                    self.files.filter.push_str(&clean);
                    true
                }
                _ => false,
            }
        }
    }

    fn pointer_move(&mut self, at: Point) {
        let before = self.pointer;
        self.pointer = at;

        if self.wm.is_dragging() {
            self.wm.update_drag(at.x, at.y);
        }

        let now = self.hot.hit(at.x, at.y);
        if now != self.hovered {
            // Repaint what was hovered and what is now, so the highlight moves
            // without repainting the whole screen.
            for id in [self.hovered, now].into_iter().flatten() {
                if let Some(r) = self.hot.rect_of(id) {
                    self.damage.push(r.inflate(2.0));
                }
            }
            self.hovered = now;
        }

        // A window's buttons show their glyph while the pointer is within
        // 4 logical px of them (views::draw_windows). They are not hotspots, so
        // crossing into or out of that zone repaints the button here; the
        // cursor rectangles below do not reach the far side of the button.
        let near = self.tokens.px(4.0);
        let chrome = *self.wm.chrome();
        let frames: Vec<WmRect> = self.wm.visible().iter().map(|w| w.frame).collect();
        for frame in frames {
            for n in 0..3u32 {
                let b = to_rect(chrome.button_rect(&frame, n));
                let zone = b.inflate(near);
                if zone.contains(before.x, before.y) != zone.contains(at.x, at.y) {
                    self.damage.push(b.inflate(2.0));
                }
            }
        }

        // The cursor itself: repaint where it was and where it is.
        let s = self.tokens.scale;
        self.damage
            .push(crate::ui::cursor_bounds(before.x, before.y, s));
        self.damage.push(crate::ui::cursor_bounds(at.x, at.y, s));
    }

    fn pointer_button(
        &mut self,
        button: PointerButton,
        pressed: bool,
        at: Point,
        clicks: u8,
        _mods: Modifiers,
    ) -> Vec<Effect> {
        self.pointer = at;
        if !pressed {
            let released_on = self.hot.hit(at.x, at.y);
            let was = self.pressed.take();
            if self.wm.is_dragging() {
                self.wm.end_drag(at.x, at.y);
                return Vec::new();
            }
            if let (Some(a), Some(b)) = (was, released_on) {
                if a == b {
                    return self.activate(a);
                }
            }
            if was.is_some() {
                self.request_full_redraw();
            }
            return Vec::new();
        }

        if button == PointerButton::Secondary {
            // A right-click opens the context menu for whatever surface the
            // pointer is on: wallpaper, Files, Browser, Trash. Menus are the
            // shell's own actions, so a row can never exist that does nothing.
            if self.overlay.is_open() {
                self.overlay = Overlay::None;
                self.overlay_damage();
            }
            if pressed {
                let surface = self.menu_surface_at(at);
                // Explorer's rule: right-clicking a row also selects it, so
                // the menu's Copy / Rename / Trash act on what was clicked.
                if surface == crate::apps::MenuSurface::FilesRow {
                    if let Some(Hot::Files(crate::apps::FilesHot::Row(i))) =
                        self.hot.hit(at.x, at.y)
                    {
                        self.files.select_row(i);
                    }
                }
                self.context_menu =
                    Some(crate::apps::ContextMenu::for_surface(surface, (at.x, at.y)));
                self.request_full_redraw();
            }
            return Vec::new();
        }

        self.keyboard_focus = None;

        // The top bar, overlays, menus and the dock sit above every window —
        // but a desktop icon does not. An icon beneath a window is invisible
        // and must not answer its clicks: fall through to the window manager
        // (title-bar drag, chrome buttons, body focus) instead of launching
        // a shortcut the user cannot even see. This was the reported bug:
        // pressing a window's title bar opened the icon hiding behind it.
        let hit = match self.hot.hit(at.x, at.y) {
            Some(h @ Hot::DesktopIcon(_)) => {
                if self.wm.hit_test(at.x, at.y).is_some() {
                    None
                } else {
                    Some(h)
                }
            }
            other => other,
        };
        if let Some(hot) = hit {
            self.pressed = Some(hot);
            if let Some(r) = self.hot.rect_of(hot) {
                self.damage.push(r.inflate(2.0));
            }
            return Vec::new();
        }

        // Clicking outside an open overlay dismisses it, which is what every
        // menu everywhere does.
        if self.overlay.is_open() {
            self.overlay = Overlay::None;
            self.overlay_damage();
            return Vec::new();
        }

        if let Some(hit) = self.wm.hit_test(at.x, at.y) {
            self.wm.focus_window(hit.window);
            match hit.zone {
                HitZone::Close => {
                    self.wm.close(hit.window);
                }
                HitZone::Minimize => {
                    self.wm.minimize(hit.window);
                }
                HitZone::Maximize => {
                    self.wm.toggle_maximize(hit.window);
                }
                HitZone::TitleBar => {
                    if clicks >= 2 {
                        self.wm.toggle_maximize(hit.window);
                    } else {
                        self.wm.begin_drag(hit, at.x, at.y);
                    }
                }
                HitZone::Resize(_) => {
                    self.wm.begin_drag(hit, at.x, at.y);
                }
                // HitZone is non-exhaustive. A zone this build does not know
                // about focuses the window and does nothing else, which is the
                // safe default for a click on an unfamiliar part of a frame.
                _ => {}
            }
        }
        Vec::new()
    }

    fn scroll(&mut self, dy: f32, at: Point) {
        // Scrolling over the top bar's workspace area moves between workspaces,
        // which is a gesture people expect and costs nothing to support.
        if self.top_bar().contains(at.x, at.y) && self.wm.cycle_workspace(dy > 0.0) {
            self.request_full_redraw();
            return;
        }
        // A wheel over the Settings pane scrolls its rows. The pane rectangle
        // is the one the last frame actually drew, so the wheel cannot scroll a
        // pane that is not on screen.
        if let Some(pane) = self.settings.pane {
            if pane.contains(at.x, at.y) && self.focused_app() == Some("com.litric.settings") {
                let rows = self.settings.visible().len();
                let last = rows.saturating_sub(1);
                if dy < 0.0 {
                    self.settings.scroll = self.settings.scroll.saturating_sub(1).min(last);
                } else if dy > 0.0 {
                    self.settings.scroll = self.settings.scroll.saturating_add(1).min(last);
                }
                self.request_full_redraw();
            }
        }
    }

    /// The focused window's application id, if a window has focus.
    #[must_use]
    pub fn focused_app(&self) -> Option<&str> {
        self.wm
            .focused()
            .and_then(|id| self.wm.window(id))
            .map(|w| w.app_id.as_str())
    }

    /// Click a Settings row: switches flip, choices cycle, text rows start an
    /// edit, numbers do nothing (their step buttons do the work).
    fn activate_settings_row(&mut self, index: u16) -> Vec<Effect> {
        let index = usize::from(index);
        self.settings.selection = Some(index);
        let Some(spec) = self.settings.visible().get(index).copied() else {
            return Vec::new();
        };
        if matches!(spec.kind, crate::settings::Kind::Number { .. }) {
            return Vec::new();
        }
        let step = spec
            .cycled(&self.settings.values.get(&spec.id))
            .unwrap_or(crate::settings::Value::On(true));
        let id = spec.id.clone();
        if matches!(spec.kind, crate::settings::Kind::Text { .. }) {
            self.settings.editing = Some(id.clone());
            self.request_full_redraw();
            return Vec::new();
        }
        if matches!(spec.kind, crate::settings::Kind::Toggle) {
            let on = !self.settings.values.on(&id);
            return self.change_setting(&id, crate::settings::Value::On(on));
        }
        self.change_setting(&id, step)
    }

    /// What a click on a hotspot does.
    fn activate(&mut self, hot: Hot) -> Vec<Effect> {
        self.request_full_redraw();
        match hot {
            Hot::FirstRun(action) => {
                let mut effects = self
                    .first_run
                    .as_mut()
                    .map_or_else(Vec::new, |fr| fr.activate(action));
                self.commit_first_run_language(&mut effects);
                effects
            }
            // The LitricOS mark opens the system menu (session settings, lock).
            // The launcher keeps the Apps menu and its own shortcut; one click
            // on the product name must not be the only way to reach every app.
            Hot::Logo => self.perform(Action::ToggleLitroMenu),
            Hot::MenuApps => self.perform(Action::ToggleLauncher),
            // A click on a window's body focuses the window and nothing else:
            // it is the backdrop the real controls sit on, never a shortcut
            // for whatever happens to be under the window.
            Hot::Window(id) => {
                self.wm.focus_window(id);
                Vec::new()
            }
            Hot::DockItem(i) => {
                let Some(app) = crate::shortcuts::App::ALL.get(usize::from(i)) else {
                    return Vec::new();
                };
                self.dock_click(*app)
            }
            Hot::EngineChip(i) => {
                if let Some(e) = crate::apps::SearchEngine::ALL.get(usize::from(i)) {
                    self.browser.engine = *e;
                    self.browser.status = format!("search engine: {}", e.label());
                }
                Vec::new()
            }
            Hot::QuickLink(i) => {
                let urls = [
                    "https://www.google.com",
                    "https://th.wikipedia.org",
                    "https://github.com",
                ];
                if let Some(url) = urls.get(usize::from(i)) {
                    let url = (*url).to_owned();
                    self.browser.navigate_active_to(&url);
                    self.browser_navigate(&url);
                    return vec![Effect::PersistBrowserData];
                }
                Vec::new()
            }
            Hot::TerminalInput => {
                self.terminal.focused = true;
                Vec::new()
            }
            Hot::TerminalClear => {
                self.terminal.clear_screen();
                Vec::new()
            }
            Hot::TerminalNewSession => {
                if !self.terminal.new_session() {
                    self.terminal.note = Some(
                        "A command is still running; wait for it to finish before starting a new session."
                            .to_owned(),
                    );
                }
                Vec::new()
            }
            Hot::ActivityFilter => Vec::new(),
            Hot::ActivitySort(column) => {
                let key = column.sort_key();
                let ascending = if self.activity.prefs.sort.0 == key {
                    !self.activity.prefs.sort.1
                } else {
                    !self.activity.prefs.first_click_desc(column)
                };
                self.activity.prefs.sort_by(key, ascending);
                Vec::new()
            }
            Hot::TrashFilter => Vec::new(),
            Hot::MenuWorkspace | Hot::StatusMetrics => self.perform(Action::Open(App::Activity)),
            Hot::MenuEnvironment => self.perform(Action::ToggleQuickSettings),
            // The power glyph opens the power menu, not quick settings: the icon
            // is a promise about what the click does.
            Hot::StatusPower => self.perform(Action::TogglePowerMenu),
            Hot::Workspace(n) => {
                self.wm.set_workspace(usize::from(n));
                Vec::new()
            }
            Hot::StatusNetwork | Hot::StatusAudio | Hot::StatusBattery => {
                self.perform(Action::ToggleQuickSettings)
            }
            Hot::StatusNotifications => self.perform(Action::ToggleNotifications),
            Hot::StatusClock => self.perform(Action::ToggleNotifications),
            // The person icon opens the power menu: four transitions, each of
            // which asks before it acts.
            Hot::StatusUser => self.perform(Action::TogglePowerMenu),
            Hot::LauncherSearch => Vec::new(),
            Hot::LockWake => {
                self.lock.wake();
                Vec::new()
            }
            Hot::LockEntry => Vec::new(),
            Hot::LockSubmit => self.submit_unlock(),
            Hot::LockCustomize => {
                self.lock.edit = true;
                Vec::new()
            }
            Hot::LockTab(i) => {
                self.lock.tab = crate::lock::Tab::from_index(i);
                Vec::new()
            }
            Hot::LockOption(group, option) => {
                self.lock.tweaks.choose(group, option);
                Vec::new()
            }
            Hot::LockSwitch(index) => {
                self.lock.tweaks.toggle(index);
                Vec::new()
            }
            Hot::LockReset => {
                self.lock.tweaks = crate::lock::Tweaks::default();
                Vec::new()
            }
            Hot::LockDone => {
                self.lock.edit = false;
                Vec::new()
            }
            Hot::SettingsCategory(index) => {
                if let Some(page) = crate::settings::Category::ALL.get(usize::from(index)) {
                    self.settings.page = *page;
                    self.settings.query.clear();
                    self.settings.scroll = 0;
                    self.settings.selection = None;
                }
                Vec::new()
            }
            Hot::SettingsSearch => {
                self.settings.editing = None;
                Vec::new()
            }
            Hot::SettingsAdvanced => {
                self.settings.advanced = !self.settings.advanced;
                self.settings.scroll = 0;
                Vec::new()
            }
            Hot::SettingsReset => {
                let reset = if self.settings.query.trim().is_empty() {
                    self.settings.reset_category(self.settings.page)
                } else {
                    let before = self.settings.values.changed_count();
                    self.settings.values.reset_all();
                    self.settings.save = crate::settings::SaveState::Saving;
                    self.settings.dirty = true;
                    before
                };
                self.settings.message = Some((
                    if reset > 0 {
                        format!("{reset} setting(s) back to their defaults")
                    } else {
                        "Already at the defaults".to_owned()
                    },
                    true,
                ));
                self.apply_settings();
                self.persist_settings()
            }
            Hot::SettingsExport => vec![Effect::ExportSettings(self.settings.values.clone())],
            Hot::SettingsImport => vec![Effect::ImportSettings],
            Hot::SettingsRow(index) => self.activate_settings_row(index),
            Hot::SettingsStep(index, up) => {
                self.settings.selection = Some(usize::from(index));
                let _ = self.settings.step_selected(up);
                self.settings.editing = None;
                self.apply_settings();
                self.persist_settings()
            }
            Hot::SettingsLanguage => self.toggle_language(),
            Hot::SettingsTheme => self.cycle_theme(),
            Hot::SettingsMotion => self.toggle_reduced_motion(),
            // The dock and the launcher it becomes.
            Hot::DockToggle => self.perform(Action::ToggleLauncher),
            Hot::DockSearch => self.perform(Action::ToggleLauncher),
            Hot::DockCollapse => self.perform(Action::CloseOverlay),
            Hot::DockLock => self.perform(Action::Lock),
            Hot::LauncherItem(i) => {
                let query = match &self.overlay {
                    Overlay::Launcher { query, .. } => query.clone(),
                    _ => String::new(),
                };
                let app = App::search(&query).get(i).copied();
                self.overlay = Overlay::None;
                app.map_or_else(Vec::new, |a| self.open_app(a))
            }
            Hot::SwitcherItem(i) => {
                let order: Vec<WindowId> = self.wm.switcher_order().iter().map(|w| w.id).collect();
                if let Some(id) = order.get(i).copied() {
                    self.wm.unminimize(id);
                    self.wm.focus_window(id);
                }
                self.overlay = Overlay::None;
                self.switcher_holding = false;
                Vec::new()
            }
            Hot::QuickToggle(n) => {
                match n {
                    0 => self.quick.do_not_disturb = !self.quick.do_not_disturb,
                    1 => self.quick.adaptive_idle = !self.quick.adaptive_idle,
                    2 => self.quick.reduce_transparency = !self.quick.reduce_transparency,
                    _ => self.quick.reduce_motion = !self.quick.reduce_motion,
                }
                self.notifications.do_not_disturb = self.quick.do_not_disturb;
                Vec::new()
            }
            Hot::QuickPower(n) => {
                let profile = PowerProfile::ALL
                    .get(usize::from(n))
                    .copied()
                    .unwrap_or(PowerProfile::Balanced);
                self.quick.power = profile;
                vec![Effect::SetPowerProfile(profile)]
            }
            Hot::QuickPowerAction(n) => {
                // Clicking a row asks; it does not do. The panel closes and the
                // confirmation takes the screen, because a menu row that
                // suspends a machine on one click is the defect the live-consent
                // rule exists to prevent.
                if let Some(command) = PowerCommand::ALL.get(usize::from(n)).copied() {
                    self.overlay_damage();
                    self.overlay = Overlay::None;
                    self.desktop_dialog =
                        Some(crate::apps::DesktopDialog::PowerConfirm { command });
                    self.desktop_dialog_field.clear();
                    self.desktop_dialog_selection = None;
                    self.request_full_redraw();
                }
                Vec::new()
            }
            Hot::PowerItem(n) => {
                // Row order is lock, sleep, shut down, restart. Lock is
                // reversible and needs no confirmation; the three that end the
                // session open the same confirmation the quick-settings rows
                // did, because a menu row that powers a machine off is exactly
                // what the live-consent rule forbids.
                self.overlay_damage();
                self.overlay = Overlay::None;
                let command = match n {
                    0 => return vec![Effect::RequestLock],
                    1 => PowerCommand::Suspend,
                    2 => PowerCommand::PowerOff,
                    _ => PowerCommand::Restart,
                };
                self.desktop_dialog =
                    Some(crate::apps::DesktopDialog::PowerConfirm { command });
                self.desktop_dialog_field.clear();
                self.desktop_dialog_selection = None;
                self.request_full_redraw();
                Vec::new()
            }
            Hot::LitroSession => {
                self.overlay_damage();
                self.overlay = Overlay::None;
                self.perform(Action::Open(App::Settings))
            }
            Hot::LitroLock => {
                self.overlay_damage();
                self.overlay = Overlay::None;
                vec![Effect::RequestLock]
            }
            Hot::QuickEnvironment(n) => vec![Effect::SwitchEnvironment(n.saturating_add(1))],
            // Browser chrome clicks delegate to the model; a navigation the
            // model returns is a URL the caller hands to the engine session.
            Hot::BrowserGo => {
                // The model's navigation result IS the command to run: a
                // search committed from the omnibox must reach the engine.
                if let Some(url) = crate::apps::browser_action(&mut self.browser, hot) {
                    self.browser_navigate(&url);
                    return vec![Effect::PersistBrowserData];
                }
                Vec::new()
            }
            Hot::BrowserBack => {
                if let Some(url) = self.browser.back() {
                    self.browser_navigate(&url);
                }
                Vec::new()
            }
            Hot::BrowserForward => {
                if let Some(url) = self.browser.forward() {
                    self.browser_navigate(&url);
                }
                Vec::new()
            }
            Hot::BrowserBookmark => {
                let saved = self.browser.toggle_bookmark();
                self.browser.status = if saved {
                    "bookmark added".to_owned()
                } else {
                    "bookmark removed".to_owned()
                };
                vec![Effect::PersistBrowserData]
            }
            Hot::BrowserHistory => {
                self.browser.panel = crate::apps::BrowserPanel::History;
                Vec::new()
            }
            Hot::BrowserBookmarks => {
                self.browser.panel = crate::apps::BrowserPanel::Bookmarks;
                Vec::new()
            }
            Hot::BrowserDownloads => {
                self.browser.panel = crate::apps::BrowserPanel::Downloads;
                self.browser.downloads.clear();
                self.browser.downloads_status = Some("Loading…".to_owned());
                if let Some(link) = self.browser_session.as_mut() {
                    if let Err(error) = link.send_line("DOWNLOADS") {
                        self.browser.downloads_status = Some(format!("Unavailable: {error}"));
                    }
                } else {
                    self.browser.downloads_status =
                        Some("Browser engine session is not connected".to_owned());
                }
                Vec::new()
            }
            Hot::BrowserSitePermissions => {
                self.browser.panel = crate::apps::BrowserPanel::SitePermissions;
                Vec::new()
            }
            Hot::BrowserHistoryItem(index) => {
                if let Some(url) = self.browser.history.get(index).cloned() {
                    self.browser.navigate_active_to(&url);
                    self.browser_navigate(&url);
                    return vec![Effect::PersistBrowserData];
                }
                Vec::new()
            }
            Hot::BrowserBookmarkItem(index) => {
                if let Some(url) = self.browser.bookmarks.get(index).cloned() {
                    self.browser.navigate_active_to(&url);
                    self.browser_navigate(&url);
                    return vec![Effect::PersistBrowserData];
                }
                Vec::new()
            }
            Hot::BrowserNewTab | Hot::BrowserTab(_) | Hot::BrowserCloseTab(_) => {
                let _ = crate::apps::browser_action(&mut self.browser, hot);
                Vec::new()
            }
            Hot::BrowserOmnibox => {
                self.browser.omnibox_focus = true;
                // The browser convention: focusing the omnibox shows the
                // address fully selected, so typing replaces it instead of
                // appending to the middle of the last URL. `omnibox_fresh`
                // marks that selection; the first typed character clears it.
                self.browser.omnibox = self
                    .browser
                    .tab()
                    .map(|t| t.url.clone())
                    .unwrap_or_default();
                self.browser.omnibox_fresh = true;
                self.request_full_redraw();
                Vec::new()
            }
            // Files clicks mutate the model; the view refreshes the listing
            // when it sees `listed` lag `path`.
            Hot::Files(h) => {
                let _ = crate::apps::files_action(&mut self.files, h);
                Vec::new()
            }
            Hot::NotificationItem(i) => {
                let id = self.notifications.items().get(i).map(|n| n.id);
                if let Some(id) = id {
                    self.notifications.dismiss(id);
                }
                Vec::new()
            }
            Hot::NotificationsClear => {
                self.notifications.clear();
                Vec::new()
            }
            Hot::Desktop => {
                // A click on the wallpaper clears selections everywhere the
                // desktop has them, which is what every file manager does.
                self.desktop_icons.select(None);
                self.files.selected = None;
                self.files.selected_rows.clear();
                self.context_menu = None;
                Vec::new()
            }
            Hot::DesktopIcon(i) => self.open_desktop_icon(i),
            Hot::MenuRow(action) => {
                self.context_menu = None;
                self.request_full_redraw();
                self.perform_menu(action)
            }
            Hot::TrashRow(i) => {
                self.trash.selected = Some(i);
                self.trash.confirm = None;
                Vec::new()
            }
            Hot::TrashEmpty => self.perform_menu(crate::apps::MenuAction::EmptyTrash),
            Hot::TrashRestore => self.perform_menu(crate::apps::MenuAction::Restore),
            Hot::TrashPurge => self.perform_menu(crate::apps::MenuAction::PurgeTrashItem),
            Hot::TrashConfirm => match self.trash.confirm.clone() {
                Some(crate::apps::TrashConfirmation::Empty) => {
                    self.perform_menu(crate::apps::MenuAction::EmptyTrash)
                }
                Some(crate::apps::TrashConfirmation::Purge(_)) => {
                    self.perform_menu(crate::apps::MenuAction::PurgeTrashItem)
                }
                None => Vec::new(),
            },
            Hot::TrashCancel => {
                self.trash.confirm = None;
                Vec::new()
            }
            Hot::DialogConfirm => {
                if self.desktop_dialog.is_some() {
                    self.submit_desktop_dialog()
                } else {
                    self.submit_files_dialog();
                    Vec::new()
                }
            }
            Hot::DialogCancel => {
                if self.desktop_dialog.is_some() {
                    self.desktop_dialog = None;
                    self.request_full_redraw();
                } else {
                    self.files.close_dialog();
                    self.request_full_redraw();
                }
                Vec::new()
            }
        }
    }

    /// Which menu surface a right-click at this point belongs to.
    fn menu_surface_at(&self, at: Point) -> crate::apps::MenuSurface {
        use crate::apps::MenuSurface;
        for (id, frame, app_id) in self
            .wm
            .visible()
            .iter()
            .map(|w| (w.id, w.frame, w.app_id.clone()))
        {
            if frame.contains(at.x, at.y) {
                if app_id == "com.litric.files" {
                    // A row under the pointer means the row menu; otherwise
                    // the pane menu.
                    if let Some(h) = self.hot.hit(at.x, at.y) {
                        if matches!(h, Hot::Files(crate::apps::FilesHot::Row(_))) {
                            return MenuSurface::FilesRow;
                        }
                    }
                    return MenuSurface::Files;
                }
                if app_id == "com.litric.browser" {
                    if let Some(h) = self.hot.hit(at.x, at.y) {
                        if matches!(h, Hot::BrowserTab(_)) {
                            return MenuSurface::BrowserTab;
                        }
                    }
                    return MenuSurface::Browser;
                }
                if app_id == "com.litric.trash" {
                    return MenuSurface::Trash;
                }
                let _ = id;
                return MenuSurface::Desktop;
            }
        }
        MenuSurface::Desktop
    }

    /// Carry out an action from the shortcut table or a click.
    ///
    /// Overlays and the diagnostic panel repaint the whole screen. Window and
    /// workspace actions rely on the damage the window manager records, and
    /// actions that only produce an effect repaint nothing themselves: what
    /// `main` does with the effect (a notification) asks for its own repaint.
    pub fn perform(&mut self, action: Action) -> Vec<Effect> {
        let focused = self.wm.focused();
        match action {
            Action::ToggleLauncher => {
                self.overlay_damage();
                self.overlay = if matches!(self.overlay, Overlay::Launcher { .. }) {
                    Overlay::None
                } else {
                    Overlay::Launcher {
                        query: String::new(),
                        selected: 0,
                    }
                };
                Vec::new()
            }
            Action::CloseOverlay => {
                if self.overlay.is_open() {
                    self.overlay_damage();
                }
                self.overlay = Overlay::None;
                self.switcher_holding = false;
                Vec::new()
            }
            Action::SwitchEnvironment(n) => vec![Effect::SwitchEnvironment(n)],
            Action::ReturnToHost => vec![Effect::SwitchEnvironment(2)],
            Action::NextWindow | Action::PrevWindow => {
                let forward = action == Action::NextWindow;
                let n = self.wm.switcher_order().len();
                if n == 0 {
                    return Vec::new();
                }
                self.overlay_damage();
                let selected = match self.overlay {
                    Overlay::Switcher { selected } if forward => {
                        selected.saturating_add(1).checked_rem(n).unwrap_or(0)
                    }
                    Overlay::Switcher { selected } => {
                        selected.checked_sub(1).unwrap_or(n.saturating_sub(1))
                    }
                    // The first press goes to the previously used window, which
                    // is index 1: index 0 is the one you are already on.
                    _ if forward => 1usize.checked_rem(n).unwrap_or(0),
                    _ => n.saturating_sub(1),
                };
                self.overlay = Overlay::Switcher { selected };
                self.switcher_holding = true;
                Vec::new()
            }
            Action::CloseWindow => {
                if let Some(id) = focused {
                    self.wm.close(id);
                }
                Vec::new()
            }
            Action::MinimizeWindow => {
                if let Some(id) = focused {
                    self.wm.minimize(id);
                }
                Vec::new()
            }
            Action::ToggleMaximize => {
                if let Some(id) = focused {
                    self.wm.toggle_maximize(id);
                }
                Vec::new()
            }
            Action::SnapLeft => {
                if let Some(id) = focused {
                    self.wm.snap(id, litric_wm::SnapEdge::Left);
                }
                Vec::new()
            }
            Action::SnapRight => {
                if let Some(id) = focused {
                    self.wm.snap(id, litric_wm::SnapEdge::Right);
                }
                Vec::new()
            }
            Action::CenterWindow => {
                if let Some(id) = focused {
                    self.wm.center(id);
                }
                Vec::new()
            }
            Action::Workspace(n) => {
                self.wm.set_workspace(usize::from(n));
                Vec::new()
            }
            Action::NextWorkspace => {
                self.wm.cycle_workspace(true);
                Vec::new()
            }
            Action::PrevWorkspace => {
                self.wm.cycle_workspace(false);
                Vec::new()
            }
            Action::MoveToWorkspace(n) => {
                if let Some(id) = focused {
                    self.wm.move_to_workspace(id, usize::from(n));
                }
                Vec::new()
            }
            Action::ShowDesktop => {
                let ids: Vec<WindowId> = self.wm.visible().iter().map(|w| w.id).collect();
                for id in ids {
                    self.wm.minimize(id);
                }
                Vec::new()
            }
            Action::Open(app) => self.open_app(app),
            Action::ToggleQuickSettings => {
                self.overlay_damage();
                self.overlay = if self.overlay == Overlay::QuickSettings {
                    Overlay::None
                } else {
                    Overlay::QuickSettings
                };
                Vec::new()
            }
            Action::TogglePowerMenu => {
                self.overlay_damage();
                self.overlay = if self.overlay == Overlay::Power {
                    Overlay::None
                } else {
                    Overlay::Power
                };
                Vec::new()
            }
            Action::ToggleLitroMenu => {
                self.overlay_damage();
                self.overlay = if self.overlay == Overlay::Litro {
                    Overlay::None
                } else {
                    Overlay::Litro
                };
                Vec::new()
            }
            Action::ToggleNotifications => {
                self.overlay_damage();
                self.overlay = if self.overlay == Overlay::Notifications {
                    Overlay::None
                } else {
                    Overlay::Notifications
                };
                Vec::new()
            }
            Action::ToggleDiagnostics => {
                self.request_full_redraw();
                self.diagnostics = !self.diagnostics;
                Vec::new()
            }
            Action::Lock => vec![Effect::RequestLock],
            Action::VolumeUp => vec![Effect::Volume(1)],
            Action::VolumeDown => vec![Effect::Volume(-1)],
            Action::VolumeMute => vec![Effect::Volume(0)],
            Action::BrightnessUp => vec![Effect::Brightness(1)],
            Action::BrightnessDown => vec![Effect::Brightness(-1)],
            // ---- Files ------------------------------------------------------------
            Action::FilesNewFolder => {
                self.files.ask_new_folder();
                self.request_full_redraw();
                Vec::new()
            }
            Action::FilesNewFile => {
                self.files.ask_new_file();
                self.request_full_redraw();
                Vec::new()
            }
            Action::FilesRename => {
                self.files.ask_rename();
                self.request_full_redraw();
                Vec::new()
            }
            Action::FilesCopy => {
                self.files.copy_selection();
                Vec::new()
            }
            Action::FilesCut => {
                self.files.cut_selection();
                Vec::new()
            }
            Action::FilesPaste => {
                if let Some((cut, p)) = self.files.clipboard.clone() {
                    let op = if cut {
                        crate::apps::FilesOp::PasteCut
                    } else {
                        crate::apps::FilesOp::PasteCopy
                    };
                    let what = if cut { "Move" } else { "Copy" };
                    self.files.ask_confirm(
                        what,
                        &format!("{what} {p} into {}?", self.files.path),
                        op,
                    );
                    self.request_full_redraw();
                }
                Vec::new()
            }
            Action::FilesTrash => {
                let paths = self.files.selected_paths();
                if let Some(action) = selection_action(
                    &paths,
                    crate::apps::FilesOp::Trash,
                    crate::apps::FilesOp::TrashMany,
                ) {
                    self.trash_with_confirm(
                        "Move to trash",
                        &format!("Move {} selected item(s) to the trash?", paths.len()),
                        action,
                    );
                }
                Vec::new()
            }
            Action::FilesCompress => {
                let _ = crate::apps::files_action(&mut self.files, crate::apps::FilesHot::Compress);
                self.request_full_redraw();
                Vec::new()
            }
            Action::FilesExtract => {
                let _ = crate::apps::files_action(&mut self.files, crate::apps::FilesHot::Extract);
                self.request_full_redraw();
                Vec::new()
            }
            Action::FilesSearch => {
                self.files.focus = crate::apps::FilesFocus::Table;
                Vec::new()
            }
            Action::FilesUp => {
                self.files.go_up();
                Vec::new()
            }
            // ---- Defender / browser / trash --------------------------------------
            Action::OpenDefender => self.perform(Action::Open(App::Defender)),
            Action::ScanNow => self.start_scan(),
            Action::BrowserAddress => {
                self.browser.omnibox_focus = true;
                Vec::new()
            }
            Action::BrowserReload => self.perform_menu(crate::apps::MenuAction::BrowserReload),
            Action::ViewTrash => self.perform(Action::Open(App::Trash)),
            Action::OpenIcon => self.perform_menu(crate::apps::MenuAction::OpenSelection),
        }
    }

    /// Open an application's window, or focus it if it is already open.
    ///
    /// The window manager damages the window it opens, raises or restores;
    /// only an overlay that closes on the way repaints the whole screen.
    pub fn open_app(&mut self, app: App) -> Vec<Effect> {
        if self.overlay.is_open() {
            self.overlay = Overlay::None;
            self.overlay_damage();
        }
        if let Some(existing) = self
            .wm
            .all()
            .iter()
            .find(|w| w.app_id == app.app_id())
            .map(|w| w.id)
        {
            self.wm.unminimize(existing);
            self.wm.focus_window(existing);
            return Vec::new();
        }
        if app == App::Terminal && self.settings.values.on("terminal.clear-on-open") {
            self.terminal.lines.clear();
            self.terminal.push_line(String::new());
        }
        let (w, h) = app.default_size();
        let scale = self.tokens.scale;
        self.wm
            .open(app.app_id(), app.title(), (w * scale, h * scale));
        vec![Effect::Launched(app)]
    }

    /// Post a notification, respecting Do Not Disturb.
    pub fn notify(
        &mut self,
        source: &str,
        title: &str,
        body: &str,
        priority: Priority,
        ttl_ms: Option<u64>,
    ) -> u32 {
        // The banner's lifetime is the preference's, except for a critical
        // notice: its whole point is that it is still there when the person
        // looks back, so "sticky" wins over the timer when it is on.
        let ttl = match (ttl_ms, priority) {
            (_, Priority::Critical) if self.settings.values.on("notifications.critical-sticky") => {
                None
            }
            (Some(_), _) => Some(
                (self
                    .settings
                    .values
                    .num("notifications.banner-ms")
                    .clamp(1_000.0, 15_000.0)) as u64,
            ),
            (None, _) => None,
        };
        let id = self
            .notifications
            .post(source, title, body, priority, self.now_ms, ttl);
        if self.notifications.should_banner(priority) {
            self.request_full_redraw();
        }
        id
    }

    /// The current instant in seconds since the epoch, for the clock.
    #[must_use]
    pub fn clock_seconds(&self) -> i64 {
        i64::try_from(self.now_ms.checked_div(1000).unwrap_or(0)).unwrap_or(0)
    }

    /// The windows to draw, back to front, as canvas rectangles.
    #[must_use]
    pub fn visible_windows(&self) -> Vec<(WindowId, Rect, String, bool, WindowState)> {
        let focused = self.wm.focused();
        self.wm
            .visible()
            .iter()
            .map(|w| {
                (
                    w.id,
                    to_rect(w.frame),
                    w.title.clone(),
                    Some(w.id) == focused,
                    w.state,
                )
            })
            .collect()
    }

    /// Lines for the status file and the diagnostic overlay.
    #[must_use]
    pub fn diagnostic_lines(&self) -> Vec<String> {
        let mut out = vec![
            format!("mode={}", self.mode.token()),
            format!("life={}", self.life().token()),
            format!("frame_interval_ms={}", self.frame_interval_ms()),
            format!("overlay={}", self.overlay.token()),
            format!("frames={}", self.frames),
            format!("hotspots={}", self.hot.len()),
            format!("notifications={}", self.notifications.len()),
            format!("pointer={:.0},{:.0}", self.pointer.x, self.pointer.y),
            format!("facts.source={}", self.facts.source),
        ];
        // Browser + reachability: the engine's own words about its session,
        // and the one-shot network probe result. A page that fails to render
        // must be explainable from the serial log alone — DNS vs TCP vs no
        // engine session are three different fixes.
        out.push(format!("browser.status={}", self.browser.status));
        if let Some(t) = self.browser.tab() {
            out.push(format!("browser.tab={}", t.url));
        }
        out.push(format!(
            "browser.frame={}",
            self.browser_frame
                .as_ref()
                .map_or_else(|| "none".to_owned(), |f| format!("{} bytes", f.len())),
        ));
        out.push(format!("net.probe={}", net_probe_token()));
        // The event log and the windows nobody is looking at. Both are off by
        // default: one is a timeline of notifications and the other is a list
        // of what is on the other workspaces, and neither is a fact about the
        // frame being debugged unless somebody asked for it.
        if self.settings.values.on("diagnostics.events") {
            for n in self.notifications.items().iter().take(8) {
                out.push(format!("event={:?} {}: {}", n.priority, n.source, n.title));
            }
        }
        if self.settings.values.on("diagnostics.fps") {
            out.push(format!(
                "fps={}",
                self.fps_last
                    .map_or_else(|| "warming".to_owned(), |fps| fps.to_string())
            ));
        }
        if self.settings.values.on("diagnostics.show-hidden") {
            let active = self.wm.active_workspace();
            let names = self.wm.workspaces().to_vec();
            for w in self.wm.all().iter().filter(|w| w.workspace != active) {
                let name = names.get(w.workspace).map_or("?", String::as_str);
                out.push(format!("hidden={} workspace={name}", w.app_id));
            }
        }
        out.extend(self.wm.diagnostics());
        out.extend(
            self.regional
                .lines(i64::try_from(self.now_ms.checked_div(1000).unwrap_or(0)).unwrap_or(0)),
        );
        out
    }
}

/// Convert a window-manager rectangle to a canvas one.
#[must_use]
pub fn to_rect(r: WmRect) -> Rect {
    Rect::new(r.x, r.y, r.w, r.h)
}

/// One volume notch, in percent of the sink's range.
///
/// Five is the classic increment: twenty presses span the range, and one press
/// is a step a person can hear as a step rather than a jump.
pub const VOLUME_NOTCH_PCT: i8 = 5;

/// One brightness notch, in percent of the backlight's range.
pub const BRIGHTNESS_NOTCH_PCT: i8 = 5;

/// The request a volume effect sends.
///
/// Zero is the mute key. The shell holds no mixer state — it never reads the
/// device — so it asks the daemon to *flip* the mute rather than claiming to
/// know it, and a relative step rather than "read, add, write" across the
/// socket, which would race with every other change.
#[must_use]
pub fn volume_request(notches: i8) -> String {
    match notches {
        0 => "AUDIO MUTE toggle".to_owned(),
        n => format!("AUDIO STEP {}", n.saturating_mul(VOLUME_NOTCH_PCT)),
    }
}

/// The request a brightness effect sends.
#[must_use]
pub fn brightness_request(notches: i8) -> String {
    format!(
        "DISPLAY STEP {}",
        notches.saturating_mul(BRIGHTNESS_NOTCH_PCT)
    )
}

/// What the top bar shows from the facts, as it shows it: the CPU and memory
/// text of `views::draw_top_bar`, and the battery level and network state its
/// icons are coloured by.
/// The file operation for a selection: the single-item variant for one item,
/// the bulk variant for the rest, and nothing at all for an empty selection.
///
/// Every destructive action in Files needs this decision, and the way it was
/// written — `if !paths.is_empty() { if paths.len() == 1 { paths[0] … } }` — is a
/// panic waiting for the one caller that forgets the guard. Matching on the
/// slice says the same thing in one place, and cannot fail.
fn selection_action<T>(
    paths: &[String],
    one: fn(String) -> T,
    many: fn(Vec<String>) -> T,
) -> Option<T> {
    match paths {
        [] => None,
        [only] => Some(one(only.clone())),
        several => Some(many(several.to_vec())),
    }
}

fn top_bar_view(f: &SystemFacts) -> (String, Option<u8>, Option<String>) {
    let mem = match (f.mem_used_mb, f.mem_total_mb) {
        (Some(u), Some(_)) => format!("{u} MB"),
        _ => "-- MB".to_owned(),
    };
    let cpu = f
        .cpu_fraction
        .map_or_else(|| "--%".to_owned(), |v| format!("{:.0}%", v * 100.0));
    (
        format!("{cpu}  {mem}"),
        f.battery_percent,
        f.network.clone(),
    )
}

/// What an Activity window shows from the facts: every field it draws, by
/// value because the meter bars move with any change, except uptime, which is
/// compared as the text it is drawn as. Fields it does not draw are blanked.
fn activity_view(f: &SystemFacts) -> (SystemFacts, Option<String>) {
    (
        SystemFacts {
            uptime_secs: None,
            cpus: None,
            trash: None,
            battery_percent: None,
            battery_charging: None,
            network: None,
            ..f.clone()
        },
        f.uptime_secs.map(crate::desktop::format_uptime),
    )
}

/// The denylist every shell-built scanner carries: the EICAR test file, the
/// industry-standard harmless proof that a detection path works end to end.
#[must_use]
pub fn eicar_denylist() -> litric_defender::denylist::Denylist {
    // One literal backslash: `\\` in a Rust byte string. Escaped twice, this
    // hashed a sequence no EICAR file contains, so the shell's own denylist
    // could not match the standard test file while every other copy of the
    // constant in this workspace could — a scan of a real EICAR file reported
    // clean. The e2e journey below caught it.
    let eicar = b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
    let digest = litric_defender::sha256::digest(eicar);
    // `fold` + `write!` rather than `map(format!).collect()`: the latter
    // allocates a throwaway `String` per byte (clippy::format_collect).
    // Writing into a `String` cannot fail, so the `Result` is dropped
    // rather than unwrapped, which the workspace forbids in shipping code.
    use std::fmt::Write as _;
    let mut hex = String::with_capacity(digest.len().saturating_mul(2));
    for b in &digest {
        let _ = write!(hex, "{b:02x}");
    }
    let mut list = litric_defender::denylist::Denylist::new();
    list.insert(&hex, "eicar-test-file");
    list
}

/// `"tab 1280 800"` in a FRAME header to `(1280, 800)`.
#[must_use]
fn parse_frame_dims(rest: &str) -> Option<(u32, u32)> {
    let mut it = rest.split(' ').filter_map(|v| v.parse::<u32>().ok());
    let w = it.next()?;
    let h = it.next()?;
    if w == 0 || h == 0 || w > 8192 || h > 8192 {
        return None;
    }
    Some((w, h))
}

/// The FRAME header length rule for the browser link: `FRAME <key> <len>`.
fn browser_frame_len(header: &str) -> Result<usize, String> {
    header
        .strip_prefix("FRAME ")
        .and_then(|r| r.split(' ').nth(1))
        .ok_or_else(|| "bad FRAME header".to_owned())?
        .parse::<usize>()
        .map_err(|_| "bad FRAME length".to_owned())
        .and_then(|n| {
            if n <= 1920 * 1080 * 4 {
                Ok(n)
            } else {
                Err("frame too large".to_owned())
            }
        })
}

/// Depth-limited file collection for the Defender's queue.
fn collect_files(
    bridge: &crate::fsbridge::FsBridge,
    dir: &str,
    out: &mut Vec<String>,
    depth: usize,
) {
    if out.len() >= 2_000 || depth > 8 {
        return;
    }
    let Ok(entries) = bridge.list_dir(dir) else {
        return;
    };
    for e in entries {
        if out.len() >= 2_000 {
            return;
        }
        let p = format!("{dir}/{}", e.name);
        match e.kind {
            crate::apps::FileKind::Dir => collect_files(bridge, &p, out, depth.saturating_add(1)),
            crate::apps::FileKind::File => out.push(p),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Network reachability probe
//
// The serial report already carries the engine's own status line; what it
// could not say was whether the guest has a working route at all. A page that
// never renders is three different bugs — no engine session, a session that
// cannot resolve DNS, a session that cannot connect — and only the second and
// third look identical from the outside. The probe answers once per boot, in
// a background thread, and never touches a frame.
// ---------------------------------------------------------------------------

#[cfg(unix)]
static NET_PROBE: AtomicU8 = AtomicU8::new(0); // 0 unknown, 1 up, 2 down
#[cfg(unix)]
static NET_PROBE_STARTED: Once = Once::new();

/// A "down" must be earned by repeated attempts, not read off one failure
/// before networkd finished: a boot-time resolve can fail while the lease is
/// still arriving, and a probe that cached that failure would report a dead
/// network on a live one forever. Ceiling = ~3 minutes of honest retries.
#[cfg(unix)]
const NET_PROBE_MAX_ATTEMPTS: u32 = 30;
#[cfg(unix)]
const NET_PROBE_INTERVAL: Duration = Duration::from_secs(6);
#[cfg(unix)]
const NET_PROBE_TIMEOUT: Duration = Duration::from_secs(4);

/// Kick off the probe thread (idempotent): it retries the real handshake
/// until the stack answers or the ceiling is reached, so "down" always means
/// "we tried, this often, this recently" — never "we gave up at boot".
#[cfg(unix)]
fn start_net_probe() {
    NET_PROBE_STARTED.call_once(|| {
        std::thread::Builder::new()
            .name("litric-net-probe".to_owned())
            .spawn(|| {
                for _ in 0..NET_PROBE_MAX_ATTEMPTS {
                    let stage = net_attempt();
                    NET_PROBE.store(stage, Relaxed);
                    if stage == 1 {
                        return; // up
                    }
                    std::thread::sleep(NET_PROBE_INTERVAL);
                }
            })
            .map(|_| ())
            .map_err(|_| ())
            .ok();
    });
}

/// One probe attempt: resolve, then connect — the two halves fail for
/// different reasons and the token records which half broke. The route table
/// is read first because "no default route at all" is a different bug from
/// "DNS broken" and previously both reported the same `down`.
#[cfg(unix)]
fn net_attempt() -> u8 {
    if !has_default_route() {
        return 3; // no route: networkd never configured the NIC
    }
    match ("dns.google", 443u16).to_socket_addrs() {
        Err(_) => 2,
        Ok(mut addrs) => match addrs.next() {
            None => 2,
            Some(addr) => match TcpStream::connect_timeout(&addr, NET_PROBE_TIMEOUT) {
                Ok(_) => 1,
                Err(_) => 2,
            },
        },
    }
}

/// `pending` until the probe finishes, then `up`, `down` (DNS/TCP broken),
/// or `noroute` (no default route — the networkd-config bug class).
#[cfg(unix)]
fn net_probe_token() -> &'static str {
    match NET_PROBE.load(Relaxed) {
        1 => "up",
        2 => "down",
        3 => "noroute",
        _ => {
            start_net_probe();
            "pending"
        }
    }
}

/// True when the route table carries a default route (destination 00000000).
/// `/proc/net/route` needs no privileges and no libc: the guest either has a
/// gateway or it does not, and that fact should not cost a syscall guessing.
#[cfg(unix)]
fn has_default_route() -> bool {
    std::fs::read_to_string("/proc/net/route")
        .map(|text| {
            text.lines().skip(1).any(|line| {
                // Destination is the second whitespace field.
                let mut fields = line.split_whitespace();
                fields.next().is_some() && fields.next() == Some("00000000")
            })
        })
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn net_probe_token() -> &'static str {
    "host-build"
}

#[cfg(all(test, unix))]
mod net_probe_tests {
    use super::*;

    #[test]
    fn probe_reports_a_definite_answer() {
        // Not a pass/fail of the lab's internet: a probe that never answers
        // is the bug. Whatever the network is, the token must resolve to a
        // definite state within the connect timeout plus slack.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            let t = net_probe_token();
            if t != "pending" || std::time::Instant::now() > deadline {
                assert_ne!(t, "pending", "probe never resolved to up/down");
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use litric_design::{Theme, ThemeKind};
    use litric_input::keys::Side;

    #[test]
    fn every_power_command_names_itself_for_the_user_and_for_the_daemon() {
        // Two vocabularies that must not drift apart: the word on the button
        // and the word in the request. `Restart` is `REBOOT` on the wire, and
        // this is where that is decided.
        let words: Vec<(&str, &str)> = PowerCommand::ALL
            .iter()
            .map(|c| (c.title(), c.token()))
            .collect();
        assert_eq!(
            words,
            vec![
                ("Suspend", "SUSPEND"),
                ("Restart", "REBOOT"),
                ("Power off", "POWEROFF"),
            ]
        );
        for c in PowerCommand::ALL {
            assert!(!c.question().is_empty());
            assert!(!c.consequence().is_empty());
        }
    }

    #[test]
    fn a_power_request_carries_the_confirmation_word() {
        // The one string the daemon needs and the shell can get wrong. A
        // request without `CONFIRM` is refused by the broker, so this asserts
        // the shell never builds one that way.
        for c in PowerCommand::ALL {
            let line = c.request_line();
            assert!(line.starts_with("POWER "), "{line}");
            assert!(line.ends_with(" CONFIRM"), "{line}");
            assert!(line.contains(c.token()), "{line}");
        }
        assert_eq!(
            PowerCommand::PowerOff.request_line(),
            "POWER POWEROFF CONFIRM"
        );
    }

    #[test]
    fn volume_brightness_and_profile_requests_speak_the_daemons_grammar() {
        // The shell and the daemon are two crates; the vocabulary between them
        // is the protocol, so the requests the shell builds are checked
        // against the daemon's own parser rather than against a string this
        // test made up.
        use litric_sysd::proto::Request;
        assert_eq!(volume_request(0), "AUDIO MUTE toggle");
        assert_eq!(volume_request(1), "AUDIO STEP 5");
        assert_eq!(volume_request(-1), "AUDIO STEP -5");
        assert_eq!(brightness_request(1), "DISPLAY STEP 5");
        for line in [
            volume_request(0),
            volume_request(1),
            volume_request(-2),
            brightness_request(1),
            brightness_request(-1),
        ] {
            assert!(Request::parse(&line).is_ok(), "{line}");
        }
        for profile in PowerProfile::ALL {
            let line = profile.request_line();
            assert!(Request::parse(&line).is_ok(), "{line}");
            assert_eq!(
                litric_sysd::providers::PowerProfileAction::parse(profile.token()),
                match profile {
                    PowerProfile::Eco => Some(litric_sysd::providers::PowerProfileAction::Eco),
                    PowerProfile::Balanced => {
                        Some(litric_sysd::providers::PowerProfileAction::Balanced)
                    }
                    PowerProfile::Performance => {
                        Some(litric_sysd::providers::PowerProfileAction::Performance)
                    }
                    PowerProfile::Maximum => {
                        Some(litric_sysd::providers::PowerProfileAction::Maximum)
                    }
                },
                "{} names a profile the daemon knows",
                profile.token()
            );
        }
    }

    fn proc(pid: u32, ticks: u64, rss_mb: Option<u64>) -> ProcRow {
        ProcRow {
            pid,
            name: format!("p{pid}"),
            cmdline: format!("/usr/bin/p{pid}"),
            state: "sleeping".to_owned(),
            ppid: 1,
            threads: 1,
            ticks,
            rss_mb,
            uid: Some(1000),
            cpu: None,
        }
    }

    #[test]
    fn a_process_share_is_of_the_whole_machine_and_absent_until_two_samples_exist() {
        // The first sample cannot have a rate: it has nothing to subtract. The
        // second one has to be a share of the machine's own tick delta, so that
        // one busy core out of eight reads 12.5%, not 100%.
        let mut facts = SystemFacts {
            cpu_ticks: Some((100, 800)),
            processes: vec![proc(10, 500, Some(12)), proc(11, 0, None)],
            ..SystemFacts::default()
        };
        assert!(
            facts.processes.iter().all(|p| p.cpu.is_none()),
            "a first sample must not claim a rate"
        );
        facts.observe(SystemFacts {
            cpu_ticks: Some((200, 1_600)),
            processes: vec![
                proc(10, 600, Some(14)),
                ProcRow {
                    pid: 12,
                    name: "new".to_owned(),
                    cmdline: String::new(),
                    state: "running".to_owned(),
                    ppid: 1,
                    threads: 2,
                    ticks: 9_000,
                    rss_mb: Some(1),
                    uid: None,
                    cpu: None,
                },
            ],
            ..SystemFacts::default()
        });
        let of = |pid: u32| facts.processes.iter().find(|p| p.pid == pid);
        let ten = of(10).expect("pid 10 survives");
        assert_eq!(
            ten.cpu,
            Some(0.125),
            "100 of 800 machine ticks is one core of eight"
        );
        assert_eq!(ten.cpu_pct(), "12.5");
        assert_eq!(ten.rss_text(), "14 MB");
        assert!(
            of(11).is_none(),
            "a process that is gone is absent, not reported at 0%"
        );
        assert_eq!(
            of(12).map(|p| p.cpu),
            Some(None),
            "a process with no earlier sample has no rate, and 0% is not it"
        );
        assert_eq!(
            of(12).map(ProcRow::cpu_pct),
            Some("\u{2014}".to_owned()),
            "so the table draws a dash rather than a plausible zero"
        );
        assert_eq!(of(12).map(ProcRow::rss_text), Some("1 MB".to_owned()));
    }

    #[test]
    fn a_process_whose_pid_was_reused_does_not_report_a_spike() {
        let mut facts = SystemFacts {
            cpu_ticks: Some((0, 0)),
            processes: vec![proc(10, 9_000, None)],
            ..SystemFacts::default()
        };
        facts.observe(SystemFacts {
            cpu_ticks: Some((10, 100)),
            processes: vec![proc(10, 4, None)],
            ..SystemFacts::default()
        });
        assert_eq!(
            facts.processes.first().and_then(|p| p.cpu),
            None,
            "a counter that went backwards has no rate"
        );
        assert_eq!(
            facts.processes.first().map(ProcRow::cpu_pct),
            Some("\u{2014}".to_owned()),
            "and the table says so rather than printing 0.0"
        );
    }

    #[test]
    fn a_machine_with_no_processes_says_unavailable_rather_than_empty() {
        let mut facts = SystemFacts::default();
        facts.observe(SystemFacts {
            cpu_ticks: Some((10, 100)),
            ..SystemFacts::default()
        });
        assert!(
            facts.processes.is_empty(),
            "the monitor draws the unmeasured case from this being empty"
        );
    }

    fn shell() -> Shell {
        let tokens = Tokens::new(Theme::of(ThemeKind::Dark), 1.0);
        let mut s = Shell::new(tokens, 1280.0, 800.0, Regional::fallback("test"));
        s.mode = DesktopMode::Ready;
        s
    }

    fn key(k: Key, mods: Modifiers) -> InputEvent {
        InputEvent::KeyDown {
            key: k,
            code: 0,
            mods,
            repeat: false,
        }
    }

    fn key_up(k: Key) -> InputEvent {
        InputEvent::KeyUp {
            key: k,
            code: 0,
            mods: Modifiers::NONE,
        }
    }

    fn meta() -> Modifiers {
        Modifiers {
            meta: true,
            ..Modifiers::NONE
        }
    }

    fn alt() -> Modifiers {
        Modifiers {
            alt: true,
            ..Modifiers::NONE
        }
    }

    #[test]
    fn win_1_asks_for_an_environment_switch() {
        // The signature shortcut. It must produce an effect the caller acts on,
        // not a change to a label on screen.
        let mut s = shell();
        let out = s.handle(&key(Key::Char('1'), meta()), 1000);
        assert_eq!(out, vec![Effect::SwitchEnvironment(1)]);
        let out3 = s.handle(&key(Key::Char('3'), meta()), 1100);
        assert_eq!(out3, vec![Effect::SwitchEnvironment(3)]);
    }

    #[test]
    fn ctrl_alt_home_always_returns_to_the_host() {
        let mut s = shell();
        let mods = Modifiers {
            ctrl: true,
            alt: true,
            ..Modifiers::NONE
        };
        assert_eq!(
            s.handle(&key(Key::Home, mods), 1000),
            vec![Effect::SwitchEnvironment(2)]
        );
    }

    #[test]
    fn tapping_meta_opens_the_launcher_and_a_combination_does_not() {
        let mut s = shell();
        s.handle(&key(Key::Meta(Side::Left), Modifiers::NONE), 100);
        s.handle(&key_up(Key::Meta(Side::Left)), 200);
        assert!(
            matches!(s.overlay, Overlay::Launcher { .. }),
            "{:?}",
            s.overlay
        );

        s.overlay = Overlay::None;
        s.handle(&key(Key::Meta(Side::Left), Modifiers::NONE), 300);
        s.handle(&key(Key::Char('e'), meta()), 320);
        s.handle(&key_up(Key::Meta(Side::Left)), 340);
        assert!(
            !matches!(s.overlay, Overlay::Launcher { .. }),
            "Win+E also opened the launcher"
        );
    }

    #[test]
    fn typing_in_the_launcher_filters_and_enter_opens_the_match() {
        let mut s = shell();
        s.perform(Action::ToggleLauncher);
        for ch in ["b", "r", "o"] {
            s.handle(
                &InputEvent::TextInput {
                    text: ch.to_owned(),
                },
                500,
            );
        }
        let Overlay::Launcher { ref query, .. } = s.overlay else {
            panic!("launcher closed while typing");
        };
        assert_eq!(query, "bro");
        let out = s.handle(&key(Key::Enter, Modifiers::NONE), 600);
        assert_eq!(out, vec![Effect::Launched(App::Browser)]);
        assert_eq!(s.overlay, Overlay::None);
        assert!(s.wm.all().iter().any(|w| w.app_id == App::Browser.app_id()));
    }

    #[test]
    fn backspace_in_the_launcher_deletes_rather_than_closing_it() {
        let mut s = shell();
        s.perform(Action::ToggleLauncher);
        s.handle(
            &InputEvent::TextInput {
                text: "ab".to_owned(),
            },
            100,
        );
        s.handle(&key(Key::Backspace, Modifiers::NONE), 200);
        let Overlay::Launcher { ref query, .. } = s.overlay else {
            panic!("launcher closed");
        };
        assert_eq!(query, "a");
    }

    #[test]
    fn escape_closes_the_launcher_and_leaves_the_desktop_alone() {
        let mut s = shell();
        s.perform(Action::ToggleLauncher);
        s.handle(&key(Key::Escape, Modifiers::NONE), 100);
        assert_eq!(s.overlay, Overlay::None);
    }

    #[test]
    fn the_launcher_does_not_type_into_the_desktop_when_it_is_closed() {
        let mut s = shell();
        s.handle(
            &InputEvent::TextInput {
                text: "hello".to_owned(),
            },
            100,
        );
        assert_eq!(s.overlay, Overlay::None);
    }

    #[test]
    fn alt_tab_opens_the_switcher_and_releasing_alt_commits() {
        let mut s = shell();
        s.open_app(App::Activity);
        s.open_app(App::Files);
        s.open_app(App::Terminal);
        let terminal = s.wm.focused();

        s.handle(&key(Key::Tab, alt()), 1000);
        assert!(
            matches!(s.overlay, Overlay::Switcher { selected: 1 }),
            "{:?}",
            s.overlay
        );
        s.handle(&key(Key::Tab, alt()), 1050);
        assert!(matches!(s.overlay, Overlay::Switcher { selected: 2 }));

        s.handle(&key_up(Key::Alt(Side::Left)), 1100);
        assert_eq!(s.overlay, Overlay::None, "switcher stayed open");
        assert_ne!(s.wm.focused(), terminal, "focus did not move");
    }

    #[test]
    fn alt_tab_with_nothing_open_does_nothing() {
        let mut s = shell();
        s.handle(&key(Key::Tab, alt()), 1000);
        assert_eq!(s.overlay, Overlay::None);
    }

    #[test]
    fn window_shortcuts_act_on_the_focused_window() {
        let mut s = shell();
        let id = {
            s.open_app(App::Activity);
            s.wm.focused().unwrap_or(0)
        };
        s.perform(Action::ToggleMaximize);
        assert_eq!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Maximized)
        );
        s.perform(Action::SnapLeft);
        assert!(matches!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Snapped(_))
        ));
        s.perform(Action::MinimizeWindow);
        assert_eq!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Minimized)
        );
        s.perform(Action::CloseWindow);
        // Focus moved off the minimized window, so Close acted on nothing;
        // the window is still there, which is correct and worth pinning down.
        assert!(s.wm.window(id).is_some());
    }

    #[test]
    fn opening_an_application_twice_focuses_the_first_window() {
        let mut s = shell();
        s.open_app(App::Files);
        let first = s.wm.focused();
        s.open_app(App::Activity);
        let out = s.open_app(App::Files);
        assert!(out.is_empty(), "a second window was opened: {out:?}");
        assert_eq!(s.wm.focused(), first);
        assert_eq!(
            s.wm.all()
                .iter()
                .filter(|w| w.app_id == App::Files.app_id())
                .count(),
            1
        );
    }

    #[test]
    fn workspaces_switch_and_windows_move_between_them() {
        let mut s = shell();
        s.open_app(App::Activity);
        let id = s.wm.focused().unwrap_or(0);
        s.perform(Action::MoveToWorkspace(1));
        assert!(
            s.wm.visible().is_empty(),
            "window did not leave workspace 1"
        );
        s.perform(Action::Workspace(1));
        assert!(s.wm.visible().iter().any(|w| w.id == id));
    }

    #[test]
    fn show_desktop_minimizes_everything() {
        let mut s = shell();
        s.open_app(App::Activity);
        s.open_app(App::Files);
        s.perform(Action::ShowDesktop);
        assert!(s.wm.visible().is_empty());
    }

    #[test]
    fn clicking_a_hotspot_activates_it_only_if_the_release_lands_there_too() {
        let mut s = shell();
        s.hotspots_mut()
            .push(Rect::new(0.0, 0.0, 40.0, 34.0), Hot::Logo);

        let down = InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            at: Point::new(20.0, 17.0),
            clicks: 1,
            mods: Modifiers::NONE,
        };
        let up_elsewhere = InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            at: Point::new(600.0, 400.0),
            clicks: 1,
            mods: Modifiers::NONE,
        };
        s.handle(&down, 100);
        s.handle(&up_elsewhere, 200);
        assert_eq!(
            s.overlay,
            Overlay::None,
            "a drag off the button activated it"
        );

        let up_on = InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            at: Point::new(20.0, 17.0),
            clicks: 1,
            mods: Modifiers::NONE,
        };
        s.handle(&down, 300);
        s.handle(&up_on, 400);
        // The wordmark opens the LitricOS menu (session settings, lock); the
        // launcher moved to the Apps menu, and a release on the mark must
        // activate the mark and nothing else.
        assert!(matches!(s.overlay, Overlay::Litro));
    }

    #[test]
    fn clicking_a_title_bar_starts_a_drag_and_a_double_click_maximizes() {
        let mut s = shell();
        s.open_app(App::Activity);
        let id = s.wm.focused().unwrap_or(0);
        let frame = s.wm.window(id).map(|w| w.frame).unwrap_or(WmRect::ZERO);
        let (cx, cy) = s.wm.chrome().title_rect(&frame).center();

        s.handle(
            &InputEvent::PointerButton {
                button: PointerButton::Primary,
                pressed: true,
                at: Point::new(cx, cy),
                clicks: 1,
                mods: Modifiers::NONE,
            },
            100,
        );
        assert!(s.wm.is_dragging(), "a title-bar press did not start a drag");
        s.handle(
            &InputEvent::PointerMove {
                at: Point::new(cx + 40.0, cy + 20.0),
                delta: Point::new(40.0, 20.0),
            },
            120,
        );
        assert_eq!(s.wm.window(id).map(|w| w.frame.x), Some(frame.x + 40.0));
        s.handle(
            &InputEvent::PointerButton {
                button: PointerButton::Primary,
                pressed: false,
                at: Point::new(cx + 40.0, cy + 20.0),
                clicks: 1,
                mods: Modifiers::NONE,
            },
            140,
        );
        assert!(!s.wm.is_dragging());

        let frame2 = s.wm.window(id).map(|w| w.frame).unwrap_or(WmRect::ZERO);
        let (cx2, cy2) = s.wm.chrome().title_rect(&frame2).center();
        s.handle(
            &InputEvent::PointerButton {
                button: PointerButton::Primary,
                pressed: true,
                at: Point::new(cx2, cy2),
                clicks: 2,
                mods: Modifiers::NONE,
            },
            200,
        );
        assert_eq!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Maximized)
        );
    }

    #[test]
    fn the_close_button_closes_the_window_it_is_on() {
        let mut s = shell();
        s.open_app(App::Activity);
        let id = s.wm.focused().unwrap_or(0);
        let frame = s.wm.window(id).map(|w| w.frame).unwrap_or(WmRect::ZERO);
        let (bx, by) = s.wm.chrome().button_rect(&frame, 0).center();
        s.handle(
            &InputEvent::PointerButton {
                button: PointerButton::Primary,
                pressed: true,
                at: Point::new(bx, by),
                clicks: 1,
                mods: Modifiers::NONE,
            },
            100,
        );
        assert!(
            s.wm.window(id).is_none(),
            "the close button did not close it"
        );
    }

    #[test]
    fn clicking_outside_an_overlay_dismisses_it() {
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        s.handle(
            &InputEvent::PointerButton {
                button: PointerButton::Primary,
                pressed: true,
                at: Point::new(200.0, 600.0),
                clicks: 1,
                mods: Modifiers::NONE,
            },
            100,
        );
        assert_eq!(s.overlay, Overlay::None);
    }

    #[test]
    fn a_quick_settings_toggle_changes_real_state() {
        let mut s = shell();
        s.hotspots_mut()
            .push(Rect::new(10.0, 10.0, 100.0, 30.0), Hot::QuickToggle(0));
        assert!(!s.quick.do_not_disturb);
        s.activate(Hot::QuickToggle(0));
        assert!(s.quick.do_not_disturb);
        assert!(
            s.notifications.do_not_disturb,
            "Do Not Disturb did not reach the notification queue"
        );
    }

    #[test]
    fn choosing_a_power_profile_emits_an_effect_rather_than_only_a_label() {
        let mut s = shell();
        let out = s.activate(Hot::QuickPower(2));
        assert_eq!(s.quick.power, PowerProfile::Performance);
        assert_eq!(
            out,
            vec![Effect::SetPowerProfile(PowerProfile::Performance)]
        );
    }

    #[test]
    fn adaptive_idle_lowers_the_frame_rate_and_input_brings_it_straight_back() {
        let mut s = shell();
        s.handle(&key(Key::Char('x'), Modifiers::NONE), 1_000);
        assert_eq!(s.life(), LifeState::Active);
        assert_eq!(s.frame_interval_ms(), 16);

        s.tick(1_000 + 25_000);
        assert_eq!(s.life(), LifeState::Idle);
        assert_eq!(s.frame_interval_ms(), 33, "idle should target 30 FPS");

        s.tick(1_000 + 200_000);
        assert_eq!(s.life(), LifeState::DeepIdle);
        assert!(s.frame_interval_ms() >= 200);

        s.handle(
            &InputEvent::PointerMove {
                at: Point::new(10.0, 10.0),
                delta: Point::new(1.0, 0.0),
            },
            1_000 + 200_100,
        );
        assert_eq!(
            s.life(),
            LifeState::Active,
            "input did not wake the compositor"
        );
        assert!(s.needs_full_redraw(), "waking did not repaint");
    }

    #[test]
    fn dragging_a_window_gets_the_fastest_frame_budget() {
        let mut s = shell();
        s.open_app(App::Activity);
        let id = s.wm.focused().unwrap_or(0);
        let frame = s.wm.window(id).map(|w| w.frame).unwrap_or(WmRect::ZERO);
        let hit =
            s.wm.hit_test(frame.center().0, frame.y + 8.0)
                .expect("title bar");
        s.wm.begin_drag(hit, frame.center().0, frame.y + 8.0);
        assert_eq!(s.life(), LifeState::WindowInteraction);
        assert!(s.frame_interval_ms() <= 8);
    }

    #[test]
    fn an_idle_desktop_sleeps_instead_of_polling_its_own_frame_rate() {
        let mut s = shell();
        let t = 2_000;
        s.tick(t);
        let _ = s.take_damage();
        assert!(!s.is_dirty(), "fixture: nothing should need drawing");
        assert_eq!(
            s.life(),
            LifeState::Active,
            "fixture: two seconds is not idle"
        );

        let sleep = s.wake_after_ms(t);
        assert!(
            sleep > s.frame_interval_ms(),
            "a desktop with nothing to draw asked to be woken in {sleep} ms, which is its own \
             frame pace: the wake-up is the bug, not the frame"
        );
        assert!(
            sleep <= Shell::IDLE_SLEEP_CAP_MS,
            "{sleep} ms is longer than the clock can be allowed to go stale"
        );
        assert!(
            sleep >= 250,
            "{sleep} ms is close enough to a frame that the loop is still spinning"
        );
    }

    #[test]
    fn motion_that_outlives_the_idle_timer_still_runs_at_the_active_pace() {
        let mut s = shell();
        s.activate(Hot::Logo);
        let t = 30_000;
        s.tick(t);
        assert_eq!(s.life(), LifeState::Idle, "fixture: the desktop is idle");
        assert!(s.anims().animating(), "fixture: the launcher is opening");
        assert_eq!(
            s.wake_after_ms(t),
            LifeState::Active.frame_interval_ms(),
            "an opening panel must not run at half speed because nobody has typed"
        );
    }

    #[test]
    fn a_spring_settles_within_its_token_budget_at_every_frame_pace() {
        // The defect this pins down: `tick` stepped every spring by a constant
        // 16 ms, so at the idle pace (33 ms) an animation took twice its
        // token's budget in wall time, and under a drag (8 ms) half of it.
        // A settle budget is in milliseconds and has to mean milliseconds.
        for pace in [8_u64, 16, 33, 100] {
            let mut s = shell();
            s.activate(Hot::Logo);
            let mut t = 1_000;
            s.tick(t);
            let started = t;
            assert!(
                !s.anims().launcher.settled(),
                "fixture: the launcher's spring should be opening"
            );
            let mut steps: u32 = 0;
            // This waits on the launcher's own spring, whose token is FAST.
            // `animating()` would also wait for the dock's morph, which is a
            // different token with a different budget, and this test is about
            // one budget meaning one number of milliseconds.
            while !s.anims().launcher.settled() && steps < 400 {
                t = t.saturating_add(pace);
                s.tick(t);
                steps = steps.saturating_add(1);
            }
            assert!(
                s.anims().launcher.settled(),
                "at a {pace} ms pace the launcher never settled"
            );
            let elapsed = t.saturating_sub(started);
            let budget = u64::from(litric_design::Motion::FAST.settle_ms);
            assert!(
                elapsed <= budget.saturating_add(pace),
                "at a {pace} ms pace the panel took {elapsed} ms to settle, over its {budget} ms budget"
            );
        }
    }

    #[test]
    fn turning_adaptive_idle_off_keeps_the_compositor_active() {
        let mut s = shell();
        s.quick.adaptive_idle = false;
        s.tick(500_000);
        assert_eq!(s.life(), LifeState::Active);
    }

    #[test]
    fn a_network_rate_is_a_difference_and_nothing_is_invented_without_one() {
        // 2 000 bytes received in two seconds is 1 000 bytes a second. The
        // counters are cumulative, so this is the only way to know it.
        let first = SystemFacts {
            sampled_ms: Some(1_000),
            net_rx_bytes: Some(1_000),
            net_tx_bytes: Some(500),
            ..SystemFacts::default()
        };
        let second = SystemFacts {
            sampled_ms: Some(3_000),
            net_rx_bytes: Some(3_000),
            net_tx_bytes: Some(1_500),
            ..SystemFacts::default()
        };
        let mut seen = first.clone();
        seen.observe(second.clone());
        assert_eq!(seen.net_rx_per_s, Some(1_000.0));
        assert_eq!(seen.net_tx_per_s, Some(500.0));

        // One sample is not a rate, whatever it says.
        let mut only = SystemFacts::default();
        only.observe(first.clone());
        assert_eq!(only.net_rx_per_s, None, "a first sample has no interval");

        // An interface that reset reports nothing rather than a negative rate.
        let after_reset = SystemFacts {
            sampled_ms: Some(5_000),
            net_rx_bytes: Some(10),
            net_tx_bytes: Some(1_500),
            ..SystemFacts::default()
        };
        let mut reset = second.clone();
        reset.observe(after_reset);
        assert_eq!(reset.net_rx_per_s, None, "a counter that went backwards");

        // A machine that reports no counters keeps saying so.
        let mut bare = SystemFacts::default();
        bare.observe(SystemFacts {
            sampled_ms: Some(1_000),
            ..SystemFacts::default()
        });
        assert_eq!(bare.net_rx_per_s, None);
    }

    #[test]
    fn an_idle_desktop_asks_for_no_repaint_at_all() {
        let mut s = shell();
        let _ = s.take_damage();
        s.tick(2_000);
        assert!(!s.is_dirty(), "an idle desktop asked to be repainted");
    }

    #[test]
    fn moving_the_pointer_damages_only_the_cursor_and_what_it_crossed() {
        let mut s = shell();
        let _ = s.take_damage();
        s.handle(
            &InputEvent::PointerMove {
                at: Point::new(400.0, 400.0),
                delta: Point::new(4.0, 0.0),
            },
            100,
        );
        let d = s.take_damage();
        assert!(!d.is_empty());
        let area: f32 = d.iter().map(|r| r.w * r.h).sum();
        assert!(
            area < 1280.0 * 800.0 * 0.05,
            "a pointer move damaged {area} pixels of a 1024000-pixel screen"
        );
    }

    #[test]
    fn closing_a_window_damages_all_of_its_shadow() {
        let mut s = shell();
        s.open_app(App::Activity);
        let id = s.wm.focused().unwrap_or(0);
        let frame = to_rect(s.wm.window(id).map(|w| w.frame).unwrap_or(WmRect::ZERO));
        let _ = s.take_damage();
        s.perform(Action::CloseWindow);
        let d = s.take_damage();
        // A focused window's shadow reaches 30 * 11/12 + 12 + 0.5 = 40 px below
        // its frame and 28 px to each side. The window manager alone damaged 24.
        assert!(
            d.iter().any(|r| r.x <= frame.x - 28.0
                && r.right() >= frame.right() + 28.0
                && r.bottom() >= frame.bottom() + 40.0),
            "{d:?}"
        );
    }

    #[test]
    fn a_focus_change_damages_both_windows_and_the_top_bar() {
        let mut s = shell();
        s.open_app(App::Activity);
        s.open_app(App::Files);
        let files = s.wm.focused().unwrap_or(0);
        let activity =
            s.wm.all()
                .iter()
                .find(|w| w.app_id == App::Activity.app_id())
                .map_or(0, |w| w.id);
        let _ = s.take_damage();
        assert!(!s.is_dirty());

        s.wm.focus_window(activity);
        assert!(s.is_dirty());
        let d = s.take_damage();
        let covered = |id: WindowId| {
            let f = to_rect(s.wm.window(id).map_or(WmRect::ZERO, |w| w.frame));
            d.iter().any(|r| {
                r.x <= f.x && r.y <= f.y && r.right() >= f.right() && r.bottom() >= f.bottom()
            })
        };
        assert!(covered(files), "the window that lost focus: {d:?}");
        assert!(covered(activity), "the window that gained focus: {d:?}");
        assert!(
            d.contains(&s.top_bar()),
            "the top bar names the focused window: {d:?}"
        );
        let _ = s.take_damage();
        assert!(!s.is_dirty(), "the focus change was reported twice");
    }

    #[test]
    fn crossing_into_a_window_button_repaints_the_whole_button() {
        let mut s = shell();
        s.open_app(App::Activity);
        let id = s.wm.focused().unwrap_or(0);
        let frame = s.wm.window(id).map_or(WmRect::ZERO, |w| w.frame);
        let b = to_rect(s.wm.chrome().button_rect(&frame, 1));
        let far = Point::new(b.right() + 40.0, b.bottom() + 80.0);
        s.handle(
            &InputEvent::PointerMove {
                at: far,
                delta: Point::new(1.0, 1.0),
            },
            100,
        );
        let _ = s.take_damage();
        let (cx, cy) = b.center();
        s.handle(
            &InputEvent::PointerMove {
                at: Point::new(cx, cy),
                delta: Point::new(1.0, 1.0),
            },
            200,
        );
        let d = s.take_damage();
        assert!(
            d.iter().any(|r| r.x <= b.x
                && r.y <= b.y
                && r.right() >= b.right()
                && r.bottom() >= b.bottom()),
            "the glyph appears on the whole button but {b:?} was not damaged: {d:?}"
        );
    }

    #[test]
    fn opening_a_window_repaints_the_window_and_an_overlay_repaints_everything() {
        let mut s = shell();
        let _ = s.take_damage();
        s.open_app(App::Files);
        assert!(s.is_dirty());
        assert!(
            !s.needs_full_redraw(),
            "opening a window repainted the screen"
        );
        s.perform(Action::ToggleLauncher);
        assert!(s.needs_full_redraw());
    }

    #[test]
    fn facts_damage_only_what_displays_them() {
        let mut s = shell();
        s.facts.cpu_fraction = Some(0.071);
        s.facts.uptime_secs = Some(100.2);
        let _ = s.take_damage();

        // Nothing shows the CPU count, and 7.1% and 7.4% both read "7%".
        let mut f = s.facts.clone();
        f.cpus = Some(4);
        f.cpu_fraction = Some(0.074);
        s.set_facts(f.clone());
        assert!(!s.is_dirty(), "{:?}", s.take_damage());

        // Uptime is shown only by Activity, and no Activity window is open.
        f.uptime_secs = Some(160.0);
        s.set_facts(f.clone());
        assert!(!s.is_dirty());

        // A CPU change that reads differently repaints the top bar only.
        f.cpu_fraction = Some(0.35);
        s.set_facts(f.clone());
        let bar = s.top_bar();
        assert_eq!(s.take_damage(), vec![bar]);

        s.open_app(App::Activity);
        let _ = s.take_damage();
        let frame = to_rect(
            s.wm.all()
                .iter()
                .find(|w| w.app_id == App::Activity.app_id())
                .map_or(WmRect::ZERO, |w| w.frame),
        );
        f.uptime_secs = Some(200.0);
        s.set_facts(f.clone());
        assert_eq!(s.take_damage(), vec![frame], "uptime with Activity open");

        // 35.2% still reads "35%" in the top bar, but the meter bar moves.
        f.cpu_fraction = Some(0.352);
        s.set_facts(f);
        assert_eq!(s.take_damage(), vec![frame]);
    }

    #[test]
    fn the_clock_repaints_the_top_bar_only_when_its_text_changes() {
        let mut s = shell();
        s.tick(60_000);
        let _ = s.take_damage();
        s.tick(61_000);
        s.tick(119_999);
        assert!(!s.is_dirty(), "the clock text did not change");
        s.tick(120_000);
        let bar = s.top_bar();
        assert_eq!(s.take_damage(), vec![bar]);
        assert_eq!(s.next_clock_change_ms(), 180_000);
    }

    #[test]
    fn the_next_expiry_is_the_earliest_one() {
        let mut n = Notifications::default();
        assert_eq!(n.next_expiry(), None);
        n.post("t", "a", "", Priority::Normal, 1_000, Some(9_000));
        n.post("t", "b", "", Priority::Normal, 2_000, Some(3_000));
        n.post("t", "c", "", Priority::Critical, 0, Some(1));
        assert_eq!(n.next_expiry(), Some(5_000), "critical never expires");
    }

    #[test]
    fn notifications_expire_but_critical_ones_never_do() {
        let mut n = Notifications::default();
        n.post(
            "test",
            "Transient",
            "",
            Priority::Normal,
            1_000,
            Some(5_000),
        );
        n.post(
            "test",
            "Disk full",
            "",
            Priority::Critical,
            1_000,
            Some(5_000),
        );
        assert_eq!(n.len(), 2);
        assert_eq!(n.expire(7_000), 1);
        assert_eq!(n.len(), 1);
        assert_eq!(
            n.items().first().map(|x| x.priority),
            Some(Priority::Critical)
        );
    }

    #[test]
    fn clearing_notifications_leaves_the_critical_ones() {
        let mut n = Notifications::default();
        n.post("t", "a", "", Priority::Normal, 0, None);
        n.post("t", "b", "", Priority::Critical, 0, None);
        assert_eq!(n.clear(), 1);
        assert_eq!(n.len(), 1);
    }

    #[test]
    fn the_notification_queue_is_bounded_and_drops_the_oldest_ordinary_one() {
        let mut n = Notifications::default();
        n.post("t", "keep me", "", Priority::Critical, 0, None);
        for i in 0..Notifications::CAPACITY + 10 {
            n.post("t", &format!("n{i}"), "", Priority::Normal, 0, None);
        }
        assert_eq!(n.len(), Notifications::CAPACITY);
        assert!(
            n.items().iter().any(|x| x.title == "keep me"),
            "a critical notification was evicted"
        );
    }

    #[test]
    fn do_not_disturb_silences_everything_except_critical() {
        let n = Notifications {
            do_not_disturb: true,
            ..Notifications::default()
        };
        assert!(!n.should_banner(Priority::Normal));
        assert!(!n.should_banner(Priority::High));
        assert!(
            n.should_banner(Priority::Critical),
            "DND hid a critical alert"
        );
    }

    #[test]
    fn a_non_interactive_desktop_ignores_input() {
        let mut s = shell();
        s.mode = DesktopMode::Headless {
            reason: "no framebuffer".to_owned(),
        };
        let out = s.handle(&key(Key::Char('1'), meta()), 100);
        assert!(out.is_empty(), "headless mode acted on a shortcut");
        assert!(!s.mode.is_interactive());
    }

    #[test]
    fn degraded_mode_still_works_and_names_what_is_missing() {
        let mut s = shell();
        s.mode = DesktopMode::Degraded {
            reason: "no keyboard".to_owned(),
        };
        assert!(s.mode.is_interactive());
        assert!(s.mode.token().contains("no keyboard"));
        assert_eq!(
            s.handle(&key(Key::Char('1'), meta()), 100),
            vec![Effect::SwitchEnvironment(1)]
        );
    }

    #[test]
    fn unknown_facts_are_reported_as_unavailable_rather_than_zero() {
        assert_eq!(SystemFacts::or_unknown(None::<u32>), "Unavailable");
        assert_eq!(SystemFacts::or_unknown(Some(8u32)), "8");
        let f = SystemFacts::default();
        assert_eq!(SystemFacts::or_unknown(f.gpu_fraction), "Unavailable");
    }

    #[test]
    fn scrolling_the_top_bar_changes_workspace() {
        let mut s = shell();
        assert_eq!(s.wm.active_workspace(), 0);
        s.handle(
            &InputEvent::Scroll {
                dx: 0.0,
                dy: 1.0,
                at: Point::new(600.0, 10.0),
                mods: Modifiers::NONE,
            },
            100,
        );
        assert_eq!(s.wm.active_workspace(), 1);
    }

    #[test]
    fn scrolling_the_desktop_does_not_change_workspace() {
        let mut s = shell();
        s.handle(
            &InputEvent::Scroll {
                dx: 0.0,
                dy: 1.0,
                at: Point::new(600.0, 500.0),
                mods: Modifiers::NONE,
            },
            100,
        );
        assert_eq!(s.wm.active_workspace(), 0);
    }

    #[test]
    fn diagnostics_report_the_state_that_matters() {
        let mut s = shell();
        s.open_app(App::Activity);
        let d = s.diagnostic_lines();
        assert!(d.iter().any(|l| l == "mode=ready"), "{d:?}");
        assert!(d.iter().any(|l| l.starts_with("life=")), "{d:?}");
        assert!(
            d.iter().any(|l| l.starts_with("frame_interval_ms=")),
            "{d:?}"
        );
        assert!(d.iter().any(|l| l.starts_with("windows=")), "{d:?}");
        assert!(d.iter().any(|l| l.starts_with("tz.")), "{d:?}");
    }

    #[test]
    fn the_fps_line_is_gated_by_its_setting_and_counts_a_real_second() {
        let mut s = shell();
        // Frames presented every 16 ms: a hair over 60 per second. The probe
        // divides by the window's real length, so this cadence must read in
        // the low sixties, not at some nominal 60.
        for i in 0..=80_u64 {
            s.tick(i.saturating_mul(16));
            s.counted_frame();
        }
        let quiet = s.diagnostic_lines();
        assert!(
            !quiet.iter().any(|l| l.starts_with("fps=")),
            "the fps line is off by default: {quiet:?}"
        );
        let _ = s
            .settings
            .values
            .set("diagnostics.fps", crate::settings::Value::On(true));
        let loud = s.diagnostic_lines();
        let fps = loud
            .iter()
            .find_map(|l| l.strip_prefix("fps="))
            .unwrap_or_else(|| panic!("no fps line in {loud:?}"));
        let rate: u32 = fps.parse().expect("a number");
        assert!((55..=65).contains(&rate), "fps={rate} from a 16 ms cadence");
    }
}
