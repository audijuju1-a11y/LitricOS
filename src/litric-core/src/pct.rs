//! Percent-encoding for LitricOS line protocols.
//!
//! Every LitricOS daemon speaks a text protocol of space-separated fields, one
//! request per line (`litric-sysd`, LBP/1 in ADR-0018, LFP/1 in ADR-0019). A
//! file name, a page title or a URL can contain spaces, newlines and bytes that
//! are not UTF-8, so free text travels percent-encoded: every byte outside
//! `A-Z a-z 0-9 - . _ ~` becomes `%XX`.
//!
//! One convention is added on top of RFC 3986: a field that is exactly `-`
//! means *empty*. A real one-character value `-` is therefore written `%2D`, so
//! the two can never be confused.
//!
//! Decoding is strict. A `%` not followed by two hex digits is an error rather
//! than a literal, because a lenient decoder is how two parsers come to disagree
//! about what a request said.

use core::fmt;

/// Why a field could not be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PctError {
    /// A `%` at `index` is not followed by two hexadecimal digits.
    BadEscape {
        /// Byte offset of the `%`.
        index: usize,
    },
    /// A byte that must have been escaped appears raw (a space, a control byte
    /// or anything outside ASCII).
    Unescaped {
        /// Byte offset of the offending byte.
        index: usize,
    },
    /// The decoded bytes are not UTF-8 and text was required.
    NotUtf8,
}

impl fmt::Display for PctError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadEscape { index } => write!(f, "malformed %-escape at byte {index}"),
            Self::Unescaped { index } => write!(f, "unescaped byte at {index}"),
            Self::NotUtf8 => f.write_str("decoded field is not UTF-8"),
        }
    }
}

impl std::error::Error for PctError {}

const fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Encode arbitrary bytes.
#[must_use]
pub fn encode_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if is_unreserved(b) {
            out.push(char::from(b));
        } else {
            out.push('%');
            out.push(char::from(
                HEX.get(usize::from(b >> 4)).copied().unwrap_or(b'0'),
            ));
            out.push(char::from(
                HEX.get(usize::from(b & 0x0F)).copied().unwrap_or(b'0'),
            ));
        }
    }
    out
}

/// Encode text.
#[must_use]
pub fn encode(text: &str) -> String {
    encode_bytes(text.as_bytes())
}

/// Encode a protocol field: empty becomes `-`, and a literal `-` becomes `%2D`.
#[must_use]
pub fn encode_field(text: &str) -> String {
    match text {
        "" => "-".to_owned(),
        "-" => "%2D".to_owned(),
        other => encode(other),
    }
}

const fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(b.wrapping_sub(b'a').wrapping_add(10)),
        b'A'..=b'F' => Some(b.wrapping_sub(b'A').wrapping_add(10)),
        _ => None,
    }
}

/// Decode to bytes.
///
/// # Errors
/// [`PctError::BadEscape`] for a truncated or non-hex escape, and
/// [`PctError::Unescaped`] for a raw byte that the encoder would have escaped.
pub fn decode_bytes(field: &str) -> Result<Vec<u8>, PctError> {
    let raw = field.as_bytes();
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0usize;
    while let Some(&b) = raw.get(i) {
        if b == b'%' {
            let hi = raw.get(i.saturating_add(1)).copied().and_then(hex_value);
            let lo = raw.get(i.saturating_add(2)).copied().and_then(hex_value);
            match (hi, lo) {
                (Some(h), Some(l)) => out.push((h << 4) | l),
                _ => return Err(PctError::BadEscape { index: i }),
            }
            i = i.saturating_add(3);
        } else if is_unreserved(b) {
            out.push(b);
            i = i.saturating_add(1);
        } else {
            return Err(PctError::Unescaped { index: i });
        }
    }
    Ok(out)
}

/// Decode to text.
///
/// # Errors
/// As [`decode_bytes`], plus [`PctError::NotUtf8`].
pub fn decode(field: &str) -> Result<String, PctError> {
    String::from_utf8(decode_bytes(field)?).map_err(|_| PctError::NotUtf8)
}

/// Decode a protocol field written by [`encode_field`]: `-` is empty.
///
/// # Errors
/// As [`decode`].
pub fn decode_field(field: &str) -> Result<String, PctError> {
    if field == "-" {
        Ok(String::new())
    } else {
        decode(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_byte_round_trips() {
        let all: Vec<u8> = (0u8..=255).collect();
        let enc = encode_bytes(&all);
        assert!(
            enc.bytes().all(|b| is_unreserved(b) || b == b'%'),
            "an encoded field contains a byte that would break a line protocol"
        );
        assert_eq!(decode_bytes(&enc).expect("decodes"), all);
    }

    #[test]
    fn spaces_newlines_and_percent_are_escaped() {
        assert_eq!(encode("a b\nc%"), "a%20b%0Ac%25");
        assert_eq!(decode("a%20b%0Ac%25").expect("decodes"), "a b\nc%");
    }

    #[test]
    fn thai_text_round_trips_through_utf8_escapes() {
        let s = "ไฟล์ใหม่ (2).txt";
        let enc = encode(s);
        assert!(enc.is_ascii());
        assert_eq!(decode(&enc).expect("decodes"), s);
    }

    #[test]
    fn empty_and_a_literal_dash_are_distinct_fields() {
        assert_eq!(encode_field(""), "-");
        assert_eq!(encode_field("-"), "%2D");
        assert_eq!(decode_field("-").expect("empty"), "");
        assert_eq!(decode_field("%2D").expect("dash"), "-");
        assert_eq!(decode_field(&encode_field("a-b")).expect("kept"), "a-b");
    }

    #[test]
    fn malformed_escapes_are_refused_not_passed_through() {
        assert_eq!(decode("%"), Err(PctError::BadEscape { index: 0 }));
        assert_eq!(decode("ab%4"), Err(PctError::BadEscape { index: 2 }));
        assert_eq!(decode("%zz"), Err(PctError::BadEscape { index: 0 }));
        assert_eq!(decode("%4G"), Err(PctError::BadEscape { index: 0 }));
    }

    #[test]
    fn raw_bytes_the_encoder_would_escape_are_refused() {
        // A space inside a field means two parsers could split the line
        // differently, so it is an error rather than a tolerated literal.
        assert_eq!(decode("a b"), Err(PctError::Unescaped { index: 1 }));
        assert_eq!(decode("a/b"), Err(PctError::Unescaped { index: 1 }));
        assert_eq!(decode("é"), Err(PctError::Unescaped { index: 0 }));
    }

    #[test]
    fn lowercase_hex_is_accepted_on_input() {
        assert_eq!(decode("%2f%2F").expect("decodes"), "//");
    }

    #[test]
    fn invalid_utf8_is_refused_for_text_but_available_as_bytes() {
        assert_eq!(decode("%FF"), Err(PctError::NotUtf8));
        assert_eq!(decode_bytes("%FF").expect("bytes"), vec![0xFF]);
    }
}
