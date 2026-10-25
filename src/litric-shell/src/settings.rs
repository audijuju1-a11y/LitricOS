//! Typed settings state owned by the shell.

use litric_design::{Rgb, Theme, ThemeKind};

use crate::i18n::Language;

/// The accent colour a person picked, laid over the theme's own.
///
/// `System` is not a seventh colour: it is "whatever this theme's accent is",
/// which is how the shell drew every control before the preference existed, so a
/// machine that never opens Settings renders the same bytes it always did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Accent {
    /// The theme's accent.
    #[default]
    System,
    /// Blue.
    Blue,
    /// Graphite: the neutral accent macOS offers for people who want less colour.
    Graphite,
    /// Green.
    Green,
    /// Orange.
    Orange,
    /// Pink.
    Pink,
    /// Purple.
    Purple,
}

impl Accent {
    /// Every accent, in the order Settings offers them.
    pub const ALL: [Self; 7] = [
        Self::System,
        Self::Blue,
        Self::Graphite,
        Self::Green,
        Self::Orange,
        Self::Pink,
        Self::Purple,
    ];

    /// Stable token for the preference file.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Blue => "blue",
            Self::Graphite => "graphite",
            Self::Green => "green",
            Self::Orange => "orange",
            Self::Pink => "pink",
            Self::Purple => "purple",
        }
    }

    /// Read a token, ignoring case and surrounding space.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        Self::ALL
            .iter()
            .copied()
            .find(|accent| accent.token().eq_ignore_ascii_case(raw))
    }

    /// Resolve a control index, so a stale hotspot cannot pick a colour that
    /// does not exist.
    #[must_use]
    pub fn from_index(index: u8) -> Option<Self> {
        Self::ALL.get(usize::from(index)).copied()
    }

    /// The colour to draw, or `None` for the theme's own accent.
    ///
    /// These are the sRGB values macOS ships as system colours. They are
    /// deliberately brighter than a document accent: an accent is a *signal* on
    /// glass, and a muted one stops reading at 44% alpha.
    #[must_use]
    pub const fn rgb(self) -> Option<Rgb> {
        match self {
            Self::System => None,
            Self::Blue => Some(Rgb::hex(0x0A_84_FF)),
            Self::Graphite => Some(Rgb::hex(0x8E_8E_93)),
            Self::Green => Some(Rgb::hex(0x30_D1_58)),
            Self::Orange => Some(Rgb::hex(0xFF_9F_0A)),
            Self::Pink => Some(Rgb::hex(0xFF_37_5F)),
            Self::Purple => Some(Rgb::hex(0xBF_5A_F2)),
        }
    }

    /// What the swatch says under it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "Theme",
            Self::Blue => "Blue",
            Self::Graphite => "Graphite",
            Self::Green => "Green",
            Self::Orange => "Orange",
            Self::Pink => "Pink",
            Self::Purple => "Purple",
        }
    }
}

/// Preferences that the shell can apply itself and persist safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences {
    /// Built-in colour theme.
    pub theme: ThemeKind,
    /// Accent colour laid over the theme's own.
    pub accent: Accent,
    /// Whether outlines are darkened and secondary text pulled toward the body
    /// colour. The accessibility setting macOS calls "Increase contrast".
    pub contrast: bool,
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
            accent: Accent::System,
            contrast: false,
            language,
            reduce_motion: false,
            onboarded: true,
        }
    }

    /// The theme these preferences describe.
    ///
    /// The built-in theme with this account's accent laid over its accent and,
    /// when contrast is on, hairlines and secondary text pulled toward the body
    /// colour. Everything else is the theme's business, which is why this is the
    /// only place that knows how a preference becomes a colour: the shell, the
    /// first frame and the tests all ask here.
    #[must_use]
    pub fn theme(&self) -> Theme {
        let base = Theme::of(self.theme);
        let mut palette = base.palette;
        if let Some(accent) = self.accent.rgb() {
            palette.accent = accent;
            // Text on the accent has to survive the accent. The rule is not
            // "white on a coloured fill" -- that is 3.6:1 on the system blue and
            // 3.5:1 on the pink, both under the 4.5:1 this tree holds body text
            // to -- but the better of the two candidate text colours, so an
            // accent keeps its exact hue and the label on it stays readable.
            let light = Rgb::hex(0xFF_FF_FF);
            let dark = Rgb::hex(0x06_08_0C);
            palette.on_accent = if accent.contrast(light) >= accent.contrast(dark) {
                light
            } else {
                dark
            };
        }
        if self.contrast {
            // Two levers, both blends toward the theme's own body colour, so a
            // light theme gets darker outlines and a dark theme lighter ones.
            palette.border = palette.border.over(palette.text_primary, 0.45);
            palette.text_secondary = palette.text_secondary.over(palette.text_primary, 0.5);
        }
        Theme { palette, ..base }
    }
}

