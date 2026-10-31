//! Typed settings state owned by the shell.

use litric_design::{Accent, ThemeKind};

/// When the shell follows the clock instead of the chosen theme.
///
/// Both reference desktops that the research corpus catalogues offer this as a
/// third value beside light and dark (`MD.11` Appearance › Auto, `WB.04.03`),
/// and it is the one appearance behaviour that a desk-bound user notices
/// without opening Settings: the screen dims in the evening on its own.
///
/// It is a separate key rather than a fourth [`ThemeKind`] because it answers a
/// different question. `theme` is *which* look, `theme_schedule` is *when* — and
/// the user who turns the schedule off must get the look they chose back, not
/// the one the clock last picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeSchedule {
    /// Use the chosen theme at every hour. The default: a desktop that changes
    /// appearance overnight without being asked is a surprise, not a feature.
    #[default]
    Manual,
    /// Dark from 19:00 to 07:00 local time, light otherwise.
    Auto,
}

/// The hour the automatic schedule turns dark, local time, inclusive.
pub const DARK_FROM_HOUR: u8 = 19;

/// The hour the automatic schedule turns light again, local time, inclusive.
pub const LIGHT_FROM_HOUR: u8 = 7;

impl ThemeSchedule {
    /// Both values, in the order the settings row steps through them.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Manual, Self::Auto]
    }

    /// Stable token, written to the preference file.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
        }
    }

    /// Parse a stored token; `None` for anything this build does not know, so a
    /// file from a newer build keeps the caller's default.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        Self::all().iter().copied().find(|s| s.token() == raw)
    }

    /// The name shown on the Appearance row.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::Auto => "Automatic",
        }
    }

    /// The theme to paint at `hour` local time.
    ///
    /// High-contrast themes are left alone. Auto picks between dark and light,
    /// and a user who chose a high-contrast theme chose contrast over
    /// appearance; silently swapping it for a standard theme at 19:00 would
    /// remove the accessibility setting they came for.
    #[must_use]
    pub const fn effective(self, chosen: ThemeKind, hour: u8) -> ThemeKind {
        match self {
            Self::Manual => chosen,
            Self::Auto => {
                if chosen.requires_aaa() {
                    chosen
                } else if hour >= DARK_FROM_HOUR || hour < LIGHT_FROM_HOUR {
                    ThemeKind::Dark
                } else {
                    ThemeKind::Light
                }
            }
        }
    }
}

/// How much larger the interface is drawn, as a step rather than a number.
///
/// Three steps, and the store keeps the step rather than the multiplier: a
/// float written by one build would be read by the next as a size nobody
/// measured, while a step name is either known or ignored. The multiplier is a
/// property of the step, so there is one place to change it and one place to
/// test it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiScale {
    /// The size the display asks for.
    #[default]
    Normal,
    /// A quarter larger.
    Large,
    /// Half again as large.
    Larger,
}

impl UiScale {
    /// Every step, smallest first — the order the Display row steps through.
    pub const ALL: [Self; 3] = [Self::Normal, Self::Large, Self::Larger];

    /// What this step multiplies the display's own scale by.
    #[must_use]
    pub const fn factor(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Large => 1.25,
            Self::Larger => 1.5,
        }
    }

    /// Stable token, written to the preference file.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Normal => "1",
            Self::Large => "1.25",
            Self::Larger => "1.5",
        }
    }

    /// Parse a stored token. A bare multiplier from a hand-edited file is
    /// accepted when it names a step exactly; anything else keeps the default.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if let Some(step) = Self::ALL.iter().copied().find(|step| step.token() == raw) {
            return Some(step);
        }
        raw.parse::<f32>().ok().and_then(Self::from_factor)
    }

    /// The step a multiplier names, or `None` for a size this build does not
    /// offer.
    #[must_use]
    pub fn from_factor(factor: f32) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|step| (step.factor() - factor).abs() < 0.001)
    }

    /// The name shown on the Display row.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Normal => "100%",
            Self::Large => "125%",
            Self::Larger => "150%",
        }
    }
}

