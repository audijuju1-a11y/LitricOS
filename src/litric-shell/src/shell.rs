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
    /// Mission Control: every window, on every workspace, at once.
    Overview {
        /// Which card is highlighted.
        selected: usize,
    },
    /// Quick settings.
    QuickSettings,
    /// The power menu under the power and person icons on the right of the
    /// system bar: lock, sleep, shut down, restart.
    Power,
    /// The LitricOS menu under the wordmark at the left of the system bar:
    /// the session's own settings, and the lock.
    Litro,
    /// The notification list.
    Notifications,
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
            Self::Overview { .. } => "overview",
            Self::QuickSettings => "quick-settings",
            Self::Power => "power-menu",
            Self::Litro => "litro-menu",
            Self::Notifications => "notifications",
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
    /// The user menu.
    StatusUser,
    /// A row in the power menu: lock, sleep, shut down, restart.
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
    /// A window card in Mission Control.
    OverviewItem(usize),
    /// The dimmed desktop behind Mission Control: a click here puts the
    /// overview away without choosing a window.
    OverviewBackdrop,
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
    /// Change the shell interface language.
    SettingsLanguage,
    /// Cycle the supported colour themes.
    SettingsTheme,
    /// Cycle the accent colour choices.
    SettingsAccent,
    /// Cycle the wallpaper choices.
    SettingsWallpaper,
    /// Switch the appearance schedule between manual and automatic.
    SettingsSchedule,
    /// Toggle reduced motion in the shell.
    SettingsMotion,
    /// Finish first-run setup and show the desktop.
    SetupDone,
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
    /// Whether the daemon says locking is refused (`auth.lock=refused`).
    ///
    /// `None` when the line was not in the reply at all, which is what a
    /// daemon that did not answer looks like. Absent and "not refused" are the
    /// same thing to a lock screen: without a verifier there is nothing to
    /// unlock with, so [`Shell::can_lock`] refuses in both cases.
    pub auth_lock: Option<bool>,
    /// The desktop account a lock checks (`auth.account`), not whichever
    /// logind session happens to be listed first: the diagnostic serial
    /// console autologins as root, and checking root's locked password
    /// against the password first-run set for `litric` can never unlock.
    pub auth_account: Option<String>,
    /// `set`, `locked`, `absent` or `unknown` (`auth.password`).
    pub auth_password: Option<String>,
    /// Why the daemon refuses the lock (`auth.reason`).
    pub auth_reason: Option<String>,
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
    /// Take one sample of the machine.
    ///
    /// The process table's rates are differenced against the previous sample, so
    /// the monitor has to see every sample the shell does; doing it in one place
    /// is what makes that true by construction rather than by remembering.
    pub fn sample(&mut self, fresh: SystemFacts) {
        // The shell owns the clock, so the sample carries its time: a byte rate
        // cannot be computed from counters alone. A caller that already stamped
        // the facts -- a replay, a test -- keeps its own stamp.
        let mut fresh = fresh;
        if fresh.sampled_ms.is_none() {
            fresh.sampled_ms = Some(self.now_ms);
        }
        self.facts.observe(fresh);
        self.activity.observe(&self.facts);
    }
}

/// Something `main` has to do that the shell cannot do itself.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    PersistPreferences(crate::settings::Preferences),
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
    /// Adjust the volume by a step.
    Volume(i8),
    /// Adjust the brightness by a step.
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

    /// Drain the browser link's inbox.
    ///
    /// `TAB` lines update the tab they name, a `FRAME` payload becomes the
    /// pending framebuffer, `ERR` lines become the status the page area shows,
    /// and a closed link is torn down with its reason. A frame for a tab that
    /// is not on screen is dropped: drawing one tab's pixels under another
    /// tab's address bar is the stale-frame bug, not a rendering nicety.
    pub fn pump_browser(&mut self) {
        use crate::link::Incoming;
        let Some(link) = &self.browser_session else {
            return;
        };
        let messages = link.drain();
        if messages.is_empty() {
            return;
        }
        let mut closed = None;
        for m in messages {
            match m {
                Incoming::Payload { header, bytes } => self.accept_frame(&header, bytes),
                Incoming::Line(line) => {
                    if let Some(update) = parse_tab_line(&line) {
                        self.apply_tab_update(&update);
                    } else if let Some(rest) = line.strip_prefix("HELLO ") {
                        self.browser.status = format!("engine ready: {rest}");
                    } else if let Some(rest) = line.strip_prefix("ERR ") {
                        self.apply_engine_error(rest);
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
                Incoming::Closed(why) => closed = Some(why),
            }
        }
        if let Some(why) = closed {
            self.browser_session = None;
            self.browser.status = format!("engine session ended ({why})");
            self.request_full_redraw();
        }
    }

    /// Take one `FRAME <tab> <w> <h> <len>` payload and its pixels.
    fn accept_frame(&mut self, header: &str, bytes: Vec<u8>) {
        let Some(tab) = frame_tab(header) else {
            return;
        };
        if Some(tab) != self.browser.active_engine_tab() {
            return;
        }
        let Some((w, h)) = parse_frame_dims(header) else {
            self.browser.status = "engine sent a frame with nonsense dimensions".to_owned();
            return;
        };
        let expected = u64::from(w)
            .checked_mul(u64::from(h))
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| usize::try_from(n).ok());
        if expected != Some(bytes.len()) {
            // Never scale a short buffer over the page: the bytes that are
            // missing would be read as another row's pixels.
            self.browser.status = format!("engine sent {} bytes for a {w}x{h} frame", bytes.len());
            return;
        }
        self.browser_frame = Some(Arc::from(bytes.into_boxed_slice()));
        self.browser.last_frame = Some((w, h));
        // The TAB line that precedes every frame already put the page's own
        // state and title here ("loaded - Example", "failed", ...). Writing a
        // generic "page loaded" over it hid the title and told the user the
        // page had loaded even when the line said otherwise, so the status is
        // left to the line that actually knows.
        self.request_full_redraw();
    }

    /// Apply one `TAB` line to the tab whose engine id it names.
    fn apply_tab_update(&mut self, update: &TabUpdate) {
        let Some(index) = self
            .browser
            .tabs
            .iter()
            .position(|tab| tab.engine_tab == update.tab)
        else {
            return;
        };
        let active = index == self.browser.active;
        let url = update.url.clone();
        if let Some(tab) = self.browser.tabs.get_mut(index) {
            if !update.url.is_empty() {
                tab.url.clone_from(&url);
            }
            if !update.title.is_empty() {
                tab.title.clone_from(&update.title);
            }
            tab.error = update.state == "failed";
        }
        if active {
            if !self.browser.omnibox_focus {
                self.browser.omnibox.clone_from(&url);
            }
            let title = self
                .browser
                .tabs
                .get(index)
                .map(|tab| tab.title.clone())
                .unwrap_or_default();
            self.browser.status = if update.state.is_empty() {
                title
            } else {
                format!("{} — {title}", update.state)
            };
            if update.state == "loaded" && !url.is_empty() {
                self.browser.record_visit(&url);
            }
        }
        self.request_full_redraw();
    }

    /// Put the engine's reason for refusing on the browser's status line.
    fn apply_engine_error(&mut self, rest: &str) {
        let (code, detail) = rest.split_once(' ').unwrap_or((rest, ""));
        let detail = litric_core::pct::decode(detail).unwrap_or_else(|_| detail.to_owned());
        if let Some(tab) = self.browser.tabs.get_mut(self.browser.active) {
            tab.error = true;
        }
        self.browser.status = match code {
            "engine-unavailable" => format!("the browser engine is not available: {detail}"),
            "engine-failed" => format!("the page could not be shown: {detail}"),
            "bad-request" => format!("the engine rejected a command: {detail}"),
            _ => format!("the engine reported: {detail}"),
        };
        self.request_full_redraw();
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
        let socket = self.browser_socket.clone();
        match crate::link::Link::connect(&socket, framing, Box::new(|| {})) {
            Ok(link) => {
                self.browser_session = Some(link);
                self.browser.wants_engine = true;
                self.browser.status = format!("engine session started ({})", socket.display());
                // Handshake before the first navigation: the broker says which
                // engine it owns, and an image without Chromium says so here
                // instead of after a URL has been typed.
                let (w, h) = self.browser_viewport;
                self.send_line("HELLO 1");
                self.send_line(&format!("VIEWPORT {w} {h}"));
                // The session exists to show something: navigate the active
                // tab right away, the way a browser loads its start page.
                if let Some(url) = self.browser.navigate_active() {
                    self.send_go(&url);
                }
                Vec::new()
            }
            Err(e) => {
                self.browser.status = format!(
                    "browser engine not reachable at {} ({e}); the image needs litric-browserd and its socket",
                    socket.display()
                );
                self.request_full_redraw();
                Vec::new()
            }
        }
    }

    /// Send one LBP/1 line. A write failure tears the session down: a link whose
    /// commands cannot be written must not keep drawing itself as connected.
    fn send_line(&mut self, line: &str) -> bool {
        let Some(link) = self.browser_session.as_mut() else {
            return false;
        };
        match link.send_line(line) {
            Ok(()) => true,
            Err(e) => {
                self.browser.status = format!("engine write failed: {e}");
                self.browser_session = None;
                self.request_full_redraw();
                false
            }
        }
    }

    /// Send one `GO <tab> <url>` to the engine session.
    ///
    /// This is the line a search or an address commit has to produce; a
    /// navigation that stops at the model never lights a pixel of the web.
    fn send_go(&mut self, url: &str) {
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        let pct = litric_core::pct::encode(url);
        // A navigation invalidates the previous picture at once: keeping the
        // old page under the new address is exactly the stale frame this
        // browser must never show.
        self.browser_frame = None;
        self.browser.last_frame = None;
        self.browser_pointer_sent = None;
        if self.send_line(&format!("GO {tab} {pct}")) {
            self.browser.status = format!("loading {url}");
            self.request_full_redraw();
        }
    }

    /// Navigate to a URL: start the engine session if needed, then GO.
    pub fn browser_navigate(&mut self, url: &str) {
        if self.browser_session.is_none() {
            self.browser_start_session();
            // A successful start already navigated the tab it found; when the
            // caller asked for a different address, that address has to be
            // sent after all, or the first navigation of a session is ignored.
            if self.browser_session.is_some() && self.browser.tab().is_none_or(|tab| tab.url != url)
            {
                self.send_go(url);
            }
            self.request_full_redraw();
            return;
        }
        self.send_go(url);
        self.request_full_redraw();
    }

    /// Where the engine frame was drawn, once the renderer has placed one.
    pub fn set_browser_engine_area(&mut self, area: Option<Rect>) {
        self.browser_engine_area = area;
    }

    /// Whether the browser's page surface owns the keyboard right now.
    fn browser_page_focused(&self) -> bool {
        if self.overlay.is_open() || self.desktop_dialog.is_some() || self.browser.omnibox_focus {
            return false;
        }
        if self.browser.panel != crate::apps::BrowserPanel::Page {
            return false;
        }
        let Some(focused) = self.wm.focused() else {
            return false;
        };
        self.wm
            .all()
            .iter()
            .any(|w| w.id == focused && w.app_id == App::Browser.app_id())
    }

    /// Whether a key is the page's rather than the shell's.
    fn engine_takes_key(&self, key: Key, mods: Modifiers) -> bool {
        if mods.meta {
            return false;
        }
        // Alt+Tab is the window switcher even while a page is focused.
        if mods.alt && key == Key::Tab {
            return false;
        }
        self.browser_page_focused()
    }

    /// A pointer position on the desktop as a pixel in the page Chromium is
    /// rendering, when the browser owns the pointer and a frame defines the
    /// mapping.
    fn browser_page_point(&self, at: Point) -> Option<(f64, f64)> {
        if !self.browser_page_focused() {
            return None;
        }
        let area = self.browser_engine_area?;
        let (w, h) = self.browser.last_frame?;
        if area.w <= 0.0 || area.h <= 0.0 || w == 0 || h == 0 {
            return None;
        }
        if !area.contains(at.x, at.y) {
            return None;
        }
        let x = f64::from((at.x - area.x) / area.w) * f64::from(w);
        let y = f64::from((at.y - area.y) / area.h) * f64::from(h);
        Some((
            x.clamp(0.0, f64::from(w.saturating_sub(1))),
            y.clamp(0.0, f64::from(h.saturating_sub(1))),
        ))
    }

    /// Forward a pointer move, coalesced to whole page pixels.
    fn browser_engine_move(&mut self, at: Point) {
        let Some((x, y)) = self.browser_page_point(at) else {
            self.browser_pointer_sent = None;
            return;
        };
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        if let Some((px, py)) = self.browser_pointer_sent {
            if (px - x).abs() < 2.0 && (py - y).abs() < 2.0 {
                return;
            }
        }
        if self.send_line(&format!("MOUSE {tab} move {x:.1} {y:.1} none 0 0")) {
            self.browser_pointer_sent = Some((x, y));
        }
    }

    /// Forward a press or release, then ask for the frame that shows it.
    fn browser_engine_button(
        &mut self,
        button: PointerButton,
        pressed: bool,
        at: Point,
        clicks: u8,
        mods: Modifiers,
    ) {
        let Some((x, y)) = self.browser_page_point(at) else {
            return;
        };
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        let name = match button {
            PointerButton::Primary => "left",
            PointerButton::Secondary => "right",
            PointerButton::Middle => "middle",
            // Forward/back and any future button are not part of the engine's
            // mouse model here; they are the shell's to interpret, not a click
            // to invent.
            _ => return,
        };
        let kind = if pressed { "down" } else { "up" };
        let clicks = if pressed { clicks.max(1) } else { 1 };
        let line = format!(
            "MOUSE {tab} {kind} {x:.1} {y:.1} {name} {clicks} {}",
            modifier_bits(mods)
        );
        if self.send_line(&line) {
            self.browser_pointer_sent = Some((x, y));
            if !pressed {
                self.browser_refresh();
            }
        }
    }

    /// Forward a wheel notch (positive `dy` scrolls down).
    fn browser_engine_wheel(&mut self, dy: f32, at: Point) {
        let Some((x, y)) = self.browser_page_point(at) else {
            return;
        };
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        // Chromium counts wheel deltas in pixels, the shell in lines.
        let delta = f64::from(dy) * 60.0;
        if self.send_line(&format!("WHEEL {tab} {x:.1} {y:.1} 0 {delta:.1}")) {
            self.browser_refresh();
        }
    }

    /// Forward one key event. Text arrives separately, as `TEXT`, so a key that
    /// produces a character is never typed twice.
    fn browser_engine_key(&mut self, key: Key, down: bool, mods: Modifiers) {
        if key.is_modifier() {
            return;
        }
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        let (name, code, vk, text) = engine_key(key);
        if name.is_empty() {
            return;
        }
        let kind = if down { "down" } else { "up" };
        let line = format!(
            "KEY {tab} {kind} {} {} {vk} {} {}",
            litric_core::pct::encode(&name),
            litric_core::pct::encode(&code),
            modifier_bits(mods),
            litric_core::pct::encode(if down { text.as_str() } else { "" })
        );
        if self.send_line(&line) && down {
            self.browser_refresh();
        }
    }

    /// Forward typed text, including anything an input method committed.
    fn browser_engine_text(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if clean.is_empty() {
            return;
        }
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        if self.send_line(&format!("TEXT {tab} {}", litric_core::pct::encode(&clean))) {
            self.browser_refresh();
        }
    }

    /// Ask for a fresh frame, at most every 50 ms: a held key or a spin of the
    /// wheel must not queue a screenshot each time.
    fn browser_refresh(&mut self) {
        const MIN_GAP_MS: u64 = 50;
        // The start page is the shell's own surface; there is nothing for the
        // engine to capture on it.
        if !self.browser.active_tab_has_page() {
            return;
        }
        if self.browser_refresh_ms != 0
            && self.now_ms.saturating_sub(self.browser_refresh_ms) < MIN_GAP_MS
        {
            return;
        }
        let Some(tab) = self.browser.active_engine_tab() else {
            return;
        };
        self.browser_refresh_ms = self.now_ms.max(1);
        self.send_line(&format!("ACTIVATE {tab}"));
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
            // A command run from the Terminal is a real application launch,
            // so it gets the session's display environment. Without this a GUI
            // program started here would look for a server that is running
            // right next to it and find nothing.
            let envs = self.application_env();
            let spawn = std::thread::Builder::new()
                .name("litric-terminal".to_owned())
                .spawn(move || {
                    let out = std::process::Command::new("timeout")
                        .args(["30", "/bin/sh", "-c", &cmd])
                        .envs(envs)
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
                self.notify("Files", "Done", &msg, Priority::Low, Some(4_000));
            }
            Err(e) => {
                // Keep the dialog up with the real reason, and force the table
                // to be read from disk again: a failed delete/move may still
                // have changed something, and a table that keeps showing the
                // old rows while claiming nothing happened is a lie either way.
                self.files.dialog = Some(crate::apps::DialogKind::Error(e.to_string()));
            }
        }
        // Both outcomes re-read the directory. Success because it changed;
        // failure because the interface must show what is actually there.
        self.files.listed = None;
        self.request_full_redraw();
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
                self.notify("Files", "Done", &what, Priority::Low, Some(3_000));
            }
            Err(e) => {
                // The dialog stays open with the cause rather than closing as
                // though the name had been accepted.
                self.files.dialog = Some(DialogKind::Error(e.to_string()));
            }
        }
        self.files.listed = None;
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
                    self.set_desktop_dialog(Some(crate::apps::DesktopDialog::NewFolder));
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
                    self.files.ask_confirm(
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
                let _ = self.browser.navigate_active();
                self.browser_frame = None;
                self.browser.last_frame = None;
                if self.browser.active_tab_has_page() {
                    if let Some(tab) = self.browser.active_engine_tab() {
                        self.browser.status = "reloading".to_owned();
                        self.send_line(&format!("RELOAD {tab}"));
                    }
                }
            }
            MA::BrowserCloseTab => {
                let i = self.browser.active;
                if let Some(tab) = self.browser.tabs.get(i).map(|t| t.engine_tab) {
                    self.send_line(&format!("CLOSE {tab}"));
                }
                self.browser_frame = None;
                self.browser.last_frame = None;
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
            // The row says "Next wallpaper"; it used to cycle the theme, which
            // made the menu item a lie about the thing it changed.
            MA::NextWallpaper => {
                return self.cycle_wallpaper();
            }
            // The engine is a setting, so it is saved as one: the old code
            // assigned the field and returned no effect, which made the choice
            // last exactly as long as the window it was made in.
            MA::EngineGoogle => {
                return self.set_search_engine(crate::apps::SearchEngine::Google);
            }
            MA::EngineDuckDuckGo => {
                return self.set_search_engine(crate::apps::SearchEngine::DuckDuckGo);
            }
            MA::EngineBing => {
                return self.set_search_engine(crate::apps::SearchEngine::Bing);
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
                self.set_desktop_dialog(Some(DesktopDialog::Error(
                    "the file service is not available on this platform".to_owned(),
                )));
                return Vec::new();
            };
            match bridge.mkdir("/Documents", &name) {
                Ok(()) => {
                    self.set_desktop_dialog(None);
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
                    self.set_desktop_dialog(Some(DesktopDialog::Error(e.to_string())));
                }
            }
            self.request_full_redraw();
            return Vec::new();
        }
        let confirmed = match self.desktop_dialog {
            Some(DesktopDialog::PowerConfirm { command }) => Some(command),
            _ => None,
        };
        self.set_desktop_dialog(None);
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
    /// The last non-launcher overlay that has stopped receiving input but is
    /// still being drawn through its exit spring. Keeping visual lifetime
    /// separate from logical lifetime prevents a closing panel from stealing
    /// clicks while also avoiding the old one-frame disappearance.
    retiring_overlay: Option<Overlay>,
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
    /// A context-menu snapshot retained only for its short visual exit.
    retiring_context_menu: Option<crate::apps::ContextMenu>,
    /// The desktop's icon grid.
    pub desktop_icons: crate::apps::DesktopIcons,
    /// An open dialog on the desktop itself (new folder on the wallpaper).
    pub desktop_dialog: Option<crate::apps::DesktopDialog>,
    /// A dialog that is logically gone but still completing its exit. It has
    /// no hotspots, so confirmation and cancellation can never be repeated by
    /// clicking a fading image of the old dialog.
    retiring_desktop_dialog: Option<crate::apps::DesktopDialog>,
    /// The desktop dialog's text field.
    pub desktop_dialog_field: String,
    /// The desktop dialog field's selection.
    pub desktop_dialog_selection: Option<(usize, usize)>,
    /// The browser engine session, when one has been started.
    pub browser_session: Option<crate::link::Link>,
    /// Foreign applications: the Wayland server, their surfaces and their
    /// windows. `None` when no runtime directory was available, which is a
    /// desktop that hosts only its own applications — reported, never silent.
    pub clients: Option<crate::clients::Host>,
    /// The last frame the engine produced: RGBA8 pixels plus dimensions.
    /// One frame only — a newer frame replaces an undrawn older one, which is
    /// what bounds the memory a fast page can make the compositor hold.
    pub browser_frame: Option<Arc<[u8]>>,
    /// The screen rectangle the last engine frame was drawn into. Pointer
    /// positions are mapped through it before Chromium sees a coordinate, so a
    /// click lands on the pixel the user aimed at rather than on the window
    /// edge.
    pub browser_engine_area: Option<Rect>,
    /// The viewport the broker was last asked to render at, in engine pixels.
    browser_viewport: (u32, u32),
    /// The last pointer position already sent, so a 1 kHz mouse does not turn
    /// into a 1 kHz CDP stream.
    browser_pointer_sent: Option<(f64, f64)>,
    /// When a fresh frame was last requested, in shell milliseconds.
    browser_refresh_ms: u64,
    /// The socket the browser broker is reached through. A field rather than a
    /// constant so a test (or a hand-run broker) can point the shell at its own
    /// socket instead of the system one.
    pub browser_socket: std::path::PathBuf,
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
    /// The pointer's last move, when that move changed nothing but the pointer
    /// itself: no hover change, no drag, no title-bar zone crossed. The
    /// compositor can then animate the arrow without composing the desktop.
    cursor_step: Option<(Point, Point)>,
    hovered: Option<Hot>,
    /// Current target in the shell-wide Tab/Shift+Tab focus order.
    keyboard_focus: Option<Hot>,
    pressed: Option<Hot>,
    meta_tap: MetaTap,
    last_input_ms: u64,
    pub(crate) now_ms: u64,
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
    /// The shell's overlay springs, stepped once per frame in `tick`.
    anims: crate::anim::ShellAnims,
    /// Notification copies retained while their banner spring exits. The live
    /// notification queue remains authoritative; this is drawing data only and
    /// is bounded by the queue's capacity plus at most one short exit per item.
    retiring_banners: Vec<Notification>,
    /// Whether the frame being drawn belongs to a surface that is leaving.
    /// Its controls are pixels, not targets, so they must not take a click
    /// from whatever replaced them.
    drawing_retired: bool,
    /// Hotspots registered by a leaving surface. Thrown away when it is done,
    /// which is what keeps an exit from leaving invisible buttons behind.
    retired_hot: Hotspots<Hot>,
    /// Whether Alt is still held while the switcher is open.
    switcher_holding: bool,
    /// The focused window when damage was last taken. A focus change repaints
    /// both windows and the top bar's title.
    drawn_focus: Option<WindowId>,
    /// The dock's fully open rectangle, as the draw side last laid it out.
    /// Damaged instead of the whole screen while the dock animates.
    dock_settled: Option<Rect>,
    /// The clock text as of the last time the shell's time moved. The top bar
    /// is damaged when it changes and not otherwise.
    drawn_clock: String,
    /// When the boot intro started, or `None` when no intro is playing.
    boot_started_ms: Option<u64>,
}

