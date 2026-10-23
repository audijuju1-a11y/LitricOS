//! Global shortcuts: from a key press to a thing that happens.
//!
//! A shortcut table is a policy, so it is data rather than a chain of `if`
//! statements in the key handler. That makes three things possible that
//! matter: the launcher can show the real binding beside every action, the
//! settings application can rebind one without touching the shell, and the
//! whole table can be tested without a keyboard.
//!
//! # Meta is the system's, not the application's
//!
//! Every LitricOS shortcut is on Meta (the Windows key) or on a combination no
//! application uses. That is deliberate: a system shortcut an application can
//! swallow is a system shortcut that stops working exactly when the machine is
//! misbehaving and you most need it. `Ctrl+Alt+Home` is reserved and
//! unrebindable for the same reason — it is the way back when everything else
//! is wedged.

use litric_input::keys::{Key, Modifiers};

/// Something the desktop can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Action {
    /// Open the application launcher, or close it if it is open.
    ToggleLauncher,
    /// Dismiss whatever overlay is showing.
    CloseOverlay,
    /// Switch to environment 1, 2 or 3.
    SwitchEnvironment(u8),
    /// Focus the next window, most-recently-used order.
    NextWindow,
    /// Focus the previous window.
    PrevWindow,
    /// Close the focused window.
    CloseWindow,
    /// Minimize the focused window.
    MinimizeWindow,
    /// Maximize the focused window, or restore it.
    ToggleMaximize,
    /// Snap the focused window to the left half.
    SnapLeft,
    /// Snap the focused window to the right half.
    SnapRight,
    /// Snap the focused window to the top-left quarter.
    SnapTopLeft,
    /// Snap the focused window to the top-right quarter.
    SnapTopRight,
    /// Snap the focused window to the bottom-left quarter.
    SnapBottomLeft,
    /// Snap the focused window to the bottom-right quarter.
    SnapBottomRight,
    /// Put the focused window back to the size it had before it was snapped or
    /// maximized.
    RestoreWindow,
    /// Cycle through the focused application's own windows.
    CycleAppWindow,
    /// Show every window, on every workspace, at once.
    ToggleOverview,
    /// Centre the focused window.
    CenterWindow,
    /// Show workspace `n`, counting from zero.
    Workspace(u8),
    /// Next workspace.
    NextWorkspace,
    /// Previous workspace.
    PrevWorkspace,
    /// Move the focused window to workspace `n`.
    MoveToWorkspace(u8),
    /// Minimize everything.
    ShowDesktop,
    /// Open a built-in application.
    Open(App),
    /// Open the quick-settings panel.
    ToggleQuickSettings,
    /// Open the power menu: lock, sleep, shut down, restart.
    TogglePowerMenu,
    /// Open the LitricOS menu under the wordmark: the session's own settings.
    ToggleLitroMenu,
    /// Open the notification list.
    ToggleNotifications,
    /// Show the diagnostic overlay.
    ToggleDiagnostics,
    /// Lock the session.
    Lock,
    /// Volume down.
    VolumeDown,
    /// Volume up.
    VolumeUp,
    /// Mute.
    VolumeMute,
    /// Screen brighter.
    BrightnessUp,
    /// Screen dimmer.
    BrightnessDown,
    /// The escape hatch: return to the LitricOS host from any environment.
    ReturnToHost,
    /// Files: create a folder in the folder being shown.
    FilesNewFolder,
    /// Files: create an empty text file in the folder being shown.
    FilesNewFile,
    /// Files: rename the selection.
    FilesRename,
    /// Files: copy the selection to the Files clipboard.
    FilesCopy,
    /// Files: cut the selection to the Files clipboard.
    FilesCut,
    /// Files: paste from the Files clipboard into the folder being shown.
    FilesPaste,
    /// Files: move the selection to the trash.
    FilesTrash,
    /// Files: archive the selection into a ZIP.
    FilesCompress,
    /// Files: extract the selected archive into a folder beside it.
    FilesExtract,
    /// Files: focus the filter / address field.
    FilesSearch,
    /// Files: move up one folder.
    FilesUp,
    /// Defender: open the application.
    OpenDefender,
    /// Defender: start a scan of the home folders.
    ScanNow,
    /// Browser: focus the address bar.
    BrowserAddress,
    /// Browser: reload the focused tab.
    BrowserReload,
    /// Open the trash window.
    ViewTrash,
    /// Desktop icons: open the selection.
    OpenIcon,
}

