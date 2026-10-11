//! The events the desktop actually consumes.
//!
//! Nothing above this module knows what evdev is. That is the point: the
//! desktop reacts to "the pointer moved to (412, 88)" and "the user typed é",
//! never to `EV_REL/REL_X/-7`, so replacing the backend — libinput, a Wayland
//! seat, a remote session, a test harness — changes one crate and nothing else.

use crate::keys::{Key, Modifiers};

/// A pointer button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PointerButton {
    /// Primary, usually left.
    Primary,
    /// Secondary, usually right. Opens context menus.
    Secondary,
    /// Middle, usually the wheel.
    Middle,
    /// Back.
    Back,
    /// Forward.
    Forward,
}

impl PointerButton {
    /// A short name for diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Middle => "middle",
            Self::Back => "back",
            Self::Forward => "forward",
        }
    }
}

/// A point in display coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// Horizontal.
    pub x: f32,
    /// Vertical.
    pub y: f32,
}

impl Point {
    /// Construct.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Distance to another point.
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Something the user did.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum InputEvent {
    /// A key went down. `repeat` is the kernel's auto-repeat.
    KeyDown {
        /// Which key, after the layout.
        key: Key,
        /// The raw scancode, for diagnostics and for remapping.
        code: u16,
        /// Modifiers held at the moment it went down.
        mods: Modifiers,
        /// Whether this is an auto-repeat rather than a fresh press.
        repeat: bool,
    },
    /// A key came up.
    KeyUp {
        /// Which key.
        key: Key,
        /// The raw scancode.
        code: u16,
        /// Modifiers still held.
        mods: Modifiers,
    },
    /// The user typed something. Separate from [`Self::KeyDown`] because a text
    /// field wants characters and a shortcut wants keys, and a system that
    /// conflates them either eats shortcuts or types into documents when you
    /// press Ctrl+S.
    TextInput {
        /// What was typed. One character today; a string so that a future input
        /// method can commit a whole syllable at once.
        text: String,
    },
    /// The pointer moved.
    PointerMove {
        /// Where it is now, already clamped to the display.
        at: Point,
        /// How far it moved. Not `at` minus the last `at`: at the edge of the
        /// screen the position stops changing while the device keeps moving,
        /// and gestures need the real delta.
        delta: Point,
    },
    /// A pointer button changed.
    PointerButton {
        /// Which button.
        button: PointerButton,
        /// Down or up.
        pressed: bool,
        /// Where the pointer was.
        at: Point,
        /// 1 for a single click, 2 for a double, 3 for a triple. Only
        /// meaningful when `pressed` is true.
        clicks: u8,
        /// Modifiers held, so Shift-click and Ctrl-click work.
        mods: Modifiers,
    },
    /// A scroll wheel or gesture. Positive `y` scrolls the content down.
    Scroll {
        /// Horizontal, in lines.
        dx: f32,
        /// Vertical, in lines.
        dy: f32,
        /// Where the pointer was.
        at: Point,
        /// Modifiers, so Ctrl+scroll can zoom.
        mods: Modifiers,
    },
    /// An input device appeared.
    DeviceAdded {
        /// Its name.
        name: String,
    },
    /// An input device went away.
    DeviceRemoved {
        /// Its name.
        name: String,
    },
}

impl InputEvent {
    /// Whether this event should wake the desktop out of an idle state.
    ///
    /// Everything a person does counts. Device add and remove do not: hotplug
    /// noise from a docking station should not keep a machine awake.
    #[must_use]
    pub const fn is_user_activity(&self) -> bool {
        matches!(
            self,
            Self::KeyDown { .. }
                | Self::KeyUp { .. }
                | Self::TextInput { .. }
                | Self::PointerMove { .. }
                | Self::PointerButton { .. }
                | Self::Scroll { .. }
        )
    }

    /// A one-line form for the diagnostic overlay.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::KeyDown {
                key, mods, repeat, ..
            } => format!(
                "key down {}{}{}",
                mods.prefix(),
                key,
                if *repeat { " (repeat)" } else { "" }
            ),
            Self::KeyUp { key, .. } => format!("key up {key}"),
            Self::TextInput { text } => format!("text {text:?}"),
            Self::PointerMove { at, .. } => format!("move {:.0},{:.0}", at.x, at.y),
            Self::PointerButton {
                button,
                pressed,
                clicks,
                ..
            } => format!(
                "{} {}{}",
                button.name(),
                if *pressed { "down" } else { "up" },
                if *pressed && *clicks > 1 {
                    format!(" x{clicks}")
                } else {
                    String::new()
                }
            ),
            Self::Scroll { dx, dy, .. } => format!("scroll {dx:+.1},{dy:+.1}"),
            Self::DeviceAdded { name } => format!("device + {name}"),
            Self::DeviceRemoved { name } => format!("device - {name}"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::keys::Modifiers;

    #[test]
    fn user_activity_is_what_a_person_did() {
        let key = InputEvent::KeyDown {
            key: Key::Char('a'),
            code: 30,
            mods: Modifiers::NONE,
            repeat: false,
        };
        assert!(key.is_user_activity());
        assert!(InputEvent::PointerMove {
            at: Point::new(1.0, 2.0),
            delta: Point::new(1.0, 0.0),
        }
        .is_user_activity());
        assert!(!InputEvent::DeviceAdded {
            name: "dock".to_owned()
        }
        .is_user_activity());
    }

    #[test]
    fn descriptions_include_the_modifiers_that_were_held() {
        let e = InputEvent::KeyDown {
            key: Key::Char('1'),
            code: 2,
            mods: Modifiers {
                meta: true,
                ..Modifiers::NONE
            },
            repeat: false,
        };
        assert_eq!(e.describe(), "key down Win+1");
    }

    #[test]
    fn a_double_click_says_so_and_a_release_does_not() {
        let down = InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: true,
            at: Point::default(),
            clicks: 2,
            mods: Modifiers::NONE,
        };
        assert_eq!(down.describe(), "primary down x2");
        let up = InputEvent::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            at: Point::default(),
            clicks: 2,
            mods: Modifiers::NONE,
        };
        assert_eq!(up.describe(), "primary up");
    }

    #[test]
    fn distance_is_euclidean() {
        assert!((Point::new(0.0, 0.0).distance(Point::new(3.0, 4.0)) - 5.0).abs() < 1e-6);
    }
}
