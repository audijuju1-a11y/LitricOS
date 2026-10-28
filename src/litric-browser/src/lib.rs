//! Litric Browser.
//!
//! # Layer position
//!
//! **Layer 6 (Applications).** It sits on `litric-caps` for permissions and on
//! the shell for its window, and it decides what a browser *is* on this system:
//! how an address is read, what a site is allowed to do, and what happens when
//! it is refused.
//!
//! # The honest boundary
//!
//! **There is no rendering engine here, and there is never going to be one.**
//! A modern web engine is millions of lines and a permanent, full-time security
//! commitment; writing one would be the single most reckless thing this project
//! could do. Litric Browser integrates an existing engine and owns everything
//! around it — the interface, the tab and history model, the URL handling, and
//! the privacy and permission policy, which is where a browser's actual
//! character lives.
//!
//! Until an engine is integrated, [`engine::Engine`] has one implementation:
//! [`engine::NoEngine`], which loads nothing and says so. Every tab it is asked
//! to open ends in [`tab::LoadState::Failed`] with a message naming the missing
//! piece. That is the whole point — a browser window that renders a fake page
//! would be worse than one that admits it cannot render yet.
//!
//! ```text
//!   address bar ─► url::interpret ─► policy::check_navigation ─► engine
//!                        │                     │                    │
//!                   Navigate/Search        Refusal              LoadState
//! ```
//!
//! # What is real today
//!
//! * URL parsing, including the address-bar attacks it has to refuse.
//! * Tab and history semantics, including the ones that are usually wrong.
//! * A privacy policy whose defaults are the strict ones.
//! * Permissions mapped onto the system capability model rather than a private
//!   one.

#![forbid(unsafe_code)]

pub mod base64;
pub mod engine;
pub mod jpeg;
pub mod json;
pub mod policy;
pub mod tab;
pub mod url;

pub use policy::{Decision, Policy, PrivacyPolicy, Refusal, SitePermission};
pub use tab::{LoadState, Tab, TabId};
pub use url::{interpret, Intent, Scheme, Url, UrlError};

/// A browser session: the tabs, the policy, and the engine behind them.
#[derive(Debug)]
pub struct Session {
    tabs: Vec<Tab>,
    active: usize,
    next_id: TabId,
    /// The privacy and permission policy in force.
    pub policy: Policy,
    /// The rendering engine.
    pub engine: Box<dyn engine::Engine>,
    closed: Vec<Tab>,
}

impl Session {
    /// The most recently-closed tabs kept for reopening.
    pub const REOPEN_CAPACITY: usize = 16;

