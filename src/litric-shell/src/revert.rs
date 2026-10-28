//! A change that undoes itself unless a person says it was right.
//!
//! Some settings can take the screen away from the person who changed them. An
//! interface scale that is too large puts the control that would undo it off
//! the edge of the display; a resolution the monitor cannot lock leaves a black
//! panel. The platform answer is not a warning dialog before the change — it is
//! a change that is *temporary until confirmed*: apply it, start a clock, and
//! put the old value back if nobody says otherwise. A user who can still see
//! the screen says "keep"; a user who cannot does nothing, and the desktop
//! returns on its own.
//!
//! # What this type is, and is not
//!
//! It is the clock and the two values, and nothing else. It has no reference to
//! the shell, no drawing, and no clock of its own: the caller feeds it elapsed
//! milliseconds, which is what makes it testable without a display and what
//! keeps it honest about who owns the decision.
//!
//! # Why keep leaves the value in place
//!
//! [`RevertWindow::keep`] does not return the value: the value was already
//! applied, so "keep" is the absence of an undo. Returning it would invite a
//! caller to re-apply it and call that confirmation.
//!
//! # Why the countdown is not reset on activity
//!
//! A person who is reading the screen is not necessarily a person who can read
//! it. Extending the clock whenever the pointer moves would keep a broken
//! display up for exactly as long as the user kept trying to fix it, which is
//! the failure this exists to prevent.

/// What a tick decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Still inside the window: nothing to undo yet.
    Pending,
    /// The window closed and the caller must restore the previous value.
    Expired,
    /// A person kept the change; the clock no longer matters.
    Kept,
}

/// One applied change, and the value to put back if it is not kept.
#[derive(Debug, Clone, PartialEq)]
pub struct RevertWindow<T> {
    previous: T,
    applied: T,
    label: String,
    remaining_ms: u64,
    total_ms: u64,
    kept: bool,
}

impl<T: Copy> RevertWindow<T> {
    /// Arm a window for a change that has just been applied.
    ///
    /// A window of zero milliseconds is treated as no time at all rather than
    /// as an infinite one: the safe reading of "no time to confirm" is that the
    /// change was never confirmed.
    #[must_use]
    pub fn arm(previous: T, applied: T, label: impl Into<String>, window_ms: u64) -> Self {
        Self {
            previous,
            applied,
            label: label.into(),
            remaining_ms: window_ms,
            total_ms: window_ms,
            kept: false,
        }
    }

    /// The value to restore if the window expires.
    #[must_use]
    pub const fn previous(&self) -> T {
        self.previous
    }

    /// The value that was applied.
    #[must_use]
    pub const fn applied(&self) -> T {
        self.applied
    }

    /// What changed, in the user's words — "UI scale", not "display.scale".
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Milliseconds left, saturating at zero once the window has closed.
    #[must_use]
    pub const fn remaining_ms(&self) -> u64 {
        self.remaining_ms
    }

    /// Whole seconds left, rounded up, for a countdown that never reads "0s"
    /// while the change is still in effect.
    #[must_use]
    pub const fn seconds_left(&self) -> u64 {
        self.remaining_ms.div_ceil(1000)
    }

    /// The window this change was given, for a progress indicator that does not
    /// have to remember the original number.
    #[must_use]
    pub const fn total_ms(&self) -> u64 {
        self.total_ms
    }

    /// Whether a person has kept this change.
    #[must_use]
    pub const fn kept(&self) -> bool {
        self.kept
    }

    /// Advance the clock by `dt_ms` and say what the caller should do.
    ///
    /// A kept window is inert: calling `tick` after [`Self::keep`] reports
    /// `Kept` forever, which is what makes an accidental later tick harmless
    /// instead of an undo of a choice the user made.
    pub fn tick(&mut self, dt_ms: u64) -> Verdict {
        if self.kept {
            return Verdict::Kept;
        }
        self.remaining_ms = self.remaining_ms.saturating_sub(dt_ms);
        if self.remaining_ms == 0 {
            Verdict::Expired
        } else {
            Verdict::Pending
        }
    }

    /// Keep the change. Idempotent.
    pub fn keep(&mut self) {
        self.kept = true;
        self.remaining_ms = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_runs_out_exactly_once_at_the_end_of_the_window() {
        let mut window = RevertWindow::arm(1.0_f32, 1.5_f32, "UI scale", 15_000);
        assert_eq!(window.seconds_left(), 15);
        assert_eq!(window.tick(7_000), Verdict::Pending);
        assert_eq!(window.seconds_left(), 8);
        assert_eq!(window.tick(7_999), Verdict::Pending);
        assert_eq!(window.remaining_ms(), 1);
        assert_eq!(window.tick(1), Verdict::Expired);
        assert_eq!(window.remaining_ms(), 0);
        assert_eq!(window.previous(), 1.0);
    }

    #[test]
    fn a_kept_change_never_expires() {
        let mut window = RevertWindow::arm(1.0_f32, 1.25_f32, "UI scale", 15_000);
        window.keep();
        assert!(window.kept());
        assert_eq!(window.tick(60_000), Verdict::Kept);
        assert_eq!(window.applied(), 1.25);
    }

    #[test]
    fn a_zero_window_expires_on_the_first_tick_and_a_whole_one_does_not() {
        let mut none = RevertWindow::arm(1.0_f32, 1.5_f32, "UI scale", 0);
        assert_eq!(none.tick(0), Verdict::Expired);
        let mut full = RevertWindow::arm(1.0_f32, 1.5_f32, "UI scale", 1_000);
        assert_eq!(full.tick(999), Verdict::Pending);
        assert_eq!(full.seconds_left(), 1);
    }
}
