//! Session facts, and the honest state of authentication.
//!
//! # Sessions are read, not invented
//!
//! `systemd-logind` already models sessions, seats and users, and writes each
//! one as a small text record in `/run/systemd/sessions/`. This module reads
//! that record. Inventing a parallel session model — the thing
//! `docs/design/wave1/04-authentication.md` §2.2 argues against — would give
//! the shell a second opinion about who is logged in, and only one of the two
//! can be right.
//!
//! # Authentication is reported, never invented
//!
//! Two facts are reported, and they answer two different questions:
//!
//! * `auth.available` — **can a password be checked on this machine?**
//!   `unix_chkpwd` when the distribution's helper is on the image, `none`
//!   otherwise. The helper is found by looking for it, not by a build-time
//!   constant that can drift from the image.
//! * `auth.lock` — **is locking refused?** Present only while it is, which is
//!   exactly when nothing on the machine can verify a credential. Absent means
//!   the shell holds a lock surface that can be unlocked.
//!
//! A lock that accepts anything is worse than no lock; that sentence is already
//! in `docs/SYSTEM-SCOPE.md` and this module is where it is enforced rather than
//! repeated. [`Verifier::can_lock`] follows [`Verifier::can_verify`] since
//! 2026-10-12 — the shell's lock surface landed that day — so the refusal now
//! covers the case it was always about: an image with no way to check a
//! password must not offer a screen that appears to be locked. The test at the
//! bottom of this file asserts the pairing in both directions, so neither half
//! can ship without the other.

use std::collections::BTreeMap;
use std::path::Path;

use super::{read_dir, read_file};

/// Who can verify a credential on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verifier {
    /// Nothing can: there is no authentication subsystem in this image.
    None,
    /// `pam_unix`'s own helper, `/usr/bin/unix_chkpwd`, present and setuid root.
    ///
    /// This is what `litric-sysd` runs for `AUTH VERIFY` (ADR-0023 decision 2):
    /// the binary the distribution's PAM stack already executes, spoken to the
    /// way it is meant to be spoken to — the password on standard input, the
    /// answer in the exit status — so that no password database, hash format or
    /// `libpam` FFI exists in this tree.
    UnixChkpwd,
    /// A helper of ours holding the only `dlopen("libpam.so")` in the system,
    /// which does not exist. Named here so the fact has a spelling before the
    /// implementation has a body; nothing produces it today.
    PamHelper,
}

impl Verifier {
    /// The word used in the reply.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::UnixChkpwd => "unix-chkpwd",
            Self::PamHelper => "pam-helper",
        }
    }

    /// Parse the word.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        match text.trim() {
            "unix-chkpwd" => Self::UnixChkpwd,
            "pam-helper" => Self::PamHelper,
            _ => Self::None,
        }
    }

    /// Whether a credential can be verified at all.
    ///
    /// `false` means a verification request cannot succeed and a lock action
    /// must be **refused**, not performed.
    #[must_use]
    pub const fn can_verify(self) -> bool {
        matches!(self, Self::UnixChkpwd | Self::PamHelper)
    }

    /// Whether the lock action may proceed.
    ///
    /// Locking requires a way back in, so this is exactly
    /// [`Verifier::can_verify`]: a machine that cannot check a password must not
    /// present a screen that looks locked, because the only exit from such a
    /// screen would be a lie. `auth.lock=refused` is therefore in the reply when
    /// this is false, and absent when it is true — the absence is the claim that
    /// `AUTH VERIFY` on this machine can open the lock again.
    #[must_use]
    pub const fn can_lock(self) -> bool {
        self.can_verify()
    }

    /// The sentence a user interface shows when locking is refused.
    ///
    /// Reached through [`Verifier::None`] in every build that can produce it.
    /// The other two arms exist because the function has to be total, and they
    /// name what would be wrong about refusing on a machine that can verify: the
    /// refusal would be the shell's, not the image's.
    #[must_use]
    pub const fn refusal(self) -> &'static str {
        match self {
            Self::UnixChkpwd | Self::PamHelper => {
                "This machine can check a password, so a refusal here would come from \
                 the desktop and not from the image. Locking is refused rather than \
                 shown working."
            }
            Self::None => {
                "This image cannot verify a password, so it will not pretend to lock \
                 the screen. Locking is refused rather than shown working."
            }
        }
    }
}

