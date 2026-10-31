//! The App Sandbox page's shell model and wire parser.
//!
//! This is a view model. It turns `STORE UPDATES`, `STORE SANDBOX <id>` and the
//! confirmed writes into rows, a selection and one pending confirmation. Every
//! process, capability check and consent check belongs to `litric-sysd`; this
//! module never starts Flatpak and never invents a permission the daemon did
//! not report.

use crate::facts::unescape_field;
use crate::store::{parse_apps, StoreApp};

/// The two system overrides this page can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Access to the host's filesystem.
    HostFilesystem,
    /// The network namespace.
    Network,
}

impl Key {
    /// Both keys, in the order the page draws them.
    pub const ALL: [Self; 2] = [Self::HostFilesystem, Self::Network];

    /// The token the protocol and the daemon use.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::HostFilesystem => "host-filesystem",
            Self::Network => "network",
        }
    }

    /// The words on the page.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::HostFilesystem => "Host filesystem",
            Self::Network => "Network",
        }
    }

    /// The `(kind, value)` an effective fact uses for this grant.
    #[must_use]
    pub const fn grant(self) -> (&'static str, &'static str) {
        match self {
            Self::HostFilesystem => ("filesystem", "host"),
            Self::Network => ("shared", "network"),
        }
    }

    /// Parse the protocol token.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|key| key.token() == raw)
    }
}

/// One application update as the daemon reported it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Update {
    /// Reverse-DNS application id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Version being offered.
    pub version: String,
    /// The update's branch.
    pub branch: String,
    /// The remote it comes from.
    pub origin: String,
}

/// One discrete permission fact.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Permission {
    /// `base`, `global`, `app` or `effective`.
    pub source: String,
    /// `filesystem`, `socket`, `device`, `shared` or `unshare`.
    pub kind: String,
    /// The entry itself.
    pub value: String,
}

/// A write waiting for confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirm {
    /// Apply every listed update.
    UpdateAll,
    /// Apply one application's update.
    Update(Update),
    /// Set one override for one application.
    Set {
        /// Application id.
        id: String,
        /// Display name.
        name: String,
        /// Which access.
        key: Key,
        /// Grant (`true`) or remove (`false`).
        on: bool,
    },
    /// Remove the system overrides for one application.
    Reset {
        /// Application id.
        id: String,
        /// Display name.
        name: String,
    },
}

/// A request the page asks `main` to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Read the available updates.
    Updates,
    /// Read installed system applications to choose one from.
    Apps,
    /// Read one application's effective permissions.
    Permissions(String),
    /// Apply every listed update.
    UpdateAll,
    /// Apply one application's update.
    Update {
        /// Application id.
        id: String,
    },
    /// Set one system override.
    Set {
        /// Application id.
        id: String,
        /// Which access.
        key: Key,
        /// Grant or remove.
        on: bool,
    },
    /// Reset one application's system overrides.
    Reset {
        /// Application id.
        id: String,
    },
}

impl Request {
    /// The exact protocol line. The daemon validates it again.
    #[must_use]
    pub fn line(&self) -> String {
        match self {
            Self::Updates => "STORE UPDATES".to_owned(),
            Self::Apps => "STORE LIST".to_owned(),
            Self::Permissions(id) => format!("STORE SANDBOX {id}"),
            Self::UpdateAll => "STORE UPDATE all CONFIRM".to_owned(),
            Self::Update { id } => format!("STORE UPDATE {id} CONFIRM"),
            Self::Set { id, key, on } => format!(
                "STORE SANDBOX SET {id} {} {} CONFIRM",
                key.token(),
                if *on { "on" } else { "off" }
            ),
            Self::Reset { id } => format!("STORE SANDBOX RESET {id} CONFIRM"),
        }
    }

    /// Whether a successful reply should make the page re-read the machine.
    #[must_use]
    pub const fn is_write(&self) -> bool {
        matches!(
            self,
            Self::UpdateAll | Self::Update { .. } | Self::Set { .. } | Self::Reset { .. }
        )
    }

    /// The installed application a write names, if any.
    #[must_use]
    pub fn app_id(&self) -> Option<&str> {
        match self {
            Self::Permissions(id)
            | Self::Update { id }
            | Self::Set { id, .. }
            | Self::Reset { id } => Some(id),
            Self::Updates | Self::Apps | Self::UpdateAll => None,
        }
    }

