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

use litric_caps::{Broker, Capability};
use litric_core::AppId;
use litric_env::{EnvironmentId, EnvironmentManager, SwitchOutcome};
use litric_storage::{Access, Layout, StorageBroker, StorageUri};
use litric_trash::{Retention, Trash};

use crate::facts::{read_facts, Facts};
use crate::proto::{AccessWord, Request, Response};

/// Seconds since the Unix epoch.
pub type Timestamp = u64;

/// The whole daemon, minus its transport.
#[derive(Debug)]
pub struct Sysd {
    env: EnvironmentManager,
    trash: Trash,
    caps: Broker,
    storage: StorageBroker,
    /// Where `/proc` and `/sys` are rooted. `/` in production.
    root: PathBuf,
}

impl Sysd {
    /// Build the daemon's state for a machine of this size.
    ///
    /// `home` is the user's home directory, which the storage layout is derived
    /// from.
    #[must_use]
    pub fn new(total_cpus: u32, total_memory_mb: u32, home: &str, root: &Path) -> Self {
        Self {
            env: EnvironmentManager::new(total_cpus, total_memory_mb),
            trash: Trash::new(Retention::default()),
            caps: Broker::new(),
            storage: StorageBroker::new(Layout::for_home(home)),
            root: root.to_path_buf(),
        }
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
            Request::Facts => Response::ok(self.facts().to_lines()),
            Request::EnvList => Response::ok(self.env_list()),
            Request::EnvPlan(token) => self.env_plan(token),
            Request::EnvSwitch(token) => self.env_switch(token),
            Request::TrashList => Response::ok(self.trash_list()),
            Request::TrashPlanEmpty => Response::ok(self.trash_plan_empty(now)),
            Request::CapsAudit => Response::ok(self.caps_audit()),
            Request::Resolve { app, uri, access } => self.resolve(app, uri, *access, now),
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
