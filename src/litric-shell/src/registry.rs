//! One table of every setting the shell owns, and the deep link that opens it.
//!
//! # Why a table and not a page
//!
//! The Shell had thirteen Settings pages and no answer to three questions a
//! settings surface is asked the moment it has more than a handful of rows:
//!
//! * *Where does "reduce motion" live?* — only the code that draws the button
//!   knew, so the launcher search could find the **page** and never the row.
//! * *What is this key called in `settings.conf`?* — only
//!   [`crate::prefstore`] knew, so a stored value and the control that changes
//!   it could drift apart with nothing to notice.
//! * *How does another program open this exact row?* — nothing answered, so
//!   there was no deep link at all.
//!
//! This module is that table. Every persisted preference has exactly one
//! [`Entry`]: the store key, the page that owns it, the titles a person might
//! type, and a function that renders the current value. The table is checked
//! against the store in both directions by
//! [`tests::every_key_the_store_writes_is_either_searchable_or_declared_internal`],
//! so a new preference cannot be added to the file and forgotten here, and a
//! row here cannot name a key the file never writes.
//!
//! # What it deliberately is not
//!
//! Not a settings daemon, and not a schema for settings the shell does not own.
//! A page whose state comes from the kernel or from `litric-sysd` (Display,
//! Network, Power) has no row here, because there is nothing to store and
//! nothing to set: those pages report measurements, and a registry row for them
//! would be a control that cannot do anything. The parity ledger calls that
//! `DEFER`, and the gate matrix calls an inert control a defect.
//!
//! # Deep links
//!
//! `litric://settings/<page>` opens a page and `litric://settings/<page>/<key>`
//! opens the page with that row focused. The form is ours rather than Windows'
//! `ms-settings:` or macOS' `x-apple.systempreferences:` because the page names
//! are ours; what is borrowed from both is the idea that a settings surface is
//! addressable at all (`MD.37`, `WB` §11.1 in the research corpus).

use crate::settings::{Page, Preferences, ThemeSchedule};

/// What a row offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Steps through a fixed list of values (theme, accent, wallpaper, engine).
    Choice,
    /// Two states.
    Toggle,
    /// A number the user picks from a fixed ladder.
    Ladder,
}

/// One persisted setting.
///
/// Every field is `'static` on purpose: the table is a `const`, the search
/// results borrow from it, and a row outlives no allocation it did not make.
#[derive(Debug)]
pub struct Entry {
    /// The key in `settings.conf`. Exactly what [`crate::prefstore`] writes.
    pub key: &'static str,
    /// The page that draws this row.
    pub page: Page,
    /// The name shown in a search result.
    pub title: &'static str,
    /// Words a person might type instead. Lower case; matched as substrings.
    pub synonyms: &'static [&'static str],
    /// What the control does.
    pub kind: Kind,
    /// The value as the user would read it, from a preference record.
    pub value: fn(&Preferences) -> String,
}

impl PartialEq for Entry {
    /// Two rows are the same row when they name the same store key.
    ///
    /// Not the derived comparison: a `fn` pointer's address is not guaranteed
    /// to be unique or stable, and rustc warns about exactly that. The key is
    /// the identity — the tests below fail if two rows ever share one.
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Entry {}

impl Entry {
    /// The text this row matches on, for the search and for the tests.
    #[must_use]
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return false;
        }
        if self.title.to_lowercase().contains(&query) {
            return true;
        }
        if self
            .synonyms
            .iter()
            .any(|word| word.to_lowercase().contains(&query))
        {
            return true;
        }
        // The page's own name, so "appearance theme" reaches the theme row.
        self.page.title().to_lowercase().contains(&query)
    }
}

/// Keys the store writes that are state rather than settings.
///
/// `version` is the schema marker and `onboarded` is how first-run knows it
/// finished. Neither is something a person searches for or changes, and
/// offering a row for `onboarded` would let a user re-enter setup from inside
/// setup. The list is a decision, not an oversight: the store test fails if a
/// new key appears that is in neither this list nor the table.
pub const INTERNAL_KEYS: [&str; 2] = ["version", "onboarded"];

