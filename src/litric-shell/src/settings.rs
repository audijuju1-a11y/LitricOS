//! Typed settings state owned by the shell.

use litric_design::ThemeKind;

use crate::i18n::Language;

/// Preferences that the shell can apply itself and persist safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences {
    /// Built-in colour theme.
    pub theme: ThemeKind,
    /// Shell interface language.
    pub language: Language,
    /// Whether shell animation is reduced.
    pub reduce_motion: bool,
    /// Whether first-run setup has finished. Until it has, the desktop shows the
    /// setup screen instead of the desktop.
    pub onboarded: bool,
}

impl Preferences {
    /// Defaults for a finished setup. The loader clears `onboarded` when the
    /// account has no preference file yet.
    #[must_use]
    pub const fn new(theme: ThemeKind, language: Language) -> Self {
        Self {
            theme,
            language,
            reduce_motion: false,
            onboarded: true,
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
    /// Audio devices and actual service state.
    Sound,
    /// Privacy and browser permission policy.
    Privacy,
    /// Session account and lock state.
    Accounts,
    /// Shell and daemon diagnostic facts.
    Diagnostics,
}

impl Page {
    /// Pages shown by Settings in sidebar order.
    pub const ALL: [Self; 13] = [
        Self::Appearance,
        Self::Display,
        Self::Input,
        Self::Network,
        Self::Bluetooth,
        Self::Power,
        Self::Region,
        Self::Environments,
        Self::Accessibility,
        Self::Sound,
        Self::Privacy,
        Self::Accounts,
        Self::Diagnostics,
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
            Self::Sound => "Sound",
            Self::Privacy => "Privacy",
            Self::Accounts => "Accounts",
            Self::Diagnostics => "Diagnostics",
        }
    }

    /// Resolve a sidebar index without indexing past the page table.
    #[must_use]
    pub fn from_index(index: u8) -> Option<Self> {
        Self::ALL.get(usize::from(index)).copied()
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
    previous: Option<Preferences>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new(ThemeKind::Dark, Language::Thai)
    }
}

impl Model {
    /// Construct from an already-loaded preference record.
    #[must_use]
    pub fn new(preferences: Preferences) -> Self {
        Self {
            preferences,
            page: Page::Display,
            save: SaveState::Idle,
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
}
