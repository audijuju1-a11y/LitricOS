//! Base64 with the RFC 4648 §4 standard alphabet.
//!
//! Chromium delivers screencast frames as base64 text inside JSON. Decoding is
//! strict, following RFC 4648 §3.3 and §3.5:
//!
//! * the length must be a multiple of four;
//! * `=` may appear only as the last one or two characters of the input;
//! * every other character must be in the alphabet (no whitespace, no line
//!   breaks, no URL-safe `-`/`_`);
//! * the bits discarded by padding must be zero, so every byte string has
//!   exactly one accepted encoding.

use core::fmt;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Marks a byte outside the alphabet in [`DECODE`].
const INVALID: u8 = 0xFF;

/// Byte → 6-bit value, or [`INVALID`].
const DECODE: [u8; 256] = {
    let mut table = [INVALID; 256];
    let mut i = 0usize;
    while i < 64 {
        // Const evaluation: an out-of-range index is a compile error, never a
        // runtime panic.
        #[allow(clippy::indexing_slicing)]
        {
            table[ALPHABET[i] as usize] = i as u8;
        }
        i += 1;
    }
    table
};

/// Why text is not valid base64.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Base64Error {
    /// The length is not a multiple of four.
    BadLength,
    /// A character outside the alphabet, or `=` somewhere other than the end.
    BadCharacter {
        /// Byte offset of the character.
        offset: usize,
    },
    /// The bits discarded by the padding are not zero.
    NonCanonical,
}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadLength => f.write_str("base64 length is not a multiple of four"),
            Self::BadCharacter { offset } => write!(f, "invalid base64 character at {offset}"),
            Self::NonCanonical => f.write_str("base64 padding bits are not zero"),
        }
    }
}

impl std::error::Error for Base64Error {}

fn sextet(b: u8) -> u32 {
    u32::from(DECODE.get(usize::from(b)).copied().unwrap_or(INVALID))
}

fn symbol(v: u32) -> char {
    char::from(
        ALPHABET
            .get(usize::try_from(v & 0x3F).unwrap_or(0))
            .copied()
            .unwrap_or(b'A'),
    )
}

/// Encode bytes, with padding.
#[must_use]
pub fn encode(bytes: &[u8]) -> String {
    let groups = bytes.len().div_ceil(3);
    let mut out = String::with_capacity(groups.saturating_mul(4));
    let mut chunks = bytes.chunks_exact(3);
    for c in &mut chunks {
        if let [a, b, d] = *c {
            let n = (u32::from(a) << 16) | (u32::from(b) << 8) | u32::from(d);
            out.push(symbol(n >> 18));
            out.push(symbol(n >> 12));
            out.push(symbol(n >> 6));
            out.push(symbol(n));
        }
    }
    match *chunks.remainder() {
        [a] => {
            let n = u32::from(a) << 16;
            out.push(symbol(n >> 18));
            out.push(symbol(n >> 12));
            out.push_str("==");
        }
        [a, b] => {
            let n = (u32::from(a) << 16) | (u32::from(b) << 8);
            out.push(symbol(n >> 18));
            out.push(symbol(n >> 12));
            out.push(symbol(n >> 6));
            out.push('=');
        }
        _ => {}
    }
    out
}