use crate::apps::SearchEngine;
use crate::desktop::Wallpaper;
use crate::i18n::Language;

/// Preferences that the shell can apply itself and persist safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences {
    /// Built-in colour theme.
    pub theme: ThemeKind,
    /// Accent colour applied on top of `theme`. High-contrast themes keep their
    /// own accent, which is why this is a separate choice rather than a theme.
    pub accent: Accent,
    /// Wallpaper the desktop paints.
    pub wallpaper: Wallpaper,
    /// Shell interface language.
    pub language: Language,
    /// Whether shell animation is reduced.
    pub reduce_motion: bool,
    /// Whether glass surfaces are drawn opaque instead of sampling the
    /// wallpaper. A readability preference, applied by the theme itself.
    pub reduce_transparency: bool,
    /// Whether notifications may raise a banner.
    ///
    /// Do Not Disturb is a display-time decision, not a queue filter: a note
    /// raised while the desktop is quiet is still recorded in the centre. Only
    /// critical alerts keep their banner, because a switch that also hides
    /// "your disk is full" is a switch that loses data.
    pub do_not_disturb: bool,
    /// Whether the Files table lists dot-prefixed entries.
    ///
    /// The file service has always been able to return them (`LIST path
    /// hidden=1`); this is the choice that decides whether the shell asks. Off
    /// by default, matching Explorer and every other file manager: hidden
    /// entries are hidden until the user asks to see them.
    pub show_hidden: bool,
    /// How much larger the interface is drawn. See [`UiScale`].
    pub ui_scale: UiScale,
    /// Whether first-run setup has finished. Until it has, the desktop shows the
    /// setup screen instead of the desktop.
    pub onboarded: bool,
    /// Whether `theme` follows the clock. See [`ThemeSchedule`].
    pub theme_schedule: ThemeSchedule,
    /// The engine the omnibox searches with.
    ///
    /// Here rather than in `browser-data.conf` because it is a preference, not
    /// browsing history: it must survive the "clear history" that the browser
    /// data file is allowed to lose, and it belongs with the other choices the
    /// Settings window owns.
    pub search_engine: SearchEngine,
}

impl Preferences {
    /// Defaults for a finished setup. The loader clears `onboarded` when the
    /// account has no preference file yet.
    #[must_use]
    pub const fn new(theme: ThemeKind, language: Language) -> Self {
        Self {
            theme,
            accent: Accent::Default,
            wallpaper: Wallpaper::System,
            language,
            reduce_motion: false,
            reduce_transparency: false,
            do_not_disturb: false,
            show_hidden: false,
            ui_scale: UiScale::Normal,
            onboarded: true,
            theme_schedule: ThemeSchedule::Manual,
            search_engine: SearchEngine::Google,
        }
    }
}

/// A Settings page. The numeric value is the row index in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    /// Theme, wallpaper, and shell language.
    Appearance,
    /// Output resolution and framebuffer facts.
    #[default]
    Display,
    /// Keyboard and pointer facts.
    Input,
    /// Network facts exposed by the system.
    Network,
    /// Bluetooth radio facts: whether a radio exists, what blocks it, and
    /// whether anything in this image could pair with it.
    Bluetooth,
    /// Power facts and supported requests.
    Power,
    /// Region, date, and interface language.
    Region,
    /// Available execution environments and their live state.
    Environments,
    /// Motion and contrast preferences.
    Accessibility,
    /// Notification banners and Do Not Disturb.
    Notifications,
    /// Audio devices and actual service state.
    Sound,
    /// Privacy and browser permission policy.
    Privacy,
    /// Session account and lock state.
    Accounts,
    /// Shell and daemon diagnostic facts.
    Diagnostics,
    /// Update generations, health evidence and the daemon's status.
    Updates,
    /// AppArmor, Polkit, firewalld, Flatpak/App Store and NetworkManager.
    Services,
}

