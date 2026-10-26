//! LitricOS notification queue.
//!
//! # Why this daemon exists
//!
//! The shell has a notification centre, and it was honest about its limit: the
//! queue lived in the compositor's memory, so a notification raised while the
//! shell was restarting — or before it started at all — was lost, and the centre
//! could not say it had ever existed.
//!
//! This daemon is the record. A program posts a notification here; the desktop
//! takes it, and what it took is gone from the queue. Between those two events
//! the notification survives a shell restart, a crash and a reboot, because the
//! queue is a file written atomically rather than a list in a process.
//!
//! # What it deliberately does not decide
//!
//! **Whether to disturb the user.** The queue has no Do Not Disturb flag. The
//! shell owns that switch, and it decides at display time — so there is one
//! answer to "should this interrupt?" instead of two that can disagree. A
//! notification taken while the desktop is quiet still appears in the centre.
//!
//! **How long it stays on screen.** A `ttl_ms` in a post is the sender's
//! request, converted to an absolute expiry here so that a reboot cannot
//! resurrect a five-second toast. It is not a display duration: the shell draws
//! the banner for as long as its own motion and priority rules say.
//!
//! **What it looks like.** No rendering, no icons, no colours — text out, text
//! in. A daemon that drew would be a second implementation of the design system,
//! reachable by anyone who can open a socket.
//!
//! # The failure that matters
//!
//! This queue exists to keep one promise: *a notification raised while the
//! desktop is down still arrives*. A write that fails breaks that promise
//! silently, which is the one thing the project refuses, so every mutating
//! request reports whether the record reached the disk (`persisted=0|1`) and the
//! daemon logs the reason to the journal. The notification is still queued in
//! memory: a full disk must not lose the message, only the promise that it will
//! outlive a reboot.
//!
//! # Layer position
//!
//! **Layer 4 (Runtime)**, beside `litric-sysd` and `litric-envd`. It is a
//! library as well as a binary because the client half of the protocol is what
//! the shell links to take its inbox, and two implementations of one wire format
//! is how the browser's frame header came to disagree with itself.

#![forbid(unsafe_code)]

pub mod proto;
pub mod queue;

#[cfg(unix)]
pub mod client;
#[cfg(unix)]
pub mod server;
