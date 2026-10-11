//! LitricOS environment manager.
//!
//! # Layer position
//!
//! **Layer 2 (Core Services).** Depends downward on `litric-core` and on nothing
//! else. Performs no I/O and starts no virtual machines: it decides *what should
//! happen* when the user presses `Win+N`, and `litric-envd` carries it out.
//!
//! Splitting the decision from the execution is what makes the policy testable
//! on any host, with no hypervisor, which is why every rule below is backed by a
//! test that runs today.
//!
//! # What this models
//!
//! LitricOS is the host. Windows and Linux are guests (ADR-0015). The user
//! switches between them with a hotkey, and the machine stays owned by LitricOS
//! throughout:
//!
//! ```text
//!   Win+1 -> Windows      Win+2 -> LitricOS      Win+3 -> Linux
//! ```
//!
//! # The invariant everything else protects
//!
//! **The host cannot be suspended, hibernated or stopped.** Not "should not" —
//! [`EnvironmentManager`] has no code path that permits it. If the control plane
//! could be suspended, then a guest could take the machine with it, and the
//! entire premise of LitricOS being the trust anchor would be false.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::BTreeMap;

/// Which environment.
///
/// Deliberately a closed set rather than an open registry: each entry implies
/// hypervisor configuration, firmware choices and licensing consequences that
/// cannot be filled in generically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EnvironmentId {
    /// The host. Always present, always running.
    Litric,
    /// A Windows guest, supplied and licensed by the user.
    Windows,
    /// A Linux guest, installed on demand.
    Linux,
}

impl EnvironmentId {
    /// Every environment, in the order the UI presents them.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Windows, Self::Litric, Self::Linux]
    }

    /// Stable token for logs and configuration.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Litric => "litric",
            Self::Windows => "windows",
            Self::Linux => "linux",
        }
    }

    /// Display name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Litric => "LitricOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
        }
    }

    /// Whether this environment is the host.
    ///
    /// The host is not a guest with extra privileges; it is the thing running
    /// the guests. Every rule that protects the control plane keys off this.
    #[must_use]
    pub const fn is_host(self) -> bool {
        matches!(self, Self::Litric)
    }

    /// Whether LitricOS may distribute this environment.
    ///
    /// `false` for Windows: the user supplies their own media and a licence
    /// valid for a virtual machine. LitricOS installs *from media the user
    /// provides* and never ships Windows itself (ADR-0015 constraint 2).
    #[must_use]
    pub const fn is_distributable(self) -> bool {
        !matches!(self, Self::Windows)
    }
}

impl fmt::Display for EnvironmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Lifecycle state of an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EnvState {
    /// No installation exists.
    NotInstalled,
    /// Installed, not running. Uses disk, no CPU or RAM.
    Stopped,
    /// Booting.
    Starting,
    /// Executing.
    Running,
    /// Execution paused; memory still resident. Resumes in seconds.
    Suspended,
    /// Memory persisted to disk and released. Resumes in tens of seconds.
    Hibernated,
    /// Entered a state it cannot leave without intervention.
    Failed,
}

impl EnvState {
    /// Stable token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::NotInstalled => "not-installed",
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Suspended => "suspended",
            Self::Hibernated => "hibernated",
            Self::Failed => "failed",
        }
    }

    /// Whether the environment is currently consuming CPU.
    #[must_use]
    pub const fn consumes_cpu(self) -> bool {
        matches!(self, Self::Starting | Self::Running)
    }

    /// Whether the environment is currently holding RAM.
    ///
    /// Suspended still holds memory — that is the whole difference between
    /// suspend and hibernate, and it is why suspend is fast and hibernate frees
    /// the machine.
    #[must_use]
    pub const fn holds_memory(self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Suspended)
    }

    /// Transitions permitted from this state.
    ///
    /// Enumerated rather than allowing arbitrary assignment. "Suspend a
    /// hibernated environment" and "stop something mid-start" are the kind of
    /// requests that produce corrupted guest state, and they are refused here
    /// rather than half-executed.
    #[must_use]
    pub fn allowed_transitions(self) -> &'static [EnvState] {
        match self {
            Self::NotInstalled => &[Self::Stopped],
            Self::Stopped => &[Self::Starting, Self::NotInstalled],
            Self::Starting => &[Self::Running, Self::Failed],
            Self::Running => &[
                Self::Suspended,
                Self::Hibernated,
                Self::Stopped,
                Self::Failed,
            ],
            Self::Suspended => &[Self::Running, Self::Hibernated, Self::Stopped, Self::Failed],
            Self::Hibernated => &[Self::Running, Self::Stopped, Self::Failed],
            // A failed environment is reset, never resumed. Resuming state that
            // is already known bad is how a fault becomes data loss.
            Self::Failed => &[Self::Stopped, Self::NotInstalled],
        }
    }

    /// Whether `self -> next` is legal.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        self.allowed_transitions().contains(&next)
    }
}