/// A built-in application the shell can open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum App {
    /// The system monitor.
    Activity,
    /// The environment switcher.
    Environments,
    /// System settings.
    Settings,
    /// The file manager.
    Files,
    /// Litric Browser.
    Browser,
    /// A terminal.
    Terminal,
    /// The trash.
    Trash,
    /// Litric Defender.
    Defender,
}

impl App {
    /// Every application, in launcher order.
    pub const ALL: [Self; 8] = [
        Self::Activity,
        Self::Environments,
        Self::Settings,
        Self::Files,
        Self::Browser,
        Self::Terminal,
        Self::Trash,
        Self::Defender,
    ];

    /// The name shown to the user.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Activity => "Activity",
            Self::Environments => "Environments",
            Self::Settings => "Settings",
            Self::Files => "Files",
            Self::Browser => "Litric Browser",
            Self::Terminal => "Terminal",
            Self::Trash => "Trash",
            Self::Defender => "Defender",
        }
    }

    /// The reverse-DNS identifier the window manager uses.
    #[must_use]
    pub const fn app_id(self) -> &'static str {
        match self {
            Self::Activity => "com.litric.activity",
            Self::Environments => "com.litric.environments",
            Self::Settings => "com.litric.settings",
            Self::Files => "com.litric.files",
            Self::Browser => "com.litric.browser",
            Self::Terminal => "com.litric.terminal",
            Self::Trash => "com.litric.trash",
            Self::Defender => "com.litric.defender",
        }
    }

    /// The application a window's `app_id` names, if this shell has it.
    ///
    /// The window manager stores the reverse-DNS identifier, so naming the
    /// application that has focus means turning that identifier back into the
    /// name the user knows. A window whose `app_id` is not one of ours -- a
    /// future component, a test fixture -- is not an error: the caller falls
    /// back to the window's own title rather than inventing a name.
    #[must_use]
    pub fn from_app_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.app_id() == id)
    }

    /// A one-line description for the launcher.
    #[must_use]
    pub const fn subtitle(self) -> &'static str {
        match self {
            Self::Activity => "CPU, memory and system state",
            Self::Environments => "Windows, Linux and LitricOS",
            Self::Settings => "Appearance, input, power, region",
            Self::Files => "Browse and manage files",
            Self::Browser => "The web, on your terms",
            Self::Terminal => "A command line",
            Self::Trash => "Deleted items",
            Self::Defender => "Scan for threats and malware",
        }
    }

    /// The category it appears under.
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::Activity | Self::Settings | Self::Terminal | Self::Trash => "System",
            Self::Environments => "Environments",
            Self::Defender => "Security",
            Self::Files | Self::Browser => "Applications",
        }
    }

    /// The size its window opens at.
    #[must_use]
    pub const fn default_size(self) -> (f32, f32) {
        match self {
            // Activity is a system dashboard, not a meter card. Give its
            // overview and process table enough room to coexist at the default
            // size on a 1280x800 display; the shell clips each section to its
            // own pane on smaller displays.
            Self::Activity => (620.0, 620.0),
            Self::Environments => (420.0, 300.0),
            Self::Settings => (720.0, 520.0),
            Self::Files => (760.0, 500.0),
            Self::Browser => (900.0, 600.0),
            Self::Terminal => (640.0, 400.0),
            Self::Trash => (560.0, 380.0),
            Self::Defender => (560.0, 420.0),
        }
    }

    /// Whether a launcher search matches it.
    ///
    /// Case-insensitive over the title, the subtitle and the identifier, so
    /// "act", "cpu" and "litric.act" all find Activity.
    #[must_use]
    pub fn matches(self, query: &str) -> bool {
        self.score(query).is_some()
    }

    /// How well this application matches a query. Higher is better.
    ///
    /// Ranking exists because a plain substring test is not enough: "bro"
    /// appears in Files' description ("Browse and manage files") as well as in
    /// Browser's name, and a launcher that offers the file manager when you
    /// type "bro" is a launcher people stop trusting.
    #[must_use]
    pub fn score(self, query: &str) -> Option<u32> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Some(0);
        }
        let title = self.title().to_lowercase();
        if title == q {
            return Some(100);
        }
        if title.starts_with(&q) {
            return Some(90);
        }
        // A word inside the title, e.g. "browser" in "Litric Browser".
        if title.split_whitespace().any(|w| w.starts_with(&q)) {
            return Some(80);
        }
        if title.contains(&q) {
            return Some(60);
        }
        if self.app_id().to_lowercase().contains(&q) {
            return Some(40);
        }
        if self.category().to_lowercase().starts_with(&q) {
            return Some(30);
        }
        if self.subtitle().to_lowercase().contains(&q) {
            return Some(20);
        }
        // Synonyms people actually type: "virus" should find the antivirus
        // even though no field of the app's listing says the word.
        let keywords = self.keywords();
        if keywords.iter().any(|k| k.starts_with(&q)) {
            return Some(15);
        }
        None
    }

    /// Search synonyms beyond the listing's fields.
    #[must_use]
    pub const fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Defender => &["virus", "malware", "antivirus", "scan"],
            Self::Files => &["explorer", "folder"],
            Self::Browser => &["web", "internet"],
            Self::Trash => &["bin", "deleted"],
            _ => &[],
        }
    }

    /// The applications matching a query, best first.
    ///
    /// Ties keep the order in [`Self::ALL`], so an empty query always shows the
    /// same list rather than one that reshuffles itself.
    #[must_use]
    pub fn search(query: &str) -> Vec<Self> {
        let mut scored: Vec<(u32, usize, Self)> = Self::ALL
            .iter()
            .enumerate()
            .filter_map(|(i, a)| a.score(query).map(|s| (s, i, *a)))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.into_iter().map(|(_, _, a)| a).collect()
    }
}

