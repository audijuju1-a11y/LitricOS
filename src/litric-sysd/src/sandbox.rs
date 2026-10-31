//! Flatpak updates and system sandbox permissions, behind the daemon.
//!
//! The App Sandbox surface is a view of two Flatpak interfaces, not a second
//! policy engine:
//!
//! * `flatpak remote-ls --updates` lists the updates Flatpak itself would
//!   install. Applying one is `flatpak update --noninteractive --assumeyes`,
//!   because a daemon must never sit waiting for a question nobody can see.
//! * `flatpak info --show-permissions` and `flatpak override --system --show`
//!   are Flatpak's own account of an application's context. The effective set
//!   is built from those three answers in the order Flatpak applies them:
//!   the application's own metadata, the global system override, then the
//!   application's system override. A `!entry` removes an inherited entry; it
//!   is reported as an `unshare` fact rather than being silently folded away.
//!
//! Writes are fixed argument vectors with a bounded deadline and always name
//! the system installation and one validated application id, so a write can
//! never touch another app or the host's own configuration. Nothing here opens
//! a D-Bus connection, runs a shell, or starts a long-lived process. A host
//! without `flatpak` on `PATH` is [`StoreReadError::Unavailable`] — a state,
//! not an empty update list and not a malformed request.

use std::collections::BTreeSet;
use std::io;

use crate::action::Runner;
use crate::providers::{Plan, Refusal};
use crate::store::{valid_app_id, StoreReadError, FLATPAK};

/// How long listing updates may take, in seconds.
pub const READ_TIMEOUT_SECS: u32 = 45;

/// How long applying updates may take, in seconds. Flatpak fetches runtimes
/// and app data, so this is the install deadline rather than a local change's.
pub const UPDATE_TIMEOUT_SECS: u32 = 900;

/// How long one override write may take, in seconds.
pub const OVERRIDE_TIMEOUT_SECS: u32 = 120;

/// How many updates one read may return.
pub const MAX_UPDATES: usize = 64;

/// How many permission facts one read may return.
pub const MAX_PERMISSIONS: usize = 128;

/// The columns the update read asks for, in the order they are parsed.
const UPDATE_COLUMNS: &str = "application,name,version,branch,origin";

/// One application update as Flatpak describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppUpdate {
    /// Reverse-DNS application id.
    pub id: String,
    /// Display name, falling back to the id when Flatpak printed none.
    pub name: String,
    /// The version being offered.
    pub version: String,
    /// The branch the update belongs to.
    pub branch: String,
    /// The remote the update comes from.
    pub origin: String,
}

/// Where a permission entry was read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionSource {
    /// The application's own metadata.
    Base,
    /// The global system override.
    Global,
    /// This application's system override.
    App,
    /// The set that results from applying all three in order.
    Effective,
}

impl PermissionSource {
    /// The stable token the protocol and the interface use.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Global => "global",
            Self::App => "app",
            Self::Effective => "effective",
        }
    }
}

/// One discrete permission fact.
///
/// Flatpak's context is semicolon-separated lists inside four keys. A fact is
/// one entry of one of those lists, never the list itself: a panel that showed
/// `filesystems=host;home;` as a blob could not say which entries the
/// application's own metadata granted and which an administrator's override
/// removed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PermissionFact {
    /// Which of the three answers this entry came from, or the merged result.
    pub source: PermissionSource,
    /// `filesystem`, `socket`, `device`, `shared`, or `unshare` for a `!entry`.
    pub kind: String,
    /// The entry itself, without the `!` marker.
    pub value: String,
}

/// A section of Flatpak's `[Context]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Section {
    Filesystem,
    Socket,
    Device,
    Shared,
}

impl Section {
    fn parse(key: &str) -> Option<Self> {
        Some(match key {
            "filesystems" => Self::Filesystem,
            "sockets" => Self::Socket,
            "devices" => Self::Device,
            "shared" => Self::Shared,
            _ => return None,
        })
    }

    const fn token(self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::Socket => "socket",
            Self::Device => "device",
            Self::Shared => "shared",
        }
    }
}

/// One entry inside a `[Context]` key, before it is merged.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    section: Section,
    value: String,
    negated: bool,
}

impl Entry {
    fn fact(&self, source: PermissionSource) -> PermissionFact {
        PermissionFact {
            source,
            kind: if self.negated {
                "unshare".to_owned()
            } else {
                self.section.token().to_owned()
            },
            value: self.value.clone(),
        }
    }
}