impl fmt::Display for EnvState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Observed health, independent of lifecycle state.
///
/// An environment can be `Running` and `Compromised` at the same time; those are
/// different questions. Collapsing them would mean the only way to express "this
/// is running but should not be trusted" is to kill it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Health {
    /// Normal.
    #[default]
    Healthy,
    /// Working, with faults.
    Degraded,
    /// Not answering its heartbeat.
    Unresponsive,
    /// Integrity or security violation detected.
    Compromised,
    /// Persistent state is damaged.
    Corrupted,
}

impl Health {
    /// Stable token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unresponsive => "unresponsive",
            Self::Compromised => "compromised",
            Self::Corrupted => "corrupted",
        }
    }

    /// Whether the user should be allowed to switch into this environment
    /// without an explicit warning.
    ///
    /// A compromised environment is still switchable — the user may need to get
    /// data out of it — but never silently.
    #[must_use]
    pub const fn switchable_without_warning(self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }
}

/// CPU and memory an environment is permitted.
///
/// Budgets are enforced by the host (cgroups and KVM limits), so a guest cannot
/// take resources the host reserved for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// Logical CPUs the environment may use.
    pub cpus: u32,
    /// Memory ceiling in MiB.
    pub memory_mb: u32,
}

impl Budget {
    /// Construct.
    #[must_use]
    pub const fn new(cpus: u32, memory_mb: u32) -> Self {
        Self { cpus, memory_mb }
    }
}

/// What the host always keeps, no matter what guests ask for.
///
/// The control plane must never be starved: if LitricOS cannot schedule, the
/// user cannot switch away from whatever is starving it, and the machine is
/// effectively lost to a guest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostReservation {
    /// Logical CPUs reserved for the host.
    pub cpus: u32,
    /// Memory reserved for the host, MiB.
    pub memory_mb: u32,
}

impl Default for HostReservation {
    /// Two CPUs and 512 MiB.
    ///
    /// Chosen so the compositor can still present a frame and the shell can
    /// still respond to the switch hotkey while a guest is saturating the
    /// machine. Below this, "press Win+2 to escape" stops being reliable, which
    /// is precisely when the user needs it.
    fn default() -> Self {
        Self {
            cpus: 2,
            memory_mb: 512,
        }
    }
}

/// One environment's record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    id: EnvironmentId,
    state: EnvState,
    health: Health,
    budget: Budget,
}

impl Environment {
    /// Create an environment record.
    #[must_use]
    pub const fn new(id: EnvironmentId, state: EnvState, budget: Budget) -> Self {
        Self {
            id,
            state,
            health: Health::Healthy,
            budget,
        }
    }

    /// Which environment.
    #[must_use]
    pub const fn id(&self) -> EnvironmentId {
        self.id
    }
    /// Lifecycle state.
    #[must_use]
    pub const fn state(&self) -> EnvState {
        self.state
    }
    /// Observed health.
    #[must_use]
    pub const fn health(&self) -> Health {
        self.health
    }
    /// Resource budget.
    #[must_use]
    pub const fn budget(&self) -> Budget {
        self.budget
    }

    /// Record observed health.
    pub fn set_health(&mut self, health: Health) {
        self.health = health;
    }
}

/// What actually happened when the user asked to switch.
///
/// The manager reports the mechanism, not just success. "It took twelve seconds"
/// is a useful thing to be able to say; appearing to hang is not.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SwitchOutcome {
    /// Already there. Nothing to do.
    AlreadyActive,
    /// The target was running; the compositor swaps the presented surface.
    /// Roughly one frame.
    Presented,
    /// The target was suspended and is being resumed first. Seconds.
    Resuming,
    /// The target was hibernated and is being restored from disk. Tens of
    /// seconds.
    Restoring,
    /// The target was stopped and is being booted. Boot time.
    Starting,
    /// The target is not installed. **No switch occurs**; the caller should
    /// offer installation rather than appear to switch and then do nothing.
    NotInstalled {
        /// Whether LitricOS may supply this environment, or the user must.
        user_must_supply_media: bool,
    },
    /// The target is in a state that needs attention first.
    NeedsRecovery {
        /// Why.
        health: Health,
    },
    /// The target failed and must be reset before it can run again.
    Failed,
}

