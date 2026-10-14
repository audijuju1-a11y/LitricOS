//! Bluetooth facts: adapters, radio blocks, and whether a daemon is present.
//!
//! Bluetooth is the one subsystem in this wave with **no userspace daemon in
//! the image**: `bluez` is in no profile, so there is no `bluetoothd` to pair
//! through and pairing, trust and connect are not implementable here yet
//! (`docs/design/wave1/01-bluetooth.md` §2 — a product decision, not an
//! oversight).
//!
//! What is readable anyway, and what this module therefore reports:
//!
//! * `/sys/class/bluetooth/hci*` — whether a radio exists at all, and its
//!   address. `bluez` is a client of this, not its owner.
//! * `/sys/class/rfkill/rfkill*` — whether the radio is blocked, and **which
//!   kind of blocked**: `hard` is a physical or firmware switch the software
//!   cannot undo, `soft` is the software's own opinion. Reporting them as one
//!   boolean is how a UI ends up offering a switch that cannot work.
//! * `usr/bin/bluetoothd` — whether a daemon is actually installed, checked on
//!   the filesystem rather than asserted from a package list, so the reply and
//!   the image cannot drift apart.
//!
//! Three answers, kept apart on purpose: **no radio present**, **radio present
//! and blocked**, **radio present and usable but nothing to pair with**. The
//! first is a fact about the machine; the third is a fact about this image, and
//! a Settings row that says "Bluetooth: unavailable" for the third one is
//! saying something the user cannot act on.

use std::path::Path;

use super::{read_attr, read_dir};

/// One HCI adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adapter {
    /// Kernel name, e.g. `hci0`.
    pub name: String,
    /// The adapter's own address, lower-case, as sysfs writes it.
    pub address: Option<String>,
    /// The device name if the kernel exposes one. Often absent before the
    /// adapter is powered, which is why it is an `Option`.
    pub device_name: Option<String>,
}

/// One rfkill entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radio {
    /// Directory name, e.g. `rfkill0`.
    pub name: String,
    /// `bluetooth`, `wlan`, … as the kernel writes it.
    pub kind: Option<String>,
    /// The kernel's own name for the switch.
    pub label: Option<String>,
    /// Blocked by software.
    pub soft: Option<bool>,
    /// Blocked by a physical switch or the firmware.
    pub hard: Option<bool>,
}

impl Radio {
    /// Whether anything at all is blocking this radio.
    #[must_use]
    pub const fn blocked(&self) -> bool {
        matches!(self.soft, Some(true)) || matches!(self.hard, Some(true))
    }

    /// The word for what is blocking it, or `none`.
    #[must_use]
    pub const fn blocked_by(&self) -> &'static str {
        // Hard first: when both are set, the one the software cannot undo is
        // the one worth telling the user about.
        if matches!(self.hard, Some(true)) {
            "hard"
        } else if matches!(self.soft, Some(true)) {
            "soft"
        } else {
            "none"
        }
    }
}

/// Everything this module reports about Bluetooth.
#[derive(Debug, Clone, Default)]
pub struct BtFacts {
    /// Adapters.
    pub adapters: Vec<Adapter>,
    /// rfkill entries, all of them: the Bluetooth ones are the interesting
    /// ones, but a facts reader that filters is a facts reader that hides the
    /// reason a radio is off.
    pub radios: Vec<Radio>,
    /// Whether `bluetoothd` is installed.
    pub daemon: bool,
}

impl BtFacts {
    /// The three-word verdict the user interface keys off.
    ///
    /// * `no-adapter` — this machine has no radio.
    /// * `blocked` — there is a radio and something is stopping it.
    /// * `no-daemon` — a radio exists and nothing blocks it, but this image has
    ///   no `bluetoothd`, so nothing can be paired. Without this third word the
    ///   Settings row would claim Bluetooth works.
    #[must_use]
    pub fn state(&self) -> &'static str {
        if self.adapters.is_empty() {
            return "no-adapter";
        }
        if self
            .radios
            .iter()
            .filter(|r| r.kind.as_deref() == Some("bluetooth"))
            .any(Radio::blocked)
        {
            return "blocked";
        }
        if !self.daemon {
            return "no-daemon";
        }
        "ready"
    }

    /// The facts as `key=value` lines.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!("bt.adapter.count={}", self.adapters.len()));
        out.push(format!("bt.state={}", self.state()));
        out.push(format!(
            "bt.daemon={}",
            if self.daemon { "present" } else { "absent" }
        ));
        for a in &self.adapters {
            out.push(format!(
                "bt.adapter name={} address={} device_name={}",
                a.name,
                a.address.as_deref().unwrap_or("-"),
                a.device_name.as_deref().unwrap_or("-")
            ));
        }
        for r in &self.radios {
            out.push(format!(
                "bt.rfkill name={} type={} block={} soft={} hard={}",
                r.name,
                r.kind.as_deref().unwrap_or("-"),
                r.blocked_by(),
                flag(r.soft),
                flag(r.hard)
            ));
        }
        out
    }
}

/// A sysfs boolean as `1`/`0`/`-`.
fn flag(v: Option<bool>) -> &'static str {
    match v {
        Some(true) => "1",
        Some(false) => "0",
        None => "-",
    }
}

