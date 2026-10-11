//! What is known about running a particular Windows program here.
//!
//! # Why a database and not an attempt
//!
//! A translation layer either works for a program or it does not, and for a
//! large class of software the answer is known in advance and is "no" — not
//! because the translation is incomplete, but because the program contains a
//! component whose entire purpose is to detect that it is being translated.
//! Kernel-level anti-cheat is the clearest case: it is *designed* to refuse to
//! run under Wine or Proton, and no amount of work on our side changes that,
//! because refusing is the feature.
//!
//! Discovering that after a four-gigabyte download and a ten-minute install is
//! a bad experience and, worse, it looks like our bug. So the answer is given
//! before the download, with the reason and with whatever route does work.
//!
//! # What this is not
//!
//! It is not a promise that anything absent from the table will work. The
//! default verdict is [`Verdict::Unknown`], and unknown means unknown: the
//! interface says "this has not been tested here" rather than "this works".
//!
//! # Where the entries come from
//!
//! Published behaviour of the software itself — an anti-cheat vendor stating
//! that virtualised and translated environments are refused, a publisher
//! documenting a platform requirement. Each entry carries its reason so the
//! claim can be checked rather than believed.

/// What is known about running a program under the compatibility layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Verdict {
    /// Nothing is recorded. Not a prediction either way.
    Unknown,
    /// Known to work.
    Works,
    /// Works with a caveat that the user should know about first.
    Caveat,
    /// Known not to work under translation, and known why.
    Blocked,
}

impl Verdict {
    /// Whether the compatibility layer should even be offered.
    #[must_use]
    pub const fn should_offer_bottle(self) -> bool {
        !matches!(self, Self::Blocked)
    }

    /// A short token for the interface and the status file.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Works => "works",
            Self::Caveat => "caveat",
            Self::Blocked => "blocked",
        }
    }
}

/// The route that does work, when the compatibility layer does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Alternative {
    /// Nothing else to suggest.
    None,
    /// Run it in the Windows environment: a real Windows guest, reached with
    /// `Win+1`. Anti-cheat that refuses translation generally accepts a real
    /// Windows kernel, though some also refuse virtual machines.
    WindowsEnvironment,
    /// The publisher ships a native Linux build.
    NativeLinux,
    /// The publisher ships a build for another platform that runs here through
    /// a different route, e.g. an Android build under a container.
    OtherPlatformBuild,
    /// It runs in a browser.
    WebVersion,
}

impl Alternative {
    /// What to tell the user to do instead.
    #[must_use]
    pub const fn advice(self) -> &'static str {
        match self {
            Self::None => "There is no known way to run this here.",
            Self::WindowsEnvironment => {
                "Install it in the Windows environment instead, reached with Win+1. \
                 That is a real Windows system rather than a translation layer. \
                 Note that some anti-cheat also refuses virtual machines."
            }
            Self::NativeLinux => "The publisher ships a native Linux build; install that instead.",
            Self::OtherPlatformBuild => {
                "The publisher ships a build for another platform that can run here \
                 through a different route."
            }
            Self::WebVersion => "It runs in a browser; no installation is needed.",
        }
    }
}

/// One thing known about one program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// A lowercase substring matched against the file name and the program's
    /// declared name.
    pub key: &'static str,
    /// The name shown to the user.
    pub title: &'static str,
    /// What is known.
    pub verdict: Verdict,
    /// Why, in the user's words.
    pub reason: &'static str,
    /// What to do instead.
    pub alternative: Alternative,
}

/// Programs whose behaviour under a translation layer is known in advance.
///
/// Short on purpose. A long list of guesses is worse than a short list of
/// facts, and every row here is something that can be checked.
pub const KNOWN: &[Entry] = &[
    Entry {
        key: "roblox",
        title: "Roblox",
        verdict: Verdict::Blocked,
        reason: "Roblox's Windows client ships the Hyperion (Byfron) anti-cheat, \
                 which detects translation layers and refuses to start under them. \
                 This is deliberate on Roblox's part, not a gap in the compatibility \
                 layer, so no amount of work here will change it.",
        alternative: Alternative::WindowsEnvironment,
    },
    Entry {
        key: "valorant",
        title: "VALORANT",
        verdict: Verdict::Blocked,
        reason: "Vanguard is a kernel-mode anti-cheat that requires a Windows kernel \
                 and refuses translated and virtualised environments.",
        alternative: Alternative::None,
    },
    Entry {
        key: "easyanticheat",
        title: "Easy Anti-Cheat",
        verdict: Verdict::Caveat,
        reason: "Easy Anti-Cheat can run under a translation layer, but only when the \
                 publisher has enabled it for that platform. Whether a particular game \
                 works is the publisher's decision, not ours.",
        alternative: Alternative::WindowsEnvironment,
    },
    Entry {
        key: "battleye",
        title: "BattlEye",
        verdict: Verdict::Caveat,
        reason: "BattlEye supports translation layers only where the publisher has \
                 opted in.",
        alternative: Alternative::WindowsEnvironment,
    },
    Entry {
        key: "notepad++",
        title: "Notepad++",
        verdict: Verdict::Works,
        reason: "A plain Win32 application with no driver, no anti-cheat and no \
                 platform-specific service.",
        alternative: Alternative::None,
    },
    Entry {
        key: "7z",
        title: "7-Zip",
        verdict: Verdict::Works,
        reason: "A plain Win32 application with no driver and no background service. \n                 A native build also exists, which is usually the better choice.",
        alternative: Alternative::NativeLinux,
    },
];

