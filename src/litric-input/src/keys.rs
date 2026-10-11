//! Keys, modifiers and keyboard layouts.
//!
//! The kernel reports a **scancode**: a number for a physical key, the same
//! number whichever alphabet is printed on it. Turning that into "the user
//! typed é" is the layout's job, and the layout is data, not code, so that
//! adding Thai or German is adding a table rather than editing a renderer.
//!
//! Two different questions are being answered here and they must not be
//! confused, because confusing them is why some desktops put shortcuts on the
//! wrong keys for anyone outside the United States:
//!
//! * **Which key is this, for a shortcut?** [`Key`], resolved through the
//!   layout's unshifted character. `Win+1` is the key that types `1`.
//! * **What did the user type, for a text field?** [`Layout::text`], which
//!   applies shift and caps lock and returns nothing at all for a key that
//!   produces no character.

use core::fmt;

/// Which side of the keyboard a paired modifier is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Left.
    Left,
    /// Right.
    Right,
}

/// A key, after the layout has been consulted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    /// A key that produces this character with no modifiers held.
    Char(char),
    /// Escape.
    Escape,
    /// Return or keypad enter.
    Enter,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Insert.
    Insert,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// A function key, 1-24.
    Function(u8),
    /// Caps lock.
    CapsLock,
    /// Num lock.
    NumLock,
    /// Scroll lock.
    ScrollLock,
    /// Print screen / SysRq.
    PrintScreen,
    /// Pause / Break.
    Pause,
    /// The context-menu key.
    Menu,
    /// Shift.
    Shift(Side),
    /// Control.
    Ctrl(Side),
    /// Alt. The right-hand one is AltGr on most layouts.
    Alt(Side),
    /// The Windows / Command / Super key. LitricOS calls it Meta and uses it
    /// for system shortcuts, which is what makes `Win+1` a system action rather
    /// than something an application can steal.
    Meta(Side),
    /// Mute.
    Mute,
    /// Volume down.
    VolumeDown,
    /// Volume up.
    VolumeUp,
    /// Brightness down.
    BrightnessDown,
    /// Brightness up.
    BrightnessUp,
    /// Power button.
    Power,
    /// A key this build has no name for. Carried rather than dropped so a
    /// diagnostic can show what an unrecognised keyboard is sending.
    Unknown(u16),
}

impl Key {
    /// Whether this key is a modifier rather than something to act on.
    #[must_use]
    pub const fn is_modifier(self) -> bool {
        matches!(
            self,
            Self::Shift(_) | Self::Ctrl(_) | Self::Alt(_) | Self::Meta(_)
        )
    }

    /// A short name for shortcut labels and diagnostics.
    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Char(c) => c.to_uppercase().collect(),
            Self::Function(n) => format!("F{n}"),
            Self::Escape => "Esc".to_owned(),
            Self::Enter => "Enter".to_owned(),
            Self::Tab => "Tab".to_owned(),
            Self::Backspace => "Backspace".to_owned(),
            Self::Delete => "Del".to_owned(),
            Self::Insert => "Ins".to_owned(),
            Self::Home => "Home".to_owned(),
            Self::End => "End".to_owned(),
            Self::PageUp => "PgUp".to_owned(),
            Self::PageDown => "PgDn".to_owned(),
            Self::Left => "Left".to_owned(),
            Self::Right => "Right".to_owned(),
            Self::Up => "Up".to_owned(),
            Self::Down => "Down".to_owned(),
            Self::CapsLock => "CapsLock".to_owned(),
            Self::NumLock => "NumLock".to_owned(),
            Self::ScrollLock => "ScrollLock".to_owned(),
            Self::PrintScreen => "PrtSc".to_owned(),
            Self::Pause => "Pause".to_owned(),
            Self::Menu => "Menu".to_owned(),
            Self::Shift(_) => "Shift".to_owned(),
            Self::Ctrl(_) => "Ctrl".to_owned(),
            Self::Alt(_) => "Alt".to_owned(),
            Self::Meta(_) => "Win".to_owned(),
            Self::Mute => "Mute".to_owned(),
            Self::VolumeDown => "Vol-".to_owned(),
            Self::VolumeUp => "Vol+".to_owned(),
            Self::BrightnessDown => "Bright-".to_owned(),
            Self::BrightnessUp => "Bright+".to_owned(),
            Self::Power => "Power".to_owned(),
            Self::Unknown(c) => format!("key{c}"),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

/// Which modifiers are held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    /// Either shift.
    pub shift: bool,
    /// Either control.
    pub ctrl: bool,
    /// Left alt.
    pub alt: bool,
    /// Right alt, which is AltGr on layouts that have one. Tracked separately
    /// because on those layouts it composes characters rather than acting as a
    /// modifier, and treating it as Alt breaks typing in half of Europe.
    pub altgr: bool,
    /// Either Meta / Windows / Super.
    pub meta: bool,
    /// Caps lock is on.
    pub caps: bool,
    /// Num lock is on.
    pub num: bool,
}

