//! The daemon's live state, and what every request does to it.
//!
//! This is the join between the Layer 2 and Layer 3 crates and the running
//! system. Those crates decide *what is true*; this holds one instance of each
//! of them, for the whole machine, and answers questions about it.
//!
//! It contains no socket code and no clock of its own: the caller supplies the
//! time. That keeps the entire behaviour of the daemon reachable from a test
//! that runs in microseconds on a host with no `/proc`, no Unix sockets and no
//! notion of what time it is.
//!
//! # What is not here yet, and is not pretended
//!
//! * The trash is in memory. Nothing has read `~/.local/share/Trash` from disk
//!   yet, so `TRASH LIST` on a real boot correctly reports nothing.
//! * `ENV SWITCH` moves the state machine. It does not start a virtual machine,
//!   because `litric-envd` does not exist. The reply says which it did.

use std::path::{Path, PathBuf};

use litric_caps::{Broker, Capability, CapabilityId, Grant, Issuer, Scope};
use litric_core::AppId;
use litric_env::{EnvironmentId, EnvironmentManager, SwitchOutcome};
use litric_storage::{Access, Layout, StorageBroker, StorageUri};
use litric_trash::{Retention, Trash};

use crate::action::{self, PowerAction, Runner, SystemRunner};
use crate::facts::{read_facts, Facts};
use crate::proto::{AccessWord, Request, Response};
use crate::providers::{self, BtAction, SystemAction};

/// The application the local session client is, for capability purposes.
///
/// This is a fact about the daemon's own configuration, **not** a claim from a
/// request: the socket is `0600` and owned by the daemon's user, so in Phase 1
/// there is exactly one kind of local client, and the daemon names it here
/// rather than asking the caller to name itself. A request that could choose
/// its own identity could choose the shell's, which is the confused-deputy
/// mistake the capability layer exists to prevent (ADR-0008, ADR-0016).
///
/// It is a constant rather than an argument because a daemon that could be
/// told which application it is speaking for would grant the caller whatever
/// the caller asked to be. When the authentication helper exists and peers can
/// be identified by credential (`SO_PEERCRED`), this becomes a per-connection
/// lookup and this constant becomes the fallback for an unidentified peer.
pub const LOCAL_CLIENT: &str = "os.litric.shell";

/// The local session client's identity, or `None` if the constant above is not
/// a valid application id.
///
/// `None` is a fail-closed answer rather than a panic: the daemon starts and
/// refuses every action, which is the correct behaviour for a build whose own
/// identity is malformed. A test asserts the `Some` case, so the failure is a
/// test failure rather than a discovery on a user's machine.
fn session_client() -> Option<AppId> {
    // `os.litric.*` is the platform's reserved namespace, which the ordinary
    // parser refuses on purpose — that refusal is what stops a third-party
    // package from impersonating a system component in a permission dialog.
    AppId::parse_platform_reserved(LOCAL_CLIENT).ok()
}

/// Seconds since the Unix epoch.
pub type Timestamp = u64;

/// The provider scopes the local session client holds, each with the capability
/// that owns it.
///
/// A table rather than a list of constructions so that a grant and the scope it
/// covers cannot drift apart, and so the test that counts grants has one place to
/// read. `iface://` and `wifi://` are two grants of one capability: a link and a
/// network are different resources with different consequences, and a single
/// grant covering both would make narrowing one impossible later.
pub const PROVIDER_GRANTS: [(CapabilityId, &str); 6] = [
    (CapabilityId::AudioSet, "audio://"),
    (CapabilityId::NetConfigure, "iface://"),
    (CapabilityId::NetConfigure, "wifi://"),
    (CapabilityId::BluetoothAdmin, "bt://"),
    (CapabilityId::AuthSession, "lock://"),
    (CapabilityId::ProcessSpawn, "app://"),
];

/// The whole daemon, minus its transport.
#[derive(Debug)]
pub struct Sysd {
    env: EnvironmentManager,
    trash: Trash,
    caps: Broker,
    storage: StorageBroker,
    /// Where `/proc` and `/sys` are rooted. `/` in production.
    root: PathBuf,
    /// Who the local session client is, or `None` when [`LOCAL_CLIENT`] is not
    /// a valid id. `None` means no action is granted to anyone.
    client: Option<AppId>,
    /// How a program is run. Real spawning in production, a recorder in tests.
    runner: Box<dyn Runner>,
    /// The default sink's volume and mute flag, as the sound server last
    /// answered, or `None` before it has been asked.
    ///
    /// Cached because the question costs a process: `FACTS` is polled twice a
    /// second and must not spawn anything, while the value is only needed after
    /// a change or when a panel opens. `None` is not zero — nothing has been
    /// measured, and the interface says so.
    audio: Option<(u8, bool)>,
    /// Whether the Bluetooth radio is powered on, as BlueZ last answered, on the
    /// same terms as [`Sysd::audio`] and for a stronger reason: the sysfs facts
    /// do not carry the radio's power at all, so this cache is the *only* place
    /// the interface can get a measurement from.
    bt: Option<bool>,
}

impl Sysd {
    /// Build the daemon's state for a machine of this size.
    ///
    /// `home` is the user's home directory, which the storage layout is derived
    /// from.
    #[must_use]
    pub fn new(total_cpus: u32, total_memory_mb: u32, home: &str, root: &Path) -> Self {
        let client = session_client();
        let mut caps = Broker::new();
        if let Some(c) = &client {
            Self::grant_local_session(&mut caps, c);
        }
        Self {
            env: EnvironmentManager::new(total_cpus, total_memory_mb),
            trash: Trash::new(Retention::default()),
            caps,
            storage: StorageBroker::new(Layout::for_home(home)),
            audio: None,
            bt: None,
            root: root.to_path_buf(),
            client,
            runner: Box::new(SystemRunner),
        }
    }