impl Action {
    /// A one-line description for the launcher and the settings application.
    #[must_use]
    pub fn describe(self) -> String {
        match self {
            Self::ToggleLauncher => "Open the launcher".to_owned(),
            Self::CloseOverlay => "Dismiss".to_owned(),
            Self::SwitchEnvironment(n) => format!("Switch to environment {n}"),
            Self::NextWindow => "Next window".to_owned(),
            Self::PrevWindow => "Previous window".to_owned(),
            Self::CloseWindow => "Close window".to_owned(),
            Self::MinimizeWindow => "Minimize window".to_owned(),
            Self::ToggleMaximize => "Maximize or restore".to_owned(),
            Self::SnapLeft => "Snap left".to_owned(),
            Self::SnapRight => "Snap right".to_owned(),
            Self::SnapTopLeft => "Snap to the top-left quarter".to_owned(),
            Self::SnapTopRight => "Snap to the top-right quarter".to_owned(),
            Self::SnapBottomLeft => "Snap to the bottom-left quarter".to_owned(),
            Self::SnapBottomRight => "Snap to the bottom-right quarter".to_owned(),
            Self::RestoreWindow => "Restore window size".to_owned(),
            Self::CycleAppWindow => "Cycle this application's windows".to_owned(),
            Self::ToggleOverview => "Show every window".to_owned(),
            Self::CenterWindow => "Centre window".to_owned(),
            Self::Workspace(n) => format!("Workspace {}", n.saturating_add(1)),
            Self::NextWorkspace => "Next workspace".to_owned(),
            Self::PrevWorkspace => "Previous workspace".to_owned(),
            Self::MoveToWorkspace(n) => {
                format!("Move window to workspace {}", n.saturating_add(1))
            }
            Self::ShowDesktop => "Show desktop".to_owned(),
            Self::Open(app) => format!("Open {}", app.title()),
            Self::ToggleQuickSettings => "Quick settings".to_owned(),
            Self::TogglePowerMenu => "Power menu".to_owned(),
            Self::ToggleLitroMenu => "LitricOS menu".to_owned(),
            Self::ToggleNotifications => "Notifications".to_owned(),
            Self::ToggleDiagnostics => "Diagnostics".to_owned(),
            Self::Lock => "Lock".to_owned(),
            Self::VolumeDown => "Volume down".to_owned(),
            Self::VolumeUp => "Volume up".to_owned(),
            Self::VolumeMute => "Mute".to_owned(),
            Self::BrightnessUp => "Brighter".to_owned(),
            Self::BrightnessDown => "Dimmer".to_owned(),
            Self::ReturnToHost => "Return to LitricOS".to_owned(),
            Self::FilesNewFolder => "New folder".to_owned(),
            Self::FilesNewFile => "New text file".to_owned(),
            Self::FilesRename => "Rename".to_owned(),
            Self::FilesCopy => "Copy".to_owned(),
            Self::FilesCut => "Cut".to_owned(),
            Self::FilesPaste => "Paste".to_owned(),
            Self::FilesTrash => "Move to trash".to_owned(),
            Self::FilesCompress => "Compress".to_owned(),
            Self::FilesExtract => "Extract here".to_owned(),
            Self::FilesSearch => "Search files".to_owned(),
            Self::FilesUp => "Up one folder".to_owned(),
            Self::OpenDefender => "Open Defender".to_owned(),
            Self::ScanNow => "Scan now".to_owned(),
            Self::BrowserAddress => "Address bar".to_owned(),
            Self::BrowserReload => "Reload".to_owned(),
            Self::ViewTrash => "Open Trash".to_owned(),
            Self::OpenIcon => "Open selection".to_owned(),
        }
    }
}

