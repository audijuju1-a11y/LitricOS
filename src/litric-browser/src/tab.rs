//! Tabs, history and what a page is doing.
//!
//! The model a browser window shows. It holds no engine and performs no
//! navigation: it records what was asked for and what came back, which is
//! exactly the part that has to be right before anything is rendering.
//!
//! # Back and forward
//!
//! One list with a cursor, not two stacks. The two-stack version is the classic
//! way to write this and it is subtly wrong: navigating somewhere new while
//! part-way back through history has to *truncate* the forward entries, and
//! with two stacks it is easy to leave them there — so Forward takes you to a
//! page you can no longer reach by going Back.

use crate::url::Url;

/// A tab's identity. Never reused within a session.
pub type TabId = u32;

/// What a tab is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadState {
    /// Nothing has been asked for.
    Empty,
    /// A request is in flight.
    Loading {
        /// 0-100 where the engine reports it, `None` where it does not.
        progress: Option<u8>,
    },
    /// The page is there.
    Loaded,
    /// The request failed.
    Failed {
        /// Why, in the user's words.
        reason: String,
    },
    /// The policy refused it before it was sent.
    Blocked {
        /// Why.
        reason: String,
    },
}

impl LoadState {
    /// Whether a spinner should be showing.
    #[must_use]
    pub const fn is_busy(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }

    /// A short token for diagnostics.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Loading { .. } => "loading",
            Self::Loaded => "loaded",
            Self::Failed { .. } => "failed",
            Self::Blocked { .. } => "blocked",
        }
    }
}

/// One tab's navigation history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    entries: Vec<Url>,
    /// Which entry is showing. Meaningless when `entries` is empty.
    index: usize,
}

impl History {
    /// How many entries there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there is nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entry showing now.
    #[must_use]
    pub fn current(&self) -> Option<&Url> {
        self.entries.get(self.index)
    }

    /// Every entry, oldest first.
    #[must_use]
    pub fn entries(&self) -> &[Url] {
        &self.entries
    }

    /// Where the cursor is.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Go somewhere new.
    ///
    /// Anything ahead of the cursor is discarded, because it is no longer
    /// reachable: you cannot go Back to it from here.
    pub fn push(&mut self, url: Url) {
        if self.current() == Some(&url) {
            // Reloading the same address is not a new history entry, or Back
            // stops working after a refresh.
            return;
        }
        if !self.entries.is_empty() {
            self.entries.truncate(self.index.saturating_add(1));
            self.index = self.index.saturating_add(1);
        }
        self.entries.push(url);
        if self.entries.len() == 1 {
            self.index = 0;
        }
    }

    /// Whether Back would do anything.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.index > 0
    }

    /// Whether Forward would do anything.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.index.saturating_add(1) < self.entries.len()
    }

    /// Go back one entry.
    pub fn back(&mut self) -> Option<&Url> {
        if !self.can_go_back() {
            return None;
        }
        self.index = self.index.saturating_sub(1);
        self.current()
    }

    /// Go forward one entry.
    pub fn forward(&mut self) -> Option<&Url> {
        if !self.can_go_forward() {
            return None;
        }
        self.index = self.index.saturating_add(1);
        self.current()
    }
}

/// One tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// Identity.
    pub id: TabId,
    /// What the page called itself, or the host when it has not said.
    pub title: String,
    /// What it is doing.
    pub state: LoadState,
    /// Where it has been.
    pub history: History,
    /// Whether it is pinned to the left of the strip.
    pub pinned: bool,
    /// Whether the user muted it.
    pub muted: bool,
    /// Whether it is currently making noise. Never set by anything yet: no
    /// engine reports it.
    pub audible: bool,
}