    /// Replace the command runner.
    ///
    /// The seam that makes every action reachable from a test: the alternative
    /// is a suite that suspends the machine it runs on.
    #[must_use]
    pub fn with_runner(mut self, runner: Box<dyn Runner>) -> Self {
        self.runner = runner;
        self
    }

    /// Which application this daemon treats as its local client.
    #[must_use]
    pub const fn client(&self) -> Option<&AppId> {
        self.client.as_ref()
    }

    /// Build the daemon by measuring the machine it is running on.
    ///
    /// Falls back to a single CPU and 512 MiB when `/proc` cannot be read,
    /// because a daemon that refuses to start on an unfamiliar system is worse
    /// than one that starts small and says so.
    #[must_use]
    pub fn from_system(home: &str, root: &Path) -> Self {
        let f = read_facts(root);
        let cpus = f.cpus.unwrap_or(1).max(1);
        let mem_mb = f
            .mem
            .map_or(512, |m| {
                // KiB to MiB. checked_div rather than the bare operator, which
                // the workspace denies so that every divisor is visibly not
                // zero at the point of use.
                let mb = m.total_kb.checked_div(1024).unwrap_or(0);
                u32::try_from(mb).unwrap_or(u32::MAX)
            })
            .max(64);
        Self::new(cpus, mem_mb, home, root)
    }

    /// The environment manager.
    #[must_use]
    pub const fn environments(&self) -> &EnvironmentManager {
        &self.env
    }

    /// The trash.
    #[must_use]
    pub const fn trash(&self) -> &Trash {
        &self.trash
    }

    /// Mutable access to the capability broker, for the parts of the system
    /// that issue grants.
    pub fn caps_mut(&mut self) -> &mut Broker {
        &mut self.caps
    }

    /// Read the current facts.
    #[must_use]
    pub fn facts(&self) -> Facts {
        read_facts(&self.root)
    }

    /// Handle one request.
    ///
    /// `now` is supplied rather than read so that expiry, revocation and
    /// retention are all reachable from a test.
    pub fn handle(&mut self, req: &Request, now: Timestamp) -> Response {
        match req {
            Request::Ping => Response::ok(vec!["pong".to_owned()]),
            Request::Quit => Response::ok(Vec::new()),
            Request::Facts => {
                let mut lines = self.facts().to_lines();
                // Only when a measurement exists. A machine whose sound server
                // has never answered has no level, and a line saying `volume=0`
                // would be this daemon inventing one.
                if let Some((volume, muted)) = self.audio {
                    lines.push(format!(
                        "audio.level volume={volume} muted={}",
                        u8::from(muted)
                    ));
                }
                // The same rule as the volume: a line exists only when a
                // measurement does, so `bt.powered=0` always means a radio BlueZ
                // said is off.
                if let Some(powered) = self.bt {
                    lines.push(format!("bt.powered={}", u8::from(powered)));
                }
                Response::ok(lines)
            }
            Request::AudioLevel => self.audio_level(),
            Request::BtStatus => self.bt_status(),
            Request::EnvList => Response::ok(self.env_list()),
            Request::EnvPlan(token) => self.env_plan(token),
            Request::EnvSwitch(token) => self.env_switch(token),
            Request::TrashList => Response::ok(self.trash_list()),
            Request::TrashPlanEmpty => Response::ok(self.trash_plan_empty(now)),
            Request::CapsAudit => Response::ok(self.caps_audit()),
            Request::Power { action, consent } => self.power(*action, *consent, now),
            Request::Action { action, consent } => self.action(action, *consent, now),
            Request::Resolve { app, uri, access } => self.resolve(app, uri, *access, now),
            Request::LaunchApp { argv } => self.launch_app(argv),
        }
    }