/// A key of the overlay (an override file) rather than of the base metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxKey {
    /// Access to the host's filesystem as `host`.
    HostFilesystem,
    /// The network namespace, as `network`.
    Network,
}

impl SandboxKey {
    /// Both keys, in the order the page shows them.
    pub const ALL: [Self; 2] = [Self::HostFilesystem, Self::Network];

    /// The stable token the protocol and the interface use.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::HostFilesystem => "host-filesystem",
            Self::Network => "network",
        }
    }

    /// The word on the page.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::HostFilesystem => "Host filesystem",
            Self::Network => "Network",
        }
    }

    /// Parse the protocol token.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|key| key.token() == raw)
    }

    /// The override argument that grants this access.
    const fn allow_arg(self) -> &'static str {
        match self {
            Self::HostFilesystem => "--filesystem=host",
            Self::Network => "--share=network",
        }
    }

    /// The override argument that removes it for one application.
    ///
    /// `--nofilesystem=` and `--unshare=` write a `!entry` into the
    /// application's own override, which removes the inherited grant rather
    /// than granting anything to the host.
    const fn deny_arg(self) -> &'static str {
        match self {
            Self::HostFilesystem => "--nofilesystem=host",
            Self::Network => "--unshare=network",
        }
    }
}

/// A write the App Sandbox surface can ask the daemon to perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxAction {
    /// Apply every listed update, or one application's update.
    Update {
        /// The application, or `None` for every listed update.
        id: Option<String>,
    },
    /// Set one system override for one application.
    Set {
        /// Validated reverse-DNS application id.
        id: String,
        /// Which access.
        key: SandboxKey,
        /// Whether to grant (`true`) or remove it.
        on: bool,
    },
    /// Remove every system override this daemon wrote for one application.
    Reset {
        /// Validated reverse-DNS application id.
        id: String,
    },
}

impl SandboxAction {
    /// The word the protocol and the reply use.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Update { .. } => "update",
            Self::Set { .. } => "override",
            Self::Reset { .. } => "reset",
        }
    }

    /// The resource this action is scoped to.
    ///
    /// `appupdate://<id>` or `appupdate://all` for an update, and
    /// `sandbox://<id>` for an override. The capability table grants the broad
    /// root; the broker still checks this narrower scope for every request, so
    /// authority over one app is not authority over another.
    #[must_use]
    pub fn scope(&self) -> Option<String> {
        match self {
            Self::Update { id: None } => Some("appupdate://all".to_owned()),
            Self::Update { id: Some(id) } if valid_app_id(id) => Some(format!("appupdate://{id}")),
            Self::Set { id, .. } | Self::Reset { id } if valid_app_id(id) => {
                Some(format!("sandbox://{id}"))
            }
            _ => None,
        }
    }

    /// Turn the action into its one fixed plan.
    ///
    /// # Errors
    /// [`Refusal::illegal`] when an id is not one Flatpak could receive as a
    /// single argument.
    pub fn plan(&self) -> Result<Plan, Refusal> {
        match self {
            Self::Update { id } => {
                if let Some(id) = id {
                    if !valid_app_id(id) {
                        return Err(Refusal::illegal(
                            "application id",
                            "is not a reverse-DNS Flatpak application id",
                        ));
                    }
                }
                let mut args = vec![
                    "update".to_owned(),
                    "--system".to_owned(),
                    "--noninteractive".to_owned(),
                    "--assumeyes".to_owned(),
                    // `--app` restricts the update to applications. With no id
                    // after it Flatpak updates every installed application,
                    // which is exactly the "all listed updates" request.
                    "--app".to_owned(),
                ];
                if let Some(id) = id {
                    args.push(id.clone());
                }
                Ok(Plan {
                    program: FLATPAK,
                    args,
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![
                        format!("id={}", id.as_deref().unwrap_or("all")),
                        "scope=system".to_owned(),
                    ],
                    timeout_secs: UPDATE_TIMEOUT_SECS,
                })
            }
            Self::Set { id, key, on } => {
                if !valid_app_id(id) {
                    return Err(Refusal::illegal(
                        "application id",
                        "is not a reverse-DNS Flatpak application id",
                    ));
                }
                Ok(Plan {
                    program: FLATPAK,
                    args: vec![
                        "override".to_owned(),
                        "--system".to_owned(),
                        if *on { key.allow_arg() } else { key.deny_arg() }.to_owned(),
                        id.clone(),
                    ],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![
                        format!("id={id}"),
                        format!("key={}", key.token()),
                        format!("on={on}"),
                        "scope=system".to_owned(),
                    ],
                    timeout_secs: OVERRIDE_TIMEOUT_SECS,
                })
            }
            Self::Reset { id } => {
                if !valid_app_id(id) {
                    return Err(Refusal::illegal(
                        "application id",
                        "is not a reverse-DNS Flatpak application id",
                    ));
                }
                Ok(Plan {
                    program: FLATPAK,
                    args: vec![
                        "override".to_owned(),
                        "--system".to_owned(),
                        "--reset".to_owned(),
                        id.clone(),
                    ],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![format!("id={id}"), "scope=system".to_owned()],
                    timeout_secs: OVERRIDE_TIMEOUT_SECS,
                })
            }
        }
    }
}

