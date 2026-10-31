//! System-service facts and the small administrative actions that reload them.
//!
//! LitricOS is built on the distribution's own subsystems rather than a second
//! implementation of each one. This module is the read half for the services
//! that were already present in the image or are installed as optional
//! platforms:
//!
//! | Service | What is read | Why this surface |
//! |---|---|---|
//! | AppArmor | kernel enable flag, loaded profile modes | `/sys/module/apparmor` and securityfs are the kernel's own report |
//! | Polkit | daemon, `pkcheck`, installed action files | the daemon and policy tree are what makes an action available |
//! | firewalld | tool, zones, default zone, runtime directory | the config tree and runtime directory say what is installed and active |
//! | Flatpak | tool, sandbox helper, system app/remote counts | Flatpak's app and repo directories are the installed state |
//! | NetworkManager | tool, connection profiles, runtime directory | the configuration and runtime tree are the manager's own state |
//!
//! Nothing here opens a D-Bus socket or links a client library. The action half
//! uses each service's own command-line client through [`crate::action::Runner`],
//! with a fixed argument vector and a bounded deadline. That is the same
//! decision as the audio, network and Bluetooth providers (ADR-0023): the
//! provider owns the interface, and the daemon does not grow a second protocol
//! implementation.
//!
//! Absence is a state, not zero. A missing tool is reported as `absent` and a
//! missing directory is reported as `unknown` where the distinction matters.
//! The Settings page must be able to say "not installed in this image" rather
//! than draw a control that cannot work.

use std::path::Path;

use crate::action::SYSTEMCTL;
use crate::platform::{read_attr, read_dir, read_file};
use crate::providers::{Plan, Refusal};

/// AppArmor's profile parser, installed by the `apparmor` package.
pub const APPARMOR_PARSER: &str = "/usr/sbin/apparmor_parser";

/// AppArmor's own status client.
pub const AA_STATUS: &str = "/usr/sbin/aa-status";

/// firewalld's own client.
pub const FIREWALL_CMD: &str = "/usr/bin/firewall-cmd";

/// NetworkManager's own client.
pub const NMCLI: &str = "/usr/bin/nmcli";

/// Flatpak's own client.
pub const FLATPAK: &str = "/usr/bin/flatpak";

/// Flatpak's unprivileged sandbox helper, shipped beside the client on Arch.
pub const BWRAP: &str = "/usr/bin/bwrap";

/// The alternate path used by some Flatpak builds.
pub const FLATPAK_BWRAP: &str = "/usr/libexec/flatpak-bwrap";

/// The time a reload may take before it is killed.
pub const RELOAD_TIMEOUT_SECS: u32 = 30;

/// The time a catalog refresh may take. It may fetch an index over the network.
pub const CATALOG_TIMEOUT_SECS: u32 = 60;

/// AppArmor's kernel and userspace state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppArmorFacts {
    /// `Some(true)` when the kernel parameter says AppArmor is enabled.
    pub enabled: Option<bool>,
    /// Whether the loaded-profile list could be read.
    pub profiles_known: bool,
    /// Loaded profiles, total.
    pub profiles: usize,
    /// Loaded profiles in enforcing mode.
    pub enforce: usize,
    /// Loaded profiles in complain mode.
    pub complain: usize,
    /// Whether `apparmor_parser` is installed.
    pub parser: bool,
    /// Whether `aa-status` is installed.
    pub status_tool: bool,
}

