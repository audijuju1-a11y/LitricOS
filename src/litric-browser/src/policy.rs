//! What a site is allowed to do, and who decides.
//!
//! A browser is the largest attack surface on a desktop and the one place where
//! a user routinely runs code written by strangers. LitricOS already has a
//! capability model for exactly this problem, so the browser does not invent a
//! second one: a site asking for the camera is an application asking for the
//! camera, and it goes through [`litric_caps`] like everything else.
//!
//! # The defaults are the policy
//!
//! Every permission below defaults to **deny**, and every one of them is scoped
//! to an origin rather than to a tab or a page. A permission granted to one
//! page of a site is granted to the site; pretending it is narrower would be
//! security theatre, because any page on a site can reach any other.
//!
//! # What is decided here and what is not
//!
//! This module decides *policy*. It does not enforce anything, because nothing
//! is running yet to enforce it against — see [`crate::engine`]. Writing the
//! policy first is deliberate: a permission model retrofitted onto a working
//! engine is a permission model with holes in it.

use litric_caps::CapabilityId;

use crate::url::{Scheme, Url};

/// Something a site can ask to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SitePermission {
    /// Read the camera.
    Camera,
    /// Read the microphone.
    Microphone,
    /// Read the device's location.
    Location,
    /// Post desktop notifications.
    Notifications,
    /// Read the clipboard. Writing is allowed on a user gesture; reading never
    /// is without permission, because a page that can read the clipboard can
    /// read whatever you last copied out of a password manager.
    ClipboardRead,
    /// Save files to disk.
    Downloads,
    /// Keep data after the tab closes.
    PersistentStorage,
    /// Play sound without a user gesture.
    Autoplay,
    /// Use the pointer-lock and fullscreen APIs.
    FullscreenPointer,
}

impl SitePermission {
    /// Every permission, in the order a settings page should list them.
    pub const ALL: [Self; 9] = [
        Self::Camera,
        Self::Microphone,
        Self::Location,
        Self::Notifications,
        Self::ClipboardRead,
        Self::Downloads,
        Self::PersistentStorage,
        Self::Autoplay,
        Self::FullscreenPointer,
    ];

    /// The name shown to the user.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Camera => "Camera",
            Self::Microphone => "Microphone",
            Self::Location => "Location",
            Self::Notifications => "Notifications",
            Self::ClipboardRead => "Read the clipboard",
            Self::Downloads => "Save files",
            Self::PersistentStorage => "Store data",
            Self::Autoplay => "Play sound automatically",
            Self::FullscreenPointer => "Full screen and pointer lock",
        }
    }

    /// The system capability this maps to, when there is one.
    ///
    /// Where a capability exists, the browser does not get to decide on its
    /// own: it asks the broker, and the broker's audit log records it. Where
    /// none exists yet the answer is `None` and the permission is browser-local
    /// — stated rather than hidden.
    #[must_use]
    pub const fn capability(self) -> Option<CapabilityId> {
        match self {
            Self::Camera => Some(CapabilityId::Camera),
            Self::Microphone => Some(CapabilityId::Microphone),
            Self::Location => Some(CapabilityId::Location),
            Self::Notifications => Some(CapabilityId::Notify),
            Self::Downloads => Some(CapabilityId::FsWrite),
            Self::ClipboardRead => Some(CapabilityId::ClipboardRead),
            Self::PersistentStorage | Self::Autoplay | Self::FullscreenPointer => None,
        }
    }

    /// Whether granting this needs an explicit prompt rather than a setting.
    ///
    /// The ones that read the physical world do. A prompt for everything trains
    /// people to click through prompts, which is how the ones that matter stop
    /// working.
    #[must_use]
    pub const fn needs_prompt(self) -> bool {
        matches!(
            self,
            Self::Camera | Self::Microphone | Self::Location | Self::ClipboardRead
        )
    }

    /// Whether an insecure origin may ever hold it.
    ///
    /// Nothing that reads the physical world or persists data is available over
    /// plain HTTP: anybody on the path can both see the answer and forge the
    /// request.
    #[must_use]
    pub const fn allowed_on_insecure(self) -> bool {
        matches!(self, Self::Autoplay | Self::FullscreenPointer)
    }
}

/// What a site was granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Decision {
    /// Never asked.
    Ask,
    /// Allowed.
    Allow,
    /// Refused.
    Deny,
}

/// Browser-wide privacy settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyPolicy {
    /// Refuse plain HTTP and offer to upgrade.
    pub https_only: bool,
    /// Refuse cookies set by a site other than the one being visited.
    pub block_third_party_cookies: bool,
    /// Refuse requests to hosts on the tracker list.
    pub block_trackers: bool,
    /// Send a Do Not Track signal. Off by default and documented as
    /// near-useless: it is a request nobody is obliged to honour, and it makes
    /// a browser more identifiable rather than less.
    pub do_not_track: bool,
    /// Clear cookies and storage when the browser closes.
    pub clear_on_exit: bool,
    /// Run scripts at all.
    pub javascript: bool,
}

