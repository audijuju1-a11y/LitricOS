//! The Linux evdev wire format.
//!
//! `/dev/input/event*` delivers a stream of fixed-size records. That is all
//! this module knows: it turns bytes into [`RawEvent`] and does not care where
//! the bytes came from, which is what lets the whole input stack be tested from
//! a byte array on a machine with no input devices.
//!
//! ```text
//!   struct input_event {          64-bit ABI       32-bit ABI
//!       struct timeval time;      16 bytes          8 bytes
//!       __u16 type;                2                2
//!       __u16 code;                2                2
//!       __s32 value;               4                4
//!   };                            24 bytes         16 bytes
//! ```
//!
//! The record size is part of the kernel ABI rather than something we chose, so
//! it is named and checked rather than assumed. Reading a 24-byte record as
//! 16 bytes does not fail: it produces a plausible-looking stream of nonsense
//! key presses, which is a far worse failure than an error.

/// Event type: a synchronisation marker.
pub const EV_SYN: u16 = 0x00;
/// Event type: a key or button changed.
pub const EV_KEY: u16 = 0x01;
/// Event type: relative motion, such as a mouse.
pub const EV_REL: u16 = 0x02;
/// Event type: absolute position, such as a touchscreen or tablet.
pub const EV_ABS: u16 = 0x03;
/// Event type: miscellaneous, including scancodes.
pub const EV_MSC: u16 = 0x04;
/// Event type: LED state.
pub const EV_LED: u16 = 0x11;

/// `SYN_REPORT`: everything since the last one is one atomic update.
pub const SYN_REPORT: u16 = 0;

/// Relative axis: horizontal motion.
pub const REL_X: u16 = 0x00;
/// Relative axis: vertical motion.
pub const REL_Y: u16 = 0x01;
/// Relative axis: vertical wheel, in detents.
pub const REL_WHEEL: u16 = 0x08;
/// Relative axis: horizontal wheel, in detents.
pub const REL_HWHEEL: u16 = 0x06;
/// Relative axis: high-resolution vertical wheel, 120 units per detent.
pub const REL_WHEEL_HI_RES: u16 = 0x0B;
/// Relative axis: high-resolution horizontal wheel, 120 units per detent.
pub const REL_HWHEEL_HI_RES: u16 = 0x0C;

/// Absolute axis: horizontal position.
pub const ABS_X: u16 = 0x00;
/// Absolute axis: vertical position.
pub const ABS_Y: u16 = 0x01;

/// Button: left mouse.
pub const BTN_LEFT: u16 = 0x110;
/// Button: right mouse.
pub const BTN_RIGHT: u16 = 0x111;
/// Button: middle mouse.
pub const BTN_MIDDLE: u16 = 0x112;
/// Button: side, usually "back".
pub const BTN_SIDE: u16 = 0x113;
/// Button: extra, usually "forward".
pub const BTN_EXTRA: u16 = 0x114;
/// Button: a finger is touching the surface.
pub const BTN_TOUCH: u16 = 0x14A;

/// Key value: released.
pub const VALUE_UP: i32 = 0;
/// Key value: pressed.
pub const VALUE_DOWN: i32 = 1;
/// Key value: auto-repeat while held.
pub const VALUE_REPEAT: i32 = 2;

/// The size of one `struct input_event` on a 64-bit kernel.
pub const RECORD_SIZE_64: usize = 24;
/// The size of one `struct input_event` on a 32-bit kernel.
pub const RECORD_SIZE_32: usize = 16;

/// Which ABI the records are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abi {
    /// 64-bit `time_t`: 24-byte records.
    Bits64,
    /// 32-bit `time_t`: 16-byte records.
    Bits32,
}

impl Abi {
    /// The ABI this build is running on.
    ///
    /// Decided from the pointer width rather than probed. Every Linux target
    /// LitricOS supports uses a `time_t` the same width as a pointer; if that
    /// ever stops being true, this is the one place it is wrong.
    #[must_use]
    pub const fn native() -> Self {
        if usize::BITS >= 64 {
            Self::Bits64
        } else {
            Self::Bits32
        }
    }

