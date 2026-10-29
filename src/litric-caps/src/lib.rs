//! The LitricOS capability engine.
//!
//! # Layer position
//!
//! **Layer 2 / 3 (Core Services / System API).** This crate is the decision
//! logic behind `litric-capd`, the single authority for every permission
//! decision on the system. It depends downward on `litric-core` and on nothing
//! else, and it performs no I/O — which is what makes it testable on any host
//! and auditable by reading it.
//!
//! # The rules it enforces
//!
//! 1. **Deny by default.** An application with no grants can do nothing. There
//!    is no ambient authority anywhere in this crate: every `Allow` traces to a
//!    specific grant issued to a specific application.
//! 2. **Least privilege by scope.** A grant names a resource, not a class.
//! 3. **Revocation is immediate.** A revoked grant stops authorising on the very
//!    next decision, with no restart and no cache to go stale.
//! 4. **Expiry is enforced.** A time-boxed grant stops working when it expires.
//! 5. **Every decision is recorded.** Allow and deny alike. A permission system
//!    the user cannot audit is a permission system they have to take on faith.
//! 6. **No confused deputy.** Authority is looked up from the *caller's*
//!    identity. A caller never presents a capability and is never believed about
//!    who it is; see `check` and RUNTIME section 37.
//!
//! # Why no clock
//!
//! Time is injected rather than read. `SystemTime::now()` inside a decision
//! engine makes expiry untestable without sleeping, and makes decisions
//! irreproducible in the audit log. The daemon supplies the timestamp; the
//! engine is a pure function of its inputs.

#![forbid(unsafe_code)]

pub mod capability;

pub use capability::{Capability, CapabilityId, Risk, Scope, ScopeError};

use litric_core::AppId;
use std::collections::HashMap;

/// Seconds since the Unix epoch. Injected, never read from a clock here.
pub type Timestamp = u64;

/// Who issued a grant. Recorded so the audit log can distinguish a decision the
/// user made from one policy made on their behalf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Issuer {
    /// The user answered a prompt.
    User,
    /// The user performed an action that *is* the grant — choosing a file in
    /// the picker, or sharing a specific window. The strongest form of consent
    /// there is, because the user selected the exact resource.
    UserAction,
    /// System policy, e.g. a capability granted to a first-party component.
    Policy,
    /// A developer-mode override. Never present in a production session.
    Developer,
}

impl Issuer {
    /// Stable token for the audit log.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::UserAction => "user-action",
            Self::Policy => "policy",
            Self::Developer => "developer",
        }
    }

    /// Parse the audit token back.
    ///
    /// Unknown tokens are `None`, never a guess. A durable record whose issuer
    /// cannot be read must not be restored as `Policy`: that would turn a
    /// damaged line into an authority the user never granted.
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "user" => Self::User,
            "user-action" => Self::UserAction,
            "policy" => Self::Policy,
            "developer" => Self::Developer,
            _ => return None,
        })
    }
}

/// A capability granted to an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    capability: Capability,
    issuer: Issuer,
    issued_at: Timestamp,
    expires_at: Option<Timestamp>,
    revoked_at: Option<Timestamp>,
}

impl Grant {
    /// Create a grant with no expiry.
    #[must_use]
    pub const fn new(capability: Capability, issuer: Issuer, issued_at: Timestamp) -> Self {
        Self {
            capability,
            issuer,
            issued_at,
            expires_at: None,
            revoked_at: None,
        }
    }

    /// Create a grant that expires.
    ///
    /// Time-boxing is the default for anything a user was prompted about.
    /// "Allow while using the app" is a better answer than "allow forever" and
    /// this is the mechanism behind it.
    #[must_use]
    pub const fn expiring(
        capability: Capability,
        issuer: Issuer,
        issued_at: Timestamp,
        expires_at: Timestamp,
    ) -> Self {
        Self {
            capability,
            issuer,
            issued_at,
            expires_at: Some(expires_at),
            revoked_at: None,
        }
    }

    /// The capability granted.
    #[must_use]
    pub const fn capability(&self) -> &Capability {
        &self.capability
    }
    /// Who issued it.
    #[must_use]
    pub const fn issuer(&self) -> Issuer {
        self.issuer
    }
    /// When it was issued.
    #[must_use]
    pub const fn issued_at(&self) -> Timestamp {
        self.issued_at
    }
    /// When it expires, if ever.
    #[must_use]
    pub const fn expires_at(&self) -> Option<Timestamp> {
        self.expires_at
    }
    /// When it was revoked, if it has been.
    #[must_use]
    pub const fn revoked_at(&self) -> Option<Timestamp> {
        self.revoked_at
    }

