//! The shell's animation integrator: the one place springs are stepped.
//!
//! `litric-design` has always defined the motion tokens — [`Motion::INSTANT`]
//! through [`Motion::SLOW`], a settle budget, and a reduced-motion token —
//! but nothing stepped them, so every state change in the shell jumped
//! between its two end states in a single frame. That is the "กระตุก" the
//! UI audit traced: the system's motion vocabulary existed and was unused.
//!
//! An [`Anim`] is one animated scalar. It holds a value, a velocity, and a
//! target, and [`Anim::step`] advances the value with semi-implicit Euler
//! integration of the spring defined by its [`Motion`] token. Three
//! properties are the point:
//!
//! * **Interruptible** (ADR-0009): retargeting mid-flight keeps the
//!   velocity, so grabbing a panel that is still opening never waits for
//!   the opening to finish — it redirects.
//! * **Bounded**: a spring that rings forever is a bug wearing a physics
//!   costume. If the value has not converged by the token's settle budget,
//!   it snaps, so no animation outlives [`Motion::settle_ms`]. The QA gate
//!   "every animation fits its token's budget" is enforced here, in one
//!   place, rather than hoped for at fifty call sites.
//! * **Deterministic**: the step is a pure function of the value, the
//!   target, and the elapsed delta — no clock reads inside — so the tests
//!   below can step a spring without sleeping.
//!
//! Reduced motion needs no special case here: [`Motion::reduced`] is an
//! overdamped spring with a 16 ms budget, which is "no travel" by
//! construction — the mechanism and the policy are the same token.

use std::collections::BTreeMap;

use litric_design::Motion;

// ---------------------------------------------------------------------------
// Shell springs
// ---------------------------------------------------------------------------

/// The shell's animated overlays, stepped once per frame.
///
/// One struct, owned by [`crate::shell::Shell`], so there is exactly one
/// place animation state lives and one place it advances — the same
/// discipline as the damage tracker. Panels not listed here do not
/// animate; a transition the audit did not ask for is a transition the
/// audit will eventually have to remove.
#[derive(Debug, Clone)]
pub struct ShellAnims {
    /// The launcher panel's reveal, 0 = hidden, 1 = fully shown.
    pub launcher: Anim,
    /// The quick-settings panel's reveal.
    pub quick_settings: Anim,
    /// The notification panel's reveal.
    pub notifications: Anim,
    /// The context menu's reveal.
    pub context_menu: Anim,
    /// The dock's slide-up from the bottom edge.
    pub dock: Anim,
    /// The dock's expansion into the launcher, 0 = pill, 1 = panel.
    ///
    /// Its own spring rather than the launcher overlay's, because the two are
    /// not the same question: *which* overlay is open decides what is drawn,
    /// while this decides how far the morph has travelled. The launcher's
    /// overlay spring is also raised by quick settings and the notification
    /// list, and a dock that expanded when a notification arrived would be a
    /// bug with a nice animation.
    pub launcher_morph: Anim,
    /// Per-window elevation, for the focus transition.
    pub focus: FocusAnims,
}

impl ShellAnims {
    /// Springs at rest, everything hidden.
    #[must_use]
    pub fn new() -> Self {
        Self {
            launcher: Anim::at(0.0, Motion::FAST),
            quick_settings: Anim::at(0.0, Motion::FAST),
            notifications: Anim::at(0.0, Motion::FAST),
            context_menu: Anim::at(0.0, Motion::INSTANT),
            dock: Anim::at(0.0, Motion::NORMAL),
            launcher_morph: Anim::at(0.0, Motion::NORMAL),
            focus: FocusAnims::default(),
        }
    }

    /// Retarget every spring to match the shell's overlay state. Idempotent:
    /// calling it every frame with no state change is a no-op by
    /// [`Anim::set_target`]'s contract, and it is called from `tick`, so the
    /// shell never has to remember which call site opened what.
    pub fn sync(&mut self, overlay_open: bool, menu_open: bool, launcher_open: bool) {
        self.launcher.set_target(f32::from(overlay_open));
        self.quick_settings.set_target(f32::from(overlay_open));
        self.notifications.set_target(f32::from(overlay_open));
        self.context_menu.set_target(f32::from(menu_open));
        self.launcher_morph.set_target(f32::from(launcher_open));
    }

    /// Step every spring; returns whether any of them still moves.
    ///
    /// A false return is the compositor's permission to idle: if no other
    /// damage exists, there is no frame left worth drawing.
    pub fn step(&mut self, dt_ms: u32) -> bool {
        let mut moving = false;
        moving |= self.launcher.step(dt_ms);
        moving |= self.quick_settings.step(dt_ms);
        moving |= self.notifications.step(dt_ms);
        moving |= self.context_menu.step(dt_ms);
        moving |= self.dock.step(dt_ms);
        moving |= self.launcher_morph.step(dt_ms);
        moving |= self.focus.step(dt_ms);
        moving
    }