/// Decode padded base64 text.
///
/// # Errors
/// [`Base64Error`] for anything that is not the canonical encoding of some
/// byte string.
pub fn decode(text: &[u8]) -> Result<Vec<u8>, Base64Error> {
    if text.len() % 4 != 0 {
        return Err(Base64Error::BadLength);
    }
    let mut out = Vec::with_capacity((text.len() >> 2).saturating_mul(3));
    let last_group = text.len().saturating_sub(4);
    for (g, quad) in text.chunks_exact(4).enumerate() {
        let base = g.saturating_mul(4);
        let [c0, c1, c2, c3] = *quad else {
            return Err(Base64Error::BadLength);
        };
        let is_last = base == last_group;
        let bad = |i: usize| Base64Error::BadCharacter {
            offset: base.saturating_add(i),
        };
        let v0 = sextet(c0);
        let v1 = sextet(c1);
        if v0 > 63 {
            return Err(bad(0));
        }
        if v1 > 63 {
            return Err(bad(1));
        }
        let head = (v0 << 18) | (v1 << 12);
        match (c2, c3) {
            (b'=', b'=') if is_last => {
                if v1 & 0x0F != 0 {
                    return Err(Base64Error::NonCanonical);
                }
                out.push(((head >> 16) & 0xFF) as u8);
            }
            (_, b'=') if is_last => {
                let v2 = sextet(c2);
                if v2 > 63 {
                    return Err(bad(2));
                }
                if v2 & 0x03 != 0 {
                    return Err(Base64Error::NonCanonical);
                }
                let n = head | (v2 << 6);
                out.push(((n >> 16) & 0xFF) as u8);
                out.push(((n >> 8) & 0xFF) as u8);
            }
            _ => {
                let v2 = sextet(c2);
                let v3 = sextet(c3);
                if v2 > 63 {
                    return Err(bad(2));
                }
                if v3 > 63 {
                    return Err(bad(3));
                }
                let n = head | (v2 << 6) | v3;
                out.push(((n >> 16) & 0xFF) as u8);
                out.push(((n >> 8) & 0xFF) as u8);
                out.push((n & 0xFF) as u8);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    #[test]
    fn rfc_4648_test_vectors() {
        // RFC 4648 §10.
        let vectors = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (plain, coded) in vectors {
            assert_eq!(encode(plain.as_bytes()), coded);
            assert_eq!(decode(coded.as_bytes()).expect(coded), plain.as_bytes());
        }
    }

    #[test]
    fn every_byte_value_and_length_round_trips() {
        let all: Vec<u8> = (0..=255u8).rev().chain(0..=255u8).collect();
        for len in 0..all.len() {
            let enc = encode(&all[..len]);
            assert_eq!(enc.len() % 4, 0);
            assert_eq!(decode(enc.as_bytes()).expect("round trip"), &all[..len]);
        }
    }

    #[test]
    fn the_whole_alphabet_decodes() {
        let text = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes = decode(text).expect("alphabet");
        assert_eq!(encode(&bytes).as_bytes(), text);
    }

    #[test]
    fn malformed_input_is_refused() {
        assert_eq!(decode(b"Zg="), Err(Base64Error::BadLength));
        assert_eq!(decode(b"Zg"), Err(Base64Error::BadLength));
        assert_eq!(decode(b"Z"), Err(Base64Error::BadLength));
        assert_eq!(decode(b"Zm9v\n"), Err(Base64Error::BadLength));
        assert_eq!(
            decode(b"Zm 9"),
            Err(Base64Error::BadCharacter { offset: 2 })
        );
        assert_eq!(
            decode(b"Zm9-"),
            Err(Base64Error::BadCharacter { offset: 3 })
        );
        assert_eq!(
            decode(b"Zm9_"),
            Err(Base64Error::BadCharacter { offset: 3 })
        );
        // Padding in the middle of the input.
        assert_eq!(
            decode(b"Zg==Zm9v"),
            Err(Base64Error::BadCharacter { offset: 2 })
        );
        assert_eq!(
            decode(b"Zm9=Zm9v"),
            Err(Base64Error::BadCharacter { offset: 3 })
        );
        // Padding in the wrong place within the final group.
        assert_eq!(
            decode(b"Z==="),
            Err(Base64Error::BadCharacter { offset: 1 })
        );
        assert_eq!(
            decode(b"Zg=a"),
            Err(Base64Error::BadCharacter { offset: 2 })
        );
        assert_eq!(
            decode(b"===="),
            Err(Base64Error::BadCharacter { offset: 0 })
        );
        // Non-ASCII.
        assert_eq!(decode("Zm9\u{e9}".as_bytes()), Err(Base64Error::BadLength));
        assert_eq!(
            decode(b"Zm9\xff"),
            Err(Base64Error::BadCharacter { offset: 3 })
        );
    }

    #[test]
    fn nonzero_padding_bits_are_refused() {
        // "Zh==" and "Zm9=" differ from the canonical "Zg==" / "Zm8=" only in
        // bits the padding discards.
        assert_eq!(decode(b"Zh=="), Err(Base64Error::NonCanonical));
        assert_eq!(decode(b"Zm9="), Err(Base64Error::NonCanonical));
        assert!(decode(b"Zg==").is_ok());
        assert!(decode(b"Zm8=").is_ok());
    }
}