/// One key combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Binding {
    /// Which modifiers must be held. Lock state is ignored.
    pub mods: Modifiers,
    /// Which key.
    pub key: Key,
}

impl Binding {
    /// A binding with these modifiers.
    #[must_use]
    pub const fn new(mods: Modifiers, key: Key) -> Self {
        Self { mods, key }
    }

    /// Meta plus a key.
    #[must_use]
    pub const fn meta(key: Key) -> Self {
        Self::new(
            Modifiers {
                meta: true,
                ..Modifiers::NONE
            },
            key,
        )
    }

    /// Meta and Shift plus a key.
    #[must_use]
    pub const fn meta_shift(key: Key) -> Self {
        Self::new(
            Modifiers {
                meta: true,
                shift: true,
                ..Modifiers::NONE
            },
            key,
        )
    }

    /// Ctrl plus a key.
    #[must_use]
    pub const fn ctrl(key: Key) -> Self {
        Self::new(
            Modifiers {
                ctrl: true,
                ..Modifiers::NONE
            },
            key,
        )
    }

    /// Alt plus a key.
    #[must_use]
    pub const fn alt(key: Key) -> Self {
        Self::new(
            Modifiers {
                alt: true,
                ..Modifiers::NONE
            },
            key,
        )
    }

    /// Alt and Shift plus a key.
    #[must_use]
    pub const fn alt_shift(key: Key) -> Self {
        Self::new(
            Modifiers {
                alt: true,
                shift: true,
                ..Modifiers::NONE
            },
            key,
        )
    }

    /// A key with no modifiers.
    #[must_use]
    pub const fn bare(key: Key) -> Self {
        Self::new(Modifiers::NONE, key)
    }

    /// The label the user sees, e.g. `Win+1`, `Alt+Tab`.
    #[must_use]
    pub fn label(&self) -> String {
        format!("{}{}", self.mods.prefix(), self.key)
    }
}

/// One row of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    /// The keys.
    pub binding: Binding,
    /// What they do.
    pub action: Action,
    /// Whether the user may change it.
    pub rebindable: bool,
}

/// The shortcut table.
#[derive(Debug, Clone)]
pub struct Shortcuts {
    entries: Vec<Shortcut>,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self::system_defaults()
    }
}

