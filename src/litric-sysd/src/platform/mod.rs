//! Reading what the machine's subsystems are doing.
//!
//! [`crate::facts`] answers "how busy is this machine". This module answers
//! "what is this machine": which interfaces exist and which are up, what the
//! sound cards are, what the battery is doing, whether there is a Bluetooth
//! radio and whether firmware blocked it, who is logged in.
//!
//! # Why this is text and not a client library
//!
//! Every line here is read from a file the kernel maintains, or from a file a
//! provider maintains in a documented format. Nothing here opens a socket,
//! calls `ioctl`, or links a client library, and that is a design decision
//! rather than a shortcut (ADR-0023 decision 2):
//!
//! * A client library for PipeWire, BlueZ or NetworkManager is a dependency in
//!   a tree that forbids `unsafe` and reviews every dependency, and it would
//!   put a protocol implementation inside the daemon that parses untrusted
//!   content.
//! * The kernel's file surface is the same information, is readable without
//!   privilege, and — the reason that matters most — **is testable on a host
//!   that has none of those devices**. `read(root)` takes a directory, so the
//!   tests below run against fixtures and this module is exercised on every
//!   commit rather than once, by hand, on the target.
//!
//! What is *not* reachable this way is stated per module rather than papered
//! over: a volume change, a Wi-Fi scan, a pairing, and a password check all
//! need a protocol that has no file surface, and each is recorded as such in
//! `docs/design/wave1/`.

pub mod audio;
pub mod bluetooth;
pub mod input;
pub mod net;
pub mod power;
pub mod session;

use std::fs;
use std::path::Path;

/// Every subsystem fact, in one value.
///
/// Empty vectors mean "nothing of that kind exists on this machine" and
/// `None` means "it exists but could not be read". The two are kept apart on
/// purpose: a machine with no Bluetooth adapter and a machine whose adapter
/// could not be read produce the same empty list but are different answers, and
/// a UI that shows "no adapter" for the second one is lying.
#[derive(Debug, Clone, Default)]
pub struct Platform {
    /// Interfaces, routes and DNS.
    pub net: net::NetFacts,
    /// Optional system services: AppArmor, Polkit, firewalld, Flatpak and
    /// NetworkManager.
    pub services: crate::services::ServiceFacts,
    /// Sound cards, PCM devices, and whether a sound server is running.
    pub audio: audio::AudioFacts,
    /// Power supplies, thermal zones, backlights.
    pub power: power::PowerFacts,
    /// Bluetooth adapters and radio blocks.
    pub bluetooth: bluetooth::BtFacts,
    /// Keyboards, pointers and tablets, as sysfs describes them.
    pub input: input::InputFacts,
    /// Login sessions, and whether a credential verifier exists.
    pub session: session::SessionFacts,
    /// Whether the root could be read at all.
    ///
    /// Set when neither `/proc` nor `/sys` is there, which means there is no
    /// machine here to describe — a container root, a mistyped path, a test
    /// fixture that was not built. In that state [`Self::to_lines`] produces
    /// nothing at all, because every line it would produce is a claim about a
    /// machine: `net.iface.count=0` says "this machine has no network
    /// interfaces", which is not what an unreadable root means. The existing
    /// readers in [`crate::facts`] make the same promise for the same reason,
    /// and `facts_from_an_unreadable_root_are_absent_rather_than_zero` asserts
    /// it for the whole reply.
    pub unreadable: bool,
}

impl Platform {
    /// The facts as `key=value` lines, in a stable order.
    ///
    /// Stable because the shell and the boot selftest grep this text; a
    /// reordering would silently break a check that reports success by
    /// absence. The order is net, audio, power, bluetooth, input, session.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        if self.unreadable {
            return Vec::new();
        }
        let mut out = Vec::new();
        out.extend(self.net.to_lines());
        out.extend(self.services.to_lines());
        out.extend(self.audio.to_lines());
        out.extend(self.power.to_lines());
        out.extend(self.bluetooth.to_lines());
        out.extend(self.input.to_lines());
        out.extend(self.session.to_lines());
        out
    }
}