    /// Rebuild a grant from a durable record.
    ///
    /// Only a store calls this. It is deliberately not a `FromStr`: the fields
    /// arrive as separate columns, the caller has already refused anything it
    /// could not parse, and keeping the constructor explicit makes the one
    /// place a revoked-at can be set from outside the broker visible.
    #[must_use]
    pub const fn restore(
        capability: Capability,
        issuer: Issuer,
        issued_at: Timestamp,
        expires_at: Option<Timestamp>,
        revoked_at: Option<Timestamp>,
    ) -> Self {
        Self {
            capability,
            issuer,
            issued_at,
            expires_at,
            revoked_at,
        }
    }

    /// Whether this grant is usable at `now`.
    ///
    /// Expiry uses `>=`, not `>`: a grant that expires at T is not valid *at* T.
    /// An off-by-one here is a permission that outlives its own deadline.
    #[must_use]
    pub fn is_active(&self, now: Timestamp) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        match self.expires_at {
            Some(exp) => now < exp,
            None => true,
        }
    }
}

/// Why a request was allowed or denied. Every decision carries one.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reason {
    /// An active grant authorised it.
    Granted {
        /// Who issued the authorising grant.
        issuer: Issuer,
        /// The scope of the authorising grant, which may be broader than the
        /// request.
        scope: Scope,
    },
    /// The application holds no grant for this capability at all.
    NoGrant,
    /// A grant exists but its scope does not cover the requested resource.
    OutOfScope {
        /// The scope actually held.
        held: Scope,
    },
    /// A grant existed and was revoked.
    Revoked {
        /// When.
        at: Timestamp,
    },
    /// A grant existed and has expired.
    Expired {
        /// When.
        at: Timestamp,
    },
    /// The capability requires consent in the moment and none was supplied.
    NeedsLiveConsent,
    /// The application is not known to the broker.
    UnknownApplication,
}

impl Reason {
    /// Stable token for the audit log.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Granted { .. } => "granted",
            Self::NoGrant => "no-grant",
            Self::OutOfScope { .. } => "out-of-scope",
            Self::Revoked { .. } => "revoked",
            Self::Expired { .. } => "expired",
            Self::NeedsLiveConsent => "needs-live-consent",
            Self::UnknownApplication => "unknown-application",
        }
    }
}

/// The outcome of a permission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    allowed: bool,
    reason: Reason,
}

impl Decision {
    /// Whether the operation may proceed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        self.allowed
    }
    /// Why.
    #[must_use]
    pub const fn reason(&self) -> &Reason {
        &self.reason
    }
}

/// One entry in the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    /// Which application.
    pub app: AppId,
    /// What it asked for.
    pub request: Capability,
    /// What happened.
    pub allowed: bool,
    /// Why.
    pub reason: Reason,
    /// When.
    pub at: Timestamp,
}

/// The capability broker.
///
/// Holds every grant on the system and answers every permission question.
/// Contains no I/O: `litric-capd` owns the socket, the persistence and the
/// prompting, and calls into this for the decision.
#[derive(Debug, Default)]
pub struct Broker {
    grants: HashMap<AppId, Vec<Grant>>,
    audit: Vec<AuditEntry>,
    audit_cap: usize,
}

impl Broker {
    /// Default number of audit entries retained in memory.
    ///
    /// The durable log is `litric-capd`'s responsibility; this is the ring the
    /// settings UI reads for "recent activity".
    pub const DEFAULT_AUDIT_CAP: usize = 4096;

    /// Hard ceiling on grants the broker will hold.
    ///
    /// One record per granted capability per application. At 1,024 the broker
    /// holds 1,024 × (a grant plus a map entry) — tens of kilobytes — and a
    /// hostile or damaged store cannot grow it without bound. The store writes
    /// and reads against this same number so the two ceilings cannot drift.
    pub const MAX_GRANTS: usize = 1024;

