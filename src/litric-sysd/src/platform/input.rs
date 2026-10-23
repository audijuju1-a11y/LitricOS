//! Input device facts: every keyboard, pointer and tablet that is attached.
//!
//! The shell reads `/dev/input/event*` itself — `litric-input` opens the
//! character devices and parses the evdev stream. What it cannot answer from a
//! character device before opening it is *what kind of thing it is*, and that is
//! exactly the question Settings has to answer: a machine with no keyboard
//! attached and a machine whose keyboard failed to probe both deliver no keys,
//! and only one of them is worth telling the user about.
//!
//! sysfs answers it without opening anything:
//!
//! * `/sys/class/input/eventN/device/name` — what the kernel calls it.
//! * `/sys/class/input/eventN/device/capabilities/key` — which key codes it can
//!   emit. A device with `KEY_A` is a keyboard; a pointer's buttons are a
//!   different set entirely.
//! * `.../capabilities/rel` and `.../capabilities/abs` — whether it reports
//!   relative motion (a mouse) or absolute position (a tablet, a touchscreen).
//!
//! The list is sorted by node name so two reads of the same machine produce the
//! same text, and a device whose name cannot be read is still listed with
//! `name=-`: "there is an event node I cannot describe" is a finding, and
//! dropping it would turn "unreadable" into "not there".
//!
//! Nothing here filters devices. A power button is not a keyboard, but a facts
//! reader that silently drops what it does not recognise hides the reason a
//! panel looks empty.

use std::path::Path;

use super::{read_attr, read_dir};

/// What a device is for, decided from the capabilities it advertises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Emits letter keys: a keyboard.
    Keyboard,
    /// Emits relative motion and buttons: a mouse or trackball.
    Mouse,
    /// Emits absolute position: a tablet, a touchscreen, a trackpad in absolute
    /// mode.
    Tablet,
    /// Anything else: a power button, a lid switch, a consumer-control node.
    Other,
}

impl Kind {
    /// The word written on the wire and drawn in Settings.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Tablet => "tablet",
            Self::Other => "other",
        }
    }
}

/// One `/dev/input/event*` node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// The node name, e.g. `event3`.
    pub node: String,
    /// The kernel's name for the device, when it published one.
    pub name: Option<String>,
    /// What the capabilities say it is.
    pub kind: Kind,
}

/// Everything this module reports about input.
#[derive(Debug, Clone, Default)]
pub struct InputFacts {
    /// Every event node, in node order.
    pub devices: Vec<Device>,
}

impl InputFacts {
    /// How many of the devices are keyboards.
    #[must_use]
    pub fn keyboards(&self) -> usize {
        self.devices
            .iter()
            .filter(|d| d.kind == Kind::Keyboard)
            .count()
    }

    /// How many of them are pointers (mice and tablets together, because the
    /// question the user interface asks is "can I point at things").
    #[must_use]
    pub fn pointers(&self) -> usize {
        self.devices
            .iter()
            .filter(|d| matches!(d.kind, Kind::Mouse | Kind::Tablet))
            .count()
    }

    /// The facts as `key=value` lines.
    ///
    /// The name is last because it contains spaces; every other field is a
    /// single token, so a reader can split the line and stop at `name=`.
    #[must_use]
    pub fn to_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.push(format!("input.device.count={}", self.devices.len()));
        out.push(format!("input.kbd.count={}", self.keyboards()));
        out.push(format!("input.pointer.count={}", self.pointers()));
        for d in &self.devices {
            out.push(format!(
                "input.device {} kind={} name={}",
                d.node,
                d.kind.as_str(),
                d.name.as_deref().unwrap_or("-")
            ));
        }
        out
    }
}

/// The key code for `A`. A device that can emit it can type a word; a device
/// with only `KEY_POWER` and `KEY_SLEEP` cannot, and calling that a keyboard is
/// how a Settings row ends up promising typing that does not work.
const KEY_A: u32 = 30;
/// `REL_X` / `REL_Y`: the two axes a mouse reports as deltas.
const REL_X: u32 = 0;
const REL_Y: u32 = 1;
/// `ABS_X` / `ABS_Y`: the two axes a tablet reports as positions.
const ABS_X: u32 = 0;
const ABS_Y: u32 = 1;

/// Whether `bit` is set in a sysfs capability bitmap.
///
/// sysfs writes these as space-separated hex words, least significant first, so
/// bit 30 lives in the first word and bit 272 in the fifth. Parsing it as one
/// big hex number happens to work for small values and silently misreads every
/// pointer, whose button codes are all above 255.
#[must_use]
fn has_bit(bitmap: &str, bit: u32) -> bool {
    let word = usize::try_from(bit / 64).unwrap_or(usize::MAX);
    let Some(text) = bitmap.split_whitespace().nth(word) else {
        return false;
    };
    let Ok(value) = u64::from_str_radix(text, 16) else {
        return false;
    };
    value & (1u64 << (bit % 64)) != 0
}

/// Decide what a node is from its capability bitmaps.
#[must_use]
fn classify(key: &str, rel: &str, abs: &str) -> Kind {
    if has_bit(key, KEY_A) {
        return Kind::Keyboard;
    }
    if has_bit(rel, REL_X) && has_bit(rel, REL_Y) {
        return Kind::Mouse;
    }
    if has_bit(abs, ABS_X) && has_bit(abs, ABS_Y) {
        return Kind::Tablet;
    }
    Kind::Other
}

