//! Finding input devices and working out what they are.
//!
//! `/dev/input/event*` gives no hint what a device is: a keyboard, a mouse, a
//! lid switch and a power button all look alike until you ask what they can
//! report. The kernel publishes that as text under
//! `/sys/class/input/eventN/device/capabilities/`, so classification is file
//! reads and bit tests — no `ioctl`, no `unsafe`, and testable from fixtures.
//!
//! ```text
//!   /sys/class/input/event3/device/name          "AT Translated Set 2 keyboard"
//!   /sys/class/input/event3/device/capabilities/ev    "120013"
//!   /sys/class/input/event3/device/capabilities/key   "402000000 3803078f800d001 ..."
//! ```
//!
//! # The bitmap format
//!
//! Each file is a list of `unsigned long` words in hex, **most significant
//! first**, space separated, with leading all-zero words omitted and no zero
//! padding. So the word's index is its position counted from the right, not its
//! width — which is why this reverses the list rather than slicing fixed-size
//! chunks. Getting that wrong classifies every keyboard as a mouse.

use std::fs;
use std::path::{Path, PathBuf};

use crate::evdev::{ABS_X, ABS_Y, BTN_LEFT, BTN_TOUCH, EV_ABS, EV_KEY, EV_REL, REL_X, REL_Y};

/// Bits in one `unsigned long` on the kernel we read from.
///
/// 64 on every target LitricOS supports. Named because it is the one number in
/// this module that would silently mis-index every capability if it were wrong.
const BITS_PER_WORD: usize = 64;

/// A capability bitmap read from sysfs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bitmap {
    /// Words, least significant first.
    words: Vec<u64>,
}

impl Bitmap {
    /// Parse the sysfs form.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut words: Vec<u64> = text
            .split_whitespace()
            .map(|w| u64::from_str_radix(w, 16).unwrap_or(0))
            .collect();
        words.reverse();
        Self { words }
    }

    /// Whether bit `n` is set.
    #[must_use]
    pub fn has(&self, n: u16) -> bool {
        let n = n as usize;
        let Some(word) = n.checked_div(BITS_PER_WORD) else {
            return false;
        };
        let Some(bit) = n.checked_rem(BITS_PER_WORD) else {
            return false;
        };
        self.words.get(word).is_some_and(|w| w & (1u64 << bit) != 0)
    }

    /// Whether every one of these bits is set.
    #[must_use]
    pub fn has_all(&self, bits: &[u16]) -> bool {
        bits.iter().all(|b| self.has(*b))
    }
}

/// What a device can be used for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeviceKinds {
    /// It has a full alphabetic keyboard.
    pub keyboard: bool,
    /// It reports relative motion and has a primary button: a mouse or
    /// trackball.
    pub pointer: bool,
    /// It reports absolute position and touch: a touchpad or touchscreen.
    pub touch: bool,
}

impl DeviceKinds {
    /// Whether this device is worth opening at all.
    #[must_use]
    pub const fn is_useful(self) -> bool {
        self.keyboard || self.pointer || self.touch
    }

    /// A short description for diagnostics.
    #[must_use]
    pub fn label(self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.keyboard {
            parts.push("keyboard");
        }
        if self.pointer {
            parts.push("pointer");
        }
        if self.touch {
            parts.push("touch");
        }
        if parts.is_empty() {
            "other".to_owned()
        } else {
            parts.join("+")
        }
    }
}

/// Classify a device from its capability bitmaps.
///
/// The tests below are deliberately strict. A power button reports `EV_KEY`
/// and a lid switch reports a couple of key codes; accepting either as a
/// keyboard means the desktop opens a device that will never send a letter and,
/// worse, believes a keyboard is present when none is.
#[must_use]
pub fn classify(ev: &Bitmap, key: &Bitmap, rel: &Bitmap, abs: &Bitmap) -> DeviceKinds {
    // KEY_A, KEY_Z, KEY_SPACE and KEY_ENTER. Anything with all four is a
    // keyboard; nothing that is not a keyboard has all four.
    const KEYBOARD_PROOF: &[u16] = &[30, 44, 57, 28];

    let keyboard = ev.has(EV_KEY) && key.has_all(KEYBOARD_PROOF);
    // Relative mice (EV_REL + BTN_LEFT) AND absolute tablets (EV_ABS + BTN_LEFT)
    // are both pointer devices.  The USB tablet QEMU exposes for VNC has EV_ABS
    // with ABS_X/ABS_Y and BTN_LEFT — without this branch the desktop would
    // ignore it entirely, leaving the mouse stuck at screen centre.
    let pointer = (ev.has(EV_REL) && rel.has_all(&[REL_X, REL_Y]) && key.has(BTN_LEFT))
        || (ev.has(EV_ABS) && abs.has_all(&[ABS_X, ABS_Y]) && key.has(BTN_LEFT));
    let touch = ev.has(EV_ABS) && abs.has_all(&[ABS_X, ABS_Y]) && key.has(BTN_TOUCH);

    DeviceKinds {
        keyboard,
        pointer,
        touch,
    }
}