impl AppArmorFacts {
    /// The one-word verdict the Settings page uses.
    #[must_use]
    pub fn state(&self) -> &'static str {
        match self.enabled {
            Some(false) => "disabled",
            None => "unknown",
            Some(true) if self.profiles_known && self.profiles == 0 => "enabled-no-profiles",
            Some(true) if self.complain == 0 && self.enforce > 0 => "enforce",
            Some(true) if self.enforce == 0 && self.complain > 0 => "complain",
            Some(true) => "mixed",
        }
    }

    /// Whether it is meaningful to offer a profile reload.
    #[must_use]
    pub fn reloadable(&self) -> bool {
        self.enabled == Some(true) && self.parser && self.status_tool
    }

    fn to_lines(&self) -> Vec<String> {
        vec![format!(
            "svc.apparmor state={} enabled={} profiles={} enforce={} complain={} known={} parser={} status={}",
            self.state(),
            self.enabled
                .map_or_else(|| "-".to_owned(), |v| u8::from(v).to_string()),
            self.profiles,
            self.enforce,
            self.complain,
            u8::from(self.profiles_known),
            u8::from(self.parser),
            u8::from(self.status_tool)
        )]
    }
}

/// Polkit's daemon and policy tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolkitFacts {
    /// Whether a PolicyKit authority daemon is installed.
    pub daemon: bool,
    /// Whether `pkcheck` is installed.
    pub pkcheck: bool,
    /// Installed `.policy` action files.
    pub actions: usize,
}

impl PolkitFacts {
    /// The one-word verdict.
    #[must_use]
    pub fn state(&self) -> &'static str {
        match (self.daemon, self.pkcheck) {
            (true, true) => "ready",
            (true, false) | (false, true) => "partial",
            (false, false) => "absent",
        }
    }

    fn to_lines(&self) -> Vec<String> {
        vec![format!(
            "svc.polkit state={} daemon={} pkcheck={} actions={}",
            self.state(),
            u8::from(self.daemon),
            u8::from(self.pkcheck),
            self.actions
        )]
    }
}

/// firewalld's installed and runtime state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FirewallFacts {
    /// Whether `firewall-cmd` is installed.
    pub tool: bool,
    /// Whether the configuration tree exists.
    pub configured: bool,
    /// Whether the runtime directory says the daemon is running.
    pub running: bool,
    /// The default zone, when the configuration names one.
    pub default_zone: Option<String>,
    /// Number of zone XML files.
    pub zones: usize,
}

impl FirewallFacts {
    /// The one-word verdict.
    #[must_use]
    pub fn state(&self) -> &'static str {
        if self.running {
            "running"
        } else if self.tool && self.configured {
            "configured"
        } else if self.tool {
            "installed"
        } else if self.configured {
            "config-only"
        } else {
            "absent"
        }
    }

    /// Whether a reload can be attempted.
    #[must_use]
    pub fn reloadable(&self) -> bool {
        self.tool && self.running
    }

    fn to_lines(&self) -> Vec<String> {
        vec![format!(
            "svc.firewall state={} tool={} configured={} running={} zones={} default_zone={}",
            self.state(),
            u8::from(self.tool),
            u8::from(self.configured),
            u8::from(self.running),
            self.zones,
            self.default_zone.as_deref().unwrap_or("-")
        )]
    }
}

/// Flatpak's installed system applications and sandbox support.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlatpakFacts {
    /// Whether the Flatpak client is installed.
    pub tool: bool,
    /// Whether a bubblewrap helper is installed.
    pub sandbox: bool,
    /// System-wide applications visible under `/var/lib/flatpak/app`.
    pub apps: usize,
    /// Configured remotes in the system repository.
    pub remotes: usize,
}

impl FlatpakFacts {
    /// The one-word verdict.
    #[must_use]
    pub fn state(&self) -> &'static str {
        match (self.tool, self.sandbox) {
            (true, true) => "ready",
            (true, false) => "no-sandbox",
            (false, true) => "client-missing",
            (false, false) => "absent",
        }
    }

    /// Whether refreshing the app catalog can be attempted.
    #[must_use]
    pub fn catalog_reloadable(&self) -> bool {
        self.tool
    }

    fn to_lines(&self) -> Vec<String> {
        vec![format!(
            "svc.flatpak state={} tool={} sandbox={} apps={} remotes={}",
            self.state(),
            u8::from(self.tool),
            u8::from(self.sandbox),
            self.apps,
            self.remotes
        )]
    }
}