    /// Start a program the user chose, if the session client may ask.
    ///
    /// The capability is checked here, with the same broker as every other
    /// action, before the vector reaches [`crate::launch`]. The daemon's own
    /// root is deliberately not consulted: a program is named by its own
    /// absolute path, and a request that could choose a root could choose which
    /// binary the path meant.
    fn launch_app(&mut self, argv: &[String]) -> Response {
        let Some(client) = self.client.clone() else {
            return Response::err("refused", "this build has no local client identity");
        };
        let Some(scope) = Scope::parse_resource("app://").ok() else {
            return Response::err("bad-argument", "no scope for this action");
        };
        let Some(capability) = Capability::new(CapabilityId::ProcessSpawn, scope) else {
            return Response::err("bad-argument", "no capability for this action");
        };
        let decision = self.caps.check_with_consent(&client, &capability, false, 0);
        if !decision.is_allowed() {
            return Response::err(
                "refused",
                format!("{} {capability}", decision.reason().token()),
            );
        }
        match crate::launch::launch(argv) {
            Ok(pid) => {
                let program = argv.first().map_or("the program", String::as_str);
                let name = Path::new(program)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| program.to_owned());
                Response::ok(vec![format!("pid={pid}"), format!("program={name}")])
            }
            Err(error) => Response::err(error.kind(), error.detail()),
        }
    }

    /// Ask the sound server where the volume is, and remember the answer.
    ///
    /// Allowed to anyone who can reach the socket, like [`Request::Facts`]: it
    /// is a question about the machine this process is running on, it changes
    /// nothing, and the shell is the only client that can reach the socket at
    /// all (mode 0600, owner root).
    fn audio_level(&mut self) -> Response {
        match providers::audio_level(self.runner.as_ref()) {
            Ok((volume, muted)) => {
                self.audio = Some((volume, muted));
                Response::ok(vec![format!(
                    "audio.level volume={volume} muted={}",
                    u8::from(muted)
                )])
            }
            Err(reason) => Response::err("audio-unavailable", reason),
        }
    }

    /// Ask BlueZ whether the radio is on, and remember the answer.
    ///
    /// Allowed to anyone who can reach the socket, exactly as
    /// [`Sysd::audio_level`] is: it is a question about the machine this process
    /// runs on, it changes nothing, and the socket is mode 0600 and owned by root.
    fn bt_status(&mut self) -> Response {
        match providers::bt_powered(self.runner.as_ref()) {
            Ok(powered) => {
                self.bt = Some(powered);
                Response::ok(vec![format!("bt.powered={}", u8::from(powered))])
            }
            Err(reason) => Response::err("bt-unavailable", reason),
        }
    }

    /// Perform a power transition, if the client may and the user said yes.
    ///
    /// Two questions, asked in this order and both by the broker: does the
    /// local client hold `PowerControl` scoped to *this* verb, and did the user
    /// consent in the moment. A grant is permission to ask; consent is the
    /// answer to one ask. Neither substitutes for the other, which is why this
    /// goes through [`Broker::check_with_consent`] rather than `check`.
    fn power(&mut self, action: PowerAction, consent: bool, now: Timestamp) -> Response {
        let Some(capability) = action.capability() else {
            // Unreachable for the three constants, and refused rather than
            // panicked if a fourth verb is ever added without a scope.
            return Response::err("bad-argument", format!("no scope for {action}"));
        };
        let Some(client) = self.client.clone() else {
            return Response::err("refused", "this build has no local client identity");
        };
        let decision = self
            .caps
            .check_with_consent(&client, &capability, consent, now);
        if !decision.is_allowed() {
            // The reason token is the machine-readable half: a shell can tell
            // `needs-live-consent` (show the prompt) from `no-grant` (this build
            // cannot do it at all) without parsing prose.
            return Response::err(
                "refused",
                format!("{} {capability}", decision.reason().token()),
            );
        }
        let outcome = action::power(action, self.runner.as_ref());
        if outcome.applied {
            Response::ok(outcome.lines)
        } else {
            // The transition was authorised and did not happen. That is a
            // failure the caller must see as one: a shell that reported "the
            // machine is going down" over a machine that is still up would be
            // lying at the one moment the user is watching.
            let detail = outcome
                .lines
                .iter()
                .find_map(|l| l.strip_prefix("detail="))
                .unwrap_or("the transition did not happen")
                .to_owned();
            Response::err("action-failed", detail)
        }
    }

    /// Perform a provider action: sound, the network, Bluetooth, a credential
    /// check.
    ///
    /// The same two-part question as [`Sysd::power`] — may this client ask, and
    /// did the user say yes — and the same broker answers it. The consent half
    /// only decides anything for a verification, whose capability is a
    /// live-consent one; for the reversible three it is reported and ignored, so
    /// that one code path serves all four families.
    fn action(&mut self, action: &SystemAction, consent: bool, now: Timestamp) -> Response {
        let Some(capability) = action.capability() else {
            return Response::err("bad-argument", "no scope for this action");
        };
        let Some(client) = self.client.clone() else {
            return Response::err("refused", "this build has no local client identity");
        };
        let decision = self
            .caps
            .check_with_consent(&client, &capability, consent, now);
        if !decision.is_allowed() {
            return Response::err(
                "refused",
                format!("{} {capability}", decision.reason().token()),
            );
        }
        // The root is the daemon's own, never a request's: the same rule as the
        // facts reader. A caller that could choose the root could choose which
        // credential file `iwd` is told about.
        let outcome = providers::perform(action, self.runner.as_ref(), &self.root);
        if outcome.applied {
            // The level that was just set is read back rather than assumed: the
            // tool accepting a number is not the same as the sink taking it, and
            // a panel showing the number somebody typed is not showing the
            // machine. A failed readback leaves the previous measurement in
            // place instead of replacing it with a guess.
            if matches!(action, SystemAction::Audio(_)) {
                if let Ok(level) = providers::audio_level(self.runner.as_ref()) {
                    self.audio = Some(level);
                }
            }
            // A radio that accepted the request is not the same as a radio that
            // is on: `bluetoothctl` returns when BlueZ has taken the property,
            // and the property is what the panel must show. Only a power change
            // is re-read; a scan or a pairing says nothing about power.
            if matches!(action, SystemAction::Bluetooth(BtAction::Power(_))) {
                if let Ok(powered) = providers::bt_powered(self.runner.as_ref()) {
                    self.bt = Some(powered);
                }
            }
            return Response::ok(outcome.lines);
        }
        let reason = outcome
            .lines
            .iter()
            .find_map(|l| l.strip_prefix("reason="))
            .unwrap_or("unknown");
        let detail = outcome
            .lines
            .iter()
            .find_map(|l| l.strip_prefix("detail="))
            .unwrap_or("the action did not happen")
            .to_owned();
        // A value the caller got wrong is the caller's error; everything else —
        // a tool that is not there, a helper that refused, a file that could not
        // be written — is the action failing, which is a different token for the
        // same reason the refusal reasons are: a client can tell "you sent
        // nonsense" from "this machine said no" without reading prose.
        let kind = if reason == "bad-argument" {
            "bad-argument"
        } else {
            "action-failed"
        };
        Response::err(kind, detail)
    }

    /// Grant the local session client the capabilities this build can honour.
    ///
    /// Nothing here invents authority: the set is exactly the actions that have
    /// an implementation, and each of them is still subject to consent in the
    /// moment. The issuer is [`Issuer::Policy`] and not `User` because nobody
    /// answered a prompt for it — saying otherwise in the audit log would make
    /// the log say the user agreed to something they were never shown.
    ///
    /// Four of the five scopes are the broad form on purpose: the shell does not
    /// know which SSID, which device or which interface a user is about to choose
    /// when the grant is issued, and the narrower form would make every such
    /// choice a request for a new grant. `Scope::parse_resource` spells that as
    /// an empty path (`wifi://`), which reads as "every network" rather than as
    /// a prefix match. Each individual request is still checked and audited.
    fn grant_local_session(broker: &mut Broker, client: &AppId) {
        broker.register(client.clone());
        for action in PowerAction::ALL {
            if let Some(capability) = action.capability() {
                broker.grant(client, Grant::new(capability, Issuer::Policy, 0));
            }
        }
        for (id, raw) in PROVIDER_GRANTS {
            let Ok(scope) = Scope::parse_resource(raw) else {
                continue;
            };
            let Some(capability) = Capability::new(id, scope) else {
                continue;
            };
            broker.grant(client, Grant::new(capability, Issuer::Policy, 0));
        }
    }

    fn env_list(&self) -> Vec<String> {
        let reservation = self.env.reservation();
        let active = self.env.active();
        let mut out = vec![
            format!("active={}", active.token()),
            format!(
                "host.reserved_cpus={} host.reserved_memory_mb={}",
                reservation.cpus, reservation.memory_mb
            ),
        ];
        for e in self.env.all() {
            out.push(format!(
                "env={} state={} health={} cpus={} memory_mb={} host={}",
                e.id().token(),
                e.state().token(),
                e.health().token(),
                e.budget().cpus,
                e.budget().memory_mb,
                e.id().is_host()
            ));
        }
        out
    }

    fn lookup(token: &str) -> Option<EnvironmentId> {
        EnvironmentId::all()
            .iter()
            .copied()
            .find(|e| e.token() == token)
    }

    fn outcome_lines(outcome: &SwitchOutcome) -> Vec<String> {
        let word = match outcome {
            SwitchOutcome::AlreadyActive => "already-active",
            SwitchOutcome::Presented => "presented",
            SwitchOutcome::Resuming => "resuming",
            SwitchOutcome::Restoring => "restoring",
            SwitchOutcome::Starting => "starting",
            SwitchOutcome::NotInstalled { .. } => "not-installed",
            SwitchOutcome::NeedsRecovery { .. } => "needs-recovery",
            SwitchOutcome::Failed => "failed",
            // SwitchOutcome is non-exhaustive. A variant added upstream must
            // report as unrecognised rather than be folded into the nearest
            // familiar word, which would make a new failure mode look like an
            // old success.
            _ => "unrecognised",
        };
        let latency = match outcome.latency_class() {
            litric_env::LatencyClass::Instant => "instant",
            litric_env::LatencyClass::Seconds => "seconds",
            litric_env::LatencyClass::TensOfSeconds => "tens-of-seconds",
            litric_env::LatencyClass::NoSwitch => "no-switch",
        };
        let mut out = vec![
            format!("outcome={word}"),
            format!("latency={latency}"),
            format!("will_switch={}", outcome.will_switch()),
        ];
        if let SwitchOutcome::NotInstalled {
            user_must_supply_media,
        } = outcome
        {
            out.push(format!("user_must_supply_media={user_must_supply_media}"));
        }
        if let SwitchOutcome::NeedsRecovery { health } = outcome {
            out.push(format!("health={}", health.token()));
        }
        out
    }

    fn env_plan(&self, token: &str) -> Response {
        match Self::lookup(token) {
            Some(id) => {
                let mut lines = vec!["applied=false".to_owned()];
                lines.extend(Self::outcome_lines(&self.env.plan_switch(id)));
                Response::ok(lines)
            }
            None => Response::err("unknown-environment", token),
        }
    }

    fn env_switch(&mut self, token: &str) -> Response {
        let Some(id) = Self::lookup(token) else {
            return Response::err("unknown-environment", token);
        };
        let outcome = self.env.switch_to(id);
        let mut lines = vec![
            // The state machine moved. No virtual machine did, because
            // litric-envd does not exist yet, and a reply that did not say so
            // would be the daemon claiming work it had not done.
            "applied=state-only".to_owned(),
            format!("active={}", self.env.active().token()),
        ];
        lines.extend(Self::outcome_lines(&outcome));
        Response::ok(lines)
    }

    fn trash_list(&self) -> Vec<String> {
        let mut out = vec![
            format!("count={}", self.trash.len()),
            format!("used_bytes={}", self.trash.used_bytes()),
            format!("capacity_bytes={}", self.trash.retention().capacity_bytes),
        ];
        for item in self.trash.items() {
            out.push(format!(
                "item={} size={} deleted_at={} original={}",
                item.name(),
                item.size(),
                item.deleted_at(),
                item.original().to_uri()
            ));
        }
        out
    }

    fn trash_plan_empty(&self, now: Timestamp) -> Vec<String> {
        let plan = self.trash.plan_auto_empty(now);
        let reason = match plan.reason {
            litric_trash::EmptyReason::Nothing => "nothing",
            litric_trash::EmptyReason::Age => "age",
            litric_trash::EmptyReason::Capacity => "capacity",
            litric_trash::EmptyReason::AgeAndCapacity => "age-and-capacity",
            _ => "unrecognised",
        };
        let mut out = vec![
            format!("reason={reason}"),
            format!("bytes_freed={}", plan.bytes_freed),
            format!("count={}", plan.remove.len()),
        ];
        for name in &plan.remove {
            out.push(format!("remove={name}"));
        }
        out
    }

    fn caps_audit(&self) -> Vec<String> {
        let mut out = vec![format!("count={}", self.caps.audit_log().len())];
        for e in self.caps.audit_log() {
            out.push(format!(
                "at={} app={} capability={} allowed={} reason={}",
                e.at,
                e.app.as_str(),
                e.request,
                e.allowed,
                e.reason.token()
            ));
        }
        out
    }

    fn resolve(&mut self, app: &str, uri: &str, access: AccessWord, now: Timestamp) -> Response {
        let Ok(app_id) = AppId::parse(app) else {
            return Response::err("bad-app-id", app);
        };
        let parsed = match StorageUri::parse(uri) {
            Ok(u) => u,
            Err(e) => return Response::err("bad-uri", e.to_string()),
        };
        let access = match access {
            AccessWord::Read => Access::Read,
            AccessWord::Write => Access::Write,
            AccessWord::List => Access::List,
        };

        // The granted set comes from the broker, never from the request. This
        // is the confused-deputy defence: the daemon has more authority than
        // any of its callers, so it must never act on a claim of identity or
        // permission that arrived in the message.
        let granted: Vec<Capability> = self.caps.active_capabilities(&app_id, now);
        match self.storage.resolve(&app_id, &parsed, access, &granted) {
            Ok(path) => Response::ok(vec![
                format!("uri={}", parsed.to_uri()),
                format!("path={path}"),
            ]),
            Err(e) => Response::err("denied", e.to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use litric_caps::{Capability, CapabilityId, Grant, Issuer, Scope};

    const NOW: Timestamp = 1_800_000_000;

    fn sysd() -> Sysd {
        Sysd::new(8, 16_384, "/home/alice", Path::new("no-such-root"))
    }

    fn line<'a>(r: &'a Response, prefix: &str) -> Option<&'a String> {
        r.lines().iter().find(|l| l.starts_with(prefix))
    }

    #[test]
    fn the_audio_level_is_a_measurement_and_not_a_default() {
        // Nothing has been asked yet, so FACTS carries no level at all: a
        // `volume=0` line here would be this daemon inventing a mute machine.
        let runner = RecordingRunner::succeeding(0).answering(0, "Volume: 0.60\n");
        let mut s = sysd_with(&runner);
        let before = s.handle(&Request::Facts, NOW);
        assert!(
            line(&before, "audio.level").is_none(),
            "{:?}",
            before.lines()
        );

        let asked = s.handle(&Request::AudioLevel, NOW);
        assert!(asked.is_ok(), "{}", asked.encode());
        assert_eq!(
            line(&asked, "audio.level"),
            Some(&"audio.level volume=60 muted=0".to_owned())
        );
        // And from then on the twice-a-second facts poll carries it without
        // asking anything: the question costs a process.
        let calls = runner.calls().len();
        let after = s.handle(&Request::Facts, NOW);
        assert_eq!(
            line(&after, "audio.level"),
            Some(&"audio.level volume=60 muted=0".to_owned()),
            "{:?}",
            after.lines()
        );
        assert_eq!(
            runner.calls().len(),
            calls,
            "a facts poll must not run a tool"
        );
    }

    #[test]
    fn a_sound_server_that_cannot_be_asked_says_so() {
        let runner = RecordingRunner::failing(std::io::ErrorKind::NotFound, "no wpctl here");
        let mut s = sysd_with(&runner);
        let reply = s.handle(&Request::AudioLevel, NOW);
        assert!(!reply.is_ok());
        assert!(
            reply.encode().starts_with("ERR audio-unavailable"),
            "{}",
            reply.encode()
        );
        assert!(
            line(&reply, "audio.level").is_none(),
            "an unanswerable question must not produce a number"
        );
        // And no level appears in the facts either.
        let facts = s.handle(&Request::Facts, NOW);
        assert!(line(&facts, "audio.level").is_none());
    }

    #[test]
    fn a_volume_change_is_read_back_rather_than_assumed() {
        use crate::providers::AudioAction;
        // The tool accepts the number, and then the sink reports something
        // else: the panel must show the sink's answer.
        let runner = RecordingRunner::succeeding(0)
            .answering(0, "Volume: 0.35\n")
            .answering(0, "Volume: 0.35 [MUTED]\n");
        let mut s = sysd_with(&runner);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Audio(AudioAction::Volume(35)),
                consent: false,
            },
            NOW,
        );
        assert!(reply.is_ok(), "{}", reply.encode());
        let facts = s.handle(&Request::Facts, NOW);
        assert_eq!(
            line(&facts, "audio.level"),
            Some(&"audio.level volume=35 muted=1".to_owned()),
            "the readback is the measurement, not the requested number"
        );
    }

    #[test]
    fn the_radio_power_is_a_measurement_and_not_an_echo() {
        // `bluetoothctl show` says the controller exists and is powered off.
        let runner = RecordingRunner::succeeding(0)
            .answering(0, "Controller 00:1A:7D:DA:71:13 (public)\n\tPowered: no\n");
        let mut s = sysd_with(&runner);
        // Nothing has been asked yet: the sysfs facts carry no radio power, so
        // there is no line until BlueZ answers. A `bt.powered=0` invented here
        // would be this daemon describing a radio it has not looked at.
        let before = s.handle(&Request::Facts, NOW);
        assert!(
            line(&before, "bt.powered").is_none(),
            "{:?}",
            before.lines()
        );

        let asked = s.handle(&Request::BtStatus, NOW);
        assert!(asked.is_ok(), "{}", asked.encode());
        assert_eq!(line(&asked, "bt.powered"), Some(&"bt.powered=0".to_owned()));

        // The twice-a-second poll carries the measurement without running a
        // tool: the question costs a process, and a poll must not.
        let calls = runner.calls().len();
        let after = s.handle(&Request::Facts, NOW);
        assert_eq!(line(&after, "bt.powered"), Some(&"bt.powered=0".to_owned()));
        assert_eq!(
            runner.calls().len(),
            calls,
            "a facts poll must not run a tool"
        );
    }

    #[test]
    fn a_radio_that_cannot_be_asked_says_so() {
        let runner = RecordingRunner::failing(std::io::ErrorKind::NotFound, "no bluetoothctl");
        let mut s = sysd_with(&runner);
        let reply = s.handle(&Request::BtStatus, NOW);
        assert!(!reply.is_ok());
        assert!(
            reply.encode().starts_with("ERR bt-unavailable"),
            "{}",
            reply.encode()
        );
        assert!(line(&reply, "bt.powered").is_none());
        // A machine whose radio cannot be read has no power state, and the
        // facts say nothing rather than saying off.
        let facts = s.handle(&Request::Facts, NOW);
        assert!(line(&facts, "bt.powered").is_none());
    }

    #[test]
    fn a_radio_change_is_read_back_rather_than_assumed() {
        use crate::providers::BtAction;
        // The tool accepts `power on`, and BlueZ then reports the radio on. The
        // request is not the state; the answer to the question that follows is.
        let runner = RecordingRunner::succeeding(0)
            .answering(0, "Controller 00:1A:7D:DA:71:13 (public)\n\tPowered: yes\n");
        let mut s = sysd_with(&runner);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Bluetooth(BtAction::Power(true)),
                consent: false,
            },
            NOW,
        );
        assert!(reply.is_ok(), "{}", reply.encode());
        let facts = s.handle(&Request::Facts, NOW);
        assert_eq!(line(&facts, "bt.powered"), Some(&"bt.powered=1".to_owned()));

        // A change that is not a power change says nothing about power, so the
        // last measurement stands rather than being replaced by a guess.
        let before = runner.calls().len();
        let scan = s.handle(
            &Request::Action {
                action: SystemAction::Bluetooth(BtAction::Scan(true)),
                consent: false,
            },
            NOW,
        );
        assert!(scan.is_ok(), "{}", scan.encode());
        assert_eq!(
            runner.calls().len(),
            before.saturating_add(1),
            "a scan is one call and no readback"
        );
        let facts = s.handle(&Request::Facts, NOW);
        assert_eq!(line(&facts, "bt.powered"), Some(&"bt.powered=1".to_owned()));
    }

    #[test]
    fn ping_answers() {
        let mut s = sysd();
        let r = s.handle(&Request::Ping, NOW);
        assert!(r.is_ok());
        assert_eq!(r.lines(), ["pong".to_owned()]);
    }

    #[test]
    fn env_list_reports_every_environment_and_the_host_reservation() {
        let mut s = sysd();
        let r = s.handle(&Request::EnvList, NOW);
        assert!(r.is_ok());
        assert!(line(&r, "active=").is_some());
        assert!(
            line(&r, "host.reserved_cpus=2").is_some(),
            "{:?}",
            r.lines()
        );
        let envs = r.lines().iter().filter(|l| l.starts_with("env=")).count();
        assert_eq!(envs, EnvironmentId::all().len());
    }

    #[test]
    fn planning_a_switch_changes_nothing() {
        let mut s = sysd();
        let before = s.environments().active();
        let target = EnvironmentId::all()
            .iter()
            .copied()
            .find(|e| *e != before)
            .expect("more than one environment");
        let r = s.handle(&Request::EnvPlan(target.token().to_owned()), NOW);
        assert!(r.is_ok());
        assert!(line(&r, "applied=false").is_some());
        assert_eq!(
            s.environments().active(),
            before,
            "PLAN must not move the active environment"
        );
    }

    #[test]
    fn a_switch_says_it_only_moved_the_state_machine() {
        // The daemon must never report work it did not do. litric-envd does not
        // exist, so no virtual machine changed state, and the reply says so.
        let mut s = sysd();
        let r = s.handle(&Request::EnvSwitch("litric".to_owned()), NOW);
        assert!(r.is_ok(), "{:?}", r.encode());
        assert!(line(&r, "applied=state-only").is_some(), "{:?}", r.lines());
    }

    #[test]
    fn an_unknown_environment_is_refused_by_name() {
        let mut s = sysd();
        let r = s.handle(&Request::EnvSwitch("beos".to_owned()), NOW);
        assert!(!r.is_ok());
        assert_eq!(r.encode(), "ERR unknown-environment beos\n");
    }

    #[test]
    fn an_empty_trash_reports_zero_rather_than_failing() {
        let mut s = sysd();
        let r = s.handle(&Request::TrashList, NOW);
        assert!(r.is_ok());
        assert!(line(&r, "count=0").is_some());
        let p = s.handle(&Request::TrashPlanEmpty, NOW);
        assert!(line(&p, "reason=nothing").is_some(), "{:?}", p.lines());
        assert!(
            s.trash().is_empty(),
            "nothing has read the on-disk trash yet, and the daemon must not              invent entries to look complete"
        );
    }

    #[test]
    fn resolving_without_a_grant_is_denied() {
        let mut s = sysd();
        let r = s.handle(
            &Request::Resolve {
                app: "com.example.editor".to_owned(),
                uri: "user://Documents/notes.md".to_owned(),
                access: AccessWord::Read,
            },
            NOW,
        );
        assert!(!r.is_ok(), "an ungranted read must not resolve");
        assert!(r.encode().starts_with("ERR denied"), "{}", r.encode());
    }

    #[test]
    fn resolving_with_a_grant_returns_a_real_path() {
        let mut s = sysd();
        let app = AppId::parse("com.example.editor").expect("valid id");
        let scope = Scope::parse_uri("user://Documents").expect("valid scope");
        let cap = Capability::new(CapabilityId::FsRead, scope).expect("valid capability");
        s.caps_mut().register(app.clone());
        s.caps_mut().grant(&app, Grant::new(cap, Issuer::User, NOW));

        let r = s.handle(
            &Request::Resolve {
                app: "com.example.editor".to_owned(),
                uri: "user://Documents/notes.md".to_owned(),
                access: AccessWord::Read,
            },
            NOW,
        );
        assert!(r.is_ok(), "{}", r.encode());
        assert_eq!(
            line(&r, "path="),
            Some(&"path=/home/alice/Documents/notes.md".to_owned()),
            "{:?}",
            r.lines()
        );
    }

    #[test]
    fn a_grant_that_has_expired_no_longer_resolves() {
        let mut s = sysd();
        let app = AppId::parse("com.example.editor").expect("valid id");
        let scope = Scope::parse_uri("user://Documents").expect("valid scope");
        let cap = Capability::new(CapabilityId::FsRead, scope).expect("valid capability");
        s.caps_mut().register(app.clone());
        s.caps_mut()
            .grant(&app, Grant::expiring(cap, Issuer::User, NOW, NOW + 60));

        let req = Request::Resolve {
            app: "com.example.editor".to_owned(),
            uri: "user://Documents/notes.md".to_owned(),
            access: AccessWord::Read,
        };
        assert!(s.handle(&req, NOW + 30).is_ok(), "still inside the window");
        assert!(
            !s.handle(&req, NOW + 61).is_ok(),
            "an expired grant must stop resolving without anyone revoking it"
        );
    }

    #[test]
    fn one_app_cannot_resolve_another_apps_private_storage() {
        let mut s = sysd();
        let r = s.handle(
            &Request::Resolve {
                app: "com.example.editor".to_owned(),
                uri: "app://com.example.other/secret".to_owned(),
                access: AccessWord::Read,
            },
            NOW,
        );
        assert!(!r.is_ok(), "{}", r.encode());
    }

    #[test]
    fn a_malformed_app_id_is_rejected_before_anything_else_happens() {
        let mut s = sysd();
        let r = s.handle(
            &Request::Resolve {
                app: "../../etc".to_owned(),
                uri: "user://Documents/x".to_owned(),
                access: AccessWord::Read,
            },
            NOW,
        );
        assert!(r.encode().starts_with("ERR bad-app-id"), "{}", r.encode());
    }

    // ---- the write half: power transitions --------------------------------

    use crate::action::testing::RecordingRunner;
    use crate::action::{PowerAction, SYSTEMCTL};

    /// A daemon that runs no command, plus the record of what it would have
    /// run. The clone is the point: the daemon takes one, the test keeps the
    /// other.
    fn sysd_with(runner: &RecordingRunner) -> Sysd {
        sysd().with_runner(Box::new(runner.clone()))
    }

    #[test]
    fn the_local_client_is_the_platform_shell_and_holds_only_implemented_actions() {
        let s = sysd();
        let client = s.client().expect("the constant is a valid platform id");
        assert!(client.is_platform(), "{client}");
        let held = s.caps.grants_for(client).len();
        assert_eq!(
            held,
            PowerAction::ALL.len() + PROVIDER_GRANTS.len(),
            "exactly the actions with an implementation are granted"
        );
    }

    #[test]
    fn a_power_request_without_consent_never_runs_anything() {
        // The property the whole design turns on: the grant is permission to
        // ask, and asking is not doing.
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Power {
                action: PowerAction::PowerOff,
                consent: false,
            },
            NOW,
        );
        assert!(!reply.is_ok(), "{}", reply.encode());
        assert!(
            reply.encode().contains("needs-live-consent"),
            "{}",
            reply.encode()
        );
        assert!(
            r.calls().is_empty(),
            "an unconfirmed request must not reach the tool: {:?}",
            r.calls()
        );
    }

    #[test]
    fn a_confirmed_power_request_runs_the_base_tool_once() {
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Power {
                action: PowerAction::Suspend,
                consent: true,
            },
            NOW,
        );
        assert!(reply.is_ok(), "{}", reply.encode());
        assert_eq!(r.calls().len(), 1, "exactly one transition was asked for");
        assert!(
            line(&reply, "applied=true").is_some(),
            "{:?}",
            reply.lines()
        );
        assert_eq!(
            line(&reply, "command="),
            Some(&format!("command={SYSTEMCTL} suspend")),
            "{:?}",
            reply.lines()
        );
    }

    #[test]
    fn a_transition_that_did_not_happen_is_an_error_not_a_success() {
        let r = RecordingRunner::failing(std::io::ErrorKind::NotFound, "no such file");
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Power {
                action: PowerAction::Reboot,
                consent: true,
            },
            NOW,
        );
        assert!(!reply.is_ok(), "a machine that is still up is not a reboot");
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
    }

    #[test]
    fn a_revoked_grant_cannot_power_the_machine_off() {
        // Consent does not manufacture authority, and neither does a grant that
        // has been taken back: revoking every grant and then confirming the
        // power-off must refuse, and must not reach the tool.
        let r = RecordingRunner::succeeding(0);
        let mut s = Sysd::new(8, 16_384, "/home/alice", Path::new("no-such-root"))
            .with_runner(Box::new(r.clone()));
        let client = s.client().expect("client").clone();
        s.caps_mut().revoke_all(&client, NOW);
        let reply = s.handle(
            &Request::Power {
                action: PowerAction::PowerOff,
                consent: true,
            },
            NOW,
        );
        assert!(!reply.is_ok(), "{}", reply.encode());
        assert!(reply.encode().contains("revoked"), "{}", reply.encode());
        assert!(r.calls().is_empty(), "nothing may run without a grant");
    }

    #[test]
    fn every_power_request_appears_in_the_audit_log() {
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let before = s.caps.audit_log().len();
        let _ = s.handle(
            &Request::Power {
                action: PowerAction::Suspend,
                consent: false,
            },
            NOW,
        );
        let log = s.caps.audit_log();
        assert_eq!(log.len(), before + 1, "a refusal is a decision too");
        assert!(log.last().is_some_and(|e| !e.allowed));
    }

    // ---- the write half: providers ----------------------------------------

    #[test]
    fn an_authorised_volume_change_runs_the_audio_tool_once() {
        use crate::providers::{AudioAction, SystemAction};
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Audio(AudioAction::Volume(25)),
                consent: false,
            },
            NOW,
        );
        assert!(reply.is_ok(), "{}", reply.encode());
        // Two calls, and they are the two halves of one change: the set, and
        // the read-back that says what the sink actually holds. A test that
        // accepted one call would pass while the panel showed the number
        // somebody requested rather than the machine's.
        let calls = r.calls();
        assert_eq!(calls.len(), 2, "{calls:?}");
        let first = calls.first().expect("the set");
        let second = calls.get(1).expect("the read-back");
        assert_eq!(first.0, "/usr/bin/wpctl");
        assert_eq!(first.1.first().map(String::as_str), Some("set-volume"));
        assert_eq!(second.1.first().map(String::as_str), Some("get-volume"));
        assert!(
            line(&reply, "command=/usr/bin/wpctl").is_some(),
            "{:?}",
            reply.lines()
        );
    }

    #[test]
    fn a_provider_action_without_a_grant_runs_nothing() {
        // The grant is what makes a second client unable to change the volume,
        // the network or the radio on this machine.
        use crate::providers::{AudioAction, SystemAction};
        let r = RecordingRunner::succeeding(0);
        let mut s = Sysd::new(8, 16_384, "/home/alice", Path::new("no-such-root"))
            .with_runner(Box::new(r.clone()));
        let client = s.client().expect("client").clone();
        s.caps_mut().revoke_all(&client, NOW);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Audio(AudioAction::Volume(25)),
                consent: false,
            },
            NOW,
        );
        assert!(!reply.is_ok(), "{}", reply.encode());
        assert!(reply.encode().contains("revoked"), "{}", reply.encode());
        assert!(r.calls().is_empty(), "{:?}", r.calls());
    }

    #[test]
    fn a_verification_without_the_confirmation_word_never_reaches_the_helper() {
        // `AuthSession` is a live-consent capability, and this is the request
        // that proves it is enforced at the daemon rather than by the caller's
        // good manners: a line carrying a password and no `CONFIRM` word must
        // not be checked against anything.
        use crate::providers::{AuthAction, SystemAction};
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Auth(AuthAction::Verify {
                    user: "alice".to_owned(),
                    password: "correct horse".to_owned(),
                }),
                consent: false,
            },
            NOW,
        );
        assert!(!reply.is_ok(), "{}", reply.encode());
        assert!(
            reply.encode().contains("needs-live-consent"),
            "{}",
            reply.encode()
        );
        assert!(r.calls().is_empty(), "nothing may run: {:?}", r.calls());
        assert_eq!(r.stdin_len(), None, "not even fed to a helper");
    }

    #[test]
    fn a_confirmed_verification_reaches_the_helper_with_the_password_on_stdin() {
        use crate::providers::{AuthAction, SystemAction};
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Auth(AuthAction::Verify {
                    user: "alice".to_owned(),
                    password: "correct horse".to_owned(),
                }),
                consent: true,
            },
            NOW,
        );
        assert!(reply.is_ok(), "{}", reply.encode());
        assert_eq!(r.stdin_len(), Some(14), "thirteen characters and a newline");
        let encoded = reply.encode();
        assert!(
            !encoded.contains("correct horse"),
            "a password must not come back in a reply: {encoded}"
        );
    }

    #[test]
    fn a_provider_action_that_failed_is_an_error_and_says_which_kind() {
        use crate::providers::{AudioAction, SystemAction};
        let r = RecordingRunner::succeeding(1);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Audio(AudioAction::Mute(true)),
                consent: false,
            },
            NOW,
        );
        assert!(!reply.is_ok(), "a tool that exited 1 did not do the thing");
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
    }

    #[test]
    fn a_provider_action_with_an_illegal_argument_is_the_callers_error() {
        use crate::providers::{NetAction, SystemAction};
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let reply = s.handle(
            &Request::Action {
                action: SystemAction::Net(NetAction::SetLink {
                    iface: "eth/0".to_owned(),
                    up: true,
                }),
                consent: false,
            },
            NOW,
        );
        assert!(
            reply.encode().starts_with("ERR bad-argument"),
            "{}",
            reply.encode()
        );
        assert!(r.calls().is_empty(), "{:?}", r.calls());
    }

    #[test]
    fn every_provider_request_appears_in_the_audit_log() {
        use crate::providers::{AudioAction, SystemAction};
        let r = RecordingRunner::succeeding(0);
        let mut s = sysd_with(&r);
        let before = s.caps.audit_log().len();
        let _ = s.handle(
            &Request::Action {
                action: SystemAction::Audio(AudioAction::Volume(10)),
                consent: false,
            },
            NOW,
        );
        let log = s.caps.audit_log();
        assert_eq!(log.len(), before + 1);
        assert!(
            log.last().is_some_and(|e| e.allowed),
            "an allowed action is a decision too"
        );
    }

    #[test]
    fn facts_from_an_unreadable_root_are_absent_rather_than_zero() {
        let mut s = sysd();
        let r = s.handle(&Request::Facts, NOW);
        assert!(r.is_ok());
        assert!(
            r.lines().is_empty(),
            "a host with no /proc must report nothing, not zeroes: {:?}",
            r.lines()
        );
    }

    #[test]
    fn from_system_falls_back_instead_of_refusing_to_start() {
        let s = Sysd::from_system("/home/alice", Path::new("no-such-root"));
        // One CPU and 512 MiB, minus the host reservation, is a machine the
        // manager can still describe.
        assert!(!s.environments().all().is_empty());
    }
}
