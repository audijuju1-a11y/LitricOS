//! CRC-32 as gzip (RFC 1952 section 8) and ZIP (APPNOTE 4.4.7) define it: the
//! reflected polynomial `0xEDB88320`, initial value and final XOR `0xFFFFFFFF`.

use std::sync::OnceLock;

/// The reflected generator polynomial.
const POLY: u32 = 0xEDB8_8320;

fn table() -> &'static [u32; 256] {
    static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (n, slot) in (0u32..).zip(t.iter_mut()) {
            let mut c = n;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    POLY ^ c.wrapping_shr(1)
                } else {
                    c.wrapping_shr(1)
                };
            }
            *slot = c;
        }
        t
    })
}

/// A running CRC-32.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc32 {
    state: u32,
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Crc32 {
    /// A CRC over no bytes.
    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0xFFFF_FFFF }
    }

    /// Feed bytes.
    pub fn update(&mut self, data: &[u8]) {
        let t = table();
        let mut c = self.state;
        for &b in data {
            let idx = usize::from((c as u8) ^ b);
            c = t.get(idx).copied().unwrap_or(0) ^ c.wrapping_shr(8);
        }
        self.state = c;
    }

    /// The CRC of everything fed so far.
    #[must_use]
    pub const fn value(&self) -> u32 {
        self.state ^ 0xFFFF_FFFF
    }
}

/// The CRC-32 of `data`.
#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut c = Crc32::new();
    c.update(data);
    c.value()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_standard_check_value() {
        // The check value every CRC-32/ISO-HDLC description gives.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn incremental_updates_equal_one_update() {
        let data: Vec<u8> = (0u32..10_000)
            .map(|i| (i.wrapping_mul(31) >> 3) as u8)
            .collect();
        let mut c = Crc32::new();
        for chunk in data.chunks(97) {
            c.update(chunk);
        }
        assert_eq!(c.value(), crc32(&data));
    }
}