impl SwitchOutcome {
    /// Whether the display will end up showing the requested environment.
    #[must_use]
    pub const fn will_switch(&self) -> bool {
        matches!(
            self,
            Self::AlreadyActive
                | Self::Presented
                | Self::Resuming
                | Self::Restoring
                | Self::Starting
        )
    }

    /// Rough latency class, for the UI to decide whether to show progress.
    #[must_use]
    pub const fn latency_class(&self) -> LatencyClass {
        match self {
            Self::AlreadyActive | Self::Presented => LatencyClass::Instant,
            Self::Resuming => LatencyClass::Seconds,
            Self::Restoring | Self::Starting => LatencyClass::TensOfSeconds,
            Self::NotInstalled { .. } | Self::NeedsRecovery { .. } | Self::Failed => {
                LatencyClass::NoSwitch
            }
        }
    }
}

/// How long a switch will take, coarsely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyClass {
    /// About one frame.
    Instant,
    /// A few seconds. Show progress.
    Seconds,
    /// Tens of seconds. Show progress and what is happening.
    TensOfSeconds,
    /// Nothing will be switched.
    NoSwitch,
}

/// A hotkey binding.
///
/// Modelled rather than hard-coded because `Win+N` collides with Windows' own
/// taskbar shortcuts, so it must be remappable, and because the binding has to
/// be grabbed by the host compositor *before* a guest sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hotkey {
    /// Modifier bitmask. See [`Hotkey::SUPER`] and friends.
    pub modifiers: u8,
    /// Key identifier. Opaque here; the compositor maps it to a keysym.
    pub key: u16,
}

impl Hotkey {
    /// Super / Windows key.
    pub const SUPER: u8 = 1 << 0;
    /// Control.
    pub const CTRL: u8 = 1 << 1;
    /// Alt.
    pub const ALT: u8 = 1 << 2;
    /// Shift.
    pub const SHIFT: u8 = 1 << 3;

    /// Construct.
    #[must_use]
    pub const fn new(modifiers: u8, key: u16) -> Self {
        Self { modifiers, key }
    }

    /// Whether the binding has at least one modifier.
    ///
    /// An unmodified key cannot be a global switch binding: the host would have
    /// to grab a bare keypress before every guest and every application, which
    /// would make that key unusable everywhere.
    #[must_use]
    pub const fn has_modifier(self) -> bool {
        self.modifiers != 0
    }
}

/// Why a binding was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindError {
    /// No modifier. See [`Hotkey::has_modifier`].
    NoModifier,
    /// Already bound to another environment.
    Conflict {
        /// What holds it.
        existing: EnvironmentId,
    },
    /// Collides with the reserved escape binding.
    ReservedEscape,
}

impl fmt::Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoModifier => write!(f, "a switch binding must include a modifier"),
            Self::Conflict { existing } => write!(f, "already bound to {existing}"),
            Self::ReservedEscape => write!(f, "collides with the reserved escape binding"),
        }
    }
}

impl std::error::Error for BindError {}

/// The environment manager.
///
/// Holds every environment, the active one, the hotkey map and the host
/// reservation, and answers "what should happen if the user presses this".
#[derive(Debug, Clone)]
pub struct EnvironmentManager {
    environments: BTreeMap<EnvironmentId, Environment>,
    active: EnvironmentId,
    bindings: BTreeMap<Hotkey, EnvironmentId>,
    reservation: HostReservation,
    total_cpus: u32,
    total_memory_mb: u32,
}

impl EnvironmentManager {
    /// The escape binding, `Ctrl+Alt+Home`.
    ///
    /// Always returns to the host, is never remappable, and never bindable to
    /// anything else. It exists for the case where a guest has grabbed input or
    /// is hung — precisely when the ordinary switch key may not get through.
    pub const ESCAPE: Hotkey = Hotkey::new(Hotkey::CTRL | Hotkey::ALT, 0x0050);