    /// Bytes per record.
    #[must_use]
    pub const fn record_size(self) -> usize {
        match self {
            Self::Bits64 => RECORD_SIZE_64,
            Self::Bits32 => RECORD_SIZE_32,
        }
    }
}

/// One record straight off an input device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawEvent {
    /// Seconds since the epoch, from the kernel's own clock.
    pub sec: i64,
    /// Microseconds within the second.
    pub usec: i64,
    /// `EV_*`.
    pub kind: u16,
    /// Key code, axis, or button.
    pub code: u16,
    /// Value: for keys 0/1/2, for relative axes a delta, for absolute a position.
    pub value: i32,
}

impl RawEvent {
    /// Parse one record.
    ///
    /// Returns `None` if the slice is shorter than the record. Timestamps come
    /// from the kernel and are used for double-click timing, so they are read
    /// rather than replaced by a clock read here — an event queued behind a
    /// slow frame must keep the time it actually happened.
    #[must_use]
    pub fn parse(abi: Abi, bytes: &[u8]) -> Option<Self> {
        fn le_u16(b: &[u8], at: usize) -> Option<u16> {
            let s = b.get(at..at.checked_add(2)?)?;
            Some(u16::from_le_bytes([*s.first()?, *s.get(1)?]))
        }
        fn le_i32(b: &[u8], at: usize) -> Option<i32> {
            let s = b.get(at..at.checked_add(4)?)?;
            Some(i32::from_le_bytes([
                *s.first()?,
                *s.get(1)?,
                *s.get(2)?,
                *s.get(3)?,
            ]))
        }
        fn le_i64(b: &[u8], at: usize) -> Option<i64> {
            let s = b.get(at..at.checked_add(8)?)?;
            let mut a = [0u8; 8];
            for (i, dst) in a.iter_mut().enumerate() {
                *dst = *s.get(i)?;
            }
            Some(i64::from_le_bytes(a))
        }

        match abi {
            Abi::Bits64 => {
                if bytes.len() < RECORD_SIZE_64 {
                    return None;
                }
                Some(Self {
                    sec: le_i64(bytes, 0)?,
                    usec: le_i64(bytes, 8)?,
                    kind: le_u16(bytes, 16)?,
                    code: le_u16(bytes, 18)?,
                    value: le_i32(bytes, 20)?,
                })
            }
            Abi::Bits32 => {
                if bytes.len() < RECORD_SIZE_32 {
                    return None;
                }
                Some(Self {
                    sec: i64::from(le_i32(bytes, 0)?),
                    usec: i64::from(le_i32(bytes, 4)?),
                    kind: le_u16(bytes, 8)?,
                    code: le_u16(bytes, 10)?,
                    value: le_i32(bytes, 12)?,
                })
            }
        }
    }

    /// The event's timestamp in milliseconds since the epoch.
    ///
    /// Saturating rather than wrapping: a device with a broken clock should
    /// produce a click that never double-clicks, not one that appears to have
    /// happened in 1970.
    #[must_use]
    pub const fn millis(&self) -> i64 {
        self.sec
            .saturating_mul(1000)
            .saturating_add(self.usec.saturating_div(1000))
    }

    /// Whether this is the end-of-packet marker.
    #[must_use]
    pub const fn is_sync(&self) -> bool {
        self.kind == EV_SYN && self.code == SYN_REPORT
    }
}