impl Modifiers {
    /// No modifiers.
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        altgr: false,
        meta: false,
        caps: false,
        num: false,
    };

    /// Whether any modifier that changes the meaning of a key is held.
    ///
    /// Caps and num lock are excluded: they change which *character* a key
    /// produces, not which *action* it is.
    #[must_use]
    pub const fn any_action_modifier(self) -> bool {
        self.ctrl || self.alt || self.meta
    }

    /// The modifiers that identify a shortcut, ignoring lock state and the
    /// side a key was pressed on.
    #[must_use]
    pub const fn normalised(self) -> Self {
        Self {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            altgr: false,
            meta: self.meta,
            caps: false,
            num: false,
        }
    }

    /// A label like `Win+Shift+` for a shortcut hint.
    #[must_use]
    pub fn prefix(self) -> String {
        let mut s = String::new();
        if self.meta {
            s.push_str("Win+");
        }
        if self.ctrl {
            s.push_str("Ctrl+");
        }
        if self.alt {
            s.push_str("Alt+");
        }
        if self.shift {
            s.push_str("Shift+");
        }
        s
    }
}

/// One physical key's characters on a given layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyDef {
    /// Kernel scancode.
    pub code: u16,
    /// The character with no modifiers.
    pub plain: char,
    /// The character with shift held.
    pub shifted: char,
}

/// A keyboard layout: which characters the letter keys produce.
///
/// Data rather than code so that a new layout is a new table, added by the
/// localisation layer, and never a branch in the input handler.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Identifier, e.g. `us`, `th`, `de`.
    pub name: &'static str,
    /// Human-readable name.
    pub display: &'static str,
    /// The character-producing keys.
    pub keys: &'static [KeyDef],
}

macro_rules! kd {
    ($code:expr, $plain:expr, $shifted:expr) => {
        KeyDef {
            code: $code,
            plain: $plain,
            shifted: $shifted,
        }
    };
}

/// US QWERTY. The development default, and the fallback when the configured
/// layout is unknown.
pub const US_QWERTY: Layout = Layout {
    name: "us",
    display: "English (US)",
    keys: &[
        kd!(2, '1', '!'),
        kd!(3, '2', '@'),
        kd!(4, '3', '#'),
        kd!(5, '4', '$'),
        kd!(6, '5', '%'),
        kd!(7, '6', '^'),
        kd!(8, '7', '&'),
        kd!(9, '8', '*'),
        kd!(10, '9', '('),
        kd!(11, '0', ')'),
        kd!(12, '-', '_'),
        kd!(13, '=', '+'),
        kd!(16, 'q', 'Q'),
        kd!(17, 'w', 'W'),
        kd!(18, 'e', 'E'),
        kd!(19, 'r', 'R'),
        kd!(20, 't', 'T'),
        kd!(21, 'y', 'Y'),
        kd!(22, 'u', 'U'),
        kd!(23, 'i', 'I'),
        kd!(24, 'o', 'O'),
        kd!(25, 'p', 'P'),
        kd!(26, '[', '{'),
        kd!(27, ']', '}'),
        kd!(30, 'a', 'A'),
        kd!(31, 's', 'S'),
        kd!(32, 'd', 'D'),
        kd!(33, 'f', 'F'),
        kd!(34, 'g', 'G'),
        kd!(35, 'h', 'H'),
        kd!(36, 'j', 'J'),
        kd!(37, 'k', 'K'),
        kd!(38, 'l', 'L'),
        kd!(39, ';', ':'),
        kd!(40, '\'', '"'),
        kd!(41, '`', '~'),
        kd!(43, '\\', '|'),
        kd!(44, 'z', 'Z'),
        kd!(45, 'x', 'X'),
        kd!(46, 'c', 'C'),
        kd!(47, 'v', 'V'),
        kd!(48, 'b', 'B'),
        kd!(49, 'n', 'N'),
        kd!(50, 'm', 'M'),
        kd!(51, ',', '<'),
        kd!(52, '.', '>'),
        kd!(53, '/', '?'),
        kd!(57, ' ', ' '),
    ],
};