/// Read the system applications that have an update available.
///
/// # Errors
/// See [`StoreReadError`]. An installation with nothing to update is
/// `Ok(vec![])`, which the panel shows as "up to date" rather than as a
/// failure.
pub fn list_updates(runner: &dyn Runner) -> Result<Vec<AppUpdate>, StoreReadError> {
    let columns = format!("--columns={UPDATE_COLUMNS}");
    let args = ["remote-ls", "--updates", "--system", "--app", &columns];
    let printed = capture(runner, &args, READ_TIMEOUT_SECS)?;
    Ok(parse_updates(&printed))
}

/// Read one application's effective system sandbox permissions.
///
/// Three answers, in the order Flatpak applies them: the application's own
/// context, the global system override, and this application's system
/// override.
///
/// # Errors
/// See [`StoreReadError`]. An invalid id is refused before a process starts.
pub fn permissions(runner: &dyn Runner, id: &str) -> Result<Vec<PermissionFact>, StoreReadError> {
    if !valid_app_id(id) {
        return Err(StoreReadError::BadArgument(
            "The application id is not a reverse-DNS Flatpak id.".to_owned(),
        ));
    }
    let base = capture(
        runner,
        &["info", "--system", "--show-permissions", id],
        READ_TIMEOUT_SECS,
    )?;
    let global = capture(
        runner,
        &["override", "--system", "--show"],
        READ_TIMEOUT_SECS,
    )?;
    let app = capture(
        runner,
        &["override", "--system", "--show", id],
        READ_TIMEOUT_SECS,
    )?;
    Ok(effective_permissions(&base, &global, &app))
}

fn capture(
    runner: &dyn Runner,
    args: &[&str],
    timeout_secs: u32,
) -> Result<String, StoreReadError> {
    let (code, printed) =
        runner
            .capture(FLATPAK, args, timeout_secs)
            .map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => StoreReadError::Unavailable(
                    "Flatpak is not installed in this image, so there are no updates or sandbox \
                     permissions to read."
                        .to_owned(),
                ),
                io::ErrorKind::TimedOut => StoreReadError::Failed(format!(
                    "Flatpak did not answer within {timeout_secs} seconds."
                )),
                _ => StoreReadError::Failed(format!("Flatpak could not be started: {error}")),
            })?;
    if code != 0 {
        return Err(StoreReadError::Failed(format!(
            "Flatpak exited {code} while reading the sandbox."
        )));
    }
    Ok(printed)
}

/// Parse `remote-ls --columns=application,name,version,branch,origin`.
///
/// A header row, a partial row and an id this build would not pass to a
/// command are all skipped: a partial row is not an update.
#[must_use]
pub fn parse_updates(printed: &str) -> Vec<AppUpdate> {
    let mut updates = Vec::new();
    for line in printed.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        if fields.len() < 5 {
            continue;
        }
        let id = fields.first().copied().unwrap_or_default();
        if id == "Application" || !valid_app_id(id) {
            continue;
        }
        let name = fields.get(1).copied().unwrap_or_default();
        updates.push(AppUpdate {
            id: id.to_owned(),
            name: if name.is_empty() {
                id.to_owned()
            } else {
                name.to_owned()
            },
            version: fields.get(2).copied().unwrap_or_default().to_owned(),
            branch: fields.get(3).copied().unwrap_or_default().to_owned(),
            origin: fields.get(4).copied().unwrap_or_default().to_owned(),
        });
    }
    updates.sort_by(|a, b| {
        a.id.to_ascii_lowercase()
            .cmp(&b.id.to_ascii_lowercase())
            .then_with(|| a.branch.cmp(&b.branch))
            .then_with(|| a.origin.cmp(&b.origin))
    });
    updates.dedup_by(|a, b| a.id == b.id && a.branch == b.branch && a.origin == b.origin);
    updates.truncate(MAX_UPDATES);
    updates
}