impl Shortcuts {
    /// The shipped table.
    #[must_use]
    pub fn system_defaults() -> Self {
        let mut entries = Vec::new();
        let mut add = |binding: Binding, action: Action, rebindable: bool| {
            entries.push(Shortcut {
                binding,
                action,
                rebindable,
            });
        };

        // Environments. The signature shortcut of the whole system, so it is
        // first and it is not rebindable: a machine where Win+2 might not
        // return you to LitricOS is a machine you cannot trust.
        add(
            Binding::meta(Key::Char('1')),
            Action::SwitchEnvironment(1),
            false,
        );
        add(
            Binding::meta(Key::Char('2')),
            Action::SwitchEnvironment(2),
            false,
        );
        add(
            Binding::meta(Key::Char('3')),
            Action::SwitchEnvironment(3),
            false,
        );
        add(
            Binding::new(
                Modifiers {
                    ctrl: true,
                    alt: true,
                    ..Modifiers::NONE
                },
                Key::Home,
            ),
            Action::ReturnToHost,
            false,
        );

        // The launcher, on Meta alone. Handled specially by the resolver: a
        // modifier can only be a shortcut when it is pressed and released
        // without anything in between, or Meta+anything would open the
        // launcher first.
        add(
            Binding::bare(Key::Meta(litric_input::keys::Side::Left)),
            Action::ToggleLauncher,
            true,
        );

        // Windows.
        add(Binding::alt(Key::Tab), Action::NextWindow, true);
        add(Binding::alt_shift(Key::Tab), Action::PrevWindow, true);
        add(Binding::alt(Key::Function(4)), Action::CloseWindow, true);
        add(Binding::meta(Key::Char('q')), Action::CloseWindow, true);
        add(Binding::meta(Key::Down), Action::MinimizeWindow, true);
        add(Binding::meta(Key::Up), Action::ToggleMaximize, true);
        add(Binding::meta(Key::Left), Action::SnapLeft, true);
        add(Binding::meta(Key::Right), Action::SnapRight, true);
        // Quarters: the same two arrows with Ctrl, so the halves stay one
        // modifier press away and the quarters are the deliberate larger step.
        // Shift, not Ctrl: Win+Ctrl+arrows are the workspace cycle, and a
        // shortcut table may not bind one combination to two meanings.
        add(Binding::meta_shift(Key::Left), Action::SnapTopLeft, true);
        add(Binding::meta_shift(Key::Right), Action::SnapTopRight, true);
        add(Binding::meta_shift(Key::Down), Action::SnapBottomLeft, true);
        add(Binding::meta_shift(Key::Up), Action::SnapBottomRight, true);
        add(Binding::meta(Key::Char('r')), Action::RestoreWindow, true);
        // The window-level cycle. Alt+Tab walks every window; this walks the
        // ones belonging to the application the user is already in.
        add(Binding::meta(Key::Char('`')), Action::CycleAppWindow, true);
        // Mission Control. Meta+Tab rather than Alt+Tab, which is the switcher.
        add(Binding::meta(Key::Tab), Action::ToggleOverview, true);
        add(Binding::meta(Key::Char('c')), Action::CenterWindow, true);
        add(Binding::meta(Key::Char('d')), Action::ShowDesktop, true);

        // Workspaces. Meta+Shift+N moves the window; Meta+Ctrl+N switches.
        for n in 0..4u8 {
            let digit = char::from_digit(u32::from(n).saturating_add(1), 10).unwrap_or('1');
            add(
                Binding::new(
                    Modifiers {
                        meta: true,
                        ctrl: true,
                        ..Modifiers::NONE
                    },
                    Key::Char(digit),
                ),
                Action::Workspace(n),
                true,
            );
            add(
                Binding::meta_shift(Key::Char(digit)),
                Action::MoveToWorkspace(n),
                true,
            );
        }
        add(
            Binding::new(
                Modifiers {
                    meta: true,
                    ctrl: true,
                    ..Modifiers::NONE
                },
                Key::Right,
            ),
            Action::NextWorkspace,
            true,
        );
        add(
            Binding::new(
                Modifiers {
                    meta: true,
                    ctrl: true,
                    ..Modifiers::NONE
                },
                Key::Left,
            ),
            Action::PrevWorkspace,
            true,
        );

        // Applications.
        add(
            Binding::meta(Key::Char('e')),
            Action::Open(App::Files),
            true,
        );
        add(
            Binding::meta(Key::Char('b')),
            Action::Open(App::Browser),
            true,
        );
        add(
            Binding::meta(Key::Char('t')),
            Action::Open(App::Terminal),
            true,
        );
        add(
            Binding::meta(Key::Char('i')),
            Action::Open(App::Settings),
            true,
        );
        add(
            Binding::meta(Key::Char('a')),
            Action::Open(App::Activity),
            true,
        );
        add(
            Binding::meta_shift(Key::Char('v')),
            Action::OpenDefender,
            true,
        );
        add(Binding::meta(Key::Char('g')), Action::ScanNow, true);

        // Panels.
        add(
            Binding::meta(Key::Char('n')),
            Action::ToggleNotifications,
            true,
        );
        add(
            Binding::meta(Key::Char('s')),
            Action::ToggleQuickSettings,
            true,
        );
        add(Binding::meta(Key::Char('l')), Action::Lock, true);
        add(
            Binding::meta_shift(Key::Char('d')),
            Action::ToggleDiagnostics,
            true,
        );
        add(Binding::bare(Key::Escape), Action::CloseOverlay, false);

        // Media keys, which are bare keys on their own and belong to the system
        // wherever the focus is.
        add(Binding::bare(Key::VolumeUp), Action::VolumeUp, false);
        add(Binding::bare(Key::VolumeDown), Action::VolumeDown, false);
        add(Binding::bare(Key::Mute), Action::VolumeMute, false);
        add(
            Binding::bare(Key::BrightnessUp),
            Action::BrightnessUp,
            false,
        );
        add(
            Binding::bare(Key::BrightnessDown),
            Action::BrightnessDown,
            false,
        );

        Self { entries }
    }

