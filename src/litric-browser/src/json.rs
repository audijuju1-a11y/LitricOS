//! Strict JSON (RFC 8259): a parser and a writer.
//!
//! The Chrome DevTools Protocol speaks JSON, and everything Chromium sends is
//! derived from web content an attacker may control (titles, URLs, dialog
//! messages). This parser is therefore strict rather than forgiving:
//!
//! * the input must be UTF-8, with no byte-order mark;
//! * exactly one value, surrounded only by the four JSON whitespace bytes;
//! * no trailing commas, comments, single quotes, `NaN`, `Infinity`, leading
//!   zeros or `+` signs;
//! * `\u` escapes must pair surrogates correctly; a lone surrogate is an error;
//! * raw control characters (below U+0020) inside strings are an error;
//! * a number whose value does not fit a finite `f64` is an error;
//! * an object with two members of the same name is an error, because two
//!   parsers disagreeing about which duplicate wins is a known way to smuggle a
//!   value past a check;
//! * arrays and objects nest at most [`MAX_DEPTH`] levels.
//!
//! Strings are copied in runs between escapes, so a one-megabyte string (a
//! screencast frame is a base64 string of that order) costs one scan and one
//! copy, not a copy per character.

use core::fmt;
use std::collections::HashSet;

/// The deepest nesting of arrays and objects accepted.
pub const MAX_DEPTH: usize = 128;

/// A JSON value.
///
/// Objects keep their members in document order. Duplicate names are refused
/// by the parser, so [`Value::get`] is unambiguous for parsed values.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number. Always finite.
    Number(f64),
    /// A string.
    String(String),
    /// An array.
    Array(Vec<Value>),
    /// An object, members in document order.
    Object(Vec<(String, Value)>),
}

/// Why a document was refused. Every variant carries the byte offset at which
/// the problem was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JsonError {
    /// The input is not UTF-8.
    NotUtf8 {
        /// Offset of the first invalid byte.
        offset: usize,
    },
    /// The input ended inside a value.
    UnexpectedEnd {
        /// Offset of the end.
        offset: usize,
    },
    /// A byte that cannot start or continue a value here.
    Unexpected {
        /// Offset of the byte.
        offset: usize,
    },
    /// A number that breaks the grammar or does not fit a finite `f64`.
    BadNumber {
        /// Offset where the number starts.
        offset: usize,
    },
    /// A malformed escape sequence or a lone surrogate.
    BadEscape {
        /// Offset of the backslash.
        offset: usize,
    },
    /// A raw control character inside a string.
    ControlCharacter {
        /// Offset of the character.
        offset: usize,
    },
    /// Nesting deeper than [`MAX_DEPTH`].
    TooDeep {
        /// Offset of the bracket that went too deep.
        offset: usize,
    },
    /// An object member name that already appeared in the same object.
    DuplicateKey {
        /// Offset of the object's opening brace.
        offset: usize,
    },
    /// Something other than whitespace after the value.
    TrailingData {
        /// Offset of the first extra byte.
        offset: usize,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8 { offset } => write!(f, "invalid UTF-8 at byte {offset}"),
            Self::UnexpectedEnd { offset } => write!(f, "unexpected end at byte {offset}"),
            Self::Unexpected { offset } => write!(f, "unexpected byte at {offset}"),
            Self::BadNumber { offset } => write!(f, "malformed number at byte {offset}"),
            Self::BadEscape { offset } => write!(f, "malformed escape at byte {offset}"),
            Self::ControlCharacter { offset } => {
                write!(f, "unescaped control character at byte {offset}")
            }
            Self::TooDeep { offset } => {
                write!(f, "nesting deeper than {MAX_DEPTH} at byte {offset}")
            }
            Self::DuplicateKey { offset } => {
                write!(f, "duplicate member name in the object at byte {offset}")
            }
            Self::TrailingData { offset } => write!(f, "data after the value at byte {offset}"),
        }
    }
}

impl std::error::Error for JsonError {}

/// Parse exactly one JSON document.
///
/// # Errors
/// [`JsonError`] naming the first problem found.
pub fn parse(input: &[u8]) -> Result<Value, JsonError> {
    let text = core::str::from_utf8(input).map_err(|e| JsonError::NotUtf8 {
        offset: e.valid_up_to(),
    })?;
    let mut p = Parser {
        src: text.as_bytes(),
        text,
        pos: 0,
    };
    p.skip_ws();
    let value = p.value(0)?;
    p.skip_ws();
    if p.pos < p.src.len() {
        return Err(JsonError::TrailingData { offset: p.pos });
    }
    Ok(value)
}