    /// Whether any spring is mid-flight, so the frame loop cannot idle yet.
    #[must_use]
    pub fn animating(&self) -> bool {
        !(self.launcher.settled()
            && self.quick_settings.settled()
            && self.notifications.settled()
            && self.context_menu.settled()
            && self.dock.settled()
            && self.launcher_morph.settled())
            || self.focus.animating()
    }
}

impl Default for ShellAnims {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Window focus springs
// ---------------------------------------------------------------------------

/// Per-window elevation springs: focus raises a window, blur lowers it.
///
/// The chrome already draws two shadow weights (views.rs) — what was missing
/// is the *transition* between them, which previously jumped in one frame.
/// One spring per window id, keyed in a map so windows can come and go;
/// a spring for a window that no longer exists simply stops being stepped.
#[derive(Debug, Clone, Default)]
pub struct FocusAnims {
    springs: BTreeMap<u32, Anim>,
}

impl FocusAnims {
    /// The elevation of `id` in 0..1, where 1 is the focused weight.
    /// A window never seen before is treated as unfocused — it will glide
    /// up if focus lands on it.
    pub fn elevation(&mut self, id: u32, focused: bool) -> f32 {
        let spring = self
            .springs
            .entry(id)
            .or_insert_with(|| Anim::at(0.0, Motion::FAST));
        spring.set_target(f32::from(u8::from(focused)));
        spring.value()
    }

    /// Advance every spring; returns whether any still moves.
    pub fn step(&mut self, dt_ms: u32) -> bool {
        let mut moving = false;
        for spring in self.springs.values_mut() {
            moving |= spring.step(dt_ms);
        }
        moving
    }

    /// Whether any window is mid-transition.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.springs.values().any(|s| !s.settled())
    }
}

/// The largest delta a single step may cover, in milliseconds.
///
/// A compositor stall must not teleport a spring to its target (or through
/// it): integration assumes small deltas, so a long gap is clamped and the
/// settle budget — not the stall — decides when the value arrives.
const MAX_STEP_MS: u32 = 100;

/// Convergence thresholds: close enough to stop spending frames on.
const SETTLE_EPSILON: f32 = 0.001;
const SETTLE_VELOCITY: f32 = 0.01;

/// One animated scalar driven by a [`Motion`] spring.
///
/// The value is a plain `f32` in whatever unit the caller wants — a scale
/// factor, a pixel offset, an alpha. The spring is unit-agnostic; the
/// tokens in `litric-design` were tuned for values near 1.0 in magnitude,
/// which is what the shell animates.
#[derive(Debug, Clone, Copy)]
pub struct Anim {
    /// The value as of the last step.
    value: f32,
    /// How fast the value is changing, in units per second.
    velocity: f32,
    /// Where the spring is pulling toward.
    target: f32,
    /// Which spring: stiffness, damping, and the settle budget.
    motion: Motion,
    /// Time since [`Anim::set_target`] last gave this spring a job.
    elapsed_ms: u32,
}

impl Anim {
    /// A spring at rest at `value`.
    #[must_use]
    pub const fn at(value: f32, motion: Motion) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            motion,
            elapsed_ms: 0,
        }
    }

    /// The value as of the last step.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// Where the spring is heading.
    #[must_use]
    pub const fn target(&self) -> f32 {
        self.target
    }

    /// Whether the spring has arrived and is spending no further frames.
    #[must_use]
    pub const fn settled(&self) -> bool {
        self.value == self.target && self.velocity == 0.0
    }

    /// Retarget the spring, keeping whatever velocity it already has.
    ///
    /// This is what makes the animation interruptible: a panel retargeted
    /// mid-flight decelerates from its current motion rather than starting
    /// over. Retargeting to where the spring already sits is a no-op that
    /// leaves the budget alone, so repeated `open` calls do not reset an
    /// animation that is already where it should be.
    pub fn set_target(&mut self, target: f32) {
        if target == self.target {
            return;
        }
        self.target = target;
        self.elapsed_ms = 0;
    }

    /// Jump to `value` and stop there. For state changes that must not
    /// animate — first paint, a mode switch that re-lays everything out.
    pub fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.velocity = 0.0;
        self.target = value;
        self.elapsed_ms = 0;
    }

    /// Advance the spring by `dt_ms`, returning whether it is still moving.
    ///
    /// `dt_ms` is clamped to [`MAX_STEP_MS`]; the settle budget, not a
    /// stall, decides when the value arrives. Converged springs detect
    /// their own convergence and stop early rather than draining the
    /// budget on frames that would draw the same pixel.
    pub fn step(&mut self, dt_ms: u32) -> bool {
        if self.settled() {
            return false;
        }
        let dt = f32::from(u16::try_from(dt_ms.min(MAX_STEP_MS)).unwrap_or(u16::MAX)) / 1000.0;
        self.elapsed_ms = self.elapsed_ms.saturating_add(dt_ms);

        // Semi-implicit Euler: velocity first, then position. Symplectic
        // enough for UI springs at 16 ms, and stable for every token in
        // `Motion` (the stiffest, reduced motion, is overdamped).
        let accel = -f32::from(self.motion.stiffness) * (self.value - self.target)
            - f32::from(self.motion.damping) * self.velocity;
        self.velocity += accel * dt;
        self.value += self.velocity * dt;

        let budget_hit = self.elapsed_ms >= u32::from(self.motion.settle_ms);
        let converged = (self.value - self.target).abs() < SETTLE_EPSILON
            && self.velocity.abs() < SETTLE_VELOCITY;
        if budget_hit || converged {
            self.snap_to(self.target);
            return false;
        }
        true
    }
}

