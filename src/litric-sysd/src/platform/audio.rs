//! Audio facts: cards, PCM devices, and whether a sound server is running.
//!
//! # What is reachable, and what is not
//!
//! PipeWire is the sound server and WirePlumber is its policy; neither has a
//! file interface. Their protocol is the PipeWire native protocol over
//! `/run/user/<uid>/pipewire-0`, and speaking it in-tree is a subsystem rather
//! than a task — and taking `libpipewire` as a dependency breaks the dependency
//! policy this tree is built on. So this module reports what the *kernel* knows
//! and what can be seen on the filesystem:
//!
//! * `/proc/asound/cards` — the sound cards, their ids, drivers and names.
//! * `/proc/asound/pcm` — the PCM devices and whether each plays, records, or
//!   does both.
//! * `/run/user/<uid>/pipewire-0` — whether a sound server is up, as a socket
//!   that exists. That is *presence*, not a connection: this module never opens
//!   it, cannot say whether it answers, and does not claim to.
//!
//! ## The volume control, and how it is reached
//!
//! The ALSA control API is an `ioctl` on `/dev/snd/controlC<card>`, which needs
//! `unsafe` (forbidden in this crate) or `libasound` (a dependency this tree
//! reviews). Neither is used. What is used, since 2026-10-12, is the sound
//! server's **own client**: `wpctl`, shipped by `wireplumber`, which the desktop
//! profile already carries. It sets the default sink's volume and mute, and the
//! daemon reaches it through `crate::providers` with a fixed argument vector and
//! a validated percentage.
//!
//! This module's job is the *fact* that says a control exists, read from the
//! filesystem rather than assumed — the same rule as the credential verifier in
//! `session.rs`. `audio.control=wpctl` when the tool is in the image and
//! `audio.control=none` when it is not, so a caller cannot mistake "no sound
//! server answered" for "there is no way to ask".
//!
//! That leaves the honest split: **state is read, control is real where a
//! server and a tool exist, and the reply says which of the two it is.**

use std::path::Path;

use super::{read_dir, read_file};

/// One sound card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Kernel card index.
    pub index: u32,
    /// Card id, e.g. `PCH`.
    pub id: String,
    /// Driver name, e.g. `HDA-Intel`.
    pub driver: String,
    /// Human-readable name, e.g. `HDA Intel PCH`.
    pub name: String,
}

/// One PCM device on a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcm {
    /// Card index.
    pub card: u32,
    /// Device index on that card.
    pub device: u32,
    /// Device name from `/proc/asound/pcm`.
    pub name: String,
    /// Whether it can play.
    pub playback: bool,
    /// Whether it can record.
    pub capture: bool,
}

/// A sound server seen from the outside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// `running` when a socket exists, `none` when none does.
    pub state: &'static str,
    /// The socket path when one exists. Derived from the directory the socket
    /// was found in, never from a request field: a fact about another user's
    /// session must not be producible by asking for one (threat case A5).
    pub socket: Option<String>,
}

/// Everything this module reports about audio.
#[derive(Debug, Clone, Default)]
pub struct AudioFacts {
    /// Sound cards.
    pub cards: Vec<Card>,
    /// PCM devices.
    pub pcms: Vec<Pcm>,
    /// Sound-server state.
    pub server: Option<Server>,
    /// The tool that can change a level, when one is installed.
    ///
    /// `None` means the image has no such tool and a volume change cannot be
    /// made — not that it was not looked for. `to_lines` writes the word
    /// `none` for it, so the difference between "no tool" and "a tool that
    /// failed" is the reply's exit status, and never a silent zero.
    pub control: Option<&'static str>,
}

impl AudioFacts {
    /// The facts as `key=value` lines.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!("audio.card.count={}", self.cards.len()));
        for c in &self.cards {
            out.push(format!(
                "audio.card index={} id={} name={} driver={}",
                c.index, c.id, c.name, c.driver
            ));
        }
        for p in &self.pcms {
            out.push(format!(
                "audio.pcm card={} device={} name={} playback={} capture={}",
                p.card,
                p.device,
                p.name,
                u8::from(p.playback),
                u8::from(p.capture)
            ));
        }
        out.push(match &self.server {
            Some(s) => format!(
                "audio.state server={} socket={}",
                s.state,
                s.socket.as_deref().unwrap_or("-")
            ),
            // No `run/user` at all: the daemon could not look, which is a
            // different thing from "no server is running".
            None => "audio.state server=unknown socket=-".to_owned(),
        });
        out.push(format!("audio.control={}", self.control.unwrap_or("none")));
        out
    }
}

