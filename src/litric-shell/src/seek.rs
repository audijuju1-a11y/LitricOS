//! One search field for the whole machine.
//!
//! The launcher used to search applications. That is the smallest possible
//! reading of a search field and the wrong one: a person who types "lock" in
//! the place where every other operating system lets them type anything has
//! asked the system to lock, not to find an application called Lock. The same
//! is true of a window they left on another workspace, and of the settings page
//! they cannot remember the name of the category for.
//!
//! This module is the index over those four kinds of thing. It is deliberately
//! a pure function of the shell's state: no caching, no background thread, no
//! ordering that depends on when something was last focused. A list that
//! reorders itself while the user is reaching for the third row is a list that
//! moves the target.
//!
//! Ranking is text first and kind second. An exact application name beats a
//! settings page with the same word in it; a window title beats an action that
//! merely mentions the word. Within a kind, the order is alphabetical, so the
//! result of the same query is the same list on every machine.

use litric_wm::WindowId;

use crate::settings::Page;
use crate::shell::Shell;
use crate::shortcuts::{Action, App};

/// What a result stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    /// Open an application.
    App(App),
    /// Run a system action.
    Action(Action),
    /// Open a Settings page.
    Setting(Page),
    /// Bring a window to the user.
    Window(WindowId),
}

/// One row in the search results.
#[derive(Debug, Clone)]
pub struct Result {
    /// What activating the row does.
    pub hit: Hit,
    /// The name shown in the row.
    pub title: String,
    /// Where the thing lives: `Application`, `Window · Workspace 2`, ...
    pub detail: String,
    /// The shortcut that does the same thing, when there is one.
    pub key: Option<String>,
    /// Higher sorts first. Text match plus a small bonus for the kind.
    score: u32,
    /// Stable tie-break: kind, then title.
    rank: (u8, String),
}

/// How many rows the search field will show.
pub const LIMIT: usize = 8;

/// Score one piece of text against a query, or nothing when it does not match.
///
/// The tiers exist because "sett" matching "Settings" (a prefix) should not
/// lose to "Reset settings" (a substring) merely because the latter is longer.
fn text_score(text: &str, query: &str) -> Option<u32> {
    let text = text.to_lowercase();
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return None;
    }
    if text == query {
        return Some(1000);
    }
    if text.starts_with(&query) {
        return Some(800);
    }
    if text
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| !word.is_empty() && word.starts_with(&query))
    {
        return Some(650);
    }
    if text.contains(&query) {
        return Some(500);
    }
    // Every word of a multi-word query appears somewhere: "open settings"
    // reaches Settings even though no single label contains the phrase.
    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.len() > 1 && tokens.iter().all(|token| text.contains(token)) {
        return Some(400);
    }
    None
}