struct Parser<'a> {
    src: &'a [u8],
    text: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) {
        self.pos = self.pos.saturating_add(1);
    }

    fn skip_ws(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.peek() {
            self.bump();
        }
    }

    fn end_or_unexpected(&self) -> JsonError {
        if self.pos >= self.src.len() {
            JsonError::UnexpectedEnd { offset: self.pos }
        } else {
            JsonError::Unexpected { offset: self.pos }
        }
    }

    fn literal(&mut self, word: &[u8], value: Value) -> Result<Value, JsonError> {
        let end = self.pos.saturating_add(word.len());
        match self.src.get(self.pos..end) {
            Some(s) if s == word => {
                self.pos = end;
                Ok(value)
            }
            _ => {
                // Report the first byte that differs, or the end.
                let mut i = self.pos;
                for &w in word {
                    match self.src.get(i) {
                        Some(&b) if b == w => i = i.saturating_add(1),
                        Some(_) => return Err(JsonError::Unexpected { offset: i }),
                        None => return Err(JsonError::UnexpectedEnd { offset: i }),
                    }
                }
                Err(JsonError::Unexpected { offset: self.pos })
            }
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, JsonError> {
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string().map(Value::String),
            Some(b't') => self.literal(b"true", Value::Bool(true)),
            Some(b'f') => self.literal(b"false", Value::Bool(false)),
            Some(b'n') => self.literal(b"null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.end_or_unexpected()),
        }
    }

    fn array(&mut self, depth: usize) -> Result<Value, JsonError> {
        let depth = depth.saturating_add(1);
        if depth > MAX_DEPTH {
            return Err(JsonError::TooDeep { offset: self.pos });
        }
        self.bump(); // [
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.bump();
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value(depth)?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b']') => {
                    self.bump();
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.end_or_unexpected()),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, JsonError> {
        let depth = depth.saturating_add(1);
        let start = self.pos;
        if depth > MAX_DEPTH {
            return Err(JsonError::TooDeep { offset: self.pos });
        }
        self.bump(); // {
        let mut members: Vec<(String, Value)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.bump();
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.end_or_unexpected());
            }
            let key = self.string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.end_or_unexpected());
            }
            self.bump();
            self.skip_ws();
            let value = self.value(depth)?;
            members.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b'}') => {
                    self.bump();
                    break;
                }
                _ => return Err(self.end_or_unexpected()),
            }
        }
        if has_duplicate_keys(&members) {
            return Err(JsonError::DuplicateKey { offset: start });
        }
        Ok(Value::Object(members))
    }

    fn number(&mut self) -> Result<Value, JsonError> {
        let start = self.pos;
        let bad = JsonError::BadNumber { offset: start };
        if self.peek() == Some(b'-') {
            self.bump();
        }
        match self.peek() {
            Some(b'0') => {
                self.bump();
                if let Some(b'0'..=b'9') = self.peek() {
                    return Err(bad); // leading zero
                }
            }
            Some(b'1'..=b'9') => self.digits(),
            _ => return Err(bad),
        }
        if self.peek() == Some(b'.') {
            self.bump();
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(bad);
            }
            self.digits();
        }
        if let Some(b'e' | b'E') = self.peek() {
            self.bump();
            if let Some(b'+' | b'-') = self.peek() {
                self.bump();
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(bad);
            }
            self.digits();
        }
        let slice = self.text.get(start..self.pos).ok_or(bad)?;
        // The grammar above is a subset of what `f64::from_str` accepts, and
        // that conversion rounds correctly.
        let n: f64 = slice.parse().map_err(|_| bad)?;
        if !n.is_finite() {
            return Err(bad);
        }
        Ok(Value::Number(n))
    }

    fn digits(&mut self) {
        while let Some(b'0'..=b'9') = self.peek() {
            self.bump();
        }
    }

    fn hex4(&self, at: usize, escape_at: usize) -> Result<u32, JsonError> {
        let end = at.saturating_add(4);
        let digits = self
            .src
            .get(at..end)
            .ok_or(JsonError::BadEscape { offset: escape_at })?;
        let mut v = 0u32;
        for &d in digits {
            let n = match d {
                b'0'..=b'9' => d.wrapping_sub(b'0'),
                b'a'..=b'f' => d.wrapping_sub(b'a').wrapping_add(10),
                b'A'..=b'F' => d.wrapping_sub(b'A').wrapping_add(10),
                _ => return Err(JsonError::BadEscape { offset: escape_at }),
            };
            v = (v << 4) | u32::from(n);
        }
        Ok(v)
    }

    /// Parse a string starting at the opening quote.
    fn string(&mut self) -> Result<String, JsonError> {
        self.bump(); // opening quote
        let mut out = String::new();
        let mut run_start = self.pos;
        loop {
            let Some(b) = self.peek() else {
                return Err(JsonError::UnexpectedEnd { offset: self.pos });
            };
            match b {
                b'"' => {
                    self.flush_run(&mut out, run_start)?;
                    self.bump();
                    return Ok(out);
                }
                b'\\' => {
                    self.flush_run(&mut out, run_start)?;
                    self.escape(&mut out)?;
                    run_start = self.pos;
                }
                0x00..=0x1F => return Err(JsonError::ControlCharacter { offset: self.pos }),
                // Any other byte, including UTF-8 continuation bytes, is part
                // of the run. The whole input was validated as UTF-8 and runs
                // only ever end at ASCII bytes, so every run is valid text.
                _ => self.bump(),
            }
        }
    }

    fn flush_run(&self, out: &mut String, run_start: usize) -> Result<(), JsonError> {
        let run = self
            .text
            .get(run_start..self.pos)
            .ok_or(JsonError::Unexpected { offset: run_start })?;
        out.push_str(run);
        Ok(())
    }

    fn escape(&mut self, out: &mut String) -> Result<(), JsonError> {
        let at = self.pos;
        let bad = JsonError::BadEscape { offset: at };
        let code = self.src.get(at.saturating_add(1)).copied().ok_or(bad)?;
        let simple = match code {
            b'"' => Some('"'),
            b'\\' => Some('\\'),
            b'/' => Some('/'),
            b'b' => Some('\u{08}'),
            b'f' => Some('\u{0C}'),
            b'n' => Some('\n'),
            b'r' => Some('\r'),
            b't' => Some('\t'),
            b'u' => None,
            _ => return Err(bad),
        };
        if let Some(c) = simple {
            out.push(c);
            self.pos = at.saturating_add(2);
            return Ok(());
        }
        let first = self.hex4(at.saturating_add(2), at)?;
        let after_first = at.saturating_add(6);
        let scalar = match first {
            0xD800..=0xDBFF => {
                // A high surrogate must be followed immediately by an escaped
                // low surrogate.
                if self.src.get(after_first..after_first.saturating_add(2)) != Some(b"\\u") {
                    return Err(bad);
                }
                let second = self.hex4(after_first.saturating_add(2), at)?;
                if !(0xDC00..=0xDFFF).contains(&second) {
                    return Err(bad);
                }
                self.pos = after_first.saturating_add(6);
                let high = first.wrapping_sub(0xD800);
                let low = second.wrapping_sub(0xDC00);
                0x1_0000u32.wrapping_add((high << 10) | low)
            }
            0xDC00..=0xDFFF => return Err(bad),
            _ => {
                self.pos = after_first;
                first
            }
        };
        let c = char::from_u32(scalar).ok_or(bad)?;
        out.push(c);
        Ok(())
    }
}

