//! LFP/1 — the Litric Files Protocol, version 1 (ADR-0019).
//!
//! Request: one UTF-8 line ending in `\n`, at most 16384 bytes, fields
//! separated by exactly one space, free text percent-encoded
//! ([`litric_core::pct`]). Response: `OK <n>` followed by exactly `n`
//! lines, or one line `ERR <kind> <pct-msg>`. Long operations may send any
//! number of `PROGRESS <done> <total> <pct>` lines before the final
//! `OK`/`ERR`. Entry lines are `key=value` pairs joined by single spaces.
//!
//! This module is the pure request/response codec: no socket, no I/O. The
//! server ([`crate::server`]) frames bytes into lines and hands them here;
//! tests feed strings directly.

use crate::{Error, ErrorKind};
use litric_core::pct;

/// A request line may not exceed this many bytes (ADR-0019).
pub const MAX_LINE: usize = 16_384;

/// One parsed request: the verb and its raw (still percent-encoded) fields.
///
/// Decoding is deliberately left to the command layer: percent-encoding
/// applies to *free text* fields, and each command decides which of its
/// fields those are. Paths additionally go through [`crate::path`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The command verb, uppercased (`LIST`, `TRASH`, ...).
    pub verb: String,
    /// The raw fields after the verb, in order.
    pub fields: Vec<String>,
}

/// Why a request line was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Empty line.
    Empty,
    /// The line (without its terminator) exceeded [`MAX_LINE`] bytes.
    TooLong {
        /// The offending length.
        len: usize,
    },
    /// The line contained a control byte other than the terminator.
    ControlByte {
        /// Offset of the byte.
        offset: usize,
    },
    /// The line was not valid UTF-8.
    NotUtf8 {
        /// Offset where decoding failed.
        offset: usize,
    },
    /// The line had no verb, or a verb longer than any command.
    BadVerb {
        /// The offending token.
        token: String,
    },
    /// Two spaces occurred in a row (ADR-0019: fields are separated by
    /// exactly one space).
    DoubleSpace {
        /// Offset of the second space.
        offset: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty request line"),
            Self::TooLong { len } => write!(f, "request is {len} bytes, limit is {MAX_LINE}"),
            Self::ControlByte { offset } => write!(f, "control byte at offset {offset}"),
            Self::NotUtf8 { offset } => write!(f, "invalid UTF-8 at offset {offset}"),
            Self::BadVerb { token } => write!(f, "malformed command verb: {token:?}"),
            Self::DoubleSpace { offset } => {
                write!(
                    f,
                    "fields are separated by exactly one space (offset {offset})"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::new(ErrorKind::BadRequest, e.to_string())
    }
}

/// Parse one request line, terminator already stripped.
///
/// # Errors
/// [`ParseError`] naming the exact rule that failed.
pub fn parse_request(line: &str) -> Result<Request, ParseError> {
    if line.len() > MAX_LINE {
        return Err(ParseError::TooLong { len: line.len() });
    }
    if line.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Some(offset) = line
        .char_indices()
        .find(|(_, c)| c.is_control())
        .map(|(i, _)| i)
    {
        return Err(ParseError::ControlByte { offset });
    }
    let mut fields = line.split(' ');
    let Some(verb) = fields.next() else {
        return Err(ParseError::Empty);
    };
    if verb.is_empty() || !verb.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err(ParseError::BadVerb {
            token: verb.to_string(),
        });
    }
    let mut rest = Vec::new();
    for field in fields {
        if field.is_empty() {
            // An empty field means two consecutive spaces (or a trailing
            // one) — the protocol allows neither.
            let offset = line.len();
            return Err(ParseError::DoubleSpace { offset });
        }
        rest.push(field.to_string());
    }
    Ok(Request {
        verb: verb.to_ascii_uppercase(),
        fields: rest,
    })
}

/// Decode a percent-encoded free-text field.
///
/// # Errors
/// [`Error`] of kind `bad-request` on malformed escapes.
pub fn decode_field(field: &str) -> Result<String, Error> {
    pct::decode_field(field)
        .map_err(|e| Error::new(ErrorKind::BadRequest, format!("percent-decode: {e}")))
}

/// Build the `OK <n>` response head for `n` following lines.
#[must_use]
pub fn ok(count: usize) -> String {
    format!("OK {count}")
}

/// Build one `key=value` entry line from validated pairs. Keys must be
/// plain tokens; values are percent-encoded here.
#[must_use]
pub fn entry(pairs: &[(&str, &str)]) -> String {
    let mut line = String::new();
    for (key, value) in pairs {
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(key);
        line.push('=');
        line.push_str(&pct::encode_field(value));
    }
    line
}

/// Build an `ERR` line.
#[must_use]
pub fn err(kind: ErrorKind, message: &str) -> String {
    format!("ERR {} {}", kind.token(), pct::encode_field(message))
}

/// Build a `PROGRESS` line.
#[must_use]
pub fn progress(done: u64, total: u64, current_pct: u64) -> String {
    format!("PROGRESS {done} {total} {current_pct}")
}