/// Read everything from a filesystem root.
#[must_use]
pub fn read(root: &Path) -> AudioFacts {
    AudioFacts {
        cards: read_file(root, "proc/asound/cards")
            .as_deref()
            .map_or_else(Vec::new, parse_cards),
        pcms: read_file(root, "proc/asound/pcm")
            .as_deref()
            .map_or_else(Vec::new, parse_pcm),
        server: find_server(root),
        // Found by being there. A build without `wpctl` says `none` and the
        // action is then reported as failed rather than silently succeeding.
        control: if root.join("usr/bin/wpctl").exists() {
            Some("wpctl")
        } else {
            None
        },
    }
}

/// Parse `/proc/asound/cards`.
///
/// Two lines per card, the second of which is detail:
///
/// ```text
///  0 [PCH            ]: HDA-Intel - HDA Intel PCH
///                       HDA Intel PCH at 0xf7f10000 irq 30
/// ```
///
/// The id inside the brackets is padded to a fixed width and must be trimmed;
/// a line whose first token is not a bare number is skipped, which is what
/// keeps the detail lines from being parsed as cards.
#[must_use]
pub fn parse_cards(text: &str) -> Vec<Card> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((idx, rest)) = line.trim().split_once(' ') else {
            continue;
        };
        let Ok(index) = idx.trim().parse::<u32>() else {
            continue;
        };
        let Some((bracket, tail)) = rest.trim().split_once(']') else {
            continue;
        };
        let id = bracket.trim_start_matches('[').trim().to_owned();
        let tail = tail.trim_start_matches(':').trim();
        // A card with no name ends in ` - `, and `split_once` on the trimmed
        // tail finds no separator at all, because the trim ate the space that
        // made it one. Both spellings mean the same thing: driver, no name.
        let (driver, name) = match tail.split_once(" - ") {
            Some((d, n)) => (d.trim(), n.trim()),
            None => (tail.trim_end_matches(" -").trim(), ""),
        };
        out.push(Card {
            index,
            id,
            driver: driver.to_owned(),
            name: name.to_owned(),
        });
    }
    out
}

/// Parse `/proc/asound/pcm`.
///
/// ```text
/// 00-00: HDA Intel PCH : playback 1 : capture 1
/// 00-03: HDA Intel PCH : playback 1
/// ```
///
/// The `playback`/`capture` fields carry a subdevice count, not a boolean, and
/// the presence of the field is the fact: a device with `capture 1` can record,
/// and the number after it is how many streams at once.
#[must_use]
pub fn parse_pcm(text: &str) -> Vec<Pcm> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((prefix, rest)) = line.split_once(':') else {
            continue;
        };
        let Some((card, device)) = prefix.trim().split_once('-') else {
            continue;
        };
        let (Ok(card), Ok(device)) = (card.parse::<u32>(), device.parse::<u32>()) else {
            continue;
        };
        let mut name = String::new();
        let (mut playback, mut capture) = (false, false);
        for (i, part) in rest.split(" : ").enumerate() {
            let part = part.trim();
            if i == 0 {
                name = part.to_owned();
                continue;
            }
            if part.starts_with("playback") {
                playback = true;
            }
            if part.starts_with("capture") {
                capture = true;
            }
        }
        out.push(Pcm {
            card,
            device,
            name,
            playback,
            capture,
        });
    }
    out
}