impl Default for PrivacyPolicy {
    /// The defaults are the strict ones.
    ///
    /// A browser whose privacy settings have to be turned on is a browser whose
    /// privacy settings are off for almost everybody.
    fn default() -> Self {
        Self {
            https_only: true,
            block_third_party_cookies: true,
            block_trackers: true,
            do_not_track: false,
            clear_on_exit: false,
            javascript: true,
        }
    }
}

/// Why a navigation was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refusal {
    /// The policy requires HTTPS and this was plain HTTP.
    InsecureBlocked {
        /// The address that would work instead.
        upgrade_to: String,
    },
    /// The host is on the tracker list.
    Tracker {
        /// Which host.
        host: String,
    },
    /// The scheme is one the browser will not open.
    SchemeRefused {
        /// Which scheme.
        scheme: String,
    },
}

impl Refusal {
    /// What to tell the user.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::InsecureBlocked { upgrade_to } => format!(
                "This site was asked for over an unencrypted connection. \
                 Anybody on the network could read or change it. \
                 The encrypted address is {upgrade_to}."
            ),
            Self::Tracker { host } => {
                format!("{host} is on the tracker list and was not contacted.")
            }
            Self::SchemeRefused { scheme } => {
                format!("Litric Browser does not open {scheme}: addresses.")
            }
        }
    }
}

/// One site's settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteSettings {
    /// The origin these apply to.
    pub origin: String,
    /// Per-permission decisions, in [`SitePermission::ALL`] order.
    decisions: Vec<(SitePermission, Decision)>,
}

impl SiteSettings {
    /// Fresh settings for an origin: everything unasked.
    #[must_use]
    pub fn new(origin: &str) -> Self {
        Self {
            origin: origin.to_owned(),
            decisions: Vec::new(),
        }
    }

    /// What was decided for a permission.
    #[must_use]
    pub fn decision(&self, p: SitePermission) -> Decision {
        self.decisions
            .iter()
            .find(|(k, _)| *k == p)
            .map_or(Decision::Ask, |(_, d)| *d)
    }

    /// Record a decision.
    pub fn set(&mut self, p: SitePermission, d: Decision) {
        if let Some(slot) = self.decisions.iter_mut().find(|(k, _)| *k == p) {
            slot.1 = d;
        } else {
            self.decisions.push((p, d));
        }
    }

    /// Everything that was decided, for a settings page.
    #[must_use]
    pub fn decided(&self) -> &[(SitePermission, Decision)] {
        &self.decisions
    }
}

/// The browser's policy engine.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    /// Browser-wide settings.
    pub privacy: PrivacyPolicy,
    sites: Vec<SiteSettings>,
    trackers: Vec<String>,
}

impl Policy {
    /// A policy with the default settings and a tracker list.
    #[must_use]
    pub fn new(trackers: Vec<String>) -> Self {
        Self {
            privacy: PrivacyPolicy::default(),
            sites: Vec::new(),
            trackers: trackers.into_iter().map(|t| t.to_lowercase()).collect(),
        }
    }

    /// Whether a host is on the tracker list.
    ///
    /// Matches the host and any subdomain of it, so blocking `tracker.test`
    /// also blocks `a.b.tracker.test` — which is how these are actually served.
    #[must_use]
    pub fn is_tracker(&self, host: &str) -> bool {
        let h = host.to_lowercase();
        self.trackers
            .iter()
            .any(|t| h == *t || h.ends_with(&format!(".{t}")))
    }

    /// Whether a navigation is allowed, and why not when it is not.
    ///
    /// # Errors
    /// A [`Refusal`] the interface can show, with an alternative where one
    /// exists.
    pub fn check_navigation(&self, url: &Url) -> Result<(), Refusal> {
        if self.privacy.block_trackers && self.is_tracker(&url.host) {
            return Err(Refusal::Tracker {
                host: url.host.clone(),
            });
        }
        if self.privacy.https_only && url.scheme == Scheme::Http {
            let mut upgraded = url.clone();
            upgraded.scheme = Scheme::Https;
            // The default port for the old scheme must not be carried over, or
            // the upgrade offers https on port 80.
            if upgraded.port == Some(80) {
                upgraded.port = None;
            }
            return Err(Refusal::InsecureBlocked {
                upgrade_to: upgraded.to_url(),
            });
        }
        Ok(())
    }