    /// Create a manager for a machine with the given resources.
    ///
    /// The host starts installed and running because it is the thing doing the
    /// creating; guests start `NotInstalled`.
    #[must_use]
    pub fn new(total_cpus: u32, total_memory_mb: u32) -> Self {
        let mut environments = BTreeMap::new();
        environments.insert(
            EnvironmentId::Litric,
            Environment::new(
                EnvironmentId::Litric,
                EnvState::Running,
                Budget::new(total_cpus, total_memory_mb),
            ),
        );
        for id in [EnvironmentId::Windows, EnvironmentId::Linux] {
            environments.insert(
                id,
                Environment::new(id, EnvState::NotInstalled, Budget::new(0, 0)),
            );
        }

        let mut bindings = BTreeMap::new();
        // Win+1 / Win+2 / Win+3. Keys are opaque identifiers here; the
        // compositor maps them to real keysyms.
        bindings.insert(Hotkey::new(Hotkey::SUPER, 0x0031), EnvironmentId::Windows);
        bindings.insert(Hotkey::new(Hotkey::SUPER, 0x0032), EnvironmentId::Litric);
        bindings.insert(Hotkey::new(Hotkey::SUPER, 0x0033), EnvironmentId::Linux);

        Self {
            environments,
            active: EnvironmentId::Litric,
            bindings,
            reservation: HostReservation::default(),
            total_cpus,
            total_memory_mb,
        }
    }

    /// The environment currently on screen.
    #[must_use]
    pub const fn active(&self) -> EnvironmentId {
        self.active
    }

    /// The host reservation.
    #[must_use]
    pub const fn reservation(&self) -> HostReservation {
        self.reservation
    }

    /// Look up an environment.
    #[must_use]
    pub fn get(&self, id: EnvironmentId) -> Option<&Environment> {
        self.environments.get(&id)
    }

    /// Every environment, in UI order.
    #[must_use]
    pub fn all(&self) -> Vec<&Environment> {
        EnvironmentId::all()
            .iter()
            .filter_map(|id| self.environments.get(id))
            .collect()
    }

    /// Which environment a hotkey selects, if any.
    #[must_use]
    pub fn binding_for(&self, key: Hotkey) -> Option<EnvironmentId> {
        if key == Self::ESCAPE {
            return Some(EnvironmentId::Litric);
        }
        self.bindings.get(&key).copied()
    }

    /// Rebind a switch hotkey.
    ///
    /// # Errors
    /// [`BindError`] if the binding has no modifier, collides with another
    /// environment, or collides with the escape binding.
    pub fn rebind(&mut self, key: Hotkey, target: EnvironmentId) -> Result<(), BindError> {
        if !key.has_modifier() {
            return Err(BindError::NoModifier);
        }
        if key == Self::ESCAPE {
            return Err(BindError::ReservedEscape);
        }
        if let Some(existing) = self.bindings.get(&key) {
            if *existing != target {
                return Err(BindError::Conflict {
                    existing: *existing,
                });
            }
        }
        self.bindings.retain(|_, v| *v != target);
        self.bindings.insert(key, target);
        Ok(())
    }

    /// Decide what pressing a hotkey should do. Does not mutate.
    ///
    /// Separated from [`Self::switch_to`] so the shell can show the user what
    /// will happen — "Windows is hibernated, this will take about 15 seconds" —
    /// before committing to it.
    #[must_use]
    pub fn plan_switch(&self, target: EnvironmentId) -> SwitchOutcome {
        if target == self.active {
            return SwitchOutcome::AlreadyActive;
        }
        let Some(env) = self.environments.get(&target) else {
            return SwitchOutcome::NotInstalled {
                user_must_supply_media: !target.is_distributable(),
            };
        };

        // Health is checked before state: a compromised environment that happens
        // to be running must not switch silently just because switching would be
        // fast.
        if !env.health.switchable_without_warning() {
            return SwitchOutcome::NeedsRecovery { health: env.health };
        }

        match env.state {
            EnvState::Running => SwitchOutcome::Presented,
            EnvState::Starting => SwitchOutcome::Starting,
            EnvState::Suspended => SwitchOutcome::Resuming,
            EnvState::Hibernated => SwitchOutcome::Restoring,
            EnvState::Stopped => SwitchOutcome::Starting,
            EnvState::Failed => SwitchOutcome::Failed,
            EnvState::NotInstalled => SwitchOutcome::NotInstalled {
                user_must_supply_media: !target.is_distributable(),
            },
        }
    }

    /// Perform a switch.
    ///
    /// Returns what happened. The display only changes when the outcome's
    /// [`SwitchOutcome::will_switch`] is true.
    pub fn switch_to(&mut self, target: EnvironmentId) -> SwitchOutcome {
        let outcome = self.plan_switch(target);
        if outcome.will_switch() {
            self.active = target;
        }
        outcome
    }

