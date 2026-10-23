//! Keyboard events as the web platform names them.
//!
//! Chromium's input API wants three names for every key event: the DOM `key`
//! (what the key means: `"Enter"`, `"a"`, `"ArrowLeft"`), the DOM `code` (which
//! physical key: `"KeyA"`, `"Enter"`) and the Windows virtual-key code, which
//! Blink still consults to decide what Enter, Backspace and the arrows do
//! inside a text field. The shell has a Linux scancode and a layout-resolved
//! [`Key`]; this module turns them into those names.
//!
//! Written from the W3C *UI Events KeyboardEvent code Values* and *key Values*
//! specifications, the Linux input-event-codes UAPI numbering, and Microsoft's
//! published virtual-key code table. Positions follow the US layout, which is
//! what `code` is defined against.

use litric_input::keys::{Key, Modifiers, Side};

/// The physical-key name and the Windows virtual-key code for a Linux
/// scancode, or `None` for a key with no DOM `code`.
#[must_use]
pub const fn physical(scancode: u16) -> Option<(&'static str, u16)> {
    Some(match scancode {
        1 => ("Escape", 0x1B),
        2 => ("Digit1", 0x31),
        3 => ("Digit2", 0x32),
        4 => ("Digit3", 0x33),
        5 => ("Digit4", 0x34),
        6 => ("Digit5", 0x35),
        7 => ("Digit6", 0x36),
        8 => ("Digit7", 0x37),
        9 => ("Digit8", 0x38),
        10 => ("Digit9", 0x39),
        11 => ("Digit0", 0x30),
        12 => ("Minus", 0xBD),
        13 => ("Equal", 0xBB),
        14 => ("Backspace", 0x08),
        15 => ("Tab", 0x09),
        16 => ("KeyQ", 0x51),
        17 => ("KeyW", 0x57),
        18 => ("KeyE", 0x45),
        19 => ("KeyR", 0x52),
        20 => ("KeyT", 0x54),
        21 => ("KeyY", 0x59),
        22 => ("KeyU", 0x55),
        23 => ("KeyI", 0x49),
        24 => ("KeyO", 0x4F),
        25 => ("KeyP", 0x50),
        26 => ("BracketLeft", 0xDB),
        27 => ("BracketRight", 0xDD),
        28 => ("Enter", 0x0D),
        29 => ("ControlLeft", 0x11),
        30 => ("KeyA", 0x41),
        31 => ("KeyS", 0x53),
        32 => ("KeyD", 0x44),
        33 => ("KeyF", 0x46),
        34 => ("KeyG", 0x47),
        35 => ("KeyH", 0x48),
        36 => ("KeyJ", 0x4A),
        37 => ("KeyK", 0x4B),
        38 => ("KeyL", 0x4C),
        39 => ("Semicolon", 0xBA),
        40 => ("Quote", 0xDE),
        41 => ("Backquote", 0xC0),
        42 => ("ShiftLeft", 0x10),
        43 => ("Backslash", 0xDC),
        44 => ("KeyZ", 0x5A),
        45 => ("KeyX", 0x58),
        46 => ("KeyC", 0x43),
        47 => ("KeyV", 0x56),
        48 => ("KeyB", 0x42),
        49 => ("KeyN", 0x4E),
        50 => ("KeyM", 0x4D),
        51 => ("Comma", 0xBC),
        52 => ("Period", 0xBE),
        53 => ("Slash", 0xBF),
        54 => ("ShiftRight", 0x10),
        55 => ("NumpadMultiply", 0x6A),
        56 => ("AltLeft", 0x12),
        57 => ("Space", 0x20),
        58 => ("CapsLock", 0x14),
        59 => ("F1", 0x70),
        60 => ("F2", 0x71),
        61 => ("F3", 0x72),
        62 => ("F4", 0x73),
        63 => ("F5", 0x74),
        64 => ("F6", 0x75),
        65 => ("F7", 0x76),
        66 => ("F8", 0x77),
        67 => ("F9", 0x78),
        68 => ("F10", 0x79),
        69 => ("NumLock", 0x90),
        70 => ("ScrollLock", 0x91),
        71 => ("Numpad7", 0x67),
        72 => ("Numpad8", 0x68),
        73 => ("Numpad9", 0x69),
        74 => ("NumpadSubtract", 0x6D),
        75 => ("Numpad4", 0x64),
        76 => ("Numpad5", 0x65),
        77 => ("Numpad6", 0x66),
        78 => ("NumpadAdd", 0x6B),
        79 => ("Numpad1", 0x61),
        80 => ("Numpad2", 0x62),
        81 => ("Numpad3", 0x63),
        82 => ("Numpad0", 0x60),
        83 => ("NumpadDecimal", 0x6E),
        86 => ("IntlBackslash", 0xE2),
        87 => ("F11", 0x7A),
        88 => ("F12", 0x7B),
        96 => ("NumpadEnter", 0x0D),
        97 => ("ControlRight", 0x11),
        98 => ("NumpadDivide", 0x6F),
        99 => ("PrintScreen", 0x2C),
        100 => ("AltRight", 0x12),
        102 => ("Home", 0x24),
        103 => ("ArrowUp", 0x26),
        104 => ("PageUp", 0x21),
        105 => ("ArrowLeft", 0x25),
        106 => ("ArrowRight", 0x27),
        107 => ("End", 0x23),
        108 => ("ArrowDown", 0x28),
        109 => ("PageDown", 0x22),
        110 => ("Insert", 0x2D),
        111 => ("Delete", 0x2E),
        113 => ("AudioVolumeMute", 0xAD),
        114 => ("AudioVolumeDown", 0xAE),
        115 => ("AudioVolumeUp", 0xAF),
        119 => ("Pause", 0x13),
        125 => ("MetaLeft", 0x5B),
        126 => ("MetaRight", 0x5C),
        127 => ("ContextMenu", 0x5D),
        183 => ("F13", 0x7C),
        184 => ("F14", 0x7D),
        185 => ("F15", 0x7E),
        186 => ("F16", 0x7F),
        187 => ("F17", 0x80),
        188 => ("F18", 0x81),
        189 => ("F19", 0x82),
        190 => ("F20", 0x83),
        191 => ("F21", 0x84),
        192 => ("F22", 0x85),
        193 => ("F23", 0x86),
        194 => ("F24", 0x87),
        _ => return None,
    })
}

