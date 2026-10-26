//! The desktop's side of the notification queue.
//!
//! `litric-notifyd` owns the inbox; this module is how the shell empties it.
//! Three decisions live here and nowhere else:
//!
//! * **Taking is not showing.** A note taken at startup goes into the centre and
//!   does not raise a banner. "This happened while you were away" is information,
//!   not an interruption: a desktop that barges in with six banners the moment it
//!   boots is worse than one that was never told.
//! * **The queue's timestamps are the truth.** A note posted twenty minutes ago
//!   says twenty minutes ago, and one whose lifetime ran out while the shell was
//!   down is not shown at all — it is dropped by `Queue::expire` before it is
//!   ever handed over.
//! * **A missing daemon is not an error.** Before the queue exists, or while it
//!   is restarting, the shell shows its own notifications and says nothing about
//!   it. The alternative is a desktop that starts with an error banner because a
//!   helper is not up yet, which trains people to ignore banners.

use std::path::{Path, PathBuf};

use litric_notifyd::proto::{Note, Priority as WirePriority};

use crate::shell::Priority;

/// Where the queue listens.
///
/// Under `/run` beside the other daemon sockets, in the directory the shell's own
/// unit already owns. Not configurable from the command line: a desktop that
/// connects to a different inbox depending on how it was started is a desktop
/// whose notifications can silently go missing.
pub const SOCKET: &str = "/run/litric/notify.sock";

/// The shell's priority for a queued one.
///
/// The two ladders are the same four levels, and the mapping is exhaustive so a
/// level added to the daemon fails to compile here rather than arriving as
/// `normal` — a critical notification shown as routine is the one mistake this
/// mapping must not make.
#[must_use]
pub const fn priority(wire: WirePriority) -> Priority {
    match wire {
        WirePriority::Low => Priority::Low,
        WirePriority::Normal => Priority::Normal,
        WirePriority::High => Priority::High,
        WirePriority::Critical => Priority::Critical,
    }
}

/// The default socket, for callers with no path in hand.
#[must_use]
pub fn default_socket() -> PathBuf {
    PathBuf::from(SOCKET)
}

/// Take everything the queue is holding.
///
/// `Ok(vec![])` means "nothing queued"; `Err` means "no queue answered", which a
/// caller is expected to ignore rather than report.
///
/// # Errors
/// The daemon is absent, refused, or answered with something that is not a
/// queue. The message is for a log line, not the user.
pub fn take(socket: &Path) -> Result<Vec<Note>, String> {
    litric_notifyd::client::take(socket).map(|taken| taken.notes)
}

/// Take from the default socket.
///
/// # Errors
/// As [`take`].
pub fn take_default() -> Result<Vec<Note>, String> {
    take(&default_socket())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_queued_priority_maps_to_a_distinct_desktop_one() {
        // A level added to the daemon does not compile here without a decision,
        // and no two levels collapse into one.
        let mut seen: Vec<Priority> = Vec::new();
        for wire in WirePriority::all() {
            let mapped = priority(*wire);
            assert!(
                !seen.contains(&mapped),
                "{wire:?} and another level both became {mapped:?}"
            );
            seen.push(mapped);
        }
        assert_eq!(seen.len(), WirePriority::all().len());
        // The two ends of the ladder are the ones that change behaviour: low is
        // quiet, critical is never auto-dismissed.
        assert_eq!(priority(WirePriority::Low), Priority::Low);
        assert_eq!(priority(WirePriority::Critical), Priority::Critical);
    }

    #[test]
    fn the_socket_is_where_the_unit_puts_it() {
        // The path is written twice — here and in the unit's ExecStart — so it is
        // asserted rather than remembered.
        assert_eq!(SOCKET, "/run/litric/notify.sock");
        assert_eq!(default_socket(), PathBuf::from(SOCKET));
    }

    #[test]
    fn a_missing_queue_is_an_error_the_caller_can_ignore() {
        let nowhere = PathBuf::from("/nonexistent/litric/notify.sock");
        assert!(take(&nowhere).is_err(), "silence is not an empty inbox");
    }
}