    /// Whether a site may use a permission right now.
    #[must_use]
    pub fn permission(&self, url: &Url, p: SitePermission) -> Decision {
        if !url.is_secure() && !p.allowed_on_insecure() {
            // Not "Ask": there is no answer the user could give that would make
            // this safe, so offering the choice would be dishonest.
            return Decision::Deny;
        }
        self.site(&url.origin())
            .map_or(Decision::Ask, |s| s.decision(p))
    }

    /// Record a decision for a site.
    pub fn decide(&mut self, url: &Url, p: SitePermission, d: Decision) {
        let origin = url.origin();
        if let Some(s) = self.sites.iter_mut().find(|s| s.origin == origin) {
            s.set(p, d);
        } else {
            let mut s = SiteSettings::new(&origin);
            s.set(p, d);
            self.sites.push(s);
        }
    }

    /// One site's settings.
    #[must_use]
    pub fn site(&self, origin: &str) -> Option<&SiteSettings> {
        self.sites.iter().find(|s| s.origin == origin)
    }

    /// Every site that has been decided about, for a settings page.
    #[must_use]
    pub fn sites(&self) -> &[SiteSettings] {
        &self.sites
    }

    /// Forget everything about a site.
    pub fn forget(&mut self, origin: &str) -> bool {
        let before = self.sites.len();
        self.sites.retain(|s| s.origin != origin);
        before != self.sites.len()
    }

    /// Lines for the status file and diagnostics.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("browser.https_only={}", self.privacy.https_only),
            format!("browser.block_trackers={}", self.privacy.block_trackers),
            format!(
                "browser.block_third_party_cookies={}",
                self.privacy.block_third_party_cookies
            ),
            format!("browser.javascript={}", self.privacy.javascript),
            format!("browser.tracker_hosts={}", self.trackers.len()),
            format!("browser.sites_decided={}", self.sites.len()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect(s)
    }

    fn policy() -> Policy {
        Policy::new(vec!["tracker.test".to_owned(), "ads.example".to_owned()])
    }

    #[test]
    fn every_permission_starts_at_ask_and_nothing_is_granted_by_default() {
        let p = policy();
        let u = url("https://example.com/");
        for perm in SitePermission::ALL {
            assert_eq!(
                p.permission(&u, perm),
                Decision::Ask,
                "{perm:?} was not Ask by default"
            );
        }
    }

    #[test]
    fn a_decision_applies_to_the_whole_origin_not_the_page() {
        let mut p = policy();
        p.decide(
            &url("https://example.com/a"),
            SitePermission::Camera,
            Decision::Allow,
        );
        assert_eq!(
            p.permission(&url("https://example.com/b"), SitePermission::Camera),
            Decision::Allow,
            "a permission must follow the origin"
        );
        assert_eq!(
            p.permission(&url("https://other.com/a"), SitePermission::Camera),
            Decision::Ask,
            "a permission must not leak to another origin"
        );
    }

    #[test]
    fn a_different_scheme_is_a_different_origin() {
        let mut p = policy();
        p.privacy.https_only = false;
        p.decide(
            &url("https://example.com/"),
            SitePermission::Notifications,
            Decision::Allow,
        );
        assert_eq!(
            p.permission(&url("http://example.com/"), SitePermission::Notifications),
            Decision::Deny,
            "an insecure origin must not inherit a secure one's grants"
        );
    }

    #[test]
    fn an_insecure_origin_can_never_hold_a_sensitive_permission() {
        let mut p = policy();
        p.privacy.https_only = false;
        let u = url("http://example.com/");
        // Even after an explicit grant.
        p.decide(&u, SitePermission::Camera, Decision::Allow);
        assert_eq!(
            p.permission(&u, SitePermission::Camera),
            Decision::Deny,
            "plain HTTP was allowed to use the camera"
        );
        // The harmless ones still work.
        p.decide(&u, SitePermission::Autoplay, Decision::Allow);
        assert_eq!(p.permission(&u, SitePermission::Autoplay), Decision::Allow);
    }

    #[test]
    fn https_only_refuses_http_and_offers_the_upgrade() {
        let p = policy();
        let err = p
            .check_navigation(&url("http://example.com/page"))
            .expect_err("should be refused");
        assert_eq!(
            err,
            Refusal::InsecureBlocked {
                upgrade_to: "https://example.com/page".to_owned()
            }
        );
        assert!(err.message().contains("https://example.com/page"));
    }

    #[test]
    fn upgrading_drops_the_old_default_port() {
        // Otherwise the offered address is https on port 80, which fails.
        let p = policy();
        let err = p
            .check_navigation(&url("http://example.com:80/x"))
            .expect_err("refused");
        assert_eq!(
            err,
            Refusal::InsecureBlocked {
                upgrade_to: "https://example.com/x".to_owned()
            }
        );
    }