impl Tab {
    /// A new, empty tab.
    #[must_use]
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            title: "New tab".to_owned(),
            state: LoadState::Empty,
            history: History::default(),
            pinned: false,
            muted: false,
            audible: false,
        }
    }

    /// Where it is.
    #[must_use]
    pub fn url(&self) -> Option<&Url> {
        self.history.current()
    }

    /// What the tab strip should show.
    ///
    /// The title when the page gave one, the host when it did not, and the
    /// reason when it failed — never a blank tab the user cannot identify.
    #[must_use]
    pub fn label(&self) -> String {
        match &self.state {
            LoadState::Blocked { .. } => "Blocked".to_owned(),
            LoadState::Failed { .. } => self.url().map_or_else(
                || "Failed".to_owned(),
                |u| format!("{} (failed)", u.tab_label()),
            ),
            _ if self.title.is_empty() => self
                .url()
                .map_or_else(|| "New tab".to_owned(), Url::tab_label),
            _ => self.title.clone(),
        }
    }

    /// Begin loading an address.
    pub fn navigate(&mut self, url: Url) {
        self.title = url.tab_label();
        self.history.push(url);
        self.state = LoadState::Loading { progress: None };
    }

    /// Record that the policy refused it.
    ///
    /// The address still enters history: the user asked for it, and a Back
    /// button that skips the page explaining why something was blocked is a
    /// Back button that loses the explanation.
    pub fn blocked(&mut self, url: Url, reason: &str) {
        self.title = url.tab_label();
        self.history.push(url);
        self.state = LoadState::Blocked {
            reason: reason.to_owned(),
        };
    }

    /// Record that the load failed.
    pub fn failed(&mut self, reason: &str) {
        self.state = LoadState::Failed {
            reason: reason.to_owned(),
        };
    }

    /// Record that the load finished.
    pub fn loaded(&mut self, title: Option<&str>) {
        if let Some(t) = title.map(str::trim).filter(|t| !t.is_empty()) {
            self.title = t.to_owned();
        }
        self.state = LoadState::Loaded;
    }

    /// Go back, returning the address to load.
    pub fn back(&mut self) -> Option<Url> {
        let u = self.history.back().cloned()?;
        self.title = u.tab_label();
        self.state = LoadState::Loading { progress: None };
        Some(u)
    }

    /// Go forward, returning the address to load.
    pub fn forward(&mut self) -> Option<Url> {
        let u = self.history.forward().cloned()?;
        self.title = u.tab_label();
        self.state = LoadState::Loading { progress: None };
        Some(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::Url;

    fn url(s: &str) -> Url {
        Url::parse(s).expect(s)
    }

    #[test]
    fn a_new_tab_is_empty_and_named() {
        let t = Tab::new(1);
        assert_eq!(t.state, LoadState::Empty);
        assert!(t.url().is_none());
        assert_eq!(t.label(), "New tab");
        assert!(!t.history.can_go_back() && !t.history.can_go_forward());
    }

    #[test]
    fn navigating_records_history_and_starts_loading() {
        let mut t = Tab::new(1);
        t.navigate(url("https://example.com/"));
        assert!(t.state.is_busy());
        assert_eq!(
            t.url().map(Url::to_url).as_deref(),
            Some("https://example.com/")
        );
        assert_eq!(t.history.len(), 1);
    }

    #[test]
    fn back_and_forward_walk_the_history() {
        let mut t = Tab::new(1);
        t.navigate(url("https://a.test/"));
        t.loaded(Some("A"));
        t.navigate(url("https://b.test/"));
        t.loaded(Some("B"));
        t.navigate(url("https://c.test/"));
        t.loaded(Some("C"));

        assert!(t.history.can_go_back());
        assert_eq!(t.back().map(|u| u.host), Some("b.test".to_owned()));
        assert_eq!(t.back().map(|u| u.host), Some("a.test".to_owned()));
        assert!(!t.history.can_go_back());
        assert_eq!(t.back(), None, "back past the start does nothing");
        assert_eq!(t.forward().map(|u| u.host), Some("b.test".to_owned()));
    }

    #[test]
    fn navigating_while_part_way_back_discards_the_forward_entries() {
        // The bug the one-list model exists to prevent: Forward taking you to a
        // page you can no longer reach by going Back.
        let mut t = Tab::new(1);
        for h in ["https://a.test/", "https://b.test/", "https://c.test/"] {
            t.navigate(url(h));
            t.loaded(None);
        }
        t.back();
        t.back();
        assert!(t.history.can_go_forward());
        t.navigate(url("https://d.test/"));
        assert!(
            !t.history.can_go_forward(),
            "forward history survived a new navigation"
        );
        assert_eq!(t.history.len(), 2, "a.test then d.test");
    }

    #[test]
    fn reloading_the_same_address_does_not_grow_the_history() {
        // Otherwise Back after three refreshes needs three presses to leave.
        let mut t = Tab::new(1);
        t.navigate(url("https://a.test/"));
        t.navigate(url("https://a.test/"));
        t.navigate(url("https://a.test/"));
        assert_eq!(t.history.len(), 1);
        assert!(!t.history.can_go_back());
    }

    #[test]
    fn a_blocked_page_still_enters_history() {
        // The page explaining the block must be reachable with Back.
        let mut t = Tab::new(1);
        t.navigate(url("https://a.test/"));
        t.loaded(None);
        t.blocked(url("http://insecure.test/"), "http was refused");
        assert!(matches!(t.state, LoadState::Blocked { .. }));
        assert_eq!(t.label(), "Blocked");
        assert!(t.history.can_go_back());
        assert_eq!(t.back().map(|u| u.host), Some("a.test".to_owned()));
    }

    #[test]
    fn a_failed_tab_says_which_site_failed() {
        let mut t = Tab::new(1);
        t.navigate(url("https://a.test/page"));
        t.failed("the name could not be resolved");
        assert_eq!(t.label(), "a.test (failed)");
        assert!(!t.state.is_busy());
    }

    #[test]
    fn a_page_with_no_title_is_labelled_by_its_host() {
        let mut t = Tab::new(1);
        t.navigate(url("https://example.com/deep/path"));
        t.loaded(Some("   "));
        assert_eq!(t.label(), "example.com");
        t.loaded(Some("Real Title"));
        assert_eq!(t.label(), "Real Title");
    }

    #[test]
    fn history_of_a_single_page_has_no_back_or_forward() {
        let mut h = History::default();
        h.push(url("https://only.test/"));
        assert_eq!(h.len(), 1);
        assert_eq!(h.index(), 0);
        assert!(!h.can_go_back() && !h.can_go_forward());
        assert_eq!(h.back(), None);
        assert_eq!(h.forward(), None);
    }

    #[test]
    fn an_empty_history_is_safe_to_walk() {
        let mut h = History::default();
        assert!(h.is_empty());
        assert_eq!(h.current(), None);
        assert_eq!(h.back(), None);
        assert_eq!(h.forward(), None);
    }
}