/// One input device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// The character device to read, e.g. `/dev/input/event3`.
    pub path: PathBuf,
    /// The sysfs directory it was found through.
    pub sysfs: PathBuf,
    /// The kernel's name for it.
    pub name: String,
    /// What it can do.
    pub kinds: DeviceKinds,
}

impl DeviceInfo {
    /// A one-line form for the console and the status file.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} [{}] {}",
            self.path.display(),
            self.kinds.label(),
            self.name
        )
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_owned())
}

/// Read one device's description from its sysfs directory.
///
/// `sysfs` is `/sys/class/input/eventN`. The interesting files live under
/// `device/`, which on a real machine is a symlink up to the parent input
/// device — the same physical keyboard can present several event nodes.
#[must_use]
pub fn describe(sysfs: &Path, dev_dir: &Path) -> Option<DeviceInfo> {
    let node = sysfs.file_name()?.to_str()?.to_owned();
    if !node.starts_with("event") {
        return None;
    }
    let device = sysfs.join("device");
    let caps = device.join("capabilities");
    let bit = |name: &str| -> Bitmap {
        read_trimmed(&caps.join(name)).map_or_else(Bitmap::default, |t| Bitmap::parse(&t))
    };

    let kinds = classify(&bit("ev"), &bit("key"), &bit("rel"), &bit("abs"));
    let name = read_trimmed(&device.join("name")).unwrap_or_else(|| node.clone());
    let path = dev_dir.join(&node);

    Some(DeviceInfo {
        path,
        sysfs: sysfs.to_path_buf(),
        name,
        kinds,
    })
}

