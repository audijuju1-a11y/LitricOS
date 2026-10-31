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
use litric_core::{AppId, Version};
use litric_env::{EnvironmentId, EnvironmentManager, SwitchOutcome};
use litric_storage::{Access, Layout, StorageBroker, StorageUri};
use litric_trash::{Retention, Trash};

use crate::action::{self, PowerAction, Runner, SystemRunner};
use crate::backup::{BackupError, BackupService, Summary};
use crate::facts::{read_facts, Facts};
use crate::install::{self, Installed};
use crate::proto::{AccessWord, Request, Response};
use crate::provider_contract::{AudioLevel, Provider, ToolProvider};
use crate::providers::{BtAction, SystemAction};
use crate::update::{HealthSignal, UpdateService};

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
pub const PROVIDER_GRANTS: [(CapabilityId, &str); 10] = [
    (CapabilityId::AudioSet, "audio://"),
    (CapabilityId::NetConfigure, "iface://"),
    (CapabilityId::NetConfigure, "wifi://"),
    (CapabilityId::BluetoothAdmin, "bt://"),
    (CapabilityId::AuthSession, "lock://"),
    (CapabilityId::ProcessSpawn, "app://"),
    (CapabilityId::UpdateControl, "update://stage"),
    (CapabilityId::UpdateControl, "update://commit"),
    (CapabilityId::UpdateControl, "update://rollback"),
    (CapabilityId::BackupControl, "backup://create"),
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
    /// The privileged account backend. Real helpers in production, an
    /// in-memory fake in the state tests.
    auth: Box<dyn crate::auth::AuthProvider>,
    /// The contract every audio, network, Bluetooth and authentication action
    /// goes through. Production runs the base tools; a test substitutes a fake.
    provider: Box<dyn Provider>,
    /// Failed verification attempts, in memory, so guessing a password over the
    /// socket is metered before a helper is spawned.
    attempts: crate::auth::Throttle,
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
    /// Where grants and the durable audit are written, when this build has a
    /// store. `None` means memory-only: the daemon's tests and `--report` must
    /// not touch `/var`.
    grants_store: Option<crate::grants::Store>,
    /// What happened when the store was opened, for `CAPS LIST`: `memory`,
    /// `loaded:<n>`, `seeded`, or `corrupt:<why>`. A corrupt record is a
    /// denial, and the user is entitled to see that it happened.
    grants_note: String,
    /// Durable lockout counters, beside the grant store.
    lockout_store: Option<crate::lockout::Store>,
    /// Whether the lockout file has been read. Reading waits for the first
    /// request so the constructor needs no clock.
    lockout_loaded: bool,
    /// Counters changed since the last write.
    lockout_dirty: bool,
    /// A lockout began or a success cleared one: write on this request rather
    /// than waiting for the interval.
    lockout_urgent: bool,
    /// When the last write was attempted, for the interval.
    lockout_written_at: Timestamp,
    /// How the lockout file was read, for diagnostics and tests.
    lockout_note: String,
    /// Generation state, boot counts and rollback. `disabled()` until
    /// the daemon is given the one root it is allowed to write.
    updates: UpdateService,
    /// The XDG data root below which applications are installed. It is
    /// configuration, not a request field: a request that could name a root
    /// could name `/`.
    applications_root: PathBuf,
    /// The local backup service. Disabled until the daemon is given the one
    /// directory it may copy into; a request cannot choose that directory.
    backup: BackupService,
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
            grants_store: None,
            grants_note: "memory".to_owned(),
            root: root.to_path_buf(),
            client,
            runner: Box::new(SystemRunner),
            provider: Box::new(ToolProvider),
            auth: Box::new(crate::auth::SystemAuthProvider::system()),
            attempts: crate::auth::Throttle::new(),
            lockout_store: None,
            lockout_loaded: false,
            lockout_dirty: false,
            lockout_urgent: false,
            lockout_written_at: 0,
            lockout_note: "disabled".to_owned(),
            updates: UpdateService::disabled(),
            applications_root: Path::new(home).join(".local/share"),
            backup: BackupService::disabled(),
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

    /// Replace the account backend, the same seam [`Sysd::with_runner`] gives
    /// the rest of the actions.
    #[must_use]
    pub fn with_auth(mut self, auth: Box<dyn crate::auth::AuthProvider>) -> Self {
        self.auth = auth;
        self
    }

    /// Replace the provider contract implementation.
    ///
    /// This is the C1 seam: a test can answer audio, network or Bluetooth from
    /// [`crate::provider_contract::FakeProvider`] without a runner, while
    /// production keeps [`ToolProvider`] and the same audited plans.
    #[must_use]
    pub fn with_provider(mut self, provider: Box<dyn Provider>) -> Self {
        self.provider = provider;
        self
    }

    /// Attach the one update root this daemon may read or write.
    ///
    /// The path is configuration, never a request field. Without it every
    /// update request is refused and status is `state=disabled`, which keeps
    /// `--report`, unit tests and a daemon without an image from touching
    /// `/var`.
    #[must_use]
    pub fn with_update_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.updates = UpdateService::new(root);
        self
    }

    /// Attach the XDG data root that application installs use.
    ///
    /// Production derives it from the user's home. A test supplies a temporary
    /// root, and nothing else can change it per request.
    #[must_use]
    pub fn with_application_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.applications_root = root.into();
        self
    }

    /// Attach the backup directory and enable the daily schedule.
    ///
    /// The path is configuration, never a request field. Without it `BACKUP`
    /// status is `disabled` and no scheduler is started, which keeps unit tests
    /// and `--report` from writing a backup into the development host's home.
    #[must_use]
    pub fn with_backup_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.backup = BackupService::new(root);
        self
    }

    /// Set the automatic backup interval. Zero disables the scheduler but
    /// leaves manual `BACKUP CREATE` available.
    #[must_use]
    pub fn with_backup_interval(mut self, secs: u64) -> Self {
        self.backup = self.backup.with_interval(secs);
        self
    }

    /// The configured interval, or zero when backups are disabled.
    #[must_use]
    pub fn backup_interval_secs(&self) -> u64 {
        if self.backup.root().is_none() {
            0
        } else {
            self.backup.interval_secs()
        }
    }

    /// Run the automatic backup if the interval says it is due.
    ///
    /// This is the only entrypoint the serve loop's timer uses. It performs the
    /// same snapshot as `BACKUP CREATE`; the scheduler is not a second
    /// implementation.
    pub fn backup_tick(&mut self, now: Timestamp) -> Option<Result<Summary, BackupError>> {
        self.backup.tick(&self.applications_root, now)
    }

    /// Let the runnning serve loop attest the candidate booted by `--boot`.
    ///
    /// This is deliberately best-effort at the caller: a corrupt or absent
    /// update root must not stop the daemon from serving the rest of the
    /// machine. A failure is logged by the server, and the commit gate stays
    /// closed until the evidence exists.
    ///
    /// # Errors
    /// A root lock or I/O failure, or a health record that cannot be written.
    pub fn attest_update_health(&self, now: Timestamp) -> Result<(), crate::update::UpdateFailure> {
        self.updates.attest_serving(now).map(|_| ())
    }

    /// Attach the durable grant record at `path`.
    ///
    /// Three outcomes, all deliberate:
    ///
    /// * the file exists and parses — its grants and audit replace the policy
    ///   set the constructor built, because the file is the record of what this
    ///   machine has already decided;
    /// * the file does not exist — the policy grants stay and are written
    ///   immediately, so the next start has something to prove survived;
    /// * the file exists and cannot be read — the broker is emptied, including
    ///   the policy grants, and [`Self::grants_note`] says why. A damaged
    ///   permission record must not be a permission.
    #[must_use]
    pub fn with_grant_store(mut self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        // A sibling file, not a record in the grant file: a damaged lockout
        // meter must not deny grants, and an older daemon must not reject the
        // grant file because it does not know this record type.
        let lockout_store = crate::lockout::Store::new(path.with_extension("lockout"));
        let store = crate::grants::Store::new(path);
        match store.load() {
            Ok(Some(loaded)) => {
                self.caps = Broker::new();
                let accepted = self.caps.restore_grants(loaded.grants);
                self.caps.restore_audit(loaded.audit);
                self.grants_note = format!("loaded:{accepted}");
                self.grants_store = Some(store);
            }
            Ok(None) => {
                self.grants_store = Some(store);
                self.grants_note = "seeded".to_owned();
                if let Err(error) = self.persist_grants() {
                    self.grants_note = format!("seed-failed:{error}");
                }
            }
            Err(error) => {
                self.caps = Broker::new();
                self.grants_note = format!("corrupt:{error}");
                self.grants_store = Some(store);
            }
        }
        self.lockout_store = Some(lockout_store);
        self.lockout_note = "not-loaded".to_owned();
        self
    }

    /// How the grant store was read, for the journal and `CAPS LIST`.
    #[must_use]
    pub fn grants_note(&self) -> &str {
        &self.grants_note
    }

    /// How many grant records this broker holds, revoked ones included.
    #[must_use]
    pub fn grant_count(&self) -> usize {
        self.caps.grant_count()
    }

    /// Write the current grants and audit now.
    ///
    /// Decisions are recorded in memory on every check and written here, at the
    /// moments a user or a client asks the daemon to account for them (and on
    /// every grant change). This is a checkpoint, not a write-ahead log: the
    /// decisions made since the last checkpoint are lost if the process is
    /// killed without one, which is stated in §63 rather than papered over.
    pub fn save_grants(&self) -> Result<(), String> {
        self.persist_grants()
    }

    /// Write the current broker state, atomically.
    ///
    /// A store that failed to parse is never overwritten: the damaged file is
    /// the evidence of what happened, and replacing it with the empty broker it
    /// produced would erase the only copy.
    fn persist_grants(&self) -> Result<(), String> {
        let Some(store) = &self.grants_store else {
            return Ok(());
        };
        if self.grants_note.starts_with("corrupt") {
            return Err("the grant record could not be read; refusing to overwrite it".to_owned());
        }
        let mut grants: Vec<crate::grants::Record> = self
            .caps
            .all_grants()
            .map(|(app, grant)| (app.clone(), grant.clone()))
            .collect();
        grants.sort_by(|a, b| {
            a.0.as_str()
                .cmp(b.0.as_str())
                .then_with(|| {
                    a.1.capability()
                        .to_string()
                        .cmp(&b.1.capability().to_string())
                })
                .then_with(|| a.1.issued_at().cmp(&b.1.issued_at()))
        });
        store.save(&grants, self.caps.audit_log())
    }

    /// Issue a grant and write it down.
    ///
    /// Returns the write error rather than swallowing it: a grant the user
    /// believes is durable but is not would come back as a permission that
    /// silently evaporated after a reboot, which is worse than a refusal.
    pub fn grant_and_save(&mut self, app: &AppId, grant: Grant) -> Result<(), String> {
        self.caps.grant(app, grant);
        self.persist_grants()
    }

    /// Revoke one capability and write the result down.
    ///
    /// The revocation is also recorded in the audit ring, as the decision it
    /// creates: after this call, a check for that capability is refused with
    /// `Reason::Revoked`. Live consent and revocation are different questions,
    /// and the log keeps them distinguishable.
    pub fn revoke_and_save(
        &mut self,
        app: &AppId,
        id: CapabilityId,
        now: Timestamp,
    ) -> Result<usize, String> {
        let affected: Vec<Capability> = self
            .caps
            .active_capabilities(app, now)
            .into_iter()
            .filter(|capability| capability.id() == id)
            .collect();
        let revoked = self.caps.revoke(app, id, now);
        for capability in affected {
            self.caps.record_event(litric_caps::AuditEntry {
                app: app.clone(),
                request: capability,
                allowed: false,
                reason: litric_caps::Reason::Revoked { at: now },
                at: now,
            });
        }
        self.persist_grants()?;
        Ok(revoked)
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
        self.ensure_lockout_loaded(now);
        let reply = match req {
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
            Request::CapsAudit => {
                let reply = Response::ok(self.caps_audit());
                let _ = self.persist_grants();
                reply
            }
            Request::CapsList => Response::ok(self.caps_list(now)),
            Request::Power { action, consent } => self.power(*action, *consent, now),
            Request::Action { action, consent } => self.action(action, *consent, now),
            Request::Resolve { app, uri, access } => self.resolve(app, uri, *access, now),
            Request::LaunchApp { argv } => self.launch_app(argv, &[]),
            Request::LaunchAppEnv { argv, env } => self.launch_app(argv, env),
            Request::AppInstall { id, bundle } => self.app_install(id, bundle),
            Request::AppUninstall { id } => self.app_uninstall(id),
            Request::UpdateStatus { schema } => self.update_status(*schema, now),
            Request::UpdateStage {
                version,
                source,
                consent,
            } => self.update_stage(version, source, *consent, now),
            Request::UpdateCommit { health, consent } => self.update_commit(*health, *consent, now),
            Request::UpdateRollback { consent } => self.update_rollback(*consent, now),
            Request::BackupStatus => Response::ok(self.backup.status(now)),
            Request::BackupList => self.backup_list(),
            Request::BackupCreate { consent } => self.backup_create(*consent, now),
            Request::BackupVerify { id } => self.backup_verify(id),
        };
        self.flush_lockouts(now, false);
        reply
    }

    /// Read the durable lockout counters once, at the first request.
    ///
    /// A file that cannot be read is logged and treated as empty. That is the
    /// opposite of the grant store's rule on purpose: refusing every password
    /// because a counter file is damaged would be a denial of service on the
    /// only account that can fix it.
    fn ensure_lockout_loaded(&mut self, now: Timestamp) {
        if self.lockout_loaded {
            return;
        }
        self.lockout_loaded = true;
        let Some(store) = self.lockout_store.clone() else {
            return;
        };
        match store.load() {
            Ok(None) => self.lockout_note = "seeded".to_owned(),
            Ok(Some(records)) => {
                let count = records.len();
                self.attempts = crate::auth::Throttle::restore(records, now);
                self.lockout_note = format!("loaded:{count}");
            }
            Err(error) => {
                self.attempts = crate::auth::Throttle::new();
                self.lockout_note = format!("corrupt:{error}");
                eprintln!("litric-sysd: lockout state ignored: {error}");
            }
        }
    }

    /// Write the counters when they changed and the interval allows it.
    ///
    /// A lockout starting or ending writes immediately; a run of failures
    /// writes at most once per [`crate::lockout::PERSIST_INTERVAL_SECS`], so the
    /// meter cannot be turned into a disk-write loop.
    fn flush_lockouts(&mut self, now: Timestamp, force: bool) {
        if !self.lockout_dirty {
            return;
        }
        let Some(store) = self.lockout_store.clone() else {
            self.lockout_dirty = false;
            return;
        };
        if !force
            && !self.lockout_urgent
            && now.saturating_sub(self.lockout_written_at) < crate::lockout::PERSIST_INTERVAL_SECS
        {
            return;
        }
        let records = self.attempts.export(now);
        match store.save(&records) {
            Ok(()) => {
                self.lockout_dirty = false;
                self.lockout_urgent = false;
            }
            Err(error) => {
                // Keep the change for the next interval, but stop retrying on
                // every request: a full disk must not become a log flood.
                self.lockout_urgent = false;
                eprintln!("litric-sysd: cannot write lockout state: {error}");
            }
        }
        self.lockout_written_at = now;
    }

    /// Install one validated bundle if the session client may manage apps.
    ///
    /// Reuses the local shell's `ProcessSpawn`/`app://` authority: installing
    /// is making a program the shell may start, and a separate capability would
    /// not narrow that authority while the shell can already name any absolute
    /// program. The installer, not the broker, is what enforces the bundle
    /// shape and the destination root.
    fn app_install(&mut self, id: &str, bundle: &str) -> Response {
        if let Err(refusal) = self.app_management_authorised() {
            return refusal;
        }
        match install::install(&self.applications_root, id, bundle) {
            Ok(Installed { id, .. }) => {
                Response::ok(vec![format!("id={id}"), "state=installed".to_owned()])
            }
            Err(error) => Response::err(error.kind(), error.detail()),
        }
    }

    /// Remove exactly what [`install::install`] wrote, if authorised.
    fn app_uninstall(&mut self, id: &str) -> Response {
        if let Err(refusal) = self.app_management_authorised() {
            return refusal;
        }
        match install::uninstall(&self.applications_root, id) {
            Ok(()) => Response::ok(vec![
                format!("id={}", id.to_ascii_lowercase()),
                "state=removed".to_owned(),
            ]),
            Err(error) => Response::err(error.kind(), error.detail()),
        }
    }

    fn app_management_authorised(&mut self) -> Result<(), Response> {
        let Some(client) = self.client.clone() else {
            return Err(Response::err(
                "refused",
                "this build has no local client identity",
            ));
        };
        let Ok(scope) = Scope::parse_resource("app://") else {
            return Err(Response::err("bad-argument", "no scope for this action"));
        };
        let Some(capability) = Capability::new(CapabilityId::ProcessSpawn, scope) else {
            return Err(Response::err(
                "bad-argument",
                "no capability for this action",
            ));
        };
        let decision = self.caps.check_with_consent(&client, &capability, false, 0);
        if decision.is_allowed() {
            Ok(())
        } else {
            Err(Response::err(
                "refused",
                format!("{} {capability}", decision.reason().token()),
            ))
        }
    }

    /// Start a program the user chose, if the session client may ask.
    ///
    /// The capability is checked here, with the same broker as every other
    /// action, before the vector reaches [`crate::launch`]. The daemon's own
    /// root is deliberately not consulted: a program is named by its own
    /// absolute path, and a request that could choose a root could choose which
    /// binary the path meant.
    fn launch_app(&mut self, argv: &[String], env: &[(String, String)]) -> Response {
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
        match crate::launch::launch_with_env(argv, env) {
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

    /// Authorise one update operation through the same broker as everything
    /// else. The scope names exactly one operation, so a grant to stage a
    /// candidate cannot commit it.
    fn update_authorised(
        &mut self,
        operation: &str,
        consent: bool,
        now: Timestamp,
    ) -> Option<Response> {
        let Some(client) = self.client.clone() else {
            return Some(Response::err(
                "refused",
                "this build has no local client identity",
            ));
        };
        let Ok(scope) = Scope::parse_resource(&format!("update://{operation}")) else {
            return Some(Response::err(
                "bad-argument",
                "no scope for this update operation",
            ));
        };
        let Some(capability) = Capability::new(CapabilityId::UpdateControl, scope) else {
            return Some(Response::err(
                "bad-argument",
                "no capability for this update operation",
            ));
        };
        let decision = self
            .caps
            .check_with_consent(&client, &capability, consent, now);
        if !decision.is_allowed() {
            return Some(Response::err(
                "refused",
                format!("{} {capability}", decision.reason().token()),
            ));
        }
        None
    }

    /// Answer a status question, refusing a schema this daemon cannot speak.
    ///
    /// Schema 0 has never existed and a schema above [`crate::update::SCHEMA`]
    /// asks for fields this build does not have; both are typed refusals rather
    /// than a truncated answer a newer client could mistake for schema 3. A
    /// schema-1 or schema-2 client is served the schema-3 answer with the
    /// health fields omitted: the remaining lines are additive and a client
    /// that ignores unknown fields still works.
    fn update_status(&self, requested: Option<u8>, now: Timestamp) -> Response {
        if let Some(schema) = requested {
            if schema == 0 || schema > crate::update::SCHEMA {
                return Response::err(
                    "schema-unsupported",
                    format!(
                        "requested schema {schema}; this daemon speaks 1..={}",
                        crate::update::SCHEMA
                    ),
                );
            }
        }
        Response::ok(self.updates.status(now, requested))
    }

    fn update_stage(
        &mut self,
        version: &Version,
        source: &str,
        consent: bool,
        now: Timestamp,
    ) -> Response {
        if let Some(refused) = self.update_authorised("stage", consent, now) {
            return refused;
        }
        match self.updates.stage(version.clone(), source) {
            Ok(lines) => Response::ok(lines),
            Err(error) => Response::err(error.kind(), error.to_string()),
        }
    }

    fn update_commit(&mut self, health: HealthSignal, consent: bool, now: Timestamp) -> Response {
        if let Some(refused) = self.update_authorised("commit", consent, now) {
            return refused;
        }
        match self.updates.commit(health, now) {
            Ok(lines) => Response::ok(lines),
            Err(error) => Response::err(error.kind(), error.to_string()),
        }
    }

    fn update_rollback(&mut self, consent: bool, now: Timestamp) -> Response {
        if let Some(refused) = self.update_authorised("rollback", consent, now) {
            return refused;
        }
        match self
            .updates
            .rollback(litric_update::RollbackReason::Manual, now)
        {
            Ok(lines) => Response::ok(lines),
            Err(error) => Response::err(error.kind(), error.to_string()),
        }
    }

    /// Authorise one manual backup. The automatic scheduler does not call the
    /// broker: it is the daemon acting on configuration, not a client request.
    fn backup_authorised(&mut self, consent: bool, now: Timestamp) -> Option<Response> {
        let Some(client) = self.client.clone() else {
            return Some(Response::err(
                "refused",
                "this build has no local client identity",
            ));
        };
        let Ok(scope) = Scope::parse_resource("backup://create") else {
            return Some(Response::err("bad-argument", "no scope for this action"));
        };
        let Some(capability) = Capability::new(CapabilityId::BackupControl, scope) else {
            return Some(Response::err(
                "bad-argument",
                "no capability for this action",
            ));
        };
        let decision = self
            .caps
            .check_with_consent(&client, &capability, consent, now);
        if !decision.is_allowed() {
            return Some(Response::err(
                "refused",
                format!("{} {capability}", decision.reason().token()),
            ));
        }
        None
    }

    fn backup_lines(summary: &Summary, state: &str) -> Vec<String> {
        vec![
            format!("id={}", summary.id),
            format!("created={}", summary.created),
            format!("entries={}", summary.entries),
            format!("bytes={}", summary.bytes),
            format!("state={state}"),
        ]
    }

    fn backup_create(&mut self, consent: bool, now: Timestamp) -> Response {
        if let Some(refused) = self.backup_authorised(consent, now) {
            return refused;
        }
        match self.backup.create(&self.applications_root, now) {
            Ok(summary) => Response::ok(Self::backup_lines(&summary, "created")),
            Err(error) => Response::err(error.kind(), error.detail()),
        }
    }

    fn backup_list(&self) -> Response {
        match self.backup.list() {
            Ok(summaries) => Response::ok(
                summaries
                    .iter()
                    .map(|summary| {
                        format!(
                            "id={} created={} entries={} bytes={}",
                            summary.id, summary.created, summary.entries, summary.bytes
                        )
                    })
                    .collect(),
            ),
            Err(error) => Response::err(error.kind(), error.detail()),
        }
    }

    fn backup_verify(&self, id: &str) -> Response {
        match self.backup.verify(id) {
            Ok(summary) => Response::ok(Self::backup_lines(&summary, "verified")),
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
        match self.provider.audio_level(self.runner.as_ref()) {
            Ok(AudioLevel { volume, muted }) => {
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
        match self.provider.bt_powered(self.runner.as_ref()) {
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
        // A verification is the one action whose helper is meant to be guessed
        // against. It is metered before the helper is spawned, so a locked
        // account costs no process, and a refused attempt cannot be used to
        // tell a wrong password from a missing account by timing the helper.
        let verify_user = match action {
            SystemAction::Auth(crate::providers::AuthAction::Verify { user, .. }) => {
                Some(user.as_str())
            }
            _ => None,
        };
        if let Some(user) = verify_user {
            if let Some(wait) = self.attempts.retry_after(user, now) {
                self.record_lockout(user, now);
                return Response::err(
                    "locked-out",
                    format!("too many failed attempts; retry_after={wait}"),
                );
            }
        }
        // Creation and change are two ordered helper invocations, so they go
        // through the account backend rather than the single-plan path. A
        // verification keeps its existing plan path byte for byte.
        if let SystemAction::Auth(auth) = action {
            if !matches!(auth, crate::providers::AuthAction::Verify { .. }) {
                return self.account(auth);
            }
        }
        // The root is the daemon's own, never a request's: the same rule as the
        // facts reader. A caller that could choose the root could choose which
        // credential file `iwd` is told about.
        let outcome = match action {
            SystemAction::Audio(a) => self.provider.audio(a, self.runner.as_ref(), &self.root),
            SystemAction::Net(n) => self.provider.network(n, self.runner.as_ref(), &self.root),
            SystemAction::Bluetooth(b) => {
                self.provider.bluetooth(b, self.runner.as_ref(), &self.root)
            }
            SystemAction::Auth(a) => self.provider.auth(a, self.runner.as_ref(), &self.root),
        };
        // Only a helper that ran and rejected the password counts as a guess.
        // A missing tool, a timeout or a refusal is the daemon failing, and
        // counting those would lock an account out for the machine's fault.
        if let Some(user) = verify_user {
            if outcome.applied {
                self.attempts.record_success(user);
                self.lockout_dirty = true;
                // A cleared meter is worth writing at once: the next restart
                // must not bring back a lockout the user already answered.
                self.lockout_urgent = true;
            } else if outcome.lines.iter().any(|line| line == "reason=exit") {
                self.attempts.record_failure(user, now);
                self.lockout_dirty = true;
                if self.attempts.retry_after(user, now).is_some() {
                    // The threshold is the meaningful change; write it now.
                    self.lockout_urgent = true;
                }
            }
        }
        if outcome.applied {
            // The level that was just set is read back rather than assumed: the
            // tool accepting a number is not the same as the sink taking it, and
            // a panel showing the number somebody typed is not showing the
            // machine. A failed readback leaves the previous measurement in
            // place instead of replacing it with a guess.
            if matches!(action, SystemAction::Audio(_)) {
                if let Ok(AudioLevel { volume, muted }) =
                    self.provider.audio_level(self.runner.as_ref())
                {
                    self.audio = Some((volume, muted));
                }
            }
            // A radio that accepted the request is not the same as a radio that
            // is on: `bluetoothctl` returns when BlueZ has taken the property,
            // and the property is what the panel must show. Only a power change
            // is re-read; a scan or a pairing says nothing about power.
            if matches!(action, SystemAction::Bluetooth(BtAction::Power(_))) {
                if let Ok(powered) = self.provider.bt_powered(self.runner.as_ref()) {
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

    /// Put a refused verification on the audit record.
    ///
    /// The broker has already recorded the authorisation decision as allowed;
    /// this is the separate fact that the meter refused the attempt anyway. A
    /// refusal is bounded by the audit ring like every other check, and it is
    /// not written to disk here: the next `CAPS AUDIT` or store save does that,
    /// so a caller cannot turn a locked retry loop into a disk-write loop.
    fn record_lockout(&mut self, user: &str, now: Timestamp) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Ok(scope) = Scope::parse_resource(&format!("lock://{user}")) else {
            return;
        };
        let Some(capability) = Capability::new(CapabilityId::AuthSession, scope) else {
            return;
        };
        self.caps.record_event(litric_caps::AuditEntry {
            app: client,
            request: capability,
            allowed: false,
            reason: litric_caps::Reason::RateLimited,
            at: now,
        });
    }

    /// Validate and execute one account creation or change.
    ///
    /// The rule set is [`crate::auth::validate_password`], the same one the
    /// shell calls before sending: the daemon revalidates because the socket is
    /// not a trust boundary. A refusal is answered with its stable token and a
    /// fixed sentence, never with the request's own text.
    fn account(&mut self, action: &crate::providers::AuthAction) -> Response {
        use crate::providers::AuthAction;
        let result = match action {
            AuthAction::Create {
                user,
                password,
                confirm,
            } => match crate::auth::validate_password(user, password, confirm) {
                Ok(()) => self.auth.create(user, password),
                Err(error) => Err(error),
            },
            AuthAction::Change {
                user,
                current,
                new_password,
                confirm,
            } => match crate::auth::validate_password(user, new_password, confirm) {
                Ok(()) => self.auth.change(user, current, new_password),
                Err(error) => Err(error),
            },
            AuthAction::Verify { .. } => {
                return Response::err("internal-error", "verification is not a two-step action")
            }
        };
        match result {
            Ok(()) => Response::ok(vec![
                format!("auth={}", action.token()),
                format!("user={}", action.user()),
            ]),
            Err(error) => Response::err(error.token(), error.detail().to_owned()),
        }
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

    fn caps_list(&self, now: Timestamp) -> Vec<String> {
        let mut out = vec![
            format!("store={}", self.grants_note),
            format!("lockout={}", self.lockout_note),
            format!("grant_records={}", self.caps.grant_count()),
            format!("audit_records={}", self.caps.audit_log().len()),
        ];
        if let Some(client) = &self.client {
            out.push(format!("client={}", client.as_str()));
            let mut active = self.caps.active_capabilities(client, now);
            active.sort_by_cached_key(std::string::ToString::to_string);
            for capability in active {
                out.push(format!("capability={capability}"));
            }
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

    // ---- durable grants (§63) ----------------------------------------------

    fn grants_fixture(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("litric-sysd-grants-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root.join("var/lib/litric-sysd/grants.conf")
    }

    fn grants_cleanup(path: &Path) {
        if let Some(root) = path.ancestors().nth(4) {
            let _ = std::fs::remove_dir_all(root);
        }
    }

    fn camera() -> Capability {
        Capability::unscoped(CapabilityId::Camera).expect("valid")
    }

    #[test]
    fn a_grant_and_a_denial_survive_a_restart() {
        // The property the batch exists for. Dropping the `Sysd` and building a
        // new one from the same file is exactly what a daemon restart does to
        // this state; nothing is kept in a static.
        let path = grants_fixture("restart");
        let app = AppId::parse("com.example.photos").expect("app");
        {
            let mut first = Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root"))
                .with_grant_store(&path);
            assert!(
                first.grants_note().starts_with("seeded"),
                "first start seeds the record: {}",
                first.grants_note()
            );
            first
                .grant_and_save(&app, Grant::new(camera(), Issuer::User, 100))
                .expect("the grant is written");
            let mic = Capability::unscoped(CapabilityId::Microphone).expect("mic");
            assert!(!first.caps.check(&app, &mic, 101).is_allowed());
            first.save_grants().expect("the audit is checkpointed");
        }

        let second =
            Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root")).with_grant_store(&path);
        assert!(
            second.grants_note().starts_with("loaded:"),
            "second start loads the record: {}",
            second.grants_note()
        );
        assert!(
            second
                .caps
                .active_capabilities(&app, 150)
                .iter()
                .any(|c| c.id() == CapabilityId::Camera),
            "the camera grant must still be active after the restart"
        );
        let history = second.caps.audit_for(&app);
        assert_eq!(history.len(), 1, "the denial is part of the record");
        assert!(history.first().is_some_and(|e| !e.allowed));
        grants_cleanup(&path);
    }

    #[test]
    fn a_corrupt_record_grants_nothing_including_the_policy_set() {
        // A damaged file must be a denial, never a subset nobody audited.
        let path = grants_fixture("corrupt");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(
            &path,
            "litric-grants\t1\ngrant\tcom.example.app\tcamera\tuser\t1\n",
        )
        .expect("write");

        let mut state =
            Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root")).with_grant_store(&path);
        assert!(
            state.grants_note().starts_with("corrupt:"),
            "the refusal must be visible: {}",
            state.grants_note()
        );
        let client = state.client().expect("client").clone();
        assert!(
            state.caps.active_capabilities(&client, 0).is_empty(),
            "not even the policy grants may survive a corrupt record"
        );
        assert_eq!(
            state.caps.check(&client, &camera(), 0).reason(),
            &litric_caps::Reason::UnknownApplication,
            "an empty broker denies by default"
        );
        grants_cleanup(&path);
    }

    #[test]
    fn a_corrupt_record_is_not_overwritten_by_a_new_grant() {
        let path = grants_fixture("preserve");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        let damaged = "this is not a grant record\n";
        std::fs::write(&path, damaged).expect("write");
        let mut state =
            Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root")).with_grant_store(&path);
        let app = AppId::parse("com.example.app").expect("app");
        let err = state
            .grant_and_save(&app, Grant::new(camera(), Issuer::User, 1))
            .expect_err("must refuse to write over the evidence");
        assert!(err.contains("refusing to overwrite"), "{err}");
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            damaged,
            "the damaged file is evidence and must be left alone"
        );
        grants_cleanup(&path);
    }

    #[test]
    fn a_revocation_survives_a_restart_and_is_audited() {
        let path = grants_fixture("revocation");
        let app = AppId::parse("com.example.recorder").expect("app");
        {
            let mut first = Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root"))
                .with_grant_store(&path);
            first
                .grant_and_save(&app, Grant::new(camera(), Issuer::User, 100))
                .expect("granted");
            let n = first
                .revoke_and_save(&app, CapabilityId::Camera, 200)
                .expect("revoked and written");
            assert_eq!(n, 1);
        }

        let second =
            Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root")).with_grant_store(&path);
        assert!(
            second.caps.active_capabilities(&app, 250).is_empty(),
            "a revoked grant must not come back active"
        );
        let held = second.caps.grants_for(&app);
        assert_eq!(held.len(), 1, "history is kept, not deleted");
        assert_eq!(held.first().and_then(Grant::revoked_at), Some(200));
        let history = second.caps.audit_for(&app);
        assert!(
            history
                .iter()
                .any(|e| e.reason.token() == "revoked" && e.at == 200),
            "the revocation is in the persisted audit: {history:?}"
        );
        grants_cleanup(&path);
    }

    #[test]
    fn an_expiring_grant_still_expires_after_a_restart() {
        let path = grants_fixture("expiry");
        let app = AppId::parse("com.example.photos").expect("app");
        {
            let mut first = Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root"))
                .with_grant_store(&path);
            first
                .grant_and_save(&app, Grant::expiring(camera(), Issuer::User, 100, 200))
                .expect("granted");
        }
        let second =
            Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root")).with_grant_store(&path);
        assert!(second
            .caps
            .active_capabilities(&app, 199)
            .iter()
            .any(|c| c.id() == CapabilityId::Camera));
        assert!(
            second.caps.active_capabilities(&app, 200).is_empty(),
            "the deadline must survive the file as a deadline, not be forgotten"
        );
        grants_cleanup(&path);
    }

    #[test]
    fn caps_list_reports_the_store_and_the_grants() {
        let path = grants_fixture("list");
        let mut state =
            Sysd::new(4, 1024, "/home/alice", Path::new("no-such-root")).with_grant_store(&path);
        let reply = state.handle(&crate::proto::Request::CapsList, NOW);
        assert!(reply.is_ok(), "{reply:?}");
        assert!(line(&reply, "store=seeded").is_some());
        assert!(
            reply
                .lines()
                .iter()
                .any(|l| l.starts_with("capability=power.control")),
            "the policy grants are listed: {:?}",
            reply.lines()
        );
        grants_cleanup(&path);
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
    fn a_fake_provider_serves_the_contract_without_a_tool() {
        use crate::provider_contract::FakeProvider;
        use crate::providers::NetAction;

        let fake = FakeProvider::new()
            .with_audio_level(37, true)
            .with_bt_powered(true);
        let mut s = sysd().with_provider(Box::new(fake.clone()));

        let audio = s.handle(&Request::AudioLevel, NOW);
        assert!(audio.is_ok(), "{}", audio.encode());
        assert_eq!(
            line(&audio, "audio.level"),
            Some(&"audio.level volume=37 muted=1".to_owned())
        );
        let bluetooth = s.handle(&Request::BtStatus, NOW);
        assert!(bluetooth.is_ok(), "{}", bluetooth.encode());
        assert_eq!(
            line(&bluetooth, "bt.powered"),
            Some(&"bt.powered=1".to_owned())
        );

        let network = s.handle(
            &Request::Action {
                action: SystemAction::Net(NetAction::Reconfigure("eth0".to_owned())),
                consent: false,
            },
            NOW,
        );
        assert!(network.is_ok(), "{}", network.encode());
        assert!(
            fake.calls()
                .iter()
                .any(|call| call.contains("Reconfigure(\"eth0\")")),
            "the fake did not receive the network action: {:?}",
            fake.calls()
        );
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

    fn as_hex(text: &str) -> String {
        text.bytes().map(|b| format!("{b:02x}")).collect()
    }

    #[derive(Debug, Default)]
    struct RejectingRunner {
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl crate::action::Runner for RejectingRunner {
        fn run(
            &self,
            _program: &str,
            _args: &[&str],
            _stdin: Option<&[u8]>,
            _timeout_secs: u32,
        ) -> std::io::Result<i32> {
            self.calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(1)
        }

        fn capture(
            &self,
            _program: &str,
            _args: &[&str],
            _timeout_secs: u32,
        ) -> std::io::Result<(i32, String)> {
            Err(std::io::Error::other(
                "this test runner has no captured answer",
            ))
        }
    }

    fn lockout_dir(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("litric-sysd-lockout-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn verify_line(attempt: u32) -> String {
        format!(
            "AUTH VERIFY alice {} CONFIRM",
            as_hex(&format!("wrong password {attempt}"))
        )
    }

    #[test]
    fn a_lockout_survives_a_restart() {
        let root = lockout_dir("restart");
        std::fs::create_dir_all(&root).expect("test directory");
        let path = root.join("grants.conf");

        let first_runner = RejectingRunner::default();
        let first_calls = std::sync::Arc::clone(&first_runner.calls);
        let mut first = sysd()
            .with_runner(Box::new(first_runner))
            .with_grant_store(&path);
        for attempt in 0..crate::auth::VERIFY_FAILURE_LIMIT {
            let reply = first.handle(&Request::parse(&verify_line(attempt)).expect("verify"), NOW);
            assert!(
                reply.encode().starts_with("ERR action-failed"),
                "{}",
                reply.encode()
            );
        }
        assert_eq!(first_calls.load(std::sync::atomic::Ordering::Relaxed), 5);
        first.flush_lockouts(NOW, true);
        assert!(
            path.with_extension("lockout").is_file(),
            "the meter was written beside the grant store"
        );
        drop(first);

        let second_runner = RejectingRunner::default();
        let second_calls = std::sync::Arc::clone(&second_runner.calls);
        let mut second = sysd()
            .with_runner(Box::new(second_runner))
            .with_grant_store(&path);
        let reply = second.handle(&Request::parse(&verify_line(99)).expect("verify"), NOW + 1);
        assert!(
            reply.encode().starts_with("ERR locked-out"),
            "{}",
            reply.encode()
        );
        assert!(
            reply.encode().contains("retry_after=59"),
            "{}",
            reply.encode()
        );
        assert_eq!(
            second_calls.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "a restart must not reset the meter"
        );
        assert_eq!(second.lockout_note, "loaded:2");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn an_expired_lockout_is_dropped_when_it_is_loaded() {
        let root = lockout_dir("expired");
        std::fs::create_dir_all(&root).expect("test directory");
        let path = root.join("grants.conf");
        let records = vec![crate::auth::LockoutRecord {
            user: Some("alice".to_owned()),
            count: 5,
            window_start: NOW - 3_600,
            locked_until: 0,
        }];
        crate::lockout::Store::new(path.with_extension("lockout"))
            .save(&records)
            .expect("writes the stale record");

        let runner = RejectingRunner::default();
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut state = sysd().with_runner(Box::new(runner)).with_grant_store(&path);
        let reply = state.handle(&Request::parse(&verify_line(1)).expect("verify"), NOW);
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            1,
            "an expired entry does not block the helper"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_damaged_lockout_file_fails_open_and_leaves_the_daemon_alive() {
        let root = lockout_dir("damaged");
        std::fs::create_dir_all(&root).expect("test directory");
        let path = root.join("grants.conf");
        std::fs::write(path.with_extension("lockout"), b"not a lockout record\n")
            .expect("writes the damaged file");

        let runner = RejectingRunner::default();
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut state = sysd().with_runner(Box::new(runner)).with_grant_store(&path);
        assert!(state.handle(&Request::Ping, NOW).is_ok());
        let reply = state.handle(&Request::parse(&verify_line(1)).expect("verify"), NOW);
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert!(
            state.lockout_note.starts_with("corrupt:"),
            "{}",
            state.lockout_note
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_forward_clock_jump_drops_a_loaded_lockout() {
        let root = lockout_dir("clock-forward");
        std::fs::create_dir_all(&root).expect("test directory");
        let path = root.join("grants.conf");
        let records = vec![crate::auth::LockoutRecord {
            user: Some("alice".to_owned()),
            count: crate::auth::VERIFY_FAILURE_LIMIT,
            window_start: NOW,
            locked_until: NOW + crate::auth::VERIFY_LOCKOUT_SECS,
        }];
        crate::lockout::Store::new(path.with_extension("lockout"))
            .save(&records)
            .expect("writes the live record");
        drop(records);

        // The clock is an hour ahead by the time the daemon loads the record:
        // both the lockout deadline and its failure window are in the past.
        let later = NOW + 3_600;
        let runner = RejectingRunner::default();
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut state = sysd().with_runner(Box::new(runner)).with_grant_store(&path);
        let reply = state.handle(&Request::parse(&verify_line(1)).expect("verify"), later);
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            1,
            "a lockout expired by a forward jump must not reach the helper"
        );
        assert_eq!(state.lockout_note, "loaded:1");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_backward_clock_jump_caps_a_loaded_lockout_at_one_minute() {
        let root = lockout_dir("clock-backward");
        std::fs::create_dir_all(&root).expect("test directory");
        let path = root.join("grants.conf");
        let records = vec![crate::auth::LockoutRecord {
            user: Some("alice".to_owned()),
            count: crate::auth::VERIFY_FAILURE_LIMIT,
            window_start: NOW,
            locked_until: NOW + crate::auth::VERIFY_LOCKOUT_SECS,
        }];
        crate::lockout::Store::new(path.with_extension("lockout"))
            .save(&records)
            .expect("writes the live record");
        drop(records);

        // The clock is five minutes behind. The stored deadline appears 360 s
        // away, but loading may only re-arm one configured lockout.
        let loaded_at = NOW - 300;
        let runner = RejectingRunner::default();
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut state = sysd().with_runner(Box::new(runner)).with_grant_store(&path);
        let reply = state.handle(&Request::parse(&verify_line(1)).expect("verify"), loaded_at);
        assert!(
            reply.encode().starts_with("ERR locked-out"),
            "{}",
            reply.encode()
        );
        assert!(
            reply.encode().contains("retry_after=60"),
            "the skew must not become the wait: {}",
            reply.encode()
        );
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "the capped lockout still blocks the helper"
        );

        // Exactly one lockout after load the cap ends, despite the stored
        // wall-clock deadline being 300 s later.
        let reply = state.handle(
            &Request::parse(&verify_line(2)).expect("verify"),
            loaded_at + crate::auth::VERIFY_LOCKOUT_SECS,
        );
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            1,
            "the backward jump must not extend the lock past one lockout from load"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_verifications_are_locked_out_before_the_helper_runs() {
        let runner = RejectingRunner::default();
        let calls = std::sync::Arc::clone(&runner.calls);
        let mut s = sysd().with_runner(Box::new(runner));
        let line = |attempt: u32| {
            format!(
                "AUTH VERIFY alice {} CONFIRM",
                as_hex(&format!("wrong password {attempt}"))
            )
        };

        for attempt in 0..crate::auth::VERIFY_FAILURE_LIMIT {
            let reply = s.handle(&Request::parse(&line(attempt)).expect("verify"), NOW);
            assert!(
                reply.encode().starts_with("ERR action-failed"),
                "{}",
                reply.encode()
            );
        }
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 5);

        let reply = s.handle(&Request::parse(&line(99)).expect("verify"), NOW);
        assert!(
            reply.encode().starts_with("ERR locked-out"),
            "{}",
            reply.encode()
        );
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            5,
            "a locked account must not reach unix_chkpwd"
        );
        assert!(
            reply.encode().contains("retry_after="),
            "the client is told how long to wait: {}",
            reply.encode()
        );
        assert!(
            s.caps.audit_log().iter().any(|entry| !entry.allowed
                && entry.reason.token() == "rate-limited"
                && entry.request.to_string().ends_with("lock://alice")),
            "the lockout is on the audit record"
        );

        // The window ends with the lockout: the next attempt reaches the
        // helper again, so a legitimate user is not locked out forever.
        let reply = s.handle(
            &Request::parse(&line(100)).expect("verify"),
            NOW + crate::auth::VERIFY_LOCKOUT_SECS,
        );
        assert!(
            reply.encode().starts_with("ERR action-failed"),
            "{}",
            reply.encode()
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 6);
    }

    #[test]
    fn account_creation_and_change_cross_the_protocol_with_shared_validation() {
        let strong = "correct horse battery";
        let other = "another horse battery";
        let mut s = sysd().with_auth(Box::new(crate::auth::FakeAuth::new()));

        let line = format!(
            "AUTH CREATE alice {} {} CONFIRM",
            as_hex(strong),
            as_hex(strong)
        );
        let reply = s.handle(&Request::parse(&line).expect("create"), NOW);
        assert!(reply.is_ok(), "{}", reply.encode());

        // The short spec form defaults the confirmation to the password, so it
        // passes the same validator rather than skipping it.
        let short = format!("AUTH CREATE dave {} CONFIRM", as_hex(strong));
        let reply = s.handle(&Request::parse(&short).expect("short create"), NOW);
        assert!(reply.is_ok(), "{}", reply.encode());
        let short_change = format!(
            "AUTH CHANGE dave {} {} CONFIRM",
            as_hex(strong),
            as_hex(other)
        );
        let reply = s.handle(&Request::parse(&short_change).expect("short change"), NOW);
        assert!(reply.is_ok(), "{}", reply.encode());

        // The shared validator runs on the daemon side: a short password, a
        // mismatched confirmation, and a password equal to the name are refused
        // before the backend is touched. No reply repeats the payload.
        for (line, token) in [
            (
                format!(
                    "AUTH CREATE bob {} {} CONFIRM",
                    as_hex("short"),
                    as_hex("short")
                ),
                "weak-password",
            ),
            (
                format!(
                    "AUTH CREATE bob {} {} CONFIRM",
                    as_hex(strong),
                    as_hex(other)
                ),
                "password-mismatch",
            ),
            (
                format!(
                    "AUTH CREATE carol {} {} CONFIRM",
                    as_hex("carol"),
                    as_hex("carol")
                ),
                "weak-password",
            ),
        ] {
            let reply = s.handle(&Request::parse(&line).expect("valid line"), NOW);
            let encoded = reply.encode();
            assert!(encoded.starts_with(&format!("ERR {token}")), "{encoded}");
            assert!(!encoded.contains(&as_hex(strong)), "{encoded}");
        }

        // A wrong current password is answered about the attempt and writes
        // nothing; the right one goes through and the new password verifies.
        let wrong = format!(
            "AUTH CHANGE alice {} {} {} CONFIRM",
            as_hex("wrong password"),
            as_hex(other),
            as_hex(other)
        );
        let reply = s.handle(&Request::parse(&wrong).expect("change"), NOW);
        assert!(
            reply.encode().starts_with("ERR verify-failed"),
            "{}",
            reply.encode()
        );

        let right = format!(
            "AUTH CHANGE alice {} {} {} CONFIRM",
            as_hex(strong),
            as_hex(other),
            as_hex(other)
        );
        let reply = s.handle(&Request::parse(&right).expect("change"), NOW);
        assert!(reply.is_ok(), "{}", reply.encode());

        // The new password is now the current one: a second change that uses it
        // as the current password succeeds, which the fake can only answer if
        // the first write landed.
        let again = format!(
            "AUTH CHANGE alice {} {} {} CONFIRM",
            as_hex(other),
            as_hex(strong),
            as_hex(strong)
        );
        let reply = s.handle(&Request::parse(&again).expect("change"), NOW);
        assert!(reply.is_ok(), "{}", reply.encode());
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
    fn update_actions_need_consent_and_a_scope_that_survives_revocation() {
        let root =
            std::env::temp_dir().join(format!("litric-sysd-update-state-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let source = root.join("incoming/release");
        std::fs::create_dir_all(&source).expect("incoming");
        std::fs::write(source.join("app"), "candidate").expect("candidate");
        litric_update::Engine::init(&root, Version::parse("1.0.0").expect("version"))
            .expect("generation state");

        let mut s = sysd().with_update_root(&root);
        let version = Version::parse("1.1.0").expect("version");
        let unconfirmed = s.handle(
            &Request::UpdateStage {
                version: version.clone(),
                source: "release".to_owned(),
                consent: false,
            },
            NOW,
        );
        assert!(!unconfirmed.is_ok(), "{}", unconfirmed.encode());
        assert!(
            unconfirmed.encode().contains("needs-live-consent"),
            "{}",
            unconfirmed.encode()
        );

        let staged = s.handle(
            &Request::UpdateStage {
                version,
                source: "release".to_owned(),
                consent: true,
            },
            NOW + 1,
        );
        assert!(staged.is_ok(), "{}", staged.encode());
        assert!(line(&staged, "slot=B").is_some(), "{:?}", staged.lines());

        let client = s.client().expect("client").clone();
        s.caps_mut()
            .revoke(&client, CapabilityId::UpdateControl, NOW + 2);
        let denied = s.handle(
            &Request::UpdateCommit {
                health: HealthSignal::Passed,
                consent: true,
            },
            NOW + 3,
        );
        assert!(!denied.is_ok(), "{}", denied.encode());
        assert!(
            denied.encode().contains("revoked") || denied.encode().contains("no-grant"),
            "{}",
            denied.encode()
        );

        let status = s.handle(&Request::UpdateStatus { schema: Some(2) }, NOW + 4);
        assert!(status.is_ok(), "{}", status.encode());
        assert!(
            line(&status, "active_slot=A").is_some(),
            "a refused commit must not activate the candidate: {:?}",
            status.lines()
        );
        let _ = std::fs::remove_dir_all(root);
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