/// A parsed reply, for the shell's client side and for tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// `OK <n>` with the `n` entry lines that followed.
    Ok(Vec<String>),
    /// `ERR <kind> <msg>`.
    Err {
        /// The LFP/1 error kind token.
        kind: String,
        /// The decoded message.
        message: String,
    },
    /// A `PROGRESS` line.
    Progress {
        /// Bytes done.
        done: u64,
        /// Bytes total.
        total: u64,
        /// Percent.
        pct: u64,
    },
}

/// Parse one reply line. `OK` needs its entry lines supplied separately by
/// the caller (they are independent lines); this validates the head.
///
/// # Errors
/// [`Error`] of kind `bad-request` for a malformed reply.
pub fn parse_reply(line: &str) -> Result<Reply, Error> {
    let bad = |m: &str| Error::new(ErrorKind::BadRequest, m);
    let mut fields = line.split(' ');
    match fields.next() {
        Some("OK") => {
            let count = fields
                .next()
                .ok_or_else(|| bad("OK reply missing count"))?
                .parse::<usize>()
                .map_err(|_| bad("OK reply count is not a number"))?;
            if fields.next().is_some() {
                return Err(bad("OK reply has trailing fields"));
            }
            Ok(Reply::Ok(Vec::with_capacity(count.min(10_000))))
        }
        Some("ERR") => {
            let kind = fields.next().ok_or_else(|| bad("ERR reply missing kind"))?;
            let message = fields
                .next()
                .ok_or_else(|| bad("ERR reply missing message"))?;
            let message =
                pct::decode_field(message).map_err(|e| bad(&format!("ERR message decode: {e}")))?;
            Ok(Reply::Err {
                kind: kind.to_string(),
                message,
            })
        }
        Some("PROGRESS") => {
            let done = fields
                .next()
                .ok_or_else(|| bad("PROGRESS missing done"))?
                .parse::<u64>()
                .map_err(|_| bad("PROGRESS done is not a number"))?;
            let total = fields
                .next()
                .ok_or_else(|| bad("PROGRESS missing total"))?
                .parse::<u64>()
                .map_err(|_| bad("PROGRESS total is not a number"))?;
            let p = fields
                .next()
                .ok_or_else(|| bad("PROGRESS missing pct"))?
                .parse::<u64>()
                .map_err(|_| bad("PROGRESS pct is not a number"))?;
            Ok(Reply::Progress {
                done,
                total,
                pct: p,
            })
        }
        _ => Err(bad("unknown reply type")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verbs_and_fields() {
        let r = parse_request("LIST /Documents hidden=1").expect("valid");
        assert_eq!(r.verb, "LIST");
        assert_eq!(r.fields, vec!["/Documents", "hidden=1"]);
        // Case is normalized.
        let r = parse_request("hello 1").expect("valid");
        assert_eq!(r.verb, "HELLO");
    }

    #[test]
    fn rejects_the_documented_shapes() {
        assert_eq!(parse_request(""), Err(ParseError::Empty));
        assert!(matches!(
            parse_request("LIST a  b"),
            Err(ParseError::DoubleSpace { .. })
        ));
        assert!(matches!(
            parse_request("LIST a "),
            Err(ParseError::DoubleSpace { .. })
        ));
        assert!(matches!(
            parse_request("L!ST"),
            Err(ParseError::BadVerb { .. })
        ));
        assert!(matches!(
            parse_request("\u{7}ring"),
            Err(ParseError::ControlByte { offset: 0 })
        ));
        let long = "x".repeat(MAX_LINE + 1);
        assert!(matches!(
            parse_request(&long),
            Err(ParseError::TooLong { .. })
        ));
    }

    #[test]
    fn max_line_is_allowed_exactly() {
        // 4-byte verb + space + MAX_LINE-5 filler is exactly the limit.
        let filler = "x".repeat(MAX_LINE - 5);
        parse_request(&format!("LIST {filler}")).expect("exactly at the limit is fine");
    }

    #[test]
    fn entry_lines_encode_and_are_parseable_back() {
        let e = entry(&[("name", "my file.txt"), ("kind", "file"), ("size", "123")]);
        assert_eq!(e, "name=my%20file.txt kind=file size=123");
        // Every value decodes back.
        for pair in e.split(' ') {
            let (k, v) = pair.split_once('=').expect("pair");
            assert_eq!(
                pct::decode_field(v).expect("valid"),
                if k == "name" { "my file.txt" } else { v }
            );
        }
    }

    #[test]
    fn replies_round_trip() {
        assert_eq!(ok(2), "OK 2");
        let e = err(ErrorKind::NotFound, "no such file");
        assert_eq!(e, "ERR not-found no%20such%20file");
        match parse_reply(&e).expect("valid") {
            Reply::Err { kind, message } => {
                assert_eq!(kind, "not-found");
                assert_eq!(message, "no such file");
            }
            other => panic!("wrong reply: {other:?}"),
        }
        let p = progress(100, 400, 25);
        assert_eq!(p, "PROGRESS 100 400 25");
        assert!(matches!(
            parse_reply(&p).expect("valid"),
            Reply::Progress {
                done: 100,
                total: 400,
                pct: 25
            }
        ));
        match parse_reply("OK 0").expect("valid") {
            Reply::Ok(lines) => assert!(lines.is_empty()),
            other => panic!("wrong reply: {other:?}"),
        }
        assert!(parse_reply("BOOM").is_err());
        assert!(parse_reply("OK many").is_err());
    }
}
