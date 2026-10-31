//! Flatpak's app catalog, behind the daemon.
//!
//! The App Store surface is a view of Flatpak, not a second package manager.
//! Reads ask Flatpak's own client for machine columns; writes become a
//! [`Plan`] with a fixed argument vector and a bounded deadline. Nothing here
//! links a D-Bus client, spawns a shell, or opens an interactive prompt: the
//! system-wide installation is named explicitly, `--noninteractive` and
//! `--assumeyes` are always present, and a tool that never answers is killed by
//! the runner rather than holding the daemon.
//!
//! Absence is a state. A host without `/usr/bin/flatpak` (or with no executable
//! on `PATH`) gets [`StoreReadError::Unavailable`], which the protocol reports
//! as `store.state=unavailable`, not as a malformed request or an empty catalog.

use std::io;

use crate::action::Runner;
use crate::providers::{Plan, Refusal};

/// Flatpak's executable, resolved through the image's `PATH`.
///
/// The service facts use the absolute image path for installation checks. The
/// store deliberately does not: `PATH` is how the image's package manager
/// exposes the client, including on a development host, and the daemon has no
/// shell to interpret anything else.
pub const FLATPAK: &str = "flatpak";

/// The columns both reads ask for, in the order they are parsed.
const SEARCH_COLUMNS: &str = "application,name,version,remotes";
/// Installed rows add size and the origin remote.
const LIST_COLUMNS: &str = "application,name,version,origin,size";

/// How many results one read may return.
///
/// A cap, not a page: the panel draws a bounded list and a remote can contain
/// thousands of matches. The sort below is applied before the cap, so the same
/// catalog always produces the same first page.
pub const MAX_RESULTS: usize = 24;

/// How long a catalog read may take, in seconds. Both may fetch a summary.
pub const READ_TIMEOUT_SECS: u32 = 45;

/// How long installing from a remote may take, in seconds. Flatpak fetches
/// runtimes and app data, so this is much longer than a local change; it is
/// still bounded.
pub const INSTALL_TIMEOUT_SECS: u32 = 900;

/// How long removing an installed app may take, in seconds.
pub const UNINSTALL_TIMEOUT_SECS: u32 = 120;

/// What a search query may contain.
pub const MAX_QUERY: usize = 100;

/// One application as Flatpak describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreApp {
    /// Reverse-DNS application id.
    pub id: String,
    /// Display name, when the remote supplied one.
    pub name: String,
    /// Version, when the remote or installation supplied one.
    pub version: String,
    /// Origin remote for an installed app, or the remotes that offer a search
    /// result.
    pub origin: String,
    /// Installed size as Flatpak formatted it, or the remote's absence marker.
    pub size: String,
    /// The single remote an install may name. Empty for a row that has none.
    pub remote: String,
}

impl StoreApp {
    /// The safest remote to name when installing this result.
    ///
    /// Flatpak can list several remotes for one app id. The first one Flatpak
    /// names is the one its own search would install from, so it is kept rather
    /// than sorted: sorting would silently prefer a different remote from the
    /// one the catalog presented.
    #[must_use]
    pub fn first_remote(remotes: &str) -> String {
        remotes
            .split([',', ' ', '\t'])
            .map(str::trim)
            .find(|part| !part.is_empty())
            .unwrap_or_default()
            .to_owned()
    }
}

/// Why a catalog read produced no rows.
///
/// This is not [`crate::providers::Refusal`]: a read changes nothing and has no
/// argument to refuse. The three cases need three different sentences in the
/// interface, so they stay separate here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreReadError {
    /// The image has no Flatpak client. This is a state, not a failure.
    Unavailable(String),
    /// The client exists but could not answer.
    Failed(String),
    /// The caller's query is not one this daemon will pass to Flatpak.
    BadArgument(String),
}

impl StoreReadError {
    /// The stable token a protocol client can branch on.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Unavailable(_) => "unavailable",
            Self::Failed(_) => "error",
            Self::BadArgument(_) => "bad-argument",
        }
    }

    /// The sentence shown to a person.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::Unavailable(detail) | Self::Failed(detail) | Self::BadArgument(detail) => detail,
        }
    }
}

/// A write the App Store can ask the daemon to perform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreAction {
    /// Install one application from one named remote.
    Install {
        /// Validated reverse-DNS application id.
        id: String,
        /// Validated remote name.
        remote: String,
    },
    /// Remove one installed application.
    Uninstall {
        /// Validated reverse-DNS application id.
        id: String,
    },
}