    /// Every shortcut, for the settings application and the launcher.
    #[must_use]
    pub fn all(&self) -> &[Shortcut] {
        &self.entries
    }

    /// The label for an action, if it has a binding.
    #[must_use]
    pub fn label_for(&self, action: Action) -> Option<String> {
        self.entries
            .iter()
            .find(|s| s.action == action)
            .map(|s| s.binding.label())
    }

    /// Resolve a key press.
    ///
    /// A bare modifier never resolves here: Meta on its own is the launcher,
    /// and deciding that on key *down* would open the launcher every time
    /// somebody started to type Meta+E. The shell watches for the release
    /// instead, through [`MetaTap`].
    #[must_use]
    pub fn resolve(&self, key: Key, mods: Modifiers) -> Option<Action> {
        if key.is_modifier() {
            return None;
        }
        let want = mods.normalised();
        self.entries
            .iter()
            .find(|s| {
                !s.binding.key.is_modifier() && s.binding.key == key && s.binding.mods == want
            })
            .map(|s| s.action)
    }

    /// Rebind an action. Refuses to touch a reserved shortcut.
    ///
    /// # Errors
    /// Returns the conflicting action when the binding is already taken, so
    /// the settings application can say which one rather than silently
    /// shadowing it.
    pub fn rebind(&mut self, action: Action, binding: Binding) -> Result<(), RebindError> {
        if let Some(existing) = self.entries.iter().find(|s| s.action == action) {
            if !existing.rebindable {
                return Err(RebindError::Reserved);
            }
        } else {
            return Err(RebindError::NoSuchAction);
        }
        if let Some(clash) = self
            .entries
            .iter()
            .find(|s| s.binding == binding && s.action != action)
        {
            return Err(RebindError::Taken(clash.action));
        }
        for s in &mut self.entries {
            if s.action == action {
                s.binding = binding;
            }
        }
        Ok(())
    }
}

/// Why a rebind was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RebindError {
    /// The shortcut is reserved and may not be changed.
    Reserved,
    /// There is no such action in the table.
    NoSuchAction,
    /// Another action already uses that combination.
    Taken(Action),
}

/// Watches for Meta being pressed and released on its own.
///
/// Every desktop that puts a menu on the Windows key has to solve this: the key
/// is both a modifier and a shortcut. Acting on the press makes `Win+E` open
/// the launcher and then the file manager; acting on the release only when
/// nothing happened in between is what users actually expect.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetaTap {
    armed: bool,
    contaminated: bool,
}

impl MetaTap {
    /// Meta went down.
    pub fn press(&mut self) {
        self.armed = true;
        self.contaminated = false;
    }

    /// Some other key went down while Meta was held.
    pub fn interfere(&mut self) {
        if self.armed {
            self.contaminated = true;
        }
    }

    /// Meta came up. Returns whether this was a clean tap.
    pub fn release(&mut self) -> bool {
        let clean = self.armed && !self.contaminated;
        self.armed = false;
        self.contaminated = false;
        clean
    }