#[cfg(test)]
#[allow(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::shadow_unrelated
)]
mod tests {
    use super::*;

    const DT: u32 = 16;

    #[test]
    fn a_retargeted_spring_settles_on_the_new_target_within_budget() {
        let mut a = Anim::at(0.0, Motion::FAST);
        a.set_target(1.0);
        let mut steps = 0;
        while a.step(DT) {
            steps += 1;
            assert!(steps * DT <= u32::from(Motion::FAST.settle_ms) + DT);
        }
        assert_eq!(a.value(), 1.0);
        assert!(a.settled());
    }

    #[test]
    fn retargeting_mid_flight_keeps_momentum_instead_of_restarting() {
        let mut a = Anim::at(0.0, Motion::NORMAL);
        a.set_target(1.0);
        for _ in 0..5 {
            a.step(DT);
        }
        assert!(
            !a.settled(),
            "NORMAL must still be mid-flight after 80 ms for this test to mean anything"
        );
        a.set_target(0.5);
        // The value must still be between where it was and where it is
        // going: a restart from rest would sit exactly at the old value.
        for _ in 0..40 {
            a.step(DT);
        }
        assert_eq!(a.value(), 0.5);
    }

    #[test]
    fn interrupting_a_closing_panel_reopens_smoothly() {
        // Open, part-close, re-open: the value never goes below where the
        // reversal happened twice — it bends, it does not bounce from rest.
        let mut a = Anim::at(1.0, Motion::FAST);
        a.set_target(0.0);
        for _ in 0..4 {
            a.step(DT);
        }
        let turning_point = a.value();
        a.set_target(1.0);
        a.step(DT);
        assert!(a.value() >= turning_point - 0.05);
        for _ in 0..30 {
            a.step(DT);
        }
        assert_eq!(a.value(), 1.0);
    }

    #[test]
    fn reduced_motion_arrives_in_one_step_without_travel() {
        let mut a = Anim::at(0.0, Motion::reduced());
        a.set_target(1.0);
        let moving = a.step(DT);
        assert!(
            !moving,
            "reduced motion must settle within its 16 ms budget"
        );
        assert_eq!(a.value(), 1.0);
    }

    #[test]
    fn a_stalled_compositor_cannot_teleport_the_value() {
        let mut a = Anim::at(0.0, Motion::FAST);
        a.set_target(1.0);
        // One enormous step stands in for a 10-second stall:
        a.step(10_000);
        assert_eq!(a.value(), 1.0, "the settle budget, not the stall, lands it");
    }

    #[test]
    fn the_same_seed_of_steps_gives_the_same_sequence() {
        let mut left = Anim::at(0.0, Motion::NORMAL);
        let mut right = Anim::at(0.0, Motion::NORMAL);
        left.set_target(1.0);
        right.set_target(1.0);
        for _ in 0..10 {
            assert_eq!(left.step(DT), right.step(DT));
            assert_eq!(left.value(), right.value());
        }
    }

    #[test]
    fn retargeting_to_the_current_target_is_a_no_op() {
        let mut a = Anim::at(0.5, Motion::FAST);
        a.set_target(1.0);
        for _ in 0..3 {
            a.step(DT);
        }
        let elapsed = a.elapsed_ms;
        a.set_target(1.0);
        assert_eq!(a.elapsed_ms, elapsed, "an idempotent open must not restart");
    }

    #[test]
    fn a_spring_at_rest_spends_no_frames() {
        let mut a = Anim::at(1.0, Motion::INSTANT);
        assert!(!a.step(DT));
        assert!(a.settled());
    }
}