    /// Create an empty broker. **Every application starts with nothing.**
    #[must_use]
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
            audit: Vec::new(),
            audit_cap: Self::DEFAULT_AUDIT_CAP,
        }
    }

    /// Create with a specific audit ring size.
    #[must_use]
    pub fn with_audit_capacity(cap: usize) -> Self {
        Self {
            grants: HashMap::new(),
            audit: Vec::new(),
            audit_cap: cap.max(1),
        }
    }

    /// Register an application. Registration conveys **no** authority; it only
    /// distinguishes "an app we know with no permissions" from "an app we have
    /// never heard of", which are different things in an audit log.
    pub fn register(&mut self, app: AppId) {
        self.grants.entry(app).or_default();
    }

    /// Whether the broker knows this application.
    #[must_use]
    pub fn is_registered(&self, app: &AppId) -> bool {
        self.grants.contains_key(app)
    }

    /// How many grant records are held, revoked and expired ones included.
    #[must_use]
    pub fn grant_count(&self) -> usize {
        self.grants.values().map(Vec::len).sum()
    }

    /// Every grant on record, paired with its application.
    ///
    /// The daemon's store sorts this before writing so the file's contents do
    /// not depend on hash order.
    pub fn all_grants(&self) -> impl Iterator<Item = (&AppId, &Grant)> {
        self.grants
            .iter()
            .flat_map(|(app, list)| list.iter().map(move |grant| (app, grant)))
    }

    /// Restore grants read from a durable record.
    ///
    /// The caller has already validated every field. This refuses to exceed
    /// [`Self::MAX_GRANTS`], so a damaged or hostile file cannot grow the
    /// broker without bound, and returns how many records were accepted.
    pub fn restore_grants<I: IntoIterator<Item = (AppId, Grant)>>(&mut self, grants: I) -> usize {
        let mut accepted: usize = 0;
        for (app, grant) in grants {
            if accepted >= Self::MAX_GRANTS {
                break;
            }
            self.grants.entry(app).or_default().push(grant);
            accepted = accepted.saturating_add(1);
        }
        accepted
    }

    /// Restore the audit ring, oldest first, through the same bound every
    /// decision uses.
    pub fn restore_audit<I: IntoIterator<Item = AuditEntry>>(&mut self, entries: I) {
        for entry in entries {
            self.record(entry);
        }
    }

    /// Record an event that is decision-shaped but was not a `check` — the
    /// daemon writes a revocation here so the log answers "when did this stop
    /// working?" as well as "what did it ask for?".
    pub fn record_event(&mut self, entry: AuditEntry) {
        self.record(entry);
    }

    /// Issue a grant.
    ///
    /// Registers the application if it was not already known, so that a grant
    /// can never be issued into a void.
    pub fn grant(&mut self, app: &AppId, grant: Grant) {
        self.grants.entry(app.clone()).or_default().push(grant);
    }

    /// Revoke every grant of `id` held by `app`, effective at `now`.
    ///
    /// Returns how many grants were revoked.
    ///
    /// Revocation marks rather than deletes. A deleted grant leaves no trace
    /// that the permission was ever held, and "what did this app have access to
    /// last week?" is a question the user is entitled to an answer to.
    pub fn revoke(&mut self, app: &AppId, id: CapabilityId, now: Timestamp) -> usize {
        let Some(list) = self.grants.get_mut(app) else {
            return 0;
        };
        let mut n: usize = 0;
        for g in list.iter_mut() {
            if g.capability.id() == id && g.revoked_at.is_none() {
                g.revoked_at = Some(now);
                n = n.saturating_add(1);
            }
        }
        n
    }

    /// Revoke every grant held by `app`.
    pub fn revoke_all(&mut self, app: &AppId, now: Timestamp) -> usize {
        let Some(list) = self.grants.get_mut(app) else {
            return 0;
        };
        let mut n: usize = 0;
        for g in list.iter_mut() {
            if g.revoked_at.is_none() {
                g.revoked_at = Some(now);
                n = n.saturating_add(1);
            }
        }
        n
    }

    /// Decide whether `app` may perform `request` at `now`, and record it.
    ///
    /// # Confused-deputy defence
    ///
    /// `app` is the *authenticated caller identity*, established by the daemon
    /// from `SO_PEERCRED` and the caller's cgroup. It is never a value the
    /// caller supplied. A caller cannot name a different application, and
    /// cannot present a capability it claims to hold — authority is looked up
    /// here, from the broker's own records. That is the whole defence, and it
    /// works only because there is no code path in this function that trusts
    /// anything the caller said about itself.
    ///
    /// This answers the *grant* question only. A capability whose
    /// [`Capability::requires_live_consent`] is true needs
    /// [`Self::check_with_consent`] instead; using this one for an action that
    /// ends the session would let an hour-old grant stand in for the user
    /// saying yes now.
    pub fn check(&mut self, app: &AppId, request: &Capability, now: Timestamp) -> Decision {
        let decision = self.evaluate(app, request, now);
        self.record(AuditEntry {
            app: app.clone(),
            request: request.clone(),
            allowed: decision.allowed,
            reason: decision.reason.clone(),
            at: now,
        });
        decision
    }

    /// Decide whether `app` may perform `request` *right now*, given whether
    /// the user consented in this moment, and record the answer.
    ///
    /// The vocabulary has declared live consent since the capability file was
    /// written — `Camera`, `Microphone`, `AuthSession`, and every scope for
    /// which [`Scope::is_destructive`] is true — and until this method existed
    /// [`Reason::NeedsLiveConsent`] had no producer anywhere in the tree. A
    /// rule that is written down and not enforced is a rule the system
    /// contradicts, so the prompt is a broker decision rather than a habit of
    /// whatever caller happens to be drawing a dialog.
    ///
    /// `consent` is the *UI layer's* report that the user answered a prompt
    /// naming this action. It is not authority: a caller with no grant is still
    /// refused, and consenting does not widen a scope. It only decides the one
    /// question the grant cannot — whether yes was said this time.
    pub fn check_with_consent(
        &mut self,
        app: &AppId,
        request: &Capability,
        consent: bool,
        now: Timestamp,
    ) -> Decision {
        let evaluated = self.evaluate(app, request, now);
        let decision = if evaluated.allowed && request.requires_live_consent() && !consent {
            Decision {
                allowed: false,
                reason: Reason::NeedsLiveConsent,
            }
        } else {
            evaluated
        };
        self.record(AuditEntry {
            app: app.clone(),
            request: request.clone(),
            allowed: decision.allowed,
            reason: decision.reason.clone(),
            at: now,
        });
        decision
    }

    /// Decide without recording. Used by the settings UI to show what *would*
    /// happen, which must not pollute the log with decisions nobody acted on.
    #[must_use]
    pub fn would_allow(&self, app: &AppId, request: &Capability, now: Timestamp) -> Decision {
        self.evaluate(app, request, now)
    }

    fn evaluate(&self, app: &AppId, request: &Capability, now: Timestamp) -> Decision {
        let deny = |reason| Decision {
            allowed: false,
            reason,
        };

        let Some(list) = self.grants.get(app) else {
            return deny(Reason::UnknownApplication);
        };

        // Walk every grant for this capability id. The FIRST active grant whose
        // scope covers the request wins.
        //
        // Non-matching grants are remembered only to produce a better denial
        // reason. They never influence the allow/deny outcome -- an expired
        // grant is exactly as authoritative as no grant at all, and the reason
        // text must never be allowed to leak into the decision.
        let mut best_denial: Option<Reason> = None;

        for g in list.iter().filter(|g| g.capability.id() == request.id()) {
            if let Some(at) = g.revoked_at {
                best_denial.get_or_insert(Reason::Revoked { at });
                continue;
            }
            if let Some(exp) = g.expires_at {
                if now >= exp {
                    best_denial.get_or_insert(Reason::Expired { at: exp });
                    continue;
                }
            }
            if g.capability.authorises(request) {
                return Decision {
                    allowed: true,
                    reason: Reason::Granted {
                        issuer: g.issuer,
                        scope: g.capability.scope().clone(),
                    },
                };
            }
            // Active, right capability, wrong resource.
            best_denial = Some(Reason::OutOfScope {
                held: g.capability.scope().clone(),
            });
        }

        deny(best_denial.unwrap_or(Reason::NoGrant))
    }

    fn record(&mut self, entry: AuditEntry) {
        if self.audit.len() >= self.audit_cap {
            // Drop the oldest. An unbounded audit buffer in a system service is
            // a memory-exhaustion denial of service triggerable by any
            // application that calls in a loop (RESOURCE GOVERNANCE, §54).
            self.audit.remove(0);
        }
        self.audit.push(entry);
    }

    /// Every grant held by an application, including revoked and expired ones.
    ///
    /// The settings UI shows history, not just current state.
    #[must_use]
    pub fn grants_for(&self, app: &AppId) -> &[Grant] {
        self.grants.get(app).map_or(&[], Vec::as_slice)
    }

    /// The capabilities an application can actually use right now.
    #[must_use]
    pub fn active_capabilities(&self, app: &AppId, now: Timestamp) -> Vec<Capability> {
        self.grants
            .get(app)
            .map(|l| {
                l.iter()
                    .filter(|g| g.is_active(now))
                    .map(|g| g.capability.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The audit log, oldest first.
    #[must_use]
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit
    }

    /// Audit entries for one application.
    #[must_use]
    pub fn audit_for(&self, app: &AppId) -> Vec<&AuditEntry> {
        self.audit.iter().filter(|e| &e.app == app).collect()
    }

    /// Discard the audit log. Exposed because the user must be able to clear
    /// their own history (Phase 09 task 7: one-click purge).
    pub fn purge_audit(&mut self) {
        self.audit.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(s: &str) -> AppId {
        AppId::parse(s).expect("valid app id")
    }
    fn uri(s: &str) -> Scope {
        Scope::parse_uri(s).expect("valid scope")
    }
    fn fs_read(path: &str) -> Capability {
        Capability::new(CapabilityId::FsRead, uri(path)).expect("valid capability")
    }
    fn cam() -> Capability {
        Capability::unscoped(CapabilityId::Camera).expect("valid capability")
    }

    const T0: Timestamp = 1_000;

    // ---- rule 1: deny by default ------------------------------------------

    #[test]
    fn an_app_with_no_grants_can_do_nothing() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.register(a.clone());

        for c in [
            cam(),
            Capability::unscoped(CapabilityId::Microphone).expect("v"),
            Capability::unscoped(CapabilityId::Location).expect("v"),
            Capability::unscoped(CapabilityId::NetClient).expect("v"),
            fs_read("user://Documents"),
            fs_read("user://"),
        ] {
            let d = b.check(&a, &c, T0);
            assert!(!d.is_allowed(), "{c} must be denied by default");
            assert_eq!(*d.reason(), Reason::NoGrant);
        }
    }

    #[test]
    fn an_unregistered_app_is_denied_and_distinguishable() {
        let mut b = Broker::new();
        let d = b.check(&app("com.unknown.app"), &cam(), T0);
        assert!(!d.is_allowed());
        assert_eq!(*d.reason(), Reason::UnknownApplication);
    }

    #[test]
    fn registration_conveys_no_authority() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.register(a.clone());
        assert!(b.is_registered(&a));
        assert!(b.active_capabilities(&a, T0).is_empty());
        assert!(!b.check(&a, &cam(), T0).is_allowed());
    }

    // ---- rule 2: least privilege by scope ---------------------------------

    #[test]
    fn a_grant_authorises_only_within_its_scope() {
        let mut b = Broker::new();
        let a = app("com.example.editor");
        b.grant(
            &a,
            Grant::new(fs_read("user://Documents"), Issuer::UserAction, T0),
        );

        assert!(b.check(&a, &fs_read("user://Documents"), T0).is_allowed());
        assert!(b
            .check(&a, &fs_read("user://Documents/tax/2026.pdf"), T0)
            .is_allowed());

        // Outside: sibling, parent, and the lookalike prefix.
        for outside in ["user://Pictures", "user://", "user://DocumentsSecret"] {
            let d = b.check(&a, &fs_read(outside), T0);
            assert!(!d.is_allowed(), "{outside} must be denied");
            assert!(matches!(d.reason(), Reason::OutOfScope { .. }));
        }
    }

    #[test]
    fn read_does_not_imply_write() {
        let mut b = Broker::new();
        let a = app("com.example.editor");
        b.grant(
            &a,
            Grant::new(fs_read("user://Documents"), Issuer::UserAction, T0),
        );

        let write = Capability::new(CapabilityId::FsWrite, uri("user://Documents")).expect("v");
        let d = b.check(&a, &write, T0);
        assert!(!d.is_allowed());
        assert_eq!(
            *d.reason(),
            Reason::NoGrant,
            "no fs.write grant exists at all"
        );
    }

    // ---- rule 3: revocation is immediate ----------------------------------

    #[test]
    fn revocation_takes_effect_on_the_very_next_check() {
        // GATE 07: "runtime revocation of microphone takes effect on a RUNNING
        // app". No restart, no cache, no grace period.
        let mut b = Broker::new();
        let a = app("com.example.recorder");
        let mic = Capability::unscoped(CapabilityId::Microphone).expect("v");
        b.grant(&a, Grant::new(mic.clone(), Issuer::User, T0));

        assert!(b.check(&a, &mic, T0).is_allowed());

        let n = b.revoke(&a, CapabilityId::Microphone, T0 + 10);
        assert_eq!(n, 1);

        let d = b.check(&a, &mic, T0 + 10);
        assert!(!d.is_allowed(), "revocation must be immediate");
        assert_eq!(*d.reason(), Reason::Revoked { at: T0 + 10 });
    }

    #[test]
    fn revoking_one_capability_leaves_the_others_alone() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        let mic = Capability::unscoped(CapabilityId::Microphone).expect("v");
        b.grant(&a, Grant::new(mic.clone(), Issuer::User, T0));
        b.grant(&a, Grant::new(cam(), Issuer::User, T0));

        b.revoke(&a, CapabilityId::Microphone, T0 + 5);
        assert!(!b.check(&a, &mic, T0 + 5).is_allowed());
        assert!(
            b.check(&a, &cam(), T0 + 5).is_allowed(),
            "camera must be unaffected"
        );
    }

    #[test]
    fn revoke_all_removes_everything_and_reports_the_count() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::new(cam(), Issuer::User, T0));
        b.grant(
            &a,
            Grant::new(fs_read("user://Documents"), Issuer::UserAction, T0),
        );
        b.grant(
            &a,
            Grant::new(
                Capability::unscoped(CapabilityId::Notify).expect("v"),
                Issuer::Policy,
                T0,
            ),
        );

        assert_eq!(b.revoke_all(&a, T0 + 1), 3);
        assert!(b.active_capabilities(&a, T0 + 1).is_empty());
        // A second revoke_all finds nothing left to do.
        assert_eq!(b.revoke_all(&a, T0 + 2), 0);
    }

    #[test]
    fn revocation_preserves_history() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::new(cam(), Issuer::User, T0));
        b.revoke(&a, CapabilityId::Camera, T0 + 1);

        // The grant is still on record, marked revoked -- the user can still ask
        // "did this app ever have my camera?".
        let grants = b.grants_for(&a);
        assert_eq!(grants.len(), 1);
        assert_eq!(grants.first().and_then(Grant::revoked_at), Some(T0 + 1));
    }

    // ---- rule 4: expiry ----------------------------------------------------

    #[test]
    fn an_expiring_grant_stops_working_at_its_deadline() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        let loc = Capability::unscoped(CapabilityId::Location).expect("v");
        b.grant(&a, Grant::expiring(loc.clone(), Issuer::User, T0, T0 + 100));

        assert!(b.check(&a, &loc, T0).is_allowed());
        assert!(b.check(&a, &loc, T0 + 99).is_allowed());

        // Boundary: a grant that expires at T is NOT valid at T.
        let d = b.check(&a, &loc, T0 + 100);
        assert!(
            !d.is_allowed(),
            "a grant must not be valid at its own expiry instant"
        );
        assert_eq!(*d.reason(), Reason::Expired { at: T0 + 100 });

        assert!(!b.check(&a, &loc, T0 + 1_000_000).is_allowed());
    }

    #[test]
    fn an_expired_grant_is_exactly_as_powerful_as_no_grant() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::expiring(cam(), Issuer::User, T0, T0 + 10));
        assert!(!b.check(&a, &cam(), T0 + 11).is_allowed());
        assert!(b.active_capabilities(&a, T0 + 11).is_empty());
    }

    #[test]
    fn a_later_valid_grant_overrides_an_earlier_expired_one() {
        // Re-granting must work. Otherwise "allow again" after an expiry would
        // silently do nothing, which is the kind of bug users describe as
        // "the permission button doesn't work".
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::expiring(cam(), Issuer::User, T0, T0 + 10));
        b.grant(&a, Grant::new(cam(), Issuer::User, T0 + 20));

        let d = b.check(&a, &cam(), T0 + 30);
        assert!(d.is_allowed(), "the newer unexpired grant must win");
        assert!(matches!(d.reason(), Reason::Granted { .. }));
    }

    #[test]
    fn a_revoked_grant_does_not_shadow_a_valid_one() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::new(cam(), Issuer::User, T0));
        b.revoke(&a, CapabilityId::Camera, T0 + 1);
        b.grant(&a, Grant::new(cam(), Issuer::User, T0 + 2));

        assert!(b.check(&a, &cam(), T0 + 3).is_allowed());
    }

    // ---- rule 5: consent in the moment ------------------------------------

    fn power(verb: &str) -> Capability {
        Capability::new(
            CapabilityId::PowerControl,
            Scope::parse_resource(&format!("power://system/{verb}")).expect("valid scope"),
        )
        .expect("valid capability")
    }

    #[test]
    fn a_destructive_scope_without_consent_is_refused_by_the_broker() {
        // The vocabulary has said since it was written that suspending the
        // machine needs a yes in the moment. Before check_with_consent existed
        // nothing produced Reason::NeedsLiveConsent, so the rule was a
        // document rather than a decision.
        let mut b = Broker::new();
        let a = app("com.litric.shell");
        b.grant(&a, Grant::new(power("poweroff"), Issuer::User, T0));

        let denied = b.check_with_consent(&a, &power("poweroff"), false, T0);
        assert!(!denied.is_allowed());
        assert_eq!(*denied.reason(), Reason::NeedsLiveConsent);

        let allowed = b.check_with_consent(&a, &power("poweroff"), true, T0 + 1);
        assert!(allowed.is_allowed(), "{allowed:?}");
        assert!(matches!(allowed.reason(), Reason::Granted { .. }));
    }

    #[test]
    fn consent_is_not_a_grant() {
        // Saying yes to a prompt must not manufacture authority: an app with
        // no grant is still refused, and one granted only the brightness
        // cannot suspend the machine by asking again.
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.register(a.clone());
        assert_eq!(
            *b.check_with_consent(&a, &power("reboot"), true, T0)
                .reason(),
            Reason::NoGrant
        );

        b.grant(
            &a,
            Grant::new(
                Capability::new(
                    CapabilityId::PowerControl,
                    Scope::parse_resource("backlight://intel_backlight").expect("v"),
                )
                .expect("v"),
                Issuer::User,
                T0,
            ),
        );
        let d = b.check_with_consent(&a, &power("reboot"), true, T0 + 1);
        assert!(!d.is_allowed(), "a backlight grant is not a reboot grant");
        assert!(
            matches!(d.reason(), Reason::OutOfScope { .. }),
            "{:?}",
            d.reason()
        );
    }

    #[test]
    fn a_capability_that_needs_no_consent_is_unaffected_by_the_flag() {
        // Reading a file must not start demanding a prompt because it went
        // through the consent-aware path.
        let mut b = Broker::new();
        let a = app("com.example.editor");
        let c = fs_read("user://Documents");
        b.grant(&a, Grant::new(c.clone(), Issuer::User, T0));
        assert!(b.check_with_consent(&a, &c, false, T0).is_allowed());
    }

    #[test]
    fn a_refused_consent_is_recorded_with_its_reason() {
        let mut b = Broker::new();
        let a = app("com.litric.shell");
        b.grant(&a, Grant::new(power("suspend"), Issuer::User, T0));
        let _ = b.check_with_consent(&a, &power("suspend"), false, T0);
        let last = b.audit_log().last().expect("an entry");
        assert!(!last.allowed);
        assert_eq!(last.reason.token(), "needs-live-consent");
    }

    // ---- rule 6: everything is audited ------------------------------------

    #[test]
    fn both_allows_and_denies_are_recorded() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::new(cam(), Issuer::User, T0));

        b.check(&a, &cam(), T0);
        b.check(
            &a,
            &Capability::unscoped(CapabilityId::Microphone).expect("v"),
            T0 + 1,
        );

        let log = b.audit_log();
        assert_eq!(log.len(), 2);
        assert!(log.first().is_some_and(|e| e.allowed));
        assert!(log.get(1).is_some_and(|e| !e.allowed));
        assert!(log.get(1).is_some_and(|e| e.reason == Reason::NoGrant));
    }

    #[test]
    fn preview_checks_do_not_pollute_the_log() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.register(a.clone());
        let _ = b.would_allow(&a, &cam(), T0);
        assert!(b.audit_log().is_empty(), "would_allow must not record");
    }

    #[test]
    fn the_audit_buffer_is_bounded() {
        // An app calling in a loop must not be able to exhaust memory in a
        // system service (RESOURCE GOVERNANCE §54).
        let mut b = Broker::with_audit_capacity(8);
        let a = app("com.example.spammer");
        b.register(a.clone());
        for i in 0..1000 {
            b.check(&a, &cam(), T0 + i);
        }
        assert_eq!(b.audit_log().len(), 8);
        // And it keeps the NEWEST, not the oldest.
        assert!(b.audit_log().last().is_some_and(|e| e.at == T0 + 999));
    }

    #[test]
    fn the_user_can_purge_their_own_history() {
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.register(a.clone());
        b.check(&a, &cam(), T0);
        assert!(!b.audit_log().is_empty());
        b.purge_audit();
        assert!(b.audit_log().is_empty());
    }

    #[test]
    fn audit_can_be_filtered_per_application() {
        let mut b = Broker::new();
        let a = app("com.example.one");
        let c = app("com.example.two");
        b.register(a.clone());
        b.register(c.clone());
        b.check(&a, &cam(), T0);
        b.check(&c, &cam(), T0);
        b.check(&a, &cam(), T0 + 1);

        assert_eq!(b.audit_for(&a).len(), 2);
        assert_eq!(b.audit_for(&c).len(), 1);
    }

    // ---- rule 6: no confused deputy ---------------------------------------

    #[test]
    fn one_apps_grant_never_authorises_another_app() {
        // The confused-deputy property. App A is fully trusted; app B must gain
        // nothing from that, no matter how similar its identity looks.
        let mut b = Broker::new();
        let trusted = app("com.example.trusted");
        b.grant(
            &trusted,
            Grant::new(fs_read("user://Documents"), Issuer::UserAction, T0),
        );
        b.grant(&trusted, Grant::new(cam(), Issuer::User, T0));

        for other in [
            "com.example.untrusted",
            "com.example.trusted2",
            "com.example.trusted.helper",
            "org.example.trusted",
        ] {
            let o = app(other);
            assert!(
                !b.check(&o, &cam(), T0).is_allowed(),
                "{other} must not inherit camera"
            );
            assert!(
                !b.check(&o, &fs_read("user://Documents"), T0).is_allowed(),
                "{other} must not inherit file access"
            );
        }
    }

    #[test]
    fn identity_is_case_normalised_so_a_lookalike_is_the_same_app_not_a_new_one() {
        // AppId canonicalises case, so "Com.Example.App" cannot register as a
        // second identity that a user might confuse with the first.
        let mut b = Broker::new();
        let a = app("com.example.app");
        b.grant(&a, Grant::new(cam(), Issuer::User, T0));
        let same = AppId::parse("Com.Example.App").expect("valid");
        assert!(b.check(&same, &cam(), T0).is_allowed());
    }

    // ---- scenario ----------------------------------------------------------

    #[test]
    fn full_lifecycle_of_a_photo_app() {
        let mut b = Broker::new();
        let a = app("com.example.photos");
        b.register(a.clone());

        // 1. Installed. Nothing works.
        assert!(!b.check(&a, &cam(), T0).is_allowed());
        assert!(!b.check(&a, &fs_read("media://Pictures"), T0).is_allowed());

        // 2. The user picks a folder. The picker IS the grant.
        b.grant(
            &a,
            Grant::new(fs_read("media://Pictures"), Issuer::UserAction, T0 + 1),
        );
        assert!(b
            .check(&a, &fs_read("media://Pictures/2026/june"), T0 + 2)
            .is_allowed());
        // ...but only that folder.
        assert!(!b
            .check(&a, &fs_read("user://Documents"), T0 + 2)
            .is_allowed());

        // 3. The user allows the camera for this session only.
        b.grant(&a, Grant::expiring(cam(), Issuer::User, T0 + 3, T0 + 3600));
        assert!(b.check(&a, &cam(), T0 + 100).is_allowed());

        // 4. Session ends. Camera stops working; files still work.
        assert!(!b.check(&a, &cam(), T0 + 3600).is_allowed());
        assert!(b
            .check(&a, &fs_read("media://Pictures"), T0 + 3600)
            .is_allowed());

        // 5. The user revokes file access from Settings.
        b.revoke(&a, CapabilityId::FsRead, T0 + 4000);
        assert!(!b
            .check(&a, &fs_read("media://Pictures"), T0 + 4001)
            .is_allowed());

        // 6. Everything is on the record.
        let entries = b.audit_for(&a);
        assert!(entries.len() >= 8);
        assert!(entries.iter().any(|e| e.allowed));
        assert!(entries.iter().any(|e| !e.allowed));
    }
}