/// Find every input device under `root`, in a stable order.
///
/// Sorted by node name so that two runs on the same machine agree, which
/// matters because the diagnostics print the list and a shuffling list looks
/// like devices appearing and disappearing.
#[must_use]
pub fn discover(root: &Path) -> Vec<DeviceInfo> {
    let class = root.join("sys/class/input");
    let dev_dir = root.join("dev/input");
    let Ok(entries) = fs::read_dir(&class) else {
        return Vec::new();
    };
    let mut out: Vec<DeviceInfo> = entries
        .flatten()
        .filter_map(|e| describe(&e.path(), &dev_dir))
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The devices worth opening: the ones that can actually drive a desktop.
#[must_use]
pub fn usable(root: &Path) -> Vec<DeviceInfo> {
    discover(root)
        .into_iter()
        .filter(|d| d.kinds.is_useful())
        .collect()
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    /// A real keyboard's `key` bitmap, as printed by a 6.x kernel.
    const KEYBOARD_KEY: &str = "402000000 3803078f800d001 feffffdfffefffff fffffffffffffffe";
    const KEYBOARD_EV: &str = "120013";

    /// A real optical mouse.
    const MOUSE_EV: &str = "17";
    const MOUSE_KEY: &str = "70000 0 0 0 0";
    const MOUSE_REL: &str = "1943";

    /// A power button: EV_KEY, but only KEY_POWER.
    const POWER_EV: &str = "3";
    const POWER_KEY: &str = "10000000000000 0";

    #[test]
    fn a_bitmap_indexes_words_from_the_right() {
        // "1 0" is bit 64 set, not bit 0. Reading it the other way round makes
        // every device look like it reports EV_SYN and nothing else.
        let b = Bitmap::parse("1 0");
        assert!(!b.has(0));
        assert!(b.has(64));
    }

    #[test]
    fn a_single_word_bitmap_reads_its_low_bits() {
        let b = Bitmap::parse("13");
        assert!(b.has(0) && b.has(1) && b.has(4));
        assert!(!b.has(2) && !b.has(3));
    }

    #[test]
    fn a_bit_past_the_end_is_absent_rather_than_a_panic() {
        let b = Bitmap::parse("1");
        assert!(!b.has(u16::MAX));
        assert!(!Bitmap::default().has(0));
    }

    #[test]
    fn junk_in_a_bitmap_reads_as_zero_rather_than_failing() {
        let b = Bitmap::parse("zzz 1");
        assert!(b.has(0), "the readable word still counts");
        assert!(!b.has(64), "the unreadable one is zero, not garbage");
    }

    #[test]
    fn a_keyboard_is_recognised() {
        let k = classify(
            &Bitmap::parse(KEYBOARD_EV),
            &Bitmap::parse(KEYBOARD_KEY),
            &Bitmap::default(),
            &Bitmap::default(),
        );
        assert!(k.keyboard, "a real keyboard was not classified as one");
        assert!(!k.pointer);
        assert_eq!(k.label(), "keyboard");
    }

    #[test]
    fn a_mouse_is_recognised() {
        let k = classify(
            &Bitmap::parse(MOUSE_EV),
            &Bitmap::parse(MOUSE_KEY),
            &Bitmap::parse(MOUSE_REL),
            &Bitmap::default(),
        );
        assert!(k.pointer, "a real mouse was not classified as one");
        assert!(!k.keyboard);
    }

    #[test]
    fn a_power_button_is_not_a_keyboard() {
        // The bug this prevents: opening the power button as a keyboard, then
        // reporting that input is available when nothing can type.
        let k = classify(
            &Bitmap::parse(POWER_EV),
            &Bitmap::parse(POWER_KEY),
            &Bitmap::default(),
            &Bitmap::default(),
        );
        assert!(!k.keyboard, "the power button passed as a keyboard");
        assert!(!k.is_useful());
        assert_eq!(k.label(), "other");
    }

    #[test]
    fn a_touchscreen_is_recognised_and_a_tablet_without_touch_is_not() {
        let abs = Bitmap::parse("3");
        let mut key = Bitmap::parse("0");
        key.words = vec![0, 0, 0, 0, 0, 1 << (BTN_TOUCH as usize % 64)];
        let k = classify(&Bitmap::parse("b"), &key, &Bitmap::default(), &abs);
        assert!(k.touch);

        let no_touch = classify(
            &Bitmap::parse("b"),
            &Bitmap::default(),
            &Bitmap::default(),
            &abs,
        );
        assert!(!no_touch.touch);
    }

    #[test]
    fn discovery_reads_a_fixture_tree_and_sorts_it() {
        let root = std::env::temp_dir().join("litric-input-devices");
        let _ = fs::remove_dir_all(&root);
        for (node, ev, key, rel, name) in [
            ("event5", MOUSE_EV, MOUSE_KEY, MOUSE_REL, "Logitech Mouse"),
            (
                "event1",
                KEYBOARD_EV,
                KEYBOARD_KEY,
                "0",
                "AT Translated Set 2 keyboard",
            ),
            ("event9", POWER_EV, POWER_KEY, "0", "Power Button"),
        ] {
            let caps = root
                .join("sys/class/input")
                .join(node)
                .join("device/capabilities");
            fs::create_dir_all(&caps).expect("fixture dirs");
            fs::write(caps.join("ev"), ev).expect("ev");
            fs::write(caps.join("key"), key).expect("key");
            fs::write(caps.join("rel"), rel).expect("rel");
            fs::write(
                root.join("sys/class/input").join(node).join("device/name"),
                name,
            )
            .expect("name");
        }

        let all = discover(&root);
        assert_eq!(all.len(), 3, "{all:?}");
        assert!(
            all.first()
                .map(|d| d.path.ends_with("event1"))
                .unwrap_or(false),
            "not sorted: {:?}",
            all.iter().map(|d| d.path.clone()).collect::<Vec<_>>()
        );

        let good = usable(&root);
        assert_eq!(good.len(), 2, "the power button should be filtered out");
        assert!(good.iter().any(|d| d.kinds.keyboard));
        assert!(good.iter().any(|d| d.kinds.pointer));
        assert!(good.iter().any(|d| d.name.contains("Logitech")));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_root_with_no_input_class_yields_nothing_rather_than_failing() {
        assert!(discover(Path::new("no-such-root-4b2e")).is_empty());
        assert!(usable(Path::new("no-such-root-4b2e")).is_empty());
    }

    #[test]
    fn a_device_with_no_name_file_still_appears() {
        let root = std::env::temp_dir().join("litric-input-noname");
        let _ = fs::remove_dir_all(&root);
        let caps = root.join("sys/class/input/event2/device/capabilities");
        fs::create_dir_all(&caps).expect("dirs");
        fs::write(caps.join("ev"), MOUSE_EV).expect("ev");
        fs::write(caps.join("key"), MOUSE_KEY).expect("key");
        fs::write(caps.join("rel"), MOUSE_REL).expect("rel");

        let found = discover(&root);
        assert_eq!(found.len(), 1);
        assert_eq!(found.first().map(|d| d.name.as_str()), Some("event2"));
        fs::remove_dir_all(&root).ok();
    }
}