impl Page {
    /// Pages shown by Settings in sidebar order.
    pub const ALL: [Self; 16] = [
        Self::Appearance,
        Self::Display,
        Self::Input,
        Self::Network,
        Self::Bluetooth,
        Self::Power,
        Self::Region,
        Self::Environments,
        Self::Accessibility,
        Self::Notifications,
        Self::Sound,
        Self::Privacy,
        Self::Accounts,
        Self::Diagnostics,
        Self::Updates,
        Self::Services,
    ];

    /// Stable user-facing English title (translated by the text catalog).
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Display => "Display",
            Self::Input => "Input",
            Self::Network => "Network",
            Self::Bluetooth => "Bluetooth",
            Self::Power => "Power",
            Self::Region => "Region",
            Self::Environments => "Environments",
            Self::Accessibility => "Accessibility",
            Self::Notifications => "Notifications",
            Self::Sound => "Sound",
            Self::Privacy => "Privacy",
            Self::Accounts => "Accounts",
            Self::Diagnostics => "Diagnostics",
            Self::Updates => "Updates",
            Self::Services => "Services",
        }
    }

    /// Resolve a sidebar index without indexing past the page table.
    #[must_use]
    pub fn from_index(index: u8) -> Option<Self> {
        Self::ALL.get(usize::from(index)).copied()
    }

    /// The lower-case name this page answers to in a `litric://settings/` link.
    ///
    /// A path segment, so it is lower case, hyphen-free for the single-word
    /// pages that exist today, and stable: changing one breaks every link a
    /// person has written down or bookmarked.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Display => "display",
            Self::Input => "input",
            Self::Network => "network",
            Self::Bluetooth => "bluetooth",
            Self::Power => "power",
            Self::Region => "region",
            Self::Environments => "environments",
            Self::Accessibility => "accessibility",
            Self::Notifications => "notifications",
            Self::Sound => "sound",
            Self::Privacy => "privacy",
            Self::Accounts => "accounts",
            Self::Diagnostics => "diagnostics",
            Self::Updates => "updates",
            Self::Services => "services",
        }
    }

    /// The page a link names, or `None` when it names no page this build has.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        let slug = slug.trim();
        Self::ALL
            .iter()
            .copied()
            .find(|page| page.slug().eq_ignore_ascii_case(slug))
    }
}

/// Result state shown while applying a shell-owned preference.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SaveState {
    /// No edit has happened yet.
    #[default]
    Idle,
    /// The current value has been applied and is being written atomically.
    Saving,
    /// The current value was written successfully.
    Saved,
    /// Persistence failed and the previous value was restored.
    Failed(String),
}

/// Complete Settings model, including rollback state.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Current shell-owned preferences.
    pub preferences: Preferences,
    /// Page being displayed.
    pub page: Page,
    /// Last apply/persist state.
    pub save: SaveState,
    /// The store key a deep link asked to highlight, until the user moves.
    ///
    /// Not persisted: it is where a link pointed, not a preference. Held as the
    /// key rather than the row so nothing here borrows from the registry's
    /// static table and nothing can outlive it.
    pub focus: Option<&'static str>,
    /// How far the page list is scrolled, in device pixels.
    ///
    /// Fifteen 30 px pages are 450 px of rows; a window shorter than that had
    /// no way to reach the last ones. The view clamps this against the pane's
    /// real height, which is the only place the height is known.
    pub sidebar_scroll: f32,
    previous: Option<Preferences>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new(ThemeKind::Dark, Language::Thai)
    }
}

impl Model {
    /// Scroll the page list by `dy` wheel notches.
    ///
    /// The view clamps the result: only it knows the pane's height and how
    /// many rows fit, and a model that guessed would keep an offset that no
    /// longer describes a scrollable list.
    pub fn scroll_sidebar(&mut self, dy: f32) {
        self.sidebar_scroll += dy * 44.0;
        if self.sidebar_scroll < 0.0 {
            self.sidebar_scroll = 0.0;
        }
    }

    /// Construct from an already-loaded preference record.
    #[must_use]
    pub fn new(preferences: Preferences) -> Self {
        Self {
            preferences,
            page: Page::Display,
            save: SaveState::Idle,
            focus: None,
            sidebar_scroll: 0.0,
            previous: None,
        }
    }