    /// What the page says while the request is in flight.
    #[must_use]
    pub fn progress(&self) -> String {
        match self {
            Self::Updates => "Reading available updates…".to_owned(),
            Self::Apps => "Reading installed applications…".to_owned(),
            Self::Permissions(id) => format!("Reading the sandbox for {id}…"),
            Self::UpdateAll => "Applying every listed update…".to_owned(),
            Self::Update { id } => format!("Updating {id}…"),
            Self::Set { id, key, on } => format!(
                "{} {} for {id}…",
                if *on { "Allowing" } else { "Denying" },
                key.title().to_lowercase()
            ),
            Self::Reset { id } => format!("Resetting system overrides for {id}…"),
        }
    }
}

/// The App Sandbox page's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct View {
    /// Last update verdict: `ready`, `unavailable`, `error`, or empty.
    pub state: String,
    /// The daemon's own sentence for an unavailable or failed update read.
    pub error: String,
    /// The last terminal sentence the page shows.
    pub note: String,
    /// Updates Flatpak offers.
    pub updates: Vec<Update>,
    /// Installed system applications to choose from.
    pub apps: Vec<StoreApp>,
    /// Selected installed application.
    pub selected_app: Option<usize>,
    /// Effective and attributed permission facts.
    pub permissions: Vec<Permission>,
    /// Last permission verdict.
    pub perm_state: String,
    /// The daemon's own sentence for an unavailable or failed permission read.
    pub perm_error: String,
    /// A write waiting for confirmation.
    pub confirm: Option<Confirm>,
    /// Whether the page has asked for its first update read.
    pub loaded: bool,
}

impl View {
    /// The selected installed application, if any.
    #[must_use]
    pub fn app(&self) -> Option<&StoreApp> {
        self.selected_app.and_then(|index| self.apps.get(index))
    }

    /// The selected update row, if any.
    #[must_use]
    pub fn update(&self, index: usize) -> Option<&Update> {
        self.updates.get(index)
    }

    /// Whether an access is currently granted, according to the effective set.
    ///
    /// `None` when the effective set was not read: a switch drawn from no
    /// measurement would be the page claiming a state the machine never
    /// reported. An `unshare` fact in an overlay is not used here because the
    /// `effective` rows are the merged answer, which already excludes it.
    #[must_use]
    pub fn granted(&self, key: Key) -> Option<bool> {
        if self.perm_state != "ready" {
            return None;
        }
        let (kind, value) = key.grant();
        Some(
            self.permissions
                .iter()
                .any(|fact| fact.source == "effective" && fact.kind == kind && fact.value == value),
        )
    }

    /// Ask for a confirmation to apply every listed update.
    pub fn ask_update_all(&mut self) {
        if self.updates.is_empty() {
            self.note = "There are no listed updates to apply.".to_owned();
            return;
        }
        self.confirm = Some(Confirm::UpdateAll);
        self.note.clear();
    }

    /// Ask for a confirmation to apply one update row.
    pub fn ask_update(&mut self, index: usize) {
        let Some(update) = self.updates.get(index).cloned() else {
            self.note = "Select an update first.".to_owned();
            return;
        };
        self.confirm = Some(Confirm::Update(update));
        self.note.clear();
    }

    /// Ask for a confirmation to flip one access for the selected application.
    pub fn ask_toggle(&mut self, key: Key) {
        let Some(app) = self.app().cloned() else {
            self.note = "Select an installed application first.".to_owned();
            return;
        };
        let Some(on) = self.granted(key) else {
            self.note =
                "The current sandbox has not been read, so it cannot be changed.".to_owned();
            return;
        };
        self.confirm = Some(Confirm::Set {
            id: app.id,
            name: app.name,
            key,
            on: !on,
        });
        self.note.clear();
    }

    /// Ask for a confirmation to reset the selected application's overrides.
    pub fn ask_reset(&mut self) {
        let Some(app) = self.app().cloned() else {
            self.note = "Select an installed application first.".to_owned();
            return;
        };
        self.confirm = Some(Confirm::Reset {
            id: app.id,
            name: app.name,
        });
        self.note.clear();
    }

    /// The request a confirmed action becomes.
    #[must_use]
    pub fn confirmed_request(&self) -> Option<Request> {
        match self.confirm.as_ref()? {
            Confirm::UpdateAll => Some(Request::UpdateAll),
            Confirm::Update(update) => Some(Request::Update {
                id: update.id.clone(),
            }),
            Confirm::Set { id, key, on, .. } => Some(Request::Set {
                id: id.clone(),
                key: *key,
                on: *on,
            }),
            Confirm::Reset { id, .. } => Some(Request::Reset { id: id.clone() }),
        }
    }