/// The DOM `key` value: the typed text for a printing key, otherwise the
/// named-key value from the specification.
#[must_use]
pub fn dom_key(key: Key, text: Option<&str>) -> String {
    let named = match key {
        Key::Escape => "Escape",
        Key::Enter => "Enter",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Delete => "Delete",
        Key::Insert => "Insert",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::Left => "ArrowLeft",
        Key::Right => "ArrowRight",
        Key::Up => "ArrowUp",
        Key::Down => "ArrowDown",
        Key::Function(n) => return format!("F{n}"),
        Key::CapsLock => "CapsLock",
        Key::NumLock => "NumLock",
        Key::ScrollLock => "ScrollLock",
        Key::PrintScreen => "PrintScreen",
        Key::Pause => "Pause",
        Key::Menu => "ContextMenu",
        Key::Shift(_) => "Shift",
        Key::Ctrl(_) => "Control",
        Key::Alt(Side::Left) => "Alt",
        Key::Alt(Side::Right) => "AltGraph",
        Key::Meta(_) => "Meta",
        Key::Mute => "AudioVolumeMute",
        Key::VolumeDown => "AudioVolumeDown",
        Key::VolumeUp => "AudioVolumeUp",
        Key::BrightnessDown => "BrightnessDown",
        Key::BrightnessUp => "BrightnessUp",
        Key::Power => "Power",
        Key::Char(c) => {
            return match text.filter(|t| !t.is_empty()) {
                Some(t) => t.to_owned(),
                None => c.to_string(),
            }
        }
        // Key is non-exhaustive; an unnamed key is what the specification
        // calls "Unidentified".
        _ => "Unidentified",
    };
    named.to_owned()
}

/// The text a key event carries to Chromium, if any.
///
/// Enter carries a carriage return, which is what makes it submit a form or
/// break a line in a text area; every other non-printing key carries none.
#[must_use]
pub fn event_text(key: Key, typed: Option<&str>) -> Option<String> {
    match key {
        Key::Enter => Some("\r".to_owned()),
        Key::Char(_) => typed.filter(|t| !t.is_empty()).map(str::to_owned),
        _ => None,
    }
}