/// How long the boot intro runs, in ms. The pen write takes most of it; the
/// rest is the short hold before the desktop takes over.
pub const BOOT_INTRO_MS: u64 = 2_400;

/// The part of the intro during which the pen writes the mark. The remainder
/// holds the finished mark before the desktop appears.
pub const BOOT_WRITE_MS: u64 = 1_900;

/// The theme a preference record resolves to: the chosen theme with the chosen
/// accent applied.
///
/// One function rather than three call sites doing it themselves, so loading,
/// applying and rolling back a preference cannot disagree about what the
/// accent means.
fn resolved_theme(preferences: crate::settings::Preferences, hour: u8) -> litric_design::Theme {
    let kind = preferences
        .theme_schedule
        .effective(preferences.theme, hour);
    litric_design::Theme::of(kind).with_accent(preferences.accent)
}

impl Shell {
    /// Replace startup defaults with a validated preference record.
    pub fn load_preferences(&mut self, preferences: crate::settings::Preferences) {
        self.settings = crate::settings::Model::new(preferences);
        self.tokens.theme = resolved_theme(preferences, self.local_hour());
        self.quick.reduce_motion = preferences.reduce_motion;
        self.browser.engine = preferences.search_engine;
        self.request_full_redraw();
    }

    /// The local hour, which is what an appearance schedule is written against.
    #[must_use]
    pub fn local_hour(&self) -> u8 {
        self.regional.local(self.clock_seconds()).hour
    }

    /// Follow the clock into or out of the dark, when the user asked for it.
    ///
    /// Called from [`Self::tick`], the only place that knows time moved.
    /// Nothing is persisted: the schedule is a rule and the hour is read again
    /// on the next tick, so a power cut at 18:59 still comes back light and one
    /// at 19:01 comes back dark without writing to disk.
    fn follow_schedule(&mut self) {
        if self.settings.preferences.theme_schedule != crate::settings::ThemeSchedule::Auto {
            return;
        }
        let theme = resolved_theme(self.settings.preferences, self.local_hour());
        if theme.kind != self.tokens.theme.kind {
            self.tokens.theme = theme;
            // Wallpaper, glass alphas and every text role resolve from the
            // theme, so this is a whole-screen change rather than a panel's.
            self.request_full_redraw();
        }
    }

    /// Begin a shell-owned preference update and return its persistence effect.
    fn save_preferences(&mut self, preferences: crate::settings::Preferences) -> Vec<Effect> {
        let Some(next) = self.settings.begin(preferences) else {
            return Vec::new();
        };
        self.tokens.theme = resolved_theme(next, self.local_hour());
        self.quick.reduce_motion = next.reduce_motion;
        self.browser.engine = next.search_engine;
        self.request_full_redraw();
        vec![Effect::PersistPreferences(next)]
    }

    /// Complete a preference write, rolling back the live theme and motion if
    /// persistence failed.
    pub fn finish_preferences_save(&mut self, result: Result<(), String>) {
        self.settings.finish(result);
        let preferences = self.settings.preferences;
        self.tokens.theme = resolved_theme(preferences, self.local_hour());
        self.quick.reduce_motion = preferences.reduce_motion;
        self.request_full_redraw();
    }

    fn toggle_language(&mut self) -> Vec<Effect> {
        let mut next = self.settings.preferences;
        next.language = match next.language {
            crate::i18n::Language::Thai => crate::i18n::Language::English,
            crate::i18n::Language::English => crate::i18n::Language::Thai,
        };
        self.save_preferences(next)
    }

    fn cycle_theme(&mut self) -> Vec<Effect> {
        let themes = litric_design::ThemeKind::all();
        let current = self.settings.preferences.theme;
        let next_theme = themes
            .iter()
            .position(|kind| *kind == current)
            .and_then(|index| themes.get(index.saturating_add(1)).copied())
            .or_else(|| themes.first().copied());
        let Some(theme) = next_theme else {
            return Vec::new();
        };
        let mut next = self.settings.preferences;
        next.theme = theme;
        self.save_preferences(next)
    }

    /// Switch the appearance schedule on or off.
    ///
    /// Turning it on does not change what is on screen immediately: the theme
    /// is re-resolved from the clock by the next tick, through the same
    /// `resolved_theme` every other path uses. That is deliberate — a toggle
    /// that jumped the screen into dark the moment it was pressed would look
    /// like a theme change rather than a schedule, and the user would not know
    /// which of the two they had just done.
    fn cycle_theme_schedule(&mut self) -> Vec<Effect> {
        use crate::settings::ThemeSchedule;
        let schedules = ThemeSchedule::all();
        let next = schedules
            .iter()
            .position(|schedule| *schedule == self.settings.preferences.theme_schedule)
            .and_then(|index| schedules.get(index.saturating_add(1)).copied())
            .or_else(|| schedules.first().copied());
        let Some(schedule) = next else {
            return Vec::new();
        };
        let mut preferences = self.settings.preferences;
        preferences.theme_schedule = schedule;
        let effects = self.save_preferences(preferences);
        self.follow_schedule();
        effects
    }