    /// Apply one daemon reply.
    ///
    /// A reply that is not a state answer leaves the previous rows in place and
    /// shows exactly what arrived. A missing answer is an error, never an empty
    /// list: "Flatpak did not answer" and "there are no updates" are different
    /// facts about the machine.
    pub fn apply(&mut self, request: &Request, reply: Option<&str>) {
        let Some(reply) = reply else {
            self.note = "litric-sysd did not answer: nothing was read or changed.".to_owned();
            if matches!(request, Request::Updates | Request::Apps) {
                self.state = "error".to_owned();
                self.error = self.note.clone();
            } else if matches!(request, Request::Permissions(_)) {
                self.perm_state = "error".to_owned();
                self.perm_error = self.note.clone();
            }
            self.confirm = None;
            return;
        };
        if let Some(error) = reply.strip_prefix("ERR ") {
            self.note = error.trim().to_owned();
            if matches!(request, Request::Updates | Request::Apps) {
                self.state = "error".to_owned();
                self.error = self.note.clone();
            } else if matches!(request, Request::Permissions(_)) {
                self.perm_state = "error".to_owned();
                self.perm_error = self.note.clone();
            }
            self.confirm = None;
            return;
        }
        let lines: Vec<&str> = reply.lines().collect();

        // `STORE LIST` answers in the store's own vocabulary because it is the
        // same read the App Store page uses; the sandbox page only borrows its
        // rows, not its model.
        if matches!(request, Request::Apps) {
            let state = field(&lines, "store.state").unwrap_or_default();
            if state == "ready" {
                self.apps = parse_apps(&lines);
                self.selected_app = self.apps.first().map(|_| 0);
                self.state = "ready".to_owned();
                self.error.clear();
                if self.apps.is_empty() {
                    self.note = "No system applications are installed.".to_owned();
                }
            } else {
                self.state = state.to_owned();
                self.error = field(&lines, "store.error")
                    .map(unescape_field)
                    .unwrap_or_else(|| "The installed applications could not be read.".to_owned());
                self.note = self.error.clone();
                self.apps.clear();
                self.selected_app = None;
            }
            return;
        }

        // Writes answer with `action=` and `applied=` lines; only the two reads
        // carry a `sandbox.state`. Reading a state from a write reply would
        // report "no state" over a write that actually happened.
        if matches!(request, Request::Updates | Request::Permissions(_)) {
            let state = field(&lines, "sandbox.state").unwrap_or_default();
            if state == "unavailable" || state == "error" || state.is_empty() {
                let error = field(&lines, "sandbox.error")
                    .map(unescape_field)
                    .unwrap_or_else(|| {
                        if state.is_empty() {
                            "The daemon's answer did not name a sandbox state.".to_owned()
                        } else {
                            "Flatpak is not available.".to_owned()
                        }
                    });
                if matches!(request, Request::Permissions(_)) {
                    self.perm_state = if state.is_empty() { "error" } else { state }.to_owned();
                    self.perm_error = error.clone();
                    self.permissions.clear();
                } else {
                    self.state = if state.is_empty() { "error" } else { state }.to_owned();
                    self.error = error.clone();
                    self.updates.clear();
                }
                self.note = error;
                self.confirm = None;
                return;
            }
        }

        match request {
            Request::Updates => {
                self.updates = parse_updates(&lines);
                self.state = "ready".to_owned();
                self.error.clear();
                self.loaded = true;
                self.note = if self.updates.is_empty() {
                    "Every installed system application is up to date.".to_owned()
                } else {
                    format!("{} updates are available.", self.updates.len())
                };
            }
            Request::Permissions(id) => {
                self.permissions = parse_permissions(&lines);
                self.perm_state = "ready".to_owned();
                self.perm_error.clear();
                self.note = format!("{} permission facts read for {id}.", self.permissions.len());
            }
            Request::UpdateAll => {
                self.confirm = None;
                self.note = match applied(&lines) {
                    Some(false) => "The update was authorised but not applied.".to_owned(),
                    _ => "Every listed update was applied.".to_owned(),
                };
            }
            Request::Update { id } => {
                self.confirm = None;
                self.note = match applied(&lines) {
                    Some(false) => format!("{id} was authorised but not updated."),
                    _ => format!("Updated {id}."),
                };
            }
            Request::Set { id, key, on } => {
                self.confirm = None;
                self.note = match applied(&lines) {
                    Some(false) => {
                        format!("The {} override for {id} was not changed.", key.title())
                    }
                    _ => format!(
                        "{} {} for {id}.",
                        if *on { "Allowed" } else { "Denied" },
                        key.title().to_lowercase()
                    ),
                };
            }
            Request::Reset { id } => {
                self.confirm = None;
                self.note = match applied(&lines) {
                    Some(false) => format!("The overrides for {id} were not reset."),
                    _ => format!("Reset the system overrides for {id}."),
                };
            }
            Request::Apps => {}
        }
    }
}