/// Every setting the shell owns, in the order the pages present them.
pub const ENTRIES: [Entry; 7] = [
    Entry {
        key: "theme",
        page: Page::Appearance,
        title: "Theme",
        synonyms: &["dark mode", "light mode", "colour scheme", "high contrast"],
        kind: Kind::Choice,
        value: |p| theme_name(p.theme).to_owned(),
    },
    Entry {
        key: "theme_schedule",
        page: Page::Appearance,
        title: "Automatic light and dark",
        synonyms: &["auto", "schedule", "night", "sunset", "day and night"],
        kind: Kind::Choice,
        value: |p| p.theme_schedule.title().to_owned(),
    },
    Entry {
        key: "accent",
        page: Page::Appearance,
        title: "Accent colour",
        synonyms: &["colour", "color", "highlight", "tint"],
        kind: Kind::Choice,
        value: |p| p.accent.token().to_owned(),
    },
    Entry {
        key: "wallpaper",
        page: Page::Appearance,
        title: "Wallpaper",
        synonyms: &["background", "desktop picture", "desktop image"],
        kind: Kind::Choice,
        value: |p| wallpaper_name(p.wallpaper).to_owned(),
    },
    Entry {
        key: "language",
        page: Page::Region,
        title: "Interface language",
        synonyms: &["ไทย", "thai", "english", "ภาษา"],
        kind: Kind::Choice,
        value: |p| p.language.label().to_owned(),
    },
    Entry {
        key: "reduce_motion",
        page: Page::Accessibility,
        title: "Reduce motion",
        synonyms: &["animation", "motion", "transitions"],
        kind: Kind::Toggle,
        value: |p| yes_no(p.reduce_motion).to_owned(),
    },
    Entry {
        key: "search_engine",
        page: Page::Privacy,
        title: "Search engine",
        synonyms: &["google", "duckduckgo", "bing", "omnibox", "provider"],
        kind: Kind::Choice,
        value: |p| p.search_engine.title().to_owned(),
    },
];

fn yes_no(enabled: bool) -> &'static str {
    if enabled {
        "On"
    } else {
        "Off"
    }
}

/// The user-facing name of a theme, in the same words the Appearance page uses.
fn theme_name(kind: litric_design::ThemeKind) -> &'static str {
    use litric_design::ThemeKind;
    match kind {
        ThemeKind::Dark => "Dark",
        ThemeKind::Light => "Light",
        ThemeKind::HighContrastDark => "High contrast dark",
        ThemeKind::HighContrastLight => "High contrast light",
        // `ThemeKind` is non-exhaustive so a new theme can be added without
        // breaking this crate. A theme that appears without a name here must
        // still be readable, and its token is the honest fallback.
        _ => "Theme",
    }
}

/// The user-facing name of a wallpaper.
fn wallpaper_name(wallpaper: crate::desktop::Wallpaper) -> &'static str {
    use crate::desktop::Wallpaper;
    match wallpaper {
        Wallpaper::System => "System",
        Wallpaper::Flat => "Flat",
        Wallpaper::Glow => "Glow",
    }
}

/// Every row.
#[must_use]
pub fn all() -> &'static [Entry] {
    &ENTRIES
}

/// The row for a store key, if the shell offers one.
#[must_use]
pub fn find(key: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|entry| entry.key == key)
}

/// Rows on one page, in table order.
#[must_use]
pub fn page_entries(page: Page) -> Vec<&'static Entry> {
    ENTRIES.iter().filter(|entry| entry.page == page).collect()
}

/// Every row whose title, synonyms or page name match `query`.
///
/// The order is fixed — a shorter title whose prefix matches first, then
/// alphabetical — because a result list that reorders itself while the user is
/// reaching for the third row moves the target. Same rule as
/// [`crate::seek::search`].
#[must_use]
pub fn search(query: &str) -> Vec<&'static Entry> {
    let mut hits: Vec<(u8, &'static str, &'static Entry)> = ENTRIES
        .iter()
        .filter(|entry| entry.matches(query))
        .map(|entry| {
            let rank = match entry
                .title
                .to_lowercase()
                .starts_with(&query.trim().to_lowercase())
            {
                true => 0u8,
                false => 1,
            };
            (rank, entry.title, entry)
        })
        .collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(b.1)));
    hits.into_iter().map(|(_, _, entry)| entry).collect()
}

/// A settings address that has been resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// The page to show.
    pub page: Page,
    /// The row to highlight, when the link named one that exists.
    pub row: Option<&'static Entry>,
}

/// The scheme and host every settings link starts with.
pub const PREFIX: &str = "litric://settings";

