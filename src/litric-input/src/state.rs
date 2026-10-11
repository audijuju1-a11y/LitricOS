//! Turning raw device records into desktop events.
//!
//! This is where the accumulated state lives: which modifiers are held, where
//! the pointer is, when the last click was and where. It is a pure state
//! machine over [`RawEvent`], with no clock, no files and no threads, so every
//! behaviour below — double-click timing, edge clamping, wheel accumulation —
//! is reachable from a test that runs in microseconds.
//!
//! # Why events are batched to `SYN_REPORT`
//!
//! A mouse reports `REL_X`, then `REL_Y`, then `SYN_REPORT`. Emitting a move
//! after each axis produces two events for one motion, the first of them
//! diagonal-free, and anything doing hit-testing sees the pointer take a
//! staircase path. The kernel marks where one physical update ends; this
//! respects that mark.

use crate::evdev::{
    RawEvent, ABS_X, ABS_Y, BTN_EXTRA, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, BTN_SIDE, EV_ABS, EV_KEY,
    EV_REL, EV_SYN, REL_HWHEEL, REL_HWHEEL_HI_RES, REL_WHEEL, REL_WHEEL_HI_RES, REL_X, REL_Y,
    SYN_REPORT, VALUE_DOWN, VALUE_REPEAT, VALUE_UP,
};
use crate::event::{InputEvent, Point, PointerButton};
use crate::keys::{Key, Layout, Modifiers, Side, US_QWERTY};

/// How long after a click a second one still counts as a double, in
/// milliseconds.
///
/// 400 ms is the interval most desktops use and the one most people's hands
/// are already calibrated to. Faster excludes people with limited dexterity;
/// slower makes two deliberate clicks turn into an accidental double.
pub const DOUBLE_CLICK_MS: i64 = 400;

/// How far the pointer may move between clicks and still count as a double, in
/// pixels. A hand that shifts three pixels did not mean to click somewhere else.
pub const DOUBLE_CLICK_SLOP: f32 = 6.0;

/// High-resolution wheels report 120 units per detent.
const HI_RES_PER_DETENT: f32 = 120.0;

fn button_of(code: u16) -> Option<PointerButton> {
    match code {
        BTN_LEFT => Some(PointerButton::Primary),
        BTN_RIGHT => Some(PointerButton::Secondary),
        BTN_MIDDLE => Some(PointerButton::Middle),
        BTN_SIDE => Some(PointerButton::Back),
        BTN_EXTRA => Some(PointerButton::Forward),
        _ => None,
    }
}

/// Accumulated input state for one seat.
#[derive(Debug, Clone)]
pub struct InputState {
    layout: Layout,
    mods: Modifiers,
    pointer: Point,
    bounds: (f32, f32),
    /// Pending motion within the current packet.
    pending: Point,
    pending_wheel: Point,
    /// Whether anything in this packet needs emitting at the next sync.
    dirty_move: bool,
    dirty_wheel: bool,
    /// Buttons produced within the current packet, held until sync so they
    /// carry the position the motion in the same packet moved to.
    pending_buttons: Vec<(PointerButton, bool)>,
    last_click: Option<(PointerButton, i64, Point, u8)>,
    now_ms: i64,
    /// ABS device range: maximum X and Y values reported by the device.
    /// The QEMU USB tablet reports 0..16383 on both axes.
    abs_max: (f32, f32),
}