    #[test]
    fn a_non_default_port_survives_the_upgrade() {
        let p = policy();
        let err = p
            .check_navigation(&url("http://example.com:8080/x"))
            .expect_err("refused");
        assert_eq!(
            err,
            Refusal::InsecureBlocked {
                upgrade_to: "https://example.com:8080/x".to_owned()
            }
        );
    }

    #[test]
    fn https_is_allowed() {
        assert!(policy()
            .check_navigation(&url("https://example.com/"))
            .is_ok());
    }

    #[test]
    fn trackers_are_blocked_including_their_subdomains() {
        let p = policy();
        assert!(p.is_tracker("tracker.test"));
        assert!(p.is_tracker("cdn.a.tracker.test"), "a subdomain must match");
        assert!(p.is_tracker("TRACKER.TEST"), "matching is case-insensitive");
        assert!(!p.is_tracker("nottracker.test"), "a suffix must not match");
        assert!(!p.is_tracker("example.com"));
        assert!(matches!(
            p.check_navigation(&url("https://tracker.test/pixel")),
            Err(Refusal::Tracker { .. })
        ));
    }

    #[test]
    fn turning_tracker_blocking_off_lets_them_through() {
        let mut p = policy();
        p.privacy.block_trackers = false;
        assert!(p
            .check_navigation(&url("https://tracker.test/pixel"))
            .is_ok());
    }

    #[test]
    fn the_defaults_are_the_strict_ones() {
        let d = PrivacyPolicy::default();
        assert!(d.https_only, "https-only must be on by default");
        assert!(d.block_third_party_cookies);
        assert!(d.block_trackers);
        assert!(!d.do_not_track, "an unenforceable request is not a default");
    }

    #[test]
    fn permissions_that_read_the_world_map_to_system_capabilities() {
        // The browser does not get its own private permission model for the
        // things the rest of the system already governs.
        assert!(SitePermission::Camera.capability().is_some());
        assert!(SitePermission::Microphone.capability().is_some());
        assert!(SitePermission::Location.capability().is_some());
        assert!(SitePermission::Downloads.capability().is_some());
        // The browser-local ones are honestly marked as such.
        assert!(SitePermission::Autoplay.capability().is_none());
    }

    #[test]
    fn only_the_physical_world_permissions_prompt() {
        assert!(SitePermission::Camera.needs_prompt());
        assert!(SitePermission::ClipboardRead.needs_prompt());
        assert!(!SitePermission::Autoplay.needs_prompt());
        assert!(!SitePermission::PersistentStorage.needs_prompt());
    }

    #[test]
    fn forgetting_a_site_removes_everything_about_it() {
        let mut p = policy();
        let u = url("https://example.com/");
        p.decide(&u, SitePermission::Notifications, Decision::Deny);
        assert!(p.site("https://example.com").is_some());
        assert!(p.forget("https://example.com"));
        assert_eq!(
            p.permission(&u, SitePermission::Notifications),
            Decision::Ask
        );
        assert!(
            !p.forget("https://example.com"),
            "forgetting twice is not an error"
        );
    }

    #[test]
    fn changing_a_decision_replaces_it_rather_than_stacking() {
        let mut p = policy();
        let u = url("https://example.com/");
        p.decide(&u, SitePermission::Camera, Decision::Allow);
        p.decide(&u, SitePermission::Camera, Decision::Deny);
        assert_eq!(p.permission(&u, SitePermission::Camera), Decision::Deny);
        assert_eq!(
            p.site("https://example.com").map(|s| s.decided().len()),
            Some(1)
        );
    }

    #[test]
    fn a_default_port_cannot_step_around_a_refusal() {
        // The origin is the permission key, and `https://site:443` is the same
        // origin as `https://site`. Before the parser normalised it, the second
        // spelling missed the stored Deny and fell back to Ask.
        let mut p = policy();
        p.decide(
            &url("https://example.com/"),
            SitePermission::Camera,
            Decision::Deny,
        );
        assert_eq!(
            p.permission(&url("https://example.com:443/"), SitePermission::Camera),
            Decision::Deny,
            "the default port must not create a second origin"
        );
    }

    #[test]
    fn a_tracker_cannot_hide_behind_a_default_port_or_a_bad_one() {
        let p = policy();
        assert!(matches!(
            p.check_navigation(&url("https://tracker.test:443/pixel")),
            Err(Refusal::Tracker { .. })
        ));
        // A non-numeric port no longer becomes part of the host, so the address
        // is refused as malformed before any host-based decision is made.
        assert!(Url::parse("https://tracker.test:abc/pixel").is_err());
    }

    #[test]
    fn status_lines_report_the_policy_in_force() {
        let lines = policy().lines();
        assert!(
            lines.contains(&"browser.https_only=true".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"browser.tracker_hosts=2".to_owned()),
            "{lines:?}"
        );
    }
}