/// Find a PipeWire socket under `/run/user`, if there is one.
///
/// Every user directory is checked rather than one uid's, because the daemon
/// runs as root and the shell as the logged-in user; reporting the *path* it
/// found (including the uid in it) is honest about whose session it is. What is
/// never done is opening it.
#[must_use]
pub fn find_server(root: &Path) -> Option<Server> {
    // `None` — nothing to look at — when /run/user does not exist at all.
    if !root.join("run/user").is_dir() {
        return None;
    }
    for uid in read_dir(root, "run/user") {
        let rel = format!("run/user/{uid}/pipewire-0");
        if root.join(&rel).exists() {
            return Some(Server {
                state: "running",
                socket: Some(format!("/{rel}")),
            });
        }
    }
    Some(Server {
        state: "none",
        socket: None,
    })
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn cards_are_parsed_and_the_detail_line_is_not_a_card() {
        let text = "\
 0 [PCH            ]: HDA-Intel - HDA Intel PCH
                      HDA Intel PCH at 0xf7f10000 irq 30
 1 [Device         ]: USB-Audio - USB Audio Device
                      USB Audio Device at usb-0000:00:14.0-1, full speed
";
        let cards = parse_cards(text);
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].index, 0);
        assert_eq!(cards[0].id, "PCH");
        assert_eq!(cards[0].driver, "HDA-Intel");
        assert_eq!(cards[0].name, "HDA Intel PCH");
        assert_eq!(cards[1].id, "Device");
        assert_eq!(cards[1].driver, "USB-Audio");
    }

    #[test]
    fn a_card_line_with_no_name_still_reports_the_driver() {
        let cards = parse_cards(" 0 [Loopback       ]: Loopback - \n");
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].driver, "Loopback");
        assert_eq!(cards[0].name, "");
    }

    #[test]
    fn pcm_devices_report_playback_and_capture_separately() {
        let text = "\
00-00: HDA Intel PCH : playback 1 : capture 1
00-03: HDA Intel PCH : playback 1
00-08: HDA Intel PCH : capture 1
";
        let pcms = parse_pcm(text);
        assert_eq!(pcms.len(), 3);
        assert!(pcms[0].playback && pcms[0].capture);
        assert!(pcms[1].playback && !pcms[1].capture);
        assert!(!pcms[2].playback && pcms[2].capture);
        assert_eq!(pcms[0].card, 0);
        assert_eq!(pcms[1].device, 3);
    }

    #[test]
    fn no_sound_server_is_reported_as_none_and_no_look_at_all_as_unknown() {
        let root = std::env::temp_dir().join("litric-audio-fixture");
        let _ = std::fs::remove_dir_all(&root);

        // No /run/user: the daemon could not look.
        assert_eq!(find_server(&root), None);

        // /run/user exists and holds nothing: there is no server.
        std::fs::create_dir_all(root.join("run/user")).expect("mkdir");
        let s = find_server(&root).expect("a verdict");
        assert_eq!(s.state, "none");
        assert_eq!(s.socket, None);

        // A socket appears. The path says whose session it is.
        std::fs::create_dir_all(root.join("run/user/1000")).expect("mkdir");
        std::fs::write(root.join("run/user/1000/pipewire-0"), "").expect("write");
        let s = find_server(&root).expect("a verdict");
        assert_eq!(s.state, "running");
        assert_eq!(s.socket.as_deref(), Some("/run/user/1000/pipewire-0"));

        let text = AudioFacts {
            server: Some(s),
            ..AudioFacts::default()
        }
        .to_lines()
        .join("\n");
        assert!(text.contains("audio.state server=running socket=/run/user/1000/pipewire-0"));
        // ...and the reply says out loud what can be *set*, so no caller can
        // mistake presence for control. This fixture has no `wpctl`, which is
        // the honest answer for a root without the tool.
        assert!(text.contains("audio.control=none"), "{text}");

        // With the sound server's own client in the image, the same reply names
        // it — the fact is read from the root, not assumed from the profile.
        std::fs::create_dir_all(root.join("usr/bin")).expect("mkdir");
        std::fs::write(root.join("usr/bin/wpctl"), "").expect("write");
        assert_eq!(read(&root).control, Some("wpctl"));
        assert!(read(&root)
            .to_lines()
            .iter()
            .any(|l| l == "audio.control=wpctl"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_machine_with_no_sound_hardware_is_reported_as_having_none() {
        let facts = AudioFacts::default();
        let lines = facts.to_lines();
        assert!(lines.iter().any(|l| l == "audio.card.count=0"));
        // With a root that has no /run/user the state is `unknown`, which is
        // the module-level default: `server=None` means not looked at.
        assert!(lines
            .iter()
            .any(|l| l == "audio.state server=unknown socket=-"));
    }
}