impl InputState {
    /// A fresh state for a display of this size.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            layout: US_QWERTY,
            mods: Modifiers::NONE,
            pointer: Point::new(width / 2.0, height / 2.0),
            bounds: (width.max(1.0), height.max(1.0)),
            pending: Point::default(),
            pending_wheel: Point::default(),
            dirty_move: false,
            dirty_wheel: false,
            pending_buttons: Vec::new(),
            last_click: None,
            now_ms: 0,
            // The divisor must be the device's real absolute range, and the
            // only honest source of that is the device itself: the backend
            // queries EVIOCGABS when it opens each node and calls
            // [`InputState::set_abs_max`] with what it learned. This default
            // covers the case where no query ran — the QEMU USB tablet's HID
            // descriptor declares 0..0xFFFF, and the automation harness scales
            // its coordinates to that same constant. Both facts are measured,
            // not assumed: the harness constant is what photo_tour_lib.py
            // sends, and 0xFFFF is what EVIOCGABS answered in the live guest
            // (input.absmax=65535, serial diagnostics 2026-09-24). Hardcoding
            // anything else was the bug that hid for weeks: with 16383 here
            // while the device spoke 0xFFFF, a click aimed at the dock landed
            // near the top-left corner and the whole desktop looked dead to
            // automation. Never guess a device's range again — ask it.
            abs_max: (65_535.0, 65_535.0),
        }
    }

    /// Use a different absolute range, as learned from the device itself.
    ///
    /// Called by the backend with the EVIOCGABS maximum of the device it
    /// opened. Each axis is adopted only when the value is a sane positive
    /// range, so a device that reports no Y axis (or an ioctl that failed)
    /// leaves that axis on the default instead of collapsing every position
    /// onto one edge.
    pub fn set_abs_max(&mut self, x: f32, y: f32) {
        for (learned, current) in [(x, &mut self.abs_max.0), (y, &mut self.abs_max.1)] {
            if learned.is_finite() && learned >= 1.0 {
                *current = learned;
            }
        }
    }

    /// The absolute range currently in use, per axis.
    #[must_use]
    pub const fn abs_max(&self) -> (f32, f32) {
        self.abs_max
    }

    /// Use a different keyboard layout, e.g. because the locale changed.
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    /// The layout in use.
    #[must_use]
    pub const fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The display size the pointer is clamped to.
    pub fn set_bounds(&mut self, width: f32, height: f32) {
        self.bounds = (width.max(1.0), height.max(1.0));
        self.pointer = self.clamp(self.pointer);
    }

    /// Where the pointer is.
    #[must_use]
    pub const fn pointer(&self) -> Point {
        self.pointer
    }

    /// Which modifiers are held.
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        self.mods
    }

    /// Move the pointer directly, for a test or a warp.
    pub fn set_pointer(&mut self, at: Point) {
        self.pointer = self.clamp(at);
    }

    fn clamp(&self, p: Point) -> Point {
        Point::new(
            p.x.clamp(0.0, self.bounds.0 - 1.0),
            p.y.clamp(0.0, self.bounds.1 - 1.0),
        )
    }

    fn apply_modifier(&mut self, key: Key, down: bool) {
        match key {
            Key::Shift(_) => self.mods.shift = down,
            Key::Ctrl(_) => self.mods.ctrl = down,
            Key::Alt(Side::Left) => self.mods.alt = down,
            Key::Alt(Side::Right) => self.mods.altgr = down,
            Key::Meta(_) => self.mods.meta = down,
            // Lock keys toggle on press and are unaffected by release.
            Key::CapsLock if down => self.mods.caps = !self.mods.caps,
            Key::NumLock if down => self.mods.num = !self.mods.num,
            _ => {}
        }
    }

    /// Feed one raw record. Events come out at packet boundaries.
    pub fn feed(&mut self, raw: RawEvent) -> Vec<InputEvent> {
        self.now_ms = raw.millis();
        let mut out = Vec::new();
        match raw.kind {
            EV_KEY => {
                if let Some(button) = button_of(raw.code) {
                    if raw.value != VALUE_REPEAT {
                        self.pending_buttons.push((button, raw.value == VALUE_DOWN));
                    }
                } else {
                    out.extend(self.key_event(raw));
                }
            }
            EV_REL => match raw.code {
                REL_X => {
                    self.pending.x += raw.value as f32;
                    self.dirty_move = true;
                }
                REL_Y => {
                    self.pending.y += raw.value as f32;
                    self.dirty_move = true;
                }
                REL_WHEEL => {
                    self.pending_wheel.y -= raw.value as f32;
                    self.dirty_wheel = true;
                }
                REL_HWHEEL => {
                    self.pending_wheel.x += raw.value as f32;
                    self.dirty_wheel = true;
                }
                REL_WHEEL_HI_RES => {
                    self.pending_wheel.y -= raw.value as f32 / HI_RES_PER_DETENT;
                    self.dirty_wheel = true;
                }
                REL_HWHEEL_HI_RES => {
                    self.pending_wheel.x += raw.value as f32 / HI_RES_PER_DETENT;
                    self.dirty_wheel = true;
                }
                _ => {}
            },
            EV_ABS => {
                // A tablet reports where it is, not how far it moved. Scale
                // from device coordinates (0..abs_max) to screen coordinates.
                // abs_max comes from the device (EVIOCGABS) or the 0xFFFF default.
                if raw.code == ABS_X {
                    self.pending.x =
                        (raw.value as f32 / self.abs_max.0) * self.bounds.0 - self.pointer.x;
                    self.dirty_move = true;
                } else if raw.code == ABS_Y {
                    self.pending.y =
                        (raw.value as f32 / self.abs_max.1) * self.bounds.1 - self.pointer.y;
                    self.dirty_move = true;
                }
            }
            EV_SYN if raw.code == SYN_REPORT => out.extend(self.flush()),
            _ => {}
        }
        out
    }

    fn key_event(&mut self, raw: RawEvent) -> Vec<InputEvent> {
        let key = self.layout.key(raw.code);
        let mut out = Vec::new();
        match raw.value {
            VALUE_DOWN | VALUE_REPEAT => {
                let repeat = raw.value == VALUE_REPEAT;
                if !repeat {
                    self.apply_modifier(key, true);
                }
                out.push(InputEvent::KeyDown {
                    key,
                    code: raw.code,
                    mods: self.mods,
                    repeat,
                });
                if let Some(c) = self.layout.text(raw.code, self.mods) {
                    out.push(InputEvent::TextInput {
                        text: c.to_string(),
                    });
                }
            }
            VALUE_UP => {
                self.apply_modifier(key, false);
                out.push(InputEvent::KeyUp {
                    key,
                    code: raw.code,
                    mods: self.mods,
                });
            }
            _ => {}
        }
        out
    }

    /// Emit whatever accumulated during this packet.
    fn flush(&mut self) -> Vec<InputEvent> {
        let mut out = Vec::new();

        if self.dirty_move {
            let before = self.pointer;
            self.pointer = self.clamp(Point::new(
                self.pointer.x + self.pending.x,
                self.pointer.y + self.pending.y,
            ));
            // The delta reported is the device's, not the clamped difference:
            // at the edge of the screen the position stops changing while the
            // hand keeps moving, and a gesture that reads the position would
            // conclude the hand had stopped.
            let delta = self.pending;
            self.pending = Point::default();
            self.dirty_move = false;
            if before != self.pointer || delta.x != 0.0 || delta.y != 0.0 {
                out.push(InputEvent::PointerMove {
                    at: self.pointer,
                    delta,
                });
            }
        }

        // Buttons after motion, so a click carries the position the same packet
        // moved to. A tap on a touchpad arrives as move-then-click in one
        // packet, and reporting the click at the old position misses the target
        // by however far the finger travelled.
        let buttons = core::mem::take(&mut self.pending_buttons);
        for (button, pressed) in buttons {
            let clicks = if pressed {
                self.count_click(button)
            } else {
                self.last_click.map_or(1, |(_, _, _, n)| n)
            };
            out.push(InputEvent::PointerButton {
                button,
                pressed,
                at: self.pointer,
                clicks,
                mods: self.mods,
            });
        }

        if self.dirty_wheel {
            let (dx, dy) = (self.pending_wheel.x, self.pending_wheel.y);
            self.pending_wheel = Point::default();
            self.dirty_wheel = false;
            if dx != 0.0 || dy != 0.0 {
                out.push(InputEvent::Scroll {
                    dx,
                    dy,
                    at: self.pointer,
                    mods: self.mods,
                });
            }
        }

        out
    }

    fn count_click(&mut self, button: PointerButton) -> u8 {
        let n = match self.last_click {
            Some((last_btn, last_ms, last_at, count))
                if last_btn == button
                    && self.now_ms.saturating_sub(last_ms) <= DOUBLE_CLICK_MS
                    && self.pointer.distance(last_at) <= DOUBLE_CLICK_SLOP =>
            {
                count.saturating_add(1).min(3)
            }
            _ => 1,
        };
        self.last_click = Some((button, self.now_ms, self.pointer, n));
        n
    }
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use crate::evdev::Abi;

    fn ev(kind: u16, code: u16, value: i32, ms: i64) -> RawEvent {
        RawEvent {
            sec: ms.saturating_div(1000),
            usec: ms.checked_rem(1000).unwrap_or(0).saturating_mul(1000),
            kind,
            code,
            value,
        }
    }

    fn sync(ms: i64) -> RawEvent {
        ev(EV_SYN, SYN_REPORT, 0, ms)
    }

    fn state() -> InputState {
        let mut s = InputState::new(1280.0, 800.0);
        s.set_pointer(Point::new(100.0, 100.0));
        s
    }

    fn drain(s: &mut InputState, events: &[RawEvent]) -> Vec<InputEvent> {
        let mut out = Vec::new();
        for e in events {
            out.extend(s.feed(*e));
        }
        out
    }

    #[test]
    fn a_keypress_produces_a_key_event_and_the_text_it_typed() {
        let mut s = state();
        let out = drain(&mut s, &[ev(EV_KEY, 30, VALUE_DOWN, 0)]);
        assert_eq!(out.len(), 2, "{out:?}");
        assert!(matches!(
            out.first(),
            Some(InputEvent::KeyDown {
                key: Key::Char('a'),
                ..
            })
        ));
        assert!(matches!(out.get(1), Some(InputEvent::TextInput { text }) if text == "a"));
    }

    #[test]
    fn holding_shift_changes_the_text_but_not_the_key() {
        let mut s = state();
        let out = drain(
            &mut s,
            &[
                ev(EV_KEY, 42, VALUE_DOWN, 0),
                ev(EV_KEY, 30, VALUE_DOWN, 10),
            ],
        );
        assert!(matches!(out.last(), Some(InputEvent::TextInput { text }) if text == "A"));
        let key = out.iter().find_map(|e| match e {
            InputEvent::KeyDown { key, code: 30, .. } => Some(*key),
            _ => None,
        });
        assert_eq!(key, Some(Key::Char('a')), "the key is still the 'a' key");
    }

    #[test]
    fn a_modifier_release_clears_it() {
        let mut s = state();
        drain(&mut s, &[ev(EV_KEY, 42, VALUE_DOWN, 0)]);
        assert!(s.modifiers().shift);
        drain(&mut s, &[ev(EV_KEY, 42, VALUE_UP, 10)]);
        assert!(!s.modifiers().shift);
    }

    #[test]
    fn caps_lock_toggles_on_press_and_survives_release() {
        let mut s = state();
        drain(
            &mut s,
            &[ev(EV_KEY, 58, VALUE_DOWN, 0), ev(EV_KEY, 58, VALUE_UP, 5)],
        );
        assert!(s.modifiers().caps, "caps did not latch");
        drain(
            &mut s,
            &[
                ev(EV_KEY, 58, VALUE_DOWN, 100),
                ev(EV_KEY, 58, VALUE_UP, 105),
            ],
        );
        assert!(!s.modifiers().caps, "caps did not unlatch");
    }

    #[test]
    fn a_command_key_combination_types_nothing() {
        let mut s = state();
        let out = drain(
            &mut s,
            &[ev(EV_KEY, 125, VALUE_DOWN, 0), ev(EV_KEY, 2, VALUE_DOWN, 5)],
        );
        assert!(
            !out.iter()
                .any(|e| matches!(e, InputEvent::TextInput { .. })),
            "Win+1 typed something: {out:?}"
        );
        assert!(out.iter().any(|e| matches!(
            e,
            InputEvent::KeyDown {
                key: Key::Char('1'),
                mods,
                ..
            } if mods.meta
        )));
    }

    #[test]
    fn auto_repeat_is_marked_and_still_types() {
        let mut s = state();
        drain(&mut s, &[ev(EV_KEY, 30, VALUE_DOWN, 0)]);
        let out = drain(&mut s, &[ev(EV_KEY, 30, VALUE_REPEAT, 500)]);
        assert!(out
            .iter()
            .any(|e| matches!(e, InputEvent::KeyDown { repeat: true, .. })));
        assert!(out
            .iter()
            .any(|e| matches!(e, InputEvent::TextInput { .. })));
    }

    #[test]
    fn one_packet_of_motion_produces_one_move() {
        // Two axes and a sync is one physical movement. Emitting per axis makes
        // the pointer travel in right angles.
        let mut s = state();
        let out = drain(
            &mut s,
            &[ev(EV_REL, REL_X, 10, 0), ev(EV_REL, REL_Y, -4, 0), sync(0)],
        );
        assert_eq!(out.len(), 1, "{out:?}");
        assert_eq!(
            s.pointer(),
            Point::new(110.0, 96.0),
            "both axes applied once"
        );
    }

    #[test]
    fn an_absolute_tablet_position_maps_the_full_device_range_to_the_screen() {
        // Mapping is exact only when the divisor equals the device's real
        // range. The backend learns that from EVIOCGABS and calls
        // set_abs_max; here a 0..16383 device (what a small touch panel
        // reports) is pinned against its own maximum: half-range reaches the
        // middle, full-range the bottom.
        let mut s = state();
        s.set_abs_max(16_383.0, 16_383.0);
        assert_eq!(s.abs_max(), (16_383.0, 16_383.0));
        drain(
            &mut s,
            &[
                ev(EV_ABS, ABS_X, 8191, 0),
                ev(EV_ABS, ABS_Y, 8191, 0),
                sync(0),
            ],
        );
        let half = s.pointer();
        assert!(
            (half.x - 640.0).abs() < 1.0 && (half.y - 400.0).abs() < 1.0,
            "half of the device range must be the screen center, got {half:?}"
        );
        drain(
            &mut s,
            &[
                ev(EV_ABS, ABS_X, 16383, 0),
                ev(EV_ABS, ABS_Y, 16383, 0),
                sync(0),
            ],
        );
        let full = s.pointer();
        assert!(
            (full.x - 1279.0).abs() < 1.0 && (full.y - 799.0).abs() < 1.0,
            "the device maximum must be the bottom-right pixel, got {full:?}"
        );
    }

    #[test]
    fn the_default_range_matches_what_the_automation_harness_sends() {
        // The regression that hid for weeks, pinned: QEMU passes QMP abs
        // values through 1:1 (value 666 arrived as raw 666 — proven live),
        // so the harness scales its screen pixels to the tablet's declared
        // 0..0xFFFF (measured live via EVIOCGABS: 65535) and the guest must
        // divide by that same maximum. The
        // round-trip below is the contract between tools/photo_tour_lib.py
        // and this device range: pixel 666 must come back as pixel 666.
        let mut s = state();
        assert_eq!(s.abs_max(), (65_535.0, 65_535.0));
        let raw = (666.0f32 * 65_535.0 / 1279.0).round() as i32;
        drain(
            &mut s,
            &[
                ev(EV_ABS, ABS_X, raw, 0),
                ev(
                    EV_ABS,
                    ABS_Y,
                    (400.0f32 * 65_535.0 / 799.0).round() as i32,
                    0,
                ),
                sync(0),
            ],
        );
        let p = s.pointer();
        assert!(
            (p.x - 666.0).abs() < 1.0 && (p.y - 400.0).abs() < 1.0,
            "harness round-trip must land on the aimed pixel, got {p:?}"
        );
    }

    #[test]
    fn set_abs_max_ignores_ranges_that_cannot_be_real() {
        // A failed EVIOCGABS (or a half-alive device) must not collapse the
        // pointer onto one edge: only sane positive ranges are adopted, per
        // axis, so a Y-only result still fixes Y.
        let mut s = state();
        s.set_abs_max(0.0, f32::NAN);
        assert_eq!(s.abs_max(), (65_535.0, 65_535.0));
        s.set_abs_max(800.0, -1.0);
        assert_eq!(s.abs_max(), (800.0, 65_535.0));
    }

    #[test]
    fn motion_is_clamped_to_the_display_but_the_delta_is_not() {
        let mut s = InputState::new(200.0, 100.0);
        s.set_pointer(Point::new(195.0, 50.0));
        let out = drain(&mut s, &[ev(EV_REL, REL_X, 40, 0), sync(0)]);
        assert_eq!(s.pointer().x, 199.0, "clamped to the last column");
        let Some(InputEvent::PointerMove { delta, .. }) = out.first() else {
            panic!("expected a move, got {out:?}");
        };
        assert_eq!(delta.x, 40.0, "the device really did move 40");
    }

    #[test]
    fn a_click_lands_at_the_position_the_same_packet_moved_to() {
        // A touchpad tap arrives as motion and a button in one packet. Ordering
        // the button first misses the target by however far the finger went.
        let mut s = state();
        let out = drain(
            &mut s,
            &[
                ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 0),
                ev(EV_REL, REL_X, 25, 0),
                sync(0),
            ],
        );
        let click = out.iter().find_map(|e| match e {
            InputEvent::PointerButton { at, .. } => Some(*at),
            _ => None,
        });
        assert_eq!(click, Some(Point::new(125.0, 100.0)));
    }

    #[test]
    fn two_quick_clicks_in_the_same_place_are_a_double_click() {
        let mut s = state();
        let a = drain(
            &mut s,
            &[ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 1000), sync(1000)],
        );
        assert!(matches!(
            a.first(),
            Some(InputEvent::PointerButton { clicks: 1, .. })
        ));
        drain(&mut s, &[ev(EV_KEY, BTN_LEFT, VALUE_UP, 1050), sync(1050)]);
        let b = drain(
            &mut s,
            &[ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 1200), sync(1200)],
        );
        assert!(
            matches!(b.first(), Some(InputEvent::PointerButton { clicks: 2, .. })),
            "{b:?}"
        );
    }

    #[test]
    fn a_slow_second_click_is_a_fresh_click() {
        let mut s = state();
        drain(
            &mut s,
            &[ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 1000), sync(1000)],
        );
        let b = drain(
            &mut s,
            &[
                ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 1000 + DOUBLE_CLICK_MS + 1),
                sync(1000 + DOUBLE_CLICK_MS + 1),
            ],
        );
        assert!(matches!(
            b.first(),
            Some(InputEvent::PointerButton { clicks: 1, .. })
        ));
    }

    #[test]
    fn a_second_click_somewhere_else_is_a_fresh_click() {
        let mut s = state();
        drain(
            &mut s,
            &[ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 1000), sync(1000)],
        );
        let b = drain(
            &mut s,
            &[
                ev(EV_REL, REL_X, 40, 1100),
                ev(EV_KEY, BTN_LEFT, VALUE_DOWN, 1100),
                sync(1100),
            ],
        );
        assert!(
            matches!(b.last(), Some(InputEvent::PointerButton { clicks: 1, .. })),
            "{b:?}"
        );
    }

    #[test]
    fn a_triple_click_stops_at_three() {
        let mut s = state();
        let mut last = 0;
        for i in 0..5 {
            let t = 1000 + i * 100;
            last = drain(&mut s, &[ev(EV_KEY, BTN_LEFT, VALUE_DOWN, t), sync(t)])
                .iter()
                .find_map(|e| match e {
                    InputEvent::PointerButton { clicks, .. } => Some(*clicks),
                    _ => None,
                })
                .unwrap_or(0);
        }
        assert_eq!(last, 3, "click counting must saturate, not wrap");
    }

    #[test]
    fn the_wheel_scrolls_content_down_when_it_turns_forward() {
        // REL_WHEEL is +1 when the wheel turns away from the user, which moves
        // the content up and the viewport down. Getting this backwards is the
        // single most complained-about bug in any new desktop.
        let mut s = state();
        let out = drain(&mut s, &[ev(EV_REL, REL_WHEEL, 1, 0), sync(0)]);
        let Some(InputEvent::Scroll { dy, .. }) = out.first() else {
            panic!("expected a scroll, got {out:?}");
        };
        assert_eq!(*dy, -1.0, "one detent away from the user");
    }

    #[test]
    fn a_high_resolution_wheel_reports_fractions_of_a_detent() {
        let mut s = state();
        let out = drain(&mut s, &[ev(EV_REL, REL_WHEEL_HI_RES, 30, 0), sync(0)]);
        let Some(InputEvent::Scroll { dy, .. }) = out.first() else {
            panic!("expected a scroll, got {out:?}");
        };
        assert!((dy + 0.25).abs() < 1e-6, "quarter detent, got {dy}");
    }

    #[test]
    fn nothing_is_emitted_without_a_sync() {
        let mut s = state();
        let out = drain(&mut s, &[ev(EV_REL, REL_X, 10, 0)]);
        assert!(out.is_empty(), "motion escaped its packet: {out:?}");
        assert_eq!(s.pointer(), Point::new(100.0, 100.0));
    }

    #[test]
    fn an_empty_packet_produces_nothing() {
        let mut s = state();
        assert!(drain(&mut s, &[sync(0), sync(1), sync(2)]).is_empty());
    }

    #[test]
    fn resizing_the_display_brings_the_pointer_back_on_screen() {
        let mut s = InputState::new(1920.0, 1080.0);
        s.set_pointer(Point::new(1900.0, 1000.0));
        s.set_bounds(800.0, 600.0);
        assert_eq!(s.pointer(), Point::new(799.0, 599.0));
    }

    #[test]
    fn changing_the_layout_changes_what_is_typed() {
        let mut s = state();
        s.set_layout(crate::keys::UK_QWERTY);
        let out = drain(
            &mut s,
            &[ev(EV_KEY, 42, VALUE_DOWN, 0), ev(EV_KEY, 3, VALUE_DOWN, 5)],
        );
        assert!(
            matches!(out.last(), Some(InputEvent::TextInput { text }) if text == "\""),
            "{out:?}"
        );
    }

    #[test]
    fn a_whole_recorded_packet_parses_and_replays() {
        // End to end from bytes: what a real mouse sends for a small move.
        let mut bytes = Vec::new();
        for (kind, code, value) in [
            (EV_REL, REL_X, 5i32),
            (EV_REL, REL_Y, 3),
            (EV_SYN, SYN_REPORT, 0),
        ] {
            let mut b = [0u8; 24];
            b[0..8].copy_from_slice(&1i64.to_le_bytes());
            b[8..16].copy_from_slice(&0i64.to_le_bytes());
            b[16..18].copy_from_slice(&kind.to_le_bytes());
            b[18..20].copy_from_slice(&code.to_le_bytes());
            b[20..24].copy_from_slice(&value.to_le_bytes());
            bytes.extend_from_slice(&b);
        }
        let (raws, used) = crate::evdev::parse_stream(Abi::Bits64, &bytes);
        assert_eq!(used, bytes.len());
        let mut s = state();
        let out = drain(&mut s, &raws);
        assert_eq!(out.len(), 1);
        assert_eq!(s.pointer(), Point::new(105.0, 103.0));
    }
}