fn has_duplicate_keys(members: &[(String, Value)]) -> bool {
    // Small objects are the overwhelming majority in CDP traffic; a pairwise
    // scan is cheaper than hashing for them and a set keeps large ones linear.
    const PAIRWISE_LIMIT: usize = 16;
    if members.len() <= PAIRWISE_LIMIT {
        for (i, (k, _)) in members.iter().enumerate() {
            if members
                .iter()
                .skip(i.saturating_add(1))
                .any(|(other, _)| other == k)
            {
                return true;
            }
        }
        return false;
    }
    let mut seen: HashSet<&str> = HashSet::with_capacity(members.len());
    members.iter().any(|(k, _)| !seen.insert(k.as_str()))
}

impl Value {
    /// An object from `(name, value)` pairs, in order.
    ///
    /// Names are not checked for duplicates; callers building a message pass
    /// distinct names.
    #[must_use]
    pub fn object<const N: usize>(members: [(&str, Value); N]) -> Self {
        Self::Object(
            members
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
        )
    }

    /// The member named `key`, when this is an object that has one.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Object(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The text, when this is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// The number, when this is one.
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// The number as a non-negative integer, when it is exactly one and at
    /// most 2^53 (the largest range in which every integer is representable).
    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        const LIMIT: f64 = 9_007_199_254_740_992.0; // 2^53
        let n = self.as_f64()?;
        if (0.0..=LIMIT).contains(&n) && n.fract() == 0.0 {
            // In range and integral, so the conversion is exact.
            Some(n as u64)
        } else {
            None
        }
    }

    /// The number as an integer, when it is exactly one within ±2^53.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        const LIMIT: f64 = 9_007_199_254_740_992.0; // 2^53
        let n = self.as_f64()?;
        if (-LIMIT..=LIMIT).contains(&n) && n.fract() == 0.0 {
            // In range and integral, so the conversion is exact.
            Some(n as i64)
        } else {
            None
        }
    }

    /// The flag, when this is a boolean.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The items, when this is an array.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Serialise compactly (no whitespace).
    ///
    /// A number that is not finite cannot be represented in JSON and is
    /// written as `null`; the parser never produces one.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write_to(&mut out);
        out
    }

    /// Append the compact serialisation to `out`.
    pub fn write_to(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(true) => out.push_str("true"),
            Self::Bool(false) => out.push_str("false"),
            Self::Number(n) => write_number(*n, out),
            Self::String(s) => write_string(s, out),
            Self::Array(items) => {
                out.push('[');
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write_to(out);
                }
                out.push(']');
            }
            Self::Object(members) => {
                out.push('{');
                for (i, (k, v)) in members.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(k, out);
                    out.push(':');
                    v.write_to(out);
                }
                out.push('}');
            }
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