    /// Change an environment's lifecycle state.
    ///
    /// # Errors
    /// Returns the current state unchanged when the transition is not permitted,
    /// or when the caller tried to take down the host.
    pub fn transition(&mut self, id: EnvironmentId, next: EnvState) -> Result<(), TransitionError> {
        // The invariant this whole crate exists to protect. Checked before the
        // environment is even looked up, so no future refactor of the lookup can
        // route around it.
        if id.is_host() && !matches!(next, EnvState::Running) {
            return Err(TransitionError::HostCannotLeaveRunning { attempted: next });
        }

        let Some(env) = self.environments.get_mut(&id) else {
            return Err(TransitionError::Unknown);
        };
        if !env.state.can_transition_to(next) {
            return Err(TransitionError::Illegal {
                from: env.state,
                to: next,
            });
        }
        env.state = next;

        // If the environment that was on screen stopped being able to draw,
        // fall back to the host rather than presenting a dead surface.
        if self.active == id && !matches!(next, EnvState::Running | EnvState::Starting) {
            self.active = EnvironmentId::Litric;
        }
        Ok(())
    }

    /// Record observed health for an environment.
    ///
    /// # Errors
    /// [`TransitionError::Unknown`] if the environment is not known.
    pub fn set_health(&mut self, id: EnvironmentId, health: Health) -> Result<(), TransitionError> {
        let Some(env) = self.environments.get_mut(&id) else {
            return Err(TransitionError::Unknown);
        };
        env.set_health(health);
        // A compromised or corrupted environment must not stay on screen
        // silently; the shell needs to surface it, and the safe place to be
        // while that happens is the host.
        if self.active == id && !health.switchable_without_warning() {
            self.active = EnvironmentId::Litric;
        }
        Ok(())
    }

    /// Set a guest's resource budget.
    ///
    /// # Errors
    /// [`BudgetError`] when the request would breach the host reservation or
    /// oversubscribe the machine.
    pub fn set_budget(&mut self, id: EnvironmentId, budget: Budget) -> Result<(), BudgetError> {
        if id.is_host() {
            return Err(BudgetError::HostBudgetIsNotSettable);
        }

        // Sum every other guest's committed resources, then check that this
        // request still leaves the host its reservation.
        let (mut other_cpus, mut other_mem) = (0u32, 0u32);
        for (other_id, env) in &self.environments {
            if *other_id == id || other_id.is_host() {
                continue;
            }
            if env.state.holds_memory() {
                other_mem = other_mem.saturating_add(env.budget.memory_mb);
            }
            if env.state.consumes_cpu() {
                other_cpus = other_cpus.saturating_add(env.budget.cpus);
            }
        }

        let cpu_ceiling = self.total_cpus.saturating_sub(self.reservation.cpus);
        let mem_ceiling = self
            .total_memory_mb
            .saturating_sub(self.reservation.memory_mb);

        if other_cpus.saturating_add(budget.cpus) > cpu_ceiling {
            return Err(BudgetError::WouldStarveHost {
                resource: "cpus",
                requested: budget.cpus,
                available: cpu_ceiling.saturating_sub(other_cpus),
            });
        }
        if other_mem.saturating_add(budget.memory_mb) > mem_ceiling {
            return Err(BudgetError::WouldStarveHost {
                resource: "memory_mb",
                requested: budget.memory_mb,
                available: mem_ceiling.saturating_sub(other_mem),
            });
        }

        let Some(env) = self.environments.get_mut(&id) else {
            return Err(BudgetError::Unknown);
        };
        env.budget = budget;
        Ok(())
    }

    /// Memory currently committed by environments that hold it.
    #[must_use]
    pub fn committed_memory_mb(&self) -> u32 {
        self.environments
            .values()
            .filter(|e| !e.id.is_host() && e.state.holds_memory())
            .fold(0u32, |acc, e| acc.saturating_add(e.budget.memory_mb))
    }
}

/// Why a lifecycle transition was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransitionError {
    /// The environment is not known.
    Unknown,
    /// The transition is not in the permitted set.
    Illegal {
        /// Current state.
        from: EnvState,
        /// Requested state.
        to: EnvState,
    },
    /// Something tried to suspend, hibernate or stop the host.
    HostCannotLeaveRunning {
        /// What was attempted.
        attempted: EnvState,
    },
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown environment"),
            Self::Illegal { from, to } => write!(f, "cannot go from {from} to {to}"),
            Self::HostCannotLeaveRunning { attempted } => write!(
                f,
                "the host cannot enter {attempted}: suspending the control plane \
                 would leave no way to switch back"
            ),
        }
    }
}

impl std::error::Error for TransitionError {}