/// Build the effective permission facts from Flatpak's three answers.
///
/// The merge is Flatpak's own: entries accumulate, and a `!entry` in a later
/// overlay removes the matching inherited entry. An effective set is a set:
/// the same entry granted twice appears once. Every source entry is kept in
/// the result, so the panel can say *where* a grant came from and *where* it
/// was removed, not only what survived.
#[must_use]
pub fn effective_permissions(base: &str, global: &str, app: &str) -> Vec<PermissionFact> {
    let base_entries = parse_entries(base);
    let global_entries = parse_entries(global);
    let app_entries = parse_entries(app);

    let mut effective: BTreeSet<(Section, String)> = BTreeSet::new();
    for entry in &base_entries {
        // The base metadata is Flatpak's own description of the application;
        // a `!entry` there is not an overlay, so it is still applied to the
        // set this function starts from.
        apply(&mut effective, entry);
    }
    for entry in global_entries.iter().chain(app_entries.iter()) {
        apply(&mut effective, entry);
    }

    let mut facts = Vec::new();
    for (source, entries) in [
        (PermissionSource::Base, &base_entries),
        (PermissionSource::Global, &global_entries),
        (PermissionSource::App, &app_entries),
    ] {
        let mut rendered: Vec<PermissionFact> =
            entries.iter().map(|entry| entry.fact(source)).collect();
        rendered.sort();
        rendered.dedup();
        facts.extend(rendered);
    }
    let mut merged: Vec<PermissionFact> = effective
        .iter()
        .map(|(section, value)| PermissionFact {
            source: PermissionSource::Effective,
            kind: section.token().to_owned(),
            value: value.clone(),
        })
        .collect();
    merged.sort();
    facts.extend(merged);
    facts.truncate(MAX_PERMISSIONS);
    facts
}

fn apply(effective: &mut BTreeSet<(Section, String)>, entry: &Entry) {
    let key = (entry.section, entry.value.clone());
    if entry.negated {
        effective.remove(&key);
    } else {
        effective.insert(key);
    }
}