/// Read every input device from a filesystem root.
#[must_use]
pub fn read(root: &Path) -> InputFacts {
    let mut devices = Vec::new();
    for node in read_dir(root, "sys/class/input") {
        if !node.starts_with("event") {
            continue;
        }
        let base = format!("sys/class/input/{node}/device");
        let caps = format!("{base}/capabilities");
        let key = read_attr(root, &format!("{caps}/key")).unwrap_or_default();
        let rel = read_attr(root, &format!("{caps}/rel")).unwrap_or_default();
        let abs = read_attr(root, &format!("{caps}/abs")).unwrap_or_default();
        devices.push(Device {
            kind: classify(&key, &rel, &abs),
            name: read_attr(root, &format!("{base}/name")),
            node,
        });
    }
    InputFacts { devices }
}

#[cfg(test)]
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
    fn a_machine_with_no_input_reports_zero_and_not_none() {
        let facts = InputFacts::default();
        let lines = facts.to_lines();
        assert!(lines.contains(&"input.device.count=0".to_owned()));
        assert!(lines.contains(&"input.kbd.count=0".to_owned()));
        assert!(lines.contains(&"input.pointer.count=0".to_owned()));
    }

    #[test]
    fn a_keyboard_is_recognised_by_the_letter_a_and_not_by_its_buttons() {
        // A power button has KEY_POWER (116); a mouse has BTN_LEFT (272). Both
        // live in the same bitmap as letters do, which is why the test is a
        // specific letter rather than "has any key".
        let root = std::env::temp_dir().join("litric-input-fixture-kbd");
        let _ = std::fs::remove_dir_all(&root);
        fixture(
            &root,
            &[
                ("sys/class/input/event0/device/name", "QEMU USB Keyboard\n"),
                // Bits 30 (KEY_A) and 57 (KEY_SPACE) in the first word:
                // 1<<57 | 1<<30 = 0x0200_0000_4000_0000.
                (
                    "sys/class/input/event0/device/capabilities/key",
                    "2000000040000000 0 0 0 0\n",
                ),
                ("sys/class/input/event0/device/capabilities/rel", "0\n"),
                ("sys/class/input/event0/device/capabilities/abs", "\n"),
                ("sys/class/input/event2/device/name", "Power Button\n"),
                (
                    "sys/class/input/event2/device/capabilities/key",
                    "100000 0\n",
                ),
            ],
        );
        let facts = read(&root);
        assert_eq!(facts.devices.len(), 2, "{:?}", facts.devices);
        assert_eq!(facts.keyboards(), 1);
        assert_eq!(facts.pointers(), 0);
        assert_eq!(
            facts.devices[0].name.as_deref(),
            Some("QEMU USB Keyboard"),
            "devices are in node order"
        );
        assert_eq!(facts.devices[1].kind, Kind::Other);
        let text = facts.to_lines().join("\n");
        assert!(
            text.contains("input.device event0 kind=keyboard name=QEMU USB Keyboard"),
            "{text}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_tablet_is_a_pointer_even_though_it_reports_no_deltas() {
        let root = std::env::temp_dir().join("litric-input-fixture-tablet");
        let _ = std::fs::remove_dir_all(&root);
        fixture(
            &root,
            &[
                ("sys/class/input/event1/device/name", "QEMU USB Tablet\n"),
                ("sys/class/input/event1/device/capabilities/key", "0\n"),
                ("sys/class/input/event1/device/capabilities/rel", "0\n"),
                // ABS_X (0), ABS_Y (1) and BTN_LEFT (272, fifth word).
                (
                    "sys/class/input/event1/device/capabilities/abs",
                    "3 0 0 0 0\n",
                ),
            ],
        );
        let facts = read(&root);
        assert_eq!(facts.devices.len(), 1);
        assert_eq!(facts.devices[0].kind, Kind::Tablet);
        assert_eq!(facts.pointers(), 1);
        assert_eq!(facts.keyboards(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_mouse_is_recognised_by_its_two_relative_axes() {
        let root = std::env::temp_dir().join("litric-input-fixture-mouse");
        let _ = std::fs::remove_dir_all(&root);
        fixture(
            &root,
            &[
                ("sys/class/input/event4/device/name", "Logitech USB Mouse\n"),
                ("sys/class/input/event4/device/capabilities/key", "0\n"),
                ("sys/class/input/event4/device/capabilities/rel", "3\n"),
                ("sys/class/input/event4/device/capabilities/abs", "\n"),
            ],
        );
        let facts = read(&root);
        assert_eq!(facts.devices[0].kind, Kind::Mouse);
        assert_eq!(facts.pointers(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_node_whose_name_cannot_be_read_is_listed_rather_than_dropped() {
        let root = std::env::temp_dir().join("litric-input-fixture-noname");
        let _ = std::fs::remove_dir_all(&root);
        fixture(
            &root,
            &[("sys/class/input/event9/device/capabilities/key", "0\n")],
        );
        let facts = read(&root);
        assert_eq!(facts.devices.len(), 1);
        assert_eq!(facts.devices[0].name, None);
        assert!(facts
            .to_lines()
            .iter()
            .any(|l| l == "input.device event9 kind=other name=-"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_bitmap_is_read_word_by_word_not_as_one_number() {
        // Bit 272 is BTN_LEFT. In the fifth 64-bit word that is bit 16, so the
        // fifth word is 0x10000. Reading the whole string as one number would
        // place it somewhere else entirely.
        assert!(has_bit("0 0 0 0 10000", 272));
        assert!(!has_bit("0 0 0 0 10000", 271));
        assert!(has_bit("40000000", KEY_A));
    }
}