/// NetworkManager's optional compatibility backend.
///
/// LitricOS uses `systemd-networkd` and `iwd` as its native network stack. A
/// machine may still have NetworkManager installed for a compatibility
/// environment or a distribution image, and Settings must report it without
/// pretending it is the manager LitricOS is using.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkManagerFacts {
    /// Whether `nmcli` is installed.
    pub tool: bool,
    /// Whether the configuration tree exists.
    pub configured: bool,
    /// Whether the runtime directory says the daemon is running.
    pub running: bool,
    /// Saved connection profiles.
    pub connections: usize,
}

impl NetworkManagerFacts {
    /// The one-word verdict.
    #[must_use]
    pub fn state(&self) -> &'static str {
        if self.running {
            "running"
        } else if self.tool && self.configured {
            "configured"
        } else if self.tool {
            "installed"
        } else {
            "absent"
        }
    }

    /// Whether a configuration reload can be attempted.
    #[must_use]
    pub fn reloadable(&self) -> bool {
        self.tool && self.running
    }

    fn to_lines(&self) -> Vec<String> {
        vec![format!(
            "svc.networkmanager state={} tool={} configured={} running={} connections={}",
            self.state(),
            u8::from(self.tool),
            u8::from(self.configured),
            u8::from(self.running),
            self.connections
        )]
    }
}

/// Everything this module reports about the service surfaces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceFacts {
    /// AppArmor.
    pub apparmor: AppArmorFacts,
    /// Polkit.
    pub polkit: PolkitFacts,
    /// firewalld.
    pub firewall: FirewallFacts,
    /// Flatpak and its sandbox helper.
    pub flatpak: FlatpakFacts,
    /// NetworkManager, when the image has it.
    pub networkmanager: NetworkManagerFacts,
}

impl ServiceFacts {
    /// The facts as stable `key=value` lines.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.extend(self.apparmor.to_lines());
        out.extend(self.polkit.to_lines());
        out.extend(self.firewall.to_lines());
        out.extend(self.flatpak.to_lines());
        out.extend(self.networkmanager.to_lines());
        out
    }
}

/// Read all service facts from a filesystem root.
///
/// The root is the daemon's configured root, never a request field. Tests point
/// it at a fixture tree; production points it at `/`.
#[must_use]
pub fn read(root: &Path) -> ServiceFacts {
    ServiceFacts {
        apparmor: read_apparmor(root),
        polkit: read_polkit(root),
        firewall: read_firewall(root),
        flatpak: read_flatpak(root),
        networkmanager: read_networkmanager(root),
    }
}

fn read_apparmor(root: &Path) -> AppArmorFacts {
    let enabled = read_attr(root, "sys/module/apparmor/parameters/enabled")
        .as_deref()
        .and_then(parse_enabled);
    let profiles_text = read_file(root, "sys/kernel/security/apparmor/profiles");
    let profiles_known = profiles_text.is_some();
    let mut profiles = 0usize;
    let mut enforce = 0usize;
    let mut complain = 0usize;
    for line in profiles_text.unwrap_or_default().lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        profiles = profiles.saturating_add(1);
        if line.contains("(enforce)") {
            enforce = enforce.saturating_add(1);
        } else if line.contains("(complain)") {
            complain = complain.saturating_add(1);
        }
    }
    AppArmorFacts {
        enabled,
        profiles_known,
        profiles,
        enforce,
        complain,
        parser: installed(root, APPARMOR_PARSER),
        status_tool: installed(root, AA_STATUS),
    }
}

fn parse_enabled(raw: &str) -> Option<bool> {
    match raw.trim() {
        "Y" | "y" | "1" | "yes" | "true" => Some(true),
        "N" | "n" | "0" | "no" | "false" => Some(false),
        _ => None,
    }
}