/// Everything the query matches, best first, at most [`LIMIT`] rows.
#[must_use]
pub fn search(shell: &Shell, query: &str) -> Vec<Result> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Result> = Vec::new();

    // Applications are ranked by their own search, which knows their
    // descriptions -- a result the user cannot name is still findable by what
    // it does.
    for app in App::search(query) {
        out.push(Result {
            hit: Hit::App(app),
            title: app.title().to_owned(),
            detail: "Application".to_owned(),
            key: shell.shortcuts.label_for(Action::Open(app)),
            // An application is what the user most often means. Its own
            // search contributes, and a clean title match contributes more,
            // so "settings" opens Settings while "quick settings" still finds
            // the action by its name.
            score: text_score(app.title(), query)
                .unwrap_or(0)
                .saturating_add(app.score(query).unwrap_or(0))
                .saturating_add(300),
            rank: (0, app.title().to_lowercase()),
        });
    }

    for page in Page::ALL {
        if let Some(result) = scored(
            Hit::Setting(page),
            page.title(),
            "Settings".to_owned(),
            None,
            1,
            60,
            query,
        ) {
            out.push(result);
        }
    }

    for window in shell.wm.all() {
        let detail = format!(
            "Window  ·  {}",
            shell
                .wm
                .workspaces()
                .get(window.workspace)
                .map_or("Workspace", String::as_str)
        );
        if let Some(result) = scored(
            Hit::Window(window.id),
            &window.title,
            detail,
            None,
            2,
            55,
            query,
        ) {
            out.push(result);
        }
    }

    for entry in shell.shortcuts.all() {
        if matches!(entry.action, Action::Open(_)) {
            // The application rows above already carry this action, with the
            // same key beside them.
            continue;
        }
        if let Some(result) = scored(
            Hit::Action(entry.action),
            &entry.action.describe(),
            "Action".to_owned(),
            shell.shortcuts.label_for(entry.action),
            3,
            40,
            query,
        ) {
            out.push(result);
        }
    }

    // A window titled "Settings" and an application named Settings can both
    // match; showing the same title twice is noise, so the strongest one wins.
    out.sort_by(|a, b| b.score.cmp(&a.score).then(a.rank.cmp(&b.rank)));
    let mut seen: Vec<(Hit, String)> = Vec::new();
    out.retain(|r| {
        let key = (r.hit, r.title.to_lowercase());
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
    out.truncate(LIMIT);
    out
}

/// Build one row if its title or its detail matches.
#[allow(clippy::too_many_arguments)]
fn scored(
    hit: Hit,
    title: &str,
    detail: String,
    key: Option<String>,
    kind: u8,
    bonus: u32,
    query: &str,
) -> Option<Result> {
    // A match in the detail line still finds the row ("settings" opens the
    // Settings pages), but it is worth a quarter of a title match: otherwise
    // the word "Settings" in every page's detail would bury the Settings
    // application itself.
    let text = text_score(title, query)
        .or_else(|| text_score(&detail, query).map(|score| score.saturating_div(4)))?;
    Some(Result {
        hit,
        title: title.to_owned(),
        detail,
        key,
        score: text.saturating_add(bonus),
        rank: (kind, title.to_lowercase()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell() -> Shell {
        let tokens = crate::ui::Tokens::new(
            litric_design::Theme::of(litric_design::ThemeKind::Dark),
            1.0,
        );
        let mut s = Shell::new(
            tokens,
            1280.0,
            800.0,
            litric_locale::Regional::fallback("test"),
        );
        s.mode = crate::shell::DesktopMode::Ready;
        s
    }

    #[test]
    fn a_query_with_no_words_finds_nothing() {
        let s = shell();
        assert!(search(&s, "").is_empty());
        assert!(search(&s, "   ").is_empty());
    }

    #[test]
    fn the_application_comes_before_an_action_that_mentions_it() {
        let s = shell();
        let results = search(&s, "settings");
        assert_eq!(
            results.first().map(|r| r.hit),
            Some(Hit::App(App::Settings)),
            "{results:?}"
        );
    }

    #[test]
    fn an_action_is_reachable_by_what_it_does() {
        let s = shell();
        let results = search(&s, "lock");
        assert!(
            results
                .iter()
                .any(|r| matches!(r.hit, Hit::Action(Action::Lock))),
            "{results:?}"
        );
    }

    #[test]
    fn a_window_is_reachable_by_its_title() {
        let mut s = shell();
        s.wm.open("com.litric.files", "Quarterly report", (400.0, 300.0));
        let results = search(&s, "quarterly");
        assert!(
            results.iter().any(|r| matches!(r.hit, Hit::Window(_))),
            "{results:?}"
        );
    }

    #[test]
    fn a_settings_page_is_reachable_by_its_name() {
        let s = shell();
        let results = search(&s, "bluetooth");
        assert!(
            results
                .iter()
                .any(|r| matches!(r.hit, Hit::Setting(Page::Bluetooth))),
            "{results:?}"
        );
    }

    #[test]
    fn the_same_query_returns_the_same_list() {
        let mut s = shell();
        s.wm.open("com.litric.files", "Documents", (400.0, 300.0));
        s.wm.open("com.litric.terminal", "Terminal", (400.0, 300.0));
        let a: Vec<String> = search(&s, "e").iter().map(|r| r.title.clone()).collect();
        let b: Vec<String> = search(&s, "e").iter().map(|r| r.title.clone()).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn the_list_is_bounded() {
        let mut s = shell();
        for i in 0..40 {
            s.wm.open("com.litric.files", &format!("Window {i}"), (320.0, 200.0));
        }
        assert!(search(&s, "window").len() <= LIMIT);
    }
}