impl StoreAction {
    /// The word the protocol and reply use.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Install { .. } => "install",
            Self::Uninstall { .. } => "uninstall",
        }
    }

    /// The resource this action is scoped to.
    ///
    /// `appmanage://<id>/<remote>` for an install and `appmanage://<id>` for a
    /// removal. The remote is part of the install scope on purpose: authority
    /// to install an app from one remote is not authority to fetch it from
    /// another.
    #[must_use]
    pub fn scope(&self) -> Option<String> {
        match self {
            Self::Install { id, remote } if valid_app_id(id) && valid_remote(remote) => {
                Some(format!("appmanage://{id}/{remote}"))
            }
            Self::Uninstall { id } if valid_app_id(id) => Some(format!("appmanage://{id}")),
            _ => None,
        }
    }

    /// Turn the action into its one fixed plan.
    ///
    /// # Errors
    /// [`Refusal::illegal`] when the id or remote is not one Flatpak could
    /// receive as a single argument.
    pub fn plan(&self) -> Result<Plan, Refusal> {
        match self {
            Self::Install { id, remote } => {
                if !valid_app_id(id) {
                    return Err(Refusal::illegal(
                        "application id",
                        "is not a reverse-DNS Flatpak application id",
                    ));
                }
                if !valid_remote(remote) {
                    return Err(Refusal::illegal("remote", "is not a Flatpak remote name"));
                }
                Ok(Plan {
                    program: FLATPAK,
                    args: vec![
                        "install".to_owned(),
                        "--system".to_owned(),
                        "--noninteractive".to_owned(),
                        "--assumeyes".to_owned(),
                        remote.clone(),
                        id.clone(),
                    ],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![
                        format!("id={id}"),
                        format!("remote={remote}"),
                        "scope=system".to_owned(),
                    ],
                    timeout_secs: INSTALL_TIMEOUT_SECS,
                })
            }
            Self::Uninstall { id } => {
                if !valid_app_id(id) {
                    return Err(Refusal::illegal(
                        "application id",
                        "is not a reverse-DNS Flatpak application id",
                    ));
                }
                Ok(Plan {
                    program: FLATPAK,
                    args: vec![
                        "uninstall".to_owned(),
                        "--system".to_owned(),
                        "--noninteractive".to_owned(),
                        "--assumeyes".to_owned(),
                        id.clone(),
                    ],
                    stdin: None,
                    files: Vec::new(),
                    notes: vec![format!("id={id}"), "scope=system".to_owned()],
                    timeout_secs: UNINSTALL_TIMEOUT_SECS,
                })
            }
        }
    }
}

/// Whether `id` is a reverse-DNS Flatpak application id.
///
/// Flatpak accepts ids this daemon does not: uppercase and the characters it
/// allows in desktop ids are deliberately refused here because the id becomes
/// one argv element and one protocol field. A valid id has at least two
/// non-empty dot-separated segments and no whitespace, control character,
/// slash, leading dash, or `..` segment.
#[must_use]
pub fn valid_app_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 255 || !id.contains('.') {
        return false;
    }
    if id.starts_with('-') || id.starts_with('.') || id.ends_with('.') {
        return false;
    }
    let mut segments = 0usize;
    for segment in id.split('.') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return false;
        }
        if !segment
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return false;
        }
        segments = segments.saturating_add(1);
    }
    segments >= 2
}

/// Whether `remote` is a name Flatpak could receive without option injection.
#[must_use]
pub fn valid_remote(remote: &str) -> bool {
    !remote.is_empty()
        && remote.len() <= 64
        && !remote.starts_with('-')
        && remote
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Whether `query` is safe to pass to `flatpak search`.
#[must_use]
pub fn valid_query(query: &str) -> bool {
    let query = query.trim();
    !query.is_empty()
        && query.len() <= MAX_QUERY
        && !query.starts_with('-')
        && !query.bytes().any(|b| b.is_ascii_control())
}

/// Read the installed system applications.
///
/// # Errors
/// See [`StoreReadError`]. An empty installation is `Ok(vec![])`, not an error.
pub fn list(runner: &dyn Runner) -> Result<Vec<StoreApp>, StoreReadError> {
    let columns = format!("--columns={LIST_COLUMNS}");
    let args = ["list", "--system", "--app", &columns];
    let printed = capture(runner, &args, READ_TIMEOUT_SECS)?;
    Ok(parse_apps(&printed, false))
}

/// Search the configured system remotes.
///
/// # Errors
/// See [`StoreReadError`]. A syntactically bad query is refused before a
/// process is spawned.
pub fn search(runner: &dyn Runner, query: &str) -> Result<Vec<StoreApp>, StoreReadError> {
    if !valid_query(query) {
        return Err(StoreReadError::BadArgument(
            "Enter a search of 100 characters or fewer, without a leading dash.".to_owned(),
        ));
    }
    let columns = format!("--columns={SEARCH_COLUMNS}");
    let args = ["search", "--system", &columns, query.trim()];
    let printed = capture(runner, &args, READ_TIMEOUT_SECS)?;
    Ok(parse_apps(&printed, true))
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
                    "Flatpak is not installed in this image, so there is no app catalog."
                        .to_owned(),
                ),
                io::ErrorKind::TimedOut => StoreReadError::Failed(format!(
                    "Flatpak did not answer within {timeout_secs} seconds."
                )),
                _ => StoreReadError::Failed(format!("Flatpak could not be started: {error}")),
            })?;
    if code != 0 {
        return Err(StoreReadError::Failed(format!(
            "Flatpak exited {code} while reading the catalog."
        )));
    }
    Ok(printed)
}