/// What is known about a file, from its name.
///
/// Matching on the name is a heuristic and is treated as one: it produces a
/// warning to show the user before they commit to a download, never a refusal
/// to proceed. The authoritative check happens on the bytes.
#[must_use]
pub fn lookup(file_name: &str) -> Option<&'static Entry> {
    let lower = file_name.to_lowercase();
    KNOWN.iter().find(|e| lower.contains(e.key))
}

/// A complete answer for the interface: verdict, reason and what to do instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    /// What is known.
    pub verdict: Verdict,
    /// The name, when recognised.
    pub title: Option<String>,
    /// Why, in the user's words.
    pub reason: String,
    /// What to do instead.
    pub advice: Option<String>,
}

impl Assessment {
    /// Assess a file by name.
    #[must_use]
    pub fn of(file_name: &str) -> Self {
        match lookup(file_name) {
            Some(e) => Self {
                verdict: e.verdict,
                title: Some(e.title.to_owned()),
                reason: e.reason.to_owned(),
                advice: match e.alternative {
                    Alternative::None if e.verdict == Verdict::Works => None,
                    a => Some(a.advice().to_owned()),
                },
            },
            None => Self {
                verdict: Verdict::Unknown,
                title: None,
                // The honest default. "Untested" is information; "supported"
                // would be a guess, and an optimistic one.
                reason: "This program has not been tested here. It may work, it may \
                         not, and nothing about it is known in advance."
                    .to_owned(),
                advice: None,
            },
        }
    }

    /// Lines for the status file and diagnostics.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!("compat.verdict={}", self.verdict.token())];
        if let Some(t) = &self.title {
            out.push(format!("compat.title={t}"));
        }
        if self.advice.is_some() {
            out.push("compat.alternative=yes".to_owned());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roblox_is_recorded_as_blocked_with_a_reason_and_a_way_forward() {
        // The specific question this module was written to answer honestly.
        let a = Assessment::of("RobloxPlayerInstaller.exe");
        assert_eq!(a.verdict, Verdict::Blocked);
        assert_eq!(a.title.as_deref(), Some("Roblox"));
        assert!(
            a.reason.to_lowercase().contains("anti-cheat"),
            "{}",
            a.reason
        );
        assert!(
            a.advice.as_deref().is_some_and(|s| s.contains("Win+1")),
            "no route offered: {:?}",
            a.advice
        );
        assert!(
            !a.verdict.should_offer_bottle(),
            "a blocked program must not be offered a bottle"
        );
    }

    #[test]
    fn matching_is_case_insensitive_and_matches_anywhere_in_the_name() {
        for name in [
            "ROBLOX.exe",
            "Downloads/roblox-installer.exe",
            "setup_Roblox_2026.exe",
        ] {
            assert_eq!(Assessment::of(name).verdict, Verdict::Blocked, "{name}");
        }
    }

    #[test]
    fn an_unknown_program_is_unknown_and_not_optimistic() {
        // The default must not be "supported": a user who is told a program
        // will work and then finds it does not stops believing every other
        // thing the system says.
        let a = Assessment::of("SomeInternalTool.exe");
        assert_eq!(a.verdict, Verdict::Unknown);
        assert!(a.title.is_none());
        assert!(a.reason.contains("has not been tested"), "{}", a.reason);
        assert!(
            a.verdict.should_offer_bottle(),
            "unknown is still worth trying"
        );
    }

    #[test]
    fn a_known_working_program_offers_no_alternative_it_does_not_need() {
        let a = Assessment::of("notepad++-8.6-installer.exe");
        assert_eq!(a.verdict, Verdict::Works);
        assert!(a.advice.is_none());
    }

    #[test]
    fn a_caveat_is_neither_a_promise_nor_a_refusal() {
        let a = Assessment::of("game_with_easyanticheat_setup.exe");
        assert_eq!(a.verdict, Verdict::Caveat);
        assert!(a.verdict.should_offer_bottle());
        assert!(a.advice.is_some());
    }

    #[test]
    fn every_entry_carries_a_reason_that_could_be_checked() {
        for e in KNOWN {
            assert!(!e.reason.is_empty(), "{} has no reason", e.key);
            assert!(
                e.reason.len() > 40,
                "{}'s reason is too short to be checkable: {}",
                e.key,
                e.reason
            );
            assert_eq!(e.key, e.key.to_lowercase(), "{} is not lowercase", e.key);
            assert!(!e.title.is_empty());
        }
    }

    #[test]
    fn no_two_entries_share_a_key() {
        let mut keys: Vec<&str> = KNOWN.iter().map(|e| e.key).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "a duplicate key makes one entry dead");
    }

    #[test]
    fn every_blocked_or_caveat_entry_says_what_to_do_instead() {
        for e in KNOWN {
            if matches!(e.verdict, Verdict::Blocked | Verdict::Caveat) {
                let a = Assessment::of(e.key);
                assert!(
                    a.advice.is_some(),
                    "{} is {} with no alternative offered",
                    e.key,
                    e.verdict.token()
                );
            }
        }
    }

    #[test]
    fn status_lines_carry_the_verdict() {
        let lines = Assessment::of("roblox.exe").lines();
        assert!(
            lines.contains(&"compat.verdict=blocked".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"compat.title=Roblox".to_owned()),
            "{lines:?}"
        );
    }
}