/// The CDP modifier bitmask: Alt 1, Ctrl 2, Meta 4, Shift 8.
///
/// AltGr is deliberately not reported as Alt: on layouts that have it, it
/// composes characters, and a page that sees Alt held treats the keystroke as
/// a shortcut instead of typing.
#[must_use]
pub const fn cdp_modifiers(mods: Modifiers) -> u8 {
    let mut m = 0u8;
    if mods.alt {
        m |= 1;
    }
    if mods.ctrl {
        m |= 2;
    }
    if mods.meta {
        m |= 4;
    }
    if mods.shift {
        m |= 8;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_digits_and_arrows_have_their_specified_names() {
        assert_eq!(physical(30), Some(("KeyA", 0x41)));
        assert_eq!(physical(44), Some(("KeyZ", 0x5A)));
        assert_eq!(physical(2), Some(("Digit1", 0x31)));
        assert_eq!(physical(11), Some(("Digit0", 0x30)));
        assert_eq!(physical(105), Some(("ArrowLeft", 0x25)));
        assert_eq!(physical(108), Some(("ArrowDown", 0x28)));
        assert_eq!(physical(28), Some(("Enter", 0x0D)));
        assert_eq!(physical(96), Some(("NumpadEnter", 0x0D)));
        assert_eq!(physical(57), Some(("Space", 0x20)));
    }

    #[test]
    fn every_letter_maps_to_its_uppercase_ascii_virtual_key() {
        // The US layout's letter rows, by scancode. The virtual-key code of a
        // letter is its uppercase ASCII value, so the code name and the VK must
        // agree for every one of them.
        for sc in (16..=25).chain(30..=38).chain(44..=50) {
            let (code, vk) = physical(sc).expect("letter scancode");
            let letter = code.strip_prefix("Key").expect("a Key* code");
            let ascii = u16::from(*letter.as_bytes().first().expect("one letter"));
            assert_eq!(vk, ascii, "{code}");
        }
    }

    #[test]
    fn function_keys_run_contiguously_including_f13_to_f24() {
        for (n, sc) in (1u16..=10).zip(59u16..=68) {
            assert_eq!(physical(sc).map(|p| p.1), Some(0x6F + n));
        }
        assert_eq!(physical(87), Some(("F11", 0x7A)));
        assert_eq!(physical(88), Some(("F12", 0x7B)));
        assert_eq!(physical(194), Some(("F24", 0x87)));
    }

    #[test]
    fn an_unknown_scancode_has_no_physical_name() {
        assert_eq!(physical(0), None);
        assert_eq!(physical(84), None);
        assert_eq!(physical(u16::MAX), None);
    }

    #[test]
    fn a_printing_key_is_named_by_what_it_typed() {
        // Shift+a types "A"; the DOM key is the typed text, not the key cap.
        assert_eq!(dom_key(Key::Char('a'), Some("A")), "A");
        assert_eq!(dom_key(Key::Char('a'), None), "a");
        assert_eq!(dom_key(Key::Char(' '), Some(" ")), " ");
        assert_eq!(dom_key(Key::Char('ก'), Some("ก")), "ก");
    }

    #[test]
    fn named_keys_use_the_specification_values() {
        assert_eq!(dom_key(Key::Left, None), "ArrowLeft");
        assert_eq!(dom_key(Key::Backspace, None), "Backspace");
        assert_eq!(dom_key(Key::Ctrl(Side::Left), None), "Control");
        assert_eq!(dom_key(Key::Menu, None), "ContextMenu");
        assert_eq!(dom_key(Key::Function(5), None), "F5");
        assert_eq!(dom_key(Key::Unknown(250), None), "Unidentified");
        // A named key never takes text, even if some arrives.
        assert_eq!(dom_key(Key::Tab, Some("\t")), "Tab");
    }

    #[test]
    fn enter_carries_a_carriage_return_and_other_named_keys_carry_nothing() {
        assert_eq!(event_text(Key::Enter, None).as_deref(), Some("\r"));
        assert_eq!(event_text(Key::Backspace, None), None);
        assert_eq!(event_text(Key::Tab, Some("\t")), None);
        assert_eq!(event_text(Key::Char('x'), Some("x")).as_deref(), Some("x"));
        assert_eq!(event_text(Key::Char('x'), None), None);
    }

    #[test]
    fn modifiers_map_to_the_cdp_bits_and_altgr_is_not_alt() {
        let all = Modifiers {
            shift: true,
            ctrl: true,
            alt: true,
            meta: true,
            ..Modifiers::NONE
        };
        assert_eq!(cdp_modifiers(all), 15);
        assert_eq!(cdp_modifiers(Modifiers::NONE), 0);
        let altgr = Modifiers {
            altgr: true,
            ..Modifiers::NONE
        };
        assert_eq!(cdp_modifiers(altgr), 0);
        let ctrl_shift = Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(cdp_modifiers(ctrl_shift), 10);
    }
}