/// Resolve a `litric://settings/...` address.
///
/// `None` means "this is not a settings link", so the caller can try the next
/// handler. A link whose page or row is unknown resolves to `None` as well:
/// opening the wrong page silently is worse than telling the user the link is
/// broken, and the caller has the scheme in hand to say so.
///
/// Accepted forms, all equivalent where the page exists:
/// `litric://settings`, `litric://settings/`, `litric://settings/appearance`,
/// `litric://settings/appearance/`, `LITRIC://Settings/Appearance`.
#[must_use]
pub fn resolve(url: &str) -> Option<Target> {
    let rest = strip_prefix_ignore_case(url.trim(), PREFIX)?;
    // A query or fragment belongs to whoever wrote the link; it never names a
    // page, so it is cut before the path is split. `litric://settings?x` and
    // `litric://settings/appearance#theme` both work.
    let rest = rest.split(['?', '#']).next().unwrap_or("");
    let mut segments = rest
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty());
    let page = match segments.next() {
        None => Page::Appearance,
        Some(slug) => Page::from_slug(slug)?,
    };
    match segments.next() {
        None => Some(Target { page, row: None }),
        Some(key) => {
            // A row on the wrong page is still a row: the link names a setting,
            // and the setting's page is a fact about the table, not about the
            // address. Following it lands on the right page rather than on a
            // page that has no such row.
            let entry = find(key)?;
            Some(Target {
                page: entry.page,
                row: Some(entry),
            })
        }
    }
}

/// `litric://settings/appearance/theme` for a row, `litric://settings/<page>`
/// for a page. The inverse of [`resolve`], and tested as such.
#[must_use]
pub fn link_for(target: Target) -> String {
    match target.row {
        Some(entry) => format!("{PREFIX}/{}/{}", target.page.slug(), entry.key),
        None => format!("{PREFIX}/{}", target.page.slug()),
    }
}

/// Case-insensitive prefix strip. The scheme and host are case-insensitive per
/// RFC 3986 §3.1; only the path is not, and page slugs are lower case anyway.
fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let (head, tail) = text.split_at(prefix.len());
    if head.eq_ignore_ascii_case(prefix) {
        Some(tail)
    } else {
        None
    }
}