/// One logind session record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    /// The record's file name, which is logind's session id.
    pub id: String,
    /// User name.
    pub user: Option<String>,
    /// Numeric user id.
    pub uid: Option<u32>,
    /// `seat0`, `seat1`, … or absent for a session with no seat.
    pub seat: Option<String>,
    /// `tty`, `x11`, `wayland`, `unspecified` — logind's own vocabulary.
    pub kind: Option<String>,
    /// `online`, `active`, `closing` — normalised from logind's `STATE=` and
    /// `ACTIVE=` pair, because logind splits "is this the foreground session"
    /// from "is this session running" and a UI wants one answer.
    pub state: String,
    /// The controlling terminal, if any.
    pub tty: Option<String>,
    /// The display, e.g. `:0` or `serial`.
    pub display: Option<String>,
    /// The session leader's pid.
    pub leader: Option<u32>,
}

/// Everything this module reports about sessions and authentication.
#[derive(Debug, Clone)]
pub struct SessionFacts {
    /// Sessions, sorted by id.
    pub sessions: Vec<Session>,
    /// Whether `/run/systemd/sessions` was there to read at all.
    pub logind_present: bool,
    /// The credential verifier, defaulting to [`Verifier::None`].
    pub verifier: Verifier,
}

impl Default for SessionFacts {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            logind_present: false,
            // Default is `None`: a build that has not been told about a
            // verifier has not got one. Fail closed, at the type level.
            verifier: Verifier::None,
        }
    }
}

impl SessionFacts {
    /// The active session, if there is exactly one.
    ///
    /// Two active sessions on one seat is a state logind does not produce, and
    /// picking the first would hide a machine that is behaving oddly. `None`
    /// when there is no active session.
    #[must_use]
    pub fn active(&self) -> Option<&Session> {
        let mut it = self.sessions.iter().filter(|s| s.state == "active");
        let first = it.next()?;
        if it.next().is_some() {
            return None;
        }
        Some(first)
    }

    /// The facts as `key=value` lines.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = vec![
            format!("session.count={}", self.sessions.len()),
            format!(
                "session.logind={}",
                if self.logind_present {
                    "present"
                } else {
                    "absent"
                }
            ),
            format!("auth.available={}", self.verifier.token()),
        ];
        if !self.verifier.can_lock() {
            out.push("auth.lock=refused".to_owned());
        }
        for s in &self.sessions {
            out.push(format!(
                "session.id {} user={} uid={} seat={} type={} state={} tty={} display={} leader={}",
                s.id,
                s.user.as_deref().unwrap_or("-"),
                opt_num(s.uid),
                s.seat.as_deref().unwrap_or("-"),
                s.kind.as_deref().unwrap_or("unspecified"),
                s.state,
                s.tty.as_deref().unwrap_or("-"),
                s.display.as_deref().unwrap_or("-"),
                opt_num(s.leader)
            ));
        }
        out
    }
}

/// A number as digits, or `-` when unknown. Same rule as everywhere else in
/// this module: an absent fact is not a zero.
fn opt_num<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map_or_else(|| "-".to_owned(), |v| v.to_string())
}

/// Read everything from a filesystem root.
#[must_use]
pub fn read(root: &Path) -> SessionFacts {
    let dir = "run/systemd/sessions";
    let logind_present = root.join(dir).is_dir();
    let mut sessions = Vec::new();
    if logind_present {
        for name in read_dir(root, dir) {
            let Some(text) = read_file(root, &format!("{dir}/{name}")) else {
                continue;
            };
            sessions.push(parse_session_record(&name, &text));
        }
    }
    SessionFacts {
        sessions,
        logind_present,
        // Nothing on this image can verify a credential. When the helper
        // exists it will report itself the same way `bluetoothd` is reported —
        // by being found on the filesystem — rather than by a build-time
        // constant that can drift from the image.
        // The distribution's helper first, because that is what this build
        // actually runs; ours (which does not exist) is the fallback name so a
        // future change that adds it does not have to touch this function.
        verifier: if root.join("usr/bin/unix_chkpwd").exists() {
            Verifier::UnixChkpwd
        } else if root.join("usr/lib/litric/litric-authd").exists() {
            Verifier::PamHelper
        } else {
            Verifier::None
        },
    }
}

