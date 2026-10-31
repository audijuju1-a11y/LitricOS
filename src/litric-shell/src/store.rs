//! The App Store's shell model and wire parser.
//!
//! This is a view model, not a package manager. It turns `STORE LIST` and
//! `STORE SEARCH` replies into rows, keeps the search text and selection, and
//! names the one confirmed write the page may ask for. The daemon owns every
//! process, capability check and consent check.

use crate::facts::unescape_field;

/// One Flatpak application as the daemon reported it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreApp {
    /// Reverse-DNS application id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Origin or offering remotes.
    pub origin: String,
    /// Installed size, when Flatpak reported one.
    pub size: String,
    /// The remote an install may name.
    pub remote: String,
}

/// A write waiting for the user's confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirm {
    /// Install this result from its own remote.
    Install(StoreApp),
    /// Remove this installed app.
    Uninstall(StoreApp),
}

/// A request the page asks `main` to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Read the installed system applications.
    List,
    /// Search the configured remotes.
    Search(String),
    /// Install one app from one remote.
    Install {
        /// Application id.
        id: String,
        /// Remote name.
        remote: String,
    },
    /// Remove one installed app.
    Uninstall {
        /// Application id.
        id: String,
    },
}

impl Request {
    /// The exact protocol line. The daemon validates it again.
    #[must_use]
    pub fn line(&self) -> String {
        match self {
            Self::List => "STORE LIST".to_owned(),
            Self::Search(query) => format!("STORE SEARCH {query}"),
            Self::Install { id, remote } => format!("STORE INSTALL {id} {remote} CONFIRM"),
            Self::Uninstall { id } => format!("STORE UNINSTALL {id} CONFIRM"),
        }
    }

    /// Whether a successful reply should make the page re-read installed rows.
    #[must_use]
    pub const fn is_write(&self) -> bool {
        matches!(self, Self::Install { .. } | Self::Uninstall { .. })
    }

    /// What the page says while the request is in flight.
    #[must_use]
    pub fn progress(&self) -> String {
        match self {
            Self::List => "Reading installed applications…".to_owned(),
            Self::Search(query) => format!("Searching for “{query}”…"),
            Self::Install { id, .. } => format!("Installing {id}…"),
            Self::Uninstall { id } => format!("Removing {id}…"),
        }
    }
}

/// The App Store page's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct View {
    /// Last catalog verdict: `ready`, `unavailable`, `error`, or empty.
    pub state: String,
    /// The daemon's own sentence for an unavailable or failed catalog.
    pub error: String,
    /// The text in the search field.
    pub query: String,
    /// Rows offered by the last search.
    pub results: Vec<StoreApp>,
    /// Installed system apps from the last list.
    pub installed: Vec<StoreApp>,
    /// Selected search result.
    pub selected_result: Option<usize>,
    /// Selected installed app.
    pub selected_installed: Option<usize>,
    /// A destructive write waiting for confirmation.
    pub confirm: Option<Confirm>,
    /// The last terminal sentence the page shows.
    pub note: String,
    /// Whether the search field has keyboard focus.
    pub search_focus: bool,
    /// Whether the page has asked for its first installed list.
    pub loaded: bool,
}

impl View {
    /// The selected result, if any.
    #[must_use]
    pub fn result(&self) -> Option<&StoreApp> {
        self.selected_result
            .and_then(|index| self.results.get(index))
    }

    /// The selected installed app, if any.
    #[must_use]
    pub fn installed_app(&self) -> Option<&StoreApp> {
        self.selected_installed
            .and_then(|index| self.installed.get(index))
    }

    /// Ask for a confirmation for the selected result.
    pub fn ask_install(&mut self) -> bool {
        let Some(app) = self.result().cloned() else {
            self.note = "Select a search result first.".to_owned();
            return false;
        };
        if app.remote.is_empty() {
            self.note = format!("{} did not name a remote to install from.", app.name);
            return false;
        }
        self.confirm = Some(Confirm::Install(app));
        self.note.clear();
        true
    }

    /// Ask for a confirmation for the selected installed app.
    pub fn ask_uninstall(&mut self) -> bool {
        let Some(app) = self.installed_app().cloned() else {
            self.note = "Select an installed application first.".to_owned();
            return false;
        };
        self.confirm = Some(Confirm::Uninstall(app));
        self.note.clear();
        true
    }

    /// The request a confirmed action becomes.
    #[must_use]
    pub fn confirmed_request(&self) -> Option<Request> {
        match self.confirm.as_ref()? {
            Confirm::Install(app) => Some(Request::Install {
                id: app.id.clone(),
                remote: app.remote.clone(),
            }),
            Confirm::Uninstall(app) => Some(Request::Uninstall { id: app.id.clone() }),
        }
    }