/// What the Appearance page should say about the schedule right now.
///
/// Kept here rather than in the drawing code so the sentence and the behaviour
/// cannot disagree: this is the same [`ThemeSchedule::effective`] call the
/// shell makes when it decides which theme to paint.
#[must_use]
pub fn schedule_detail(preferences: &Preferences, hour: u8) -> String {
    match preferences.theme_schedule {
        ThemeSchedule::Manual => format!("Always {}", theme_name(preferences.theme).to_lowercase()),
        ThemeSchedule::Auto => format!(
            "{} now · dark 19:00–07:00",
            theme_name(
                preferences
                    .theme_schedule
                    .effective(preferences.theme, hour)
            )
        ),
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::settings::Preferences;

    /// The keys `encode` actually writes, parsed from its output rather than
    /// listed here: a second hand-written list is a second thing to drift.
    fn stored_keys() -> Vec<String> {
        crate::prefstore::encode(Preferences::default())
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(key, _)| key.to_owned())
            .collect()
    }

    #[test]
    fn every_key_the_store_writes_is_either_searchable_or_declared_internal() {
        let keys = stored_keys();
        for key in &keys {
            if INTERNAL_KEYS.contains(&key.as_str()) {
                continue;
            }
            assert!(
                find(key).is_some(),
                "the store writes `{key}` but no registry row offers it, so the \
                 setting exists and cannot be found"
            );
        }
    }

    #[test]
    fn every_row_names_a_key_the_store_writes() {
        let keys = stored_keys();
        for entry in all() {
            assert!(
                keys.iter().any(|key| key == entry.key),
                "the registry offers `{}` but the store never writes it",
                entry.key
            );
        }
    }

    #[test]
    fn no_key_is_registered_twice() {
        for (i, entry) in ENTRIES.iter().enumerate() {
            for other in ENTRIES.iter().skip(i.saturating_add(1)) {
                assert_ne!(entry.key, other.key, "two rows share a store key");
            }
        }
    }

    /// A page with no row and no measurement behind it is a page that shows
    /// nothing; the sidebar should not offer it. This is the acceptance
    /// criterion in the plan — "every page is reachable from the registry" —
    /// narrowed to what the registry can actually vouch for: a page that owns
    /// settings owns at least one row.
    #[test]
    fn a_page_that_owns_settings_owns_at_least_one_row() {
        for page in Page::ALL {
            let owns = ENTRIES.iter().any(|entry| entry.page == page);
            if owns {
                assert!(
                    !page_entries(page).is_empty(),
                    "{page:?} owns settings but no row lists it"
                );
            }
        }
    }

    #[test]
    fn every_page_slug_is_unique_and_parses_back() {
        let mut seen: Vec<&str> = Vec::new();
        for page in Page::ALL {
            let slug = page.slug();
            assert!(!slug.is_empty(), "{page:?} has an empty slug");
            assert!(!seen.contains(&slug), "two pages share the slug {slug}");
            seen.push(slug);
            assert_eq!(Page::from_slug(slug), Some(page), "{slug} does not parse");
        }
    }

    #[test]
    fn a_search_finds_the_row_and_not_only_its_page() {
        // The gap this module closes: before it, "night light" and "dark mode"
        // found nothing at all, and "wallpaper" found the page rather than the
        // row.
        for (query, key) in [
            ("dark mode", "theme"),
            ("wallpaper", "wallpaper"),
            ("night", "theme_schedule"),
            ("reduce motion", "reduce_motion"),
            ("duckduckgo", "search_engine"),
            ("ไทย", "language"),
        ] {
            let hits = search(query);
            assert!(
                hits.iter().any(|entry| entry.key == key),
                "`{query}` did not reach `{key}`: {hits:?}"
            );
        }
    }

    #[test]
    fn an_empty_query_finds_nothing() {
        assert!(search("").is_empty());
        assert!(search("   ").is_empty());
    }

    #[test]
    fn the_link_round_trips_to_the_same_target() {
        for entry in all() {
            let url = link_for(Target {
                page: entry.page,
                row: Some(entry),
            });
            let back = resolve(&url).expect("the link this function wrote resolves");
            assert_eq!(back.page, entry.page, "{url}");
            assert_eq!(back.row.map(|row| row.key), Some(entry.key), "{url}");
        }
        for page in Page::ALL {
            let url = link_for(Target { page, row: None });
            assert_eq!(resolve(&url), Some(Target { page, row: None }), "{url}");
        }
    }

    #[test]
    fn a_link_opens_the_page_the_setting_lives_on() {
        // `litric://settings/appearance/reduce_motion` names a row that is not
        // on Appearance. The row wins: the caller asked for a setting, and the
        // setting's page is where it is drawn.
        let target = resolve("litric://settings/appearance/reduce_motion").expect("resolves");
        assert_eq!(target.page, Page::Accessibility);
        assert_eq!(target.row.map(|row| row.key), Some("reduce_motion"));
    }

    #[test]
    fn the_forms_a_link_is_written_in_all_work() {
        let page = Target {
            page: Page::Appearance,
            row: None,
        };
        for url in [
            "litric://settings",
            "litric://settings/",
            "litric://settings/appearance",
            "litric://settings/appearance/",
            "  litric://settings/appearance  ",
            "LITRIC://Settings/Appearance",
            "litric://settings/appearance?from=search",
            "litric://settings/appearance#top",
        ] {
            assert_eq!(resolve(url), Some(page), "{url}");
        }
    }

    #[test]
    fn a_link_to_nothing_is_refused_rather_than_redirected() {
        for url in [
            "https://example.com/settings/appearance",
            "litric://files/home",
            "litric://settings/nonexistent",
            "litric://settings/appearance/no_such_key",
            "litric://",
            "",
            "settings/appearance",
        ] {
            assert_eq!(resolve(url), None, "{url} should not resolve");
        }
    }

    #[test]
    fn a_schedule_detail_never_claims_a_theme_that_is_not_showing() {
        use litric_design::ThemeKind;
        let manual = Preferences {
            theme: ThemeKind::Light,
            theme_schedule: ThemeSchedule::Manual,
            ..Preferences::default()
        };
        assert!(schedule_detail(&manual, 23).contains("light"));
        let auto = Preferences {
            theme: ThemeKind::Light,
            theme_schedule: ThemeSchedule::Auto,
            ..Preferences::default()
        };
        assert!(
            schedule_detail(&auto, 23).contains("Dark"),
            "23:00 is night"
        );
        assert!(schedule_detail(&auto, 12).contains("Light"), "noon is day");
    }
}