/// A Settings page. The numeric value is the row index in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    /// The first page: live cards for the handful of settings people change
    /// most, and the way back to the rest. Windows opens here (Part 2
    /// WB.00.F12); macOS does not, which is one of the few places this shell
    /// deliberately follows the Windows shape.
    Home,
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
    pub const ALL: [Self; 14] = [
        Self::Home,
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
            Self::Home => "Home",
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

/// The cards the home page offers, in draw order.
///
/// Four, not fourteen: a home page that lists everything is the sidebar again.
/// These are the settings a person opens this application to change -- the way
/// the interface looks, how it moves, the screen, the network -- and each card
/// carries its current value, so the page answers before it is clicked.
pub const HOME_CARDS: [Page; 4] = [
    Page::Appearance,
    Page::Accessibility,
    Page::Display,
    Page::Network,
];

impl Page {
    /// The labels on this page that a search should match.
    ///
    /// This is the page's own vocabulary, written here rather than scraped from
    /// the drawn rows, because a search that only knows what is on screen can
    /// only find what the user already found. Each entry is what a person would
    /// type: the setting's name, not its value.
    #[must_use]
    pub const fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Home => &["home", "overview", "recommended"],
            Self::Appearance => &[
                "theme",
                "dark",
                "light",
                "high contrast",
                "language",
                "thai",
                "english",
                "accent colour",
                "accent color",
                "colour",
                "color",
            ],
            Self::Display => &["display", "resolution", "screen", "ui scale", "framebuffer"],
            Self::Input => &["input", "keyboard", "pointer", "mouse", "layout", "devices"],
            Self::Network => &["network", "wifi", "wi-fi", "ethernet", "interface", "connection"],
            Self::Bluetooth => &["bluetooth", "radio", "adapter", "pairing"],
            Self::Power => &["power", "battery", "energy", "suspend", "restart"],
            Self::Region => &["region", "locale", "date", "time", "clock", "timezone"],
            Self::Environments => &["environment", "container", "vm", "runtime"],
            Self::Accessibility => &[
                "accessibility",
                "reduce motion",
                "motion",
                "animation",
                "contrast",
                "increase contrast",
            ],
            Self::Sound => &["sound", "audio", "volume", "output", "speaker"],
            Self::Privacy => &["privacy", "permission", "browser", "camera", "microphone"],
            Self::Accounts => &["account", "user", "password", "lock", "session"],
            Self::Diagnostics => &["diagnostics", "status", "facts", "renderer", "log"],
        }
    }
}

/// One search result: a setting, and the page it lives on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    /// The page that owns the setting.
    pub page: Page,
    /// What matched, in the page's own words.
    pub label: &'static str,
}