/// Parse every `[Context]` entry in one Flatpak permission answer.
///
/// Other sections (`[Application]`, `[Session Bus Policy]`, `[Environment]`)
/// are not sandbox context and are deliberately ignored: showing bus policy as
/// if it were a filesystem grant would be a category error, and inventing a
/// fifth kind is not what this build's page can explain.
fn parse_entries(text: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut in_context = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_context = line.eq_ignore_ascii_case("[Context]");
            continue;
        }
        if !in_context {
            continue;
        }
        let Some((key, raw)) = line.split_once('=') else {
            continue;
        };
        let Some(section) = Section::parse(key.trim()) else {
            continue;
        };
        for value in raw.split(';').map(str::trim) {
            if value.is_empty() {
                continue;
            }
            let (value, negated) = match value.strip_prefix('!') {
                Some(rest) => (rest, true),
                None => (value, false),
            };
            // A bare `!` would otherwise become an empty permission name; it
            // is malformed, and a fact with no value is not a fact.
            if value.is_empty() || value.bytes().any(|b| b.is_ascii_control()) {
                continue;
            }
            entries.push(Entry {
                section,
                value: value.to_owned(),
                negated,
            });
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::testing::RecordingRunner;

    const UPDATES: &str = "Application\tName\tVersion\tBranch\tOrigin\n\
        org.gnome.Builder\tBuilder\t46.2\tstable\tflathub\n\
        org.videolan.VLC\tVLC\t3.0.21\tstable\tflathub\n\
        org.gnome.Builder\tBuilder\t46.2\tstable\tflathub\n";

    const BASE: &str = "[Application]\n\
        name=org.gnome.Builder\n\n\
        [Context]\n\
        shared=network;ipc;\n\
        sockets=x11;pulseaudio;\n\
        devices=dri;\n\
        filesystems=/home/alice/Documents;host;\n";

    const GLOBAL: &str = "[Context]\n\
        filesystems=!host;\n";

    const APP: &str = "[Context]\n\
        shared=!network;\n\
        devices=all;\n";

    #[test]
    fn update_rows_are_parsed_sorted_and_deduplicated() {
        let updates = parse_updates(UPDATES);
        assert_eq!(updates.len(), 2, "the duplicate row was removed");
        let first = updates.first().expect("the first update");
        let second = updates.get(1).expect("the second update");
        assert_eq!(first.id, "org.gnome.Builder");
        assert_eq!(first.name, "Builder");
        assert_eq!(first.branch, "stable");
        assert_eq!(second.origin, "flathub");
    }

    #[test]
    fn a_partial_row_header_or_bad_id_is_not_an_update() {
        let text = "Application\tName\tVersion\tBranch\tOrigin\n\
            broken\trow\n\
            Bad/Id\tName\t1\tstable\tflathub\n\
            org.good.App\tGood\t1\tstable\tflathub\n";
        let updates = parse_updates(text);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates.first().expect("one update").id, "org.good.App");
    }

    #[test]
    fn a_missing_flatpak_is_unavailable_not_an_empty_update_list() {
        let runner = RecordingRunner::failing(io::ErrorKind::NotFound, "no flatpak");
        let error = list_updates(&runner).expect_err("no tool is not up to date");
        assert_eq!(error.token(), "unavailable");
        assert!(error.detail().contains("not installed"));
    }

    #[test]
    fn a_failed_read_is_an_error_not_an_empty_update_list() {
        let runner = RecordingRunner::succeeding(0).answering(7, "");
        let error = list_updates(&runner).expect_err("a failed tool is not up to date");
        assert_eq!(error.token(), "error");
        assert!(error.detail().contains("exited 7"));
    }

    #[test]
    fn update_reads_ask_flatpak_for_the_four_context_columns() {
        let runner = RecordingRunner::succeeding(0).answering(0, UPDATES);
        let updates = list_updates(&runner).expect("an update list");
        assert_eq!(updates.len(), 2);
        assert_eq!(
            runner.calls(),
            vec![(
                FLATPAK.to_owned(),
                vec![
                    "remote-ls".to_owned(),
                    "--updates".to_owned(),
                    "--system".to_owned(),
                    "--app".to_owned(),
                    "--columns=application,name,version,branch,origin".to_owned()
                ]
            )]
        );
        assert_eq!(runner.timeout_secs(), Some(READ_TIMEOUT_SECS));
    }

    #[test]
    fn permission_reads_use_info_then_the_global_and_app_overrides() {
        let runner =
            RecordingRunner::succeeding(0).answering_queue(&[(0, BASE), (0, GLOBAL), (0, APP)]);
        let facts = permissions(&runner, "org.gnome.Builder").expect("permissions");
        let calls = runner.calls();
        assert_eq!(calls.len(), 3, "{calls:?}");
        assert_eq!(
            calls.first().expect("the base read").1,
            vec![
                "info".to_owned(),
                "--system".to_owned(),
                "--show-permissions".to_owned(),
                "org.gnome.Builder".to_owned()
            ]
        );
        assert_eq!(
            calls.get(1).expect("the global override").1,
            vec![
                "override".to_owned(),
                "--system".to_owned(),
                "--show".to_owned()
            ]
        );
        assert_eq!(
            calls.get(2).expect("the app override").1,
            vec![
                "override".to_owned(),
                "--system".to_owned(),
                "--show".to_owned(),
                "org.gnome.Builder".to_owned()
            ]
        );
        assert_eq!(runner.timeout_secs(), Some(READ_TIMEOUT_SECS));
        assert!(facts.iter().any(|fact| {
            fact.source == PermissionSource::Effective
                && fact.kind == "device"
                && fact.value == "all"
        }));
    }

    #[test]
    fn the_effective_set_applies_removals_from_both_overrides() {
        let facts = effective_permissions(BASE, GLOBAL, APP);
        let effective: Vec<&PermissionFact> = facts
            .iter()
            .filter(|fact| fact.source == PermissionSource::Effective)
            .collect();
        let kinds: Vec<(&str, &str)> = effective
            .iter()
            .map(|fact| (fact.kind.as_str(), fact.value.as_str()))
            .collect();
        assert_eq!(
            kinds,
            vec![
                ("device", "all"),
                ("device", "dri"),
                ("filesystem", "/home/alice/Documents"),
                ("shared", "ipc"),
                ("socket", "pulseaudio"),
                ("socket", "x11"),
            ],
            "host and network were removed, not merged back in"
        );
        // The removals are facts in their own right, attributed to the answer
        // that made them rather than disappearing into the merge.
        assert!(facts.iter().any(|fact| {
            fact.source == PermissionSource::Global
                && fact.kind == "unshare"
                && fact.value == "host"
        }));
        assert!(facts.iter().any(|fact| {
            fact.source == PermissionSource::App
                && fact.kind == "unshare"
                && fact.value == "network"
        }));
    }

    #[test]
    fn permission_entries_are_discrete_facts_not_one_blob() {
        let facts = effective_permissions(BASE, GLOBAL, APP);
        let base: Vec<&PermissionFact> = facts
            .iter()
            .filter(|fact| fact.source == PermissionSource::Base)
            .collect();
        assert!(
            base.iter().all(|fact| !fact.value.contains(';')),
            "an entry must not carry the whole list: {base:?}"
        );
        assert!(base
            .iter()
            .any(|fact| fact.kind == "filesystem" && fact.value == "/home/alice/Documents"));
        assert!(base
            .iter()
            .any(|fact| fact.kind == "shared" && fact.value == "network"));
    }

    #[test]
    fn update_and_override_plans_are_fixed_and_noninteractive() {
        let all = SandboxAction::Update { id: None }
            .plan()
            .expect("a valid update");
        assert_eq!(
            all.args,
            vec![
                "update",
                "--system",
                "--noninteractive",
                "--assumeyes",
                "--app"
            ]
        );
        assert_eq!(all.timeout_secs, UPDATE_TIMEOUT_SECS);

        let one = SandboxAction::Update {
            id: Some("org.gnome.Builder".to_owned()),
        }
        .plan()
        .expect("a valid update");
        assert_eq!(
            one.args,
            vec![
                "update",
                "--system",
                "--noninteractive",
                "--assumeyes",
                "--app",
                "org.gnome.Builder"
            ]
        );

        let allow = SandboxAction::Set {
            id: "org.gnome.Builder".to_owned(),
            key: SandboxKey::HostFilesystem,
            on: true,
        }
        .plan()
        .expect("a valid override");
        assert_eq!(
            allow.args,
            vec![
                "override",
                "--system",
                "--filesystem=host",
                "org.gnome.Builder"
            ]
        );
        assert_eq!(allow.timeout_secs, OVERRIDE_TIMEOUT_SECS);

        let deny = SandboxAction::Set {
            id: "org.gnome.Builder".to_owned(),
            key: SandboxKey::Network,
            on: false,
        }
        .plan()
        .expect("a valid override");
        assert_eq!(
            deny.args,
            vec![
                "override",
                "--system",
                "--unshare=network",
                "org.gnome.Builder"
            ]
        );

        let reset = SandboxAction::Reset {
            id: "org.gnome.Builder".to_owned(),
        }
        .plan()
        .expect("a valid reset");
        assert_eq!(
            reset.args,
            vec!["override", "--system", "--reset", "org.gnome.Builder"]
        );
    }

    #[test]
    fn a_bad_application_id_is_refused_before_a_plan_exists() {
        for id in [
            "",
            "Builder",
            "org..Builder",
            "org.gnome/Bad",
            "-org.gnome.Builder",
        ] {
            assert!(
                SandboxAction::Update {
                    id: Some(id.to_owned())
                }
                .plan()
                .is_err(),
                "{id:?}"
            );
            assert!(
                SandboxAction::Set {
                    id: id.to_owned(),
                    key: SandboxKey::Network,
                    on: false
                }
                .plan()
                .is_err(),
                "{id:?}"
            );
            assert!(
                SandboxAction::Reset { id: id.to_owned() }.plan().is_err(),
                "{id:?}"
            );
        }
    }

    #[test]
    fn scopes_name_the_app_or_the_whole_update_set() {
        assert_eq!(
            SandboxAction::Update { id: None }.scope().as_deref(),
            Some("appupdate://all")
        );
        assert_eq!(
            SandboxAction::Update {
                id: Some("org.gnome.Builder".to_owned())
            }
            .scope()
            .as_deref(),
            Some("appupdate://org.gnome.Builder")
        );
        assert_eq!(
            SandboxAction::Set {
                id: "org.gnome.Builder".to_owned(),
                key: SandboxKey::Network,
                on: true
            }
            .scope()
            .as_deref(),
            Some("sandbox://org.gnome.Builder")
        );
        assert_eq!(
            SandboxAction::Reset {
                id: "org.gnome.Builder".to_owned()
            }
            .scope()
            .as_deref(),
            Some("sandbox://org.gnome.Builder")
        );
    }

    #[test]
    fn every_key_has_one_token_and_round_trips() {
        for key in SandboxKey::ALL {
            assert_eq!(SandboxKey::parse(key.token()), Some(key));
        }
        assert_eq!(SandboxKey::parse("host"), None);
    }
}