    /// Apply one daemon reply.
    ///
    /// A reply that is not a `store.state` answer leaves the previous rows in
    /// place and reports exactly what arrived. An empty successful answer is a
    /// real empty state, not the previous results.
    pub fn apply(&mut self, request: &Request, reply: Option<&str>) {
        let Some(reply) = reply else {
            self.state = "error".to_owned();
            self.error = "litric-sysd did not answer.".to_owned();
            self.note = self.error.clone();
            return;
        };
        if let Some(error) = reply.strip_prefix("ERR ") {
            self.state = "error".to_owned();
            self.error = error.trim().to_owned();
            self.note = self.error.clone();
            return;
        }
        let lines: Vec<&str> = reply.lines().collect();
        let state = field(&lines, "store.state").unwrap_or_default();
        let error = field(&lines, "store.error").map(unescape_field);
        if state == "unavailable" {
            self.state = state.to_owned();
            self.error = error.unwrap_or_else(|| "Flatpak is not available.".to_owned());
            self.note = self.error.clone();
            self.installed.clear();
            self.results.clear();
            self.selected_result = None;
            self.selected_installed = None;
            return;
        }
        if state == "error" {
            self.state = state.to_owned();
            self.error = error.unwrap_or_else(|| "Flatpak could not read the catalog.".to_owned());
            self.note = self.error.clone();
            return;
        }
        let apps = parse_apps(&lines);
        self.state = "ready".to_owned();
        self.error.clear();
        match request {
            Request::List => {
                self.installed = apps;
                self.loaded = true;
                self.selected_installed = self.installed.first().map(|_| 0);
                self.note = format!("{} installed system applications.", self.installed.len());
            }
            Request::Search(query) => {
                self.results = apps;
                self.selected_result = self.results.first().map(|_| 0);
                self.note = if self.results.is_empty() {
                    format!("No applications matched “{query}”.")
                } else {
                    format!("{} matching applications.", self.results.len())
                };
            }
            Request::Install { id, .. } => {
                self.confirm = None;
                self.note = format!("Installed {id}.");
            }
            Request::Uninstall { id } => {
                self.confirm = None;
                self.note = format!("Removed {id}.");
            }
        }
    }
}

fn field<'a>(lines: &'a [&'a str], key: &str) -> Option<&'a str> {
    lines.iter().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix('='))
    })
}

/// Parse `store.app id|name|version|origin|size|remote` lines.
#[must_use]
pub fn parse_apps(lines: &[&str]) -> Vec<StoreApp> {
    let mut apps = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("store.app ") else {
            continue;
        };
        let fields: Vec<&str> = rest.split('|').collect();
        if fields.len() != 6 {
            continue;
        }
        let at = |index: usize| fields.get(index).copied().unwrap_or_default();
        apps.push(StoreApp {
            id: unescape_field(at(0)),
            name: unescape_field(at(1)),
            version: unescape_field(at(2)),
            origin: unescape_field(at(3)),
            size: unescape_field(at(4)),
            remote: unescape_field(at(5)),
        });
    }
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_parsed_and_unecaped() {
        let lines = [
            "store.state=ready",
            "store.app org.gnome.Builder|Builder|46.1|flathub|1.2 GB|flathub",
            "store.app org.example.A|A%7CB|1|-|42 MB|flathub",
        ];
        let apps = parse_apps(&lines);
        assert_eq!(apps.len(), 2);
        let first = apps.first().expect("the first row");
        assert_eq!(first.name, "Builder");
        let second = apps.get(1).expect("the second row");
        assert_eq!(second.name, "A|B");
    }

    #[test]
    fn a_search_reply_replaces_rows_and_selects_the_first() {
        let mut view = View::default();
        view.apply(
            &Request::Search("builder".to_owned()),
            Some("store.state=ready\nstore.count=1\nstore.app org.gnome.Builder|Builder|46.1|flathub||flathub"),
        );
        assert_eq!(view.state, "ready");
        assert_eq!(view.results.len(), 1);
        assert_eq!(view.selected_result, Some(0));
        assert!(view.note.contains("1 matching"));
    }

    #[test]
    fn an_empty_search_is_a_real_empty_state() {
        let mut view = View::default();
        view.apply(
            &Request::Search("nothing".to_owned()),
            Some("store.state=ready\nstore.count=0"),
        );
        assert!(view.results.is_empty());
        assert_eq!(view.state, "ready");
        assert!(view.note.contains("No applications"));
    }

    #[test]
    fn unavailable_is_not_an_error_and_clears_controls() {
        let mut view = View {
            results: vec![StoreApp::default()],
            ..View::default()
        };
        view.apply(
            &Request::List,
            Some("store.state=unavailable\nstore.error=Flatpak%20is%20missing"),
        );
        assert_eq!(view.state, "unavailable");
        assert_eq!(view.error, "Flatpak is missing");
        assert!(view.results.is_empty());
    }

    #[test]
    fn install_requires_a_confirmation_and_keeps_the_remote() {
        let mut view = View {
            results: vec![StoreApp {
                id: "org.gnome.Builder".to_owned(),
                name: "Builder".to_owned(),
                remote: "flathub".to_owned(),
                ..StoreApp::default()
            }],
            selected_result: Some(0),
            ..View::default()
        };
        assert!(view.ask_install());
        assert_eq!(
            view.confirmed_request(),
            Some(Request::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "flathub".to_owned()
            })
        );
        view.apply(
            &Request::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "flathub".to_owned(),
            },
            Some("action=store.install\napplied=true"),
        );
        assert!(view.confirm.is_none());
        assert!(view.note.contains("Installed"));
    }
}