/// UK QWERTY. Differs from US on eight keys, all of them ones people notice.
pub const UK_QWERTY: Layout = Layout {
    name: "gb",
    display: "English (UK)",
    keys: &[
        kd!(2, '1', '!'),
        kd!(3, '2', '"'),
        kd!(4, '3', '\u{00A3}'),
        kd!(5, '4', '$'),
        kd!(6, '5', '%'),
        kd!(7, '6', '^'),
        kd!(8, '7', '&'),
        kd!(9, '8', '*'),
        kd!(10, '9', '('),
        kd!(11, '0', ')'),
        kd!(12, '-', '_'),
        kd!(13, '=', '+'),
        kd!(16, 'q', 'Q'),
        kd!(17, 'w', 'W'),
        kd!(18, 'e', 'E'),
        kd!(19, 'r', 'R'),
        kd!(20, 't', 'T'),
        kd!(21, 'y', 'Y'),
        kd!(22, 'u', 'U'),
        kd!(23, 'i', 'I'),
        kd!(24, 'o', 'O'),
        kd!(25, 'p', 'P'),
        kd!(26, '[', '{'),
        kd!(27, ']', '}'),
        kd!(30, 'a', 'A'),
        kd!(31, 's', 'S'),
        kd!(32, 'd', 'D'),
        kd!(33, 'f', 'F'),
        kd!(34, 'g', 'G'),
        kd!(35, 'h', 'H'),
        kd!(36, 'j', 'J'),
        kd!(37, 'k', 'K'),
        kd!(38, 'l', 'L'),
        kd!(39, ';', ':'),
        kd!(40, '\'', '@'),
        kd!(41, '`', '\u{00AC}'),
        kd!(43, '#', '~'),
        kd!(86, '\\', '|'),
        kd!(44, 'z', 'Z'),
        kd!(45, 'x', 'X'),
        kd!(46, 'c', 'C'),
        kd!(47, 'v', 'V'),
        kd!(48, 'b', 'B'),
        kd!(49, 'n', 'N'),
        kd!(50, 'm', 'M'),
        kd!(51, ',', '<'),
        kd!(52, '.', '>'),
        kd!(53, '/', '?'),
        kd!(57, ' ', ' '),
    ],
};

/// Every layout this build knows.
pub const LAYOUTS: &[Layout] = &[US_QWERTY, UK_QWERTY];

/// Look a layout up by name, e.g. from the locale.
///
/// Returns `None` rather than silently falling back, so the caller can say in
/// its diagnostics that the configured layout was not found instead of quietly
/// typing the wrong characters.
#[must_use]
pub fn layout_by_name(name: &str) -> Option<Layout> {
    LAYOUTS.iter().copied().find(|l| l.name == name)
}

impl Layout {
    /// The character this key produces with no modifiers, if any.
    #[must_use]
    pub fn plain(&self, code: u16) -> Option<char> {
        self.keys.iter().find(|k| k.code == code).map(|k| k.plain)
    }