    /// A session with a policy and an engine.
    #[must_use]
    pub fn new(policy: Policy, engine: Box<dyn engine::Engine>) -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            next_id: 1,
            policy,
            engine,
            closed: Vec::new(),
        }
    }

    /// Open a new tab and make it active.
    pub fn open_tab(&mut self) -> TabId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.tabs.push(Tab::new(id));
        self.active = self.tabs.len().saturating_sub(1);
        id
    }

    /// Every tab, in strip order.
    #[must_use]
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// How many tabs are open.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// The active tab.
    #[must_use]
    pub fn active(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    /// Make a tab active.
    pub fn focus(&mut self, id: TabId) -> bool {
        match self.tabs.iter().position(|t| t.id == id) {
            Some(i) => {
                self.active = i;
                true
            }
            None => false,
        }
    }

    /// Close a tab, keeping it for reopening.
    pub fn close(&mut self, id: TabId) -> bool {
        let Some(i) = self.tabs.iter().position(|t| t.id == id) else {
            return false;
        };
        let tab = self.tabs.remove(i);
        self.closed.push(tab);
        if self.closed.len() > Self::REOPEN_CAPACITY {
            self.closed.remove(0);
        }
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
        true
    }

    /// Reopen the most recently closed tab.
    pub fn reopen(&mut self) -> Option<TabId> {
        let tab = self.closed.pop()?;
        let id = tab.id;
        self.tabs.push(tab);
        self.active = self.tabs.len().saturating_sub(1);
        Some(id)
    }

    /// Do what the user meant by what they typed in the address bar.
    ///
    /// Returns what happened, so the interface can show a refusal rather than
    /// an empty page.
    pub fn go(&mut self, id: TabId, input: &str) -> Outcome {
        let intent = url::interpret(input);
        let target = match intent {
            Intent::Navigate(u) => u,
            Intent::Search(q) => {
                // No search provider is configured, and inventing one would
                // send everything the user types to a third party they never
                // chose. Until a provider is a setting, a search is refused
                // with the reason.
                return Outcome::NoSearchProvider { query: q };
            }
            Intent::AddressError { input, error } => {
                let reason = error.to_string();
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                    tab.failed(&format!("that address is not valid: {reason}"));
                }
                return Outcome::InvalidAddress { input, reason };
            }
        };

        if let Err(refusal) = self.policy.check_navigation(&target) {
            let message = refusal.message();
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                tab.blocked(target, &message);
            }
            return Outcome::Blocked(refusal);
        }

        let engine_name = self.engine.name().to_owned();
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) else {
            return Outcome::NoSuchTab;
        };
        tab.navigate(target.clone());
        match self.engine.load(&target) {
            Ok(page) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                    tab.loaded(page.title.as_deref());
                }
                Outcome::Loaded
            }
            Err(reason) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                    tab.failed(&reason);
                }
                Outcome::Failed {
                    engine: engine_name,
                    reason,
                }
            }
        }
    }

    /// Lines for the status file and diagnostics.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![
            format!("browser.tabs={}", self.tabs.len()),
            format!("browser.engine={}", self.engine.name()),
            format!("browser.engine_ready={}", self.engine.is_ready()),
        ];
        out.extend(self.policy.lines());
        out
    }
}