/// Why a budget was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BudgetError {
    /// The environment is not known.
    Unknown,
    /// The host's budget is the machine minus the guests; it is not set.
    HostBudgetIsNotSettable,
    /// Granting it would breach the host reservation.
    WouldStarveHost {
        /// Which resource.
        resource: &'static str,
        /// What was asked for.
        requested: u32,
        /// What remains available to guests.
        available: u32,
    },
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown environment"),
            Self::HostBudgetIsNotSettable => {
                write!(f, "the host budget is whatever the guests do not hold")
            }
            Self::WouldStarveHost {
                resource,
                requested,
                available,
            } => write!(
                f,
                "requested {requested} {resource} but only {available} remain after \
                 the host reservation"
            ),
        }
    }
}

impl std::error::Error for BudgetError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 16 GB, 8-CPU reference machine.
    fn mgr() -> EnvironmentManager {
        EnvironmentManager::new(8, 16384)
    }

    fn installed(m: &mut EnvironmentManager, id: EnvironmentId, state: EnvState) {
        m.transition(id, EnvState::Stopped).expect("install");
        if state == EnvState::Stopped {
            return;
        }
        m.transition(id, EnvState::Starting).expect("start");
        m.transition(id, EnvState::Running).expect("run");
        if state != EnvState::Running {
            m.transition(id, state).expect("reach target state");
        }
    }

    // ---- the invariant ------------------------------------------------------

    #[test]
    fn the_host_can_never_be_suspended_hibernated_or_stopped() {
        // If this ever passes for any of these states, a guest can take the
        // machine down with it and LitricOS is not the trust anchor.
        let mut m = mgr();
        for bad in [
            EnvState::Suspended,
            EnvState::Hibernated,
            EnvState::Stopped,
            EnvState::Failed,
            EnvState::NotInstalled,
        ] {
            let r = m.transition(EnvironmentId::Litric, bad);
            assert!(
                matches!(r, Err(TransitionError::HostCannotLeaveRunning { .. })),
                "host must refuse {bad}, got {r:?}"
            );
            assert_eq!(
                m.get(EnvironmentId::Litric).map(Environment::state),
                Some(EnvState::Running),
                "a refused transition must not mutate the host"
            );
        }
    }

    #[test]
    fn the_host_budget_cannot_be_set_by_anyone() {
        let mut m = mgr();
        assert_eq!(
            m.set_budget(EnvironmentId::Litric, Budget::new(1, 128)),
            Err(BudgetError::HostBudgetIsNotSettable)
        );
    }

    #[test]
    fn a_guest_cannot_be_budgeted_into_starving_the_host() {
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Windows, EnvState::Running);

        // 8 CPUs total, 2 reserved -> 6 available to guests.
        assert!(m
            .set_budget(EnvironmentId::Windows, Budget::new(6, 8192))
            .is_ok());
        let r = m.set_budget(EnvironmentId::Windows, Budget::new(7, 8192));
        assert!(
            matches!(
                r,
                Err(BudgetError::WouldStarveHost {
                    resource: "cpus",
                    ..
                })
            ),
            "got {r:?}"
        );

        // 16384 MiB total, 512 reserved -> 15872 available.
        let r = m.set_budget(EnvironmentId::Windows, Budget::new(4, 16000));
        assert!(
            matches!(
                r,
                Err(BudgetError::WouldStarveHost {
                    resource: "memory_mb",
                    ..
                })
            ),
            "got {r:?}"
        );
    }

    #[test]
    fn two_guests_are_budgeted_against_each_other_not_independently() {
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Windows, EnvState::Running);
        installed(&mut m, EnvironmentId::Linux, EnvState::Running);

        m.set_budget(EnvironmentId::Windows, Budget::new(4, 8192))
            .expect("windows");
        // Linux may take what is left, not the full ceiling again.
        assert!(m
            .set_budget(EnvironmentId::Linux, Budget::new(2, 7000))
            .is_ok());
        let r = m.set_budget(EnvironmentId::Linux, Budget::new(2, 8192));
        assert!(
            matches!(r, Err(BudgetError::WouldStarveHost { .. })),
            "got {r:?}"
        );
    }

    #[test]
    fn a_hibernated_guest_frees_its_memory_for_another() {
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Windows, EnvState::Running);
        installed(&mut m, EnvironmentId::Linux, EnvState::Running);
        m.set_budget(EnvironmentId::Windows, Budget::new(4, 12000))
            .expect("windows");

        // With Windows resident there is not room for a large Linux.
        assert!(m
            .set_budget(EnvironmentId::Linux, Budget::new(2, 6000))
            .is_err());

        // Hibernating Windows releases its memory; suspend would not.
        m.transition(EnvironmentId::Windows, EnvState::Hibernated)
            .expect("hibernate");
        assert!(m
            .set_budget(EnvironmentId::Linux, Budget::new(2, 6000))
            .is_ok());
    }

    #[test]
    fn suspend_holds_memory_and_hibernate_releases_it() {
        // The distinction that makes suspend fast and hibernate freeing.
        assert!(EnvState::Suspended.holds_memory());
        assert!(!EnvState::Suspended.consumes_cpu());
        assert!(!EnvState::Hibernated.holds_memory());
        assert!(!EnvState::Hibernated.consumes_cpu());
        assert!(EnvState::Running.holds_memory());
        assert!(EnvState::Running.consumes_cpu());
    }

    // ---- switching ----------------------------------------------------------

    #[test]
    fn switching_reports_the_mechanism_not_just_success() {
        let mut m = mgr();

        // Not installed: no switch, and the caller is told the user must supply
        // Windows media rather than being shown a spinner.
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows),
            SwitchOutcome::NotInstalled {
                user_must_supply_media: true
            }
        );
        assert_eq!(
            m.plan_switch(EnvironmentId::Linux),
            SwitchOutcome::NotInstalled {
                user_must_supply_media: false
            }
        );

        installed(&mut m, EnvironmentId::Windows, EnvState::Running);
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows),
            SwitchOutcome::Presented
        );
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows).latency_class(),
            LatencyClass::Instant
        );

        m.transition(EnvironmentId::Windows, EnvState::Suspended)
            .expect("suspend");
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows),
            SwitchOutcome::Resuming
        );
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows).latency_class(),
            LatencyClass::Seconds
        );

        m.transition(EnvironmentId::Windows, EnvState::Hibernated)
            .expect("hibernate");
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows),
            SwitchOutcome::Restoring
        );
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows).latency_class(),
            LatencyClass::TensOfSeconds
        );
    }

    #[test]
    fn planning_a_switch_does_not_perform_it() {
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Windows, EnvState::Running);
        let _ = m.plan_switch(EnvironmentId::Windows);
        assert_eq!(m.active(), EnvironmentId::Litric, "plan must not mutate");
        m.switch_to(EnvironmentId::Windows);
        assert_eq!(m.active(), EnvironmentId::Windows);
    }

    #[test]
    fn a_failed_switch_leaves_the_display_where_it_was() {
        let mut m = mgr();
        assert_eq!(m.active(), EnvironmentId::Litric);
        let out = m.switch_to(EnvironmentId::Windows); // not installed
        assert!(!out.will_switch());
        assert_eq!(m.active(), EnvironmentId::Litric);
    }

    #[test]
    fn a_compromised_environment_never_switches_silently() {
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Windows, EnvState::Running);
        m.switch_to(EnvironmentId::Windows);
        assert_eq!(m.active(), EnvironmentId::Windows);

        // Detection while it is on screen pulls the user back to the host.
        m.set_health(EnvironmentId::Windows, Health::Compromised)
            .expect("health");
        assert_eq!(
            m.active(),
            EnvironmentId::Litric,
            "a compromised environment must not stay on screen"
        );

        // And it cannot be switched back into without a warning path.
        assert_eq!(
            m.plan_switch(EnvironmentId::Windows),
            SwitchOutcome::NeedsRecovery {
                health: Health::Compromised
            }
        );
    }

    #[test]
    fn losing_the_active_environment_falls_back_to_the_host() {
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Linux, EnvState::Running);
        m.switch_to(EnvironmentId::Linux);
        assert_eq!(m.active(), EnvironmentId::Linux);

        m.transition(EnvironmentId::Linux, EnvState::Failed)
            .expect("fail");
        assert_eq!(
            m.active(),
            EnvironmentId::Litric,
            "must not keep presenting a dead surface"
        );
    }

    // ---- lifecycle ----------------------------------------------------------

    #[test]
    fn a_failed_environment_is_reset_never_resumed() {
        // Resuming state that is already known bad is how a fault becomes data
        // loss.
        let mut m = mgr();
        installed(&mut m, EnvironmentId::Windows, EnvState::Running);
        m.transition(EnvironmentId::Windows, EnvState::Failed)
            .expect("fail");

        for bad in [EnvState::Running, EnvState::Suspended, EnvState::Hibernated] {
            assert!(
                m.transition(EnvironmentId::Windows, bad).is_err(),
                "failed -> {bad} must be refused"
            );
        }
        assert!(m
            .transition(EnvironmentId::Windows, EnvState::Stopped)
            .is_ok());
    }

    #[test]
    fn illegal_transitions_are_refused_without_mutating() {
        let mut m = mgr();
        // NotInstalled cannot jump straight to Running.
        let r = m.transition(EnvironmentId::Linux, EnvState::Running);
        assert!(
            matches!(r, Err(TransitionError::Illegal { .. })),
            "got {r:?}"
        );
        assert_eq!(
            m.get(EnvironmentId::Linux).map(Environment::state),
            Some(EnvState::NotInstalled)
        );
    }

    // ---- hotkeys ------------------------------------------------------------

    #[test]
    fn the_default_bindings_are_win_1_2_3() {
        let m = mgr();
        assert_eq!(
            m.binding_for(Hotkey::new(Hotkey::SUPER, 0x0031)),
            Some(EnvironmentId::Windows)
        );
        assert_eq!(
            m.binding_for(Hotkey::new(Hotkey::SUPER, 0x0032)),
            Some(EnvironmentId::Litric)
        );
        assert_eq!(
            m.binding_for(Hotkey::new(Hotkey::SUPER, 0x0033)),
            Some(EnvironmentId::Linux)
        );
    }

    #[test]
    fn the_escape_binding_always_reaches_the_host_and_cannot_be_taken() {
        let mut m = mgr();
        assert_eq!(
            m.binding_for(EnvironmentManager::ESCAPE),
            Some(EnvironmentId::Litric)
        );
        // It cannot be rebound to a guest, because it is the way out when a
        // guest has grabbed input or hung.
        assert_eq!(
            m.rebind(EnvironmentManager::ESCAPE, EnvironmentId::Windows),
            Err(BindError::ReservedEscape)
        );
        assert_eq!(
            m.binding_for(EnvironmentManager::ESCAPE),
            Some(EnvironmentId::Litric)
        );
    }

    #[test]
    fn bindings_are_remappable_and_conflicts_are_refused() {
        let mut m = mgr();
        // Win+1 is already Windows; binding it to Linux must be refused rather
        // than silently stealing it.
        let r = m.rebind(Hotkey::new(Hotkey::SUPER, 0x0031), EnvironmentId::Linux);
        assert_eq!(
            r,
            Err(BindError::Conflict {
                existing: EnvironmentId::Windows
            })
        );

        // A free chord works, and replaces that target's old binding.
        assert!(m
            .rebind(
                Hotkey::new(Hotkey::CTRL | Hotkey::ALT, 0x0033),
                EnvironmentId::Linux
            )
            .is_ok());
        assert_eq!(
            m.binding_for(Hotkey::new(Hotkey::CTRL | Hotkey::ALT, 0x0033)),
            Some(EnvironmentId::Linux)
        );
        assert_eq!(
            m.binding_for(Hotkey::new(Hotkey::SUPER, 0x0033)),
            None,
            "the old binding must be released, not left as a duplicate"
        );
    }

    #[test]
    fn an_unmodified_key_cannot_be_a_switch_binding() {
        // The host would have to grab a bare keypress ahead of every guest and
        // every application, making that key unusable everywhere.
        let mut m = mgr();
        assert_eq!(
            m.rebind(Hotkey::new(0, 0x0041), EnvironmentId::Linux),
            Err(BindError::NoModifier)
        );
    }

    // ---- licensing ----------------------------------------------------------

    #[test]
    fn windows_is_not_distributable_and_the_model_says_so() {
        assert!(!EnvironmentId::Windows.is_distributable());
        assert!(EnvironmentId::Linux.is_distributable());
        assert!(EnvironmentId::Litric.is_distributable());
    }

    #[test]
    fn only_litric_is_the_host() {
        assert!(EnvironmentId::Litric.is_host());
        assert!(!EnvironmentId::Windows.is_host());
        assert!(!EnvironmentId::Linux.is_host());
        assert_eq!(
            EnvironmentId::all().iter().filter(|e| e.is_host()).count(),
            1,
            "exactly one host"
        );
    }

    #[test]
    fn tokens_are_unique_across_environments_and_states() {
        let mut seen = std::collections::BTreeSet::new();
        for e in EnvironmentId::all() {
            assert!(seen.insert(e.token()), "duplicate environment token");
        }
        let mut seen = std::collections::BTreeSet::new();
        for s in [
            EnvState::NotInstalled,
            EnvState::Stopped,
            EnvState::Starting,
            EnvState::Running,
            EnvState::Suspended,
            EnvState::Hibernated,
            EnvState::Failed,
        ] {
            assert!(seen.insert(s.token()), "duplicate state token");
        }
    }
}