    /// The text this key produces with these modifiers, if any.
    ///
    /// Returns `None` for a key that types nothing, and for any combination
    /// held with Ctrl or Meta: `Ctrl+C` is a command, not the letter c, and a
    /// text field that receives "c" from it is a text field that corrupts the
    /// document every time somebody copies.
    #[must_use]
    pub fn text(&self, code: u16, mods: Modifiers) -> Option<char> {
        if mods.ctrl || mods.meta {
            return None;
        }
        let def = self.keys.iter().find(|k| k.code == code)?;
        // Caps lock affects letters only. Applying it to the number row is a
        // classic bug that turns "1" into "!" whenever caps is on.
        let is_letter = def.plain.is_alphabetic();
        let upper = if is_letter {
            mods.shift ^ mods.caps
        } else {
            mods.shift
        };
        Some(if upper { def.shifted } else { def.plain })
    }

    /// Resolve a scancode into a [`Key`].
    #[must_use]
    pub fn key(&self, code: u16) -> Key {
        if let Some(c) = self.plain(code) {
            return Key::Char(c);
        }
        named_key(code)
    }
}

/// Keys that are the same on every layout because they are not characters.
#[must_use]
pub fn named_key(code: u16) -> Key {
    match code {
        1 => Key::Escape,
        14 => Key::Backspace,
        15 => Key::Tab,
        28 | 96 => Key::Enter,
        29 => Key::Ctrl(Side::Left),
        42 => Key::Shift(Side::Left),
        54 => Key::Shift(Side::Right),
        56 => Key::Alt(Side::Left),
        58 => Key::CapsLock,
        59..=68 => Key::Function((code.saturating_sub(58)) as u8),
        69 => Key::NumLock,
        70 => Key::ScrollLock,
        87 => Key::Function(11),
        88 => Key::Function(12),
        97 => Key::Ctrl(Side::Right),
        99 => Key::PrintScreen,
        100 => Key::Alt(Side::Right),
        102 => Key::Home,
        103 => Key::Up,
        104 => Key::PageUp,
        105 => Key::Left,
        106 => Key::Right,
        107 => Key::End,
        108 => Key::Down,
        109 => Key::PageDown,
        110 => Key::Insert,
        111 => Key::Delete,
        113 => Key::Mute,
        114 => Key::VolumeDown,
        115 => Key::VolumeUp,
        116 => Key::Power,
        119 => Key::Pause,
        125 => Key::Meta(Side::Left),
        126 => Key::Meta(Side::Right),
        139 => Key::Menu,
        224 => Key::BrightnessDown,
        225 => Key::BrightnessUp,
        other => Key::Unknown(other),
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn letters_resolve_through_the_layout() {
        assert_eq!(US_QWERTY.key(30), Key::Char('a'));
        assert_eq!(US_QWERTY.key(16), Key::Char('q'));
        assert_eq!(US_QWERTY.key(2), Key::Char('1'));
    }

    #[test]
    fn non_character_keys_are_layout_independent() {
        for l in LAYOUTS {
            assert_eq!(l.key(1), Key::Escape);
            assert_eq!(l.key(125), Key::Meta(Side::Left));
            assert_eq!(l.key(15), Key::Tab);
            assert_eq!(l.key(59), Key::Function(1));
            assert_eq!(l.key(88), Key::Function(12));
        }
    }

    #[test]
    fn shift_produces_the_shifted_character() {
        let shift = Modifiers {
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(US_QWERTY.text(30, shift), Some('A'));
        assert_eq!(US_QWERTY.text(2, shift), Some('!'));
    }

    #[test]
    fn caps_lock_affects_letters_and_leaves_the_number_row_alone() {
        // The bug this prevents: caps lock turning "1" into "!" and every
        // password with a digit in it becoming unenterable.
        let caps = Modifiers {
            caps: true,
            ..Modifiers::NONE
        };
        assert_eq!(US_QWERTY.text(30, caps), Some('A'), "letter uppercases");
        assert_eq!(US_QWERTY.text(2, caps), Some('1'), "digit unchanged");
    }

    #[test]
    fn shift_and_caps_together_cancel_for_letters() {
        let both = Modifiers {
            shift: true,
            caps: true,
            ..Modifiers::NONE
        };
        assert_eq!(US_QWERTY.text(30, both), Some('a'));
        assert_eq!(US_QWERTY.text(2, both), Some('!'), "digit still shifts");
    }

    #[test]
    fn a_command_is_not_text() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        };
        let meta = Modifiers {
            meta: true,
            ..Modifiers::NONE
        };
        assert_eq!(US_QWERTY.text(46, ctrl), None, "Ctrl+C must not type c");
        assert_eq!(US_QWERTY.text(2, meta), None, "Win+1 must not type 1");
    }

    #[test]
    fn a_key_with_no_character_produces_no_text() {
        assert_eq!(US_QWERTY.text(1, Modifiers::NONE), None, "Escape");
        assert_eq!(US_QWERTY.text(103, Modifiers::NONE), None, "Up arrow");
    }

    #[test]
    fn the_uk_layout_differs_where_it_should_and_agrees_elsewhere() {
        let shift = Modifiers {
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(UK_QWERTY.text(3, shift), Some('"'), "UK Shift+2");
        assert_eq!(US_QWERTY.text(3, shift), Some('@'), "US Shift+2");
        assert_eq!(UK_QWERTY.text(30, Modifiers::NONE), Some('a'));
        assert_eq!(US_QWERTY.text(30, Modifiers::NONE), Some('a'));
    }

    #[test]
    fn a_shortcut_key_is_the_same_on_both_layouts_where_the_key_is_the_same() {
        // Win+1 has to be Win+1 everywhere, which is why shortcuts resolve
        // through the unshifted character rather than the shifted one.
        assert_eq!(US_QWERTY.key(2), UK_QWERTY.key(2));
        assert_eq!(US_QWERTY.key(2), Key::Char('1'));
    }

    #[test]
    fn every_layout_covers_the_whole_alphabet_and_the_digits() {
        for l in LAYOUTS {
            for want in "abcdefghijklmnopqrstuvwxyz0123456789".chars() {
                assert!(
                    l.keys.iter().any(|k| k.plain == want),
                    "layout {} has no key for {want:?}",
                    l.name
                );
            }
        }
    }

    #[test]
    fn no_layout_maps_one_scancode_twice() {
        for l in LAYOUTS {
            let mut codes: Vec<u16> = l.keys.iter().map(|k| k.code).collect();
            codes.sort_unstable();
            let before = codes.len();
            codes.dedup();
            assert_eq!(before, codes.len(), "layout {} has a duplicate key", l.name);
        }
    }

    #[test]
    fn layouts_are_found_by_name_and_missing_ones_are_reported_not_guessed() {
        assert_eq!(layout_by_name("us").map(|l| l.name), Some("us"));
        assert_eq!(layout_by_name("gb").map(|l| l.name), Some("gb"));
        assert!(layout_by_name("th").is_none(), "Thai is not shipped yet");
    }

    #[test]
    fn modifier_keys_are_recognised_as_modifiers() {
        assert!(Key::Shift(Side::Left).is_modifier());
        assert!(Key::Meta(Side::Right).is_modifier());
        assert!(!Key::Char('a').is_modifier());
        assert!(!Key::Enter.is_modifier());
    }

    #[test]
    fn normalising_drops_lock_state_but_keeps_the_real_modifiers() {
        let m = Modifiers {
            shift: true,
            meta: true,
            caps: true,
            num: true,
            altgr: true,
            ..Modifiers::NONE
        };
        let n = m.normalised();
        assert!(n.shift && n.meta);
        assert!(!n.caps && !n.num && !n.altgr);
    }

    #[test]
    fn shortcut_labels_read_the_way_they_are_written() {
        let m = Modifiers {
            meta: true,
            ..Modifiers::NONE
        };
        assert_eq!(format!("{}{}", m.prefix(), Key::Char('1')), "Win+1");
        let alt = Modifiers {
            alt: true,
            ..Modifiers::NONE
        };
        assert_eq!(format!("{}{}", alt.prefix(), Key::Tab), "Alt+Tab");
    }

    #[test]
    fn an_unknown_scancode_is_carried_not_dropped() {
        assert_eq!(named_key(0xFFF), Key::Unknown(0xFFF));
        assert_eq!(US_QWERTY.key(0xFFF), Key::Unknown(0xFFF));
    }
}