    /// Forget any press in progress, e.g. because focus was lost.
    pub fn reset(&mut self) {
        self.armed = false;
        self.contaminated = false;
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use litric_input::keys::Side;

    fn meta() -> Modifiers {
        Modifiers {
            meta: true,
            ..Modifiers::NONE
        }
    }

    #[test]
    fn the_environment_shortcuts_resolve() {
        let s = Shortcuts::default();
        assert_eq!(
            s.resolve(Key::Char('1'), meta()),
            Some(Action::SwitchEnvironment(1))
        );
        assert_eq!(
            s.resolve(Key::Char('2'), meta()),
            Some(Action::SwitchEnvironment(2))
        );
        assert_eq!(
            s.resolve(Key::Char('3'), meta()),
            Some(Action::SwitchEnvironment(3))
        );
    }

    #[test]
    fn lock_state_does_not_change_a_shortcut() {
        // Caps lock on must not stop Win+1 working.
        let s = Shortcuts::default();
        let with_caps = Modifiers {
            meta: true,
            caps: true,
            num: true,
            ..Modifiers::NONE
        };
        assert_eq!(
            s.resolve(Key::Char('1'), with_caps),
            Some(Action::SwitchEnvironment(1))
        );
    }

    #[test]
    fn an_extra_modifier_is_a_different_shortcut() {
        let s = Shortcuts::default();
        let meta_shift = Modifiers {
            meta: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(
            s.resolve(Key::Char('1'), meta_shift),
            Some(Action::MoveToWorkspace(0)),
            "Win+Shift+1 must not be Win+1"
        );
    }

    #[test]
    fn alt_tab_and_alt_shift_tab_go_opposite_ways() {
        let s = Shortcuts::default();
        let alt = Modifiers {
            alt: true,
            ..Modifiers::NONE
        };
        let alt_shift = Modifiers {
            alt: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(s.resolve(Key::Tab, alt), Some(Action::NextWindow));
        assert_eq!(s.resolve(Key::Tab, alt_shift), Some(Action::PrevWindow));
    }

    #[test]
    fn a_plain_key_is_not_a_shortcut() {
        let s = Shortcuts::default();
        assert_eq!(s.resolve(Key::Char('a'), Modifiers::NONE), None);
        assert_eq!(s.resolve(Key::Char('1'), Modifiers::NONE), None);
    }

    #[test]
    fn escape_dismisses_and_is_reserved() {
        let s = Shortcuts::default();
        assert_eq!(
            s.resolve(Key::Escape, Modifiers::NONE),
            Some(Action::CloseOverlay)
        );
        assert!(s
            .all()
            .iter()
            .any(|e| e.action == Action::CloseOverlay && !e.rebindable));
    }

    #[test]
    fn a_modifier_alone_never_resolves_on_press() {
        // Otherwise Win+E opens the launcher and then the file manager.
        let s = Shortcuts::default();
        assert_eq!(s.resolve(Key::Meta(Side::Left), meta()), None);
        assert_eq!(s.resolve(Key::Shift(Side::Left), Modifiers::NONE), None);
    }

    #[test]
    fn a_clean_meta_tap_opens_the_launcher_and_a_dirty_one_does_not() {
        let mut t = MetaTap::default();
        t.press();
        assert!(t.release(), "a clean tap should open the launcher");

        t.press();
        t.interfere();
        assert!(!t.release(), "Win+E must not also open the launcher");

        assert!(!t.release(), "a release with no press is not a tap");
    }

    #[test]
    fn resetting_a_meta_tap_forgets_it() {
        let mut t = MetaTap::default();
        t.press();
        t.reset();
        assert!(!t.release());
    }

    #[test]
    fn every_shortcut_has_a_readable_label() {
        let s = Shortcuts::default();
        assert_eq!(
            s.label_for(Action::SwitchEnvironment(1)).as_deref(),
            Some("Win+1")
        );
        assert_eq!(s.label_for(Action::NextWindow).as_deref(), Some("Alt+Tab"));
        assert_eq!(
            s.label_for(Action::Open(App::Browser)).as_deref(),
            Some("Win+B")
        );
        for e in s.all() {
            assert!(!e.binding.label().is_empty(), "{:?} has no label", e.action);
            assert!(!e.action.describe().is_empty());
        }
    }

    #[test]
    fn no_two_shortcuts_share_a_binding() {
        // A duplicate makes one of them dead, and which one depends on table
        // order, which is the worst kind of bug to find by hand.
        let s = Shortcuts::default();
        let mut seen: Vec<Binding> = Vec::new();
        for e in s.all() {
            assert!(
                !seen.contains(&e.binding),
                "{} is bound twice, second to {:?}",
                e.binding.label(),
                e.action
            );
            seen.push(e.binding);
        }
    }

    #[test]
    fn rebinding_works_and_refuses_a_collision() {
        let mut s = Shortcuts::default();
        let free = Binding::meta(Key::Char('z'));
        assert_eq!(s.rebind(Action::Open(App::Terminal), free), Ok(()));
        assert_eq!(
            s.resolve(Key::Char('z'), meta()),
            Some(Action::Open(App::Terminal))
        );
        assert_eq!(
            s.resolve(Key::Char('t'), meta()),
            None,
            "old binding is gone"
        );

        let taken = Binding::meta(Key::Char('e'));
        assert_eq!(
            s.rebind(Action::Open(App::Browser), taken),
            Err(RebindError::Taken(Action::Open(App::Files)))
        );
    }

    #[test]
    fn a_reserved_shortcut_cannot_be_rebound() {
        // Win+2 must always come back to LitricOS, whatever the user has done
        // to the rest of the table.
        let mut s = Shortcuts::default();
        assert_eq!(
            s.rebind(Action::SwitchEnvironment(2), Binding::meta(Key::Char('9'))),
            Err(RebindError::Reserved)
        );
        assert_eq!(
            s.rebind(Action::ReturnToHost, Binding::meta(Key::Char('0'))),
            Err(RebindError::Reserved)
        );
        assert_eq!(
            s.resolve(Key::Char('2'), meta()),
            Some(Action::SwitchEnvironment(2))
        );
    }

    #[test]
    fn rebinding_something_that_is_not_in_the_table_is_refused() {
        let mut s = Shortcuts::default();
        assert_eq!(
            s.rebind(Action::Workspace(9), Binding::meta(Key::Char('9'))),
            Err(RebindError::NoSuchAction)
        );
    }

    #[test]
    fn search_ranks_a_name_match_above_a_description_match() {
        // "bro" is in Browser's name and in Files' description. The name wins,
        // or typing "bro" and pressing Enter opens the file manager.
        let results = App::search("bro");
        assert_eq!(results.first().copied(), Some(App::Browser), "{results:?}");
        assert!(
            results.contains(&App::Files),
            "the weaker match is still offered"
        );
    }

    #[test]
    fn the_defender_is_findable_and_bound() {
        let s = Shortcuts::default();
        assert_eq!(
            s.label_for(Action::OpenDefender).as_deref(),
            Some("Win+Shift+V")
        );
        assert_eq!(s.label_for(Action::ScanNow).as_deref(), Some("Win+G"));
        let results = App::search("defender");
        assert_eq!(results.first().copied(), Some(App::Defender));
        let results = App::search("virus");
        assert_eq!(results.first().copied(), Some(App::Defender));
    }

    #[test]
    fn an_empty_search_lists_everything_in_a_stable_order() {
        assert_eq!(App::search("").len(), App::ALL.len());
        assert_eq!(App::search("  "), App::search(""));
        assert_eq!(App::search("").first().copied(), App::ALL.first().copied());
    }

    #[test]
    fn a_search_that_matches_nothing_returns_nothing() {
        assert!(App::search("spreadsheet").is_empty());
    }

    #[test]
    fn an_exact_name_beats_everything() {
        assert_eq!(
            App::search("settings").first().copied(),
            Some(App::Settings)
        );
        assert_eq!(App::search("Trash").first().copied(), Some(App::Trash));
    }

    #[test]
    fn launcher_search_finds_applications_by_more_than_their_name() {
        assert!(App::Activity.matches(""), "an empty query shows everything");
        assert!(App::Activity.matches("act"));
        assert!(App::Activity.matches("CPU"), "matched on the description");
        assert!(App::Browser.matches("litric.browser"), "matched on the id");
        assert!(!App::Browser.matches("spreadsheet"));
    }

    #[test]
    fn every_application_has_a_distinct_identity_and_a_usable_window_size() {
        let mut ids: Vec<&str> = App::ALL.iter().map(|a| a.app_id()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "two applications share an id");
        for a in App::ALL {
            let (w, h) = a.default_size();
            assert!(w >= 320.0 && h >= 240.0, "{:?} opens too small", a);
        }
    }
}