fn write_number(n: f64, out: &mut String) {
    use core::fmt::Write as _;
    if n.is_finite() {
        // `Display` for f64 prints the shortest digits that round-trip and
        // never uses exponent notation or a trailing `.`, so the result is
        // always inside the JSON number grammar.
        let _ = write!(out, "{n}");
    } else {
        out.push_str("null");
    }
}

/// Append `s` as a JSON string literal.
pub fn write_string(s: &str, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.reserve(s.len().saturating_add(2));
    out.push('"');
    let mut run_start = 0usize;
    for (i, b) in s.bytes().enumerate() {
        let escaped: Option<&str> = match b {
            b'"' => Some("\\\""),
            b'\\' => Some("\\\\"),
            b'\n' => Some("\\n"),
            b'\r' => Some("\\r"),
            b'\t' => Some("\\t"),
            0x08 => Some("\\b"),
            0x0C => Some("\\f"),
            0x00..=0x1F => Some(""),
            _ => None,
        };
        let Some(esc) = escaped else { continue };
        out.push_str(s.get(run_start..i).unwrap_or(""));
        if esc.is_empty() {
            out.push_str("\\u00");
            out.push(char::from(
                HEX.get(usize::from(b >> 4)).copied().unwrap_or(b'0'),
            ));
            out.push(char::from(
                HEX.get(usize::from(b & 0x0F)).copied().unwrap_or(b'0'),
            ));
        } else {
            out.push_str(esc);
        }
        run_start = i.saturating_add(1);
    }
    out.push_str(s.get(run_start..).unwrap_or(""));
    out.push('"');
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
mod tests {
    use super::*;

    fn ok(s: &str) -> Value {
        parse(s.as_bytes()).unwrap_or_else(|e| panic!("{s:?} failed: {e}"))
    }

    fn err(s: &[u8]) -> JsonError {
        match parse(s) {
            Ok(v) => panic!("{:?} was accepted as {v:?}", String::from_utf8_lossy(s)),
            Err(e) => e,
        }
    }

    #[test]
    fn scalars_parse() {
        assert_eq!(ok("null"), Value::Null);
        assert_eq!(ok("true"), Value::Bool(true));
        assert_eq!(ok(" false "), Value::Bool(false));
        assert_eq!(ok("0"), Value::Number(0.0));
        assert_eq!(ok("-0"), Value::Number(-0.0));
        assert_eq!(ok("12.5e2"), Value::Number(1250.0));
        assert_eq!(ok("1E-2"), Value::Number(0.01));
        assert_eq!(ok("-1.0e+3"), Value::Number(-1000.0));
        assert_eq!(ok("\"hi\""), Value::String("hi".to_owned()));
        assert_eq!(ok("\t\r\n[]\n"), Value::Array(vec![]));
    }

    #[test]
    fn nested_structures_parse_in_order() {
        let v = ok(r#"{"id":7,"result":{"a":[1,2,{"b":null}],"s":"x"},"z":true}"#);
        assert_eq!(v.get("id").and_then(Value::as_u64), Some(7));
        let result = v.get("result").expect("result");
        let a = result.get("a").and_then(Value::as_array).expect("a");
        assert_eq!(a.len(), 3);
        assert_eq!(a[2].get("b"), Some(&Value::Null));
        let Value::Object(members) = &v else {
            panic!("object")
        };
        let names: Vec<&str> = members.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["id", "result", "z"]);
    }

    #[test]
    fn escapes_decode_including_surrogate_pairs() {
        assert_eq!(
            ok(r#""a\"b\\c\/d\b\f\n\r\t""#),
            Value::String("a\"b\\c/d\u{8}\u{c}\n\r\t".to_owned())
        );
        assert_eq!(
            ok(r#""\u0041\u00e9\u0E44""#),
            Value::String("Aéไ".to_owned())
        );
        assert_eq!(
            ok(r#""\uD83D\uDE00""#),
            Value::String("\u{1F600}".to_owned())
        );
        assert_eq!(
            ok(r#""\ud83d\ude00x""#),
            Value::String("\u{1F600}x".to_owned())
        );
        assert_eq!(ok("\"ไทย 😀\""), Value::String("ไทย 😀".to_owned()));
    }

    #[test]
    fn invalid_documents_are_refused_with_the_right_error() {
        let cases: &[(&[u8], JsonError)] = &[
            (b"", JsonError::UnexpectedEnd { offset: 0 }),
            (b"   ", JsonError::UnexpectedEnd { offset: 3 }),
            (b"nul", JsonError::UnexpectedEnd { offset: 3 }),
            (b"nulL", JsonError::Unexpected { offset: 3 }),
            (b"True", JsonError::Unexpected { offset: 0 }),
            (b"[1,]", JsonError::Unexpected { offset: 3 }),
            (b"[1 2]", JsonError::Unexpected { offset: 3 }),
            (b"{\"a\":1,}", JsonError::Unexpected { offset: 7 }),
            (b"{'a':1}", JsonError::Unexpected { offset: 1 }),
            (b"{\"a\" 1}", JsonError::Unexpected { offset: 5 }),
            (b"{\"a\":1", JsonError::UnexpectedEnd { offset: 6 }),
            (b"[", JsonError::UnexpectedEnd { offset: 1 }),
            (b"01", JsonError::BadNumber { offset: 0 }),
            (b"-", JsonError::BadNumber { offset: 0 }),
            (b"+1", JsonError::Unexpected { offset: 0 }),
            (b"1.", JsonError::BadNumber { offset: 0 }),
            (b".5", JsonError::Unexpected { offset: 0 }),
            (b"1e", JsonError::BadNumber { offset: 0 }),
            (b"1e+", JsonError::BadNumber { offset: 0 }),
            (b"1e400", JsonError::BadNumber { offset: 0 }),
            (b"-1e400", JsonError::BadNumber { offset: 0 }),
            (b"NaN", JsonError::Unexpected { offset: 0 }),
            (b"Infinity", JsonError::Unexpected { offset: 0 }),
            (b"\"abc", JsonError::UnexpectedEnd { offset: 4 }),
            (b"\"a\x01b\"", JsonError::ControlCharacter { offset: 2 }),
            (b"\"a\nb\"", JsonError::ControlCharacter { offset: 2 }),
            (b"\"\\x\"", JsonError::BadEscape { offset: 1 }),
            (b"\"\\u12\"", JsonError::BadEscape { offset: 1 }),
            (b"\"\\u12G4\"", JsonError::BadEscape { offset: 1 }),
            (b"\"\\uD800\"", JsonError::BadEscape { offset: 1 }),
            (b"\"\\uDC00\"", JsonError::BadEscape { offset: 1 }),
            (b"\"\\uD800\\u0041\"", JsonError::BadEscape { offset: 1 }),
            (b"\"\\uD800x\"", JsonError::BadEscape { offset: 1 }),
            (b"1 2", JsonError::TrailingData { offset: 2 }),
            (b"{} x", JsonError::TrailingData { offset: 3 }),
            (b"null\0", JsonError::TrailingData { offset: 4 }),
            (b"\xEF\xBB\xBF{}", JsonError::Unexpected { offset: 0 }),
            (b"\"\xFF\"", JsonError::NotUtf8 { offset: 1 }),
            (b"\"\xC3\"", JsonError::NotUtf8 { offset: 1 }),
            (b"{\"a\":1,\"a\":2}", JsonError::DuplicateKey { offset: 0 }),
            (b"[1]/*c*/", JsonError::TrailingData { offset: 3 }),
        ];
        for (input, expected) in cases {
            assert_eq!(
                err(input),
                *expected,
                "input {:?}",
                String::from_utf8_lossy(input)
            );
        }
    }

    #[test]
    fn duplicate_keys_are_found_in_large_objects_too() {
        let mut doc = String::from("{");
        for i in 0..100 {
            doc.push_str(&format!("\"k{i}\":{i},"));
        }
        doc.push_str("\"k57\":0}");
        assert_eq!(err(doc.as_bytes()), JsonError::DuplicateKey { offset: 0 });
        let distinct = doc.replace("\"k57\":0}", "\"k100\":0}");
        assert!(parse(distinct.as_bytes()).is_ok());
    }

    #[test]
    fn nesting_is_limited_to_the_documented_depth() {
        let deep_ok = format!("{}{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
        assert!(parse(deep_ok.as_bytes()).is_ok());
        let too_deep = format!("{}{}", "[".repeat(MAX_DEPTH + 1), "]".repeat(MAX_DEPTH + 1));
        assert_eq!(
            err(too_deep.as_bytes()),
            JsonError::TooDeep { offset: MAX_DEPTH }
        );
        let objects = format!(
            "{}1{}",
            "{\"a\":".repeat(MAX_DEPTH + 1),
            "}".repeat(MAX_DEPTH + 1)
        );
        assert!(matches!(err(objects.as_bytes()), JsonError::TooDeep { .. }));
        // A hostile document far deeper than the limit must fail cleanly
        // rather than exhaust the stack.
        let hostile = "[".repeat(1_000_000);
        assert!(matches!(err(hostile.as_bytes()), JsonError::TooDeep { .. }));
    }

    #[test]
    fn a_megabyte_string_parses_and_serialises_in_linear_time() {
        let body = "QUJD".repeat(256 * 1024); // 1 MiB of base64-like text
        let doc = format!("{{\"data\":\"{body}\",\"n\":1}}");
        let started = std::time::Instant::now();
        for _ in 0..5 {
            let v = parse(doc.as_bytes()).expect("parses");
            assert_eq!(
                v.get("data").and_then(Value::as_str).map(str::len),
                Some(body.len())
            );
        }
        let escaped = "a\\nb".repeat(256 * 1024);
        let v = parse(format!("\"{escaped}\"").as_bytes()).expect("escaped");
        assert_eq!(v.as_str().map(str::len), Some(3 * 256 * 1024));
        let json = v.to_json();
        assert_eq!(json.len(), escaped.len() + 2);
        // Generous bound: a quadratic implementation takes minutes here, even
        // unoptimised; the linear one takes milliseconds.
        assert!(
            started.elapsed() < std::time::Duration::from_secs(20),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn the_writer_escapes_exactly_what_must_be_escaped() {
        let s = "q\"b\\s/\u{0}\u{1f}\u{7f}\n\r\t\u{8}\u{c}ไ😀";
        let json = Value::from(s).to_json();
        assert_eq!(
            json,
            "\"q\\\"b\\\\s/\\u0000\\u001f\u{7f}\\n\\r\\t\\b\\fไ😀\""
        );
        assert_eq!(ok(&json), Value::from(s));
    }

    #[test]
    fn numbers_are_written_inside_the_json_grammar() {
        for n in [
            0.0,
            -0.0,
            1.0,
            -1.5,
            0.1,
            1e-7,
            1e21,
            123_456_789.0,
            f64::MAX,
            f64::MIN_POSITIVE,
        ] {
            let json = Value::Number(n).to_json();
            assert_eq!(ok(&json), Value::Number(n), "{json}");
        }
        assert_eq!(Value::Number(f64::NAN).to_json(), "null");
        assert_eq!(Value::Number(f64::INFINITY).to_json(), "null");
        assert_eq!(Value::from(42u32).to_json(), "42");
    }

    #[test]
    fn integer_accessors_are_exact_or_absent() {
        assert_eq!(ok("9007199254740992").as_u64(), Some(9_007_199_254_740_992));
        assert_eq!(ok("9007199254740994").as_u64(), None);
        assert_eq!(ok("1.5").as_u64(), None);
        assert_eq!(ok("-1").as_u64(), None);
        assert_eq!(ok("-1").as_i64(), Some(-1));
        assert_eq!(ok("\"1\"").as_i64(), None);
    }

    #[test]
    fn a_document_round_trips_through_the_writer() {
        let src = r#"{"id":12,"method":"Page.navigate","params":{"url":"https://example.com/?q=a b","flags":[true,false,null,-2.25e-3]},"sessionId":"ABC"}"#;
        let v = ok(src);
        assert_eq!(ok(&v.to_json()), v);
        let built = Value::object([
            ("id", Value::from(12u32)),
            ("method", Value::from("Page.navigate")),
        ]);
        assert_eq!(built.to_json(), r#"{"id":12,"method":"Page.navigate"}"#);
    }

    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn below(&mut self, n: usize) -> usize {
            (self.next() % (n as u64)) as usize
        }
    }

    #[test]
    fn randomized_mutations_never_panic_and_are_deterministic() {
        let corpus: &[&[u8]] = &[
            br#"{"id":1,"result":{"frameTree":{"frame":{"id":"A","url":"about:blank"}}}}"#,
            br#"{"method":"Page.screencastFrame","params":{"data":"/9j/4AAQ","metadata":{"deviceWidth":800,"deviceHeight":600.5},"sessionId":3},"sessionId":"S"}"#,
            br#"[1,-2.5e10,"\u00e9\uD83D\uDE00",true,false,null,[],{}]"#,
            "\"ไทย\\n\\t\"".as_bytes(),
            br#"{"a":{"b":{"c":[[[[1]]]]}}}"#,
        ];
        const SPECIAL: &[u8] = b"{}[]\",:\\/-+.eE0123456789tfnu \t\n\x00\x7f\xc3\xa9\xed\xa0\x80";
        let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
        let mut accepted = 0usize;
        for round in 0..20_000 {
            let base = corpus[rng.below(corpus.len())];
            let mut doc = base.to_vec();
            let edits = 1 + rng.below(4);
            for _ in 0..edits {
                match rng.below(5) {
                    0 if !doc.is_empty() => {
                        let i = rng.below(doc.len());
                        doc[i] ^= 1 << rng.below(8);
                    }
                    1 => {
                        let i = rng.below(doc.len() + 1);
                        doc.insert(i, SPECIAL[rng.below(SPECIAL.len())]);
                    }
                    2 if !doc.is_empty() => {
                        let i = rng.below(doc.len());
                        doc.remove(i);
                    }
                    3 => {
                        let n = rng.below(doc.len() + 1);
                        doc.truncate(n);
                    }
                    _ if !doc.is_empty() => {
                        let i = rng.below(doc.len());
                        doc[i] = SPECIAL[rng.below(SPECIAL.len())];
                    }
                    _ => {}
                }
            }
            let first = parse(&doc);
            let second = parse(&doc);
            assert_eq!(first, second, "round {round}: nondeterministic");
            if let Ok(v) = first {
                accepted += 1;
                // Anything accepted must survive the writer unchanged.
                assert_eq!(
                    parse(v.to_json().as_bytes()).as_ref(),
                    Ok(&v),
                    "round {round}"
                );
            }
        }
        // The mutation mix must exercise both outcomes to mean anything.
        assert!(accepted > 100 && accepted < 19_000, "accepted {accepted}");
    }
}