/// Parse Flatpak's machine columns into rows.
///
/// Flatpak's `--columns` output is tab-separated. A header, if one is printed,
/// starts with `Application`; it is dropped rather than shown as an app named
/// "Name". Rows without the expected number of fields are skipped: a partial
/// row is not an application, and inventing empty fields would put a blank tile
/// in the store.
#[must_use]
pub fn parse_apps(printed: &str, search: bool) -> Vec<StoreApp> {
    let expected = if search { 4 } else { 5 };
    let mut apps = Vec::new();
    for line in printed.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        if fields.len() < expected {
            continue;
        }
        let id = fields.first().copied().unwrap_or_default();
        if id == "Application" || !valid_app_id(id) {
            continue;
        }
        let name = fields.get(1).copied().unwrap_or_default();
        let version = fields.get(2).copied().unwrap_or_default();
        let origin = fields.get(3).copied().unwrap_or_default();
        let size = if search {
            String::new()
        } else {
            fields.get(4).copied().unwrap_or_default().to_owned()
        };
        let remote = if search {
            StoreApp::first_remote(origin)
        } else {
            origin.to_owned()
        };
        apps.push(StoreApp {
            id: id.to_owned(),
            name: if name.is_empty() {
                id.to_owned()
            } else {
                name.to_owned()
            },
            version: version.to_owned(),
            origin: origin.to_owned(),
            size,
            remote,
        });
    }
    apps.sort_by(|a, b| {
        a.id.to_ascii_lowercase()
            .cmp(&b.id.to_ascii_lowercase())
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.origin.cmp(&b.origin))
    });
    apps.dedup_by(|a, b| a.id == b.id && a.origin == b.origin);
    apps.truncate(MAX_RESULTS);
    apps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::testing::RecordingRunner;

    const SEARCH: &str = "Application\tName\tVersion\tRemotes\n\
        org.gnome.Builder\tBuilder\t46.1\tflathub\n\
        org.mozilla.firefox\tFirefox\t128.0\tflathub\n\
        org.gnome.Builder\tBuilder\t46.1\tflathub\n";

    const LIST: &str = "Application\tName\tVersion\tOrigin\tSize\n\
        org.gnome.Builder\tBuilder\t46.1\tflathub\t1.2 GB\n\
        org.videolan.VLC\tVLC\t3.0.20\tflathub\t120.4 MB\n";

    #[test]
    fn search_rows_are_parsed_sorted_and_capped() {
        let apps = parse_apps(SEARCH, true);
        assert_eq!(apps.len(), 2, "the duplicate row was removed");
        let first = apps.first().expect("the first result");
        let second = apps.get(1).expect("the second result");
        assert_eq!(first.id, "org.gnome.Builder");
        assert_eq!(first.remote, "flathub");
        assert_eq!(second.id, "org.mozilla.firefox");
        assert!(first.size.is_empty());
    }

    #[test]
    fn installed_rows_keep_origin_size_and_versions() {
        let apps = parse_apps(LIST, false);
        assert_eq!(apps.len(), 2);
        let first = apps.first().expect("the first installed app");
        let second = apps.get(1).expect("the second installed app");
        assert_eq!(first.size, "1.2 GB");
        assert_eq!(second.origin, "flathub");
        assert_eq!(second.version, "3.0.20");
    }

    #[test]
    fn a_header_partial_row_or_bad_id_is_not_an_application() {
        let text = "Application\tName\tVersion\tRemotes\n\
            broken\trow\n\
            Bad/Id\tName\t1\tflathub\n\
            org.good.App\tGood\t1\tflathub\n";
        let apps = parse_apps(text, true);
        assert_eq!(apps.len(), 1);
        let app = apps.first().expect("one valid app");
        assert_eq!(app.id, "org.good.App");
    }

    #[test]
    fn the_first_remote_is_the_install_target() {
        assert_eq!(StoreApp::first_remote("flathub,beta"), "flathub");
        assert_eq!(StoreApp::first_remote(" beta flathub "), "beta");
        assert_eq!(StoreApp::first_remote(""), "");
    }

    #[test]
    fn reads_ask_flatpak_for_machine_columns_with_bounded_timeouts() {
        let runner = RecordingRunner::succeeding(0).answering(0, SEARCH);
        let apps = search(&runner, "builder").expect("a catalog");
        assert_eq!(apps.len(), 2);
        assert_eq!(
            runner.calls(),
            vec![(
                FLATPAK.to_owned(),
                vec![
                    "search".to_owned(),
                    "--system".to_owned(),
                    "--columns=application,name,version,remotes".to_owned(),
                    "builder".to_owned()
                ]
            )]
        );
        assert_eq!(runner.timeout_secs(), Some(READ_TIMEOUT_SECS));

        let runner = RecordingRunner::succeeding(0).answering(0, LIST);
        let apps = list(&runner).expect("an installation");
        assert_eq!(apps.len(), 2);
        let (_, args) = runner.calls().pop().expect("one call");
        assert_eq!(args.first().map(String::as_str), Some("list"));
        assert_eq!(args.get(1).map(String::as_str), Some("--system"));
        assert!(args.iter().any(|a| a == "--app"));
        assert!(args
            .iter()
            .any(|a| a == "--columns=application,name,version,origin,size"));
    }

    #[test]
    fn a_missing_flatpak_is_unavailable_not_an_empty_catalog() {
        let runner = RecordingRunner::failing(io::ErrorKind::NotFound, "no flatpak");
        assert_eq!(
            list(&runner).expect_err("no tool is not an empty installation"),
            StoreReadError::Unavailable(
                "Flatpak is not installed in this image, so there is no app catalog.".to_owned()
            )
        );
    }

    #[test]
    fn a_failed_read_is_an_error_not_an_empty_catalog() {
        let runner = RecordingRunner::succeeding(0).answering(7, "");
        let error = list(&runner).expect_err("a failed tool is not an empty installation");
        assert_eq!(error.token(), "error");
        assert!(error.detail().contains("exited 7"));
    }

    #[test]
    fn install_and_uninstall_are_fixed_noninteractive_plans() {
        let install = StoreAction::Install {
            id: "org.gnome.Builder".to_owned(),
            remote: "flathub".to_owned(),
        }
        .plan()
        .expect("a valid install");
        assert_eq!(
            install.args,
            vec![
                "install",
                "--system",
                "--noninteractive",
                "--assumeyes",
                "flathub",
                "org.gnome.Builder"
            ]
        );
        assert_eq!(install.timeout_secs, INSTALL_TIMEOUT_SECS);

        let uninstall = StoreAction::Uninstall {
            id: "org.gnome.Builder".to_owned(),
        }
        .plan()
        .expect("a valid removal");
        assert_eq!(
            uninstall.args,
            vec![
                "uninstall",
                "--system",
                "--noninteractive",
                "--assumeyes",
                "org.gnome.Builder"
            ]
        );
        assert_eq!(uninstall.timeout_secs, UNINSTALL_TIMEOUT_SECS);
    }

    #[test]
    fn option_injection_and_bad_identifiers_are_refused() {
        for id in [
            "",
            "Builder",
            "org..Builder",
            "org..Builder",
            "org.gnome/Bad",
            "-org.gnome.Builder",
            "org.gnome.Builder x",
        ] {
            assert!(!valid_app_id(id), "{id:?}");
        }
        assert!(valid_app_id("org.gnome.Builder"));
        assert!(valid_app_id("io.github.some_app.Test-1"));

        for remote in ["", "-flathub", "flat hub", "flathub/beta", "../flatpak"] {
            assert!(!valid_remote(remote), "{remote:?}");
        }
        assert!(valid_remote("flathub"));
        assert!(valid_remote("flathub-beta"));

        for query in ["", "   ", "-v", "line\nbreak"] {
            assert!(!valid_query(query), "{query:?}");
        }
        assert!(valid_query("photo editor"));
    }

    #[test]
    fn scopes_name_the_app_and_the_remote() {
        assert_eq!(
            StoreAction::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "flathub".to_owned()
            }
            .scope()
            .as_deref(),
            Some("appmanage://org.gnome.Builder/flathub")
        );
        assert_eq!(
            StoreAction::Uninstall {
                id: "org.gnome.Builder".to_owned()
            }
            .scope()
            .as_deref(),
            Some("appmanage://org.gnome.Builder")
        );
        assert_eq!(
            StoreAction::Install {
                id: "org.gnome.Builder".to_owned(),
                remote: "bad remote".to_owned()
            }
            .scope(),
            None
        );
    }
}