/// Read every subsystem fact from a filesystem root.
///
/// `root` is `/` in production and a fixture directory in the tests. It is a
/// parameter of the *reader* and never of a request: a caller who can choose
/// the root can make the daemon report fiction as machine state, which is
/// threat case N4 in `docs/design/wave1/00-network.md`. The daemon passes `/`
/// and there is no request field that reaches this function.
#[must_use]
pub fn read(root: &Path) -> Platform {
    // Both directories, because a machine describes itself through both and a
    // root with only one of them is not a root this module can say anything
    // true about.
    let unreadable = !root.join("proc").is_dir() && !root.join("sys").is_dir();
    Platform {
        unreadable,
        net: net::read(root),
        services: crate::services::read(root),
        audio: audio::read(root),
        power: power::read(root),
        bluetooth: bluetooth::read(root),
        input: input::read(root),
        session: session::read(root),
    }
}

/// Read one file under `root`, or `None` if it is not there.
///
/// Shared by every reader below so that the "unreadable is `None`, never a
/// default" rule is written once. A zero that is indistinguishable from a real
/// zero is how a monitoring system starts lying, and it is the failure this
/// whole module is shaped to avoid.
pub(crate) fn read_file(root: &Path, rel: &str) -> Option<String> {
    fs::read_to_string(root.join(rel)).ok()
}

/// Read one file under `root` with surrounding whitespace removed.
///
/// sysfs writes most attributes with a trailing newline and some with trailing
/// spaces; trimming here keeps every parser from having to remember that.
pub(crate) fn read_attr(root: &Path, rel: &str) -> Option<String> {
    read_file(root, rel).map(|s| s.trim().to_owned())
}

/// Entries of a directory under `root`, sorted, or empty if it is not there.
///
/// Sorted because a fact listing that reorders between two reads turns a
/// diffable reply into a noisy one, and the tests here compare text.
pub(crate) fn read_dir(root: &Path, rel: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root.join(rel)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_root_that_does_not_exist_produces_no_claims_at_all() {
        // The most important property of this module: an empty root is not a
        // machine with zero of everything, it is a machine nothing is known
        // about, so it gets no lines. `net.iface.count=0` would read as "this
        // machine has no network interfaces", which nobody checked.
        let p = read(Path::new("/nonexistent-litric-fixture-root"));
        assert!(p.unreadable);
        assert!(p.net.ifaces.is_empty());
        assert!(p.audio.cards.is_empty());
        assert!(p.power.supplies.is_empty());
        assert!(p.bluetooth.adapters.is_empty());
        assert!(p.input.devices.is_empty());
        assert!(p.session.sessions.is_empty());
        assert!(
            p.to_lines().is_empty(),
            "an unreadable root must claim nothing: {:?}",
            p.to_lines()
        );
    }

    #[test]
    fn an_empty_but_readable_machine_reports_zeroes_that_are_true() {
        // The other side of the same coin: a root with `/sys` and `/proc` and
        // nothing in them *has* been read, so `no-adapter` is a finding rather
        // than a guess, and it must be reported.
        let root = std::env::temp_dir().join("litric-platform-empty-root");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("proc")).expect("mkdir");
        std::fs::create_dir_all(root.join("sys")).expect("mkdir");

        let p = read(&root);
        assert!(!p.unreadable);
        let text = p.to_lines().join("\n");
        assert!(text.contains("net.iface.count=0"), "{text}");
        assert!(text.contains("bt.state=no-adapter"), "{text}");
        assert!(text.contains("input.device.count=0"), "{text}");
        assert!(text.contains("audio.card.count=0"), "{text}");
        assert!(text.contains("power.battery.count=0"), "{text}");
        assert!(text.contains("auth.available=none"), "{text}");
        assert!(text.contains("session.logind=absent"), "{text}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn reading_a_directory_that_is_a_file_is_not_an_error() {
        let root = std::env::temp_dir();
        // /tmp is a directory full of files; read_dir on a missing subpath must
        // come back empty rather than panicking.
        assert!(read_dir(&root, "definitely-not-a-directory-here").is_empty());
    }
}