fn field<'a>(lines: &'a [&'a str], key: &str) -> Option<&'a str> {
    lines.iter().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix('='))
    })
}

fn applied(lines: &[&str]) -> Option<bool> {
    field(lines, "applied").map(|value| value == "true")
}

/// Parse `sandbox.update id|name|version|branch|origin` rows.
#[must_use]
pub fn parse_updates(lines: &[&str]) -> Vec<Update> {
    let mut updates = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("sandbox.update ") else {
            continue;
        };
        let fields: Vec<&str> = rest.split('|').collect();
        if fields.len() != 5 {
            continue;
        }
        let at = |index: usize| fields.get(index).copied().unwrap_or_default();
        updates.push(Update {
            id: unescape_field(at(0)),
            name: unescape_field(at(1)),
            version: unescape_field(at(2)),
            branch: unescape_field(at(3)),
            origin: unescape_field(at(4)),
        });
    }
    updates
}

/// Parse `sandbox.perm source|kind|value` rows.
#[must_use]
pub fn parse_permissions(lines: &[&str]) -> Vec<Permission> {
    let mut permissions = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("sandbox.perm ") else {
            continue;
        };
        let fields: Vec<&str> = rest.split('|').collect();
        if fields.len() != 3 {
            continue;
        }
        let at = |index: usize| fields.get(index).copied().unwrap_or_default();
        permissions.push(Permission {
            source: unescape_field(at(0)),
            kind: unescape_field(at(1)),
            value: unescape_field(at(2)),
        });
    }
    permissions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::StoreApp;

    fn app() -> StoreApp {
        StoreApp {
            id: "org.gnome.Builder".to_owned(),
            name: "Builder".to_owned(),
            version: "46.2".to_owned(),
            origin: "flathub".to_owned(),
            size: "1.2 GB".to_owned(),
            remote: "flathub".to_owned(),
        }
    }

    fn ready_view() -> View {
        View {
            state: "ready".to_owned(),
            updates: vec![Update {
                id: "org.gnome.Builder".to_owned(),
                name: "Builder".to_owned(),
                version: "46.2".to_owned(),
                branch: "stable".to_owned(),
                origin: "flathub".to_owned(),
            }],
            apps: vec![app()],
            selected_app: Some(0),
            permissions: vec![Permission {
                source: "effective".to_owned(),
                kind: "filesystem".to_owned(),
                value: "host".to_owned(),
            }],
            perm_state: "ready".to_owned(),
            ..View::default()
        }
    }

    #[test]
    fn update_rows_are_parsed_with_every_field() {
        let rows = parse_updates(&[
            "sandbox.state=ready",
            "sandbox.update org.gnome.Builder|Builder|46.2|stable|flathub",
        ]);
        assert_eq!(rows.len(), 1);
        let update = rows.first().expect("one update");
        assert_eq!(update.id, "org.gnome.Builder");
        assert_eq!(update.version, "46.2");
        assert_eq!(update.branch, "stable");
        assert_eq!(update.origin, "flathub");
    }

    #[test]
    fn permission_rows_keep_their_source_instead_of_a_blob() {
        let facts = parse_permissions(&[
            "sandbox.perm base|filesystem|host",
            "sandbox.perm global|unshare|host",
            "sandbox.perm effective|socket|x11",
        ]);
        assert_eq!(facts.len(), 3);
        assert_eq!(facts.first().map(|f| f.source.as_str()), Some("base"));
        assert_eq!(facts.get(1).map(|f| f.kind.as_str()), Some("unshare"));
        assert!(facts.iter().all(|fact| !fact.value.contains(';')));
    }

    #[test]
    fn requests_build_exact_protocol_lines() {
        assert_eq!(Request::Updates.line(), "STORE UPDATES");
        assert_eq!(Request::Apps.line(), "STORE LIST");
        assert_eq!(
            Request::Permissions("org.gnome.Builder".to_owned()).line(),
            "STORE SANDBOX org.gnome.Builder"
        );
        assert_eq!(Request::UpdateAll.line(), "STORE UPDATE all CONFIRM");
        assert_eq!(
            Request::Set {
                id: "org.gnome.Builder".to_owned(),
                key: Key::Network,
                on: false
            }
            .line(),
            "STORE SANDBOX SET org.gnome.Builder network off CONFIRM"
        );
        assert_eq!(
            Request::Reset {
                id: "org.gnome.Builder".to_owned()
            }
            .line(),
            "STORE SANDBOX RESET org.gnome.Builder CONFIRM"
        );
    }

    #[test]
    fn a_toggle_asks_for_the_opposite_of_the_effective_grant() {
        let mut view = ready_view();
        assert_eq!(view.granted(Key::HostFilesystem), Some(true));
        assert_eq!(view.granted(Key::Network), Some(false));
        view.ask_toggle(Key::HostFilesystem);
        assert_eq!(
            view.confirmed_request(),
            Some(Request::Set {
                id: "org.gnome.Builder".to_owned(),
                key: Key::HostFilesystem,
                on: false
            })
        );
        view.confirm = None;
        view.ask_toggle(Key::Network);
        assert_eq!(
            view.confirmed_request(),
            Some(Request::Set {
                id: "org.gnome.Builder".to_owned(),
                key: Key::Network,
                on: true
            })
        );
    }

    #[test]
    fn a_toggle_without_a_read_sandbox_refuses_instead_of_guessing() {
        let mut view = ready_view();
        view.perm_state.clear();
        view.permissions.clear();
        view.ask_toggle(Key::Network);
        assert!(view.confirm.is_none());
        assert!(view.note.contains("has not been read"));
    }

    #[test]
    fn update_all_and_reset_require_a_confirmation() {
        let mut view = ready_view();
        view.ask_update_all();
        assert_eq!(view.confirmed_request(), Some(Request::UpdateAll));
        view.confirm = None;
        view.ask_reset();
        assert_eq!(
            view.confirmed_request(),
            Some(Request::Reset {
                id: "org.gnome.Builder".to_owned()
            })
        );
    }

    #[test]
    fn unavailable_is_a_state_not_an_empty_update_list() {
        let mut view = View {
            updates: vec![Update::default()],
            ..View::default()
        };
        view.apply(
            &Request::Updates,
            Some("sandbox.state=unavailable\nsandbox.updates=0\nsandbox.error=Flatpak%20is%20missing"),
        );
        assert_eq!(view.state, "unavailable");
        assert_eq!(view.error, "Flatpak is missing");
        assert!(view.updates.is_empty());
    }

    #[test]
    fn an_app_list_reply_uses_the_installed_rows_and_selects_the_first() {
        let mut view = View::default();
        view.apply(
            &Request::Apps,
            Some("store.state=ready\nstore.app org.gnome.Builder|Builder|46.2|flathub|1.2 GB|flathub"),
        );
        assert_eq!(view.state, "ready");
        assert_eq!(view.selected_app, Some(0));
        assert_eq!(view.app().map(|app| app.name.as_str()), Some("Builder"));
    }

    #[test]
    fn a_write_reply_clears_the_confirmation_and_reports_a_terminal_sentence() {
        let mut view = ready_view();
        view.ask_update_all();
        view.apply(
            &Request::UpdateAll,
            Some("action=sandbox.update\napplied=true"),
        );
        assert!(view.confirm.is_none());
        assert_eq!(view.note, "Every listed update was applied.");

        view.ask_toggle(Key::Network);
        view.apply(
            &Request::Set {
                id: "org.gnome.Builder".to_owned(),
                key: Key::Network,
                on: true,
            },
            Some("action=sandbox.override\napplied=true"),
        );
        assert!(view.confirm.is_none());
        assert_eq!(view.note, "Allowed network for org.gnome.Builder.");
    }

    #[test]
    fn a_missing_answer_is_an_error_and_never_an_empty_machine() {
        let mut view = ready_view();
        view.apply(&Request::Permissions("org.gnome.Builder".to_owned()), None);
        assert_eq!(view.perm_state, "error");
        assert!(view.note.contains("did not answer"));
    }
}