/// What a navigation did.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// The page loaded.
    Loaded,
    /// The policy refused it.
    Blocked(Refusal),
    /// The engine could not load it.
    Failed {
        /// Which engine.
        engine: String,
        /// Why.
        reason: String,
    },
    /// What was typed was a search, and no search provider is configured.
    NoSearchProvider {
        /// What was typed.
        query: String,
    },
    /// What was typed names a supported address and does not parse.
    InvalidAddress {
        /// What was typed.
        input: String,
        /// Why it is not an address.
        reason: String,
    },
    /// There is no tab with that id.
    NoSuchTab,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::NoEngine;

    fn session() -> Session {
        Session::new(
            Policy::new(vec!["tracker.test".to_owned()]),
            Box::new(NoEngine),
        )
    }

    #[test]
    fn a_session_starts_empty_and_opens_tabs() {
        let mut s = session();
        assert!(s.is_empty());
        let a = s.open_tab();
        let b = s.open_tab();
        assert_eq!(s.len(), 2);
        assert_eq!(s.active().map(|t| t.id), Some(b));
        assert!(s.focus(a));
        assert_eq!(s.active().map(|t| t.id), Some(a));
        assert!(!s.focus(999));
    }

    #[test]
    fn closing_the_active_tab_moves_focus_and_does_not_leave_it_dangling() {
        let mut s = session();
        let a = s.open_tab();
        let b = s.open_tab();
        assert!(s.close(b));
        assert_eq!(s.active().map(|t| t.id), Some(a));
        assert!(s.close(a));
        assert!(s.is_empty());
        assert_eq!(s.active(), None, "focus survived the last tab");
    }

    #[test]
    fn a_closed_tab_can_be_reopened_most_recent_first() {
        let mut s = session();
        let a = s.open_tab();
        let b = s.open_tab();
        s.close(a);
        s.close(b);
        assert_eq!(s.reopen(), Some(b));
        assert_eq!(s.reopen(), Some(a));
        assert_eq!(s.reopen(), None);
    }

    #[test]
    fn without_an_engine_a_navigation_fails_and_names_what_is_missing() {
        // The honesty requirement in one test: the browser must not render a
        // fake page, and it must say why there is nothing.
        let mut s = session();
        let t = s.open_tab();
        let out = s.go(t, "https://example.com/");
        let Outcome::Failed { engine, reason } = out else {
            panic!("expected a failure, got {out:?}");
        };
        assert_eq!(engine, "none");
        assert!(
            reason.to_lowercase().contains("no rendering engine"),
            "{reason}"
        );
        assert!(matches!(
            s.active().map(|x| x.state.clone()),
            Some(LoadState::Failed { .. })
        ));
    }

    #[test]
    fn plain_http_is_refused_by_policy_before_the_engine_is_asked() {
        let mut s = session();
        let t = s.open_tab();
        let out = s.go(t, "http://example.com/");
        assert!(
            matches!(out, Outcome::Blocked(Refusal::InsecureBlocked { .. })),
            "{out:?}"
        );
        assert!(matches!(
            s.active().map(|x| x.state.clone()),
            Some(LoadState::Blocked { .. })
        ));
    }

    #[test]
    fn a_tracker_is_refused() {
        let mut s = session();
        let t = s.open_tab();
        let out = s.go(t, "https://cdn.tracker.test/pixel.gif");
        assert!(
            matches!(out, Outcome::Blocked(Refusal::Tracker { .. })),
            "{out:?}"
        );
    }

    #[test]
    fn a_malformed_address_is_reported_on_the_tab_instead_of_becoming_a_search() {
        // Caller-level: the parser's new outcome reaches `Session::go`, becomes
        // the tab's own error state (which the interface renders), and does not
        // enter history or reach the engine.
        let mut s = session();
        let t = s.open_tab();
        let out = s.go(t, "https://example.com:abc/");
        let Outcome::InvalidAddress { input, reason } = out else {
            panic!("expected an address error, got {out:?}");
        };
        assert_eq!(input, "https://example.com:abc/");
        assert!(reason.contains("port"), "{reason}");
        match s.active().map(|x| x.state.clone()) {
            Some(LoadState::Failed { reason }) => {
                assert!(reason.contains("not valid"), "{reason}");
            }
            other => panic!("expected a failed tab, got {other:?}"),
        }
        assert!(
            s.active().is_some_and(|x| x.history.is_empty()),
            "a rejected address must not enter history"
        );
        // The paths that were already right are unchanged.
        assert_eq!(
            s.go(t, "how do compositors work"),
            Outcome::NoSearchProvider {
                query: "how do compositors work".to_owned()
            }
        );
        assert!(matches!(
            s.go(t, "https://example.com/"),
            Outcome::Failed { .. }
        ));
    }

    #[test]
    fn typing_words_reports_that_there_is_no_search_provider() {
        // Silently sending everything typed to a search engine the user never
        // chose is the single most privacy-hostile default a browser has.
        let mut s = session();
        let t = s.open_tab();
        let out = s.go(t, "how do compositors work");
        assert_eq!(
            out,
            Outcome::NoSearchProvider {
                query: "how do compositors work".to_owned()
            }
        );
    }

    #[test]
    fn navigating_a_tab_that_is_gone_is_reported_not_ignored() {
        let mut s = session();
        let t = s.open_tab();
        s.close(t);
        assert_eq!(s.go(t, "https://example.com/"), Outcome::NoSuchTab);
    }

    #[test]
    fn the_reopen_list_is_bounded() {
        let mut s = session();
        for _ in 0..Session::REOPEN_CAPACITY + 5 {
            let id = s.open_tab();
            s.close(id);
        }
        let mut reopened = 0;
        while s.reopen().is_some() {
            reopened += 1;
        }
        assert_eq!(reopened, Session::REOPEN_CAPACITY);
    }

    #[test]
    fn status_lines_say_that_no_engine_is_integrated() {
        let s = session();
        let lines = s.lines();
        assert!(
            lines.contains(&"browser.engine=none".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"browser.engine_ready=false".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"browser.https_only=true".to_owned()),
            "{lines:?}"
        );
    }
}