/// Read everything from a filesystem root.
#[must_use]
pub fn read(root: &Path) -> BtFacts {
    let mut adapters = Vec::new();
    for name in read_dir(root, "sys/class/bluetooth") {
        if !name.starts_with("hci") {
            continue;
        }
        let base = format!("sys/class/bluetooth/{name}");
        adapters.push(Adapter {
            address: read_attr(root, &format!("{base}/address")).map(|s| s.to_ascii_lowercase()),
            device_name: read_attr(root, &format!("{base}/name")),
            name,
        });
    }

    let mut radios = Vec::new();
    for name in read_dir(root, "sys/class/rfkill") {
        if !name.starts_with("rfkill") {
            continue;
        }
        let base = format!("sys/class/rfkill/{name}");
        radios.push(Radio {
            kind: read_attr(root, &format!("{base}/type")),
            label: read_attr(root, &format!("{base}/name")),
            soft: read_attr(root, &format!("{base}/soft")).map(|s| parse_bool(&s)),
            hard: read_attr(root, &format!("{base}/hard")).map(|s| parse_bool(&s)),
            name,
        });
    }

    BtFacts {
        adapters,
        radios,
        daemon: root.join("usr/bin/bluetoothd").exists(),
    }
}

/// Parse a sysfs boolean that may be `1`/`0` or a word.
///
/// Anything unrecognised is `false` rather than an error, because the only
/// wrong answer that matters here is claiming a *hard* block that is not there
/// — that would tell a user their machine has a physical switch it does not.
#[must_use]
pub fn parse_bool(text: &str) -> bool {
    matches!(text.trim(), "1" | "yes" | "true")
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn fixture(root: &Path, files: &[(&str, &str)]) {
        for (rel, content) in files {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            std::fs::write(p, content).expect("write");
        }
    }

    #[test]
    fn a_machine_with_no_radio_says_so() {
        let facts = BtFacts::default();
        assert_eq!(facts.state(), "no-adapter");
        assert!(facts.to_lines().iter().any(|l| l == "bt.adapter.count=0"));
    }

    #[test]
    fn a_radio_with_no_daemon_is_not_reported_as_working() {
        let facts = BtFacts {
            adapters: vec![Adapter {
                name: "hci0".to_owned(),
                address: Some("00:1a:7d:da:71:13".to_owned()),
                device_name: None,
            }],
            radios: vec![],
            daemon: false,
        };
        // A radio that nothing blocks is still useless without bluetoothd, and
        // the word for that is not "ready".
        assert_eq!(facts.state(), "no-daemon");
    }

    #[test]
    fn hard_and_soft_blocks_are_told_apart() {
        let mk = |soft: bool, hard: bool| Radio {
            name: "rfkill0".to_owned(),
            kind: Some("bluetooth".to_owned()),
            label: Some("hci0".to_owned()),
            soft: Some(soft),
            hard: Some(hard),
        };
        assert!(!mk(false, false).blocked());
        assert_eq!(mk(false, false).blocked_by(), "none");
        assert!(mk(true, false).blocked());
        assert_eq!(mk(true, false).blocked_by(), "soft");
        assert!(mk(false, true).blocked());
        assert_eq!(mk(false, true).blocked_by(), "hard");
        // Both set: the one software cannot undo is the one to report.
        assert_eq!(mk(true, true).blocked_by(), "hard");

        let facts = BtFacts {
            adapters: vec![Adapter {
                name: "hci0".to_owned(),
                address: None,
                device_name: None,
            }],
            radios: vec![mk(false, true)],
            daemon: true,
        };
        assert_eq!(facts.state(), "blocked");
        let text = facts.to_lines().join("\n");
        assert!(text.contains("bt.rfkill name=rfkill0 type=bluetooth block=hard soft=0 hard=1"));
    }

    #[test]
    fn only_bluetooth_radios_decide_the_bluetooth_verdict() {
        // A blocked wlan radio must not make the Bluetooth row read "blocked":
        // the two subsystems are separate and the wifi switch is not this one.
        let facts = BtFacts {
            adapters: vec![Adapter {
                name: "hci0".to_owned(),
                address: None,
                device_name: None,
            }],
            radios: vec![Radio {
                name: "rfkill0".to_owned(),
                kind: Some("wlan".to_owned()),
                label: Some("phy0".to_owned()),
                soft: Some(true),
                hard: Some(false),
            }],
            daemon: true,
        };
        assert_eq!(facts.state(), "ready");
    }

    #[test]
    fn a_host_fixture_is_read_end_to_end() {
        let root = std::env::temp_dir().join("litric-bt-fixture");
        let _ = std::fs::remove_dir_all(&root);
        fixture(
            &root,
            &[
                ("sys/class/bluetooth/hci0/address", "00:1A:7D:DA:71:13\n"),
                ("sys/class/bluetooth/hci0/name", "litric-dev\n"),
                ("sys/class/rfkill/rfkill0/type", "bluetooth\n"),
                ("sys/class/rfkill/rfkill0/name", "hci0\n"),
                ("sys/class/rfkill/rfkill0/soft", "1\n"),
                ("sys/class/rfkill/rfkill0/hard", "0\n"),
                ("usr/bin/bluetoothd", "\u{7f}ELF"),
            ],
        );
        let facts = read(&root);
        assert_eq!(facts.adapters.len(), 1);
        assert_eq!(
            facts.adapters[0].address.as_deref(),
            Some("00:1a:7d:da:71:13")
        );
        assert_eq!(facts.adapters[0].device_name.as_deref(), Some("litric-dev"));
        assert!(facts.daemon);
        // Soft-blocked, and the daemon is present: still blocked.
        assert_eq!(facts.state(), "blocked");
        let _ = std::fs::remove_dir_all(&root);
    }
}