    /// Start an update, returning `None` when it is already current or a save
    /// is still pending.
    pub fn begin(&mut self, next: Preferences) -> Option<Preferences> {
        if self.previous.is_some() || self.preferences == next {
            return None;
        }
        self.previous = Some(self.preferences);
        self.preferences = next;
        self.save = SaveState::Saving;
        Some(next)
    }

    /// Mark the preference write as complete.
    pub fn finish(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => self.save = SaveState::Saved,
            Err(error) => {
                if let Some(previous) = self.previous {
                    self.preferences = previous;
                }
                self.save = SaveState::Failed(error);
            }
        }
        self.previous = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_persistence_restores_the_previous_values() {
        let mut model = Model::new(Preferences::new(ThemeKind::Dark, Language::Thai));
        let next = Preferences::new(ThemeKind::Light, Language::English);
        assert_eq!(model.begin(next), Some(next));
        model.finish(Err("read-only".to_owned()));
        assert_eq!(
            model.preferences,
            Preferences::new(ThemeKind::Dark, Language::Thai)
        );
        assert!(matches!(model.save, SaveState::Failed(_)));
    }

    #[test]
    fn the_schedule_switches_at_the_hours_the_row_promises() {
        use ThemeKind;
        // The boundaries are inclusive at the start and exclusive at the end, so
        // 19:00 is dark and 07:00 is light. An off-by-one here is a screen that
        // turns over an hour late, which nobody would notice and nobody could
        // explain.
        for (hour, expected) in [
            (LIGHT_FROM_HOUR, ThemeKind::Light),
            (12, ThemeKind::Light),
            (DARK_FROM_HOUR.saturating_sub(1), ThemeKind::Light),
            (DARK_FROM_HOUR, ThemeKind::Dark),
            (23, ThemeKind::Dark),
            (0, ThemeKind::Dark),
            (LIGHT_FROM_HOUR.saturating_sub(1), ThemeKind::Dark),
        ] {
            assert_eq!(
                ThemeSchedule::Auto.effective(ThemeKind::Light, hour),
                expected,
                "hour {hour}"
            );
        }
    }

    #[test]
    fn manual_keeps_the_chosen_theme_at_every_hour() {
        for hour in 0..24u8 {
            assert_eq!(
                ThemeSchedule::Manual.effective(ThemeKind::Light, hour),
                ThemeKind::Light
            );
            assert_eq!(
                ThemeSchedule::Manual.effective(ThemeKind::Dark, hour),
                ThemeKind::Dark
            );
        }
    }

    #[test]
    fn the_schedule_leaves_a_high_contrast_theme_alone() {
        // Auto picks dark or light. A user who chose a contrast theme chose
        // contrast, and swapping it for a standard theme at 19:00 would take
        // their accessibility setting away without asking.
        for hour in [0u8, 8, 12, 19, 23] {
            for theme in [ThemeKind::HighContrastDark, ThemeKind::HighContrastLight] {
                assert_eq!(ThemeSchedule::Auto.effective(theme, hour), theme);
            }
        }
    }

    #[test]
    fn the_schedule_tokens_round_trip_and_unknown_ones_are_refused() {
        for schedule in ThemeSchedule::all() {
            assert_eq!(ThemeSchedule::parse(schedule.token()), Some(*schedule));
        }
        assert_eq!(ThemeSchedule::parse("sometimes"), None);
        assert_eq!(ThemeSchedule::parse(""), None);
        assert_eq!(ThemeSchedule::parse("  auto  "), Some(ThemeSchedule::Auto));
    }

    #[test]
    fn a_page_slug_is_the_page_whatever_its_case() {
        for page in Page::ALL {
            assert_eq!(Page::from_slug(page.slug()), Some(page));
            assert_eq!(Page::from_slug(&page.slug().to_uppercase()), Some(page));
        }
        assert_eq!(Page::from_slug("nonsense"), None);
    }
}