fn read_polkit(root: &Path) -> PolkitFacts {
    let daemon = installed(root, "/usr/lib/polkit-1/polkitd")
        || installed(root, "/usr/libexec/polkitd")
        || installed(root, "/usr/bin/polkitd");
    PolkitFacts {
        daemon,
        pkcheck: installed(root, "/usr/bin/pkcheck"),
        actions: read_dir(root, "usr/share/polkit-1/actions")
            .iter()
            .filter(|name| name.ends_with(".policy"))
            .count(),
    }
}

fn read_firewall(root: &Path) -> FirewallFacts {
    let default_zone = read_file(root, "etc/firewalld/firewalld.conf").and_then(|text| {
        text.lines().find_map(|line| {
            line.strip_prefix("DefaultZone=")
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty() && !value.contains(char::is_whitespace))
        })
    });
    FirewallFacts {
        tool: installed(root, FIREWALL_CMD),
        configured: root.join("etc/firewalld").is_dir(),
        running: root.join("run/firewalld").is_dir(),
        default_zone,
        zones: read_dir(root, "etc/firewalld/zones")
            .iter()
            .filter(|name| name.ends_with(".xml"))
            .count(),
    }
}

fn read_flatpak(root: &Path) -> FlatpakFacts {
    let remotes = read_file(root, "var/lib/flatpak/repo/config")
        .map(|text| {
            text.lines()
                .filter(|line| line.trim_start().starts_with("[remote \""))
                .count()
        })
        .unwrap_or(0);
    FlatpakFacts {
        tool: installed(root, FLATPAK),
        sandbox: installed(root, BWRAP) || installed(root, FLATPAK_BWRAP),
        apps: read_dir(root, "var/lib/flatpak/app")
            .iter()
            .filter(|name| name.contains('.') && !name.starts_with('.'))
            .count(),
        remotes,
    }
}

fn read_networkmanager(root: &Path) -> NetworkManagerFacts {
    NetworkManagerFacts {
        tool: installed(root, NMCLI),
        configured: root.join("etc/NetworkManager").is_dir(),
        running: root.join("run/NetworkManager").is_dir(),
        connections: read_dir(root, "etc/NetworkManager/system-connections")
            .iter()
            .filter(|name| name.ends_with(".nmconnection"))
            .count(),
    }
}

fn installed(root: &Path, absolute: &str) -> bool {
    root.join(absolute.trim_start_matches('/')).is_file()
}

/// One bounded administrative action for a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    /// Ask the distribution's AppArmor unit to reload the loaded profiles.
    AppArmorReload,
    /// Ask firewalld to reload its configuration.
    FirewallReload,
    /// Refresh Flatpak's application metadata for the App Store surface.
    StoreRefresh,
    /// Ask NetworkManager to reload its configuration.
    NetworkManagerReload,
}