    /// Step to the next accent choice. The cycle starts at the theme's own
    /// accent, so the shipped look is reachable again without a reset.
    ///
    /// High-contrast themes keep their accent while the choice is live; what
    /// makes that true is `resolved_theme`, so the step is still recorded and
    /// takes effect the moment the user returns to a standard theme.
    fn cycle_accent(&mut self) -> Vec<Effect> {
        let accents = litric_design::Accent::all();
        let current = self.settings.preferences.accent;
        let next_accent = accents
            .iter()
            .position(|accent| *accent == current)
            .and_then(|index| accents.get(index.saturating_add(1)).copied())
            .or_else(|| accents.first().copied());
        let Some(accent) = next_accent else {
            return Vec::new();
        };
        let mut next = self.settings.preferences;
        next.accent = accent;
        self.save_preferences(next)
    }

    /// Choose the engine the omnibox searches with, and persist it.
    ///
    /// Returns the persistence effect rather than assigning the field directly:
    /// a preference the window keeps and the disk does not is the failure this
    /// whole settings path exists to avoid.
    fn set_search_engine(&mut self, engine: crate::apps::SearchEngine) -> Vec<Effect> {
        if self.settings.preferences.search_engine == engine {
            return Vec::new();
        }
        let mut next = self.settings.preferences;
        next.search_engine = engine;
        self.save_preferences(next)
    }

    /// Step to the next wallpaper. Like the accent, the first entry is the
    /// shipped look, so the default is reachable from the cycle.
    fn cycle_wallpaper(&mut self) -> Vec<Effect> {
        let wallpapers = crate::desktop::Wallpaper::all();
        let current = self.settings.preferences.wallpaper;
        let next_wallpaper = wallpapers
            .iter()
            .position(|wallpaper| *wallpaper == current)
            .and_then(|index| wallpapers.get(index.saturating_add(1)).copied())
            .or_else(|| wallpapers.first().copied());
        let Some(wallpaper) = next_wallpaper else {
            return Vec::new();
        };
        let mut next = self.settings.preferences;
        next.wallpaper = wallpaper;
        self.save_preferences(next)
    }