/// Parse one `/run/systemd/sessions/<id>` record.
///
/// The file is `KEY=value` lines in no particular order. Two of the keys decide
/// the state:
///
/// * `ACTIVE=1` — this is the session in the foreground on its seat.
/// * `STATE=active|online|closing` — whether it is running at all.
///
/// An `ACTIVE=1` session is `active`; otherwise `STATE=` is normalised, and an
/// unrecognised state becomes `unknown` rather than being assumed online. The
/// `ACTIVE=1` line wins because that is the question a window manager and the
/// shell both ask: "is this the session the user is looking at".
#[must_use]
pub fn parse_session_record(id: &str, text: &str) -> Session {
    let mut fields: BTreeMap<&str, &str> = BTreeMap::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            fields.insert(k.trim(), v.trim());
        }
    }
    let get = |k: &str| fields.get(k).copied().filter(|v| !v.is_empty());

    let active = get("ACTIVE").map(|v| v == "1").unwrap_or(false);
    let state = if active {
        "active".to_owned()
    } else {
        match get("STATE") {
            Some("active") => "active".to_owned(),
            Some("online") => "online".to_owned(),
            Some("closing") => "closing".to_owned(),
            Some(other) => format!("unknown:{other}"),
            None => "unknown".to_owned(),
        }
    };

    Session {
        id: id.to_owned(),
        user: get("USER").map(str::to_owned),
        uid: get("UID").and_then(|v| v.parse().ok()),
        seat: get("SEAT").map(str::to_owned),
        kind: get("TYPE").map(str::to_owned),
        state,
        tty: get("TTY").map(str::to_owned),
        display: get("DISPLAY").map(str::to_owned),
        leader: get("LEADER").and_then(|v| v.parse().ok()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape logind actually writes. The comment above the parser claims
    /// it; this string is what makes the claim checkable.
    const RECORD: &str = "\
UID=1000
USER=audim
ACTIVE=1
STATE=active
REMOTE=0
TYPE=tty
CLASS=user
SEAT=seat0
TTY=tty1
DISPLAY=:0
LEADER=612
SCOPE=session-1.scope
SERVICE=login
";

    #[test]
    fn a_logind_record_is_parsed_field_by_field() {
        let s = parse_session_record("1", RECORD);
        assert_eq!(s.id, "1");
        assert_eq!(s.user.as_deref(), Some("audim"));
        assert_eq!(s.uid, Some(1000));
        assert_eq!(s.seat.as_deref(), Some("seat0"));
        assert_eq!(s.kind.as_deref(), Some("tty"));
        assert_eq!(s.state, "active");
        assert_eq!(s.tty.as_deref(), Some("tty1"));
        assert_eq!(s.display.as_deref(), Some(":0"));
        assert_eq!(s.leader, Some(612));
    }

    #[test]
    fn a_session_that_is_not_in_the_foreground_is_not_called_active() {
        let text = "UID=1000\nUSER=audim\nACTIVE=0\nSTATE=online\nSEAT=seat0\n";
        let s = parse_session_record("2", text);
        assert_eq!(s.state, "online");

        // A state this build does not know is reported as unknown, not folded
        // into "online" where it would read as a working session.
        let text = "UID=1000\nACTIVE=0\nSTATE=lingering\n";
        assert_eq!(parse_session_record("3", text).state, "unknown:lingering");
        assert_eq!(parse_session_record("4", "UID=1000\n").state, "unknown");
    }

    #[test]
    fn a_record_missing_everything_but_an_id_still_parses() {
        let s = parse_session_record("9", "JUNK\n\n");
        assert_eq!(s.id, "9");
        assert_eq!(s.user, None);
        assert_eq!(s.uid, None);
        assert_eq!(s.state, "unknown");
    }

    #[test]
    fn a_lock_is_offered_exactly_when_a_password_can_be_verified() {
        // The security-critical assertion of this file. It is checked in both
        // directions on purpose: a lock offered where nothing can verify is the
        // "accepts anything" failure, and a refusal kept where verification
        // works is the mirror image of it -- a lock the user cannot have.
        let facts = SessionFacts::default();
        assert_eq!(facts.verifier, Verifier::None);
        assert!(!facts.verifier.can_verify());
        assert!(
            !facts.verifier.can_lock(),
            "an image that cannot check a password must not offer a lock"
        );
        assert!(!Verifier::None.refusal().is_empty());
        let text = facts.to_lines().join("\n");
        assert!(text.contains("auth.available=none"), "{text}");
        assert!(text.contains("auth.lock=refused"), "{text}");

        for v in [Verifier::UnixChkpwd, Verifier::PamHelper] {
            assert!(v.can_verify(), "{v:?} is a verifier");
            assert!(
                v.can_lock(),
                "{v:?} can verify a password, so the lock must be available: the \
                 shell's lock surface is unlocked by exactly that verification"
            );
            // The fact and the refusal move together: with the lock available
            // there is no refusal to report, and a reply that carried one would
            // tell the desktop to lock something it could not unlock.
            let lines = SessionFacts {
                verifier: v,
                ..SessionFacts::default()
            }
            .to_lines();
            assert!(!lines.iter().any(|l| l == "auth.lock=refused"), "{v:?}");
            assert!(!v.refusal().is_empty());
        }
        assert_eq!(Verifier::parse("unix-chkpwd"), Verifier::UnixChkpwd);
        assert_eq!(Verifier::parse("pam-helper"), Verifier::PamHelper);
        assert_eq!(Verifier::parse(""), Verifier::None);
        assert_eq!(Verifier::parse("yes"), Verifier::None);
    }

    #[test]
    fn a_machine_with_two_active_sessions_has_no_single_active_one() {
        let a = parse_session_record("1", "ACTIVE=1\nUSER=audim\n");
        let b = parse_session_record("2", "ACTIVE=1\nUSER=root\n");
        let facts = SessionFacts {
            sessions: vec![a.clone()],
            logind_present: true,
            verifier: Verifier::None,
        };
        assert_eq!(facts.active().map(|s| s.id.as_str()), Some("1"));

        let facts = SessionFacts {
            sessions: vec![a, b],
            logind_present: true,
            verifier: Verifier::None,
        };
        // "Which session is the user in" answered by picking one of two is a
        // guess, and a guess here locks the wrong session.
        assert!(facts.active().is_none());
    }

    #[test]
    fn sessions_are_read_from_disk_and_their_absence_is_reported() {
        let root = std::env::temp_dir().join("litric-session-fixture");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("run/systemd/sessions")).expect("mkdir");
        std::fs::write(root.join("run/systemd/sessions/1"), RECORD).expect("write");

        let facts = read(&root);
        assert!(facts.logind_present);
        assert_eq!(facts.sessions.len(), 1);
        assert_eq!(facts.active().map(|s| s.id.as_str()), Some("1"));
        assert!(facts
            .to_lines()
            .iter()
            .any(|l| l == "session.logind=present"));

        // No logind directory: reported as absent, with no sessions claimed.
        let empty = std::env::temp_dir().join("litric-session-fixture-empty");
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).expect("mkdir");
        let facts = read(&empty);
        assert!(!facts.logind_present);
        assert!(facts.sessions.is_empty());
        assert!(facts
            .to_lines()
            .iter()
            .any(|l| l == "session.logind=absent"));

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&empty);
    }

    #[test]
    fn the_verifier_is_found_on_the_filesystem_not_assumed() {
        let root = std::env::temp_dir().join("litric-auth-fixture");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("mkdir");
        assert_eq!(read(&root).verifier, Verifier::None);

        // This build's verifier: the distribution's helper, found by being
        // there. With it present the lock is *not* refused — see `can_lock`.
        std::fs::create_dir_all(root.join("usr/bin")).expect("mkdir");
        std::fs::write(root.join("usr/bin/unix_chkpwd"), "").expect("write");
        let with_helper = read(&root);
        assert_eq!(with_helper.verifier, Verifier::UnixChkpwd);
        let text = with_helper.to_lines().join("\n");
        assert!(text.contains("auth.available=unix-chkpwd"), "{text}");
        assert!(!text.contains("auth.lock=refused"), "{text}");

        // Ours, if it ever exists, is the second name for the same job.
        std::fs::remove_file(root.join("usr/bin/unix_chkpwd")).expect("remove");
        std::fs::create_dir_all(root.join("usr/lib/litric")).expect("mkdir");
        std::fs::write(root.join("usr/lib/litric/litric-authd"), "").expect("write");
        assert_eq!(read(&root).verifier, Verifier::PamHelper);
        let _ = std::fs::remove_dir_all(&root);
    }
}
