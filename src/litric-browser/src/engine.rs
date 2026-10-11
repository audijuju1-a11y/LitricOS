//! Where a rendering engine plugs in.
//!
//! # This is a boundary, not a placeholder
//!
//! Nothing behind this trait will ever be written here. A modern web engine is
//! millions of lines, a permanent security commitment, and the single largest
//! attack surface on any desktop. Writing one from scratch would be reckless in
//! a way no amount of care could fix, and pretending to have one would be worse.
//!
//! So the engine is an integration point. What LitricOS owns is everything
//! around it, which is where a browser's character actually lives: how an
//! address is read, what a site is allowed to do, what happens when a request
//! is refused, and how any of that is presented. Those are decisions, and this
//! project makes them; rendering HTML is engineering somebody else has already
//! done well.
//!
//! # Until then
//!
//! [`NoEngine`] is the only implementation, and it loads nothing. Every
//! navigation fails with a message naming the missing piece. A browser window
//! that showed a convincing blank page would be a lie, and one that showed a
//! hand-drawn imitation of a website would be a worse one.

use crate::url::Url;

/// What an engine returns when a page loads.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Page {
    /// The document's title, when it has one.
    pub title: Option<String>,
    /// Bytes received, for the diagnostics.
    pub bytes: u64,
    /// Whether the connection was encrypted end to end.
    pub secure: bool,
}

/// A rendering engine.
///
/// Deliberately small. Everything a browser does *around* rendering — history,
/// permissions, the address bar, refusals — lives outside this trait, so
/// swapping the engine cannot change any of it.
pub trait Engine: core::fmt::Debug + Send {
    /// The engine's name, for the diagnostics and the about page.
    fn name(&self) -> &str;

    /// Whether it can actually load anything.
    fn is_ready(&self) -> bool;

    /// Load an address.
    ///
    /// # Errors
    /// A message for the user explaining why the page is not there.
    fn load(&mut self, url: &Url) -> Result<Page, String>;
}

/// No engine is integrated.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoEngine;

impl Engine for NoEngine {
    fn name(&self) -> &str {
        "none"
    }

    fn is_ready(&self) -> bool {
        false
    }

    fn load(&mut self, url: &Url) -> Result<Page, String> {
        Err(format!(
            "No rendering engine is integrated, so {} was not fetched. \
             Litric Browser's interface, tab model, address handling and privacy \
             policy are implemented; the engine is integrated rather than written, \
             and none is present in this image yet.",
            url.origin()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_null_engine_is_honest_about_itself() {
        let mut e = NoEngine;
        assert_eq!(e.name(), "none");
        assert!(!e.is_ready());
        let err = e
            .load(&Url::parse("https://example.com/").expect("url"))
            .expect_err("must not pretend to load");
        assert!(err.contains("No rendering engine"), "{err}");
        assert!(err.contains("https://example.com"), "{err}");
    }

    #[test]
    fn every_navigation_fails_the_same_way_rather_than_sometimes_succeeding() {
        let mut e = NoEngine;
        for u in [
            "https://a.test/",
            "https://b.test/x",
            "file:///etc/hostname",
        ] {
            assert!(
                e.load(&Url::parse(u).expect(u)).is_err(),
                "{u} appeared to load"
            );
        }
    }
}