/// Settings whose name or page matches `query`, best first.
///
/// The order is the sidebar order, and within a page the order the page lists
/// its keywords, so the list does not reshuffle while a person types. A query
/// matching the page name returns that page's first keyword, which is what
/// makes "network" open Network rather than listing five rows of the same page.
#[must_use]
pub fn search(query: &str) -> Vec<Hit> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<Hit> = Vec::new();
    for page in Page::ALL.iter().copied() {
        let title = page.title().to_lowercase();
        let keywords = page.keywords();
        if title.contains(needle.as_str()) {
            if let Some(label) = keywords.first() {
                hits.push(Hit {
                    page,
                    label,
                });
                continue;
            }
        }
        for label in keywords {
            if label.to_lowercase().contains(needle.as_str()) {
                hits.push(Hit { page, label });
            }
        }
    }
    hits
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
    /// Pages visited, most recent last: what the back control walks.
    pub history: Vec<Page>,
    /// The search field's text. Drawn only while `search_focus` is set or the
    /// field is not empty, so a page with no search looks like a page.
    pub search: crate::textedit::TextEdit,
    /// Whether the search field has the keyboard.
    pub search_focus: bool,
    /// Which search result Enter would open.
    pub selected: usize,
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
            page: Page::Home,
            history: Vec::new(),
            search: crate::textedit::TextEdit::new(),
            search_focus: false,
            selected: 0,
            save: SaveState::Idle,
            previous: None,
        }
    }

    /// Go to a page, remembering where we came from.
    ///
    /// The history is capped: a session that has visited every page four times
    /// does not need to remember it, and an unbounded stack is a leak with a
    /// friendly name.
    pub fn go(&mut self, page: Page) {
        if page == self.page {
            return;
        }
        if self.history.len() >= Self::HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.history.push(self.page);
        self.page = page;
        self.search.clear();
        self.search_focus = false;
        self.selected = 0;
    }

    /// The page the back control returns to, if there is one.
    #[must_use]
    pub fn back_target(&self) -> Option<Page> {
        self.history.last().copied()
    }

    /// Walk back one page. Returns whether anything changed.
    pub fn back(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        self.page = previous;
        self.search.clear();
        self.search_focus = false;
        self.selected = 0;
        true
    }

    /// How many pages the back control can walk.
    pub const HISTORY_LIMIT: usize = 16;

    /// The results the current query produces.
    #[must_use]
    pub fn hits(&self) -> Vec<Hit> {
        search(self.search.text())
    }

    /// Move the selection within the current results, wrapping at both ends.
    pub fn select_next(&mut self, forward: bool) -> bool {
        let count = self.hits().len();
        if count == 0 {
            return false;
        }
        // No `%`: the modulo of a running index and a list length is the
        // arithmetic-overflow lint's favourite false positive, and a branch
        // says the same thing without asking anyone to prove it.
        let last = count.saturating_sub(1);
        let next = if forward {
            if self.selected >= last {
                0
            } else {
                self.selected.saturating_add(1)
            }
        } else if self.selected == 0 {
            last
        } else {
            self.selected.saturating_sub(1)
        };
        if next == self.selected {
            return false;
        }
        self.selected = next;
        true
    }

    /// The result Enter would open, if any.
    #[must_use]
    pub fn selected_hit(&self) -> Option<Hit> {
        self.hits().get(self.selected).copied()
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
    fn the_default_accent_leaves_the_theme_exactly_as_it_was() {
        // The preference is additive: a machine that never opens Settings must
        // render the same bytes as before it existed. `frame_dump`'s 24 scene
        // fingerprints are the check for the drawing; this is the check for the
        // theme.
        for kind in ThemeKind::all().iter().copied() {
            let prefs = Preferences::new(kind, Language::Thai);
            assert_eq!(prefs.accent, Accent::System);
            assert!(!prefs.contrast);
            assert_eq!(prefs.theme(), Theme::of(kind), "{kind:?} changed");
        }
    }

    #[test]
    fn an_accent_replaces_the_themes_accent_and_keeps_text_on_it_readable() {
        let mut prefs = Preferences::new(ThemeKind::Dark, Language::Thai);
        prefs.accent = Accent::Pink;
        let theme = prefs.theme();
        assert_eq!(theme.palette.accent, Rgb::hex(0xFF_37_5F));
        assert!(
            theme.palette.accent.contrast(theme.palette.on_accent) >= 4.5,
            "text on {} has {}:1",
            theme.palette.accent,
            theme.palette.accent.contrast(theme.palette.on_accent)
        );
        // Two accents are two pictures, not one.
        prefs.accent = Accent::Green;
        assert_ne!(prefs.theme().palette.accent, theme.palette.accent);
    }

    #[test]
    fn contrast_is_two_blends_toward_the_body_colour() {
        let mut prefs = Preferences::new(ThemeKind::Dark, Language::Thai);
        let before = prefs.theme().palette;
        prefs.contrast = true;
        let after = prefs.theme().palette;
        // A hairline that is easier to see against what it separates, and
        // secondary text nearer the body colour than the muted grey.
        assert!(
            after.border.contrast(after.bg_base) > before.border.contrast(before.bg_base),
            "the outline did not gain contrast"
        );
        assert!(
            after.text_secondary.contrast(after.bg_base)
                > before.text_secondary.contrast(before.bg_base),
            "secondary text did not gain contrast"
        );
        // And nothing else about the theme moved.
        assert_eq!(after.bg_base, before.bg_base);
        assert_eq!(after.accent, before.accent);
    }

    #[test]
    fn every_accent_round_trips_through_its_token() {
        for accent in Accent::ALL {
            assert_eq!(Accent::parse(accent.token()), Some(accent));
            assert_eq!(Accent::from_index(
                Accent::ALL
                    .iter()
                    .position(|a| *a == accent)
                    .and_then(|i| u8::try_from(i).ok())
                    .unwrap_or(255)
            ), Some(accent));
        }
        assert_eq!(Accent::parse("Blue"), Some(Accent::Blue));
        assert_eq!(Accent::parse("nonsense"), None);
    }

    #[test]
    fn search_finds_a_setting_by_its_name_and_its_page() {
        // A setting's own name finds it...
        let hits = search("contrast");
        assert!(
            hits.iter()
                .any(|hit| hit.page == Page::Accessibility && hit.label == "increase contrast"),
            "contrast did not find the switch: {hits:?}"
        );
        // ...and a page name finds the page once, not once per keyword.
        let hits = search("network");
        assert_eq!(hits.len(), 1, "network listed one page twice: {hits:?}");
        assert_eq!(hits.first().map(|hit| hit.page), Some(Page::Network));
        // A query with no matches is empty rather than a list of everything.
        assert!(search("zzzz").is_empty());
        // And nothing typed is nothing listed: the sidebar is still the sidebar.
        assert!(search("   ").is_empty());
        // Case is not the user's problem.
        assert!(!search("BLUETOOTH").is_empty());
    }

    #[test]
    fn every_page_can_be_found_by_its_own_title() {
        for page in Page::ALL.iter().copied() {
            let hits = search(page.title());
            assert!(
                hits.iter().any(|hit| hit.page == page),
                "{} is in the sidebar but not in the search",
                page.title()
            );
        }
    }

    #[test]
    fn the_back_control_walks_the_pages_that_were_visited() {
        let mut model = Model::new(Preferences::new(ThemeKind::Dark, Language::Thai));
        assert_eq!(model.page, Page::Home);
        assert_eq!(model.back_target(), None, "there is nowhere to go back to");
        model.go(Page::Appearance);
        model.go(Page::Accessibility);
        assert_eq!(model.back_target(), Some(Page::Appearance));
        assert!(model.back());
        assert_eq!(model.page, Page::Appearance);
        assert!(model.back());
        assert_eq!(model.page, Page::Home);
        assert!(!model.back(), "the first page is the end of the history");
        // Going where you already are is not a step.
        model.go(Page::Display);
        model.go(Page::Display);
        assert_eq!(model.history, vec![Page::Home]);
    }

    #[test]
    fn the_history_is_bounded() {
        let mut model = Model::new(Preferences::new(ThemeKind::Dark, Language::Thai));
        for _ in 0..(Model::HISTORY_LIMIT + 8) {
            model.go(Page::Display);
            model.go(Page::Home);
        }
        assert!(model.history.len() <= Model::HISTORY_LIMIT);
    }

    #[test]
    fn the_selection_wraps_at_both_ends_of_the_result_list() {
        let mut model = Model::new(Preferences::new(ThemeKind::Dark, Language::Thai));
        model.search.set_text("contrast");
        let count = model.hits().len();
        assert!(count >= 2, "fixture: two entries mention contrast");
        assert_eq!(model.selected, 0);
        assert!(model.select_next(true));
        assert_eq!(model.selected, 1);
        assert!(model.select_next(false));
        assert_eq!(model.selected, 0);
        // Up from the first row is the last row, not a dead key.
        assert!(model.select_next(false));
        assert_eq!(model.selected, count - 1);
        assert_eq!(model.selected_hit().map(|hit| hit.page), Some(Page::Accessibility));
    }

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