/// Split a buffer into whole records, returning them and the number of bytes
/// consumed.
///
/// A partial record at the end is left for the next read. Devices deliver whole
/// records, but a `read()` can still return a partial one if the buffer fills,
/// and treating the remainder as a fresh record is how a stream desynchronises
/// permanently.
#[must_use]
pub fn parse_stream(abi: Abi, bytes: &[u8]) -> (Vec<RawEvent>, usize) {
    let size = abi.record_size();
    let mut out = Vec::new();
    let mut at = 0usize;
    while let Some(end) = at.checked_add(size) {
        if end > bytes.len() {
            break;
        }
        let Some(slice) = bytes.get(at..end) else {
            break;
        };
        let Some(ev) = RawEvent::parse(abi, slice) else {
            break;
        };
        out.push(ev);
        at = end;
    }
    (out, at)
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn record64(sec: i64, usec: i64, kind: u16, code: u16, value: i32) -> [u8; 24] {
        let mut b = [0u8; 24];
        b[0..8].copy_from_slice(&sec.to_le_bytes());
        b[8..16].copy_from_slice(&usec.to_le_bytes());
        b[16..18].copy_from_slice(&kind.to_le_bytes());
        b[18..20].copy_from_slice(&code.to_le_bytes());
        b[20..24].copy_from_slice(&value.to_le_bytes());
        b
    }

    #[test]
    fn a_key_press_parses() {
        let b = record64(1_700_000_000, 500_000, EV_KEY, 30, VALUE_DOWN);
        let e = RawEvent::parse(Abi::Bits64, &b).expect("parses");
        assert_eq!(e.kind, EV_KEY);
        assert_eq!(e.code, 30);
        assert_eq!(e.value, VALUE_DOWN);
        assert_eq!(e.sec, 1_700_000_000);
        assert_eq!(e.millis(), 1_700_000_000_500);
    }

    #[test]
    fn a_negative_relative_motion_keeps_its_sign() {
        // A mouse moving left sends REL_X = -7. Read as unsigned it becomes
        // four billion, and the cursor leaves the screen on the first move.
        let b = record64(0, 0, EV_REL, REL_X, -7);
        let e = RawEvent::parse(Abi::Bits64, &b).expect("parses");
        assert_eq!(e.value, -7);
    }

    #[test]
    fn a_short_buffer_is_refused_rather_than_padded() {
        let b = [0u8; 20];
        assert!(RawEvent::parse(Abi::Bits64, &b).is_none());
        assert!(RawEvent::parse(Abi::Bits64, &[]).is_none());
    }

    #[test]
    fn the_record_size_is_the_kernel_abi_not_a_guess() {
        assert_eq!(Abi::Bits64.record_size(), 24);
        assert_eq!(Abi::Bits32.record_size(), 16);
        assert_eq!(Abi::native(), Abi::Bits64, "this build targets 64-bit");
    }

    #[test]
    fn a_stream_yields_whole_records_and_leaves_a_partial_one() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&record64(1, 0, EV_KEY, 30, 1));
        buf.extend_from_slice(&record64(1, 1, EV_SYN, SYN_REPORT, 0));
        buf.extend_from_slice(&record64(2, 0, EV_KEY, 30, 0)[..10]); // truncated

        let (events, used) = parse_stream(Abi::Bits64, &buf);
        assert_eq!(events.len(), 2, "two whole records");
        assert_eq!(used, 48, "the partial record was not consumed");
        assert!(events[1].is_sync());
    }

    #[test]
    fn an_empty_stream_consumes_nothing() {
        let (events, used) = parse_stream(Abi::Bits64, &[]);
        assert!(events.is_empty());
        assert_eq!(used, 0);
    }

    #[test]
    fn a_32_bit_record_parses_with_the_32_bit_abi() {
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(&7i32.to_le_bytes());
        b[4..8].copy_from_slice(&250_000i32.to_le_bytes());
        b[8..10].copy_from_slice(&EV_REL.to_le_bytes());
        b[10..12].copy_from_slice(&REL_Y.to_le_bytes());
        b[12..16].copy_from_slice(&(-3i32).to_le_bytes());
        let e = RawEvent::parse(Abi::Bits32, &b).expect("parses");
        assert_eq!(
            (e.sec, e.usec, e.kind, e.code, e.value),
            (7, 250_000, EV_REL, REL_Y, -3)
        );
        assert_eq!(e.millis(), 7_250);
    }

    #[test]
    fn a_broken_clock_cannot_wrap_the_millisecond_conversion() {
        let e = RawEvent {
            sec: i64::MAX,
            usec: 999_999,
            kind: EV_KEY,
            code: 1,
            value: 1,
        };
        assert_eq!(e.millis(), i64::MAX);
    }
}