    fn toggle_reduced_motion(&mut self) -> Vec<Effect> {
        let mut next = self.settings.preferences;
        next.reduce_motion = !next.reduce_motion;
        self.save_preferences(next)
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
            retiring_overlay: None,
            notifications: Notifications::default(),
            quick: QuickSettings::default(),
            environments: Vec::new(),
            settings: crate::settings::Model::new(crate::settings::Preferences::new(
                theme, language,
            )),
            facts: SystemFacts::default(),
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
            retiring_context_menu: None,
            desktop_icons: crate::apps::DesktopIcons::minimal(),
            desktop_dialog: None,
            retiring_desktop_dialog: None,
            desktop_dialog_field: String::new(),
            desktop_dialog_selection: None,
            browser_session: None,
            clients: None,
            browser_frame: None,
            browser_engine_area: None,
            browser_viewport: (1280, 800),
            browser_pointer_sent: None,
            browser_refresh_ms: 0,
            browser_socket: std::env::var_os("LITRIC_BROWSER_SOCK").map_or_else(
                || std::path::PathBuf::from(BROWSER_SOCKET),
                std::path::PathBuf::from,
            ),
            diagnostics: false,
            idle_policy: IdlePolicy::default(),
            lock: crate::lock::LockState::default(),
            lock_cache: crate::lock::Cache::default(),
            hot: Hotspots::default(),
            pointer: Point::new(width / 2.0, height / 2.0),
            cursor_step: None,
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
            anims: crate::anim::ShellAnims::new(),
            retiring_banners: Vec::new(),
            drawing_retired: false,
            retired_hot: Hotspots::default(),
            switcher_holding: false,
            drawn_focus: None,
            dock_settled: None,
            boot_started_ms: None,
            drawn_clock,
        }
    }

    /// Publish the rectangle the dock occupies when its panel is fully open.
    ///
    /// Only the draw side knows how tall the panel is for the current contents
    /// (a search that matches two rows is a short panel), so it says so here.
    /// The morph grows bottom-up at a fixed width, so every frame of it lies
    /// inside this rectangle -- which is why an animation frame damages the
    /// panel rather than the screen.
    pub fn set_dock_settled(&mut self, rect: Rect) {
        self.dock_settled = Some(rect);
    }

    /// The rectangle of the fully open dock, if it has ever been drawn.
    #[must_use]
    pub const fn dock_settled(&self) -> Option<Rect> {
        self.dock_settled
    }

    /// The top bar's height in logical units. There is no bottom bar: one
    /// system bar carries everything, which is the whole layout decision.
    ///
    /// The dock is not a system bar -- it holds applications, not the machine's
    /// state -- so it is not measured here.
    pub const TOP_BAR_LOGICAL: f32 = 24.0;

    /// The top bar's rectangle.
    #[must_use]
    pub fn top_bar(&self) -> Rect {
        let s = self.wm.screen();
        Rect::new(0.0, 0.0, s.w, self.tokens.px(Self::TOP_BAR_LOGICAL))
    }

    /// The windows Mission Control shows, in card order: by workspace, then by
    /// the order they were opened.
    ///
    /// Stable ordering is the point. Cards that reordered themselves as the
    /// pointer moved would move the target of a click that was already on its
    /// way to one of them.
    #[must_use]
    pub fn overview_ids(&self) -> Vec<WindowId> {
        let mut ids: Vec<(usize, WindowId)> =
            self.wm.all().iter().map(|w| (w.workspace, w.id)).collect();
        ids.sort_unstable();
        ids.into_iter().map(|(_, id)| id).collect()
    }

    /// Bring the window a Mission Control card stands for to the user: its
    /// workspace, out of a minimize if it was in one, and the keyboard.
    pub fn overview_open(&mut self, index: usize) {
        let ids = self.overview_ids();
        if let Some(id) = ids.get(index).copied() {
            self.wm.reveal(id);
        }
        self.set_overlay(Overlay::None);
        self.overlay_damage();
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
    ///
    /// A surface that is leaving never highlights: it is a picture of a
    /// control, not one, and a hover that followed the pointer into an exit
    /// would suggest a click that cannot land.
    #[must_use]
    pub fn is_highlighted(&self, hot: Hot) -> bool {
        !self.drawing_retired
            && (matches!(self.hovered, Some(h) if h == hot)
                || matches!(self.keyboard_focus, Some(h) if h == hot))
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
            Some(Hot::TerminalInput | Hot::BrowserOmnibox | Hot::TrashFilter | Hot::ActivityFilter)
                | Some(Hot::Files(crate::apps::FilesHot::Filter))
                | Some(Hot::LauncherSearch)
        )
    }

    /// What the pointer is pressed on.
    #[must_use]
    pub const fn pressed(&self) -> Option<Hot> {
        self.pressed
    }

    /// The hotspot table, for the renderer to fill in during layout.
    ///
    /// While a leaving surface is being drawn this is a scratch table, so its
    /// controls are drawn but cannot be clicked: the input path only ever
    /// consults `hot`, and the scratch table is cleared when the draw ends.
    #[must_use]
    pub fn hotspots_mut(&mut self) -> &mut Hotspots<Hot> {
        if self.drawing_retired {
            &mut self.retired_hot
        } else {
            &mut self.hot
        }
    }

    /// Draw a surface that has already left, for the pixels and nothing else.
    ///
    /// The closure runs with [`Self::hotspots_mut`] diverted to a scratch
    /// table and [`Self::is_highlighted`] answered false, so a menu or dialog
    /// on its way out cannot answer a click or repeat an action.
    pub fn draw_retired<R>(&mut self, draw: impl FnOnce(&mut Self) -> R) -> R {
        self.retired_hot.clear();
        self.drawing_retired = true;
        let out = draw(self);
        self.drawing_retired = false;
        self.retired_hot.clear();
        out
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

    /// An overlay that is visually leaving but no longer owns input.
    #[must_use]
    pub const fn retiring_overlay(&self) -> Option<&Overlay> {
        self.retiring_overlay.as_ref()
    }

    /// A context menu that is visually leaving but cannot be activated.
    #[must_use]
    pub const fn retiring_context_menu(&self) -> Option<&crate::apps::ContextMenu> {
        self.retiring_context_menu.as_ref()
    }

    /// A desktop dialog that is visually leaving but cannot be confirmed
    /// again. This is intentionally separate from `desktop_dialog`, whose
    /// presence is the source of truth for input and system actions.
    #[must_use]
    pub const fn retiring_desktop_dialog(&self) -> Option<&crate::apps::DesktopDialog> {
        self.retiring_desktop_dialog.as_ref()
    }

    /// Notification banners that are finishing their visual exit.
    #[must_use]
    pub fn retiring_banners(&self) -> &[Notification] {
        &self.retiring_banners
    }

    /// Replace the logical overlay while allowing its visual predecessor to
    /// complete a short, non-interactive exit.
    ///
    /// A surface is only kept as an exit snapshot when the spring its reveal
    /// reads is going to fall. The launcher owns the dock, whose morph already
    /// closes in place, and the panels that share one reveal spring (quick
    /// settings, power and the LitricOS menu) replace each other with no
    /// spring left to fall. Keeping a snapshot in either case would draw two
    /// panels over each other until something else moved, so it is not kept.
    fn set_overlay(&mut self, next: Overlay) {
        if self.overlay == next {
            return;
        }
        if self
            .retiring_overlay
            .as_ref()
            .is_some_and(|old| Self::same_overlay_family(old, &next))
        {
            // Reopening the same surface, or swapping between panels that
            // share a spring, reverses that spring instead of drawing two
            // copies of a panel over each other.
            self.retiring_overlay = None;
        }
        let old = std::mem::replace(&mut self.overlay, next);
        if old.is_open()
            && !matches!(old, Overlay::Launcher { .. })
            && !Self::same_overlay_family(&old, &self.overlay)
        {
            self.retiring_overlay = Some(old);
        }
        self.overlay_damage();
    }

    /// Whether two overlay values draw the same kind of surface, or read the
    /// same reveal spring. Their query or keyboard-selection payload does not
    /// alter that visual ownership.
    fn same_overlay_family(left: &Overlay, right: &Overlay) -> bool {
        std::mem::discriminant(left) == std::mem::discriminant(right)
            || matches!(
                (left, right),
                (
                    Overlay::QuickSettings | Overlay::Power | Overlay::Litro,
                    Overlay::QuickSettings | Overlay::Power | Overlay::Litro
                )
            )
    }

    /// The spring that decides how much of `overlay` is on screen. It is the
    /// same value the draw side reads, so a snapshot can be dropped exactly
    /// when its last pixel is.
    fn overlay_reveal(&self, overlay: &Overlay) -> f32 {
        match overlay {
            Overlay::Launcher { .. } => self.anims.launcher_morph.value(),
            Overlay::QuickSettings | Overlay::Power | Overlay::Litro => {
                self.anims.quick_settings.value()
            }
            Overlay::Notifications => self.anims.notifications.value(),
            Overlay::Switcher { .. } => self.anims.switcher.value(),
            Overlay::Overview { .. } => self.anims.overview.value(),
            Overlay::None => 0.0,
        }
    }

    /// Replace a context menu and preserve only the outgoing pixels needed for
    /// its exit. A newly opened menu cancels any old exit so stale menu items
    /// cannot briefly appear beside the new pointer location.
    fn set_context_menu(&mut self, next: Option<crate::apps::ContextMenu>) {
        if self.context_menu == next {
            return;
        }
        if next.is_some() {
            self.retiring_context_menu = None;
        }
        let old = std::mem::replace(&mut self.context_menu, next);
        if let Some(menu) = old {
            self.retiring_context_menu = Some(menu);
        }
        self.request_full_redraw();
    }

    /// Replace or dismiss a desktop dialog. The live option controls input;
    /// its former value is only a display snapshot until the dialog spring
    /// reaches rest at zero.
    fn set_desktop_dialog(&mut self, next: Option<crate::apps::DesktopDialog>) {
        if self.desktop_dialog == next {
            return;
        }
        if next.is_some() {
            self.retiring_desktop_dialog = None;
        }
        let old = std::mem::replace(&mut self.desktop_dialog, next);
        if let Some(dialog) = old {
            self.retiring_desktop_dialog = Some(dialog);
        }
        self.request_full_redraw();
    }

    /// Retain an outgoing banner only when it has actually been made visible.
    /// This keeps Do Not Disturb-suppressed notifications from allocating a
    /// redundant visual snapshot.
    fn retire_banner(&mut self, notification: Notification) {
        let id = notification.id;
        if self.anims.banner_reveal(id) <= 0.0 {
            return;
        }
        if !self.retiring_banners.iter().any(|old| old.id == id) {
            self.retiring_banners.push(notification);
        }
        // Retire the id that was actually dismissed. This used to read the
        // id of the last snapshot pushed, which is a different banner as soon
        // as one is already retiring: dismissing a second notification left
        // the first one's exit running and this one never leaving.
        self.anims.retire_banner(id);
    }

    /// Dismiss one notification while preserving a banner exit when needed.
    fn dismiss_notification(&mut self, id: u32) -> bool {
        let snapshot = self
            .notifications
            .items()
            .into_iter()
            .find(|notification| notification.id == id)
            .cloned();
        let removed = self.notifications.dismiss(id);
        if removed {
            if let Some(notification) = snapshot {
                self.retire_banner(notification);
            }
            self.request_full_redraw();
        }
        removed
    }

    /// Clear every bulk-dismissible notification while giving visible banners
    /// the same exit path as individually dismissed items.
    fn clear_notifications(&mut self) -> usize {
        let snapshots: Vec<Notification> = self
            .notifications
            .items()
            .into_iter()
            .filter(|notification| notification.priority != Priority::Critical)
            .cloned()
            .collect();
        let cleared = self.notifications.clear();
        if cleared > 0 {
            for notification in snapshots {
                self.retire_banner(notification);
            }
            self.request_full_redraw();
        }
        cleared
    }

    /// Count a frame.
    pub fn counted_frame(&mut self) {
        self.frames = self.frames.saturating_add(1);
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
        if self.boot_intro_progress(now_ms).is_some() {
            return LifeState::Active.frame_interval_ms();
        }
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
            && self
                .facts
                .auth_account
                .as_deref()
                .is_some_and(|account| !account.is_empty() && account != "-")
            && self.facts.auth_password.as_deref() == Some("set")
    }

    /// Why locking is refused, in a sentence that names the next action.
    ///
    /// The daemon reports a reason token rather than a sentence because the
    /// same refusal has to be usable in the facts protocol and in the
    /// interface: "the account has no password" and "the helper is missing"
    /// need different actions, and a single "locking is unavailable" hides
    /// which one the person is looking at.
    #[must_use]
    pub fn lock_refusal(&self) -> String {
        let Some(verifier) = self.verifier() else {
            return "The system daemon did not answer, so this session cannot check a \
                    password and will not pretend to lock the screen."
                .to_owned();
        };
        let account = self
            .facts
            .auth_account
            .as_deref()
            .filter(|a| !a.is_empty() && *a != "-");
        match self.facts.auth_reason.as_deref() {
            Some("helper-missing") => "No password checker is installed in this image, so \
                 the screen could not be unlocked. Locking is refused rather than shown \
                 working."
                .to_owned(),
            Some("account-missing") => "This image has no desktop account to check a \
                 password for. Create the desktop account and set its password before \
                 using the lock."
                .to_owned(),
            Some("password-locked") => format!(
                "The desktop account {} has no usable password (it is locked), so this \
                 screen could not be unlocked. Set a password with `passwd {}` on a \
                 console, then start the session again.",
                account.unwrap_or("for this session"),
                account.unwrap_or("<account>"),
            ),
            Some("password-unset") => format!(
                "The desktop account {} has no password set, so this screen could not be \
                 unlocked. Set one with `passwd {}` on a console, then start the session \
                 again.",
                account.unwrap_or("for this session"),
                account.unwrap_or("<account>"),
            ),
            Some("password-unknown") => format!(
                "The password state of the desktop account {} could not be read, so \
                 locking is refused rather than risk a screen that cannot unlock. Check \
                 it with `passwd -S {}` on a console before trying again.",
                account.unwrap_or("for this session"),
                account.unwrap_or("<account>"),
            ),
            Some(_) => "The system daemon refuses locking on this machine, so the screen \
                 was not locked."
                .to_owned(),
            None if self.facts.auth_lock == Some(true) => "The system daemon refuses \
                 locking on this machine, so the screen was not locked."
                .to_owned(),
            None if self.facts.auth_password.as_deref() != Some("set") => "The desktop \
                 account's password could not be confirmed, so locking is refused rather \
                 than risk a screen that cannot unlock."
                .to_owned(),
            None => verifier.refusal().to_owned(),
        }
    }

    /// Lock the session. Returns whether the lock was taken.
    ///
    /// While locked the shell draws one surface and refuses input to everything
    /// else; the content that was on the screen is gone from the frame buffer on
    /// the next frame, because the lock surface starts by clearing it. The
    /// account the password is checked against is the one the daemon names as
    /// the desktop account, not whichever logind session is listed first: the
    /// diagnostic serial console autologins as root on this image.
    pub fn lock_now(&mut self) -> bool {
        if !self.can_lock() {
            return false;
        }
        self.lock.locked = true;
        self.lock.verdict = crate::lock::Verdict::Idle;
        // `can_lock` has already checked that this is present and not `-`.
        self.lock.user = self.facts.auth_account.clone().unwrap_or_default();
        // The password field is immediately visible. Requiring an extra
        // Space/Enter press hid the only useful lock action behind a hint.
        self.lock.wake();
        self.lock.flow_ms = 0;
        self.pointer = Point::new(self.wm.screen().w / 2.0, self.wm.screen().h / 2.0);
        // Every overlay is a surface the lock does not draw, so an open menu
        // must not still be on the shell's mind when the session comes back.
        self.set_overlay(Overlay::None);
        self.set_context_menu(None);
        // The lock replaces the frame wholesale rather than fading it: a
        // departing panel drawn over a locked session would be exactly the
        // leak the line above is about, one frame later.
        self.retiring_overlay = None;
        self.retiring_context_menu = None;
        self.retiring_desktop_dialog = None;
        self.retiring_banners.clear();
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
    /// Input for the first-run setup screen: the lock screen's hotspots and
    /// pointer handling, plus Tab and Enter so every choice works from the
    /// keyboard. The desktop does not run until setup is recorded as done.
    fn setup_input(&mut self, ev: &InputEvent) -> Vec<Effect> {
        match ev {
            InputEvent::KeyDown {
                key, mods, repeat, ..
            } => {
                if *repeat {
                    return Vec::new();
                }
                match key {
                    Key::Tab => {
                        self.move_keyboard_focus(mods.shift);
                        Vec::new()
                    }
                    Key::Enter => match self.keyboard_focus {
                        Some(hot) => self.activate(hot),
                        None => Vec::new(),
                    },
                    _ => Vec::new(),
                }
            }
            _ => self.locked_input(ev),
        }
    }

    /// Record first-run setup as finished. A failed write rolls back in
    /// `settings`, so the setup screen stays up and the choice is not lost.
    fn finish_first_run(&mut self) -> Vec<Effect> {
        let mut next = self.settings.preferences;
        next.onboarded = true;
        self.save_preferences(next)
    }

    /// The lock surface owns the keyboard and the pointer: nothing else in the
    /// shell sees an event while this returns `true`, which is what "locked"
    /// means on a framebuffer desktop with no client surfaces to hide.
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
    ///
    /// The gate is the same one that raised the lock: a verifier and a daemon
    /// that did not refuse. If the facts have gone away since — the daemon
    /// missed a poll — there is nothing on the screen this session could use to
    /// check a password, and it says so instead of collecting one it could not
    /// use. It never unlocks on its own; the screen stays up and the attempt is
    /// simply not made.
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

    /// Begin the boot intro at `now_ms`. The desktop is drawn underneath it.
    pub fn start_boot_intro(&mut self, now_ms: u64) {
        self.boot_started_ms = Some(now_ms);
        self.request_full_redraw();
    }

    /// Start the boot intro at the shell's own clock.
    ///
    /// `main` passes wall time because it has it; a renderer that has been
    /// ticking a shell for its own reasons (the frame dump) does not, and
    /// guessing produced an intro that had already finished before its first
    /// frame.
    pub fn restart_boot_intro(&mut self) {
        self.start_boot_intro(self.now_ms);
    }

    /// The boot intro's progress from 0 to 1 at `now_ms`, or `None` once it
    /// has finished or was never started.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn boot_intro_progress(&self, now_ms: u64) -> Option<f32> {
        let started = self.boot_started_ms?;
        let elapsed = now_ms.saturating_sub(started);
        if elapsed >= BOOT_INTRO_MS {
            return None;
        }
        // Reduced motion: no pen write, so the finished mark is shown for the
        // same hold and then the desktop takes over, as it does otherwise.
        if self.quick.reduce_motion {
            return Some(1.0);
        }
        Some((elapsed as f32 / BOOT_WRITE_MS as f32).min(1.0))
    }

    /// What the boot intro says under the mark at this moment.
    ///
    /// A stage, not a spinner: "Getting things ready" is the first thing the
    /// machine says, and it keeps saying something true as the session comes
    /// up. The words are stages of the same length as the intro, so the note
    /// and the pen finish together rather than one running out before the
    /// other.
    #[must_use]
    pub fn boot_intro_note(&self) -> &'static str {
        let Some(started) = self.boot_started_ms else {
            return "";
        };
        let elapsed = self.now_ms.saturating_sub(started);
        if elapsed < 800 {
            "Getting things ready"
        } else if elapsed < 1_700 {
            "Starting your session"
        } else {
            "Almost there"
        }
    }

    /// [`Self::boot_intro_progress`] at the shell's own clock, for drawing.
    #[must_use]
    pub fn boot_intro_now(&self) -> Option<f32> {
        self.boot_intro_progress(self.now_ms)
    }

    /// Move time forward without any input, for the idle timers and the clock.
    pub fn tick(&mut self, now_ms: u64) {
        self.set_now(now_ms);
        // An appearance schedule is time passing, so it belongs here rather
        // than in a timer of its own: the shell already wakes for the clock.
        self.follow_schedule();
        // The intro repaints every tick while it plays, and once more when it
        // ends so the desktop comes back without waiting for input.
        if self.boot_started_ms.is_some() {
            if self.boot_intro_progress(now_ms).is_none() {
                self.boot_started_ms = None;
            }
            self.request_full_redraw();
        }
        // A still lock has no reason to repaint until its clock changes. When
        // the user enables motion, redraw at the animation's bounded 30 Hz.
        if self.lock.locked && self.lock.advance(now_ms) {
            self.request_full_redraw();
        }
        if self.terminal_pump() {
            self.request_full_redraw();
        }
        // Client surfaces: mapping, titles, frames, a dead application and a
        // window the user closed all arrive here, not through input.
        self.pump_clients();
        // The overlay springs follow the overlay state and advance one step.
        // sync() is idempotent, so doing it here means no call site that
        // opens or closes an overlay has to remember the animation exists.
        // The power and LitricOS menus are popovers of the same kind as quick
        // settings, so they share its reveal spring: two panels that can never
        // be open at once do not need two identical springs, and the one they
        // share already has the grammar every other top-bar panel uses.
        let panel_open = matches!(
            self.overlay,
            Overlay::QuickSettings | Overlay::Power | Overlay::Litro
        );
        let active: Vec<u32> = self.notifications.items().iter().map(|n| n.id).collect();
        let retiring: Vec<u32> = self.retiring_banners.iter().map(|n| n.id).collect();
        self.anims.sync(
            matches!(self.overlay, Overlay::Launcher { .. }),
            panel_open,
            matches!(self.overlay, Overlay::Notifications),
            matches!(self.overlay, Overlay::Switcher { .. }),
            matches!(self.overlay, Overlay::Overview { .. }),
            self.context_menu.is_some(),
            self.desktop_dialog.is_some(),
            self.quick.reduce_motion,
            &active,
            &retiring,
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
        // A surface that has finished leaving stops being pixels. The springs
        // are the clock, so a snapshot lives exactly as long as its reveal is
        // non-zero; holding one longer would keep a drawn panel on screen after
        // its animation had ended, and dropping it earlier would cut the exit.
        if self
            .retiring_overlay
            .as_ref()
            .is_some_and(|old| self.overlay_reveal(old) <= 0.0)
        {
            self.retiring_overlay = None;
        }
        if self.anims.context_menu.settled() && self.context_menu.is_none() {
            self.retiring_context_menu = None;
        }
        if self.anims.dialog.settled() && self.desktop_dialog.is_none() {
            self.retiring_desktop_dialog = None;
        }
        let live_banners: Vec<u32> = self.notifications.items().iter().map(|n| n.id).collect();
        let keep: Vec<u32> = self
            .retiring_banners
            .iter()
            .map(|banner| banner.id)
            .filter(|id| !live_banners.contains(id) && self.anims.banner_reveal(*id) > 0.0)
            .collect();
        self.retiring_banners
            .retain(|banner| keep.contains(&banner.id));
        if moved.dock {
            // A spring that moved is damage (BUG-0019). `step` reports whether
            // the value changed, not whether it is still going, so a spring
            // that reaches its settle budget and snaps the rest of the way in
            // a stalled frame still has pixels that must be drawn.
            //
            // The dock's pixels are bounded: it grows out of a strip at the
            // bottom centre into a panel the same width, so every frame of the
            // morph lies inside the settled panel. Damaging that rectangle
            // instead of the screen is what takes the launcher from a handful
            // of composed frames a second to a smooth one: a full frame is
            // millions of pixels of wallpaper and glass, and the panel is a
            // fraction of it. Falls back to a full redraw before the dock has
            // ever been drawn, when there is no rectangle to trust.
            match self.dock_settled {
                Some(rect) => self.damage.push(rect.inflate(self.tokens.px(56.0))),
                None => self.request_full_redraw(),
            }
        }
        if moved.other {
            self.request_full_redraw();
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
    fn set_now(&mut self, now_ms: u64) {
        self.now_ms = now_ms;
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
        // The launcher is the dock, and the dock is the one animated surface
        // whose extent the shell knows: the draw side publishes the rectangle
        // it occupies fully open, and every frame of the morph lies inside it.
        // So opening or closing the launcher damages that rectangle -- which is
        // most of what made the animation smooth. Any other overlay still takes
        // the whole screen, because its extent is not tracked and a repaint
        // that misses a pixel is worse than a slow one.
        let dock_only = matches!(self.overlay, Overlay::Launcher { .. })
            || (matches!(self.overlay, Overlay::None) && self.retiring_overlay.is_none());
        if dock_only {
            if let Some(rect) = self.dock_settled {
                self.damage.push(rect.inflate(self.tokens.px(56.0)));
                return;
            }
        }
        // Overlays are large and can move; repainting the whole screen when one
        // opens or closes is cheaper than tracking their exact extent and
        // getting it subtly wrong.
        self.request_full_redraw();
    }

    // ---- input ------------------------------------------------------------

    /// Handle one input event. Returns anything `main` must act on.
    pub fn handle(&mut self, ev: &InputEvent, now_ms: u64) -> Vec<Effect> {
        self.set_now(now_ms);
        // A move is the only thing that can be answered by moving the arrow
        // alone; anything else drops that shortcut.
        if !matches!(ev, InputEvent::PointerMove { .. }) {
            self.cursor_step = None;
        }
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
        if !self.settings.preferences.onboarded {
            return self.setup_input(ev);
        }

        match ev {
            // The page owns the keyboard while its window is focused: Meta
            // combinations stay with the shell, everything else is Chromium's.
            InputEvent::KeyDown {
                key,
                code,
                mods,
                repeat,
            } => {
                if self.engine_takes_key(*key, *mods) {
                    self.browser_engine_key(*key, true, *mods);
                    return Vec::new();
                }
                if self.client_takes_key(*key, *mods) {
                    // The key goes to the application and stops there: a
                    // keystroke must not both type into a window and trigger
                    // the shell's own shortcut for the same letter.
                    self.client_key(u32::from(*code), *key, true, now_ms);
                    return Vec::new();
                }
                self.key_down(*key, *mods, *repeat)
            }
            InputEvent::KeyUp { key, code, mods } => {
                if self.engine_takes_key(*key, *mods) {
                    self.browser_engine_key(*key, false, *mods);
                    return Vec::new();
                }
                if self.client_takes_key(*key, *mods) {
                    self.client_key(u32::from(*code), *key, false, now_ms);
                    return Vec::new();
                }
                self.key_up(*key, *mods)
            }
            InputEvent::TextInput { text } => {
                if self.browser_page_focused() {
                    self.browser_engine_text(text);
                } else if self.client_focused().is_none() {
                    // A client is typing from the key events above; sending
                    // the pre-composed text as well would double every letter.
                    self.text_input(text);
                }
                Vec::new()
            }
            InputEvent::PointerMove { at, .. } => {
                self.browser_engine_move(*at);
                self.pointer_move(*at);
                let over = self.client_under(*at);
                if let Some(host) = self.clients.as_mut() {
                    host.pointer(over, (at.x, at.y));
                }
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
                let over = self.client_under(*at);
                let took = self
                    .clients
                    .as_mut()
                    .is_some_and(|host| host.scroll(over, (at.x, at.y), *dy));
                if !took {
                    self.scroll(*dy, *at);
                }
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
            && !mods.meta
            && !mods.ctrl
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
            self.set_overlay(Overlay::None);
            self.overlay_damage();
        }
        Vec::new()
    }

    /// Keys the open overlay consumes. `None` means it did not want the key.
    fn overlay_key(&mut self, key: Key, mods: Modifiers) -> Option<Vec<Effect>> {
        // The launcher is handled before the mutable match: a system-wide
        // search reads the window list, which borrows the manager the match
        // would otherwise hold.
        if let Overlay::Launcher { query, selected } = &self.overlay {
            let query = query.clone();
            let selected = *selected;
            // With nothing typed the launcher is the application grid, and
            // the grid's rows are `App::ALL` in draw order. Once a query is
            // typed the same row indices address the system-wide results.
            let searching = !query.trim().is_empty();
            let count = if searching {
                crate::seek::search(self, &query).len()
            } else {
                App::ALL.len()
            };
            match key {
                Key::Escape => {
                    self.set_overlay(Overlay::None);
                    self.overlay_damage();
                }
                Key::Down => {
                    let next = if count == 0 {
                        0
                    } else {
                        selected.saturating_add(1).checked_rem(count).unwrap_or(0)
                    };
                    self.set_overlay(Overlay::Launcher {
                        query,
                        selected: next,
                    });
                    self.overlay_damage();
                }
                Key::Up => {
                    let next = if count == 0 {
                        0
                    } else {
                        selected.checked_sub(1).unwrap_or(count.saturating_sub(1))
                    };
                    self.set_overlay(Overlay::Launcher {
                        query,
                        selected: next,
                    });
                    self.overlay_damage();
                }
                Key::Enter => {
                    let hit = if searching {
                        crate::seek::search(self, &query)
                            .get(selected)
                            .map(|result| result.hit)
                    } else {
                        App::ALL.get(selected).copied().map(crate::seek::Hit::App)
                    };
                    self.set_overlay(Overlay::None);
                    self.overlay_damage();
                    return Some(hit.map_or_else(Vec::new, |hit| self.seek_open(hit)));
                }
                Key::Backspace => {
                    let mut shorter = query;
                    shorter.pop();
                    self.set_overlay(Overlay::Launcher {
                        query: shorter,
                        selected: 0,
                    });
                    self.overlay_damage();
                }
                _ => return None,
            }
            return Some(Vec::new());
        }

        // Mission Control is handled before the mutable match: it needs to
        // read the window list, which borrows the manager the match would hold.
        if let Overlay::Overview { selected } = &self.overlay {
            let mut selected = *selected;
            let ids = self.overview_ids();
            let count = ids.len();
            let cols = crate::overview::columns(count);
            match key {
                Key::Escape => {
                    self.set_overlay(Overlay::None);
                    self.overlay_damage();
                }
                Key::Enter => {
                    self.overview_open(selected);
                }
                Key::Left | Key::Up | Key::Right | Key::Down | Key::Tab => {
                    let step = match key {
                        Key::Left => crate::overview::Step::Left,
                        Key::Up => crate::overview::Step::Up,
                        Key::Right => crate::overview::Step::Right,
                        _ => crate::overview::Step::Down,
                    };
                    selected = crate::overview::step(selected, count, cols, step);
                    self.set_overlay(Overlay::Overview { selected });
                    self.overlay_damage();
                }
                _ => return None,
            }
            return Some(Vec::new());
        }
        // The match takes a copy: an arm that closes its overlay calls
        // `set_overlay`, which needs the whole shell, and a live borrow of the
        // field would forbid that.
        match self.overlay.clone() {
            // Handled before the mutable match for the same reason as Mission
            // Control: searching the system needs to read the window list.
            Overlay::Launcher { .. } => None,
            Overlay::Switcher { selected } => match key {
                Key::Tab => {
                    let n = self.wm.switcher_order().len();
                    let selected = if n > 0 {
                        selected.saturating_add(1).checked_rem(n).unwrap_or(0)
                    } else {
                        selected
                    };
                    self.set_overlay(Overlay::Switcher { selected });
                    Some(Vec::new())
                }
                Key::Escape => {
                    self.set_overlay(Overlay::None);
                    self.switcher_holding = false;
                    Some(Vec::new())
                }
                _ => None,
            },
            // Mission Control returned above; it never reaches this match.
            Overlay::Overview { .. } => None,
            Overlay::QuickSettings | Overlay::Notifications | Overlay::Power | Overlay::Litro => {
                if key == Key::Escape {
                    self.set_overlay(Overlay::None);
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
                Key::Char('a') if mods.ctrl => Some(self.cycle_accent()),
                Key::Char('w') if mods.ctrl => Some(self.cycle_wallpaper()),
                Key::Char('m') if mods.ctrl => Some(self.toggle_reduced_motion()),
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
                            self.files.ask_confirm(
                                "Move to trash",
                                &format!("Move {} selected item(s) to the trash?", paths.len()),
                                action,
                            );
                            self.request_full_redraw();
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
        // An open launcher owns the keyboard, including over a focused
        // application. It is a search field drawn on top of the desktop; a
        // character typed into it that landed in the Files filter behind it
        // would be a character the user never sees.
        if self.text_into_launcher(text) {
            return;
        }
        // Otherwise a focused app text field eats characters — the omnibox,
        // the Files filter and the Files dialogs.
        let _ = self.text_into_app(text);
    }

    /// Feed typed characters into the launcher's search field. Returns whether
    /// the launcher is open and therefore owns the keyboard.
    fn text_into_launcher(&mut self, text: &str) -> bool {
        let Overlay::Launcher { query, selected } = &mut self.overlay else {
            return false;
        };
        // Control characters never reach a text field: a Tab or a newline in a
        // search box is a rendering problem waiting to happen.
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if !clean.is_empty() {
            query.push_str(&clean);
            *selected = 0;
            self.overlay_damage();
        }
        true
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

        // Coalesce with the previous move when that one was also the arrow
        // alone. A 1 kHz mouse otherwise queues a hundred damage pairs before
        // the next frame and looks like a burst of work.
        //
        // The *origin* has to survive the merge. The compositor will only move
        // the arrow it actually drew, so the step has to say where the arrow
        // was left, not where the pointer was one event ago -- otherwise every
        // merged move fails its check and composes the whole screen, which is
        // the stutter this coalescing exists to prevent.
        let mut origin = None;
        if self.cursor_step.is_some() && self.damage.len() == 2 {
            self.damage.clear();
            origin = self.cursor_step.map(|(from, _)| from);
        }

        let dragging = self.wm.is_dragging();
        if dragging {
            self.wm.update_drag(at.x, at.y);
        }

        let now = self.hot.hit(at.x, at.y);
        // A hover change repaints two controls and is still cheap; a drag moves
        // a window and is not. Only the plain move is the cursor's own frame.
        let mut only_cursor = now == self.hovered && !dragging;
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
                    only_cursor = false;
                    self.damage.push(b.inflate(2.0));
                }
            }
        }

        // The cursor itself: repaint where it was left and where it is now.
        let s = self.tokens.scale;
        let from = origin.unwrap_or(before);
        self.damage
            .push(crate::ui::cursor_bounds(from.x, from.y, s));
        self.damage.push(crate::ui::cursor_bounds(at.x, at.y, s));
        self.cursor_step = if only_cursor { Some((from, at)) } else { None };
    }

    /// Whether the last move is a cursor-only step, without consuming it.
    ///
    /// The display loop paces these: it asks first, waits out the rest of the
    /// pointer's own interval, and only then takes one. Peeking matters --
    /// taking a step early would move the compositor's arrow to a position the
    /// shell then treats as its last drawn one only on the next take, and the
    /// step after that would fail its damage comparison and compose a whole
    /// frame, which is the stutter pacing is meant to remove.
    #[must_use]
    pub fn cursor_step_pending(&self) -> bool {
        self.cursor_step.is_some()
    }

    /// Take the pointer's last move when the only damage it caused was the
    /// arrow, and clear that damage.
    ///
    /// `None` on any frame with something else to draw: a full redraw, a moved
    /// window, a spring in flight. The caller composes normally then. When this
    /// returns `Some`, the only thing that changed on screen is where the arrow
    /// is, which is exactly what [`crate::desktop::Compositor::move_cursor`]
    /// can paint on its own.
    pub fn take_cursor_step(&mut self) -> Option<(Point, Point)> {
        let step = self.cursor_step.take()?;
        if self.full_redraw || self.wm.is_dirty() || self.wm.focused() != self.drawn_focus {
            return None;
        }
        let s = self.tokens.scale;
        let from = crate::ui::cursor_bounds(step.0.x, step.0.y, s);
        let to = crate::ui::cursor_bounds(step.1.x, step.1.y, s);
        if self.damage.len() != 2 || !self.damage.contains(&from) || !self.damage.contains(&to) {
            return None;
        }
        self.damage.clear();
        Some(step)
    }

    fn pointer_button(
        &mut self,
        button: PointerButton,
        pressed: bool,
        at: Point,
        clicks: u8,
        mods: Modifiers,
    ) -> Vec<Effect> {
        self.pointer = at;
        // A click on a client's window belongs to that application. It is
        // forwarded *and* the window manager still sees it, because a click
        // must also focus and raise the window it lands on.
        let over = self.client_under(at);
        if let Some(host) = self.clients.as_mut() {
            host.button(
                over,
                (at.x, at.y),
                Self::pointer_button_code(button),
                pressed,
            );
        }
        // A button inside the page belongs to the web page, not to the shell:
        // forwarding it and stopping here is what makes a link clickable.
        if self.browser_page_point(at).is_some() {
            self.browser_engine_button(button, pressed, at, clicks, mods);
            return Vec::new();
        }
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
                self.set_overlay(Overlay::None);
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
                self.set_context_menu(Some(crate::apps::ContextMenu::for_surface(
                    surface,
                    (at.x, at.y),
                )));
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
            self.set_overlay(Overlay::None);
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
        if self.browser_page_point(at).is_some() {
            self.browser_engine_wheel(dy, at);
            return;
        }
        // Scrolling over the top bar's workspace area moves between workspaces,
        // which is a gesture people expect and costs nothing to support.
        if self.top_bar().contains(at.x, at.y) && self.wm.cycle_workspace(dy > 0.0) {
            self.request_full_redraw();
        }
    }

    /// The client window under a point, if the topmost thing there is one.
    ///
    /// The window manager answers with z-order and workspace already applied,
    /// so a client window buried under another window correctly yields `None`
    /// and the click does not reach the buried application.
    fn client_under(&self, at: Point) -> Option<WindowId> {
        let hit = self.wm.hit_test(at.x, at.y)?;
        if !matches!(hit.zone, HitZone::Content) {
            return None;
        }
        let hosts = self.clients.as_ref().is_some_and(|h| h.hosts(hit.window));
        hosts.then_some(hit.window)
    }

    /// Wayland's button numbering for the shell's own pointer buttons.
    const fn pointer_button_code(button: PointerButton) -> u8 {
        match button {
            PointerButton::Primary => 1,
            PointerButton::Middle => 2,
            PointerButton::Secondary => 3,
            _ => 1,
        }
    }

    /// What a click on a hotspot does.
    fn activate(&mut self, hot: Hot) -> Vec<Effect> {
        self.request_full_redraw();
        match hot {
            // The wordmark is the one label on the bar that means the system
            // itself, so the session's settings belong under it. The Apps menu
            // opens the launcher, which is what its name says.
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
                let Some(engine) = crate::apps::SearchEngine::ALL.get(usize::from(i)) else {
                    return Vec::new();
                };
                let engine = *engine;
                self.browser.status = format!("search engine: {}", engine.label());
                self.set_search_engine(engine)
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
            // The person icon opens the same power menu: the two icons sit
            // together at the end of the bar and mean one thing between them.
            Hot::StatusUser => self.perform(Action::TogglePowerMenu),
            Hot::PowerItem(n) => {
                // Row order is lock, sleep, shut down, restart. Lock is
                // reversible and needs no confirmation; the three that end the
                // session open the same confirmation the panel's session rows
                // did, because a menu row that powers a machine off on one
                // click is exactly what the live-consent rule forbids.
                self.overlay_damage();
                self.set_overlay(Overlay::None);
                let command = match n {
                    0 => return vec![Effect::RequestLock],
                    1 => PowerCommand::Suspend,
                    2 => PowerCommand::PowerOff,
                    _ => PowerCommand::Restart,
                };
                self.set_desktop_dialog(Some(crate::apps::DesktopDialog::PowerConfirm { command }));
                self.desktop_dialog_field.clear();
                self.desktop_dialog_selection = None;
                self.request_full_redraw();
                Vec::new()
            }
            Hot::LitroSession => {
                self.overlay_damage();
                self.set_overlay(Overlay::None);
                self.perform(Action::Open(App::Settings))
            }
            Hot::LitroLock => {
                self.overlay_damage();
                self.set_overlay(Overlay::None);
                vec![Effect::RequestLock]
            }
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
                if let Some(page) = crate::settings::Page::from_index(index) {
                    self.settings.page = page;
                    // A deep link's highlight belongs to the page it named. A
                    // user who navigates away has moved on, and a ring around a
                    // row they did not ask for is noise on the next page.
                    self.settings.focus = None;
                }
                Vec::new()
            }
            Hot::SettingsLanguage => self.toggle_language(),
            Hot::SettingsTheme => self.cycle_theme(),
            Hot::SettingsAccent => self.cycle_accent(),
            Hot::SettingsWallpaper => self.cycle_wallpaper(),
            Hot::SettingsSchedule => self.cycle_theme_schedule(),
            Hot::SettingsMotion => self.toggle_reduced_motion(),
            Hot::SetupDone => self.finish_first_run(),
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
                let hit = if query.trim().is_empty() {
                    App::ALL.get(i).copied().map(crate::seek::Hit::App)
                } else {
                    crate::seek::search(self, &query)
                        .get(i)
                        .map(|result| result.hit)
                };
                self.set_overlay(Overlay::None);
                hit.map_or_else(Vec::new, |hit| self.seek_open(hit))
            }
            Hot::OverviewBackdrop => {
                self.set_overlay(Overlay::None);
                self.overlay_damage();
                Vec::new()
            }
            Hot::OverviewItem(i) => {
                self.overview_open(i);
                Vec::new()
            }
            Hot::SwitcherItem(i) => {
                let order: Vec<WindowId> = self.wm.switcher_order().iter().map(|w| w.id).collect();
                if let Some(id) = order.get(i).copied() {
                    self.wm.unminimize(id);
                    self.wm.focus_window(id);
                }
                self.set_overlay(Overlay::None);
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
                    self.set_overlay(Overlay::None);
                    self.set_desktop_dialog(Some(crate::apps::DesktopDialog::PowerConfirm {
                        command,
                    }));
                    self.desktop_dialog_field.clear();
                    self.desktop_dialog_selection = None;
                    self.request_full_redraw();
                }
                Vec::new()
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
            // Tabs are the engine's tabs: the model and the broker are told
            // about every open, close and focus, or one of them ends up with a
            // tab the other has never heard of.
            Hot::BrowserNewTab => {
                self.browser.new_tab();
                self.browser_frame = None;
                self.browser.last_frame = None;
                if let Some(tab) = self.browser.active_engine_tab() {
                    self.send_line(&format!("NEW {tab}"));
                }
                self.request_full_redraw();
                Vec::new()
            }
            Hot::BrowserTab(i) => {
                if i != self.browser.active {
                    // The new tab's page has not been seen yet: keeping the old
                    // picture would put one site's pixels under another's URL.
                    self.browser_frame = None;
                    self.browser.last_frame = None;
                }
                let _ = crate::apps::browser_action(&mut self.browser, hot);
                if self.browser.active_tab_has_page() {
                    if let Some(tab) = self.browser.active_engine_tab() {
                        self.send_line(&format!("ACTIVATE {tab}"));
                    }
                }
                self.request_full_redraw();
                Vec::new()
            }
            Hot::BrowserCloseTab(i) => {
                let engine = self.browser.tabs.get(i).map(|tab| tab.engine_tab);
                if let Some(tab) = engine {
                    self.send_line(&format!("CLOSE {tab}"));
                }
                self.browser_frame = None;
                self.browser.last_frame = None;
                let _ = crate::apps::browser_action(&mut self.browser, hot);
                if self.browser.active_tab_has_page() {
                    if let Some(tab) = self.browser.active_engine_tab() {
                        self.send_line(&format!("ACTIVATE {tab}"));
                    }
                }
                self.request_full_redraw();
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
                    self.dismiss_notification(id);
                }
                Vec::new()
            }
            Hot::NotificationsClear => {
                self.clear_notifications();
                Vec::new()
            }
            Hot::Desktop => {
                // A click on the wallpaper clears selections everywhere the
                // desktop has them, which is what every file manager does.
                self.desktop_icons.select(None);
                self.files.selected = None;
                self.files.selected_rows.clear();
                self.set_context_menu(None);
                Vec::new()
            }
            Hot::DesktopIcon(i) => self.open_desktop_icon(i),
            Hot::MenuRow(action) => {
                self.set_context_menu(None);
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
                    self.set_desktop_dialog(None);
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
                let open = matches!(self.overlay, Overlay::Launcher { .. });
                self.set_overlay(if open {
                    Overlay::None
                } else {
                    Overlay::Launcher {
                        query: String::new(),
                        selected: 0,
                    }
                });
                Vec::new()
            }
            Action::CloseOverlay => {
                if self.overlay.is_open() {
                    self.overlay_damage();
                }
                self.set_overlay(Overlay::None);
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
                self.set_overlay(Overlay::Switcher { selected });
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
                // Windows' one sensible rule: the first press puts a snapped or
                // maximized window back, the second one minimizes it. A single
                // press that hides a window the user only wanted smaller is a
                // press that loses their work.
                if let Some(id) = focused {
                    let state = self.wm.window(id).map(|w| w.state);
                    if matches!(
                        state,
                        Some(WindowState::Maximized | WindowState::Snapped(_))
                    ) {
                        self.wm.restore(id);
                    } else {
                        self.wm.minimize(id);
                    }
                }
                Vec::new()
            }
            Action::ToggleMaximize => {
                if let Some(id) = focused {
                    self.wm.toggle_maximize(id);
                }
                Vec::new()
            }
            Action::SnapLeft
            | Action::SnapRight
            | Action::SnapTopLeft
            | Action::SnapTopRight
            | Action::SnapBottomLeft
            | Action::SnapBottomRight => {
                let edge = match action {
                    Action::SnapLeft => litric_wm::SnapEdge::Left,
                    Action::SnapRight => litric_wm::SnapEdge::Right,
                    Action::SnapTopLeft => litric_wm::SnapEdge::TopLeft,
                    Action::SnapTopRight => litric_wm::SnapEdge::TopRight,
                    Action::SnapBottomLeft => litric_wm::SnapEdge::BottomLeft,
                    _ => litric_wm::SnapEdge::BottomRight,
                };
                if let Some(id) = focused {
                    self.wm.snap(id, edge);
                }
                Vec::new()
            }
            Action::RestoreWindow => {
                if let Some(id) = focused {
                    self.wm.restore(id);
                }
                Vec::new()
            }
            Action::CycleAppWindow => {
                // The window the user lands on is revealed by the manager:
                // another workspace and a minimize are both undone, so the key
                // never focuses something that cannot be seen.
                self.wm.cycle_app(true);
                Vec::new()
            }
            Action::ToggleOverview => {
                self.overlay_damage();
                let open = matches!(self.overlay, Overlay::Overview { .. });
                let next = if open {
                    Overlay::None
                } else {
                    let selected = self
                        .wm
                        .focused()
                        .and_then(|id| self.wm.all().iter().position(|w| w.id == id))
                        .unwrap_or(0);
                    Overlay::Overview { selected }
                };
                self.set_overlay(next);
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
                let next = if self.overlay == Overlay::QuickSettings {
                    Overlay::None
                } else {
                    Overlay::QuickSettings
                };
                self.set_overlay(next);
                Vec::new()
            }
            Action::TogglePowerMenu => {
                self.overlay_damage();
                let next = if self.overlay == Overlay::Power {
                    Overlay::None
                } else {
                    Overlay::Power
                };
                self.set_overlay(next);
                Vec::new()
            }
            Action::ToggleLitroMenu => {
                self.overlay_damage();
                let next = if self.overlay == Overlay::Litro {
                    Overlay::None
                } else {
                    Overlay::Litro
                };
                self.set_overlay(next);
                Vec::new()
            }
            Action::ToggleNotifications => {
                self.overlay_damage();
                let next = if self.overlay == Overlay::Notifications {
                    Overlay::None
                } else {
                    Overlay::Notifications
                };
                self.set_overlay(next);
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
                    self.files.ask_confirm(
                        "Move to trash",
                        &format!("Move {} selected item(s) to the trash?", paths.len()),
                        action,
                    );
                    self.request_full_redraw();
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
            self.set_overlay(Overlay::None);
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
        let (w, h) = app.default_size();
        let scale = self.tokens.scale;
        self.wm
            .open(app.app_id(), app.title(), (w * scale, h * scale));
        vec![Effect::Launched(app)]
    }

    /// Run one system-wide search result.
    ///
    /// This is the only place that knows how a result becomes a consequence, so
    /// the keyboard and the pointer cannot disagree about what a row means.
    pub fn seek_open(&mut self, hit: crate::seek::Hit) -> Vec<Effect> {
        match hit {
            crate::seek::Hit::App(app) => self.open_app(app),
            crate::seek::Hit::Action(action) => self.perform(action),
            crate::seek::Hit::Setting(page) => {
                self.settings.page = page;
                self.settings.focus = None;
                self.open_app(App::Settings)
            }
            crate::seek::Hit::SettingRow(entry) => {
                self.settings.page = entry.page;
                self.settings.focus = Some(entry.key);
                self.open_app(App::Settings)
            }
            crate::seek::Hit::Window(id) => {
                // The manager undoes a minimize and switches workspaces; the
                // search field never has to know which of those is needed.
                self.wm.reveal(id);
                Vec::new()
            }
        }
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
        let id = self
            .notifications
            .post(source, title, body, priority, self.now_ms, ttl_ms);
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

/// The socket the browser broker listens on, when the shell has no override.
const BROWSER_SOCKET: &str = "/run/litric/browser.sock";

/// The tab a `FRAME <tab> <w> <h> <len>` header belongs to.
#[must_use]
fn frame_tab(header: &str) -> Option<u32> {
    header
        .strip_prefix("FRAME ")?
        .split(' ')
        .next()?
        .parse::<u32>()
        .ok()
}

/// `FRAME <tab> <w> <h> <len>` to `(w, h)`.
///
/// The header used to be `FRAME <tab> <len>`, and the two numeric fields were
/// read as width and height: every real frame was rejected as the wrong size
/// and the page stayed on its placeholder. The tab is skipped explicitly here
/// so the shape cannot drift back without a test failing.
#[must_use]
fn parse_frame_dims(header: &str) -> Option<(u32, u32)> {
    let mut it = header.strip_prefix("FRAME ")?.split(' ');
    it.next()?;
    let w = it.next()?.parse::<u32>().ok()?;
    let h = it.next()?.parse::<u32>().ok()?;
    if w == 0 || h == 0 || w > 8192 || h > 8192 {
        return None;
    }
    Some((w, h))
}

/// The FRAME header length rule for the browser link: `FRAME <tab> <w> <h> <len>`.
///
/// Every reply that is not a frame — `HELLO`, `PONG`, `OK`, `TAB`, `ERR` — has
/// no payload and must return `0`. Returning an error for them (which this
/// function used to do for any line at all) closed the browser session on the
/// first reply the broker sent, which made the handshake itself fatal.
fn browser_frame_len(header: &str) -> Result<usize, String> {
    let Some(rest) = header.strip_prefix("FRAME ") else {
        return Ok(0);
    };
    let fields: Vec<&str> = rest.split(' ').collect();
    let len = fields
        .get(3)
        .ok_or_else(|| "bad FRAME header".to_owned())?
        .parse::<usize>()
        .map_err(|_| "bad FRAME length".to_owned())?;
    // Bounded before allocating: a broken or hostile broker cannot make the
    // shell reserve more than one 1920x1080 BGRA frame.
    if len == 0 || len > 1920 * 1080 * 4 {
        return Err("frame too large".to_owned());
    }
    Ok(len)
}

/// One `TAB` line: what the engine says about a tab it owns.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct TabUpdate {
    /// The engine's stable tab id.
    tab: u32,
    /// `empty`, `loading`, `loaded` or `failed`.
    state: String,
    /// The address the engine actually has, percent-decoded.
    url: String,
    /// The document title, percent-decoded.
    title: String,
}

/// Parse `TAB <tab> state=<s> url=<pct> title=<pct> back=<0|1> fwd=<0|1> secure=<0|1>`.
///
/// Unknown fields are ignored rather than rejected: the broker may add one, and
/// a tab strip that stops updating because of a new key would be worse than the
/// missing feature.
#[must_use]
fn parse_tab_line(line: &str) -> Option<TabUpdate> {
    let mut fields = line.split_whitespace();
    if fields.next()? != "TAB" {
        return None;
    }
    let mut update = TabUpdate {
        tab: fields.next()?.parse::<u32>().ok()?,
        ..TabUpdate::default()
    };
    for field in fields {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        match key {
            "state" => value.clone_into(&mut update.state),
            "url" => {
                update.url = litric_core::pct::decode(value).unwrap_or_else(|_| value.to_owned())
            }
            "title" => {
                update.title = litric_core::pct::decode(value).unwrap_or_else(|_| value.to_owned());
            }
            _ => {}
        }
    }
    Some(update)
}

/// CDP's modifier bitmask: Alt = 1, Ctrl = 2, Meta = 4, Shift = 8.
fn modifier_bits(mods: Modifiers) -> u32 {
    let mut bits = 0;
    if mods.alt || mods.altgr {
        bits |= 1;
    }
    if mods.ctrl || mods.altgr {
        bits |= 2;
    }
    if mods.meta {
        bits |= 4;
    }
    if mods.shift {
        bits |= 8;
    }
    bits
}

/// `(key, code, windowsVirtualKeyCode, text)` for `Input.dispatchKeyEvent`.
///
/// `text` is only filled for keys whose character Chromium needs to see as part
/// of the key event (Enter and Tab); printable characters arrive through
/// `Input.insertText` instead, so nothing is typed twice.
fn engine_key(key: Key) -> (String, String, i64, String) {
    fn named(name: &str, vk: i64) -> (String, String, i64, String) {
        (name.to_owned(), name.to_owned(), vk, String::new())
    }
    match key {
        Key::Char(c) => {
            let (code, vk) = if c.is_ascii_alphabetic() {
                (format!("Key{}", c.to_ascii_uppercase()), {
                    i64::from(c.to_ascii_uppercase() as u8)
                })
            } else if c.is_ascii_digit() {
                let digit = i64::from(c.to_digit(10).unwrap_or(0));
                (format!("Digit{c}"), 48i64.saturating_add(digit))
            } else if c == ' ' {
                ("Space".to_owned(), 32)
            } else {
                (String::new(), 0)
            };
            (c.to_string(), code, vk, String::new())
        }
        Key::Enter => ("Enter".to_owned(), "Enter".to_owned(), 13, "\r".to_owned()),
        Key::Tab => ("Tab".to_owned(), "Tab".to_owned(), 9, "\t".to_owned()),
        Key::Escape => named("Escape", 27),
        Key::Backspace => named("Backspace", 8),
        Key::Delete => named("Delete", 46),
        Key::Insert => named("Insert", 45),
        Key::Home => named("Home", 36),
        Key::End => named("End", 35),
        Key::PageUp => named("PageUp", 33),
        Key::PageDown => named("PageDown", 34),
        Key::Left => named("ArrowLeft", 37),
        Key::Up => named("ArrowUp", 38),
        Key::Right => named("ArrowRight", 39),
        Key::Down => named("ArrowDown", 40),
        Key::CapsLock => named("CapsLock", 20),
        Key::Function(n) if (1..=12).contains(&n) => {
            let name = format!("F{n}");
            (
                name.clone(),
                name,
                111i64.saturating_add(i64::from(n)),
                String::new(),
            )
        }
        _ => (String::new(), String::new(), 0, String::new()),
    }
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
    fn the_accent_applies_on_standard_themes_and_not_on_high_contrast_ones() {
        // The UI says a high-contrast theme keeps its accent. This is the
        // assertion behind that sentence: the choice is recorded either way,
        // and resolved by one function rather than by the drawing code.
        let mut preferences =
            crate::settings::Preferences::new(ThemeKind::Dark, crate::i18n::Language::English);
        preferences.accent = litric_design::Accent::Violet;

        // Noon, so the schedule cannot be what produced the dark theme.
        let dark = resolved_theme(preferences, 12);
        assert_ne!(
            dark.palette.accent,
            Theme::of(ThemeKind::Dark).palette.accent
        );
        assert!(dark.palette.on_accent.contrast(dark.palette.accent) >= 4.5);

        preferences.theme = ThemeKind::Light;
        let light = resolved_theme(preferences, 12);
        assert_ne!(
            light.palette.accent,
            Theme::of(ThemeKind::Light).palette.accent,
            "the same violet token must resolve to the light theme's violet"
        );
        assert!(light.palette.on_accent.contrast(light.palette.accent) >= 4.5);

        for kind in [ThemeKind::HighContrastDark, ThemeKind::HighContrastLight] {
            preferences.theme = kind;
            assert_eq!(
                resolved_theme(preferences, 12),
                Theme::of(kind),
                "{}: a high-contrast theme must keep its own accent",
                kind.token()
            );
        }
    }

    #[test]
    fn the_appearance_schedule_follows_the_clock_and_nothing_else() {
        // 2000-01-01 in UTC, which is the zone the test locale falls back to.
        const NOON_MS: u64 = 946_728_000_000;
        const NIGHT_MS: u64 = 946_767_600_000;

        let preferences = crate::settings::Preferences {
            theme: ThemeKind::Light,
            theme_schedule: crate::settings::ThemeSchedule::Auto,
            ..crate::settings::Preferences::default()
        };

        let mut scheduled = shell();
        scheduled.load_preferences(preferences);
        scheduled.tick(NOON_MS);
        assert_eq!(
            scheduled.tokens.theme.kind,
            ThemeKind::Light,
            "noon is light under an automatic schedule"
        );
        scheduled.tick(NIGHT_MS);
        assert_eq!(
            scheduled.tokens.theme.kind,
            ThemeKind::Dark,
            "23:00 is dark under an automatic schedule"
        );

        // Without the schedule the clock is not consulted at all, so a manual
        // light theme stays light at midnight.
        let mut manual = shell();
        manual.load_preferences(crate::settings::Preferences {
            theme_schedule: crate::settings::ThemeSchedule::Manual,
            ..preferences
        });
        manual.tick(NIGHT_MS);
        assert_eq!(manual.tokens.theme.kind, ThemeKind::Light);
    }

    #[test]
    fn opening_a_setting_row_from_search_lands_on_its_page_with_the_row_focused() {
        let mut s = shell();
        let entry = crate::registry::find("reduce_motion").expect("reduce_motion is registered");
        s.seek_open(crate::seek::Hit::SettingRow(entry));
        assert_eq!(s.settings.page, crate::settings::Page::Accessibility);
        assert_eq!(s.settings.focus, Some("reduce_motion"));

        // Choosing a category by hand clears the highlight: the ring belongs to
        // the link, not to the page.
        s.activate(Hot::SettingsCategory(0));
        assert_eq!(s.settings.focus, None);
    }

    #[test]
    fn the_next_wallpaper_row_changes_the_wallpaper_and_not_the_theme() {
        // The row has always said "Next wallpaper" and used to cycle the theme.
        // A menu item that names one thing and changes another is a lie the UI
        // tells on every right-click.
        let mut s = shell();
        let before_theme = s.settings.preferences.theme;
        let before_wallpaper = s.settings.preferences.wallpaper;
        let effects = s.perform_menu(crate::apps::MenuAction::NextWallpaper);
        assert_ne!(
            s.settings.preferences.wallpaper, before_wallpaper,
            "the row named next wallpaper and left it alone"
        );
        assert_eq!(
            s.settings.preferences.theme, before_theme,
            "the row named next wallpaper and changed the theme"
        );
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::PersistPreferences(_))),
            "a wallpaper change that is not persisted is gone at logout"
        );
    }

    #[test]
    fn choosing_a_search_engine_persists_it_and_a_restart_applies_it() {
        use crate::apps::SearchEngine;

        let mut s = shell();
        assert_eq!(s.browser.engine, SearchEngine::Google);

        let effects = s.perform_menu(crate::apps::MenuAction::EngineDuckDuckGo);
        assert_eq!(s.browser.engine, SearchEngine::DuckDuckGo);
        let saved = effects
            .iter()
            .find_map(|e| match e {
                Effect::PersistPreferences(p) => Some(*p),
                _ => None,
            })
            .expect("choosing an engine must be persisted, not remembered for one window");
        assert_eq!(saved.search_engine, SearchEngine::DuckDuckGo);
        // The write completed, as it does in the event loop before the next
        // click: a second change while one is in flight is refused on purpose.
        s.finish_preferences_save(Ok(()));

        // The start page's chips are the second route to the same setting, and
        // they must not disagree with the menu.
        let effects = s.activate(Hot::EngineChip(2));
        assert_eq!(s.browser.engine, SearchEngine::Bing);
        assert!(effects.iter().any(
            |e| matches!(e, Effect::PersistPreferences(p) if p.search_engine == SearchEngine::Bing)
        ));

        // What a restart does: a fresh shell loaded from that record.
        let mut restarted = shell();
        restarted.load_preferences(saved);
        assert_eq!(restarted.browser.engine, SearchEngine::DuckDuckGo);
    }

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

    #[test]
    fn a_frame_header_is_read_as_tab_width_height_and_length() {
        let header = "FRAME 7 1280 800 4096000";
        assert_eq!(frame_tab(header), Some(7));
        assert_eq!(parse_frame_dims(header), Some((1280, 800)));
        assert_eq!(browser_frame_len(header), Ok(1280 * 800 * 4));
        // The shape the broker used to write, `FRAME <tab> <len>`, must not be
        // read as a 7-pixel-wide frame: that mismatch silently dropped every
        // real frame and left the page on its placeholder.
        assert_eq!(parse_frame_dims("FRAME 7 4096000"), None);
        assert!(browser_frame_len("FRAME 7 4096000").is_err());
        // A length the shell would allocate is bounded before it allocates.
        assert!(browser_frame_len("FRAME 7 1280 800 999999999").is_err());
        assert!(browser_frame_len("FRAME 7 1280 800 0").is_err());
        // Every non-frame reply has no payload. Treating one as a broken frame
        // header closed the session on the broker's first reply.
        for line in [
            "HELLO 1 engine=chromium",
            "OK viewport=1280x800",
            "PONG",
            "TAB 1 state=loaded",
            "ERR engine-failed x",
        ] {
            assert_eq!(browser_frame_len(line), Ok(0), "{line}");
        }
    }

    #[test]
    fn a_tab_line_carries_the_address_and_title_a_tab_shows() {
        let line = "TAB 3 state=loaded \
             url=https%3A%2F%2Fexample.com%2Fa%20b \
             title=Example%20%E0%B8%97%E0%B8%94%E0%B8%AA%E0%B8%AD%E0%B8%9A \
             back=1 fwd=0 secure=1";
        let update = parse_tab_line(line).expect("a TAB line");
        assert_eq!(update.tab, 3);
        assert_eq!(update.state, "loaded");
        assert_eq!(update.url, "https://example.com/a b");
        assert_eq!(update.title, "Example ทดสอบ");
        assert!(parse_tab_line("PONG").is_none());
        assert!(parse_tab_line("TAB").is_none());
    }

    #[test]
    fn a_key_event_does_not_also_carry_printable_text() {
        // Printable characters arrive through TEXT. If the key event carried
        // the character too, every letter typed into a page would arrive twice.
        let (name, code, vk, text) = engine_key(Key::Char('a'));
        assert_eq!(
            (name.as_str(), code.as_str(), vk, text.as_str()),
            ("a", "KeyA", 65, "")
        );
        let (_, _, vk, text) = engine_key(Key::Enter);
        assert_eq!((vk, text.as_str()), (13, "\r"));
        assert_eq!(
            modifier_bits(Modifiers {
                shift: true,
                ctrl: true,
                ..Modifiers::NONE
            }),
            10
        );
        let (name, code, _, _) = engine_key(Key::Left);
        assert_eq!((name.as_str(), code.as_str()), ("ArrowLeft", "ArrowLeft"));
    }

    /// A broker stand-in that speaks LBP/1 over a real Unix socket and reports
    /// every command it receives. The point of the test is the wire: the shell's
    /// framing, its TAB handling and the lines its input path writes.
    #[cfg(unix)]
    fn fake_broker(path: &std::path::Path) -> std::sync::mpsc::Receiver<String> {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;

        let listener = UnixListener::bind(path).expect("bind the fake broker");
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut line = String::new();
            let (w, h, tab) = (4u32, 2u32, 1u32);
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let cmd = line.trim_end().to_owned();
                let _ = tx.send(cmd.clone());
                if cmd.starts_with("HELLO") {
                    let _ = writeln!(stream, "HELLO 1 engine=chromium version=test");
                } else if cmd.starts_with("VIEWPORT") {
                    let _ = writeln!(stream, "OK viewport=1280x800");
                } else if cmd.starts_with("GO ")
                    || cmd.starts_with("NEW ")
                    || cmd.starts_with("ACTIVATE ")
                {
                    let _ = writeln!(
                        stream,
                        "TAB {tab} state=loaded url=https%3A%2F%2Fexample.com%2F \
                         title=Example back=0 fwd=0 secure=1"
                    );
                    let mut pixels = vec![0u8; (w * h * 4) as usize];
                    // The shell reads this payload's length and its alpha, so
                    // one opaque byte stands for the whole buffer.
                    if let Some(alpha) = pixels.get_mut(3) {
                        *alpha = 0xFF;
                    }
                    let _ = writeln!(stream, "FRAME {tab} {w} {h} {}", pixels.len());
                    let _ = stream.write_all(&pixels);
                } else if cmd.starts_with("QUIT") {
                    break;
                }
                let _ = stream.flush();
                line.clear();
            }
        });
        rx
    }

    #[cfg(unix)]
    fn wait_for_browser_frame(s: &mut Shell) {
        for _ in 0..200 {
            s.pump_browser();
            if s.browser_frame.is_some() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[cfg(unix)]
    #[test]
    fn the_shell_shows_the_frame_a_broker_sends_and_forwards_a_click() {
        let dir = std::env::temp_dir().join(format!(
            "litric-shell-broker-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("browser.sock");
        let commands = fake_broker(&path);

        let mut s = shell();
        s.browser_socket = path.clone();
        s.open_app(App::Browser);
        s.browser_navigate("https://example.com/");
        wait_for_browser_frame(&mut s);

        let frame = s.browser_frame.as_ref().expect("a frame reached the shell");
        assert_eq!(frame.len(), 4 * 2 * 4, "the payload length is the header's");
        assert_eq!(s.browser.last_frame, Some((4, 2)));
        let tab = s.browser.tabs.first().expect("a tab was opened");
        assert_eq!(tab.url, "https://example.com/");
        assert_eq!(tab.title, "Example");
        // The frame must not replace the tab's own status with a generic one:
        // the title is how a person knows which page they are looking at.
        assert!(
            s.browser.status.contains("Example"),
            "the status line lost the page title: {:?}",
            s.browser.status
        );

        // Where the view drew the frame is the mapping the pointer uses.
        s.set_browser_engine_area(Some(Rect::new(100.0, 100.0, 800.0, 400.0)));
        let at = Point::new(500.0, 300.0); // centre of the area -> (2, 1)
        for (pressed, now) in [(true, 1_000u64), (false, 1_016)] {
            s.handle(
                &InputEvent::PointerButton {
                    button: PointerButton::Primary,
                    pressed,
                    at,
                    clicks: 1,
                    mods: Modifiers::NONE,
                },
                now,
            );
        }
        s.handle(
            &InputEvent::KeyDown {
                key: Key::Enter,
                code: 28,
                mods: Modifiers::NONE,
                repeat: false,
            },
            1_032,
        );
        s.handle(
            &InputEvent::TextInput {
                text: "hi".to_owned(),
            },
            1_040,
        );

        let mut seen = Vec::new();
        while let Ok(cmd) = commands.recv_timeout(std::time::Duration::from_millis(500)) {
            seen.push(cmd);
            if seen.iter().any(|c| c.starts_with("TEXT ")) {
                break;
            }
        }
        assert!(seen.iter().any(|c| c == "HELLO 1"), "{seen:?}");
        assert!(
            seen.iter()
                .any(|c| c.starts_with("GO 1 https%3A%2F%2Fexample.com%2F")),
            "the navigation never reached the broker: {seen:?}"
        );
        assert!(
            seen.iter().any(|c| c == "MOUSE 1 down 2.0 1.0 left 1 0"),
            "the press was not forwarded at the page pixel: {seen:?}"
        );
        assert!(
            seen.iter().any(|c| c == "MOUSE 1 up 2.0 1.0 left 1 0"),
            "the release was not forwarded: {seen:?}"
        );
        // Enter carries its character in the key event; typed text goes as TEXT.
        assert!(
            seen.iter().any(|c| c == "KEY 1 down Enter Enter 13 0 %0D"),
            "{seen:?}"
        );
        assert!(seen.iter().any(|c| c == "TEXT 1 hi"), "{seen:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn the_shell_says_why_a_missing_engine_cannot_show_a_page() {
        let dir = std::env::temp_dir().join(format!(
            "litric-shell-noengine-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("browser.sock");
        let commands = fake_broker(&path);
        // A broker that answers, then refuses the load by name.
        let mut s = shell();
        s.browser_socket = path.clone();
        s.open_app(App::Browser);
        s.browser_navigate("https://example.com/");
        // Replace the status the fake broker left with the refusal the real one
        // writes, and check the shell turns it into a sentence.
        s.apply_engine_error("engine-unavailable No%20Chromium%20binary%20is%20present");
        assert!(
            s.browser
                .status
                .contains("the browser engine is not available: No Chromium binary is present"),
            "{}",
            s.browser.status
        );
        assert!(
            s.browser.tabs.first().is_some_and(|tab| tab.error),
            "a failed tab is marked as failed"
        );
        drop(commands);
        let _ = std::fs::remove_dir_all(&dir);
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
        // Win+Down restores a snapped window first, and only the second press
        // hides it: the user asked for smaller, not for gone.
        s.perform(Action::MinimizeWindow);
        assert_eq!(s.wm.window(id).map(|w| w.state), Some(WindowState::Normal));
        assert_eq!(
            s.wm.window(id).map(|w| w.frame),
            s.wm.window(id).map(|w| w.restore),
            "restoring did not put the window back to its own size"
        );
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
    fn a_pointer_move_repaints_only_the_arrow_and_matches_a_whole_frame() {
        // The shortcut is only worth having if it is invisible. Two identical
        // shells get the same move: one answers it by restoring the pixels
        // under the old arrow, the other by composing the frame the slow way.
        // Every pixel of the two frames must agree, or the arrow is smearing
        // something the composition would have fixed.
        use crate::desktop::Compositor;
        use litric_design::{Theme, ThemeKind};
        let theme = Theme::of(ThemeKind::Dark);
        let mut fast = shell();
        let mut full = shell();
        let mut a = Compositor::new(1280, 800, &theme, crate::desktop::Wallpaper::System);
        let mut b = Compositor::new(1280, 800, &theme, crate::desktop::Wallpaper::System);
        let move_to = |s: &mut Shell, x: f32, y: f32, t: u64| {
            s.handle(
                &InputEvent::PointerMove {
                    at: Point::new(x, y),
                    delta: Point::new(1.0, 1.0),
                },
                t,
            );
        };
        // `take_damage` is what the loop does before it composes: it is also
        // what clears the full-redraw flag a frame answers.
        let _ = fast.take_damage();
        let _ = full.take_damage();
        let _ = a.repaint(&mut fast, true, &[]);
        let _ = b.repaint(&mut full, true, &[]);

        move_to(&mut fast, 100.0, 120.0, 1_000);
        move_to(&mut full, 100.0, 120.0, 1_000);
        let _ = fast.take_damage();
        let _ = full.take_damage();
        let _ = a.repaint(&mut fast, true, &[]);
        let _ = b.repaint(&mut full, true, &[]);

        move_to(&mut fast, 300.0, 260.0, 1_016);
        move_to(&mut full, 300.0, 260.0, 1_016);
        let (from, to) = fast
            .take_cursor_step()
            .expect("a move over the bare desktop is the arrow's own frame");
        let plan = a
            .move_cursor(&fast, from, to)
            .expect("the cursor was where the last frame left it");
        assert!(
            matches!(plan, crate::canvas::Repaint::Regions(_)),
            "the shortcut repainted: {plan:?}"
        );
        let damage = full.take_damage();
        let _ = b.repaint(&mut full, false, &damage);

        let mut mismatched = 0u32;
        for y in 0..800 {
            for x in 0..1280 {
                if a.scene.get(x, y) != b.scene.get(x, y) {
                    mismatched = mismatched.saturating_add(1);
                }
            }
        }
        assert_eq!(
            mismatched, 0,
            "{mismatched} pixels differ from a full composition"
        );
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
        // The mark opens the LitricOS menu; the Apps menu is what opens the
        // launcher. Either answer proves the release-on-target rule; the
        // mark's own menu is what this hotspot is.
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
            s.activate(Hot::MenuApps);
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
    fn mission_control_shows_every_workspace_and_opens_the_chosen_window() {
        let mut s = shell();
        let a = s.wm.open("com.litric.files", "Documents", (500.0, 400.0));
        s.wm.set_workspace(1);
        let b = s.wm.open("com.litric.terminal", "Terminal", (500.0, 400.0));
        s.wm.set_workspace(0);
        assert_eq!(s.wm.focused(), Some(a));

        s.handle(&key(Key::Tab, meta()), 1000);
        assert!(
            matches!(s.overlay, Overlay::Overview { .. }),
            "Win+Tab did not open Mission Control: {:?}",
            s.overlay
        );
        // Cards are ordered by workspace, so the second card is the Terminal
        // on workspace 2 even though it is not the most recently used window.
        s.handle(&key(Key::Right, Modifiers::NONE), 1010);
        s.handle(&key(Key::Enter, Modifiers::NONE), 1020);
        assert_eq!(s.overlay, Overlay::None);
        assert_eq!(s.wm.active_workspace(), 1);
        assert_eq!(s.wm.focused(), Some(b));
    }

    #[test]
    fn mission_control_closes_on_escape_and_the_backdrop() {
        let mut s = shell();
        s.wm.open("com.litric.files", "Documents", (400.0, 300.0));
        s.perform(Action::ToggleOverview);
        assert!(matches!(s.overlay, Overlay::Overview { .. }));
        s.handle(&key(Key::Escape, Modifiers::NONE), 1000);
        assert_eq!(s.overlay, Overlay::None);

        s.perform(Action::ToggleOverview);
        let out = s.activate(Hot::OverviewBackdrop);
        assert!(out.is_empty());
        assert_eq!(s.overlay, Overlay::None);
    }

    #[test]
    fn the_quarters_snap_to_the_work_area() {
        let mut s = shell();
        let id = s.wm.open("com.litric.files", "Documents", (400.0, 300.0));
        for (action, edge) in [
            (Action::SnapTopLeft, litric_wm::SnapEdge::TopLeft),
            (Action::SnapTopRight, litric_wm::SnapEdge::TopRight),
            (Action::SnapBottomLeft, litric_wm::SnapEdge::BottomLeft),
            (Action::SnapBottomRight, litric_wm::SnapEdge::BottomRight),
        ] {
            s.perform(action);
            let w = s.wm.window(id).expect("the window is still open");
            assert_eq!(w.state, litric_wm::WindowState::Snapped(edge));
            assert_eq!(w.frame, edge.bounds(s.wm.work_area()));
        }
    }

    #[test]
    fn the_minimize_key_restores_before_it_hides() {
        let mut s = shell();
        let id = s.wm.open("com.litric.files", "Documents", (400.0, 300.0));
        s.perform(Action::ToggleMaximize);
        assert_eq!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Maximized)
        );
        s.perform(Action::MinimizeWindow);
        assert_eq!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Normal),
            "the first minimize hid a window the user only wanted smaller"
        );
        s.perform(Action::MinimizeWindow);
        assert_eq!(
            s.wm.window(id).map(|w| w.state),
            Some(WindowState::Minimized),
            "the second minimize did not hide the window"
        );
    }

    #[test]
    fn the_application_cycle_stays_inside_the_application() {
        let mut s = shell();
        let first = s.wm.open("com.litric.files", "One", (400.0, 300.0));
        let second = s.wm.open("com.litric.files", "Two", (400.0, 300.0));
        s.wm.open("com.litric.terminal", "Terminal", (400.0, 300.0));
        s.wm.focus_window(second);
        s.perform(Action::CycleAppWindow);
        assert_eq!(s.wm.focused(), Some(first));
        assert_eq!(
            s.wm.focused()
                .and_then(|id| s.wm.window(id))
                .map(|w| w.app_id.clone()),
            Some("com.litric.files".to_owned())
        );
    }

    #[test]
    fn first_run_setup_finishes_from_the_keyboard() {
        let mut s = shell();
        s.settings.preferences.onboarded = false;

        // Enter on the focused Finish control records setup as done.
        s.keyboard_focus = Some(Hot::SetupDone);
        let effects = s.handle(&key(Key::Enter, Modifiers::NONE), 3_000);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::PersistPreferences(p) if p.onboarded)),
            "finishing setup persists onboarded=true"
        );
    }

    // ---- exit animations ---------------------------------------------------
    //
    // A surface that closes is not gone the frame it stops accepting input.
    // Its reveal spring has to fall, and the snapshot that keeps its pixels
    // alive is dropped by that spring rather than by a timer.

    #[test]
    fn a_closed_panel_is_kept_exactly_while_its_exit_spring_has_pixels() {
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        s.tick_after(16);
        assert!(
            s.overlay_reveal(&s.overlay) > 0.05,
            "fixture: the panel is open enough to have pixels to lose"
        );

        s.perform(Action::CloseOverlay);
        let old = s
            .retiring_overlay()
            .cloned()
            .expect("a closing panel must keep its pixels");
        assert_eq!(old, Overlay::QuickSettings);
        assert!(s.overlay_reveal(&old) > 0.0);

        let mut frames = 0;
        while s.retiring_overlay().is_some() {
            s.tick_after(16);
            frames += 1;
            assert!(frames < 60, "the snapshot outlived its settle budget");
        }
        assert!(
            frames > 1,
            "the panel vanished in one frame, which is the defect this fixes"
        );
    }

    #[test]
    fn reopening_a_panel_reverses_its_spring_instead_of_drawing_two_copies() {
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        s.tick_after(16);
        s.perform(Action::CloseOverlay);
        assert!(s.retiring_overlay().is_some());
        s.perform(Action::ToggleQuickSettings);
        assert!(
            s.retiring_overlay().is_none(),
            "reopening the same panel must not leave a second copy on screen"
        );
    }

    #[test]
    fn swapping_between_panels_that_share_a_spring_does_not_keep_a_ghost() {
        let mut s = shell();
        s.perform(Action::ToggleQuickSettings);
        s.tick_after(16);
        s.perform(Action::TogglePowerMenu);
        assert_eq!(s.overlay, Overlay::Power);
        assert!(
            s.retiring_overlay().is_none(),
            "quick settings and the power menu share one reveal spring, so \
             there is no exit to animate between them"
        );
    }

    #[test]
    fn a_retired_menu_is_still_painted_but_registers_no_click_target() {
        let mut s = shell();
        s.set_context_menu(Some(crate::apps::ContextMenu::for_surface(
            crate::apps::MenuSurface::Desktop,
            (40.0, 40.0),
        )));
        s.tick_after(16);

        // Fixture: while it is open the row is both drawn and clickable.
        let mut open = crate::canvas::Canvas::new(1280, 800, 0);
        crate::views::draw(&mut open, &mut s);
        assert!(s
            .hotspots()
            .rect_of(Hot::MenuRow(crate::apps::MenuAction::NewFolder))
            .is_some());

        s.set_context_menu(None);
        assert!(s.retiring_context_menu().is_some());
        // One frame of the exit, so the retiring menu is drawn smaller than
        // the open one instead of at exactly the value it was left at.
        s.tick_after(16);
        let mut leaving = crate::canvas::Canvas::new(1280, 800, 0);
        crate::views::draw(&mut leaving, &mut s);
        assert!(
            s.hotspots()
                .rect_of(Hot::MenuRow(crate::apps::MenuAction::NewFolder))
                .is_none(),
            "a menu on its way out must not answer the next click"
        );
        assert!(
            open.pixels() != leaving.pixels(),
            "the retiring menu drew nothing: the close is still a cut"
        );

        // Once its spring lands, the last copy is gone too: the same corner
        // of the screen carries strictly less ink once the menu has left.
        let ink = |c: &crate::canvas::Canvas| {
            let mut n = 0;
            for y in 30i64..220 {
                for x in 30i64..250 {
                    if c.get(x, y).unwrap_or(0) != 0 {
                        n += 1;
                    }
                }
            }
            n
        };
        let while_leaving = ink(&leaving);
        for _ in 0..40 {
            s.tick_after(16);
        }
        assert!(s.retiring_context_menu().is_none());
        let mut settled = crate::canvas::Canvas::new(1280, 800, 0);
        crate::views::draw(&mut settled, &mut s);
        assert!(
            while_leaving > ink(&settled),
            "the retired menu painted no pixels: {while_leaving} then, {} after",
            ink(&settled)
        );
    }

    #[test]
    fn a_dismissed_dialog_keeps_its_panel_but_not_its_buttons() {
        let mut s = shell();
        s.set_desktop_dialog(Some(crate::apps::DesktopDialog::NewFolder));
        s.tick_after(16);
        assert!(
            s.anims.dialog.value() > 0.05,
            "fixture: the dialog is opening"
        );

        s.set_desktop_dialog(None);
        assert!(s.retiring_desktop_dialog().is_some());
        let mut c = crate::canvas::Canvas::new(1280, 800, 0);
        crate::views::draw(&mut c, &mut s);
        assert!(
            s.hotspots().rect_of(Hot::DialogConfirm).is_none(),
            "a fading dialog must not offer its button again"
        );

        for _ in 0..80 {
            s.tick_after(16);
        }
        assert!(
            s.retiring_desktop_dialog().is_none(),
            "the dialog snapshot outlived its exit"
        );
    }

    #[test]
    fn a_dismissed_banner_keeps_its_pixels_until_its_spring_lands() {
        let mut s = shell();
        let id = s.notify(
            "test",
            "Disk almost full",
            "12% remaining",
            Priority::High,
            Some(5_000),
        );
        s.tick_after(16);
        assert!(
            s.anims.banner_reveal(id) > 0.0,
            "fixture: the banner is entering"
        );

        assert!(s.dismiss_notification(id));
        assert!(s.retiring_banners().iter().any(|b| b.id == id));
        s.tick_after(16);
        assert!(
            s.retiring_banners().iter().any(|b| b.id == id),
            "the banner was dropped before it could leave"
        );

        for _ in 0..40 {
            s.tick_after(16);
        }
        assert!(
            s.retiring_banners().is_empty(),
            "a banner outlived its exit spring"
        );
    }
}