impl ServiceAction {
    /// The stable family token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::AppArmorReload => "apparmor.reload",
            Self::FirewallReload => "firewall.reload",
            Self::StoreRefresh => "store.refresh",
            Self::NetworkManagerReload => "networkmanager.reload",
        }
    }

    /// The scope that authorises exactly this reload.
    #[must_use]
    pub const fn scope_token(self) -> &'static str {
        match self {
            Self::AppArmorReload => "service://apparmor",
            Self::FirewallReload => "service://firewall",
            Self::StoreRefresh => "service://store",
            Self::NetworkManagerReload => "service://networkmanager",
        }
    }

    /// Every action, in the order the Settings page draws its controls.
    pub const ALL: [Self; 4] = [
        Self::AppArmorReload,
        Self::FirewallReload,
        Self::StoreRefresh,
        Self::NetworkManagerReload,
    ];

    /// The command and bounded deadline, with no request-supplied arguments.
    ///
    /// # Errors
    /// No refusal is possible for these closed constants; the `Result` keeps the
    /// shape identical to the other provider plans and leaves room for a future
    /// action that validates a value.
    pub fn plan(self) -> Result<Plan, Refusal> {
        let (program, args, timeout_secs) = match self {
            Self::AppArmorReload => (
                SYSTEMCTL,
                vec!["try-reload-or-restart".to_owned(), "apparmor".to_owned()],
                RELOAD_TIMEOUT_SECS,
            ),
            Self::FirewallReload => (
                FIREWALL_CMD,
                vec!["--reload".to_owned()],
                RELOAD_TIMEOUT_SECS,
            ),
            Self::StoreRefresh => (
                FLATPAK,
                vec![
                    "update".to_owned(),
                    "--appstream".to_owned(),
                    "-y".to_owned(),
                ],
                CATALOG_TIMEOUT_SECS,
            ),
            Self::NetworkManagerReload => (
                NMCLI,
                vec!["general".to_owned(), "reload".to_owned()],
                RELOAD_TIMEOUT_SECS,
            ),
        };
        Ok(Plan {
            program,
            args,
            stdin: None,
            files: Vec::new(),
            notes: vec![format!("service={}", self.token())],
            timeout_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("litric-services-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("fixture root");
        dir
    }

    fn write(path: &Path, text: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        fs::write(path, text).expect("fixture file");
    }

    fn tool(root: &Path, path: &str) {
        write(&root.join(path.trim_start_matches('/')), "#!/bin/sh\n");
    }

    #[test]
    fn an_empty_fixture_reports_every_service_as_absent_without_inventing_counts() {
        let root = scratch("empty");
        let facts = read(&root);
        assert_eq!(facts.apparmor.state(), "unknown");
        assert_eq!(facts.apparmor.profiles, 0);
        assert!(!facts.apparmor.profiles_known);
        assert_eq!(facts.polkit.state(), "absent");
        assert_eq!(facts.firewall.state(), "absent");
        assert_eq!(facts.flatpak.state(), "absent");
        assert_eq!(facts.networkmanager.state(), "absent");
        assert!(facts
            .to_lines()
            .iter()
            .any(|l| l.starts_with("svc.apparmor state=unknown")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_configured_machine_reports_the_real_service_counts_and_modes() {
        let root = scratch("configured");
        write(&root.join("sys/module/apparmor/parameters/enabled"), "Y\n");
        write(
            &root.join("sys/kernel/security/apparmor/profiles"),
            "usr.bin.man (enforce)\nusr.sbin.rsyslogd (complain)\n",
        );
        tool(&root, APPARMOR_PARSER);
        tool(&root, AA_STATUS);
        tool(&root, "/usr/lib/polkit-1/polkitd");
        tool(&root, "/usr/bin/pkcheck");
        write(
            &root.join("usr/share/polkit-1/actions/org.example.policy"),
            "<policyconfig/>\n",
        );
        tool(&root, FIREWALL_CMD);
        fs::create_dir_all(root.join("etc/firewalld/zones")).expect("zones");
        fs::create_dir_all(root.join("run/firewalld")).expect("runtime");
        write(&root.join("etc/firewalld/zones/public.xml"), "<zone/>\n");
        write(
            &root.join("etc/firewalld/firewalld.conf"),
            "DefaultZone=work\n",
        );
        tool(&root, FLATPAK);
        tool(&root, BWRAP);
        fs::create_dir_all(root.join("var/lib/flatpak/app/org.example.One")).expect("app");
        fs::create_dir_all(root.join("var/lib/flatpak/app/org.example.Two")).expect("app");
        write(
            &root.join("var/lib/flatpak/repo/config"),
            "[remote \"flathub\"]\nurl=https://example.invalid\n[remote \"local\"]\n",
        );
        tool(&root, NMCLI);
        fs::create_dir_all(root.join("etc/NetworkManager/system-connections")).expect("nm");
        fs::create_dir_all(root.join("run/NetworkManager")).expect("nm run");
        write(
            &root.join("etc/NetworkManager/system-connections/home.nmconnection"),
            "[connection]\nid=home\n",
        );

        let facts = read(&root);
        assert_eq!(facts.apparmor.state(), "mixed");
        assert_eq!(facts.apparmor.profiles, 2);
        assert_eq!(facts.apparmor.enforce, 1);
        assert_eq!(facts.apparmor.complain, 1);
        assert!(facts.apparmor.reloadable());
        assert_eq!(facts.polkit.state(), "ready");
        assert_eq!(facts.polkit.actions, 1);
        assert_eq!(facts.firewall.state(), "running");
        assert_eq!(facts.firewall.default_zone.as_deref(), Some("work"));
        assert_eq!(facts.firewall.zones, 1);
        assert!(facts.firewall.reloadable());
        assert_eq!(facts.flatpak.state(), "ready");
        assert_eq!(facts.flatpak.apps, 2);
        assert_eq!(facts.flatpak.remotes, 2);
        assert!(facts.flatpak.catalog_reloadable());
        assert_eq!(facts.networkmanager.state(), "running");
        assert_eq!(facts.networkmanager.connections, 1);
        assert!(facts.networkmanager.reloadable());

        let lines = facts.to_lines().join("\n");
        assert!(lines.contains("svc.apparmor state=mixed enabled=1 profiles=2"));
        assert!(lines.contains("svc.firewall state=running"));
        assert!(lines.contains("svc.flatpak state=ready"));
        assert!(lines.contains("svc.networkmanager state=running"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn disabled_apparmor_is_not_reloadable_even_when_the_tools_are_present() {
        let root = scratch("disabled");
        write(&root.join("sys/module/apparmor/parameters/enabled"), "N\n");
        tool(&root, APPARMOR_PARSER);
        tool(&root, AA_STATUS);
        let facts = read(&root);
        assert_eq!(facts.apparmor.state(), "disabled");
        assert!(!facts.apparmor.reloadable());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_actions_are_closed_argument_vectors_with_bounded_deadlines() {
        let cases = [
            (
                ServiceAction::AppArmorReload,
                SYSTEMCTL,
                "try-reload-or-restart",
            ),
            (ServiceAction::FirewallReload, FIREWALL_CMD, "--reload"),
            (ServiceAction::StoreRefresh, FLATPAK, "update"),
            (ServiceAction::NetworkManagerReload, NMCLI, "general"),
        ];
        for (action, program, first) in cases {
            let plan = action.plan().expect("the closed action plans");
            assert_eq!(plan.program, program);
            assert_eq!(plan.args.first().map(String::as_str), Some(first));
            assert!(plan.stdin.is_none());
            assert!(plan.files.is_empty());
            assert!(plan.timeout_secs <= CATALOG_TIMEOUT_SECS);
            assert!(plan
                .notes
                .iter()
                .any(|note| note == &format!("service={}", action.token())));
        }
    }

    #[test]
    fn every_action_has_a_distinct_scope_and_token() {
        let mut scopes = Vec::new();
        let mut tokens = Vec::new();
        for action in ServiceAction::ALL {
            scopes.push(action.scope_token());
            tokens.push(action.token());
            assert!(ScopeAllows::is_resource(action.scope_token()));
        }
        scopes.sort_unstable();
        scopes.dedup();
        tokens.sort_unstable();
        tokens.dedup();
        assert_eq!(scopes.len(), ServiceAction::ALL.len());
        assert_eq!(tokens.len(), ServiceAction::ALL.len());
    }

    // Keep the scope check local to this module so the fact reader and action
    // planner can be tested without pulling the capability crate into the
    // fixture builder.
    struct ScopeAllows;

    impl ScopeAllows {
        fn is_resource(raw: &str) -> bool {
            raw.starts_with("service://") && raw.len() > "service://".len()
        }
    }
}
